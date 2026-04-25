# Bug: Truth Social poller 用不透明响应掩盖并持续触发 source 断流

状态：`Fixing`

最新进展：2026-04-24 已补偿日志，且本轮巡检看到了第一条 live 样本。`TruthSocialPoller` 现在确实会把 search / statuses 响应先读成文本，再在非 2xx 或 JSON 解码失败时输出 `status`、`content_type` 与截断 `body_prefix`；`data/runtime/logs/web.log.2026-04-24:614` 已明确记录 `truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix=...`。这说明“日志不可定位 / 不可排障”的缺口已收口，但 `truth_social.realdonaldtrump` source 依旧 0 条事件，当前问题已经从“opaque decode 难排查”收敛为“enabled source 在真实运行中持续被 403 HTML 响应拦截”。

最新巡检补充：2026-04-24T18:34:27Z 的增量窗口里，同一问题在重启后继续复现。`data/runtime/logs/web.log.2026-04-24:3167/3177`、`3652/3663`、`4113` 分别记录了 `truth_social poller starting` 后紧接 `initial poll failed` / `poll failed: truth_social statuses HTTP 403 Forbidden ...`；`data/events.sqlite3` 对 `source LIKE 'truth_social.%'` 仍然查不到任何事件。这说明当前坏态不是旧日志残留，而是在最近一轮真实运行中持续断流。

## Summary

已启用的 `truth_social.realdonaldtrump` source 在本地库里仍然 `0` 条事件；本轮 live 日志已经明确显示 Truth Social `statuses` 接口返回 `HTTP 403 + text/html`，说明 observability 补丁生效了，但 source 断流本身仍在持续发生。

## Observed Symptoms

- `config.yaml:238-243` 明确启用了 Truth Social 轮询，并且把 `account_id` 固定为 `"107780257626128497"`、`interval_secs=3600`，注释还直接写明“国内/数据中心 IP 可能被 Cloudflare 拦截，poller 会 warn! 后下轮重试”。
- 本轮新增窗口里的 live 进程再次装配了该 poller，并立刻输出 source-specific 失败样本：
  - `data/runtime/logs/web.log.2026-04-24:598`
    - `2026-04-24 12:05:18.498 INFO  truth_social poller starting`
  - `data/runtime/logs/web.log.2026-04-24:614`
    - `2026-04-24 12:05:20.030 WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> <!--[if lt IE 7]> <html class=\"no-js ie6 oldie\" lang=\"en-US\"> <![endif]--> <!--[if IE 7]> <html class=\"no-js ie7 oldie\" lang=\"en-US\"> <![endif]--> <!--[if IE 8]> <html class=\"no-js ie8 oldie\" lang=\"en-US\"> <![endif]--> <!--["`
- 这条样本正好落在上一个 `poll failed: error decoding response body: expected value at line 1 column 1` 小时节拍之后，说明“报错正文不透明”已经修好，但 live 请求仍被返回 HTML 403 页面。
- `data/events.sqlite3` 只读查询显示，同一套 social poller 里 Telegram source 仍在正常产出，但 Truth Social source 仍然完全没有事件：
  - `select count(*) from events where source='telegram.watcherguru'` -> `31`
  - `select datetime(max(created_at_ts),'unixepoch') from events where source='telegram.watcherguru'` -> `2026-04-23 21:52:50`
  - `select count(*) from events where source='truth_social.realdonaldtrump'` -> `0`
  - `select datetime(max(created_at_ts),'unixepoch') from events where source='truth_social.realdonaldtrump'` -> `NULL`
