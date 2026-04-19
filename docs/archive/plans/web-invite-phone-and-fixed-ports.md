# Plan

- title: Web 邀请码手机号绑定与固定端口切换
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: Codex
- related_files:
  - bins/hone-desktop/src/sidecar.rs
  - crates/hone-web-api/src/lib.rs
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-web-api/src/routes/web_users.rs
  - crates/hone-web-api/src/types.rs
  - memory/src/web_auth.rs
  - packages/app/src/lib/api.ts
  - packages/app/src/lib/types.ts
  - packages/app/src/pages/chat.tsx
  - packages/app/src/pages/settings.tsx
- related_docs:
  - docs/archive/index.md
  - docs/current-plan.md
  - docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md
  - docs/runbooks/desktop-release-app-runtime.md

## Goal

修复 Web 管理端与用户端端口不固定的问题，确保 bundled desktop 启动后始终使用管理端 `8077` 与用户端 `8088`；同时将 Web 邀请码改为与手机号强绑定，管理端发码时必须填写手机号，用户端登录时必须同时填写邀请码与手机号。

## Scope

- 修复 desktop bundled 启动链与 `hone-web-api` 默认绑定逻辑，消除管理端随机端口
- 扩展 Web 邀请码存储结构，持久化手机号并要求登录双因子匹配
- 更新管理端邀请码生成/展示逻辑与用户端登录表单
- 按 runbook 重新构建桌面 App，停掉现有所有 `hone-*` 进程与锁，切换到新服务

## Validation

- `cargo test -p hone-memory web_auth`
- `cargo test -p hone-web-api`
- `cargo check -p hone-web-api -p hone-memory`
- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web`
- `bun run build:web:public`
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build`
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- 运行态验证：`8077` 管理端、`8088` 用户端、手机号+邀请码登录链路、旧 `hone-*` 进程已清理并切到新的 `.app` runtime

## Documentation Sync

- 更新 `docs/current-plan.md`、`docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md` 与 `docs/archive/index.md`
- 任务已完成，本计划页已移入 `docs/archive/plans/`

## Risks / Open Questions

- 当前工作区已存在多处与本任务无关的未提交改动，实施时只能局部修改，不能回滚他人变更
- Web 邀请码 SQLite 通过增量列迁移兼容了既有数据，但历史邀请码会以空手机号保留，后续若需继续使用应重新发码
- 桌面 bundled 启动链对环境变量和锁文件仍然敏感；本轮已经按 runbook 清理并切换，但 Telegram 渠道额外暴露出 `Invalid bot token` 配置问题，后续需由有效凭证修复
