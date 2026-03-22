# Handoff: Actor Isolation And Feishu Post Support

日期：2026-03-07
状态：已完成

## 本次目标

- 把用户数据隔离规则统一为 `channel + user_id + channel_scope`
- 追平仍使用裸 `user_id` 的落盘数据面
- 补齐 Feishu `post` 消息与控制台附件展示能力

## 已完成

- 新增 `ActorIdentity`，作为会话、任务和其它用户落盘数据的统一归属模型
- 会话历史、定时任务、持仓、生成图片目录统一改为按 actor 隔离
- iMessage、Feishu、Discord、Telegram、CLI、Web 控制台统一走 actor 会话键
- Web 控制台持仓接口从裸 `user_id` 改为显式 `channel` / `user_id` / `channel_scope`
- 前端持仓模块改为按 actor key 选择、路由和请求
- Feishu facade 新增 `post` 消息正文、图片和文件附件解析
- Web 控制台历史消息新增附件预览和文件链接渲染

## 影响范围

- Rust：
  - `memory/*`
  - `crates/hone-core/*`
  - `crates/hone-channels/*`
  - `crates/hone-scheduler/*`
  - `crates/hone-tools/*`
  - `crates/hone-integrations/*`
  - `bins/hone-*/*`
- Web：
  - `packages/app/src/context/*`
  - `packages/app/src/lib/*`
  - `packages/app/src/components/*`
  - `packages/app/src/pages/layout.tsx`
- Feishu bridge：
  - `bridges/hone-feishu-facade/main.go`

## 验证

- 已运行 `cargo fmt --all`
- 已运行 `cargo test --workspace --all-targets`
- 已运行 `bun run typecheck`（`packages/app`）
- 已运行 `bun test --preload ./happydom.ts ./src`（`packages/app`）
- 已运行 `bash tests/regression/run_ci.sh`
  - 当前结果：`tests/regression/ci/` 下暂无脚本

## 长期行为变化

- 新增或修改任何按用户归属的落盘数据时，应优先使用 `ActorIdentity`
- Web 层若提供 actor 相关读写接口，应显式要求 `channel` 与 `user_id`，群范围使用 `channel_scope`

## 剩余风险

- 尚未把 actor 隔离补成 CI-safe 回归脚本，主干门禁对此没有专门黑盒覆盖
- 远端如继续新增新的用户级存储面，若未遵循 actor 规则，仍可能再次出现串数据
