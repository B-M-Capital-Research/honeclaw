//! `NotificationRouter::dispatch` —— 主分发管线。
//!
//! 输入一个 `MarketEvent`,做 5 步:
//! 1. **黑名单**:`disabled_kinds` 命中直接 (0,0);
//! 2. **预升级**:多信号合流(`maybe_upgrade_news`)+ 系统级策略
//!    (`apply_system_event_policy`);
//! 3. **解析订阅**:从 `SharedRegistry` 拿到匹配的 (actor, sev) 列表,空则 (0,0);
//! 4. **per-actor 过滤** loop:LLM 仲裁 / per_actor severity override / quiet
//!    mode / close-alert demote / prefs filter / high_daily_cap / price band
//!    advance 规则 / same-symbol cooldown;
//! 5. **路由**:High → polish + sink.send + delivery_log;
//!    Medium/Low → digest.enqueue + delivery_log。
//!
//! 实现尽量保持「线性、扁平」,把每个降级原因记成不同的 `demoted_by_*` flag
//! 而不是嵌套 if-else,这样 grep `delivery_log.status` 能直接对账。

use tracing::info;

use crate::digest::time_window::EffectiveTz;
use crate::earnings_continuity::continuity_review_stage;
use crate::earnings_document::{
    earnings_research_material_kind, earnings_research_object_key_for_event,
    is_earnings_release_document_event,
};
use crate::event::{EventKind, MarketEvent, Severity};
use crate::prefs::{NotificationPrefs, kind_tag};
use crate::renderer::{self, RenderFormat};

use super::config::NotificationRouter;
use super::policy::{
    event_category, is_intraday_price_band_alert, is_price_close_alert, local_day_start,
    price_alert_symbol_direction,
};
use super::sink::{actor_key, body_preview};

