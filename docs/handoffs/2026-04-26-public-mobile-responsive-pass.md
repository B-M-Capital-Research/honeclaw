# Public Website Mobile Responsive Pass Handoff

- title: Public Website Mobile Responsive Pass
- status: done
- created_at: 2026-04-26
- updated_at: 2026-04-26
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-site.css`
- related_docs:
  - `docs/archive/plans/public-mobile-responsive-pass.md`
  - `docs/archive/index.md`
- related_prs: N/A

## Summary

官网公开站完成一轮移动端适配收口。重点修复移动端 header 挤压、路线图页卡片和列表撑宽、首页视频/轮播区域小屏排版拥挤的问题。

## What Changed

- 在公开站共享 CSS 中为 public surface 增加 `box-sizing: border-box` 防溢出基线。
- 收紧 `.page-header`、`.header-logo`、`.header-actions`、语言切换和 CTA 按钮在 768px / 480px / 360px 下的尺寸与换行策略。
- 首页移动端约束 hero、按钮组、视频容器、轮播导航、轮播图文区域，避免内容撑出视口。
- 路线图移动端约束主内容、卡片、阶段列表、能力矩阵、代码块和底部 CTA；安装命令在手机上改为折行展示。

## Verification

- `bun run build:web:public` 通过。
- `bun run typecheck:web` 通过。
- Playwright 本地预览审计通过：`/`、`/chat`、`/roadmap`、`/me`、`/terms`、`/privacy` 在 360、390、430、768 宽度下没有横向页面溢出，header 宽度在视口内。
- `git diff --check` 通过。

## Risks / Follow-ups

- 发布到 Cloudflare Pages 后建议在真实手机或浏览器设备模拟器再抽查 `hone-claw.com` 的首页、`/chat` 和 `/roadmap`。
- 若线上样式没有立即变化，优先检查 Pages 最新部署状态与 Cloudflare 缓存。

## Next Entry Point

- 样式入口：`packages/app/src/pages/public-site.css`
- 构建命令：`bun run build:web:public`
