# Archive Index

Last updated: 2026-04-26

## 2026-04-26

### Non-P1 Fixing Bug Batch

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/non-p1-fixing-bug-batch.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `0e917fe fix scheduler fixing bug batch`
- Related runbooks / regressions: `cargo test -p hone-channels scheduler::tests`, `cargo test -p hone-channels prompt::tests`, `cargo test -p hone-channels`, `cargo test -p hone-feishu failed_reply_text`, `git diff --check`
- Current conclusion: 非 P1 `Fixing` 批次已完成代码止血与文档同步；按新口径，已代码修复但只待真实窗口复核的缺陷统一标记为 `Later`，不再占活跃队列，后续复现时改回 `New`
- Next entry point: `docs/bugs/README.md#later--待复现`

### Remove Truth Social Source

- Status: done
- Date: 2026-04-26
- Plan: N/A, single-session deletion did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-04-26-remove-truth-social-source.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check -p hone-web-api`
- Current conclusion: Truth Social 已从 event-engine 活跃 source 集合删除；`truth_social_accounts` 配置、`TruthSocialPoller` 模块、engine 装配、主配置启用项和本机 ignored effective config 均已移除，历史 403 断流 bug 标记为 Closed
- Next entry point: `docs/handoffs/2026-04-26-remove-truth-social-source.md`

## 2026-04-24

### Price Event Lane 增量改造

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/price-event-lane.md`
- Handoff: `docs/handoffs/2026-04-24-price-event-lane.md`
- Decision / ADR: `docs/decisions.md#d-2026-04-24-01-route-price-alerts-through-directional-band-lanes`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine price --lib`, `cargo test -p hone-event-engine router --lib`, `cargo test -p hone-event-engine digest --lib`, `cargo test -p hone-core --lib`, `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, `cargo test --workspace --all-targets --exclude hone-desktop`
- Current conclusion: 价格事件已从日级去重改为 low/band/close 分层 id；盘中 `price_band:{symbol}:{date}:{up|down}:{band_bps}` 可在同日多次跨新档时形成独立事件，router 使用价格专属 gap/cap 控频，digest 对同一 actor/symbol/date/window 保留最新价格态，收盘价格默认摘要化
- Next entry point: `docs/handoffs/2026-04-24-price-event-lane.md`

### Event Engine Close Price 与 Truth Social 后续修复

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/event-engine-close-price-truth-social-followup.md`
- Handoff: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine --lib`, `cargo fmt --all -- --check`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`, `env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-24 --actor telegram::::8039067465 --include-body`
- Current conclusion: Truth Social poller 已补 status / content-type / body-prefix 失败诊断，`price_close` 高波动已恢复 High / immediate 路由；真实模型 baseline 已从 12 条 LLM 样本扩到 15 条并 15/15 matched；2026-04-24 Telegram digest 省略项已可通过 `digest_item omitted` 审计，低信号 news/social/macro/no-op analyst 噪声已降噪
- Next entry point: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`

## 2026-04-23

### Event Engine 推送质量全量修复

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/event-engine-push-quality.md`
- Handoff: `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- Decision / ADR: N/A
- Related PRs / commits: `0ff23d4 feat(event-engine): improve push quality routing`, `df820ca feat(event-engine): add daily push calibration export`
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo test -p hone-core --lib`, `cargo check -p hone-web-api`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- Current conclusion: event engine 的 24 项推送质量清单已全部收口，新增 digest 去重 / min-gap / topic memory、source/channel 偏好、分类预算、方向性价格阈值、macro/earnings 时窗、delivery observability，以及 `amazon/nova-lite-v1` 不确定来源新闻分类基线
- Next entry point: `docs/handoffs/2026-04-23-event-engine-push-quality.md`

### Core Runtime 职责与类型收敛

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/core-runtime-type-consolidation.md`
- Handoff: `docs/handoffs/2026-04-23-core-runtime-type-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels agent_session`, `cargo test -p hone-channels runners::tests`, `cargo test -p hone-event-engine subscription`, `cargo test -p hone-web-api routes::history`, `bun run test:web`, `bun --filter @hone-financial/app typecheck`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`
- Current conclusion: `AgentSession` 的 prompt/skill turn 构建与 response finalization 已从主编排里拆出，runner/session 内部事件收敛到 canonical `run_event`，runner kind / CLI probe 逻辑有了统一 helper，前端历史附件类型已和 Rust 对齐，本地图片 marker 也补了 Rust/前端共享 fixture
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-22

### Git Hook Auto Format

- Status: done
- Date: 2026-04-22
- Plan: `docs/archive/plans/git-hook-auto-format.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `.githooks/pre-commit` hook smoke test with staged Rust formatting, `bash -n .githooks/pre-commit`, `bash -n scripts/install_gitleaks.sh`
- Current conclusion: 本地 Git hook 现在会在 commit 前自动格式化已暂存 Rust 文件并重新暂存，push 前的 rustfmt / gitleaks 仍作为兜底门禁；同一 Rust 文件如果同时有已暂存和未暂存改动，pre-commit 会停止以避免把未选择的内容混入 commit
- Next entry point: `.githooks/pre-commit`

