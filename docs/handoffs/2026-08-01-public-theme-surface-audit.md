# Public 全站主题表面审计与集中修复

- title: Public 全站主题表面审计与集中修复
- status: done
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-home.tsx`
  - `packages/app/src/pages/public-roadmap.tsx`
  - `packages/app/src/pages/public-plan.tsx`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/public-polish.css`
  - `packages/app/src/pages/public-community.css`
  - `packages/app/src/pages/public-plan-theme-contract.test.ts`
  - `packages/app/src/pages/public-theme-surface-contract.test.ts`
- related_docs:
  - `docs/archive/plans/public-theme-surface-audit.md`
  - `docs/archive/index.md`
- related_prs: []
- verification:
  - `bun test packages/app/src/pages/public-theme-surface-contract.test.ts packages/app/src/pages/public-plan-theme-contract.test.ts packages/app/src/pages/public-login-theme-contract.test.ts packages/app/src/pages/public-legal-theme-contract.test.ts packages/app/src/pages/public-whop-activation-contract.test.ts`：20 pass
  - `bun --filter @hone-financial/app typecheck`：通过
  - `bun run test:web`：334 pass，0 fail
  - `HONE_APP_SURFACE=public HONE_APP_OUT_DIR=dist-public-theme-audit bun --filter @hone-financial/app build`：通过；仅保留既有大 chunk 警告
  - `git diff --check`：通过
  - 本地 Public 真实浏览器覆盖浅色 / 深色的 `/`、`/roadmap`、`/plan`、`/blog`、唯一文章 slug、`/me`、`/activate/whop`、`/community`、`/terms`、`/privacy`、`/chat`、`/__share-preview`；全部横向溢出为 0，内容型页面可滚动，`/portfolio` 与 `/invest` 均跳转 `/me`
- risks:
  - 匿名浏览器不能进入认证后的工作台与社区时间线；这部分由已有完整 dark workspace contract 和新增社区主题契约覆盖，没有伪造会话
  - 本次只提交源码，不手工部署生产、不创建 release 或 tag

## Summary

Public 全站不再由用户逐页发现白底浅字。首次访问继续默认浅色，显式浅色、深色与 `auto` 偏好仍保留；首页、路线图、定价、Blog 列表与正文、认证后的社区内容、共享导航弹层、移动菜单和页脚统一使用成对的主题 surface / foreground token。登录、Whop、协议、聊天登录态、分享预览等此前已修复或本轮无异常的页面也重新通过真实浏览器检查。

## What Changed

- 将首页窗口、卖点、案例、Blog、Plan 预告和主次按钮从固定白底 / 白字迁到 `surface-raised`、`control-surface`、`action-fg`。
- 将路线图卡片、版本标签、嵌套内容卡、主按钮与代码区改成主题 surface，并新增稳定的 inverse token；生产状态徽标和小号代码标签达到 AA 对比度。
- 将定价页统计、社交入口、内容卡、会员卡、按钮、促销标签和弹层关闭按钮迁到成对 token。
- 将 Blog 页面背景、文章卡、语言切换卡、Markdown 变量、blockquote 和继续阅读按钮迁到主题 token，修复深色文章正文仍继承 UI 全局深色字的问题。
- 将登录后社区时间线的页面、卡片、文件、状态和操作按钮迁到主题 token；文档 iframe 和全屏媒体灯箱保持其固有媒体配色。
- 将共享导航“更多”面板、移动菜单 / tab、联系弹层与页脚的弱化文字改成可读配色；浅色小号珊瑚文字由 4.06:1 提升到 5.25:1。
- 新增全 Public 路由清单与首页、路线图、定价、Blog、社区、导航 / 页脚主题契约回归。

## Verification

真实浏览器计算样式显示：深色定价卡正文为 `#d8dcd7` / `#242825`（10.77:1），购买按钮为 `#17201f` / `#ffad9d`（9.28:1）；浅色小号珊瑚文字为 5.25:1，路线图生产徽标为 5.21:1，页脚弱化文字为 5.40:1。自动扫描把品牌图标容器、视频渐变遮罩和聊天渐变 CTA 报为低值时，又逐项读取实际子文本 / 渐变前景并确认是扫描器无法合成渐变或子元素覆盖造成的假阳性。

没有发送验证码、提交登录、触发购买、修改生产数据或伪造认证状态。构建产物已移出工作区到 `/tmp/codex-hone-public-theme-audit-20260801`，不会进入提交。

## Risks / Follow-ups

- 生产站需要由正常 Pages 流程消费本次 `main` 提交；若生产仍显示旧样式，先核对部署 commit 与缓存，不要继续在运行态写覆盖 CSS。
- 认证后页面的最终人工验收应使用正常用户会话，只做只读查看；当前自动化已经锁住社区与 workspace 的 dark surface 合同。
- 图片、海报、二维码、视频预览、文档 iframe 和全屏媒体灯箱的固定黑白色属于媒体内容，不应机械替换为页面主题 token。

## Next Entry Point

若再出现主题问题，先运行 `public-theme-surface-contract.test.ts` 与对应页面 contract，再在显式 `light` / `dark` 下读取目标元素和真实文本子节点的计算前景 / 背景；渐变和图片遮罩不能只看 `background-color`。
