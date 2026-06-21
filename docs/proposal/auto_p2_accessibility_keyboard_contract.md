# Proposal: Accessibility and Keyboard Contract for Trust-Critical Surfaces

status: proposed
priority: P2
created_at: 2026-06-22 03:02:49 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p2_surface-design-contract.md`
- `docs/proposal/auto_p1_workspace-command-palette.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/layout.tsx`
- `packages/app/src/components/public-nav.tsx`
- `packages/app/src/components/chat-view.tsx`
- `packages/app/src/components/chat-share-modal.tsx`
- `packages/app/src/components/public-checkbox.tsx`
- `packages/app/src/components/sidebar-nav.tsx`
- `packages/app/src/components/notification-preferences-card.tsx`
- `packages/ui/src/components/button.tsx`
- `packages/ui/src/components/input.tsx`
- `packages/ui/src/components/textarea.tsx`
- `packages/ui/src/styles/index.css`
- `packages/app/package.json`
- `packages/app/e2e/console.spec.ts`
- `packages/app/e2e/public-chat-upload.spec.ts`
- `packages/app/e2e/public-sms-login.spec.ts`

## 背景与现状

Honeclaw 的产品表面已经从单页聊天扩展成一个严肃投资工作台：

- Public surface 通过 `packages/app/src/app.tsx` 暴露 `/`、`/roadmap`、`/blog`、`/chat`、`/me`、`/portfolio`、`/terms`、`/privacy`，承担获客、登录、公共聊天、用户持仓和长期研究记忆入口。
- Admin surface 暴露 `/dashboard`、`/sessions`、`/skills`、`/tasks`、`/users`、`/research`、`/llm-audit`、`/logs`、`/task-health`、`/notifications`、`/schedule`、`/settings`，承担配置、排障、用户支持和运营。
- Desktop surface 复用 Web console，但运行在 Tauri 容器中，还承载 bundled/remote backend、sidecar 状态和本地渠道配置。
- 多渠道入口包括 Feishu、Telegram、Discord、iMessage 和 public Web push。很多问题最终仍需要用户或管理员回到 Web/desktop 完成登录、配置、授权、查看失败原因或继续会话。

当前前端已经有一些局部可访问性基础：

- `packages/app/src/components/sidebar-nav.tsx` 的 locale switch 使用 `role="group"`、`aria-label` 和 `aria-pressed`。
- `packages/app/src/components/public-checkbox.tsx` 自己实现了 `role="checkbox"`、`aria-checked`、`tabIndex` 和键盘切换。
- `packages/app/src/components/chat-share-modal.tsx` 使用 `role="dialog"`、`aria-modal`、Escape 关闭和 close aria label。
- `packages/app/src/pages/chat.tsx` 在偏好面板、账号面板、主动提示、滚动到底部、复制和分享按钮上已经有部分 `aria-label` / `aria-expanded` / `aria-live`。
- `packages/app/package.json` 已经有 `bun test`、Playwright `test:e2e` 和现有 public/admin e2e 框架，说明可以加入低成本的可访问性 smoke。

但这些能力是页面局部实现，不是产品级契约。基础 UI 组件如 `packages/ui/src/components/button.tsx`、`input.tsx`、`textarea.tsx` 只提供样式和基础属性透传，没有统一的 icon-only label 规则、busy/pressed/disclosure 状态、错误关联、说明文本关联或 reduced-motion 约束。Public nav、public chat、share modal、notification preferences、settings 和 admin lists 各自处理键盘与焦点，导致新功能很容易只通过鼠标路径验收。

这与 Hone 的产品定位有冲突：Hone 不是一次性营销站点，而是长时间陪伴用户做投资纪律、研究复盘和通知处理的工具。用户可能在移动端、桌面端、外接键盘、缩放显示、低视力场景或系统减少动态效果设置下使用它。管理员也需要高效键盘路径处理用户、任务、通知和配置。可访问性缺口会直接影响信任、留存、客服效率和未来商业化合规准备。

## 问题或机会

### 问题

1. **可访问性能力没有统一验收标准。**
   代码里有 `aria-*`、`role`、Escape 关闭、focus ring 等点状实现，但没有说明哪些页面必须 keyboard-only 可完成、哪些 modal 必须 focus trap、哪些状态必须 `aria-live`、哪些图标按钮必须可读屏。

2. **公共用户关键链路依赖复杂交互。**
   `/chat` 包含登录恢复、附件上传、composer、quota、复制、分享、偏好、账号面板、主动通知提示、滚动到底部等复杂交互；`/portfolio` 和 `/me` 承载用户资产与账号状态。如果这些链路在键盘、读屏或高缩放下不稳定，用户会把问题归因于产品不可靠。

3. **管理端/桌面端高频操作需要键盘效率。**
   Admin console 的 sidebar、session list、actor list、task list、settings form、notification preferences 和 logs/task-health 过滤都适合键盘操作。当前已有 `auto_p1_workspace-command-palette.md` 规划 Cmd/Ctrl+K，但如果基础焦点顺序、modal 返回焦点和 roving tabindex 没有契约，command palette 只会增加入口，不会保证可操作性。

4. **动效和虚拟列表缺少辅助技术边界。**
   `packages/app/src/components/chat-view.tsx` 使用 `virtua/solid` 虚拟列表和多种 `animate-pulse` / `animate-bounce` 状态；public nav 和 chat 页面有 transition、blur、菜单展开。它们对视觉用户友好，但需要 reduced-motion 降级、aria-live 节流和可恢复的滚动/焦点策略。

5. **测试覆盖偏向数据模型和 happy path e2e。**
   现有前端测试覆盖 public chat model、settings model、notification model、company profile transfer、public SMS login 等，但没有系统化断言：页面没有未命名按钮、dialog 有可访问名称、Tab 顺序可达关键动作、Escape 能关闭并归还焦点、提交失败会关联错误提示。

### 机会

新增 **Accessibility and Keyboard Contract** 可以让 Hone 的体验质量从“看起来正常”升级为“可被不同输入方式可靠操作”：

- Public trial：降低登录、聊天、上传、分享、持仓查看的流失。
- Admin support：让运营人员通过键盘更快处理用户、任务、通知和配置。
- Desktop workbench：更符合桌面应用预期，支持外接键盘、快捷键、窗口缩放和系统减少动态效果。
- 多渠道闭环：IM 中的深链最终落到 Web/desktop 时，页面能在窄屏和辅助技术下完成后续动作。
- 工程治理：用少量组件契约和 e2e smoke 防止新页面继续复制不可访问的按钮、modal、菜单和状态提示。

本提案标为 P2：它不直接改变 agent 推理、通知投递或数据安全，但对产品可信度、增长转化、客服效率和后续 UI 提案落地质量有明确收益。它可以增量落地，不需要大规模重写。

## 方案概述

建立一个跨 public/admin/desktop 的可访问性与键盘契约，第一版聚焦最关键的交互对象：

1. **Accessible primitives**
   在 `packages/ui` 中收敛按钮、输入、文本域、modal、disclosure、menu、status、toast、icon button 的语义和状态规则。

2. **Focus and keyboard policy**
   为 route shell、dialog、popover、side panel、virtual list、composer、settings form 建立焦点进入、Tab 顺序、Escape 关闭、返回焦点和快捷键冲突规则。

3. **Live region and async status policy**
   统一 chat streaming、uploading、quota exhausted、run pending、settings save、channel readiness、notification delivery 的 `aria-live` 和视觉状态表达。

4. **Reduced-motion and high-contrast baseline**
   对 pulse/bounce/blur/transition、scroll animation、loading indicator 和 skeleton 提供 `prefers-reduced-motion` 降级。

5. **Automated smoke gates**
   在现有 Bun + Playwright 基础上加入轻量检查：未命名可交互元素、dialog 可访问名称、Tab 可达关键动作、Escape 行为、错误提示关联和 reduced-motion 截图/DOM smoke。

## 用户体验变化

### 用户端

- Public `/chat` 可以只用键盘完成登录、输入、发送、上传附件、删除附件、复制回答、打开/关闭分享 modal、打开账号面板和返回 composer。
- 分享 modal、偏好 panel、账号 panel、主动提示 dialog 都有明确可访问名称，打开后焦点进入，关闭后焦点回到触发按钮。
- Chat streaming、上传、额度耗尽、恢复失败、发送失败通过稳定 live region 提示，不依赖用户盯住动画或颜色。
- `/me` 和 `/portfolio` 的关键卡片、按钮、company profile modal、导入导出或跳转动作在高缩放和读屏下仍能理解对象、状态和可执行动作。
- 系统开启减少动态效果时，loading dots、pulse、smooth scroll、blur/transition 不再制造不必要的运动负担。

### 管理端

- Sidebar、列表、过滤器、详情页和设置表单遵循一致 Tab 顺序；列表项能通过 Enter 打开，Escape 不会意外丢失未保存输入。
- Settings 中 provider/channel/runner 表单的错误、说明、保存状态、需要重启提示与字段通过 `aria-describedby` 关联。
- Notifications、Task Health、Logs、LLM Audit 等排障页的筛选和结果表能用键盘高效操作；失败状态不只靠颜色区分。
- 如果后续落地 `workspace-command-palette`，它可以复用同一套 dialog/menu/focus contract，而不是单独实现键盘语义。

### 桌面端

- Tauri bundled/remote 模式下，键盘焦点不会被 Web iframe-like shell、菜单、sidecar status panel 或 settings overlay 困住。
- Desktop 窄窗口下的 chat、settings、users、logs 页面保留清晰焦点顺序和可见焦点 ring。
- 桌面系统减少动态效果时，应用内转场、loading、消息滚动和通知提示同步降级。

### 多渠道

- Feishu/Telegram/Discord/iMessage 中的链接或提示如果把用户带回 Web/desktop，落地页能通过键盘/读屏完成目标动作。
- 渠道配置、channel target readiness、通知偏好和错误排查界面不只依赖颜色或 hover tooltip，便于用户按提示自助修复。
- 本提案不改变 IM 消息格式，但可以为未来“从 IM 打开 Web 操作”的深链目标定义可访问验收标准。

## 技术方案

### 1. 定义 `docs/frontend-accessibility-contract.md`

新增一份长期前端契约文档，记录最小规则：

- 所有 icon-only button 必须有 `aria-label` 或可见文本。
- Disclosure button 必须维护 `aria-expanded`，必要时关联 `aria-controls`。
- Dialog 必须有 `role="dialog"` 或原生 `<dialog>`，有可访问名称，打开后焦点进入，关闭后返回触发源。
- Popover/menu 必须支持 Escape 关闭；非 modal popover 不应吞掉页面 Tab。
- 表单错误必须与字段关联，不能只靠 toast。
- 异步状态必须区分 `polite` 与 `assertive` live region；streaming 内容不能每个 token 都打断读屏。
- loading/skeleton/animation 必须响应 `prefers-reduced-motion`。
- 色彩状态必须有文本、图标或语义补充，不只靠红/绿。

这份文档应和 `auto_p2_surface-design-contract.md` 衔接：Surface Design Contract 管视觉、滚动和组件边界；本契约管语义、输入方式和辅助技术。

### 2. 扩展 shared UI primitives

在 `packages/ui/src/components` 增加或升级：

- `IconButton`
  - `ariaLabel` 必填。
  - 支持 `pressed`、`expanded`、`controls`、`busy`。
  - 统一 focus-visible ring、disabled 和 loading。
- `Field`
  - 包装 label、description、error、required。
  - 自动生成 `id`、`aria-describedby`、`aria-invalid`。
- `DialogShell`
  - 管理初始焦点、Escape、背景点击策略、返回焦点和 heading id。
  - 第一版可不做复杂 focus trap，但必须保证 Tab 不落到不可见内容；若实现 trap，需测试嵌套弹层边界。
- `Disclosure`
  - 统一 account panel、preferences panel、mobile nav、notification preference sections。
- `LiveStatus`
  - 包装 `role="status"` / `aria-live` / 去抖文本，供 chat run state、uploading、settings save、quota 使用。
- `VisuallyHidden`
  - 替代散落的 `sr-only` 或自定义隐藏策略。

兼容策略：旧组件继续可用，新代码优先使用这些 primitives。高风险页面按阶段迁移，不做一次性全仓替换。

### 3. Public chat keyboard path

先把 `/chat` 作为第一条验收链路，因为它是 public conversion 和日常使用核心：

1. 登录态：
   - 手机号、验证码、同意条款、自定义 checkbox、提交按钮全键盘可达。
   - SMS/Captcha/登录错误关联到字段或表单 summary。
2. Composer：
   - 文本域、附件按钮、附件列表、删除附件、发送按钮、主动提示、滚动到底部按钮全部有可访问名称。
   - Enter/Shift+Enter 语义在可见说明或 aria description 中保持一致。
3. Messages：
   - 用户/助手/系统/计划任务消息有角色或可读 label。
   - 复制、分享、展开附件不依赖 hover 才能发现；移动端和键盘焦点下也可见。
4. Share modal：
   - 打开后 focus 到标题或主要按钮；Escape/close 后返回触发的消息按钮。
   - 选择消息、字体大小、分享/下载状态可读屏。
5. Async status：
   - run pending、streaming、uploading、restore retry、quota exhausted 使用 `LiveStatus`，避免 token 级读屏噪音。

### 4. Admin/desktop form and list path

第二阶段覆盖管理和桌面高频链路：

- `packages/app/src/pages/settings.tsx`
  - provider/channel/runner credential fields 使用 `Field`。
  - secret/credential 输入的 show/hide、probe、save、restart required 状态有明确 label。
- `packages/app/src/components/sidebar-nav.tsx` 和列表组件
  - 统一 list item keyboard activation。
  - 对当前选中项使用 `aria-current` 或等价语义。
- `packages/app/src/components/notification-preferences-card.tsx`
  - 自定义开关、分组、quiet hours、渠道选择都使用 shared disclosure/field/status。
- `packages/app/src/pages/logs.tsx`、`task-health.tsx`、`llm-audit.tsx`
  - 过滤器 label 和结果状态可读；空状态和错误状态不只靠颜色。

### 5. Reduced-motion and contrast baseline

在 `packages/ui/src/styles/index.css` 或 Surface Contract 后续拆出的 CSS 中加入：

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.001ms !important;
    animation-iteration-count: 1 !important;
    scroll-behavior: auto !important;
    transition-duration: 0.001ms !important;
  }
}
```

