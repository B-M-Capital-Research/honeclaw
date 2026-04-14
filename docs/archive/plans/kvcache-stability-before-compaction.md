# Plan

- title: Pre-Compact KV Cache Stability
- status: archived
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: codex
- related_files:
  - crates/hone-channels/src/agent_session.rs
  - crates/hone-channels/src/prompt.rs
  - crates/hone-channels/src/runners/acp_common.rs
  - agents/codex_cli/src/lib.rs
  - docs/invariants.md
- related_docs:
  - docs/archive/index.md
  - docs/handoffs/2026-04-15-kvcache-stability-before-compaction.md
  - docs/invariants.md
  - docs/adr/0002-agent-runtime-acp-refactor.md

## Goal

把“上一次 compact 之后到下一次 compact 之前，Hone 自己生成的上下文前缀应尽量保持稳定，不要由 Hone 自己提前滚动或改写而破坏 cache 命中”收敛为实现约束。

## Scope

- 识别并收敛 pre-compact 阶段会提前破坏 cache 的主要来源
- 优先处理 Hone 可控的 prompt / restore / runner 侧行为
- 不在本轮引入新的外部 provider 依赖或 runner 大重构

## Validation

- `cargo test -p hone-channels`
- `cargo test -p hone-agent-codex-cli`

## Documentation Sync

- 已更新 `docs/invariants.md` 记录 pre-compact cache-stability 约束
- 已从 `docs/current-plan.md` 移除活跃索引并追加 archive 索引

## Risks / Open Questions

- 不同 runner / provider 对 prompt cache 或 message cache 的实现不同，本轮只能保证 Hone 自身不主动制造额外 cache miss
- Gemini CLI 仍有总 prompt 字节上限；若固定前缀继续膨胀，未来可能还需要把 compression threshold 与 runner budget 再统一收口
