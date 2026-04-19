# Plan

- title: Public Web 邀请码与公网暴露安全加固
- status: done
- created_at: 2026-04-19 15:35 CST
- updated_at: 2026-04-19 16:25 CST
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/routes/web_users.rs`
  - `crates/hone-web-api/src/types.rs`
  - `crates/hone-web-api/src/state.rs`
  - `memory/src/web_auth.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/pages/settings.tsx`
  - `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
  - `docs/repo-map.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/2026-04-19-web-admin-public-isolation.md`

## Goal

收口 public web 当前已知的公网暴露风险，补齐邀请码暴力尝试防护、邀请码泄露后的止血能力，以及 public 侧 Cookie / CORS 的基础加固，确保 `8088` 暴露到公网时不再明显处于“无防刷、无撤销、无会话清退”的状态。

## Scope

- 为 public 邀请码登录补应用层失败限流 / 冷却机制
- 为邀请码增加禁用 / 恢复、重置邀请码、清理现有会话等管理能力
- 加固 public 会话 Cookie 与跨域暴露面，避免默认对任意来源开放 public API
- 补前端设置页对应管理操作与状态展示
- 为存储和路由补自动化测试，并验证关键管理端 / 用户端路径

## Validation

- `cargo test -p hone-memory web_auth`
- `cargo test -p hone-web-api`
- `cargo check -p hone-web-api -p hone-memory`
- `bun run typecheck:web`
- `bun run test:web`
- 关键手工验证：
  - public 邀请码连续错误尝试会被限流
  - 管理端可禁用邀请码并使既有 public 会话失效
  - 管理端可重置邀请码并清理旧会话
  - public 端登录 Cookie 在 HTTPS 反代头存在时带 `Secure`

## Documentation Sync

- 已更新 `docs/current-plan.md`
- 已更新 `docs/handoffs/2026-04-19-web-admin-public-isolation.md`，补充本轮安全加固结论
- 已归档到 `docs/archive/plans/public-web-security-hardening.md` 并更新 `docs/archive/index.md`
- 若模块边界或部署建议有变化，补充 `docs/repo-map.md`

## Risks / Open Questions

- public 登录限流当前只在应用内存态生效，进程重启后会清空；公网长期暴露仍建议在反向代理 / WAF 层补 IP 级限流
- Cookie `Secure` 依赖 `Origin` / `Referer` / `X-Forwarded-Proto` 等请求头判断；若反代层不透传 HTTPS 语义，可能退化为非 `Secure` cookie
- public CORS 已默认收紧为同源；若未来要支持跨域嵌入或独立前端部署，需要重新明确允许源策略
