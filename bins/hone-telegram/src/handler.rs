use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hone_channels::agent_session::{AgentRunOptions, AgentSession, MessageMetadata};
use hone_channels::attachments::{
    AttachmentIngestRequest, AttachmentPersistRequest, RawAttachment, build_attachment_ack_message,
    build_user_input, ingest_raw_attachments, spawn_attachment_persist_pipeline,
};
use hone_channels::channel_download_dir;
use hone_channels::ingress::{
    ActiveSessionInfo, ActorScopeResolver, BufferedGroupMessage, GroupTrigger, IncomingEnvelope,
    MessageDeduplicator, SessionLockRegistry, persist_buffered_group_messages,
};
use hone_channels::outbound::{run_session_with_outbound, split_segments};
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_core::SessionIdentity;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::SchedulerEvent;
use serde_json::json;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, Message, User};
use teloxide::update_listeners;
use teloxide::utils::html;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use super::listener::{
    TelegramOutboundAdapter, prepend_reply_prefix, send_message_with_fallback, send_segments,
};
use super::markdown_v2::sanitize_telegram_html_public;
use super::types::{MediaGroupBuffer, TelegramAppState};

const THINKING_PLACEHOLDER_TEXT: &str = "正在思考中...";
const TELEGRAM_GROUP_PRIVACY_GUARD: &str = "【群聊隐私约束】\
    \n1. 禁止在群聊索取或引导补全持仓明细（股数、成本、成交价、交易单等）。\
    \n2. 禁止在群聊查询或确认用户个人持仓；用户问“我现在持有哪些”时，直接提示转私聊处理。\
    \n3. 只提供通用信息与私聊引导，不给出任何个人资产判断或推断。";

fn telegram_speaker_label(user: &User) -> String {
    let full_name = format!(
        "{} {}",
        user.first_name.trim(),
        user.last_name.as_deref().unwrap_or("").trim()
    )
    .trim()
    .to_string();
    if !full_name.is_empty() {
        full_name
    } else if let Some(username) = user
        .username
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        username.to_string()
    } else {
        user.id.0.to_string()
    }
}

fn build_group_user_input_with_speaker(label: &str, text: &str) -> String {
    format!("[{label}] {}", text.trim())
}

fn build_group_busy_text(speaker_label: &str) -> String {
    format!("正在处理 {speaker_label} 的消息，请等上一条完成后再 @ 我。")
}

