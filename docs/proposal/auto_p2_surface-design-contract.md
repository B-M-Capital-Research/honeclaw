# Proposal: Surface Design Contract for Public, Admin, and Desktop UX

status: proposed
priority: P2
created_at: 2026-05-15 14:04:30 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_zero-config-demo-workspace.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p2_locale-content-contract.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `packages/app/src/app.tsx`
- `packages/app/src/index.css`
- `packages/app/src/pages/public-site.css`
- `packages/app/src/pages/public-home.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/dashboard.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/components/public-nav.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/components/company-profile-detail.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `packages/ui/src/styles/index.css`
- `packages/ui/src/components/{button,input,textarea,badge,empty-state,skeleton,markdown}.tsx`
- `packages/ui/src/context/theme.tsx`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/lib/admin-content/*`

## 背景与现状

Honeclaw 当前已经不是单一聊天页面，而是由几个产品表面组合起来的投资助手：

- Public surface：`packages/app/src/app.tsx` 通过 `VITE_HONE_APP_SURFACE=public` 暴露 `/`、`/roadmap`、`/chat`、`/me`、`/portfolio`、`/terms`、`/privacy`，承担获客、登录、公共聊天、账号和用户投资上下文。
- Admin surface：同一个 SolidJS app 在 admin 模式下暴露 `/dashboard`、`/sessions`、`/skills`、`/tasks`、`/users`、`/research`、`/llm-audit`、`/logs`、`/task-health`、`/notifications`、`/schedule`、`/settings`，承担运维、客服、配置和长期资产管理。
- Desktop surface：Tauri host 主要承载同一套 Web console，同时有 bundled/remote backend、sidecar 管理、渠道状态和本地配置语义。
- Shared UI package：`packages/ui` 已有 `Button`、`Input`、`Textarea`、`Badge`、`EmptyState`、`Skeleton`、`Markdown`、`ThemeProvider` 等基础件，`packages/app/src/index.css` 也引入了 `@hone-financial/ui/styles`。

这些基础说明 Hone 已经具备抽象公共体验层的条件，但当前 UI 架构仍分裂明显：

- `packages/ui/src/styles/index.css` 定义了 `--bg`、`--panel`、`--surface`、`--border`、`--text-*`、`--accent`、`--success`、`--danger` 等 token，同时强制 `html, body, #root` 为 `height: 100%` 和 `overflow: hidden`，这是 admin app shell 的合理默认。
- `packages/app/src/pages/public-site.css` 为 public surface 反向覆盖 `html, body, #root`，使用 `!important` 把页面恢复成浏览器自然滚动，并维护另一组公共站点导航、按钮、背景和响应式样式。
- Public pages 大量使用 TSX inline style，例如 `/me`、`/portfolio`、`/chat` 中的卡片、按钮、modal、状态提示和布局，而 admin pages 更多使用 Tailwind class 与 `packages/ui` 组件。
- Public navigation、admin sidebar、dashboard cards、portfolio cards、company profile modal、chat composer、settings forms 各自定义尺寸、圆角、颜色、阴影、字体和滚动行为，用户从官网进入 chat、再进入 `/portfolio`，或管理员从 desktop shell 进入同一用户的 `/users/:actor/portfolio` 时，会感到是多个不同产品拼在一起。
- 当前已有 `auto_p2_locale-content-contract.md` 关注文案、语言和 API error code，但它不治理视觉 token、布局 primitives、滚动模型、组件边界和跨 surface 截图回归。

这不是单纯“重画 UI”。Hone 的产品形态正在从开源工具进入 public trial、桌面工作台、多渠道助手和可能的 Hone Cloud API。产品表面如果持续按页面局部内联样式扩张，会让后续所有提案落地时都付出更高成本：新功能很难一次性适配 public/admin/desktop，也很难用自动化验证它是否看起来像同一个产品。

## 问题或机会

### 问题

1. 跨 surface 的体验一致性不可控。
   Public `/portfolio` 和 admin `/users/:actor/mainline` 展示的是同类投资上下文，但前者主要靠 inline style 和 `public-site.css`，后者靠 Tailwind、admin content 和 UI package。相同产品对象没有共享视觉和交互契约。

