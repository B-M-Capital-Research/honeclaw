# Handoff：大文件重构 + 附件能力收敛

日期：2026-03-14

## 目标
- 拆分 `hone-console-page` 与 `hone-discord` 大文件为模块化结构
- Discord/飞书附件能力统一到 `hone-channels`，预留 Telegram 接口
- 移除 console page 旧静态 HTML

## 关键变更

### Console Page 模块化
- 新增 `logging/state/runtime/types/routes/*` 模块，`main.rs` 仅保留启动与路由组装
- 修复拆分后编译问题：
  - `hone_memory` 类型路径（`cron_job::CronJob`、`portfolio::*`）
  - `auth` 中 `Request<Body>` 与返回类型统一
  - `events` 使用 `tokio_stream::StreamExt`
  - `research` 使用 `config.web.research_api_base` 并内置 URL 编码
  - `runtime` 去除不存在的 `web.port` 覆盖
- 删除 `bins/hone-console-page/static/index.html`，仅通过 `packages/app/dist` 提供前端

### Discord 模块化
- 新增 `handlers/group_reply/attachments/utils/types`，`main.rs` 仅保留启动逻辑
- 群聊 @ 触发与 cron 禁用逻辑保持不变
- 群聊聚合队列、占位符、发送分段逻辑拆分至 `group_reply` + `utils`

### 附件统一
- `hone-channels/src/attachments.rs` 新增 `AttachmentFetcher` 接口与 `enrich_attachment_with_extract_dir`
- Discord/飞书使用统一 `ReceivedAttachment` + `build_user_input` 逻辑
- Discord 保留原有解压目录命名（通过传入 extract_dir）

## 验证
- `cargo check -p hone-console-page`
- `cargo check -p hone-discord`
- `cargo check -p hone-channels`
- `cargo test -p hone-discord`

## 未覆盖/注意事项
- 未做手工 smoke（console `/api/meta`/`/api/users`/`/api/chat`、Discord 群聊/私聊）
