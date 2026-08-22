# Market Move Date Grounding Handoff

- title: 涨跌归因当日新闻与研究流启动交接
- status: done
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `agents/function_calling/src/lib.rs`
- related_docs:
  - `docs/archive/plans/market-move-date-grounding-2026-08-22.md`
  - `docs/archive/index.md`
- related_prs: none; direct `main` implementation commit `e08bb4607a5a8cd559c4320db220063fc021e0b4`

## Summary

修复已完成并进入发布流程：涨跌归因预取从 `week/general` 改为 Tavily `day/news`，从 provider 结果读取 `published_date`；带服务端涨跌日期锚点的请求从第一轮进入现有投资研究证据流，第一轮为 Required 工具轮，不能再由 Web-only 普通路径直接作答。

## What Changed

- WebSearch 新增可选 `topic=general|news|finance`，原样保留 news 结果的 `published_date`，并提示查询日期不等于文章发布日期。
- 两路涨跌归因预取（用户原话检索和身份锚定检索）都使用 `time_range=day, topic=news`；普通投研预取仍保持 `week`。
- FunctionCalling Agent 在发现现有涨跌归因日期锚点时立即激活 Agent-owned finance loop；非涨跌问题仍由原 DataFetch 边界激活。
- 宽市场旧回归契约同步为首轮 `Required`、证据满足后 `Auto`。

## Verification

- `hone-agent`：152/152。
- `investment_response_guard`：136/136。
- WebSearch：19/19。
- `cargo check -p hone-tools -p hone-agent -p hone-channels`：通过。
- `hone-tools` 全包的 26 个失败均因本机无可用 PostgreSQL 测试服务；同包其余 168 项通过，WebSearch 子集独立全绿。完整 PostgreSQL-backed 门禁由 GitHub CI 承担。

## Deployment Evidence

- 实现 commit：`e08bb4607a5a8cd559c4320db220063fc021e0b4`，已直接推送 `main`；无 PR、release 或 tag。
- Runtime Image run `32544207247` 成功；不可变 manifest digest 为 `sha256:314d82c90f6e78f0880a083d654145c7dd65956f88c174335e027b2628bcbf96`。
- GCE bundle 逐文件与 embedded revision 校验通过，`/opt/hone/current` 已原子切换至 `e08bb460…-ghcr-runtime`；切换前 `d9620309…-ghcr-runtime` 保留为即时回滚。
- `/api/meta`：`build.git_sha=e08bb460…`、`build.source=ghcr_linux_oci`、`cloud_mode=cloud`、PostgreSQL/S3 均健康、`cloud_storage_authoritative=true`、`local_durable_dependency_count=0`。
- 切换前两次和切换后/soak 的 active chat 均为 0；`hone-web.service` active、`NRestarts=0`，近端日志无 panic/fatal，单体 runtime 内 Feishu stream 已重连。
- `timezone: "Asia/Shanghai"` 保持不变；`https://hone-claw.com/api/public/auth/me` 返回 application JSON `401`。
- GitHub CI run `32544207221` 的唯一失败是未改动的 `soul.md` 固定字符预算；父提交 CI `32531473440` 在同一测试、同一行、同一原因失败。本次没有为通过机械内容门禁而改 Prompt/阈值。

## Risks / Follow-ups

- 发布过程中未修改生产数据库，未打正式版本 tag。
- 没有通过用户的已登录账号发送真实消息；若获得明确授权，再用 `mrvl下跌原因是啥呢` 核对 day/news、`published_date`、首轮 Required 与 DataFetch 当前轮证据。
- `soul.md` 字符预算基线应作为独立任务处理：先复核 Prompt 内容与生成型工作流治理，不应顺带抬高固定阈值或删减规则。
- `hone-claw.com` 的用户 API 路径返回正常 application JSON 401，发布后也已有活动会话；但 `origin.hone-claw.com` 直连仍 307 到旧 ngrok not-found。该域名/隧道别名未被本次代码或切换修改，二进制回滚不会修复，应按 runbook 独立审计 DNS 与隧道配置。

## Next Entry Point

从本 handoff 的 Deployment Evidence 和上述三个实现文件进入；运行时异常时按 `docs/runbooks/backend-deployment.md` 回滚到 `d9620309…-ghcr-runtime`。
