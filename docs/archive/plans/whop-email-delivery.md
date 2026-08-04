# Whop 购买邮箱真实投递

- title: Whop 购买邮箱真实投递
- status: done
- created_at: 2026-07-28
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - `.env.example`
  - `crates/hone-web-api/src/email_verification.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `crates/hone-web-api/src/routes/whop.rs`
  - `docs/runbooks/whop-hone-activation.md`
- related_docs:
  - `docs/handoffs/whop-email-delivery.md`
  - `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`
  - `docs/decisions.md#d-2026-08-04-01-make-stripe-the-only-external-billing-provider`
- superseded_by: `docs/archive/plans/stripe-whop-parallel-billing.md`

## Goal

把 Whop 购买邮箱验证码从 fail-closed 接口替换为 Cloudflare Email
Sending 的真实事务邮件投递，并用真实收件箱完成购买邮箱激活验收。

## Scope

- 使用 Cloudflare Email Sending REST API，不新增独立事务邮件账号。
- 通过运行时环境变量注入 Cloudflare account ID、最小权限 API token
  与发件人地址。
- 在 Cloudflare 为 `hone-claw.com` 开通发送域，并保留所需 DNS 记录。
- 为请求内容、鉴权头、失败边界和配置缺失补自动化测试。
- 不在代码、文档、日志或 Git 历史中保存 token、验证码或购买邮箱。

## Validation

- `cargo test -p hone-web-api email_verification`
- `cargo test -p hone-web-api whop`
- `cargo check -p hone-web-api --all-targets`
- `bun run test:web`
- `bun run typecheck:web`
- Cloudflare 控制台确认发送域状态正常。
- 浏览器从统一 `/activate?provider=whop` 请求验证码，真实邮箱收到并完成 `/me` 登录。

## Progress

- 用户已明确确认 Workers Paid 的 `$5/month + usage` 订阅，Billing 页面显示
  `Workers Paid Active`。
- `hone-claw.com` Email Sending 显示 `Enabled`、DNS `Configured`、发送信誉
  `Healthy`；公网 DNS 已验证 `cf-bounce` MX/SPF/DKIM 与 DMARC。
- 已创建仅含 `Email Sending: Edit` 的 account-scoped token，并写入本机
  Git 忽略的 `.env`，权限为 `0600`。token 未进入代码、文档或命令输出。
- 真实 Cloudflare 调用暴露 Beta 会返回非空 `message_id` 但省略
  delivered/queued 数组；sender 已兼容该接受态并增加回归测试。
- Cloudflare Activity Log 确认两次真实验证码投递均为 `Delivered`。
- 浏览器在隔离 SQLite membership 上完成
  `/activate?provider=whop` → `/me?checkout=success`，会员状态、周期、邮箱掩码和
  Whop 管理入口均正确。
- Gmail connector 未连接，Chrome 登录的是另一个邮箱；因此没有读取用户
  指定收件箱。浏览器登录阶段使用隔离数据库的等价已知 challenge，真实
  provider 投递与登录表单/后端校验分别完成验收。
- 用户在隔离验收结束后回传了实际收到的验证码，确认真实收件箱已收到
  HONE 邮件；验证码本身未写入代码、文档或日志。由于隔离运行时已清理，
  该验证码未被回放到已经销毁的 challenge。
- Whop 当前签名 secret 使用原始 `ws_...` 格式；verifier 已按当前格式使用
  完整 secret 作为 HMAC key，并明确拒绝旧 `whsec_...` 格式。
- 精确提交 `482c34d54aef4f0d9726acea0b753d751a5973be` 已构建为
  `target/deploy-482c34d5`；五个运行二进制与 498 个 runtime payload
  均通过清单校验，manifest SHA-256 为
  `e09f7716a0a07f5c2e9fbe4195cbdc0de1474afb62a6da77d37e3b5aee91a518`，
  两个 runtime secret 均未嵌入包内。
- 生产在连续两次零活跃会话后受控切换到该精确包。启动日志确认
  `Cloudflare 邮箱验证码服务已装配`；PostgreSQL/R2 authoritative、零本地
  durable dependency、端口 `8077/8088`、单 Feishu 进程和公网路由均健康。
- 本地与公网签名探针均证明：使用当前 secret 的有效签名无副作用事件返回
  `200 ignored=true`，复用签名篡改正文或完全不带签名均返回 `401`；探针
  不写入会员状态。
- 代码交付边界仍为直接推送 `main`，不创建 release tag。

## Documentation Sync

- 更新 `docs/runbooks/whop-hone-activation.md` 的 Cloudflare 配置与验收步骤。
- 记录长期 provider 选择到 `docs/decisions.md`。
- 完成后新增 handoff、更新 `docs/archive/index.md`，并把本计划移入
  `docs/archive/plans/`。

## Risks / Open Questions

- Owner 已决定永久退出 Whop，因此真实非 owner buyer 的最终 Whop 验收被
  有意取消，不再是阻塞项。Whop 产品/计划已隐藏，HONE webhook 已删除。
- Cloudflare Email Sending 能力仍被 Stripe-only `/activate` 复用，并已在
  生产通过真实收件、同 challenge 验证与 Checkout 创建。其三个变量必须
  由生产 secret 管理注入 `/etc/hone/runtime.env`，不能依赖开发机忽略的
  `.env`；缺失时接口按设计 fail-closed `503`。

## Completion

本任务的邮件发送目标已经通过 Stripe-only 激活链路完成真实生产验收；
Whop 专属验证范围由 owner 决策终止。历史实现和证据保留，本计划随
Stripe-only 生产切换一起归档。
