- title: Skill Runtime Stage Visibility
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - crates/hone-channels/src/agent_session.rs
  - crates/hone-channels/src/mcp_bridge.rs
  - crates/hone-tools/src/skill_runtime.rs
  - crates/hone-tools/src/skill_tool.rs
  - crates/hone-tools/src/discover_skills.rs
  - crates/hone-tools/src/load_skill.rs
  - crates/hone-tools/src/cron_job_tool.rs
  - docs/invariants.md
  - tests/regression/manual/test_hone_mcp_skill_dir_env.sh
  - tests/regression/manual/test_hone_mcp_cron_visibility.sh
  - tests/regression/manual/test_codex_acp_session_reload_cron_visibility.sh
- related_docs:
  - docs/current-plans/skill-runtime-align-claude-code.md
  - docs/current-plan.md
- related_prs:
  - N/A

## Summary

本轮把“看得见的 skill 应该保证能用，disable 的直接看不见”落实成运行时契约。此前 prompt / discover surface 会把 `scheduled_task` 暴露给模型，但实际 `hone-mcp` 阶段可能既拿不到正确的 `HONE_SKILLS_DIR`，也拿不到 `cron_job`；结果就是模型能看到 skill，却在真正调用时失败。现在 skill 的可见性、可搜索性和可加载性都统一受 stage constraints 约束，并且 `HONE_SKILLS_DIR` 会稳定透传到 `hone-mcp`。

同一天后续继续排查 `telegram + codex_acp` 真链路时，又定位到另一条 runner-specific 断点：即使 direct chat 已经允许 cron、`scheduled_task` 也能被 `skill_tool` 加载，`codex_acp` 仍可能把 `cron_job` 表现成“没暴露”。真实根因不是 channel gating，而是 `cron_job` 的 MCP `inputSchema` 不合法，导致 Codex 在把 `mcp__hone__cron_job` 转成 OpenAI tool 时直接丢弃。

## What Changed

- 在 `crates/hone-channels/src/mcp_bridge.rs` 里把 `HONE_SKILLS_DIR` 加入 hone MCP 环境透传，修复 prompt-time 可见但 sandbox 内 load 不到 skill 的路径。
- 在 `crates/hone-tools/src/skill_runtime.rs` 新增 `SkillStageConstraints`，统一表达：
  - 当前 stage 是否允许 `cron_job`
  - `HONE_MCP_ALLOWED_TOOLS` 这类显式 allowed-tools 限制
- 让 `discover_skills`、`load_skill`、`skill_tool`、prompt-time related-skill hints 与 `/skill` / direct invoke 全部走同一套 stage-aware 过滤。
- 为被 stage 阻断的 skill 返回显式错误，而不是模糊地表现成“skill 存在但调用挂了”。
- 修正 `crates/hone-tools/src/cron_job_tool.rs` 里的 `tags` 数组参数 schema，把 `items` 从裸字符串 `"string"` 改成合法 JSON Schema `{ "type": "string" }`，避免 `codex_acp` 丢弃整条 `cron_job` 工具。
- 在 `docs/invariants.md` 补一条长期约束：visible skills must be usable in the current stage。
- 新增三条手工回归脚本，覆盖 `HONE_SKILLS_DIR` 透传、`cron_job` 暴露/隐藏一致性，以及 `codex_acp session/load` 后 cron tool 仍可见可调。

## Verification

- `cargo test -p hone-tools`
- `cargo test -p hone-channels handle_tools_list_exposes_cron_job_only_when_allow_cron_is_enabled`
- `cargo test -p hone-channels handle_tools_call_rejects_cron_job_when_stage_allowed_tools_excludes_it`
- `cargo test -p hone-channels resolve_prompt_input_hides_cron_only_skills_when_cron_is_not_allowed`
- `bash tests/regression/manual/test_hone_mcp_skill_dir_env.sh`
- `bash tests/regression/manual/test_hone_mcp_cron_visibility.sh`
- `cargo test -p hone-tools openai_schema_uses_object_items_for_tags_array`
- `cargo test -p hone-tools cron_job_tool_add_list_update_remove_flow`
- `bash tests/regression/manual/test_codex_acp_session_reload_cron_visibility.sh`

## Risks / Follow-ups

- 更完整的 Claude Code 对齐工作还没结束；hooks 真执行、watcher 热重载和更细粒度 runner enforcement 仍在活跃计划内。
- 后续如果再新增 skill surface，必须复用 `SkillStageConstraints`，否则会重新出现“prompt 可见但运行不可用”的能力漂移。
- 对 `codex_acp` / 其它 ACP runner 来说，MCP `inputSchema` 的合法性本身也是兼容面；以后新增数组 / object 参数时，不能再依赖宽松解析。

## Next Entry Point

- `docs/current-plans/skill-runtime-align-claude-code.md`