2. 滚动模型是隐性全局冲突。
   Admin shell 需要锁住 `html/body/#root` 并让内部 pane 自己滚动；public website 需要页面级自然滚动。现在 public CSS 通过全局 `!important` 反向覆盖，后续新增 public-like page、embedded desktop shell 或 share preview 时容易出现双滚动、不可滚动或移动端溢出。

3. Shared UI package 没有成为真实的产品系统。
   `packages/ui` 已有基础组件，但 public pages 仍重复写按钮、卡片、信息行、modal、header action 等样式。这样会让 accessibility、focus、loading、disabled、density、mobile behavior 在每个页面重新实现。

4. 新功能难以形成稳定第一印象。
   Hone 的核心卖点是严肃投资纪律、长期研究记忆和跨渠道工作流。如果公开站点、聊天页、用户上下文页、管理控制台和桌面壳的视觉语言不统一，用户更容易把它理解成 demo 集合，而不是一个可靠的个人投研系统。

5. 自动化无法判断 UI 是否漂移。
   现在有内容结构测试、若干 model tests 和 chat tests，但没有“surface contract”层的检查：比如是否新增了 page-level global override、是否又写了一套 button/card/modal、是否在 mobile viewport 发生文本溢出、是否 public/admin 同类对象截图差异过大。

### 机会

建立 Surface Design Contract 可以带来几个直接收益：

- 产品留存：用户从 public chat 到投资上下文、再到桌面/管理端时，看到的是同一个可信工作台。
- 研发效率：新 proposal 落地时复用 page shell、cards、toolbars、modals、forms、status badges，不再每页重写。
- 运维质量：admin、desktop、public 的错误、空状态、能力缺口和配置状态用同一套 primitives 表达，客服截图更容易理解。
- 增长体验：官网、public trial 和 self-hosted console 的视觉一致性更强，开源用户更容易相信它能长期维护。
- 测试可落地：不用追求完整视觉回归，先锁住滚动模型、关键 viewport smoke、公共组件使用边界和高风险页面截图。

本提案标为 P2：它不会直接修复核心链路或安全问题，但会显著降低后续 public/admin/desktop 产品迭代的体验漂移和维护成本。

## 方案概述

新增一个 **Surface Design Contract**，把 Hone 的前端体验从“页面各自实现”收敛为“surface shell + design tokens + shared primitives + visual smoke”的产品架构。

核心原则：

1. 不做一次性大改版。
   第一阶段只建立契约和迁移最高复用 primitives，不重写所有页面。

2. 不把 public 和 admin 做成完全一样。
   Public 可以更具营销和引导感，admin/desktop 保持密集、工具化，但二者共享基础 token、动作组件、状态语义、modal/form/empty/error pattern。

3. 不用 CSS 全局覆盖解决 surface 差异。
   Admin locked-shell 和 public document-scroll 应成为显式 surface root class，而不是相互用 `!important` 抢 `html/body/#root`。

4. 不替代 locale content contract。
   文案仍归 `public-content.ts` / `admin-content/*`；本提案只治理视觉和交互架构。

5. 不强迫每个页面立即迁移到 UI package。
   新代码先强约束，旧代码按高价值路径逐步迁移：public nav、chat composer、portfolio/mainline cards、modals、settings forms。

建议新增的契约层：

- `SurfaceRoot`：根据 `public | admin | desktop-embedded | share-preview` 设置滚动模型、背景、字体、theme scope。
- `PageShell`：页面级布局 primitives，例如 `PublicPageShell`、`AdminPaneShell`、`DetailSplitShell`。
- `ProductCard` / `StatusPanel` / `ActionButton` / `Toolbar` / `InlineNotice` / `ConfirmModal`：跨 surface 复用的高频组件。
- `design-tokens.css`：从 `packages/ui/src/styles/index.css` 抽出 semantic tokens、density tokens、radius/spacing/shadow tokens，避免 public CSS 重建一套颜色和形状。
- `surface-smoke`：Playwright 或 Vitest + DOM rules 的最小视觉/布局验证，覆盖 desktop/mobile viewport。

