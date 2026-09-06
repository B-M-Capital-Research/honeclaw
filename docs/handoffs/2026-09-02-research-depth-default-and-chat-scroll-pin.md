# 投研回答默认研究深度 + 公开端聊天回答结束后的滚动回弹

- title: 投研回答默认研究深度；回答结束后上滑被拉回底部
- status: done
- created_at: 2026-09-02
- updated_at: 2026-09-02
- owner: Finn-Fengming
- related_files:
  - `crates/hone-channels/src/prompt.rs`（`DEFAULT_HARI_INVEST_POLICY` 新增「研究深度默认」）
  - `crates/hone-channels/src/investment_response_guard.rs`（终稿契约新增【研究深度：每一问都是一次研究，篇幅跟着证据走】）
  - `soul.md`（六、输出纪律 第 6 条）
  - `skills/hari-invest/references/conversation-contract.md`、`CHANGELOG.md`（0.3.1）、`references/provenance.md`
  - `skills/earnings-readout/SKILL.md`、`skills/moat/SKILL.md`、`skills/stock_research/SKILL.md`
  - `packages/app/src/pages/chat.tsx`、`packages/app/src/lib/public-chat.ts`、`packages/app/src/pages/chat.test.ts`
- related_docs: `docs/handoffs/2026-08-29-r3-r6-eval-driven-harness.md`（上一轮 harness 评测方法）

## 用户要求

> 目前回答普遍偏短，每次提问其实都是一次研究，应该更长（例：早上问「戴尔超预期吗」只拿到结论 + 三块要点）。
> 另外结果出来后往上滑动时页面又刷了一下、回到了底部，不友好。

## 一、回答偏短：根因与改法

根因不在取数。戴尔那轮工具已经把营收 / EPS 对一致预期、AI 服务器收入、backlog、盘后报价都拉回来了，
但三处文本明文允许短答：

1. `skills/hari-invest/references/conversation-contract.md`：「简单问题可以压缩成三至五段，不强制标题……不要把每次回答写成固定九段报告」。
   `hari-invest` 每个投研轮强制加载，这句就是默认篇幅上限。
2. `skills/earnings-readout/SKILL.md` 产出形态：用户直接点名时「不套九段模板……复盘给幅度一行 + 三项变量 + 一句中枢动没动，
   结尾问一句要不要接着把倍数与合理价区间算出来」。「超预期吗」正好落在这条上，于是估值被留给了下一轮。
3. `skills/moat/SKILL.md` 同款「结尾问一句要不要接着算」。

改法是把「每一问都是一次研究，篇幅由本轮证据决定而不是问句字数」写成三层一致的口径（skill 说要展开、系统提示又允许压缩的冲突以前者赢不了）：

- 系统提示层：`prompt.rs` 的 Hari Invest 默认策略 + `investment_response_guard.rs` 的终稿契约，列出终稿默认逐块展开的十块
  （结论 / 已核验事实 / 分部与驱动 / 指引与管理层表述 / 市场反应与时段 / 改写了哪个长期变量 / 估值再锚定 / Bull-Bear-Base / 催化与证伪 / 动作框架），
  无关块可合并、不得整块省略，每块写到数字与因果链；发送前对照本轮工具结果把漏写的口径补进正文；简短只留给问候、记账追问、实体澄清与产品使用。
- `soul.md` 六、输出纪律第 6 条同文。
- skill 层：`conversation-contract.md` 改成「研究深度」一节；`earnings-readout` 直接点名时改九块完整复盘 / 前瞻，估值再锚定在输入可得时直接乘完；
  `moat` 不再以问句收尾；`stock_research` 把「quote-only 可以简短」收窄为行情速查也带口径、语境与含义。

### 第二轮：补可执行的篇幅下限

用户看过第一轮后仍反馈「还是偏短，报告字数要拉长」。原因是只说「跟着证据走」没有可对照的量，
模型仍会收在一屏内。补的三条同样进 `prompt.rs` / `investment_response_guard.rs` / `soul.md` /
`conversation-contract.md` 四处：

1. **下限写成数**：公司深度、财报解读、估值、板块产业链、宏观市场、持仓复核六类终稿，
   中文正文不少于 1500 字，证据充分时通常 2500–4000 字；英文按等量信息折算。
   行情速查、关系确认、单点事实不设下限，但要带数据口径、当日语境与一句含义。
2. **字数只能由证据堆出来**：复述问题、重复结论、罗列免责声明、堆通用投资常识、交代自己做过什么，
   一律不算篇幅——凑字数比短答更差。短于下限时补的是本轮已取到却没写进正文的口径、分部、时段、
   指引、现金流与估值输入，以及还没展开的推演；补不出来就如实写缺口。
3. **禁止把内容留到下一轮**：不以「需要的话我可以继续展开」「要不要接着算」「如需详细分析请告诉我」收尾。

