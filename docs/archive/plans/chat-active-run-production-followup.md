- title: Web 活动任务修复生产前端漏部署跟进
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - packages/app/src/pages/chat.tsx
  - packages/app/src/lib/public-chat.ts
  - crates/hone-web-api/src/routes/public.rs
  - bins/hone-cli/src/start.rs
  - packages/app/dist-public
- related_docs:
  - docs/archive/plans/chat-active-run-ux.md
  - docs/handoffs/2026-07-16-chat-active-run-recovery.md
  - docs/runbooks/backend-deployment.md

## Goal

修复 active-run 源码和后端已更新、但真实公网仍加载旧 `dist-public` 的发布漏项，确保 `hone-claw.com/chat` 与 8088 使用同一套活动任务恢复协议，并用生产资产而不是源码状态判定部署完成。

## Scope

- 对齐源码、`packages/app/dist-public`、8088 与 Cloudflare Pages 四层资产。
- 修复慢 bootstrap 轮询互相取消、恢复期间重复发送、无 terminal EOF 永久 pending 和动态状态缓存风险。
- 让 CLI 子进程隔离终端 SIGINT，保证受控重启先查询并排空 Web 活动任务。
- 增加生产 hash/chunk 验证门禁，禁止仅凭后端重启或通用 `dist` 构建宣称公网已更新。

## Verification

- 原始真实 RMBS run `7366363c-384b-4510-a014-4fc35ca3a5b1`：92.492 秒成功落库、3323 字符；故障期间后台活动数最终为 0，证明旧页面显示的是假 pending。
- Web 263 passed；typecheck 和 `build:web:public` 通过；最终本地/8088/生产入口均为 `index-DmyhjLnz.js`。
- 生产 lazy chunks 为 `chat-B6liblxH.js` 与 `public-chat-LkMkttVo.js`，包含 `active_run`、`interrupted_run`、`run_progress`、`started_at_ms`，不含旧 `pendingSince/recoveredPending` 分支。
- Web API 113 passed、2 ignored；CLI start 14 passed；active state 双端 `no-store` 回归通过。
- 最终 supervisor `9304` 启动后，Web/Discord/Feishu、Postgres/S3、8077/8088、Cloudflare Worker 401 JSON 与活动数 0 均健康；子进程 PGID 与 supervisor 分离。
- 生产提交：`335c4b73`、`92d776a8`、`4aa21b29`。

## Documentation Sync

- 更新既有 handoff、archive index 与部署 runbook；本计划从 active index 移除并归档。

## Risks / Open Questions

- active-run registry 仍是单 Web 进程真相源；多实例前必须采用共享 lease/fencing 或 sticky ownership。
- 静态前端、Worker 和 backend origin 是独立发布层，后续协议变更必须继续执行三层资产/健康核验。