pub(crate) async fn run() {
    let (config, config_path) = match hone_channels::load_runtime_config() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("❌ 配置加载失败: {e}");
            std::process::exit(1);
        }
    };
    let core = Arc::new(hone_channels::HoneBotCore::new(config));

    hone_core::logging::setup_logging(&core.config.logging);
    info!("🤖 Hone Telegram Bot 启动");
    core.log_startup_routing("telegram", &config_path);

    if !core.config.telegram.enabled {
        warn!("telegram.enabled=false，Telegram Bot 不会启动。");
        return;
    }

    let _process_lock = match hone_core::acquire_runtime_process_lock(
        &core.config,
        hone_core::PROCESS_LOCK_TELEGRAM,
    ) {
        Ok(lock) => lock,
        Err(error) => {
            error!(
                "{}",
                hone_core::format_lock_failure_message(
                    hone_core::PROCESS_LOCK_TELEGRAM,
                    &hone_core::process_lock_path(
                        &hone_core::runtime_heartbeat_dir(&core.config),
                        hone_core::PROCESS_LOCK_TELEGRAM
                    ),
                    &error,
                    "Telegram"
                )
            );
            return;
        }
    };

    let _heartbeat = match hone_core::spawn_process_heartbeat(&core.config, "telegram") {
        Ok(heartbeat) => heartbeat,
        Err(err) => {
            error!("无法启动 Telegram heartbeat: {err}");
            return;
        }
    };

    let token = core.config.telegram.bot_token.trim().to_string();
    if token.is_empty() {
        warn!("⚠️  未设置 telegram.bot_token，请在 config.yaml 中配置");
        return;
    }

    let bot = Bot::new(token);
    let me = match bot.get_me().await {
        Ok(me) => me,
        Err(e) => {
            error!("无法获取 Telegram Bot 信息: {e}");
            return;
        }
    };
    let bot_id = me.user.id.0;
    let bot_username = me.user.username.clone().unwrap_or_default();
    let app_state = Arc::new(TelegramAppState {
        dedup: MessageDeduplicator::new(Duration::from_secs(120), 2048),
        session_locks: SessionLockRegistry::new(),
        scope_resolver: ActorScopeResolver::new("telegram"),
        pretrigger: hone_channels::ingress::GroupPretriggerWindowRegistry::new(
            core.config.group_context.pretrigger_window_max_messages,
            Duration::from_secs(core.config.group_context.pretrigger_window_max_age_seconds),
        ),
        media_groups: MediaGroupBuffer::new(),
    });

    let (scheduler, event_rx) = core.create_scheduler(vec!["telegram".to_string()]);
    tokio::spawn(async move {
        scheduler.start().await;
    });

    let scheduler_bot = bot.clone();
    let scheduler_core = core.clone();
    tokio::spawn(async move {
        handle_scheduler_events(scheduler_bot, scheduler_core, event_rx).await;
    });

    let handler = Update::filter_message().endpoint(handle_message);
    let listener = update_listeners::polling_default(bot.clone()).await;
    let error_handler = Arc::new(|err: teloxide::RequestError| async move {
        match err {
            teloxide::RequestError::Api(teloxide::ApiError::TerminatedByOtherGetUpdates) => {
                error!(
                    "Telegram 更新轮询被终止：检测到其他实例正在 getUpdates。请停止其它实例后再启动。"
                );
                std::process::exit(1);
            }
            other => {
                error!("Telegram update listener error: {other}");
            }
        }
    });
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            core,
            Arc::new(bot_username),
            bot_id,
            app_state
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, error_handler)
        .await;
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    core: Arc<hone_channels::HoneBotCore>,
    bot_username: Arc<String>,
    bot_id: u64,
    app_state: Arc<TelegramAppState>,
) -> ResponseResult<()> {
    let Some(user) = msg.from.clone() else {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            "telegram inbound ignored: missing sender"
        );
        return Ok(());
    };
    if user.is_bot {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            author_id = user.id.0,
            "telegram inbound ignored: sender is bot"
        );
        return Ok(());
    }

    let author_id = user.id.0.to_string();
    let raw_text = message_text(&msg);
    let is_private = matches!(msg.chat.kind, ChatKind::Private(_));
    let direct_mention = !is_private && is_direct_mention_message(&msg, &bot_username, bot_id);
    let reply_to_bot = is_reply_to_bot(&msg, bot_id);
    let media_group_id = msg.media_group_id().map(str::to_string);

    info!(
        chat_id = msg.chat.id.0,
        message_id = msg.id.0,
        author_id = author_id,
        is_private,
        direct_mention,
        reply_to_bot,
        media_group_id = media_group_id.as_deref().unwrap_or(""),
        text = raw_text.as_str(),
        "telegram inbound received"
    );

    if !is_allowed_author(&author_id, &core.config.telegram.allow_from) {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            author_id = author_id,
            "telegram inbound ignored: author not allowed"
        );
        return Ok(());
    }
    let dedup_key = format!("{}:{}", msg.chat.id.0, msg.id.0);
    if app_state.dedup.is_duplicate(&dedup_key) {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            dedup_key = dedup_key,
            "telegram inbound ignored: duplicate message"
        );
        return Ok(());
    }

    if !is_private && !core.config.telegram.chat_scope.allows_group() {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            "telegram inbound ignored: group chat blocked by chat_scope"
        );
        return Ok(());
    }
    if is_private && !core.config.telegram.chat_scope.allows_direct() {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            "telegram inbound ignored: direct chat blocked by chat_scope"
        );
        return Ok(());
    }

    let text = raw_text.trim();
    if text.is_empty() && !message_has_supported_attachments(&msg) {
        info!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            "telegram inbound ignored: empty text"
        );
        return Ok(());
    }

    if let Some(group_id) = media_group_id {
        let should_flush = app_state.media_groups.push(&group_id, msg.clone()).await;
        if should_flush {
            let bot_clone = bot.clone();
            let core_clone = core.clone();
            let bot_username_clone = bot_username.clone();
            let app_state_clone = app_state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(750)).await;
                let messages = app_state_clone.media_groups.take(&group_id).await;
                if messages.is_empty() {
                    return;
                }
                if let Err(err) = process_telegram_message_batch(
                    bot_clone,
                    core_clone,
                    bot_username_clone,
                    bot_id,
                    app_state_clone,
                    messages,
                )
                .await
                {
                    warn!("[Telegram] 媒体组批处理失败: {err}");
                }
            });
        }
        return Ok(());
    }

    process_telegram_message_batch(bot, core, bot_username, bot_id, app_state, vec![msg]).await
}

