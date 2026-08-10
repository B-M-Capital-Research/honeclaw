# Earnings Workflow 原流程直接迁移

- title: Earnings Workflow 原流程直接迁移
- status: in_progress
- created_at: 2026-08-05
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - `skills/earnings-research/SKILL.md`
  - `skills/earnings-research/scripts/render_report_pdf.py`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/mcp_bridge.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `tests/regression/ci/test_earnings_research_pdf_markdown.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-10-earnings-opencode-signature-recovery.md`

## Goal

把本地 BamangResearch 对应 Dify 的财报前瞻 V2、财报分析 V2 和近期新闻子流程按原有简单数据流与 prompt 直接迁回 HONE；删除后来叠加的 `preview_audit`、固定 8–10 条新闻、固定句数/页数/标题及 renderer 内容裁判，解决长循环、高成本、占位来源和内容反而弱于原流程的问题。

## Scope

- 专用财报轮次只接收当前结构化请求、附件、技能和本轮证据；不把此前会话消息或 compact summary 送入 runner。完成报告仍写回原会话，供后续普通对话继续引用。
- 保留原流程：实体确认 → 当前财务/财报数据 → 原 query prompt 生成 5–8 个查询 → 搜索聚合 → 原前瞻或分析 prompt → 前瞻追加原新闻 prompt → PDF。
- 原 BamangResearch prompt 是报告内容与结构的真相源；不再增加第二套预审 schema、机构字段、新闻数量、自然段句数或页数要求。
- 真实性约束留在研究阶段：重要事实缺失或矛盾时先做针对性搜索；仍不可核验时明确写“未找到可核验来源”或省略。不得编造来源、URL、机构、引语、数字、事件或因果关系。
- renderer 只保真排版任意 Markdown，并加水印、免责声明和知识星球分享页；不改写内容，也不规定来源章节的格式。renderer 启动前，专用 MCP child 用本轮成功的 DataFetch/Web Search 结果建立短生命周期证据账本，并要求隐藏的 `evidence_manifest` 将每条重大断言映射到可见 URL 和同一结果中的逐字摘录；URL、摘录或数字不闭环时不产生文件。
- 保留宿主 PDF terminal closure：只有官方 renderer 成功且 PDF 被当前 actor 持久化，专用轮次才算成功。

## Verification

