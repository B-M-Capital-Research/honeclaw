# 大V每日速报仪表盘

- status: done
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex

## Goal

实现首页“大V速报”入口：每天北京时间 19:50 读取已确认来源的近 36 小时公开内容，覆盖 Serenity/白毛、Jukan、SemiAnalysis，保留原文、作者、发布时间和数据状态，并在严格边界内生成观点摘要、方向、期限、相关标的、事实/观点区分和反方提醒。

## Completed Scope

- 内置作者身份只用于展示与精确配置匹配，不根据相似名称猜账号。
- 复用 `event_engine.sources.rss_feeds`；SemiAnalysis 使用官方 feed，Serenity 使用用户确认的 aichainmap 公共整理 feed 但只接受精确 X 原链，Jukan 没有合法 bridge 时明确显示未配置。
- 模型只处理抓取到的公开摘要；ID、枚举和 ticker 都经过确定性校验，模型不可用时只展示来源事实。
- 全局快照只读，UI 提供作者筛选、原文链接、边界标签和发送到对话，不自动产生买卖或仓位动作。

## Validation

Core 153/153；Web API 248/2 ignored；Web 433/433；TypeScript；production build；本地 Web-only worker 与 authenticated mobile browser acceptance 均通过。