async fn process_telegram_message_batch(
    bot: Bot,
    core: Arc<hone_channels::HoneBotCore>,
    bot_username: Arc<String>,
    bot_id: u64,
    app_state: Arc<TelegramAppState>,
    mut messages: Vec<Message>,
) -> ResponseResult<()> {
    if messages.is_empty() {
        return Ok(());
    }

    messages.sort_by_key(|message| message.id.0);
    let Some(first_msg) = messages.first().cloned() else {
        return Ok(());
    };
    let Some(user) = first_msg.from.clone() else {
        return Ok(());
    };
    if user.is_bot {
        return Ok(());
    }

    let author_id = user.id.0.to_string();
    let is_private = matches!(first_msg.chat.kind, ChatKind::Private(_));
    let direct_mention = !is_private
        && messages
            .iter()
            .any(|message| is_direct_mention_message(message, &bot_username, bot_id));
    let reply_to_bot = messages
        .iter()
        .any(|message| is_reply_to_bot(message, bot_id));
    let raw_text = collect_message_text(&messages);
    let media_group_id = first_msg.media_group_id().map(str::to_string);
    let text = raw_text.trim();

    let (actor, target, chat_mode) = if is_private {
        app_state
            .scope_resolver
            .direct(&author_id, format!("dm:{}", first_msg.chat.id.0))
            .expect("telegram direct actor should be valid")
    } else {
        app_state
            .scope_resolver
            .group(
                &author_id,
                format!("chat:{}", first_msg.chat.id.0),
                format!("chat:{}", first_msg.chat.id.0),
            )
            .expect("telegram group actor should be valid")
    };
    let session_identity = SessionIdentity::from_actor(&actor)
        .expect("telegram actor should always map to a session identity");
    let session_id = session_identity.session_id();
    let speaker_label = telegram_speaker_label(&user);
    let is_triggered = is_private || direct_mention || reply_to_bot;

    if !is_triggered {
        if !text.is_empty() && core.config.group_context.pretrigger_window_enabled {
            app_state
                .pretrigger
                .push(
                    &session_id,
                    BufferedGroupMessage::new(
                        "telegram",
                        first_msg.id.0.to_string(),
                        speaker_label,
                        text.to_string(),
                    ),
                )
                .await;
            info!(
                chat_id = first_msg.chat.id.0,
                message_id = first_msg.id.0,
                session_id = session_id,
                media_group_id = media_group_id.as_deref().unwrap_or(""),
                "telegram group message buffered for pretrigger window"
            );
        } else {
            info!(
                chat_id = first_msg.chat.id.0,
                message_id = first_msg.id.0,
                text = raw_text.as_str(),
                media_group_id = media_group_id.as_deref().unwrap_or(""),
                bot_username = bot_username.as_str(),
                "telegram inbound ignored: group message without explicit trigger"
            );
        }
        return Ok(());
    }

    let _active_guard = if !is_private {
        match app_state.session_locks.try_begin_active(
            &session_id,
            ActiveSessionInfo {
                speaker_label: speaker_label.clone(),
                message_id: Some(first_msg.id.0.to_string()),
            },
        ) {
            Ok(guard) => Some(guard),
            Err(active) => {
                if !text.is_empty() && core.config.group_context.pretrigger_window_enabled {
                    app_state
                        .pretrigger
                        .push(
                            &session_id,
                            BufferedGroupMessage::new(
                                "telegram",
                                first_msg.id.0.to_string(),
                                speaker_label.clone(),
                                text.to_string(),
                            ),
                        )
                        .await;
                }
                let busy_text = sanitize_telegram_html_public(&prepend_reply_prefix(
                    Some(&user_reply_prefix(&user)),
                    &build_group_busy_text(&active.speaker_label),
                ));
                let _ = send_message_with_fallback(&bot, first_msg.chat.id, &busy_text, None).await;
                info!(
                    chat_id = first_msg.chat.id.0,
                    message_id = first_msg.id.0,
                    session_id = session_id,
                    active_speaker = active.speaker_label,
                    "telegram inbound busy: group trigger deferred to pretrigger window"
                );
                return Ok(());
            }
        }
    } else {
        None
    };
    if core
        .session_storage
        .load_session(&session_id)
        .map_err(|err| {
            error!("[Telegram] 加载 session 失败 session_id={session_id}: {err}");
            err
        })
        .ok()
        .flatten()
        .is_none()
    {
        let _ = core
            .session_storage
            .create_session_for_identity(&session_identity, Some(&actor));
    }
    let buffered_messages = if !is_private && core.config.group_context.pretrigger_window_enabled {
        app_state
            .pretrigger
            .take_recent(&session_id, Some(&first_msg.id.0.to_string()))
            .await
    } else {
        Vec::new()
    };
    let buffered_count = match persist_buffered_group_messages(
        &core.session_storage,
        &session_id,
        &buffered_messages,
    ) {
        Ok(count) => count,
        Err(err) => {
            error!("[Telegram] 预触发窗口写入 session 失败 session_id={session_id}: {err}");
            0
        }
    };

    let raw_attachments = collect_raw_attachments(&bot, &messages).await;
    let attachments = ingest_raw_attachments(
        core.as_ref(),
        AttachmentIngestRequest {
            channel: "telegram".to_string(),
            actor: actor.clone(),
            session_id: session_id.clone(),
            attachments: raw_attachments,
        },
    )
    .await;
    if text.is_empty() && attachments.is_empty() && buffered_count == 0 {
        info!(
            chat_id = first_msg.chat.id.0,
            message_id = first_msg.id.0,
            media_group_id = media_group_id.as_deref().unwrap_or(""),
            "telegram inbound ignored: empty trigger without buffered context"
        );
        return Ok(());
    }
    if core.try_intercept_admin_registration(&actor, text) {
        bot.send_message(
            first_msg.chat.id,
            hone_channels::core::REGISTER_ADMIN_INTERCEPT_ACK,
        )
        .await?;
        return Ok(());
    }
    if !attachments.is_empty() {
        spawn_attachment_persist_pipeline(
            core.clone(),
            AttachmentPersistRequest {
                channel: "telegram".to_string(),
                user_id: author_id.clone(),
                session_id: session_id.clone(),
                attachments: attachments.clone(),
            },
        );
    }
    let normalized = if text.is_empty() && attachments.is_empty() {
        "@bot".to_string()
    } else {
        build_user_input(text, &attachments)
    };
    let input_text = if is_private {
        normalized
    } else {
        build_group_user_input_with_speaker(&speaker_label, &normalized)
    };
    let envelope = IncomingEnvelope {
        message_id: Some(first_msg.id.0.to_string()),
        actor,
        session_identity,
        session_id: session_id.clone(),
        channel_target: target,
        chat_mode,
        text: input_text,
        attachments: attachments.clone(),
        trigger: GroupTrigger {
            direct_mention,
            reply_to_bot,
            question_signal: false,
        },
        recv_extra: None,
        session_metadata: None,
        message_metadata: MessageMetadata::default(),
    };

    info!(
        chat_id = first_msg.chat.id.0,
        message_id = first_msg.id.0,
        session_id = session_id,
        chat_mode = ?envelope.chat_mode,
        attachments = attachments.len(),
        buffered_messages = buffered_count,
        media_group_id = media_group_id.as_deref().unwrap_or(""),
        "telegram inbound accepted"
    );

    let is_admin = core.is_admin_actor(&envelope.actor);
    let mut prompt_options = PromptOptions {
        is_admin,
        ..PromptOptions::default()
    };
    if envelope.is_group() {
        prompt_options.privacy_guard = Some(TELEGRAM_GROUP_PRIVACY_GUARD.to_string());
    }

    let mut session = AgentSession::new(
        core.clone(),
        envelope.actor.clone(),
        envelope.channel_target.clone(),
    )
    .with_session_identity(envelope.session_identity.clone())
    .with_message_id(envelope.message_id.clone())
    .with_prompt_options(prompt_options)
    .with_message_metadata(envelope.message_metadata.clone())
    .with_cron_allowed(envelope.cron_allowed());

    let reply_prefix = if envelope.is_group() {
        Some(user_reply_prefix(&user))
    } else {
        None
    };
    let placeholder_body = if attachments.is_empty() {
        THINKING_PLACEHOLDER_TEXT.to_string()
    } else {
        build_attachment_ack_message(&attachments)
    };
    let placeholder_text = sanitize_telegram_html_public(&prepend_reply_prefix(
        reply_prefix.as_deref(),
        &placeholder_body,
    ));
    let summary = run_session_with_outbound(
        &mut session,
        TelegramOutboundAdapter {
            bot: bot.clone(),
            chat_id: first_msg.chat.id,
            max_len: core.config.telegram.max_message_length,
            reply_prefix,
            show_reasoning: !envelope.is_group(),
        },
        &envelope.text,
        &placeholder_text,
        AgentRunOptions::default(),
    )
    .await;

    if summary.placeholder_sent {
        core.log_message_step(
            "telegram",
            &envelope.actor.user_id,
            &session_id,
            "reply.placeholder",
            "sent",
            None,
            None,
        );
    } else {
        core.log_message_step(
            "telegram",
            &envelope.actor.user_id,
            &session_id,
            "reply.placeholder",
            "failed",
            None,
            None,
        );
    }
    if summary.result.response.success {
        core.log_message_step(
            "telegram",
            &envelope.actor.user_id,
            &session_id,
            "reply.send",
            &format!("segments.sent={}", summary.sent_segments),
            None,
            None,
        );
    }

    Ok(())
}

