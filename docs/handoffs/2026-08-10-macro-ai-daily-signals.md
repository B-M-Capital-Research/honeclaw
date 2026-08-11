# 宏观红绿灯与 AI 红绿灯交接

- status: done locally; not committed or deployed
- date: 2026-08-10
- plan: `docs/archive/plans/macro-ai-daily-signals.md`

## Summary

本地 HONE 用户端对话上方已有“宏观红绿灯”和“AI 红绿灯”两个入口。入口只读取保存报告；20:00 北京时间 worker 负责独立生成和原子保存。两套报告共享可追溯证据、状态、历史、变化和报告上下文问答，但保持各自评分语义。

## Runtime Contract

- Latest：`data/daily_signals/macro/latest.json`、`data/daily_signals/ai/latest.json`。
- History：`data/daily_signals/{kind}/history/YYYY-MM-DD.json`。
- API：`GET /api/public/daily-signals/{kind}` 与 `GET /api/public/daily-signals/{kind}/history?limit=14`，均要求登录。
- 生成：每天 20:00 `Asia/Shanghai`，周末照常；启动时若当天没有非框架快照会补跑一次，仍不完整时每 15 分钟重试，成功后回到每日调度。
- 原子性：History 与 Latest 均通过同目录临时文件 rename；单进程锁和报告日去重阻止重复任务。
- 失败：记录具体数据源失败；没有有效证据时不计分；既有成功快照不会被空结果覆盖。

## Scoring Boundary

- 宏观健康分越高越好，原始风险分 0–10 越高越差。因果链为实际收入/工资 → 实际消费 → 制造业 → 利润/标普确认 → 实际 Capex；就业和 GDP 权重低且仅作滞后确认。
- AI 健康分阈值固定为绿 80–100、黄 60–79、红 0–59。FMP 可提供财报与行情；没有 FMP key 时，以 SEC EDGAR Company Facts 作为四大 CSP 标准财报底座。AI 收入、RPO、订单等没有统一字段时明确标未知，不从总公司数据偷换概念。
- 硬件链的动态分只叫“市场确认”，不冒充订单、供给或交期事实。

## 2026-08-10 Live-data Recovery

- FRED 多序列下载会返回 ZIP，不适合按 CSV 直接解析；改为并发拉取单序列 CSV，允许部分成功。
- FRED 防护层会挂起旧的 `HONE/0.15 daily-signals` User-Agent；改为带项目地址的标准 `honeclaw/0.15` 标识后恢复。
- AI 在 FMP key pool 为空时并发读取 SEC Company Facts；对同一财务期间配对收入、利润、经营现金流、Capex、流动性与债务，缺失值仍不当作零。
- 本地实测宏观为 `live`、11/11 维覆盖、66.3 分黄灯，截止 `2026-08-07`；AI 为 `partial`、72.6 分黄灯，四家 CSP 均有 6–7 项财务指标，截止 `2026-06-30`。硬件行情因未配置 FMP 继续明确为空。
- 登录后的两个 API 均返回 200；本地后端保持 8077/8088，用户端保持 3001，运行渠道仍只有 Web。

## Next Entry Point

刷新 `http://127.0.0.1:3001/chat` 即可读取已保存的 live/partial 快照。若后续配置 FMP key，AI 仪表盘会再补硬件行情市场确认；不要为了视觉完整度写死行情或专项 AI 指标。
