# 安全审计：公开面认证与限流安全隐患

- 严重等级: P1（高风险子项）/ P2（中风险子项）
- 状态: Fixing
- 发现日期: 2026-04-20
- 审计范围: 2026-04-17 ~ 2026-04-20 提交（`9ec634e`、`38d1b8e`、`29096d2`、`73ed37c` 等）

## 概述

对公开面（public web）认证、限流和 workflow runner 调用链路做安全审计，发现 6 个安全或质量问题。其中 3 个为高风险，可直接被外部攻击者利用；3 个为中风险，需择期修复。

---

## 🔴 高风险

### 1. Rate Limiter 可被伪造 Header 绕过（邀请码暴力破解）

- **文件**: `crates/hone-web-api/src/routes/public.rs:317-332` → `public_client_key()`
- **涉及 Commit**: `38d1b8e`

`public_client_key()` 使用 `X-Forwarded-For` / `X-Real-IP` 作为 rate limit key。攻击者可以在每次请求中发送不同的伪造 header 值，从而完全绕过暴力破解限制（8 次/10 分钟窗口）。

**建议**:
- 如果前面有受信反向代理，应由代理层覆盖 `X-Forwarded-For`，后端取最右侧可信 IP。
- 如果直接暴露，使用 Axum 的 `ConnectInfo<SocketAddr>` extractor 获取 TCP 连接源地址。
- 不应信任请求中可被攻击者控制的 header。

### 2. Secure Cookie Flag 依赖可伪造请求头

- **文件**: `crates/hone-web-api/src/routes/public.rs:269-274` → `request_is_secure()`
- **涉及 Commit**: `38d1b8e`

`build_session_cookie` 是否设置 `Secure` 取决于 `x-forwarded-proto`、`Origin`、`Referer` 等可伪造的请求头。攻击者在 HTTP 明文环境下可伪造 `Origin: https://...` 导致误判；在 HTTPS 环境下也可能因缺少 header 而漏判。

**建议**:
- HTTPS 判断应在部署配置层面确定（环境变量 / 编译选项），不应逐请求依赖 header。

### 3. Workflow Runner `validateCode` 鉴权被移除

- **文件**: `crates/hone-channels/src/core.rs:1254-1262` → `build_report_run_input()`
- **涉及 Commit**: `29096d2`

`build_report_run_input` 不再发送 `validateCode` 字段给 workflow runner API (`/api/runs`)。`workflow_runner_request`（第 715 行起）也没有设置任何 auth header。如果 runner 没有其他鉴权机制（如网络隔离、API key header），则任何可达该地址的人都能触发研报生成。

**建议**:
- 确认 workflow runner 是否有其他层面的访问控制（网络层隔离、API key、mTLS）。
- 如果没有，需要恢复或替换为新的鉴权方案。

---

## 🟡 中风险

### 4. 邀请码只有 48 bits 熵

- **文件**: `memory/src/web_auth.rs:392-395` → `generate_invite_code()`

邀请码取 UUID v4 前 12 个 hex 字符（48 bits 熵），格式 `HONE-XXXXXX-YYYYYY`。配合 rate limiter 可绕过（#1），在分布式攻击场景下可能被暴力搜索。

**建议**: 增长邀请码到至少 16-20 字符（80-100 bits 熵），或先修复 rate limiter。

### 5. `ensure_column` 使用字符串拼接 SQL DDL

- **文件**: `memory/src/web_auth.rs:499-509` → `ensure_column()`

```rust
conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"), [])
```

当前所有调用参数均为硬编码字面值（第 93、99 行），不存在运行时注入风险。但此模式本身不安全——如果未来有人用外部输入调用此函数就会产生 SQL 注入。

**建议**: 加 `// SAFETY:` 注释标记参数必须为硬编码值，或对 table/column 做 identifier quoting。

### 6. 公开聊天登录态 `ready` 提前设置导致空白闪烁

- **文件**: `packages/app/src/pages/chat.tsx:382-386`
- **涉及 Commit**: `73ed37c`

`setAuthState("ready")` 被移到 `getPublicHistory()` 之前（第 382 行），UI 先切到聊天界面但消息列表为空，直到历史加载完成（第 384 行起）才显示。

**建议**: 将 `setAuthState("ready")` 移回历史加载完成之后，或增加 loading skeleton。

---

## ✅ 做得好的地方

- ZIP 画像包导入做了完整的路径遍历防护（`parse_bundle_entry_path` + `Component::Normal` 校验）
- Session 管理采用单一 session 策略，每次登录清理旧 session，避免 session fixation
- 公开端口移除了宽松 CORS（`allow_origin(Any)`），默认同源策略
- 邀请码登录现在要求邀请码 + 手机号双因素匹配
- 支持邀请码停用 / 重置，停用时自动清理登录态

---

## 修复优先级

1. **立即**: 修复 rate limiter IP 获取（#1）—— 这直接决定暴力破解防护是否有效
2. **尽快**: 确认 workflow runner 鉴权（#3）
3. **短期**: 改为部署配置决定 Secure flag（#2）
4. **择期**: 增加邀请码熵（#4）、标记 ensure_column 安全注释（#5）、修复 UX 闪烁（#6）
