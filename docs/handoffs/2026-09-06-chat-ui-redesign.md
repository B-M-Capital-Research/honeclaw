# 对话页与工作台壳重构：文档式回答、工作轨迹、工具栏输入框

- title: /chat 手机端 + PC 端交互重构（参照 Codex / Claude Code 桌面端）
- status: done（分支 `feat/chat-ui-redesign`，待合入 main）
- created_at: 2026-09-06
- updated_at: 2026-09-06
- owner: Finn-Fengming
- related_files:
  - `packages/app/src/pages/public-chat.css`（重写：`--hc-*` 令牌 + `hc-` 组件层）
  - `packages/app/src/pages/public-chat-modals.css`（新增：持仓说明 / 财报工作流 / 财经日历弹窗）
  - `packages/app/src/pages/public-chat-accessibility.css`（重写：工具菜单 + 焦点）
  - `packages/app/src/pages/public-agent-workspace.css`（重写：侧栏 / 顶栏 / 手机头部 / 底栏 / 抽屉 / dock）
  - `packages/app/src/components/public-agent-workspace.tsx`（顶栏副标题、头像、删掉无引用的 Overview / RightRail）
  - `packages/app/src/components/public-prefs-button.css`
  - `packages/app/src/pages/chat.tsx`（展示层：回合、工作轨迹、输入框工具栏、demo 夹具）
  - `packages/app/src/lib/public-chat.ts`（`finishedAt`、`carryOverLocalRunTrail`）
  - `packages/app/src/lib/public-chat-demo.ts`（新增：`/chat?demo=1` 夹具，仅 DEV）
  - `packages/app/src/lib/public-content.ts`（`chat_page.trail.*`、`composer.stop_aria/quota_remaining`、`workspace.restore_notice/reconnect/community_action`、`attachments.remove_aria`、`misc.disclaimer`）
  - `packages/app/src/pages/public-chat-style-contract.test.ts`、`chat-accessibility-layout.test.ts`、`components/public-language-contract.test.ts`、`pages/chat.test.ts`
- related_docs: 设计稿 https://claude.ai/code/artifact/f60bde1c-87fe-4697-8f33-11b8aea7d5d4；`docs/handoffs/2026-09-02-research-depth-default-and-chat-scroll-pin.md`（本分支第一个提交把它未提交的前端改动带了进来）

## 用户要求

> 目前大家反馈交互优化很丑，手机端的交互还是很丑，全部重构优化一把，包括 PC 端一起，要看起来是专业的投研助手，
> 交互参考 Codex、Claude Code 桌面端，先设计再重构；已有的各种样式库也都重构了，防止把全局拉垮。

## 一、诊断

对照反馈截图，问题是四个系统性决定叠在一起，而不是某个颜色：

1. 正文 18px / 1.75 行高、数字与标题 800 字重——一屏只装六七行。
2. 手机端从上到下叠着 58px 头部、工具 chip 行、输入框、免责声明、5 格底栏，回答区不到六成。
3. 等待期只有"HONE 执行中"一行加三个点；每一问其实是多步研究，但用户看不到步骤。
4. `public-chat.css` 2 812 行、五段 768px 媒体块互相用 `!important` 覆盖，桌面 769px 与手机壳 820px 两套断点；
   `public-agent-workspace.css` 又叠一层 `--agent-*` 硬编码色。

## 二、设计（先出稿再动手）

设计稿见上面的 artifact：诊断、原则、令牌与字号表、手机 / 桌面示意、组件解剖、状态矩阵、实施范围。三条原则：

- **文档而非气泡**：助手回答通栏无底色，只有一行 12px 署名（珊瑚点 · `HONE` 等宽 · 时间或状态）；用户提问是唯一的填充块。
- **看得见的工作轨迹**：`run_progress / tool_call / reasoning_delta` 排成步骤列表（✓ / 旋转 / 空心圆 + 思考摘要），正文开始输出后折叠成"已完成 N 步 · 用时 Ns"。
- **克制的 chrome**：1px 细线代替阴影与卡片，全局只有珊瑚一个强调色；侧栏 / 顶栏 / 底栏统一 13–14px、500/600 字重。

字号：回答桌面 15px、手机 16px，行高 1.7，强调 600；字号偏好 s/l/xl 只缩放正文、提问与输入框（14/17/19）。

## 三、实现

