- title: 演讲公司投资逻辑 Skill 与每日评级榜
- status: done
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files:
  - skills/company-thesis-ratings/
  - crates/hone-web-api/src/routes/company_ratings.rs
  - packages/app/src/components/company-rating-dashboard.tsx
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/handoffs/2026-08-10-company-thesis-daily-ratings.md
  - docs/invariants.md

## Goal

把演讲逐字稿中的公司基本面、估值、护城河、反方与证伪条件沉淀为 HONE 可调用的专业 Skill，并在用户端对话上方提供每日公司评级榜。评级在北京时间 19:30 生成，20:00 前可见；每个分数必须能追溯到演讲逻辑、最新数据时间与来源，不能冒充 Seeking Alpha 专有评级或投资建议。

## Delivered

- 51/51 逐字稿完成来源覆盖；52 张美国市场可交易公司卡，4 份主题/策略材料进入证据层。
- 可维护 Skill、研究更新规范、原创六维评分方法和工作簿生成脚本完成。
- 认证只读 API、原子快照、19:30 Asia/Shanghai worker、FMP 行情/财务 enrichment、partial/stale/transcript-only 回退完成。
- 对话上方入口、红黄绿列表、搜索筛选、六维详情、数据覆盖/更新时间、桌面/移动/暗色布局完成。

## Validation

- Skill 校验通过，逐字稿来源 0 缺失。
- Web API 214 passed / 2 ignored；Web 412 passed；TypeScript 与 production build 通过。
- 本地 `8077/8088/3000/3001` 运行正常，快照包含 52 项。
- 本机无 FMP key，当前运行结果正确降级为 `transcript_only`；这是数据配置边界，不是静默伪实时。

## Documentation Sync

- `docs/current-plan.md` 活跃项移除。
- `skills/README.md` 增加内部研究 Skill 边界。
- 完成交接 `docs/handoffs/2026-08-10-company-thesis-daily-ratings.md` 与归档索引。
