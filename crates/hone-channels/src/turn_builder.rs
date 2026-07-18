use hone_core::{ActorIdentity, HoneResult};

use crate::HoneBotCore;
use chrono::{DateTime, FixedOffset};

use crate::prompt::{PromptOptions, build_prompt_bundle_at};

#[derive(Debug, Clone)]
pub(crate) struct SlashSkillExpansion {
    pub(crate) raw_input: String,
    pub(crate) invoked_prompt: String,
    pub(crate) runtime_input: String,
    pub(crate) skill_id: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PromptTurnInput {
    pub(crate) system_prompt: String,
    pub(crate) runtime_input: String,
    pub(crate) answer_time_beijing: String,
}

pub(crate) struct PromptTurnBuilder<'a> {
    core: &'a HoneBotCore,
    actor: &'a ActorIdentity,
    session_id: &'a str,
    prompt_options: PromptOptions,
    allow_cron: bool,
    recv_extra: Option<&'a str>,
}

impl<'a> PromptTurnBuilder<'a> {
    pub(crate) fn new(
        core: &'a HoneBotCore,
        actor: &'a ActorIdentity,
        session_id: &'a str,
        prompt_options: PromptOptions,
        allow_cron: bool,
        recv_extra: Option<&'a str>,
    ) -> Self {
        Self {
            core,
            actor,
            session_id,
            prompt_options,
            allow_cron,
            recv_extra,
        }
    }

    pub(crate) fn resolve_prompt_input_at(
        &self,
        user_input: &str,
        prompt_time_beijing: DateTime<FixedOffset>,
    ) -> PromptTurnInput {
        let mut prompt_options = self.prompt_options.clone();
        if self.allow_cron {
            prompt_options
                .extra_sections
                .push(crate::prompt::DEFAULT_CRON_TASK_POLICY.to_string());
            if self.actor.channel == "web" {
                prompt_options
                    .extra_sections
                    .push(crate::prompt::DEFAULT_WEB_CRON_DELIVERY_POLICY.to_string());
            }
        }
        let stage_constraints =
            hone_tools::skill_runtime::SkillStageConstraints::new(self.allow_cron, None);
        let skill_runtime = self.build_skill_runtime();
        prompt_options.extra_sections.push(
            "【SkillTool】\n\
             - 本轮相关技能提示匹配任务时，先调用 skill_tool（MCP 名称可能是 hone/skill_tool）再继续。\n\
             - 没有匹配项、任务中途转向或现有技能不足时，调用 discover_skills（可能是 hone/discover_skills）。\n\
             - 不要声称已经加载技能；必须真实调用工具。附件类技能仅在当前消息确有对应附件时使用。"
                .to_string(),
        );
        let related_skills = skill_runtime.search_for_stage(
            user_input,
            &extract_possible_file_paths(user_input),
            5,
            &stage_constraints,
        );
        let mut bundle = build_prompt_bundle_at(
            &self.core.config,
            &self.core.session_storage,
            &self.actor.channel,
            self.session_id,
            &Default::default(),
            &prompt_options,
            prompt_time_beijing,
        );
        if self.core.effective_runner_manages_own_context(self.actor) {
            bundle.conversation_context = None;
        }
        let runtime_user_input = if related_skills.is_empty() {
            user_input.to_string()
        } else {
            let listing = related_skills
                .into_iter()
                .map(|skill| {
                    let mut line = format!("- {}: {}", skill.id, skill.description);
                    if let Some(when_to_use) = skill
                        .when_to_use
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                    {
                        line.push_str(" - ");
                        line.push_str(when_to_use.trim());
                    }
                    line
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "【本轮相关技能提示】\n{}\n如这些技能已覆盖下一步，就直接用 skill_tool（或 MCP 下的 hone/skill_tool）；否则再调用 discover_skills（或 hone/discover_skills）。\n\n{}",
                listing, user_input
            )
        };

        PromptTurnInput {
            system_prompt: bundle.system_prompt(),
            runtime_input: compose_runtime_input(&bundle, &runtime_user_input, self.recv_extra),
            answer_time_beijing: bundle.answer_time_beijing,
        }
    }

    pub(crate) fn expand_slash_skill_input(
        &self,
        user_input: &str,
    ) -> HoneResult<Option<SlashSkillExpansion>> {
        let trimmed = user_input.trim();
        if !trimmed.starts_with('/') {
            return Ok(None);
        }

        let runtime = self.build_skill_runtime();
        let stage_constraints =
            hone_tools::skill_runtime::SkillStageConstraints::new(self.allow_cron, None);

        if trimmed.strip_prefix("/skill").is_some() {
            let lines = trimmed.lines().collect::<Vec<_>>();
            let first_line = lines.first().copied().unwrap_or_default();
            let query = first_line.trim_start_matches("/skill").trim();
            if query.is_empty() {
                return Ok(None);
            }
            if let Some(skill) = runtime.resolve_skill_via_search_for_stage(
                query,
                &extract_possible_file_paths(user_input),
                &stage_constraints,
            ) {
                let invoked_prompt =
                    runtime.render_invocation_prompt(&skill, self.session_id, None);
                let tail = lines.iter().skip(1).copied().collect::<Vec<_>>().join("\n");
                let runtime_input =
                    compose_invoked_skill_runtime_input(&invoked_prompt, Some(tail.trim()));
                return Ok(Some(SlashSkillExpansion {
                    raw_input: user_input.to_string(),
                    invoked_prompt,
                    runtime_input,
                    skill_id: skill.id,
                }));
            }
            return Ok(None);
        }

        let command = trimmed.trim_start_matches('/');
        let mut parts = command.splitn(2, char::is_whitespace);
        let skill_id = parts.next().unwrap_or_default();
        let args = parts.next().map(str::trim);
        if skill_id.is_empty() {
            return Ok(None);
        }
        if let Some(skill) =
            runtime.resolve_user_invocable_direct_for_stage(skill_id, &stage_constraints)
        {
            let invoked_prompt = runtime.render_invocation_prompt(&skill, self.session_id, args);
            return Ok(Some(SlashSkillExpansion {
                raw_input: user_input.to_string(),
                invoked_prompt: invoked_prompt.clone(),
                runtime_input: compose_invoked_skill_runtime_input(&invoked_prompt, args),
                skill_id: skill.id,
            }));
        }
        Ok(None)
    }

    fn build_skill_runtime(&self) -> hone_tools::SkillRuntime {
        hone_tools::SkillRuntime::new(
            self.core.configured_system_skills_dir(),
            self.core.configured_custom_skills_dir(),
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        )
        .with_registry_path(self.core.configured_skill_registry_path())
    }
}

pub(crate) fn compose_runtime_input(
    bundle: &crate::prompt::PromptBundle,
    user_input: &str,
    recv_extra: Option<&str>,
) -> String {
    let extra = recv_extra.map(str::trim).filter(|value| !value.is_empty());
    if extra.is_none() {
        return bundle.compose_user_input(user_input);
    }

    let mut sections = Vec::new();

    if let Some(extra) = extra {
        sections.push(extra.to_string());
    }

    if let Some(context) = bundle
        .conversation_context
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        sections.push(context.to_string());
    }

