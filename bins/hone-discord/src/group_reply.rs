use std::collections::HashMap;
use std::sync::Arc;

use hone_channels::agent_session::{AgentRunOptions, AgentSession};
use hone_channels::prompt::{DEFAULT_GROUP_PRIVACY_GUARD, PromptOptions};
use hone_core::SessionIdentity;
use serenity::all::ChannelId;
use serenity::http::Http;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, Instant};
use tracing::warn;

use crate::types::{ChannelKey, GroupQueuedMessage, GroupReplyRuntimeConfig, ReplyMentionMode};
use crate::utils::{
    DiscordProgressTranscript, DiscordReasoningListener, discord_actor, prepend_reply_prefix,
    send_or_edit_segments, send_placeholder_message, should_trigger_by_mode, split_into_segments,
    truncate_chars, update_or_send_plain_text,
};

const GROUP_REPLY_IDLE_SECONDS: u64 = 15 * 60;
const THINKING_PLACEHOLDER_TEXT: &str = "正在思考中...";

#[derive(Clone)]
pub(crate) struct GroupReplyCoordinator {
    core: Arc<hone_channels::HoneBotCore>,
    cfg: GroupReplyRuntimeConfig,
    workers: Arc<Mutex<HashMap<ChannelKey, mpsc::Sender<GroupQueuedMessage>>>>,
}

impl GroupReplyCoordinator {
    pub(crate) fn new(core: Arc<hone_channels::HoneBotCore>) -> Self {
        let cfg = GroupReplyRuntimeConfig::from_config(&core.config);
        Self {
            core,
            cfg,
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.cfg.enabled
    }

    pub(crate) fn cfg(&self) -> &GroupReplyRuntimeConfig {
        &self.cfg
    }

    #[allow(dead_code)]
    pub(crate) fn should_trigger(&self, direct_mention: bool, question_signal: bool) -> bool {
        should_trigger_by_mode(self.cfg.trigger_mode, direct_mention, question_signal)
    }

    pub(crate) async fn enqueue(&self, msg: GroupQueuedMessage, http: Arc<Http>) {
        let channel_key = msg.channel_key;
        let mut pending = msg;
        for _ in 0..2 {
            let tx = self.ensure_worker(channel_key, http.clone()).await;
            match tx.send(pending).await {
                Ok(()) => return,
                Err(err) => {
                    pending = err.0;
                    let mut workers = self.workers.lock().await;
                    workers.remove(&channel_key);
                    warn!("[Discord/GroupReply] worker channel closed, recreating...");
                }
            }
        }
        warn!(
            "[Discord/GroupReply] enqueue failed after retry: guild={} channel={}",
            channel_key.guild_id, channel_key.channel_id
        );
    }

    async fn ensure_worker(
        &self,
        channel_key: ChannelKey,
        http: Arc<Http>,
    ) -> mpsc::Sender<GroupQueuedMessage> {
        {
            let workers = self.workers.lock().await;
            if let Some(tx) = workers.get(&channel_key) {
                return tx.clone();
            }
        }

        let (tx, rx) = mpsc::channel(self.cfg.queue_capacity_per_channel);
        {
            let mut workers = self.workers.lock().await;
            if let Some(existing) = workers.get(&channel_key) {
                return existing.clone();
            }
            workers.insert(channel_key, tx.clone());
        }

        let core = self.core.clone();
        let cfg = self.cfg.clone();
        let workers = self.workers.clone();
        tokio::spawn(async move {
            run_group_channel_worker(core, cfg, channel_key, http, rx).await;
            let mut map = workers.lock().await;
            map.remove(&channel_key);
        });

        tx
    }
}

async fn run_group_channel_worker(
    core: Arc<hone_channels::HoneBotCore>,
    cfg: GroupReplyRuntimeConfig,
    channel_key: ChannelKey,
    http: Arc<Http>,
    mut rx: mpsc::Receiver<GroupQueuedMessage>,
) {
    loop {
        let first =
            match tokio::time::timeout(Duration::from_secs(GROUP_REPLY_IDLE_SECONDS), rx.recv())
                .await
            {
                Ok(Some(v)) => v,
                Ok(None) => break,
                Err(_) => break,
            };

        let batch = collect_group_batch(first, &mut rx, &cfg).await;
        let (batch, backlog_summary) = apply_backlog_policy(batch, &cfg);
        if batch.is_empty() {
            continue;
        }

        if let Err(e) = process_group_batch(
            core.clone(),
            &http,
            channel_key,
            &cfg,
            batch,
            backlog_summary,
        )
        .await
        {
            warn!(
                "[Discord/GroupReply] batch process failed guild={} channel={}: {}",
                channel_key.guild_id, channel_key.channel_id, e
            );
        }
    }
}

pub(crate) async fn collect_group_batch(
    first: GroupQueuedMessage,
    rx: &mut mpsc::Receiver<GroupQueuedMessage>,
    cfg: &GroupReplyRuntimeConfig,
) -> Vec<GroupQueuedMessage> {
    let mut batch = vec![first];
    let mut deadline = Instant::now() + Duration::from_secs(cfg.window_seconds);
    if batch[0].direct_mention {
        deadline =
            tighten_deadline_for_mention(deadline, Instant::now(), cfg.mention_fast_delay_seconds);
    }

    loop {
        let sleep = tokio::time::sleep_until(deadline);
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => break,
            maybe = rx.recv() => {
                let Some(msg) = maybe else {
                    break;
                };
                if msg.direct_mention {
                    deadline = tighten_deadline_for_mention(
                        deadline,
                        Instant::now(),
                        cfg.mention_fast_delay_seconds,
                    );
                }
                batch.push(msg);
            }
        }
    }

