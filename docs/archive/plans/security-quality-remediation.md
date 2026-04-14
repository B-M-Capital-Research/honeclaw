# GitHub Security / Quality 高优问题收口

- title: GitHub Security / Quality 高优问题收口
- status: archived
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/research.rs
  - crates/hone-web-api/src/routes/chat.rs
  - memory/src/company_profile.rs
  - memory/src/session.rs
  - .github/workflows/ci.yml
  - .github/workflows/release-cache-warm.yml
  - packages/app/package.json
  - Cargo.lock
  - docs/releases/v0.1.24.md
- related_docs:
  - docs/archive/index.md
  - docs/handoffs/2026-04-15-security-quality-remediation.md
  - docs/releases/v0.1.24.md
  - docs/handoffs/

## Goal

- 优先修复 GitHub 上当前高优的 security / quality issues，先覆盖可控且高收益的依赖漏洞、Actions 权限过宽以及明显的输入校验缺口。
- 对无法在当前回合内以低风险方式收口的问题，明确记录风险与停止边界，避免为了消除告警引入大改。
- 完成验证、文档同步、push 与版本 tag。

## Scope

- 先处理 `jspdf` 等高风险依赖告警，以及可能连带解决的 lockfile 告警。
- 为 GitHub Actions workflow 增加最小权限声明，消除 `missing-workflow-permissions`。
- 审查 `crates/hone-web-api/src/routes/research.rs` 中被标记为 SSRF 的接口，若可通过白名单 / scheme 校验低侵入修复则直接修。
- 审查 `memory` 中被标记为 path injection 的路径拼接点，若可通过相对路径约束和组件校验低侵入修复则直接修；若涉及广泛存储契约调整，则在风险中说明并停在安全边界。
- 审查 `chat.rs` 的明文日志告警，若可在不损失排障能力的前提下脱敏则直接修。

## Validation

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `cargo test --workspace --all-targets --exclude hone-desktop`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`
- 如有必要，补充针对修复点的定向测试或 lint / lockfile 更新验证

## Documentation Sync

- 更新 `docs/current-plan.md` 活跃任务索引。
- 发布前补齐 `docs/releases/v0.1.24.md`。
- 已补 `docs/handoffs/2026-04-15-security-quality-remediation.md`、`docs/releases/v0.1.24.md`，并将计划归档到 `docs/archive/plans/`，同时更新 `docs/archive/index.md`。
- 若修复过程改变长期约束或模块边界，再决定是否同步 `docs/invariants.md` / `docs/repo-map.md`；若没有，则在交付说明中明确无需更新原因。

## Risks / Open Questions

- `request-forgery` 与 `path-injection` 告警已在当前入口增加 URL / path 组件校验，但仍需等待 GitHub 重新扫描确认命中是否全部清零。
- 依赖升级可能引入前端或 Rust 侧兼容性变化，需以现有 CI 契约为准验证。
- `glib`（desktop GTK/Tauri 链）与 `lru` / `rand 0.10.0`（低优或上游 transitive 约束）未在本轮继续深挖，后者若要进一步收口，需要继续抬 `feishu-sdk` / `salvo_core` 上游依赖面。
