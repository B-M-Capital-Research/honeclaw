# Hone 个人交易管理工具工程交接

- status: done — 已合入主干并部署上线（2026-11-20）
- created_at: 2026-11-16
- updated_at: 2026-11-20
- owner: Codex / Hone engineering
- upstream_base: f7d87aff034507a529ccf34b84de2aff7879f5ec
- related_files: `routes/trade_manager.rs`, `routes/trade_manager/`, `packages/app/public/trade-manager/`, `components/trade-manager-dialog.tsx`, `lib/public-auth-lifecycle.ts`, `lib/trade-manager-session.ts`
- related_docs: [接入手册](../runbooks/trade-manager.md), [API 契约](../runbooks/trade-manager-api.md), [迁移](../sql/trade-manager-v1.sql)

## 交付结果

普通用户工具菜单新增交易管理，复用 Hone 登录与访问权限。交易台原有核算保留，个人财务数据移至独立 PG 账本。服务端从认证用户决定 actor，客户端不传 userId；会话绑定阻止旧页面在账号切换后串写，revision CAS 防并发覆盖，当前与此前 8 版本可追溯恢复。

所有登录方式在开始和完成时广播无身份的失效通知；退出、身份变化和旧请求晚回均清除或拒绝旧数据。工具内“返回 Hone”走父弹窗关闭保护，不另建一套退出登录流程。导出经过当前身份复核后，由同源父弹窗提供用户可点击的临时原生下载链接；旧链接随关闭、切换或到期销毁。

## 验证及边界

- Rust 后端：10 项测试通过；其中 3 项使用全新本地 PostgreSQL 集群和两个真实数据库连接，覆盖 A/B 隔离、历史 ID 归属、并发初次/后续保存、幂等和恢复窗口。其余覆盖会话绑定、拒绝客户端身份、响应 no-store、V1/V1.1 数据兼容及迁移一致性。
- JavaScript：65 项通过（模型 30、报价/报告 15、异步认证存储 18、认证失效消息 2）。包括 401/402/403、409、服务故障、UTF-8 大小、失效后的迟到 JSON/保存响应和待保存副本；区分正常重新加载与真正认证失效，后者同时销毁父窗口的下载链接。
- Hone Web：693 项通过，0 失败；类型检查和公共前端构建通过。构建有既有大分块提醒，没有阻断错误。交易模块 JavaScript 语法及 git diff 空白检查通过。
- 浏览器：实际 Hone 公共前端 + **模拟 API**，普通虚构账号从原生工具菜单打开，父弹窗内编辑保存成功，dirty 关闭提示、503 未保存提示、A/B 独立空账本和旧 A 页面拒绝写入已观察。同源父弹窗的原生链接实际下载 JSON 成功；一个标签页通过 Hone 原入口退出后，另一页已打开的交易工具立即卸载并回到登录页。浏览器验收没有使用真实 Hone 账户；模拟 API 不随生产代码部署。
- 完整 AppState + 真正登录 cookie + 生产数据库的端到端认证流程未启动；上线前必须在工程测试环境完成 runbook 的真实账号及反向代理验收。
- 原 Hone 工作树、运行服务与线上站点均未修改；无订单、模型付费请求或真实金融数据操作。

## 工程合并要点

1. 目标分支可能比公开快照新，请用补丁精确合并，不整体覆盖 chat.tsx、api.ts、routes/mod.rs 等共享文件。
2. API 与静态工具必须一起发布；数据库先迁移，DML 应用角色无需扩大 DDL 权限。
3. Pages worker 的同源可嵌入例外仅适用于固定工具入口；其他反向代理需提供等价 CSP。
4. 账本与既有 Hone portfolio /研究画像不自动同步。若后续打通，先定义唯一数据源、币种与流水口径，不做双向盲写。
5. 回滚入口和代码即可，保留数据库表及备份。恢复历史是有限窗口，不是不可删的法定审计账簿。

## 2026-11-20 · 合入当前主干（HEAD `e899a342`）验证记录

