# Bug: Web / Feishu 直聊公司画像沉淀后向用户暴露内部相对文件路径

- **发现时间**: 2026-06-02 11:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 时间窗：2026-06-02 22:59-23:01 CST
  - session_id: `Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`
  - 用户输入摘要：`HPE现在可以建仓吗`
  - Feishu direct 最终 assistant final 已完成 HPE 建仓判断、估值区间、证伪条件与来源，并正常写入会话。
  - 最终用户可见正文末尾包含内部相对路径：`company_profiles/hpe/profile.md` 与 `company_profiles/hpe/events/2026-06-02-build-position-check.md`。
- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-02 10:58-11:00 CST
  - session_id: `Actor_web__direct__web-user-14f4cadb069f`
  - 用户输入摘要：`avgo财报如何看`
  - ACP 事件显示该轮有 `session/prompt`、公司画像文件写入 tool update，以及最终 `response stopReason=end_turn`，说明 Web direct 回复链路已收口。
  - 最终用户可见正文末尾包含：`我已把 AVGO 财报前框架沉淀到 company_profiles/AVGO.md，后续财报出来可以直接对照更新。`
- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-03 19:01-19:03 CST
  - session_id: `Actor_web__direct__web-user-c394f2531362`
  - 用户输入摘要：`帮我评估一下nok`
  - ACP 事件显示该轮 Web direct 已完成行情/公司分析、写入 `company_profiles/NOK.md`，并以 `response stopReason=end_turn` 收口。
  - 最终用户可见流式 chunk 在 19:03:06 CST 拆成两段输出：`本地画像：company_profiles` 与 `/NOK.md。`，合并后仍是内部相对路径 `company_profiles/NOK.md`。
- `data/sessions.sqlite3`
  - 时间窗：2026-06-03 19:02-23:02 CST
  - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 在 19:38 CST 完成 XFAB 长期画像沉淀，但最终用户可见正文列出 `company_profiles/xfab/profile.md`、`company_profiles/xfab/events/2026-06-03-q1-2026-research.md` 与后续长期画像事件路径。
  - `session_id=Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a` 在 22:58 CST 完成 HPE 深度分析并正常收口，但正文末尾写出 `本轮已新增长期画像：company_profiles/HPE.md`。
  - 同窗 assistant final 污染扫描只命中上述 2 条内部相对路径；未命中空回复、`hone-mcp binary not found`、本机绝对路径、raw tool 字段、思维痕迹或 provider 原始错误。
- `data/sessions.sqlite3`
  - 时间窗：2026-06-04 07:02-11:01 CST
  - `session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c` 在 10:32 CST 收到用户输入“腾讯控股的画像”。
  - 10:35 CST assistant final 已完成腾讯控股长期画像正文并正常收口；本轮没有 `company_profiles/...` 相对路径、绝对路径、raw tool 字段、思维痕迹或 provider 原始错误进入 final。
  - 但最终用户可见开头写出：`我已为腾讯控股建立长期画像，路径是：\n公司画像公司画像`。这说明相对路径净化已生效，但替换结果仍保留“路径”概念并产生重复“公司画像”文本，属于同一公司画像沉淀输出边界的产品文案退化。
- `data/sessions.sqlite3`
  - 时间窗：2026-06-04 19:02-23:05 CST
  - `session_id=Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2` 在 23:01 CST 收到用户输入“分析一下cien的财报”，23:03 CST assistant final 已完成 CIEN 财报分析并正常收口。
  - 该 final 末尾仍写出内部相对路径：`我已把这次 FY2026 Q2 财报结论沉淀到本地公司画像：company_profiles/Ciena_CIEN.md`。
  - `session_id=Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a` 在 23:02 CST 收到用户输入“美股NOK详细分析，和建仓价格”，23:04 CST assistant final 已完成 NOK 分析、估值区间与建仓建议并正常收口。
  - 该 final 末尾仍写出内部相对路径：`本轮已新增长期画像：company_profiles/NOK.md。`
  - 同窗另有两个更晚 Feishu direct 用户请求截至本轮轮询仍停在 user / streaming 状态，未作为未回复缺陷登记；`cron_job_runs` 在 19:02-23:05 CST 无新记录。
- `data/sessions.sqlite3`
  - 时间窗：2026-06-04 23:01-2026-06-05 03:02 CST
  - `session_id=Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2` 23:03 CST CIEN 财报分析继续外露 `company_profiles/Ciena_CIEN.md`。
  - `session_id=Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a` 在 23:04 / 23:21 / 23:33 / 23:41 CST 分别完成 NOK、DXYZ、NASA ETF、NBIS 分析并正常收口，但 final 末尾分别写出 `company_profiles/NOK.md`、`company_profiles/DXYZ.md`、`company_profiles/NASA.md`、`company_profiles/NBIS.md`。
  - 同窗 `session_messages` 有 31 个 user turn 与 31 个 assistant turn，Feishu / Discord 会话均成对收口；普通 scheduler 只有 1 条记录，状态为 `completed + sent + delivered=1`。
  - assistant final 污染扫描未命中空回复、`hone-mcp binary not found`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、panic 或 `index out of bounds`。
