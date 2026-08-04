# Production Deployment dede2d61

- title: 远端 10 提交审查与生产部署
- status: in_progress
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
  - docs/current-plan.md
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-08-04-production-deployment-dede2d61.md

## Goal

审查 `ee7024b6..dede2d61` 的十个远端提交，确认国际化、Stripe-only 文档与扩展时段行情修复没有阻断性问题；随后以精确 Git revision 和 GHCR digest 部署生产后端，并验收 Cloudflare Pages、公共 API、管理员面与云权威状态。

## Scope

- 逐提交和逐模块审查 63 个变更文件，重点检查语言偏好传递、中文回退、组件状态、扩展时段行情预载、管理员 API 和计费/Discord 文档一致性。
- 跑仓库默认 CI 门禁、Public 生产构建和与变更相关的聚焦回归；发现阻断问题时先修复并形成新目标 revision。
- 等待目标 revision 的 GitHub Actions runtime image，核验镜像 provenance 和 immutable digest。
- 通过私有 IAP 连接核对 GCE 生产基线，执行 runtime env 检查、两次 active-chat idle 读取、不可变 staging、原子切换、systemd 重启与回滚保留。
- 验收 `/api/meta` 云权威字段、origin/public 未登录边界、前端资产 revision、管理员入口和扩展时段行情行为。

## Validation

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `cd workers/public-community-edge && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- `bun run build:web:public`
- GitHub Actions runtime-image SHA/digest/provenance；生产 `/api/meta`、active-chat、origin/public API、Cloudflare Pages 资产与真实管理员 smoke。

## Documentation Sync

- 将不含具体生产主机标识的稳定 GCE/IAP/OS Login 约束更新到 `docs/runbooks/backend-deployment.md`；具体连接命令只保存在被忽略的 `.git-tools/production-gce.md`。
- 完成后新增 `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`，将本计划移入 `docs/archive/plans/`，更新 `docs/current-plan.md` 与 `docs/archive/index.md`。

## Risks / Open Questions

- 远端同时包含大规模国际化和投研运行时修复，必须防止只验证 UI 而遗漏 prompt/API 契约变化。
- 生产重启必须以两次零活跃会话和可回滚旧 release 为前置；任何云权威、环境校验、镜像 provenance 或公网验收失败都应立即停止或回滚。
- 关闭实例级 OS Login 2FA 降低登录摩擦，也降低一层访问防护；只能保留 IAP、IAM/OS Login 最小权限和审计，不把具体主机或项目标识写进公开仓库。