    batch
}

pub(crate) fn tighten_deadline_for_mention(
    current_deadline: Instant,
    now: Instant,
    mention_fast_delay_seconds: u64,
) -> Instant {
    let fast_deadline = now + Duration::from_secs(mention_fast_delay_seconds);
    std::cmp::min(current_deadline, fast_deadline)
}

pub(crate) fn apply_backlog_policy(
    messages: Vec<GroupQueuedMessage>,
    cfg: &GroupReplyRuntimeConfig,
) -> (Vec<GroupQueuedMessage>, Option<String>) {
    if messages.len() <= cfg.max_batch_messages {
        return (messages, None);
    }

    let keep_n = cfg.backlog_keep_latest.min(messages.len()).max(1);
    let split_index = messages.len().saturating_sub(keep_n);
    let older = &messages[..split_index];
    let kept = messages[split_index..].to_vec();
    let summary = summarize_older_messages(older, cfg.backlog_summary_max_chars);
    (kept, Some(summary))
}

fn summarize_older_messages(messages: &[GroupQueuedMessage], max_chars: usize) -> String {
    let mut summary = String::new();
    for msg in messages {
        let normalized = msg
            .user_input
            .replace('\n', " ")
            .trim()
            .chars()
            .take(120)
            .collect::<String>();
        let line = format!("{}: {}; ", msg.author_name, normalized);
        if summary.chars().count() + line.chars().count() > max_chars {
            break;
        }
        summary.push_str(&line);
    }
    if summary.is_empty() {
        "（略）".to_string()
    } else {
        summary
    }
}

