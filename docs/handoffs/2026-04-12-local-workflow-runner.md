# 2026-04-12 Local 私有 Workflow Runner

## 目标

- 在 `local/workflow/` 下落地独立私有 workflow runner
- 跑通公司研报主链 `company_report.yml`
- 提供可视化、运行、观测和产物查看能力

## 结果

- 已新增独立本地服务与页面：
  - workflow 自动发现
  - graph 可视化
  - 运行表单
  - 节点状态
  - SSE 日志
  - 运行产物 / markdown / 上传结果查看
- 已支持主链依赖的 4 份 YAML：
  - `company_report.yml`
  - `搜索引擎补强.yml`
  - `公司财务指标获取.yml`
  - `精准查询财务信息.yml`
- LLM、Tavily、Python code、递归 workflow tool 均已接通
- `company_report.yml` 仅开放 `完整跑完` 模式，其它分支会被显式拒绝

## 验证

- `cd local/workflow && bun test`
- `cd local/workflow && bun run bootstrap-config`
- `cd local/workflow && bun run run -- --workflow company_report --input ...`

## 风险与后续

- 当前只兼容这 4 份 YAML 所需的 Dify 子集，不是通用完整兼容层
- 真实外网运行依赖：
  - OpenRouter API key
  - Tavily API key
  - Python `requests`
  - 本机可用 `python3`
- `精准查询财务信息.yml` 中仍保留原始 code node 里的外部接口逻辑与硬编码内容；本地系统只负责兼容执行，不对原 DSL 做语义重写

## v1.1 可用性与闭环修复

- 页面改为紧凑工作台：
  - 顶部只保留 `companyName` 与 `research_topic`
  - `news` / `task_id` / `validateCode` 收进 `Advanced`
  - 右侧拆成 `Graph` / `Prompts` / `Artifacts`
- 新增运行级 prompt 覆盖：
  - 页面会聚合主流程与本地可发现子 workflow 的 LLM 节点
  - 修改只作用于当前 run，不写回 YAML
- 运行生命周期补齐：
  - 新增 `POST /api/runs/:id/stop`
  - 同一 workflow 增加单实例串行保护，重复启动返回 `409`
  - `RunSummary.status` 支持 `stopped`
  - `NodeStatus` 支持 `cancelled`
- SSE 重连重复日志问题已修复：
  - 事件流增加游标续流
  - 前端按 event id 去重
  - 收到 `run_finished` 后主动关闭 `EventSource`
- Python code 节点 UTF-8 问题已修复：
  - wrapper 改为 ASCII-only
  - 用户代码单独写成 UTF-8 文件读取执行
  - 已验证含中文注释 / 中文字符串的 code node 正常运行

## v1.1 验证补充

- `cd local/workflow && bun test`
- `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`
- 真实服务 smoke：
  - `WORKFLOW_RUNNER_PORT=3213 bun run start`
  - `GET /api/workflows/company_report` 返回 `launchConfig` 与 `editablePrompts`
  - `POST /api/runs` 发起 `company_report`
  - `POST /api/runs/:id/stop` 成功把 run 收口到 `stopped`
  - 停止后的 run 节点状态统计包含 `cancelled: 1`

## v1.1.1 Python 兼容补丁

- 额外修复了本机 `python3=3.8.8` / 3.9 系列下的 code node 兼容问题：
  - runner 在执行用户代码前统一注入 `from __future__ import annotations`
  - 这样 `Dict[str, Any] | None`、`list[str]` 等现代注解不会在模块加载阶段被立即求值
- wrapper 现在在 `exec` 前就接管 `stdout/stderr`，顶层 import / warning 不会再直接漏到子进程 stderr
- 已知的 `urllib3` `NotOpenSSLWarning`（LibreSSL 环境噪音）会被过滤，避免把正常执行刷成报错日志
- 真实验证：
  - 之前失败的 `精准查询财务信息` 已不再卡在 `unsupported operand type(s) for |` 这条错误上
  - 新 run `2026-04-12T09-33-51-190Z-e9146f98` 已成功跑过 `精准查询财务信息 -> workflow_finished:succeeded`

## 当前建议入口

- 页面地址：`http://127.0.0.1:3213/`
- 建议先在页面里验证：
  - 输入 `companyName` / `research_topic`
  - 按需展开 `Advanced`
  - 如需临时改词，切到 `Prompts`
  - 点 `Run` 后观察左侧日志与右侧 graph
  - 若要中止，直接点 `Stop`

## v1.2 可观测性与外部调用补充

- 可观测性增强：
  - `run` 详情和 SSE 事件现在会返回结构化 `progress`
  - 节点事件会携带 `workflowId/workflowName/nodeKey`
  - 节点记录会保留：
    - `inputs`
    - `outputs`
    - `inputPreview`
    - `outputPreview`
    - `error`
  - 页面日志会显示 `node running / succeeded / failed / cancelled`
  - 页面新增 `Nodes` 面板，按 workflow 展示节点详情，输入输出默认折叠
  - 页面摘要新增整体进度、当前活跃节点、分 workflow 进度、运行输入和最终输出
- API 新增：
  - `GET /api/runs/:id/progress`
  - `GET /api/runs/:id` / `GET /api/runs` 现在也带 `progress`
- 本机脚本入口：
  - `cd local/workflow && bun run client -- --workflow company_report --input '{"companyName":"Tempus AI","research_topic":"AI 医疗数据","news":"无","validateCode":"bamangniubi","task_id":"demo"}'`
  - 只启动不监听：`--no-watch`
  - 监听已有 run：`--run-id <run_id>`

## v1.2 验证补充

- `cd local/workflow && bun test`
- `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts scripts/run_workflow_client.ts --outdir /tmp/local-workflow-build`
- 真实服务 smoke：
  - `GET /api/runs/:id/progress` 已返回：
    - `percent`
    - `terminalNodes`
    - `runningNodes`
    - `activeNodes`
    - `workflows`
  - `bun run client -- --run-id <run_id>` 已能持续输出进度变化
  - client 监控输出示例中已看到：
    - `percent=30.2% terminal=19/63`
    - `active=（V3-修改版）带图片·真·全自动工作流/公司财务指标获取, 公司财务指标获取/精准查询财务信息, 精准查询财务信息/提取股票Code`
  - stop 后 client 能收到终态：
    - `[done] run_id=... status=stopped error=run stopped by user`
