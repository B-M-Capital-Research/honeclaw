use super::*;

use std::sync::Arc;

#[tokio::test]
async fn start_respects_disabled_flag() {
    let engine = EventEngine::new(EventEngineConfig::default(), FmpConfig::default());
    engine.start().await.unwrap();
}

#[tokio::test]
async fn start_warns_when_enabled_but_no_key() {
    let mut event_engine_config = EventEngineConfig::default();
    event_engine_config.enabled = true;
    let engine = EventEngine::new(event_engine_config, FmpConfig::default());
    engine.start().await.unwrap();
}

/// 真实 E2E：engine → EarningsPoller → EventStore → Router(LogSink)。
/// 触发：`HONE_FMP_API_KEY=xxx cargo test -p hone-event-engine \
///        --  --ignored live_engine_e2e --nocapture`
#[tokio::test]
#[ignore]
async fn live_engine_e2e() {
    let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
    let fmp_cfg = FmpConfig {
        api_key: key,
        api_keys: vec![],
        base_url: "https://financialmodelingprep.com/api".into(),
        timeout: 30,
    };
    let mut engine_cfg = EventEngineConfig::default();
    engine_cfg.enabled = true;
    // earnings poller 在 v0.1.46 起改为 cron-aligned,冷启动会立即跑一次然后
    // 等到下一个 prefetch 窗口。8 秒 sleep 只会命中冷启动那一次 poll,足够做 e2e 校验。

    let temp_dir = tempfile::tempdir().unwrap();
    let store_path = temp_dir.path().join("event-store");
    let jsonl_path = temp_dir.path().join("events.jsonl");
    let portfolio_dir = temp_dir.path().join("portfolio");
    let engine = EventEngine::new(engine_cfg, fmp_cfg)
        .with_store_path(store_path.clone())
        .with_events_jsonl_path(Some(jsonl_path.clone()))
        .with_portfolio_dir(portfolio_dir)
        .with_retention_days(0);
    engine.start().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let store = EventStore::open(&store_path).unwrap();
    let stored_event_count = store.count_events().unwrap();
    let jsonl_lines = std::fs::read_to_string(&jsonl_path)
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as i64)
        .unwrap_or(-1);
    println!("e2e count_events = {stored_event_count} jsonl_lines = {jsonl_lines}");
    assert!(stored_event_count > 0, "PostgreSQL 应写入事件");
    assert!(jsonl_lines > 0, "JSONL 镜像应同步写入事件");
    assert_eq!(
        jsonl_lines, stored_event_count,
        "JSONL 行数应与 PostgreSQL events 行数一致（单次冷启，无去重丢失）"
    );
}

