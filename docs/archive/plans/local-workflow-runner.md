# Local 私有 Workflow Runner（公司研报 v1）

最后更新：2026-04-12
状态：已完成

## 目标

- 在 `local/workflow/` 下搭建一套与主系统解耦的本地私有 workflow runner
- 自动发现并执行 `local/*.yml` workflow，支持主链路 `company_report.yml`
- 提供图可视化、运行表单、节点状态、SSE 日志和运行产物查看
- 运行时复用主系统配置中的 OpenRouter / Gemini 3.1 Pro 与 Tavily 能力，但配置副本写入 `local/workflow/config/`

## 本次交付

- 新增独立本地服务与静态前端，统一由 `local/workflow` 启动
- 支持的节点子集：
  - `start`
  - `llm`
  - `tool`
  - `code`（`python3`）
  - `if-else`
  - `template-transform`
  - `variable-aggregator`
  - `end`
- 支持的工具子集：
  - `time.current_time`
  - `tavily.tavily_search`
  - `provider_type=workflow` 本地递归子 workflow
- `company_report.yml` 限定只支持 `genPost=完整跑完`
- 运行产物落到 `local/workflow/runs/<run_id>/`

## 关键实现点

- YAML 通过 `python3 + PyYAML` 解析，避免给本地子系统额外安装依赖
- 所有 LLM 节点统一走本地配置中的 OpenRouter 路由，模型固定 `google/gemini-3.1-pro-preview`
- Python code 节点通过受控 wrapper 执行，统一 JSON stdin/stdout，并捕获 stdout/stderr
- 子 workflow 输出统一规范化为：
  - `text`
  - `output`
  - `json`
  - 以及 end 节点声明的具体字段
- `company_report.yml` 的 `A股（热点个股）`、`美股`、`全跑完-A`、`全跑完-美` 分支明确标记为 unsupported

## 验证

- `cd local/workflow && bun test`
- `cd local/workflow && bun run bootstrap-config`
- `cd local/workflow && bun run run -- --workflow company_report --input '{"companyName":"Apple","genPost":"完整跑完","validateCode":"bamangniubi","news":"无","task_id":"demo-task","research_topic":"AI"}'`

## 说明

- 计划里要求的“把现有 `local/*.yml` 从 git 索引移除”在本仓库当前状态下无需执行：
  - `git ls-files local` 为空
  - `local/` 当前本身就是未跟踪目录
- 因为 `local/` 被整体 git ignore，`local/workflow/` 作为本地私有系统不会进入正式仓库发布面

## v1.1 增补

- 紧凑工作台替换原始 schema 大页面：
  - `company_report` 入口缩成主输入 + Advanced
  - `Graph` / `Prompts` / `Artifacts` 分面展示
- 新增运行级 prompt override：
  - `GET /api/workflows/:id` 返回 `editablePrompts`
  - `POST /api/runs` 支持 `promptOverrides`
- 新增停止与单活闭环：
  - `POST /api/runs/:id/stop`
  - 同 workflow 并发启动返回 `409`
  - run 支持 `stopped`，node 支持 `cancelled`
- SSE 事件流支持基于 cursor 的增量续流，前端按事件 id 去重，解决失败后历史日志重复回放
- Python code 节点改为 ASCII wrapper + UTF-8 用户代码文件执行，修复中文代码触发的 `Non-UTF-8 code` 问题

## v1.1 额外验证

- `cd local/workflow && bun test`
- `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`
- 手工 smoke：
  - 启动本地服务并访问 `http://127.0.0.1:3213/`
  - 发起 `company_report` 后再调用 stop，确认 run 最终为 `stopped`
  - 停止后的节点统计包含 `cancelled`

## v1.1.1 补丁

- Python code runner 额外增加了对 3.8/3.9 环境的现代类型注解兼容：
  - 执行前统一注入 `from __future__ import annotations`
  - 修复 `Dict[str, Any] | None` 在旧 Python 上触发的 `unsupported operand type(s) for |` 问题
- 顶层 `stderr` 现在在 `exec` 前就被接管，并过滤已知的 `urllib3 NotOpenSSLWarning`
- 回归验证：
  - `cd local/workflow && bun test`
  - 真实 `company_report` run 已确认不再卡死在 `精准查询财务信息` 的该注解错误上

## v1.2 可观测性与脚本调用

- 新增结构化进度：
  - `GET /api/runs/:id/progress`
  - `GET /api/runs/:id` / `GET /api/runs` 现在也带 `progress`
- `progress` 内容包含：
  - `percent`
  - `totalNodes / terminalNodes / runningNodes / pendingNodes`
  - `activeNodes`
  - `workflows`
- 节点观测增强：
  - `node_state` 带 `nodeKey`
  - 节点记录保留输入、输出和预览摘要
  - run 目录增加：
    - `node-records.json`
    - `progress.json`
- 页面增强：
  - 左侧摘要显示整体进度、活跃节点、分 workflow 进度、运行输入与最终输出
  - 日志显示节点级 running/succeeded/failed/cancelled
  - 右侧新增 `Nodes` 面板，默认折叠输入/输出
- 新增外部调用脚本：
  - `cd local/workflow && bun run client -- --workflow company_report --input '<json>'`
  - `cd local/workflow && bun run client -- --run-id <run_id>`

## v1.2 额外验证

- `cd local/workflow && bun test`
- `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts scripts/run_workflow_client.ts --outdir /tmp/local-workflow-build`
- `GET /api/runs/:id/progress` 已验证返回活跃节点和百分比
- `bun run client -- --run-id <run_id>` 已验证能连续打印进度并在 stop 后收到终态
