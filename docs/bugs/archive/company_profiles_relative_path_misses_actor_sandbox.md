# Bug: 深度分析链路持续访问不存在的 `company_profiles` 相对路径，导致画像记忆静默失效

- **发现时间**: 2026-04-16 19:10 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **证据来源**:
  - 2026-04-24 16:46-16:49 最新真实回归样本：
    - `data/runtime/logs/acp-events.log`
      - `2026-04-24T08:47:02.018772+00:00`，`session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`，工具成功写入 `/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21/company_profiles/soitec/profile.md`
      - `2026-04-24T08:48:13.401516+00:00` 至 `2026-04-24T08:48:13.475069+00:00`，同一 actor sandbox 下再次成功写入 `company_profiles/alphabet/profile.md` 与 `company_profiles/alphabet/events/2026-04-24-initial-thesis.md`
      - 同一小时检索 `acp-events.log` / `sidecar.log` 未再出现 `目录不存在: company_profiles`、`文件不存在: company_profiles` 或把缺目录解释给用户的样本
    - `data/sessions.sqlite3`
      - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
      - 用户消息：`2026-04-24 16:46:58 CST`，`"请详细分析下谷歌"`
      - assistant 最终在 `2026-04-24 16:49:12 CST` 返回 4954 字完整分析，期间运行日志已证明画像成功落到 actor sandbox；说明这条链路不再是“先报 company_profiles 路径错，再静默降级”
  - 2026-04-21 21:00 修复后回归样本：
    - `data/runtime/logs/acp-events.log`
      - `2026-04-21T13:00:04.165140+00:00`，`session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`，工具返回 `工具执行错误: 目录不存在: company_profiles`
      - `2026-04-21T13:00:04.172741+00:00` 同一轮 assistant chunk 对用户可见地解释：`本地没有现成的 company_profiles/ 目录，我直接按实时研究走...`
    - `data/sessions.sqlite3`
      - 同一会话上一轮用户真实请求为 `2026-04-20T16:56:38+08:00` 的 `亚马逊能否买入`，assistant 最终在 `2026-04-20T16:58:00+08:00` 完成 AMZN 分析；到 `2026-04-21T21:00` 的后续盘前任务仍沿用该会话并再次暴露 `company_profiles` 目录缺失。
    - 这说明 2026-04-20 记录的 `ensure_actor_sandbox` 预建目录修复没有覆盖当前生产 ACP 事件路径，至少老会话或当前 runner 工作目录仍可能看不到 `company_profiles/`。
  - `data/runtime/logs/web.log`
    - `2026-04-19 12:22:57.347` `session=Actor_feishu__direct__ou_5ff0946a82698f7d16d9a5684696c84185` 在用户消息“我想系统研究一家公司，比如分析一下GOOGL...”的搜索阶段先执行 `discover_skills query="company profile portrait save write GOOGL"`，随后 `12:22:59.247` 调用 `skill_tool company_portrait`
    - `2026-04-19 12:23:05.811` 同一会话继续执行 `local_read_file path="company_profiles/GOOGL/profile.md"`，紧接着记录 `tool_execute_error ... 文件不存在: company_profiles/GOOGL/profile.md`
    - `2026-04-19 12:23:13.632` 同轮又执行 `local_search_files query="company_profiles" path="."`，随后记录 `tool_execute_error ... IO 错误: stream did not contain valid UTF-8`
    - 这说明最新小时窗里，链路已经不只是“泛搜 company_profiles 目录不存在”，而是明确尝试读取具体画像文件 `company_profiles/GOOGL/profile.md`，仍然沿用错误的相对路径假设
    - `2026-04-18 14:46:45.468` `session=Actor_feishu__direct__ou_5f3fd89de56543549db707217b4e1952bf` 在用户消息 `rklb，tem分析下` 的搜索阶段连续两次调用 `local_search_files ... path="company_profiles"`，随后连续记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-18 14:49:10.714` 同一会话 `multi_agent.answer.done success=true tool_calls=0`，说明画像路径错误在最新一小时仍未阻断主链路，但继续以静默降级形态存在
    - `2026-04-18 12:16:44.558` `session=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052` 在用户消息 `预测联合健康财报` 的 compact 重试阶段再次调用 `local_search_files query="UnitedHealth UNH" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 23:54:44.989` `session=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052` 在用户消息 `开启新的话题：请预测联合健康的财报` 中调用 `local_search_files query="UnitedHealth UNH" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 21:01:05.261` `session=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 在定时任务 `OWALERT_PreMarket` 执行过程中调用 `local_list_files path="company_profiles"`，随后记录 `tool_execute_error ... 目录不存在: company_profiles`
    - `2026-04-17 21:01:13.821` `session=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21` 在用户消息 `请对 FORM 进行下详细分析` 中调用 `local_search_files query="FormFactor FORM" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 17:00:29.381` `session=Actor_feishu__direct__ou_5f54788f6258d2bce10d70fc267161accb` 在用户追问 `分析AAOI` 时执行 `local_search_files query="AAOI Applied Optoelectronics" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 17:01:31.207` 同一会话在 `context_overflow_recovery` 后再次执行 `local_list_files path="company_profiles"`，随后记录 `tool_execute_error ... 目录不存在: company_profiles`
    - `2026-04-17 10:46:35.585` `session=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21` 在用户再次追问 `ciena 是否值得买入` 时执行 `local_search_files query="CIEN Ciena AI 光网络 DSP WaveLogic"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 10:46:35.747` 同一会话紧接着记录 `tool_execute_error ... IO 错误: stream did not contain valid UTF-8`
    - `2026-04-17 10:47:52.336` 同一会话再次记录 `local_search_files ... IO 错误: stream did not contain valid UTF-8`
    - `2026-04-17 10:24:40.824` `session=Actor_feishu__direct__ou_5fcd8d8940cb280ac50df027d46bd9f56c` 在用户请求“微软分析”时执行 `local_search_files query="MSFT 微软 Azure"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-17 10:28:28.748` 同一会话继续执行 `local_list_files path="company_profiles"`，随后再次报 `目录不存在: company_profiles`
    - `2026-04-16 18:43:58.887` `session=Actor_feishu__direct__ou_5fe1213e63da238b10e346a384843b434c` 在用户请求“深度分析 Dell”时执行 `local_list_files path="company_profiles"`，随后记录 `tool_execute_error ... 目录不存在: company_profiles`
    - `2026-04-16 13:05:45.267` `session=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 执行 `local_search_files query="RKLB Rocket Lab" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-16 13:09:40.780` 同一会话再次执行 `local_list_files path="company_profiles"`，随后再次报 `目录不存在: company_profiles`
    - 同类报错自 `2026-04-13` 起持续出现，说明不是单次偶发目录缺失
  - `data/sessions.sqlite3`
    - `session_id=Actor_feishu__direct__ou_5ff0946a82698f7d16d9a5684696c84185`
    - 用户消息：`2026-04-19 12:21:47 CST`，`"我想系统研究一家公司，比如分析一下GOOGL，按基本面、护城河、估值、风险逐层拆解，长期结论自动沉淀为画像"`
    - `2026-04-19 12:23:13 CST` 同轮 assistant 最终只返回 `已达最大迭代次数 8`；结合运行日志可见，失败前已经显式尝试读取 `company_profiles/GOOGL/profile.md`
    - 说明画像路径错误不再只是“主链路成功但记忆静默缺失”的质量退化，在最新深度研究样本里已与 search 触顶故障叠加，放大整轮失败概率
    - `session_id=Actor_feishu__direct__ou_5f3fd89de56543549db707217b4e1952bf`
    - 用户消息：`2026-04-18 14:46:22 CST`，`"rklb，tem分析下"`
    - `2026-04-18 14:49:10 CST` assistant 仍返回完整长文分析，但运行日志已确认同轮先连续两次命中 `company_profiles` 不存在，说明最新一小时的真实直聊仍在“先丢失画像，再继续答复”的静默降级路径上
    - `session_id=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052`
    - 用户消息：`2026-04-18 12:15:59 CST`，`"预测联合健康财报"`
    - `2026-04-18 12:16:35 CST` 同轮再次触发 `context_overflow_recovery` 写入 compact summary，`2026-04-18 12:16:58 CST` assistant 仍只返回“当前会话上下文过长”，说明 `company_profiles` 路径错误仍在最新一小时参与放大 `UNH` 新话题的重试降级
    - 用户消息：`2026-04-17 23:54:40 CST`，`"开启新的话题：请预测联合健康的财报"`
    - `2026-04-17 23:55:10 CST` 同轮已触发 `context_overflow_recovery` 写入 compact summary，`2026-04-17 23:55:32 CST` assistant 仍只返回“当前会话上下文过长”，说明画像路径错误至少参与了这轮新话题切换时的长耗时降级
    - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
    - 用户消息：`2026-04-17 21:01:06 CST`，`"请对 FORM 进行下详细分析"`
    - 到本轮巡检结束时该会话最新落库仍只有 user turn，尚未看到 assistant 新回复；日志已确认搜索阶段再次命中 `company_profiles` 路径错误
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - 用户消息：`2026-04-17 21:00:59 CST`，`"[定时任务触发] 任务名称：OWALERT_PreMarket..."`
    - 同轮运行日志已确认在定时任务搜索阶段再次命中 `company_profiles` 目录不存在
    - `session_id=Actor_feishu__direct__ou_5f54788f6258d2bce10d70fc267161accb`
    - 用户消息：`2026-04-17 17:00:14 CST`，`"分析AAOI"`
    - `2026-04-17 17:01:22 CST` 同一会话已被强制 compact 并重试，但直到本轮巡检时仍只有用户消息与 compact summary，说明画像路径错误至少参与了这轮长耗时重试
    - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
    - 用户消息：`2026-04-17 10:46:17 CST`，`"ciena 是否值得买入"`
    - assistant 最终仍返回长文分析：`2026-04-17 10:48:22 CST`
    - `session_id=Actor_feishu__direct__ou_5fcd8d8940cb280ac50df027d46bd9f56c`
    - 用户消息：`2026-04-17 10:24:22 CST`，`"微软分析"`
    - assistant 最终仍返回长文分析：`2026-04-17 10:25:46 CST`
    - `session_id=Actor_feishu__direct__ou_5fe1213e63da238b10e346a384843b434c`
    - 用户消息：`2026-04-16 18:43:50 CST`，`"深度分析 Dell"`
    - assistant 最终仍返回长文分析：`2026-04-16 18:45:53 CST`
  - 文档约束：
    - `docs/invariants.md` 明确公司画像应位于 actor sandbox 下的 `company_profiles/`
    - `docs/repo-map.md` 说明画像真相源路径为 `data/agent-sandboxes/<channel>/<scope__user>/company_profiles/<profile_id>/profile.md`
  - 实际文件布局：
    - 本地 `find data/agent-sandboxes -type d -name company_profiles` 未找到任何现成目录，说明工具当前并未在 actor sandbox 内解析到正确画像路径

## 端到端链路

1. 用户在 Feishu 直聊中发起“深度分析 Dell”等研究型请求，预期系统先读取该用户长期沉淀的公司画像与历史事件，再结合实时数据完成更连续的分析。
2. 搜索阶段会先尝试执行 `local_list_files` 或 `local_search_files`，目标路径写成相对路径 `company_profiles`。
3. 当前运行目录下并不存在这个相对路径，工具立即报错，但主链路不会失败。
4. agent 继续只依赖 `data_fetch`、`web_search` 等实时信息生成回复，或者在长链路中继续带着缺失画像的上下文运行，用户侧收到的是“能答复但少了历史画像记忆”的降级结果。

## 期望效果

- 搜索阶段应能从当前 actor sandbox 读取 `company_profiles/<profile_id>/profile.md` 和相关 `events/*.md`。
- 当用户已沉淀长期跟踪画像时，深度分析应优先利用这部分上下文，而不是每次从零开始。
- 若画像目录确实不存在，系统至少应显式区分“无画像数据”与“路径解析错误”，避免静默把工具失败伪装成正常完成。

## 当前实现效果

- 2026-04-24 16:46-16:49 的真实 `GOOGL` 会话里，agent 已连续成功写入 actor sandbox 下的 `company_profiles/alphabet/profile.md` 与 `events/2026-04-24-initial-thesis.md`，同一 actor 稍早也成功写入 `company_profiles/soitec/profile.md`。
- 最近一小时的 `acp-events.log` / `sidecar.log` 未再出现 `目录不存在: company_profiles`、`文件不存在: company_profiles` 或把缺目录解释给用户的可见文本；说明当前生产路径已能对齐到 `data/agent-sandboxes/<channel>/<scope__user>/company_profiles/...`。
- 历史上这条缺陷确实长期存在，并且曾放大深度研究、compact 重试与定时任务链路的质量退化；但按本轮真实回归样本看，当前坏态已不再活跃。

## 用户影响

- 这是质量类缺陷。它不会直接造成无回复、错误投递、数据破坏或调度失败，因此不影响主功能链路。
- 但对“深度分析”“结合历史跟踪继续判断”这类请求，系统会静默丢失用户长期沉淀的公司画像，回答深度和连续性明显下降。
- 因为当前仍能返回可读答复，没有阻断用户完成核心任务，所以按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 搜索工具侧仍在使用裸相对路径 `company_profiles`，没有对齐 actor sandbox 的实际目录根。
- 约束文档已经把画像路径定义为 `data/agent-sandboxes/.../company_profiles/...`，说明更可能是运行时路径拼接或工作目录假设没有与现行 sandbox 布局同步。
- 工具失败后主链路没有把“画像读取失败”上浮成可见降级信号，导致故障只能从日志里发现。

## 修复情况（2026-04-20）

根因确认：`ensure_actor_sandbox` 只创建 `uploads/` 和 `runtime/`，不创建 `company_profiles/`。当 runner 第一次对某用户初始化 sandbox 时，`local_list_files path="company_profiles"` 返回"目录不存在"而非空列表，导致模型把它当工具错误反复重试，最终耗尽迭代次数。

修复：`crates/hone-channels/src/sandbox.rs` — `ensure_actor_sandbox` 增加 `fs::create_dir_all(root.join("company_profiles"))`，确保 sandbox 初始化时预建空目录，工具返回空列表而非路径错误。`cargo test -p hone-channels` 全部通过。

## 回归情况（2026-04-24）

2026-04-24 16:46-16:49 的真实 Feishu 直聊 `请详细分析下谷歌` 已连续成功写入 actor sandbox 下的 `company_profiles/alphabet/*`，同一 actor 稍早还成功写入 `company_profiles/soitec/profile.md`。本轮同时检索最近一小时 `acp-events.log` / `sidecar.log`，未再找到 `company_profiles` 相对路径不存在的错误样本。基于最新真实会话证据，这条缺陷状态更新为 `Fixed`。

## 回归情况（2026-04-21）

2026-04-21 21:00 的真实 ACP 事件再次出现 `目录不存在: company_profiles`，并且 assistant 对用户暴露“本地没有现成的 company_profiles/ 目录”。这证明此前修复最多覆盖了部分 sandbox 初始化路径，尚未保证当前生产 runner 的实际工作目录、旧会话恢复路径或 ACP 工具路径都能稳定看到预建目录；本缺陷状态从 `Fixed` 重新打开为 `New`。

## 下一步建议

- 优先检查研究/画像相关工具在 runtime 中如何解析 `company_profiles` 根目录，确认是否仍假设旧工作目录。
- 若当前用户尚未建立画像目录，也应让工具返回“无画像数据”而不是路径不存在，避免误判为正常空结果。
- 为这类读取失败补可观测标记，至少让会话级日志能区分“没有画像”与“画像路径解析错误”。
