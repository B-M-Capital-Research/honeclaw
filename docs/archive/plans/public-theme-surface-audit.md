# Public 全站主题表面审计与集中修复

- title: Public 全站主题表面审计与集中修复
- status: archived
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files:
  - `packages/app/src/app.tsx`
  - `packages/app/src/pages/public-*.tsx`
  - `packages/app/src/pages/public-*.css`
  - `packages/app/src/components/public-*.tsx`
  - `packages/app/src/lib/public-prefs.ts`
- related_docs:
  - `docs/handoffs/2026-08-01-public-theme-surface-audit.md`
  - `docs/archive/index.md`
- related_prs: []
- verification:
  - 20 个聚焦主题契约通过
  - Web typecheck 通过，334 个 Web 测试通过，Public production build 通过
  - 浅色 / 深色真实浏览器覆盖全部 Public UI 路由和两个重定向别名，横向溢出为 0
  - `git diff --check` 通过
- risks:
  - 认证后状态由静态 contract 补足，未伪造登录或发送验证码
  - 生产部署不在本任务范围

## Goal

一次性审计全部 Public 路由和共享组件，找出浅色 / 深色主题下写死白底、黑底、白字或黑字造成的不可读组合，集中修复并建立全站回归。

## Scope

- UI 路由：`/`、`/roadmap`、`/plan`、`/blog`、`/blog/:slug`、`/me`、`/activate/whop`、`/community`、`/terms`、`/privacy`、`/chat`、`/__share-preview`
- 别名：`/portfolio`、`/invest`，均验证跳转 `/me`
- 共享表面：导航、页脚、登录表单、卡片、按钮、弹层、提示、媒体容器和移动端吸附操作栏
- 主题：首次默认浅色、显式浅色、显式深色和保留的 `auto`

## Tasks

- [x] 从 Public 路由表建立审计清单
- [x] 静态扫描页面与共享组件里的硬编码表面 / 前景色并分类
- [x] 用真实浏览器覆盖可匿名访问页面的浅色 / 深色关键状态
- [x] 集中修复定价页和其余确认问题
- [x] 增加全站主题契约回归
- [x] 运行 Web 测试、类型检查、Public 构建与 diff 检查
- [x] 更新 handoff / archive，归档本计划并移出活跃索引
- [x] 精确提交并推送 `main`

## Validation

完整证据见 `docs/handoffs/2026-08-01-public-theme-surface-audit.md`。浏览器扫描的渐变 / 子节点假阳性均以实际文本节点计算样式和截图复核，没有将媒体本身的固定配色误改成页面主题。

## Documentation Sync

已新增 handoff、归档本计划、从 `docs/current-plan.md` 移除并更新 `docs/archive/index.md`。本次没有改变模块边界、数据流、运维方式或架构决策，因此不更新 `docs/repo-map.md`、`docs/invariants.md`、runbook 或 ADR。

## Risks / Open Questions

没有待处理的代码阻塞。生产站仍需正常部署消费本次提交；认证后真实数据状态未做写入式验收。
