# Handoff: 公开聊天进度叙事 + 失败文案去内部化

- title: 公开聊天分步进度叙事与「门禁/未核验」话术治理
- status: done
- created_at: 2026-08-20
- updated_at: 2026-08-20
- owner: ecohnoch + Claude
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `crates/hone-web-api/src/state.rs`
  - `packages/app/src/pages/chat.tsx` / `public-chat.css`
  - `packages/app/src/lib/public-chat.ts`
  - `soul.md`（与 `data/runtime/soul.md` 同步）

## 背景（用户报障）

用户在 hone-claw.com/chat 问「tem为什么涨这么多」，等 3–4 分钟后得到一堆
「没取到数据 / 未核验 / 完整性检查没通过」式回复。两类问题：

1. **内部流程话术泄给用户**。Interactive 链路本身没有强制门禁（契约只在
   Scheduled/Heartbeat 生效），这些话术是模型照着 prompt 里的核验规则写进
   正文的：soul.md §7.3 曾明确要求写「本轮未核验」，发现上下文通篇核验指令；
   另有 `fail_run`/run_error 路径会把 runner 配置错误原样透出。
2. **长等待没有过程感**。进度基建（active-run steps + 前端 steps trail）
   齐全但普通消息未启用，阶段文案没有「已确认 XX」的完成态。

## 改动

### 进度叙事（类 Codex 分步过程）

- 新 stage `session.clock`：投研类 Interactive 问题在准备阶段就发出一条
  服务端时钟事实（北京/纽约时间 + 美股盘前盘后状态），来源
  `market_session_clock_fact` + `wants_market_session_clock_step`（guard）。
  native codex 与 strict fallback 两条路径都会发；重试轮（prepared_investment
  已存在）不重复发。
- 预取阶段新增完成态 `preturn.identity.done`（「已核对标的：Tempus AI（TEM）」），
  `preturn.enrichment.done` 带取证组数；`preturn.evidence` 文案含新闻。
- `routes/chat.rs` 的 `public_tool_status_text` 改为带主语（ticker/检索词）：
  「正在读取 TEM 实时行情」「正在检索：CoreWeave IPO」；detail 过滤器
  `public_progress_fragment` 扩到 90 字符并允许 `：（），·%` 等展示字符
  （ASCII `()` 与 `<>&;=` 仍被剔除）。
- 前端所有消息启用 steps trail（原来只有财报工作流有）；上限 6→8
  （`PUBLIC_CHAT_MAX_PROGRESS_STEPS`，与服务端 `ACTIVE_RUN_MAX_STEPS` 对齐）；
  已完成步骤渲染绿色 ✓，进行中步骤保留珊瑚点脉冲（public-chat.css）。

### 失败/缺数据文案

- 发现上下文新增【最终正文的语言边界】：正文禁用核验/未核验/门禁/工具名等
  流程词；缺口用一句自然话说明并继续回答；只有行情与检索全部失败才可说
  暂时查不到。soul.md §1.2/§7.3/§11 与各深度模板同步去掉「本轮未核验」的
  机械句式（定时契约轮不受影响：其 enforcement block 单独注入字面短语，
  checker 只对契约轮生效）。
- 全部种子低置信但问题在问行情/涨跌时，改注入「先确认再继续」的引导，
  不再劝退模型把小写 ticker 当缩写放弃。
- 涨跌归因日期锚点块中的「原因本轮未完全核验」改为自然表述 + 列候选原因。
- `CONTRACT_FAILURE_MESSAGE`（定时契约兜底）改为人话。
- `user_visible_error_message` 新增改写：`执行器不可用/执行器循环/
  function-calling llm/cli-acp` 类配置错误 → 通用失败文案。
  `routes/chat.rs` run_error 事件也改走该净化（原来裸截 120 字节）。

### 数据修复

- `run_pre_turn_enrichment` 实体解析接受「多行结果中恰有一行 symbol 与候选
  相同」（`unambiguous_identity_row`）：修复 “TEM” 这类 search 返回多行导致
  预取不到行情/财报的问题。

## 验证

- `cargo test -p hone-channels -p hone-web-api`：与干净树失败集逐字一致
  （全部为本机缺 Docker/PostgreSQL 的环境性失败），新增 5+ 测试全过。
- `packages/app`：`bun test`（522/522）、`tsc`、`vite build` 通过。
- 本地起栈（dev login + 无外部 key）浏览器实测：失败文案已净化为
  「抱歉，这次处理失败了。请稍后再试。」（此前泄漏「CLI/ACP…」原文）。
  进度 trail 的阶段映射与截断由单测覆盖；带真实 key 的完整 trail 需在
  配好 provider 的环境回归。

## 风险与后续

- soul.md 模板措辞变化只影响 prompt 行为，不影响 scheduled checker 的
  字面短语校验（后者由 per-turn enforcement block 驱动），已用测试确认；
  但建议下次在配好 key 的环境对「tem为什么涨这么多」跑一轮真实回归，
  确认 grok 按新语言边界收敛。
- 进度步骤的完成态是「下一条出现即上一条完成」的近似，没有失败态区分；
  若需要精确 per-step 失败展示，需要 stage 级 done/failed 协议。