- **令牌与层次**：对话页 `--hc-*`、壳层 `--agent-*` 全部从 `--hone-*` 派生，暗色主题就是令牌翻转；一个断点 820px；
  五个样式文件零 `!important`（契约测试直接断言）。弹窗样式独立到 `public-chat-modals.css`，日历 1080×1350 画板几何原样保留。
- **回合**：`hc-turn--assistant`（署名头 / 工作轨迹 / 正文 / 复制分享）、`hc-turn--user`。轨迹在正文出现后折叠为
  `<details class="hc-trail hc-trail--summary">`；流式光标改为 CSS `::after` 画在最后一个块末尾。
  `finishedAt` 在终态帧写入；`carryOverLocalRunTrail` 在回答后的 restore 中保留本地轨迹（服务端历史不带 steps）。
- **输入框**：一个 14px 圆角盒：附件预览 → 文本域 → 底部工具栏（+ 附件、工具菜单、持仓分析、财经日历、社区，
  管理员多两项财报工作流；桌面显示图标+文字，手机只留图标可横滚）；右侧"今日剩余 N 次"+ 圆形发送键，
  本页拥有可中断的流时变成停止键。
- **壳**：桌面侧栏 264px（品牌 → 新对话 → 五项导航 → 对话记录按日分组 → 用户）、顶栏 48px（标题 + tagline）；
  手机头部 52px，底栏五格不变；手机 dock 仍固定在底栏上方，但列表底部留白改由 ResizeObserver 写入的
  `--hc-dock-height` 决定，多行草稿或附件不再盖住最新回合。
- **未登录**：手机头部改为通栏细线（原来是浮动药丸，且品牌字与"HONE 投资助手"重复）。
- **评审夹具**：`vite dev` 下 `/chat?demo=1` 灌入完成 / 流式 / 执行中 / 出错 / 推送卡五种回合；生产构建不可达。

## 第二轮：上线后的四条反馈（同日）

> PC 端下面的点了也没用，看起来功能缺失；手机端底部还是太高；一堆图标看不懂；很多推荐的问题用户看不到。

根因与改法：

1. **弹窗点了没反应**：集成到大V速报分支时，拼接脚本丢了 `import "./public-chat-modals.css"`。
   财报前瞻 / 财报分析 / 财经日历其实都弹开了，但没有定位样式，被渲染到视口下方（y = 视口高度，全宽），
   按钮停在 `aria-expanded="true"` 的填充态——就是截图里两个财报按钮"按下去"的样子。
   补回 import，并加契约测试逐个断言四个样式文件都被引入。
2. **工具栏溢出**：七个 chip + 配额 + 发送在 768px 列里差 17px，"查看社区动态"被裁。
   管理员的两个财报入口收进「工具」菜单的「财报工作流」组；工具栏改 `flex-wrap`，永不裁切。
3. **手机端底部太高 / 图标看不懂**：手机端输入框改成一行「+ · 输入 · 发送」，底部叠层从约 180px
   降到 108px（dock 52 + 底栏 56）。所有入口收进「+」的底部面板，四组：附件、快捷入口（3D 数据中心 /
   财经日历 / 社区，管理员多两个财报工作流）、每日研究、推荐提问，每项都有图标 + 名称 + 一句说明。
   免责声明在手机端只在空会话显示（`.is-empty`）。
4. **推荐提问看不见**：起步卡只在空会话出现，有历史的用户永远看不到。现在会话非空时输入框上方
   常驻一条可横滚的推荐条（`hc-suggest`，× 收起当天），手机面板里也有「推荐提问」一节。
   进行中的回合、草稿非空时自动隐藏。

实现上把财报 / 日历对话框改成无触发器的 headless 组件（`EarningsResearchDialog` /
`FinanceCalendarDialog`，用 `openRequest` 计数打开），输入框维护一份 `ComposerAction[]`：
桌面渲染成 chip，手机渲染进面板，管理员项进工具菜单——一份列表三处复用。

## 第三轮：分享（同日）

> 分享这个也要跟上，把分享的样式和交互全部重构。

原来是两步向导（勾消息 → 生成 PNG → 预览图上再选动作），400 行样式内联在 TSX 里，卡片是深色居中气泡。改成：