impl NotificationRouter {
    /// 对一个事件执行分发。High 立即推；其余当前只记 pending-digest 日志。
    ///
    /// 返回 `(immediate_sent, pending_digest)` 数量。
    pub async fn dispatch(&self, event: &MarketEvent) -> anyhow::Result<(u32, u32)> {
        // 全局 kind 黑名单：部署方 YAML 里关掉的 kind 直接短路,不走 resolve/prefs/cap。
        // 事件已经由调用方负责入库,这里只是不分发。
        let tag = kind_tag(&event.kind);
        if self.disabled_kinds.contains(tag) {
            tracing::info!(
                event_id = %event.id,
                kind = %tag,
                "event kind globally disabled; dispatch skipped"
            );
            return Ok((0, 0));
        }
        let upgraded = self.maybe_upgrade_news(event);
        let routed = self.apply_system_event_policy(&upgraded);
        // 评级事件注入「近 30 日共识计数」锚点(actor 无关,dispatch 前一次算好;
        // 事件本身已入库,计数含当前这条)。查询失败不阻断分发。
        let routed = self.annotate_analyst_consensus(routed);
        let event = &routed;
        // 每次 dispatch 都拿最新快照——用户持仓更新后下一条事件即可感知。
        let hits = self.registry.load().resolve(event);
        if hits.is_empty() {
            let _ = self.store.log_delivery(
                &event.id,
                "event_engine::::no_actor",
                "router",
                event.severity,
                "no_actor",
                None,
            );
            info!(
                event_id = %event.id,
                kind = %kind_tag(&event.kind),
                source = %event.source,
                symbols = ?event.symbols,
                "dispatch skipped: no matching actor"
            );
            return Ok((0, 0));
        }
        let mut sent = 0u32;
        let mut pending = 0u32;
        for (actor, sev) in hits {
            let user_prefs = self.prefs.load(&actor);
            let price_policy = user_prefs.effective_price_alert_policy(self.price_policy_defaults);
            // LLM 仲裁:不确定来源的 Low NewsCritical,按 actor 重要性 prompt
            // 决定是否升 Medium。结果只影响本 actor 的本次分发,不污染原 event。
            let actor_upgraded_event;
            let (event, sev) = match self.maybe_llm_upgrade_for_actor(event, &user_prefs).await {
                Some(upgraded) => {
                    actor_upgraded_event = upgraded;
                    (&actor_upgraded_event, Severity::Medium)
                }
                None => (event, sev),
            };
            // 仓位上下文注入:actor 持有事件标的时,给 actor 级克隆写入
            // 美元影响 / 距成本 / portfolio_weight_pct(供下方 price policy 的
            // 大仓位判断与 renderer 的持仓行使用)。原始事件与 store 不动。
            let actor_position_event;
            let event = match self.position_annotated_event_for(event, &actor) {
                Some(annotated) => {
                    actor_position_event = annotated;
                    &actor_position_event
                }
                None => event,
            };
            // per-actor severity policy:用户可自定义
            //   (a) 首次价格阈值:达到时升 High,未达到时即使系统候选为 High 也降级;
            //   (b) immediate_kinds:某些 kind 无条件升 High 即时推(例如 52 周高/低、
            //       分析师评级；price_alert 仍不能绕过显式价格阈值)。
            // 升级后仍要走 high_daily_cap / cooldown,保持 burst 防护。
            let mut sev =
                self.apply_per_actor_severity_policy(event, sev, &user_prefs, &price_policy);
            sev = self.apply_quiet_mode(event, sev, &user_prefs);
            if is_price_close_alert(event)
                && !price_policy.close_direct_enabled
                && matches!(sev, Severity::High)
            {
                tracing::info!(
                    actor = %actor_key(&actor),
                    event_id = %event.id,
                    source = %event.source,
                    "price_close high demoted to digest because price_close_direct_enabled=false"
                );
                sev = Severity::Medium;
            }
            if !user_prefs.should_deliver_with_severity(event, sev) {
                let _ = self.store.log_delivery(
                    &event.id,
                    &actor_key(&actor),
                    "prefs",
                    sev,
                    "filtered",
                    None,
                );
                info!(
                    actor = %actor_key(&actor),
                    event_id = %event.id,
                    kind = %kind_tag(&event.kind),
                    source = %event.source,
                    symbols = ?event.symbols,
                    "skipped by user prefs"
                );
                continue;
            }
            // T0 推送不能等待第二次、actor-scoped 的 Grok 对账。A 级画像在后台
            // 把本季事实与旧问题/承诺逐项核对并落入 append-only 研究账本；失败
            // 只影响深度卡，不阻塞已核验的即时事实卡。
            self.schedule_earnings_material_record(&actor, event);
            self.schedule_earnings_continuity(&actor, event);
            // 结构化财报卡已经对该 actor 成功交付时，同一新闻稿型
            // 8-K 不再入摘要。只认 sent/dryrun 成功证据；财报卡失败或
            // quiet-held 时保留 SEC 项，queued 时由 digest buffer 保留优先级
            // 更高的结构化卡，不能把待投递路径整体清空。
            if is_earnings_release_document_event(event) {
                match event.url.as_deref().map(|url| {
                    self.store
                        .actor_has_delivered_earnings_for_document(&actor_key(&actor), url)
                }) {
                    Some(Ok(true)) => {
                        let _ = self.store.log_delivery(
                            &event.id,
                            &actor_key(&actor),
                            "router",
                            sev,
                            "superseded",
                            None,
                        );
                        info!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            "earnings-release SEC document suppressed after structured earnings delivery"
                        );
                        continue;
                    }
                    Some(Err(error)) => tracing::warn!(
                        actor = %actor_key(&actor),
                        event_id = %event.id,
                        degraded = true,
                        "earnings delivery lookup failed; keeping SEC digest fallback: {error:#}"
                    ),
                    _ => {}
                }
            }
            // High daily cap:同一 actor 当日 sink-sent High 条数达到上限后,
            // 后续 High 一律降级到 digest,避免"某 ticker 一天连发 8-K + 财报 +
            // 价格异动"把用户淹没。降级路径不双写 log:digest 入队时 status 写
            // "capped" 而不是 "queued",便于复盘统计"今日被降级多少条"。
            // cap=0 关闭该逻辑,与历史行为兼容。
            let mut demoted_by_cap = false;
            let mut demoted_by_cooldown = false;
            let mut demoted_by_price_advance = false;
            let mut effective_sev = if matches!(sev, Severity::High) && self.high_daily_cap > 0 {
                let since = local_day_start(chrono::Utc::now(), self.tz_offset_hours);
                let category = event_category(event);
                match self.store.count_high_sent_since_for_category(
                    &actor_key(&actor),
                    since,
                    category,
                ) {
                    Ok(n) if n >= self.high_daily_cap as i64 => {
                        tracing::info!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            source = %event.source,
                            category = %category,
                            today_high = n,
                            cap = self.high_daily_cap,
                            "High 事件降级进 digest(已超当日上限)"
                        );
                        demoted_by_cap = true;
                        Severity::Medium
                    }
                    Ok(_) => sev,
                    Err(e) => {
                        tracing::warn!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            source = %event.source,
                            category = %category,
                            since = %since,
                            "count_high_sent_since failed: {e:#}"
                        );
                        sev
                    }
                }
            } else {
                sev
            };
            // 价格 band 单一推送规则:新档 pct 必须比当日已 sink-sent 最大档 pct
            // 至少高出 actor 最终 `repeat_step_pct`,否则降级 digest。这一条规则
            // 替代了旧的 daily cap + intraday gap —— 因为 band id 已自带「同档位
            // INSERT IGNORE」防重,所以不再需要时间 gap 兜底;`monotone 新高 + N」
            // 既保护了「同档位反复震荡不刷屏」(任何 ≤max 的 band 都被挡),又允许
            // 大行情按节奏全档位推送(每 +N pct 一条)。N=2.0 默认与 band step 一致,
            // 等价于「每跨一个新 band 必推」。
            if matches!(effective_sev, Severity::High)
                && is_intraday_price_band_alert(event)
                && price_policy.repeat_step_pct > 0.0
            {
                if let Some((symbol, direction)) = price_alert_symbol_direction(event) {
                    let day_start = local_day_start(chrono::Utc::now(), self.tz_offset_hours);
                    let current_bps = event
                        .payload
                        .get("hone_price_band_bps")
                        .and_then(|v| v.as_i64());
                    let min_advance_bps = (price_policy.repeat_step_pct * 100.0).round() as i64;
                    match (
                        current_bps,
                        self.store.last_price_band_max_bps_for_symbol_direction(
                            &actor_key(&actor),
                            symbol,
                            direction,
                            day_start,
                        ),
                    ) {
                        (Some(cur), Ok(Some(prev_max))) if cur < prev_max + min_advance_bps => {
                            tracing::info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                source = %event.source,
                                symbol = %symbol,
                                direction = %direction,
                                current_band_bps = cur,
                                prev_max_band_bps = prev_max,
                                min_advance_pct = price_policy.repeat_step_pct,
                                "price band demoted to digest (no monotone advance ≥ min_advance_pct)"
                            );
                            demoted_by_price_advance = true;
                            effective_sev = Severity::Medium;
                        }
                        (Some(_), Ok(None)) => {
                            // 当日首条 band —— 直接放行。
                        }
                        (Some(_), Ok(Some(_))) => {
                            // 满足 advance 条件,继续直推。
                        }
                        (None, _) => {
                            // current_bps 取不到,说明 payload 没写 band_bps 字段,
                            // fallback 到放行(兼容旧事件 / 异常)。
                        }
                        (_, Err(e)) => {
                            tracing::warn!(
                                "last_price_band_max_bps_for_symbol_direction failed: {e:#}"
                            );
                        }
                    }
                }
            }
            // 同 ticker 冷却:如果事件还是 High,且 cooldown>0,检查任一 symbol 最近一次
            // High+sink+sent 的时间戳,若在冷却窗口内则降级进 digest。
            // AnalystGrade 额外按 gradingCompany 拆冷却 key —— 同 ticker 不同投行
            // 同分钟到达视为独立信号,不互相冷却(同投行同 ticker 仍受 60min 冷却约束)。
            // 但同一来源文章(newsURL)拆出的多投行 fanout 仍视为同一批信号,保留
            // 第一条代表即可,后续降级进 digest。
            if matches!(effective_sev, Severity::High)
                && self.same_symbol_cooldown_minutes > 0
                && matches!(event.kind, EventKind::AnalystGrade)
            {
                if let Some((symbol, news_url)) = analyst_grade_source_article_key(event) {
                    let cutoff = chrono::Utc::now()
                        - chrono::Duration::minutes(self.same_symbol_cooldown_minutes as i64);
                    match self.store.last_high_sink_send_for_analyst_news_url(
                        &actor_key(&actor),
                        symbol,
                        news_url,
                        cutoff,
                    ) {
                        Ok(Some(ts)) => {
                            tracing::info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                source = %event.source,
                                symbol = %symbol,
                                news_url = %news_url,
                                last_sent_at = %ts,
                                cooldown_min = self.same_symbol_cooldown_minutes,
                                "AnalystGrade fanout demoted to digest(same source article already sent)"
                            );
                            demoted_by_cooldown = true;
                            effective_sev = Severity::Medium;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::warn!(
                                "last_high_sink_send_for_analyst_news_url failed for {symbol}: {e:#}"
                            );
                        }
                    }
                }
            }
            if matches!(effective_sev, Severity::High)
                && self.same_symbol_cooldown_minutes > 0
                && !event.symbols.is_empty()
                && !is_intraday_price_band_alert(event)
            {
                let cutoff = chrono::Utc::now()
                    - chrono::Duration::minutes(self.same_symbol_cooldown_minutes as i64);
                let firm: Option<String> = if matches!(event.kind, EventKind::AnalystGrade) {
                    event
                        .payload
                        .get("gradingCompany")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                } else {
                    None
                };
                for sym in &event.symbols {
                    match self.store.last_high_sink_send_for_symbol_category(
                        &actor_key(&actor),
                        sym,
                        event_category(event),
                        firm.as_deref(),
                    ) {
                        Ok(Some(ts)) if ts >= cutoff => {
                            tracing::info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                source = %event.source,
                                symbol = %sym,
                                last_sent_at = %ts,
                                cooldown_min = self.same_symbol_cooldown_minutes,
                                "High 事件降级进 digest(同 ticker 冷却中)"
                            );
                            demoted_by_cooldown = true;
                            effective_sev = Severity::Medium;
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!(
                                "last_high_sink_send_for_symbol failed for {sym}: {e:#}"
                            );
                        }
                    }
                }
            }
            // quiet_hours hold:用户设了勿扰时段且当前时刻在区间内,High 即时推不发,
            // 写 delivery_log status='quiet_held',留给 UnifiedDigestScheduler 在
            // quiet.to 时刻的 quiet_flush 合集里复活(过保鲜期则 drop)。Medium/Low
            // 进入 buffer;unified scheduler 在 quiet 区间内会跳过 fire,自然累积到 to。
            // exempt_kinds 命中的 kind 即使在 quiet 内仍然立即推。
            if matches!(effective_sev, Severity::High) {
                if let Some(qh) = user_prefs.quiet_hours.as_ref() {
                    let tz = EffectiveTz::from_actor_prefs(
                        user_prefs.timezone.as_deref(),
                        self.tz_offset_hours,
                    );
                    let now = chrono::Utc::now();
                    let kind_t = kind_tag(&event.kind);
                    let exempt = qh.exempt_kinds.iter().any(|t| t == kind_t);
                    if !exempt && tz.in_quiet_window(now, &qh.from, &qh.to) {
                        let _ = self.store.log_delivery(
                            &event.id,
                            &actor_key(&actor),
                            "sink",
                            sev,
                            "quiet_held",
                            None,
                        );
                        tracing::info!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            kind = %kind_t,
                            quiet_from = %qh.from,
                            quiet_to = %qh.to,
                            "High event held by quiet_hours, will be flushed at quiet.to"
                        );
                        continue;
                    }
                }
            }
            match effective_sev {
                Severity::High => {
                    // 批模式下,盘中 band High 先进合流缓冲,批尾统一决定
                    // 「逐条发」还是「合并成一条集体异动」——防止开盘集体跳空
                    // 时 10 秒内连发 N 条 DM(见 config::PRICE_BURST_MIN_MERGE)。
                    if is_intraday_price_band_alert(event) {
                        let stashed = {
                            let mut burst =
                                self.price_burst.lock().expect("price burst lock poisoned");
                            if let Some(map) = burst.as_mut() {
                                map.entry(actor_key(&actor))
                                    .or_insert_with(|| (actor.clone(), Vec::new()))
                                    .1
                                    .push((event.clone(), sev));
                                true
                            } else {
                                false
                            }
                        };
                        if stashed {
                            continue;
                        }
                    }
                    if self
                        .deliver_high_immediate(&actor, event, sev, &user_prefs)
                        .await
                    {
                        sent += 1;
                    }
                }
                Severity::Medium | Severity::Low => {
                    match self.digest.enqueue(&actor, event) {
                        Ok(()) => {
                            // 被 cap 降级的条目记 status="capped",被同 ticker 冷却降级的
                            // 记 "cooled_down",正常流程记 "queued"。severity 仍记原始严重度
                            // (sev),方便事后 grep "high + capped/cooled_down" 对账。
                            let status = if demoted_by_cap {
                                "capped"
                            } else if demoted_by_price_advance {
                                "price_low_advance"
                            } else if demoted_by_cooldown {
                                "cooled_down"
                            } else {
                                "queued"
                            };
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                status,
                                None,
                            );
                            info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                kind = %kind_tag(&event.kind),
                                source = %event.source,
                                symbols = ?event.symbols,
                                severity = ?sev,
                                status = %status,
                                "digest queued"
                            );
                            pending += 1;
                        }
                        Err(e) => {
                            tracing::warn!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                kind = %kind_tag(&event.kind),
                                source = %event.source,
                                symbols = ?event.symbols,
                                severity = ?sev,
                                "digest enqueue failed: {e:#}"
                            );
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                "failed",
                                None,
                            );
                        }
                    }
                }
            }
        }
        Ok((sent, pending))
    }

    /// 评级事件 → 注入近 30 日共识计数(`hone_analyst_consensus_30d`)。
    /// 非评级 / 无 symbol / 查询失败 → 原样返回。
    fn annotate_analyst_consensus(&self, event: MarketEvent) -> MarketEvent {
        if !matches!(event.kind, EventKind::AnalystGrade) {
            return event;
        }
        let Some(symbol) = event.symbols.first() else {
            return event;
        };
        let end = event.occurred_at.max(chrono::Utc::now());
        let start = end - chrono::Duration::days(30);
        let payloads = match self
            .store
            .list_analyst_grade_payloads_in_window(symbol, start, end)
        {
            Ok(payloads) => payloads,
            Err(error) => {
                tracing::warn!(event_id = %event.id, "共识计数查询失败: {error:#}");
                return event;
            }
        };
        let counts = crate::pollers::analyst_grade::consensus_counts_from_payloads(&payloads);
        if counts.total() == 0 {
            return event;
        }
        let mut annotated = event;
        if let Some(obj) = annotated.payload.as_object_mut() {
            obj.insert(
                "hone_analyst_consensus_30d".into(),
                serde_json::json!({
                    "down": counts.down,
                    "up": counts.up,
                    "initiated": counts.init,
                    "reiterated": counts.reiter,
                }),
            );
        }
        annotated
    }

    /// actor 持有事件任一标的时返回注入仓位上下文的克隆,否则 None。
    fn position_annotated_event_for(
        &self,
        event: &MarketEvent,
        actor: &hone_core::ActorIdentity,
    ) -> Option<MarketEvent> {
        let registry = self.registry.load();
        let position = event
            .symbols
            .iter()
            .find_map(|symbol| registry.position_for(actor, symbol))?;
        super::position::position_annotated_event(event, position)
    }

    /// High 即时推送的完整出站路径:render → polish → send → 审计日志 →
    /// 财报卡 supersede。从 `dispatch` 的 High arm 原样抽出,行为不变;
    /// `flush_dispatch_batch` 的「小于合并阈值逐条发」也复用这条路径。
    /// 返回是否成功送达。
    async fn deliver_high_immediate(
        &self,
        actor: &hone_core::ActorIdentity,
        event: &MarketEvent,
        sev: Severity,
        user_prefs: &NotificationPrefs,
    ) -> bool {
        let fmt = self.sink.format_for(actor);
        let mainline = actor_mainline_for_event(event, user_prefs);
        let default_body = renderer::render_immediate_with_mainline(event, fmt, mainline);
        let body = if matches!(fmt, RenderFormat::Plain) && !is_structured_earnings_review(event) {
            match self.polisher.polish(event, &default_body).await {
                Some(polished) => polished,
                None => default_body,
            }
        } else {
            default_body
        };
        if let Err(e) = self.sink.send(actor, &body).await {
            tracing::warn!(
                actor = %actor_key(actor),
                event_id = %event.id,
                kind = %kind_tag(&event.kind),
                source = %event.source,
                symbols = ?event.symbols,
                body_len = body.chars().count(),
                body_preview = %body_preview(&body),
                "sink send failed: {e:#}"
            );
            let _ = self.store.log_delivery(
                &event.id,
                &actor_key(actor),
                "sink",
                sev,
                "failed",
                Some(&body),
            );
            return false;
        }
        let success_status = self.sink.success_status_for(actor);
        let delivery_result = if success_status == "sent" {
            self.store
                .log_confirmed_delivery(&event.id, actor, "sink", sev, &body, None)
        } else {
            self.store.log_delivery(
                &event.id,
                &actor_key(actor),
                "sink",
                sev,
                success_status,
                Some(&body),
            )
        };
        if let Err(error) = delivery_result {
            tracing::warn!(
                actor = %actor_key(actor),
                event_id = %event.id,
                "confirmed delivery audit failed: {error:#}"
            );
        }
        tracing::info!(
            actor = %actor_key(actor),
            event_id = %event.id,
            kind = %kind_tag(&event.kind),
            source = %event.source,
            symbols = ?event.symbols,
            severity = ?sev,
            status = %success_status,
            body_len = body.chars().count(),
            body_preview = %body_preview(&body),
            "sink delivered"
        );
        if is_structured_earnings_review(event)
            && let Some(document_url) = event.url.as_deref()
        {
            match self
                .digest
                .remove_earnings_release_documents(actor, document_url)
            {
                Ok(removed) => {
                    for superseded in removed {
                        let _ = self.store.log_delivery(
                            &superseded.id,
                            &actor_key(actor),
                            "router",
                            superseded.severity,
                            "superseded",
                            None,
                        );
                    }
                }
                Err(error) => tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    degraded = true,
                    "failed to remove superseded earnings-release SEC digest item: {error:#}"
                ),
            }
        }
        true
    }

    /// 激活批内合流。`process_events` 在每批 poll 入口调用;直接调 `dispatch`
    /// 的调用方(测试/嵌入)不激活,维持逐条即时行为。
    pub fn begin_dispatch_batch(&self) {
        *self.price_burst.lock().expect("price burst lock poisoned") =
            Some(std::collections::HashMap::new());
    }

    /// 批尾清算合流缓冲:同 actor ≥ `PRICE_BURST_MIN_MERGE` 条盘中 band High
    /// 合成一条「集体异动」汇总消息;更少则复用 `deliver_high_immediate` 逐条发。
    /// 返回实际出站的消息条数。
    pub async fn flush_dispatch_batch(&self) -> u32 {
        let stashed = self
            .price_burst
            .lock()
            .expect("price burst lock poisoned")
            .take();
        let Some(map) = stashed else {
            return 0;
        };
        let mut sent = 0u32;
        for (_key, (actor, items)) in map {
            if items.len() < super::config::PRICE_BURST_MIN_MERGE {
                let user_prefs = self.prefs.load(&actor);
                for (event, sev) in items {
                    if self
                        .deliver_high_immediate(&actor, &event, sev, &user_prefs)
                        .await
                    {
                        sent += 1;
                    }
                }
                continue;
            }
            let fmt = self.sink.format_for(&actor);
            let body = render_price_burst(&items, fmt);
            match self.sink.send(&actor, &body).await {
                Ok(()) => {
                    let success_status = self.sink.success_status_for(&actor);
                    for (event, sev) in &items {
                        let log_result = if success_status == "sent" {
                            self.store.log_confirmed_delivery(
                                &event.id, &actor, "sink", *sev, &body, None,
                            )
                        } else {
                            self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "sink",
                                *sev,
                                success_status,
                                None,
                            )
                        };
                        if let Err(error) = log_result {
                            tracing::warn!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                "burst delivery audit failed: {error:#}"
                            );
                        }
                    }
                    tracing::info!(
                        actor = %actor_key(&actor),
                        items = items.len(),
                        status = %success_status,
                        body_preview = %body_preview(&body),
                        "price burst merged and delivered"
                    );
                    sent += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        actor = %actor_key(&actor),
                        items = items.len(),
                        body_preview = %body_preview(&body),
                        "price burst sink send failed: {e:#}"
                    );
                    for (event, sev) in &items {
                        let _ = self.store.log_delivery(
                            &event.id,
                            &actor_key(&actor),
                            "sink",
                            *sev,
                            "failed",
                            None,
                        );
                    }
                }
            }
        }
        sent
    }
}

