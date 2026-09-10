# 2026-09-06 行业分析页：按「带着问题看行业」重组阅读顺序，内容逐条带日期

- title: 行业分析页阅读顺序重组、行业简报与逐条截至日、线上改动折回底稿
- status: 代码完成，本地全部验证通过；后端未部署，前端未推送
- created_at: 2026-09-06
- updated_at: 2026-09-06
- owner: Claude（Fable 5.1）主会话；Rust 数据层由 codex（gpt-6-astra）实现、底稿标注由 grok（grok-4.6-build）实现，均经主会话逐行审查
- related_files: `packages/app/src/pages/public-industry-map.tsx`; `packages/app/src/pages/public-industry-map/{shared,brief,changes,company-lens,watch-sources,dossier,admin-editors}.tsx`; `packages/app/src/pages/public-industry-map.css`; `packages/app/src/lib/{industry-brief,industry-lens,industry-valuation,industry-map-navigation}.ts`; `packages/app/src/lib/types.ts`; `packages/app/e2e/public-industry-map.spec.ts`; `crates/hone-core/src/industry_map.rs`; `crates/hone-tools/src/industry_edit_tool.rs`; `crates/hone-web-api/src/routes/industry_map.rs`; `skills/industry-map/references/industry-map.json`
- related_docs: `docs/handoffs/2026-09-06-industry-ontology-v3-hone-forward-valuation.md`; `docs/handoffs/2026-09-03-industry-ontology-v2.md`; `docs/decisions.md#d-2026-09-06-01-date-every-piece-of-industry-tree-content`; `skills/industry-map/SKILL.md`
- related_prs: 直接落在 main 的一串独立 commit（`c3131f3d` 起）

## Summary

线上 `/industry-map` 内容很全，但打开先看到方法论与倍数规则，「这个行业最近变了什么、影响谁、下次看什么」埋在长文里；同时 AVGO 的公司介绍已是 FY26Q3、关注点与传导链变量仍写 FY26Q2，页头「研究底稿更新」是底稿的静态版本日期，读者误以为整页都刷新过。

这一轮把每个行业改成 **当前重点 → 最近变化 → 相关公司与影响（含公司视角）→ 接下来重点看什么 → 完整研究底稿（折叠）→ 研报与数据来源** 的顺序；每条关注点、传导链变量、简报都带自己的 `as_of`，行级 `content_as_of` 取代 `generated_at` 表示新鲜度；线上改动日志里的 4 条 AVGO / DELL 改动折回底稿并让重放幂等。

注意：在这轮进行中，`1f29f85c` 把行业本体改成了**仅管理员可见**（GET 对非管理员 403）。所以现在的「读者」就是管理员的只读态；页面重组对管理员同样成立，也是将来向读者放开前的先决条件。

## What Changed

### 数据层（Rust）

- `CoreWatch.as_of: String`（serde default，空串 = 未标注）；`Industry.brief: Option<IndustryBrief{question, body, next[], as_of}>`。
- 新 op：`set_watch { what, watch }`（按现有标题定位、整条替换，可改名）、`set_brief`、`clear_brief`；`AddSource` 按非空 url 幂等（`Ok` 不追加）、`AddWatch` 拒绝重复标题；`parse_as_of` 接受 `YYYY-MM-DD` 与 `YYYY-MM`；`Industry::content_as_of()` / `IndustryMap::content_as_of()` 取各带日期字段的最大值（同日日精度优先，原样写法返回）。`generated_at` 不动，语义改为底稿版本日期。
- 工具 `industry_map_edit`：`set_watch`（工具层合并缺省字段）、`set_brief`（`as_of` 缺省当天）、`clear_brief`；`show` 带 `content_as_of`。
- API：每行 `brief`、`content_as_of`；顶层 `content_as_of`。GET 的管理员门槛（`1f29f85c`）保持。
- 修了 `1f29f85c` 留下的半截测试：读者在管理员改动后再读仍是 403。

### 底稿内容（`industry-map.json`）

- 线上 `data/industry_map/edits.json` 的 4 条改动（AVGO / DELL 的 `role` 与各一条 source）逐字折回底稿；`SetMemberRole` 重放会覆盖底稿，所以这两段文本必须与日志逐字一致——`live_edits_are_folded_into_the_base_verbatim` 钉住。
- AVGO：`core_watch[3]`、`key_variables[5]` 的 FY26Q2 数字改为 FY26Q3 实际 + Q4 指引（数字只取线上日志 / 8-K 文本），`multiple_anchor` 里的 FCF / 资本开支改 Q3、调整后 EBITDA 明确标 FY26Q2。
- 8 个行业 103 条关注点与传导链变量里 96 条标了 `as_of`（= 该条 `why` 里数字所引用来源的日期；7 条拿不准的留空）。grok 的逐条判定表在会话记录里，其中的判断口径：`what` / `where` 里的数字不计入；多源取最新；「不要用 X 倒推」的对照数字不算引用。
- 8 条行业简报初稿：只用该行底稿已有的事实与日期改写（写入脚本逐个数字核对过），`as_of` = 该行最新来源日期。管理员可在页面编辑态或用 `set_brief` 覆盖。

### 前端

