# 财报前瞻失败与搜索资料缺失诊断

- title: 财报前瞻失败与搜索资料缺失诊断
- status: done（诊断完成；供应商恢复与恢复后完整报告验收待执行）
- created_at: 2026-10-20
- updated_at: 2026-10-20
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
- related_prs: none

## Summary

此次调查已区分三个供应商故障：财报专用 OpenRouter 账户余额不足；主搜索 Gemini 请求返回 404；回退 Tavily 达到套餐额度上限。它们分别解释近期整条前瞻失败与此前 PDF 成功但缺少最新网页资料。

以下任务时间均为北京时间。诊断不等于恢复验收；没有更改生产代码、模型、凭证、额度或服务进程，也没有重放用户任务。

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
