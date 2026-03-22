# 分段输出 Markdown 换行修复交接

日期：2026-03-16

## 变更摘要

- 修复首段分段输出被吞换行：`clean_msg_markers` 不再将所有空白压成单空格，改为仅压缩空格/制表并保留换行。
- 新增单元测试确保 Markdown 中的空行与列表换行不被破坏。

## 影响范围

- 影响所有使用 `clean_msg_markers` 的分段路径（Web SSE 分段、通用流式分段）。

## 验证

- 未运行。建议：`cargo test -p hone-channels`。

## 风险与注意事项

- 若 downstream 依赖“强制单行化”行为，可能出现显示格式变化；目前预计只会改善 Markdown 格式。
