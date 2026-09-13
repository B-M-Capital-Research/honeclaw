# 原始 Workflow 对照

核对日期：2026-09-13。通过已登录 Dify 编排页面逐节点只读核对；未触发旧系统研究运行或发布。

- 财报分析：[V2-财报分析](http://bm.vsource.club:3004/app/1180240f-1417-4a28-9982-d9d35916034f/workflow)
- 财报前瞻：[V2-财报前瞻](http://bm.vsource.club:3004/app/2f660087-1f01-45d1-8327-82101bf75a5b/workflow)
- 前瞻新闻子流程：[公司近期新闻时间线分析模块](http://bm.vsource.club:3004/app/6bc49b91-577b-42e0-a11f-f0a55e40cb30/workflow)

`SKILL.md` 保留原节点 Prompt 正文、案例、示例与原有措辞。仅将 Dify 变量芯片换成具名占位符，合并编辑器的空白行与不换行空格；不对报告建立内容 validator。原 Prompt 的篇幅/章节/JSON 示例用于生成，不成为代码拒绝条件。

| 节点 | 本地位置 | 数据来源 |
| --- | --- | --- |
| LLM 2 查询生成 | 共同的查询 SYSTEM/USER | company、北京时间 |
| Tavily Search + 聚合 | 共同定向搜索阶段 | 原查询，basic/general、10 条；原界面 days=7 |
| 精准查询财务信息 | 内置财务取数 | 当前公司 financials / earnings_outlook 等 |
| 前瞻 LLM 2 | Preview original V2 | 财务 + 聚合搜索结果 |
| 新闻 LLM | Preview news 查询 SYSTEM/USER | 公司及日期；保留原文 4 组请求、3 项 JSON 示例 |
| 新闻 Tavily + 代码执行 2 | 新闻搜索与聚合 | news、days=30、basic、10 条 |
| 新闻 LLM 2 | Preview news 报告 Prompt | 聚合新闻 |
| 分析 LLM 2 | Analysis 第一段 | 最新已发布财报、同期电话会、财务与搜索 |
| 分析 LLM 4 | Analysis 第二段 | 相同素材 + 第一段结果，原样拼接 |
| 模板转换/结束 | PDF delivery | 完整最终 Markdown，技术渲染与附件持久化 |

输入适配：用户只填公司。原分析起点的 sec_analysis / earning_calls 由系统检索原文填充；symbol 由公司输入解析。旧系统 task_id / 上报进度节点由 HONE 的既有 run_progress 替代。不得调用 Dify 来完成本地研究。

本轮恢复此前迁移中删改的分析表格、估值案例、风险/投资建议/结论示例及前瞻原词“纰漏”，补回遗漏的查询 USER 和新闻查询节点。删除后来附加的“发布状态与单位自检”段；取材时选择已发布/未发布季度与缺失事实披露属于研究过程，不增加终稿审核或循环重写。
