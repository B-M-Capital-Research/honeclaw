# 财报前瞻失败与搜索资料缺失诊断

- title: 财报前瞻失败与搜索资料缺失诊断
- status: done / deployed（供应商及技术交付已恢复；研究质量和遗留 MCP 风险见下）
- created_at: 2026-10-20
- updated_at: 2026-10-21
- owner: Codex
- related_files:
  - crates/hone-channels/src/core/bot_core.rs
  - crates/hone-channels/src/earnings_materials.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-tools/src/web_search.rs
  - skills/earnings-research/SKILL.md
  - scripts/diagnose_fmp_tavily.sh
- related_docs:
  - docs/archive/plans/earnings-search-failure-20261020.md
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-09-13-earnings-workflow-refactor.md
  - docs/handoffs/2026-10-11-report-reliability-and-valuation-prompts.md
- related_prs: https://github.com/B-M-Capital-Research/honeclaw/pull/59

## Summary

此次调查已区分三个供应商故障：财报专用 OpenRouter 账户余额不足；主搜索 Gemini 请求返回 404；回退 Tavily 达到套餐额度上限。它们分别解释近期整条前瞻失败与此前 PDF 成功但缺少最新网页资料。2026-10-21 已按用户授权上线现有 Gemini 专用路由和新 Tavily 凭证，修复代理多轮工具协议；隔离 WDC 前瞻和线上配置/服务/搜索验收完成。

以下任务时间均为北京时间。接下来 What Changed、Verification、Risks / Follow-ups 与 Next Entry Point 保留 10 月 20 日诊断阶段快照；当时未修改生产或重放用户任务。10 月 21 日用户授权的恢复结果见后续章节。

## What Changed

- `git fetch` 和 `git pull --ff-only origin main` 完成，远端无新提交。保留本地 `8cc040df` 公司分析 V4 提交及所有无关未提交文件；生产仍为 `1ca6197c`。
- 只更新排查计划、handoff、历史索引与部署 runbook 的供应商诊断步骤，没有修改应用代码。此次根因已有真实供应商响应证明，无需以修改 PDF renderer、内容校验或 Prompt 来代替恢复供应商服务。
- 实际服务的 canonical/effective config 均使用管理主机上的 loopback 搜索代理。主机上另一份旧 effective config 不是运行时真相源；其健康的 Tavily 用量响应已排除，不能用来反驳当前服务的额度错误。

## Verification

### 财报专用模型：账户余额不足

- 有效配置：`opencode_acp` / `anthropic/claude-opus-4.8`；OpenRouter 配置池只有一个 key。该专用路由直接调用 OpenRouter，与普通聊天的模型路由不同。
- 三次 WDC 前瞻均出现相同的可用余额/在途请求预留额度拒绝：

| 开始时间 | 任务标识前缀 | 结果 |
| --- | --- | --- |
| 2026-10-18 21:49:48 | `58087d40` | 模型请求被拒绝 |
| 2026-10-19 19:54:17 | `ebe3d06f` | 模型请求被拒绝 |
| 2026-10-20 10:37:03 | `03529f3e` | 模型请求被拒绝 |

- 错误关键句：`This request would exceed your available credits given your current in-flight requests.`
- 实际所选 key 的只读 `/api/v1/key` 返回 HTTP 200、`expires_at=null`、每周 limit=100、remaining=98.5337944；不是 key 过期或每周 key 限额耗尽。
- 同一凭证的 `/api/v1/credits` 返回 HTTP 200：累计 credits=220、usage=219.834207384，剩余约 **0.165793 美元**。这个余额快照与三次历史拒绝互相印证；不能把剩余 key 周限额当作账户余额。

### 搜索：主路由和备用路由同时不可用

