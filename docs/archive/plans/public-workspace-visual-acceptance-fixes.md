# Public Workspace 视觉验收缺陷修复

- title: Public Workspace 视觉验收缺陷修复
- status: archived
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/components/chat-share-card.tsx`
  - `packages/app/src/components/public-workspace-shell.tsx`
  - `packages/app/src/components/public-agent-workspace.tsx`
  - `packages/app/src/components/chat-share-modal.tsx`
  - `packages/app/src/components/public-prefs-button.tsx`
  - `packages/app/src/lib/public-workspace-research.ts`
  - `packages/app/src/pages/public-community.tsx`
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-chat.css`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/public-community.css`
  - `packages/app/src/pages/public-workspace.css`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/2026-07-27-public-workspace-visual-acceptance-fixes.md`
  - `docs/archive/index.md`

## Goal

修复已登录 Public Workspace 的严格视觉与交互验收缺陷：统一明暗主题、颜色对比度与焦点态，补齐跨页研究记录和通知中心行为，并消除 PDF 空白预览、假空状态、附件菜单错位、无障碍命名、恢复期白屏和分享长图预览可用性问题。

## Scope

- 让已登录工作区在 Agent、投资、洞察和账户页面都能访问主题与字号设置。
- 跨页复用研究历史与推送未读状态，并让铃铛直接打开通知中心。
- 让“新研究”稳定进入干净的研究总览，搜索空结果使用准确文案。
- 为会话恢复提供明确骨架或状态，不再出现长时间空白工作区。
- 修复附件菜单定位、附件按钮可访问名称、统一可见焦点样式。
- 提升小字号正文与辅助文本的 WCAG 对比度，并补齐暗色主题。
- 为 PDF 资源提供可见加载 / 失败回退，改善分享长图和模态框可读性。
- 补充样式与纯逻辑契约测试，避免上述问题回归。

## Validation

- `bun run test:web`：285 pass，0 fail。
- `bun --filter @hone-financial/app typecheck`：通过。
- `bun --filter @hone-financial/app build`：通过；仅保留现有 chunk-size 警告。
- 使用本地 Vite Public Surface、真实 Chromium 渲染和受控 API fixture 复验 Agent、洞察、通知中心、主题、字号、附件、PDF 与分享预览。
- 390 × 844 深色移动端复验：页面无横向溢出，composer 与底部导航不重叠，主题和通知入口可见。
- PDF 成功路径显示明确宿主回退提示；503 路径显示失败说明和下载入口。
- 1.6 秒 bootstrap 延迟期间显示骨架和同步状态，完成后恢复会话。
- 键盘焦点使用 3px 高对比轮廓；附件按钮具备名称、展开状态并与 composer 左边缘对齐。
- `git diff --check`：通过。

## Documentation Sync

- 开始时更新 `docs/current-plan.md` 并登记本计划。
- 完成后新增 `docs/handoffs/2026-07-27-public-workspace-visual-acceptance-fixes.md`。
- 完成后将本计划移入 `docs/archive/plans/`，从 `docs/current-plan.md` 移除，并更新 `docs/archive/index.md`。
- 本次不改模块边界、长期架构决策或运维流程，因此无需更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md` 或 runbook。

## Outcome / Risks

- Public Workspace 历史仍来自单条会话消息而非独立 session；界面已准确呈现为“研究记录”，本次没有引入新的后端数据模型。
- PDF 内嵌仍取决于宿主 WebView；现在无论空白、阻止或请求失败，都有可见状态和下载回退。
- `/Applications/HONE.app` 当前加载已部署的 `hone-claw.com` 旧前端；本次只修复并验证仓库本地源，没有擅自发布、替换应用或重启生产。
- 事件引擎四个既有未提交文件未被本任务修改。
