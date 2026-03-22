# AgentSession 覆盖率提升（Gemini CLI）交接

日期：2026-03-16

## 变更摘要

- `stream_gemini_prompt` 支持通过环境变量 `HONE_GEMINI_BIN` 指定 gemini CLI 路径，便于本地/测试替身。
- 新增 mock gemini CLI 单测，覆盖 content/thought/error/context overflow 分支。
- 新增手工回归脚本 `tests/regression/manual/test_gemini_streaming.sh`。

## 影响范围

- 影响所有 Gemini CLI 流式调用路径（默认仍使用 `gemini`，仅在环境变量存在时替换）。

## 验证

- 未运行。建议：
  - `cargo test -p hone-channels`
  - `bash tests/regression/manual/test_gemini_streaming.sh`

## 风险与注意事项

- 环境变量仅用于覆盖命令路径，不影响默认行为。
- 手工回归脚本依赖本地 gemini-cli 和登录态。
