- title: Search Failure Tavily And Tool Call Fix
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - crates/hone-tools/src/web_search.rs
  - crates/hone-channels/src/runners/multi_agent.rs
  - docs/bugs/README.md
  - docs/bugs/openai_compatible_tool_call_protocol_mismatch_invalid_params.md
  - docs/bugs/minimax_search_http_transport_failure_no_retry.md
- related_docs:
  - docs/archive/plans/search-failure-tavily-and-tool-call-fix.md
  - docs/runbooks/desktop-release-app-runtime.md
- related_prs:
  - N/A

## Summary

本轮先确认了最近统一失败提示并不等于“Tavily 全挂”。`web_search` 工具确实走 Tavily，但当前 desktop 生效配置中 Tavily key 池有 4 个 key，抽查结果为 1 个 `HTTP 432` 配额拒绝、3 个可正常返回 `200`；与此同时，multi-agent search provider 走的是 MiniMax。随后定位并修复了最近主导性失败根因：搜索阶段清洗历史上下文时只删 `tool` 消息、没删对应 assistant `tool_calls`，导致 OpenAI-compatible provider 报 `tool call result does not follow tool call (2013)`。

## What Changed

- 修复 `crates/hone-channels/src/runners/multi_agent.rs` 中 `sanitize_search_context(...)`：
  - 删除历史 `role=tool` 消息
  - 同时剥离历史 assistant 消息上的 `tool_calls`
  - 若 assistant 消息只剩空壳 tool-call、无正文内容，则直接丢弃
- 新增 2 个定向回归测试，锁住“剥离历史 assistant tool_calls”和“删除空 assistant tool-call shell”这两条行为。
- 更新 bug 台账：
  - `openai_compatible_tool_call_protocol_mismatch_invalid_params.md` 改为 `Fixed`
  - `minimax_search_http_transport_failure_no_retry.md` 补充 Tavily / MiniMax 路由与 key 抽查结论
  - `docs/bugs/README.md` 同步更新导航状态
- 按 runbook 成功重建 desktop release app 与 dmg。

## Verification

- `cargo test -p hone-channels sanitize_search_context -- --nocapture`
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Tavily 连通性抽查：
  - key#1 `HTTP 432`
  - key#2 `200`
  - key#3 `200`
  - key#4 `200`
- MiniMax auxiliary / search 路由抽查：
  - `https://api.minimaxi.com/v1/chat/completions` 返回 `200`

## Risks / Follow-ups

- Tavily key 池里仍有 1 个 key 已达到套餐额度，当前因为 fallback 仍可工作，未单独升级为功能缺陷；后续可考虑清理无效 key，减少一次无效尝试。
- `minimax_search_http_transport_failure_no_retry.md` 仍处于活跃状态，本轮没有为 MiniMax 传输抖动补自动重试。
- 需要继续观察运行日志，确认 `tool call result does not follow tool call (2013)` 是否已明显收敛。

## Next Entry Point

- `docs/bugs/README.md`
