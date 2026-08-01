# Public 未登录页主题与 Whop 布局滚动修复

- title: Public 未登录页主题与 Whop 布局滚动修复
- status: done
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files:
  - `packages/app/src/app.tsx`
  - `packages/app/src/lib/public-prefs.ts`
  - `packages/app/src/lib/public-prefs.test.ts`
  - `packages/app/src/components/public-login-form.tsx`
  - `packages/app/src/components/public-checkbox.tsx`
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/public-whop-activate.tsx`
  - `packages/app/src/pages/public-login-theme-contract.test.ts`
  - `packages/app/src/pages/public-whop-activation-contract.test.ts`
- related_docs:
  - `docs/archive/plans/public-login-theme-contrast.md`
  - `docs/archive/index.md`
- related_prs: []
- verification:
  - `bun run test:web`：322 pass，0 fail
  - `bun --filter @hone-financial/app typecheck`
  - `HONE_APP_SURFACE=public HONE_APP_OUT_DIR=dist-public-theme-check bun --filter @hone-financial/app build`
  - `cd workers/public-community-edge && bun run typecheck && bun run test`：45 pass，0 fail
  - `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - `bash tests/regression/run_ci.sh`
  - 桌面与移动端浅色 / 暗色真实浏览器渲染、计算样式、横向溢出和 Whop 纵向滚动检查
- risks:
  - 没有保存偏好的首次访问默认浅色；已显式保存的 `light`、`dark`、`auto` 均继续保留
  - 未修改认证、验证码或 Whop membership 业务语义；验收没有发送验证码或提交表单
  - 本次只提交并推送源码，不部署生产、不创建 release 或 tag

## Summary

Public 未登录页现在把首次访问稳定初始化为浅色，并为品牌、卡片、输入框、复选框、按钮、链接与状态信息使用同一套语义颜色。显式切到暗色后，HONE 品牌字样和表单内容均保持清晰对比度。Whop 直达页会主动加载 Public 基础样式，品牌图恢复正常尺寸，移动端内容完整进入页面滚动区。

## What Changed

- Public surface 在路由加载前统一初始化偏好；缺少或损坏的主题存储值归一为 `light`，有效显式偏好不变。
- 共用登录表单增加主题入口，并以主题 token 取代写死的白色表面和浅色文字组合。
- Whop 页面直接导入 Public 基础样式，复用登录页语义表面，并为窄屏补齐纵向节奏、验证码行折叠和可滚动布局。
- 新增主题偏好、登录表面对比度和 Whop 直达样式契约回归。

## Verification

- 首次空存储访问得到 `preference=light`、`theme=light`。
- 登录页浅色品牌 / 标题对比度 15.16:1，暗色 14.68:1；输入文字分别为 16.63:1 与 16:1；暗色辅助文字 5.96:1，暗色禁用按钮 6.93:1。
- 1920px 桌面与 390px 手机登录页在两种主题下均无横向溢出。
- Whop 390x667 页面 `scrollHeight=724`，实际 `scrollY` 可从 0 移到 57，底部链接可见；`scrollWidth=384` 未超过 390px 视口。桌面与移动端的浅色 / 暗色渲染均已截图留证。
- 视觉证据位于 `/Users/zhangxuanren/.codex/visualizations/2026/08/01/019fbd41-5b0d-7e43-895b-9f788b259d0b/hone-login-theme/`。

## Risks / Follow-ups

- `auto` 仍会跟随系统主题，因此已有该显式偏好的用户不会被强制改成浅色。
- Public build 仍有仓库既有的 chunk-size warning，本次没有扩大为构建失败。
- 这次恢复既有主题与布局契约，没有改变模块边界、主数据流或长期架构，因此无需更新 `docs/repo-map.md`、`docs/invariants.md` 或 ADR。

## Next Entry Point

若生产页面仍显示旧样式，先确认 Pages 构建是否包含本提交，再检查根节点的 `data-theme-pref` / `data-theme` 与登录卡、品牌、输入框的计算样式；Whop 直达页同时确认 `public-foundation.css` 和 `public-site.css` 已进入对应 lazy chunk。