/// 合并 N 条盘中 band High 为一条「集体异动」正文。
fn render_price_burst(items: &[(MarketEvent, Severity)], fmt: RenderFormat) -> String {
    let header = format!("⚡ 盘中集体异动 · {} 只", items.len());
    let header = match fmt {
        RenderFormat::DiscordMarkdown => format!("**{header}**"),
        _ => header,
    };
    let mut lines = vec![header];
    for (event, _) in items {
        if event.summary.trim().is_empty() {
            lines.push(format!("• {}", event.title));
        } else {
            lines.push(format!("• {} · {}", event.title, event.summary));
        }
    }
    lines.join("\n")
}

impl NotificationRouter {
    fn schedule_earnings_material_record(
        &self,
        actor: &hone_core::ActorIdentity,
        event: &MarketEvent,
    ) {
        let Some(material_kind) = earnings_research_material_kind(event) else {
            return;
        };
        let Some(reconciler) = self.earnings_continuity.clone() else {
            return;
        };
        let materials = if material_kind == "earnings_release" {
            let Some(research_object_key) = earnings_research_object_key_for_event(event) else {
                return;
            };
            match self
                .store
                .list_earnings_research_materials(&research_object_key)
            {
                Ok(materials) => materials,
                Err(error) => {
                    tracing::warn!(
                        actor = %actor_key(actor),
                        event_id = %event.id,
                        research_object_key = %research_object_key,
                        "earnings linked material lookup failed: {error:#}"
                    );
                    return;
                }
            }
        } else {
            vec![event.clone()]
        };
        if materials.is_empty() {
            return;
        }
        let actor = actor.clone();
        tokio::task::spawn_blocking(move || {
            for material in materials {
                if let Some(outcome) = reconciler.record_material(&actor, &material) {
                    tracing::info!(
                        actor = %actor_key(&actor),
                        event_id = %material.id,
                        research_object_key = %outcome.research_object_key,
                        profile_id = %outcome.profile_id,
                        material_kind = %outcome.material_kind,
                        recorded_event_id = %outcome.recorded_event_id,
                        "earnings research material appended to quarterly object"
                    );
                }
            }
        });
    }

