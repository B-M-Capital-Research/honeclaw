use std::sync::Arc;

use hone_channels::agent_session::{AgentRunOptions, AgentSession};
use hone_channels::attachments::{
    AttachmentIngestRequest, AttachmentPersistRequest, build_attachment_ack_message,
    ingest_raw_attachments, spawn_attachment_persist_pipeline,
};
use hone_channels::ingress::{
    ActiveSessionInfo, ActorScopeResolver, BufferedGroupMessage, GroupTrigger, IncomingEnvelope,
    MessageDeduplicator, SessionLockRegistry, persist_buffered_group_messages,
};
use hone_channels::outbound::run_session_with_outbound;
use hone_channels::prompt::{DEFAULT_GROUP_PRIVACY_GUARD, PromptOptions};
use hone_channels::think::{ThinkRenderStyle, render_think_blocks};
use hone_core::SessionIdentity;
use hone_tools::LoadSkillTool;
use serenity::all::{
    Command, CommandInteraction, Context, CreateAutocompleteResponse, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse, EventHandler, Interaction, Message,
    Ready,
};
use tracing::{error, info};

use crate::attachments::{build_dm_user_input, build_group_user_input, collect_raw_attachments};
use crate::utils::{
    DISCORD_SKILL_COMMAND, DiscordOutboundAdapter, build_skill_command_input,
    build_skill_slash_command, configured_skill_dirs, discord_actor, is_allowed_author,
    is_direct_mention_message, slash_option_string, split_into_segments, truncate_chars,
};

const MAX_DISCORD_AUTOCOMPLETE_CHOICES: usize = 25;

fn build_group_user_input_with_speaker(label: &str, text: &str) -> String {
    format!("[{label}] {}", text.trim())
}

fn build_group_busy_text(speaker_label: &str) -> String {
    format!("正在处理 {speaker_label} 的消息，请等上一条完成后再 @ 我。")
}

pub(crate) struct DiscordHandler {
    pub(crate) core: Arc<hone_channels::HoneBotCore>,
    pub(crate) dedup: MessageDeduplicator,
    pub(crate) session_locks: SessionLockRegistry,
    pub(crate) scope_resolver: ActorScopeResolver,
    pub(crate) pretrigger: hone_channels::GroupPretriggerWindowRegistry,
}

#[serenity::async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("✅ Discord 已登录: {} ({})", ready.user.name, ready.user.id);
        info!(
            "   chat_scope={:?} max_message_length={}",
            self.core.config.discord.chat_scope, self.core.config.discord.max_message_length
        );
        info!(
            "   group_reply.enabled={} pretrigger_window.enabled={} max_messages={} max_age={}s",
            self.core.config.discord.group_reply.enabled,
            self.core.config.group_context.pretrigger_window_enabled,
            self.core
                .config
                .group_context
                .pretrigger_window_max_messages,
            self.core
                .config
                .group_context
                .pretrigger_window_max_age_seconds
        );

        match Command::set_global_commands(&ctx.http, vec![build_skill_slash_command()]).await {
            Ok(_) => info!("   slash_commands=/skill"),
            Err(err) => error!("注册 Discord slash commands 失败: {}", err),
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = self.handle_message(&ctx, &msg).await {
            error!("处理 Discord 消息失败: {e}");
            let _ = msg
                .channel_id
                .say(&ctx.http, "抱歉，处理消息时发生内部错误。")
                .await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Autocomplete(command) => {
                if let Err(err) = self.handle_skill_autocomplete(&ctx, &command).await {
                    error!("处理 Discord autocomplete 失败: {}", err);
                }
            }
            Interaction::Command(command) => {
                if let Err(err) = self.handle_slash_command(&ctx, &command).await {
                    error!("处理 Discord slash command 失败: {}", err);
                }
            }
            _ => {}
        }
    }
}

