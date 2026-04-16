- title: Search Failure Tavily And Tool Call Fix
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - crates/hone-tools/src/web_search.rs
  - crates/hone-channels/src/runners/multi_agent.rs
  - crates/hone-llm/src/openai_compatible.rs
  - bins/hone-desktop/src/sidecar.rs
  - docs/bugs/README.md
  - docs/bugs/openai_compatible_tool_call_protocol_mismatch_invalid_params.md
  - docs/bugs/minimax_search_http_transport_failure_no_retry.md
- related_docs:
  - docs/repo-map.md
  - docs/invariants.md
  - docs/runbooks/desktop-release-app-runtime.md

## Goal

确认最近“抱歉，这次处理失败了。请稍后再试。”是否由 Tavily / `web_search` 配置问题触发，补齐根因判断并修复当前仍在影响用户的主链路问题，然后按 runbook 重建 desktop release app。

## Scope

- 核对 `web_search` 工具实现与实际路由，确认是否走 Tavily。
- 检查当前运行配置中 Tavily、multi-agent search、multi-agent answer 的 key 来源与生效路径。
- 对照最近真实会话与日志，区分 API key / 上游网络问题 与 transcript/tool-call 协议问题。
- 在不修改无关业务逻辑的前提下修复已确认的主根因，并补回归测试。
- 如修复结论影响缺陷台账，更新 `docs/bugs/README.md` 与对应 bug 文档。
- 按 `docs/runbooks/desktop-release-app-runtime.md` 重新构建 release app。

## Validation

- 用源码核对 `web_search` 实现与 runtime 配置读取逻辑。
- 用最近真实日志 / 会话样本复核失败模式是否与修复点一致。
- 运行与修复点直接相关的 Rust 测试。
- 执行 desktop release build，确认打包命令成功完成。

## Documentation Sync

- 本任务满足跨模块、行为修复、需要 release 验证的准入标准，因此已落盘并完成归档。
- 若 bug 状态发生变化，更新 `docs/bugs/README.md` 与对应缺陷文档。
- 本轮已完成，计划页移入 `docs/archive/plans/`，并补 `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md` 与 `docs/archive/index.md` 索引。

## Risks / Open Questions

- 最近统一失败提示可能对应多个根因，不能把所有样本都归因到 Tavily。
- desktop 当前实际生效配置可能来自 canonical config、runtime overlay、环境变量或 sidecar 写入链路，需要逐层核对。
- release build 依赖本机桌面打包环境，若环境缺失需单独记录。
