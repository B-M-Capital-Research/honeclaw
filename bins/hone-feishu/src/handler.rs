use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use feishu_sdk::core::{Config as FeishuConfig, noop_logger};
use feishu_sdk::event::{Event, EventDispatcher, EventDispatcherConfig, EventHandler, EventResp};
use feishu_sdk::ws::StreamClient;
use hone_channels::ChatMode;
use hone_channels::agent_session::{AgentRunOptions, AgentSession, MessageMetadata};
use hone_channels::attachments::{
    AttachmentIngestRequest, AttachmentPersistRequest, RawAttachment, build_attachment_ack_message,
    build_user_input, ingest_raw_attachments, spawn_attachment_persist_pipeline,
};
use hone_channels::channel_download_dir;
use hone_channels::ingress::{
    ActorScopeResolver, GroupTrigger, IncomingEnvelope, MessageDeduplicator, SessionLockRegistry,
};
use hone_channels::outbound::attach_stream_activity_probe;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_core::SessionIdentity;
use hone_scheduler::SchedulerEvent;
use serde_json::{Value, json};
use tracing::{error, info, warn};

use super::card::CardKitSession;
use super::client::FeishuApiClient;
use super::listener::FeishuStreamListener;
use super::markdown::{preprocess_markdown_for_feishu, render_outbound_messages};
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
                    process_incoming_message(state, msg).await;
                });
            }
            Ok(None)
        })
    }
}

pub(crate) async fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let (config, config_path) = match hone_channels::load_runtime_config() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("❌ 配置加载失败: {e}");
            std::process::exit(1);
        }
    };
    let core = hone_channels::HoneBotCore::new(config);

    hone_core::logging::setup_logging(&core.config.logging);
    info!("📨 Hone Feishu 渠道启动");
    core.log_startup_routing("feishu", &config_path);

    if !core.config.feishu.enabled {
        warn!("feishu.enabled=false，Feishu 渠道不会启动。");
        return;
    }

    let _heartbeat = match hone_core::spawn_process_heartbeat(&core.config, "feishu") {
        Ok(heartbeat) => heartbeat,
        Err(err) => {
            error!("无法启动 Feishu heartbeat: {err}");
            std::process::exit(1);
        }
    };

    let app_id = core.config.feishu.app_id.trim().to_string();
    let app_secret = core.config.feishu.app_secret.trim().to_string();

    if app_id.is_empty() || app_secret.is_empty() {
        eprintln!("❌ 缺少 feishu.app_id 或 feishu.app_secret 配置!");
        std::process::exit(1);
    }

    let core = Arc::new(core);
    let facade = FeishuApiClient::new(app_id.clone(), app_secret.clone());

    let state = Arc::new(AppState {
        core: core.clone(),
        facade: facade.clone(),
        dedup: MessageDeduplicator::new(Duration::from_secs(60), 4096),
        session_locks: SessionLockRegistry::new(),
        scope_resolver: ActorScopeResolver::new("feishu"),
    });

    let event_config = EventDispatcherConfig::new();
    let dispatcher = EventDispatcher::new(event_config, noop_logger());
    dispatcher
        .register_handler(Box::new(FeishuEventHandler {
            state: state.clone(),
        }))
        .await;

    let feishu_config = FeishuConfig::builder(&app_id, &app_secret).build();
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
        _ = stream_handle => {
            error!("Feishu StreamClient background task exited unexpectedly");
        }
    }
    info!("👋 Feishu 渠道已停止");
}

