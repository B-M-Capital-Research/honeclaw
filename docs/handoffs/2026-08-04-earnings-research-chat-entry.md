# Earnings Research Chat Entry Handoff

- title: 管理员财报前瞻 / 财报分析原生 Skill 与 PDF 交付
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - packages/app/src/pages/chat.tsx
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/execution.rs
  - crates/hone-tools/src/skill_tool.rs
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
- related_docs:
  - docs/archive/plans/earnings-research-chat-entry.md
  - docs/archive/plans/earnings-preview-expectation-model.md
  - docs/archive/plans/earnings-preview-news-page.md
  - docs/archive/plans/earnings-preview-news-freshness.md
  - docs/decisions.md#d-2026-08-04-01-run-earnings-research-as-an-actor-scoped-native-skill
  - docs/archive/index.md
- related_prs: none; delivered as a direct `main` feature commit, without release or deployment

## Summary

HONE 公共用户端现在在“持仓分析”旁为管理员显示“财报前瞻”和“财报分析”。弹窗将公司、模式与可选财报附件作为结构化请求提交；服务端重新核验管理员、生成不可由客户端覆盖的 skill 指令，并强制激活 `earnings-research`。skill 按固定阶段完成证据研究，并按旧 Workflow 的模式专属格式生成最终报告，再生成带 HONE 水印、免责声明与知识星球分享页的 PDF，作为同一聊天回复的附件恢复显示。

## What Changed

- 前端新增两个管理员入口、模式化弹窗、公司校验、多文件选择和结构化发送。
- 公共聊天 API 新增 `earnings_workflow`，普通用户结构化请求或显式 `/earnings-research` 绕过均返回拒绝。
- 历史投影只显示用户可读请求，不泄露服务端内部 slash skill/workflow block。
- `skill_tool` 支持受控 PDF document artifact；Web 文件代理只放行当前 actor sandbox 或既有托管上传。
- 新增原生 `earnings-research` skill 与无外部 Python 包的 Chromium HTML→PDF 渲染器。
- 数据库复核后的公共管理员通过内部 `ConfiguredTrustedAdministrator` 选择使用 configured native runner；该信任位只由服务端构造，普通 actor 仍走 strict fallback。
- PDF Markdown 渲染器现在把标准表格输出为语义化、可换行的表格，不再把财务指标表压成普通段落。
- 最终正文不再使用 Hone 普通九段式问答：`preview` 严格固定为“整体分析 / 核心股价因素 / 业绩指引 vs 机构观点”层级；`analysis` 严格固定为“利润表 / 资产负债表 / 现金流量表 / 补充财务增长指标 / 数据总结”。渲染器在生成 PDF 前校验标题、层级、顺序及禁用章节。
- PDF 免责声明已并入知识星球分享页，避免正文刚好占满页面时产生只含免责声明的空白中间页。
- Workflow 文风也纳入 Skill 契约：前瞻正文第一句先做 `超出分析师预期 / 低于分析师预期 / 与分析师持平` 三选一判断，`1.2.1` 必须保持同一结论；管理层指引、机构预期、近期新闻/订单/产品和财务假设必须形成一条因果链。两种模式均禁止时间口径开场、`事实/推断/本轮` 等模型元话术和重复的分节结论。

## Verification