## 2026-04-20

### Hone 内置技能高置信度收敛

- Status: done
- Date: 2026-04-20
- Plan: `docs/archive/plans/hone-skill-consolidation.md`
- Handoff: `docs/handoffs/2026-04-20-hone-skill-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `cargo test -p hone-tools load_skill_and_direct_invocation_accept_aliases`, `cargo fmt --all --check`
- Current conclusion: Hone 的高重叠金融 skill 已收敛到更小的维护面：`one_sentence_memory` 被删除，`major_alert` 被并入 `scheduled_task`，`valuation` 与 `stock_selection` 被并入带兼容 alias 的 `stock_research`；finance regression 已改为验证新的 canonical skill 形态
- Next entry point: `skills/stock_research/SKILL.md`

## 2026-04-19

### Hone 半小时健康巡检补齐用户端静态资源检查

- Status: done
- Date: 2026-04-19
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-19-hone-health-automation-public-web-check.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sed -n '1,220p' ~/.codex/automations/hone-health-30m/automation.toml`, `bun run build:web:public`, `curl http://127.0.0.1:8088/`, `ls packages/app/dist-public`
- Current conclusion: `hone-health-30m` 现在不会再把“`8088` 正在监听”误判成用户端健康；它新增了 `packages/app/dist-public/index.html` 与 `8088` HTML 返回检查，并在只缺用户端静态资源时优先执行 `bun run build:web:public` 做最小止血，只有仍不健康时才整套重启
- Next entry point: `.codex/automations/hone-health-30m/automation.toml`

### Web 邀请码手机号绑定与固定端口切换

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-phone-and-fixed-ports.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `bun run build:web`, `bun run build:web:public`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8088/api/public/auth/me`, `curl -I http://127.0.0.1:8088/chat`
- Current conclusion: bundled desktop 现在固定使用管理端 `8077` 与用户端 `8088`；Web 邀请码已改为与手机号强绑定，管理端发码必须填手机号，用户端登录必须同时提交邀请码和手机号。新的 release app 已按 runbook 切换到 `.app` runtime；`discord` / `feishu` 在线，`telegram` 仍因配置里的 `Invalid bot token` 处于 `degraded`
- Next entry point: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`

### 用户可见内部工作说明泄露修复

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/user-visible-internal-working-note-fix.md`
- Handoff: `docs/handoffs/2026-04-19-user-visible-internal-working-note-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels runners::tests -- --nocapture`, `cargo test -p hone-channels agent_session -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`
- Current conclusion: public web 不再把 `company_profiles/`、actor 用户空间、目录结构这类内部工作说明直接作为最终答复或执行中状态暴露给用户；ACP runner 在本轮发生工具调用时只接受“最后一个 tool 之后的 assistant 文本”作为最终答复候选，session 成功态也会对明显的内部 working note 触发安全 fallback
- Next entry point: `crates/hone-channels/src/agent_session.rs`

