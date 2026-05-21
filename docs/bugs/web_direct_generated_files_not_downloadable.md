# Bug: Web 直聊生成 Excel/CSV 只回文件名，手机端无法下载或打开

- **发现时间**: 2026-05-21 15:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无；本单不是 P1，暂不创建。

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_web__direct__web-user-f40ae1caa720`
  - `2026-05-21T14:20:48+08:00`，用户要求把投资策略整理成 Excel 表格，字段包括类型、标的、代码、金额、投入金额时间等。
  - `2026-05-21T14:23:53+08:00`，assistant final 回复“已整理成 Excel，共三页”，但用户可见正文只给出文件名 `A股三年投资策略表.xlsx`，没有可下载附件或链接。
  - `2026-05-21T14:27:17+08:00`，用户反馈“我看不到文件”。
  - `2026-05-21T14:32:38+08:00`，assistant 再次声称已生成 Excel 和 CSV，并只列出 `A股三年投资策略表.xlsx` 与 `A股三年投资策略表.csv`。
  - `2026-05-21T14:33:15+08:00`，用户仍然看不到，要求改成纯文本表格。
  - `2026-05-21T14:36:16+08:00` 到 `14:49:24+08:00`，用户尝试询问文件中转站和 Google Drive 连接入口；assistant 继续建议启用连接器或换电脑端，但未能把本轮生成文件交付成手机端可打开的附件。
- 同窗会话质量对照：
  - 最近四小时共有 `20` 个 user turn 与 `20` 个 assistant final，Feishu / Web 直聊均有收口。
  - assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`Param Incorrect`、`Resource temporarily unavailable`、`reasoning_content` 或 provider 原始 `quota exhausted`。
  - 因此本单不是全局直聊失败，而是 Web direct 生成文件后的 artifact / 下载交付链路缺陷。
- 去重检查：
  - `web_scheduler_mobile_push_not_delivered.md` 覆盖 Web scheduler 手机系统通知承诺与真实投递能力不一致。
  - `feishu_company_profile_absolute_path_leak.md` 覆盖本机路径外泄。
  - 现有台账未覆盖 Web direct 生成 Excel/CSV 后无法以用户可下载文件交付的链路。

## 端到端链路

1. Web direct 用户要求生成 Excel 或 CSV 表格。
2. agent 在 Web actor sandbox 内尝试生成或查找文件。
3. 最终回复只把本地文件名写入文本，没有绑定为 Web 前端可见附件、下载 URL 或可点击 artifact。
4. 用户在手机端只能看到文件名，无法打开或下载。
5. 后续对话退化为让用户找 Google Drive / Connectors 入口或改用纯文本复制，原本的文件交付目标没有完成。

## 期望效果

- 当 assistant 声称生成 Excel / CSV / PDF / 图片等文件时，Web 用户应能在当前页面直接看到可下载附件、可点击 artifact、或明确的“当前 Web 端不支持文件交付，以下改用纯文本/CSV 文本”的替代输出。
- 如果当前 Web direct 不具备上传网盘或外部中转能力，assistant 不应反复声称“文件已生成/重新发送”但只给文件名。
- 移动端和桌面端都应有一致、可验证的文件交付反馈。

## 当前实现效果

- assistant final 多次只返回文件名，未形成手机端可点击下载入口。
- 用户连续反馈看不到文件，说明从用户视角任务未完成。
- assistant 后续建议连接 Google Drive，但用户当前 Web 手机端没有可见连接器入口；这仍不能交付已经生成的文件。

## 用户影响

- 这是功能性 bug，不是单纯表达质量问题。
- 用户要求的是可打开的 Excel/CSV 文件，最终只能拿到文件名或纯文本替代方案，核心交付物不可用。
- 定级为 `P2`：它会阻断 Web direct 的文件产出交付链路，影响用户完成任务；但不涉及跨用户错投、数据破坏、系统级未回复或批量消息投递失败，因此不定为 P1。

## 根因判断

- 初步判断是 Web direct 的 agent sandbox 文件与用户可见 artifact / 附件系统之间没有稳定桥接。
- 回复层缺少“文件已生成但无法投递”的能力边界判断，导致 assistant 把本地文件名当成交付结果。
- Google Drive / 外部中转能力也没有在当前 Web 手机端形成可用入口，不能作为默认兜底。

## 下一步建议

- 梳理 Web direct 文件产出协议：本地 sandbox 文件需要如何登记为 public/download artifact，前端如何展示，移动端如何打开。
- 在 finalizer 或 Web 回复层增加文件交付校验：如果 final 文本引用本地生成文件名但没有可下载 artifact，应改为明确失败/降级提示，并提供纯文本 CSV 内容。
- 为 Web direct 增加回归：用户要求生成 `.xlsx` / `.csv` 后，最终响应必须包含可下载 artifact metadata；若不支持，则不得声称“已发送文件”。
