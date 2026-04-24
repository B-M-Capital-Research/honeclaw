//! ACP 协议栈共享的**数据类型 + 常量 + regex**。没有行为,只有形。
//!
//! 其他 sibling module(`ingest` / `tool_state` / `protocol` / `log`)都
//! 依赖本文件里定义的 `AcpPromptState` 等结构,所以保持这个文件零 I/O、零
//! 异步,任何时候单独 read 都能一眼看懂全貌。

use hone_core::agent::{AgentMessage, ToolCallMade};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Duration;

/// session_metadata 上记录的"上一轮 prompt 完成时 ACP runner 报告的 usage.used 峰值"。
/// 用作下一轮 compact 检测的基线（opencode 不推 compact 字面量，只能靠 used 骤降识别）。
pub(crate) const ACP_PREV_PROMPT_PEAK_KEY: &str = "acp_prev_prompt_peak_used";

/// session_metadata 上记录的"下一轮需要重新塞 system_prompt"标志。
/// 写入条件：本轮 ACP runner 报告了 compact 事件（codex 字面量 / opencode used drop）。
/// 消费方：prompt 构建层下一轮检查到 true 时，把完整 system_prompt 重新拼入 user message。
pub(crate) const ACP_NEEDS_SP_RESEED_KEY: &str = "acp_needs_sp_reseed";

/// codex-acp 在内置 compact 触发后推回的字面量 chunk（实测：
/// `agent_message_chunk text="Context compacted\n"`，单独一条）。
/// 同时也匹配 honeclaw 老 SessionCompactor 历史写入的 `Conversation compacted` 字符串。
pub(super) static RE_ACP_COMPACT_STATUS_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\s*(context|conversation)\s+compacted\.?\s*$").expect("valid regex")
});

/// opencode 在内置 compact 触发后会把"重启会话"的 markdown summary 拼到本轮 reply 后面，
/// 形如 `OK\n---\n## Goal\n...\n## Relevant Files\n- (none)\n---\nI don't have...`。
/// 我们用 `\n---\n## ` / `^---\n## ` 作为 compact 已发生的补充检测信号。
/// 注意：opencode 实测会把这段边界拆到多条 `agent_message_chunk` 里（如 `---\n` /
/// `## ` / ` Goal`），单条 chunk 上 regex 必然漏；因此 ingest 时必须在 **累积 buffer**
/// 上扫描。
pub(super) static RE_OPENCODE_SUMMARY_BOUNDARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(^|\n)---\s*\n##\s+\w").expect("valid regex"));

/// 跨 chunk 扫 opencode summary boundary 时，从 buffer 尾部回看的窗口字节数。
/// 取 64 足以覆盖 `\n---\n## <heading>` 的最长合法变体（含 trailing 空白）。
pub(super) const ACP_BOUNDARY_SCAN_TAIL_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpToolRenderPhase {
    Start,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpPermissionDecision {
    RejectOnce,
    ApproveForSession,
}

impl AcpPermissionDecision {
    pub(super) fn preferred_kind(self) -> &'static str {
        match self {
            Self::RejectOnce => "reject_once",
            Self::ApproveForSession => "allow_always",
        }
    }

    pub(super) fn fallback_kind(self) -> &'static str {
        match self {
            Self::RejectOnce => "reject_once",
            Self::ApproveForSession => "allow_once",
        }
    }

    pub(super) fn progress_label(self) -> &'static str {
        match self {
            Self::RejectOnce => "rejected",
            Self::ApproveForSession => "approved-for-session",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AcpRenderedToolStatus {
    pub(crate) tool: String,
    pub(crate) message: Option<String>,
    pub(crate) reasoning: Option<String>,
}

pub(crate) type AcpToolStatusRenderer = fn(
    update: &Value,
    phase: AcpToolRenderPhase,
    default_tool: &str,
    default_message: Option<String>,
    default_reasoning: Option<String>,
) -> AcpRenderedToolStatus;

pub(crate) type AcpSessionUpdateTransformer = fn(&Value) -> Option<Value>;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AcpResponseTimeouts {
    pub(crate) idle: Duration,
    pub(crate) overall: Duration,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpToolCallRecord {
    pub(crate) name: String,
    pub(crate) arguments: Value,
}

#[derive(Default)]
pub(crate) struct AcpPromptState {
    pub(crate) full_reply: String,
    pub(crate) pending_tool_calls: HashMap<String, AcpToolCallRecord>,
    pub(crate) finished_tool_calls: Vec<ToolCallMade>,
    pub(crate) completed_tool_call_ids: HashSet<String>,
    pub(crate) context_messages: Vec<AgentMessage>,
    pub(crate) pending_assistant_content: String,
    pub(crate) pending_assistant_tool_calls: Vec<Value>,
    /// 入口由 runner 在 spawn 时从 session_metadata 读取
    /// `ACP_PREV_PROMPT_PEAK_KEY`，作为本轮 usage_update.used 骤降判定的基线。
    /// `None` 表示本 session 是第一次有 ACP 流，不做骤降判定。
    pub(crate) prev_prompt_peak_used: Option<u64>,
    /// 本轮 prompt 流中观测到的 usage.used 峰值，结束后 runner 写回 metadata。
    pub(crate) current_prompt_peak_used: u64,
    /// 本轮 prompt 流中是否检测到 ACP runner 触发了内置 compact。
    /// 触发源：codex 推 `Context compacted` 字面量 / opencode used 骤降 (>50%)。
    /// 检测后：runner 应在 metadata 写 `ACP_NEEDS_SP_RESEED_KEY=true`，下一轮重塞 SP。
    pub(crate) compact_detected: bool,
    /// 流中是否已经收到第一条 usage_update（用于"首次观测时与 prev_peak 比较"）。
    pub(crate) usage_update_seen: bool,
}

/// 沿着 `text` 从 `pos` 向左退到最近的 UTF-8 字符边界,不会跨出字符中段切片。
/// 专门给「从 buffer 尾部回看窗口」扫 boundary regex 的场景用,
/// 中文 / emoji 等多字节字符下防 panic。
pub(super) fn floor_char_boundary(text: &str, pos: usize) -> usize {
    let mut boundary = pos.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}
