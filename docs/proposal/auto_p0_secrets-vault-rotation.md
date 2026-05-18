# Proposal: Secrets Vault and Credential Rotation Center

status: proposed
priority: P0
created_at: 2026-05-18 08:03:34 +0800
owner: automation

related_files:

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/current-plans/canonical-config-runtime-apply.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/proposal/auto_p1_local-backup-restore-vault.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `config.example.yaml`
- `crates/hone-core/src/config/mutation.rs`
- `crates/hone-web-api/src/routes/channel_settings.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `memory/src/web_auth.rs`
- `bins/hone-desktop/src/sidecar/settings.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `packages/app/src/pages/settings.tsx`
- `crates/hone-web-api/src/lib.rs`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 现在已经把用户工作流扩展到 Web、桌面、CLI、Hone Cloud API、Feishu、Telegram、Discord、iMessage、event-engine、FMP/Tavily、OpenRouter/OpenAI-compatible providers、OpenCode/Codex ACP 和 Aliyun SMS/Captcha。也就是说，产品可信度不仅取决于 agent 回答质量，还取决于一组长期有效的外部凭证是否被安全保存、最小暴露、可轮换、可审计、可恢复。

当前仓库已经完成了几项重要基础：

- `docs/decisions.md` 的 `D-2026-05-11-01` 明确 LLM credentials 走 config-only，不再从父进程环境变量兜底读取。
- `config.example.yaml` 把 Feishu app secret、Telegram/Discord bot token、FMP key pool、OpenRouter provider keys、Hone Cloud API key 等放进 canonical config 体系。
- `crates/hone-core/src/config/mutation.rs` 已有敏感路径识别与脱敏辅助，能按 `api_key` / `secret` / `token` / `password` 等关键词判断敏感字段。
- `memory/src/web_auth.rs` 对公开 Web 用户 API key 采用 hash + prefix + 一次性明文返回，已经证明“用户 API key 可以不以明文长期保存”。
- `crates/hone-web-api/src/routes/web_users.rs` 支持管理员为 public user 获取或重置 API key，并提醒明文仅显示一次。
- `docs/current-plans/canonical-config-runtime-apply.md` 明确当前密钥仍留在 YAML，本轮不引入系统 keychain。这说明 keychain/vault 是一个尚未解决的后续架构层。

但当前普通 runtime / channel / provider secrets 仍是“明文配置字段”模型：

- `crates/hone-web-api/src/routes/channel_settings.rs` 的 `ChannelSettings` 会把 `feishu_app_secret`、`telegram_bot_token`、`discord_bot_token` 从 `config.yaml` 读出并通过 GET 返回给管理端。
- `bins/hone-desktop/src/sidecar/settings.rs` 和 `bins/hone-desktop/src/sidecar.rs` 也会把 channel token、agent API key、OpenRouter/FMP/Tavily key pool 读出并返回给桌面设置页。
- Settings UI 主要依赖 password input / copy flows 来降低误看风险，但 API 响应里仍可能包含完整 secret。
- `apply_config_mutations` 可以脱敏日志展示，但写入源仍是 YAML 明文字段，且缺少 secret id、版本、创建时间、last used、rotation 状态、权限边界和过期策略。
- `crates/hone-web-api/src/lib.rs` 等 runtime 路径直接从 `HoneConfig` 读取 secret 值来构建 sinks / clients，尚未有统一 resolver 或 secret reference 层。

这不是单纯“配置文件明文是否可接受”的问题。Hone 的定位是长期投资助理，沉淀的是用户研究资产、持仓上下文、自动化推送和多渠道工作流；一旦某个管理端 bearer token、桌面 profile、日志包、备份包、远端浏览器会话或本机磁盘权限泄露，外部 bot token、LLM provider key、Hone Cloud key、market data key 会同时变成横向移动入口。

## 问题或机会

这是 P0，因为它直接影响数据安全、付费/试用入口、外部渠道控制权和运维可信度。已有 `operator-access-audit` 能限制“谁能操作”，但如果 secret 本身仍在普通 GET 设置响应和 YAML 中长期明文往返，权限审计只能降低误用，不能降低 secret 生命周期风险。