async fn handle_scheduler_events(
    bot: Bot,
    core: Arc<hone_channels::HoneBotCore>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: telegram）");
    while let Some(event) = event_rx.recv().await {
        if event.channel != "telegram" {
            continue;
        }

        let bot_clone = bot.clone();
        let core_clone = core.clone();
        tokio::spawn(async move {
            let storage = core_clone.cron_job_storage();
            let result = run_scheduled_task(&core_clone, &event).await;
            if !result.should_deliver {
                info!(
                    "[Telegram] 心跳任务未命中，本轮不发送: job={} target={}",
                    event.job_name, event.channel_target
                );
                let _ = storage.record_execution_event(
                    &event.actor,
                    &event.job_id,
                    &event.job_name,
                    &event.channel_target,
                    event.heartbeat,
                    CronJobExecutionInput {
                        execution_status: if result.error.is_some() {
                            "execution_failed".to_string()
                        } else {
                            "noop".to_string()
                        },
                        message_send_status: if result.error.is_some() {
                            "skipped_error".to_string()
                        } else {
                            "skipped_noop".to_string()
                        },
                        should_deliver: false,
                        delivered: false,
                        response_preview: None,
                        error_message: result.error.clone(),
                        detail: result.metadata.clone(),
                    },
                );
                return;
            }
            let response = result
                .error
                .clone()
                .unwrap_or_else(|| result.content.clone());
            let chat_id: i64 = match event.channel_target.parse() {
                Ok(id) => id,
                Err(_) => {
                    error!(
                        "[Telegram] 定时任务目标解析失败: job={} target={} ",
                        event.job_name, event.channel_target
                    );
                    let _ = storage.record_execution_event(
                        &event.actor,
                        &event.job_id,
                        &event.job_name,
                        &event.channel_target,
                        event.heartbeat,
                        CronJobExecutionInput {
                            execution_status: if result.error.is_some() {
                                "execution_failed".to_string()
                            } else {
                                "completed".to_string()
                            },
                            message_send_status: "target_resolution_failed".to_string(),
                            should_deliver: true,
                            delivered: false,
                            response_preview: Some(response.clone()),
                            error_message: Some("Telegram 定时任务目标解析失败".to_string()),
                            detail: result.metadata.clone(),
                        },
                    );
                    return;
                }
            };
            let segments = split_segments(
                &response,
                core_clone.config.telegram.max_message_length,
                3500,
            );
            let total_segments = segments.len();
            let sent = send_segments(&bot_clone, ChatId(chat_id), segments, None).await;
            let _ = storage.record_execution_event(
                &event.actor,
                &event.job_id,
                &event.job_name,
                &event.channel_target,
                event.heartbeat,
                CronJobExecutionInput {
                    execution_status: if result.error.is_some() {
                        "execution_failed".to_string()
                    } else {
                        "completed".to_string()
                    },
                    message_send_status: if sent > 0 {
                        "sent".to_string()
                    } else {
                        "send_failed".to_string()
                    },
                    should_deliver: true,
                    delivered: sent > 0,
                    response_preview: Some(response),
                    error_message: result.error.clone(),
                    detail: json!({
                        "sent_segments": sent,
                        "total_segments": total_segments,
                        "scheduler": result.metadata,
                    }),
                },
            );
        });
    }
}

