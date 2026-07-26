# Public Workspace 视觉验收缺陷修复

- title: Public Workspace 视觉验收缺陷修复
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/components/public-workspace-shell.tsx`
  - `packages/app/src/components/public-agent-workspace.tsx`
  - `packages/app/src/components/public-prefs-button.tsx`
  - `packages/app/src/components/chat-share-modal.tsx`
  - `packages/app/src/components/chat-share-card.tsx`
  - `packages/app/src/pages/public-community.tsx`
  - `packages/app/src/lib/public-workspace-research.ts`
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-agent-workspace.css`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/public-community.css`
  - `packages/app/src/pages/public-workspace.css`
- related_docs:
  - `docs/archive/plans/public-workspace-visual-acceptance-fixes.md`
  - `docs/archive/index.md`
- related_prs: []

## Summary

完成已登录 Public Workspace 的严格视觉与交互缺陷修复。明暗主题、字号入口、辅助文字对比度和键盘焦点现已统一；Agent 之外的页面可读取研究记录并原地打开通知中心；新研究、搜索空结果、附件菜单、恢复期加载、PDF 预览、分享长图和社区空状态均有明确且可访问的行为。

## What Changed

- 新增跨页面复用的主题 / 字号控件，并为桌面和移动端补齐可访问名称、关闭行为与暗色样式。
- 从真实 bootstrap 消息建立稳定的研究记录列表；非 Agent 页面不再显示假空历史，研究项和“新研究”可深链回 Agent 并清理旧会话状态。
- 通知铃铛在当前页面原地打开推送中心和详情，不再只跳转到 `/chat`。
- 登录恢复期显示骨架和同步状态；无搜索结果与真实空历史使用不同文案。
- composer 使用居中定位容器，附件菜单与按钮稳定对齐；按钮具备 `aria-label`、`aria-haspopup` 和展开状态。
- 提升浅色辅助文字对比度，补齐中性暗色主题和 3px 全局焦点轮廓；导出分享卡片固定使用独立浅色 token，避免暗色主题下白底白字。
- PDF 使用已认证 blob URL，包含加载、慢载、成功和失败状态；宿主不绘制或请求失败时仍保留下载入口。
- 分享预览可键盘聚焦和滚动，长图有显式提示；模态框字号与 CTA 比例收敛。
- 社区空状态、文件预览和工作区表面在明暗主题下统一。
- 新增研究记录纯逻辑测试及视觉契约回归断言。

## Verification

- `bun run test:web`：285 pass，0 fail，863 assertions。
- `bun --filter @hone-financial/app typecheck`：通过。
- `bun --filter @hone-financial/app build`：通过，仅有既有的 chunk-size 警告。
- `git diff --check`：通过。
- 本地 Vite + Chromium 真实渲染验收：
  - 明暗主题桌面 Agent、洞察、通知中心、附件菜单和分享预览通过。
  - 研究搜索无结果显示准确文案；新研究清空历史查询、旧消息和草稿。
  - 附件菜单与 composer 左边缘对齐，暗色菜单正文与辅助文案可读。
  - PDF 正常响应、Chrome 阻止内嵌和 503 失败均显示可理解状态与下载回退。
  - 1.6 秒 bootstrap 延迟显示骨架，响应后恢复四条会话消息。
  - 390 × 844 深色移动端 `scrollWidth === innerWidth`，composer 与 56px 底部导航无重叠。
  - 键盘聚焦研究历史按钮时计算样式为 3px 高对比 outline、3px offset。

## Same-day Visual Follow-up

- 用户复查首轮移动端证据时发现深色消息区仍混用 `#212121` 页面、`#202421` 助手块、`#303030` 用户气泡以及三套不同文字白，造成助手消息整块底色与页面明显割裂。
- 移动端深色消息现统一到 Public Workspace token：页面 / shell 为 `#181b19`，助手消息背景透明，用户气泡为 `#2b302d`，消息文字统一为 `#f2f4f1`；桌面端保留同色相的层级卡片。
- 深色移动端契约测试新增对 token 使用和旧硬编码颜色消失的断言；前端 285 项测试、类型检查与构建再次通过。
- 修复前后及全部视觉验收截图保存在 `/Users/zhangxuanren/.codex/visualizations/2026/07/27/hone-visual-fixes/`；`16-all-fixes-contact-sheet.png` 是 16 张逐项证据的总览。

## Risks / Follow-ups

- `/Applications/HONE.app` 当前指向已部署的 `hone-claw.com/chat`，所以仍显示旧线上 bundle。用户随后明确要求提交并推送源码；此次仍未部署、打包、替换应用或执行正式 release。
- PDF 是否直接绘制仍由 WebView / 浏览器能力决定；下载回退是长期保留的可靠路径。
- 研究记录继续以当前单会话消息时间线为真相源。若未来需要真正的多会话历史，应单独设计 session API 与数据模型。
- 构建保留项目既有的大 chunk 警告，本任务未扩大到 bundle 拆分。

## Next Entry Point

从 `packages/app/src/pages/chat.tsx`、`packages/app/src/components/public-workspace-shell.tsx` 和 `packages/app/src/pages/public-community.tsx` 继续；视觉 token 入口位于 `packages/app/src/pages/public-foundation.css`。