- 这说明当前异常不是“整个 social 子系统都坏了”，而是 Truth Social 这一条 enabled source 在本轮继续被明确的 `403 text/html` 响应挡住。

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/lib.rs:522-533` 把 Truth Social source 以 `interval_secs` 固定节拍挂到 `spawn_event_source`，而 `spawn_event_source` 的固定间隔分支只在消息正文里打印通用 `poll failed`，source 名称只存在结构化字段里；当前文件日志没有把这些字段展开到正文，因此肉眼无法直接知道是哪条 source 在失败：

```rust
for cfg in &sources.truth_social_accounts {
    let poller = TruthSocialPoller::new(
        cfg.username.clone(),
        cfg.account_id.clone(),
        Duration::from_secs(cfg.interval_secs),
    );
    info!(
        username = %cfg.username,
        interval_secs = cfg.interval_secs,
        "truth_social poller starting"
    );
    spawn_event_source(Arc::new(poller), store.clone(), router.clone());
}
```

`crates/hone-event-engine/src/lib.rs:930-955` 的固定间隔事件源循环会在冷启动和后续每个 interval 重试时把错误统一折叠为 `initial poll failed` / `poll failed`。这意味着只要上游持续返回 403，当前 source 就会稳定停留在“每小时 degraded 一次、但没有补救路径”的状态：

```rust
match schedule {
    SourceSchedule::FixedInterval(interval) => {
        if let Err(e) = run_once(&name, &*source, &store, &router).await {
            warn!(
                poller = %name,
                source = %name,
                url_class = "event_source",
                degraded = true,
                "initial poll failed: {e:#}"
            );
        }
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = run_once(&name, &*source, &store, &router).await {
                warn!(
                    poller = %name,
                    source = %name,
                    url_class = "event_source",
                    degraded = true,
                    "poll failed: {e:#}"
                );
            }
        }
    }
