- title: Desktop OpenRouter Key Pool Migration Gap
- status: done
- created_at: 2026-04-14 23:13 CST
- updated_at: 2026-04-14 23:46 CST
- owner: Codex
- related_files:
  - crates/hone-core/src/config.rs
  - docs/bugs/desktop_openrouter_key_pool_migration_gap.md
  - docs/current-plans/canonical-config-runtime-apply.md
- related_docs:
  - docs/current-plan.md
  - docs/runbooks/desktop-release-app-runtime.md
  - docs/bugs/desktop_openrouter_key_pool_migration_gap.md
- related_prs:

## Summary

已修复 desktop legacy runtime 漏迁 `llm.openrouter.api_keys` 的源码缺口，并补上自动化回归测试。随后已完成一次正式重启验证：新的 release desktop `.app` 已启动，remote-mode 对应的 release backend 与启用渠道也已切到 `/tmp/honeclaw-target/release` 这一套 release 二进制。

## What Changed

- 在 `crates/hone-core/src/config.rs` 中补齐 `promote_legacy_runtime_agent_settings(...)` 对 `llm.openrouter.api_keys` 的迁移。
- 收紧 `llm.openrouter.api_key` 的 legacy 单值补迁，只在 legacy 值非空时才写回 canonical，避免空字符串伪变更。
- 新增回归测试，覆盖 canonical key 池为空而 legacy 仅持有 `llm.openrouter.api_keys` 的升级场景。
- 更新 `docs/bugs/desktop_openrouter_key_pool_migration_gap.md` 状态为 `Fixed`，并记录本次验证证据。

## Verification

- `cargo test -p hone-core promote_legacy_runtime_agent_settings`
- `cargo check -p hone-core --all-targets`
- `cargo fmt --all`
- `rustfmt --edition 2024 --check crates/hone-core/src/config.rs`
- release app build:
  - `env CARGO_TARGET_DIR=/tmp/honeclaw-target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
  - 结果：`.app` 已产出到 `/tmp/honeclaw-target/release/bundle/macos/Hone Financial.app`
  - 附加失败：DMG bundling 仍失败于 `bundle_dmg.sh`
- runtime restart validation:
  - desktop process: `/tmp/honeclaw-target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`
  - backend process: `/tmp/honeclaw-target/release/hone-console-page`
  - channel processes: `/tmp/honeclaw-target/release/hone-discord`、`/tmp/honeclaw-target/release/hone-feishu`、`/tmp/honeclaw-target/release/hone-telegram`
  - `curl http://127.0.0.1:8077/api/meta`
  - `curl http://127.0.0.1:8077/api/channels`
  - `/api/channels` 已确认 `discord` / `feishu` / `telegram` 都为 `running`

## Risks / Follow-ups

- desktop 当前持久化后端模式仍是 `remote`，因此这次正式重启采用了“release `.app` + release backend/channel binaries”组合，而不是 bundled sidecar 全包式启动。
- 由于 DMG bundling 失败，正式 release 资产仍需继续检查 `bundle_dmg.sh` 失败原因。
- 本次只完成代码修复与运行态重启验证；未执行 git commit / push / tag / release。

## Next Entry Point

1. 若要继续发布，先修复 DMG bundling 失败，再补对应版本的 `docs/releases/vX.Y.Z.md`。
2. 在当前运行态基础上，继续执行 git commit、push、tag 和 release 前，请先确认是否仍希望保持 `remote` desktop 模式。
3. 如需再次重启，复用当前 release 形态：
   - desktop: `/tmp/honeclaw-target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`
   - backend/channels: `/tmp/honeclaw-target/release/hone-console-page`、`hone-discord`、`hone-feishu`、`hone-telegram`
