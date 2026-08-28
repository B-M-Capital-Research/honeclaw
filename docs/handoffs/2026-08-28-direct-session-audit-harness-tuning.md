# Direct 会话审计与分场景 harness 调整（2026-08-28）

- title: 生产 direct（用户主动）会话正确性审计 + SKILL/TOOL 描述分场景优化
- status: done
- created_at: 2026-08-28
- updated_at: 2026-08-28
- owner: ecohnoch
- related_files:
  - `skills/stock_research/SKILL.md`
  - `skills/scheduled_task/SKILL.md`
  - `skills/market_analysis/SKILL.md`
  - `skills/portfolio_management/SKILL.md`
  - `skills/valuation-audit/SKILL.md`
  - `skills/options-analysis/SKILL.md`
  - `skills/chart_visualization/SKILL.md`
  - `skills/image_understanding/SKILL.md`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-tools/src/portfolio_tool.rs`
  - `crates/hone-tools/src/cron_job_tool.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `crates/hone-tools/src/local_files.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runners/tool_reasoning.rs`
- related_docs: `docs/bugs/README.md`、`docs/runbooks/backend-deployment.md`

## Summary

从生产 PostgreSQL 拉取近 7 天（2026-08-21 ~ 08-28）全部 64 个 `__direct__` 会话，
配合生产 journald 的 `runner.tool` 日志建立**逐轮工具调用索引**，对用户主动轮做正确性审计，
再把确认的缺陷落到 SKILL / TOOL 描述层。

样本：64 会话、**152 个真实用户主动轮**（已剔除 `[定时任务触发]` 触发消息、
`【Invoked Skill Context】` 注入消息和推送消息）；web 104 轮、飞书 48 轮；152 轮全部得到了回复，无掉答。

确定性指标（regex + 工具日志，不依赖模型判断）：

| 指标 | 数量 | 占用户轮 |
| --- | --- | --- |
| 整轮没有调用任何取证工具（data_fetch / web_search / portfolio） | 40 | 26.3% |
| ↳ 其中该轮工具调用里包含加载生产上并不存在的 `hari-invest` / `company-thesis-ratings`（其余只有本地文件读取） | 17 | 11.2% |
| 无取证工具却用「行情口径：…最新价 $X」把答案包装成已核验 | 23 | 15.1% |
| 用户轮直接吃到通用失败兜底话术 | 5 | 3.3% |
| 首行打出内部占位符（`数据时间：运行时时区` / `Asia/Shanghai`） | 111 / 260 条带首行的回答 | 43% |

模型审阅：3 波共 187 条缺陷候选，经 10 个对抗性核验 agent 判定 98 条，
其中 CONFIRMED 56、PARTIAL 19、REFUTED 23；缺口类型 BURIED 41、MISSING 25、CONFLICT 17、NOT_HARNESS 14。
「规则已存在但没下沉到调用点」（BURIED）是最大类，因此本轮改动一律往
**工具描述 > 场景 SKILL.md > prompt.rs 策略常量**方向放，不再往已 3 万字符的 `soul.md` 里加规则。

## What Changed

**根因层（最重要，非本次代码改动可解决）**：生产 `/srv/honeclaw/skills` 缺少
`hari-invest` 与 `company-thesis-ratings` 两个 skill，而 `prompt.rs:DEFAULT_HARI_INVEST_POLICY`
每轮强制要求加载它们。`/api/skills/hari-invest` 返回 `{"error":"skill not found"}`，
近两天日志里 `skill_tool hari-invest` 690 次、`company-thesis-ratings` 300 次全部落在失败分支。
典型链路（`web-user-eda6b969ee27` [5][9][11]、`web-user-c4268f42d3d0` [1][3]）：
模型按指令先加载 skill → 加载失败 → 把这次失败当成「本轮已经取过证」→ 直接凭记忆写出带
「行情口径」抬头的答案。**修复动作是同步 skill 到生产，不是再写提示词。**

**SKILL 层**

- `stock_research`：新增 “Evidence Floor” 一节——首行是对本轮工具结果的声明；本轮没有该 symbol 的
  quote/snapshot/extended_hours 时，首行只能写「本轮未取到行情」，正文禁止出现任何价格/涨跌幅/市值/倍数/价位区间；
  明确 skill 加载失败、读本地文件、查技能索引都不算取到行情；「本轮已核验」「本轮检索到 + URL」
  「据 SEC 原文」三类出处声明各自需要对应工具；跨轮复用旧报价只能标原始日期。
- `scheduled_task`：与 `cron_job` 工具契约对齐——`remove` 需两步且必须 `confirm="yes"`；补 `remove_all`、
  `repeat="heartbeat"`、`bypass_quiet_hours`；删除失实的「每人最多 5 个任务」（代码无此限制，线上最多 18 个）；
  修正 `portfolio(action="get")` → `view`（工具 enum 无 `get`）；新增建任务前实跑取数可行性、
  `task_prompt` 里的 ticker 必须先 search 解析并回显、不得向用户复述参数字面。
