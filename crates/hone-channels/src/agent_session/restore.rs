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

use hone_core::agent::{AgentContext, AgentMessage, RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY};
use hone_memory::session::{Session, SessionMessage};
use hone_memory::{
    SessionStorage, assistant_tool_calls_from_metadata, has_compact_skill_snapshot,
    invoked_skills_from_metadata, message_is_compact_boundary, message_is_compact_summary,
    message_is_slash_skill, restore_tool_message, select_messages_after_compact_boundary,
    session_message_text, session_message_to_agent_messages,
};

use super::helpers::{
    history_message_is_automation, history_message_is_failed_terminal,
    sanitize_assistant_context_content,
};

const RECENT_INTERACTIVE_USER_REFERENCE_LIMIT: usize = 4;

fn restore_invoked_skill_prompts(
    context: &mut AgentContext,
    session: &Session,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) {
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
        context.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some(skill.prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: Some(std::collections::HashMap::from([(
                RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY.to_string(),
                serde_json::Value::Bool(true),
            )])),
        });
    }
}

pub fn restore_context(
    storage: &SessionStorage,
    session_id: &str,
    max_messages: Option<usize>,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) -> AgentContext {
    let Ok(Some(session)) = storage.load_session(session_id) else {
        return AgentContext::new(session_id.to_string());
    };

    restore_context_from_snapshot(&session, max_messages, skill_runtime)
}

pub(super) fn restore_context_from_snapshot(
    session: &Session,
    max_messages: Option<usize>,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) -> AgentContext {
    let mut restored_context = AgentContext::new(session.id.clone());

    if let Some(actor) = &session.actor {
        restored_context.set_actor_identity(actor);
    }

    let messages = select_messages_after_compact_boundary(&session.messages, max_messages);
    let has_skill_snapshots = has_compact_skill_snapshot(&messages);
    if !has_skill_snapshots {
        restore_invoked_skill_prompts(&mut restored_context, session, skill_runtime);
    }

    for message in messages {
        match message.role.as_str() {
            "user" => {
                if !message_is_slash_skill(message.metadata.as_ref())
                    && !message_is_compact_summary(message.metadata.as_ref())
                {
                    let content = session_message_text(message);
                    if !content.trim().is_empty() {
                        restored_context.messages.push(AgentMessage {
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
                    restored_context.messages.push(restored);
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

    restored_context
}

fn group_is_operational_or_failed(group: &[SessionMessage]) -> bool {
    group.iter().any(|message| {
        let user_content = (message.role == "user")
            .then(|| session_message_first_text(message))
            .flatten();
        history_message_is_automation(&message.role, user_content, message.metadata.as_ref())
            || history_message_is_failed_terminal(message.metadata.as_ref())
    })
}

fn session_message_first_text(message: &SessionMessage) -> Option<&str> {
    message
        .content
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .find(|text| !text.is_empty())
}

fn session_message_text_matches(message: &SessionMessage, expected: &str) -> bool {
    let mut offset = 0usize;
    let mut found = false;
    for text in message
        .content
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        if found {
            if expected.as_bytes().get(offset) != Some(&b'\n') {
                return false;
            }
            offset += 1;
        }
        if !expected
            .get(offset..)
            .is_some_and(|remaining| remaining.starts_with(text))
        {
            return false;
        }
        offset += text.len();
        found = true;
    }
    found && offset == expected.len()
}

fn reference_user_is_eligible(message: &SessionMessage) -> bool {
    if message.role != "user"
        || message_is_slash_skill(message.metadata.as_ref())
        || message_is_compact_summary(message.metadata.as_ref())
        || message_is_compact_boundary(message.metadata.as_ref())
    {
        return false;
    }
    session_message_first_text(message).is_some_and(|text| !text.trim_start().starts_with('/'))
}

/// Restore only the durable user wording needed to resolve follow-up
/// references for an initial strict Interactive research turn. The current
/// turn is identified by the final durable user-row position and excluded by
/// index, so an older row with identical bytes cannot be mistaken for it.
/// Compact boundaries do not truncate this bounded user-only view.
pub(super) fn restore_recent_interactive_user_references(
    session: &Session,
    current_user_input: &str,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) -> AgentContext {
    let mut context = AgentContext::new(session.id.clone());
    if let Some(actor) = &session.actor {
        context.set_actor_identity(actor);
    }
    // The fast path intentionally omits compact snapshots, summaries, and
    // assistant/tool history. Re-inject the durable invoked-skill prompts from
    // the same Session snapshot so follow-ups retain explicitly selected skill
    // semantics while still respecting the current activation registry.
    restore_invoked_skill_prompts(&mut context, session, skill_runtime);

    let Some(current_user_index) = session
        .messages
        .iter()
        .rposition(|message| message.role == "user")
    else {
        return context;
    };
    if !session_message_text_matches(&session.messages[current_user_index], current_user_input) {
        return context;
    }

    let history = &session.messages[..current_user_index];
    let mut eligible_user_indices = Vec::new();
    let mut group_start = None;
    for (index, message) in history.iter().enumerate() {
        if message.role != "user" {
            continue;
        }
        if let Some(start) = group_start {
            let group = &history[start..index];
            if !group_is_operational_or_failed(group) && reference_user_is_eligible(&history[start])
            {
                eligible_user_indices.push(start);
            }
        }
        group_start = Some(index);
    }
    if let Some(start) = group_start {
        let group = &history[start..];
        if !group_is_operational_or_failed(group) && reference_user_is_eligible(&history[start]) {
            eligible_user_indices.push(start);
        }
    }

    let selected_start = eligible_user_indices
        .len()
        .saturating_sub(RECENT_INTERACTIVE_USER_REFERENCE_LIMIT);
    for index in eligible_user_indices.into_iter().skip(selected_start) {
        let message = &history[index];
        context.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some(session_message_text(message)),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: message.metadata.clone(),
        });
    }

    context
}
