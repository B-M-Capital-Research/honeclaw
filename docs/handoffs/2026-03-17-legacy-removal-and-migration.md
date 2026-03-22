# 2026-03-17 Legacy 兼容移除与数据迁移

## 结果

- 删除了运行时对旧 session summary 格式的兼容读取；`memory/src/session.rs` 现在只接受显式 `summary { content, updated_at }`
- 删除了压缩链路中对伪 `system` summary 消息的兜底读取；会话总结只认显式 `summary`
- 删除了 skill 运行时对 `skills/<name>.yaml|yml` 的兼容读取；`LoadSkillTool` 现在只加载 `skills/<name>/SKILL.md`
- 删除了 Web scheduler 对 `web_test` 和空 channel 的兼容；Web 侧调度渠道统一为 `web`
- `kb_analysis` 已从 legacy `create_agent()` 切到统一 runner contract，`HoneBotCore::create_agent()` 已删除
- 新增数据迁移脚本：
  - `scripts/migrate_sessions.py`
  - `scripts/migrate_cron_jobs.py`
  - `scripts/migrate_skills.py`
  - `scripts/migrate_legacy_data.py`

## 迁移范围

- session：
  - v1/v2 混合 summary 结构
  - 伪 `system` summary message
  - 基于旧 actor/session id 的文件名
  - `web_test` → `web`
- cron：
  - actor 缺失或 job.channel 仍为 `web_test` / `""`
  - 文件名与 actor storage key 不一致
- skill：
  - 顶层 `*.yaml` / `*.yml` 技能文件
  - 迁移为 `skills/<name>/SKILL.md`

## 验证

- `cargo test -p hone-memory -p hone-tools -p hone-channels`
- `cargo check -p hone-web-api`
- `bash tests/regression/ci/test_session_migration.sh`
- `bash tests/regression/ci/test_legacy_data_migration.sh`
- `bash tests/regression/run_ci.sh`
- `python3 -m py_compile scripts/migrate_sessions.py scripts/migrate_cron_jobs.py scripts/migrate_skills.py scripts/migrate_legacy_data.py`

## 后续注意

- 升级旧环境时，先执行 `python3 scripts/migrate_legacy_data.py --write`，再启动新版本
- 这次没有移除 Gemini CLI 对旧输出事件协议的兼容解析；该部分属于外部 CLI 协议兼容，不在本轮数据迁移清理范围内
