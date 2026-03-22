# 2026-03-11 iMessage 稳定性收口

## 归并范围

- 大量旧消息误触发修复
- session 文件并发写保护
- Gemini CLI `E2BIG` prompt 预算治理

## 已完成

- `hone-imessage` 轮询查询增加基于 macOS epoch 的时间窗保护，避免 iCloud 同步把多年历史消息重新当作“新消息”处理。
- `memory/src/session.rs` 引入基于 `session_id` 的细粒度进程内锁，修复并发追加时的最后写覆盖问题。
- `agents/gemini_cli` 为 prompt 构建增加总预算、工具结果截断与按新到旧的历史裁剪，避免 `Argument list too long (os error 7)`。
- 各主要渠道的 `restore_context` 统一收口为最多 12 条历史消息；session 压缩阈值同步收紧。

## 验证

- `cargo test --workspace --all-targets`
- `cargo test -p hone-memory`
- `cargo check -p hone-agent-gemini-cli -p hone-imessage -p hone-channels -p hone-feishu -p hone-discord`

## 风险

- session 锁为单进程内存锁；若多个进程同时直接写同一 session 目录，仍需依赖更高层的调度/运行约束避免冲突。