对 chat pending dots、public nav blur/transition、smooth scroll、share modal animation、skeleton shimmer 做专项检查。不要只用全局硬关：需要保留必要状态变化，但不能依赖运动表达状态。

### 6. Automated validation

不要求第一版引入大型可访问性平台，但建议增加以下轻量门槛：

- Bun DOM tests：
  - `IconButton` 没有 label 时抛错或测试失败。
  - `Field` 正确串联 label/description/error。
  - `LiveStatus` 对重复 streaming 文本做节流或只发布 phase 级状态。
- Playwright e2e smoke：
  - public chat login/composer/share modal 的 Tab 顺序和 Escape 行为。
  - admin settings 中一个 channel credential 表单的 label/error/save status。
  - desktop-like viewport 下 sidebar/list/detail 可键盘打开。
  - reduced-motion emulation 下页面不依赖 animated-only status。
- 可选依赖：
  - 后续可引入 `axe-core` 或 Playwright accessibility snapshot 做无名按钮、无名 dialog、表单 label 的自动扫描。
  - 第一阶段若不加新依赖，也可以用 `page.getByRole(...)` 作为可访问语义的实用测试。

## 实施步骤

### Phase 1: Contract and primitives

- 新增 `docs/frontend-accessibility-contract.md`。
- 在 `packages/ui` 新增 `IconButton`、`Field`、`DialogShell`、`Disclosure`、`LiveStatus`、`VisuallyHidden`。
- 为这些 primitives 加 Bun 单元测试。
- 增加 reduced-motion 基线 CSS。