- 实际搜索代理先调用 Gemini 的 `googleSearch`，失败时才携带请求方的搜索凭证回退 Tavily。日志中的 `Tavily` 名称本身不能识别主路由。
- 主路由固定请求 `gemini-3.7-flash-low`；当前上游返回 HTTP 404 / `NOT_FOUND` / `Requested entity was not found`。这证明该请求路由不可用，尚不能单凭 404 确认模型被撤下或 OAuth 过期。
- 代理 journal 留存中，Gemini 最后一次成功为 **2026-10-13 08:02:53**，首次 404 为 **08:30:10**；Tavily 最后一次回退成功为 **2026-10-16 04:27:38**，十秒后开始套餐拒绝。
- 回退 Tavily 返回 HTTP 432 及套餐用量超限错误。管理服务 journal 检查窗口内最早该错误为 **2026-10-16 04:27:48**，截至 10 月 20 日 22:37 已记录 **1,021 次**真实上游套餐拒绝。这是日志次数，不是用户数或计费搜索数。
- 使用有效配置中实际传入代理的同一个 key，Tavily `GET /usage` 返回 HTTP 200：`plan_usage=1000`、`plan_limit=1000`、`key.search_usage=1000`、`paygo_usage=0`、`paygo_limit=null`。一次最小实际代理搜索也返回 HTTP 432。凭证可被识别，已证实的是套餐额度耗尽。
- 10 月 12–13 日窗口还出现 26 次 loopback 搜索连接错误，不能把整个历史窗口全部归为同一种供应商故障。

### PDF 成功并不证明搜索成功

10 月 18 日两个实际成功报告已与 ACP 原始工具状态核对：

| 报告 | 开始时间 | 原始材料预取 | 模型追加搜索 | PDF |
| --- | --- | --- | --- | --- |
| TSLA | 21:29:23 | 2 次均 retrieval_error | 2 次均失败，0 成功 | renderer 成功、end_turn |
| BE | 21:34:26 | 2 次均 retrieval_error | 2 次均失败，0 成功 | renderer 成功、end_turn |

- TSLA 的两次搜索明确请求 `topic=news,time_range=month,max_results=10`；BE 也有此新闻搜索参数。因此这两个样本不能表述为“模型根本没搜”，而是请求了最新资料但检索没有成功。
- 两轮成功直接读取公开 URL 的次数都是 0，但仍有 `data_fetch` 财务数据调用，不能扩张为“完全没有取任何新数据”。TSLA renderer 首次失败、第二次成功；BE 一次成功，故不能把 WDC 模型余额拒绝归为同一 PDF 技术问题。
- `earnings_materials.rs` 保留检索错误并允许研究流程披露或省略缺失事实；renderer 只负责排版和精确 artifact。这解释了两个样本为什么仍可交付 PDF，不代表报告具备完整的最新消息依据。
- 仅核对上述具体样本，不据此宣称全部历史 PDF 都没有搜索、所有正文都过时或所有数字都不可信。

## Risks / Follow-ups

- 恢复 OpenRouter 账户可用余额；key 周限额无需因为本次错误而盲目提高。
- 修复 Gemini 主搜索请求路由，并恢复实际 Tavily 备用账户的可用额度或更换授权可用凭证；不要只修改无关旧配置。
- 恢复后用一个独立财报前瞻做完整验收：实际搜索成功、公开原文可读、近期新闻日期合理、PDF 可下载。余额和 HTTP 探针不能代替报告验收。
- 当前未对历史全部报告逐份重跑或逐句验证。未发起充值、购买、模型替换、生产重启或用户消息发送。
- 本次仅文档改动，不运行 Rust/Web 构建或测试；`git diff --check` 与文档链接/归档检查通过。长期架构与内容约束未变，无需修改 repo-map、invariants 或 decisions。

## Next Entry Point

先按 `docs/runbooks/backend-deployment.md` 的财报供应商诊断顺序恢复实际模型余额和搜索路径，再执行隔离报告 canary；不要以重复点击失败任务来验证配额是否恢复。


## 2026-10-21 恢复阶段（done / deployed）

用户授权现有 Gemini 替代财报专用 OpenRouter 模型，并提供新 Tavily key。
从 origin/main `f7d87aff` 建独立恢复分支，未混入原工作区本地 V4 提交或其它未提交工作。
实现 `agent.earnings_workflow.provider` 指定服务器注册的 OpenAI-compatible provider，默认 OpenRouter 兼容；自定义凭据只注入隔离子进程环境，不序列化到 OpenCode 配置。

首个精确 revision `5ad8e7ba` 的 WDC canary 在首次 todowrite 后第二轮返回 Gemini 400。
同一工具调用做对照：标准消息省略工具 name 返回 400，补正确 name 返回 200/OK。
根因是既有代理把省略 name 的工具结果替换成 `tool`，与实际调用名不一致；OpenCode 按标准省略该字段，thought signature 在本地事件和会话存储中完整保留。
恢复范围包含最小代理协议补丁和多轮回归；原研究 Prompt、内容治理和 renderer 边界均未改变。

