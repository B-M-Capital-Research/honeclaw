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

## 生产部署记录（2026-08-20 17:3x–18:1x 北京时间）

- 提交 `69933303a9bac83724bc574f4f74fed9abbd56f4` 推送 main；Runtime Image
  run #107 构建成功，registry 权威 digest
  `sha256:5d0e5da94f5c5a132f93fd152b47cbc9f3904d5d79954cc2132c1d5fe90b2304`
  （workflow buildx 输出的 `4c2fd73e…` 与 crane 对 tag 的两次读数不一致，
  以 crane/registry 为准；bundle 内嵌 revision 校验兜底）。
- 主机（gcloud ssh）：磁盘 5.8GiB ≥ 2GiB → 本地同 revision 脚本上传并
  SHA-256 比对（`d3633683…`）→ `stage_ghcr_runtime.sh` 按 digest 暂存
  `[PASS]` → 两次 active-chat-runs `{"count":0}` → 临时符号链接 + `mv -Tf`
  原子切换 → `systemctl restart hone-web` → `/api/meta` 精确
  `69933303…`、`source=ghcr_linux_oci`、cloud_mode=cloud、PG/OSS ok、
  `local_durable=0`、role=all；重启后 2 分钟内 journal 零 panic/ERROR。
- 回滚保留：`253421df…`（上一版）。`hone-channel@feishu` 为 disabled 且本
  boot 从未运行（运营侧既有状态，非本次回归）。
- 生产 E2E（用户已登录会话，管理员 codex 路线）：同一问题
  「tem为什么涨这么多」部署前实录为「原因本轮未完全核验」拒答；部署后
  首行数据时间保留，结论直接归因 Moderna/Merck INTerpath-001 III 期达标
  → 验证 Personalis（Tempus 15 亿美元收购标的）→ 叠加 Q2 首次 GAAP 盈利
  与指引上调，含估值参考、风险与证伪、时段状态；全文零核验/门禁类流程词。
  遗留观察：首行中「昨日常规收盘 $61.17（+9.35%）」的百分比与同行
  +24.09% 口径不一致，疑似模型引用了错误的涨跌基准字段，待后续跟踪。
- **前端 Pages 实际部署成功（此前一度误判为故障，已用面板证据更正）**：
  `69933303` 的 Pages 构建 09:39Z 完成（真实 50s 构建，配置为
  `bun install --frozen-lockfile && bun run build:web:public` →
  `/packages/app/dist-public`），生产别名 hone-claw.com 指向 tip 部署。
  误判原因与教训：GitHub check-run 的 started==completed 只是 Cloudflare
  在构建结束时一次性上报的习惯，不代表构建被跳过；跨环境比较入口
  `index-*.js` hash 也不可靠（Pages 的 bun/环境变量与本地不同，
  同源码 hash 可不同）。**验证前端是否上线的正确方法**：直接抓生产
  懒加载分块 grep 本次改动的字面标记（本次用 `chat-CQWd35xn.js` 含
  `is-done` / `pub-assistant-turn-steps` 证实），或登录面板看部署详情
  的构建时长与别名。
- 生产 UI 复验（管理员 codex 路线）：「tsla盘前现在什么价，为什么」正确
  报出盘前 $344.50 与「距盘中开始约 52 分钟」的服务端时钟事实，分时段
  给出盘后/盘前涨跌，归因分主次（费半急杀为主、Moderna 资金轮动为次）。
  管理员轮因 codex 线程复用常在数秒内完成，进度轨迹一闪而过；轨迹的
  完整可见性主要服务公开用户的 strict 链路（预取+工具循环耗时更长）。
  遗留观察（模型服从度余量，可后续拧 prompt）：正文出现过一次
  「根据 extended_hours 核验的盘前数据」（工具名+核验字样）；TEM 首行
  「+9.35%」与 +24.09% 基准口径不一致。

## 追加：最终回答恢复流式输出（d9620309，2026-08-20 晚）

- 背景：committed terminal streaming 在 `75ca1957`（治理「不可逆表头+拒答
  回退」）时连带停用（execution 层写死 Disabled），此后 Web 最终回答一直
  整块缓冲到 Done。
- 方案（只保留避开当年隐患的形态）：
  - AgentSession 对 web 交互金融轮 opt-in `CanonicalInvestmentHeader`；
    其它渠道与 execution 默认保持 Disabled。
  - `commit_before_model` 维持 false，新增 `allow_pre_final_prefix_commit`
    把 agent 取证后的「中途预提交」也收进该授权：现在**只有模型真实终稿
    字节能触发提交**。终稿开始前的任何失败仍是干净错误卡（不会留孤表头，
    也不会与进度轨迹抢屏）。
  - `canonical_prefix_delta` 放行表头行的终止换行——终稿一开始流出，
    观察者即可校验并提交规范表头（首行立即可见）；DirectFinal 正文因
    market-move 复检可能重写而保持延迟（Done 时尾段一次补发，字节与持
    久化一致）；terminal synthesis 终稿逐行流式。
  - 管理员 codex 原生轮不经过该机制（本来就是秒级整段）。
- 测试：`web_interactive_finance_turn_opts_into_committed_terminal_streaming`
  钉住 opt-in + 不预提交；预提交契约测试改为显式授权；发布形状断言更新
  为「表头 + 尾段」两段；零覆盖澄清（不满足证据地板）保持单块。本地
  honeclaw_test PG 下 agent 151/151、agent_session 154/154、web-api
  312/312；全量并行套件失败集与干净树同级（既有环境竞态，逐模块隔离
  运行全绿）。
- 生产验证口径：策略只作用于非管理员 strict 链路，管理员账号无法直接
  观察；部署验收看 `/api/meta` sha + journal 零 panic，行为证据看下一
  条真实公开用户金融问题的 `hone_agent::ttft` 日志
  （`first_committed_prefix_ms` 出现即表头已实时提交）。若模型终稿未
  逐字复现要求首行，流式自动退化为整块缓冲（今日行为），无损。

## 风险与后续

- soul.md 模板措辞变化只影响 prompt 行为，不影响 scheduled checker 的
  字面短语校验（后者由 per-turn enforcement block 驱动），已用测试确认；
  但建议下次在配好 key 的环境对「tem为什么涨这么多」跑一轮真实回归，
  确认 grok 按新语言边界收敛。
- 进度步骤的完成态是「下一条出现即上一条完成」的近似，没有失败态区分；
  若需要精确 per-step 失败展示，需要 stage 级 done/failed 协议。