impl DiscordHandler {
    async fn handle_message(&self, ctx: &Context, msg: &Message) -> hone_core::HoneResult<()> {
        if msg.author.bot {
            self.core.log_message_step(
                "discord",
                &msg.author.id.get().to_string(),
                "-",
                "ignore",
                "author_is_bot",
                None,
                None,
            );
            return Ok(());
        }

        let dedup_key = format!("{}:{}", msg.channel_id.get(), msg.id.get());
        if self.dedup.is_duplicate(&dedup_key) {
            self.core.log_message_step(
                "discord",
                &msg.author.id.get().to_string(),
                "-",
                "ignore",
                "duplicate_message",
                None,
                None,
            );
            return Ok(());
        }

        let discord_cfg = &self.core.config.discord;
        if msg.guild_id.is_some() && !discord_cfg.chat_scope.allows_group() {
            self.core.log_message_step(
                "discord",
                &msg.author.id.get().to_string(),
                "-",
                "ignore",
                "guild_message_blocked_by_chat_scope",
                None,
                None,
            );
            return Ok(());
        }
        if msg.guild_id.is_none() && !discord_cfg.chat_scope.allows_direct() {
            self.core.log_message_step(
                "discord",
                &msg.author.id.get().to_string(),
                "-",
                "ignore",
                "dm_message_blocked_by_chat_scope",
                None,
                None,
            );
            return Ok(());
        }

        let author_id = msg.author.id.get().to_string();
        if !is_allowed_author(&author_id, &discord_cfg.allow_from) {
            info!("忽略未授权用户消息: {}", author_id);
            return Ok(());
        }

        if msg.guild_id.is_some() && self.core.config.discord.group_reply.enabled {
            return self.handle_group_message(ctx, msg, &author_id).await;
        }

        self.handle_direct_message(ctx, msg, &author_id).await
    }

