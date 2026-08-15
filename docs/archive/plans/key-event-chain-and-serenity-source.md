# 关键事件链与白毛速报数据源

- status: done
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex

## Goal

接入用户确认的 aichainmap 白毛速报公共数据，并实现 Rubin、HBM 两个主题的可追溯关键事件链。

## Completed Scope

- Serenity feed 固定 HTTPS 主机/路径、超时、大小上限、schema 与 X 原链身份校验；aichainmap 只标记为翻译/聚合层。
- 大V速报展示近 36 小时真实 Serenity 内容、原创/回复/引用类型和双层来源链接。
- 关键事件链每天 19:55 更新，读取近 30 天来源，以确定性关键词准入 Rubin/HBM，并区分无更新、来源失败、source-only、partial/live 与 stale。
- 第七个首页入口提供主题切换、时间线、影响、下一验证点、原链和保存后继续问答；不自动产生交易或仓位动作。

## Validation

Core 153/153；Web API 254/2 ignored；Web 436/436；TypeScript；production build；本地真实 feed 和 worker snapshot 通过。浏览器控制层阻止 localhost reload，未绕过；相关 UI 由组件/样式契约和构建覆盖。
