- title: 活跃计划清理
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - `docs/current-plan.md`
  - `docs/archive/plans/desktop-channel-status-multiprocess.md`
  - `docs/archive/plans/desktop-runtime-startup-locks.md`
  - `docs/archive/plans/desktop-startup-lock-ux-strategy.md`
  - `docs/archive/plans/windows-desktop-packaging.md`
  - `docs/archive/plans/file-upload-tracking.md`
  - `docs/archive/plans/report-command-bridge.md`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

清理了 `docs/current-plan.md` 中已经不适合作为“活跃任务”继续暴露的 6 个计划。判断标准不是“主题不重要”，而是这些条目已经落入以下两类之一：一类是主体已经落地，只剩模糊的验证尾巴；另一类是长期没动且计划本身仍停留在占位态，继续留在活跃列表会稀释真正正在推进的任务。

## What Changed

- 将以下计划移入 `docs/archive/plans/`，并统一改成 `status: archived`：
  - `desktop-channel-status-multiprocess.md`
  - `desktop-runtime-startup-locks.md`
  - `desktop-startup-lock-ux-strategy.md`
  - `windows-desktop-packaging.md`
  - `file-upload-tracking.md`
  - `report-command-bridge.md`
- 更新 `docs/current-plan.md`，把活跃任务数从 10 收口到 4，只保留最近仍有实际推进的计划。
- 在 `docs/archive/index.md` 追加这次清理的索引入口，说明哪些计划是因“过时/失焦”而归档，而不是因为完整交付已经结束。

## Verification

- `docs/current-plan.md` 只保留 4 个仍在推进的计划
- 原 6 个过时条目都已移入 `docs/archive/plans/`
- 每个归档计划都明确写出归档原因与重启入口

## Risks / Follow-ups

- 这些主题里有些仍然重要，但下次重启时应以更窄、更具体的任务重新开计划，而不是直接把旧占位计划搬回活跃列表。
- 如果某个归档主题很快重新开工，应优先在旧归档计划基础上复制或重写一份新的活跃计划，避免直接把过时描述重新标成 `in_progress`。

## Next Entry Point

- `docs/archive/index.md`
