# 跨渠道富文本分段渲染修复

- title: 跨渠道富文本分段渲染修复
- status: done
- created_at: 2026-04-13
- updated_at: 2026-04-13
- owner: codex
- related_files:
  - `docs/current-plan.md`
  - `docs/archive/index.md`
  - `crates/hone-channels/src/outbound.rs`
  - `bins/hone-telegram/src/listener.rs`
  - `bins/hone-telegram/src/scheduler.rs`
  - `bins/hone-discord/src/utils.rs`
  - `bins/hone-feishu/src/markdown.rs`
- related_docs:
  - `docs/archive/index.md`

## Goal

修复长回复在多渠道分段发送时破坏富文本结构的问题，避免 Telegram HTML tag、Discord Markdown 代码块、Feishu Markdown 结构在分段边界失衡后降级或渲染异常。

## Scope

- 用运行时日志确认 Telegram 长回复的 HTML 解析错误路径
- 在共享分段层新增 HTML / Markdown 感知分段器
- 让 Telegram / Discord / Feishu 使用匹配各自富文本格式的分段器
- 保持 iMessage 纯文本分段路径不变

## Validation

- `cargo check -p hone-channels -p hone-telegram -p hone-discord -p hone-feishu`
- `cargo test -p hone-channels outbound::tests::split_html_segments_rebalances_open_tags_across_segments -- --exact`
- `cargo test -p hone-channels outbound::tests::split_markdown_segments_rebalances_code_fences_across_segments -- --exact`
- 运行时日志确认原始问题形态：
  - `Bad Request: can't parse entities: Can't find end tag corresponding to start tag "pre"`
  - `Bad Request: can't parse entities: Unexpected end tag`

## Documentation Sync

- 已从 `docs/current-plan.md` 移除活跃索引
- 已归档到 `docs/archive/plans/`
- 无需更新 `docs/repo-map.md`，因为本次未改变模块边界，只是修复共享分段策略

## Risks / Open Questions

- 当前 Markdown 感知分段优先保护代码 fence；若后续发现表格、复杂引用或其它块级语法也会在分段边界失真，可继续在共享层扩展 block-aware 规则
- 当前 HTML 感知分段基于轻量 tag stack 补全策略，适配 Telegram 支持的简单 HTML 集；若未来新增更复杂的 HTML 结构，再考虑升级 parser
