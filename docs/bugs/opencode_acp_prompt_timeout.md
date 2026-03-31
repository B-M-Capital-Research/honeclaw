# opencode ACP `session/prompt timeout (300s)` 问题分析

- **状态**: 2026-03-26 已确认存在实现层 bug；本次只留档，不修改运行时代码。

## 现象

- 用户偶发在消息回复阶段收到：
  - `opencode acp session/prompt timeout (300s)`
- 现象具有偶发性，但一旦触发，通常出现在较慢模型、较长推理链路、或带工具调用的回合里。
- 从用户视角看，像是“消息发出去了，系统工作了一阵子，最后却返回 timeout”。

## 结论

这是 Hone 自身 `opencode_acp` runner 的超时策略 bug，不只是上游模型偶尔变慢。

核心问题是：当前实现把整次 ACP `session/prompt` 包在一个固定 300 秒的**总墙钟超时**里，而不是“无进展才超时”的**空闲超时**。因此只要一次正常运行超过 300 秒，即使中间持续有 `agent_message_chunk`、`tool_call_update`、`usage_update` 等进度事件，Hone 仍会在 300 秒整主动中断并向用户返回错误。

## 代码证据

### 1. `opencode_acp` 对 `session/prompt` 使用固定总超时

`crates/hone-channels/src/runners/opencode_acp.rs`

```rust
let prompt_result = tokio::time::timeout(
    request_timeout,
    wait_for_response(
        "opencode",
        &mut reader,
        &mut stdin,
        next_id,
        Some(emitter.clone()),
        Some(&mut opencode_state),
        Some(stderr_buf.clone()),
    ),
)
```

同时默认配置把 `request_timeout_seconds` 设为 300：

`crates/hone-core/src/config/agent.rs`

```rust
fn default_opencode_request_timeout() -> u64 {
    300
}
```

本机当前运行时配置也确实如此：

`data/runtime/config_runtime.yaml`

```yaml
agent:
  runner: "opencode_acp"
  opencode:
    request_timeout_seconds: 300
```

### 2. `wait_for_response` 会消费并处理流式事件，但不会刷新超时

`crates/hone-channels/src/runners/acp_common.rs` 中，`wait_for_response` 会持续读取 ACP stdout，并处理：

- `session/update`
- `session/request_permission`

其中 `session/update` 又会继续处理：

- `agent_message_chunk`
- `agent_thought_chunk`
- `tool_call`
- `tool_call_update`
- `usage_update`

这些事件说明 ACP 会话在“持续推进”。但由于 `tokio::time::timeout(...)` 包在 `wait_for_response(...)` 的最外层，任何中间进展都不会刷新 300 秒计时器。

换句话说，当前逻辑是：

1. 发出 `session/prompt`
2. 开始一个固定 300 秒倒计时
3. 即使 299 秒内一直有流式增量输出，倒计时也不会延长
4. 只要第 300 秒时最终 JSON-RPC `result` 还没回来，就直接报：
   - `opencode acp session/prompt timeout (300s)`

## 为什么它会表现为“偶发”

因为并非每次请求都超过 300 秒。

只有以下情况更容易触发：

- 模型本身推理慢
- 工具调用链较长
- 上游 provider 抖动，但仍在持续返回事件
- 用户问题复杂，最终 `stopReason=end_turn` 晚于 300 秒才抵达

因此它看起来像“偶发”，但触发机制其实是稳定且确定的。

## 为什么这属于产品级 bug

当前错误把两类完全不同的情况混在了一起：

- 真正卡死：ACP 长时间无任何输出，也无最终响应
- 正常慢请求：ACP 仍在稳定推进，只是总耗时超过 300 秒

现实现状会把第二类也当成失败。这会带来几个问题：

- 用户收到误报，误以为系统不可用
- 渠道层可能已经给用户展示了部分流式内容，最终却被一条 timeout 错误收尾
- 如果上层以后引入重试，可能放大重复回复或重复工具调用风险

## 与当前实现的关系

这个问题与此前 `opencode` 的“每轮强制 `session/new`、不复用 `session/load`”是独立问题。

- `session/new` 主要是在规避历史 `agent_message_chunk` 回放污染当前流式输出
- 本次 timeout 问题发生在**新 session 已经创建成功之后**
- 即使保持现有 `session/new` 策略不变，只要 `session/prompt` 仍是固定总超时，就仍会继续触发

## 建议修复方向

### 方案 A：改为“空闲超时”而不是“总超时”

更合理的语义是：

- 只要在最近 N 秒内持续收到 `session/update` 或其它有效事件，就继续等待
- 只有在“连续无输出”超过阈值时，才判定 timeout

这能保留对真正卡死的保护，同时避免误杀长任务。

### 方案 B：双超时

同时保留两层：

- 较短的空闲超时，例如 60-90 秒
- 较长的总超时，例如 900-1800 秒

语义上更稳妥：

- 没有任何进展太久，就快速失败
- 但允许长任务在持续有进展时跑完

### 方案 C：错误文案区分

在真正改逻辑前，至少应区分：

- `idle timeout`
- `overall timeout`

否则当前 `session/prompt timeout (300s)` 很容易让排障者误判为上游完全无响应。

## 本次结论边界

- 已确认存在实现层 bug
- 证据来自本地代码与当前运行时配置
- 本次没有改代码，也没有调整线上配置
- 因此该问题在当前 `main` 上仍可能继续出现

## 相关文件

- `crates/hone-channels/src/runners/opencode_acp.rs`
- `crates/hone-channels/src/runners/acp_common.rs`
- `crates/hone-core/src/config/agent.rs`
- `data/runtime/config_runtime.yaml`
