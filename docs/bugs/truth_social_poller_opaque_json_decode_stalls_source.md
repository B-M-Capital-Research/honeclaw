# Bug: Truth Social poller 用不透明 JSON 解码错误掩盖 source 断流

状态：`Fixing`

最新进展：2026-04-24 已补偿日志。`TruthSocialPoller` 现在对 search / statuses 响应先读取文本，再在非 2xx 或 JSON 解码失败时输出 `status`、`content_type` 与截断 `body_prefix`；同时补了本地 HTTP mock 回归测试，覆盖 `503 text/html` 与 `200 text/html` 两类 opaque decode 场景。该改动解决“日志不可定位 / 不可排障”的问题，但 Truth Social 真实断流是否来自 Cloudflare、空 body 或其它上游响应，还需要等 live 进程产出下一条补偿日志后确认。

## Summary

已启用的 `truth_social.realdonaldtrump` source 在本地库里至今 `0` 条事件；自 `2026-04-23T14:18:41.651Z` 之后，`web.log` 又按 3600 秒节拍重复出现 `poll failed: error decoding response body: expected value at line 1 column 1`。当前实现先对响应做 `resp.json()` 再判断 HTTP 状态，导致 Truth Social / Cloudflare 返回的非 JSON 页面被压扁成无 source 名称的通用解码报错，实际断流对巡检与排障都不透明。

## Observed Symptoms

- `config.yaml:238-243` 明确启用了 Truth Social 轮询，并且把 `account_id` 固定为 `"107780257626128497"`、`interval_secs=3600`，注释还直接写明“国内/数据中心 IP 可能被 Cloudflare 拦截，poller 会 warn! 后下轮重试”。
- `data/runtime/logs/web.log.2026-04-23:3238` 记录当前进程启动后装配了该 poller：
  - `2026-04-23 19:22:45.538 INFO  truth_social poller starting`
- 同一日志文件在上次巡检后的增量窗口里，随后每小时都重复同一条 WARN：
  - `data/runtime/logs/web.log.2026-04-23:3923`
    - `2026-04-23 22:22:49.175 WARN  poll failed: error decoding response body: expected value at line 1 column 1`
  - `data/runtime/logs/web.log.2026-04-23:4113`
    - `2026-04-23 23:22:47.786 WARN  poll failed: error decoding response body: expected value at line 1 column 1`
  - `data/runtime/logs/web.log.2026-04-23:4383`
    - `2026-04-24 00:22:47.928 WARN  poll failed: error decoding response body: expected value at line 1 column 1`
  - `data/runtime/logs/web.log.2026-04-23:4538`
    - `2026-04-24 01:22:47.901 WARN  poll failed: error decoding response body: expected value at line 1 column 1`
  - `data/runtime/logs/web.log.2026-04-23:4706`
    - `2026-04-24 02:22:47.904 WARN  poll failed: error decoding response body: expected value at line 1 column 1`
- `data/events.sqlite3` 只读查询显示，同一套 social poller 里 Telegram source 仍在正常产出，但 Truth Social source 完全没有事件：
  - `select count(*) from events where source='telegram.watcherguru'` -> `29`
  - `select datetime(max(created_at_ts),'unixepoch') from events where source='telegram.watcherguru'` -> `2026-04-23 18:22:49`
  - `select count(*) from events where source='truth_social.realdonaldtrump'` -> `0`
  - `select datetime(max(created_at_ts),'unixepoch') from events where source='truth_social.realdonaldtrump'` -> `NULL`
- 这说明当前异常不是“整个 social 子系统都坏了”，而是 Truth Social 这一条 source 长期静默且本轮继续失败。

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

`crates/hone-event-engine/src/lib.rs:930-955` 的固定间隔事件源循环会把 `run_once()` 的错误统一写成 `poll failed: {e:#}`；如果底层只返回 JSON decode error，日志里就只剩下一句无法定位 source/HTTP 状态的 warn：

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

`crates/hone-event-engine/src/pollers/social/truth_social.rs:93-105` 在 `fetch_statuses()` 里先调用 `resp.json().await?` 再判断 `status`；如果 Truth Social / Cloudflare 返回 HTML 挑战页、空 body 或其它非 JSON 响应，错误会在 `resp.json()` 处提前抛出，HTTP 状态码与 body 前缀都丢失，最终正好退化成日志里的 `expected value at line 1 column 1`：

```rust
async fn fetch_statuses(&self) -> anyhow::Result<Vec<Value>> {
    let account_id = self.resolve_account_id().await?;
    let url = format!(
        "{}/api/v1/accounts/{}/statuses?limit={}&exclude_replies=true",
        self.base_url, account_id, self.limit
    );
    let resp = self.http.get(&url).send().await?;
    let status = resp.status();
    let body: Value = resp.json().await?;
    if !status.is_success() {
        anyhow::bail!("truth_social statuses HTTP {status}: {body}");
    }
    Ok(body.as_array().cloned().unwrap_or_default())
}
```

结合 `config.yaml` 里对 Cloudflare 的已有注释，这条代码路径可以解释“source 一直是 0，日志却只有模糊 JSON 解码 warn”的现象。

## Evidence Gap

- 2026-04-24 代码侧已补 `status`、`content_type` 和 `body_prefix`，但当前日志样本仍是修复前产生的 opaque decode error；还需要 live 进程重启 / 部署后等下一次失败样本来坐实 Cloudflare HTML 挑战页、反爬拦截页还是上游接口返回了别的非 JSON 内容。
- 本轮巡检遵循只读约束，没有主动请求 Truth Social，也没有打开任何真实网络探测；因此缺少单次请求级复现样本。
- 文件日志当前没有把 `poller=%name` / `source=%name` 结构化字段展开到正文；“这 5 条 hourly warn 对应 truth_social”是基于 `3600s` 配置节拍、Truth Social source 总事件数恒为 0、同时 Telegram social source 正常产出的综合推断。

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

- `rtk cargo test -p hone-event-engine truth_social --lib`
- `rtk cargo test -p hone-event-engine --lib`
- `rtk cargo fmt --all -- --check`

## Severity

`sev2`。理由：这是一个已启用 source 的持续断流，用户会稳定错过 Truth Social 侧的重要社交事件；但其它 event-engine 主链路、FMP poller 和 Telegram social source 仍在工作，尚未上升到整个 event-engine 不可用的 `sev1`。

## Date Observed

`2026-04-23T18:28:33Z`
