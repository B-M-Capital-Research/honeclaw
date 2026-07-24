# Bug: Web direct 将 HIMZ 2X Long ETF 误判为反向空头 ETF

## 发现时间

2026-07-24 15:01 CST

## Bug Type

Business Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-24 11:00-15:01 CST。
  - `session_id=Actor_web__direct__web-user-5bb05078acd4`。
  - `2026-07-24T14:47:55.201008+08:00`，用户请求摘要：询问 HIMS 走势判断，并披露持有 HIMS 正股、HIMS 2026-12-18 38 Call、以及 `HIMS两倍ETF（HIMZ）540股`，要求结合当天 FDA 会议波动给出操作建议。
  - `2026-07-24T14:50:00.360676+08:00`，assistant final 在同一回复中先把产品名写作 `Defiance Daily Target 2X Long HIMS ETF`，但随后将 `HIMZ` 判断为 `2x 空头反向 ETF（-200% 每日收益）`，并据此建议重新审视或减仓 / 清仓。
  - 同条 `metadata_json` 显示本轮已调用 `data_fetch search/quote/profile` 查询 `HIMS` 与 `HIMZ`，以及多条 `web_search` 查询 FDA / HIMS / HIMZ 事件。
- 官方产品核对：
  - Defiance 官方页面 `https://www.defianceetfs.com/himz/` 当前描述为 `Defiance Daily Target 2X Long HIMS ETF`，目标是 HIMS 每日百分比变化的 `2x / 200%`，不是反向空头产品。
  - ETF.com / MarketWatch 等第三方页面也将 `HIMZ` 描述为 `2X Long HIMS ETF`。
- 去重检查：
  - `web_direct_price_target_direction_misread.md` 覆盖的是价格目标上 / 下方向语义 sanity check。
  - `feishu_direct_nonstandard_ticker_guess_for_trade_advice.md` 覆盖的是非标准 ticker 或近似实体未确认时直接给交易建议。
  - 本缺陷是已识别 ETF 之后，把产品的 Long / Inverse 方向判反，并把错误方向用于操作建议；根因和修复入口不同，因此单独登记。

## 端到端链路

1. Web direct 用户询问 HIMS 相关持仓操作，明确包含 `HIMZ` 这种 HIMS 杠杆 ETF。
2. runner 对 HIMS / HIMZ 执行行情、搜索和 profile 工具调用。
3. answer 阶段在最终回复中把 `HIMZ` 从 `2X Long HIMS ETF` 误判为 `2x 空头反向 ETF`。
4. 最终操作建议基于错误方向，提示用户 `HIMZ` 与 HIMS 正股逻辑冲突，并建议减仓 / 清仓。

## 期望效果

- 对杠杆 ETF、反向 ETF、期权等高风险金融产品，answer 阶段必须以本轮可核验的产品名称、基金目标和官方 / 可信来源为准。
- 当产品名称已经包含 `Long`，或资料显示目标为 `+200%`，不得在同一回复中改写为 `-200% inverse`。
- 如果无法确认 Long / Inverse 方向，应先明确不确定并要求用户确认产品页或券商持仓名称，而不是给出方向性操作建议。

## 当前实现效果

- assistant 已在正文里写出 `Defiance Daily Target 2X Long HIMS ETF`，但仍把 `HIMZ` 判断为 `2x 空头反向 ETF`。
- 该错误直接改变持仓解释：原本应是放大 HIMS 多头敞口的工具，被解释成与 HIMS 正股对冲 / 冲突的空头工具。
- 回复主体正常收口，未见空回复、错投、内部路径、raw tool JSON、provider 原始错误或 `<think>` 外泄。

## 用户影响

- 用户收到的金融操作框架在关键产品方向上完全反转，可能导致错误减仓、清仓或错误理解组合风险敞口。
- 这影响金融正确性和交易决策质量，已超出普通措辞问题；但本窗没有证据显示消息投递失败、跨用户错投、数据写入破坏、系统级不可用或敏感信息泄露。
- 因此定级为功能性 / 正确性 `P2`，不是 `P1`。

## 根因判断

- 初步判断 answer 阶段缺少杠杆 / 反向 ETF 产品方向 sanity check：工具或来源中存在 `Long` 与 `+200%` 证据，模型仍根据高波动或杠杆 ETF 的泛化知识 hallucinate 成 inverse。
- 金融产品类型识别没有在出站前校验 `fund objective`、`long/inverse`、`daily target` 与最终操作建议之间的一致性。

## 下一步建议

- 在金融系统 prompt 或出站 guard 中增加杠杆 / 反向 ETF 约束：产品方向必须从 `fund objective` / 官方名称抽取，若正文出现 `Long` 与 `inverse`、`+200%` 与 `-200%` 矛盾，必须阻断或改为澄清。
- 增加回归样本：`HIMZ` 应识别为 `Defiance Daily Target 2X Long HIMS ETF`，不得把它描述成 `2x 空头反向 ETF` 或建议用户按空头对冲逻辑处理。
- 对含 ETF / ETN / leveraged / inverse / option 的操作建议，加入一条产品方向一致性检查后再允许输出减仓、清仓、对冲、加仓等动作框架。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 2026-07-24 11:00-15:01 CST Web direct 会话；`docs/bugs/` 既有文档去重检索；Defiance 官方 HIMZ 产品页与公开 ETF 页面核对产品方向。
