//! `restore_context` —— 把持久化的 session 历史重建成 runner 可消费的
//! [`AgentContext`]。
//!
//! 这是 **所有 runner 共享** 的历史还原函数,不应该被某个 runner 改写或
//! 替换成「自己版本的 restore」。设计不变量：
//!
//! - `SessionStorage`/`SessionMessage` 的具体 schema 只在这里被感知,
//!   runner 层永远只消费 `AgentContext.messages`（或
//!   `AgentContext.normalized_history_json()` 的序列化形式）
//! - 新增 runner 时,若发现这里的重建结果不适用,应当**扩展**这个函数的
//!   输入/输出契约（例如加个参数决定是否展开 tool_result），而不是在
//!   runner 内部重新读一遍 session 文件
//! - 具体 runner 只负责把 `AgentContext` 喂进自家的 prompt/API
//!   （例：`codex_acp::build_codex_acp_prompt_text` 把它序列化成 JSON）
//!
//! 之所以写得这么啰嗦：仓库早期有过 runner 各自读 session JSON 的版本,
//! 每次存储 schema 升级都漏改一两个 runner,这里用注释锁死不变量。

use hone_core::agent::{AgentContext, AgentMessage};
use hone_memory::{
    SessionStorage, assistant_tool_calls_from_metadata, has_compact_skill_snapshot,
    invoked_skills_from_metadata, message_is_compact_boundary, message_is_compact_summary,
    message_is_slash_skill, restore_tool_message, select_messages_after_compact_boundary,
    session_message_text, session_message_to_agent_messages,
};

use super::helpers::sanitize_assistant_context_content;

pub fn restore_context(
    storage: &SessionStorage,
    session_id: &str,
    max_messages: Option<usize>,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) -> AgentContext {
    let mut ctx = AgentContext::new(session_id.to_string());

    let Ok(Some(session)) = storage.load_session(session_id) else {
        return ctx;
    };

    if let Some(actor) = &session.actor {
        ctx.set_actor_identity(actor);
    }

    let messages = select_messages_after_compact_boundary(&session.messages, max_messages);
    let has_skill_snapshots = has_compact_skill_snapshot(&messages);
    if !has_skill_snapshots {
        for skill in invoked_skills_from_metadata(&session.metadata)
            .into_iter()
            .filter(|skill| !skill.prompt.trim().is_empty())
            .filter(|skill| {
                skill_runtime
                    .map(|runtime| {
                        runtime
                            .load_registered_skill(&skill.skill_name)
                            .map(|definition| definition.enabled)
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
            })
        {
            ctx.add_user_message(&skill.prompt);
        }
    }

    for message in messages {
        match message.role.as_str() {
            "user" => {
                if !message_is_slash_skill(message.metadata.as_ref())
                    && !message_is_compact_summary(message.metadata.as_ref())
                {
                    let content = session_message_text(message);
                    if !content.trim().is_empty() {
                        ctx.messages.push(AgentMessage {
                            role: "user".to_string(),
                            content: Some(content),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                            metadata: message.metadata.clone(),
                        });
                    }
                }
            }
            "assistant" | "tool" => {
                for mut restored in session_message_to_agent_messages(message) {
                    if restored.role == "assistant" {
                        let sanitized_content = sanitize_assistant_context_content(
                            restored.content.as_deref().unwrap_or_default(),
                        );
                        let tool_calls = restored.tool_calls.clone().or_else(|| {
                            assistant_tool_calls_from_metadata(message.metadata.as_ref())
                        });
                        if sanitized_content.trim().is_empty()
                            && tool_calls.as_ref().is_none_or(|items| items.is_empty())
                        {
                            continue;
                        }
                        restored.content = Some(sanitized_content);
                        restored.tool_calls = tool_calls;
                    }
                    if restored.role == "tool"
                        && restore_tool_message(message).is_none()
                        && restored
                            .content
                            .as_deref()
                            .unwrap_or_default()
                            .trim()
                            .is_empty()
                    {
                        continue;
                    }
                    ctx.messages.push(restored);
                }
            }
            "system" => {
                if message_is_compact_boundary(message.metadata.as_ref())
                    || message_is_compact_summary(message.metadata.as_ref())
                {
                    continue;
                }
            }
            _ => {}
        }
    }

    ctx
}