- `market_analysis`：宏观指标的读数/均值/极值必须同指标同来源同单位；均线数值全文唯一，
  写「跌破/站上」前先做减法。
- `portfolio_management`：用户本轮粘贴的明细优先于系统存储；定性前先比现价与成本（表里浮亏正文不得称盈利）；
  估值倍数/无风险收益率等精确数字必须来自本轮 `valuation`/`macro`。
- `valuation-audit`：跨轮估值锚一致性（变了要写「上轮 X → 本轮 Y，因为 Z」）；
  对账表「来源」列改用人话，禁止抄 `balance_sheet_quarter`/`hone_ttm` 这类字段名（MRVL 一轮已外泄）。
- `options-analysis`：期权盈亏统一按「每股权利金 × 100 × 张数」列式，禁止把总额当每股价再乘一次
  （HIMZ 一例把 $352 算成 $48，低估 7 倍并据此建议平仓）。
- `chart_visualization`：渲染器不支持 pie，饼图请求改用 `horizontal_bar` 并说明；
  任何情况下不得对用户说 Hone 不能出图，也不得用 mermaid 或字符条代替。
- `image_understanding`：提取块为空时先按附件策略实际尝试读取路径；回复不得出现技能名、
  `【图片文字提取】`、runner、本地路径、文件类型或失败原因。

**TOOL 描述层**

- `data_fetch`：新增「本工具的输出怎么写给用户看」四条——内部词不外泄；盘前/盘后标签必须有
  `extended_hours` bar 支撑，涨跌幅逐字采用 `hone_change_basis.label`；倍数必须写清分母口径且
  市值 ÷ 分母 = 所报倍数，`hone_forward` 的科目名与预期窗口不得改写；不得用母公司/关联公司行情顶替用户点名的标的。
- `portfolio`：只有本轮 `view` 返回能作为持仓依据（没调 view 不得断言「你没有 X」）；
  写操作前必须先 `data_fetch(search)` 解析并回显「闪迪 → SanDisk (SNDK)」。
- `cron_job`：字段名与取值只用于调用不用于对话；add 前先实跑取数路径。
- `web_search`：来源标注只写域名与日期；核验类请求没检索到只能写「本轮未取得证据」，
  不得下「官方从未发布」这类否定性存在断言。
- `local_list_files` / `local_search_files` / `local_read_file`：明确返回的是历史快照，
  其中的价格与财报数字不得进入「行情口径」行或充当最新数据。

**时钟标签：用户不再看到「运行时时区」**（本轮回答里 43% 的首行受影响）

- `investment_response_guard.rs` 新增 `data_time_prefix()` / `data_time_prefixes()` /
  `matched_data_time_prefix()`：所有服务端自写的首行、报价源时间、兜底口径行改用已有的
  `local_clock_label()`（Asia/Shanghai → 「北京时间」），而首行校验与流式前缀识别同时接受
  新旧两种写法，历史会话与旧草稿不会因此被判失格。
- `runtime.rs` 新增 `normalize_user_visible_clock_label()`，在两条出站清洗链路上都生效
  （含 agent-owned 那条"不做语义改写"的链路——这只改时钟怎么写，不改答案主张什么）：
  `运行时时区=Asia/Shanghai`、`数据时间：Asia/Shanghai`、以及任何残留的 `运行时时区`
  统一改写成人类读法。配额提示这类非投研文案同样受益。
- `agent_session/core.rs` 的 Web 服务端首行不再写 `行情口径：运行时时区=<IANA>`。
- 提示词侧同步：`prompt.rs` 的首行模板与 `investment_response_guard.rs` 的回答契约都改成
  「数据时间：北京时间 …」，并明写禁止把 `运行时时区`、`Asia/Shanghai`、`provider timestamp` 写给用户。

**prompt.rs（每轮注入的策略文本，非逻辑）**

- `DEFAULT_FINANCE_DOMAIN_POLICY` 增加「首行的无行情分支」与「三类出处声明各需对应工具」两条。
- `attachments/ingest.rs` 的「其他文件」默认策略行改为先实际尝试读取，失败时用用户语言收口，
  不再要求模型向用户罗列可处理范围。

## Verification

- `cargo check -p hone-tools -p hone-channels` 通过。
- `cargo test -p hone-channels prompt::` 21 passed（需 `DATABASE_URL=postgres://…/honeclaw_test`）。
- `cargo test -p hone-channels attachments` 28 passed。
- `cargo test -p hone-tools -- --test-threads=1`：197 passed / 5 failed，
  与干净树（stash 后）完全一致的同 5 条 `skill_tool::tests::*` 既有漂移失败，本次改动零新增失败。
- 生产事实核验：`/api/skills/hari-invest` 与 `/api/skills/company-thesis-ratings` 均返回 skill not found；
  `sudo find /srv/honeclaw/skills` 无这两个目录；`cron_job` 的 `remove` 分支
  （`crates/hone-tools/src/cron_job_tool.rs:423`）确认 `confirm != "yes"` 即返回 `success:false`；
  `portfolio` action enum（`portfolio_tool.rs:44`）确认无 `get`；`cloud_cron_jobs` 单用户最多 18 个任务。

