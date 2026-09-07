# 3D 数据中心在线试玩入口部署

- title: 3D 数据中心在线试玩入口部署
- status: done
- created_at: 2026-09-07
- updated_at: 2026-09-07
- owner: Codex
- related_files: packages/app/src/pages/chat.tsx, packages/app/e2e/public-data-center.spec.ts
- related_docs: docs/repo-map.md, docs/runbooks/backend-deployment.md, docs/archive/plans/data-center-online-demo-deploy.md
- related_prs: https://github.com/B-M-Capital-Research/honeclaw/commit/00ea36e01b8d35d4fbd0912fd6a836ce83c1b09b

## Summary

聊天框桌面按钮与手机工具菜单的「3D 数据中心」在新标签页打开官方试玩 https://b-m-capital-research.github.io/nexus-datacenter-ceo/ ，保持聊天页面和草稿。研究台原有本地数据中心页面仍可访问。

## What Changed

- 代码 revision：`00ea36e01b8d35d4fbd0912fd6a836ce83c1b09b`；只更新前端，没有后端代码改动、服务重启或数据迁移。
- 构建从该 revision 的干净独立 checkout 执行 `bun install --frozen-lockfile && bun run build:web:public`，community edge discovery 保持默认 `0`。
- 源站 immutable bundle：`/opt/hone/public-web/releases/00ea36e01b8d35d4fbd0912fd6a836ce83c1b09b/dist-public`。
- 完整 672 文件清单留存在同 release 的 `BUILD_MANIFEST.json`，HTTP/进程验收留存在 `DEPLOYMENT_VERIFICATION.json`，两者均不在 served directory。
- Manifest SHA-256：`8256520923aa75d44c53687d620ea91812dc8842cef3ab2afa58c383638f8338`。
- 传输 archive SHA-256：`11ab0a1ac9abebb582c7adc4b33d82d60bcc837294b939ce51ab472b5c673de5`。
- index SHA-256：`5af75daba84ae054e6ee268d2a577bb190a5855fd5bb5df81d2b05d9fbb3cb98`。
- JS entry：`/assets/index-D0P1zv0l.js`，SHA-256 `92e57731446ea1680fc6cafd6bcb532ed62c13eb5f2a0fd8ccb6e7373b70759a`。
- 修改的 chat chunk：`/assets/chat-BVstabPq.js`。

## Verification

- `bun run typecheck:web` 通过；`bun run test:web` 646 passed。
- `bun run build:web:public` 通过。
- `bun run test:e2e public-data-center.spec.ts --project=public`：7 passed，2 failed。桌面 1440px 和手机 390px 新标签、noopener、草稿保留回归均通过。两项失败在直接访问旧行业详情时，原断言的「光通信」「电力」与现有 ontology 的「光通信与 AI 互连」「电力与供电」不一致；未修改无关断言。
- `bash tests/regression/run_ci.sh`：runtime env contract、billing contract 通过，随后 billing HTTP E2E 因未设置 `HONE_POSTGRES_HOST` 停止；不声称全仓门禁通过。无 Rust 改动，本次未重新运行完整 Rust workspace check/test。
- 仓库 pre-push hook 手工执行：1 commit scanned，无密钥命中。原 GitHub SSH 22 超时后，通过已登录 `gh` 的 HTTPS credential helper 推送同一 main 提交。
- 源站完整 archive/manifest/file hash 核对、deploy lock、previous symlink 比较、原子切换与 HTTP index/entry SHA 核对全部通过；managed service PID 与启动时间前后一致。
- Cloudflare Pages deployment `1566a76e-f3d7-4590-8032-c50291e6e0fb` 成功，对应上述精确 revision；GitHub `frontend-checks` 与 `gitleaks` 均成功。Rust CI 在本任务验收时仍在运行。
- Pages 独立构建的 entry `/assets/index-SI3XfZm6.js`，SHA-256 `70ca644828f7cea8cc00e33eac681d5ed0a374311808b0fd31b6e0955ce58c7d`；chat chunk `/assets/chat-DNowxEDg.js`，SHA-256 `0bc4d1ca5fe112a2dfb4602499932b0a9501047103a86660b94d95c6a380d3f9`。直接下载验证目标试玩 URL 与 `noopener,noreferrer` 均存在。Pages/local 构建的 chunk 文件名不同，分别留存各自产物哈希；源站 HTTP 响应逐字节匹配本地精确 build。
- 线上 `/`、`/chat`、`/roadmap` 均成功；HSTS `max-age=31536000`、CSP `frame-ancestors 'none'`、`X-Frame-Options: DENY`、`nosniff`、`strict-origin-when-cross-origin` 检查通过。
- 真实 Chrome 已登录会话刷新后加载新版 Pages entry，实际点击「3D 数据中心」，新标签地址精确为官方试玩，标题「NEXUS · 算力纪元」、页面「CEO 办公室」可见，HONE 原标签仍为 `/chat`。未发送聊天消息或修改业务数据。

## Risks / Follow-ups

- 外部试玩由独立 GitHub Pages 仓库提供可用性。
- 上述两项旧 E2E 名称断言和缺失 PostgreSQL 的完整回归未在本任务中处理。
- 源站上一版：`/opt/hone/public-web/releases/6cf77eaabce9408e7664e89857ade71b607ceea3/dist-public`；Pages 上一版对应 `08a7ee271a0bc03860892fd1e0e8e1a3626c6488`，entry `/assets/index-DPz-CbDr.js`。回滚依 runbook，在 deploy lock 下确认当前 revision 后原子恢复静态资源 symlink，并在 Pages 恢复上一成功版本；无需切换后端 binary。

## Next Entry Point

任务已上线并归档。入口将来变更时同步此跳转回归；若需回滚，按上述静态资源回滚入口操作。
