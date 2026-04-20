use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use feishu_sdk::core::{Config as FeishuConfig, LogLevel as FeishuLogLevel, new_logger};
use feishu_sdk::event::{Event, EventDispatcher, EventDispatcherConfig, EventHandler, EventResp};
use feishu_sdk::ws::StreamClient;
use hone_channels::ChatMode;
use hone_channels::agent_session::{AgentRunOptions, AgentSession, MessageMetadata};
use hone_channels::attachments::{
    AttachmentIngestRequest, AttachmentPersistRequest, RawAttachment, build_attachment_ack_message,
    build_user_input, ingest_raw_attachments, spawn_attachment_persist_pipeline,
};
use hone_channels::ingress::{
    ActiveSessionInfo, ActorScopeResolver, BufferedGroupMessage, GroupTrigger, IncomingEnvelope,
    MessageDeduplicator, SessionLockRegistry, persist_buffered_group_messages,
};
use hone_channels::outbound::{ReasoningVisibility, attach_stream_activity_probe};
use hone_channels::prompt::PromptOptions;
use hone_channels::runtime::user_visible_error_message;
use hone_channels::think::{ThinkRenderStyle, ThinkStreamFormatter, render_think_blocks};
use hone_core::{ActorIdentity, SessionIdentity};
use serde_json::{Value, json};
use tracing::{error, info, warn};

use super::card::CardKitSession;
use super::client::FeishuApiClient;
use super::listener::FeishuStreamListener;
use super::markdown::preprocess_markdown_for_feishu;
use super::outbound::{
    feishu_user_mention, prepend_reply_prefix, send_placeholder_message, send_plain_text,
    send_rendered_messages, update_or_send_plain_text,
};
use super::scheduler::handle_scheduler_events;
use super::types::{AppState, FeishuEventHandler, FeishuIncomingAttachment, FeishuIncomingMessage};

const THINKING_PLACEHOLDER_TEXT: &str = "正在思考中...";
const FEISHU_GROUP_PRIVACY_GUARD: &str = "【群聊隐私约束】\n1. 禁止在群聊索取或引导补全持仓明细（股数、成本、成交价、交易单等）。\n2. 禁止在群聊查询或确认用户个人持仓；用户问“我现在持有哪些”时，直接提示转私聊处理。\n3. 只提供通用信息与私聊引导，不给出任何个人资产判断或推断。";

fn feishu_speaker_label(open_id: &str, email: Option<&str>, mobile: Option<&str>) -> String {
    email
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            mobile
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| open_id.to_string())
}

fn build_group_user_input_with_speaker(label: &str, text: &str) -> String {
    format!("[{label}] {}", text.trim())
}

fn build_group_busy_text(speaker_label: &str) -> String {
    format!("正在处理 {speaker_label} 的消息，请等上一条完成后再 @ 我。")
}

fn build_direct_busy_text() -> &'static str {
    "上一条消息还在处理中，请等当前回复完成后再发送新消息。"
}

fn build_unparsed_message_text() -> &'static str {
    "抱歉，这条消息没有解析到可处理内容。请直接发送文本，或重新发送图片/文件。"
}

fn has_actionable_user_input(text: &str, attachment_count: usize, buffered_count: usize) -> bool {
    !text.trim().is_empty() || attachment_count > 0 || buffered_count > 0
}

fn build_failed_reply_text(
    reply_prefix: Option<&str>,
    saw_stream_delta: bool,
    final_text: &str,
    error: Option<&str>,
) -> String {
    let display = if saw_stream_delta && !final_text.trim().is_empty() {
        format!(
            "{}\n\n_(处理中发生错误，内容可能不完整)_",
            final_text.trim()
        )
    } else {
        user_visible_error_message(error)
    };
    prepend_reply_prefix(reply_prefix, &display)
}

fn persist_visible_assistant_message(
    state: &Arc<AppState>,
    session_id: &str,
    content: &str,
    metadata: Option<HashMap<String, Value>>,
) {
    let _ = state
        .core
        .session_storage
        .add_message(session_id, "assistant", content, metadata);
}

#[async_trait]
impl EventHandler for FeishuEventHandler {
    fn event_type(&self) -> &str {
        "im.message.receive_v1"
    }

    fn handle(
        &self,
        event: Event,
    ) -> Pin<Box<dyn Future<Output = Result<Option<EventResp>, feishu_sdk::core::Error>> + Send + '_>>
    {
        let state = self.state.clone();
        Box::pin(async move {
            if let Some(msg) = parse_feishu_event(&state, event).await {
                tokio::spawn(async move {
                    let panic_state = state.clone();
                    let panic_msg = msg.clone();
                    let join = tokio::spawn(async move {
                        process_incoming_message(state, msg).await;
                    });
                    if let Err(err) = join.await {
                        error!("[Feishu] message handler join failed: {}", err);
                        if err.is_panic() {
                            if let Err(fallback_err) =
                                send_panic_fallback(&panic_state, &panic_msg).await
                            {
                                warn!("[Feishu] panic fallback send failed: {}", fallback_err);
                            }
                        }
                    }
                });
            }
            Ok(None)
        })
    }
}

const RESTART_RECOVERY_WINDOW_MINUTES: i64 = 30;
const RESTART_RECOVERY_GRACE_SECONDS: i64 = 30;
const RESTART_RECOVERY_TEXT: &str =
    "服务重启，之前的消息处理已中断，请稍后重试。";

