# macOS DMG Release 打包收口

- title: macOS DMG Release 打包收口
- status: done
- created_at: 2026-03-31
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `make_dmg_release.sh`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

补齐 macOS release DMG 产物构建链路，并把 packaged/runtime 启动时需要的环境与启动锁重试一并收口。

## What Changed

- 新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG。
- Release 包内置 `hone-mcp` 与 macOS `opencode`。
- Desktop packaged/runtime 启动时会补齐 app sandbox data/runtime/sandbox 环境。
- Bundled runtime 启动锁冲突时增加按 pid 的定向清理重试。

## Verification

- `cargo test -p hone-channels runners::tests::resolve_opencode_command_prefers_bundled_env_override -- --exact`
- `cargo test -p hone-channels sandbox::tests::sandbox_base_dir_prefers_hone_data_dir_before_temp -- --exact`
- `cargo check -p hone-desktop -p hone-channels -p hone-mcp`
- `node --check scripts/prepare_tauri_sidecar.mjs`
- `bash -n make_dmg_release.sh`
- `./make_dmg_release.sh x86_64-apple-darwin`
- `./make_dmg_release.sh aarch64-apple-darwin`
- `./launch.sh --desktop`

## Risks / Follow-ups

- 后续若继续改 packaged runtime 或 sidecar 打包行为，应先复查 DMG 产物是否仍带齐 bundled runtime 依赖。

## Next Entry Point

- `docs/archive/index.md`
