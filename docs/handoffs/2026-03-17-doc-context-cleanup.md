# 2026-03-17 文档计划与交接清理

## 本次完成

- 将 `docs/current-plan.md` 收回为“只保留活跃任务索引”的形态，移除历史流水账式完成记录。
- `docs/current-plans/` 仅保留活跃任务计划与 `README.md`；已完成计划页在 handoff 承接后统一删除。
- 按主题合并零碎 handoff，重点收口到：
  - `2026-03-08-llm-audit-and-console-rollup.md`
  - `2026-03-11-desktop-runtime-and-observability-rollup.md`
  - `2026-03-11-imessage-stability-rollup.md`
  - `2026-03-13-prompt-and-channel-rollout-rollup.md`
  - `2026-03-14-launch-and-runtime-hardening-rollup.md`
  - `2026-03-15-imessage-disable-rollup.md`
- 删除纯过程、纯答疑或已被后续实现覆盖的碎片页，例如 `prompt-loading.md`、`gemini-link.md`、旧 `git-sync` handoff 等。
- 更新 `docs/current-plans/README.md`，明确“已完成计划默认删除、同主题优先合并 handoff”的维护规则。

## 验证

- 逐项检查 `docs/current-plan.md` 中引用的计划页与 handoff 是否存在。
- 检查 `docs/current-plans/` 仅剩活跃计划。
- 抽查 `docs/handoffs/` 中保留的历史主题是否仍有足够上下文，不再依赖被删除的 plan 页。