同时清掉两处会抵消下限的旧文本：`prompt.rs` 里 Telegram 的「输出保持简洁……避免过长段落」与飞书的
「保持简单」，改写为**排版粒度 / 结构限制，不是篇幅上限**；`skills/market_analysis/SKILL.md` 的
「篇幅要压时先砍复述与背景」前面补「压篇幅不是默认动作」。
`earnings-research` 的逐模块「150-250 字」**未动**：那是 PDF 财报报告工作流的分节配额，十节加总本身就长。

**没有加门禁**：不做「字数 < N 就拒发」这类 validator，那是 validator 驱动的循环改写；只改模型读到的默认。
终稿链路本身不截断，主模型 `max_tokens: 32768`（`config.yaml`）足够写到 4000 字以上。

## 二、滚动回弹：根因与改法

时序：`run_finished` → `pinToBottom(1400)` → `finally` 里再 `pinToBottom(1600)` 并发起 `restoreSession({ keepAtBottom: true })`。
restore 是一次网络往返，用户往往已经开始往上读；请求回来时 `shouldKeepBottom` 直接吃了发起时捕获的 `keepAtBottom=true`，
于是 `pinToBottom(1200)` 又把视口拉回底部，reconcile 重绘就是用户看到的「刷了一下」。
此外钉底窗口（1.2–1.8s）内所有 `scroll` 事件都被当成布局抖动忽略，用户在窗口内的上滑会被吞掉甚至被 `shouldRecoverPinnedBottom` 拉回。

改法（`chat.tsx` + `public-chat.ts`）：

1. `restoreSession` 去掉 `keepAtBottom` 选项，落地时用 `shouldKeepBottomAfterRestore({resetWindow, stickToBottom, distanceFromBottom})` 按**当时**的滚动状态决定；
   用户已离开底部就保留原位置（并在 reconcile 前后加 `suppressScrollUntil`，防止 DOM 收缩时浏览器 clamp 产生的假 scroll 触发加载更早历史）。
2. 用户手势直接解除钉底：滚动容器上加 `onWheel`（向上）、`onTouchStart`、`onPointerDown`（滚动条抓取），
   通过 `isLeaveBottomGesture` 判定后清掉 `pinBottomUntil` / `suppressScrollUntil`（wheel 向上同时置 `stickToBottom=false`）；
   `scrollToBottom` 在 `stickToBottom=false` 时直接返回，钉底排队的延迟跳转不再补刀。
   内容不可上滚（`scrollTop<=0`）时手势不解除，避免短对话里新回复不再跟随。

## verification

- `bun run typecheck`（packages/app）通过；`bun test`（packages/app 全量）541 pass / 0 fail，
  新增 `shouldKeepBottomAfterRestore` 与 `isLeaveBottomGesture` 两组用例。
- `cargo test -p hone-channels --lib -- prompt::tests investment_response_guard::tests`
  （`HONE_POSTGRES_DATABASE_URL` 指向 honeclaw_test）：165 passed / 0 failed；
  `turn_builder::tests` 与终稿契约测试合计 142 passed / 0 failed。新增断言覆盖 1500 字下限、
  2500–4000 字区间、「不得靠复述/免责声明凑字数」、「禁止把内容留到下一轮」，
  以及 Telegram / 飞书格式指引不再充当篇幅上限。
- `investment_response_guard::tests::interactive_agent_runtime_suffix_ends_with_time_first_answer_contract` 与
  `prompt::tests::repository_soul_keeps_full_investment_output_contract` 通过（新增断言覆盖研究深度段落）。
- **未做**：真实模型回答长度的线上评测（本机 LLM / FMP / Tavily key 为空，跑不出真实投研回答），以及带流式回答的浏览器手动复现
  （需要后端 + PG + 一次真实流式回答）。上线后按 `docs/handoffs/2026-08-29-r3-r6-eval-driven-harness.md` 的方法复测 20 题，
  重点看财报类（「X 超预期吗」）与护城河类是否不再以问句收尾、是否逐块带数字。

## risks

- 篇幅拉长后 token 成本与首字延迟都会上升；`max_tokens: 32768` 够用，但要观察平均回答 token、
  端到端耗时以及是否出现 `finish reason: length` 截断（`runtime.rs` 已有该日志）。
- 下限可能诱发凑字数。契约里已明确「凑字数比短答更差」并给了补写来源，复测时要专门看有没有出现
  重复结论、通用常识段和多余免责声明；若出现，收紧的是「只能由证据堆出来」这句，不是下调下限。
- 「无关块可合并」留给模型判断；若出现为凑块数写空话的情况，收紧的是「每块写到数字与因果链」这句，而不是回退到允许短答。
- 滚动：`onWheel` 是非 passive 监听器，处理函数不调用 `preventDefault`，不影响滚动性能；触控板惯性滚动会连发多次 wheel，
  首个向上 tick 即解除钉底，符合预期。
