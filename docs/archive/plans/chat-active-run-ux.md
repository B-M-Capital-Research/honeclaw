- title: 聊天活动任务进度与刷新恢复
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/chat.rs
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-web-api/src/state.rs
  - bins/hone-cli/src/start.rs
  - packages/app/src/pages/chat.tsx
  - packages/app/src/lib/public-chat.ts
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-07-16-chat-active-run-recovery.md

## Goal

投研回答继续在终审后只提交一次，但长时运行必须展示真实、可理解的服务端进度；页面刷新后恢复同一活动任务的原始开始时间和阶段，不得把计时器重置为 0、重复提交请求或暴露被拒绝草稿。

## Scope

- 复盘真实会话，区分数据预检、模型生成、内部重试、服务重启中断和客户端渲染耗时。
- 为活动 run 建立服务端权威的开始时间/阶段真相源，并让 Web SSE 与刷新恢复共用。
- 前端用服务端时间计算 elapsed，刷新时恢复同一 pending assistant card，而不是创建新的本地计时器。
- 为长投研等待提供安全进度文案，同时保持正文 final-only、0 reset、单 terminal。
- 受控重启先查询并排空活动聊天任务；异常中断显示明确终态，不伪装成活动任务。

## Verification

- `cargo test -p hone-web-api`：113 passed，2 ignored。
- `cargo test -p hone-cli start::tests -- --nocapture`：14 passed。
- `bun run test:web`：263 passed；Web typecheck 与 public production build 通过。
- `cargo build -p hone-cli -p hone-console-page -p hone-discord -p hone-feishu` 通过。
- 断开 SSE 回归：`run_started` 携带稳定 server `run_id/started_at_ms`，断流后活动数仍为 1，后台完成、落库后归零。
- 同 session 并发回归：第二条请求只返回一次 busy error；活动数仍为 1，最终 transcript 仅一组 user/assistant。
- 部署后 `nbis最近怎么样`：约 66 秒 runner / 69 秒端到端，等待期持续收到安全 progress，最终 1 个 `assistant_delta`、0 reset、0 run error、1 个 run finished；落库 9 节、3995 字符。
- 初次重启后发现 Cloudflare Pages 仍加载旧 bundle；生产跟进完成后，本地、8088 与 Pages 均切到 `index-DmyhjLnz.js`，lazy chunks 含完整 active-run 协议。
- 最终重启后 `/api/meta`、公共 auth config、Web/Discord/Feishu health 与 `/api/runtime/active-chat-runs={"count":0}` 正常；子进程使用独立 PGID，终端 SIGINT 不再先杀 Web 子进程。

## Documentation Sync

- 已更新 `docs/decisions.md`、`docs/invariants.md`、`docs/repo-map.md` 与 `docs/runbooks/backend-deployment.md`。
- 已新增 handoff，移出 `docs/current-plan.md` 并登记 `docs/archive/index.md`。

## Risks / Open Questions

- 当前 active-run registry 是单 Web 进程内状态；未来多 Web 实例必须改为共享 lease/fencing 或确保 sticky ownership。
- 非正常 SIGKILL 仍可能留下 quota reservation；UI 不再把它误判为活动任务，但后续应将 quota reservation 演进为可过期 lease。
- 排空端点覆盖 Web API 内登记的聊天任务；独立 channel sidecar 的在途任务仍需各自的生命周期协议。
