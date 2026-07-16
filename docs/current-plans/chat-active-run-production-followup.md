- title: Web 活动任务修复生产前端漏部署跟进
- status: in_progress
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - packages/app/src/pages/chat.tsx
  - packages/app/src/lib/public-chat.ts
  - packages/app/dist-public
  - docs/runbooks/backend-deployment.md
- related_docs:
  - docs/archive/plans/chat-active-run-ux.md
  - docs/handoffs/2026-07-16-chat-active-run-recovery.md
  - docs/current-plan.md

## Goal

修复 active-run 源码已部署后端但真实公网仍加载旧 `dist-public` 的发布漏项，确保 `hone-claw.com/chat` 与 8088 实际静态包包含 `active_run/started_at_ms/run_progress`，并对当前 RMBS 长请求的等待与刷新恢复做生产证据验证。

## Scope

- 对齐源码、`packages/app/dist-public`、8088 与 Cloudflare Pages 四层资产哈希。
- 运行正确的 public build，验证生成 chunk 含服务端活动任务恢复逻辑。
- 按仓库 runbook 发布公网前端，不重跑或中断已经完成的真实用户请求。
- 增加部署门禁，避免仅验证源码/通用 `dist` 就误报 public bundle 已部署。

## Validation

- `bun run test:web`、`bun run typecheck:web`、`bun run build:web:public`。
- 新 `dist-public`、8088 与生产 `/chat` asset hash 一致，且 chunk 含 active-run 协议、不含旧 `in_flight + Date.now()` 恢复分支。
- 真实/隔离 RMBS 长请求等待期展示 progress，刷新前后 `run_id` 与 `started_at_ms` 不变，最终只出现一次回答。
- Web/Discord/Feishu 与 active-run endpoint 健康。

## Documentation Sync

- 更新既有 handoff，明确本次生产漏部署与资产验证门禁。
- 完成后归档本计划并更新 `docs/archive/index.md`、`docs/current-plan.md`；若发布流程变化，更新 `docs/runbooks/backend-deployment.md`。

## Risks / Open Questions

- Cloudflare Pages 正常通过 production branch 自动发布；若当前环境没有 direct-upload 凭据，不得假称公网已更新。
- 工作区包含同一系列尚未提交的投资管线与 active-run 变更，不能用破坏性 Git 操作或丢弃用户改动。
