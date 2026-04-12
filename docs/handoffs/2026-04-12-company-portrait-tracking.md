# 公司画像与长期基本面追踪

- title: 公司画像与长期基本面追踪
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `memory/src/company_profile.rs`
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/sandbox.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/components/company-profile-list.tsx`
  - `packages/app/src/components/company-profile-detail.tsx`
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

新增了以 Markdown 为真相源的公司画像系统：每家公司使用 `profile.md` 存长期画像，使用 `events/*.md` 记录财报、公告、管理层变化等时间线事件，并通过记忆页只读画像视图对外展示。随后又按“每个用户 x 渠道独立空间”的方向继续收口：公司画像不再依赖专用 mutation tool 或公共目录，而是直接落到 actor sandbox 下的 `company_profiles/`，由 agent 用 runner 原生文件读写能力维护；后台只负责按 actor 扫描、展示和删除。最新一轮又进一步去掉了 KB 页面、知识记忆 tab、KB API 和 `kb_search` 暴露，让长期研究记忆只剩按 actor 隔离的公司画像。画像与事件也已补足 why / evidence / research trail，更接近专业投研档案。

## What Changed

- `memory/` 新增公司画像存储，支持：
  - 画像主文件生成与读取
  - 行业模板（`general / saas / semiconductor_hardware / consumer / industrial_defense / financials`）
  - 事件文件追加
  - section 定点回写
  - 追踪配置更新
  - 默认画像 section 现已补上 `Thesis`、`关键经营指标`、`风险台账`
  - 事件正文现已补上“为什么重要 / 证据与来源 / 本轮研究路径”
- 公司画像现在落在 actor sandbox 的 `company_profiles/`，目录按 `channel/<scope__user>/company_profiles/<profile_id>/...` 组织
- 不再向 agent 暴露专用 `company_profile` mutation tool，也不再提供画像创建 / 追加 / 回写的公共 API；统一改为 agent 在当前 actor 用户空间里直接读写 Markdown
- `hone-web-api` 新增 `/api/company-profiles*` 路由族与 capability
- Web 记忆页新增“公司画像”tab，支持：
  - 画像空间列表（按 actor）
  - 画像列表
  - 画像详情
  - 事件时间线展示
  - 追踪状态只读展示
  - 彻底删除画像
- KB 页面、知识记忆 tab、KB API、`kb_search` tool 和 KB skills 已移除；附件 ingest 到 KB 的后台入口也已停止生成新的 KB 记忆
- prompt 层增加公司画像策略说明，并新增 `skills/company_portrait/SKILL.md`
- `company_portrait` skill 已改成更接近 Codex 的“轻主文档 + references”结构：
  - `SKILL.md` 只保留触发条件、当前实现边界与 workflow
  - `references/profile-framework.md` 说明主画像写法
  - `references/event-template.md` 说明事件模板
  - `references/research-trail.md` 说明如何保留本轮研究路径

## Verification

- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`
- `bun run --cwd packages/app typecheck`

## Risks / Follow-ups

- 当前前端只支持按 actor 查看、时间线浏览与彻底删除；建档、section 回写与事件追加全部通过 agent 在 actor sandbox 里的原生文件操作完成
- 自动“检测到正在研究新公司时提示建档”当前主要依赖 prompt + skill 约束，若后续稳定性不足，可再补更显式的 runner / orchestration 规则
- 旧 `memory/src/kb.rs` 与附件相关底层代码暂未物理删除，但用户侧和 agent 侧已经不再暴露 KB 能力；若未来确认彻底不用，可再开一轮删除残留内部实现
- 当前还没有独立 `research_notes` 存储层；为了不让 skill 超前于实现，研究路径暂时保留在事件文档里，后续若要做完整研究档案室，可再单独扩一层 research note

## Next Entry Point

- `memory/src/company_profile.rs`
- `crates/hone-channels/src/{sandbox.rs,core.rs,prompt.rs}`
- `packages/app/src/context/company-profiles.tsx`