主要问题：

1. **管理端和桌面设置读取 secret 时暴露完整明文。**
   当前 channel settings / agent settings / provider key settings 会把完整 token/key 作为设置 DTO 返回。只要前端状态、浏览器调试、远端 bearer、桌面插件、日志或截图泄露，就可能泄露 bot token、provider key 或 Feishu secret。

2. **配置源缺少 secret identity。**
   `config.yaml` 里只有具体值，没有 `secret_id`、prefix、hash、created_at、rotated_at、last_used_at、owner、scope、source、expires_at。系统无法回答“这个 key 是什么时候加的、哪些 route 用过、是否该轮换、旧 key 是否还能用”。

3. **多份 key pool 只有值列表，没有可控轮换。**
   OpenRouter、FMP、Tavily 已经支持 key pool，但 pool 元素没有独立状态。无法将某个 key 标记为 draining / disabled / compromised，也无法做灰度轮换、失败隔离或成本归因。

4. **外部凭证和内部 public API key 的治理水平不一致。**
   `memory/src/web_auth.rs` 已经把 public user API key 做成 hash + prefix + 一次性明文；而 provider/channel/admin side secrets 仍长期明文留在 config 和设置响应里。两个模型并存会让产品安全边界不一致。

5. **备份、诊断、readiness、support bundle 都会被 secret 明文拖累。**
   `redacted-support-bundle` 和 `local-backup-restore-vault` 可以对导出物做脱敏/加密，但源头仍是明文。如果没有 secret refs 和统一 resolver，每个新诊断面都要重复处理 raw key，漏一个就会泄露。

6. **商业化部署缺少凭证生命周期控制。**
   Hone Cloud、自托管团队版、桌面 remote backend 和开源本地安装面对的 credential 策略不同。没有 Vault/Rotation Center，后续付费用户无法自助判断 key 是否健康、何时过期、如何无停机轮换、哪些渠道将受影响。

## 方案概述

新增 **Secrets Vault and Credential Rotation Center**：把外部 provider/channel/runtime credentials 从普通明文配置值升级为“secret reference + vault item + scoped resolver”的架构层。

目标不是立刻引入重型外部 KMS，而是先把 secret 生命周期产品化：

- `config.yaml` 继续是用户可理解的 canonical 配置源，但 secret 字段逐步支持 `secret_ref`，而不是直接保存 raw value。
- 本地/桌面默认使用 OS keychain 或加密本地 vault；server/self-host 默认使用本地 encrypted vault 文件，可选接入环境注入或外部 secret manager。
- 所有设置 GET API 只返回 prefix、created_at、last_used_at、health、scope 和 rotation 状态，不返回 raw secret。
- 写入/轮换 secret 时，明文只在 POST/command 入参里出现一次，落库后返回 ref/prefix，不再回显。
- Runtime 构建 provider/channel client 时通过 `SecretResolver` 拉取明文，并记录使用事件。
- 管理端和桌面提供 Credential Center：按用途显示 LLM、market data、channel、Hone Cloud、SMS/Captcha 等凭证健康、最近使用、轮换动作和受影响能力。

第一版优先保护最高风险的“会通过 UI/API 往返的 secrets”：channel tokens、agent/provider keys、FMP/Tavily key pools、Hone Cloud key。Aliyun SMS/Captcha 当前主要来自 server env/config，可纳入 inventory，但不必第一阶段改造所有部署方式。

## 用户体验变化

### 用户端

- Public Web 用户不接触 provider/channel secret；如果 public chat 因服务端凭证失效不可用，只看到稳定原因码和“服务配置需要管理员处理”。
- Hone Cloud API key 用户继续看到自己的 key prefix、last used、rotate 操作；这部分可复用已有 hash + prefix 模型，不和 provider vault 混淆。
- 当用户在多渠道收到“暂不可用”提示时，文案不暴露底层 token 或 provider 错误原文，只给 support reference / reason code。

### 管理端

