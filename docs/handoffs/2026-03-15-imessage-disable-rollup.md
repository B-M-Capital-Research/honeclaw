# 2026-03-15 iMessage 默认关闭与重复回复防护

## 已完成

- 项目默认配置将 `imessage.enabled` 设为 `false`，桌面设置页与任务创建 UI 也同步收敛为默认不使用 iMessage。
- Web 调度事件在 iMessage 已禁用时跳过该渠道投递，避免“配置已关但任务仍发”的漂移行为。
- `hone-imessage` 启动后会先检查 `imessage.enabled`，关闭时直接退出。
- iMessage 轮询 SQL 增加 `m.service = 'iMessage'` 过滤，避免把短信通道如 `95555` 误纳入处理。
- 增加短期重复去重：相同 `handle + text` 在 120 秒内重复出现会被跳过。

## 验证

- 未补完整手工回归；当时以代码变更和局部检查为主。

## 备注

- 本文档替代原先拆开的 `2026-03-15-imessage-disable.md` 与 `2026-03-15-imessage-disable-repeat.md`。
