//! `HoneBotCore` 的回归测试。
//!
//! 覆盖四组场景:
//! - 管理员运行时注册 (`/register-admin`) 的白名单 / 口令 / 作用域判定;
//! - `is_admin*` 对各渠道 actor 的识别;
//! - `create_tool_registry` 的 actor-scoped 工具注入;
//! - `/report` intercept 的解析与默认 payload。

use hone_core::config::AgentConversationStrategy;
use hone_core::{ActorIdentity, HoneConfig};
use serde_json::json;

use super::bot_core::{HoneBotCore, STRICT_ACTOR_MAX_ITERATIONS};
use super::intercept::{
    REGISTER_ADMIN_INTERCEPT_ACK, REGISTER_ADMIN_INTERCEPT_DENY_ACK,
    REGISTER_ADMIN_INTERCEPT_DISABLED_ACK, REGISTER_ADMIN_INTERCEPT_INVALID_ACK,
    REGISTER_ADMIN_INTERCEPT_PREFIX, REPORT_DEFAULT_MODE, REPORT_DEFAULT_RESEARCH_TOPIC,
    ReportIntercept, build_report_run_input, matches_register_admin_intercept,
    parse_report_intercept,
};

const REGISTER_ADMIN_INTERCEPT_TEXT: &str = "/register-admin secret";

#[test]
fn register_admin_intercept_matches_plain_and_quoted_text() {
    assert!(matches_register_admin_intercept(
        REGISTER_ADMIN_INTERCEPT_TEXT
    ));
    assert!(matches_register_admin_intercept(
        "' /register-admin secret '"
    ));
    assert!(matches_register_admin_intercept(
        "\"/register-admin secret\""
    ));
    assert!(!matches_register_admin_intercept("/register-admin"));
}

#[tokio::test]
async fn runtime_admin_override_requires_whitelisted_actor_and_configured_passphrase() {
    let core = HoneBotCore::new(HoneConfig::default()).await;
    let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
    assert_eq!(
        core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
        Some(REGISTER_ADMIN_INTERCEPT_DENY_ACK.to_string())
    );
}

#[tokio::test]
async fn runtime_admin_override_rejects_when_passphrase_missing_or_invalid() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.admins.discord_user_ids = vec!["alice".to_string()];
    let core = HoneBotCore::new(config.clone()).await;
    let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");

    assert_eq!(
        core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
        Some(REGISTER_ADMIN_INTERCEPT_DISABLED_ACK.to_string())
    );

    config.admins.runtime_admin_registration_passphrase = "secret".to_string();
    let core = HoneBotCore::new(config).await;
    assert_eq!(
        core.try_intercept_admin_registration(
            &actor,
            &format!("{REGISTER_ADMIN_INTERCEPT_PREFIX} wrong")
        ),
        Some(REGISTER_ADMIN_INTERCEPT_INVALID_ACK.to_string())
    );
}

#[tokio::test]
async fn runtime_admin_override_is_scoped_to_actor_identity() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.admins.discord_user_ids = vec!["alice".to_string()];
    config.admins.runtime_admin_registration_passphrase = "secret".to_string();
    let core = HoneBotCore::new(config).await;
    let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
    let other_scope = ActorIdentity::new("discord", "alice", Some("g:1:c:3")).expect("other scope");

    assert!(core.is_admin(&actor.user_id, &actor.channel));
    assert!(
        !core
            .runtime_admin_overrides
            .read()
            .unwrap()
            .contains(&actor)
    );
    assert_eq!(
        core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
        Some(REGISTER_ADMIN_INTERCEPT_ACK.to_string())
    );
    assert!(
        core.runtime_admin_overrides
            .read()
            .unwrap()
            .contains(&actor)
    );
    assert!(core.is_admin_actor(&actor));
    assert!(core.is_admin_actor(&other_scope));
}

#[tokio::test]
async fn telegram_admin_allowlist_is_honored() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.admins.telegram_user_ids = vec!["8039067465".to_string()];
    let core = HoneBotCore::new(config).await;

    assert!(core.is_admin("8039067465", "telegram"));
    assert!(!core.is_admin("999", "telegram"));

    let actor = ActorIdentity::new("telegram", "8039067465", Some("dm:8039067465")).expect("actor");
    assert!(core.is_admin_actor(&actor));
}

#[tokio::test]
async fn effective_context_owner_follows_actor_runner_route() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.agent.runner = "codex_acp".to_string();
    config.admins.discord_user_ids = vec!["admin".to_string()];
    let core = HoneBotCore::new(config).await;
    let public_actor =
        ActorIdentity::new("discord", "public", None::<String>).expect("public actor");
    let admin_actor = ActorIdentity::new("discord", "admin", None::<String>).expect("admin actor");

    assert!(core.actor_uses_strict_runner_fallback(&public_actor));
    assert_eq!(
        core.effective_runner_conversation_strategy(&public_actor),
        AgentConversationStrategy::StructuredReplay
    );
    assert!(!core.effective_runner_uses_native_codex_turns(&public_actor));
    assert!(!core.actor_uses_strict_runner_fallback(&admin_actor));
    assert_eq!(
        core.effective_runner_conversation_strategy(&admin_actor),
        AgentConversationStrategy::NativePersistent
    );
    assert!(core.effective_runner_uses_native_codex_turns(&admin_actor));
}