- Settings 新增 `Credentials` 或 `Security > Credentials` 面板：
  - 按 domain 分组：LLM providers、Hone Cloud、market data、search、Feishu、Telegram、Discord、Aliyun SMS/Captcha、service tokens。
  - 每个 secret 显示 label、prefix、scope、created_at、last_used_at、last_probe_at、status、used_by、rotation_due。
  - 高危动作如 reveal 被默认移除；只有 `replace` / `rotate` / `disable` / `test` / `delete unused`。
- 现有 channel settings 页面不再显示完整 token。字段变为：
  - `Configured: sk-...abcd`
  - `Replace token`
  - `Test connection`
  - `Disable channel`
- Key pool 支持单 key 状态：
  - active
  - draining
  - disabled
  - failed_recently
  - compromised
  - rotation_due
- 保存配置后，页面明确说明受影响 route / channel 是否需要重启，复用 `ConfigApplyPlan` 的 live/restart 语义。

### 桌面端

- Desktop bundled 默认将 secrets 存到系统 keychain 或 app-scoped encrypted vault，不把 raw key 回填进 React settings state。
- Remote mode 明确显示“凭证保存在远端 backend”，桌面只发起远端 replace/probe 请求，不在本机保存远端 provider key。
- 本地离线用户仍可以导出/备份 vault，但默认要求 passphrase；backup restore 时先恢复 secret refs，再提示用户重新输入无法迁移的 keychain-backed secrets。

### 多渠道

- Feishu/Telegram/Discord listener 启动时通过 resolver 获取 token，并把 `secret_id` 写入 heartbeat 摘要，不写明文。
- Channel 状态可以显示“token present / token invalid / app secret rotation due / last successful auth”，而不是只有 enabled + process alive。
- 轮换 token 时支持先保存新 secret、probe 成功、重启目标 channel，再把旧 secret 标记为 disabled，降低推送中断风险。

### 运维和商业化

- Self-host 管理员可以导出 credential inventory，不含明文，用于支持和合规审查。
- 未来团队版可以给不同 operator scope：只允许替换某类 secret，不允许读取明文。
- 付费转化和试用运营可以安全地展示“服务配置健康”，而不把 provider key 暴露给普通 support 角色。

## 技术方案

### 1. Secret 数据模型

新增共享纯类型，建议放在 `hone-core`，存储实现可放在 `memory` 或 `hone-core::secrets`：

```rust
pub struct SecretRef {
    pub id: String,
    pub domain: SecretDomain,
}

pub enum SecretDomain {
    LlmProvider,
    AgentRunner,
    MarketData,
    Search,
    Channel,
    SmsCaptcha,
    ServiceToken,
}

pub struct SecretMetadata {
    pub id: String,
    pub label: String,
    pub domain: SecretDomain,
    pub purpose: String,
    pub prefix: Option<String>,
    pub value_hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
    pub last_probe_at: Option<String>,
    pub expires_at: Option<String>,
    pub status: SecretStatus,
    pub used_by: Vec<String>,
}
```

`SecretStatus` 第一版覆盖：

- `active`
- `draining`
- `disabled`
- `invalid`
- `compromised`
- `rotation_due`
- `unknown`

Secret value 存储：

- 本地/桌面：优先 OS keychain；如果不可用，使用 app data dir 下 encrypted vault 文件。
- Server/self-host：默认 encrypted vault 文件；可选 `external_ref` 接外部 secret manager，但不作为第一版硬依赖。
- Source checkout dev：允许 fallback encrypted file；不要把明文写回 `config.yaml`。

### 2. Config 兼容策略

为了避免一次性破坏配置，secret 字段支持两种输入：

```yaml
telegram:
  bot_token: "legacy-plaintext"
  bot_token_ref: "secret:channel:telegram_bot:abc123"
```

Resolver 规则：

1. 如果存在 `*_ref`，优先从 vault 解析。
2. 如果只有 legacy plaintext，runtime 仍可用，但 readiness 给出 `legacy_plaintext_secret` warning。
3. Migration 命令可以把 legacy plaintext 导入 vault，写入 `*_ref`，并把原字段清空。
4. 第一阶段不删除 legacy 字段解析，避免破坏现有安装。

适用字段第一批：

