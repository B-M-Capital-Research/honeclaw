//! `NotificationRouter::dispatch` —— 主分发管线。
//!
//! 输入一个 `MarketEvent`,做 5 步:
//! 1. **黑名单**:`disabled_kinds` 命中直接 (0,0);
//! 2. **预升级**:多信号合流(`maybe_upgrade_news`)+ 系统级策略
//!    (`apply_system_event_policy`);
//! 3. **解析订阅**:从 `SharedRegistry` 拿到匹配的 (actor, sev) 列表,空则 (0,0);
//! 4. **per-actor 过滤** loop:LLM 仲裁 / per_actor severity override / quiet
//!    mode / close-alert demote / prefs filter / high_daily_cap / price band
//!    cap+gap / same-symbol cooldown;
//! 5. **路由**:High → polish + sink.send + delivery_log;
//!    Medium/Low → digest.enqueue + delivery_log。
//!
//! 实现尽量保持「线性、扁平」,把每个降级原因记成不同的 `demoted_by_*` flag
//! 而不是嵌套 if-else,这样 grep `delivery_log.status` 能直接对账。

use tracing::info;

use crate::digest::time_window::EffectiveTz;
use crate::event::{MarketEvent, Severity};
use crate::prefs::kind_tag;
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
            // LLM 仲裁:不确定来源的 Low NewsCritical,按 actor 重要性 prompt
            // 决定是否升 Medium。结果只影响本 actor 的本次分发,不污染原 event。
            let actor_event_buf;
            let (event, sev) = match self.maybe_llm_upgrade_for_actor(event, &user_prefs).await {
                Some(upgraded) => {
                    actor_event_buf = upgraded;
                    (&actor_event_buf, Severity::Medium)
                }
                None => (event, sev),
            };
            // per-actor severity override:用户可自定义
            //   (a) price_high_pct_override:价格异动绝对值触达即升 High 即时推;
            //   (b) immediate_kinds:某些 kind 无条件升 High 即时推(例如 52 周高/低、
            //       分析师评级)。
            // 升级后仍要走 high_daily_cap / cooldown,保持 burst 防护。
            let mut sev = self.apply_per_actor_severity_override(event, sev, &user_prefs);
            sev = self.apply_quiet_mode(event, sev, &user_prefs);
            if is_price_close_alert(event)
                && !self.price_close_direct_enabled
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
            if !user_prefs.should_deliver(event) {
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
            // High daily cap:同一 actor 当日 sink-sent High 条数达到上限后,
            // 后续 High 一律降级到 digest,避免"某 ticker 一天连发 8-K + 财报 +
            // 价格异动"把用户淹没。降级路径不双写 log:digest 入队时 status 写
            // "capped" 而不是 "queued",便于复盘统计"今日被降级多少条"。
            // cap=0 关闭该逻辑,与历史行为兼容。
            let mut demoted_by_cap = false;
            let mut demoted_by_cooldown = false;
            let mut demoted_by_price_cap = false;
            let mut demoted_by_price_gap = false;
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
                        tracing::warn!("count_high_sent_since failed: {e:#}");
                        sev
                    }
                }
            } else {
                sev
            };
            if matches!(effective_sev, Severity::High) && is_intraday_price_band_alert(event) {
                if let Some((symbol, direction)) = price_alert_symbol_direction(event) {
                    if self.price_symbol_direction_daily_cap > 0 {
                        let since = local_day_start(chrono::Utc::now(), self.tz_offset_hours);
                        match self.store.count_price_band_sent_since(
                            &actor_key(&actor),
                            symbol,
                            direction,
                            since,
                        ) {
                            Ok(n) if n >= self.price_symbol_direction_daily_cap as i64 => {
                                tracing::info!(
                                    actor = %actor_key(&actor),
                                    event_id = %event.id,
                                    source = %event.source,
                                    symbol = %symbol,
                                    direction = %direction,
                                    today_price_bands = n,
                                    cap = self.price_symbol_direction_daily_cap,
                                    "price band demoted to digest (symbol-direction daily cap)"
                                );
                                demoted_by_price_cap = true;
                                effective_sev = Severity::Medium;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!("count_price_band_sent_since failed: {e:#}");
                            }
                        }
                    }
                    if matches!(effective_sev, Severity::High)
                        && self.price_intraday_min_gap_minutes > 0
                    {
                        let cutoff = chrono::Utc::now()
                            - chrono::Duration::minutes(self.price_intraday_min_gap_minutes as i64);
                        match self.store.last_price_band_sink_send_for_symbol_direction(
                            &actor_key(&actor),
                            symbol,
                            direction,
                        ) {
                            Ok(Some(ts)) if ts >= cutoff => {
                                tracing::info!(
                                    actor = %actor_key(&actor),
                                    event_id = %event.id,
                                    source = %event.source,
                                    symbol = %symbol,
                                    direction = %direction,
                                    last_sent_at = %ts,
                                    gap_min = self.price_intraday_min_gap_minutes,
                                    "price band demoted to digest (symbol-direction min gap)"
                                );
                                demoted_by_price_gap = true;
                                effective_sev = Severity::Medium;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!(
                                    "last_price_band_sink_send_for_symbol_direction failed for {symbol}: {e:#}"
                                );
                            }
                        }
                    }
                }
            }
            // 同 ticker 冷却:如果事件还是 High,且 cooldown>0,检查任一 symbol 最近一次
            // High+sink+sent 的时间戳,若在冷却窗口内则降级进 digest。
            if matches!(effective_sev, Severity::High)
                && self.same_symbol_cooldown_minutes > 0
                && !event.symbols.is_empty()
                && !is_intraday_price_band_alert(event)
            {
                let cutoff = chrono::Utc::now()
                    - chrono::Duration::minutes(self.same_symbol_cooldown_minutes as i64);
                for sym in &event.symbols {
                    match self.store.last_high_sink_send_for_symbol_category(
                        &actor_key(&actor),
                        sym,
                        event_category(event),
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
            // 写 delivery_log status='quiet_held',留给 DigestScheduler 在 quiet.to
            // 时刻的 quiet_flush 合集里复活(过保鲜期则 drop)。Medium/Low 走原 digest
            // 路径,DigestScheduler 在 quiet 区间内会跳过 fire,buffer 自然累积到 to。
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
                    let fmt = self.sink.format_for(&actor);
                    let default_body = renderer::render_immediate(event, fmt);
                    let body = if matches!(fmt, RenderFormat::Plain) {
                        match self.polisher.polish(event, &default_body).await {
                            Some(polished) => polished,
                            None => default_body,
                        }
                    } else {
                        default_body
                    };
                    if let Err(e) = self.sink.send(&actor, &body).await {
                        tracing::warn!(
                            actor = %actor_key(&actor),
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
                            &actor_key(&actor),
                            "sink",
                            sev,
                            "failed",
                            Some(&body),
                        );
                        continue;
                    }
                    let success_status = self.sink.success_status();
                    let _ = self.store.log_delivery(
                        &event.id,
                        &actor_key(&actor),
                        "sink",
                        sev,
                        success_status,
                        Some(&body),
                    );
                    tracing::info!(
                        actor = %actor_key(&actor),
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
                    sent += 1;
                }
                Severity::Medium | Severity::Low => {
                    match self.digest.enqueue(&actor, event) {
                        Ok(()) => {
                            // 被 cap 降级的条目记 status="capped",被同 ticker 冷却降级的
                            // 记 "cooled_down",正常流程记 "queued"。severity 仍记原始严重度
                            // (sev),方便事后 grep "high + capped/cooled_down" 对账。
                            let status = if demoted_by_cap {
                                "capped"
                            } else if demoted_by_price_cap {
                                "price_capped"
                            } else if demoted_by_price_gap {
                                "price_cooled_down"
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
                            tracing::warn!("digest enqueue failed: {e:#}");
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
}
