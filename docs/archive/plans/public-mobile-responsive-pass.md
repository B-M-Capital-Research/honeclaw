# Public Website Mobile Responsive Pass

- title: Public Website Mobile Responsive Pass
- status: archived
- created_at: 2026-04-26
- updated_at: 2026-04-26
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-site.css`
- related_docs:
  - `docs/handoffs/2026-04-26-public-mobile-responsive-pass.md`
  - `docs/archive/index.md`

## Goal

修复官网移动端顶部菜单重叠、路线图页面横向撑宽、首页媒体与轮播区域小屏布局过挤等问题，让公开站核心页面在常见移动宽度下保持无横向滚动、按钮可点击、文本不溢出。

## Scope

- 共享公开站移动端 header 样式收紧：logo、语言切换、对话/路线图按钮在小屏下保留可用尺寸。
- 首页 hero、视频、轮播导航、轮播图文区域增加移动端宽度和换行约束。
- 路线图页卡片、阶段列表、能力矩阵、代码块、底部 CTA 增加单列和防溢出约束。
- 不改 API、路由入口、部署链路或内容源。

## Validation

- `bun run build:web:public`
- `bun run typecheck:web`
- Playwright 本地预览审计：`/`、`/chat`、`/roadmap`、`/me`、`/terms`、`/privacy` 在 360、390、430、768 宽度下 `body.scrollWidth === viewport width`，且 header 右边界未超出视口。
- `git diff --check`

## Documentation Sync

- 本任务已直接归档到 `docs/archive/plans/public-mobile-responsive-pass.md`。
- `docs/current-plan.md` 不新增活跃项：任务在当前回合内完成，没有跨会话阻塞。
- 补充 `docs/handoffs/2026-04-26-public-mobile-responsive-pass.md` 和 `docs/archive/index.md` 入口。
- 无需更新 `docs/repo-map.md`：页面入口、模块边界、主数据流未变化。

## Risks / Open Questions

- 当前移动端审计覆盖静态公开页面和 `/chat` 登录态入口；真实移动设备浏览器仍建议发布后抽查一次。
- Cloudflare Pages 上线后可能受缓存影响，若线上短时仍旧样式，应触发 Pages 重新部署或清理缓存。
