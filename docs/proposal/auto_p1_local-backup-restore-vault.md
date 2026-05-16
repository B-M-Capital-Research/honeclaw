# Proposal: Local Backup and Restore Vault for Durable Hone Workspaces

status: proposed
priority: P1
created_at: 2026-05-17 02:04:14 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `scripts/install_hone_cli.sh`
- `scripts/update_homebrew_formula.sh`
- `bins/hone-cli/src/start.rs`
- `bins/hone-cli/src/cleanup.rs`
- `bins/hone-cli/src/common.rs`
- `bins/hone-desktop/src/sidecar/runtime_env.rs`
- `crates/hone-core/src/config/materialize.rs`
- `crates/hone-core/src/config/mutation.rs`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `memory/src/portfolio.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/company_profile/{storage,transfer}.rs`
- `memory/src/web_auth.rs`
- `memory/src/llm_audit.rs`

## 背景与现状

Honeclaw 已经从源码运行的聊天助手演进成长期投资工作台。README 同时给出 `curl | bash`、Homebrew、源码启动、Web admin/user UI 与 Mac desktop；`docs/repo-map.md` 记录了 canonical `config.yaml`、`data/runtime/effective-config.yaml`、release bundle、desktop bundled/remote、actor sandbox、session SQLite rollout、公司画像、cron、portfolio、LLM audit 和 public web auth 等长期数据边界。

当前仓库已经有几块相邻能力：

- `scripts/install_hone_cli.sh` 与 Homebrew formula 把安装版放在 `~/.honeclaw`，保留 `config.yaml`、`data/`、`releases/` 和 `current` symlink。
- `bins/hone-cli/src/cleanup.rs` 能交互式删除 runtime data、release bundles、config 和 `soul.md`，但它是清理工具，不提供清理前备份或恢复。
- `crates/hone-core/src/config/materialize.rs` 明确区分用户可编辑的 canonical `config.yaml`、一次性 legacy runtime promotion、以及生成给子进程的 `data/runtime/effective-config.yaml`。
- `memory/src/session.rs` / `session_sqlite.rs` 正在支持 JSON 与 SQLite 双写/回填；`docs/invariants.md` 要求 SQLite 切换期间保留 JSON rollback mirror。
- 公司画像已有 actor-scoped bundle export/import，能处理 `profile.md` 和 `events/*.md`，但这是单一资产类型，不是整套安装状态。
- 现有提案已覆盖用户数据导出/删除、版本兼容/回滚、诊断包、研究交付物和文档收件箱。

缺口是：Hone 没有一个面向本地安装、桌面 bundled 和 self-host 用户的 **可恢复备份层**。一旦用户升级前想保留现场、换电脑、误删 `data/`、运行 `cleanup --all`、SQLite cutover 出问题、desktop app data 目录损坏、或需要从远端迁回本机，当前只能手工复制目录。手工复制既容易漏掉 config/secrets、actor sandbox、SQLite WAL、release manifest，也无法在恢复前预览冲突和兼容性。

## 问题或机会

这是 P1，因为 Hone 的核心价值来自长期积累：持仓、关注列表、公司画像、事件时间线、会话摘要、cron、通知偏好、public web 用户、API key、上传材料、研究报告、技能开关和模型审计。用户越认真使用，数据越不应依赖“自己知道该复制哪些目录”。

当前问题集中在六类：

1. **升级和回滚只保护二进制，不保护用户状态。**  
   Update Compatibility Center 可以回答版本是否兼容，但如果升级迁移写坏了 session SQLite、config schema 或 actor sandbox，用户缺少一键回到升级前数据快照的能力。

2. **清理命令是破坏性入口。**  
   `hone-cli cleanup` 默认保留 config，但会删除 runtime data 和 release bundles；`--all` 会删除 config 和 profile。它没有 dry-run backup artifact，也没有“清理后可恢复”的路径。

3. **数据域分散且语义不同。**  
   `config.yaml` 包含密钥；`data/runtime/effective-config.yaml` 是可再生快照；session 有 JSON/SQLite 双路径；company profiles 在 actor sandbox；public uploads 与 IM uploads 在不同目录；release bundles 可重下但 `current` symlink 代表安装状态。整包备份必须理解这些差异。

4. **SQLite/WAL 一致性需要产品级约束。**  
   直接复制运行中的 SQLite 文件可能漏 WAL 或拿到不一致快照。Hone 正在把 session、web auth、LLM audit、cron history 等能力更多放进 SQLite，因此备份必须通过 checkpoint/backup API 或停机快照，而不是粗暴拷贝。