## 用户体验变化

### 用户端

- Public `/chat`、`/me`、`/portfolio` 的 header、按钮、空状态、modal、登录状态和错误提示看起来属于同一套产品，不再像三个不同页面。
- `/portfolio` 的公司画像卡片、投资主线卡片和 profile modal 与 admin `/users/:actor/mainline` 的同类对象共享信息层级和状态表达。
- 移动端滚动和固定导航更稳定，减少因为 global override 或内联宽度导致的横向溢出。
- 用户进入 desktop/remote public 链接时，不会因为 surface 切换感到品牌和控件行为突然改变。

### 管理端

- Dashboard、Users、Settings、Notifications、Task Health 可以逐步统一 card、toolbar、status badge、empty state 和 confirm modal。
- 管理员看到的 actor、runner、channel、portfolio、profile、quota、notification 状态使用一致的 severity / capability pattern，更容易排障。
- 新增运营页面时只需要选用已有 shell 和 primitives，减少重复 Tailwind 拼装。

### 桌面端

- Desktop bundled/remote 模式继承 admin shell，但显式标记为 `desktop-embedded` surface，避免公共站点 CSS 或 share preview 影响 Tauri shell 滚动。
- 后续 desktop 独有设置、进程状态、channel cleanup、backend mode switch 可以复用相同 `StatusPanel` / `InlineNotice` / `ActionButton`，不再在 Tauri 命令页单独定义风格。
- 桌面窗口窄宽度下的 settings/chat/users 页面可以纳入同一套 viewport smoke。

### 多渠道

- 本提案不直接改 Feishu / Telegram / Discord 消息渲染。
- 间接受益是 Web 中用于解释渠道状态、channel scope、allowlist、delivery failure、notification prefs 的 UI 组件统一后，多渠道配置和排障更容易被普通用户理解。

## 技术方案

### 1. 明确 surface root 和滚动模型

在 `packages/app/src/app.tsx` 中让 public/admin 根节点带显式 scope：

```tsx
function PublicSurface() {
  return <SurfaceRoot surface="public">...</SurfaceRoot>
}

function AdminSurface() {
  return <SurfaceRoot surface="admin">...</SurfaceRoot>
}
```

CSS 目标：

- `html/body/#root` 只保留最小 reset，不承载具体 surface 滚动策略。
- `.hf-surface-admin` 管理 `height: 100dvh`、root overflow hidden、pane scroll。
- `.hf-surface-public` 管理 document scroll、public nav offset、内容区自然高度。
- `.hf-surface-desktop` 可在 Tauri 环境复用 admin shell，但保留未来桌面窗口安全区、titlebar、tray notice 扩展点。

迁移策略：

- 先新增 scope class 并保持现有表现。
- 再逐步移除 `public-site.css` 对 `html, body, #root` 的 `!important` 覆盖。
- 为 `/chat`、`/portfolio`、`/me`、admin `/dashboard`、`/users` 各保留一个滚动 smoke。

### 2. 抽出 semantic design tokens

将 `packages/ui/src/styles/index.css` 拆成更清晰的层：

- `tokens.css`
  - color: `--hf-color-bg`, `--hf-color-surface`, `--hf-color-border`, `--hf-color-text-*`, `--hf-color-accent`, `--hf-color-danger`。
  - radius: `--hf-radius-control`, `--hf-radius-card`, `--hf-radius-panel`。
  - spacing: `--hf-space-1` 到 `--hf-space-8`。
  - density: `--hf-control-h-sm/md/lg`、`--hf-toolbar-h`。
  - shadow/elevation: `--hf-shadow-card`, `--hf-shadow-popover`。
- `base.css`
  - font、markdown、scrollbar、focus ring。
- `surface.css`
  - admin/public/desktop root behavior。

兼容策略：

- 短期保留旧变量如 `--bg`、`--panel`、`--surface`、`--accent`，映射到新 `--hf-*`，避免一次性改全仓 class。
- 新组件只使用 `--hf-*`，旧页面逐步迁移。

### 3. 扩展 shared UI primitives

在 `packages/ui/src/components` 增加或完善：

