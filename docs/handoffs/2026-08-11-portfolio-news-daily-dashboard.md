# 持仓重点新闻分析每日仪表盘交接

日期：2026-08-11
状态：本地完成，未提交、未部署生产

## 已完成

- 在 authenticated chat 首页增加第 4 个 Button“持仓重点新闻分析”，入口位于每日公司评级之后。
- 后端每天北京时间 20:00 遍历真实 actor 持仓，排除自选，期权映射到底层标的，并对相同底层的股票/期权权重做本地合并。
- 复用现有 FMP NewsPoller 读取近 48 小时新闻；仅保留精确持仓代码且通过可信来源或重要性门槛的事件，过滤观点博客、法律广告和电话会 transcript 噪声。
- 复用 global digest 的已配置模型与 profile。模型只接收新闻事实，不接收 actor、持仓权重、股数或成本；返回值经过事件 ID 与枚举白名单校验。
- 为每个 actor 原子保存 `latest.json` 和按报告日归档的 history。页面只读取缓存，不因打开弹窗触发生成。
- 页面显示来源、发布时间、影响、期限、投资逻辑影响、关注动作、置信度和原文链接，并可把已保存报告发送到正常对话继续追问。
- 无持仓、数据源未配置/失败、无重点新闻、模型未配置/失败、持仓已变更、等待首次刷新和快照过期都有独立状态；缺失分析不会补造结论。

## 关键边界

- actor 隔离由认证会话、`PortfolioStorage` 与 `actor.storage_key()` 共同保证。
- 仓位权重只用于 HONE 后端排序，未发送给模型；当前功能不会修改仓位，也不会自动输出买卖指令。
- 持仓更新晚于新闻快照时，GET 会隐藏旧 items 并返回 `portfolio_changed`，等待下一次任务刷新。
- 本机测试账号当前没有真实持仓，因此浏览器验收应显示“尚无持仓 / 先添加持仓”，不能填充样例新闻冒充当天数据。

## 验证

- `cargo test -p hone-web-api portfolio_news --lib`: 6 passed。
- `cargo test -p hone-web-api --lib --no-fail-fast`: 235 passed，2 ignored。
- `bun run test`: 425 passed。
- `bun run typecheck`: passed。
- `bun run build`: passed（保留既有大 chunk warning）。
- authenticated local browser: 第 4 个入口、弹窗、no-portfolio 状态和退出交互通过。

## 后续

- 生产需要配置现有 FMP key pool 与 global digest model/profile，否则分别进入 `data_unavailable` 或 `source_only`。
- 如需实时重大事件提醒，应在现有每日快照之外另立计划；不要把轮询频率调整混入本功能。
- 按产品顺序的下一项是“仓位管理”，本轮未实现。
