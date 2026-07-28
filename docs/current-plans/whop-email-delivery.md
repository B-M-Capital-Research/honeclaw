# Whop 购买邮箱真实投递

- title: Whop 购买邮箱真实投递
- status: in_progress
- created_at: 2026-07-28
- updated_at: 2026-07-28
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/email_verification.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `docs/runbooks/whop-hone-activation.md`
- related_docs:
  - `docs/handoffs/whop-email-delivery.md`

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
- 浏览器从 `/activate/whop` 请求验证码，真实邮箱收到并完成 `/me` 登录。

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
  `/activate/whop` → `/me?checkout=success`，会员状态、周期、邮箱掩码和
  Whop 管理入口均正确。
- Gmail connector 未连接，Chrome 登录的是另一个邮箱；因此没有读取用户
  指定收件箱。浏览器登录阶段使用隔离数据库的等价已知 challenge，真实
  provider 投递与登录表单/后端校验分别完成验收。
- 用户在隔离验收结束后回传了实际收到的验证码，确认真实收件箱已收到
  HONE 邮件；验证码本身未写入代码、文档或日志。由于隔离运行时已清理，
  该验证码未被回放到已经销毁的 challenge。
- 代码交付边界为直接推送 `main`，不创建 release tag；生产部署、密钥注入
  和后端重启由外部部署方执行，不在本机临时 shell 中操作。

## Documentation Sync

- 更新 `docs/runbooks/whop-hone-activation.md` 的 Cloudflare 配置与验收步骤。
- 记录长期 provider 选择到 `docs/decisions.md`。
- 完成后新增 handoff、更新 `docs/archive/index.md`，并把本计划移入
  `docs/archive/plans/`。

## Risks / Open Questions

- 生产启用仍需外部部署方在 secret manager 或 supervisor 中注入
  `HONE_CLOUDFLARE_ACCOUNT_ID`、`HONE_CLOUDFLARE_EMAIL_API_TOKEN` 和
  `HONE_EMAIL_FROM`，再按 `docs/runbooks/backend-deployment.md` 受控
  重启；不得从临时 shell 直接替换生产进程。
- 生产最终验收仍应使用真实 Whop 非 owner buyer，并从实际收件箱输入同一
  封邮件中的验证码；本地真实收件已由用户回传验证码确认。
- 本机忽略的 `.env` 不会随 Git 推送；外部部署方必须通过安全渠道取得
  token，或创建只含 `Email Sending: Edit` 的独立生产 token。更换
  supervisor 工作目录或迁移主机时必须通过 secret 管理重新注入三个变量。