async fn run_scheduled_task(
    core: &Arc<hone_channels::HoneBotCore>,
    event: &SchedulerEvent,
) -> scheduler::ScheduledTaskExecution {
    let prompt_options = PromptOptions::default();
    scheduler::execute_scheduler_event(
        core.clone(),
        event,
        prompt_options,
        AgentRunOptions::default(),
    )
    .await
}

fn is_allowed_author(author_id: &str, allow_from: &[String]) -> bool {
    allow_from.is_empty() || allow_from.iter().any(|v| v == "*" || v == author_id)
}

fn is_reply_to_bot(msg: &Message, bot_id: u64) -> bool {
    msg.reply_to_message()
        .and_then(|reply| reply.from.clone())
        .map(|from| from.id.0 == bot_id)
        .unwrap_or(false)
}

fn is_direct_mention_message(msg: &Message, bot_username: &str, bot_id: u64) -> bool {
    if !bot_username.is_empty() {
        let mention_token = format!("@{}", bot_username);
        if message_text(msg).contains(&mention_token) {
            return true;
        }
    }

    is_reply_to_bot(msg, bot_id)
}

fn user_reply_prefix(user: &User) -> String {
    user.mention()
        .unwrap_or_else(|| html::user_mention(user.id, &user.full_name()))
}