    fn schedule_earnings_continuity(&self, actor: &hone_core::ActorIdentity, event: &MarketEvent) {
        if continuity_review_stage(event).is_none() {
            return;
        }
        let Some(reconciler) = self.earnings_continuity.clone() else {
            return;
        };
        if !reconciler.should_schedule(actor, event) {
            return;
        }
        let job_key = match self.store.enqueue_earnings_continuity_job(actor, event) {
            Ok(Some(job_key)) => job_key,
            Ok(None) => return,
            Err(error) => {
                tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    "earnings continuity job enqueue failed: {error:#}"
                );
                return;
            }
        };
        let store = self.store.clone();
        tokio::spawn(async move {
            if let Err(error) = run_earnings_continuity_jobs_once(store, reconciler, 4).await {
                tracing::warn!(job_key = %job_key, "earnings continuity worker failed: {error:#}");
            }
        });
    }

    /// 启动进程级恢复 worker。立即扫描一次，之后每分钟回收 pending/retry 或
    /// 15 分钟租约过期的 running 任务；模型失败按指数退避，T0 投递永不等待它。
    pub(crate) fn spawn_earnings_continuity_retry_worker(&self) {
        let Some(reconciler) = self.earnings_continuity.clone() else {
            return;
        };
        let store = self.store.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                if let Err(error) =
                    run_earnings_continuity_jobs_once(store.clone(), reconciler.clone(), 4).await
                {
                    tracing::warn!("earnings continuity retry tick failed: {error:#}");
                }
            }
        });
    }
}