- 页面拆成入口 + `public-industry-map/` 七个模块；入口只持有状态与顺序。
- URL 拥有两个选择：`?industry=` 与 `?symbol=`（公司视角）。切行业清 symbol；不是本行成员的 symbol 规范化掉；找公司命中即进视角（replace）；公司表每行「查看」进视角（push）。**行业与公司要一次 `setSearchParams` 写入**：分两次时第二次与还没更新的旧参数合并，得到「旧行业 + 新公司」再被规范化掉——e2e 抓到过。
- 「当前重点」：有 `brief` 渲染简报（问题、正文、下一次先看 + 每步一个「问 HONE」）；没有则派生（最新两条变化 + 第一条关注点）并标「底稿自动归纳」。编辑态是简报表单（`set_brief` / `clear_brief`）。
- 「最近变化」：带日期的上游动作与来源合并按日期倒序、取前 5 条（多数行只有 NVDA 一条带日期动作，来源才是主要供给）；每条一个带上下文的「问 HONE」（`askSignalPrompt` / `askSourcePrompt`）。编辑态退回逐条上游信号编辑器。
- 公司视角卡：子类型、位置、市值现价、本页各块与它有关的条数（点击跳转）、「问 HONE」。相关条目前置并打「与 X 有关」标；匹配规则在 `lib/industry-lens.ts`（代码要求词边界；中文全名去掉「科技/半导体/…」后缀再匹配简称）。
- 「接下来重点看什么」：关注点带「数字截至」与「问 HONE」，长段「为什么看它」折起；编辑态整条改（含 `as_of`，走 `set_watch`）。
- 完整研究底稿：方法论、这类公司怎么估值、底层估值逻辑、传导链折进一个 `<details>`（不能用条件渲染：编辑器草稿依赖实例存活），带目录；编辑态默认展开；锚点跳转先展开再滚。
- 详情 h2 只放行业名（e2e 全等匹配），日期单独一行：「内容截至 {content_as_of，后端没带时前端自算} · 本行最近改动 / 本行自基线后未改动」；页头改「研究底稿基线」。
- `AsOfTag` 统一日期标签（>100 天标「可能已过期」），用于简报、变化、关注点、来源、变量表。

## Verification

- `cargo test -p hone-core --lib industry_map`：19 passed（基线 11）；`-p hone-tools --lib industry_edit_tool`：8（基线 7）；`-p hone-web-api --lib routes::industry_map`：11（基线 pull 后 10 passed 1 failed，修后 11）。`cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` 通过。
- `bun run typecheck:web` 通过；`bun run test:web`：626 passed（基线 594；新增 `industry-brief.test.ts`、`industry-lens.test.ts`、`public-industry-map-style-contract.test.ts` 与既有文件里的追加用例）。
- `bunx playwright test e2e/public-industry-map.spec.ts e2e/public-data-center.spec.ts --project=public`：15 passed（新 6 + 既有 9）。`playwright.config.ts` 两处正则已加 `industry-map`，否则新文件会跑在 admin 项目下。
- 底稿改动用校验脚本逐字段比对过（只允许加 `as_of` / `brief`、AVGO 三处改写、AVGO / DELL 折回）。

## Risks / Follow-ups

- **部署顺序**：后端按 `docs/runbooks/backend-deployment.md` 发 GHCR runtime（含底稿变更，底稿是编译进二进制的）；前端随 main 推送由 Cloudflare Pages 自动构建。前端先上也可以——`brief` / `as_of` / `content_as_of` 缺失时走派生与前端自算。上线后核对 `GET /api/public/industry-map`：`ai-chip.core_watch[3].as_of == "2026-09-02"`、每行 sources url 无重复、`recent_edits` 仍是那 4 条且 `edit_count == 4`、每行 `brief` 非空。
- 线上日志里那两条 `set_member_role` 会继续覆盖底稿的 AVGO / DELL `role`：以后要改这两段，走 `set_member_role` 而不是改底稿。
- 未做（有意延后）：brief 注入 `prompt.rs::industry_baseline`（会改模型输出，按 v3 惯例需 8 题复测）；`SetUpstreamLatest` / `AddUpstreamSignal` 的日期回溯校验；`previous_latest` / 每日快照 history；optical 与 hyperscaler 里 AVGO 上游 `latest` 仍为空，上线后用 `set_upstream_latest` 填；server-oem 关注点里的 Dell 数字仍是 FY27Q1（`as_of` 标的是旧日期，页面会显示），用 `set_watch` 更新。
- 简报的季度维护还是人工：接 v3 handoff 的「财报后自动起草、管理员确认」。
- 公司视角的文本匹配是启发式（`memberMentionTokens`）；若以后加 `member.aliases`，这是唯一改点。
- 页面是管理员专属，「读者」路径只在 3D 页 e2e 里覆盖被拒；向读者放开时要回头看首屏文案与「问 HONE」的默认对象。

## Next Entry Point

- 部署后端与推送前端 → 按上面的核对清单验收线上 → 用 `industry_map_edit(action="set_upstream_latest")` 补 optical / hyperscaler 的 AVGO、`set_watch` 更新 server-oem 的 Dell 关注点。
- 若要让模型也从简报起笔：`crates/hone-channels/src/prompt.rs::industry_baseline` 在 `let mut lines = vec![head];` 后加一行「当前研究重点（截至 …）：question；接下来要确认：next[..2]」，控制在 4,600 字预算内，跑 8 题复测。
