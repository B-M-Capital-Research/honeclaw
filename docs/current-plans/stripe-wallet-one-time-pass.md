# Stripe 支付宝 / 微信单次年费通道

- title: Stripe 支付宝 / 微信单次年费通道
- status: `blocked`
- created_at: `2026-08-06`
- updated_at: `2026-08-13`
- owner: `Codex + owner`
- related_files:
  - `memory/src/billing.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `crates/hone-web-api/src/routes/{billing,stripe}.rs`
  - `packages/app/src/pages/{public-activate,public-me}.tsx`
  - `packages/app/src/lib/{api,types,public-content}.ts`
  - `tests/regression/{ci,manual}/test_stripe_billing_*.sh`
  - `.env.example`
- related_docs:
  - `docs/runbooks/stripe-billing.md`
  - `docs/decisions.md#d-2026-08-04-01-make-stripe-the-only-external-billing-provider`
  - `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`
- verification: focused Billing tests; Stripe signed-event HTTP E2E; real Stripe test-mode Alipay and WeChat Checkout lifecycle; full repository gates; exact GHCR deployment; live Checkout and `/me` browser QA without a real live payment
- risks: one-time wallet payments have no automatic renewal; refund and expiry semantics must fail closed; production payment configuration and keys must never enter Git or logs; live Alipay and WeChat Pay remain unavailable until Stripe completes its external approval

## Goal

在不改变现有 USD 199.99/year 信用卡自动续费订阅的前提下，新增 USD
229.99/12 个月的 Stripe 单次年费通道。该通道在 Stripe 生产审批和实时页面
证明通过后可开放支付宝和微信支付，明确不自动续费，并由服务端验证的 paid
webhook 写入固定期限权益。

## Scope

- 把 Billing 权益从“所有 Stripe 权益都是 subscription”重构为显式的
  `recurring_subscription` 与 `fixed_term_purchase`，不把 PaymentIntent 或
  Checkout Session 伪装成 subscription。
- 保留现有订阅 Checkout、Portal、续费失败宽限、取消与重购行为。
- 新增服务端固定目录的单次 Price 与 Checkout 入口；浏览器不得提交价格、
  币种、权益期限或 provider object ID。
- 支付宝 / 微信 Checkout 使用 `mode=payment`，只有已验证的 paid/async paid
  事件可以激活 12 个自然月；redirect、completed-but-unpaid、failed、expired、
  catalog mismatch 和伪造签名全部 fail closed。
- 固定期限购买在 `/activate` 与 `/me` 中有独立文案、到期日和“不自动续费”
  状态，不提供无意义的 Customer Portal 取消入口。
- 处理重复、乱序、全额退款和到期；任何单次事件只能影响其自己的权益。
- 在 Stripe test/live Payment Method Configuration 中开启 Alipay 与 WeChat Pay；
  生产验收默认停在官方 Live Checkout 授权前，不为技术 smoke 提交真金支付。

## Validation

- 单元测试覆盖 catalog/mode 路由、固定期限计算、leap-day、重复/乱序、支付
  成功/失败、全额退款、到期与订阅隔离。
- `test_billing_http_e2e.sh` 通过真实 HTTP、签名原始 Stripe envelope 与持久
  inbox 证明：未付款不授权、paid 在 30 秒内授权、重放不重复延长、退款撤销、
  两种 entitlement kind 并存而不互相回滚。
- Stripe test mode 的官方 Checkout 页面分别显示 Alipay 与 WeChat Pay、USD
  229.99、one-time，并完成两个真实测试付款生命周期；测试对象全部归档或删除。
- 现有订阅 lifecycle、Portal、失败宽限、恢复、取消与重购回归保持通过。
- 完整 CI 契约通过，精确 revision 由 GHCR `linux/amd64` 镜像部署。
- 外部 Chrome 验收 `/activate` 双产品、两种钱包 Checkout、`/me` 固定期限状态
  与原订阅状态；保留去敏截图，不记录邮箱、密钥、验证码或支付凭证。