- 技能结构校验与 Python 编译通过。
- CI 回归证明：无需 `preview_audit`；少量真实新闻和明确证据缺口可生成；匿名机构、`example.com` 和未替换模板被拒绝；Markdown 表格与任意原 prompt 标题被保真渲染。
- Rust 测试证明：专用财报轮次清除历史消息；原 prompt 系统覆盖取代普通投研模板；renderer 恢复提示只处理占位/虚假来源或技术错误，不要求为版式改写报告。
- 运行 changed rustfmt、相关 crate tests、CI-safe regression 和 `git diff --check`。
- `bd2eb2f99e7ff62ed856902f8771b0314887d10c` 已推送 `main`；Runtime Image、CI、Secret Scan、Code Quality 与 Release Cache Warm 均通过，精确 GHCR runtime digest 为 `sha256:f44be080c43625d3ae80fee58792a8d0e6f7c14f67ce3f72c9683ddc169b6668`。
- 生产已切到该精确 revision，技能从 system 目录加载且正文包含原流程契约；服务 `active/running`、`NRestarts=0`、云存储权威、PostgreSQL/OSS 健康、切换后 warning/error 为 0。
- 已以生产 service user 调用新 renderer 生成 CRWV smoke PDF；回传文件哈希一致、A4 两页，并逐页确认中文、表格、水印、免责声明和分享页无歪斜或截断。该 smoke 只证明 renderer/宿主环境，不替代真实 LLM 内容 canary。
- 第一轮真实 CRWV canary `1e382729-3f4f-4952-b2bd-68f667a58873` 在精确 `bd2eb2f9` 上完成 2 次结构化取数、5 次搜索和 1 次 renderer，109.220 秒内生成 551,210 字节/3 页 PDF，用量 41,165 tokens、费用 `$0.227074`；但宿主最终优先取模型的无附件文本 chunk，导致 PDF 没有挂载/持久化。独立任务还错误继承上一轮 `121,759` token 峰值并把 41,165 误判为 compact。报告包含若干本轮工具结果未覆盖且没有显示来源 URL 的事实，即使事后能从官方资料核实，也违反“只能使用本轮证据”的契约，因此本次 canary 判定失败。
- 后续窄修复让财报终稿始终使用 renderer 返回的 `validated_report_markdown + artifact`，普通 ACP 回复仍保持原终稿选择；财报任务不再读取或续存历史 usage peak。技能和系统覆盖明确禁止模型记忆补事实，要求对未覆盖断言定向搜索/删除；renderer 只新增“至少一个真实来源 URL”的最小证据门禁，不恢复固定结构审计。针对性回归、`hone-channels` 794/1 ignored、Web chat 15/15 和完整 CI-safe 回归通过。
- 修复提交 `5a43272946d99d96006b5e663e17aca3ab8dbc85` 已推送并以精确 GHCR digest `sha256:5daff66b8bce5b338b3bded180058ff0b2d35b53072425f266a2395aa0be4c84` 部署。最终生产读回证明 exact SHA、`ghcr_linux_oci`、云存储权威、PostgreSQL/OSS 健康、system skill enabled、公开鉴权 JSON `401`、`NRestarts=0` 和切换后 warning/error 为 0。生产 service user 的最小证据渲染冒烟生成 210,378 字节、两页 A4 PDF，逐页确认中文、水印、免责声明和星球分享页正常。当前只待第二次真实 LLM canary。
- 第二次生产 CRWV canary `8be00fa4-e106-4861-a820-5bfcf6126e9f` 使用 fresh ACP session `ses_0137f2b72ffe3X7ve1W2dShwnI`，无 compact/重试，在 146.625 秒、41,330 tokens、`$0.224342` 内完成四次取数、五次搜索和一次 renderer；PDF 560,683 字节、三页 A4，源码与 OSS 下载哈希同为 `387504ea73c7d9997828c3977f743a197dc7aab27da530c651ee147623cb9844`，刷新后附件仍存在。技术闭环通过，但报告把工具结果未出现的 EPS 下限、2.9 GW、2026 ARR 下限和 Goldman 判断混入只有三条真实 URL 的报告，证明“至少一条 URL”不能防止其它断言搭便车，本次内容验收失败。
- 随后的生产 LITE 对照样本 `065e6f33-934c-491c-9a81-8b71aeaadf36` 同样使用 fresh ACP session `ses_01375ab6dffeEKWg5XS90awwjU` 且 `time_compacting=NULL`，112.240 秒、40,492 tokens、`$0.39236` 后成功持久化 520,836 字节 PDF。它先因没有 URL 被 renderer 拒绝，补链接后第二次渲染成功；但来源原文的 `$960M–$1.01B` 被报告写成“9.6亿–1.01亿美元”，再次证明终态成功不等于数值真实。新增回归明确允许正确的 `$960M–$1.01B ↔ 9.6亿–10.1亿`，并在执行 PDF 前拒绝该实际数量级错误。
- 本地严格证据门禁已实现：只对 server-verified 专用 earnings turn 启用；每个 MCP child 只记录本轮成功的 `data_fetch` / `web_search` URL 和原文；重大行必须逐字进入 `evidence_manifest`，来源 URL 必须在报告可见，摘录必须属于同一 URL 的本轮结果，报告数字必须出现在映射摘录中，并统一 million/billion/亿元口径。失败返回 `side_effect_status=not_started`、零 artifact，供现有安全修正路径定向搜索或删改；普通对话、其它 skill 和 PDF 版式不受影响。工作区 check、`hone-channels` 800/1 ignored、技能/renderer 检查和完整 CI-safe regression 已通过，当前待推送和精确生产部署后再跑真实 canary。
- 部署精确 revision 与技能后，用一个生产前瞻 canary 验证：无旧会话污染、无 compact、搜索后无占位来源、renderer 调用接近一次、PDF 可下载且刷新后仍存在；记录 token、cost 和耗时。

## Documentation Sync

- 更新 `docs/invariants.md`、`docs/decisions.md`、`docs/repo-map.md` 和 `docs/current-plan.md`，将“原 prompt + 搜索真实性 + layout-only renderer”设为长期约束。
- 部署验收后追加当天已有 earnings handoff；计划完成后移入 `docs/archive/plans/`，更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks

- 搜索结果可能不足以填满原 prompt 的全部字段；必须暴露缺口或省略，不得用模型记忆补齐。
- 原 prompt 包含对预测、估值和机构观点的高要求；这些要求触发更多搜索，但不构成伪造某个数值的理由。
- 逐字摘录与数值门禁能阻止未见 URL、虚构摘录和数量级漂移，但不替代语义判断：同一真实摘录仍可能被错误解释。生产 canary 仍需人工审查因果关系、时间归属和来源是否真正支持对应断言；不要把复杂研究判断硬编码进排版器。