async fn process_incoming_message(state: Arc<AppState>, msg: FeishuIncomingMessage) {
    let chat_type = msg.chat_type.as_deref().unwrap_or("p2p");
    if state.core.config.feishu.dm_only && chat_type != "p2p" {
        warn!(
            "[Feishu] 忽略非私聊消息: chat_type={} chat_id={}",
            chat_type, msg.chat_id
        );
        return;
    }
    if chat_type != "p2p" && !msg.has_mention {
        warn!(
            "[Feishu] 群聊消息未@触发已忽略: chat_type={} chat_id={}",
            chat_type, msg.chat_id
        );
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
    if state.core.try_intercept_admin_registration(&actor, text) {
        if let Err(err) = send_plain_text(
            &state.facade,
            &outbound_receive_id,
            outbound_receive_id_type,
            hone_channels::core::REGISTER_ADMIN_INTERCEPT_ACK,
        )
        .await
        {
            warn!("[Feishu] 发送管理员拦截确认失败: {err}");
        }
        return;
    }
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
    if text.is_empty() && attachments.is_empty() {
        return;
    }
    if !attachments.is_empty() {
        spawn_attachment_persist_pipeline(
            state.core.clone(),
            AttachmentPersistRequest {
                channel: "feishu".to_string(),
                user_id: log_user.clone(),
                session_id: session_id.clone(),
                attachments: attachments.clone(),
            },
        );
    }

    let _session_guard = state.session_locks.lock(&session_id).await;

    let recv_extra = if attachments.is_empty() {
        None
    } else {
        Some(format!("attachments={}", attachments.len()))
    };

    let speaker_label = feishu_speaker_label(
        &msg.open_id,
        normalized_email.as_deref(),
        normalized_mobile.as_deref(),
    );

    let user_input = if attachments.is_empty() {
        if matches!(chat_mode, ChatMode::Group) {
            build_group_user_input_with_speaker(&speaker_label, text)
        } else {
            text.to_string()
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
        show_reasoning: true,
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

    let timeout_secs = state.core.config.llm.openrouter.timeout.max(180);
    let run_options = AgentRunOptions {
        timeout: Some(Duration::from_secs(timeout_secs)),
        segmenter: None,
        quota_mode: hone_channels::agent_session::AgentRunQuotaMode::UserConversation,
    };
    let result = session.run(&user_input, run_options).await;

    if let Some(handle) = ticker_handle {
        handle.abort();
        let _ = handle.await;
    }

    let response = result.response;
    let saw_stream_delta = stream_probe.saw_stream_delta();
    let mut final_text = response.content.trim().to_string();
    if final_text.is_empty() {
        final_text = content_buf.read().unwrap().trim().to_string();
    }

    if !response.success {
        let err = response.error.unwrap_or_else(|| "未知错误".to_string());
        let truncated: String = err.chars().take(120).collect();
        let display = if saw_stream_delta && !final_text.is_empty() {
            format!("{}\n\n_(处理中发生错误，内容可能不完整)_", final_text)
        } else {
            format!("抱歉，处理出错了: {}", truncated)
        };
        let display = prepend_reply_prefix(reply_prefix.as_deref(), &display);
        if let Some(ck) = &cardkit_session {
            ck.close(&preprocess_markdown_for_feishu(&display, true))
                .await;
        } else {
            let _ = update_or_send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                placeholder_message_id.as_deref(),
                &display,
            )
            .await;
        }
        return;
    }

    if final_text.is_empty() {
        let fallback = prepend_reply_prefix(
            reply_prefix.as_deref(),
            "抱歉，没有获取到回复内容。请稍后再试。",
        );
        if let Some(ck) = &cardkit_session {
            ck.close(&fallback).await;
        } else {
            let _ = update_or_send_plain_text(
                &state.facade,
                &outbound_receive_id,
                outbound_receive_id_type,
                placeholder_message_id.as_deref(),
                &fallback,
            )
            .await;
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

async fn handle_scheduler_events(
    state: Arc<AppState>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: feishu）");
    while let Some(event) = event_rx.recv().await {
        if event.channel != "feishu" {
            continue;
        }

        let state_clone = state.clone();
        tokio::spawn(async move {
            let response = run_scheduled_task(&state_clone, &event).await;
            let receive_id =
                match resolve_receive_id(&state_clone.facade, &event.channel_target).await {
                    Ok(id) => id,
                    Err(err) => {
                        error!(
                            "[Feishu] 定时任务目标解析失败: job={} target={} err={}",
                            event.job_name, event.channel_target, err
                        );
                        return;
                    }
                };

            if let Err(err) = send_rendered_messages(
                &state_clone.facade,
                &receive_id,
                "open_id",
                &response,
                state_clone.core.config.feishu.max_message_length,
                None,
            )
            .await
            {
                error!(
                    "[Feishu] 定时任务投递失败: job={} target={} err={}",
                    event.job_name, event.channel_target, err
                );
            }
        });
    }
}

async fn run_scheduled_task(state: &Arc<AppState>, event: &SchedulerEvent) -> String {
    let actor = &event.actor;
    let is_admin = state.core.is_admin_actor(actor);
    let prompt_options = PromptOptions {
        is_admin,
        ..PromptOptions::default()
    };
    let timeout_secs = state.core.config.llm.openrouter.timeout.max(180);
    let run_options = AgentRunOptions {
        timeout: Some(Duration::from_secs(timeout_secs)),
        segmenter: None,
        quota_mode: hone_channels::agent_session::AgentRunQuotaMode::ScheduledTask,
    };
    let result =
        scheduler::run_scheduled_task(state.core.clone(), event, prompt_options, run_options).await;
    let response = result.response;
    if response.success {
        response.content
    } else {
        response
            .error
            .unwrap_or_else(|| "定时任务执行失败".to_string())
    }
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
        local_path: None,
        error: None,
    };

    match state
        .facade
        .download_resource(message_id, file_key, resource_type)
        .await
    {
        Ok((bytes, content_type)) => {
            attachment.size = Some(bytes.len() as u32);
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

            let upload_dir = channel_download_dir("feishu");
            if let Err(e) = std::fs::create_dir_all(&upload_dir) {
                attachment.error = Some(format!("failed to create dir: {e}"));
                return attachment;
            }

            let file_path = upload_dir.join(format!(
                "{}_{}_{}",
                chrono::Utc::now().timestamp_millis(),
                resource_type,
                final_filename
            ));
            if let Err(e) = tokio::fs::write(&file_path, &bytes).await {
                attachment.error = Some(format!("failed to write file: {e}"));
                return attachment;
            }

            attachment.local_path = file_path
                .file_name()
                .map(|n| upload_dir.join(n).to_string_lossy().to_string());

            if let Ok(abs) = std::fs::canonicalize(&file_path) {
                attachment.local_path = Some(abs.to_string_lossy().to_string());
            }
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

async fn resolve_receive_id(
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
    let normalized = normalize_mobile(target);
    !normalized.is_empty() && normalized.chars().filter(|ch| ch.is_ascii_digit()).count() >= 7
}

async fn send_plain_text(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    text: &str,
) -> hone_core::HoneResult<usize> {
    if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
    } else {
        facade
            .send_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
    }
    Ok(1)
}

fn feishu_user_mention(open_id: &str) -> String {
    format!("<at id=\"{open_id}\"></at>")
}

fn prepend_reply_prefix(prefix: Option<&str>, text: &str) -> String {
    let Some(prefix) = prefix.map(str::trim).filter(|value| !value.is_empty()) else {
        return text.to_string();
    };

    let body = text.trim_start();
    if body.starts_with(prefix) {
        text.to_string()
    } else if body.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {body}")
    }
}

async fn send_placeholder_message(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    text: &str,
) -> hone_core::HoneResult<(String, Option<String>)> {
    let request_uuid = uuid::Uuid::new_v4().to_string();
    let card_content = json!({
        "schema": "2.0",
        "config": {"wide_screen_mode": true},
        "body": {
            "elements": [
                {"tag": "markdown", "content": text, "text_size": "heading"}
            ]
        }
    })
    .to_string();
    let result = if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "interactive",
                &card_content,
                Some(&request_uuid),
            )
            .await
    } else {
        facade
            .send_message(
                receive_id,
                "interactive",
                &card_content,
                Some(&request_uuid),
            )
            .await
    }
    .map_err(hone_core::HoneError::Integration)?;
    Ok((result.message_id, None))
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
            local_path: attachment.local_path.clone().map(std::path::PathBuf::from),
            data: None,
            error: attachment.error.clone(),
        });
    }
    out
}