```

`crates/hone-event-engine/src/pollers/social/truth_social.rs:102-129` 现在已经会把 `status`、`content_type` 和 `body_prefix` 记进错误；本轮 live 样本正是从这里产出的 `HTTP 403 text/html`。因此当前更可疑的根因已不是“代码吞掉了错误上下文”，而是同一个请求路径在真实环境下持续拿到 HTML 拦截页：

```rust
async fn fetch_json(&self, url: &str, endpoint: &str) -> anyhow::Result<Value> {
    let resp = self
        .http
        .get(url)
        .send()
        .await
        .with_context(|| format!("truth_social {endpoint} request failed"))?;
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let body_text = resp.text().await.with_context(|| {
        format!("truth_social {endpoint} read body failed status={status} content_type={content_type}")
    })?;
    let body_prefix = response_body_prefix(&body_text);
    if !status.is_success() {
        anyhow::bail!(
            "truth_social {endpoint} HTTP {status} content_type={content_type} body_prefix={body_prefix:?}"
        );
    }
    serde_json::from_str(&body_text).map_err(|e| {
        anyhow::anyhow!(
            "truth_social {endpoint} JSON decode failed status={status} content_type={content_type} body_prefix={body_prefix:?}: {e}"
        )
    })
}
```

结合 `config.yaml` 里对 Cloudflare 的已有注释，这条代码路径现在可以解释为“source 一直是 0，且 live 运行已经能直接看到 403 HTML 拦截页”。

## Evidence Gap

- 现在已经有 live 失败样本能坐实 `statuses` 接口返回 `403 text/html`，所以“到底是不是 Truth Social source / 到底是不是非 JSON 响应”这层疑点已经消失。
- 本轮巡检遵循只读约束，没有主动请求 Truth Social，也没有打开任何真实网络探测；因此缺少单次请求级复现样本。
- 仍不能仅凭当前 `body_prefix` 断言 403 一定来自 Cloudflare 挑战页、地区/IP 黑名单、缺少 browser-like headers，还是账号 / endpoint 本身的访问策略变化；需要后续在同一网络出口做一次受控复现，或补充更完整的响应头诊断。
- 文件日志虽然已经能把 endpoint 级错误写进正文，但还没有展开完整响应头；当前无法从本地只读证据直接拿到 `cf-ray`、`server`、`set-cookie` 等能更快定位 403 来源的字段。

## Fix Notes

- `crates/hone-event-engine/src/pollers/social/truth_social.rs`
  - 新增 `fetch_json(endpoint)` 公共响应处理，避免 `resp.json()` 在 HTTP status 判断前吞掉错误上下文。
  - 非 2xx 响应记录：`truth_social {endpoint} HTTP {status} content_type={content_type} body_prefix=...`
  - 2xx 但非 JSON 响应记录：`truth_social {endpoint} JSON decode failed status={status} content_type={content_type} body_prefix=...`
  - `body_prefix` 只保留折叠空白后的前 240 字符，避免把完整 HTML 错误页写入日志。
- 回归测试：
  - `fetch_statuses_reports_http_status_content_type_and_body_prefix`
  - `fetch_statuses_reports_json_decode_context_for_html_success`

## Verification

- `cargo test -p hone-event-engine truth_social --lib`
- `cargo test -p hone-event-engine --lib`
- `cargo fmt --all -- --check`

## Severity

`sev2`。理由：这是一个已启用 source 的持续断流，用户会稳定错过 Truth Social 侧的重要社交事件；但其它 event-engine 主链路、FMP poller 和 Telegram social source 仍在工作，尚未上升到整个 event-engine 不可用的 `sev1`。

## Latest巡检 Update

- 2026-04-24T18:34:27Z：上次巡检之后的新增窗口里，Truth Social 403 仍在继续：

```text
data/runtime/logs/web.log.2026-04-24:3167:[2026-04-25 00:13:34.788] INFO  truth_social poller starting
data/runtime/logs/web.log.2026-04-24:3177:[2026-04-25 00:13:36.013] WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-24:3652:[2026-04-25 01:06:50.835] INFO  truth_social poller starting
data/runtime/logs/web.log.2026-04-24:3663:[2026-04-25 01:06:51.991] WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-24:4113:[2026-04-25 02:06:53.106] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
```

- 同一窗口里 `data/events.sqlite3` 仍然查不到 `truth_social.%` 新事件：

```text
SELECT source, COUNT(*), datetime(MAX(created_at_ts),'unixepoch')
FROM events
WHERE source LIKE 'truth_social.%'
GROUP BY source;
-- no rows
```

- 其它主链路仍在工作：同一时间窗 `poller ok` 连续、`fmp.*` 近 24h 仍有正常记录、`delivery_log` 也持续出现 `sink/high/sent`。因此这次补充继续把问题限定在 Truth Social source 自身，而不是整个 event-engine 停摆。

## Latest巡检 Update

- 2026-04-24T22:26:46Z：在 `2026-04-24T18:25:00Z` 之后，Truth Social `statuses` 403 又连续出现了 3 次，新增窗口里的本地日志样本如下：

```text
data/runtime/logs/web.log.2026-04-24:4224:[2026-04-25 03:06:53.157] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-24:4488:[2026-04-25 05:06:53.056] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-24:4572:[2026-04-25 06:06:52.928] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
```

- 同一巡检窗口里，Telegram social source 仍在继续落库，说明 social 子系统并非整体停摆：

```text
source=telegram.watcherguru
count=40
last_created_utc=2026-04-24 22:06:55
```

- 但 `data/events.sqlite3` 对 `source LIKE 'truth_social.%'` 仍然 0 行，表示启用的 Truth Social source 依旧完全断流：

```text
SELECT source, COUNT(*), datetime(MAX(created_at_ts),'unixepoch')
FROM events
WHERE source LIKE 'truth_social.%'
GROUP BY source;
-- no rows
```

- `data/runtime/logs/web.log.2026-04-24` 在这些 403 前后仍持续出现 `poller ok`，因此这次补充继续把影响限定在 Truth Social source 自身，而不是整个 event-engine cadence 或 sink 装配失败。

## Latest巡检 Update

- 2026-04-25T02:32:11Z：在 `2026-04-24T22:25:31Z` 之后，Truth Social `statuses` 403 在新日志文件里继续复现，而且重启后立即失败：

```text
data/runtime/logs/web.log.2026-04-25:11:[2026-04-25 08:06:52.901] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:487:[2026-04-25 09:06:52.603] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:729:[2026-04-25 10:00:58.732] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-25:744:[2026-04-25 10:00:58.738] INFO  truth_social poller starting
data/runtime/logs/web.log.2026-04-25:760:[2026-04-25 10:01:00.026] WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
```

- 同一巡检窗口里，`data/events.sqlite3` 对 `source LIKE 'truth_social.%'` 仍然没有任何事件，而 Telegram social source 继续有历史产出：

```text
SELECT source, count(*), datetime(max(created_at_ts),'unixepoch')
FROM events
WHERE source LIKE 'truth_social.%'
GROUP BY source;
-- no rows