fn message_has_supported_attachments(msg: &Message) -> bool {
    msg.photo().is_some()
        || msg.document().is_some()
        || msg.video().is_some()
        || msg.audio().is_some()
        || msg.voice().is_some()
        || msg.animation().is_some()
}

fn message_text(msg: &Message) -> String {
    msg.text()
        .or_else(|| msg.caption())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_string()
}

fn collect_message_text(messages: &[Message]) -> String {
    let mut parts = Vec::new();
    for message in messages {
        let text = message_text(message);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join("\n\n")
}

async fn collect_raw_attachments(bot: &Bot, messages: &[Message]) -> Vec<RawAttachment> {
    let mut out = Vec::new();

    for msg in messages {
        if let Some(photo_sizes) = msg.photo() {
            if let Some(photo) = photo_sizes.last() {
                out.push(
                    download_telegram_attachment(
                        bot,
                        photo.file.id.clone(),
                        format!("telegram_photo_{}_{}.jpg", msg.chat.id.0, msg.id.0),
                        Some("image/jpeg".to_string()),
                        Some(photo.file.size),
                        format!("telegram://file/{}", photo.file.id),
                    )
                    .await,
                );
            }
        }

        if let Some(document) = msg.document() {
            let filename = document
                .file_name
                .clone()
                .unwrap_or_else(|| format!("telegram_document_{}_{}.bin", msg.chat.id.0, msg.id.0));
            out.push(
                download_telegram_attachment(
                    bot,
                    document.file.id.clone(),
                    filename,
                    document.mime_type.as_ref().map(ToString::to_string),
                    Some(document.file.size),
                    format!("telegram://file/{}", document.file.id),
                )
                .await,
            );
        }

        if let Some(video) = msg.video() {
            let filename = video
                .file_name
                .clone()
                .unwrap_or_else(|| format!("telegram_video_{}_{}.mp4", msg.chat.id.0, msg.id.0));
            out.push(
                download_telegram_attachment(
                    bot,
                    video.file.id.clone(),
                    filename,
                    video.mime_type.as_ref().map(ToString::to_string),
                    Some(video.file.size),
                    format!("telegram://file/{}", video.file.id),
                )
                .await,
            );
        }

        if let Some(audio) = msg.audio() {
            let filename = audio
                .file_name
                .clone()
                .unwrap_or_else(|| format!("telegram_audio_{}_{}.mp3", msg.chat.id.0, msg.id.0));
            out.push(
                download_telegram_attachment(
                    bot,
                    audio.file.id.clone(),
                    filename,
                    audio.mime_type.as_ref().map(ToString::to_string),
                    Some(audio.file.size),
                    format!("telegram://file/{}", audio.file.id),
                )
                .await,
            );
        }

        if let Some(voice) = msg.voice() {
            out.push(
                download_telegram_attachment(
                    bot,
                    voice.file.id.clone(),
                    format!("telegram_voice_{}_{}.ogg", msg.chat.id.0, msg.id.0),
                    voice.mime_type.as_ref().map(ToString::to_string),
                    Some(voice.file.size),
                    format!("telegram://file/{}", voice.file.id),
                )
                .await,
            );
        }

        if let Some(animation) = msg.animation() {
            let filename = animation.file_name.clone().unwrap_or_else(|| {
                format!("telegram_animation_{}_{}.mp4", msg.chat.id.0, msg.id.0)
            });
            out.push(
                download_telegram_attachment(
                    bot,
                    animation.file.id.clone(),
                    filename,
                    animation.mime_type.as_ref().map(ToString::to_string),
                    Some(animation.file.size),
                    format!("telegram://file/{}", animation.file.id),
                )
                .await,
            );
        }
    }

    out
}

async fn download_telegram_attachment(
    bot: &Bot,
    file_id: String,
    filename: String,
    content_type: Option<String>,
    size: Option<u32>,
    url: String,
) -> RawAttachment {
    let temp_dir = channel_download_dir("telegram");
    if let Err(err) = tokio::fs::create_dir_all(&temp_dir).await {
        return RawAttachment {
            filename,
            content_type,
            size,
            url,
            local_path: None,
            data: None,
            error: Some(format!("创建临时目录失败: {err}")),
        };
    }

    let telegram_file = match bot.get_file(file_id).await {
        Ok(file) => file,
        Err(err) => {
            return RawAttachment {
                filename,
                content_type,
                size,
                url,
                local_path: None,
                data: None,
                error: Some(format!("获取 Telegram 文件信息失败: {err}")),
            };
        }
    };

    let safe_filename: String = filename
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' => '_',
            other => other,
        })
        .collect();
    let temp_name = format!(
        "{}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default(),
        safe_filename
    );
    let temp_path = temp_dir.join(temp_name);
    let mut out: tokio::fs::File = match tokio::fs::File::create(&temp_path).await {
        Ok(file) => file,
        Err(err) => {
            return RawAttachment {
                filename,
                content_type,
                size,
                url,
                local_path: None,
                data: None,
                error: Some(format!("创建临时文件失败: {err}")),
            };
        }
    };

    if let Err(err) = bot.download_file(&telegram_file.path, &mut out).await {
        return RawAttachment {
            filename,
            content_type,
            size,
            url,
            local_path: None,
            data: None,
            error: Some(format!("下载 Telegram 文件失败: {err}")),
        };
    }
    let _ = out.flush().await;

    RawAttachment {
        filename,
        content_type,
        size,
        url,
        local_path: Some(temp_path),
        data: None,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn build_private_message(user_id: u64, text: &str) -> Message {
        serde_json::from_value(json!({
            "message_id": 990001,
            "date": 1774177200,
            "chat": {
                "id": user_id as i64,
                "type": "private",
                "first_name": "Codex",
                "username": "codex_local_test"
            },
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "Codex",
                "username": "codex_local_test",
                "language_code": "zh-hans"
            },
            "text": text
        }))
        .expect("telegram message json should deserialize")
    }

    #[tokio::test]
    #[ignore = "manual telegram callback smoke test against the live bot token"]
    async fn manual_private_message_callback_smoke() {
        let (config, config_path) =
            hone_channels::load_runtime_config().expect("runtime config should load");
        assert!(
            config.telegram.enabled,
            "telegram must be enabled in runtime config"
        );
        let target_user_id = 8039067465u64;
        let bot = Bot::new(config.telegram.bot_token.trim().to_string());
        let me = bot.get_me().await.expect("telegram getMe should succeed");
        let bot_id = me.user.id.0;
        let bot_username = Arc::new(me.user.username.unwrap_or_default());
        let core = Arc::new(hone_channels::HoneBotCore::new(config));
        let app_state = Arc::new(TelegramAppState {
            dedup: MessageDeduplicator::new(Duration::from_secs(120), 2048),
            session_locks: SessionLockRegistry::new(),
            scope_resolver: ActorScopeResolver::new("telegram"),
            pretrigger: hone_channels::ingress::GroupPretriggerWindowRegistry::new(
                core.config.group_context.pretrigger_window_max_messages,
                Duration::from_secs(core.config.group_context.pretrigger_window_max_age_seconds),
            ),
            media_groups: MediaGroupBuffer::new(),
        });
        let msg = build_private_message(
            target_user_id,
            "Reply with exactly: HONE_TELEGRAM_CALLBACK_OK",
        );

        handle_message(bot, msg, core, bot_username, bot_id, app_state)
            .await
            .expect("telegram callback path should finish");

        eprintln!(
            "manual telegram callback smoke finished using config {}",
            config_path
        );
    }
}