### Company Profile Optional Frontmatter

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-optional-frontmatter.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-optional-frontmatter.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`
- Current conclusion: 公司画像与事件现在不再在读取、列出、bundle preview/import 时硬依赖 YAML frontmatter；legacy plain Markdown 本地画像与 plain-Markdown 画像包都会推断最小 metadata 继续工作，不再因为 `缺少 frontmatter` 直接失败
- Next entry point: `memory/src/company_profile/markdown.rs`

### 公司画像包导入导出与傻瓜式导入流

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-transfer.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `bun run test:web`, `bun run typecheck:web`, `bun run build:web`, `bun run --cwd packages/app test:e2e`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: 公司画像现在支持 actor 私有画像包导入导出；Memory 页面左侧已收敛成单一“目标用户空间”列表，当前空间里的公司切换放到右侧详情内部；右侧会先自动扫描导入包，只在存在冲突时要求逐家公司选择“保留当前”或“用导入版本替换”，并在存在替换时自动生成导入前备份供用户下载；legacy plain Markdown 画像即使缺少 frontmatter，也能被 transfer 导出、自动备份并参与冲突判断
- Next entry point: `packages/app/src/context/company-profiles.tsx`

### Company Profile 模块拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-module-split.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all`, `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: `hone-memory` 里的 company profile 已按职责拆成 `types / markdown / storage / transfer / tests` 子模块，保留原有 `hone_memory::*` 导出面和导入导出语义，后续继续改画像能力时不需要再在单个超大文件里同时处理类型、Markdown、zip 和存储细节
- Next entry point: `memory/src/company_profile/mod.rs`

Use this file as the historical entry point for completed or paused work that should remain discoverable.

### Web 管理端 / 用户端端口隔离与公网暴露加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-admin-public-isolation.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `./launch.sh --web`, `curl http://127.0.0.1:8077/api/public/auth/me`, `curl http://127.0.0.1:8088/api/meta`
- Current conclusion: Web 管理端和 invite 用户端已按端口与可访问路由拆开；管理端默认监听 `8077` 并只提供 `/api/*` 与 console SPA，用户端默认监听 `8088` 并只提供 `/api/public/*` 与 `/chat`。后续安全加固已经补上 public 邀请码失败冷却、邀请码停用 / 恢复 / 重置与会话清退、单邀请码单活跃 session、HTTPS 场景 `Secure` cookie，以及 public API 默认去掉 `CORS: *`；公网暴露时仍必须确保管理端不被反代出去，并在反向代理 / WAF 层继续做 IP 级限流
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

### Public Web 邀请码与公网暴露安全加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/public-web-security-hardening.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: public 邀请码登录已从“无防刷、无撤销、无会话止血”状态提升到具备应用层失败冷却、邀请码停用 / 恢复 / 重置、旧 session 立即失效、HTTPS `Secure` cookie 和同源默认访问的基础安全面；剩余长期暴露风险主要转移到反向代理 / WAF 限流策略与管理端误暴露治理
- Next entry point: `crates/hone-web-api/src/routes/web_users.rs`

### Web 邀请码用户端与管理端入口拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-chat-user-surface.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-chat-user-surface.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`, `cd packages/app && bun run typecheck && bun run build`
- Current conclusion: 管理端现在可以在设置页生成邀请码并复制，侧边栏“开始”旁新增了用户端跳转 icon；用户侧新增 `/chat` 页面，通过邀请码登录并进入单会话 SSE 聊天窗口，过程卡片会展示 `Hone 思考中 -> 工具执行 -> 最终回复`；后端新增 `/api/public/*` 与 SQLite `web_auth` 存储，公开接口严格从 cookie 登录态反解 `web` actor，不再接受外部传入的 `channel/user_id/session_id`
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

## 2026-04-17

### 群聊中间进度改为 compact 可见

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/group-chat-compact-progress-visibility.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels outbound::tests -- --nocapture`, `cargo test -p hone-feishu listener -- --nocapture`, `cargo check --workspace --all-targets --exclude hone-desktop`
- Current conclusion: Telegram / Discord / Feishu 群聊现在都会显示处理中间进度，但默认收敛到 compact 粒度，只暴露“搜索信息 / 获取数据 / 执行命令 / 执行技能”等阶段，不再把 query、命令行和目录路径这类细节直接刷进群消息；当 runner 只吐出 `Tool` 这类泛化标签时，会结合 reasoning 回退成粗粒度动作文案，且连续多轮相同类型的工具调用也会像单聊一样逐轮追加
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 对话额度改为可配置并支持无限制

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/conversation-quota-config.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels run_success_commits_daily_conversation_quota -- --nocapture`, `cargo test -p hone-channels run_rejects_over_daily_limit_without_persisting_user_message -- --nocapture`, `cargo test -p hone-channels run_zero_daily_conversation_limit_bypasses_quota -- --nocapture`, `cargo run -q -p hone-cli -- config validate`
- Current conclusion: 用户每日成功对话额度不再固定写死为 `12`；现在由 `agent.daily_conversation_limit` 控制，`0` 表示无限制。本地 repo `config.yaml` 已切到 `0`，当前运行环境不再限制用户每日对话数
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-16

