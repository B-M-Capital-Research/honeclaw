# Bug: Feishu 直聊模糊 ETF 建档请求被错误写成 DRAM 持仓与任务

- 发现时间：2026-06-19 11:02 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed
- GitHub Issue：无，非 P1

## 修复记录

- 2026-06-20 03:05 CST 已修复：
  - `crates/hone-channels/src/prompt.rs` 的共享金融系统 prompt 新增“副作用写入确认约束”：当当前 user turn 只用“这只 / 这一只 / 这个 / 它 / 上一个 ETF”等模糊指代，且请求会写入持仓、成本、心跳任务、定时任务或公司画像时，必须先确认唯一标的；在确认前不得调用 `portfolio`、`cron_job`、画像等持久化写入工具，也不能先按上一轮标的代写再让用户纠正。
  - `crates/hone-channels/src/runners/multi_agent.rs` 的 search-stage guidance 同步加入 deictic ETF / 持久化写入护栏，明确禁止仅凭旧上下文 ticker 推断目标并发起 write-oriented tool call。
  - 这次修复复用既有 “non-standard ticker” 行为边界，但把约束提升到“模糊指代 + 有副作用写入”场景，覆盖持仓、心跳任务和画像写入这类会污染用户长期状态的路径。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-19 07:02-11:02 CST。
  - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`。
  - 10:06 CST 用户输入：`帮建立这一只etf的长期画像，并且建立心跳任务，我买了700股，平均价格24`。该轮没有在当前消息中明确 ticker，只承接上一轮上下文。
  - 10:08 CST assistant final 直接按 `DRAM（Roundhill Memory ETF）` 处理，并声明已把 DRAM 持仓更新为 `700 股 / 平均成本 24 美元`、建立长期画像、更新心跳任务，最后仅补一句“若你说的不是 DRAM 这只 ETF，需要立刻纠正我”。
  - 10:09 CST 用户纠正：`我说的是这只 Tuttle Capital Pure Play Photonics ETF`。
  - 10:12 CST 用户进一步确认：`确认股票代码 foto`。
  - 10:15 CST assistant final 承认“上一轮误加的 DRAM/存储链关注项”，并声明已清理误写的 DRAM 持仓、关注项和画像，改为 `FOTO` 持仓、画像和 `FOTO 光子学ETF心跳检测`。
  - 10:34 CST 用户要求关注成分公司变动，assistant 继续围绕 `FOTO` 更新心跳任务并以 `stopReason=end_turn` 收口。
- 同窗总体：
  - `data/sessions.sqlite3` 仍未追平真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`，因此本轮以 ACP 事件为真实会话证据。
  - ACP 窗口内共有 15 个 session、26 次 prompt、26 次 `stopReason=end_turn`、0 个 response error；该问题不是未回复、投递失败或 runner 中断。

## 端到端链路

1. Feishu direct 用户基于上一轮上下文，用“这一只 ETF”要求建立长期画像、心跳任务并记录 `700 股 / 均价 24`。
2. assistant 没有先确认当前指代的 ETF 标的，也没有在写入前要求用户确认 ticker。
3. assistant 直接把请求解释为 `DRAM`，并调用持仓 / 定时任务 / 本地画像链路执行写入。
4. 用户指出真实目标是 Tuttle Capital Pure Play Photonics ETF，并确认 ticker 为 `FOTO`。
5. assistant 才清理错误的 DRAM 记录并改写为 FOTO。

## 期望效果

- 对“这一只 ETF”这类依赖上下文指代的持仓写入请求，若当前轮没有明确 ticker，且会产生持仓、画像或定时任务副作用，应先短句确认目标标的。
- 即使模型高概率猜到一个 ETF，也不能在未确认前执行 `portfolio`、`cron_job` 或画像写入。
- 若需要承接上一轮上下文，最终写入前必须明确复述并等待用户确认，例如“你指的是 FOTO 吗？确认后我再记录 700 股、均价 24。”

## 当前实现效果

- assistant 在未确认标的的情况下直接执行了 DRAM 相关持仓、画像和心跳任务更新。
- 后续用户及时纠正后，assistant 清理并改成 FOTO，因此当前证据显示最终状态已被同会话修正。
- 但如果用户没有立即纠正，系统会保留错误持仓、错误长期画像和错误心跳任务，后续投研、提醒和组合分析都会继承错误上下文。

## 用户影响

- 这是功能性 bug：问题不只是回答质量差，而是错误执行了会改变用户投资上下文的数据写入。
- 影响范围当前集中在单个 Feishu direct 会话，且同会话已被用户纠正并清理，没有证据显示跨用户、错投、批量数据破坏或不可恢复损坏。
- 因此定级为 `P2`：它会导致用户无法信任持仓 / 心跳任务写入链路，但本轮不是大面积不可用或安全事故，不定为 `P1`。

## 根因判断

- 直接根因是持仓写入 / cron 写入前缺少“模糊指代 + 有副作用操作”的强制确认门槛。
- 既有 `feishu_direct_nonstandard_ticker_guess_for_trade_advice.md` 覆盖的是非标准 ticker 被猜成相近实体后输出交易建议；本缺陷是未确认标的前已经执行持仓、任务和画像写入，受影响链路与风险等级不同，不能合并。
- 当前 prompt 里虽然有“模糊指令不猜”和“持仓更新最少需要标的、动作、股数、成交均价”，但模型在承接上下文时仍把上一轮推断当作足够确定的标的。

## 后续观察点

- 当前代码级修复已把共享 prompt / search-stage 边界补齐，但还没有新增工具层硬拒绝；后续若运行态仍出现“模糊指代先写入再纠正”的样本，可再把约束下沉到工具调用前校验。
- 后续巡检重点观察 Feishu / Web direct 中“这只 ETF / 这个 / 它 + 建档 / 建心跳 / 买了 N 股 / 均价 X”是否还能越过确认门槛。

## 验证

- `cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture`
- `cargo test -p hone-channels search_input_guidance_allows_direct_replies_for_greetings --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