阶段验证：Rust workspace 2919 passed / 0 failed / 121 ignored；cargo check 通过；Web 669 passed；Worker typecheck 与 45 tests 通过；原有 25 个 CI regression 脚本 24 通过，finance automation 为 40/50，与 origin/main `f7d87aff` 相同 10 项失败，后续 14 项逐项完成。未把已有失败描述为全绿。


第二轮隔离 canary 的搜索已恢复（主机预取 2 次、模型搜索 8 次全部成功，4 次 news/month，4 份原文正文），但技能加载进程退出，未调用 renderer。最小 stdio 复现证实 `SkillTool::persist_invoked_skill` 仍调用测试构造器 `SessionStorage::new`；缺少 `HONE_POSTGRES_*` 时 panic。线上进程确有全部五项 PG 环境变量，本地隔离配置原先只内联 YAML；补临时数据库环境后同样技能调用退出 0 且无 stderr，因此按线上环境契约重跑。

已知遗留项（本次不扩大修改）：`crates/hone-tools/src/skill_tool.rs` 中 invoked-skill 元数据持久化误走隔离测试 schema；即使配置 PG 环境也不能写入真实会话。它与宿主消息及 PDF 附件持久化不是同一入口。后续应改用配置所属 PostgreSQL runtime，并对真实 actor 会话元数据追加测试；不要把此 canary 环境修正表述为已修复该持久化问题。


### 完整 Gemini canary 对照与选择

两轮均使用 runtime `5ad8e7ba`、同一原 Prompt/skills、同一供应商身份和修复后的 shadow proxy；独立本地 PostgreSQL 与测试 actor，不重放生产用户会话。

| 配置 | 终态 / 时间 | 搜索 / 原文 | Renderer / PDF | 人工复核 |
| --- | --- | --- | --- | --- |
| gemini-3.8-flash-medium | success / 166.3 秒 | 10 次成功、10 credits；4 份预取正文 | 1 次 / 1.543 秒；9 页、2,644,989 bytes | 连贯中文、Q1 FY2027 / 上期 Q4 一致，近期 10/16、10/20 消息进入终稿；DCF $175.37 跳至 $415–440 无量化桥，机构出处呈现不足 |
| gemini-3.8-flash-high | success / 145.5 秒 | 11 次成功、11 credits；4 份预取正文 | 1 次 / 0.993 秒；9 页、2,687,911 bytes | 本次样本未见改善；终值现值写 1284.5 亿，而按自身输入应约 509.16 亿；财报日期与预取电话会日期矛盾；未有效综合返回的 10/20–21 消息 |

选择保留 `gemini-3.8-flash-medium`；没有根据档位名称推定质量更好。medium 的主机历史/PDF 在重启隔离服务后仍可下载且 SHA256 不变：`7361a200b8e1f400bf58dd85cdcc47ac8a73716fc7f9f4f11131846268d3da1e`。9 页逐页查看，主体没有裁切/重叠，但 H5 Markdown 标记仍以文字显示；未把测试 PDF 当作内容和排版均已合格的对外交付。

medium 模型累计输入/输出 900,785/15,347 tokens，high 为 659,039/16,778；自定义 provider cost 字段为 0，不是已核实的费用或免费结论。两轮都有可恢复的数据类型/目录/grep 错误，均未重现 Gemini 400 或 MCP 断开。完整 canary 只证明实际取材、模型执行、技术交付和附件持久化恢复；事实和估值质量仍须后续研究流程迭代，未新增自动内容门禁。

最小证据保存在操作者目录 `honeclaw-artifacts/earnings-gemini-recovery-20261021/`：`canary/medium-high-canary-comparison.json`、两轮 `observed-tool-evidence.json`、medium `persistence-after-restart.json` 与 renderer 原稿/PDF；密钥及会话凭据不进入仓库。


### 生产切换与最终验收