### Phase 2: Public chat first path

- 迁移 `/chat` 中 composer、附件、复制/分享、偏好 panel、账号 panel、主动提示 dialog 到共享 primitives。
- 修复 share modal focus return、初始焦点和状态提示。
- 增加 Playwright keyboard smoke：登录表单、composer、share modal、Escape/Tab 行为。

### Phase 3: Public account and portfolio

- 迁移 `/me`、`/portfolio` 的关键按钮、modal、状态卡和错误提示。
- 确认 company profile detail、portfolio card、import/export 或跳转动作在键盘和读屏下可理解。
- 增加 mobile viewport + high zoom smoke。

### Phase 4: Admin and desktop workbench path

- 迁移 settings、sidebar/list、notification preferences 的高频自定义控件。
- 为 admin settings/channel credential 保存失败、需要重启、probe 结果建立可读状态。
- 在 desktop-like viewport 下验证 keyboard-only 操作路径。

### Phase 5: Regression guard

- 给新页面 checklist：新增 modal/menu/form/icon button 必须过 primitives 或显式说明例外。
- 在 `bun run test:web` 或 e2e 子集里保留轻量语义测试，不把昂贵视觉回归塞进默认门禁。
- 若引入 `axe-core`，先作为 targeted smoke，不要求一次清零全站历史问题。

