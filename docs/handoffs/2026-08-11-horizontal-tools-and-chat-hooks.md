# HONE 横向工具栏与新对话问题钩子交接

日期：2026-08-11
状态：本地完成，未部署

## 本次变化

- 删除“展开/收起 9 项工具”的交互。
- 10 个现有功能作为独立 Button 固定在输入框上方，并在一行内左右滑动。
- 仪表盘 Button 继续打开可关闭弹窗；估值实验室和研究资料库继续进入完整页面。
- “新对话”显示空白研究区和 5 个可直接发起提问的钩子。
- 问题钩子覆盖宏观、持仓、最近财经事件、关键事件链和估值；有持仓与日历数据时自动个性化。

## 关键文件

- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-chat-accessibility.css`
- `packages/app/src/lib/chat-empty-prompts.ts`
- `packages/app/src/lib/chat-empty-prompts.test.ts`
- `packages/app/src/pages/chat-accessibility-layout.test.ts`

## 验收数据

- 工具 Button：10 个，同一行。
- 横向工具区：可视宽 948px，内容宽 1527px；桌面端完整显示 6 个，并露出第 7 个提示可滑动。
- 工具区高度：66px；Button 尺寸：144×52px；工具栏内隐藏重复的状态副文案、数字角标和箭头，详情仍保留在弹窗中。
- 新对话问题：5 个。
- 页面横向溢出：0；对话与输入框重叠：0。
- 针对性回归测试 26/26、TypeScript、生产构建、diff check 全部通过。

## 边界

- 当前后端仍是一个服务端会话；“新对话”是前端可见分段，不会删除历史。
- 本次没有提交、推送或部署。