#[tokio::test]
async fn native_minimal_turns_are_codex_specific() {
    let actor = ActorIdentity::new("cli", "local", None::<String>).expect("actor");

    let mut opencode_config = HoneConfig::default();
    opencode_config.agent.runner = "opencode_acp".to_string();
    let opencode_core = HoneBotCore::new(opencode_config).await;
    assert_eq!(
        opencode_core.effective_runner_conversation_strategy(&actor),
        AgentConversationStrategy::EphemeralCompiledPrompt
    );
    assert!(!opencode_core.effective_runner_uses_native_codex_turns(&actor));

    let mut codex_config = HoneConfig::default();
    codex_config.agent.runner = "codex_acp".to_string();
    let codex_core = HoneBotCore::new(codex_config).await;
    assert!(codex_core.effective_runner_uses_native_codex_turns(&actor));
}

#[test]
fn strict_actor_runner_uses_the_standard_iteration_budget() {
    assert_eq!(STRICT_ACTOR_MAX_ITERATIONS, 18);
}

#[tokio::test]
async fn actor_scoped_registry_includes_local_file_tools() {
    let core = HoneBotCore::new(HoneConfig::default()).await;
    let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");

    let with_actor = core.create_tool_registry(Some(&actor), "discord", false);
    let without_actor = core.create_tool_registry(None, "discord", false);

    let with_actor_tools = with_actor.list_tool_names();
    assert!(with_actor_tools.contains(&"local_list_files"));
    assert!(with_actor_tools.contains(&"local_search_files"));
    assert!(with_actor_tools.contains(&"local_read_file"));

    let without_actor_tools = without_actor.list_tool_names();
    assert!(!without_actor_tools.contains(&"local_list_files"));
    assert!(!without_actor_tools.contains(&"local_search_files"));
    assert!(!without_actor_tools.contains(&"local_read_file"));
}

#[test]
fn report_intercept_parses_company_name_and_progress() {
    assert_eq!(
        parse_report_intercept("/report Tempus AI"),
        Some(ReportIntercept::Start {
            company_name: "Tempus AI".to_string()
        })
    );
    assert_eq!(
        parse_report_intercept("  '/report 进度'  "),
        Some(ReportIntercept::Progress)
    );
    assert_eq!(
        parse_report_intercept("/report progress"),
        Some(ReportIntercept::Progress)
    );
    assert!(parse_report_intercept("/report").is_none());
}

#[test]
fn report_run_input_includes_required_defaults() {
    assert_eq!(
        build_report_run_input("Astera Labs"),
        json!({
            "companyName": "Astera Labs",
            "genPost": REPORT_DEFAULT_MODE,
            "news": "",
            "task_id": "",
            "research_topic": REPORT_DEFAULT_RESEARCH_TOPIC,
        })
    );
}

#[tokio::test]
async fn web_admin_allowlist_is_honored() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.admins.web_user_ids = vec!["web-user-1234abcd5678".to_string()];
    let core = HoneBotCore::new(config).await;

    assert!(core.is_admin("web-user-1234abcd5678", "web"));
    assert!(!core.is_admin("web-user-other", "web"));

    let actor = ActorIdentity::new("web", "web-user-1234abcd5678", None::<String>).expect("actor");
    assert!(core.is_admin_actor(&actor));
}

#[tokio::test]
async fn conversation_profile_builds_dedicated_interactive_llm() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    let llm_yaml = r#"
providers:
  deepseek:
    kind: openai_compatible
    base_url: https://api.deepseek.com/v1
    api_key: test-key
  grok_build:
    kind: openai_compatible
    base_url: http://127.0.0.1:8899/v1
    api_key: local-proxy
profiles:
  main:
    provider: deepseek
    model: deepseek-v4-pro
  conversation:
    provider: grok_build
    model: grok-4.6
default_profile: main
conversation_profile: conversation
"#;
    config.llm = serde_yaml::from_str(llm_yaml).expect("llm config");
    let core = HoneBotCore::new(config).await;
    assert!(core.llm.is_some());
    assert!(core.conversation_llm.is_some());

    // 未配置 conversation_profile 时不建独立 provider,交互对话继续走默认 LLM。
    let mut plain = HoneConfig::default();
    plain.timezone = Some("Asia/Shanghai".to_string());
    let plain_core = HoneBotCore::new(plain).await;
    assert!(plain_core.conversation_llm.is_none());
}

#[tokio::test]
async fn admins_use_native_runner_false_routes_admins_to_strict() {
    let mut config = HoneConfig::default();
    // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
    // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
    config.timezone = Some("Asia/Shanghai".to_string());
    config.agent.runner = "codex_acp".to_string();
    config.agent.admins_use_native_runner = false;
    config.admins.web_user_ids = vec!["web-user-1234abcd5678".to_string()];
    let core = HoneBotCore::new(config).await;
    let admin_actor =
        ActorIdentity::new("web", "web-user-1234abcd5678", None::<String>).expect("actor");

    // 管理员权益(配额豁免等)仍在,但对话路由与普通用户一致走 strict。
    assert!(core.is_admin_actor(&admin_actor));
    assert!(core.actor_uses_strict_runner_fallback(&admin_actor));
    assert_eq!(
        core.effective_runner_conversation_strategy(&admin_actor),
        AgentConversationStrategy::StructuredReplay
    );
    assert!(!core.effective_runner_uses_native_codex_turns(&admin_actor));

    // 默认值保持现状:管理员继续用原生 runner。
    let mut default_config = HoneConfig::default();
    default_config.timezone = Some("Asia/Shanghai".to_string());
    default_config.agent.runner = "codex_acp".to_string();
    default_config.admins.web_user_ids = vec!["web-user-1234abcd5678".to_string()];
    assert!(default_config.agent.admins_use_native_runner);
    let default_core = HoneBotCore::new(default_config).await;
    assert!(!default_core.actor_uses_strict_runner_fallback(&admin_actor));
}
