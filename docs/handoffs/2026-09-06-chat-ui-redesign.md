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
