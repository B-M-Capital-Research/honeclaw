# Handoff

- title: Web 邀请码手机号绑定与固定端口切换
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: Codex
- related_files:
  - bins/hone-desktop/src/sidecar.rs
  - bins/hone-desktop/src/sidecar/runtime_env.rs
  - crates/hone-web-api/src/lib.rs
  - crates/hone-web-api/src/runtime.rs
  - crates/hone-web-api/src/routes/common.rs
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-web-api/src/routes/web_users.rs
  - crates/hone-web-api/src/types.rs
  - memory/src/web_auth.rs
  - packages/app/src/lib/api.ts
  - packages/app/src/lib/types.ts
  - packages/app/src/pages/chat.tsx
  - packages/app/src/pages/settings.tsx
- related_docs:
  - docs/archive/plans/web-invite-phone-and-fixed-ports.md
  - docs/archive/index.md
  - docs/runbooks/desktop-release-app-runtime.md
- related_prs:
  - N/A

## Summary

已完成 Web 邀请码与手机号绑定改造，管理端生成邀请码时必须填写手机号，用户端登录时必须同时提交邀请码与手机号；同时修复 bundled desktop 下管理端随机端口问题，当前 release app 已切到固定管理端 `8077`、用户端 `8088`。

## What Changed

- `hone-web-api` 管理端默认端口改为 `8077`，用户端默认端口改为 `8088`；desktop bundled 启动链不再移除 `HONE_WEB_PORT`，并在运行时默认补齐 `HONE_WEB_PORT/HONE_PUBLIC_WEB_PORT`
- `memory::web_auth` 新增 `phone_number` 持久化列，兼容老库增量迁移；邀请码创建必须携带手机号，登录必须同时匹配邀请码与手机号
- 管理端设置页新增手机号输入框和手机号列；用户端 `/chat` 登录卡新增手机号输入
- 按 runbook 重建前端、release app，并清理旧 `hone-*` 进程与锁文件后切换到新的 `.app` runtime

## Verification

- `cargo test -p hone-memory web_auth`
- `cargo test -p hone-web-api`
- `cargo check -p hone-web-api -p hone-memory`
- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web`
- `bun run build:web:public`
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build`
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- 运行态验证：
  - `curl http://127.0.0.1:8077/api/meta` 返回正常，最新日志显示管理端端口为 `8077`
  - `curl http://127.0.0.1:8088/api/public/auth/me` 返回 `401 {"error":"未登录"}`
  - `curl -I http://127.0.0.1:8088/chat` 返回 `200 OK`
  - `POST /api/web-users/invites` 可成功创建绑定手机号的邀请码
  - 错手机号登录返回 `401`，正确手机号登录返回 `200` 并建立 `hone_web_session`

## Risks / Follow-ups

- 旧数据里的历史邀请码因为迁移前没有手机号，当前会以空手机号展示；如需继续使用，需重新生成一组带手机号的新邀请码
- 当前 desktop runtime 下 `discord`、`feishu` 正常，`telegram` 仍为 `degraded`，根因不是本次代码改动，而是 `hone-telegram.manual.log` 明确报出 `Invalid bot token`。若要让 Telegram 渠道稳定在线，必须先修正配置里的 bot token
- 当前工作区仍有不少与本任务无关的未提交改动，本次未做回滚或整理

## Next Entry Point

- 端口与 public 登录链路：`crates/hone-web-api/src/routes/public.rs`
- 邀请码手机号存储：`memory/src/web_auth.rs`
- 桌面 bundled 启动与端口默认值：`bins/hone-desktop/src/sidecar.rs`