- `agent.hone_cloud.api_key`
- `agent.opencode.api_key`
- `agent.multi_agent.search.api_key`
- `agent.multi_agent.answer.api_key`
- `llm.auxiliary.api_key`
- `llm.providers.*.api_key`
- `llm.providers.*.api_keys`
- `fmp.api_key` / `fmp.api_keys`
- `search.api_keys`
- `feishu.app_secret`
- `telegram.bot_token`
- `discord.bot_token`
- `web.auth_token`

Aliyun SMS/Captcha env credentials第二阶段处理：先纳入 inventory 和 readiness，不强迫迁移。

### 3. SecretResolver

新增统一 resolver：

```rust
pub trait SecretResolver {
    fn resolve(&self, reference: &SecretRef) -> HoneResult<ResolvedSecret>;
    fn metadata(&self, reference: &SecretRef) -> HoneResult<SecretMetadata>;
    fn record_use(&self, reference: &SecretRef, purpose: &str) -> HoneResult<()>;
}
```

`HoneConfig` 不直接暴露 raw secret 给设置 API。Runtime client 构建点调用 resolver：

- `crates/hone-web-api/src/lib.rs` 构建 Feishu / Telegram / Discord sink 时解析 channel secret。
- runner / LLM provider 构建点解析 provider secret。
- FMP/Tavily tool client 解析 key pool，并按 key id 记录成功/失败。
- `hone-cli doctor` / Runtime Readiness Matrix 读取 metadata，不读取 raw value；probe 需要 raw value 时由 resolver 临时提供。

### 4. 设置 API 改造

Settings GET 响应只返回：

```json
{
  "telegramBotToken": {
    "configured": true,
    "secretId": "secret:channel:telegram:...",
    "prefix": "123456...",
    "createdAt": "...",
    "lastUsedAt": "...",
    "status": "active"
  }
}
```

Settings PUT/POST 支持：

- `replace`: 接收新明文，写入 vault，更新 config ref。
- `clear`: 禁用并清空 ref。
- `rotate`: 写入新 secret，旧 secret 标记 draining，probe 成功后禁用旧 secret。
- `probe`: 用 resolver 临时取值，执行最小连接测试。

兼容期可保留旧 payload 字段，但服务端收到空字符串时必须解释为“不改变现有 secret”，而不是把 token 清空；收到非空 raw value 时创建新 vault item。

### 5. Key pool 轮换

对 pool 型凭证，引入 `SecretPoolRef`：

```yaml
fmp:
  api_key_refs:
    - secret:market_data:fmp:primary
    - secret:market_data:fmp:backup
```

Key pool metadata 记录：

- per-key status
- last_success_at
- last_failure_at
- failure_count
- quota_exhausted_until
- cost/account label

Runtime 选择 key 时过滤 `disabled/compromised/draining`，并把 provider 返回的 invalid/quota 错误写回 metadata。这样 readiness 可以区分“没有 key”“key present but invalid”“pool 有备 key但主 key失败”。

### 6. 审计与权限

本提案不重复实现 operator 身份体系，但应给 `auto_p0_operator-access-audit.md` 留好事件对象：

- `secret.create`
- `secret.replace`
- `secret.rotate_start`
- `secret.rotate_complete`
- `secret.disable`
- `secret.delete`
- `secret.probe`
- `secret.legacy_migrate`

Audit event 只记录 secret id、domain、prefix、hash、scope、operator、reason、result，不记录 raw value。

在 operator scope 落地前，远端 admin 至少要把 secret write/probe 与普通 read 分离：

- `credentials.read_metadata`
- `credentials.write`
- `credentials.rotate`
- `credentials.probe`
- `credentials.delete`

### 7. 迁移命令

新增 CLI：

```shell
hone-cli secrets inventory --json
hone-cli secrets migrate-legacy --dry-run
hone-cli secrets migrate-legacy
hone-cli secrets rotate <secret-id>
hone-cli secrets probe <secret-id>
```

迁移输出：

- 将迁移哪些字段
- 是否发现 legacy plaintext
- 将写入哪个 vault backend
- 是否需要重启 backend/channel
- 是否有不可迁移的 env-only secret

首次迁移必须支持 dry-run；实际写入后保留自动备份，并把旧 plaintext 字段清空或替换为 empty + ref。