/// 手动触发 4 条不同时效/严重度的事件，分别渲染后直接推到 Telegram，
/// 验证从 renderer 到真渠道的端到端闭环。
///
/// 触发：
/// `HONE_TG_BOT_TOKEN=xxx HONE_TG_CHAT_ID=yyy cargo test \
///    -p hone-event-engine --lib tests::live_telegram_push_demo \
///    -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn live_telegram_push_demo() {
    use crate::event::{EventKind, MarketEvent, Severity};
    use chrono::Utc;

    let token = std::env::var("HONE_TG_BOT_TOKEN").expect("需要 HONE_TG_BOT_TOKEN");
    let chat_id = std::env::var("HONE_TG_CHAT_ID").expect("需要 HONE_TG_CHAT_ID");

    // 事件 1：High — 财报发布（应立即推）
    let ev_earnings = MarketEvent {
        id: "demo:earnings:aapl".into(),
        kind: EventKind::EarningsReleased,
        severity: Severity::High,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "Apple Q2 FY26 EPS $2.18 vs est $1.94，beat +12%".into(),
        summary: "营收 $97.3B（+7% YoY），服务业务创新高；公司上调回购至 $110B。".into(),
        url: Some("https://investor.apple.com/investor-relations/default.aspx".into()),
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    // 事件 2：High — SEC 8-K（应立即推）
    let ev_sec = MarketEvent {
        id: "demo:sec:tsla:8k".into(),
        kind: EventKind::SecFiling { form: "8-K".into() },
        severity: Severity::High,
        symbols: vec!["TSLA".into()],
        occurred_at: Utc::now(),
        title: "Tesla 提交 8-K：CFO 辞职".into(),
        summary: "CFO Vaibhav Taneja 于 2026-04-21 提交辞呈，立即生效；公司正在物色继任者。".into(),
        url: Some(
            "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001318605".into(),
        ),
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    // 事件 3：Medium — 拆股（正常走盘前摘要）
    let ev_split = MarketEvent {
        id: "demo:split:nvda".into(),
        kind: EventKind::Split,
        severity: Severity::Medium,
        symbols: vec!["NVDA".into()],
        occurred_at: Utc::now(),
        title: "NVDA 宣布 1-for-10 拆股，生效日 2026-05-20".into(),
        summary: "".into(),
        url: None,
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    // 事件 4：Low — 宏观数据（正常走盘后/晨间摘要）
    let ev_macro = MarketEvent {
        id: "demo:macro:cpi".into(),
        kind: EventKind::MacroEvent,
        severity: Severity::Low,
        symbols: vec![],
        occurred_at: Utc::now(),
        title: "[US] CPI MoM (Mar) · est 0.3 · prev 0.2".into(),
        summary: "".into(),
        url: None,
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    use crate::renderer::RenderFormat;

    // 每条事件推两版：Plain 与 TelegramHtml，便于在同一聊天窗口里逐条对比。
    // Plain 走 parse_mode=None；TelegramHtml 走 parse_mode=HTML。
    let variants = [RenderFormat::Plain, RenderFormat::TelegramHtml];
    let mut messages: Vec<(RenderFormat, String)> = Vec::new();
    for fmt in variants {
        let marker = match fmt {
            RenderFormat::Plain => "— Plain —".to_string(),
            RenderFormat::TelegramHtml => "— TelegramHtml —".to_string(),
            RenderFormat::DiscordMarkdown => "— Markdown —".to_string(),
            RenderFormat::FeishuPost => "— FeishuPost —".to_string(),
        };
        messages.push((fmt, marker));
        messages.push((fmt, crate::renderer::render_immediate(&ev_earnings, fmt)));
        messages.push((fmt, crate::renderer::render_immediate(&ev_sec, fmt)));
        messages.push((
            fmt,
            crate::digest::render_digest("盘前摘要 · 08:30", &[ev_split.clone()], 0, fmt),
        ));
        messages.push((
            fmt,
            crate::digest::render_digest("晨间摘要 · 09:00", &[ev_macro.clone()], 0, fmt),
        ));
    }

    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    for (fmt, text) in messages {
        let mut payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        if matches!(fmt, RenderFormat::TelegramHtml) {
            payload["parse_mode"] = serde_json::Value::String("HTML".into());
            // 锚文本已提供，禁掉 preview 让版式更紧凑
            payload["disable_web_page_preview"] = serde_json::Value::Bool(true);
        }
        let telegram_response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = telegram_response.status();
        let response_body = telegram_response.text().await.unwrap_or_default();
        println!("[tg demo] fmt={fmt:?} status={status} body={response_body}");
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {response_body}"
        );
        // Telegram 发送速率限制：每秒 30 条个人；留 500ms 间隔
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
}

/// LLM 润色演示：对若干 High severity 事件，先发默认模板，再发 LlmPolisher 润色版，
/// 直推到 Telegram，便于肉眼对比润色效果。
///
/// 触发：
/// `HONE_TG_BOT_TOKEN=xxx HONE_TG_CHAT_ID=yyy HONE_OPENROUTER_KEY=sk-or-... \
///   HONE_OPENROUTER_MODEL=google/gemini-3.1-pro-preview \
///   cargo test -p hone-event-engine --lib tests::live_telegram_push_llm_polished_demo \
///   -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn live_telegram_push_llm_polished_demo() {
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::polisher::{BodyPolisher, LlmPolisher};
    use crate::renderer::{RenderFormat, render_immediate};
    use chrono::Utc;
    use hone_llm::OpenRouterProvider;
    use std::collections::HashSet;
    use std::sync::Arc;

    let token = std::env::var("HONE_TG_BOT_TOKEN").expect("需要 HONE_TG_BOT_TOKEN");
    let chat_id = std::env::var("HONE_TG_CHAT_ID").expect("需要 HONE_TG_CHAT_ID");
    let openrouter_key = std::env::var("HONE_OPENROUTER_KEY").expect("需要 HONE_OPENROUTER_KEY");
    let openrouter_model = std::env::var("HONE_OPENROUTER_MODEL")
        .unwrap_or_else(|_| "google/gemini-3.1-pro-preview".to_string());

    // High 事件 1：财报发布
    let ev_earnings = MarketEvent {
        id: "demo:polish:earnings:aapl".into(),
        kind: EventKind::EarningsReleased,
        severity: Severity::High,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "Apple Q2 FY26 EPS $2.18 vs est $1.94，beat +12%".into(),
        summary: "营收 $97.3B（+7% YoY），服务业务创新高；公司上调回购至 $110B。".into(),
        url: Some("https://investor.apple.com/investor-relations/default.aspx".into()),
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    // High 事件 2：SEC 8-K
    let ev_sec = MarketEvent {
        id: "demo:polish:sec:tsla:8k".into(),
        kind: EventKind::SecFiling { form: "8-K".into() },
        severity: Severity::High,
        symbols: vec!["TSLA".into()],
        occurred_at: Utc::now(),
        title: "Tesla 提交 8-K：CFO 辞职".into(),
        summary: "CFO Vaibhav Taneja 于 2026-04-21 提交辞呈，立即生效；公司正在物色继任者。".into(),
        url: Some(
            "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001318605".into(),
        ),
        source: "demo".into(),
        payload: serde_json::Value::Null,
    };

    // 构建 LlmPolisher
    // 注：Gemini 3.x 是 reasoning 模型，会把大部分 token 预算花在思考链上，
    // 所以这里给到 4096 以避免"只输出标题就截断"。
    let provider = Arc::new(OpenRouterProvider::new(
        &openrouter_key,
        &openrouter_model,
        4096,
    ));
    let mut polish_levels = HashSet::new();
    polish_levels.insert(Severity::High);
    let polisher = LlmPolisher::new(provider, polish_levels);

    // 渲染四条消息：raw earnings / polished earnings / raw sec / polished sec
    let fmt = RenderFormat::TelegramHtml;
    let raw_earnings = render_immediate(&ev_earnings, fmt);
    let polished_earnings = polisher
        .polish(&ev_earnings, &raw_earnings)
        .await
        .expect("LLM 润色应返回 Some，检查 API key/网络");
    let raw_sec = render_immediate(&ev_sec, fmt);
    let polished_sec = polisher
        .polish(&ev_sec, &raw_sec)
        .await
        .expect("LLM 润色应返回 Some");

    // 打印到 stdout 方便 --nocapture 观察
    println!("\n=== RAW earnings ===\n{raw_earnings}\n");
    println!("=== POLISHED earnings ===\n{polished_earnings}\n");
    println!("=== RAW sec ===\n{raw_sec}\n");
    println!("=== POLISHED sec ===\n{polished_sec}\n");

    let messages: Vec<(bool, String)> = vec![
        (false, "— 原始模板 · Earnings —".into()),
        (true, raw_earnings),
        (false, "— LLM 润色 · Earnings —".into()),
        // 润色结果可能不是合法 HTML，按纯文本发更安全
        (false, polished_earnings),
        (false, "— 原始模板 · SEC 8-K —".into()),
        (true, raw_sec),
        (false, "— LLM 润色 · SEC 8-K —".into()),
        (false, polished_sec),
    ];

    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    for (use_html, text) in messages {
        let mut payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        if use_html {
            payload["parse_mode"] = serde_json::Value::String("HTML".into());
            payload["disable_web_page_preview"] = serde_json::Value::Bool(true);
        }
        let telegram_response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = telegram_response.status();
        let response_body = telegram_response.text().await.unwrap_or_default();
        println!("[tg polish demo] html={use_html} status={status} body={response_body}");
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {response_body}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
}

/// 真持仓回测：读 `data/portfolio/portfolio_telegram__direct__{CHAT_ID}.json`，
/// 对里面的 ticker 列表真跑 PricePoller / EarningsPoller / NewsPoller / CorpActionPoller +
/// 每只 ticker 拉最近 SEC 8-K，然后把结果组织成几条消息推到 Telegram。
///
/// 这是"盘前盘后 + 公司信息链路"端到端回测：真 actor → 真 FMP → 真 poller → 真推送。
///
/// 触发：
/// `HONE_TG_BOT_TOKEN=xxx HONE_TG_CHAT_ID=yyy HONE_FMP_API_KEY=zzz \
///   cargo test -p hone-event-engine --lib tests::live_portfolio_backtest_push \
///   -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn live_portfolio_backtest_push() {
    use crate::pollers::{
        CorpActionCalendarPoller, EarningsPoller, NewsPoller, PricePoller, SecFilingsPoller,
    };
    use crate::renderer::RenderFormat;
    use crate::source::{EventSource, SourceSchedule};
    use crate::subscription::{SharedRegistry, SubscriptionRegistry};

    let token = std::env::var("HONE_TG_BOT_TOKEN").expect("需要 HONE_TG_BOT_TOKEN");
    let chat_id = std::env::var("HONE_TG_CHAT_ID").expect("需要 HONE_TG_CHAT_ID");
    let fmp_key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");

    // 1) 读持仓：直接读 JSON，不走 PortfolioStorage，避免引入新依赖路径。
    // cargo test cwd = crate 目录，需要回到 workspace 根再进 data/
    let ws_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("无法定位 workspace 根");
    let portfolio_path = ws_root
        .join("data/portfolio")
        .join(format!("portfolio_telegram__direct__{chat_id}.json"))
        .to_string_lossy()
        .to_string();
    let raw = std::fs::read_to_string(&portfolio_path)
        .unwrap_or_else(|e| panic!("读持仓失败 {portfolio_path}: {e}"));
    let portfolio: serde_json::Value = serde_json::from_str(&raw).expect("持仓 JSON 格式错");
    let holdings = portfolio["holdings"].as_array().expect("holdings 数组缺");
    let symbols: Vec<String> = holdings
        .iter()
        .filter_map(|h| h.get("symbol")?.as_str().map(|s| s.to_uppercase()))
        .collect();
    let cost_map: std::collections::HashMap<String, (f64, f64)> = holdings
        .iter()
        .filter_map(|h| {
            let s = h.get("symbol")?.as_str()?.to_uppercase();
            let shares = h.get("shares")?.as_f64()?;
            let avg = h.get("avg_cost")?.as_f64()?;
            Some((s, (shares, avg)))
        })
        .collect();
    println!("持仓 {} 只: {}", symbols.len(), symbols.join(","));
    assert!(!symbols.is_empty(), "持仓为空");

    // 2) FMP 客户端
    let fmp_cfg = hone_core::config::FmpConfig {
        api_key: fmp_key,
        api_keys: vec![],
        base_url: "https://financialmodelingprep.com/api".into(),
        timeout: 30,
    };
    let fmp = crate::fmp::FmpClient::from_config(&fmp_cfg);

    // 3) PricePoller —— 阈值放宽到 1% 以看出所有异动；同时拿到 quote 原始 payload
    //    用于合成盘前快照（含 P&L）。
    // 测试不走 EventSource::poll（不依赖 registry）,直接用 fetch(symbols) 喂持仓列表。
    let price_registry =
        std::sync::Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new()));
    let price_poller = PricePoller::new(
        fmp.clone(),
        price_registry,
        SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
    )
    .with_thresholds(1.0, 5.0);
    let price_events = price_poller
        .fetch(&symbols)
        .await
        .expect("PricePoller poll 失败");
    println!("PriceEvents: {}", price_events.len());

    // 额外拉一次 v3/quote 拿原始价格（PricePoller 只在阈值触发时输出事件）
    let joined = symbols.join(",");
    let quote_raw = fmp
        .get_json(&format!("/v3/quote/{joined}"))
        .await
        .expect("FMP quote 请求失败");
    let quote_arr = quote_raw.as_array().cloned().unwrap_or_default();

    // 组装盘前快照正文（手动渲染，含 P&L vs 成本）
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // 按日涨跌幅绝对值从大到小排序，让"异动"首先映入眼帘
    #[derive(Clone)]
    struct Row {
        sym: String,
        price: f64,
        pct: f64,
        avg_cost: f64,
        pnl: f64,
        mv: f64,
    }
    let mut rows: Vec<Row> = quote_arr
        .iter()
        .map(|q| {
            let sym = q
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let price = q.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let pct = q
                .get("changesPercentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let (shares, avg_cost) = cost_map.get(&sym).copied().unwrap_or((0.0, 0.0));
            let mv = price * shares;
            Row {
                sym,
                price,
                pct,
                avg_cost,
                pnl: (price - avg_cost) * shares,
                mv,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.pct
            .abs()
            .partial_cmp(&a.pct.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total_value: f64 = rows.iter().map(|r| r.mv).sum();
    let total_pnl: f64 = rows.iter().map(|r| r.pnl).sum();
    let up = rows.iter().filter(|r| r.pct > 0.0).count();
    let down = rows.iter().filter(|r| r.pct < 0.0).count();

    let fmt_row = |r: &Row| {
        let arrow = if r.pct >= 0.0 { "▲" } else { "▼" };
        let pnl_sign = if r.pnl >= 0.0 { "+" } else { "" };
        format!(
            "• ${}  {:>7.2}  {arrow}{:>5.2}%   成本 {:.2} · P&L {pnl_sign}${:.0}",
            r.sym,
            r.price,
            r.pct.abs(),
            r.avg_cost,
            r.pnl
        )
    };
    let mut snapshot = format!(
        "📊 持仓盘前快照 · {today} · {} 只（↑{up} ↓{down}）\n",
        symbols.len()
    );
    for r in &rows {
        snapshot.push_str(&fmt_row(r));
        snapshot.push('\n');
    }
    snapshot.push_str(&format!(
        "\n合计市值 ${:.0} · 浮动盈亏 {}${:.0}",
        total_value,
        if total_pnl >= 0.0 { "+" } else { "-" },
        total_pnl.abs()
    ));

    // 4) EarningsPoller —— 14 天窗口；filter 到持仓
    let earn_poller = EarningsPoller::new(
        fmp.clone(),
        crate::source::SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
    );
    let earn_all = crate::source::EventSource::poll(&earn_poller)
        .await
        .expect("EarningsPoller poll 失败");
    let holdings_set: std::collections::HashSet<&str> =
        symbols.iter().map(|s| s.as_str()).collect();
    let earn_filt: Vec<_> = earn_all
        .into_iter()
        .filter(|e| e.symbols.iter().any(|s| holdings_set.contains(s.as_str())))
        .collect();
    println!("EarningsEvents (持仓过滤后): {}", earn_filt.len());

    // 5) NewsPoller —— 只拉持仓相关；拿 high + 全部 low 预览
    let news_poller = NewsPoller::new(
        fmp.clone(),
        crate::source::SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
    )
    .with_tickers(symbols.clone())
    .with_page_limit(40);
    let news_all = crate::source::EventSource::poll(&news_poller)
        .await
        .expect("NewsPoller poll 失败");
    println!(
        "NewsEvents: {} (High {} / Low {})",
        news_all.len(),
        news_all
            .iter()
            .filter(|e| matches!(e.severity, crate::event::Severity::High))
            .count(),
        news_all
            .iter()
            .filter(|e| matches!(e.severity, crate::event::Severity::Low))
            .count(),
    );

    // 6) CorpActionCalendar + SecFilings —— 现在是两个独立 EventSource。
    //    sec_recent_hours=72: 只推过去 72h 的 8-K,老文件 FMP 每次拉都会返回
    //    但上游已经消化过,再推就是刷屏。
    let cal_poller = CorpActionCalendarPoller::new(
        fmp.clone(),
        SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
    );
    let ca_calendar = EventSource::poll(&cal_poller).await.unwrap_or_else(|e| {
        println!("CorpAction calendar 失败（跳过）: {e:#}");
        vec![]
    });
    let corp_action_filtered: Vec<_> = ca_calendar
        .into_iter()
        .filter(|e| e.symbols.iter().any(|s| holdings_set.contains(s.as_str())))
        .collect();
    let sec_registry =
        std::sync::Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new()));
    let sec_poller = SecFilingsPoller::new(
        fmp.clone(),
        sec_registry,
        SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
    )
    .with_sec_recent_hours(72);
    let mut sec_events = Vec::new();
    for sym in &symbols {
        match sec_poller.fetch(sym).await {
            Ok(v) => sec_events.extend(v),
            Err(e) => println!("SEC 8-K {sym} 失败: {e:#}"),
        }
    }
    println!(
        "CorpAction: calendar={} · 8-K={}",
        corp_action_filtered.len(),
        sec_events.len()
    );

    // 7) 组装待推消息
    let fmt = RenderFormat::TelegramHtml;
    let mut messages: Vec<(bool, String)> = Vec::new();

    // 7a) LLM 生成"今日要点"摘要（可选：无 OPENROUTER_KEY 时跳过）
    if let Ok(openrouter_key) = std::env::var("HONE_OPENROUTER_KEY") {
        use hone_llm::{LlmProvider, Message, OpenRouterProvider};
        let openrouter_model = std::env::var("HONE_OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-haiku-4-5".to_string());
        let provider = OpenRouterProvider::new(&openrouter_key, &openrouter_model, 1024);

        // 只把对 LLM 最有信息量的字段喂进去；压缩到 JSON，避免 prompt 太长
        let payload = serde_json::json!({
            "date": today,
            "market_value": total_value,
            "pnl": total_pnl,
            "top_movers": rows.iter().take(5).map(|r| serde_json::json!({
                "sym": r.sym, "price": r.price, "pct": r.pct, "pnl": r.pnl, "mv": r.mv
            })).collect::<Vec<_>>(),
            "upcoming_earnings": earn_filt.iter().take(5).map(|e| serde_json::json!({
                "sym": e.symbols.first(),
                "date": e.occurred_at.date_naive().to_string(),
                "time": e.payload.get("time"),
            })).collect::<Vec<_>>(),
            "news_samples": news_all.iter().take(6).map(|e| serde_json::json!({
                "sym": e.symbols.first(),
                "title": e.title,
            })).collect::<Vec<_>>(),
        });

        let msgs = vec![
            Message {
                role: "system".into(),
                content: Some(
                    "你是持仓助理。根据输入 JSON 写「今日要点」，规则：\n\
                     1) 最多 3 行，总字数 <= 120；\n\
                     2) 第一行给出浮盈浮亏状态 + 最大涨/跌幅个股；\n\
                     3) 第二行给出本周关键财报（如有）；\n\
                     4) 第三行给出 1 条值得关注的新闻标题，没有就省略；\n\
                     5) 不做投资建议，不加前缀。直接输出正文。"
                        .into(),
                ),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                role: "user".into(),
                content: Some(payload.to_string()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];
        match provider.chat(&msgs, None).await {
            Ok(res) if !res.content.trim().is_empty() => {
                let body = format!("💡 今日要点\n{}", res.content.trim());
                messages.push((false, body));
            }
            Ok(_) => println!("LLM 返回空，跳过摘要"),
            Err(e) => println!("LLM 摘要失败，跳过: {e:#}"),
        }
    }

    // 7b) 盘前快照已经包含涨跌幅，价格异动不单独再列
    let _ = price_events; // 保留 poll 结果的调试打印，不再额外推送
    messages.push((false, snapshot));

    if !earn_filt.is_empty() {
        let today_utc = chrono::Utc::now().date_naive();
        let mut sorted: Vec<&crate::event::MarketEvent> = earn_filt.iter().collect();
        sorted.sort_by_key(|e| e.occurred_at);
        let mut digest_text = format!("📅 持仓未来 14 天财报 · {} 条", sorted.len());
        for ev in &sorted {
            let sym = ev.symbols.first().cloned().unwrap_or_default();
            let date = ev.occurred_at.date_naive();
            let dt = (date - today_utc).num_days();
            let urgency = match dt {
                d if d <= 1 => "🔴 T-1",
                d if d <= 3 => "🟠 T-3",
                d if d <= 7 => "🟡 T-7",
                _ => "⚪ T+",
            };
            // 从 payload 里拿 time(bmo/amc) + eps/rev est（原始 summary 里数字未格式化）
            let time_slot = ev
                .payload
                .get("time")
                .and_then(|v| v.as_str())
                .map(|t| match t.to_lowercase().as_str() {
                    "bmo" => "盘前",
                    "amc" => "盘后",
                    _ => "当日",
                })
                .unwrap_or("");
            let eps_est = ev.payload.get("epsEstimated").and_then(|v| v.as_f64());
            let rev_est = ev.payload.get("revenueEstimated").and_then(|v| v.as_f64());
            let fmt_rev = |r: f64| {
                if r >= 1e9 {
                    format!("${:.1}B", r / 1e9)
                } else if r >= 1e6 {
                    format!("${:.0}M", r / 1e6)
                } else {
                    format!("${r:.0}")
                }
            };
            let est_part = match (eps_est, rev_est) {
                (Some(e), Some(r)) => format!("EPS {e:.2} · Rev {}", fmt_rev(r)),
                (Some(e), None) => format!("EPS {e:.2}"),
                (None, Some(r)) => format!("Rev {}", fmt_rev(r)),
                _ => "".into(),
            };
            digest_text.push_str(&format!(
                "\n• {urgency} ${sym} · {date} {time_slot} · {est_part}"
            ));
        }
        messages.push((true, digest_text));
    } else {
        messages.push((false, "📅 持仓未来 14 天财报 · 无".into()));
    }

    // 新闻：High 逐条推；剩余按持仓 ticker 分组，每只取最近 1 条带锚文本
    let news_high: Vec<_> = news_all
        .iter()
        .filter(|e| matches!(e.severity, crate::event::Severity::High))
        .cloned()
        .collect();
    for ev in news_high.iter().take(5) {
        messages.push((true, crate::renderer::render_immediate(ev, fmt)));
    }

    // 按 ticker 分组最近新闻（只取 Low 剩下的）
    use std::collections::BTreeMap;
    let mut by_ticker: BTreeMap<String, Vec<&crate::event::MarketEvent>> = BTreeMap::new();
    for ev in news_all
        .iter()
        .filter(|e| !matches!(e.severity, crate::event::Severity::High))
    {
        if let Some(sym) = ev.symbols.first() {
            if holdings_set.contains(sym.as_str()) {
                by_ticker.entry(sym.clone()).or_default().push(ev);
            }
        }
    }
    if !by_ticker.is_empty() {
        // 每只 ticker 按时间降序取最近 2 条；整体再按时间排序，Top 10 避免刷屏
        let mut picks: Vec<&crate::event::MarketEvent> = by_ticker
            .values_mut()
            .flat_map(|v| {
                v.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
                v.iter().take(2).copied().collect::<Vec<_>>()
            })
            .collect();
        picks.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        picks.truncate(10);

        let touched_tickers: std::collections::HashSet<&str> = picks
            .iter()
            .filter_map(|e| e.symbols.first().map(|s| s.as_str()))
            .collect();

        // #13 财报窗口标记：对每条 news,看是否同 ticker 有 earnings 事件落在
        // [news - 12h, news + 2d] 窗口内,若有则 🔔 标记——这些是 Router 里
        // `maybe_upgrade_news` 会把 Low 升到 Medium 的那一批,肉眼可验证。
        let earn_by_sym: std::collections::HashMap<&str, &crate::event::MarketEvent> = earn_filt
            .iter()
            .filter_map(|e| e.symbols.first().map(|s| (s.as_str(), e)))
            .collect();
        let in_earnings_window = |ev: &crate::event::MarketEvent| -> Option<i64> {
            let sym = ev.symbols.first()?.as_str();
            let earn = earn_by_sym.get(sym)?;
            let start = ev.occurred_at - chrono::Duration::hours(12);
            let end = ev.occurred_at + chrono::Duration::days(2);
            if earn.occurred_at >= start && earn.occurred_at <= end {
                Some((earn.occurred_at.date_naive() - ev.occurred_at.date_naive()).num_days())
            } else {
                None
            }
        };
        let flagged = picks
            .iter()
            .filter(|e| in_earnings_window(e).is_some())
            .count();

        // 观察用:财报窗口触发的新闻条数 + 未来 14d 内所有持仓财报日 +
        // 每只持仓的 news 条数分布,看命中问题是数据没有还是分组策略挤掉了。
        let mut per_sym: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        for ev in &news_all {
            if let Some(sym) = ev.symbols.first() {
                *per_sym.entry(sym.as_str()).or_default() += 1;
            }
        }
        println!(
            "[#13 earnings-window] flagged={flagged} / picks={} · earnings: {:?} · news_per_sym: {:?}",
            picks.len(),
            earn_by_sym
                .iter()
                .map(|(k, v)| (*k, v.occurred_at.date_naive().to_string()))
                .collect::<Vec<_>>(),
            per_sym,
        );

        let mut digest_text = format!(
            "📰 持仓相关新闻 · {} 只有动静 · Top {}{}",
            touched_tickers.len(),
            picks.len(),
            if flagged > 0 {
                format!(" · 🔔 财报窗口 {flagged}")
            } else {
                String::new()
            }
        );
        for ev in &picks {
            let sym = ev.symbols.first().cloned().unwrap_or_default();
            let ts = ev.occurred_at.format("%m-%d %H:%M").to_string();
            let title_esc = crate::renderer::render_inline(&ev.title, fmt);
            let tag = match in_earnings_window(ev) {
                // d > 0 表示 earnings 在 news 之后 d 天(T-d),d<=0 则 news 已在财报日
                Some(d) if d <= 0 => " <b>🔔T</b>".to_string(),
                Some(d) => format!(" <b>🔔T-{d}</b>"),
                None => String::new(),
            };
            match &ev.url {
                Some(u) => {
                    let host = u
                        .split("://")
                        .nth(1)
                        .and_then(|s| s.split('/').next())
                        .unwrap_or(u);
                    digest_text.push_str(&format!(
                        "\n• ${sym}{tag} · {ts} · {title_esc} <a href=\"{u}\">{host}</a>"
                    ));
                }
                None => {
                    digest_text.push_str(&format!("\n• ${sym}{tag} · {ts} · {title_esc}"));
                }
            }
        }
        messages.push((true, digest_text));
    }

    // SEC 8-K：poller 侧已经按 72h 切过时效;这里直接按时间降序渲染。
    // payload 里无 item/description，把 accepted 时分 + EDGAR index link +
    // finalLink 文档都放出来让用户自己看。
    if !sec_events.is_empty() {
        let mut recent: Vec<&crate::event::MarketEvent> = sec_events.iter().collect();
        recent.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        if !recent.is_empty() {
            let mut digest_text = format!("📄 持仓最近 72h SEC 8-K · {} 条", recent.len());
            for ev in &recent {
                let sym = ev.symbols.first().cloned().unwrap_or_default();
                // payload.acceptedDate 可能是 "YYYY-MM-DD HH:MM:SS"；
                // 优先显示它，退化到 occurred_at
                let accepted = ev
                    .payload
                    .get("acceptedDate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let (stamp, slot_tag) = if !accepted.is_empty() {
                    // 按 NYSE 交易时段标注：9:30–16:00 ET 盘中。
                    // 这里 FMP 给的是 ET 本地时间（未加时区），按本地小时直接判断。
                    let hour = accepted
                        .split_whitespace()
                        .nth(1)
                        .and_then(|t| t.split(':').next())
                        .and_then(|h| h.parse::<u32>().ok())
                        .unwrap_or(0);
                    let tag = match hour {
                        0..=8 => "盘前",
                        9..=15 => "盘中",
                        _ => "盘后",
                    };
                    (accepted.to_string(), tag)
                } else {
                    (ev.occurred_at.format("%Y-%m-%d %H:%M").to_string(), "")
                };
                let index_link = ev
                    .payload
                    .get("link")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let doc_link = ev
                    .payload
                    .get("finalLink")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| ev.url.as_deref().unwrap_or(""));
                // 文档文件名（htm 的最后一段）
                let doc_name = doc_link
                    .rsplit('/')
                    .next()
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        if s.len() > 36 {
                            format!("{}…", &s[..33])
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or_else(|| "document".into());

                // 第一行：ticker · 时间 · 盘前/盘后
                digest_text.push_str(&format!(
                    "\n• ${sym} · {stamp}{}",
                    if slot_tag.is_empty() {
                        String::new()
                    } else {
                        format!(" ({slot_tag})")
                    }
                ));
                // 第二行：两个链接（缩进对齐）
                let mut links: Vec<String> = Vec::new();
                if !index_link.is_empty() {
                    links.push(format!("<a href=\"{index_link}\">EDGAR index</a>"));
                }
                if !doc_link.is_empty() {
                    let name_esc = crate::renderer::render_inline(&doc_name, fmt);
                    links.push(format!("<a href=\"{doc_link}\">{name_esc}</a>"));
                }
                if !links.is_empty() {
                    digest_text.push_str(&format!("\n   ↳ {}", links.join(" · ")));
                }
            }
            messages.push((true, digest_text));
        } else {
            println!("SEC 8-K 过去 72h 无；全持仓都是历史老文件");
        }
    }
    if !corp_action_filtered.is_empty() {
        let digest_text =
            crate::digest::render_digest("持仓拆股/分红", &corp_action_filtered, 0, fmt);
        messages.push((true, digest_text));
    }

    // 8) 真推 Telegram
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    for (use_html, text) in messages {
        let mut payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        if use_html {
            payload["parse_mode"] = serde_json::Value::String("HTML".into());
            payload["disable_web_page_preview"] = serde_json::Value::Bool(true);
        }
        let telegram_response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = telegram_response.status();
        let response_body = telegram_response.text().await.unwrap_or_default();
        println!(
            "[backtest push] html={use_html} status={status} body_len={}",
            response_body.len()
        );
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {response_body}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    }
}

/// DailyReport 落盘端到端验证:塞假事件 + 假 delivery_log,调用
/// `tick_once` 在 22:00 窗口命中,读回 `data/daily_reports/YYYY-MM-DD.md`
/// 肉眼检查内容。不推 Telegram——日报只服务运维视角。
///
/// 触发：
/// `cargo test -p hone-event-engine --lib tests::daily_report_roundtrip \
///   -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn daily_report_roundtrip() {
    use crate::daily_report::DailyReport;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::store::EventStore;
    use chrono::TimeZone;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(EventStore::open(temp_dir.path().join("event-store")).unwrap());
    let report_dir = temp_dir.path().join("reports");

    let now_utc = chrono::Utc::now();
    let seeded_events = vec![
        ("fmp.stock_news", EventKind::NewsCritical, 5),
        ("fmp.earning_calendar", EventKind::EarningsUpcoming, 2),
        (
            "fmp.sec_filings",
            EventKind::SecFiling { form: "8-K".into() },
            1,
        ),
        ("fmp.stock_split_calendar", EventKind::Split, 1),
        ("fmp.upgrades_downgrades", EventKind::AnalystGrade, 1),
    ];
    let mut event_idx = 0;
    for (src, kind, event_count) in seeded_events {
        for _ in 0..event_count {
            let ev = MarketEvent {
                id: format!("fake-{event_idx}"),
                kind: kind.clone(),
                severity: Severity::Medium,
                symbols: vec!["AAPL".into()],
                occurred_at: now_utc,
                title: "fake".into(),
                summary: String::new(),
                url: None,
                source: src.into(),
                payload: serde_json::Value::Null,
            };
            store.insert_event(&ev).unwrap();
            event_idx += 1;
        }
    }
    let primary_actor_key = "telegram::::8039067465";
    for _ in 0..3 {
        store
            .log_delivery(
                "f-s",
                primary_actor_key,
                "sink",
                Severity::High,
                "sent",
                None,
            )
            .unwrap();
    }
    for _ in 0..8 {
        store
            .log_delivery(
                "f-q",
                primary_actor_key,
                "digest",
                Severity::Medium,
                "queued",
                None,
            )
            .unwrap();
    }
    for _ in 0..2 {
        store
            .log_delivery(
                "f-f",
                primary_actor_key,
                "prefs",
                Severity::Low,
                "filtered",
                None,
            )
            .unwrap();
    }
    store
        .log_delivery(
            "f-o",
            "feishu::::ghost",
            "sink",
            Severity::High,
            "sent",
            None,
        )
        .unwrap();

    // 人工构造"恰好在 22:00 本地"的 now:取北京 tz,today 的 22:00。
    let tz_offset = 8_i32;
    let local_today = now_utc
        .with_timezone(&chrono::FixedOffset::east_opt(tz_offset * 3600).unwrap())
        .date_naive();
    let local_trigger = local_today.and_hms_opt(22, 0, 0).unwrap();
    let trigger_utc = chrono::FixedOffset::east_opt(tz_offset * 3600)
        .unwrap()
        .from_local_datetime(&local_trigger)
        .unwrap()
        .with_timezone(&chrono::Utc);

    let report = DailyReport::new(store.clone(), &report_dir)
        .with_tz_offset_hours(tz_offset)
        .with_trigger_time("22:00");
    let mut fired = std::collections::HashSet::new();
    let generated_report_count = report.tick_once(trigger_utc, &mut fired).await.unwrap();
    assert_eq!(generated_report_count, 1);

    let date_str = local_today.format("%Y-%m-%d").to_string();
    let report_path = report_dir.join(format!("{date_str}.md"));
    let body = std::fs::read_to_string(&report_path).expect("日报文件未生成");
    println!("\n=== daily_report {date_str}.md ===\n{body}");
    assert!(body.contains("# Hone 日报 · "));
    assert!(body.contains("合计 **10** 条"));
    // 两个 actor 行都在
    assert!(body.contains(&format!("| `{primary_actor_key}` |")));
    assert!(body.contains("| `feishu::::ghost` |"));
}

/// 真实 E2E:启动 engine → TelegramChannelPoller (冷启动立即拉一次
/// `https://t.me/s/watcherguru`) → EventStore + events.jsonl 镜像。
/// 不依赖 FMP key、不依赖 hone-cli orchestration,直接验证社交链路通。
///
/// 触发:
/// `cargo test -p hone-event-engine --lib tests::live_social_engine_e2e \
///   -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn live_social_engine_e2e() {
    use hone_core::ActorIdentity;
    use hone_core::config::event_engine::Sources;
    use hone_core::config::{FmpConfig, TelegramChannelConfig};
    use hone_memory::PortfolioStorage;
    use hone_memory::portfolio::{Holding, Portfolio};

    let temp_dir = tempfile::tempdir().unwrap();
    let store_path = temp_dir.path().join("event-store");
    let jsonl_path = temp_dir.path().join("events.jsonl");
    let portfolio_dir = temp_dir.path().join("portfolio");
    let digest_dir = temp_dir.path().join("digest");
    let prefs_dir = temp_dir.path().join("prefs");
    let daily_report_dir = temp_dir.path().join("daily_reports");
    std::fs::create_dir_all(&portfolio_dir).unwrap();

    // seed 一个 direct-actor 持仓,让 social_global GlobalSub 有 fanout 目标
    let storage = PortfolioStorage::new(&portfolio_dir);
    let actor = ActorIdentity::new("telegram", "e2e-user", None::<String>).unwrap();
    let portfolio = Portfolio {
        actor: Some(actor.clone()),
        user_id: "e2e-user".into(),
        holdings: vec![Holding {
            symbol: "AAPL".into(),
            asset_type: "stock".into(),
            shares: 1.0,
            avg_cost: 100.0,
            underlying: None,
            option_type: None,
            strike_price: None,
            expiration_date: None,
            contract_multiplier: None,
            holding_horizon: None,
            strategy_notes: None,
            notes: None,
            weight: None,
            name: None,
            tracking_only: None,
        }],
        updated_at: "2026-04-22".into(),
    };
    storage.save(&actor, &portfolio).unwrap();

    // 关掉所有 FMP poller,只开社交
    let mut engine_cfg = EventEngineConfig::default();
    engine_cfg.enabled = true;
    engine_cfg.sources = Sources {
        news: false,
        price: false,
        extended_hours: false,
        earnings_calendar: false,
        corp_action: false,
        sec_filings: false,
        macro_calendar: false,
        analyst_grade: false,
        earnings_surprise: false,
        telegram_channels: vec![TelegramChannelConfig {
            handle: "watcherguru".into(),
            interval_secs: 1800,
            extract_cashtags: true,
        }],
        rss_feeds: Vec::new(),
    };

    let engine = EventEngine::new(engine_cfg, FmpConfig::default())
        .with_store_path(store_path.clone())
        .with_events_jsonl_path(Some(jsonl_path.clone()))
        .with_portfolio_dir(portfolio_dir)
        .with_digest_dir(digest_dir)
        .with_prefs_dir(prefs_dir)
        .with_daily_report_dir(daily_report_dir)
        .with_retention_days(0);
    engine.start().await.unwrap();

    // 冷启动立即拉一次 → 等 HTTP + HTML 解析 + store 写入。
    // 正常情况下 5-10s 够了,给 20s 容 CI 慢网。
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    let store = EventStore::open(&store_path).unwrap();
    let stored_event_count = store.count_events().unwrap();
    let jsonl = std::fs::read_to_string(&jsonl_path).unwrap_or_default();
    let tg_lines: Vec<&str> = jsonl
        .lines()
        .filter(|l| l.contains("\"telegram.watcherguru\""))
        .collect();

    println!("=== live_social_engine_e2e ===");
    println!("count_events = {stored_event_count}");
    println!("telegram.watcherguru 事件数 = {}", tg_lines.len());
    if let Some(first) = tg_lines.first() {
        println!("第一条:{first}");
    }

    assert!(
        stored_event_count > 0,
        "PostgreSQL events 应有至少 1 条事件"
    );
    assert!(
        !tg_lines.is_empty(),
        "应至少有 1 条 source=telegram.watcherguru 事件(若 Telegram 改版或网络问题请另查)"
    );
    assert!(
        tg_lines
            .iter()
            .any(|l| l.contains("\"type\":\"social_post\"")),
        "社交事件 kind 应为 social_post"
    );
    assert!(
        tg_lines
            .iter()
            .any(|l| l.contains("\"source_class\":\"uncertain\"")),
        "payload 应带 source_class=uncertain(LLM 仲裁开关)"
    );
}

/// 离线回放审计(验收工具,手动运行):把真实 `events.jsonl` 重放过
/// 2026-08-15 加的两道防线,量化修复效果。
///
/// ```bash
/// HONE_REPLAY_EVENTS_JSONL=$PWD/data/events.jsonl \
///   cargo test -p hone-event-engine --lib replay_push_quality_audit -- --ignored --nocapture
/// ```
///
/// 验收断言:
/// 1. 评级:重放后不再产出任何来自「多股汇总文」的 High 事件;
///    真实单公司 High(如 GEV conviction list)保留。
/// 2. 价格:每个 (symbol, 交易日) 的 band 阶梯坍缩后只剩 1 行。
#[test]
#[ignore]
fn replay_push_quality_audit() {
    use crate::digest::coalesce::coalesce_price_alerts;
    use crate::pollers::analyst_grade::events_from_grades;
    use std::collections::HashMap;

    let path = match std::env::var("HONE_REPLAY_EVENTS_JSONL") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("HONE_REPLAY_EVENTS_JSONL 未设置,跳过");
            return;
        }
    };
    let raw = std::fs::read_to_string(&path).expect("读取 events.jsonl 失败");
    let mut grade_rows_by_symbol: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut original_grade_high = 0usize;
    let mut price_by_day: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    let mut band_lines_before = 0usize;
    for line in raw.lines() {
        let Ok(event) = serde_json::from_str::<MarketEvent>(line) else {
            continue;
        };
        match &event.kind {
            EventKind::AnalystGrade => {
                let Some(symbol) = event.symbols.first() else {
                    continue;
                };
                if event.severity == Severity::High {
                    original_grade_high += 1;
                }
                grade_rows_by_symbol
                    .entry(symbol.clone())
                    .or_default()
                    .push(event.payload.clone());
            }
            EventKind::PriceAlert { .. } => {
                if event.id.starts_with("price_band:") {
                    band_lines_before += 1;
                }
                let day = event.occurred_at.date_naive().to_string();
                price_by_day.entry(day).or_default().push(event);
            }
            _ => {}
        }
    }

    // ── 评级防线回放 ────────────────────────────────────────────────
    let cutoff = chrono::DateTime::<chrono::Utc>::MIN_UTC;
    let mut replay_high = 0usize;
    let mut replay_roundup_high = 0usize;
    let mut roundup_summaries = 0usize;
    for (symbol, rows) in &grade_rows_by_symbol {
        let raw_rows = serde_json::Value::Array(rows.clone());
        let events = events_from_grades(&raw_rows, symbol, cutoff);
        for event in &events {
            if crate::event::is_analyst_roundup_summary(event) {
                roundup_summaries += 1;
                assert_ne!(
                    event.severity,
                    Severity::High,
                    "汇总事件不允许 High: {}",
                    event.id
                );
            }
            if event.severity == Severity::High {
                replay_high += 1;
                let title = event
                    .payload
                    .get("newsTitle")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let lower = title.to_ascii_lowercase();
                if lower.contains("top analyst calls")
                    || lower.contains("stock calls")
                    || lower.contains("buy/sell:")
                {
                    replay_roundup_high += 1;
                    eprintln!("残留污染 High: {} | {}", event.id, title);
                }
            }
        }
    }
    eprintln!(
        "[评级] 原始 High 事件: {original_grade_high} → 重放后 High: {replay_high}(其中汇总文残留 {replay_roundup_high}),汇总摘要事件: {roundup_summaries}"
    );
    assert_eq!(replay_roundup_high, 0, "汇总文 High 必须清零");
    assert!(replay_high > 0, "真实单公司 High 不应被误杀(如 GEV)");
    assert!(roundup_summaries > 0, "MU/GOOGL 案例应产出汇总摘要");

    // ── 价格阶梯合流回放 ────────────────────────────────────────────
    let mut lines_after = 0usize;
    let mut close_annotated = 0usize;
    let mut per_group_max = 0usize;
    for (_day, events) in price_by_day {
        let result = coalesce_price_alerts(events);
        for kept in &result.kept {
            if kept.id.starts_with("price_band:") || kept.id.starts_with("price_close:") {
                lines_after += 1;
            }
            if kept.title.contains("盘中曾跨") {
                close_annotated += 1;
            }
        }
        // 合流后不允许同 (symbol, 日, 方向) 出现两条 band。
        let mut seen: HashMap<String, usize> = HashMap::new();
        for kept in &result.kept {
            if let Some(rest) = kept.id.strip_prefix("price_band:") {
                let key: Vec<&str> = rest.split(':').collect();
                if key.len() >= 4 {
                    let group = format!("{}:{}:{}", key[0], key[1], key[2]);
                    let count = seen.entry(group.clone()).or_default();
                    *count += 1;
                    per_group_max = per_group_max.max(*count);
                    assert_eq!(*count, 1, "阶梯残留: {group}");
                }
            }
        }
    }
    eprintln!(
        "[价格] band 行: {band_lines_before} → 合流后 band+close 行: {lines_after},收盘注记行: {close_annotated}"
    );
}

/// σ-自适应价格阈值回归(特征固定测试,验收标准 A1–A7)。
///
/// 数据集:`testdata/daily_closes_2026-01-02_2026-08-14.json` —— 18 个 watch-pool
/// 标的的真实 FMP 日线;评估窗口 2026-04-27 → 2026-08-14(真实推送期 76 个交易日),
/// 更早数据仅做 σ 热身。σ 逐日无前视(只用 t 日之前的收盘价),走生产函数
/// `sigma_pct_from_closes` + `effective_thresholds`。
/// 设计与验收标准:`docs/proposals/sigma-adaptive-price-thresholds.md`。
#[test]
fn sigma_adaptive_thresholds_regression() {
    use crate::volatility::{effective_thresholds, sigma_pct_from_closes};
    use hone_core::config::PriceSigmaThresholds;

    const EVAL_START: &str = "2026-04-27";
    const BASE_LOW: f64 = 2.5;
    const BASE_HIGH: f64 = 6.0;

    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/daily_closes_2026-01-02_2026-08-14.json"
    ))
    .expect("读取回归数据集失败");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("回归数据集非法 JSON");
    let closes_by_symbol = fixture
        .get("closes")
        .and_then(|v| v.as_object())
        .expect("缺 closes");
    assert_eq!(closes_by_symbol.len(), 18, "watch-pool 标的数");

    let cfg = PriceSigmaThresholds::default();
    let (mut fixed_alerts, mut fixed_high, mut sigma_alerts, mut sigma_high) =
        (0u32, 0u32, 0u32, 0u32);
    let mut rates_fixed = Vec::new();
    let mut rates_sigma = Vec::new();
    let mut sndk_rates = None;
    let mut max_suppressed_z: f64 = 0.0;
    let mut missed_extreme = 0u32;

    for (symbol, rows) in closes_by_symbol {
        let rows = rows.as_array().expect("closes 行");
        let dates: Vec<&str> = rows.iter().map(|r| r[0].as_str().unwrap()).collect();
        let closes: Vec<f64> = rows.iter().map(|r| r[1].as_f64().unwrap()).collect();
        let (mut fa, mut sa, mut days) = (0u32, 0u32, 0u32);
        for k in 1..closes.len() {
            if dates[k] < EVAL_START {
                continue;
            }
            days += 1;
            let ret = (closes[k] - closes[k - 1]) / closes[k - 1] * 100.0;
            let abs = ret.abs();
            // σ 窗口:t 日之前最多 60 个 close-to-close 收益率(61 个收盘价),无前视。
            let window = &closes[k.saturating_sub(61)..k];
            let sigma = sigma_pct_from_closes(window, cfg.min_samples as usize);
            let (low_eff, high_eff) = effective_thresholds(&cfg, sigma, BASE_LOW, BASE_HIGH);

            let hit_fixed = abs >= BASE_LOW;
            let hit_sigma = abs >= low_eff;
            fa += u32::from(hit_fixed);
            fixed_alerts += u32::from(hit_fixed);
            fixed_high += u32::from(abs >= BASE_HIGH);
            sa += u32::from(hit_sigma);
            sigma_alerts += u32::from(hit_sigma);
            sigma_high += u32::from(abs >= high_eff);
            if let Some(sigma) = sigma {
                let z = abs / sigma;
                if hit_fixed && !hit_sigma {
                    max_suppressed_z = max_suppressed_z.max(z);
                }
                if (z >= 3.0 || abs >= 10.0) && !hit_sigma {
                    missed_extreme += 1;
                }
            }
        }
        assert_eq!(days, 77, "{symbol} 评估窗口交易日数");
        rates_fixed.push(f64::from(fa) / f64::from(days));
        rates_sigma.push(f64::from(sa) / f64::from(days));
        if symbol == "SNDK" {
            sndk_rates = Some((
                f64::from(fa) / f64::from(days),
                f64::from(sa) / f64::from(days),
            ));
        }
    }

    // 特征固定:与设计期模拟(proposal 文档)逐一相等,防未来悄悄漂移。
    assert_eq!(
        (fixed_alerts, fixed_high, sigma_alerts, sigma_high),
        (841, 380, 255, 90),
        "警报/High 总数偏离设计模拟"
    );
    // A1: 警报总数降幅 ≥ 60%
    assert!(f64::from(sigma_alerts) <= f64::from(fixed_alerts) * 0.40);
    // A2: High 总数降幅 ≥ 70%
    assert!(f64::from(sigma_high) <= f64::from(fixed_high) * 0.30);
    // A3: 零漏报(≥3σ 或 ≥10% 的极端日全部保留)
    assert_eq!(missed_extreme, 0, "存在被静音的极端日");
    // A4: 被抑制警报统计上寻常(z < 2.0)
    assert!(
        max_suppressed_z < 2.0,
        "max_suppressed_z={max_suppressed_z}"
    );
    // A5: 跨标的警报率标准差严格下降
    let stdev = |xs: &[f64]| {
        let n = xs.len() as f64;
        let mean = xs.iter().sum::<f64>() / n;
        (xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
    };
    assert!(stdev(&rates_sigma) < stdev(&rates_fixed));
    // A6: 最吵标的警报率 ≤ 0.45(固定阈值下为 0.82)
    let max_rate = rates_sigma.iter().cloned().fold(0.0f64, f64::max);
    assert!(max_rate <= 0.45, "max_rate={max_rate}");
    // A7: SNDK(原始投诉标的)警报率 ≤ 0.35
    let (sndk_fixed, sndk_sigma) = sndk_rates.expect("SNDK 不在数据集");
    assert!(
        sndk_fixed > 0.80,
        "基线偏离:SNDK 固定阈值警报率 {sndk_fixed}"
    );
    assert!(sndk_sigma <= 0.35, "SNDK σ 警报率 {sndk_sigma}");
}

/// 两周事件重放 → 新管线 → Discord 实弹推送(2026-08-15 用户要求的演示)。
///
/// 把过去 N 天的真实事件按新管线重建:评级重跑汇总文坍缩 + 目标价锚点,
/// 价格按 σ-自适应阈值(fixture 日线逐日计算,无前视)重新过滤定级,再按
/// 事件当日口径注入持仓行 / 30 日共识 / 财报倒计时 / 主线关联,按时间顺序
/// 推到目标 Discord 用户。store namespace 会映射到当前连接的 `pg_temp`
/// schema，不读写生产表。
///
/// ```bash
/// HONE_REPLAY_EVENTS_JSONL=$PWD/data/events.jsonl \
/// HONE_REPLAY_STORE=replay-event-store \
/// HONE_REPLAY_PREFS_DIR=$PWD/data/notif_prefs \
/// HONE_REPLAY_PORTFOLIO_DIR=$PWD/data/portfolio \
/// HONE_REPLAY_OUT=/tmp/replay_dryrun.md \
/// HONE_REPLAY_SEND=0 HONE_REPLAY_TARGET_USER=... HONE_REPLAY_DISCORD_TOKEN=... \
///   cargo test -p hone-event-engine --lib replay_two_weeks_and_push -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore]
async fn replay_two_weeks_and_push() {
    use crate::digest::coalesce::coalesce_price_alerts;
    use crate::pollers::analyst_grade::events_from_grades;
    use crate::pollers::price::events_from_quotes_at;
    use crate::prefs::{FilePrefsStorage, PrefsProvider};
    use crate::renderer::{RenderFormat, render_immediate};
    use crate::router::OutboundSink;
    use crate::router::mainline_links::mainline_cross_links;
    use crate::router::position::position_annotated_event;
    use crate::sinks::DiscordSink;
    use crate::store::EventStore;
    use crate::subscription::SharedRegistry;
    use crate::volatility::{SymbolThresholds, effective_thresholds, sigma_pct_from_closes};
    use chrono::{Duration, Utc};
    use hone_core::ActorIdentity;
    use hone_core::config::PriceSigmaThresholds;
    use std::collections::{BTreeMap, HashMap, HashSet};

    let env = |k: &str| std::env::var(k).ok();
    let (Some(jsonl), Some(store_namespace)) =
        (env("HONE_REPLAY_EVENTS_JSONL"), env("HONE_REPLAY_STORE"))
    else {
        eprintln!("HONE_REPLAY_EVENTS_JSONL / HONE_REPLAY_STORE 未设置,跳过");
        return;
    };
    assert!(
        !store_namespace.trim().is_empty(),
        "HONE_REPLAY_STORE 不能为空"
    );
    let prefs_dir = env("HONE_REPLAY_PREFS_DIR").expect("HONE_REPLAY_PREFS_DIR");
    let portfolio_dir = env("HONE_REPLAY_PORTFOLIO_DIR").expect("HONE_REPLAY_PORTFOLIO_DIR");
    let days: i64 = env("HONE_REPLAY_DAYS")
        .and_then(|v| v.parse().ok())
        .unwrap_or(14);
    let target_user = env("HONE_REPLAY_TARGET_USER").unwrap_or_default();
    let do_send = env("HONE_REPLAY_SEND").as_deref() == Some("1");

    let now = Utc::now();
    let cutoff = now - Duration::days(days);
    let store = EventStore::open(&store_namespace).expect("open isolated PostgreSQL store");
    let registry = SharedRegistry::from_portfolio_dir(&portfolio_dir);
    let registry = registry.load();
    let actor = ActorIdentity::new("discord", target_user.as_str(), None::<&str>)
        .unwrap_or_else(|_| ActorIdentity::new("discord", "dryrun", None::<&str>).unwrap());
    let prefs = FilePrefsStorage::new(&prefs_dir)
        .expect("prefs dir")
        .load(&actor);

    // σ 表:committed fixture(2026-01-02..2026-08-14 日线)
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/daily_closes_2026-01-02_2026-08-14.json"
        ))
        .expect("fixture"),
    )
    .unwrap();
    let closes_by_symbol: HashMap<String, Vec<(String, f64)>> = fixture["closes"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(sym, rows)| {
            (
                sym.clone(),
                rows.as_array()
                    .unwrap()
                    .iter()
                    .map(|r| (r[0].as_str().unwrap().to_string(), r[1].as_f64().unwrap()))
                    .collect(),
            )
        })
        .collect();
    let sigma_cfg = PriceSigmaThresholds::default();
    let thresholds_for = |sym: &str, date: &str| -> (f64, f64, Option<f64>) {
        let sigma = closes_by_symbol.get(sym).and_then(|rows| {
            let closes: Vec<f64> = rows
                .iter()
                .filter(|(d, _)| d.as_str() < date)
                .map(|(_, c)| *c)
                .collect();
            let tail = &closes[closes.len().saturating_sub(61)..];
            sigma_pct_from_closes(tail, sigma_cfg.min_samples as usize)
        });
        let (low, high) = effective_thresholds(&sigma_cfg, sigma, 2.5, 6.0);
        (low, high, sigma)
    };

    // ── 1. 读 events.jsonl,分池 ─────────────────────────────────
    let mut grade_rows: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut grade_row_seen: HashSet<String> = HashSet::new();
    let mut price_pool: Vec<MarketEvent> = Vec::new();
    let mut passthrough: Vec<MarketEvent> = Vec::new(); // extended / 52w / earnings_released
    let mut raw_counts: HashMap<&'static str, usize> = HashMap::new();
    for line in std::fs::read_to_string(&jsonl).expect("read jsonl").lines() {
        let Ok(event) = serde_json::from_str::<MarketEvent>(line) else {
            continue;
        };
        if event.occurred_at < cutoff || event.occurred_at > now {
            continue;
        }
        let id = event.id.clone();
        if id.starts_with("grade:") {
            *raw_counts.entry("grade_raw").or_default() += 1;
            let Some(symbol) = event.symbols.first() else {
                continue;
            };
            let key = format!(
                "{}|{}",
                event
                    .payload
                    .get("publishedDate")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                event
                    .payload
                    .get("gradingCompany")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            );
            if grade_row_seen.insert(format!("{symbol}|{key}")) {
                grade_rows
                    .entry(symbol.clone())
                    .or_default()
                    .push(event.payload.clone());
            }
        } else if id.starts_with("grade_roundup:") {
            *raw_counts.entry("grade_roundup_stored").or_default() += 1;
            let Some(symbol) = event.symbols.first() else {
                continue;
            };
            if let Some(rows) = event.payload.get("rows").and_then(|v| v.as_array()) {
                for row in rows {
                    let key = format!(
                        "{}|{}",
                        row.get("publishedDate")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        row.get("gradingCompany")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                    );
                    if grade_row_seen.insert(format!("{symbol}|{key}")) {
                        grade_rows
                            .entry(symbol.clone())
                            .or_default()
                            .push(row.clone());
                    }
                }
            }
        } else if id.starts_with("price_low:")
            || id.starts_with("price_band:")
            || id.starts_with("price_close:")
            || id.starts_with("price:")
        {
            *raw_counts.entry("price_raw").or_default() += 1;
            price_pool.push(event);
        } else if id.starts_with("extended:") {
            *raw_counts.entry("extended_raw").or_default() += 1;
            passthrough.push(event);
        } else if id.starts_with("52h:") || id.starts_with("52l:") {
            *raw_counts.entry("week52_raw").or_default() += 1;
            passthrough.push(event);
        } else if matches!(event.kind, EventKind::EarningsReleased) {
            *raw_counts.entry("earnings_released").or_default() += 1;
            passthrough.push(event);
        }
    }

    // ── 2. 评级重建(汇总文坍缩 + 目标价锚点) ─────────────────
    let mut timeline: Vec<MarketEvent> = Vec::new();
    for (symbol, rows) in &grade_rows {
        let raw = serde_json::Value::Array(rows.clone());
        timeline.extend(events_from_grades(&raw, symbol, cutoff));
    }
    let rebuilt_grades = timeline.len();

    // ── 3. 价格重建(σ-自适应过滤 + 重定级) ───────────────────
    let mut seen_price_ids: HashSet<String> = HashSet::new();
    let mut sigma_suppressed = 0usize;
    for event in price_pool {
        let Some(symbol) = event.symbols.first().cloned() else {
            continue;
        };
        let date_key = event.occurred_at.date_naive().to_string();
        let (low_eff, high_eff, sigma) = thresholds_for(&symbol, &date_key);
        let price = event.payload.get("hone_price").and_then(|v| v.as_f64());
        let Some(pct) = event.payload.get("hone_price_pct").and_then(|v| v.as_f64()) else {
            continue;
        };
        let quote = serde_json::json!([{
            "symbol": symbol,
            "price": price,
            "changesPercentage": pct,
            "timestamp": event.occurred_at.timestamp(),
        }]);
        let mut per_symbol = HashMap::new();
        per_symbol.insert(
            symbol.clone(),
            SymbolThresholds {
                low_pct: low_eff,
                high_pct: high_eff,
                sigma_pct: sigma,
            },
        );
        let rebuilt = events_from_quotes_at(
            &quote,
            2.5,
            6.0,
            2.0,
            0.001,
            &per_symbol,
            event.occurred_at + Duration::seconds(2),
        );
        if rebuilt.is_empty() {
            sigma_suppressed += 1;
            continue;
        }
        for rebuilt_event in rebuilt {
            if seen_price_ids.insert(rebuilt_event.id.clone()) {
                timeline.push(rebuilt_event);
            }
        }
    }

    // ── 4. extended / 52w / earnings 透传(extended 按 σ 过滤) ──
    for mut event in passthrough {
        if event.id.starts_with("extended:") {
            let Some(symbol) = event.symbols.first().cloned() else {
                continue;
            };
            let date_key = event.occurred_at.date_naive().to_string();
            let (low_eff, high_eff, _) = thresholds_for(&symbol, &date_key);
            let amp = event
                .payload
                .get("changesPercentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                .abs();
            if amp < low_eff {
                sigma_suppressed += 1;
                continue;
            }
            event.severity = if amp >= high_eff {
                Severity::High
            } else {
                Severity::Low
            };
        }
        timeline.push(event);
    }
    timeline.sort_by_key(|e| e.occurred_at);

    // ── 5. 路由决策(用户策略:±8% 直推 / min_severity=medium)+ 注入 ──
    #[derive(PartialEq)]
    enum Route {
        Immediate,
        Digest,
    }
    let route_for = |event: &MarketEvent| -> Option<Route> {
        match &event.kind {
            EventKind::PriceAlert { window, .. } => {
                // 生产规则:price_close_direct_enabled=false → 收盘事件只进摘要
                if window == "close" {
                    return Some(Route::Digest);
                }
                let pct = event
                    .payload
                    .get("hone_price_pct")
                    .or_else(|| event.payload.get("changesPercentage"))
                    .and_then(|v| v.as_f64())?;
                if pct.abs() >= 8.0 {
                    Some(Route::Immediate)
                } else if !matches!(event.severity, Severity::Low) {
                    Some(Route::Digest)
                } else {
                    None
                }
            }
            EventKind::AnalystGrade => match event.severity {
                Severity::High => Some(Route::Immediate),
                Severity::Medium => Some(Route::Digest),
                Severity::Low => None,
            },
            EventKind::EarningsReleased => match event.severity {
                Severity::High => Some(Route::Immediate),
                _ => Some(Route::Digest),
            },
            EventKind::Weekly52High | EventKind::Weekly52Low => Some(Route::Digest),
            _ => None,
        }
    };

    let annotate = |mut event: MarketEvent| -> MarketEvent {
        let occurred = event.occurred_at;
        let Some(symbol) = event.symbols.first().cloned() else {
            return event;
        };
        if matches!(event.kind, EventKind::AnalystGrade)
            && let Ok(payloads) = store.list_analyst_grade_payloads_in_window(
                &symbol,
                occurred - Duration::days(30),
                occurred,
            )
        {
            let counts = crate::pollers::analyst_grade::consensus_counts_from_payloads(&payloads);
            if counts.total() > 0
                && let Some(obj) = event.payload.as_object_mut()
            {
                obj.insert(
                    "hone_analyst_consensus_30d".into(),
                    serde_json::json!({
                        "down": counts.down, "up": counts.up,
                        "initiated": counts.init, "reiterated": counts.reiter,
                    }),
                );
            }
        }
        if matches!(
            event.kind,
            EventKind::PriceAlert { .. } | EventKind::Weekly52High | EventKind::Weekly52Low
        ) && let Ok(Some(at)) = store.next_upcoming_earnings_for_symbol(&symbol, occurred, 14)
        {
            let days_to = (at.date_naive() - occurred.date_naive()).num_days().max(0);
            if let Some(obj) = event.payload.as_object_mut() {
                obj.insert(
                    "hone_next_earnings_date".into(),
                    serde_json::json!(at.date_naive().to_string()),
                );
                obj.insert("hone_days_to_earnings".into(), serde_json::json!(days_to));
            }
        }
        if let Some(position) = registry.position_for(&actor, &symbol)
            && let Some(annotated) = position_annotated_event(&event, position)
        {
            event = annotated;
        }
        if let Some(mainlines) = prefs.mainline_by_ticker.as_ref() {
            let links = mainline_cross_links(&symbol, mainlines);
            if !links.is_empty()
                && let Some(obj) = event.payload.as_object_mut()
            {
                let links_json: Vec<serde_json::Value> = links
                    .iter()
                    .map(|l| serde_json::json!({"ticker": l.ticker, "excerpt": l.excerpt}))
                    .collect();
                obj.insert("hone_mainline_links".into(), serde_json::json!(links_json));
            }
        }
        event
    };

    // ── 6. 组装消息:即时逐条,摘要按北京日合并 ──────────────
    let local_day = |e: &MarketEvent| (e.occurred_at + Duration::hours(8)).date_naive();
    let mut immediates: Vec<(chrono::NaiveDate, chrono::DateTime<Utc>, String)> = Vec::new();
    let mut digest_by_day: BTreeMap<chrono::NaiveDate, Vec<MarketEvent>> = BTreeMap::new();
    let (mut n_imm, mut n_dig) = (0usize, 0usize);
    // 生产防护:同标的 60min 冷却 + 每日 High 上限 8(超出降入摘要)
    let mut last_immediate_at: HashMap<String, chrono::DateTime<Utc>> = HashMap::new();
    let mut daily_high_count: HashMap<chrono::NaiveDate, u32> = HashMap::new();
    let mut cooled_or_capped = 0usize;
    for event in timeline {
        match route_for(&event) {
            Some(Route::Immediate) => {
                let symbol = event.symbols.first().cloned().unwrap_or_default();
                let day = local_day(&event);
                let cooled = last_immediate_at
                    .get(&symbol)
                    .is_some_and(|at| event.occurred_at - *at < Duration::minutes(60));
                let capped = *daily_high_count.get(&day).unwrap_or(&0) >= 8;
                if cooled || capped {
                    cooled_or_capped += 1;
                    let annotated = annotate(event);
                    digest_by_day.entry(day).or_default().push(annotated);
                    n_dig += 1;
                    continue;
                }
                last_immediate_at.insert(symbol, event.occurred_at);
                *daily_high_count.entry(day).or_default() += 1;
                let annotated = annotate(event);
                let stamp = (annotated.occurred_at + Duration::hours(8)).format("%m-%d %H:%M");
                let body = render_immediate(&annotated, RenderFormat::DiscordMarkdown);
                immediates.push((
                    local_day(&annotated),
                    annotated.occurred_at,
                    format!("🔁 回放 {stamp}(北京)\n{body}"),
                ));
                n_imm += 1;
            }
            Some(Route::Digest) => {
                let annotated = annotate(event);
                digest_by_day
                    .entry(local_day(&annotated))
                    .or_default()
                    .push(annotated);
                n_dig += 1;
            }
            None => {}
        }
    }

    eprintln!("[replay] 冷却/日上限降级 {cooled_or_capped} 条");
    let mut digest_messages: BTreeMap<chrono::NaiveDate, String> = BTreeMap::new();
    let mut coalesced_out = 0usize;
    for (day, events) in digest_by_day {
        let total = events.len();
        let curated = coalesce_price_alerts(events);
        coalesced_out += total - curated.kept.len();
        let mut lines: Vec<String> = Vec::new();
        for event in &curated.kept {
            let summary_head: String = event
                .summary
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(70)
                .collect();
            let sep = if summary_head.is_empty() { "" } else { " — " };
            lines.push(format!("· {}{sep}{summary_head}", event.title));
        }
        let mut msg = format!("🗞 回放摘要 · {day}\n{}", lines.join("\n"));
        if msg.chars().count() > 1700 {
            msg = msg.chars().take(1700).collect::<String>() + "…(截断)";
        }
        digest_messages.insert(day, msg);
    }

    // 按日交错:每天先即时(按时刻),后摘要
    immediates.sort_by_key(|(day, at, _)| (*day, *at));
    let mut ordered: Vec<String> = Vec::new();
    let mut all_days: Vec<chrono::NaiveDate> = immediates
        .iter()
        .map(|(d, _, _)| *d)
        .chain(digest_messages.keys().copied())
        .collect();
    all_days.sort();
    all_days.dedup();
    for day in &all_days {
        let mut first_of_day = true;
        for (imm_day, _, body) in &immediates {
            if imm_day == day {
                let header = if first_of_day {
                    format!("── 📅 {day} ──\n")
                } else {
                    String::new()
                };
                first_of_day = false;
                ordered.push(format!("{header}{body}"));
            }
        }
        if let Some(digest_msg) = digest_messages.get(day) {
            let header = if first_of_day {
                format!("── 📅 {day} ──\n")
            } else {
                String::new()
            };
            ordered.push(format!("{header}{digest_msg}"));
        }
    }

    let stats = format!(
        "🔁 回放统计:窗口 {} 天 | 原始:价格 {} + 盘后 {} + 评级 {}(含 {} 条汇总)+ 52周 {} + 财报 {}\n新管线:σ 静音 {} 条价格事件,评级重建为 {} 条,摘要阶梯再合流 {} 行\n最终:即时推送 {} 条,日摘要 {} 天,消息总数 {}",
        days,
        raw_counts.get("price_raw").unwrap_or(&0),
        raw_counts.get("extended_raw").unwrap_or(&0),
        raw_counts.get("grade_raw").unwrap_or(&0),
        raw_counts.get("grade_roundup_stored").unwrap_or(&0),
        raw_counts.get("week52_raw").unwrap_or(&0),
        raw_counts.get("earnings_released").unwrap_or(&0),
        sigma_suppressed,
        rebuilt_grades,
        coalesced_out,
        n_imm,
        digest_messages.len(),
        ordered.len() + 1,
    );

    if let Some(out) = env("HONE_REPLAY_OUT") {
        let dump = format!("{stats}\n\n====\n\n{}", ordered.join("\n\n====\n\n"));
        std::fs::write(&out, &dump).unwrap();
        eprintln!("{stats}\ndry-run 输出 → {out}");
    }

    if do_send {
        let token = env("HONE_REPLAY_DISCORD_TOKEN").expect("HONE_REPLAY_DISCORD_TOKEN");
        let sink = DiscordSink::new(token);
        sink.send(&actor, &format!("{stats}\n\n👇 以下按时间顺序重放"))
            .await
            .expect("send stats");
        for (message_index, message) in ordered.iter().enumerate() {
            if let Err(error) = sink.send(&actor, message).await {
                eprintln!("send #{message_index} failed: {error:#}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        }
        eprintln!("已发送 {} 条消息", ordered.len() + 1);
    }
}
