use super::*;

use std::sync::Arc;

#[tokio::test]
async fn start_respects_disabled_flag() {
    let engine = EventEngine::new(EventEngineConfig::default(), FmpConfig::default());
    engine.start().await.unwrap();
}

#[tokio::test]
async fn start_warns_when_enabled_but_no_key() {
    let mut cfg = EventEngineConfig::default();
    cfg.enabled = true;
    let engine = EventEngine::new(cfg, FmpConfig::default());
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

    let tmp = tempfile::tempdir().unwrap();
    let store_path = tmp.path().join("events.db");
    let jsonl_path = tmp.path().join("events.jsonl");
    let portfolio_dir = tmp.path().join("portfolio");
    let engine = EventEngine::new(engine_cfg, fmp_cfg)
        .with_store_path(store_path.clone())
        .with_events_jsonl_path(Some(jsonl_path.clone()))
        .with_portfolio_dir(portfolio_dir)
        .with_retention_days(0);
    engine.start().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let store = EventStore::open(&store_path).unwrap();
    let n = store.count_events().unwrap();
    let jsonl_lines = std::fs::read_to_string(&jsonl_path)
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as i64)
        .unwrap_or(-1);
    println!("e2e count_events = {n} jsonl_lines = {jsonl_lines}");
    assert!(n > 0, "SQLite 应写入事件");
    assert!(jsonl_lines > 0, "JSONL 镜像应同步写入事件");
    assert_eq!(
        jsonl_lines, n,
        "JSONL 行数应与 SQLite events 行数一致（单次冷启，无去重丢失）"
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
        let resp = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = resp.status();
        let body_resp = resp.text().await.unwrap_or_default();
        println!("[tg demo] fmt={fmt:?} status={status} body={body_resp}");
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {body_resp}"
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
    let or_key = std::env::var("HONE_OPENROUTER_KEY").expect("需要 HONE_OPENROUTER_KEY");
    let or_model = std::env::var("HONE_OPENROUTER_MODEL")
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
    let provider = Arc::new(OpenRouterProvider::new(&or_key, &or_model, 4096));
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
        let resp = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = resp.status();
        let body_resp = resp.text().await.unwrap_or_default();
        println!("[tg polish demo] html={use_html} status={status} body={body_resp}");
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {body_resp}"
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
    let ca_filt: Vec<_> = ca_calendar
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
        ca_filt.len(),
        sec_events.len()
    );

    // 7) 组装待推消息
    let fmt = RenderFormat::TelegramHtml;
    let mut messages: Vec<(bool, String)> = Vec::new();

    // 7a) LLM 生成"今日要点"摘要（可选：无 OPENROUTER_KEY 时跳过）
    if let Ok(or_key) = std::env::var("HONE_OPENROUTER_KEY") {
        use hone_llm::{LlmProvider, Message, OpenRouterProvider};
        let or_model = std::env::var("HONE_OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-haiku-4-5".to_string());
        let provider = OpenRouterProvider::new(&or_key, &or_model, 1024);

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
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                role: "user".into(),
                content: Some(payload.to_string()),
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
        let mut s = format!("📅 持仓未来 14 天财报 · {} 条", sorted.len());
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
            s.push_str(&format!(
                "\n• {urgency} ${sym} · {date} {time_slot} · {est_part}"
            ));
        }
        messages.push((true, s));
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
        // [news - 1d, news + 2d] 窗口内,若有则 🔔 标记——这些是 Router 里
        // `maybe_upgrade_news` 会把 Low 升到 Medium 的那一批,肉眼可验证。
        let earn_by_sym: std::collections::HashMap<&str, &crate::event::MarketEvent> = earn_filt
            .iter()
            .filter_map(|e| e.symbols.first().map(|s| (s.as_str(), e)))
            .collect();
        let in_earnings_window = |ev: &crate::event::MarketEvent| -> Option<i64> {
            let sym = ev.symbols.first()?.as_str();
            let earn = earn_by_sym.get(sym)?;
            let start = ev.occurred_at - chrono::Duration::days(1);
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

        let mut s = format!(
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
                    s.push_str(&format!(
                        "\n• ${sym}{tag} · {ts} · {title_esc} <a href=\"{u}\">{host}</a>"
                    ));
                }
                None => {
                    s.push_str(&format!("\n• ${sym}{tag} · {ts} · {title_esc}"));
                }
            }
        }
        messages.push((true, s));
    }

    // SEC 8-K：poller 侧已经按 72h 切过时效;这里直接按时间降序渲染。
    // payload 里无 item/description，把 accepted 时分 + EDGAR index link +
    // finalLink 文档都放出来让用户自己看。
    if !sec_events.is_empty() {
        let mut recent: Vec<&crate::event::MarketEvent> = sec_events.iter().collect();
        recent.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        if !recent.is_empty() {
            let mut s = format!("📄 持仓最近 72h SEC 8-K · {} 条", recent.len());
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
                s.push_str(&format!(
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
                    s.push_str(&format!("\n   ↳ {}", links.join(" · ")));
                }
            }
            messages.push((true, s));
        } else {
            println!("SEC 8-K 过去 72h 无；全持仓都是历史老文件");
        }
    }
    if !ca_filt.is_empty() {
        let s = crate::digest::render_digest("持仓拆股/分红", &ca_filt, 0, fmt);
        messages.push((true, s));
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
        let resp = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("telegram 发送请求失败");
        let status = resp.status();
        let body_resp = resp.text().await.unwrap_or_default();
        println!(
            "[backtest push] html={use_html} status={status} body_len={}",
            body_resp.len()
        );
        assert!(
            status.is_success(),
            "telegram API 返回非 2xx: {status} / {body_resp}"
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

    let tmp = tempfile::tempdir().unwrap();
    let store = Arc::new(EventStore::open(tmp.path().join("events.db")).unwrap());
    let report_dir = tmp.path().join("reports");

    let now_utc = chrono::Utc::now();
    let fake = vec![
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
    let mut idx = 0;
    for (src, kind, n) in fake {
        for _ in 0..n {
            let ev = MarketEvent {
                id: format!("fake-{idx}"),
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
            idx += 1;
        }
    }
    let ak_main = "telegram::::8039067465";
    for _ in 0..3 {
        store
            .log_delivery("f-s", ak_main, "sink", Severity::High, "sent", None)
            .unwrap();
    }
    for _ in 0..8 {
        store
            .log_delivery("f-q", ak_main, "digest", Severity::Medium, "queued", None)
            .unwrap();
    }
    for _ in 0..2 {
        store
            .log_delivery("f-f", ak_main, "prefs", Severity::Low, "filtered", None)
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
    let n = report.tick_once(trigger_utc, &mut fired).await.unwrap();
    assert_eq!(n, 1);

    let date_str = local_today.format("%Y-%m-%d").to_string();
    let file = report_dir.join(format!("{date_str}.md"));
    let body = std::fs::read_to_string(&file).expect("日报文件未生成");
    println!("\n=== daily_report {date_str}.md ===\n{body}");
    assert!(body.contains("# Hone 日报 · "));
    assert!(body.contains("合计 **10** 条"));
    // 两个 actor 行都在
    assert!(body.contains(&format!("| `{ak_main}` |")));
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

    let tmp = tempfile::tempdir().unwrap();
    let store_path = tmp.path().join("events.db");
    let jsonl_path = tmp.path().join("events.jsonl");
    let portfolio_dir = tmp.path().join("portfolio");
    let digest_dir = tmp.path().join("digest");
    let prefs_dir = tmp.path().join("prefs");
    let daily_report_dir = tmp.path().join("daily_reports");
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
    let n = store.count_events().unwrap();
    let jsonl = std::fs::read_to_string(&jsonl_path).unwrap_or_default();
    let tg_lines: Vec<&str> = jsonl
        .lines()
        .filter(|l| l.contains("\"telegram.watcherguru\""))
        .collect();

    println!("=== live_social_engine_e2e ===");
    println!("count_events = {n}");
    println!("telegram.watcherguru 事件数 = {}", tg_lines.len());
    if let Some(first) = tg_lines.first() {
        println!("第一条:{first}");
    }

    assert!(n > 0, "events SQLite 应有至少 1 条事件");
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