## 验证方式

### 静态和单元验证

- `bun run test:web` 覆盖新增 primitives：
  - icon-only button 无 label 会失败。
  - `Field` 输出正确 `label for`、`aria-describedby`、`aria-invalid`。
  - `DialogShell` 有 accessible name，Escape 触发关闭回调。
  - `LiveStatus` 不对每个 streaming token 产生新的 assertive announcement。

### E2E 验证

- `bun --filter @hone-financial/app test:e2e` 增加 targeted specs：
  - Public chat：Tab 从 nav 到 composer，到附件/发送，到消息 action；Enter/Space 激活；Escape 关闭 panel/modal。
  - Share modal：打开后 focus 在 modal 内，关闭后返回原分享按钮。
  - Public login：手机号、验证码、条款 checkbox、提交、错误提示均可通过 role/label 查询。
  - Admin settings：至少一个 channel/provider 表单的 label、error、saving、restart-needed 可通过 role/status 查询。
  - Reduced motion：`page.emulateMedia({ reducedMotion: "reduce" })` 后关键状态仍可见且测试不依赖动画结束。

### 手工验收

- 键盘-only 走完 public login -> chat -> attach -> send -> copy/share -> close modal。
- macOS VoiceOver 抽样 `/chat`、`/portfolio`、admin settings，确认按钮名称、dialog 名称、状态提示可理解。
- Desktop 窄窗口下用 Tab/Shift+Tab 检查焦点不丢失、不进入不可见菜单、不被虚拟列表困住。