- `action-button.tsx`
  - variants: `primary | secondary | ghost | danger | subtle`
  - sizes: `sm | md | icon`
  - stable disabled/loading/focus behavior。
- `product-card.tsx`
  - repeated item card、dashboard status card、public portfolio card 统一边框、padding、heading density。
- `inline-notice.tsx`
  - info/success/warning/error，支持 short summary + optional detail。
- `toolbar.tsx`
  - filter/search/action 区域的密度和 wrapping。
- `modal-shell.tsx`
  - profile modal、confirm dialog、artifact preview 共用尺寸、overlay、focus close。
- `surface-shell.tsx`
  - page/pane root，处理 title、subtitle、actions、scroll container。

迁移顺序：

1. Public nav action buttons 和 `/me` account action buttons。
2. Public `/portfolio` 的 mainline cards 和 profile modal。
3. Admin dashboard status cards 和 `/users` mainline profile modal。
4. Settings form controls 中最常见的 status/error/confirm patterns。

### 4. 建立 surface usage rules

在 `docs/repo-map.md` 或后续实现计划中补充前端约定，但本提案阶段不修改：

- 新 public/admin 页面必须选择一个 surface shell。
- 新用户可见 button/card/modal/form 优先用 `packages/ui`。
- 允许 page-local CSS 处理特定复杂布局，但不能覆盖 `html/body/#root` 或重定义全局 token。
- Inline style 只用于动态值、特殊 third-party embed 或临时计算，不用于常规 button/card/text/panel。
- Public marketing hero 可以保持更丰富视觉，但 account/chat/portfolio 这类产品面要使用 product primitives。

### 5. 增加最小视觉/布局验证

建议新增 `bun run test:surface` 或并入 `bun run test:web` 的一组轻量检查：

- DOM rule tests：
  - public/admin root 有正确 surface class。
  - public product pages 不新增 `html/body/#root` override。
  - 新增 TSX 中常规按钮/卡片不再出现大段重复 inline style。
- Playwright smoke：
  - `/chat`、`/me`、`/portfolio` public desktop/mobile。
  - `/dashboard`、`/users/:actor/mainline`、`/settings` admin desktop/narrow。
  - 检查无横向滚动、主要按钮可见、modal 不溢出、root scroll behavior 正常。
- Snapshot policy：
  - 不做像素级严格截图门禁。
  - 只保存关键 layout assertions 和 failure screenshot artifact，避免 UI 轻微调整导致频繁失败。

## 实施步骤

### Phase 1: 契约与基础设施

- 新增 `SurfaceRoot` / `surface.css`，让 public/admin root 显式拥有滚动模型。
- 抽出 `tokens.css`，让旧变量映射到新 `--hf-*`。
- 为 `packages/ui` 增加 `ActionButton`、`ProductCard`、`InlineNotice`、`ModalShell` 的最小版本。
- 添加 DOM rule tests，先只检查 root surface class 和禁止新增全局滚动 override。

### Phase 2: 迁移最高价值用户路径

- 迁移 `packages/app/src/components/public-nav.tsx` 的 CTA / link button 样式。
- 迁移 `packages/app/src/pages/public-me.tsx` 的 account card、info row、action buttons、membership placeholder。
- 迁移 `packages/app/src/pages/public-portfolio.tsx` 的 mainline cards、profile modal、notice/empty/error 状态。
- 迁移 admin `dashboard.tsx` status cards 与 `user-mainline-view.tsx` modal，使 public/admin 同类对象共享 primitives。

### Phase 3: Desktop 和运维面收敛

- 在 desktop backend context 中暴露 surface hint 或由前端按 Tauri 环境设置 `desktop-embedded` class。
- 迁移 `settings.tsx` 中 channel、runner、data-source 状态块到 `StatusPanel` / `InlineNotice`。
- 迁移 `notifications.tsx`、`task-health.tsx`、`logs.tsx` 的 toolbar 和 empty/error pattern。
- 补齐 desktop/narrow viewport smoke。

### Phase 4: 治理规则固化

