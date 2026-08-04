# Discord 社区 Stripe-only 迁移

- title: Discord 社区 Stripe-only 迁移
- status: `done`
- created_at: `2026-08-04`
- updated_at: `2026-08-04`
- owner: `Codex + owner`
- related_files:
  - `config.yaml`（Git 忽略；只读取凭证，不记录值）
  - `docs/runbooks/discord-stripe-community.md`
  - `docs/runbooks/whop-discord-fulfillment.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/decisions.md#d-2026-08-04-01-make-stripe-the-only-external-billing-provider`
  - `docs/decisions.md#d-2026-08-04-04-make-discord-community-operations-stripe-only`
  - `docs/handoffs/2026-08-04-discord-stripe-community-migration.md`
  - `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`

## Goal

让 HONE 自有 Discord bot 能以最小必要管理权限维护
`1391380994182877205`（`巴芒投研美股社群`），并把当前社区配置中仍把
Whop 作为购买或会员授权入口的内容迁移到 Stripe-only。

## Scope

- 使用本地 `config.yaml` 中受保护的现有 HONE bot token；不输出、复制、
  提交或持久化 token。
- 邀请 bot 时先尝试完成本任务需要的服务器/频道/角色/消息管理与频道读写
  权限；只有在 API 证明频道级拒绝无法由托管 bot 角色自行修复、且 owner
  明确批准后，才允许记录清楚边界并升级为 Administrator。
- 盘点服务器描述、频道名称/主题、角色名称、置顶消息与近期运营消息中的
  `Whop`、旧 Whop URL、`Claim Access` 和旧购买/授权说明。
- 将当前购买入口统一为 `https://hone-claw.com/activate`，会员管理入口统一为
  `https://hone-claw.com/me`，并明确付费权益由 Stripe webhook 驱动。
- 不批量删除成员、角色、历史消息或用户内容；不能安全原位替换的历史证据
  保留并记录，不伪装成新状态。

## Validation

- Discord API 对 bot identity、目标 guild、member、roles、channels 均返回
  `200`，并证明实际权限与 owner 批准状态一致；若存在 Administrator，必须
  记录原因、额外风险、运行时禁用状态与可验证的降权前置条件。
- 修改前后保存不含消息正文或个人信息的结构化清单；复查服务器描述、频道
  主题、角色名称及置顶内容中当前态 Whop 引用归零或有明确保留理由。
- 浏览器打开目标服务器验证关键频道与展示效果；不发送测试噪声消息。
- 所有 Discord 请求均为目标 guild 内的受控读写；不打印 token，不读取或
  留存无关聊天正文。

## Documentation Sync

- 新增 `docs/runbooks/discord-stripe-community.md`，记录 bot 权限、Stripe-only
  社区边界、日常盘点与回滚方法。
- 更新 `docs/decisions.md` 和已退休的 Whop Discord runbook，说明新社区配置
  不再依赖 Whop。
- 完成后新增 handoff、更新 `docs/archive/index.md`，将本计划移入
  `docs/archive/plans/` 并从 `docs/current-plan.md` 移除。

## Risks / Open Questions

- Discord 管理权限可造成高影响外部变更；必须坚持最小权限并逐项验证。
- 旧 Whop app/bot、`VIP 付费用户` 角色和 `#whop` 日志频道可能包含历史证据；
  在识别当前真实用途前不删除。
- HONE 目前没有 Stripe webhook 到 Discord 角色的自动同步，因此本任务只
  迁移社区内容与管理能力，不把 Discord role 当作 HONE 付费权益真相源。

## Completion

- 本地 `config.yaml` 为 Git 忽略、mode `0600`，token 对应已验证 bot
  `Hone-TEST`。bot 已加入目标 guild，并使用昵称 `HONE 社区助手`。
- 最小服务器管理权限无法跨越 `💎｜会员权益` 的频道级发送 deny，且 Discord
  禁止托管 bot 角色修改自己的频道覆盖；owner 因此在 OAuth 页面明确批准
  `Administrator`。最终 API 证明 `administrator=true`，并能跨受限频道管理。
- 新的 Stripe-only 会员说明已发布并置顶为消息
  `1534163594952966174`；它只包含 HONE `/activate`、Stripe Checkout、
  `/me` 与手工 Discord VIP 核验边界。旧 Whop 置顶
  `1419304118509375624` 在新消息验证成功后删除，复查返回 `404`。
- `📋｜whop` 已改为 `📋｜历史支付日志`，主题明确旧日志只作归档、当前新会员
  统一走 HONE Stripe。历史日志正文保留在受限频道；未复制进文档或截图。
- Guild integrations 当前只有 `Hone`、`Hone-TEST` 与
  `ad-account-detector`，Discord webhook 列表为空，没有活动 Whop integration。
- 浏览器验收证明会员权益频道的 Stripe 文案与链接可见、Whop 可见文本为 0；
  去敏截图为
  `/Users/bytedance/.codex/visualizations/2026/08/04/discord-stripe-community/01-membership-stripe-only.jpg`。

本计划完成；后续从
`docs/handoffs/2026-08-04-discord-stripe-community-migration.md` 进入。
