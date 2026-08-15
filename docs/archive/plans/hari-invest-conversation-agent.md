# Hari Invest Conversation Agent

- title: Hari Invest 对话决策层与 Agent 可用性增强
- status: done_locally
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex

## Goal

把用户提供的 `laowang-investment-internal-v0.4.0` 内部研究包转换为普通 HONE 用户可用的脱敏对话 Skill：不暴露团队工作流或冒充老王，但能用已蒸馏框架结合当前证据，先给明确研究结论、短中长期判断、行动条件和证伪点。

## Delivered

- 审核用户 ZIP、本机内部 v0.4.1 与仓库原 `hari-invest`，保留 v0.4.1 的工具无关联网规则，不公开内部维护 Skill。
- 将 `skills/hari-invest` 升级为公共对话决策层，增加决策分区、对话契约、自然中文触发与隐式调用元数据。
- 强化系统级投资回答兜底：数据时间之后首段必须先下结论，区分关键缺数和次要缺数，禁止用“支持/部分支持”等内部标签作答。
- 扩展运行时中文片段匹配，并新增静态契约和单元回归。
- 通过真实本地对话确认 Agent 确实读取 Skill 及引用文件后生成结论先行的回答。

## Verification

- Skill Creator `quick_validate.py`: passed.
- `test_hari_invest_conversation_contract.sh`: passed.
- `hone-tools`: 185 passed, 1 ignored.
- `hone-channels`: 789 passed, 1 ignored.
- `hone-console-page`: built successfully.
- Live admin dialogue: Skill and references appeared in tool trace; final answer selected `风险区` with confidence, time horizons, counterargument and change conditions.
- Public-dev dialogue: failed closed before Skill execution because no actor-safe model provider was configured; host ACP was not exposed.

## Remaining Operational Prerequisite

独立网页给普通用户使用前，必须配置 actor-safe 的服务端 function-calling 模型或 `hone_cloud`、相应密钥与健康检查，再执行真实用户通道的黄金问答验收。此项是部署配置和密钥前置条件，不应通过放宽本机 ACP 权限解决。