### Feishu 直聊 placeholder 假启动收口

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/feishu-direct-busy-placeholder-gap.md`
- Handoff: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: Feishu 私聊当前已不再在 session 冲突时先发 placeholder 再卡死等待，而是会在入口直接返回 busy 提示；这条修复针对的是“placeholder 假启动”问题，不等同于已经完全根除所有深层长时间持锁根因
- Next entry point: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`

### 搜索失败提示主根因修复与 Tavily 复核

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/search-failure-tavily-and-tool-call-fix.md`
- Handoff: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels sanitize_search_context -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: `web_search` 工具确实走 Tavily，但当前统一失败提示的主根因不是 Tavily 全局不可用，而是 multi-agent 搜索阶段历史上下文清洗不完整，遗留 assistant `tool_calls` 与被删除的 `tool` 结果失配，触发 OpenAI-compatible provider `tool call result does not follow tool call (2013)`；该问题现已修复并完成定向测试与 desktop release 打包验证
- Next entry point: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`

### Desktop 启动坑位沉淀与会话列表恢复

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `cargo test -p hone-core actor::tests::session_identity_can_be_restored_from_actor_session_id -- --exact`, `cargo test -p hone-memory session_sqlite::tests::list_sessions_skips_unreadable_rows -- --exact`, `cargo test -p hone-web-api routes::users::tests::actor_session_id_is_enough_for_listing_identity -- --exact`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8077/api/users`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: repo-local `honeclaw/data` 并未丢失，会话为空的主因是 backend session-listing 在部分脏数据路径上直接失败，导致 `/api/users` 错误返回空数组；现在列表会跳过损坏的 `normalized_json` 并从 `session_id` 回推 actor identity，desktop release runtime runbook 和 `bug-2` automation 也已经把锁文件、detached 启动静默失败、desktop/backend 分离排障、正式接口验证等坑位写清楚
- Next entry point: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`

### Desktop Agent 配置隔离修复