- 更新 `docs/repo-map.md` 的 Web Console Structure 和 Common Coupled Changes，说明新页面要选 surface shell。
- 必要时补充 `docs/invariants.md`：不要用 page CSS 覆盖 root scroll，不要为常规控件重写内联样式。
- 将 `test:surface` 接入 CI-safe 前端验证，或在 `bun run test:web` 内运行。

## 验证方式

- 静态验证：
  - `bun run test:web` 覆盖新增 UI primitive model/DOM tests。
  - 新增 surface rule test 确认 public/admin root class 存在，且 public CSS 不再新增 `html/body/#root !important` override。
  - `rg` 检查新增/迁移页面中常规 button/card/modal 是否减少重复 inline style。
- 浏览器验证：
  - Playwright 打开 public `/chat`、`/me`、`/portfolio` 的 390px mobile 与 1440px desktop viewport，确认无横向滚动、固定导航不遮挡关键内容、profile modal 可滚动关闭。
  - Playwright 打开 admin `/dashboard`、`/users`、`/settings` 的 desktop/narrow viewport，确认 admin shell 仍保持内部 pane scroll。
  - Desktop shell smoke 确认 bundled/remote 模式下没有 public surface CSS 泄漏。
- 产品验收：
  - 同一 actor 的 public `/portfolio` 与 admin `/users/:actor/mainline` 展示相同对象时，卡片、modal、状态色、空状态和动作层级一致。
  - 新增页面实现时，开发者能从 `packages/ui` 选择现成 primitives，而不是复制已有页面样式。
- 回归边界：
  - 不要求全仓像素级截图一致。
  - 不要求 public marketing hero 立刻迁移到 admin density。
  - 不把视觉 smoke 放到需要外部账号或真实 channel 的测试里。

## 风险与取舍

- 风险：抽象过早会让 UI package 变复杂。
  取舍：只抽高频 primitives，不先做完整设计系统网站或大型组件库。

- 风险：public 和 admin 统一过度后失去各自场景特性。
  取舍：共享 tokens 与 product primitives，不共享全部布局密度；public marketing 仍可有独立表达。

- 风险：迁移期间存在两套变量和样式。
  取舍：旧变量映射到新变量，按页面渐进迁移；不要一次性大规模改所有 TSX。

- 风险：Playwright 视觉验证增加 CI 时间。
  取舍：只覆盖少数入口和 layout assertions；复杂截图保存为失败证据，不做严格像素比对。

- 风险：移除 `public-site.css` 的 root override 可能暴露隐藏滚动 bug。
  取舍：先引入 surface scope 并保留兼容，再逐步删除 `!important`，每步用 public/admin smoke 保护。

- 不做：不重写产品信息架构、不替换 Tailwind、不引入外部 design system、不把多渠道 IM 消息改造成 Web 组件、不处理双语文案治理。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 下所有 `auto_p*.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p2_locale-content-contract.md` 不重复：该提案治理文案、语言、API error code 和内容树；本提案治理 visual tokens、surface root、滚动模型、共享 UI primitives 和 layout smoke。
- 与 `auto_p1_invite_activation_funnel.md`、`auto_p1_zero-config-demo-workspace.md` 不重复：它们关注首次激活、试用和 demo workspace；本提案关注所有 public/admin/desktop surface 的长期 UI 架构和体验一致性。
- 与 `auto_p1_update-compatibility-center.md`、`desktop-bundled-runtime-startup-ux.md` 不重复：它们关注安装/更新/desktop runtime 启动和兼容状态；本提案只把这些状态未来应如何在前端一致表达纳入 scope，不改底层 runtime。
- 与 `auto_p1_run_trace_workbench.md`、`auto_p1_runtime_readiness_matrix.md` 不重复：它们关注 agent/run/capability 可观测性；本提案关注这些可观测对象的 presentation contract。
- 与 `auto_p1_linked-user-workspace.md`、`auto_p1_user-data-trust-center.md` 不重复：它们关注身份、数据归属、导出删除和隐私；本提案只要求这些对象在 UI 上使用一致的 shell、notice、card 和 modal。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该历史提案关注 skill runtime 与 multi-agent 执行对齐；本提案不改变 runner、skill、MCP 或 agent prompt 行为。
