# 大V速报放开 + 研究台双视角 + 投资助手推荐问题与 influencer-views Skill

- title: 大V速报改滚动同步并对全部用户放出；研究台加说明、双视角与「问 HONE」；首页推荐问题；大V观点 Skill
- status: done
- created_at: 2026-09-06
- updated_at: 2026-09-06
- owner: Finn-Fengming
- related_files:
  - `crates/hone-web-api/src/routes/influencer_digest.rs`（滚动刷新、7 天窗口、分析缓存、作者失败沿用、`?limit=`）
  - `crates/hone-tools/src/influencer_views.rs`（新工具 `influencer_views`）、`crates/hone-tools/src/lib.rs`
  - `crates/hone-channels/src/core/bot_core.rs`（工具注册，所有用户）、`crates/hone-channels/src/prompt.rs`（领域约束加一条）
  - `crates/hone-core/src/tool_effect.rs`（只读工具名单）
  - `skills/influencer-views/SKILL.md`（新）、`skills/company-latest-developments/SKILL.md`（第 6 步）
  - `packages/app/src/pages/public-research.tsx` / `.css` / `public-research-style-contract.test.ts`
  - `packages/app/src/components/influencer-digest-dashboard.tsx` / `.css` / `.test.ts`
  - `packages/app/src/components/research/research-feed.tsx` / `.css`
  - `packages/app/src/lib/chat-empty-prompts.ts` / `.test.ts`、`packages/app/src/pages/chat.tsx`、`packages/app/src/lib/api.ts`、`types.ts`、`public-content.ts`
- related_docs: `docs/handoffs/2026-08-11-influencer-daily-brief-dashboard.md`、`docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md`

## 用户要求

> 大V速报跟进 aichainmap.com/serenity 要更快；现在只有管理员可见，可以放出来并做曝光；研究台的交互完全看不懂是干嘛的、
> 要怎么做，要加说明并做两个视角（管理员可切用户视角，用户看不到管理内容）；首页投资助手加推荐问题（AI 基础设施、
> Serenity 最新动态等）；速报内容全量重构：jukan 这类没数据的不显示，弹窗模拟 X 的样子、简单轻量一致；
> 速报内容做成投资助手的 skill，用户问个股新闻/消息时额外讲 Serenity 最近的看法。

## 改了什么

### 1. 速报后端：每日 19:50 一次 → 每 15 分钟滚动同步

- 刷新周期取 `event_engine.sources.rss_feeds` 里注册作者 feed 的最小 `interval_secs`（生产配置 Serenity 900s），
  夹在 5 分钟–6 小时之间；没有注册 feed 时默认 15 分钟。`next_refresh_at` = 现在 + 周期。
- 窗口从 36 小时改成 **7 天**，上限 24 → 80 条；快照新增 `refresh_interval_minutes`、`fresh_24h`、
  `latest_published_at(_local)`，作者状态新增 `carried_over`。旧快照无这些字段也能反序列化（serde default）。
- **分析缓存**：上一份快照里 `model_analyzed` 的条目按 id 复用（模型版本相同才复用），每轮只对新条目调模型；
  15 分钟一轮的成本是「新帖数」而不是「窗口条数」。
- **失败沿用**：某作者 feed 本轮失败，沿用上一份快照里该作者仍在窗口内的条目并标 `carried_over`；
  全部来源都失败时不覆盖旧快照。`stale` 改为「超过 3 小时没有新同步」。
- 概览卡摘要改成「作者 + 最新一条首行」（如 `Serenity 09-06 12:26：我不太明白OpenAI…`），
  指标为「24 小时内 N 条」；`source_only` 在研究台不再算「今日无新增」。
- `GET /api/public/influencer-digest?limit=N` 截断条目，聊天首页只取 3 条来拼推荐问题。
- 作者计数按截断后的条目算（否则芯片显示 89 条而列表 80 条）。

### 2. 投资助手：工具 + Skill

- 新只读工具 `influencer_views`（`hone-tools/src/influencer_views.rs`），对所有用户注册：读数据目录
  `influencer_digest/latest.json` + `history/<日期>.json`（合并去重），按 `symbol`（ticker 列表或 `$SYM` cashtag，
  ≥3 字母才允许裸词匹配）/ `query`（子串）/ `days`（默认 14，最多 30）/ `limit` 过滤，返回带作者、日期、原链、
  中文文本、HONE 观点整理（有则给）与窗口内 `focus`（ticker/话题计数）；没命中时 `note` 明确要求如实写没有。
  `tool_effect.rs` 已把它列入只读工具。