- `data/sessions.sqlite3`
  - 时间窗：2026-06-05 03:01-07:01 CST
  - 本窗有 10 个 user turn 与 10 个 assistant final，Feishu direct 均成对收口；普通 scheduler 本窗没有新增 `cron_job_runs`。
  - assistant final 污染扫描未命中空回复、`hone-mcp binary not found`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、panic 或 `index out of bounds`。
  - 唯一用户可见污染命中 `session_id=Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2`：04:36 CST 用户输入“分析一下cien的财报”，04:37 CST assistant final 已完成 CIEN 财报分析、动作建议、证伪条件与来源并正常收口，但末尾写出 `我已把本轮更新补进本地画像：company_profiles/Ciena_CIEN.md`。

## 端到端链路

1. Web / Feishu direct 用户询问个股财报、估值或建仓判断。
2. runner 校验财报、行情、新闻和估值数据，并写入 actor sandbox 下的公司画像或事件文件。
3. 最终回复完成业务分析并正常收口。
4. 回复末尾把内部长期画像相对文件路径直接展示给用户。

## 期望效果

- 对外回复可以说明“已为后续跟踪沉淀本轮公司画像 / 事件框架”。
- 不应把 `company_profiles/<ticker>.md` 这类内部文件组织路径作为用户可见结论的一部分。
- 若产品要暴露画像入口，应使用前端可点击的业务入口、附件或自然语言说明，而不是 runner sandbox 的内部目录名。

## 当前实现效果

- 主分析内容完整，用户可以基于正文理解 AVGO / HPE / NOK 等公司分析。
- 但最终回复把 `company_profiles/AVGO.md` 作为沉淀位置告诉用户；该相对路径不是 Web 用户可直接使用的稳定产品入口。
- 23:01 CST Feishu direct HPE 建仓回复也把 `company_profiles/hpe/profile.md` 与 `company_profiles/hpe/events/2026-06-02-build-position-check.md` 发给用户，说明问题不局限于 Web direct。
- 2026-06-03 19:03 CST Web direct NOK 回复再次把 `company_profiles/NOK.md` 作为“本地画像”位置发给用户，说明真实用户可见输出路径仍未被完全净化。
- 2026-06-03 19:38-22:58 CST Feishu direct 又出现 XFAB / HPE 两条同类样本，说明复发范围继续覆盖 Feishu direct 的公司画像沉淀与深度分析回复。
- 2026-06-04 10:35 CST Feishu direct 腾讯画像回复已不再出现 `company_profiles/...`，但路径短语被净化成 `公司画像公司画像`，用户仍看到不自然的内部落点说明。该现象说明当前净化层可能只做路径片段替换，没有把整句“路径是 ...”重写成稳定的业务口径。
- 2026-06-04 23:03-23:04 CST Feishu direct CIEN / NOK 两条最新 assistant final 再次直接包含 `company_profiles/Ciena_CIEN.md` 与 `company_profiles/NOK.md`，说明相对路径净化在当前运行态仍未覆盖公司画像沉淀 final。
- 2026-06-04 23:21-23:41 CST 同一 Feishu direct 会话又在 DXYZ / NASA / NBIS 三条分析 final 末尾写出 `company_profiles/DXYZ.md`、`company_profiles/NASA.md`、`company_profiles/NBIS.md`，说明问题不局限于个别 ticker，也不只是上一轮 CIEN / NOK 样本。
- 2026-06-05 04:37 CST Feishu direct CIEN 财报分析再次写出 `company_profiles/Ciena_CIEN.md`，说明 03:03 CST 的补充净化修复后，当前运行态仍有真实用户可见复现；导航页一度把本项列为 `Fixed`，本轮已按单文档和最新证据修正回活跃 `New`。
- 本轮没有看到 `/Users/...`、`data/agent-sandboxes/...`、`/var/folders/...` 等绝对路径进入最终正文；绝对路径只出现在 ACP tool update 诊断事件中。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 它暴露了公司画像的内部文件组织方式，降低回复的产品感，也可能让用户误以为自己能直接访问该路径。
- 本轮 AVGO / HPE / NOK 分析已完成、文件写入也成功、最终回复正常收口，没有未回复、错投、数据损坏或投递失败证据。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是公司画像沉淀流程把 runner 原生文件路径作为“沉淀完成”的证明写入最终用户回复。
- 既有 `feishu_company_profile_absolute_path_leak.md` 修复覆盖的是绝对路径、本地 Markdown 链接和 sandbox 标识脱敏；本轮新增证据是 Web direct 最终正文里的内部相对路径，属于相邻但独立的用户态文案边界。
- 2026-06-04 腾讯画像样本显示，修复后的路径替换策略仍可能保留原句结构，把 `路径是：<internal path>` 变成 `路径是：公司画像公司画像`。这更像净化层缺少整句级 rewrite，而不是新的存储、投递或工具执行故障。
- 该问题也不同于 `web_direct_tool_call_raw_output_leak`：本轮最终正文没有 raw JSON、工具协议或 provider 报错外泄。