## Documentation Sync

- 更新 `docs/repo-map.md` 描述双 entitlement kind 与双 Checkout 数据流。
- 更新 `docs/invariants.md` 固化 paid-webhook、固定期限和订阅隔离约束。
- 在 `docs/decisions.md` 记录自动续费订阅与单次年费的长期产品/架构决策。
- 更新 `docs/runbooks/stripe-billing.md` 的目录、事件、配置、测试、退款和生产
  验收流程。
- 已新增 `docs/handoffs/2026-08-06-stripe-wallet-one-time-pass.md` 并更新
  `docs/archive/index.md`。本计划因 Stripe 钱包外部审批保持 `blocked` 和活跃；
  审批通过并完成最终 live Checkout 验收后再归档并从 `docs/current-plan.md` 移除。

## Risks / Open Questions

- Stripe Dashboard 开启支付方式不代表 subscription Checkout 可用；Alipay 与
  WeChat Pay 必须只进入 one-time `payment` Session。
- 旧表 `provider_subscription_id` 是结构性债务；迁移必须覆盖 PostgreSQL
  中的历史行，并证明现有生产 Stripe 行无损。
- 固定期限延长、全额退款和多次购买必须有严格排序/idempotency 语义，不能按
  webhook 到达顺序累计。
- 支付方式会受账户地区、币种、Payment Method Configuration 与买家地区动态
  过滤；必须用 Stripe API 对 Session 实际配置与官方页面双重验收。
- 公开购买页不得从“期望开启”推导“当前支持”。服务端公开配置必须显式返回
  每个 offer 可宣传的支付方式；支付宝、微信的宣传开关默认关闭，只有 live API
  `available=true` 且新的 live hosted Checkout 实际显示后才能开启。
- 不在未经 owner 再次明确授权的情况下提交 live USD 229.99 支付。

## Current External State

- Live fixed-term Price: `price_1U1M0rEK7h1dD4JHbKBpIkZ2`, active USD 229.99
  one-time under `prod_V0FIIUS22IGljn`.
- Live webhook destination `we_1U0c0XEK7h1dD4JHrvQ9CRaH` listens to the exact
  ten-event contract, including `checkout.session.expired` and
  `charge.refunded`.
- Alipay and WeChat Pay enablement requests are submitted. Both have
  `display_preference=on`, but Stripe currently reports `available=false` and
  the Dashboard shows `pending approval`; this is an external go-live gate.
- Test mode completed official hosted Checkout payments for both Alipay and
  WeChat Pay at USD 229.99 without using production funds.
- Implementation revision `c99babc1e1ea3c54db41256331eb65dcefa7bd1d` is live
  from immutable GHCR digest
  `sha256:dadf8fcf340cf8fa4971605c3f085f7e097efc7cc2c9a8e1ff4a61d757ca90cb`.
- Production `/activate` exposes both offers, and an authenticated live
  fixed-term Checkout showed USD 229.99 with no recurring marker. No live
  payment was submitted.
- The live Checkout currently exposes card only. This correctly matches the
  Stripe API/Dashboard state while Alipay and WeChat Pay remain pending;
  wallet-visible live Checkout acceptance is the only remaining task.
- On 2026-08-13 the live API was read again and both methods still returned
  `available=false`. Revision `b905130158e12138fc1170c7de7e1adb54f0f08d`
  makes the public offer copy server-authoritative and fail-closed: both offers
  advertise card only unless an operator separately enables a proven wallet.
- Exact revision `e4e1e3e9df4296c25b5a8561f303c19efd5ae867` is live from
  immutable GHCR digest
  `sha256:7d43450c4559fbf2a9dcf7d41faaa475627b9dc330f653f8fc18a1651deff351`.
  Production config reports card true and both wallets false for both offers;
  external Chrome showed exactly two card-only claims and no wallet claim.
  Evidence is retained outside Git as
  `30-live-card-only-offer-copy-20260813.png`.