- 新 Skill `skills/influencer-views/SKILL.md`：别名覆盖 大V/白毛/Serenity/推特怎么看/最近有什么消息 等；
  取证顺序（symbol → 行业词 → 焦点）与输出纪律（独立段「大V观点线索」、逐条 日期·作者·观点·立场·未证实处·原链、
  与事实分层、不给买卖建议）。`company-latest-developments` 第 6 步指向它；`prompt.rs` 领域约束加一条
  「大V观点线索约束」让模型在个股新闻/消息类问题上加载它。
- 本地验证只到工具单测（4 条）与 Skill 文件被 `HONE_SKILLS_DIR` 加载；**没有跑真实对话**（本机无 LLM key）。

### 3. 研究台

- 分组为 全部 / 大盘信号 / 产业研究 / 大V观点 / 管理（管理员）；「产业研究」（3D 数据中心、行业分析）来自上游，
  本次新增「大V观点」组并把大V速报放进去，对所有用户开放。
- 管理员右上角「管理员视角 | 用户视角」切换（存 localStorage `hone.research.view`），用户视角 = 普通用户看到的页面
  （只剩宏观红绿灯与大V速报，管理面板、未发布板块全部隐藏）；非管理员没有这个开关。
- 顶部三步说明「研究台是什么、怎么用」（可折叠，折叠状态存 `hone.research.guide`）；每个板块加 `what`/`howTo`
  一句话，行尾统一「看详情」+「问 HONE」（带板块问题跳 `/chat?q=…&send=1`）。
- 三层结构（判断 / 要点 / 待更新）保留；行由 `div` + 主按钮 + 动作按钮组成，不再是按钮套按钮。

### 4. 速报面板（模拟 X 的时间线）

- 共享 `research-feed` 加了可选的头像列、类型标签、底部一行（阅读/赞 · 原链 · 动作按钮）和日期分隔
  `ResearchFeedDay`（sticky：今天 / 昨天 / MM-DD + 条数）。其它面板（事件链、社区、持仓新闻）不传这些 prop，外观不变。
- 速报面板：只显示 `configured` 的作者（Jukan 不再出现）；作者芯片 + 类型筛选（全部 / 只看原创 / 含图）一行；
  每条：头像「白」、名字 @handle、回复/引用标签、时间 HH:MM、引用卡、正文、配图、阅读/赞、查看作者原文、
  翻译/聚合源、「问 HONE」（带原话与日期跳到投资助手并直接发送）、折叠的 HONE 解读、English original。
- 说明行改为「最近同步 … · 每 15 分钟同步一次 · 近 7 天 · 中文译文来自 aichainmap 白毛速报，原文以 X 为准」。

### 5. 聊天首页与工具菜单

- 空会话推荐问题 5 → 7：新增「AI 基础设施这条链，现在最紧的是哪一环？」与「白毛 MM-DD HH:MM：<最新推文首句>」
  （有快照时引用最新一条并带 ticker；无快照时退化为「Serenity 最近在关注什么？」）；顺序
  macro / portfolio / calendar / influencer / ai-infra / industry / valuation。
- 工具菜单：大V速报从「管理」组移到「每日研究」组，对所有用户可见。

## 验证

- Rust：`cargo test -p hone-web-api --lib influencer_digest` 14 通过；`cargo test -p hone-tools influencer_views` 4 通过；
  `cargo test -p hone-core tool_effect` 2 通过；`cargo test -p hone-channels --lib prompt::tests` 通过；
  `cargo build -p hone-console-page -p hone-cli` 通过（仅既有 dead_code 警告）。
- Web：`bun test --preload ./happydom.ts ./src ./public` 545 通过（全部）；`bun run typecheck` 通过。
- 本地运行（worktree 二进制，`HONE_DATA_DIR` 独立数据目录，8177/8188 端口，前端 4178 代理）：
  启动即同步 Serenity feed（200 行 → 7 天内 92 → 过滤后 89 → 截断 80），`fresh_24h=1`，`next_refresh_secs=900`；
  研究台管理员/用户双视角、说明条、看详情/问 HONE、速报面板 80 条 + 7 个日期分隔 + 80 个「问 HONE」、
  芯片计数 80、聊天首页 7 张推荐卡（含白毛最新推文）、工具菜单每日研究组含大V速报——均在浏览器里核对。
  本机无 LLM key，速报状态为 `source_only`（仅原文），HONE 解读折叠不出现；生产配置了 pass2 模型会正常分析。

## rebase 到最新 main（2026-09-06 晚）

分支最初基于本地未推送的 `c3348391`；origin/main 当时已领先 23 个提交（聊天滚动四连修、社区 edge、3D 数据中心、
行业本体 V3 等），且 `c3348391` 的等价内容已由别的会话以 `e867eae3` / `05fdf675` 推上去。因此把本提交 rebase 到
`origin/main`，两处冲突：