- Skill schema 校验通过；真实 SNDK PDF 为 A4、5 页、可搜索中文文本，全部 5 页视觉检查确认指标表、水印、无浏览器页眉页脚和知识星球图片。
- Web 完整 354 项测试通过；类型检查与 Public 生产构建通过。
- Rust 完整 `hone-tools` 171/171、`hone-web-api` 182/182 通过（另有 1/2 个既有 opt-in ignore）；聚焦 `hone-channels` MCP 输出目录测试通过。
- 本地浏览器用管理员 mock 验收两个入口、弹窗、附件上传、loading、最终报告和 PDF 卡片；请求体分别携带 `preview` / `analysis`。普通用户 mock 下两个入口均不存在。
- 真实隔离管理员 SNDK 财报分析在 Codex CLI `0.146.0` / codex-acp `1.1.7` 上完成，`agent.run=codex_acp`、23 次工具调用、官方财报/SEC/结构化数据证据、最终正文和 PDF artifact 均成功；`hone-channels` 743/743（1 ignore）、`hone-web-api` 182/182（2 ignore）、PDF table CI 回归与 Chromium 手工回归通过。
- 格式纠偏验收读取旧 Dify `V2-财报前瞻` 编排提示和 `最新财报解析` 真实成功日志，不调用旧 Workflow 运行 API。新增 CI 反例证明普通问答标题、错误标题层级和估值等禁用章节会被拒绝；重排后的 SNDK PDF 为 3 页 A4，全部页面经 120 DPI 渲染逐页检查，无截断、重叠或空白中间页。
- 文风纠偏后用 SNDK FY2026 Q4 真实前瞻样例验收：报告第一句为“低于分析师预期”，随后以公司收入/EPS 指引、机构一致预期、Datacenter 与 NAND 价格、近期新增 NBM 协议及 Kioxia 合作串联判断。`quick_validate.py`、CI 正反例与 Chromium PDF 回归通过；最终 3 页 A4 PDF 全页视觉检查无截断、重叠或乱码。
- 预期判断流程随后做了系统性校正，不再用“管理层指引低于一致预期”直接推导 miss。前瞻必须固定同一财季和一致预期截止日，核对多来源分歧与修订，回测至少三个可比季度的指引偏差，读取最新电话会与演示材料，为合同/订单/产品逐项判断是否已计入指引，再形成收入与利润独立预测和中性带。
- PDF spec 新增私有 `preview_audit`。渲染器要求收入与至少一个利润指标、两项决策指标、历史指引、催化剂计入状态和至少三个经营桥接因素；按预测减一致预期相对于容差的方向重新计算结论。混合信号只能判为持平，审计值与 `1.2.2` 正文展示值不一致也会拒绝生成。
- 新 SNDK FY2026 Q4 样例采用截至 2026-08-04 的多来源预期区间，以 84.0 亿美元收入和 34.52 美元调整后 EPS 为主口径，独立预测为 87.5 亿美元和 36.5 美元，结论校正为“超出分析师预期”。NBM 多年合同与 Stargate QLC 均按已计入 Q4 指引处理；BiCS10/K2 进展按后续年度处理，未重复计入本季。最终 `output/pdf/SNDK-FY2026-Q4-earnings-preview-new-process-2026-08-04-34dfabff.pdf` 为 3 页 A4，全部页面按 120 DPI 视觉检查通过。
- 前瞻固定新增 `1.3 近期新闻`，以 4–8 条绝对日期倒序事件单独起一页。每条必须同时写出事件、当季经营影响、`已计入/未计入/部分计入/未知` 和来源链接；缺页、条数不足、字段不完整或没有经营变量会在渲染前失败。新版 SNDK 新闻页列出 Wedbush 预期上修、K2 第十代产线、BiCS10 送样、闪存价格、NBM 协议和 Stargate QLC，共 6 条；最终 `output/pdf/SNDK-FY2026-Q4-earnings-preview-with-news-2026-08-04-c6d2db81.pdf` 为 4 页 A4，第 3 页仅承载新闻，第 4 页保持分享图，全部页面视觉检查通过。
- 新闻时效与密度继续收紧为通用流程约束，不再依赖公司恰好发布公告。每份前瞻必须提供财报日，新闻固定 8–10 条、严格倒序、至少一半位于财报日前 14 天，并同时覆盖公司/预期、同业/供应链和需求端；每条新增显式类型。反例覆盖条数不足、字段缺失、过半事件过旧和需求端缺失。
- 增强后的 SNDK 样例为 10 条，其中 8 条位于 7 月 22 日至 8 月 5 日窗口，3 条发生在 8 月；加入 8 月 4 日 HBF 首版标准、财报日前一致预期、Seagate Q4、Amazon/Meta/Alphabet AI 基建需求和 Zacks 预期样本。最终 `output/pdf/SNDK-FY2026-Q4-earnings-preview-news-enhanced-2026-08-04-63dfcfa6.pdf` 为 4 页 A4，新闻保持一整页，全部页面以 150 DPI 检查无截断、重叠或空白页。
- Workflow 视觉对齐阶段逐页读取用户提供的 8 页参考 PDF，并把可复用视觉语法落入原生 Chromium 渲染器：米色页眉页脚、橙色章节条、深色高密度正文、章节名/页码、精确水印 `知识星球：巴芒科技` 和独立分享页。最终 `output/pdf/SNDK-FY2026-Q4-earnings-preview-news-enhanced-2026-08-04-482bdffa.pdf` 为 6 页 A4，全部页面逐页渲染检查通过；近期新闻占两页，文本可搜索。
- 用户下载验收新增 `packages/app/e2e/public-chat-pdf-download.spec.ts`。实际 HONE 管理员对话 UI 显示 `SNDK（闪迪）_财报前瞻.pdf` 附件卡片；文件 URL 经当前用户鉴权代理返回 `application/pdf` 与 `%PDF-`，点击卡片触发相同中文文件名下载，并保留验收截图 `output/pdf/screenshots/user-chat-pdf-download.png`。附件卡片不再同时打开新标签页。
- 当前源码真实管理员续验收最终闭环：按钮提交 `/earnings-research` 后，Codex ACP `1.1.7` / Codex CLI `0.146.0` 只围绕 SNDK 运行实体、预期、新闻与官方材料工具；宿主 `hone/skill_tool earnings-research` 依次返回结构校验反馈并在第三次规范修订后成功生成 `sndk-fy2026-q4-earnings-preview-5882eda4.pdf`。同一助手消息显示完整旧 Workflow 正文和 PDF 卡片，卡片已在浏览器中实际点击。
- 该续验收修复了四个通用运行边界：slash skill 正文不再参与实体发现/重执行分类；数据库管理员的原生 prompt ownership 从准备阶段开始生效；预检 Web 查询硬限制为供应商允许长度；Native Codex 可调用宿主 `skill_tool` 执行可信 skill 脚本，且 MCP 子进程通过绝对 `HONE_SKILLS_DIR` 找到仓库技能。由此 PDF 使用宿主 Chrome，而不是 actor 沙箱内的临时 ReportLab/Swift 替代实现。
- 最终真实聊天产物复制为 `earnings-reports/SNDK-闪迪-财报前瞻-真实聊天验收.pdf`：A4、5 页、620338 bytes，全部页面可搜索且含精确 `知识星球：巴芒科技` 水印；第 3–4 页承载近期新闻，第 5 页为原知识星球分享图。`pdfinfo`、PyPDF 文本抽取和 120 DPI 全页 PNG 检查均通过。

