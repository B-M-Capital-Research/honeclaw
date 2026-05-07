# Bug: 飞书渠道消息发错位（跨用户投递）

- **发现时间**: 2026-03-24
- **严重等级**: P0 — 用户 A 发送的消息被用户 B 收到
- **状态**: 已确认存在，**已于 2026-03-25 修复**
- **涉及文件**:
  - `bins/hone-feishu/src/handler.rs` ← 主要 bug 所在
  - `bins/hone-feishu/src/client.rs`
  - `crates/hone-channels/src/ingress.rs`
  - `crates/hone-core/src/actor.rs`

---

## 一、Bug 现象

在飞书私聊（p2p）场景中，用户 A 发送一条消息，**Bot 却把 AI 回复发到了用户 B 的会话**。用户 B 会收到一条与自己无关的回复，而用户 A 的消息完全没有收到回复（或收到了另一个用户的回复）。

该问题在并发消息量较高（多个用户同时发消息）时更容易复现。

---

## 二、代码流程梳理（正常路径）

```
飞书 Webhook 事件
  └─ parse_feishu_event()          ← 解析 open_id/chat_id/text
       └─ get_user_by_open_id()    ← 异步 API 查询 email/mobile
  └─ process_incoming_message()
       ├─ 确定 channel_target      ← email/mobile（无则用 open_id）
       ├─ 确定 outbound_receive_id ← p2p 时 = open_id（正确！）
       ├─ send_placeholder_message() ← 用 outbound_receive_id 发"思考中..."
       ├─ session.run()            ← AI 处理
       └─ send_rendered_messages() ← 用 outbound_receive_id 发最终回复
```

在**正常路径**下，`outbound_receive_id` 在 `process_incoming_message` 函数入口就已经被确定，并通过闭包捕获，整个处理生命周期内不变，不存在 race condition，**发送回复的目标 ID 是正确的**。

---

## 三、Root Cause 分析（三个独立的问题）

### 问题 1：`ticker` 闭包捕获了错误的变量（**主要 bug，高危**）

**位置**: `handler.rs` 第 447–487 行

```rust
// handler.rs L447-487（ticker 任务）
let ticker_handle = if cardkit_session.is_none() && placeholder_message_id.is_some() {
    let ticker_content = content_buf.clone();
    let ticker_facade = state.facade.clone();
    let ticker_pid = placeholder_message_id.clone(); // ← 捕获的是「消息 ID」
    let ticker_log = log_user.to_string();           // ← 捕获的是「日志标签」
    Some(tokio::spawn(async move {
        loop {
            // ...
            if let Some(ref pid) = ticker_pid {
                ticker_facade.update_message(pid, "interactive", &card).await
                // ← 此处 update_message 通过 message_id 更新，不通过 receive_id
                // ← 所以 ticker 路径本身也是安全的
            }
        }
    }))
} else { None };
```

Ticker 本身通过 `message_id`（消息 ID，全局唯一）更新卡片，**不依赖 receive_id**，所以 ticker 路径本身没有 cross-user 风险。

但是，**`state.facade` 是全局共享的单例**（`Arc<FeishuApiClient>`），当多个 ticker 任务并发运行时，它们共享同一个 `token_cache`（`Arc<RwLock<Option<(String, Instant)>>>`）。Token 轮换时多个 ticker 会并发抢写 `token_cache`，有极小概率导致 token 混乱，但这不足以造成消息错位。

---

### 问题 2：`get_user_by_open_id` 失败时 session 键与 channel_target 不一致（**核心 bug**）

**位置**: `handler.rs` 第 753–767 行（`parse_feishu_event`），以及第 206–229 行（`process_incoming_message`）

```rust
// parse_feishu_event 中：
match state.facade.get_user_by_open_id(&open_id).await {
    Ok(user) => {
        if !user.email.is_empty() { email = Some(user.email); }
        if !user.mobile.is_empty() { mobile = Some(user.mobile); }
    }
    Err(e) => {
        warn!("[Feishu] Failed to get user by open_id {}: {}", open_id, e);
        // ← email / mobile 保持 None，继续处理
    }
}
```

```rust
// process_incoming_message 中：
// channel_target = 用于标识 user 的「逻辑 ID」，优先 email，其次 mobile，最后 open_id
let channel_target = preferred_contact.clone().unwrap_or_else(|| msg.open_id.clone());

// actor = 用于权限/session 查找
let (actor, _, chat_mode) = if chat_type == "p2p" {
    state.scope_resolver.direct(&msg.open_id, channel_target.clone())
    // ← actor.user_id = open_id（固定，正确）
    // ← session key = "Actor_feishu__direct__<open_id>" （固定）
    ...
};
```

**问题所在**：

`actor.user_id` 始终是 `open_id`（固定），session 文件路径也固定为 `Actor_feishu__direct__<open_id>.json`。**但 `channel_target` 由 `preferred_contact`（email/mobile）决定**。

