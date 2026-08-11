- title: 本地用户端免短信测试登录
- status: archived
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files: `crates/hone-web-api/src/routes/public.rs`; `crates/hone-web-api/src/routes/mod.rs`; `packages/app/src/components/public-login-form.tsx`; `packages/app/src/lib/api.ts`; `packages/app/src/lib/public-content.ts`
- related_docs: `docs/runbooks/source-web-startup.md`; `docs/handoffs/2026-08-10-local-public-dev-login.md`; `docs/decisions.md#d-2026-08-10-02-make-local-dev-login-explicit-server-owned-and-fail-closed`

## Goal

让未配置短信或邮箱服务的本地源码环境可以通过正常后端会话直接进入用户对话页并测试功能，不在生产登录链路中加入通用后门。

## Scope

- 新增本地测试登录状态与登录接口。
- 只有 `HONE_PUBLIC_DEV_LOGIN=true`、deployment mode 为 local、cloud mode 为 local 三个条件同时满足时开放。
- 前端仅在服务端明确返回 enabled 时显示“进入本地测试账号”按钮。
- 使用正常 HttpOnly Cookie、用户记录、TOS 版本与会话存储；短信登录保持不变。

## Validation

- 后端本地门禁单元测试：通过。
- 前端本地登录契约测试：3/3 通过；TypeScript 通过。
- 实际 3001 代理链路：config enabled、POST 登录、HttpOnly Cookie、`/auth/me` 回读均通过。
- 3000、3001、8077、8088 均监听；只有 Web 渠道运行。

## Documentation Sync

- 已更新 `docs/runbooks/source-web-startup.md`、decision、handoff 与 archive index。
- 本计划已退出活跃索引并归档。

## Risks / Open Questions

- 该能力不得在 cloud/remote deployment 生效；默认关闭。
- 本地测试账号固定为非生产身份，不依赖真实手机号、验证码供应商或生产会员状态。