## Risks / Follow-ups

- 本次没有生产发布；已经连接真实 LLM/行情/官方来源跑通 SNDK 财报分析，但生产前仍应使用真实 Web 管理员账号补一轮财报前瞻和带附件财报分析，以覆盖公共 HTTP 鉴权与部署主机浏览器环境。
- PDF 依赖运行主机可发现的 Chrome/Chromium；找不到浏览器时 skill 会保留完整聊天报告并明确报告 PDF 失败。
- 旧 BamangResearch HTTPS 入口证书已过期，本次未绕过安全页；格式证据来自同域可正常访问且已有登录态的 Dify 编排与日志页面。
- 若以后开放给普通用户，必须同时修改前端可见性、服务端授权、slash bypass 规则和产品配额；不能只去掉前端 `<Show>`。
- `preview_audit` 能阻止结构和方向冲突，但不能证明独立预测必然正确；中性带和经营桥接仍需由当前证据解释，不能用公司特例或事后结果硬编码。
- 本地 `18077/18088` 隔离端口已用当前源码完成真实管理员 SNDK 前瞻、PDF 卡片与点击下载验收并在结束后停止；没有替换或部署 `8088` 的现有运行时。隔离后端复用了 cloud-authoritative 数据，启动时事件引擎曾把少量新事件加入既有 digest 队列；后续此类真实验收应增加关闭 scheduler/event-engine 的专用运行参数，避免研究验收触碰无关事件状态。

## Next Entry Point

从 `skills/earnings-research/SKILL.md` 检查研究契约，从 `crates/hone-web-api/src/routes/public.rs` 检查授权与强制激活，从 `skills/earnings-research/scripts/render_report_pdf.py` 检查 PDF 品牌交付。生产验收应复用 `tests/regression/ci/test_earnings_research_pdf_markdown.sh` 与 `tests/regression/manual/test_earnings_research_pdf.sh`，并再做一次真实管理员浏览器回合。
