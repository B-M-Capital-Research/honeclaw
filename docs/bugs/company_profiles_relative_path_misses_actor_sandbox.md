# Bug: 深度分析链路持续访问不存在的 `company_profiles` 相对路径，导致画像记忆静默失效

- **发现时间**: 2026-04-16 19:10 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **证据来源**:
  - `data/runtime/logs/web.log`
    - `2026-04-16 18:43:58.887` `session=Actor_feishu__direct__ou_5fe1213e63da238b10e346a384843b434c` 在用户请求“深度分析 Dell”时执行 `local_list_files path="company_profiles"`，随后记录 `tool_execute_error ... 目录不存在: company_profiles`
    - `2026-04-16 13:05:45.267` `session=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 执行 `local_search_files query="RKLB Rocket Lab" path="company_profiles"`，随后记录 `tool_execute_error ... 文件不存在: company_profiles`
    - `2026-04-16 13:09:40.780` 同一会话再次执行 `local_list_files path="company_profiles"`，随后再次报 `目录不存在: company_profiles`
    - 同类报错自 `2026-04-13` 起持续出现，说明不是单次偶发目录缺失
  - `data/sessions.sqlite3`
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
4. agent 继续只依赖 `data_fetch`、`web_search` 等实时信息生成回复，用户侧收到的是“能答复但少了历史画像记忆”的降级结果。

## 期望效果

- 搜索阶段应能从当前 actor sandbox 读取 `company_profiles/<profile_id>/profile.md` 和相关 `events/*.md`。
- 当用户已沉淀长期跟踪画像时，深度分析应优先利用这部分上下文，而不是每次从零开始。
- 若画像目录确实不存在，系统至少应显式区分“无画像数据”与“路径解析错误”，避免静默把工具失败伪装成正常完成。

## 当前实现效果

- 搜索代理持续把 `company_profiles` 当作当前工作目录相对路径使用，而不是 actor sandbox 下的真实画像目录。
- `local_list_files` / `local_search_files` 在日志中明确报 `目录不存在` / `文件不存在`，但 reply 仍继续生成，导致故障只体现在质量退化上。
- 最新 `18:43` 的 Dell 会话就是这种状态：用户收到了一篇完整分析，但搜索阶段并未成功读取任何长期画像记忆。
- 由于主链路仍然成功返回，问题不会像空回复、误投递那样立刻暴露，而是以“回答不够连续、没吃到历史沉淀”的形式长期潜伏。

## 用户影响

- 这是质量类缺陷。它不会直接造成无回复、错误投递、数据破坏或调度失败，因此不影响主功能链路。
- 但对“深度分析”“结合历史跟踪继续判断”这类请求，系统会静默丢失用户长期沉淀的公司画像，回答深度和连续性明显下降。
- 因为当前仍能返回可读答复，没有阻断用户完成核心任务，所以按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 搜索工具侧仍在使用裸相对路径 `company_profiles`，没有对齐 actor sandbox 的实际目录根。
- 约束文档已经把画像路径定义为 `data/agent-sandboxes/.../company_profiles/...`，说明更可能是运行时路径拼接或工作目录假设没有与现行 sandbox 布局同步。
- 工具失败后主链路没有把“画像读取失败”上浮成可见降级信号，导致故障只能从日志里发现。

## 下一步建议

- 优先检查研究/画像相关工具在 runtime 中如何解析 `company_profiles` 根目录，确认是否仍假设旧工作目录。
- 若当前用户尚未建立画像目录，也应让工具返回“无画像数据”而不是路径不存在，避免误判为正常空结果。
- 为这类读取失败补可观测标记，至少让会话级日志能区分“没有画像”与“画像路径解析错误”。