## Deployment（2026-08-28，已上线）

修订 `a43f99c8dce9cefb4d439eb715e5b9224b0b8793`。

**harness 层**（skills + soul.md，不随镜像发布，按 runbook 单独 stage）：本地从该修订
`git archive` 出 `skills/` + `soul.md`，生成 53 条 SHA-256 manifest，上传后在主机
`/srv/honeclaw/.harness-stage-<rev>` 校验（manifest 通过、无符号链接），逐目录与线上比对哈希，
只替换有差异的 11 个 skill 与 `soul.md`，先 `cp -a` 备份到
`/srv/honeclaw/skills/backups/pre-a43f99c8-20260828T104304Z/` 再 `mv -Tf` 原子换入。
替换的是：`chart_visualization`、`company-thesis-ratings`(新增)、`hari-invest`(新增)、
`image_understanding`、`market_analysis`、`notification_preferences`、`options-analysis`、
`portfolio_management`、`scheduled_task`、`stock_research`、`valuation-audit`。
无 LIVE-ONLY skill 被动到。

**镜像层**：Actions `runtime-image.yml` 出 `a43f99c8…`，主机 `crane digest` 得
`sha256:24c9fa1b423b289567a7afedb5bdd6f5781b581076e87044ee3769c6b91cb1d4`，
`stage_ghcr_runtime.sh` 校验通过后 `previous` → 1e7cfc15、`current` → a43f99c8 原子切换，
`systemctl restart hone-web`。

**上线后核验**：
- `/api/skills/hari-invest` 与 `/api/skills/company-thesis-ratings` 由 `{"error":"skill not found"}`
  变为正常返回（hari-invest markdown 2235 字符）；系统 skill 从 20 增至 22，总数 33。
- `/srv/honeclaw/skills/stock_research/SKILL.md` 含 `Evidence Floor`；
  `scheduled_task` 含两处 `confirm="yes"`，失实的「最多 5 个任务」已消失。
- `/api/meta`：`cloud_mode=cloud`、PG connected、OSS connected、`local_durable_dependency_count=0`。
- `/api/runtime/active-chat-runs` 切换前两次读、切换前一次读、重启后一次读均为 `{"count":0}`。
- 公开端 `127.0.0.1:8088/api/public/auth/me` → `401`（未认证预期值）。
- 运行中可执行文件解析到 `…/a43f99c8…-ghcr-runtime/bin/hone-cli`，`RELEASE_METADATA.git_sha` 一致，
  二进制内含「数据时间：北京时间」。
- release 保留 current + previous + 一个备用；根盘剩余 4.9G（≥ 2GiB 门槛）。
- `hone-channel@feishu` 仍为 disabled/inactive —— 与本次改动无关，是既有运营状态。

## Risks / Follow-ups

1. 主机 `/root/.docker/config.json` 常驻着 GHCR 凭据（0600 root，2026-08-05 起），
   本次 `crane digest` / stage 依赖它。`docs/runbooks/backend-deployment.md` 明确要求
   凭据只用临时 `DOCKER_CONFIG` 目录、用完即删、不得留在主机上。这是既有偏差，
   建议改为一次性 `read:packages` 凭据并在部署后清理。
2. `soul.md` 已 3 万字符 + `DEFAULT_FINANCE_DOMAIN_POLICY` 二十余条 bullet 每轮全量注入，
   规则过载本身是 BURIED 类缺陷的成因。后续新增纪律优先下沉到场景 skill 与工具描述。
3. 生产自定义 skill（`/srv/honeclaw/data/custom_skills/`，11 个）不在仓库版本管理内，
   其中 `us_stock_deep_analysis` 强制「罗列主流投行目标价」「理想入场区间」、
   `fed_rate_cut_analysis` 要求用 web_search 取宏观并禁用 Markdown，都与全局纪律冲突；
   建议在 `skill_manager` 里加自定义 skill 的编写约束，或把这批 skill 纳管。
4. 定时任务轮的「服务重启，之前的消息处理已中断」共 49 条（7 天），来自发版重启打断在途 cron；
   属部署链问题，不在 harness 层。

## Next Entry Point

- 上线后回归：等 24–48 小时真实流量，按同一口径重跑确定性指标
  （会话取自 `cloud_sessions`，工具证据取自 journald `runner.tool`，
  记得剔除 `[定时任务触发]` / `【Invoked Skill Context】` 伪 user 消息）。
  三条要盯的曲线：整轮零取证工具（基线 26.3%）、无取证工具却声称已取行情（基线 15.1%）、
  首行出现内部时区占位符（基线 43%）。
- 如需回滚：`ln -sfn /opt/hone/previous /opt/hone/current.new && mv -Tf …` 切回 1e7cfc15，
  harness 层从 `/srv/honeclaw/skills/backups/pre-a43f99c8-20260828T104304Z/` 还原。