telegram.watcherguru|40|2026-04-24 22:06:55
```

- 这次补充把“重启后会不会恢复”的疑点也排除了：最新 `MultiChannelSink` 装配完成后，Truth Social poller 仍在冷启动阶段立即拿到 `403 text/html`，因此坏态仍然限定在 enabled Truth Social source 自身。

## Latest巡检 Update

- 2026-04-25T06:33:21Z：在 `2026-04-25T02:25:52Z` 之后，这个 source 仍然没有恢复，而且在两次新的冷启动后都立即复现 `403 text/html`，随后按小时继续失败：

```text
data/runtime/logs/web.log.2026-04-25:851:[2026-04-25 10:31:18.634] INFO  truth_social poller starting
data/runtime/logs/web.log.2026-04-25:861:[2026-04-25 10:31:19.797] WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:957:[2026-04-25 10:38:25.555] INFO  truth_social poller starting
data/runtime/logs/web.log.2026-04-25:969:[2026-04-25 10:38:26.687] WARN  initial poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1064:[2026-04-25 11:38:28.046] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1115:[2026-04-25 12:38:28.297] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1140:[2026-04-25 13:38:28.343] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
```

- 同一增量窗口里，`data/events.sqlite3` 依旧查不到任何 `truth_social.%` 事件；而 `telegram.watcherguru` 仍停在旧值 `40`、最近一条 `created_at_ts=2026-04-24 22:06:55 UTC`，说明 Truth Social 至少没有因为这两次重启而恢复出数：

```text
SELECT source, count(*), datetime(max(created_at_ts),'unixepoch')
FROM events
WHERE source LIKE 'truth_social.%'
GROUP BY source;
-- no rows

telegram.watcherguru|40|2026-04-24 22:06:55
```

- 本轮其它 event-engine 主链路没有一起劣化：`poller ok` 最大间隔约 319 秒，没有 >15 分钟停摆；`event_engine.dryrun=false` 且 `MultiChannelSink` 仍反复装配成功；`fmp.earning_calendar` / `fmp.stock_dividend_calendar` / `fmp.economic_calendar` / `fmp.stock_split_calendar` / `fmp.quote` 近 24h 仍都有新记录。因此这次补充继续把影响限定在 Truth Social source 自身，而不是整个 poller cadence、sink 装配或 FMP feed 断流。

## Latest巡检 Update

- 2026-04-25T10:34:24Z：在上次巡检时间 `2026-04-25T06:26:23.700Z` 之后，Truth Social `statuses` 403 又连续出现了 4 次，而且仍是同一种 `text/html` 拦截页形态：

```text
data/runtime/logs/web.log.2026-04-25:1166:[2026-04-25 14:38:28.559] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1192:[2026-04-25 15:38:28.424] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1261:[2026-04-25 16:38:28.523] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
data/runtime/logs/web.log.2026-04-25:1288:[2026-04-25 17:38:28.469] WARN  poll failed: truth_social statuses HTTP 403 Forbidden content_type=text/html; charset=UTF-8 body_prefix="<!DOCTYPE html> ..."
```

- 同一增量窗口里，`data/events.sqlite3` 仍然没有任何 `truth_social.%` 行，说明这个 enabled source 在本轮没有恢复出数：

```text
SELECT source, COUNT(*), datetime(MAX(created_at_ts),'unixepoch')
FROM events
WHERE source LIKE 'truth_social.%'
GROUP BY source;
-- no rows
```

- 其它 event-engine 主链路在同一窗口仍持续前进，因此这次新增证据继续把坏态限定在 Truth Social source 自身，而不是整个引擎停摆：

```text
SELECT source, count(*)
FROM events
WHERE created_at_ts >= strftime('%s','2026-04-25T06:26:23.700Z')
GROUP BY source
ORDER BY count(*) DESC;

fmp.stock_news:defenseworld.net|32
fmp.economic_calendar|22
fmp.stock_news:seekingalpha.com|12
fmp.stock_news:prnewswire.com|7
fmp.stock_split_calendar|5
...
```

- `delivery_log` 在同一窗口也继续记录 `queued / filtered / no_actor`，说明 router/digest 仍在处理新增事件；本轮没有新增 `high` 事件，因此没有新的 `sent` 并不改变“Truth Social source 持续断流”这一定性。

## Date Observed

`2026-04-24T04:05:20Z`