async fn recover_interrupted_sessions(
    core: &hone_channels::HoneBotCore,
    facade: &FeishuApiClient,
) {
    let now = chrono::Utc::now();
    let updated_after = (now
        - chrono::TimeDelta::minutes(RESTART_RECOVERY_WINDOW_MINUTES))
    .to_rfc3339();
    let updated_before = (now
        - chrono::TimeDelta::seconds(RESTART_RECOVERY_GRACE_SECONDS))
    .to_rfc3339();

    let interrupted = match core
        .session_storage
        .find_interrupted_sessions("feishu", &updated_after, &updated_before)
    {
        Ok(list) => list,
        Err(err) => {
            warn!("[Feishu] 启动恢复：查询中断会话失败: {err}");
            return;
        }
    };

    if interrupted.is_empty() {
        return;
    }
    info!(
        "[Feishu] 启动恢复：发现 {} 个中断会话，补发失败提示",
        interrupted.len()
    );

    for session_info in &interrupted {
        // Only recover unscoped (direct) sessions — group sessions would need
        // a chat_id to reply to, which we don't have here.
        if session_info.actor_channel_scope.is_some() {
            continue;
        }
        let receive_id = &session_info.actor_user_id;
        if let Err(err) =
            send_plain_text(facade, receive_id, "open_id", RESTART_RECOVERY_TEXT).await
        {
            warn!(
                "[Feishu] 启动恢复：补发失败提示失败: session_id={} err={}",
                session_info.session_id, err
            );
        } else {
            // Record the failure reply in the session so last_message_role
            // flips to 'assistant' and we don't re-notify on the next restart.
            let _ = core.session_storage.add_message(
                &session_info.session_id,
                "assistant",
                RESTART_RECOVERY_TEXT,
                None,
            );
            info!(
                "[Feishu] 启动恢复：已补发失败提示: session_id={}",
                session_info.session_id
            );
        }
    }
}

pub(crate) async fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let runtime = hone_channels::bootstrap_channel_runtime(
        "feishu",
        "Feishu 渠道",
        hone_core::PROCESS_LOCK_FEISHU,
        |config| config.feishu.enabled,
    );
    let core = runtime.core;

    let app_id = core.config.feishu.app_id.trim().to_string();
    let app_secret = core.config.feishu.app_secret.trim().to_string();

    if app_id.is_empty() || app_secret.is_empty() {
        eprintln!("❌ 缺少 feishu.app_id 或 feishu.app_secret 配置!");
        std::process::exit(1);
    }

    let facade = FeishuApiClient::new(app_id.clone(), app_secret.clone());

    let state = Arc::new(AppState {
        core: core.clone(),
        facade: facade.clone(),
        dedup: MessageDeduplicator::new(Duration::from_secs(60), 4096),
        scheduled_dedup: MessageDeduplicator::new(Duration::from_secs(15 * 60), 8192),
        session_locks: SessionLockRegistry::new(),
        scope_resolver: ActorScopeResolver::new("feishu"),
        pretrigger: hone_channels::ingress::GroupPretriggerWindowRegistry::new(
            core.config.group_context.pretrigger_window_max_messages,
            Duration::from_secs(core.config.group_context.pretrigger_window_max_age_seconds),
        ),
    });

    // Recover sessions that were in-flight when the process was last killed.
    // We look for direct sessions whose last message was from the user (no reply
    // persisted) in the past 30 minutes but at least 30 seconds ago (grace period
    // for sessions just starting).
    recover_interrupted_sessions(&core, &facade).await;

    let sdk_logger = new_logger(FeishuLogLevel::Info);
    let event_config = EventDispatcherConfig::new();
    let dispatcher = EventDispatcher::new(event_config, sdk_logger.clone());
    dispatcher
        .register_handler(Box::new(FeishuEventHandler {
            state: state.clone(),
        }))
        .await;

    let feishu_config = FeishuConfig::builder(&app_id, &app_secret)
        .log_level(FeishuLogLevel::Info)
        .build();
    let stream_client = StreamClient::new(feishu_config, dispatcher)
        .expect("Failed to create feishu stream client");

    let stream_handle = stream_client.spawn();

    let (scheduler, event_rx) = core.create_scheduler(vec!["feishu".to_string()]);
    tokio::spawn(async move {
        scheduler.start().await;
    });

    let scheduler_state = state.clone();
    tokio::spawn(async move {
        handle_scheduler_events(scheduler_state, event_rx).await;
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        result = stream_handle => {
            match result {
                Ok(Ok(())) => {
                    error!("Feishu StreamClient stopped without an explicit error");
                }
                Ok(Err(err)) => {
                    error!("Feishu StreamClient exited with error: {err}");
                }
                Err(err) => {
                    error!("Feishu StreamClient join failed: {err}");
                }
            }
        }
    }
    info!("👋 Feishu 渠道已停止");
}