- 补丁合入：36 个代码与文档文件无冲突应用，`docs/archive/index.md` 与 `docs/decisions.md` 已按当前主干手工合并；未改动工作区既有未提交变更。
- 实测结果：
  - `node --test tests/trade-manager/*.test.mjs`：65/65 通过
  - `bun run test:web`：全部通过（含新增 `trade-manager-session`、`public-auth-lifecycle`、`public-session-cache`、`_worker.test.js`）
  - `bun run typecheck:web`、`bun run build:web:public`：通过，`packages/app/dist-public/trade-manager/` 产物完整
  - `cd workers/public-community-edge && bun run typecheck && bun run test`：45/45 通过
  - `rustfmt --edition 2024 --config skip_children=true --check`（改动 Rust 文件）：通过
  - `docs/sql/trade-manager-v1.sql`：在本机 `honeclaw_test` PostgreSQL 执行建表成功
  - `cargo test -p hone-web-api --lib trade_manager -- --test-threads=1`：10/10 通过（含 3 项真实双连接 PG 隔离/CAS/幂等测试）
  - `cargo test -p hone-web-api --lib`：417 passed, 0 failed, 4 ignored
  - `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`：通过
- 上线前置：按 `docs/runbooks/trade-manager.md` 在生产库先执行 `docs/sql/trade-manager-v1.sql`（若应用角色无 DDL 权限），再同步发布后端与公共前端，并在测试/线上完成真实账号与反向代理验收。

## 2026-11-20 · 生产部署与线上验收记录

- 提交：
  - 主功能合入：`584ba8e169b0c93a19d5691668a03a3a594159d7`
  - Cloudflare Pages 规范路径修复：`2a2a5af6bed8961337a6afbfa3d4bd0f9602c9f9`（Cloudflare Pages 会将 `/trade-manager/index.html` 308 规范重定向至 `/trade-manager/`；已将 iframe `src` 改为 `/trade-manager/?embedded=1`，并在 `onLoad` 同时放行 `/trade-manager/` 与 `/trade-manager/index.html`）
- 数据库迁移：已在生产 PostgreSQL 执行 `docs/sql/trade-manager-v1.sql`，`cloud_trade_workspaces` 与 `cloud_trade_revisions` 建表完成。
- 后端部署：
  - GitHub Actions `Runtime Image`（run `44654584101`）构建成功，不可变镜像 digest `sha256:61f8b0ba547a45ba4d6f9c2f5ff22e7140e8e654a2b68c43bb478bff559137bb`
  - 托管后端主机已暂存校验并原子切换 `/opt/hone/current` 至 `584ba8e169b0c93a19d5691668a03a3a594159d7-ghcr-runtime`（`/opt/hone/previous` 保留 `e899a34216470155b38a0c3a3b9e55a0b22355a4-ghcr-runtime` 供即时回滚）
  - `/api/meta` 确认 `git_sha=584ba8e169b0c93a19d5691668a03a3a594159d7`、`binary_sha256=ca6923dfa9e763b7fb000ec998584e3a2a949a968cf18d04a0357623f2247535`、`cloud_postgres_health.ok=true`、`cloud_oss_health.ok=true`、`local_durable_dependency_count=0`
- 前端部署与公网探针（`https://hone-claw.com`）：
  - `/chat` 入口已由部署前 `/assets/index-CsXXIRTt.js` 更新至 `/assets/index-DoQJKLeO.js`（`chat-DY4DcmqF.js` 已包含 `/trade-manager/?embedded=1`）
  - `GET /`：`200`，保持 `Content-Security-Policy: frame-ancestors 'none'`、`X-Frame-Options: DENY`、`Strict-Transport-Security: max-age=31536000`
  - `GET /trade-manager/?embedded=1`：`200`，`Cache-Control: no-store`、`Content-Security-Policy` 含 `frame-ancestors 'self'`、`X-Frame-Options: SAMEORIGIN`
  - `GET /api/public/trade-manager/state`（未登录）：`401`，`Cache-Control: no-store`、`Pragma: no-cache`、`Vary: Cookie`、`{"error":"未登录"}`
- 后续人工确认：请使用真实账号登录 `https://hone-claw.com/chat`，在工具箱中打开“交易管理”做一轮保存/刷新/换号隔离实测。
