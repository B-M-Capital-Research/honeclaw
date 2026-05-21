# Bug: Feishu PDF 文本提取在 CMap 解析越界 panic 后只能降级读首页

- 发现时间：2026-05-22 03:03 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无；当前不是 P1，本轮未创建 issue。

## 证据来源

- `data/sessions.sqlite3` 最近四小时真实 Feishu 直聊窗口：
  - `session_id=Actor_feishu__direct__ou_5f0bdff19e3e341fbbbffe811abecaac61`
  - `2026-05-21T23:51:07+08:00`、`2026-05-21T23:55:02+08:00`、`2026-05-22T00:55:06+08:00`，同一用户三次上传同一 17 页 PDF `rubin 价值量拆解-译文.pdf`。
  - 三条 user turn 均落库为 `下载状态=成功`，但 `PDF解析状态=失败(PDF 提取任务失败: task ... panicked with message "index out of bounds: the len is 11414 but the index is 11414")`。
  - 对应 assistant final 没有把 panic 或绝对路径继续外发，但只能说明“系统文本提取失败 / 只能读首页”，无法完整基于后续 16 页正文和表格回答。
- `data/runtime/logs/hone-feishu.runtime-recovery.log`
  - `2026-05-21 23:51 CST`、`23:55 CST`、`2026-05-22 00:55 CST` 三次出现同一 worker panic：
    - `adobe-cmap-parser-0.4.1/src/lib.rs:195:41`
    - `index out of bounds: the len is 11414 but the index is 11414`
- 去重检查：
  - `feishu_attachment_internal_transcript_leak.md` 覆盖图片附件内部 transcript 外泄，当前未见 assistant final 外泄。
  - `feishu_company_profile_absolute_path_leak.md` 覆盖 assistant final 暴露本机路径，当前 final 已避免路径外发。
  - 现有台账未覆盖 PDF 文本提取器对特定 CMap 越界 panic，导致附件正文不可用的链路。

## 端到端链路

1. Feishu direct 用户上传 PDF 附件，期望系统读取报告正文并做投资分析。
2. 附件下载成功，PDF 文本提取任务开始解析。
3. PDF 内嵌 CMap 解析触发 `adobe-cmap-parser` 越界 panic。
4. 系统把 PDF 文本提取标记为失败，并把失败摘要写入本轮 user turn / prompt。
5. assistant 只能基于 PDF 元数据或后续图片化首页读到的有限内容回答，无法覆盖完整 17 页正文。

## 期望效果

- PDF 文本提取遇到异常 CMap 时不应让 worker panic，也不应让同一 PDF 每次上传都重复失败。
- 至少应保留可恢复降级：跳过坏 CMap、继续提取其它页面文本，或稳定转图片 / OCR 并给出明确的页面覆盖范围。
- 面向 LLM 的附件摘要应使用脱敏、稳定的错误类别，不夹带本机绝对路径、第三方 crate 源码路径或 panic 细节。

## 当前实现效果

- 同一 PDF 三次上传均触发同一 `index out of bounds` panic。
- 用户任务没有完全失败，assistant 仍有最终回复；但核心输入材料的 16 页正文不可用，回答只能降级到首页摘要。
- session user turn 与运行日志记录了本机路径和第三方 crate panic 位置；本轮未见这些细节出现在 assistant final，但它们已进入 agent 上下文，增加后续外泄风险。

## 用户影响

- 这是功能性 bug，不是单纯回答质量问题。
- 用户上传的是投研 PDF，系统成功下载却无法稳定解析正文，导致分析覆盖不完整，无法完成“基于整份报告理解/拆解”的主目标。
- 定级为 `P2`：它阻断 Feishu PDF 附件理解链路的一类真实文件，但本轮只确认单个会话 / 单份 PDF，且 assistant 有明确降级说明，没有出现跨用户错投、无回复或批量出站失败，因此不定为 P1。

## 根因判断

- 直接根因是 PDF 文本提取链路依赖的 CMap 解析器对边界索引缺少保护，导致 `len == index` 时 panic。
- 目前看缺少 panic containment / per-page fallback：单个字体或 CMap 异常会使整份 PDF 文本提取失败。
- 附件摘要把内部失败细节带入 user turn，说明用户可见输出层虽能避免外发，但 prompt / session 层仍缺少对附件解析错误的脱敏归一化。

## 下一步建议

- 在 PDF 提取边界包一层 `catch_unwind` 或隔离 worker，将 parser panic 归一化为可恢复错误，不让单页 / 单字体失败拖垮整份 PDF。
- 对 CMap 失败补跳过策略或 alternate extractor fallback，并记录页面级覆盖率。
- 对进入 session / prompt 的附件解析错误做脱敏，只保留 `pdf_text_extract_failed`、页数、已成功的 fallback 类型和简短原因。
- 增加回归样本：同一 PDF 或最小 CMap 复现 fixture 应验证“不 panic、不中断整份附件处理、不会把本机路径 / crate 源码路径写进 LLM 可见摘要”。