async fn process_incoming_message(state: Arc<AppState>, msg: FeishuIncomingMessage) {
    let chat_type = msg.chat_type.as_deref().unwrap_or("p2p");
    let is_group = chat_type != "p2p";
    if is_group && !state.core.config.feishu.chat_scope.allows_group() {
        warn!(
            "[Feishu] chat_scope 拒绝群聊消息: chat_type={} chat_id={}",
            chat_type, msg.chat_id
        );
        return;
    }
    if !is_group && !state.core.config.feishu.chat_scope.allows_direct() {
        warn!("[Feishu] chat_scope 拒绝私聊消息: open_id={}", msg.open_id);
        return;
    }

    if state.dedup.is_duplicate(&msg.message_id) {
        warn!(
            "[Feishu] 重复消息已忽略(dedup): message_id={}",
            msg.message_id
        );
        return;
    }

    let text = msg.text.trim();

    let normalized_email = msg.email.as_ref().map(|value| value.trim().to_lowercase());
    let normalized_mobile = msg.mobile.as_ref().map(|value| normalize_mobile(value));
    if !is_allowed_contact(
        &msg.open_id,
        normalized_email.as_deref(),
        normalized_mobile.as_deref(),
        &state.core.config.feishu.allow_open_ids,
        &state.core.config.feishu.allow_emails,
        &state.core.config.feishu.allow_mobiles,
    ) {
        warn!(
            "[Feishu] 白名单拒绝: email={:?} mobile={:?} open_id={}",
            normalized_email, normalized_mobile, msg.open_id
        );
        return;
    }

    let preferred_contact = normalized_email
        .clone()
        .or_else(|| normalized_mobile.clone());
    let log_user = preferred_contact
        .clone()
        .unwrap_or_else(|| msg.open_id.clone());
    let outbound_receive_id = if chat_type == "p2p" {
        msg.open_id.clone()
    } else {
        msg.chat_id.clone()
    };
    let outbound_receive_id_type = if chat_type == "p2p" {
        "open_id"
    } else {
        "chat_id"
    };
    let reply_prefix = if chat_type == "p2p" {
        None
    } else {
        Some(feishu_user_mention(&msg.open_id))
    };
    let channel_target = preferred_contact
        .clone()
        .unwrap_or_else(|| msg.open_id.clone());
    let (actor, _, chat_mode) = if chat_type == "p2p" {
        state
            .scope_resolver
            .direct(&msg.open_id, channel_target.clone())
            .expect("feishu direct actor should be valid")
    } else {
        state
            .scope_resolver
            .group(
                &msg.open_id,
                format!("chat:{}", msg.chat_id),
                channel_target.clone(),
            )
            .expect("feishu group actor should be valid")
    };
    let session_identity = SessionIdentity::from_actor(&actor)
        .expect("feishu actor should always map to a session identity");
    let session_id = session_identity.session_id();
    let speaker_label = feishu_speaker_label(
        &msg.open_id,
        normalized_email.as_deref(),
        normalized_mobile.as_deref(),
    );

    if is_group && !msg.has_mention {
        if !text.is_empty() && state.core.config.group_context.pretrigger_window_enabled {
            state
                .pretrigger
                .push(
                    &session_id,
                    BufferedGroupMessage::new(
                        "feishu",
                        msg.message_id.clone(),
                        speaker_label,
                        text.to_string(),
                    ),
                )
                .await;
            info!(
                "[Feishu] 群聊消息已写入预触发窗口: chat_id={} message_id={} session_id={}",
                msg.chat_id, msg.message_id, session_id
            );
        } else {
            warn!(
                "[Feishu] 群聊消息未@触发已忽略: chat_type={} chat_id={}",
                chat_type, msg.chat_id
            );
        }
        return;
    }

    if let Some(reply) = state.core.try_handle_intercept_command(&actor, text).await {
        if let Err(err) = send_plain_text(
            &state.facade,
            &outbound_receive_id,
            outbound_receive_id_type,
            &reply,
        )
        .await
        {
            warn!("[Feishu] 发送指令拦截确认失败: {err}");
        }
        return;
    }
    let _active_guard = match state.session_locks.try_begin_active(
        &session_id,
        ActiveSessionInfo {
            speaker_label: speaker_label.clone(),
            message_id: Some(msg.message_id.clone()),
        },
    ) {
        Ok(guard) => Some(guard),
        Err(active) if is_group => {
            if !text.is_empty() && state.core.config.group_context.pretrigger_window_enabled {
                state
                    .pretrigger
                    .push(
                        &session_id,
                        BufferedGroupMessage::new(
                            "feishu",
                            msg.message_id.clone(),
                            speaker_label.clone(),
                            text.to_string(),
                        ),
                    )
                    .await;
            }
            let busy_text = prepend_reply_prefix(
                reply_prefix.as_deref(),
                &build_group_busy_text(&active.speaker_label),
            );
            if let Err(err) = send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                &busy_text,
            )
            .await
            {
                warn!("[Feishu] 发送群聊 busy 提示失败: {err}");
            }
            state.core.log_message_step(
                "feishu",
                &log_user,
                &session_id,
                "group.busy",
                "sent",
                Some(&msg.message_id),
                Some("busy"),
            );
            warn!(
                "[Feishu] 群聊触发命中 busy，已回提示并保留到预触发窗口: chat_id={} active_speaker={}",
                msg.chat_id, active.speaker_label
            );
            return;
        }
        Err(active) => {
            let busy_text = prepend_reply_prefix(reply_prefix.as_deref(), build_direct_busy_text());
            if let Err(err) = send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                &busy_text,
            )
            .await
            {
                warn!("[Feishu] 发送私聊 busy 提示失败: {err}");
            }
            state.core.log_message_step(
                "feishu",
                &log_user,
                &session_id,
                "direct.busy",
                "sent",
                Some(&msg.message_id),
                Some("busy"),
            );
            warn!(
                "[Feishu] 私聊触发命中 busy，已跳过 placeholder: session_id={} active_message_id={:?}",
                session_id, active.message_id
            );
            return;
        }
    };

    let attachments = ingest_raw_attachments(
        state.core.as_ref(),
        AttachmentIngestRequest {
            channel: "feishu".to_string(),
            actor: actor.clone(),
            session_id: session_id.clone(),
            attachments: collect_raw_attachments(&msg),
        },
    )
    .await;
    if !attachments.is_empty() {
        spawn_attachment_persist_pipeline(
            state.core.clone(),
            AttachmentPersistRequest {
                channel: "feishu".to_string(),
                actor: actor.clone(),
                user_id: log_user.clone(),
                session_id: session_id.clone(),
                attachments: attachments.clone(),
            },
        );
    }
    if state
        .core
        .session_storage
        .load_session(&session_id)
        .ok()
        .flatten()
        .is_none()
    {
        let _ = state
            .core
            .session_storage
            .create_session_for_identity(&session_identity, Some(&actor));
    }
    let buffered_messages = if is_group && state.core.config.group_context.pretrigger_window_enabled
    {
        state
            .pretrigger
            .take_recent(&session_id, Some(&msg.message_id))
            .await
    } else {
        Vec::new()
    };
    let buffered_count = persist_buffered_group_messages(
        &state.core.session_storage,
        &session_id,
        &buffered_messages,
    )
    .unwrap_or(0);

    let attachment_count = attachments.len();
    let has_actionable_input = has_actionable_user_input(text, attachment_count, buffered_count);
    if !has_actionable_input {
        let display = prepend_reply_prefix(reply_prefix.as_deref(), build_unparsed_message_text());
        if let Err(err) = send_plain_text(
            &state.facade,
            &outbound_receive_id,
            outbound_receive_id_type,
            &display,
        )
        .await
        {
            warn!("[Feishu] 发送空输入兜底提示失败: {}", err);
        }
        state.core.log_message_step(
            "feishu",
            &log_user,
            &session_id,
            "message.empty_payload",
            &format!(
                "skipped message_type={} text_chars=0 attachments=0 buffered_messages={buffered_count}",
                msg.message_type.as_deref().unwrap_or("unknown")
            ),
            Some(&msg.message_id),
            Some("ignored"),
        );
        warn!(
            "[Feishu] 消息未解析出可处理内容，已跳过主链路: session_id={} message_id={} message_type={:?}",
            session_id, msg.message_id, msg.message_type
        );
        return;
    }

    state.core.log_message_step(
        "feishu",
        &log_user,
        &session_id,
        "message.accepted",
        &format!(
            "message_type={} text_chars={} attachments={} buffered_messages={buffered_count}",
            msg.message_type.as_deref().unwrap_or("unknown"),
            text.chars().count(),
            attachment_count,
        ),
        Some(&msg.message_id),
        None,
    );

    let recv_extra = if attachments.is_empty() {
        if buffered_count > 0 {
            Some(format!("buffered_messages={buffered_count}"))
        } else {
            None
        }
    } else {
        Some(format!(
            "attachments={} buffered_messages={buffered_count}",
            attachments.len()
        ))
    };

    let user_input = if attachments.is_empty() {
        let content = if text.is_empty() { "@bot" } else { text };
        if matches!(chat_mode, ChatMode::Group) {
            build_group_user_input_with_speaker(&speaker_label, content)
        } else {
            content.to_string()
        }
    } else {
        let content = build_user_input(text, &attachments);
        if matches!(chat_mode, ChatMode::Group) {
            build_group_user_input_with_speaker(&speaker_label, &content)
        } else {
            content
        }
    };

    let placeholder_text = if attachments.is_empty() {
        prepend_reply_prefix(reply_prefix.as_deref(), THINKING_PLACEHOLDER_TEXT)
    } else {
        prepend_reply_prefix(
            reply_prefix.as_deref(),
            &build_attachment_ack_message(&attachments),
        )
    };

    let is_admin = state.core.is_admin_actor(&actor)
        || normalized_email
            .as_deref()
            .or(normalized_mobile.as_deref())
            .or(Some(msg.open_id.as_str()))
            .map(|id| state.core.is_admin(id, "feishu"))
            .unwrap_or(false);

    let mut prompt_options = PromptOptions {
        is_admin,
        ..PromptOptions::default()
    };
    if matches!(chat_mode, ChatMode::Group) {
        prompt_options.privacy_guard = Some(FEISHU_GROUP_PRIVACY_GUARD.to_string());
    }

    let session_metadata = build_session_metadata(&msg, &normalized_email, &normalized_mobile);
    let metadata = message_metadata(
        &msg,
        normalized_email.as_deref(),
        normalized_mobile.as_deref(),
    );
    let user_metadata = if matches!(chat_mode, ChatMode::Group) {
        let mut metadata = metadata.clone();
        metadata.insert("speaker_id".to_string(), Value::String(msg.open_id.clone()));
        metadata.insert(
            "speaker_label".to_string(),
            Value::String(speaker_label.clone()),
        );
        metadata.insert(
            "channel_message_id".to_string(),
            Value::String(msg.message_id.clone()),
        );
        metadata
    } else {
        metadata.clone()
    };
    let message_metadata = MessageMetadata {
        user: Some(user_metadata),
        assistant: Some(metadata),
    };
    let assistant_message_metadata = message_metadata.assistant.clone();

    let envelope = IncomingEnvelope {
        message_id: Some(msg.message_id.clone()),
        actor: actor.clone(),
        session_identity,
        session_id: session_id.clone(),
        channel_target: channel_target.clone(),
        chat_mode,
        text: user_input.clone(),
        attachments: attachments.clone(),
        trigger: GroupTrigger {
            direct_mention: msg.has_mention,
            reply_to_bot: false,
            question_signal: false,
        },
        recv_extra: recv_extra.clone(),
        session_metadata: Some(session_metadata.clone()),
        message_metadata: message_metadata.clone(),
    };

    let mut session = AgentSession::new(
        state.core.clone(),
        envelope.actor.clone(),
        envelope.channel_target.clone(),
    )
    .with_session_identity(envelope.session_identity.clone())
    .with_message_id(envelope.message_id.clone())
    .with_prompt_options(prompt_options)
    .with_session_metadata(session_metadata)
    .with_message_metadata(message_metadata)
    .with_recv_extra(recv_extra.clone())
    .with_cron_allowed(envelope.cron_allowed());
    let content_buf = Arc::new(std::sync::RwLock::new(placeholder_text.clone()));
    let (placeholder_message_id, placeholder_card_id) = match send_placeholder_message(
        &state.facade,
        &outbound_receive_id,
        outbound_receive_id_type,
        &placeholder_text,
    )
    .await
    {
        Ok((message_id, card_id)) => {
            state.core.log_message_step(
                "feishu",
                &log_user,
                &session_id,
                "reply.placeholder",
                "sent",
                Some(&msg.message_id),
                None,
            );
            (Some(message_id), card_id)
        }
        Err(err) => {
            warn!("[Feishu] 发送占位消息失败: {}", err);
            state.core.log_message_step(
                "feishu",
                &log_user,
                &session_id,
                "reply.placeholder",
                "failed",
                Some(&msg.message_id),
                None,
            );
            (None, None)
        }
    };
    let cardkit_session: Option<Arc<CardKitSession>> =
        placeholder_card_id.as_deref().map(|card_id| {
            Arc::new(CardKitSession::new(
                state.facade.clone(),
                card_id.to_string(),
            ))
        });

    session.add_listener(Arc::new(FeishuStreamListener {
        buffer: content_buf.clone(),
        cardkit: cardkit_session.clone(),
        reasoning_visibility: if matches!(chat_mode, ChatMode::Group) {
            ReasoningVisibility::Compact
        } else {
            ReasoningVisibility::Full
        },
        think_formatter: Arc::new(std::sync::RwLock::new(ThinkStreamFormatter::new(
            ThinkRenderStyle::Hidden,
        ))),
    }));
    let stream_probe = attach_stream_activity_probe(&mut session);

    let ticker_handle = if cardkit_session.is_none() && placeholder_message_id.is_some() {
        let ticker_content = content_buf.clone();
        let ticker_facade = state.facade.clone();
        let ticker_pid = placeholder_message_id.clone();
        let ticker_log = log_user.to_string();
        Some(tokio::spawn(async move {
            let mut last_char_count = ticker_content.read().unwrap().chars().count();
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                let text = ticker_content.read().unwrap().clone();
                let char_count = text.chars().count();
                if char_count > last_char_count {
                    last_char_count = char_count;
                    if let Some(ref pid) = ticker_pid {
                        let processed = preprocess_markdown_for_feishu(&text, false);
                        let card = json!({
                            "schema": "2.0",
                            "config": {"wide_screen_mode": true},
                            "body": {
                                "elements": [
                                    {"tag": "markdown", "content": processed, "text_size": "normal"}
                                ]
                            }
                        })
                        .to_string();
                        if let Err(e) = ticker_facade
                            .update_message(pid, "interactive", &card)
                            .await
                        {
                            warn!(
                                "[Feishu/stream] [{}] ticker 更新卡片失败: {}",
                                ticker_log, e
                            );
                        }
                    }
                }
            }
        }))
    } else {
        None
    };

    let run_options = AgentRunOptions {
        timeout: Some(state.core.config.agent.overall_timeout()),
        segmenter: None,
        quota_mode: hone_channels::agent_session::AgentRunQuotaMode::UserConversation,
        model_override: None,
    };
    state.core.log_message_step(
        "feishu",
        &log_user,
        &session_id,
        "handler.session_run",
        "dispatch",
        Some(&msg.message_id),
        None,
    );
    let result = session.run(&user_input, run_options).await;
    state.core.log_message_step(
        "feishu",
        &log_user,
        &session_id,
        "handler.session_run",
        &format!(
            "completed success={} reply_chars={}",
            result.response.success,
            result.response.content.chars().count()
        ),
        Some(&msg.message_id),
        None,
    );

    if let Some(handle) = ticker_handle {
        handle.abort();
        let _ = handle.await;
    }

    let response = result.response;
    let saw_stream_delta = stream_probe.saw_stream_delta();
    let mut final_text = render_think_blocks(response.content.trim(), ThinkRenderStyle::Hidden);
    if final_text.is_empty() {
        final_text = content_buf.read().unwrap().trim().to_string();
    }

    if !response.success {
        let display = build_failed_reply_text(
            reply_prefix.as_deref(),
            saw_stream_delta,
            &final_text,
            response.error.as_deref(),
        );
        persist_visible_assistant_message(
            &state,
            &session_id,
            &display,
            assistant_message_metadata.clone(),
        );
        if let Some(ck) = &cardkit_session {
            ck.close(&preprocess_markdown_for_feishu(&display, true))
                .await;
        } else {
            if let Err(err) = update_or_send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                placeholder_message_id.as_deref(),
                &display,
            )
            .await
            {
                warn!("[Feishu] 发送失败兜底消息失败: {}", err);
            }
        }
        return;
    }

    if final_text.is_empty() {
        let fallback = prepend_reply_prefix(
            reply_prefix.as_deref(),
            "抱歉，没有获取到回复内容。请稍后再试。",
        );
        persist_visible_assistant_message(
            &state,
            &session_id,
            &fallback,
            assistant_message_metadata.clone(),
        );
        if let Some(ck) = &cardkit_session {
            ck.close(&fallback).await;
        } else {
            if let Err(err) = update_or_send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                placeholder_message_id.as_deref(),
                &fallback,
            )
            .await
            {
                warn!("[Feishu] 发送空回复兜底消息失败: {}", err);
            }
        }
        return;
    }

    final_text = prepend_reply_prefix(reply_prefix.as_deref(), &final_text);
    if let Some(ck) = &cardkit_session {
        let processed = preprocess_markdown_for_feishu(&final_text, true);
        ck.close(&processed).await;
        state.core.log_message_step(
            "feishu",
            &log_user,
            &session_id,
            "reply.send",
            "cardkit.close",
            Some(&msg.message_id),
            None,
        );
    } else {
        match send_rendered_messages(
            &state.facade,
            &outbound_receive_id,
            outbound_receive_id_type,
            &final_text,
            state.core.config.feishu.max_message_length,
            placeholder_message_id.as_deref(),
            None,
        )
        .await
        {
            Ok(sent_segments) => {
                state.core.log_message_step(
                    "feishu",
                    &log_user,
                    &session_id,
                    "reply.send",
                    &format!("segments.sent={sent_segments}/{sent_segments}"),
                    Some(&msg.message_id),
                    None,
                );
            }
            Err(err) => {
                warn!("[Feishu] 发送回复失败: {}", err);
            }
        }
    }
}