    async fn handle_skill_autocomplete(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> hone_core::HoneResult<()> {
        if command.data.name != DISCORD_SKILL_COMMAND {
            return Ok(());
        }

        let query = command
            .data
            .autocomplete()
            .map(|option| option.value)
            .unwrap_or_default();
        let matches = self
            .discord_skill_loader()
            .search_skills_with_meta(query, MAX_DISCORD_AUTOCOMPLETE_CHOICES);

        let mut response = CreateAutocompleteResponse::new();
        for skill in matches {
            response = response.add_string_choice(
                truncate_chars(&format!("{} ({})", skill.display_name, skill.name), 100),
                skill.name,
            );
        }

        command
            .create_response(&ctx.http, CreateInteractionResponse::Autocomplete(response))
            .await
            .map_err(|err| hone_core::HoneError::Channel(err.to_string()))
    }

    async fn handle_slash_command(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> hone_core::HoneResult<()> {
        if command.data.name != DISCORD_SKILL_COMMAND {
            return Ok(());
        }

        let author_id = command.user.id.get().to_string();
        if !is_allowed_author(&author_id, &self.core.config.discord.allow_from) {
            return self
                .respond_to_command_once(ctx, command, "你没有权限使用这个命令。", true)
                .await;
        }
        if command.guild_id.is_some() && !self.core.config.discord.chat_scope.allows_group() {
            return self
                .respond_to_command_once(
                    ctx,
                    command,
                    "当前配置不允许在群聊中使用 Discord Bot。",
                    true,
                )
                .await;
        }
        if command.guild_id.is_none() && !self.core.config.discord.chat_scope.allows_direct() {
            return self
                .respond_to_command_once(
                    ctx,
                    command,
                    "当前配置不允许在私聊中使用 Discord Bot。",
                    true,
                )
                .await;
        }

        let skill_query = slash_option_string(command, "name").unwrap_or_default();
        let prompt = slash_option_string(command, "prompt");
        if skill_query.trim().is_empty() {
            return self
                .respond_to_command_once(ctx, command, "请先输入要触发的 skill。", true)
                .await;
        }
        let Some(skill) = self
            .discord_skill_loader()
            .search_skills_with_meta(&skill_query, 1)
            .into_iter()
            .next()
        else {
            return self
                .respond_to_command_once(
                    ctx,
                    command,
                    &format!(
                        "未找到技能 `{}`。请换个关键词，或重新输入 `/skill` 从联想列表中选择。",
                        skill_query
                    ),
                    true,
                )
                .await;
        };

        command
            .defer(&ctx.http)
            .await
            .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;

        let input = build_skill_command_input(&skill.name, prompt.as_deref());
        match self
            .run_slash_skill_agent(command, &author_id, &input, &skill.name)
            .await
        {
            Ok(content) => {
                self.send_command_response_segments(ctx, command, &content)
                    .await?;
            }
            Err(err) => {
                let tip = format!("抱歉，处理失败：{}", truncate_chars(&err.to_string(), 300));
                command
                    .edit_response(&ctx.http, EditInteractionResponse::new().content(tip))
                    .await
                    .map_err(|edit_err| hone_core::HoneError::Channel(edit_err.to_string()))?;
            }
        }

        Ok(())
    }

    async fn respond_to_command_once(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        content: &str,
        ephemeral: bool,
    ) -> hone_core::HoneResult<()> {
        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content.to_string())
                        .ephemeral(ephemeral),
                ),
            )
            .await
            .map_err(|err| hone_core::HoneError::Channel(err.to_string()))
    }

    async fn send_command_response_segments(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        content: &str,
    ) -> hone_core::HoneResult<()> {
        let mut content = content.trim().to_string();
        if content.is_empty() {
            content = "收到。".to_string();
        }

        let mut segments =
            split_into_segments(&content, self.core.config.discord.max_message_length);
        if segments.is_empty() {
            segments.push(content);
        }

        let first = segments.remove(0);
        command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(first))
            .await
            .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;

        for seg in segments {
            command
                .create_followup(
                    &ctx.http,
                    serenity::builder::CreateInteractionResponseFollowup::new().content(seg),
                )
                .await
                .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;
        }

        Ok(())
    }

    async fn run_slash_skill_agent(
        &self,
        command: &CommandInteraction,
        author_id: &str,
        input: &str,
        skill_name: &str,
    ) -> hone_core::HoneResult<String> {
        let actor = discord_actor(author_id, None);
        if let Some(reply) = self.core.try_handle_intercept_command(&actor, input).await {
            return Ok(reply);
        }

        let target = command
            .guild_id
            .map(|id| {
                format!(
                    "guild:{}:channel:{}:slash",
                    id.get(),
                    command.channel_id.get()
                )
            })
            .unwrap_or_else(|| format!("dm:{}:slash", command.channel_id.get()));
        let recv_extra = format!(
            "slash_command={} skill={}",
            DISCORD_SKILL_COMMAND, skill_name
        );
        let is_admin = self.core.is_admin_actor(&actor);
        let mut prompt_options = PromptOptions {
            is_admin,
            ..PromptOptions::default()
        };
        if command.guild_id.is_some() {
            prompt_options.privacy_guard = Some(DEFAULT_GROUP_PRIVACY_GUARD.to_string());
        }
        let allow_cron = command.guild_id.is_none();
        let session = AgentSession::new(self.core.clone(), actor.clone(), target.clone())
            .with_prompt_options(prompt_options)
            .with_recv_extra(Some(recv_extra))
            .with_cron_allowed(allow_cron);

        let response = session
            .run(input, AgentRunOptions::default())
            .await
            .response;

        if response.success {
            let content = if response.content.trim().is_empty() {
                "收到。".to_string()
            } else {
                render_think_blocks(response.content.trim(), ThinkRenderStyle::Hidden)
            };
            Ok(content)
        } else {
            let err = response.error.unwrap_or_else(|| "未知错误".to_string());
            Err(hone_core::HoneError::Channel(err))
        }
    }

    fn discord_skill_loader(&self) -> LoadSkillTool {
        LoadSkillTool::new(configured_skill_dirs(&self.core))
            .with_registry_path(self.core.configured_skill_registry_path())
    }

    async fn handle_group_message(
        &self,
        ctx: &Context,
        msg: &Message,
        author_id: &str,
    ) -> hone_core::HoneResult<()> {
        let Some(guild_id) = msg.guild_id else {
            return Ok(());
        };
        let target = format!("guild:{}:channel:{}", guild_id.get(), msg.channel_id.get());
        let (actor, channel_target, chat_mode) = self.scope_resolver.group(
            author_id,
            format!("g:{}:c:{}", guild_id.get(), msg.channel_id.get()),
            target,
        )?;
        let session_identity = SessionIdentity::from_actor(&actor)
            .expect("discord group actor should always map to a session identity");
        let session_id = session_identity.session_id();
        let bot_user_id = ctx.cache.current_user().id.get();
        let explicit_trigger = is_direct_mention_message(msg, bot_user_id);
        let group_text = msg.content.trim();
        if !explicit_trigger {
            if !group_text.is_empty() && self.core.config.group_context.pretrigger_window_enabled {
                self.pretrigger
                    .push(
                        &session_id,
                        BufferedGroupMessage::new(
                            "discord",
                            msg.id.get().to_string(),
                            msg.author.name.clone(),
                            group_text.to_string(),
                        ),
                    )
                    .await;
                self.core.log_message_step(
                    "discord",
                    author_id,
                    &session_id,
                    "group.pretrigger",
                    "buffered",
                    None,
                    None,
                );
            } else {
                self.core.log_message_step(
                    "discord",
                    author_id,
                    &session_id,
                    "ignore",
                    "group_message_not_explicitly_triggered",
                    None,
                    None,
                );
            }
            return Ok(());
        }

        if let Some(reply) = self
            .core
            .try_handle_intercept_command(&actor, msg.content.trim())
            .await
        {
            msg.channel_id
                .say(&ctx.http, reply)
                .await
                .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;
            return Ok(());
        }

        let trigger = GroupTrigger {
            direct_mention: explicit_trigger,
            reply_to_bot: false,
            question_signal: false,
        };
        let _active_guard = match self.session_locks.try_begin_active(
            &session_id,
            ActiveSessionInfo {
                speaker_label: msg.author.name.clone(),
                message_id: Some(msg.id.get().to_string()),
            },
        ) {
            Ok(guard) => guard,
            Err(active) => {
                if !group_text.is_empty()
                    && self.core.config.group_context.pretrigger_window_enabled
                {
                    self.pretrigger
                        .push(
                            &session_id,
                            BufferedGroupMessage::new(
                                "discord",
                                msg.id.get().to_string(),
                                msg.author.name.clone(),
                                group_text.to_string(),
                            ),
                        )
                        .await;
                }
                let busy_text = crate::utils::prepend_reply_prefix(
                    Some(&format!("<@{}>", msg.author.id.get())),
                    &build_group_busy_text(&active.speaker_label),
                );
                msg.channel_id
                    .say(&ctx.http, busy_text)
                    .await
                    .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;
                self.core.log_message_step(
                    "discord",
                    author_id,
                    &session_id,
                    "group.busy",
                    &format!("active_speaker={}", active.speaker_label),
                    None,
                    Some("busy"),
                );
                return Ok(());
            }
        };
        if self
            .core
            .session_storage
            .load_session(&session_id)?
            .is_none()
        {
            let _ = self
                .core
                .session_storage
                .create_session_for_identity(&session_identity, Some(&actor));
        }
        let buffered_messages = if self.core.config.group_context.pretrigger_window_enabled {
            self.pretrigger
                .take_recent(&session_id, Some(&msg.id.get().to_string()))
                .await
        } else {
            Vec::new()
        };
        let buffered_count = persist_buffered_group_messages(
            &self.core.session_storage,
            &session_id,
            &buffered_messages,
        )?;

        let raw_attachments = collect_raw_attachments(msg).await;
        let attachments = ingest_raw_attachments(
            self.core.as_ref(),
            AttachmentIngestRequest {
                channel: "discord".to_string(),
                actor: actor.clone(),
                session_id: session_id.clone(),
                attachments: raw_attachments,
            },
        )
        .await;
        if !attachments.is_empty() {
            spawn_attachment_persist_pipeline(
                self.core.clone(),
                AttachmentPersistRequest {
                    channel: "discord".to_string(),
                    actor: actor.clone(),
                    user_id: author_id.to_string(),
                    session_id: session_id.clone(),
                    attachments: attachments.clone(),
                },
            );
        }

        let mut normalized = build_group_user_input(group_text, &attachments);
        if normalized.trim().is_empty() && attachments.is_empty() && buffered_count == 0 {
            self.core.log_message_step(
                "discord",
                author_id,
                &session_id,
                "ignore",
                "empty_group_input_after_normalization",
                None,
                None,
            );
            return Ok(());
        }
        if normalized.trim().is_empty() {
            normalized = "@bot".to_string();
        }
        let input = build_group_user_input_with_speaker(&msg.author.name, &normalized);
        let recv_extra = format!(
            "attachments={} buffered_messages={} explicit_trigger=true",
            attachments.len(),
            buffered_count
        );
        let envelope = IncomingEnvelope {
            message_id: Some(msg.id.get().to_string()),
            actor,
            session_identity,
            session_id: session_id.clone(),
            channel_target,
            chat_mode,
            text: input.clone(),
            attachments,
            trigger,
            recv_extra: Some(recv_extra),
            session_metadata: None,
            message_metadata: Default::default(),
        };
        let is_admin = self.core.is_admin_actor(&envelope.actor);
        let mut prompt_options = PromptOptions {
            is_admin,
            ..PromptOptions::default()
        };
        prompt_options.privacy_guard = Some(DEFAULT_GROUP_PRIVACY_GUARD.to_string());
        let mut session = AgentSession::new(
            self.core.clone(),
            envelope.actor.clone(),
            envelope.channel_target.clone(),
        )
        .with_session_identity(envelope.session_identity.clone())
        .with_message_id(envelope.message_id.clone())
        .with_prompt_options(prompt_options)
        .with_recv_extra(envelope.recv_extra.clone())
        .with_message_metadata(envelope.message_metadata.clone())
        .with_cron_allowed(false);

        let reply_prefix = Some(format!("<@{}>", msg.author.id.get()));
        let placeholder_body = if envelope.attachments.is_empty() {
            "正在思考中...".to_string()
        } else {
            build_attachment_ack_message(&envelope.attachments)
        };
        let placeholder_text =
            crate::utils::prepend_reply_prefix(reply_prefix.as_deref(), &placeholder_body);
        let adapter = DiscordOutboundAdapter {
            http: ctx.http.clone(),
            channel_id: msg.channel_id,
            max_len: self.core.config.discord.max_message_length,
            reply_prefix,
            show_reasoning: false,
        };

        let summary = run_session_with_outbound(
            &mut session,
            adapter,
            &envelope.text,
            &placeholder_text,
            AgentRunOptions::default(),
        )
        .await;

        if summary.placeholder_sent {
            self.core.log_message_step(
                "discord",
                author_id,
                &session_id,
                "reply.placeholder",
                "sent",
                None,
                None,
            );
        } else {
            self.core.log_message_step(
                "discord",
                author_id,
                &session_id,
                "reply.placeholder",
                "failed",
                None,
                None,
            );
        }

        if summary.result.response.success {
            self.core.log_message_step(
                "discord",
                author_id,
                &session_id,
                "reply.send",
                &format!("segments.sent={}", summary.sent_segments),
                None,
                None,
            );
        }

        Ok(())
    }

    async fn handle_direct_message(
        &self,
        ctx: &Context,
        msg: &Message,
        author_id: &str,
    ) -> hone_core::HoneResult<()> {
        let target = format!("dm:{}", msg.channel_id.get());
        let (actor, channel_target, chat_mode) =
            self.scope_resolver.direct(author_id, target.clone())?;
        let envelope = IncomingEnvelope {
            message_id: Some(msg.id.get().to_string()),
            actor: actor.clone(),
            session_identity: SessionIdentity::from_actor(&actor)
                .expect("discord actor should always map to a session identity"),
            session_id: SessionIdentity::from_actor(&actor)
                .expect("discord actor should always map to a session identity")
                .session_id(),
            channel_target,
            chat_mode,
            text: msg.content.trim().to_string(),
            attachments: Vec::new(),
            trigger: GroupTrigger::default(),
            recv_extra: None,
            session_metadata: None,
            message_metadata: Default::default(),
        };
        if let Some(reply) = self
            .core
            .try_handle_intercept_command(&envelope.actor, &envelope.text)
            .await
        {
            msg.channel_id
                .say(&ctx.http, reply)
                .await
                .map_err(|err| hone_core::HoneError::Channel(err.to_string()))?;
            return Ok(());
        }
        let session_id = envelope.session_id.clone();
        let _session_guard = self.session_locks.lock(&session_id).await;

        let raw_attachments = collect_raw_attachments(msg).await;
        let attachments = ingest_raw_attachments(
            self.core.as_ref(),
            AttachmentIngestRequest {
                channel: "discord".to_string(),
                actor: envelope.actor.clone(),
                session_id: session_id.clone(),
                attachments: raw_attachments,
            },
        )
        .await;
        if !attachments.is_empty() {
            spawn_attachment_persist_pipeline(
                self.core.clone(),
                AttachmentPersistRequest {
                    channel: "discord".to_string(),
                    actor: envelope.actor.clone(),
                    user_id: author_id.to_string(),
                    session_id: session_id.clone(),
                    attachments: attachments.clone(),
                },
            );
        }

        let input = build_dm_user_input(&envelope.text, &attachments);
        if input.trim().is_empty() {
            self.core.log_message_step(
                "discord",
                &author_id,
                &session_id,
                "ignore",
                "empty_input_after_normalization",
                None,
                None,
            );
            return Ok(());
        }

        let recv_extra = format!("attachments={}", attachments.len());
        let is_admin = self.core.is_admin_actor(&envelope.actor);
        let prompt_options = PromptOptions {
            is_admin,
            ..PromptOptions::default()
        };
        let mut session = AgentSession::new(
            self.core.clone(),
            envelope.actor.clone(),
            envelope.channel_target.clone(),
        )
        .with_session_identity(envelope.session_identity.clone())
        .with_message_id(envelope.message_id.clone())
        .with_prompt_options(prompt_options)
        .with_recv_extra(Some(recv_extra))
        .with_message_metadata(envelope.message_metadata.clone())
        .with_cron_allowed(envelope.cron_allowed());

        let placeholder_text = if attachments.is_empty() {
            "正在思考中...".to_string()
        } else {
            build_attachment_ack_message(&attachments)
        };
        let adapter = DiscordOutboundAdapter {
            http: ctx.http.clone(),
            channel_id: msg.channel_id,
            max_len: self.core.config.discord.max_message_length,
            reply_prefix: None,
            show_reasoning: true,
        };

        let summary = run_session_with_outbound(
            &mut session,
            adapter,
            &input,
            &placeholder_text,
            AgentRunOptions::default(),
        )
        .await;

        if summary.placeholder_sent {
            self.core.log_message_step(
                "discord",
                &author_id,
                &session_id,
                "reply.placeholder",
                "sent",
                None,
                None,
            );
        } else {
            self.core.log_message_step(
                "discord",
                &author_id,
                &session_id,
                "reply.placeholder",
                "failed",
                None,
                None,
            );
        }

        if summary.result.response.success {
            self.core.log_message_step(
                "discord",
                &author_id,
                &session_id,
                "reply.send",
                &format!("segments.sent={}", summary.sent_segments),
                None,
                None,
            );
        }

        Ok(())
    }
}