5. **桌面用户最需要恢复，却最不该理解路径。**  
   Desktop packaged 模式把 config/data/logs/actor sandboxes 放进 app config/data/cache 目录。用户看到的是应用，不是多目录运行时。备份、恢复、迁移和升级前快照应该在桌面设置页可见。

6. **支持和商业化需要“可恢复”承诺。**  
   严肃投资助手不能只提供隐私导出，还要提供可恢复备份。对 self-host、桌面、本地开源和未来付费用户来说，“我能不能安全升级、迁移、回滚”直接影响留存。

机会是新增 **Local Backup and Restore Vault**：以安装/工作区为单位生成可校验、可加密、可演练恢复的备份包。它不替代 User Data Trust Center 的 actor 数据权利，也不替代 Update Compatibility Center 的版本判断，而是补上“本机长期工作区可恢复”的底座。

## 方案概述

新增一个本地优先的备份/恢复产品层：

1. `BackupManifest`
   记录备份版本、创建时间、Hone 版本、安装方式、config schema、data domains、文件 hashes、SQLite backup metadata、是否加密、是否包含 secrets、是否可跨设备恢复。

2. `BackupPlan`
   在真正打包前做 dry-run，列出会包含、跳过、重建或需确认的数据域。例如包含 `config.yaml` 但默认加密；跳过 `data/runtime/effective-config.yaml` 因为可再生成；包含 company profiles；可选包含 logs；release bundles 默认不包含但记录版本。

3. `RestorePreview`
   在恢复前读取 manifest，比较当前安装和备份：Hone 版本、config schema、actor 冲突、session backend、现有数据覆盖风险、缺失二进制、路径差异和需要停机的 SQLite domains。

4. `RestoreApply`
   只允许从 manifest 驱动恢复，不接受任意路径覆盖。支持 `new-home` 恢复、`merge-actors`、`replace-data`、`restore-config-only`、`restore-profiles-only` 等模式。

第一版目标是手动备份和可演练恢复；第二阶段再接入升级前自动快照、桌面 UI 和定期备份。

## 用户体验变化

### 用户端

- 本地/桌面用户在设置页看到 `Backup & Restore`：
  - 最近一次备份时间、大小、是否加密、是否覆盖 config/secrets。
  - `Create backup`、`Preview restore`、`Restore to new workspace`。
- 升级前提示：“建议创建本地备份；备份不上传云端。”
- 恢复前展示冲突：当前已有 `web` actor、已有公司画像、已有 session SQLite，用户选择替换或另存。
- public/remote 用户不暴露服务器路径，只能请求 actor 数据导出；整机备份仅对本地 admin/desktop/self-host 开放。

### 管理端

- Admin Settings 增加本机 backup status，显示：
  - `config.yaml` 是否纳入备份。
  - session backend 是 JSON 还是 SQLite。
  - company profile、portfolio、cron、web auth、LLM audit、uploads 的估算大小。
  - 最近备份是否通过 manifest 校验。
- 升级、cleanup、session backend cutover 前提供 backup dry-run 链接。
- 备份/恢复操作写入本地 audit log：操作者、范围、manifest hash、结果和失败域。

### 桌面端

- Desktop bundled 模式在 destructive actions 前调用同一备份服务：
  - 清理 runtime data 前建议备份。
  - 切换 bundled/remote 或重大升级前生成 restore point。
  - 恢复后重新生成 effective config 并重启 sidecars。
- 桌面只显示用户语义，例如“本机数据”“配置和密钥”“研究记忆”“会话历史”，不要求用户理解 `app_data_dir`、`app_config_dir` 或 WAL 文件。

### 多渠道

- IM channel 不直接触发整机恢复。
- 用户在 IM 中询问“怎么备份 Hone”时，admin actor 可以得到当前本机备份状态和 CLI 命令；普通用户只能看到数据导出/隐私入口。
- 恢复后 channel target、cron job 和 notification prefs 必须进入 readiness check，避免恢复到新机器后继续向旧平台目标发送。

## 技术方案

### 1. 备份域 registry

新增 `memory` 或 `hone-core` 级备份 registry，描述每个 domain 的备份策略：

```rust
pub struct BackupDomainDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub source_kind: BackupSourceKind,
    pub default_include: bool,
    pub contains_secrets: bool,
    pub restorable: RestoreMode,
    pub consistency: ConsistencyRequirement,
}
```

第一版 domains：