- `packages/app/src/pages/public-research.tsx`：上游新增 `industry` 分组与 `data-center` 板块、并把「行业分析」从
  管理组移到产业研究组对所有人开放。解法是两边都要——`GroupKey` 变成 `signal | industry | voices | admin`，
  GROUPS 里产业研究与大V观点并存，`data-center` / `industry-map` 补上本次新增的 `what` / `howTo` 两个必填字段。
- `packages/app/src/pages/chat.tsx`：上游删掉了 `trackingOpenRequest` 信号及其唯一使用处，只保留本次新增的
  `workspaceInfluencer`。
- 契约测试 `releases the commentator digest…` 原本按「速报 → 行业分析」的顺序切片，rebase 后顺序反转导致空切片，
  改成切到「下一个 `key: "`」，与排序无关。

## 行业分析收回为仅管理员可见（同日，用户要求）

上游把「行业分析」从管理组移到新的「产业研究」组并对所有登录用户开放（`GET /api/public/industry-map`
只校验登录、`is_admin` 仅用于编辑栏，3D 数据中心还建了跳进去的公开链路与 e2e）。用户要求收回：本体是研究底稿
（传导链、倍数锚、上游信号、成员判断），不是已发布结论。改法是读写同一条门槛，而不是靠前端藏字段：

- `crates/hone-web-api/src/routes/industry_map.rs`：`handle_get_industry_map` 在拿到 `is_admin` 后，
  非管理员一律 403「行业分析仅管理员可见」，内容根本不出服务端。原单测
  `industry_map_read_is_session_only_and_edits_remain_admin_only` 改名并把「读者拿到 200」翻成 403。
- 研究台条目加 `adminOnly: true`（仍留在「产业研究」组，管理员在该层也能看到「未发布」标记）。
- `public-data-center.tsx`：3D 场景保持公开，但它通往本体的两处入口（页头「完整行业分析」与浮窗里的行业链接）
  只对管理员渲染——公开页面不该把读者送到一扇回 403 的门。
- `public-industry-map.tsx`：已有的 `forbidden` 视图文案改成「行业分析仅管理员可见，当前账号没有查看权限。」
- `packages/app/e2e/public-data-center.spec.ts`：mock 与服务端一致（非管理员读本体返回 403）；
  原「普通读者跳进行业分析」一测改成「读者拿得到 3D、拿不到本体」，原来的导航与历史回退流程移到管理员用例。

**验证**：本地后端（8188）实测同一会话——管理员 200 且 8 个行业，撤销管理员后同一 cookie 403
（`{"error":"行业分析仅管理员可见"}`），恢复后又 200。浏览器里以真实非管理员账号确认：3D 页面 0 个
`/industry-map` 链接、六个热点仍可用；直接输 URL 得到「仅管理员可见」；研究台「产业研究」只剩 3D 数据中心，
既没有视角切换也没有管理分组。`industry_map` 的那条后端单测需要 PostgreSQL 运行时配置，本地与干净 origin/main
同样报 `PostgreSQL must be configured for the runtime`，因此该断言只在带 PG 的环境（CI）里真正执行；
上面的 curl 实测是它的替代证据。

## 风险与未决

- 生产 feed 每轮 15 分钟拉一次 aichainmap 的公开 JSON（约 340KB），比之前每天一次多出 ~96 次/天；
  feed 方若限流，作者会进入 `carried_over`，面板芯片提示「沿用上次」。
- 第一次上线后的第一轮会对窗口内所有未分析条目调模型（最多 80 条 / 10 条一批 = 8 次调用），之后每轮只分析新帖。
- 研究台把 `source_only` 从「今日无新增」里拿掉，影响所有板块的分层判断（事件链 `source_only` 也会显示为要点）。
- `influencer_views` 读取的是 web 进程写的数据目录；渠道进程（飞书等）与 web 进程必须共用同一个 `storage.data_root`，
  生产是同一台机器同一份 config，本地也是。
- 未做：Jukan 合法 bridge；面板里的「问 HONE」在飞书等渠道没有对应入口。
- 「行业分析」收回后，3D 数据中心对普通用户成了没有下一跳的展示页（浮窗里「继续看完整行业分析」整块不再渲染）。
  如果后续希望读者也有去处，应另给一个面向读者的落点，而不是把本体重新放开。
- 本次只收回「行业分析」；3D 数据中心仍对所有用户开放（上游本轮刚发布的）。要一并收回的话，
  给 `data-center` 条目加 `adminOnly: true` 即可。

## 分支与落地

- 改动在独立 worktree `/Users/bytedance/Desktop/honeclaw-worktrees/wt-kol`、分支 `feat/influencer-digest-release`
  （主 checkout 仍有另一会话 2026-09-02 的未提交改动，未动）。教训：scratchpad 目录在会话恢复时会被清空，
  worktree 不要放在 scratchpad。
- 合并后需要重建 `hone-console-page` 与 `hone-cli` 两个二进制并发版；速报数据目录无需迁移（旧快照可读）。
