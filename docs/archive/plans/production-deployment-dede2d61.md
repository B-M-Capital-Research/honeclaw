# Production Deployment dede2d61

- title: 远端 10 提交审查与生产部署
- status: archived
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - packages/app/src/lib/public-content.ts
  - packages/app/src/pages/chat.tsx
  - crates/hone-channels/src/investment_response_guard.rs
  - .github/workflows/runtime-image.yml
  - scripts/stage_ghcr_runtime.sh
- related_docs:
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-08-04-production-deployment-dede2d61.md

## Goal

审查 `ee7024b6..dede2d61` 的十个远端提交，确认国际化、Stripe-only 文档与扩展时段行情修复没有阻断性问题；随后以精确 Git revision 和 GHCR digest 部署生产后端，并验收 Cloudflare Pages、公共 API、管理员面与云权威状态。

## Scope

- 逐提交和逐模块审查 63 个变更文件，重点检查语言偏好传递、中文回退、组件状态、扩展时段行情预载、管理员 API 和计费/Discord 文档一致性。
- 跑仓库默认 CI 门禁、Public 生产构建和与变更相关的聚焦回归；发现阻断问题时先修复并形成新目标 revision。
- 等待目标 revision 的 GitHub Actions runtime image，核验镜像 provenance 和 immutable digest。
- 通过私有 IAP 连接核对 GCE 生产基线，执行 runtime env 检查、两次 active-chat idle 读取、不可变 staging、原子切换、systemd 重启与回滚保留。
- 验收 `/api/meta` 云权威字段、public 未登录边界、前端资产 revision、管理员权限边界和财报 skill/PDF 运行依赖。

## Validation

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `cd workers/public-community-edge && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- `bun run build:web:public`
- GitHub Actions runtime-image SHA/digest/provenance；生产 `/api/meta`、active-chat、public API、Cloudflare Pages 资产、skill registry 和官方 PDF renderer smoke。

## Documentation Sync

- 已将不含具体生产主机标识的稳定 IAP/OS Login、外置 skill 与 PDF runtime 依赖更新到 `docs/runbooks/backend-deployment.md`；具体连接命令仅保存在被忽略的 `.git-tools/production-gce.md`。
- 已新增 `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`，本计划已归档并从 active index 移除。

## Risks / Open Questions

- `origin.hone-claw.com` 的旧 Sunny-Ngrok alias 仍失效；用户主站 API 健康，但 legacy fallback 激活前必须单独修复或重新决策。
- executable-only GHCR release 与外置技能资产可能发生 revision 漂移；本次已增加强制 hash/readback/renderer 门禁，长期应考虑将版本化技能纳入 immutable release。