async fn run_earnings_continuity_jobs_once(
    store: std::sync::Arc<crate::store::EventStore>,
    reconciler: std::sync::Arc<dyn crate::earnings_continuity::EarningsContinuityReconciler>,
    limit: usize,
) -> anyhow::Result<usize> {
    let jobs = store.claim_due_earnings_continuity_jobs(chrono::Utc::now(), limit)?;
    let claimed = jobs.len();
    for job in jobs {
        if !reconciler.should_schedule(&job.actor, &job.event) {
            let completed = store.complete_earnings_continuity_job(&job.job_key, job.attempts)?;
            if !completed {
                tracing::warn!(
                    job_key = %job.job_key,
                    attempts = job.attempts,
                    "earnings continuity coverage-close ignored after lease ownership changed"
                );
                continue;
            }
            tracing::info!(
                actor = %actor_key(&job.actor),
                event_id = %job.event.id,
                job_key = %job.job_key,
                "earnings continuity job closed because A-tier coverage is no longer active"
            );
            continue;
        }
        if let Some(outcome) = reconciler.reconcile(&job.actor, &job.event).await {
            let completed = store.complete_earnings_continuity_job(&job.job_key, job.attempts)?;
            if !completed {
                tracing::warn!(
                    job_key = %job.job_key,
                    attempts = job.attempts,
                    "earnings continuity durable result kept but stale lease could not complete job"
                );
                continue;
            }
            tracing::info!(
                actor = %actor_key(&job.actor),
                event_id = %job.event.id,
                job_key = %job.job_key,
                attempts = job.attempts,
                research_object_key = %outcome.research_object_key,
                thesis_effect = %outcome.thesis_effect,
                checked_existing_items = outcome.checked_existing_items,
                created_questions = outcome.created_questions,
                created_commitments = outcome.created_commitments,
                active_questions_after = outcome.active_questions_after,
                active_commitments_after = outcome.active_commitments_after,
                "earnings continuity ledger reconciled"
            );
        } else {
            let retried = store.retry_earnings_continuity_job(
                &job.job_key,
                job.attempts,
                "reconciler returned no durable outcome",
                chrono::Utc::now(),
            )?;
            if !retried {
                tracing::warn!(
                    job_key = %job.job_key,
                    attempts = job.attempts,
                    "earnings continuity retry ignored after lease ownership changed"
                );
                continue;
            }
            tracing::warn!(
                actor = %actor_key(&job.actor),
                event_id = %job.event.id,
                job_key = %job.job_key,
                attempts = job.attempts,
                "earnings continuity job scheduled for retry"
            );
        }
    }
    Ok(claimed)
}

