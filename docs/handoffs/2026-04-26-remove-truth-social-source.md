# Remove Truth Social Source

- title: Remove Truth Social Source
- status: done
- created_at: 2026-04-26T04:07:59Z
- updated_at: 2026-04-26T04:07:59Z
- owner: Codex
- related_files:
  - `config.yaml`
  - `data/runtime/effective-config.yaml`
  - `crates/hone-core/src/config/event_engine.rs`
  - `crates/hone-core/src/config/mod.rs`
  - `crates/hone-event-engine/src/engine.rs`
  - `crates/hone-event-engine/src/pollers/social/mod.rs`
  - `crates/hone-event-engine/src/pollers/social/truth_social.rs`
  - `crates/hone-event-engine/src/digest/curation.rs`
- related_docs:
  - `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`
  - `docs/bugs/README.md`
  - `docs/archive/index.md`
- related_prs: N/A

## Summary

Truth Social 已从 event-engine 的活跃 source 集合中移除，不再作为可配置 poller 启动。历史 403 断流记录保留在 `docs/bugs/`，但不再作为活跃缺陷跟踪。

## What Changed

- 删除 `TruthSocialAccountConfig` 与 `Sources.truth_social_accounts` 配置字段。
- 删除 `TruthSocialPoller` 模块、公开导出、engine 装配循环和 live social E2E 中的空配置占位。
- 从 `config.yaml` 删除 `event_engine.sources.truth_social_accounts`。
- 从本机忽略的 `data/runtime/effective-config.yaml` 删除同一配置块，避免当前机器继续保留旧 source 配置。
- 将 digest 低质量社交源过滤收敛到现存 `watcherguru` source，避免继续保留已删除 source 的分支。
- 更新 Truth Social 断流 bug 为 `Closed`，并同步 `docs/bugs/README.md` 计数与状态。

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p hone-event-engine --lib`：332 passed, 14 ignored
- `cargo check -p hone-web-api`
- `rg -n "truth_social|TruthSocial|Truth Social|truth social" crates config.yaml config.example.yaml data/runtime/effective-config.yaml docs/repo-map.md docs/invariants.md docs/bugs/README.md docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`：只剩历史 bug 文档引用，`crates/`、`config*.yaml` 与本机 effective config 无活跃入口。

## Risks / Follow-ups

- 旧运行时日志和历史 `docs/releases/`、`docs/archive/`、既有 bug 证据仍会提到 Truth Social；这是历史记录，不代表当前支持。
- `data/runtime/effective-config.yaml` 是 ignored 生成态文件，这次只为当前机器清理；未来仍应以 `config.yaml` 为持久真相源。
- 若以后要恢复该源，应重新做 source POC，优先验证稳定 API / 鉴权 / 反爬策略，再重新引入配置和 poller。

## Next Entry Point

查看 `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md` 的关闭原因；当前实现入口从 `crates/hone-event-engine/src/engine.rs` 的 Telegram/RSS source 装配开始。
