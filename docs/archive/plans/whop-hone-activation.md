# Whop → HONE International Activation

- title: Whop → HONE 国际邮箱激活与付费权益
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files:
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/state.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/whop.rs`
  - `packages/app/src/pages/public-whop-activate.tsx`
  - `packages/app/src/pages/public-me.tsx`
- related_docs:
  - `docs/decisions.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/proposal/auto_p2_self-serve-billing-checkout.md`

## Goal

让 Whop 国际渠道买家不依赖手机号或 Whop 登录：HONE 从已验证的
Whop membership webhook 记录付款邮箱与权益，用户通过 HONE 自己的邮箱验证码
完成账号激活；国内手机号邀请与短信登录契约保持不变。

## Scope

- 在现有 Web 用户旁新增外部身份状态，不破坏已有手机号用户与 session。
- 验证 Standard Webhooks HMAC、时间窗、business/product/plan 和事件顺序。
- 处理 `membership.activated`、`membership.deactivated` 与
  `membership.cancel_at_period_end_changed`。
- 新增邮箱验证码 sender 接口、challenge 存储与激活 API；默认 sender 明确未配置。
- 新增 `/activate/whop`，并让 `/me` 展示真实 Whop 状态而非把所有邀请用户视为付费会员。
- Whop 原生 Discord app 继续独立负责 Discord 身份绑定和 VIP role。
- 本阶段不配置生产 webhook、不实现邮件供应商、不处理知识星球自动同步。

## Validation

- `cargo test -p hone-memory web_auth`
- `cargo test -p hone-web-api whop`
- `bun run test:web`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bash tests/regression/run_ci.sh`
- `bash scripts/ci/check_fmt_changed.sh`
- 启动本地后端与 public Vite 前端，在浏览器验收 `/activate/whop`、
  手机登录入口、未配置邮件 sender 错误态，以及 active/inactive `/me` 会员视图；
  真实邮件送达留待 sender 接入后验收

完成结果：

- 全 workspace check/test、Web `292/292`、Worker `45/45`、CI-safe regression、
  public build、显式 rustfmt 和 diff check 全部通过。
- 隔离端口 `18077` / `18088` / `13001` 的当前代码通过桌面与 `390x844`
  浏览器验收；浏览器控制台无 warning/error。
- 签名 webhook 的 create / duplicate / tamper reject / deactivate 通过真实 HTTP
  验收；inactive session 的 `/auth/me` 为 `200`，付费 `/history` 为 `402`。
- 邮件 sender 按范围保持未配置，真实邮件送达明确留到供应商接入阶段。

## Documentation Sync

- 更新 `docs/repo-map.md`：公开身份、Whop webhook、激活页和 `/me` 数据流。
- 更新 `docs/invariants.md`：国内手机号与国际 Whop 邮箱身份边界、禁止回跳参数授予权益。
- 更新 `docs/decisions.md`：扩展 D-2026-07-26-04，明确 Discord 与 HONE 权益边界。
- 完成后写 `docs/handoffs/whop-hone-activation.md`，更新 `docs/archive/index.md`，
  并把本计划移入 `docs/archive/plans/`。

## Risks / Open Questions

- 邮件 sender 默认未配置，真实激活在接入事务邮件服务前保持不可用。
- 当前只消费 membership 生命周期；退款、争议和定期 reconciliation 仍需后续实现。
- Whop 外部配置需要 company API key 与 webhook secret，不能使用当前无
  `developer:manage_webhook` scope 的 CLI OAuth token 代替。
- 真实上线仍需非 owner 买家完成购买、激活、取消、撤销与恢复验收。
