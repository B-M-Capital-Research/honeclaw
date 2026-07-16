- title: Web 聊天活动任务、刷新计时与安全重启修复
- status: done
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-web-api/src/state.rs
  - crates/hone-web-api/src/routes/chat.rs
  - crates/hone-web-api/src/routes/public.rs
  - bins/hone-cli/src/start.rs
  - packages/app/src/lib/public-chat.ts
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/archive/plans/chat-active-run-ux.md
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/runbooks/backend-deployment.md
- related_prs:
  - 335c4b73
  - 92d776a8
  - 4aa21b29

## Summary

“思考中一直增长、刷新后从 0 重新计时”不是模型自动重跑。旧页面把 quota `in_flight` 当成 runner 存活信号，并在每次刷新用 `Date.now()` 重建 pending 卡；SSE 的 detached runner 本来会在普通刷新后继续，但进程重启或崩溃会留下无终态的 user turn。投研 final-only 又让 60 秒左右的生成期没有正文，而公共页面没有消费已有 tool/progress 状态，最终形成假运行、归零和长时间无反馈的组合故障。

## What Changed

- 增加按 session 隔离的服务端 active-run registry：唯一 `run_id`、server start/update time、phase 和安全状态文案；同 session 第二次运行 fail busy，RAII guard 只清理匹配 run。
- `run_started` 带稳定 run 元数据；`RunEvent::Progress` 映射为 `run_progress`；工具状态只允许公共端读取 `public_status_text`，raw tool/text/reasoning 不参与渲染。
- public bootstrap/history 返回鉴权 actor 自己的 `active_run` 或 `interrupted_run`。判断中断时忽略 scheduled-push 投影，避免它遮掉更早的 dangling interactive user turn。
- 页面刷新只认 `active_run`，复用服务端 `started_at_ms` 并轮询最终 transcript；不再用 `user.in_flight` 或本地 `Date.now()` 猜运行态。无 live run 的未回答 turn 显示稳定“上次请求已中断，请重新发送”终态且不轮询。
- 投研候选正文继续 deferred/final-only；安全 progress 解决等待反馈，不重新开放草稿或 reset。
- 管理端新增 `/api/runtime/active-chat-runs`；`hone-cli start` 正常 Ctrl-C 后先排空活动 Web chat，再有界终止子进程。

## Verification

- Rust：`hone-web-api` 113 passed / 2 ignored；`hone-cli` start tests 14 passed。
- Web：263 tests passed，typecheck 与 public production build passed。
- Runtime build：CLI、console-page、Discord、Feishu passed。
- Live disconnect：SSE 客户端 1 秒断开后 count=1；后台完成后 count=0 且 assistant 已落库。
- Live duplicate：同 session 第二条请求返回 busy，未形成第二条 user/assistant transcript。
- Live NBIS：等待期收到 `run_started + run_progress`；最终 1 delta、0 reset、0 error、1 finished，9 节回答成功落库。
- Production assets：本地、8088 与 Cloudflare Pages 均为 `index-DmyhjLnz.js`；生产 `chat-B6liblxH.js` / `public-chat-LkMkttVo.js` 含 `active_run`、`interrupted_run`、`run_progress`、`started_at_ms`，不含旧恢复变量。
- Deployment health：Web/Discord/Feishu 均 running，active count=0，Postgres/S3 healthy，Worker auth probe 返回预期 401 JSON，未见新启动 error/panic。

## Production Follow-up

第一轮修复只重启了新后端并误构建到通用 `packages/app/dist`，Cloudflare Pages 和 8088 实际仍加载旧 public bundle。真实 RMBS 请求在后台 92.492 秒成功完成并落库 3323 字符，但旧页面忽略 progress、用 quota `in_flight + Date.now()` 伪造 pending，因此表现为无输出且刷新归零。

跟进已完成：

- 正确执行 `build:web:public`，并以线上 entry/lazy chunk 的实际 hash 和协议 marker 作为完成门禁。
- background poll 改为 settle 后递归 timeout；慢 bootstrap 不再被固定 interval 周期性 abort。
- recovered active run 与 local pending 共用 busy 状态，禁止重复提交；无 terminal EOF 有界恢复，耗尽后进入明确 error。
- bootstrap/history 客户端和服务端同时使用 `no-store`，动态 run 状态不进入浏览器或代理缓存。
- runtime child 使用独立 Unix process group；CLI supervisor 收到 SIGINT 后仍能访问 Web drain endpoint，再按顺序停止子进程。

## Risks / Follow-ups

- Registry 是当前单 Web 进程的权威状态。多实例部署前必须引入共享 lease/fencing 或 sticky ownership。
- SIGKILL 可能遗留 quota `in_flight`，虽然 UI 已完全与它解耦；quota lease 回收仍是独立后续项。
- CLI 排空只覆盖 Web API 登记的 chat run，不覆盖独立 sidecar 内部任务。

## Next Entry Point

从 `crates/hone-web-api/src/state.rs::ActiveChatRunRegistry`、`routes/chat.rs::build_chat_sse` 和 `packages/app/src/lib/public-chat.ts::resolvePublicChatRecovery` 开始。部署操作参考 `docs/runbooks/backend-deployment.md#drain-active-chats-before-a-controlled-restart`。
