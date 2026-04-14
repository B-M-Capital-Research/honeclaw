# Handoff

- title: GitHub Security / Quality Remediation
- status: done
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
- related_docs:
  - docs/archive/plans/security-quality-remediation.md
  - docs/archive/index.md
  - docs/releases/v0.1.24.md
- related_prs:
  - N/A

## Summary

优先收口了 GitHub 当前最值得先修的一批 security / quality issues：research proxy URL 边界、session / company profile 路径组件校验、console 明文 user id 日志、Actions workflow 权限声明，以及一组高优 transitive dependency 升级。

## What Changed

- `crates/hone-web-api/src/routes/research.rs`
  - 不再用字符串拼接 research proxy URL，改为先校验 `research_api_base`，只允许 HTTPS 远端地址或 HTTP loopback 地址
  - 统一通过 `Url` 构造路径与 query，并补了对应单元测试
- `memory/src/session.rs`
  - `session_id` 进入文件读写前必须通过单路径组件校验，阻断 `..` / 分隔符类路径逃逸
  - 新增非法 `session_id` 回归测试
- `memory/src/company_profile.rs`
  - profile id / 查询 id 进入文件系统前增加组件校验
  - `sanitize_id` 不再允许保留 `.` / `..` 这类危险边界
  - 新增画像 id 安全回归测试
- `crates/hone-web-api/src/routes/chat.rs`
  - 去掉明文 `user_id` 日志，仅保留渠道、附件数和消息长度
- `.github/workflows/ci.yml` 与 `.github/workflows/release-cache-warm.yml`
  - 增加顶层 `permissions: contents: read`
- 依赖升级
  - `jspdf` 升到 `4.2.1`
  - `aws-lc-rs` / `aws-lc-sys` 升到 `1.16.2` / `0.39.1`
  - `quinn-proto` 升到 `0.11.14`
  - `rustls-webpki` 升到 `0.103.10`
  - `salvo` 系列升到 `0.89.3`
  - `tar` 升到 `0.4.45`
  - `rand 0.9.x` 升到 `0.9.3`

## Verification

- `cargo test -p hone-web-api research -- --nocapture`
- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo test -p hone-memory create_session_rejects_parent_dir_component -- --nocapture`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `cargo test --workspace --all-targets --exclude hone-desktop`
- `bun run test:web`
- `bun run build:web`
- `bash tests/regression/run_ci.sh`
- `rustfmt --edition 2024 --check crates/hone-web-api/src/routes/research.rs crates/hone-web-api/src/routes/chat.rs memory/src/company_profile.rs memory/src/session.rs`

## Risks / Follow-ups

- GitHub code scanning / dependabot 结果需要等 push 后重新扫描，当前只能基于代码与 lockfile 变更判断已覆盖相应高优告警
- `glib` 告警位于 `hone-desktop` 的 GTK/Tauri 依赖链，超出默认 CI 覆盖面；若要继续收口，需要在 desktop 依赖面上做更大升级验证
- `lru` 与 `rand 0.10.0` 仍然属于低优；其中 `rand 0.10.0` 目前仍由 `feishu-sdk -> salvo_core` 链带入，要完全清零需继续评估更上游版本
- `scripts/ci/check_fmt_changed.sh` 在本机 `/bin/bash 3.2` 下因 `mapfile` 不可用无法直接运行；本次改动已用针对改动文件的 `rustfmt --check` 补验证

## Next Entry Point

- `crates/hone-web-api/src/routes/research.rs`