- `canonical_config`: `config.yaml`，默认包含但必须加密或要求用户显式选择明文。
- `soul_profile`: `soul.md`。
- `effective_config`: 默认不包含，只记录可再生成。
- `sessions_json`: `data/sessions/*.json`。
- `sessions_sqlite`: `sessions.sqlite3` 及 WAL/checkpoint backup。
- `portfolio`: `storage.portfolio_dir`。
- `cron_jobs` 与 `cron_history`。
- `notification_prefs`。
- `company_profiles`: actor sandbox `company_profiles/`。
- `public_web_auth`: web auth SQLite，含 session/API key hash，不导出明文 key。
- `uploads_documents`: public uploads、actor uploads、未来 document inbox。
- `research_artifacts`: 若 Research Artifact Library 落地则纳入。
- `llm_audit`: 默认只纳入 metadata 或加密完整包。
- `skill_registry` 与 `custom_skills`。
- `logs`: 默认不包含，可用于支持场景选择性加入。
- `release_state`: 只记录版本、install root、current release，不打包 release binaries。

### 2. 备份包格式

建议输出 `.honebackup`，内部是 zip 或 tar.zst：

```text
hone-backup/
  manifest.json
  README.md
  config/config.yaml.enc
  config/soul.md
  data/sessions/json/...
  data/sessions/sqlite/sessions.sqlite3
  data/portfolio/...
  data/cron/...
  data/notification_prefs/...
  actor_sandboxes/<actor_key>/company_profiles/...
  actor_sandboxes/<actor_key>/uploads/...
  skills/skill_registry.json
  custom_skills/...
  audit/llm_audit.redacted.jsonl
```

`manifest.json` 记录每个文件的 sha256、原 domain、相对路径、大小、mtime、是否加密和 restore policy。包内不得写入原机器绝对路径；路径只作为相对逻辑标签。

加密策略：

- 默认要求用户输入 passphrase，使用成熟 AEAD 包装 config/secrets 域。
- 如果用户选择明文备份，CLI 和 UI 必须明确提示包含 API keys/channel tokens。
- 备份 manifest 可明文，但 secrets 域和敏感 audit payload 加密。

### 3. SQLite 一致性

对 SQLite domain 不直接复制运行中文件：

- 首选通过 SQLite backup API 或 `VACUUM INTO` 到临时文件。
- 备份前可要求停止 sidecars，或在 UI 中提示“需要暂停本地服务 5-15 秒”。
- 对 WAL 模式数据库执行 checkpoint 或使用连接级 backup，manifest 记录 `sqlite_consistency=online_backup|offline_copy|skipped`.
- 恢复时先写入临时路径，校验 `PRAGMA integrity_check` 后再替换。

### 4. CLI/API/Desktop 入口

CLI：

```shell
hone-cli backup create --output ~/Hone-20260517.honebackup
hone-cli backup create --before-upgrade
hone-cli backup inspect ~/Hone-20260517.honebackup
hone-cli backup restore --preview ~/Hone-20260517.honebackup
hone-cli backup restore --home ~/.honeclaw-restored ~/Hone-20260517.honebackup
```

Web/admin API：

- `GET /api/backup/status`
- `POST /api/backup/create-preview`
- `POST /api/backup/create`
- `POST /api/backup/restore-preview`
- `POST /api/backup/restore`

Desktop commands：

- `create_backup`
- `preview_restore_backup`
- `restore_backup_to_new_home`

Restore 入口必须 admin-only/local-only。非本地 public deployment 不暴露整机 restore。

### 5. 与现有提案和运行契约的衔接

- Update Compatibility Center 在 `update apply` 前调用 `backup create --before-upgrade`，并把 backup manifest hash 写入 compatibility report。
- User Data Trust Center 继续处理单 actor 的隐私导出/删除；Backup Vault 处理本机工作区恢复，不作为给用户的数据权利包。
- Redacted Support Bundle 可引用 backup manifest 摘要，但不附带可恢复数据。
- Cleanup 在删除前提示最近备份状态；`--all` 可增加 `--backup-first`。
- Session SQLite cutover 前创建 restore point，失败时可恢复 JSON/SQLite 两域。

## 实施步骤

### Phase 1: Manifest-only dry-run

- 新增 backup domain registry 和 `BackupPlan`。
- `hone-cli backup create --dry-run --json` 输出会纳入/跳过的域、估算大小、secret 标记和一致性要求。
- Admin Settings 显示只读 backup readiness。
- 不实际打包，不恢复。

### Phase 2: Create encrypted backup

- 实现 `.honebackup` 打包、manifest hash、config/secrets 加密。
- 实现 SQLite online backup 或 offline stop-and-copy 策略。
- 增加 `backup inspect` 和 manifest 校验。
- 单元测试覆盖路径净化、manifest hash、secret domain 默认加密。

