# Plan

- title: Chart Visualization Skill 与多通道 PNG 投递
- status: in_progress
- created_at: 2026-04-17 14:10 CST
- updated_at: 2026-04-17 20:24 CST
- owner: Codex
- related_files:
  - `skills/chart_visualization/SKILL.md`
  - `skills/chart_visualization/scripts/render_chart.py`
  - `skills/market_analysis/SKILL.md`
  - `skills/stock_research/SKILL.md`
  - `skills/gold-analysis/SKILL.md`
  - `crates/hone-tools/src/skill_tool.rs`
  - `crates/hone-channels/src/outbound.rs`
  - `crates/hone-web-api/src/routes/history.rs`
  - `bins/hone-feishu/src/{client.rs,outbound.rs}`
  - `bins/hone-telegram/src/listener.rs`
  - `bins/hone-discord/src/utils.rs`
  - `packages/app/src/lib/{messages.ts,messages.test.ts}`
  - `tests/regression/manual/test_chart_visualization_{web,feishu,telegram,discord}.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/decisions.md`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-04-17-chart-visualization-skill.md`

## Goal

给 Hone 新增一个可自动发现的 `chart_visualization` skill，用 Python `matplotlib` 渲染 PNG 图表，并把“skill artifact -> assistant 可见本地图片 marker -> Web 内联渲染 / 外部通道真实图片发送”串成一条稳定契约。

## Scope

- 新增 `skills/chart_visualization/`，定义 v1 图表 spec、失败回退与 `file:///abs/path.png` 输出约束
- 扩展 `crates/hone-tools/src/skill_tool.rs`，让 `execute_script=true` 能解析结构化 JSON stdout、暴露 `artifacts`，并校验 artifact 路径与图片扩展名
- 在 `crates/hone-channels/src/outbound.rs` 增加共享分段解析，把助手最终文本拆成有序的 `text` / `local-image` 片段
- 让 Web 历史解析识别 inline `file://` 图片 marker，保留 Web 现有内联渲染能力
- 让 Feishu / Telegram / Discord 出站去掉原始本地路径，改为按顺序上传发送真实图片
- 补自动化测试、手工回归脚本与文档同步

## Validation

- `cargo test -p hone-tools`
- `cargo test -p hone-channels`
- `cargo test -p hone-web-api`
- `cargo test -p hone-feishu`
- `cargo test -p hone-telegram`
- `cargo test -p hone-discord`
- `bun run test:web`
- 定向手工回归：
  - Web 趋势问答能内联显示 PNG，且历史附件提取能识别 inline `file://` marker
  - Feishu / Telegram / Discord 能把 text-image-text 顺序投递成真实图片与文本，而不是把本地路径发出去

## Current Progress

- 已完成：
  - 新增 `chart_visualization` skill 与 Python 渲染器，支持 `line / area / bar / scatter / histogram / horizontal_bar`
  - `skill_tool` 已支持结构化 `stdout` 解析、artifact 暴露与允许根目录校验
  - 共享 response segment parser 已落到 `hone-channels`，Web 历史提取也已识别 inline 本地图片 marker
  - Telegram / Discord / Feishu 出站已改为按段发送文本和图片，并在图片发送失败时回退成不泄露本地路径的文本说明
  - 已根据真实 Telegram 坏样本补强 parser：现在除了裸 `file:///...png`，也能识别 HTML anchor 和 Markdown link 里的本地图片 marker，避免路径残片漏发到外部通道
  - 已继续补强 parser 与会话恢复：`file:///...png<br>` 这种紧跟 HTML 标签的本地图片 marker 现在也会被正确识别；历史恢复与 session compact 会把旧本地图片路径折叠成“上文包含图表”占位，避免模型跨轮复述失效临时路径
  - assistant 最终回答里若引用了已经失效的本地图片文件，会降级成不泄露绝对路径的文本提示，而不是把坏掉的 `file://` 路径继续存进会话和页面
  - 已补 renderer smoke test、artifact 校验测试、response segment 测试、Web message/history parsing 测试
- 待验证：
  - 真实 Feishu / Telegram / Discord 账号链路图片上传发送
  - 真实 Web 聊天页面里从模型回答到历史回放的完整体验

## Documentation Sync

- 更新 `docs/current-plan.md` 活跃任务索引
- 在 `docs/decisions.md` 记录助手本地图片 marker 与通道出站契约
- 在 `docs/repo-map.md` 记录 `chart_visualization` skill、`skill_tool` artifact 能力与跨通道本地图表发送路径
- 新增 `docs/handoffs/2026-04-17-chart-visualization-skill.md` 记录当前实现状态、手工回归步骤与未完成验证
- 在真实外部链路验证完成前，保留本计划页为 `in_progress`

## Risks / Open Questions

- v1 强依赖 `python3 + matplotlib`；运行时缺依赖时只保证结构化失败并回退到纯文本回答
- Feishu / Telegram / Discord 的真实图片发送仍依赖外部账号、bot 权限与本地文件可读性，本轮只能在代码层保证不再把 `file://` 原样发到外部
- 当前 Web 仍依赖最终 assistant 文本中的 `file://` marker 渲染图片，后续若引入独立媒体事件，需要再评估是否保留文本内 marker 作为长期兼容层
- 真实 Telegram / Feishu / Discord 监听进程在代码修复后仍需要重启到新二进制；`probe` 通过不代表常驻 bot 已自动吃到改动
