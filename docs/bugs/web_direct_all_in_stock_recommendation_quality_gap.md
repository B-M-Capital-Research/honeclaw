# Bug: Web direct 在 all-in 高潜力单票请求中给出具体股票排序

## 发现时间

- 2026-06-08 03:02 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- Fixed

## GitHub Issue

- 无；本缺陷不是 P1，不创建 GitHub issue。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 2026-06-08 03:01 CST，Web direct session `Actor_web__direct__web-user-be13e1f84d14` 用户明确表达小资金量、想 all in 高潜力个股、可等几年、目标 8 到 10 倍，并追问是否现在买以及建议什么股票。
  - 该轮 ACP response 在 03:02:48 CST 以 `stopReason=end_turn` 正常收口，未见 stream disconnect、runner error 或工具失败。
  - assistant final 开头提示“不建议现在直接 all in”，并包含“以下仅供分析参考，不要未经自己思考和风险评估就直接照做”，但随后仍输出候选排序：`TEM`、`CRCL`、`RDDT`、`RKLB`，并在“如果你坚持只买一只”段落按“更激进 / 更均衡 / 更平台型 / 更远期梦想”给出对应股票。
  - 该轮还给出“先买计划仓位的 30% 到 50%”的执行节奏，属于具体仓位动作建议。
  - 2026-06-08 07:01 CST，同一 Web direct session 用户要求把“抄底能力”和“高风险长期进攻账户”结合、追求更快翻倍且不想分仓太多。
  - 该轮 ACP response 在 07:01:53 CST 以 `stopReason=end_turn` 正常收口，未见 stream disconnect、runner error 或工具失败。
  - assistant final 有风险提示，并把策略降温为“长期研究池 + 低频重仓抄底 + 分段止盈”，没有直接回答“现在买哪只”；但仍给出 `主攻仓 70% 到 80%`、`备选仓 10% 到 20%`、`机动仓 10% 到 20%`，以及“第一笔 40%，确认止跌后 30%，基本面验证后 30%”等可照抄的集中仓位策略。
- `data/sessions.sqlite3`
  - 同窗本地 SQLite 只新增 1 个 Feishu user turn 与 1 个 assistant final；该 Web direct 会话未进入本地 SQLite 镜像，证据以 ACP 日志为准。
  - Feishu final 污染扫描未命中空回复、内部路径、raw tool 字段、思维痕迹或 provider 原始错误。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 同窗存在 event-engine FMP price/news poller 既有 P2 持续失败，但该 Web direct 回复已使用工具调用并正常 `end_turn`，未观察到 FMP 原始错误外泄。

## 端到端链路

1. Web direct 用户提出高风险、强操作性的集中买入请求。
2. Agent runner 接收完整历史和当前问题，执行若干工具调用后流式返回。
3. 最终回复虽然包含风险提示和不建议一次性 all in 的降温句，但仍给出具体股票排序、单票选择映射和分批买入比例。
4. 后续同一会话继续围绕高风险集中账户追求更快翻倍，回复进一步强调错杀、证伪和分段止盈，但仍输出较具体的集中仓位比例。
5. Web direct 正常收口并向用户展示该建议。

## 期望效果

- 对 all-in、单票、8 到 10 倍目标、是否现在买、建议什么股票这类高风险金融请求，应先把回答收敛到风险教育、候选研究框架、筛选条件、证伪条件和仓位纪律。
- 如果必须讨论候选标的，应明确为“研究清单 / 观察池”，避免用“如果坚持只买一只”这类口径把具体股票映射成可直接执行的买入选择。
- 仓位表达应避免给出可照抄的具体买入比例，除非明确是在抽象风险框架中说明示例，且不指向某个单票。

## 当前实现效果

- 回复先降温，但随后仍满足了用户“真要买的话建议什么股票”的直接请求，输出了明确的 4 个 ticker 排序和单票选择映射。
- 回复还给出“先买计划仓位的 30% 到 50%”的操作节奏，容易被用户理解为可执行买入计划。
- 后续 07:01 CST 样本没有直接给出“现在买哪只”的单票映射，质量较 03:01 CST 样本有所改善；但仍输出可直接套用的集中仓位比例和分批比例，说明 all-in / 高风险账户场景的仓位边界仍不够稳。
- 未发现投递失败、会话中断、内部错误外泄或跨用户数据问题。

## 用户影响

- 用户在明确表达 all-in 冲动时，系统没有充分把回答限制在研究框架和风险约束内，而是给出具体候选和仓位动作，可能放大集中投资与高波动标的风险。
- 该问题不影响 Web direct 主功能链路：会话正常收口，工具调用结果被消费，未出现格式损坏、错误投递或数据破坏。
- 因此本缺陷定级为质量性 P3，而不是 P1/P2；它需要改进高风险金融建议边界，但不是系统不可用或消息投递/数据正确性故障。

## 根因判断

- 当前金融回答约束虽然要求提醒风险、避免直接荐股，但在“用户强要求具体股票”场景下，模型仍把风险提示视为足够保护，然后继续给出可执行候选排序。
- 缺少针对 all-in / 单票 / 目标倍数 / 现在买吗 / 推荐什么股票组合语义的更强响应模板或出站 guard。
- 这与既有 `feishu_direct_nonstandard_ticker_guess_for_trade_advice.md` 不同：本轮不是实体猜测错误，而是在实体明确、工具链正常时，对高风险操作请求给出过于具体的荐股与仓位动作。

## 下一步建议

- 在金融系统 prompt 或出站 guard 中增加 all-in / 单票 / 目标倍数请求的专门约束：只给研究框架、候选池条件和证伪清单，不输出“如果只买一只就选 X”。
- 对“建议什么股票 / 现在买吗 / all in / 仓位比例”组合语义增加回归样本，确保回复不生成可直接照抄的 ticker 排序和买入比例。
- 若产品仍希望支持候选池输出，应统一表述为“待研究候选，不构成买入建议”，并避免按用户执行偏好做单票映射。

## 修复记录

- 2026-06-09 00:12 CST 进入 `Fixing`：`DEFAULT_FINANCE_DOMAIN_POLICY` 已补充 all-in / 满仓 / 单票 / 高风险进攻 / 快速翻倍 / 不想分仓场景的强约束，禁止输出可照抄的单票排序、唯一主攻标的映射、`70%-80%` 这类集中仓位模板，要求先降温并收敛到风险暴露上限、分散原则、触发条件和证伪条件；`build_prompt_bundle_always_includes_finance_domain_policy` 已补断言。
- 验证阻塞：本机 Rust toolchain 当前 `cargo` / `rustc` 均悬挂，本轮仅完成 `git diff --check`，不能标记 `Fixed`。下一轮需运行 `cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture` 与 `cargo check -p hone-channels --tests`。
- 2026-06-09 04:43 CST 状态更新为 `Fixed`：`cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture` 与 `cargo check -p hone-channels --tests` 通过。
