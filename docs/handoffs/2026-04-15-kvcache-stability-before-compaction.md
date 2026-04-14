# Handoff

- title: Pre-Compact KV Cache Stability
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: codex
- related_files:
  - crates/hone-channels/src/agent_session.rs
  - agents/codex_cli/src/lib.rs
  - docs/invariants.md
- related_docs:
  - docs/archive/plans/kvcache-stability-before-compaction.md
  - docs/archive/index.md
  - docs/invariants.md
- related_prs:
  - N/A

## Summary

按“compact 之前不应由 Hone 自己提前破坏 cache 命中”的要求，收敛了 pre-compact 恢复窗口、动态 related-skill prompt 注入位置，以及 `codex_cli` 的二次 recent-window 裁剪。

## What Changed

- `AgentSession` 的默认 restore window 现在不再早于 pre-compact 阈值滚动：
  - direct session 默认恢复上限改为 20
  - group session 至少恢复到 `compress_threshold_messages`
- 当前用户输入相关的 `related skills` 提示从 static system prompt 挪到了本轮 `runtime_input`
- `codex_cli` 不再额外只截最近 20 条上下文，而是直接消费 Hone 已准备好的上下文窗口
- `docs/invariants.md` 新增 pre-compact cache stability 约束

## Verification

- `cargo test -p hone-channels`
- `cargo test -p hone-agent-codex-cli`

## Risks / Follow-ups

- 这次修的是 Hone 自己可控的 pre-compact cache miss 源头，不保证底层 provider / ACP session 一定实现跨 turn KV cache 复用
- `gemini_cli` 仍按总 prompt 字节预算裁历史；若后续固定前缀继续变长，需要把 compression threshold 和 runner budget 再联动收口
- `group_context.recent_context_limit` 现在对 pre-compact restore 只作为下限参与，不再允许它把窗口压到 compact 阈值以下；如果后面要保留“更小 restore window”能力，应显式区分“cache-stable restore threshold”和“UI/history view limit”

## Next Entry Point

- `crates/hone-channels/src/agent_session.rs`