## 下一步建议

- 在公司画像 / 长期跟踪最终回复模板或共享出站净化层中，将 `company_profiles/<ticker>.md`、`events/*.md` 等内部相对路径改写为自然语言。
- 对“路径是：...”这类整句做业务级重写，例如改为“已沉淀为公司画像，后续可继续基于该画像更新”，避免片段替换后产生 `公司画像公司画像`。
- 对 Web / Feishu direct 增加一条回归：当 runner 成功写入公司画像文件时，最终用户可见文本只说明已沉淀，不包含内部文件路径。
- 后续巡检继续区分两类证据：绝对路径 / sandbox 标识泄漏应回看既有路径脱敏缺陷；仅相对内部路径进入自然语言回复时按本单跟踪。

## 修复记录

- 2026-06-09 继续补齐并关闭：
  - 共享 `sanitize_user_visible_output(...)` 在原有 `company_profiles/...` / `events/*.md` 相对路径脱敏基础上，新增公司画像落点文案重写：`路径是：公司画像公司画像`、`本地画像：公司画像`、`本地公司画像：公司画像`、`把本轮更新补进本地画像：公司画像` 等退化文本会统一改成自然业务表达。
  - 新增回归覆盖公司画像路径净化后的二次文案退化，避免只替换路径片段却把“路径”概念或重复“公司画像”继续暴露给用户。
  - 本轮按代码与回归验证将状态更新为 `Fixed`；未依赖当前机器 live 重启做运行态复核。