- 北京时间 **2026-10-21 22:09:40** 完成验收；生产精确 runtime 为 `5ad8e7ba980407711147f0900cd68d4e1d77debd`，来源 `ghcr_linux_oci`。应用改动在该提交，代理最小补丁及无账号回归在 `fb043599`；后续文档提交不改变已部署二进制。
- [构建 39608620192](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/39608620192)、[导出 39613151201](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/39613151201) 均成功。镜像 digest `sha256:e4c7dc8e786ac44c8830d4a588f98909c527a3629bd482b6ce29e94bcc443f86`；导出 tar SHA256 `a2e736b19c4ad62a0355e0839381db8bd2b5c92cf7aba336986d70a06a245ce7`。本机下载遇网络 EOF，改由部署主机经短期 artifact URL 下载相同产物，校验一致；GitHub 凭据没有传给生产，也没有在生产编译。
- `/srv/honeclaw/config.yaml` 与 `/srv/honeclaw/data/runtime/effective-config.yaml` 读回一致：`opencode_acp / gemini_build / gemini-3.8-flash-medium`，执行预算 `context_window=131072 / output_max_tokens=32768`，搜索 `https://api.tavily.com/search`，实际所选 key 与用户提供值一致。旧 `/var/lib/hone/data/runtime/effective-config.yaml` 仍不是本服务真相源。
- 保留其它配置、`/etc/hone/runtime.env`、公开站点和 earnings skill 链接；新旧版本之间无数据库迁移。三次空闲检查后向 CLI 主进程发 SIGINT，确认旧监听和 Web/Feishu 锁释放，再启动精确版本；没有强杀或双实例竞争。
- 新 Web 和 Feishu 的 cgroup、锁持有者、进程路径、二进制哈希均通过，跨 metadata cache 的两轮验收成功；`hone-web.service` 为 active、`NRestarts=0`、主 PID `3691590`。PostgreSQL/OSS 健康，cloud authoritative、local durable dependencies=0；skills 并发读取、公开页面 fallback 和匿名接口 401 均通过。
- 线上真实 Tavily 查询 HTTP 200、3 条结果；用量 API 亦 HTTP 200，返回 Researcher / plan_limit=1000（即时 plan_usage=0 仅为该 API 快照，不据此推断 canary 无计费）。代理源码 SHA256 为 `11d831f2aa58dd5428d08bb4d63aca48cfaaae4884b6fe7159d30c43b13f01bc`，上线时标准无 name 工具结果双轮均 HTTP 200。
- OpenCode 生产版本 1.18.13 对隔离 `hone_earnings` provider/model/budgets 的解析已单独验证；本地完整 canary 使用 1.19.0。没有向生产聊天写入合成报告，也没有声称已重跑全部历史失败用户任务。财报分析模式未另做完整报告 canary。
- 私有部署脚本通过 17 项故障注入/原子写/回滚/锁/版本/冷启动等待测试；首次验收有界等待通道子进程就绪，35 秒后的复验保持严格检查。CI 新代理回归通过，原 finance 基线失败范围未扩大。

### 留存、回滚与后续入口

- 私有服务端证据：`/var/lib/hone/deployments/earnings-gemini-20261021/` 内的 `runtime-cutover/acceptance.json`、`runtime-cutover/config.before.yaml`、`runtime-cutover/rollback.json`、`runtime-cutover.log`、`proxy-cutover/acceptance.json`、`live-verification.json`。本机最小证据：`/Users/bytedance/Desktop/honeclaw-artifacts/earnings-gemini-recovery-20261021/` 下 canary 对照/PDF/原稿、测试日志、runtime 下载/验收日志。涉及凭据或会话的文件仅留在受保护目录，不进仓库。
- `/opt/hone/previous` 指向 `1ca6197c5c9efb1fa4affe6b7a458fd25cacb8fd-ghcr-runtime`。若回滚应用，必须同时恢复 `config.before.yaml`，旧二进制不支持新的 earnings provider 字段；按 backend-deployment runbook 的空闲/SIGINT/停止确认/精确版本验收执行，不能仅换链接。该回滚会重新暴露旧 OpenRouter 余额和搜索供应商故障；代理协议修复可独立保留。
- 临时本地 canary 服务、专用 PostgreSQL 和两个 SSH 转发已停止；shadow proxy 已停止且其复制的 OAuth 文件删除；临时本地明文 canary 配置已删除，保留脱敏配置。原生产 OAuth 文件未被替换，正式代理和 Web 继续 active。
- 下一步如继续改善研究质量，应从原 Prompt 内的估值推导、针对性原文读取和机构出处披露入手，以真实样本人工/模型评审验证，不能增加机械内容拒绝。另行修复已记录的 invoked-skill 元数据误用测试 schema 问题并保留真实 actor 回归。本次供应商恢复已完成，以上不是已验证解决的问题。
