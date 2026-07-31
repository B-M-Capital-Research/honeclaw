# Handoff: SEC Filing 摘要 JSON 跨渠道归一化

- title: SEC Filing 摘要 JSON 跨渠道归一化
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/event.rs
  - crates/hone-event-engine/src/pollers/sec_enrichment.rs
  - crates/hone-event-engine/src/renderer.rs
  - crates/hone-event-engine/src/digest/render.rs
  - crates/hone-core/src/config/tests.rs
  - config.example.yaml
  - config.yaml
- related_docs:
  - docs/archive/plans/sec-filing-summary-json-normalization.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

2026-07-31 08:31（Asia/Shanghai）的真实 Discord 投递证据显示，LITE 与 TEM 的 SEC 8-K 摘要把 `{"summary": ...}` 原样展示给用户。根因是 `filing_summary` profile 强制 `response_format: json_object`，而 SEC enrichment 将 provider `content` 整体保存为字符串，后续即时 renderer 与 Digest 又直接渲染该字段。

修复同时覆盖新事件与历史/排队事件：写入前归一化 provider 响应，渲染时再次防御性归一化。所有即时渠道共享 renderer，Digest 也调用同一 `MarketEvent::normalized_llm_summary`，因此 Discord、Telegram、飞书、iMessage、即时推送和 Digest 使用一致的用户可见正文。

## What Changed

- 支持纯文本、裸 JSON、JSON 字符串和 Markdown JSON 代码块；提取非空 `summary`。
- 对损坏 JSON、无 `summary` 或空 `summary` 返回 `None`，回退到原始 filing 摘要/日期，不再暴露协议外壳。
- SEC enrichment 在写入 `payload.llm_summary` 前完成归一化并只缓存纯文本。
- 即时 renderer 与 Digest 对已有数据库/队列中的旧 JSON 包装做同一归一化。
- 从仓库 `config.example.yaml` 与本机忽略的 `config.yaml` 删除 `filing_summary.response_format: json_object`，并增加配置回归断言防止重新引入。
- 重新构建并重启本机源码实例；生成的新 `data/runtime/effective-config.yaml` 已确认 `filing_summary.response_format` 为 `null`。

## Verification

- `cargo test -p hone-event-engine --lib`: `524 passed`, `13 ignored`.
- `cargo test -p hone-core --lib`: `131 passed`;最终配置断言另以 focused test 通过。
- `cargo check -p hone-web-api`: passed；仅有既存 `feishu_direct_actor_contact_targets_from_records` `dead_code` warning。
- `cargo build --bin hone-cli --bin hone-console-page --bin hone-discord`: passed.
- 跨渠道 regression 覆盖 Plain/iMessage、Telegram HTML、Discord Markdown、Feishu Post 与 Digest，均只显示摘要正文。
- Exact changed-file `rustfmt --check`, `bash scripts/ci/check_fmt_changed.sh`, and `git diff --check`: passed.
- `cargo fmt --all -- --check`: only the untouched pre-existing drift in `crates/hone-channels/src/agent_session/artifacts.rs` and `crates/hone-channels/src/core/bot_core.rs` remains.
- Runtime restart: effective profile no longer forces JSON; Discord logged in; admin/public roots returned HTTP 200; active chat count was `0`.
- Live LLM news baseline was skipped because no classifier/model/fixture behavior changed.

## Risks / Follow-ups

- 已经发送到 Discord 的旧消息不会被重写；后续事件与仍会重新渲染的排队/历史事件受保护。
- 飞书传输协议仍然是 JSON；修复保证其用户可见 text 元素不含 `summary` 外壳。
- Telegram、飞书和 iMessage 当前在本机配置中未启用，因此以共享 renderer/协议单测证明；没有向外部账号发送 canary。
- 四个事件引擎测试命名整理与本批用户确认的通知修复一并提交并推送到 `main`；本次没有正式发布或 tag。

## Next Entry Point

如再次出现大括号，先查询 `delivery_log.body` 与 `events.payload_json.llm_summary`，再从 `MarketEvent::normalized_llm_summary`、`SecFilingSummarizer::summarize` 和对应渠道 renderer 判断是新 payload 形态还是非 SEC 的另一条结构化输出链路。
