//! 消息流审计日志 trait。
//!
//! 对应 `HoneBotCore` 的 4 个 `log_message_*` 方法：`received`（入站）、
//! `step`（阶段）、`finished`（成功结束）、`failed`（失败结束）。
//! 这些方法只做 tracing log 输出,不触碰 LLM 或 storage,因此是最早被抽
//! 成 trait 的一批。
//!
//! 用法示例：
//! ```ignore
//! fn do_work(audit: &dyn AuditRecorder, channel: &str, user: &str, session: &str) {
//!     audit.log_message_step(channel, user, session, "phase.x", "done", None, None);
//! }
//! ```

use hone_core::agent::AgentResponse;

use crate::core::HoneBotCore;

/// 记录 AgentSession 生命周期中的消息流事件。
///
/// 所有方法都是 `&self`（只读）+ 同步:实现只需要写 tracing log,不能阻塞
/// 也不能持有 async 状态。
pub trait AuditRecorder: Send + Sync {
    /// 入站：渠道接收到一条用户消息。
    #[allow(clippy::too_many_arguments)]
    fn log_message_received(
        &self,
        channel: &str,
        user_id: &str,
        channel_target: &str,
        session_id: &str,
        input: &str,
        extra: Option<&str>,
        message_id: Option<&str>,
    );

    /// 中间阶段：例如 prompt 构建完成、runner 启动、compact 触发等。
    fn log_message_step(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        step: &str,
        detail: &str,
        message_id: Option<&str>,
        state_override: Option<&str>,
    );

    /// 成功结束：带 response 统计（迭代次数、工具调用、内容长度）。
    fn log_message_finished(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        response: &AgentResponse,
        elapsed_ms: u128,
        message_id: Option<&str>,
    );

    /// 失败结束：带错误描述（会被 truncate 到 280 字符避免日志爆炸）。
    fn log_message_failed(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        error: &str,
        elapsed_ms: u128,
        message_id: Option<&str>,
    );
}

/// HoneBotCore 的实现全部转发到已有 inherent method,行为与调用方完全一致。
impl AuditRecorder for HoneBotCore {
    fn log_message_received(
        &self,
        channel: &str,
        user_id: &str,
        channel_target: &str,
        session_id: &str,
        input: &str,
        extra: Option<&str>,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_received(
            self,
            channel,
            user_id,
            channel_target,
            session_id,
            input,
            extra,
            message_id,
        )
    }

    fn log_message_step(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        step: &str,
        detail: &str,
        message_id: Option<&str>,
        state_override: Option<&str>,
    ) {
        HoneBotCore::log_message_step(
            self,
            channel,
            user_id,
            session_id,
            step,
            detail,
            message_id,
            state_override,
        )
    }

    fn log_message_finished(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        response: &AgentResponse,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_finished(
            self, channel, user_id, session_id, response, elapsed_ms, message_id,
        )
    }

    fn log_message_failed(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        error: &str,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_failed(
            self, channel, user_id, session_id, error, elapsed_ms, message_id,
        )
    }
}
