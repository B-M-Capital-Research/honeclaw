# Bug: Feishu 直聊沿用旧证券上下文，用户问 `DRAM` 却被整轮答成 `SNDK`

- **发现时间**: 2026-04-17 15:08 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a`
    - `2026-04-17T14:53:35.114837+08:00` 用户真实输入：`美股DRAM详细分析`
    - `2026-04-17T14:55:21.748970+08:00` assistant 最终回复却完整展开 `SanDisk（SNDK）` 的“今日大跌原因、目标价、建仓区间”，并写出 `当前现价919.47美元`
    - 同一会话上一条与 `SNDK` 相关的真实用户消息停留在 `2026-03-25T21:16:56.772819+08:00`：`今天美股SNDK为什么大跌，建仓买入价是多少，机构评级目标价格是多少。`
  - 运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-17T06:53:35.117749Z` `recv ... input.preview="美股DRAM详细分析"`
    - `2026-04-17T06:53:35.239046Z` `stage=search.context_sanitized removed_tool_messages=8`
    - `2026-04-17T06:53:48.578714Z` `runner.stage=tool.execute ... detail=data_fetch snapshot SNDK`
    - `2026-04-17T06:53:50.057131Z` `runner.stage=tool.execute ... detail=web_search query="SanDisk SNDK stock plunge today April 17 2026 reason"`
    - `2026-04-17T06:54:11.943160Z` `runner.stage=tool.execute ... detail=web_search query="SanDisk SNDK analyst rating price target April 2026"`
    - `2026-04-17T06:54:13.771714Z` `runner.stage=tool.execute ... detail=data_fetch financials SNDK`
    - `2026-04-17T06:55:21.751132Z` `done ... success=true ... tools=4(data_fetch,web_search) reply.chars=1824`
  - 同会话历史：
    - 当前轮之前没有新的 `SNDK` 用户输入，也没有任何 `DRAM` 细分公司澄清问题
    - 说明 search 阶段不是“用户临时改问 SNDK”，而是旧证券上下文直接劫持了当前请求

## 端到端链路

1. 用户在 Feishu 直聊里发送 `美股DRAM详细分析`，这是一个板块级问题，期望得到 DRAM 行业或主要公司的总览。
2. 系统正常接收消息，并进入 `multi_agent.search`。
3. search 阶段没有围绕 `DRAM / 美光 / SK 海力士 / 三星 / 南亚科` 等关键词搜集候选信息，而是直接执行 `SNDK` 的 snapshot、财务和两条 SanDisk 定向搜索。
4. answer 阶段随后基于这些错误检索结果，产出一整篇关于 `SanDisk（SNDK）` 的个股分析，并把它当作对 `DRAM` 请求的最终答案发给用户。
5. 渠道投递、工具调用和最终回复都显示为成功，因此如果只看执行状态，这轮会被误判为“正常完成”。

## 期望效果

- 当用户提问 `美股DRAM详细分析` 这类板块级问题时，系统应优先识别为行业分析请求，而不是直接套用旧会话中的某只股票。
- 若上下文里存在多个潜在相关标的，也应先给出范围确认或至少基于 `DRAM` 关键词展开检索，而不是静默锁定到旧 ticker。
- search 与 answer 的最终主题应和当前 user turn 对齐，不能出现“当前问题是板块，整轮却回答一个月前的个股旧问题”。

## 当前实现效果

- 当前轮 user turn 明确是 `美股DRAM详细分析`，但 search 阶段从第一条工具调用开始就把主题锁死为 `SNDK`。
- 最终 assistant 文案不仅答非所问，还进一步给出了高度具体的 `SNDK` 价格、目标价与建仓位，放大了错误结论的可信度。
- 整轮日志显示 `search.done success=true`、`answer.done success=true`、`reply.send segments.sent=2/2`，说明当前链路没有把“主题漂移”识别为异常。
- 这与单纯“回答不够深入”不同，而是当前 user intent 被旧证券上下文覆盖。

## 用户影响

- 用户问的是 DRAM 行业，却收到一篇 `SanDisk（SNDK）` 个股分析，首轮答案已经明显答非所问。
- 在投资分析场景中，这会直接误导用户把个股结论当成行业判断，降低系统可信度。
- 之所以定级为 `P3`，是因为主功能链路仍然完成了接收、检索、生成和投递，没有出现消息丢失、系统崩溃、错误投递或数据破坏；问题核心是回答主题被旧上下文劫持，属于质量性缺陷而非链路中断。

## 根因判断

- 多代理 search 阶段对“当前 user turn 的主题约束”不够强，旧会话中的证券上下文可能压过了当前输入。
- `search.context_sanitized removed_tool_messages=8` 只能说明历史工具消息被裁掉，但没有阻止历史 user/assistant 语义把本轮检索目标错误收敛到 `SNDK`。
- 当前链路缺少“当前问题和首个工具目标必须语义一致”的防漂移约束，因此模型一旦沿用旧 ticker，后续检索和 answer 会继续自洽地把错误放大。
- 该问题与 `feishu_ambiguous_lite_entity_guessed_as_litecoin.md` 不同：那条是输入本身高歧义但未澄清；本条是当前输入其实并不指向 `SNDK`，却被旧会话证券上下文直接劫持。

## 下一步建议

- 排查 multi-agent search prompt / handoff，确认是否显式要求“首个检索对象必须由当前 user turn 推导，不能默认复用旧 ticker”。
- 为“板块级问题进入旧 ticker 定向检索”的场景补回归，至少锁住 `DRAM`、`光模块`、`机器人` 这类行业词不能直接退化成上一轮个股。
- 在 search 阶段增加主题一致性检查：若当前 user turn 不包含 `SNDK` 等 ticker，但首轮工具调用已经锁定旧标的，应触发重写或澄清。