### 指标

- 新增页面/组件中无名 icon button 数量为 0。
- Public chat keyboard smoke 连续通过。
- 关键 dialog focus-return 失败数为 0。
- 可访问性相关用户反馈和客服截图定位成本下降。

## 风险与取舍

- **风险：范围容易膨胀成全站无障碍重写。**
  取舍：第一版只做 contract、primitives 和关键路径 smoke，不承诺 WCAG 全量认证，不一次性重写所有页面。

- **风险：focus trap 实现不当反而破坏复杂弹层。**
  取舍：先用 `DialogShell` 覆盖最常见 modal；对嵌套 popover、share canvas preview、mobile menu 保留明确测试，再逐步加强 trap。

- **风险：aria-live 过度播报干扰 chat streaming。**
  取舍：streaming 内容仍视觉更新，但读屏只播 phase/status summary，例如“回答生成中”“回答完成”“上传失败”，避免 token 级噪音。

- **风险：全局 reduced-motion CSS 影响已有交互感。**
  取舍：只在用户系统偏好为 reduce 时生效；保留必要状态变化，不用动画作为唯一状态载体。

- **风险：引入 axe-core 等依赖增加测试维护成本。**
  取舍：第一阶段可以只用 Playwright role 查询和自有 primitives 测试；axe-core 作为后续 targeted smoke，而非默认全站硬门禁。

- **不做的边界：**
  本提案不做完整 WCAG/ADA 法务认证，不重设计视觉系统，不替代 command palette，不改变 IM 消息格式，不要求所有历史页面一次性迁移。

## 与已有提案的差异

- 不重复 `auto_p2_surface-design-contract.md`：Surface Design Contract 关注视觉 token、surface shell、滚动模型、组件复用和截图/布局漂移；本提案关注语义、焦点、键盘、读屏、live region、reduced-motion 和可访问性测试门槛。
- 不重复 `auto_p1_workspace-command-palette.md`：Command Palette 提供跨资产搜索和命令入口；本提案确保 command palette 及其目标页面本身有可访问 dialog/menu/focus 语义。
- 不重复 `auto_p1_user-journey-replay-lab.md`：Journey Replay 关注发布信心和核心路径回放；本提案把“键盘-only、读屏语义、reduced-motion”定义成核心路径中的特殊输入/感知维度。
- 不重复 `auto_p1_invite_activation_funnel.md`：Invite Activation 关注新用户激活里程碑和转化；本提案关注这些里程碑能否通过不同输入方式可靠完成。
- 不重复 `auto_p1_public-pwa-notification-bridge.md`：PWA Notification Bridge 关注移动通知留存；本提案关注通知落地页、提示、设置和恢复动作的可访问操作性。
- 不重复 `auto_p2_locale-content-contract.md`：Locale Content Contract 关注多语言文案、错误码和内容组织；本提案关注文案如何被控件、状态和辅助技术正确关联。

查重结论：现有 proposal 已覆盖视觉系统、公共激活、命令入口、通知、旅程回放和内容本地化，但没有覆盖“跨 public/admin/desktop 的可访问性、键盘路径、焦点管理、live region、reduced-motion 与自动化语义 smoke”的产品/架构契约。因此本主题是新的、可执行的 P2 提案。

## 文档同步说明

本轮只新增 proposal，不开始执行改造，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/accessibility-keyboard-contract.md`，并在新增长期前端契约、shared UI primitives、e2e 门禁或页面交互规则后同步更新 `docs/repo-map.md`、`docs/invariants.md`，必要时补充 handoff/archive 索引。
