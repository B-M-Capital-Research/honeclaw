use hone_core::LlmAuditRecord;
use hone_memory::{
    build_compact_boundary_metadata, build_compact_skill_snapshot_metadata,
    build_compact_summary_metadata, invoked_skills_from_metadata,
    select_messages_after_compact_boundary, session::SessionSummary, session_message_from_text,
    session_message_text,
};

use crate::HoneBotCore;
use crate::core::CompactSessionOutcome;
use crate::runtime::sanitize_user_visible_output;

const POST_COMPACT_MAX_SKILL_SNAPSHOT_CHARS: usize = 12_000;
const POST_COMPACT_MAX_SKILL_SNAPSHOTS: usize = 4;

pub struct SessionCompactor<'a> {
    core: &'a HoneBotCore,
}

impl<'a> SessionCompactor<'a> {
    pub fn new(core: &'a HoneBotCore) -> Self {
        Self { core }
    }

    pub async fn compact_session(
        &self,
        session_id: &str,
        trigger: &str,
        force: bool,
        user_instructions: Option<&str>,
    ) -> hone_core::HoneResult<CompactSessionOutcome> {
        let Some(session) = self.core.session_storage.load_session(session_id)? else {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        };

        let active_messages = select_messages_after_compact_boundary(&session.messages, None);
        if active_messages.is_empty() {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        }
        let is_group_session = session
            .session_identity
            .as_ref()
            .map(|identity| identity.is_group())
            .unwrap_or(false);

        let compress_threshold = if is_group_session {
            self.core
                .config
                .group_context
                .compress_threshold_messages
                .max(1)
        } else {
            20
        };
        let compress_byte_threshold = if is_group_session {
            self.core
                .config
                .group_context
                .compress_threshold_bytes
                .max(1024)
        } else {
            80_000
        };
        let retain_recent = if is_group_session {
            self.core
                .config
                .group_context
                .retain_recent_after_compress
                .max(1)
        } else {
            6
        };

        let total_content_bytes: usize = active_messages
            .iter()
            .map(|message| session_message_text(message).len())
            .sum();
        let compact_window_size = active_messages.len().saturating_sub(retain_recent);
        let messages_to_summarize: Vec<_> = if compact_window_size > 0 {
            active_messages.iter().take(compact_window_size).collect()
        } else if force {
            let forced_window = active_messages
                .len()
                .saturating_sub(1)
                .max(1)
                .min(active_messages.len());
            active_messages.iter().take(forced_window).collect()
        } else {
            Vec::new()
        };
        let should_compress = force
            || active_messages.len() > compress_threshold
            || total_content_bytes > compress_byte_threshold;

        if !should_compress || messages_to_summarize.is_empty() {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        }

        tracing::info!(
            "[HoneBotCore] Compressing session {} with {} messages (~{} bytes)...",
            session_id,
            active_messages.len(),
            total_content_bytes,
        );

        let llm = match &self.core.auxiliary_llm {
            Some(provider) => provider.as_ref(),
            None => {
                tracing::warn!(
                    "[HoneBotCore] No LLM provider available for compression. Please configure llm provider in config.yaml. Skipping compression."
                );
                return Ok(CompactSessionOutcome {
                    compacted: false,
                    summary: None,
                });
            }
        };

        let mut history_text = String::new();
        for message in &messages_to_summarize {
            let content = match message.role.as_str() {
                "assistant" | "user" => {
                    sanitize_user_visible_output(&session_message_text(message)).content
                }
                "tool" => String::new(),
                _ => session_message_text(message),
            };
            if content.trim().is_empty() {
                continue;
            }
            history_text.push_str(&format!("{}: {}\n\n", message.role, content));
        }

        let prompt = if is_group_session {
            format!(
                "你是一个群聊上下文整理员。由于群会话历史过长，需要把更早内容压缩成稳定、简洁、适合后续继续讨论的群摘要。\n\
                \n\
                只输出纯 Markdown，并严格使用以下四段标题：\n\
                \n\
                ## 进行中议题\n\
                - 列出群里当前仍在讨论的问题或主题\n\
                \n\
                ## 已形成结论\n\
                - 只记录群内已经达成的结论或明确共识\n\
                \n\
                ## 未决问题\n\
                - 列出仍待回答、待确认、待补充的信息\n\
                \n\
                ## 群约定 / 待办\n\
                - 记录群里明确提到的后续动作、分工、约定\n\
                \n\
                额外约束：\n\
                - 不要写成员画像、长期个人偏好或性格判断\n\
                - 不要固化个人金融隐私，如持仓、成本、成交价、交易单等\n\
                - 只保留对后续群讨论真正有帮助的信息\n\
                - 不要寒暄，不要输出其它标题\n\
                {}\n\
                \n\
                以下是待压缩的群历史：\n\
                {}",
                user_instructions
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| format!(" - 额外要求：{value}\n"))
                    .unwrap_or_default(),
                history_text
            )
        } else {
            format!(
                "你是一个金融分析助手的记忆整理员。由于用户的对话历史过长，我需要你将他们进行压缩和总结。\n\
                \n\
                请按照以下格式输出：\n\
                \n\
                1. **股票关注表**（只提取对话中提到的具体股票/公司）包含以下五列：\n\
                | 股票代码 | 公司名 | 公司一句话简介 | 助手的观点 | 用户的观点 |\n\
                | --- | --- | --- | --- | --- |\n\
                （如果没有提取到股票，输出一个空表即可，但必须包含表头）\n\
                \n\
                2. **【历史对话总结】**\n\
                在表下面，用1-2段话总结上面发生的核心交互和用户的偏好习惯信息。\n\
                \n\
                额外约束：\n\
                - 你只是在总结已经发生的历史，不是在回答任何尚未解决的问题\n\
                - 不要生成新的投研结论、价格目标、持仓明细、时间线或事实数字，除非这些内容在历史里已经明确出现\n\
                - 如果历史末尾包含尚未回答的问题，只能把它记为“用户最近关心/待回答的问题”，不要替助手继续作答\n\
                - 不要把摘要写成报告、正式结论或投资建议正文\n\
                \n\
                {}\n\
                请保持纯净的 Markdown 输出，不要有多余的寒喧。\n\
                \n\
                以下是对话历史：\n\
                {}",
                user_instructions
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| format!("额外要求：{value}\n"))
                    .unwrap_or_default(),
                history_text
            )
        };

        let messages = vec![hone_llm::Message {
            role: "user".to_string(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];

        let started = std::time::Instant::now();
        let request_payload = serde_json::json!({ "messages": messages.clone() });
        let auxiliary_model = self.core.auxiliary_model_name();
        let (auxiliary_provider, _) = self.core.auxiliary_provider_hint();
        tracing::info!(
            "[SessionCompress] session={} provider={} model={} active_messages={} retained_recent={} force={} trigger={}",
            session_id,
            auxiliary_provider,
            auxiliary_model,
            active_messages.len(),
            retain_recent,
            force,
            trigger,
        );
        let (new_summary_content, usage) = match llm.chat(&messages, Some(&auxiliary_model)).await {
            Ok(result) => (result.content, result.usage),
            Err(err) => {
                self.record_llm_audit(LlmAuditRecord {
                    success: false,
                    latency_ms: Some(started.elapsed().as_millis()),
                    error: Some(err.to_string()),
                    metadata: serde_json::json!({
                        "kind": "session_compression",
                        "active_messages": active_messages.len(),
                        "summarized_messages": messages_to_summarize.len(),
                        "is_group_session": is_group_session
                    }),
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                    ..LlmAuditRecord::new(
                        session_id.to_string(),
                        session.actor.clone(),
                        "core.session_compression",
                        "chat",
                        auxiliary_provider.clone(),
                        Some(auxiliary_model.clone()),
                        request_payload.clone(),
                    )
                });
                tracing::error!("[HoneBotCore] LLM summarization failed: {}", err);
                return Ok(CompactSessionOutcome {
                    compacted: false,
                    summary: None,
                });
            }
        };

        self.record_llm_audit(LlmAuditRecord {
            success: true,
            latency_ms: Some(started.elapsed().as_millis()),
            response: Some(serde_json::json!({ "content": new_summary_content.clone() })),
            metadata: serde_json::json!({
                "kind": "session_compression",
                "active_messages": active_messages.len(),
                "summarized_messages": messages_to_summarize.len(),
                "retained_recent": retain_recent,
                "is_group_session": is_group_session,
                "trigger": trigger,
                "forced": force,
                "custom_instructions": user_instructions
            }),
            prompt_tokens: usage.as_ref().and_then(|value| value.prompt_tokens),
            completion_tokens: usage.as_ref().and_then(|value| value.completion_tokens),
            total_tokens: usage.as_ref().and_then(|value| value.total_tokens),
            ..LlmAuditRecord::new(
                session_id.to_string(),
                session.actor.clone(),
                "core.session_compression",
                "chat",
                auxiliary_provider.clone(),
                Some(auxiliary_model),
                request_payload,
            )
        });
        tracing::info!(
            "[SessionCompress] session={} provider={} model={} elapsed_ms={} summary_chars={}",
            session_id,
            auxiliary_provider,
            self.core.auxiliary_model_name(),
            started.elapsed().as_millis(),
            new_summary_content.chars().count(),
        );

        let sanitized_summary = sanitize_user_visible_output(&new_summary_content).content;
        let summary_to_store = sanitized_summary.trim();
        if summary_to_store.is_empty() {
            tracing::warn!(
                "[HoneBotCore] Session {} compression produced no user-visible summary after sanitization.",
                session_id
            );
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        }

        let mut new_messages = Vec::new();
        new_messages.push(session_message_from_text(
            "system",
            "Conversation compacted",
            hone_core::beijing_now_rfc3339(),
            Some(build_compact_boundary_metadata(
                trigger,
                active_messages.len().saturating_sub(retain_recent),
                active_messages.len(),
            )),
        ));
        new_messages.push(session_message_from_text(
            "user",
            &format!("【Compact Summary】\n{summary_to_store}"),
            hone_core::beijing_now_rfc3339(),
            Some(build_compact_summary_metadata(trigger)),
        ));
        for skill in invoked_skills_from_metadata(&session.metadata)
            .into_iter()
            .take(POST_COMPACT_MAX_SKILL_SNAPSHOTS)
        {
            let snapshot = truncate_chars(&skill.prompt, POST_COMPACT_MAX_SKILL_SNAPSHOT_CHARS);
            if snapshot.trim().is_empty() {
                continue;
            }
            new_messages.push(session_message_from_text(
                "user",
                &snapshot,
                hone_core::beijing_now_rfc3339(),
                Some(build_compact_skill_snapshot_metadata(&skill.skill_name)),
            ));
        }
        let retained: Vec<_> = active_messages
            .into_iter()
            .rev()
            .take(retain_recent)
            .collect();
        for message in retained.into_iter().rev() {
            new_messages.push(message.clone());
        }

        self.core.session_storage.replace_messages_with_summary(
            session_id,
            new_messages,
            Some(SessionSummary::new(summary_to_store)),
        )?;
        tracing::info!(
            "[HoneBotCore] Session {} compacted to boundary + summary + {} retained items.",
            session_id,
            retain_recent,
        );

        Ok(CompactSessionOutcome {
            compacted: true,
            summary: Some(summary_to_store.to_string()),
        })
    }

    fn record_llm_audit(&self, record: LlmAuditRecord) {
        if let Some(sink) = &self.core.llm_audit {
            if let Err(err) = sink.record(record) {
                tracing::warn!("[LlmAudit] failed to persist record: {}", err);
            }
        }
    }
}

fn truncate_chars(content: &str, max_chars: usize) -> String {
    if max_chars == 0 || content.chars().count() <= max_chars {
        return content.to_string();
    }
    content.chars().take(max_chars).collect::<String>()
}