async fn process_group_batch(
    core: Arc<hone_channels::HoneBotCore>,
    http: &Arc<Http>,
    channel_key: ChannelKey,
    cfg: &GroupReplyRuntimeConfig,
    messages: Vec<GroupQueuedMessage>,
    backlog_summary: Option<String>,
) -> hone_core::HoneResult<()> {
    let Some(last_msg) = messages.last() else {
        return Ok(());
    };
    let author_id = last_msg.author_id.clone();
    let actor = discord_actor(&author_id, Some(channel_key));
    let session_identity = SessionIdentity::group("discord", channel_key.scope())
        .expect("discord channel key should always map to group session");
    let session_id = session_identity.session_id();

    let input = build_group_batch_input(&messages, backlog_summary.as_deref(), cfg);
    let recv_extra = format!(
        "group_batch=true size={} direct_mentions={}",
        messages.len(),
        messages.iter().filter(|m| m.direct_mention).count()
    );
    let target = format!(
        "guild:{}:channel:{}",
        channel_key.guild_id, channel_key.channel_id
    );
    let is_admin = core.is_admin_actor(&actor);
    let mut prompt_options = PromptOptions {
        is_admin,
        ..PromptOptions::default()
    };
    prompt_options.privacy_guard = Some(DEFAULT_GROUP_PRIVACY_GUARD.to_string());
    let mut session = AgentSession::new(core.clone(), actor.clone(), target.clone())
        .with_session_identity(session_identity)
        .with_prompt_options(prompt_options)
        .with_recv_extra(Some(recv_extra))
        .with_cron_allowed(false);

    let mention_target = messages
        .iter()
        .rev()
        .find(|m| m.direct_mention)
        .map(|m| m.author_mention.clone())
        .or_else(|| messages.last().map(|m| m.author_mention.clone()));
    let placeholder_text = if let Some(mention) = &mention_target {
        format!("{mention} {THINKING_PLACEHOLDER_TEXT}")
    } else {
        THINKING_PLACEHOLDER_TEXT.to_string()
    };
    let placeholder_message = send_placeholder_message(
        http,
        ChannelId::new(channel_key.channel_id),
        &placeholder_text,
        core.config.discord.max_message_length,
    )
    .await;
    if placeholder_message.is_some() {
        core.log_message_step(
            "discord",
            &author_id,
            &session_id,
            "group.reply.placeholder",
            "sent",
            None,
            None,
        );
    } else {
        core.log_message_step(
            "discord",
            &author_id,
            &session_id,
            "group.reply.placeholder",
            "failed",
            None,
            None,
        );
    }

    let placeholder = Arc::new(Mutex::new(placeholder_message));
    session.add_listener(Arc::new(DiscordReasoningListener {
        http: http.clone(),
        channel_id: ChannelId::new(channel_key.channel_id),
        placeholder: placeholder.clone(),
        progress: Arc::new(Mutex::new(DiscordProgressTranscript::new(
            &placeholder_text,
        ))),
        max_len: core.config.discord.max_message_length,
        show_reasoning: true,
    }));

    let response = session
        .run(&input, AgentRunOptions::default())
        .await
        .response;
    if response.success {
        let mut content = response.content.trim().to_string();
        if content.is_empty() {
            content = "收到。".to_string();
        }

        content = prepend_reply_prefix(mention_target.as_deref(), &content);
        let segments = split_into_segments(&content, core.config.discord.max_message_length);
        let mut placeholder_message = placeholder.lock().await;
        let (sent_segments, total_segments) = send_or_edit_segments(
            http,
            ChannelId::new(channel_key.channel_id),
            placeholder_message.as_mut(),
            segments,
        )
        .await;
        core.log_message_step(
            "discord",
            &author_id,
            &session_id,
            "group.reply.send",
            &format!("segments.sent={sent_segments}/{total_segments}"),
            None,
            None,
        );
    } else {
        let err = response.error.unwrap_or_else(|| "未知错误".to_string());
        let tip = prepend_reply_prefix(
            mention_target.as_deref(),
            &format!("抱歉，处理失败：{}", truncate_chars(&err, 300)),
        );
        let mut placeholder_message = placeholder.lock().await;
        update_or_send_plain_text(
            http,
            ChannelId::new(channel_key.channel_id),
            placeholder_message.as_mut(),
            &tip,
            core.config.discord.max_message_length,
        )
        .await;
    }
    Ok(())
}

