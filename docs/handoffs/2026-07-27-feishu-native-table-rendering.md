# Feishu Native Table Rendering Handoff

- title: Feishu Native Table Rendering
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `bins/hone-feishu/src/markdown.rs`
  - `bins/hone-feishu/src/outbound.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `docs/repo-map.md`
  - `docs/bugs/feishu_raw_table_component_code_leak.md`
- related_docs:
  - `docs/archive/plans/feishu-native-table-rendering.md`
  - `docs/archive/plans/feishu-table-sanitization.md`
  - `docs/archive/index.md`
- related_prs: N/A（本地工作树改动，未提交或推送）

## Summary

用户截图中的 `<table columns={[...]}` 源码来自 Feishu 出站层自身：它把标准 Markdown 表格序列化成 raw tag，再把该字符串放进 JSON 2.0 卡片的 `markdown.content`。飞书不会在 Markdown 元素内解释这种组件语法。修复后，表格在消息拆分前被解析并输出为卡片根节点原生 `table` 元素。

## What Changed

- 标准 Markdown 表格与可解析的历史 raw table 共用结构化解析模型，再生成 JSON 2.0 原生 `table`。
- 单卡最多输出 5 个原生表格、单表最多 50 列；拆分发生在表格解析后，避免把表格切断。
- 损坏 raw table 变成明确错误提示；Markdown-only / 旧流式路径变成可读列表，所有降级都不包含组件源码。
- placeholder 最终更新与 direct / scheduler 最终发送共用原生卡片渲染器。
- Feishu prompt 只要求标准 Markdown 表格，并明确由运行时完成原生组件转换。
- 用户截图中的 `dataIndex` / `col0` 结构、表格上限、损坏输入和 placeholder 路径均有回归覆盖。

## Verification

- `cargo test -p hone-feishu markdown -- --nocapture`：18 个相关测试通过。
- `cargo test -p hone-feishu outbound -- --nocapture`：6 个相关测试通过。
- `cargo test -p hone-feishu -- --nocapture`：69 个测试通过。
- `cargo test -p hone-channels prompt --lib -- --nocapture`：55 个相关测试通过。
- `cargo check -p hone-feishu --tests` 与 `cargo check -p hone-channels --tests` 通过。
- 两条既有 shared sanitizer / scheduler raw-table 回归通过。
- 本次 Rust 文件 scoped `rustfmt --check` 与相关文件 `git diff --check` 通过。
- `bash scripts/ci/check_fmt_changed.sh` 仍因 `HEAD^...HEAD` 中无关既有文件的格式差异失败；未修改那些用户变更。

## Risks / Follow-ups

- 尚未部署、重启或进行真实 Feishu 实发。新版本加载后，应分别用 direct 与 scheduler 的标准 Markdown 表格复核桌面端和移动端显示。
- 若真实客户端不支持当前原生表格字段组合，可保留结构化解析，把最终输出切换为已有测试的列表降级；不要恢复 `<table .../>` 内联文本。
- 工作树存在大量无关用户改动和一个无关冲突，提交时需只选择本 handoff 列出的任务文件。

## Next Entry Point

先查看 `bins/hone-feishu/src/markdown.rs` 的 `render_feishu_card_content` 与相关测试，再从 `bins/hone-feishu/src/outbound.rs` 的 placeholder 更新入口确认所有最终发送路径仍使用同一渲染器；真实复核结果追加到 `docs/bugs/feishu_raw_table_component_code_leak.md`。
