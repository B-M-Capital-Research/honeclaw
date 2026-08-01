# Public 未登录页主题、Whop 布局与滚动修复

- title: Public 未登录页主题、Whop 布局与滚动修复
- status: archived
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files:
  - `packages/app/src/lib/public-prefs.ts`
  - `packages/app/src/lib/public-prefs.test.ts`
  - `packages/app/src/app.tsx`
  - `packages/app/src/components/public-login-form.tsx`
  - `packages/app/src/components/public-checkbox.tsx`
  - `packages/app/src/pages/public-whop-activate.tsx`
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/public-login-theme-contract.test.ts`
  - `packages/app/src/pages/public-whop-activation-contract.test.ts`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-08-01-public-auth-theme-whop-layout.md`
- related_prs: []
- verification:
  - `bun run test:web`
  - `bun --filter @hone-financial/app typecheck`
  - `bun --filter @hone-financial/app build`
  - `git diff --check`
  - 本地浏览器桌面与移动端浅色 / 暗色登录页截图及计算样式检查
  - 本地浏览器桌面与移动端 Whop 开通页完整滚动范围与响应式布局检查
- risks:
  - 已显式保存 `auto` 或 `dark` 的用户偏好必须继续保留，默认浅色只作用于没有有效保存值的首次访问
  - 登录表单由 `/chat`、`/community`、`/me` 共用，样式修复必须避免破坏三个入口
  - Whop 页面涉及邮件验证码流程，视觉验收不得发送验证码或提交开通表单

## Goal

让 Public 登录页首次访问稳定采用浅色主题；用户显式选择暗色后，品牌、卡片、输入框、说明文字、复选框、按钮和链接仍具有清晰层级与可读对比度，并允许未登录用户直接切换主题。同时让 Whop 开通页在桌面和移动端使用合理尺寸的品牌视觉，正文完整进入可滚动页面且不被首屏裁切。

## Scope

- 主题偏好缺省值与 DOM 应用逻辑
- 共用 Public 登录表单的语义色与主题切换入口
- 登录卡、输入、复选框和状态信息的暗色样式
- Whop 开通页的品牌图尺寸、纵向节奏、响应式卡片和页面滚动所有权
- 默认主题与样式契约回归测试

## Tasks

- [x] 从线上 `/community` 登录页读取主题属性与关键计算样式，确认复现
- [x] 将无保存偏好的默认主题改为 `light`，保留显式 `auto` / `dark` 行为
- [x] 让登录页所有表面和文字使用成套语义 token，并补未登录主题入口
- [x] 从本地 `/activate/whop` 读取 DOM、计算样式与滚动范围，确认根因
- [x] 收敛 Whop 页面品牌图和内容节奏，恢复桌面 / 移动完整滚动
- [x] 增加默认主题、暗色品牌、表单表面与 Whop 滚动契约测试
- [x] 运行完整仓库 push 门禁与 diff 检查
- [x] 在本地真实浏览器验收两页桌面 / 移动的浅色、暗色与滚动状态
- [x] 完成 handoff，将本计划归档并更新 `docs/archive/index.md`

## Verification

见头部 `verification`；浏览器验收至少记录 `data-theme`、品牌文字、卡片、输入框与辅助文字的计算颜色，并确认页面无横向溢出。

## Risks

见头部 `risks`。本任务不部署生产、不修改认证流程、不发送短信或提交登录表单。

## Documentation Sync

任务已完成：新增 handoff，本计划移入 `docs/archive/plans/`，从活跃索引移除，并在 `docs/archive/index.md` 添加检索入口。无需更新 `docs/repo-map.md` 或 ADR，因为模块边界、数据流和长期架构决策未变化。

## Archive

已归档为 `docs/archive/plans/public-login-theme-contrast.md`。