fn build_group_batch_input(
    messages: &[GroupQueuedMessage],
    backlog_summary: Option<&str>,
    cfg: &GroupReplyRuntimeConfig,
) -> String {
    let mut lines = vec![
        "你在 Discord 群聊中发言。以下是短窗口内聚合的用户消息，请只回复一条自然、简洁的群聊消息。"
            .to_string(),
        "要求：像真人一样说话，不要机械逐条复读；优先回答被@或最明确的问题。".to_string(),
        "系统会自动在最终回复前补上对主要提问者的 @，正文里不要重复再 @ 一次。".to_string(),
    ];

    match cfg.reply_mention_mode {
        ReplyMentionMode::Adaptive => {
            lines.push("如需点名澄清，请直接用名字称呼，不要再额外输出平台 @ 语法。".to_string())
        }
        ReplyMentionMode::Always => {
            lines.push("主要提问者会由系统自动 @，你只需要给出正文。".to_string())
        }
        ReplyMentionMode::Never => lines.push("请不要使用 @ 提及用户。".to_string()),
    }

    if messages.iter().any(|m| m.direct_mention) {
        lines.push("本轮包含直接@消息，请优先回应该意图。".to_string());
    }
    if let Some(summary) = backlog_summary {
        lines.push(format!("窗口前序堆积摘要：{}", summary));
    }

    lines.push("窗口内消息（按时间顺序）：".to_string());
    for (idx, msg) in messages.iter().enumerate() {
        let normalized = msg.user_input.trim();
        lines.push(format!(
            "{}. {} {}{}: {}",
            idx + 1,
            msg.author_name,
            msg.author_mention,
            if msg.question_signal {
                " [question]"
            } else {
                ""
            },
            normalized
        ));
    }

    lines.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_group_message(
        content: &str,
        direct_mention: bool,
        question_signal: bool,
    ) -> GroupQueuedMessage {
        GroupQueuedMessage {
            channel_key: ChannelKey {
                guild_id: 1,
                channel_id: 2,
            },
            author_id: "u1".to_string(),
            author_name: "alice".to_string(),
            author_mention: "<@u1>".to_string(),
            direct_mention,
            question_signal,
            user_input: content.to_string(),
        }
    }

    fn sample_group_cfg() -> GroupReplyRuntimeConfig {
        GroupReplyRuntimeConfig {
            enabled: true,
            trigger_mode: crate::types::GroupTriggerMode::MentionOrQuestion,
            window_seconds: 45,
            mention_fast_delay_seconds: 3,
            queue_capacity_per_channel: 200,
            max_batch_messages: 3,
            backlog_keep_latest: 2,
            backlog_summary_max_chars: 120,
            reply_mention_mode: ReplyMentionMode::Adaptive,
            question_signal_enabled: true,
        }
    }

    #[test]
    fn trigger_mode_matrix_works() {
        assert!(!should_trigger_by_mode(
            crate::types::GroupTriggerMode::MentionOnly,
            false,
            true
        ));
        assert!(should_trigger_by_mode(
            crate::types::GroupTriggerMode::MentionOnly,
            true,
            false
        ));
        assert!(should_trigger_by_mode(
            crate::types::GroupTriggerMode::MentionOrQuestion,
            false,
            true
        ));
        assert!(should_trigger_by_mode(
            crate::types::GroupTriggerMode::All,
            false,
            false
        ));
    }

    #[test]
    fn mention_fast_path_tightens_deadline() {
        let now = Instant::now();
        let normal = now + Duration::from_secs(45);
        let tightened = tighten_deadline_for_mention(normal, now, 3);
        assert!(tightened <= now + Duration::from_secs(3));
        assert!(tightened < normal);
    }

    #[test]
    fn backlog_policy_keeps_latest_and_summarizes_old() {
        let cfg = sample_group_cfg();
        let messages = vec![
            sample_group_message("m1", false, false),
            sample_group_message("m2", false, true),
            sample_group_message("m3", false, false),
            sample_group_message("m4", true, false),
        ];
        let (kept, summary) = apply_backlog_policy(messages, &cfg);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].user_input, "m3");
        assert_eq!(kept[1].user_input, "m4");
        assert!(summary.unwrap_or_default().contains("alice"));
    }

    #[tokio::test]
    async fn collect_group_batch_collects_messages_within_window() {
        let mut cfg = sample_group_cfg();
        cfg.window_seconds = 1;
        cfg.mention_fast_delay_seconds = 1;
        let (tx, mut rx) = mpsc::channel(16);

        let first = sample_group_message("m1", false, true);
        tx.send(sample_group_message("m2", false, false))
            .await
            .expect("send m2");
        let tx_bg = tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            let _ = tx_bg.send(sample_group_message("m3", true, false)).await;
        });

        let batch = collect_group_batch(first, &mut rx, &cfg).await;
        assert_eq!(batch.len(), 3);
        assert!(batch.iter().any(|m| m.direct_mention));
    }
}