- 2026-06-03 19:03 CST 复发后回退：15:01-19:02 CST `session_messages` 共有 19 个 Feishu user turn 与 19 个 assistant final，Feishu direct 均成对收口，污染关键字扫描未命中 `hone-mcp binary not found`、原始工具字段、绝对路径、provider 报错或思维痕迹；但 `acp-events.log` 同窗 Web direct session `Actor_web__direct__web-user-c394f2531362` 对 `帮我评估一下nok` 已完成 NOK 分析并 `stopReason=end_turn` 收口，用户可见流式 chunk 仍输出 `本地画像：company_profiles/NOK.md`。由于这是 6 月 2 日修复后新的真实 Web direct 用户可见样本，本缺陷从 `Fixed` 回退为 `New`。该问题不影响分析正文、文件写入或投递收口，仍为质量性 `P3`，非 P1，不创建 GitHub issue。
- 2026-06-03 23:02 CST 复核：19:02-23:02 CST `session_messages` 有 21 个 user turn 与 22 个 assistant 记录，Feishu direct 最近会话均已收口；多出的 assistant 是 daily-limit final/text 双记录，不构成重复回复缺陷。assistant final 污染扫描只命中 2 条 `company_profiles/...` 内部相对路径：19:38 CST XFAB 画像沉淀列出 profile / events 路径，22:58 CST HPE 深度分析写出 `company_profiles/HPE.md`。本轮没有绝对路径、raw tool 字段、思维痕迹或 provider 原始错误进入 final；该问题仍不影响分析正文、文件写入或投递收口，严重等级保持 `P3 / New`，非 P1，不创建 GitHub issue。
- 2026-06-04 11:02 CST 复核：07:02-11:01 CST `session_messages` 有 14 个 Feishu user turn 与 14 个 assistant final，均成对收口；assistant final 污染扫描未命中 `company_profiles/...`、本机绝对路径、raw tool 字段、思维痕迹或 provider 原始错误，但 10:35 CST 腾讯画像回复出现 `路径是：公司画像公司画像`。该样本不再是原始路径外露，而是同一净化链路的重复替换 / 内部落点文案残留；不影响画像正文、文件写入或投递收口，严重等级保持 `P3 / New`，非 P1，不创建 GitHub issue。
- 2026-06-04 23:05 CST 复核：19:02-23:05 CST `session_messages` 有 5 个 Feishu user turn 与 3 个 assistant final；CIEN 和 NOK 两条 assistant final 均已完成分析正文并正常收口，但末尾分别写出 `company_profiles/Ciena_CIEN.md` 与 `company_profiles/NOK.md`。同窗 `acp-events.log` 有 4 个 `stopReason=end_turn`，未见 `hone-mcp binary not found`、provider 原始错误、quota、HTTP 400/429、panic、空回复或思维痕迹进入 final；该问题仍只影响用户可见文案和产品感，不影响分析正文、文件写入或投递收口，严重等级保持 `P3 / New`，非 P1，不创建 GitHub issue。
- 2026-06-05 03:02 CST 复核：23:01-03:02 CST `session_messages` 有 31 个 user turn 与 31 个 assistant turn，Feishu / Discord 会话均成对收口；普通 scheduler 只有 1 条 `completed + sent + delivered=1`。assistant final 污染扫描只命中 5 条 `company_profiles/...` 相对路径外露：23:03 CIEN、23:04 NOK、23:21 DXYZ、23:33 NASA、23:41 NBIS。`acp-events.log` 同窗显示 Feishu / Discord / Web direct 均有 `stopReason=end_turn`，未见 runner error；该问题仍只影响用户可见文案和产品感，不影响分析正文、文件写入、会话收口或投递，严重等级保持 `P3 / New`，非 P1，不创建 GitHub issue。
- 2026-06-05 07:02 CST 复核并修正导航一致性：03:01-07:01 CST `session_messages` 有 10 个 user turn 与 10 个 assistant final，Feishu direct 均成对收口；普通 scheduler 本窗没有新增 `cron_job_runs`。assistant final 污染扫描只命中 1 条 `company_profiles/...` 相对路径外露：04:37 CST CIEN 财报分析末尾写出 `company_profiles/Ciena_CIEN.md`。本轮未见空回复、内部绝对路径、raw tool 字段、思维痕迹、provider 原始错误或投递失败；该问题仍不影响分析正文、文件写入、会话收口或投递，严重等级保持 `P3 / New`，非 P1，不创建 GitHub issue。
- 2026-06-02 23:06 CST 复核：本轮在 Feishu direct HPE 建仓回复中观察到同类相对路径外泄，但当前远端 main 已在 12:15 CST 合入共享净化修复并有回归；该样本按 live 未确认部署 / 旧运行态证据保留，不把状态从 `Fixed` 回退。
- **修复时间**: 2026-06-02 12:15 CST
- **上次修复状态**: Fixed，2026-06-03 19:03 CST 已因新真实样本回退为 `New`
- **修复摘要**:
  - 共享 `sanitize_user_visible_output(...)` 的路径脱敏层新增 `company_profiles/...` 与 `events/*.md` 内部相对路径改写。
  - 最终用户可见文本会把这类 runner sandbox 文件组织路径替换为自然的“公司画像”表述，保留“已沉淀 / 后续可对照更新”的业务语义。
  - 新增 `sanitize_user_visible_output_redacts_internal_relative_company_profile_paths` 回归，覆盖 `company_profiles/AVGO.md` 进入 Web direct final 的复发形态。
- **验证**:
  - `cargo test -p hone-channels sanitize_user_visible_output_redacts_internal_relative_company_profile_paths --lib -- --nocapture`
  - `cargo test -p hone-channels sanitize_user_visible_output_redacts_bare_absolute_paths --lib -- --nocapture`
  - `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- **文档同步**:
  - 已同步 `docs/bugs/README.md` 活跃计数、状态和已修复表。
  - 本修复不改变模块边界、入口、长期约束或运行工作流，不需要更新 `docs/repo-map.md`、`docs/current-plan.md` 或新增 handoff。
- 2026-06-09 00:12 CST 复核当前代码：`sanitize_user_visible_output(...)` 已有 `sanitize_user_visible_output_redacts_internal_relative_company_profile_paths` 回归覆盖 `company_profiles/AVGO.md` 用户可见文本净化，本轮未新增代码；但 Rust toolchain 当前悬挂，无法重跑该回归，因此状态先记为 `Fixing` 而不是 `Fixed`。下一轮需恢复 toolchain 后运行 `cargo test -p hone-channels sanitize_user_visible_output_redacts_internal_relative_company_profile_paths --lib -- --nocapture`，通过后再从活跃队列移出。
- 2026-06-09 04:43 CST 状态更新为 `Fixed`：Rust toolchain 已恢复，`cargo test -p hone-channels sanitize_user_visible_output_redacts_internal_relative_company_profile_paths --lib -- --nocapture` 与 `cargo check -p hone-channels --tests` 通过。本轮未新增业务代码，仅以当前代码和回归确认该相对路径净化缺陷可退出活跃队列。
