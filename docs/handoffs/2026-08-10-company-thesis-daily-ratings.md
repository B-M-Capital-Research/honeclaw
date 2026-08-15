# 演讲公司投资逻辑 Skill 与每日评级榜交接

- status: done locally; not deployed or committed
- date: 2026-08-10
- plan: `docs/archive/plans/company-thesis-daily-ratings.md`
- skill: `skills/company-thesis-ratings/`

## Summary

51 份授权演讲逐字稿和 4 份研究工作簿已整理成 HONE 专业研究 Skill：47 份公司材料形成 52 张当前美国市场可交易公司/ADR/OTC 研究卡，量子、核电、跨年策略和答疑 4 份材料进入跨公司证据层。用户端对话上方新增可交互的每日公司评级榜，支持搜索、红黄绿筛选和六维解释。

## What Changed

- Skill 将回答拆成演讲确认逻辑、当前公开事实、AI 推断和未知项；要求“最新”问题先核验公司 IR、SEC/交易所、财报和带时间戳行情。
- 公司卡保留商业模式、护城河、产业链、估值方法、跟踪项、风险、证伪条件与内部来源名，不复制大段原文，不把历史目标价当当前结论。
- 原创六维评分为护城河 20%、稀缺性 20%、当前基本面 20%、订单执行 15%、估值 15%、市场确认 10%；绿灯 `>=75`，黄灯 `55–74.9`，红灯 `<55`。
- 后端 `GET /api/public/company-ratings` 仅对登录用户开放。北京时间 19:30 启动每日更新，使用现有 FMP key pool 批量取行情、并发取最近 5 季财务；原子写入 `data/company_ratings/daily.json`。
- 无 key/缺数时保留研究基线并降低置信度，明确显示 `transcript_only`/`partial`；上游失败保留最后成功结果并标 `stale`，从不以零值补缺。
- 前端入口只在认证会话 ready 后挂载；列表可搜索 ticker/公司/主题、按灯筛选、展开查看六维分、护城河、估值方法、风险与证伪，包含桌面、移动和暗色样式。
- 公共 API 不暴露逐字稿文件名；文件名只保留在内部 Skill 研究卡中。

## Verification

- `quick_validate.py skills/company-thesis-ratings`：通过。
- 51 个逐字稿文件均被公司卡或主题证据引用；0 个悬空来源。
- `cargo test -p hone-web-api --lib`：214 passed，2 ignored。
- `bun --cwd packages/app test`：412 passed（完整套件 411 + 新增样式契约 1 的最终口径）。
- `bun run typecheck` 与 public production build：通过。
- 本地源后端启动于 `8077/8088`，评级 worker 成功写出 52 项快照并计划下一次北京时间 19:30 更新；管理端/用户端 Vite 保持 `3000/3001`。
- 未登录请求按预期返回 `401`；登录页和认证挂载边界已在本地浏览器核验。因本机没有可用登录 session，未在真实认证 UI 中完成最终展开截图。

## Current Data Boundary

本机 `config.yaml` / effective config 未配置 FMP key，所以当前本地榜单会诚实显示“仅演讲研究基线 · 非当前行情评级”，行情和财报覆盖均为 `0/52`。配置现有 `fmp.api_key`/`api_keys` 后，下次启动或 19:30 任务会自动生成动态分；在此之前不能把本地分数描述成实时评级。

## Next Entry Point

先配置有效 FMP key，再重启源后端并用已登录用户打开 `http://127.0.0.1:3001/chat`：确认覆盖数、行情/财报日期、筛选、展开与移动布局。若部分新上市/OTC ticker 不受 FMP 套餐支持，保持 `partial`/低置信度，不新增猜测值；优先为该 ticker 接入合规的一手或授权数据源。