## 实施步骤

### Phase 1: Secret inventory 和只读元数据

- 定义 secret metadata / status / domain / refs。
- 扫描 `HoneConfig` 中所有敏感字段，生成 inventory。
- Settings GET 先增加 metadata 字段，同时保留旧字段，验证 UI 能展示 configured/prefix/status。
- Runtime Readiness Matrix 可引用 inventory 判断 legacy plaintext / missing / invalid / rotation due。

### Phase 2: Vault backend 与 resolver

- 实现本地 encrypted vault 文件 backend，桌面优先接 OS keychain 的 adapter。
- 增加 `SecretResolver`，先接 channel token、Hone Cloud key、OpenRouter/FMP/Tavily key pool。
- Runtime client 构建点从 resolver 获取 secret，不再依赖设置 API 返回 raw value。
- 增加 `hone-cli secrets inventory/migrate-legacy --dry-run`。

### Phase 3: Settings replace/rotate flow

- 改造 Web 管理端 channel/agent/provider settings：
  - GET 不返回明文。
  - POST replace 创建 vault item。
  - 空字段表示 keep existing。
  - 明确 restart/live apply 影响面。
- 改造 Desktop settings，同步处理 bundled/remote 差异。
- 对 key pools 增加 per-key disable/drain/probe。

### Phase 4: Legacy migration 和审计联动

- 提供 `hone-cli secrets migrate-legacy` 和桌面引导。
- 将旧明文字段迁到 `*_ref` / `*_refs`。
- 接入 operator audit 事件，记录 secret lifecycle 操作。
- Support bundle、backup vault、update compatibility、readiness 统一读取 secret metadata，不读取 raw value。

### Phase 5: 外部 secret manager 可选接入

- 为 server/self-host 增加可选 `external_ref` provider。
- 文档化如何接 Vault、1Password CLI、AWS/GCP/Azure secret manager 或 Kubernetes secret，但不把任何一个作为默认依赖。
- 对团队部署加入 rotation policy：过期提醒、强制 probe、定期禁用未使用 secret。

## 验证方式

### 单元测试

- `is_sensitive_config_path` 覆盖新增 `*_ref` / `*_refs` / token/key path，不把 ref 当 raw value 输出。
- Legacy config 只有 `telegram.bot_token` 时 inventory 标记 `legacy_plaintext_secret`。
- 同时存在 `telegram.bot_token_ref` 和 `telegram.bot_token` 时 resolver 优先 ref。
- Settings PUT 空 secret 字段保持旧 secret，不把 vault item 清空。
- Key pool 禁用一个 key 后 runtime selection 不再选它。
- `migrate-legacy --dry-run` 不写 config/vault，输出 planned changes。
- `migrate-legacy` 写入 ref 后，旧 raw field 被清空或不再由 runtime 使用。

### 集成测试

- Web channel settings GET 不包含 raw `feishu_app_secret`、`telegram_bot_token`、`discord_bot_token`。
- Desktop agent settings GET 不包含 raw Hone Cloud / OpenRouter / FMP / Tavily key。
- 用 vault ref 启动 Telegram/Discord/Feishu sink，能解析 token 并通过已有启动检查。
- Hone Cloud runner 使用 `agent.hone_cloud.api_key_ref` 时能完成一次最小 mock chat。
- FMP/Tavily key pool refs 能被 tool client 解析，invalid key 会更新 metadata failure。

### 回归脚本

- 新增 CI-safe fixture regression：给一份含假 secret 的 config，执行 inventory/migrate dry-run，断言输出无 raw secret。
- 新增 manual regression：本机 vault + desktop settings replace/probe + backend restart，验证 UI 不回显明文。
- 支持 bundle / backup / readiness 的 fixture 检查：导出物只含 prefix/secret id/status，不含 raw key。

### 手工验收

- 管理端新增 Telegram token 后，页面刷新只显示 prefix 和 configured。
- 用户保存空 token 字段不会误删原 token。
- 用户点击 replace 并输入新 token，旧 token 进入 draining 或 disabled，新 token probe 成功后 channel 重启。
- 复制浏览器 network response，不应看到 raw provider key、bot token 或 app secret。
- Desktop remote mode 不把远端 secret 存入本地 profile。