async fn send_panic_fallback(
    state: &Arc<AppState>,
    msg: &FeishuIncomingMessage,
) -> hone_core::HoneResult<usize> {
    let chat_type = msg.chat_type.as_deref().unwrap_or("p2p");
    let receive_id = if chat_type == "p2p" {
        msg.open_id.as_str()
    } else {
        msg.chat_id.as_str()
    };
    let receive_id_type = if chat_type == "p2p" {
        "open_id"
    } else {
        "chat_id"
    };
    let reply_prefix = if chat_type == "p2p" {
        None
    } else {
        Some(feishu_user_mention(&msg.open_id))
    };
    let display = prepend_reply_prefix(
        reply_prefix.as_deref(),
        "抱歉，这次处理失败了。请稍后再试。",
    );
    send_plain_text(&state.facade, receive_id, receive_id_type, &display).await
}

fn preferred_extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type.to_lowercase().split(';').next()?.trim() {
        "image/jpeg" => Some(".jpg"),
        "image/png" => Some(".png"),
        "image/gif" => Some(".gif"),
        "image/webp" => Some(".webp"),
        "image/bmp" => Some(".bmp"),
        "image/heic" => Some(".heic"),
        "image/svg+xml" => Some(".svg"),
        "application/pdf" => Some(".pdf"),
        _ => None,
    }
}

