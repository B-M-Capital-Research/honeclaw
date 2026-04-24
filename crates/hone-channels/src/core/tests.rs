//! `HoneBotCore` 的回归测试。
//!
//! 覆盖三组场景:
//! - 管理员运行时注册 (`/register-admin`) 的白名单 / 口令 / 作用域判定;
//! - `is_admin*` 对各渠道 actor 的识别;
//! - `create_tool_registry` 的 actor-scoped 工具注入;
//! - `/report` intercept 的解析、默认 payload、multi-agent key 回退。

use hone_core::{ActorIdentity, HoneConfig};
use serde_json::json;

use super::bot_core::HoneBotCore;
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

#[test]
fn runtime_admin_override_requires_whitelisted_actor_and_configured_passphrase() {
    let core = HoneBotCore::new(HoneConfig::default());
    let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
    assert_eq!(
        core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
        Some(REGISTER_ADMIN_INTERCEPT_DENY_ACK.to_string())
    );
}

#[test]
fn runtime_admin_override_rejects_when_passphrase_missing_or_invalid() {
    let mut config = HoneConfig::default();
    config.admins.discord_user_ids = vec!["alice".to_string()];
    let core = HoneBotCore::new(config.clone());
    let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");

    assert_eq!(
        core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
        Some(REGISTER_ADMIN_INTERCEPT_DISABLED_ACK.to_string())
    );

    config.admins.runtime_admin_registration_passphrase = "secret".to_string();
    let core = HoneBotCore::new(config);
    assert_eq!(
        core.try_intercept_admin_registration(
            &actor,
            &format!("{REGISTER_ADMIN_INTERCEPT_PREFIX} wrong")
        ),
        Some(REGISTER_ADMIN_INTERCEPT_INVALID_ACK.to_string())
    );
}

#[test]
fn runtime_admin_override_is_scoped_to_actor_identity() {
    let mut config = HoneConfig::default();
    config.admins.discord_user_ids = vec!["alice".to_string()];
    config.admins.runtime_admin_registration_passphrase = "secret".to_string();
    let core = HoneBotCore::new(config);
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

#[test]
fn telegram_admin_allowlist_is_honored() {
    let mut config = HoneConfig::default();
    config.admins.telegram_user_ids = vec!["8039067465".to_string()];
    let core = HoneBotCore::new(config);

    assert!(core.is_admin("8039067465", "telegram"));
    assert!(!core.is_admin("999", "telegram"));

    let actor = ActorIdentity::new("telegram", "8039067465", Some("dm:8039067465")).expect("actor");
    assert!(core.is_admin_actor(&actor));
}

#[test]
fn actor_scoped_registry_includes_local_file_tools() {
    let core = HoneBotCore::new(HoneConfig::default());
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
    assert_eq!(parse_report_intercept("/report"), None);
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

#[test]
fn effective_multi_agent_search_config_falls_back_to_auxiliary_api_key() {
    let mut config = HoneConfig::default();
    config.agent.runner = "multi-agent".to_string();
    config.agent.multi_agent.search.base_url = "https://api.minimaxi.com/v1".to_string();
    config.agent.multi_agent.search.model = "MiniMax-M2.7-highspeed".to_string();
    config.agent.multi_agent.search.api_key = String::new();
    config.llm.auxiliary.base_url = "https://api.minimaxi.com/v1".to_string();
    config.llm.auxiliary.model = "MiniMax-M2.7-highspeed".to_string();
    config.llm.auxiliary.api_key = "sk-cp-aux".to_string();

    let core = HoneBotCore::new(config);
    let effective = core.effective_multi_agent_search_config();

    assert_eq!(effective.api_key, "sk-cp-aux");
    assert_eq!(effective.base_url, "https://api.minimaxi.com/v1");
    assert_eq!(effective.model, "MiniMax-M2.7-highspeed");
}

#[test]
fn effective_multi_agent_search_config_preserves_explicit_search_api_key() {
    let mut config = HoneConfig::default();
    config.agent.runner = "multi-agent".to_string();
    config.agent.multi_agent.search.api_key = "sk-cp-search".to_string();
    config.llm.auxiliary.api_key = "sk-cp-aux".to_string();

    let core = HoneBotCore::new(config);
    let effective = core.effective_multi_agent_search_config();

    assert_eq!(effective.api_key, "sk-cp-search");
}

#[test]
fn multi_agent_answer_zero_tool_limit_is_preserved() {
    let mut config = HoneConfig::default();
    config.agent.runner = "multi-agent".to_string();
    config.agent.multi_agent.answer.max_tool_calls = 0;

    let core = HoneBotCore::new(config);

    assert_eq!(core.effective_multi_agent_answer_max_tool_calls(), 0);
}