fn analyst_grade_source_article_key(event: &MarketEvent) -> Option<(&str, &str)> {
    if !matches!(event.kind, EventKind::AnalystGrade) {
        return None;
    }
    let symbol = event.symbols.first()?.trim();
    if symbol.is_empty() {
        return None;
    }
    let news_url = event
        .payload
        .get("newsURL")
        .and_then(|v| v.as_str())
        .or(event.url.as_deref())?
        .trim();
    if news_url.is_empty() {
        return None;
    }
    Some((symbol, news_url))
}

fn actor_mainline_for_event<'a>(
    event: &MarketEvent,
    prefs: &'a NotificationPrefs,
) -> Option<&'a str> {
    if !matches!(event.kind, EventKind::EarningsReleased) {
        return None;
    }
    let mainlines = prefs.mainline_by_ticker.as_ref()?;
    event.symbols.iter().find_map(|symbol| {
        mainlines
            .iter()
            .find(|(ticker, _)| ticker.eq_ignore_ascii_case(symbol))
            .map(|(_, mainline)| mainline.trim())
            .filter(|mainline| !mainline.is_empty())
    })
}

fn is_structured_earnings_review(event: &MarketEvent) -> bool {
    matches!(event.kind, EventKind::EarningsReleased)
        && event
            .payload
            .get("earnings_quality_review_applied")
            .and_then(|value| value.as_bool())
            == Some(true)
}