async fn parse_feishu_event(state: &Arc<AppState>, event: Event) -> Option<FeishuIncomingMessage> {
    let payload = event.event?;
    let message = payload.get("message")?;
    let sender = payload.get("sender")?;
    let open_id = sender
        .get("sender_id")?
        .get("open_id")?
        .as_str()?
        .to_string();

    let message_id = message.get("message_id")?.as_str()?.to_string();
    let chat_id = message.get("chat_id")?.as_str()?.to_string();
    let chat_type = message.get("chat_type")?.as_str().map(String::from);
    let message_type = message.get("message_type")?.as_str().map(String::from);
    let content_str = message.get("content")?.as_str()?;

    let content: Value = serde_json::from_str(content_str).ok()?;

    let mut text = String::new();
    let mut attachments = Vec::new();
    let mut has_mention = message
        .get("mentions")
        .and_then(|v| v.as_array())
        .map(|list| !list.is_empty())
        .unwrap_or(false);

    match message_type.as_deref() {
        Some("text") => {
            if let Some(t) = content.get("text").and_then(|v| v.as_str()) {
                text = t.to_string();
            }
            if content
                .get("mentions")
                .and_then(|v| v.as_array())
                .map(|list| !list.is_empty())
                .unwrap_or(false)
            {
                has_mention = true;
            } else if text.contains("<at ") {
                has_mention = true;
            }
        }
        Some("image") => {
            if let Some(image_key) = content.get("image_key").and_then(|v| v.as_str()) {
                let filename = format!("image_{}.bin", image_key);
                attachments.push(
                    download_attachment(state, &message_id, image_key, "image", &filename).await,
                );
            }
        }
        Some("file") => {
            if let Some(file_key) = content.get("file_key").and_then(|v| v.as_str()) {
                let filename = content
                    .get("file_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("file_{}.bin", file_key))
                    .to_string();
                attachments.push(
                    download_attachment(state, &message_id, file_key, "file", &filename).await,
                );
            }
        }
        Some("post") => {
            let (post_text, mut post_attachments, post_has_mention) =
                parse_post_content(state, &message_id, &content).await;
            text = post_text;
            attachments.append(&mut post_attachments);
            if post_has_mention {
                has_mention = true;
            }
        }
        _ => {}
    }

    let mut email = None;
    let mut mobile = None;
    match state.facade.get_user_by_open_id(&open_id).await {
        Ok(user) => {
            if !user.email.is_empty() {
                email = Some(user.email);
            }
            if !user.mobile.is_empty() {
                mobile = Some(user.mobile);
            }
        }
        Err(e) => {
            warn!("[Feishu] Failed to get user by open_id {}: {}", open_id, e);
        }
    }

    Some(FeishuIncomingMessage {
        message_id,
        chat_id,
        open_id,
        message_type,
        email,
        mobile,
        attachments,
        text,
        chat_type,
        has_mention,
    })
}

async fn parse_post_content(
    state: &Arc<AppState>,
    message_id: &str,
    content: &Value,
) -> (String, Vec<FeishuIncomingAttachment>, bool) {
    let mut text_parts = Vec::new();
    let mut attachments = Vec::new();
    let mut has_mention = false;

    if let Some(title) = content.get("title").and_then(|v| v.as_str()) {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            text_parts.push(trimmed.to_string());
        }
    }

    if let Some(content_array) = content.get("content").and_then(|v| v.as_array()) {
        for row in content_array {
            if let Some(nodes) = row.as_array() {
                let mut row_texts = Vec::new();
                for node in nodes {
                    let tag = node.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                    match tag {
                        "text" => {
                            if let Some(t) = node.get("text").and_then(|v| v.as_str()) {
                                row_texts.push(t.trim().to_string());
                            }
                        }
                        "at" => {
                            has_mention = true;
                            if let Some(t) = node.get("text").and_then(|v| v.as_str()) {
                                row_texts.push(t.trim().to_string());
                            }
                        }
                        "img" => {
                            if let Some(image_key) = node.get("image_key").and_then(|v| v.as_str())
                            {
                                let filename = format!("image_{}.bin", image_key);
                                attachments.push(
                                    download_attachment(
                                        state, message_id, image_key, "image", &filename,
                                    )
                                    .await,
                                );
                            }
                        }
                        "file" => {
                            if let Some(file_key) = node.get("file_key").and_then(|v| v.as_str()) {
                                let filename = node
                                    .get("file_name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&format!("file_{}.bin", file_key))
                                    .to_string();
                                attachments.push(
                                    download_attachment(
                                        state, message_id, file_key, "file", &filename,
                                    )
                                    .await,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                if !row_texts.is_empty() {
                    text_parts.push(row_texts.join(""));
                }
            }
        }
    }

    (text_parts.join("\n"), attachments, has_mention)
}

async fn download_attachment(
    state: &Arc<AppState>,
    message_id: &str,
    file_key: &str,
    resource_type: &str,
    fallback_name: &str,
) -> FeishuIncomingAttachment {
    let mut attachment = FeishuIncomingAttachment {
        filename: fallback_name.to_string(),
        content_type: None,
        size: None,
        url: format!(
            "feishu://message/{}/{}/{}",
            message_id, resource_type, file_key
        ),
        data: None,
        local_path: None,
        error: None,
    };

    match state
        .facade
        .download_resource(message_id, file_key, resource_type)
        .await
    {
        Ok((bytes, content_type)) => {
            attachment.size = Some(u32::try_from(bytes.len()).unwrap_or(u32::MAX));
            attachment.content_type = content_type.clone();

            let mut final_filename = fallback_name.to_string();
            if let Some(ct) = &content_type {
                if let Some(ext) = preferred_extension_for_content_type(ct) {
                    if final_filename.ends_with(".bin")
                        || final_filename.ends_with(".dat")
                        || final_filename.ends_with(".tmp")
                        || !final_filename.contains('.')
                    {
                        if let Some(dot_idx) = final_filename.rfind('.') {
                            final_filename = format!("{}{}", &final_filename[..dot_idx], ext);
                        } else {
                            final_filename = format!("{}{}", final_filename, ext);
                        }
                    }
                }
            }
            attachment.filename = final_filename.clone();
            attachment.data = Some(bytes);
        }
        Err(e) => {
            attachment.error = Some(e);
        }
    }

    attachment
}

fn build_session_metadata(
    msg: &FeishuIncomingMessage,
    normalized_email: &Option<String>,
    normalized_mobile: &Option<String>,
) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert("channel".to_string(), Value::String("feishu".to_string()));
    metadata.insert("open_id".to_string(), Value::String(msg.open_id.clone()));
    metadata.insert("chat_id".to_string(), Value::String(msg.chat_id.clone()));
    if let Some(email) = normalized_email {
        metadata.insert("email".to_string(), Value::String(email.clone()));
    }
    if let Some(mobile) = normalized_mobile {
        metadata.insert("mobile".to_string(), Value::String(mobile.clone()));
    }
    metadata
}

fn message_metadata(
    msg: &FeishuIncomingMessage,
    normalized_email: Option<&str>,
    normalized_mobile: Option<&str>,
) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert("channel".to_string(), Value::String("feishu".to_string()));
    metadata.insert(
        "message_id".to_string(),
        Value::String(msg.message_id.clone()),
    );
    if let Some(message_type) = &msg.message_type {
        metadata.insert(
            "message_type".to_string(),
            Value::String(message_type.clone()),
        );
    }
    metadata.insert("open_id".to_string(), Value::String(msg.open_id.clone()));
    metadata.insert("chat_id".to_string(), Value::String(msg.chat_id.clone()));
    if let Some(chat_type) = &msg.chat_type {
        metadata.insert("chat_type".to_string(), Value::String(chat_type.clone()));
    }
    if let Some(email) = normalized_email {
        metadata.insert("email".to_string(), Value::String(email.to_string()));
    }
    if let Some(mobile) = normalized_mobile {
        metadata.insert("mobile".to_string(), Value::String(mobile.to_string()));
    }
    metadata
}

fn is_allowed_contact(
    open_id: &str,
    email: Option<&str>,
    mobile: Option<&str>,
    allow_open_ids: &[String],
    allow_emails: &[String],
    allow_mobiles: &[String],
) -> bool {
    if allow_open_ids.is_empty() && allow_emails.is_empty() && allow_mobiles.is_empty() {
        return true;
    }

    if allow_open_ids.iter().any(|item| item.trim() == "*")
        || allow_emails.iter().any(|item| item.trim() == "*")
        || allow_mobiles.iter().any(|item| item.trim() == "*")
    {
        return true;
    }

    if allow_open_ids
        .iter()
        .any(|item| !item.trim().is_empty() && item.trim() == open_id)
    {
        return true;
    }

    if let Some(email) = email {
        if allow_emails
            .iter()
            .any(|item| item.trim().eq_ignore_ascii_case(email))
        {
            return true;
        }
    }

    if let Some(mobile) = mobile {
        if allow_mobiles
            .iter()
            .map(|item| normalize_mobile(item))
            .any(|item| !item.is_empty() && item == mobile)
        {
            return true;
        }
    }

    false
}

fn normalize_mobile(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '+')
        .collect()
}

pub(crate) async fn resolve_receive_id(
    facade: &FeishuApiClient,
    channel_target: &str,
) -> hone_core::HoneResult<String> {
    let target = channel_target.trim();
    if target.contains('@') {
        return Ok(facade
            .resolve_email(target)
            .await
            .map_err(hone_core::HoneError::Integration)?
            .open_id);
    }
    if looks_like_mobile(target) {
        return Ok(facade
            .resolve_mobile(target)
            .await
            .map_err(hone_core::HoneError::Integration)?
            .open_id);
    }
    Ok(target.to_string())
}

fn looks_like_mobile(target: &str) -> bool {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | ' ' | '-' | '(' | ')'))
    {
        return false;
    }
    let normalized = normalize_mobile(target);
    !normalized.is_empty() && normalized.chars().filter(|ch| ch.is_ascii_digit()).count() >= 7
}

pub(crate) fn scheduler_receive_id_for_target(
    _actor: &ActorIdentity,
    _channel_target: &str,
) -> Option<String> {
    // Always resolve via the Feishu API (resolve_receive_id) so we get the
    // current-app-scoped open_id. The old short-circuit that returned
    // actor.user_id directly caused "open_id cross app" (code 99992361) when
    // the app was migrated and the stored open_id no longer matched the
    // active app's binding.
    None
}

pub(crate) fn validate_scheduler_receive_id(
    _actor: &ActorIdentity,
    _channel_target: &str,
    _receive_id: &str,
) -> hone_core::HoneResult<()> {
    // Validation previously rejected API-resolved open_ids that didn't match
    // the stored actor.user_id. Now that we always resolve via the Feishu API
    // the returned open_id is authoritative for the current app; no extra
    // validation is needed.
    Ok(())
}

fn collect_raw_attachments(msg: &FeishuIncomingMessage) -> Vec<RawAttachment> {
    let mut out = Vec::with_capacity(msg.attachments.len());
    for attachment in &msg.attachments {
        let filename = attachment.filename.trim();
        out.push(RawAttachment {
            filename: if filename.is_empty() {
                "attachment.bin".to_string()
            } else {
                filename.to_string()
            },
            content_type: attachment.content_type.clone(),
            size: attachment.size,
            url: attachment.url.clone(),
            data: attachment.data.clone(),
            local_path: attachment.local_path.clone().map(std::path::PathBuf::from),
            error: attachment.error.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::ActorIdentity;

    #[test]
    fn allow_list_empty_means_allow_all() {
        assert!(is_allowed_contact("ou_x", None, None, &[], &[], &[]));
    }

    #[test]
    fn allow_list_supports_star_and_exact_email() {
        assert!(is_allowed_contact(
            "ou_x",
            Some("alice@example.com"),
            None,
            &[],
            &["*".to_string()],
            &[],
        ));
        assert!(is_allowed_contact(
            "ou_x",
            Some("alice@example.com"),
            None,
            &[],
            &["alice@example.com".to_string()],
            &[],
        ));
        assert!(!is_allowed_contact(
            "ou_x",
            Some("alice@example.com"),
            None,
            &[],
            &["bob@example.com".to_string()],
            &[],
        ));
    }

    #[test]
    fn allow_list_supports_exact_mobile() {
        assert!(is_allowed_contact(
            "ou_x",
            None,
            Some("+8613800138000"),
            &[],
            &[],
            &["13800138000".to_string(), "+8613800138000".to_string()],
        ));
        assert!(!is_allowed_contact(
            "ou_x",
            None,
            Some("+8613800138000"),
            &[],
            &[],
            &["13900139000".to_string()],
        ));
    }

    #[test]
    fn allow_list_supports_open_id() {
        assert!(is_allowed_contact(
            "ou_794ef8c84e1704cbbc56aa95d9688965",
            None,
            None,
            &["ou_794ef8c84e1704cbbc56aa95d9688965".to_string()],
            &[],
            &[],
        ));
    }

    #[test]
    fn scheduler_delivery_validation_is_always_ok() {
        // validate_scheduler_receive_id is now a no-op: the API-resolved
        // open_id is authoritative for the current app and needs no comparison
        // against the potentially-stale actor.user_id.
        let actor = ActorIdentity::new("feishu", "ou_creator", None::<String>).expect("actor");
        assert!(validate_scheduler_receive_id(&actor, "alice@example.com", "ou_other").is_ok());
        assert!(validate_scheduler_receive_id(&actor, "alice@example.com", "ou_creator").is_ok());
        assert!(validate_scheduler_receive_id(&actor, "+8613800138000", "ou_creator").is_ok());
        let actor_group =
            ActorIdentity::new("feishu", "ou_creator", Some("chat:42")).expect("actor");
        assert!(
            validate_scheduler_receive_id(&actor_group, "alice@example.com", "ou_other").is_ok()
        );
    }

    #[test]
    fn looks_like_mobile_does_not_treat_open_id_as_mobile() {
        assert!(!looks_like_mobile("ou_e31244b1208749f16773dce0c822801a"));
        assert!(looks_like_mobile("+8613800138000"));
        assert!(looks_like_mobile("138-0013-8000"));
    }

    #[test]
    fn direct_scheduler_always_falls_through_to_api_resolution() {
        let actor = ActorIdentity::new("feishu", "ou_creator", None::<String>).expect("actor");
        // All targets return None so the caller always invokes resolve_receive_id
        // (Feishu API), avoiding cross-app open_id errors.
        assert_eq!(
            scheduler_receive_id_for_target(&actor, "alice@example.com"),
            None
        );
        assert_eq!(
            scheduler_receive_id_for_target(&actor, "+8613800138000"),
            None
        );
        assert_eq!(scheduler_receive_id_for_target(&actor, "ou_other"), None);
    }

    #[test]
    fn failed_reply_text_maps_idle_timeout_to_friendly_message() {
        assert_eq!(
            build_failed_reply_text(
                None,
                false,
                "",
                Some("opencode acp session/prompt idle timeout (180s)"),
            ),
            "抱歉，处理超时了。请稍后再试。"
        );
    }

    #[test]
    fn failed_reply_text_keeps_partial_stream_output() {
        assert_eq!(
            build_failed_reply_text(
                Some("@alice"),
                true,
                "阶段性结果",
                Some("opencode acp session/prompt idle timeout (180s)"),
            ),
            "@alice 阶段性结果\n\n_(处理中发生错误，内容可能不完整)_"
        );
    }

    #[test]
    fn direct_busy_text_is_explicit() {
        assert_eq!(
            build_direct_busy_text(),
            "上一条消息还在处理中，请等当前回复完成后再发送新消息。"
        );
    }

    #[test]
    fn actionable_user_input_detects_empty_payload() {
        assert!(!has_actionable_user_input("", 0, 0));
        assert!(has_actionable_user_input("1", 0, 0));
        assert!(has_actionable_user_input("", 1, 0));
        assert!(has_actionable_user_input("", 0, 1));
    }
}
