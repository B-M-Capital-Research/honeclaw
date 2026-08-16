# Current Plan Index

最后更新：2026-08-16
状态：有 15 个活跃任务

## 说明

- 本文件只保留满足准入标准的活跃任务索引，不再混入“最近完成”
- 每个活跃任务必须对应一份 `docs/current-plans/*.md`
- 历史完成事项统一从 `docs/archive/index.md` 查入口，再按需查看对应 `docs/handoffs/*.md` 或 `docs/archive/plans/*.md`
- 任务退出活跃态后：
  - 从本索引移除
  - 如需交接，更新或新增 `docs/handoffs/*.md`
  - 如需长期检索，补充到 `docs/archive/index.md`
  - 如已有计划页，移入 `docs/archive/plans/*.md`

## 活跃任务

- **推送/蒸馏成本整治（P0 哈希增量蒸馏 + P1 润色按事件共享）**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/company-major-push-cost-2026-08-16.md`
  - 摘要：生产实测 749 持仓/323 去重公司/61 actor、472 份画像内容零重复但仅 28 人有画像；主线蒸馏因"缺失 ticker 永远触发 6h 重试且触发即全量重蒸"每天空转 ~2000 次 grok 调用，P0 用内容哈希增量化（零语义损失）；P1 把 High 即时推送的 LLM 润色从每持有人一次收敛为每事件一次，仓位/主线个性化降级到模板追加层。P2（digest company-major 化）已设计、待 P0/P1 生产数据后决策。实施等 track-B（存储 async 化）合并后进行，避免与 `prefs.rs`/`dispatch.rs` 重写撞车

- **调度与定时任务体系化整治**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/scheduler-runtime-hardening-2026-08-15.md`
  - 摘要：生产已保持 Web-only 止血；心跳契约重试、cron 观测、齐射/并发/流式重试、cloud runtime、僵尸记录回收与 meta/部署探针已修。2026-08-16 进一步删除 cloud-sync 固定双 worker，并确认 role=all CPU 热点来自 mainline 对每个 actor 画像做 `P+1` 次全量 PG 读取/重复建连，已收敛为每 actor 一次批量读取且通过完整 Rust/PG/regression 门禁；本地提交尚未 push，仍待接入可部署分支后重新灰度 `role=all` 并做生产波次对比

- **2026-08-11 全产品压力/功能验收与上线**
  - 状态：`blocked`
  - 计划：`docs/current-plans/full-product-qa-and-release-2026-08-11.md`
  - 摘要：代码与测试数据修复已完成：后端 298 项、前端 465 项、类型检查、构建和 CI 均通过；30,400 请求主压测及 6,000 请求复测零失败。当前只被安全对话模型/FMP/搜索凭证缺失，以及本地 main 落后 23 个提交且工作树未形成可审计候选版本所阻塞，不能用假数据替代或从脏工作树上线

- **Public 推送缺口审计与移动详情弹窗修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/public-push-gap-and-mobile-detail.md`
  - 摘要：审计指定 Web 账号“英伟达每日消息”在 2026-07-25 后两周的任务执行、生成、投递与 public push 入库闭环，并修复 iPhone Safari 推送详情弹窗横向贴边和动态视口布局

- **Stripe 支付宝 / 微信单次年费通道**
  - 状态：`blocked`
  - 计划：`docs/current-plans/stripe-wallet-one-time-pass.md`
  - 摘要：双 entitlement、单次 Checkout、退款语义和测试模式钱包付款均已完成；2026-08-13 再次读取生产 Stripe API，支付宝与微信仍为 `available=false`，因此生产 `/activate` 已改为只按服务端保守声明展示当前可用方式，两个产品均只写银行卡；精确 GHCR/GCE 和外部 Chrome 截图验收通过，最终钱包可见性仍待外部审批

- **机构化公司长期覆盖与财报研究闭环**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/institutional-company-coverage.md`
  - 摘要：前五个切口已推送，覆盖结构化财报卡、actor 主线隔离、同文档去重、Grok 4.5、A/B/C 订阅、24 个 SEC 样本连续四季对账、季度材料身份和 PostgreSQL 可恢复任务。第六切口也已完成并通过全仓门禁：8 份 AMD/MSFT/QCOM/CAT 官方 transcript 两轮全量 Grok 回放均 8/8，电话会共享事实与 actor 问题/承诺对账分层，FMP 错绑 ticker 被拒绝，未来承诺只有 `fulfilled + evidence` 才能关闭；第六切口成本约 `$0.850`。后续仍需人工盲评、专业投资者 UI、真实 A 级画像、合法可持续的自动全文来源与一个完整前瞻财报季