- **一屏直出**：桌面左栏「包含的消息」（最近 4 条，勾选行带角色标签与去掉 Markdown 标记的摘要）+ 字号四档
  （小 / 标准 / 大 / 特大，标准 = 对话页 15px）+ 一列动作（保存图片为主，复制图片 / 仅复制文字 / 分享到其他应用）；
  右栏实时渲染**真实卡片**（`ScaledCard` 按容器宽度整体缩放，预览即成图），不再等 PNG。手机端是底部 sheet：
  预览框固定 46dvh 内滚，勾选与字号在下方，动作吸底。
- **卡片**（`chat-share-card.tsx` + `chat-share.css`）：420px 暖纸底，头部 logo + HONE + 日期，提问是浅灰左对齐块，
  回答是文档排版（署名点 + HONE、650 标题、hairline 表格、等宽数字、带边框代码块），页脚品牌 + 扫码说明 +
  一行免责 + 二维码。卡片令牌钉在根元素上、颜色全是字面值——html2canvas 不认 `color-mix()`。
- PNG 仍只在动作时栅格化，并在选择变化后 300ms 预热，保住 iOS 必须在手势内 `navigator.share` 的约束。
- 纯文本复制改成「我：… / HONE：…」+ 一行来源署名（`shareTextForClipboard`）。

验证：`/__share-preview` 路由跑 html2canvas 实际导出，列表符号、代码块、表格、二维码与网页预览一致；
390 / 1280 两档弹窗截图；单测新增摘要去标记、文本复制、日期戳三组。

## 第四轮：输入框的「财经日历」换成「大V观点速报」（同日）

> 把财经日历这里改成 大V观点速报。

- `Composer` 的快捷入口列表里，财经日历那一格换成 **大V观点速报**（`id: "influencer-digest"`，喇叭图标），
  点开直接走 `onOpenPanel("influencer-digest")`——和「工具」菜单打开研究面板一样盖在对话上，不跳研究台、
  不丢当前会话；手机端 `+` 面板同一条目带一句说明（"Serenity 等作者的最新推文与观点，每 15 分钟同步"）。
- 财经日历没有删：`ComposerAction` 新增 `secondary` 标记，桌面端从 chip 行退到「工具」菜单的「工作流」组
  （原「财报工作流」改名「工作流」，非管理员也能看到这组），手机端仍在 `+` 面板的「快捷入口」里排最后。
- 「工具」菜单「每日研究」组里原来的「大V速报」一行删掉（它已经是 chip，避免桌面与手机面板各出现两次），
  对应的 `tools_influencer_*` 文案键随之移除；研究台自身的「大V速报」标题未动。
- 契约测试钉住：chip 过滤 `!adminOnly && !secondary`、digest 条目在社区之前并走 `onOpenPanel`、
  日历 `secondary: true`、菜单里不再重复 digest。Playwright 点过桌面 chip / 工具菜单 / 手机面板，
  三处弹层都在视口内。

## verification

- `bun run typecheck` 通过；`bun test --preload ./happydom.ts ./src ./public` 541 pass / 0 fail
  （重写了两份视觉契约测试，新增 `carryOverLocalRunTrail` 两组用例）。
- Playwright headless（`node_modules/.cache/shot.mjs`，dev-login 后）截图：390×844 与 1280×800、浅色与深色、
  `/chat?demo=1` 全状态、真实会话、空会话、抽屉、工具菜单、`/research`、`/me`、未登录页。
  手机端所有回合宽度 = 列宽（358px），无横向溢出。
- **未做**：真实流式回答（本机 LLM key 为空）；iOS Safari 真机的键盘 / 安全区回归；
  `e2e/public-chat-upload.spec.ts`（testid 未动，但该用例本就未跟上工作台，见记忆）。

## risks

- 其它工作台页面（/research、/pushes、/me、/community）共用壳组件，会自动换成新侧栏 / 顶栏 / 底栏；
  页面内容仍是各自的 CSS（public-workspace.css），本轮没动，风格略旧但不冲突。
- 主 checkout 里 09-02 那轮未提交的 `chat.tsx / public-chat.ts / chat.test.ts` 改动已包含在本分支首个提交；
  合并前先 `git checkout --` 这三个文件（或 stash），否则会冲突。
- `.hc-turn__body` 里把 `--text-primary/--text-secondary/--accent/--border` 覆盖成 `--hone-*`，
  让 `packages/ui` 的 `hf-markdown` 基础样式在公开端解析到正确颜色；若 ui 包改变量名要同步。
