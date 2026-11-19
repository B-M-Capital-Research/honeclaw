# Handoff: 主对话 Harness System Prompt 瘦身与下沉到 Skill / User Prompt

- title: 主对话 Harness System Prompt 瘦身与下沉到 Skill / User Prompt
- status: done
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
  - `docs/decisions.md` (D-2026-11-19-01)
  - `docs/archive/plans/harness-prompt-slim-and-skill-offload.md`
  - `docs/archive/index.md`
- related_prs: N/A (local working tree)

## Summary

按用户决策完成主对话 Harness 的 system prompt 瘦身：主对话 `system_prompt` 置空，原 `soul.md` 与 `prompt.rs` 中的硬编码领域策略下沉到 Skill 按需加载，其余运行时/按轮上下文全部挪入 user prompt；不动 `dedicated_earnings_workflow`、Event Engine 与 Web API 独立管道。

## What Changed

1. **System Prompt 置空与 User Prompt 迁移**：
   - 清空 `soul.md`。
   - `crates/hone-channels/src/prompt.rs` 移除 6 个巨型领域策略常量（`DEFAULT_FINANCE_DOMAIN_POLICY`、`DEFAULT_HARI_INVEST_POLICY`、`DEFAULT_CRON_TASK_POLICY`、`DEFAULT_USER_INFO_BOUNDARY_POLICY`、`DEFAULT_WEB_CRON_DELIVERY_POLICY`、`DEFAULT_COMPANY_PROFILE_POLICY`）。
   - `PromptBundle` 新增 `turn_guidance: String` 字段；`build_prompt_bundle_at` 固定输出空 `static_system`，将渠道排版、群聊隐私守卫、语言要求、基础模型提示、管理员提示与 `extra_sections`（`SkillTool`、命中的 `company_research_baseline` / `industry_baseline`）收集到 `turn_guidance` 并由 `compose_user_input` / `compose_runtime_input`（及 native Codex turn 组装路径）拼入 user prompt。
   - `crates/hone-channels/src/turn_builder.rs` 不再每轮注入 cron / web cron 策略。
   - `crates/hone-channels/src/runners/codex_acp.rs` 在 `developer_instructions` 为空时不写入空字符串的 `developer_instructions` 字段。
2. **领域策略下沉到 Skills**：
   - 新建 `skills/finance-research-core/SKILL.md`：承接通用投研底座、领域边界、任务路由 A–F、估值/辩证/输出/工具纪律、用户信息汇总边界与产品能力全景。
   - 补全 `skills/hari-invest/SKILL.md`（研究深度与 1500–4000 字篇幅下限、`laowang-investment-distiller` 隔离等）。
   - 补全 `skills/company_portrait/SKILL.md`（相似公司画像与宏观叙事一致性、极端风险降温、第一性原理、长答去重约束等）。
   - 补全 `skills/scheduled_task/SKILL.md` 与 `skills/notification_preferences/SKILL.md`（权威工具真相源、`automatic_push` / `disable_all`、Web 送达边界、`get_overview` 与 `set_quiet_hours` 规则）。
3. **测试与示例对齐**：
   - 更新 `prompt.rs`、`agent_session/tests.rs`、`examples/finance_consistency_llm_smoke.rs` 与 `tests/regression/ci/test_finance_automation_contracts.sh`。

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`：通过。
- `cargo test -p hone-channels --lib`（连接本地 PostgreSQL）：全量通过。
- `rustfmt --edition 2024 --config skip_children=true --check`（本次修改的所有 Rust 文件）：通过。
- `tests/regression/ci/test_finance_automation_contracts.sh`：所有涉及 `soul.md` / `prompt.rs` -> `skills/finance-research-core/SKILL.md` 的契约项均已对齐；脚本中其它与 `skills/market_analysis/SKILL.md` 及 `agents/function_calling` 相关的既有失败项与本次改动无关，未改动。

## Risks / Follow-ups

- 主对话 `system_prompt` 置空后，金融投研纪律依赖模型按需加载 `finance-research-core`、`hari-invest` 等 Skill；建议在真实对话中抽样观察 `discover_skills` / `skill_tool` 触发率，如发现某类问法漏加载，可微调对应 Skill 的 `description` / `when_to_use` / `aliases`。
- 后续若需接入精简的自定义 system prompt，可直接填回 `soul.md` 并在 `build_prompt_bundle_at` 恢复读取 `config.agent.system_prompt`。

## Next Entry Point

- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `skills/finance-research-core/SKILL.md`
