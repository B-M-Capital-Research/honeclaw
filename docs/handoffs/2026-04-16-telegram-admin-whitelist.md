- title: Telegram 管理员白名单支持
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/config.rs`
  - `crates/hone-channels/src/core.rs`
  - `config.example.yaml`
  - `config.yaml`
  - `data/runtime/config_runtime.yaml`
- related_docs:
  - `docs/archive/plans/telegram-admin-whitelist.md`
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

补齐了 Telegram 管理员白名单的正式配置和共享权限判断，不再依赖“ADMIN LOCATOR”之类的人工探测文本。当前私聊 identity `8039067465` 已写入本地 canonical config 与 runtime config，因此后续重启并加载新代码后，Telegram 直聊会被直接识别为管理员身份。

## What Changed

- 在 `AdminConfig` 中新增 `admins.telegram_user_ids` 字段，作为 Telegram 数字 user ID 白名单。
- 在 `HoneBotCore::is_admin()` 中新增 `telegram` 分支，使 Telegram 与 iMessage / Feishu / Discord 走同一条共享管理员判定逻辑。
- 为配置反序列化和 Telegram 管理员识别新增测试，防止再出现“配置层有字段、运行层没接”的断层。
- 更新 `config.example.yaml`，让示例配置对 Telegram admin 可见。
- 更新本地 `config.yaml` 与 `data/runtime/config_runtime.yaml`，把当前 Telegram 用户 `8039067465` 加入管理员白名单。

## Verification

- `cargo test -p hone-core`
- `cargo test -p hone-channels`
- 本地配置确认：
  - `config.yaml` 包含 `admins.telegram_user_ids: ["8039067465"]`
  - `data/runtime/config_runtime.yaml` 包含 `admins.telegram_user_ids: ["8039067465"]`

## Risks / Follow-ups

- 当前已修改源码和本地配置，但正在运行的已编译进程不会自动热加载 Rust 代码；要让 Telegram admin 判定立即生效，仍需要按现有运行方式做一次有意图的重启 / 重载。
- `/register-admin <secret>` 的 passphrase 拦截逻辑没有改变；只是 Telegram 现在终于能进入同一套 allowlist 判定。

## Next Entry Point

- `docs/archive/plans/telegram-admin-whitelist.md`