    if let Some(session_context) =
        Some(bundle.session_context.trim()).filter(|value| !value.is_empty())
    {
        sections.push(session_context.to_string());
    }

    sections.push(format!("【本轮用户输入】\n{}", user_input.trim()));

    sections.join("\n\n")
}

pub(crate) fn extract_possible_file_paths(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter(|token| token.contains('/') || token.contains('\\'))
        .map(|token| {
            token.trim_matches(|ch: char| ch.is_ascii_punctuation() && ch != '/' && ch != '\\')
        })
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

pub(crate) fn compose_invoked_skill_runtime_input(
    invoked_prompt: &str,
    user_supplement: Option<&str>,
) -> String {
    if let Some(supplement) = user_supplement
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        format!("{invoked_prompt}\n\n【User Task After Invoking This Skill】\n{supplement}")
    } else {
        invoked_prompt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::PromptBundle;

    #[test]
    fn runtime_input_with_recv_extra_keeps_current_turn_last() {
        let bundle = PromptBundle {
            static_system: String::new(),
            conversation_context: Some(
                "【历史会话总结】\n旧 LITE stock_research 上下文".to_string(),
            ),
            session_context: "【Session 上下文】\n当前时间：2026-05-01 12:00:00".to_string(),
            answer_time_beijing: "2026-05-01 12:00".to_string(),
        };

        let input = compose_runtime_input(
            &bundle,
            "AMD的电脑CPU是什么名字",
            Some("【接收消息元信息】"),
        );
        let extra_pos = input.find("【接收消息元信息】").expect("extra section");
        let history_pos = input.find("旧 LITE").expect("history section");
        let session_pos = input.find("【Session 上下文】").expect("session section");
        let current_pos = input.find("【本轮用户输入】").expect("current turn");

        assert!(extra_pos < current_pos);
        assert!(history_pos < current_pos);
        assert!(session_pos < current_pos);
        assert!(input.ends_with("AMD的电脑CPU是什么名字"));
    }
}
