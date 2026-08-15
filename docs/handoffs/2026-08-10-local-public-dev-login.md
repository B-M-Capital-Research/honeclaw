- title: 本地用户端免短信测试登录交接
- status: done
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files: `crates/hone-web-api/src/routes/public.rs`; `crates/hone-web-api/src/routes/mod.rs`; `packages/app/src/components/public-login-form.tsx`; `packages/app/src/lib/api.ts`; `packages/app/src/lib/public-content.ts`; `packages/app/src/pages/public-dev-login-contract.test.ts`
- related_docs: `docs/archive/plans/local-public-dev-login.md`; `docs/runbooks/source-web-startup.md`; `docs/decisions.md#d-2026-08-10-02-make-local-dev-login-explicit-server-owned-and-fail-closed`
- related_prs: none; local uncommitted change set

## Summary

本地 public user UI 在未配置短信/邮箱供应商时可以显示“进入本地测试账号”。按钮只在后端确认 `HONE_PUBLIC_DEV_LOGIN=true`、deployment mode 为 local、cloud mode 为 local 后出现，点击后由后端创建或复用本地测试身份并签发正常 HttpOnly 会话 Cookie。

## What Changed

- 新增 `GET /api/public/auth/dev-login/config` 与 `POST /api/public/auth/dev-login`。
- 默认关闭且 fail closed；remote deployment 或 cloud mode 下即使设置环境变量也返回 disabled/404。
- 前端先读取 config，只有 enabled 才渲染本地测试按钮；没有 `document.cookie` 或客户端伪造认证。
- 本地测试登录记录当前 TOS 版本，使用既有 `create_session_for_user` 和 30 天 Cookie；短信/邮箱路径没有变化。
- source startup runbook 增加 `HONE_PUBLIC_DEV_LOGIN=true cargo run -p hone-cli -- start --build`。

## Verification

- `cargo fmt --all -- --check`：通过。
- `cargo test -p hone-web-api public_dev_login_requires_explicit_local_local_enablement -- --nocapture`：1/1 通过。
- `bun test packages/app/src/pages/public-dev-login-contract.test.ts`：3/3 通过。
- `cd packages/app && bunx tsc --noEmit`：通过。
- 实际 Vite 代理验收：config 返回 enabled；POST 返回测试用户并设置 HttpOnly Cookie；同一内存 cookie jar 调用 `/auth/me` 验证成功，TOS 为 2.4，剩余额度为 100。
- 本地端口 3000/3001/8077/8088 均监听；Web 运行，Feishu/Discord/Telegram/iMessage 关闭。

## Risks / Follow-ups

- 不要在生产 service、容器、Pages 或 cloud runtime 中设置 `HONE_PUBLIC_DEV_LOGIN`。
- 如果本地后端被完全停止并重新启动，需继续带上该环境变量；runbook 已记录。
- 本地测试身份只用于功能验证，不代表真实付费会员或生产用户。

## Next Entry Point

打开或刷新 `http://127.0.0.1:3001/chat`，点击“进入本地测试账号”。若按钮不出现，先检查 `/api/public/auth/dev-login/config`，再确认后端启动命令带有 `HONE_PUBLIC_DEV_LOGIN=true` 且 `/api/meta` 的 deployment/cloud mode 都为 local。
