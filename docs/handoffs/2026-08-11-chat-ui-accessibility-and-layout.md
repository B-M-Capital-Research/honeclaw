# HONE 对话 UI 可读性与布局修复交接

日期：2026-08-11
状态：工具折叠方案已被 `2026-08-11-horizontal-tools-and-chat-hooks.md` 取代；字号与中文化部分继续有效

## 结果

- 对话页增加统一“每日投资工具”面板，空白会话展开，有历史或新增消息时自动收起，也可手动切换。
- 工具列表在自己的容器内滚动；对话和输入框各自占位，不再互相覆盖。
- 默认正文、用户提问、工具卡、侧栏和主要报告弹窗字号整体提高，并保留四档字号设置。
- 中文界面的 Agent、英文报告眉题、BJT 和英文证据口径已改为中文；品牌名、行业缩写与 ticker 保留。

## 主要文件

- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-chat-accessibility.css`
- `packages/app/src/pages/chat-accessibility-layout.test.ts`
- `packages/app/src/components/public-agent-workspace.tsx`
- 各每日仪表盘组件的中文显示文案

## 验证

- `bun test`：针对性 38/38 通过。
- `bun run --cwd packages/app typecheck`：通过。
- `bun run --cwd packages/app build`：通过。
- 本地认证浏览器 1280×720：工具收起与展开时，对话区和输入框重叠均为 0；展开工具列表不再越出父面板。

## 后续

- 本次未提交、未推送、未部署。
- 上线前建议补一次 390px iPhone 与真实老年用户的字号/对比度走查；如反馈仍偏小，优先把默认偏好从中号切到大号，而不是继续增加单页特例。
