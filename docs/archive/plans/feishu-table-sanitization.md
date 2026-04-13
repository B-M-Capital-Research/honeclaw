# 飞书表格语法护栏

- title: 飞书表格语法护栏
- status: done
- created_at: 2026-04-13
- updated_at: 2026-04-13
- owner: shared
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `bins/hone-feishu/src/markdown.rs`
- related_docs:
  - `docs/archive/index.md`

## Goal

收紧飞书渠道的表格输出约束，避免模型手写损坏的 `<table .../>` 卡片标签直接漏到用户侧，同时保留标准 Markdown 表格到飞书表格卡片的自动转换能力。

## Outcome

- 飞书提示词现在明确要求：正文/列表使用普通 Markdown；需要表格时只写标准 Markdown 表格，不要手写飞书原始卡片标签。
- `preprocess_markdown_for_feishu` 新增 raw table sanitizer：
  - 合法的 canonical `<table columns={...} data={...}/>` 会被规范化后保留。
  - 非法、截断、字段名错误或 JSON 损坏的 raw table 会安全降级为普通文本，不再作为 live card markup 发给飞书。
- 流式更新、placeholder 更新和最终消息发送路径都复用同一个预处理入口，因此自动继承这层保护。
- 为标准 Markdown 表格、合法 raw table、损坏 raw table、用户样例回归和多段拆分场景补齐单测。

## Validation

- `cargo test -p hone-feishu markdown`
- `cargo test -p hone-channels prompt`

## Risks / Open Questions

- 当前工程护栏只对 raw `<table .../>` 做规范化/降级；其它飞书原始标签（如 `<chart .../>`、`<row>`）目前主要仍靠提示词约束。
- 损坏 raw table 的降级策略是“保留为转义后的普通文本”；如果后续需要更强 UX，可再考虑统一替换为结构化错误提示。