### Phase 3: Restore preview and new-home restore

- 实现 `restore --preview`。
- 先支持恢复到空的 `--home` 目录，不覆盖当前生产目录。
- 恢复后运行 config materialize、session integrity check、company profile scan 和 cron target readiness。
- 桌面 UI 支持“恢复到新本机工作区”。

### Phase 4: In-place restore points

- 在升级、cleanup、session backend cutover 和 desktop destructive action 前创建 restore point。
- 支持 in-place restore，但必须要求停机、dry-run、二次确认和 post-restore verification。
- 增加备份 retention 策略，例如保留最近 5 个 restore points 或 14 天。

## 验证方式

- Rust 单元测试：
  - backup domain registry 不包含任意绝对路径。
  - manifest hash 与包内文件一致。
  - secret domains 默认要求加密。
  - 恢复 preview 能识别版本、schema、actor 和路径冲突。
- 集成测试：
  - 构造临时 `HONE_HOME`，包含 config、sessions JSON/SQLite、portfolio、cron、company profiles、skill registry，创建备份后恢复到新目录，校验核心文件和 SQLite integrity。
  - SQLite WAL 场景下备份不丢最近写入。
  - `effective-config.yaml` 不作为权威恢复，恢复后可重新生成。
- 回归脚本：
  - `tests/regression/ci/test_backup_manifest_contract.sh`：无外部账号、可重复、校验 dry-run 和 inspect。
  - 恢复真实桌面目录或大文件上传保留在 manual lane。
- 手工验收：
  - macOS desktop bundled 创建备份、恢复到新目录、启动 backend。
  - Homebrew 安装版升级前创建 restore point，并在失败模拟后恢复。

## 风险与取舍

- 风险：备份包包含密钥和敏感投资数据。  
  取舍：默认加密 secrets 域，明文模式必须显式确认；manifest 不含原机绝对路径。

- 风险：恢复覆盖当前数据可能造成二次损坏。  
  取舍：第一版只支持 new-home restore；in-place restore 等有足够验证后再开放。

- 风险：整包备份与 User Data Trust Center 重叠。  
  取舍：本提案面向本地工作区灾备；Data Trust Center 面向单 actor 权利、导出和删除。两者共享 domain registry 思想，但输出包和权限不同。

- 风险：SQLite online backup 增加实现复杂度。  
  取舍：优先支持少数关键 SQLite domain；不能一致备份的 domain 在 manifest 中标 `skipped_needs_shutdown`，不假装成功。

- 风险：跨版本恢复可能引入 schema 兼容问题。  
  取舍：恢复前强制跑 compatibility verdict；跨 major/schema restore 默认只允许恢复到新 home 并运行迁移检查。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 下全部自动提案和历史 `docs/proposals/`：

- 不重复 `auto_p1_user-data-trust-center.md`：该提案关注单 actor 隐私、导出、删除和数据权利；本提案关注本地安装/桌面工作区的可恢复备份、restore point、SQLite 一致性和整机迁移。
- 不重复 `auto_p1_update-compatibility-center.md`：该提案解决版本、API、安装方式、升级和二进制回滚；本提案保护升级/cleanup/cutover 前后的用户数据状态。
- 不重复 `auto_p1_redacted-support-bundle.md`：诊断包用于安全分享排障证据，不应包含可恢复的完整数据；备份包用于用户自己保存和恢复。
- 不重复 `auto_p1_research_artifact_library.md` 或 `auto_p1_investment_document_inbox.md`：它们把研究报告和上传材料产品化为独立资产；本提案只把这些资产作为未来备份域纳入。
- 不重复历史 `desktop-bundled-runtime-startup-ux.md`：桌面启动提案处理进程锁、自动接管和 sidecar 恢复；本提案处理应用数据备份与恢复。
- 不重复历史 `skill-runtime-multi-agent-alignment.md`：skill runtime 提案处理技能披露、权限和 runner 语义；本提案只备份 skill registry/custom skills 状态。

查重结论：现有 proposal 已覆盖数据权利、版本兼容、诊断、研究资产和文档资产，但没有覆盖“本地 Hone 工作区可恢复备份与 restore 演练”这一独立产品/架构层。这个主题能降低升级、迁移、误删和桌面数据损坏的核心风险，适合作为 P1 底座提案。

## 文档同步说明

本轮只新增 proposal，不开始实现，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/local-backup-restore-vault.md`，并在新增 CLI/API/Desktop 入口、备份域 registry、SQLite 备份策略或 cleanup/update 前置流程时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