- Status: done
- Date: 2026-04-16
- Plan: `docs/current-plans/canonical-config-runtime-apply.md`
- Handoff: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core promote_legacy_runtime_agent_settings`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`
- Current conclusion: desktop legacy agent config promotion no longer overwrites canonical `agent.opencode` when the canonical `api_key` is intentionally blank, and desktop settings save no longer lets `multi-agent.answer` silently overwrite `agent.opencode`; both P1 bug docs and the bug navigation table are now updated to `Fixed`
- Next entry point: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`

## 2026-04-15

### Bug 台账导航页与自动化文档模式升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-bug-index-and-automation-doc-mode.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `ls docs/bugs`, `sed -n '1,40p' docs/bugs/*.md`, `sed -n '1,220p' .codex/automations/bug/automation.toml`, `sed -n '1,220p' .codex/automations/bug-2/automation.toml`
- Current conclusion: `docs/bugs/README.md` 现在作为 bug 目录导航和状态总表存在，集中展示活跃待修复、已修复/关闭和历史分析条目；`bug` 与 `bug-2` 两个 automation 都被要求在任何 bug 状态变化时同步维护这张表
- Next entry point: `docs/bugs/README.md`

### Bug 每小时巡检自动化升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sqlite3 data/sessions.sqlite3 '.tables'`, `sqlite3 data/sessions.sqlite3 'pragma table_info(session_messages);'`, `find data/runtime -maxdepth 2 -type f`
- Current conclusion: 每小时 `bug` automation 现在会优先巡检最近一小时真实会话与运行日志，并把“AI 返回不及预期、结构/格式错误、返回质量不佳但不影响功能链路”的问题统一按 `P3` 建档；只有真正影响功能链路、正确性、稳定性或投递结果的问题，才继续提升到 `P0`-`P2`
- Next entry point: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`

### Desktop 日志接口与 multi-agent 运行态恢复

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/runtime-logs-runner-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api logs`, `cargo test -p hone-core from_file_applies_runtime_overlay`, `curl http://127.0.0.1:8077/api/logs`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: `/api/logs` 现在能容忍非 UTF-8 日志内容与日志缓冲锁中毒，不再因为多字节明文切片直接断开连接；`HoneConfig::from_file()` 也会正确合并 runtime overlay，渠道与 desktop 运行态恢复后能够稳定回到 `multi-agent`
- Next entry point: `crates/hone-web-api/src/routes/logs.rs`

### 持仓记忆补齐持有期限与策略信息

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/portfolio-memory-horizon-strategy.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory portfolio`, `cargo test -p hone-tools portfolio_`, `cargo test -p hone-web-api portfolio`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: 持仓记忆现在除标的、数量、成本和备注外，还会稳定保留 `holding_horizon`（`long_term` / `short_term`）和 `strategy_notes`；前端表单已允许负成本价输入，底层存储 / tool / API / UI 都兼容负成本与新增策略字段
- Next entry point: `memory/src/portfolio.rs`

### GitHub Security / Quality 高优问题收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/security-quality-remediation.md`
- Handoff: `docs/handoffs/2026-04-15-security-quality-remediation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bun run build:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: 已收口 research proxy URL 校验、session / company profile 路径组件校验、console 明文 user id 日志、Actions workflow 权限与一批高优 transitive dependency；剩余值得关注但未继续深挖的主要是 desktop GTK/Tauri 链上的 `glib` 告警，以及 `feishu-sdk -> salvo_core` 带入的低优 `rand 0.10.0`
- Next entry point: `docs/handoffs/2026-04-15-security-quality-remediation.md`

### Pre-Compact KV Cache 稳定性收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/kvcache-stability-before-compaction.md`
- Handoff: `docs/handoffs/2026-04-15-kvcache-stability-before-compaction.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-agent-codex-cli`
- Current conclusion: Hone 现在不会在下一次 compact 之前，由自身更小的 recent restore window、按当前用户输入动态变化的 system prompt related-skill block，或 `codex_cli` 的额外 20 条裁剪，提前制造可避免的 cache miss；compact 之后 prefix 变化仍视为正常边界
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-13

### Multi-Agent 输出净化与 think/tool_call 泄漏修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/multi-agent-output-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-feishu`, `cargo test -p hone-channels sanitize_user_visible_output -- --nocapture`, `cargo test -p hone-channels restore_context_sanitizes_polluted_assistant_history -- --nocapture`, `cargo test -p hone-channels internal_search_note_does_not_skip_answer_stage -- --nocapture`
- Current conclusion: 统一新增用户可见输出净化层后，multi-agent 搜索阶段不再把带 `<think>` / `<tool_call>` 的内部工作稿直接返回给用户；`AgentSession`、`restore_context`、`session_compactor` 会在持久化、恢复与压缩路径上拦截或清洗污染内容；Feishu / Telegram / Discord / iMessage 用户可见回复现统一隐藏 `<think>`，Feishu / iMessage 流式 formatter 也会吞掉 `<tool_call>` / `<tool_result>` / `<tool_use>` 内部块
- Next entry point: `crates/hone-channels/src/runtime.rs`

### 跨渠道富文本分段渲染修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/cross-channel-rich-text-segmentation.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-telegram -p hone-discord -p hone-feishu`, `cargo test -p hone-channels outbound::tests::split_html_segments_rebalances_open_tags_across_segments -- --exact`, `cargo test -p hone-channels outbound::tests::split_markdown_segments_rebalances_code_fences_across_segments -- --exact`
- Current conclusion: 共享分段层现在新增 HTML / Markdown 两种 format-aware segmenter；Telegram 长回复会在分段边界自动补全并重开 HTML tag，Discord / Feishu 会在 Markdown 代码块跨段时自动补全并重开 fence，避免富文本结构在长回复发送时被切坏后降级或回退纯文本
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 飞书表格语法护栏

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/feishu-table-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu markdown`, `cargo test -p hone-channels prompt`
- Current conclusion: 飞书提示词已明确禁止模型手写原始 `<table .../>` 卡片标签；运行时会继续自动把标准 Markdown 表格转换成飞书表格，同时对损坏、截断或 schema 错误的 raw table 做规范化/降级，避免坏标签直接投递到用户侧
- Next entry point: `docs/archive/plans/feishu-table-sanitization.md`

### Skill Runtime 对齐 Claude Code 与 Multi-Agent 优化提案

- Status: done
- Date: 2026-04-13
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 已完成一份 proposal，对比 Claude Code 官方 skill 模型与 Hone 当前实现差异，并分析 `multi-agent` runner 下 skill 的实际使用模式；提案建议把 active skill state 提升为 runner 一等状态，随后再补 `allowed-tools` / `context: fork` / supporting files 等执行与作者体验能力
- Next entry point: `docs/proposals/skill-runtime-multi-agent-alignment.md`

## 2026-04-14

### 会话上下文超限自动恢复与错误净化

- Status: done
- Date: 2026-04-14
- Plan: `docs/archive/plans/context-overflow-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-channels context_overflow_auto_compacts_and_retries_successfully -- --nocapture`, `cargo test -p hone-channels context_overflow_failure_is_rewritten_to_friendly_message -- --nocapture`
- Current conclusion: `AgentSession` 现在会识别上下文超限错误并在同一 turn 内先强制 compact 当前 session、再重新准备 execution 自动重试一次；若恢复后仍失败，用户只会看到稳定友好的提示，不再看到 `bad_request_error`、`invalid params`、`context window exceeds limit` 等底层 provider 原始报错
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-12

### v0.1.10 CLI Onboarding Provider 配置补齐

- Status: done
- Date: 2026-04-12
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-cli`, `bash scripts/prepare_release_notes.sh v0.1.10 /tmp/release-notes-v0.1.10.md`
- Current conclusion: `hone-cli onboard` 现在会明确要求用户对 `FMP` 和 `Tavily` API key 做出“填写或跳过”的选择；`FMP` 首装写入改为优先使用 `fmp.api_keys`，并清空旧的 `fmp.api_key` 兼容字段；对应 release notes 已补齐到 `docs/releases/v0.1.10.md`
- Next entry point: [v0.1.10 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.10)

### v0.1.9 Release 失败修复与补发

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/release-v0.1.9-publish-fix.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `a505060` (`docs: restore v0.1.9 release notes`)
- Related runbooks / regressions: `bash scripts/prepare_release_notes.sh v0.1.9 /tmp/release-notes-v0.1.9.md`, GitHub Actions `Release` run `24307695528`
- Current conclusion: 已补齐 `docs/releases/v0.1.9.md` 并重推 `v0.1.9` tag；`ensure-release` 不再因缺失 release notes 失败，三套发布产物与 `SHASUMS256.txt` 已成功上传，Homebrew formula 同步发布完成
- Next entry point: [v0.1.9 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.9)

### 公司画像与长期基本面追踪

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/company-portrait-tracking.md`, `docs/archive/plans/company-portrait-skill-framework.md`, `docs/archive/plans/company-research-actor-spaces.md`, `docs/archive/plans/remove-kb-memory-surface.md`
- Handoff: `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`, `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`, `bun run --cwd packages/app typecheck`
- Current conclusion: Hone 已具备 Markdown 形式的公司画像与事件时间线、按 actor 展示的画像 Web 视图（允许彻底删除），以及更贴近投研档案的 `company_portrait` skill；画像文档现在直接落在 actor sandbox 的 `company_profiles/` 中，由 agent 使用 runner 原生文件读写维护，不再依赖专用 mutation tool、公共画像目录或 KB 记忆入口
- Next entry point: `docs/handoffs/2026-04-12-company-portrait-tracking.md`

### CLI 首装 Onboarding 与安装向导

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/cli-onboarding-install-wizard.md`
- Handoff: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `bash tests/regression/manual/test_install_bundle_smoke.sh`, `cargo check -p hone-cli`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已支持首装 `onboard/setup` TUI，能够探测本机 runner、在不强迫 Hone 侧填写 OpenCode provider 配置的前提下切到 `opencode_acp`，并按渠道逐个引导启用与填写本地必填字段；GitHub release 安装脚本在交互终端下会询问是否立即运行该向导
- Next entry point: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`

### Desktop Rust Check 与 IDE 语法检查解耦

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/desktop-rust-check-workflow.md`
- Handoff: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`
- Current conclusion: 默认 workspace Rust 检查继续排除 `hone-desktop`；desktop crate 新增开发态 sidecar 校验豁免开关，VSCode rust-analyzer 默认携带该 env，因此 IDE / 本地 `cargo check` 不再被缺失的 Tauri bundled binaries 阻塞
- Next entry point: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`

### Hone CLI Config MVP 与可安装启动流

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/hone-cli-config-mvp.md`
- Handoff: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-core`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已具备 `config / configure / models / channels / status / doctor / start` 管理面；shared runtime overlay service 已供 CLI 与 desktop 共用；macOS / release 安装链路支持 `hone-cli start`，且已补齐首次 runtime config seed 行为
- Next entry point: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`

### Local 私有 Workflow Runner（公司研报 v1）

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/local-workflow-runner.md`
- Handoff: `docs/handoffs/2026-04-12-local-workflow-runner.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cd local/workflow && bun test`, `cd local/workflow && bun run bootstrap-config`, `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`, `WORKFLOW_RUNNER_PORT=3213 bun run start`
- Current conclusion: 在 `local/workflow/` 下新增独立本地 workflow runner，并在后续迭代中补齐紧凑工作台、运行级 prompt override、SSE 去重续流、停止接口、单实例串行、Python UTF-8/旧版本注解兼容，以及结构化进度与节点详情观测；当前 `company_report` 入口既可在页面里运行/观察/停止，也可通过 `bun run client` 从本机其它位置发起并监听进度
- Next entry point: `docs/handoffs/2026-04-12-local-workflow-runner.md`

## 2026-04-11

### 金融自动化合同回归闭环

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/finance-automation-contract-loop.md`
- Handoff: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `bash tests/regression/run_ci.sh`
- Current conclusion: finance 固定 9 样本合同切片已从 `success=5 review=1 fail=3` 收口到 `success=9 review=0 fail=0`；剩余 skill policy wording 漂移已全部修正
- Next entry point: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`

### 大文件物理拆分重构

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/large-files-refactor.md`
- Handoff: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`
- Decision / ADR: `docs/decisions.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: runtime override和渠道启动已收口到共享层；desktop sidecar、Feishu / Telegram 渠道热点与前端 settings 纯状态逻辑已按职责拆开，验证矩阵已跑通
- Next entry point: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`

## 2026-03-31

### macOS DMG Release 打包收口

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `make_dmg_release.sh`
- Current conclusion: 新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG；release 包内置 `hone-mcp` 与 macOS `opencode`，并补齐 packaged/runtime 启动环境与启动锁重试路径
- Next entry point: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`

### 定时任务输出净化与 Tavily 失败隔离

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools`, `cargo test -p hone-channels`
- Current conclusion: heartbeat / 定时任务会抽出真正 JSON 结果；Tavily 临时失败会返回脱敏 unavailable 结构，且不再持久化进会话工具上下文
- Next entry point: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`

## 2026-03-29

### 额度与定时任务可靠性修复

- Status: done
- Date: 2026-03-29
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory`, `cargo test -p hone-channels`
- Current conclusion: 普通用户每日额度调整为 12；非 heartbeat 定时任务补上“同日单次补触发”；heartbeat JSON 解析失败会安全抑制
- Next entry point: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`

## 2026-03-27

### 单一聊天范围配置与群聊忙碌态控制

- Status: done
- Date: 2026-03-27
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-core -p hone-channels`
- Current conclusion: `dm_only` 收敛为 `chat_scope`；群聊忙碌态在显式触发场景具备统一控制
- Next entry point: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`

## 2026-03-26

### 子模型配置与心跳任务调度

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`, `cargo check -p hone-desktop`
- Current conclusion: Desktop 支持 OpenRouter 子模型配置，会话压缩切到子模型，cron 新增 heartbeat 任务类型
- Next entry point: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`

### Session SQLite 影子写入与运行时切换

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_session_sqlite_migration.sh`
- Current conclusion: SessionStorage 已支持 `json | sqlite` 切换；SQLite shadow write 与 runtime 主读都已接入
- Next entry point: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`

## 2026-03-24

### 群聊预触发窗口统一改造

- Status: done
- Date: 2026-03-24
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-channels -p hone-core`
- Current conclusion: Telegram / Discord / 飞书群聊统一为“未触发先静默缓存、显式触发再执行”的预触发窗口模型
- Next entry point: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`

## 2026-03-22

### 多渠道附件工程化卡点

- Status: archived
- Date: 2026-03-22
- Plan: `docs/archive/plans/channel-attachment-gate.md`
- Handoff: `docs/handoffs/2026-03-22-channel-attachment-gate.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: 共享附件 ingest 已统一拦截超限附件与异常图片，并把拦截原因透出到渠道 ack
- Next entry point: `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## 2026-03-19

### 真群聊共享 Session 落地

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-shared-session.md`
- Handoff: `docs/handoffs/2026-03-19-group-shared-session.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`, `cargo test -p hone-memory -p hone-channels`
- Current conclusion: 群聊会话归属改为显式 `SessionIdentity`；三渠道群消息共享上下文，Web 控制台按真实 `session_id` 浏览
- Next entry point: `docs/handoffs/2026-03-19-group-shared-session.md`

### 群聊回复追加链路统一

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-reply-append-chain.md`
- Handoff: `docs/handoffs/2026-03-19-group-reply-append-chain.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-discord -p hone-telegram`
- Current conclusion: 群聊占位符、首条 `@用户` 与多段 reply 链已在 Discord / Telegram / Feishu 统一
- Next entry point: `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## 2026-03-18

### 渠道运行态心跳替代 pid 判活

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`, `cargo test -p hone-core -p hone-web-api`
- Current conclusion: `/api/channels` 已改为基于 `runtime/*.heartbeat.json` 的心跳新鲜度呈现状态
- Next entry point: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`

### launch.sh 真实进程清理修复

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash -n launch.sh`, `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: `launch.sh` 已直接启动真实 debug 二进制，pid 文件改为记录真实服务进程
- Next entry point: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`

### Discord 重复“正在思考中”排查

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- Current conclusion: 结论偏向入口被多个 consumer / 进程重复消费，而不是单次 `opencode_acp` run 自行双发 thinking
- Next entry point: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`

### Runner 切换到 Gemini 3.1 Pro

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_gemini_streaming.sh`
- Current conclusion: 默认 runner 已切到 `gemini_acp`，模型固定为 `gemini-3.1-pro-preview`
- Next entry point: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`

## 2026-03-17

### IM 渠道共享入口收口

- Status: archived
- Date: 2026-03-17
- Plan: `docs/archive/plans/attachment-ingest-unify.md`
- Handoff: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`, `cargo test -p hone-channels`
- Current conclusion: 共享 `ingress` / `outbound` 抽象已收口；Discord / 飞书附件 ingest 与 KB 管线下沉到 `hone-channels`
- Next entry point: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`

### 文档计划与 handoff 清理

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-doc-context-cleanup.md`
- Decision / ADR: `docs/adr/0001-repo-context-contract.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 清空已完成计划、合并零碎 handoff，并把 `docs/current-plan.md` 恢复为活跃任务入口
- Next entry point: `docs/handoffs/2026-03-17-doc-context-cleanup.md`

### Legacy 兼容移除与数据迁移

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体迁移细节见 handoff
- Next entry point: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`

### 项目清理（会话稳定性 / 渠道收敛）

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-project-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体清理结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-project-cleanup.md`

### 架构收敛与稳定性审计

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体审计结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`

### Identity 限额策略

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-identity-quota-policy.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体策略结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-identity-quota-policy.md`

### 运行时管理员口令拦截

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-register-admin-intercept.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体拦截链路见 handoff
- Next entry point: `docs/handoffs/2026-03-17-register-admin-intercept.md`

### Telegram 管理员白名单支持

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/telegram-admin-whitelist.md`
- Handoff: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels`
- Current conclusion: `admins` 正式支持 `telegram_user_ids`，共享管理员判定已接入 Telegram，当前私聊 identity `8039067465` 已写入本地配置
- Next entry point: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`

### 活跃计划清理

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-current-plan-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: `docs/current-plan.md` 已从 10 个活跃任务收口到 4 个；6 个长期失焦或仅剩占位语义的计划已移入 `docs/archive/plans/`
- Next entry point: `docs/handoffs/2026-04-16-current-plan-cleanup.md`
