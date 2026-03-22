# Handoff: 飞书卡片三问题修复

日期：2026-03-15  
状态：已完成

## 本次目标

修复飞书卡片渠道的三个体验问题：
1. h3+标题与标准 Markdown 表格在卡片中不渲染
2. 卡片视觉偏小
3. 流式输出竞态导致最终卡片被截断内容覆盖

## 已完成

**`bins/hone-feishu/src/main.rs`**：

- 新增辅助函数：`extract_deep_heading`、`is_table_header_line`、`is_table_separator_line`、`parse_table_row`、`convert_table_to_feishu`
- 新增核心预处理函数：`preprocess_markdown_for_feishu(text, convert_tables)`
  - h3+ 标题 → `**标题**`
  - 标准 Markdown 表格 → 飞书 `<table columns={...} data={...}/>` 语法
  - `convert_tables=false` 用于流式中间更新，避免部分表格渲染错误
- 所有卡片 JSON 加入 `"config": {"wide_screen_mode": true}`（5 处：`make_streaming_card`、`send_placeholder_message`、`update_or_send_plain_text`、`render_outbound_messages`、`send_rendered_messages` 的 post→card 转换路径）
- 流式竞态修复：`handle.abort(); let _ = handle.await;` 确保 ticker 完全结束后再做最终卡片更新
- 新增 8 个单元测试覆盖预处理逻辑

## 验证

- `cargo check -p hone-feishu` ✅
- `cargo test -p hone-feishu` ✅（12/12）

## 影响范围

仅 `bins/hone-feishu/src/main.rs`，其他渠道与模块不受影响。

## 剩余风险

- 飞书 `<table/>` 组件要求每张卡片最多 5 个表格、最多 10 列；超出限制的表格飞书会降级为文本显示。当前实现不检查此限制，若 LLM 输出含超量表格，超出部分自动降级显示。
- 流式输出的表格在生成完成前不做转换（`convert_tables=false`），用户会先看到原始 `|---` 管道符，最终发送时才看到正式表格——这是已知的权衡，符合预期。
