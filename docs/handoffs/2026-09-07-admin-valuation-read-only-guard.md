# 2026-09-07 · 管理员问「美光现在的估值怎么看」得到「数据不足」：只读守卫整批拦截的根因与修法

## 现象

2026-09-07 13:16（北京）管理员账号在 web 问「美光现在的估值怎么看」，21 秒后得到
「结论：数据不足（置信度：低）……最新可核验报价、财务报表细项与一致预期数据在本轮未成功取到」。
回答结构是完整的（对账表框架、周期 vs 去周期化的判断框架都在），但没有任何数字。

## 根因（journald + cloud_sessions 复盘）

- 第 1–2 轮正常：加载 hari-invest / company-thesis-ratings / valuation-audit 三个 skill，两次 `data_fetch search`
  把「美光」解析成 MU。
- 第 3 轮模型把 **`industry_map_edit`**（管理员专属的行业本体编辑工具，只有管理员的注册表里才有）
  和取行情 / 财报的 `data_fetch` 打在同一批里。
- `agents/function_calling/src/lib.rs` 的投研只读守卫判定「这一批不全是已知只读调用」→ **整批拦截**，
  并进入「无工具收尾」（`BLOCKED_TOOL_FINALIZATION_INSTRUCTION`）：模型只能用第 2 轮的实体证据作答，于是写了「数据不足」。
  日志：`agent-owned finance blocked unsafe tool batch before execution and will continue with a tools-disabled answer
  iteration=3 blocked_tools=industry_map_edit`。
- 48 小时内只出现这一次；非管理员账号根本没有这个工具，所以只影响 4 个管理员账号，但正是他们在验收。
- 模型为什么会去碰它：工具描述鼓励「每季财报后用 set_upstream_latest 写最近动作」，而 V5.3 注入块又强调上游最近动作；
  模型在估值回答里顺手想去改行业树。`assistant.tool_calls` 只存了执行过的调用，被拦的那批参数没有留痕。

## 修法（提交见 git log，三层）

1. **守卫从「整批拦」改成「逐条拒绝写调用」**（`lib.rs`）：
   - 已注册且分类器**明确知道是持久化写**的调用（portfolio 加自选、cron 新建、`industry_map_edit` 非 show、可执行 skill）
     在只读投研轮里逐条拒绝：返回结构化工具结果 `{"error":"read_only_research_round", …}`，写入 tool trace 与上下文，
     不通知观察者、不进注册表；**同批的只读取数照常执行**，模型继续带工具作答。
   - 只有**未注册或效果未知**的调用才保留原来的整批拦截 + 无工具收尾（`unknown_tool_in_initial_finance_batch…` 那条不变）。
   - 三个测试：portfolio 写、可执行 skill、以及复现本次事故的 `finance_batch_rejects_industry_map_edit_per_call_and_keeps_the_evidence_reads`。
2. **分类器认识这个工具**（`hone-core::tool_effect`）：`industry_map_edit` 进 `PERSISTENT_TOOL_NAMES`，
   `show` 是只读、其它 action 都是持久化写。之前它对分类器是「未知效果」，这才落进整批拦截。
3. **工具描述改口**（`industry_edit_tool.rs`）：第一句改成「只在用户明确要求查看或修改行业树时调用」，并写明投研回答轮次只读、
   写动作会被拒绝、行业逻辑已随【本轮相关行业】注入，不需要用它来读。

## 验证

- 单测：hone-core `tool_effect` 3 项、hone-agent 相关 5 项全绿；hone-agent 全量与改前失败集合一致（见提交信息）。
- 生产：二进制切换后看 journald，`rejected one persistent write call inside a read-only finance round` 取代
  `blocked unsafe tool batch`；管理员再问一次「美光现在的估值怎么看」应得到带对账表数字的正常回答。

## 顺带记下的边界

- 被拦的那批工具参数没有任何持久化留痕（prompt-audit 只存 system prompt 与 runtime_input，`assistant.tool_calls` 只存执行过的）。
  下次再遇到「模型想调什么」的问题，只能靠 WARN 行里的 `blocked_tools=` 名字。
