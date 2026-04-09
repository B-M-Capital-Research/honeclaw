# 定时任务输出净化与 Tavily 失败隔离

- title: 定时任务输出净化与 Tavily 失败隔离
- status: done
- created_at: 2026-03-31
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/deliverables.md`
- related_prs:
  - N/A

## Summary

隔离 heartbeat / 定时任务输出中的解释性文本，并把 Tavily 的临时搜索失败从会话上下文里剥离。

## What Changed

- heartbeat / 定时任务会从“前缀解释文本 + JSON”里抽出真正的 JSON 结果。
- 不再把解释过程和控制输出直接发给用户。
- `web_search` 在 Tavily 不可用时返回脱敏的 unavailable 结构。
- 这类临时失败结果不再持久化进会话工具上下文。

## Verification

- `cargo test -p hone-tools`
- `cargo test -p hone-channels`

## Risks / Follow-ups

- 以后如果搜索 provider 增加新的失败形态，应复用同样的“对用户可见但不污染上下文”的隔离策略。

## Next Entry Point

- `docs/archive/index.md`