- **Public Admin Usage 数据探索与统一上线**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/public-admin-usage-exploration.md`
  - 摘要：把管理员统计扩展为统一口径的数据探索页，增加渠道分类、14/30/90 天追溯和可点击折线精确数值；补齐长周期查询容量、筛选联动、回归测试和精确 revision 的前后端统一生产更新

- **Public Community Edge 生产分阶段上线**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/public-community-edge-production-rollout.md`
  - 摘要：私有 R2 快照已发布；全新的 `hone-public-community-edge` 已部署到精确路由并保持无 secret、无启用变量的 fail-closed `503`。实现提交 `385e35b0` / `100f5608` 已推到 `main`；自动 Pages 构建仍将 edge discovery 编译移除。精确 `100f5608` 的五个运行二进制、public bundle、skills/soul 和哈希 manifest 已准备在独立不可变目录，当前旧后端仍运行 `d58ef12b` 且新 edge-session 为 `404`。下一步只由外部服务执行受控重启，先验证 `mode=off` 的 `200 enabled=false`；共享 secret、backend `shadow/prefer`、Worker 激活和 Pages discovery 均未开始

- **跨市场 ticker 解析架构修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/ticker-resolution-architecture.md`
  - 摘要：系统按更新后的 ADR 0004 / D-2026-07-19-08 / D-2026-07-22-01 / D-2026-07-26-06 收口跨市场 ticker 与 Interactive 自然 Agent 循环。主 Agent 从完整原话识别本轮有界覆盖的点名标的，为每个接纳标的声明稳定 `entity_route` 和 call-scoped `identity_match`，普通小写/混合大小写 ticker 仍走 normalized exact-symbol；任何显式 route 缺失/非法 call-scoped match 都在 observer/registry/provider-network 前拒绝且不污染 ledger，6 路线上限从第一批 admission 即生效。实体与证据 ledger 驱动真实业务工具的 `Required → Auto`，研究最多 3 个金融工具批次、24 次总调用、20 次 DataFetch、6 次 Web，不再暴露 `finish_research`，也不执行 handoff、opaque locator 纠正、独立 terminal、终稿审计、第二次生成、固定拒答或答案回写；耗尽后同一 Agent 以 `tools=[]` 从现有证据自然收口。Web 保留原财经首行格式但撤销 T0 提前 ACK，完整回答成功后一次发布；危险/未知批次零执行并由同 Agent 做一次无工具回答，固定研究失败尾句已删除。同一上下文最多保留四条/4000 字近期用户原话用于追问指代，历史 assistant/tool/行情不会进入本轮事实链。报价源时间优先使用 `hone_quote_time.beijing`；`market_date_new_york` 不能推出“纽交所/收盘价”，交易所只能来自结构化 exchange 字段；关系强度没有当前证据时必须中性表述。umbrella 任务之后仍需处理 scheduler 800G/NAND/AST/SEC P2，因此保持 `in_progress`、不归档
  - 2026-07-22 TTFT 跟进：首轮 `b06de76a` 灰度暴露无界金融研究 fan-out，第二阶段 `820a7240` 首词已到 `182ms`，但因 provider 终稿在精确前缀前遗留换行而触发严格失败边界并立即回滚。最小修复 `2563f7ad` 只在首个非空白内容确实以 byte-exact 已 ACK 前缀开头时删除 leading Unicode whitespace；全仓门禁、精确不可变构建/manifest、零活跃会话重启和云存储/鉴权/静态资源健康检查均通过。原问题 fresh actor 最终在 `179ms` 收到精确首行，四次模型、三批、14 次实际工具（8 DataFetch/6 Web）、两条 route 后由同一 Agent `tools=[]` 自然终稿，`117.189s` 单次成功结束，无 partial/reset/error/失败尾句，8,167 字节可见内容与两行历史完全一致，active chats 回到 0。TTFT 子阶段已完成；umbrella 仅因 scheduler `800G` / `NAND` / `AST` / `SEC` P2 继续保持 `in_progress`，不归档
  - 2026-07-26 涨跌归因跟进：真实 Web/飞书样本先后暴露通用失败、把用户指定周五改答周四、日期星期错误、`change`/`changesPercentage` 混用、从普通 quote 推断“收盘/纽交所”、搜索摘要冒充同日原因，以及重复搜索造成的时延超标。最终精确 `84ca1f2114c059a157cd893c84067638c7618e84` 只允许两个不同代表组的完整 `quote`/`snapshot` 结果开放宽基证据 floor，拒绝 `quote_short`、snippet-only 原因和不匹配的百分比/交易所/close 语义，并在两组已核验行情加一次来源搜索后进入同 Agent 的有界终稿。完整仓库门禁、504 文件 immutable manifest 和替换部署均通过；无来源传言、`美股为什么大跌`、显式周五宽基、HIMS 周五四个 fresh actor 都在 `45.597–58.917s` 内唯一成功终止，无 reset/error/partial/通用失败，SSE/两行历史逐字节一致，active chats 回到 0。该子阶段已完成并记录 handoff；umbrella 只因 scheduler `800G` / `NAND` / `AST` / `SEC` P2 继续 `in_progress`、不归档。Discord token 仍被网关拒绝，Web/飞书使用同一精确 build 隔离运行
  - 2026-08-03 SNDK replacement 跟进：当前代码已隔离 malformed 已知只读调用、要求退市断言具备同代码 `inactive_listing`，并在第一次模型调用前预取身份与 snapshot。新增 SNDK `active_listing` 首模前回归；loopback FMP 测试/适配器显式绕过工作站 HTTP proxy；仓库默认、示例与 GCE effective config 的每日对话额度均已升至 100。精确 `5028870d` 已在连续零活跃会话后低影响切换，journal 显示 API 约 2 秒恢复；两轮独立真实 Web canary 均执行当前 SNDK 行情/财报取证并把公司识别为 SanDisk/闪迪，未再出现“已退市 / 未上市 / 无法提供当前财报前瞻”。该 replacement 子阶段已完成；umbrella 仍因 scheduler `800G` / `NAND` / `AST` / `SEC` P2 保持 `in_progress`

- **Active Bug Burn-down 2026-04-28**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/active-bug-burn-down-2026-04-28.md`
  - 摘要：集中清理 `docs/bugs/README.md` 活跃缺陷；2026-06-09 远端先关闭 3 条文案污染 P3，本轮继续验证并修复剩余 4 条活跃 bug，当前活跃待修复队列清空
