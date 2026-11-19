# 主对话 Harness System Prompt 瘦身与下沉到 Skill / User Prompt

- title: 主对话 Harness System Prompt 瘦身与下沉到 Skill / User Prompt
- status: archived
- created_at: 2026-11-19
- updated_at: 2026-11-19
- owner: antigravity
- related_files:
  - `soul.md`
  - `skills/finance-research-core/SKILL.md`
  - `skills/hari-invest/SKILL.md`
  - `skills/company_portrait/SKILL.md`
  - `skills/scheduled_task/SKILL.md`
  - `skills/notification_preferences/SKILL.md`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/agent_session/tests.rs`
  - `crates/hone-channels/examples/finance_consistency_llm_smoke.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs:
  - `docs/decisions.md`
  - `docs/handoffs/2026-11-19-harness-prompt-slim-and-skill-offload.md`
  - `docs/archive/index.md`

## Goal

- 将主对话 Harness 的 `system_prompt` 置空（`soul.md` 置空、`PromptBundle.static_system` 不再拼接底座人设与硬编码领域策略）。
- 将原 `soul.md` 与 `prompt.rs` 中的领域策略下沉到 Skill 按需加载。
- 将原 `system_prompt` 中的其余运行时/按轮上下文统一挪到 user prompt（`runtime_input`）。

## Outcome

- 已完成实现、单元测试与文档归档，详见 `docs/handoffs/2026-11-19-harness-prompt-slim-and-skill-offload.md` 与 `docs/decisions.md`（D-2026-11-19-01）。
