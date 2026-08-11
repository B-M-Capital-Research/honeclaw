- title: 宏观红绿灯与 AI 红绿灯每日仪表盘
- status: completed_locally
- created_at: 2026-08-10
- completed_at: 2026-08-10
- owner: Codex

## Goal

在本地 HONE 对话快捷区增加“宏观红绿灯”和“AI 红绿灯”，只读取每天北京时间 20:00 预生成的结构化 Latest 报告；支持历史、来源、变化、失败回退和基于报告上下文的连续提问，不在点击时启动完整研究任务。

## Delivered

- 认证只读 API、Latest/History 原子快照、20:00 Asia/Shanghai worker、单实例锁、同日去重、不完整快照 15 分钟重试、重试日志和成功快照保留。
- 宏观 11 维 Ahead of Curve 链、领先/确认/滞后角色、0–10 原始风险与 0–100 健康分双口径、绿黄橙红阶段和 10 年趋势。
- AI 四盏灯、四大 CSP 十项框架、六层指标、Capex 绝对额/增速/峰值状态和八个硬件/电力链市场确认。
- 两个聊天上方按钮、半圆仪表盘、变化、提醒、历史、来源/证据/阈值、桌面/移动/暗色/加载/失败/过期状态。
- 报告提问将已保存快照作为隐藏上下文发送到同一聊天，明确事实/推断/情景和截止日，不改写正式评分。

## Data Boundary

- 宏观主数据源为 FRED 官方单序列 CSV；多序列端点返回 ZIP，因此并发拉取单序列并允许部分覆盖。规范项目 User-Agent 避免 FRED 防护层挂起请求。
- AI 当前 effective config 没有 FMP key，因此以 SEC EDGAR Company Facts 提供 CSP 标准财报底座，状态为 `partial`；硬件行情继续保持未知。配置既有 key pool 后会补充 FMP 行情。
- 缺失值从不作为零；若已有成功快照而新一轮证据不足，则沿用成功结果并标记 `stale`。

## Verification

- Web API 完整套件此前为 218 passed，2 ignored；数据恢复新增信号逻辑 7/7 passed，`cargo check -p hone-web-api` 通过。
- Web 类型检查、完整测试与 production build 通过。
- 本地后端 8077/8088、管理端 3000、用户端 3001 正常；运行时 sink 只有 `web`，event engine 与其他渠道未启动。
- 本地认证 API 实测：宏观 `live` 66.3（11/11），AI `partial` 72.6（四家 CSP 均有 6–7 项 SEC 财务指标）；8077/8088/3001 正常。
