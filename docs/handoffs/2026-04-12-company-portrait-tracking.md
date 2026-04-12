# 公司画像与长期基本面追踪

- title: 公司画像与长期基本面追踪
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `memory/src/company_profile.rs`
  - `crates/hone-tools/src/company_profile.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/components/company-profile-list.tsx`
- `packages/app/src/components/company-profile-detail.tsx`
- `packages/app/src/components/kb-stock-table.tsx`
- `skills/company_portrait/SKILL.md`
  - `skills/company_portrait/references/profile-framework.md`
  - `skills/company_portrait/references/event-template.md`
  - `skills/company_portrait/references/research-trail.md`
- related_docs:
  - `docs/archive/plans/company-portrait-tracking.md`
  - `docs/archive/plans/company-portrait-skill-framework.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
- related_prs:
  - N/A

## Summary

新增了以 Markdown 为真相源的公司画像系统：每家公司使用 `profile.md` 存长期画像，使用 `events/*.md` 记录财报、公告、管理层变化等时间线事件；同时接入了 `company_profile` 工具、Web API、记忆页画像视图，以及从知识记忆表打开画像的入口。当前页面层明确收口为只读展示，建档与更新统一经由 agent 完成。后续又补了一轮公司画像 skill 框架升级，让画像与事件更贴近专业投研档案：不仅保留结论，也保留 why / evidence / research trail。

## What Changed

- `memory/` 新增公司画像存储，支持：
  - 画像主文件生成与读取
  - 行业模板（`general / saas / semiconductor_hardware / consumer / industrial_defense / financials`）
  - 事件文件追加
  - section 定点回写
  - 追踪配置更新
  - 默认画像 section 现已补上 `Thesis`、`关键经营指标`、`风险台账`
  - 事件正文现已补上“为什么重要 / 证据与来源 / 本轮研究路径”
- `hone-tools` 新增 `company_profile` 工具，支持 `exists / create / get_profile / list_profiles / append_event / rewrite_sections / set_tracking`
- `hone-web-api` 新增 `/api/company-profiles*` 路由族与 capability
- Web 记忆页新增“公司画像”tab，支持：
  - 画像列表
  - 画像详情
  - 事件时间线展示
  - 追踪状态只读展示
  - 彻底删除画像
- 知识记忆表新增“打开画像 / 通过 agent 建立”入口提示
- prompt 层增加公司画像策略说明，并新增 `skills/company_portrait/SKILL.md`
- `company_portrait` skill 已改成更接近 Codex 的“轻主文档 + references”结构：
  - `SKILL.md` 只保留触发条件、当前实现边界与 workflow
  - `references/profile-framework.md` 说明主画像写法
  - `references/event-template.md` 说明事件模板
  - `references/research-trail.md` 说明如何保留本轮研究路径

## Verification

- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo test -p hone-tools company_profile -- --nocapture`
- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`
- `bun run --cwd packages/app typecheck`
- `bun run --cwd packages/app test`

## Risks / Follow-ups

- 当前前端只支持查看、时间线浏览与彻底删除；建档、追踪参数调整、section 回写与事件追加全部通过 agent/tool 路径完成
- 自动“检测到正在研究新公司时提示建档”当前主要依赖 prompt + skill 约束，若后续稳定性不足，可再补更显式的 runner / orchestration 规则
- 画像与 KB 的联动目前是入口级联动；若后续要做更强的自动追加，需要继续设计从 KB 分析结果到画像事件的自动映射
- 当前还没有独立 `research_notes` 存储层；为了不让 skill 超前于实现，研究路径暂时保留在事件文档里，后续若要做完整研究档案室，可再单独扩一层 research note

## Next Entry Point

- `memory/src/company_profile.rs`
- `crates/hone-tools/src/company_profile.rs`
- `packages/app/src/context/company-profiles.tsx`