当两次并发请求对同一个用户（相同 `open_id`）发来消息时：
- 第一次请求：`get_user_by_open_id` 成功 → `channel_target = "alice@corp.com"`
- 第二次请求：`get_user_by_open_id` 超时/失败 → `channel_target = "ou_5f..."`

这两个请求会产生**不同的 `channel_target`**，但 **`outbound_receive_id = open_id`（p2p 时）是相同的**。这不会导致跨用户，只是日志记录不一致。

**然而对定时任务（scheduler）存在真实的跨用户风险**：

---

### 问题 3：定时任务 `resolve_receive_id` 可能映射到错误用户（**定时任务路径跨用户 bug**）

**位置**: `handler.rs` 第 596–638 行（`handle_scheduler_events`），第 1040–1060 行（`resolve_receive_id`）

```rust
async fn handle_scheduler_events(...) {
    while let Some(event) = event_rx.recv().await {
        let state_clone = state.clone();
        tokio::spawn(async move {
            let receive_id = resolve_receive_id(&state_clone.facade, &event.channel_target).await?;
            // ← channel_target 是定时任务创建时保存的「逻辑 ID」（email/mobile/open_id）
            send_rendered_messages(&state_clone.facade, &receive_id, "open_id", ...).await;
            // ← 最终发出的 receive_id_type 永远是 "open_id"
        });
    }
}

async fn resolve_receive_id(facade: &FeishuApiClient, channel_target: &str) -> HoneResult<String> {
    let target = channel_target.trim();
    if target.contains('@') {
        return Ok(facade.resolve_email(target).await?.open_id);
        // ← 通过 email 反查 open_id
    }
    if looks_like_mobile(target) {
        return Ok(facade.resolve_mobile(target).await?.open_id);
        // ← 通过 mobile 反查 open_id
    }
    Ok(target.to_string())
    // ← 否则直接当 open_id 用
}
```

**真实 bug 路径**：

1. 用户 A 的定时任务创建时，用的 `channel_target` 是 **email**（如 `alice@corp.com`）
2. 任务触发时，`resolve_email("alice@corp.com")` 返回的 `open_id`
3. **如果 Feishu 的企业邮件目录发生变化**（员工离职、邮箱变更），`batch_get_id` API 可能返回另一个用户的 `open_id`
4. 定时任务回复被投递到了**错误的用户**

此外，**`resolve_email` 的实现存在 off-by-one 风险**：

```rust
// client.rs L346-358
if let Some(serde_json::Value::Array(mut list)) = user_list {
    if let Some(first) = list.pop() {  // ← pop() 取的是最后一个元素，而不是第一个
        if let Some(user_id) = first.get("user_id").and_then(|v| v.as_str()) {
            return Ok(FeishuResolvedUser { email: email.to_string(), open_id: user_id.to_string(), ... });
        }
    }
}
```

`list.pop()` 获取的是数组**最后一个元素**，而 `list.first()` 或显式索引 `list[0]` 才是第一个元素。如果 Feishu API 返回多个用户（批量匹配场景，或 API bug），当前代码**取的是最后一个**，可能不是预期的用户。同样的问题存在于 `resolve_mobile`（`client.rs` L403-412）。

---

### 问题 4：`FeishuApiClient` 无连接池，高并发下 token 刷新存在 TOCTOU

**位置**: `client.rs` 第 53–98 行（`get_token`）

```rust
async fn get_token(&self) -> Result<String, String> {
    {
        let cache = self.token_cache.read().await;   // ← 先 read
        if let Some((token, expires_at)) = &*cache {
            if Instant::now() < *expires_at { return Ok(token.clone()); }
        }
    }
    // ← 此处释放了 read lock，多个协程可能同时进入下面的刷新逻辑
    let resp = self.http.post(...).send().await...;  // ← 并发多次刷新
    let mut cache = self.token_cache.write().await;  
    *cache = Some((token.clone(), now + valid_duration)); // ← 最后一个写入的赢得竞争
    Ok(token)
}
```

这是一个经典的 TOCTOU（Time-of-check Time-of-use）问题。多个协程可能同时发现 token 过期并同时发起刷新请求。这不会直接导致跨用户错位，但会导致**不必要的 API 调用过多**，在极端情况下可能触发 Feishu 限流，进而导致 `get_user_by_open_id` 失败又回到问题 2。

---

## 四、复现条件

| 场景 | 复现概率 | 说明 |
|------|---------|------|
| 多用户同时发消息，且 `get_user_by_open_id` 响应延迟 | 低（p2p 不跨用户） | p2p 回复用 open_id，即使 API 失败也不会错位 |
| 定时任务 + 企业邮件目录变更 | 中 | channel_target 为 email 时，email→open_id 可能解析到错误用户 |
| `resolve_email`/`resolve_mobile` 返回多个用户 | 中 | `list.pop()` 取最后一个，可能不是目标用户 |
| 高并发 + token 过期 | 低 | 限流风险，间接加剧问题 2 |

---

## 五、修复方案