- **Chart Visualization Skill 与多通道 PNG 投递**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/chart-visualization-skill.md`
  - 摘要：新增 `chart_visualization` skill 与 Python PNG 渲染器，扩展 `skill_tool` 结构化 artifact 契约，统一 `file:///abs/path.png` 助手可见媒体标记，并让 Web / Feishu / Telegram / Discord 在保留 text-image-text 顺序的同时正确渲染或上传本地图表
- **Feishu 直聊 placeholder 假启动与 release runner 生效链路修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/feishu-direct-placeholder-followup-fix.md`
  - 摘要：继续修复 Feishu 私聊消息只发 placeholder 不进主链路的问题，同时收口 release app 仍读取 legacy config 导致 runner 改完不立即生效，并修复 desktop UI 缺少 `codex_acp` 入口造成的 runner 观测不一致
- **Canonical Config 与 Runtime Apply 统一改造**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/canonical-config-runtime-apply.md`
  - 摘要：canonical config、effective-config、CLI 管理面、安装 / onboarding、标准 Homebrew tap 与 OpenCode 本机配置继承已落地；当前继续收口 `hone-cli onboard` 渠道回退体验、安装版 Web 静态资源打包，以及 desktop bundled 模式下的 live/component/full apply 语义
- **Skill Runtime 对齐 Claude Code**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/skill-runtime-align-claude-code.md`
  - 摘要：核心 skill runtime 已迁到“listing 披露 + 调用时完整注入 + slash/direct invoke + session 恢复”模型；本轮进一步补上 stage-aware skill 可见性、`HONE_SKILLS_DIR` 透传与 `cron_job` 可执行性对齐，确保当前会话里看得见的 skill 默认都能真正调用；hooks 真执行、watcher 热重载与更细粒度 turn enforcement 仍待 runner / infra 继续补齐
- **ACP 对齐的 Agent Runtime 全栈重构**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/acp-runtime-refactor.md`
  - 摘要：ACP runners 已接入 Hone MCP bridge；runner timeout 已收敛到顶层 `step=3 分钟 / overall=20 分钟` 两档。2026-08-01 会话所有权收敛为显式 `NativePersistent / StructuredReplay / EphemeralCompiledPrompt` 策略与对应输入类型：Codex ACP 通过 `CODEX_CONFIG.developer_instructions` 接收指令，每个 `session/prompt` 无论新建、续轮或 compact 后都只有当前北京时间与当前用户/附件内容，不再保留任何 seed/reseed、历史对话或工具结果拼装路径；OpenCode 保持 fresh-session replay。2026-08-03 已进一步收紧为“每个持久 SessionIdentity 只有一个 Codex 原生 session”：提示词指纹和重启不得自动分叉，首次 `session/new` ID 必须在首个 prompt 前检查点持久化，resume 失败继续 fail closed，真实 ACP 探针不得污染用户 Codex Desktop 任务列表。Codex ACP `1.1.7` 与 OpenCode `1.18.11` 仍采用独立版本化流式方言；不得回归 Codex current-turn-only 与 OpenCode 独立上下文契约