### 指标

- settings GET raw secret exposure count 降为 0。
- secret inventory 中 legacy plaintext 数量随迁移下降。
- provider/channel auth failure 能按 secret id 归因。
- rotation 操作成功率、probe 成功率、因 invalid secret 导致的 runtime failure 数下降。

## 风险与取舍

- **风险：引入 vault 后配置和排障更复杂。**
  取舍：第一阶段保留 legacy plaintext 兼容和 dry-run inventory，不强制一次迁移；UI 用 prefix/status 降低理解成本。

- **风险：OS keychain / encrypted file backend 跨平台差异大。**
  取舍：桌面优先 keychain；CLI/server 先用 encrypted file vault；外部 secret manager 留到后续阶段。

- **风险：secret ref 丢失会导致 runtime 无法启动。**
  取舍：migration 必须备份原 config；readiness 提前报 `missing_secret_ref`；backup vault 明确记录 refs 和是否包含可恢复密文。

- **风险：某些第三方 SDK 需要明文 token 常驻内存。**
  取舍：resolver 只在 client 构建或 probe 时提供明文；无法避免进程内内存存在，但可以避免 API/UI/log/backup 中长期明文暴露。

- **风险：rotation flow 可能中断 channel 推送。**
  取舍：引入 draining/probe/restart 顺序；probe 失败不替换 active secret；channel restart 走现有 apply plan。

- **风险：与 operator access audit 重叠。**
  取舍：operator audit 解决“谁可以执行高危操作”；本提案解决“secret 存在哪里、如何引用、如何轮换、如何不回显明文”。两者必须衔接，但边界不同。

- **不做的边界：**
  第一版不实现企业级集中 KMS，不做跨设备自动同步，不承诺能从所有历史 logs/audit 中删除已泄露 secret，不自动替用户去第三方平台吊销旧 token。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并额外检查了当前活跃计划与相关 handoff。结论：本提案不重复，重点差异如下：

- 不重复 `auto_p0_operator-access-audit.md`：该提案建立 operator 身份、scope、session、service token 和操作审计；本提案建立 secret vault、secret ref、resolver、rotation、metadata 和不回显明文的配置模型。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness 判断凭证是否 present/valid；本提案定义凭证如何保存、引用、轮换和归因，readiness 只是消费者。
- 不重复 `auto_p1_redacted-support-bundle.md`：support bundle 解决诊断导出的脱敏；本提案在源头减少 raw secret 经 API/UI/配置往返的机会。
- 不重复 `auto_p1_local-backup-restore-vault.md`：backup vault 解决工作区备份和恢复；本提案解决 runtime credentials 的主存储、keychain/encrypted vault、refs 和 rotation。
- 不重复 `auto_p1_hone-cloud-api-contract.md`：该提案聚焦 public user API key 的开发者体验和自助 rotation；本提案覆盖 provider/channel/runtime/operator secrets，且借鉴其 hash + prefix + one-time plaintext 模型。
- 不重复 `auto_p1_update-compatibility-center.md`：compatibility 关心版本和安装窗口；本提案关心 secrets 生命周期，与版本升级只在 migration/backup/readiness 中相交。
- 不重复 `auto_p1_user-data-trust-center.md`：用户数据权利中心关注聊天、持仓、画像、上传和审计数据导出/删除；本提案关注外部服务凭证和运行密钥，不导出用户投资数据。

查重结论：现有 proposal 已覆盖权限审计、诊断脱敏、备份加密、readiness、public API key developer experience 和数据权利，但没有覆盖“provider/channel/runtime secrets 从明文配置字段升级为 secret refs + vault + resolver + rotation center”的核心安全底座。因此本主题是新的、可落地的 P0 产品/架构提案。

## 文档同步说明

本轮只新增 proposal，不开始执行改造，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/secrets-vault-rotation.md`，并在引入 secret ref、vault backend、settings API 行为变化、migration 命令或 operator audit event 后同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`、相关 runbook 和必要的 handoff/archive 索引。
