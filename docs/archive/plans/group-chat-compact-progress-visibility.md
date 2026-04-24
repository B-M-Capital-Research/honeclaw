- title: Group Chat Compact Progress Visibility
- status: done
- created_at: 2026-04-17
- updated_at: 2026-04-17 11:18 CST
- owner: Codex
- related_files:
  - crates/hone-channels/src/outbound.rs
  - bins/hone-telegram/src/handler.rs
  - bins/hone-telegram/src/listener.rs
  - bins/hone-discord/src/handlers.rs
  - bins/hone-discord/src/utils.rs
  - bins/hone-feishu/src/handler.rs
  - bins/hone-feishu/src/listener.rs
- related_docs:
  - docs/archive/index.md

## Goal

让群聊也能看到模型处理中间阶段，同时避免把 query、命令行、目录路径等过细执行细节直接暴露到群里。

## Scope

- 在共享 outbound 层引入 `Hidden | Full | Compact` 三档进度可见性
- Telegram / Discord 群聊从“完全不显示中间进度”改为 `Compact`
- Feishu 群聊对齐到相同的 `Compact` 粒度
- 保留私聊原有的 `Full` 可见性
- 当 runner 只给出泛化工具名（如 `Tool`）时，优先从 `reasoning` 中推断粗粒度动作标签
- `Compact` 模式按轮次追加进度，不再把连续相同类型的工具调用去重折叠

## Validation

- `cargo test -p hone-channels outbound::tests -- --nocapture`
- `cargo test -p hone-feishu listener -- --nocapture`
- `cargo check --workspace --all-targets --exclude hone-desktop`

## Documentation Sync

- 已更新 `docs/archive/index.md`
- 未更新 `docs/repo-map.md` / `docs/invariants.md`：本轮是局部渠道 UX 与脱敏粒度调整，不涉及模块边界或长期架构约束变化

## Risks / Open Questions

- `Compact` 模式当前基于工具名 / 命令启发式压缩；若后续 runner 引入新的工具标签格式，可能需要补映射以维持稳定措辞
- Telegram / Discord 仍只压缩 `start` 阶段提示；Feishu 额外压缩了 `done` 文案以避免命令细节回流
- 若未来 `reasoning` 文本格式变化过大，`Tool` 这类泛化标签的动作推断可能退化到更保守的默认措辞
