# Current Plan Index

最后更新：2026-08-04
状态：有 11 个活跃任务

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

- **Stripe + Whop 并行计费接入**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/stripe-whop-parallel-billing.md`
  - 摘要：最终业务方向是新用户默认 Stripe、Whop 保留给老用户及次级渠道；当前仍是安全验收阶段，不代表最终渠道排序。无兼容层的 provider-neutral Billing 已随精确 `5028870d` 部署，生产保持 Whop 主通道、Stripe Checkout 关闭。Stripe test endpoint `we_1U0N6iEK7h1dD4JHy3zsfiN2` 的旧 signing secret 因意外暴露已立即轮换，新值已受保护写入 `root:root 0600` 的 runtime env；线上安全事件 `evt_1U0ZKeEK7h1dD4JHB59OFEjY` 返回 `200 catalog_mismatch`，PostgreSQL inbox/权益计数前后均为 0。生产 `/plan`、`/activate`、`/me` 与 Workbench delivery 已留去敏截图；`e31c29ac` 修复同路由切换 Whop 的响应式 bug，当前改动再让激活页遵循服务端 provider/Checkout 开关，避免关闭 Stripe 时仍展示不可用表单。真实 Stripe Test Clock 生命周期已处理 13 条 provider 事件，完成失败/宽限/恢复/取消/重购与清理；缺少 `rg` 的 CI 合同和两条精确 gitleaks 历史误报也已修复。live mode/catalog/webhook 未动；仅剩当前提交 CI、真实非 owner Whop buyer 与 live promotion 决策，因此不归档

- **Public Admin Usage 数据探索与统一上线**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/public-admin-usage-exploration.md`
  - 摘要：把管理员统计扩展为统一口径的数据探索页，增加渠道分类、14/30/90 天追溯和可点击折线精确数值；补齐长周期查询容量、筛选联动、回归测试和精确 revision 的前后端统一生产更新

- **Whop 购买邮箱真实投递**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/whop-email-delivery.md`
  - 摘要：Workers Paid 已由用户确认并开通，`hone-claw.com` Email Sending 域名与 DNS 已启用，最小权限 token 已安装到本机忽略且权限为 `0600` 的 `.env`；Cloudflare 活动日志确认两次真实投递均为 `Delivered`，用户随后回传验证码确认真实收件箱收到邮件，隔离 Whop membership 的浏览器流程已迁到统一 `/activate?provider=whop` 并成功进入 `/me`。Cloudflare/Whop 环境变量清单已统一到 `.env.example`，实际值只保留在忽略的 `.env`。Whop verifier 已切换为当前原始 `ws_...` secret 格式并拒绝旧格式；精确提交 `482c34d5` 已构建为不可变包并在零活跃会话后受控部署，生产公网有效签名的无副作用事件返回 `200`、篡改正文与无签名请求返回 `401`，邮箱、PostgreSQL、R2、Web 与 Feishu 均健康。当前仅剩真实非 owner Whop buyer 的同一挑战验证码最终生产验收

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