async fn update_or_send_plain_text(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    placeholder_message_id: Option<&str>,
    text: &str,
) -> hone_core::HoneResult<usize> {
    if let Some(message_id) = placeholder_message_id {
        let processed = preprocess_markdown_for_feishu(text, true);
        let card_content = json!({
            "schema": "2.0",
            "config": {"wide_screen_mode": true},
            "body": {
                "elements": [
                    {
                        "tag": "markdown",
                        "content": processed,
                        "text_size": "heading"
                    }
                ]
            }
        })
        .to_string();
        facade
            .update_message(message_id, "interactive", &card_content)
            .await
            .map_err(hone_core::HoneError::Integration)?;
        return Ok(1);
    }

    if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
        Ok(1)
    } else {
        send_plain_text(facade, receive_id, receive_id_type, text).await
    }
}

async fn send_rendered_messages(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    markdown: &str,
    max_message_length: usize,
    placeholder_message_id: Option<&str>,
) -> hone_core::HoneResult<usize> {
    let messages = render_outbound_messages(markdown, max_message_length);
    if messages.is_empty() {
        return Ok(0);
    }

    let total = messages.len();
    let mut previous_message_id = None;
    for (index, message) in messages.into_iter().enumerate() {
        if index == 0 {
            if let Some(message_id) = placeholder_message_id {
                let card_content = if message.msg_type == "interactive" {
                    message.content.clone()
                } else if message.msg_type == "post" {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&message.content)
                    {
                        if let Some(zh_cn) = parsed.get("zh_cn") {
                            let mut text_lines = Vec::new();
                            if let Some(title) = zh_cn.get("title").and_then(|t| t.as_str()) {
                                if !title.is_empty() {
                                    text_lines.push(format!("**{}**", title));
                                }
                            }
                            if let Some(content) = zh_cn.get("content").and_then(|c| c.as_array()) {
                                for row in content {
                                    if let Some(elements) = row.as_array() {
                                        let mut line_text = String::new();
                                        for el in elements {
                                            if let Some(text) =
                                                el.get("text").and_then(|t| t.as_str())
                                            {
                                                line_text.push_str(text);
                                            }
                                        }
                                        text_lines.push(line_text);
                                    }
                                }
                            }
                            json!({
                                "schema": "2.0",
                                "config": {"wide_screen_mode": true},
                                "body": {
                                    "elements": [
                                        {
                                            "tag": "markdown",
                                            "content": preprocess_markdown_for_feishu(&text_lines.join("\n"), true),
                                            "text_size": "heading"
                                        }
                                    ]
                                }
                            })
                            .to_string()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                facade
                    .update_message(message_id, "interactive", &card_content)
                    .await
                    .map_err(hone_core::HoneError::Integration)?;
                previous_message_id = Some(message_id.to_string());
                continue;
            }
        }

        let request_uuid = uuid::Uuid::new_v4().to_string();
        let sent = if let Some(parent_id) = previous_message_id.as_deref() {
            facade
                .reply_message(
                    parent_id,
                    message.msg_type,
                    &message.content,
                    Some(&request_uuid),
                )
                .await
        } else if receive_id_type == "chat_id" {
            facade
                .send_chat_message(
                    receive_id,
                    message.msg_type,
                    &message.content,
                    Some(&request_uuid),
                )
                .await
        } else {
            facade
                .send_message(
                    receive_id,
                    message.msg_type,
                    &message.content,
                    Some(&request_uuid),
                )
                .await
        }
        .map_err(hone_core::HoneError::Integration)?;
        previous_message_id = Some(sent.message_id);
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
