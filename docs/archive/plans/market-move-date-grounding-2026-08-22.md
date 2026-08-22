# Market Move Date Grounding And Research Activation

- title: 涨跌归因日期检索与投资研究证据流修复
- status: archived
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `agents/function_calling/src/lib.rs`
- related_docs:
  - `docs/handoffs/2026-08-22-market-move-date-grounding.md`
  - `docs/archive/index.md`

## Goal

让涨跌归因问题优先检索当日新闻并读取来源发布日期，同时保证此类请求从第一轮进入现有投资研究证据流，避免 Web-only 直答绕过当前轮行情与日期核验。

## Scope

- 涨跌归因预取使用 Tavily `time_range=day` 与 `topic=news`。
- WebSearch 暴露 `topic` 参数并保留 Tavily 返回的 `published_date`。
- 已带涨跌归因日期锚点的请求直接激活现有 Agent-owned 投资研究循环。
- 未新增内容质量裁判、结构化证据层或全局发布拒绝条件。

## Validation

- `cargo test -p hone-agent --lib`：152 passed。
- `cargo test -p hone-channels investment_response_guard::tests --lib`：136 passed。
- `cargo test -p hone-tools web_search::tests --lib`：19 passed。
- `cargo check -p hone-tools -p hone-agent -p hone-channels`：通过。
- `git diff --check`：通过；三个 Rust 改动文件已直接运行 rustfmt。
- `cargo test -p hone-tools --lib`：168 passed、26 failed、1 ignored；失败项均依赖 PostgreSQL。`scripts/dev_pg.sh` 因本机无 Docker 无法启动测试库，本机也没有 PostgreSQL server binary，因此未把该环境失败记为本次代码失败。
- GitHub CI `32544207221` 的 compile、frontend、格式步骤通过；Rust tests 在未改动的 `soul.md` 固定字符预算测试停止。父提交 `a0ffc0b2` 的 CI `32531473440` 在相同测试、相同行号、相同原因失败，因此记录为 `main` 已有基线例外，未通过改 Prompt 或抬高机械阈值规避。
- Runtime Image `32544207247` 成功构建并校验 revision `e08bb4607a5a8cd559c4320db220063fc021e0b4` 的不可变 manifest。

## Documentation Sync

- 任务在同一会话内完成实现并归档，没有保留活跃计划索引。
- 已写 `docs/handoffs/2026-08-22-market-move-date-grounding.md` 并更新 `docs/archive/index.md`。
- 未更新 `docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`：模块边界、长期架构与生成型内容治理没有变化。

## Risks / Open Questions

- Tavily 的 `published_date` 是 news topic 的可选字段；缺失时运行时提示要求继续补搜或披露未核验，不能把查询日期当文章日期。
- `day` 是相对当前时间窗口；用户明确询问更早历史日期时，Agent 仍需按绝对日期补充检索。
- 生产已运行 revision `e08bb4607a5a8cd559c4320db220063fc021e0b4`；不可变 digest、GCE bundle、PostgreSQL/S3 authority、零活动会话、公网 JSON 401 与 Feishu 重连均已验证。
- 未在用户的已登录账号内发送 MRVL canary；发送消息会产生外部副作用，需要另行明确授权。本次发布由定向自动化回归与生产健康/连接验收覆盖。