### Fix 1（关键）：`resolve_email` / `resolve_mobile` 改用 `first()` 而非 `pop()`

**文件**: `bins/hone-feishu/src/client.rs`

```rust
// 当前（错误）：
if let Some(first) = list.pop() { ... }

// 修复（正确）：
if let Some(first) = list.into_iter().next() { ... }
```

这是最简单且最高优先级的修复，避免多用户场景下取到错误用户。

### Fix 2（关键）：定时任务回复增加 open_id 二次验证

**文件**: `bins/hone-feishu/src/handler.rs`

在 `handle_scheduler_events` 中，对解析出的 `receive_id` 做 sanity check：若 `channel_target` 为 email/mobile，应对比定时任务创建时绑定的 `actor.user_id`（即创建时的 open_id），如果 `resolve_receive_id` 返回的 open_id 与 `actor.user_id` 不一致，应报错并拒绝发送，而不是静默发到错误用户。

```rust
// 伪代码：
let receive_id = resolve_receive_id(&facade, &event.channel_target).await?;
// 定时任务创建时 actor.user_id = open_id
if event.actor.user_id.starts_with("ou_") && receive_id != event.actor.user_id {
    error!("[Feishu/scheduler] receive_id mismatch! expected={} got={}", event.actor.user_id, receive_id);
    return; // 拒绝发送，不跨用户投递
}
```

### Fix 3（优化）：`get_token` 使用 double-checked locking

**文件**: `bins/hone-feishu/src/client.rs`

在释放 read lock 后、发起 HTTP 请求前，先尝试升级为 write lock，并在 write lock 内再次检查 token 是否已经被其他协程刷新：

```rust
async fn get_token(&self) -> Result<String, String> {
    {
        let cache = self.token_cache.read().await;
        if let Some((token, expires_at)) = &*cache {
            if Instant::now() < *expires_at { return Ok(token.clone()); }
        }
    }
    // 尝试获取写锁，先再次检查（double-checked）
    let mut cache = self.token_cache.write().await;
    if let Some((token, expires_at)) = &*cache {
        if Instant::now() < *expires_at { return Ok(token.clone()); }
    }
    // 此时确定需要刷新
    let token = fetch_new_token(&self.http, &self.app_id, &self.app_secret).await?;
    *cache = Some((token.clone(), Instant::now() + valid_duration));
    Ok(token)
}
```

### Fix 4（长期）：定时任务存储 open_id 而非 email/mobile 作为投递目标

目前定时任务在创建时存储的是 `channel_target`（可能是 email），在触发时再通过 `resolve_receive_id` 反查 open_id。

更安全的做法是：**在创建定时任务时，直接将 `actor.user_id`（即 open_id）作为 `channel_target`** 存入，消除中间的"email → open_id"反查步骤，彻底避免因目录变更导致的错位。

---

## 六、现有防护机制

以下机制**不能防御**此 bug：

| 机制 | 作用 | 为何不够 |
|------|------|----------|
| `MessageDeduplicator` | 防重复消息 | 只防同一 `message_id` 被处理两次 |
| `SessionLockRegistry` | 防同一 session 并发执行 | session 本身没错位，是投递目标错了 |
| `SESSION_RUN_LOCKS` | 防 AgentSession 并发 | 同上 |
| `outbound_receive_id` 在函数入口固定 | p2p 时用 open_id | 正确，只防 p2p。定时任务路径不走此逻辑 |

---

## 七、影响评估

- **p2p 实时消息**：正常情况下**不会**发生跨用户。`outbound_receive_id` 在 `process_incoming_message` 入口即固定，且在 p2p 下等于 `open_id`，整个处理链路不存在替换风险。
- **定时任务投递**：**存在**跨用户风险，尤其是 `channel_target` 为 email/mobile 时，且 `list.pop()` 的 bug 会在返回多个匹配用户时取到错误的 open_id。
- **日志可观测性**：`channel_target` 用于日志（`log_user`），当 `get_user_by_open_id` 失败时，同一用户在不同请求的日志中会出现两种身份标识，增加排查难度。

---

## 八、相关文件速查

| 文件 | 关键位置 | 说明 |
|------|---------|------|
| `bins/hone-feishu/src/handler.rs:162` | `process_incoming_message` | p2p 消息入口，outbound_receive_id 固定 |
| `bins/hone-feishu/src/handler.rs:596` | `handle_scheduler_events` | 定时任务投递，使用 resolve_receive_id |
| `bins/hone-feishu/src/handler.rs:1040` | `resolve_receive_id` | email/mobile → open_id 反查 |
| `bins/hone-feishu/src/client.rs:310` | `resolve_email` | `list.pop()` bug |
| `bins/hone-feishu/src/client.rs:364` | `resolve_mobile` | 同上 |
| `bins/hone-feishu/src/client.rs:53` | `get_token` | TOCTOU 问题 |
| `crates/hone-core/src/actor.rs:82` | `SessionIdentity::session_id` | session 文件名规则 |
