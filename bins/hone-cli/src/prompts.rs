//! 通用交互式 prompt 工具 + 共享的「空值恢复」动作枚举。
//!
//! 这里只放**不依赖具体业务流程**的 prompt primitives:
//! - `prompt_text` / `prompt_bool` / `prompt_secret` / `prompt_visible_credential`
//!   / `prompt_select_index` —— dialoguer 的薄包装,统一错误类型
//! - `RequiredFieldEmptyAction` / `RequiredFieldResolution` / `ProviderEmptyAction`
//!   —— 通用「必填项空值 / Provider key 空值」恢复决策类型
//! - `resolve_required_field_attempt` / `resolve_required_secret_attempt` ——
//!   把用户输入 + 已有配置值 + 恢复动作组合成最终 resolution
//! - `prompt_channel_recovery_action` / `prompt_provider_recovery_action` ——
//!   面对空必填项 / 空 provider key 时让用户选择「重试 / 放弃」
//! - `normalize_credential_value` / `normalize_credential_value_opt` ——
//!   去首尾空白的 secret 清洗器（避免粘贴空格）
//!
//! `onboard.rs` 和 `configure.rs`(以及将来可能出现的其它向导)都基于这些
//! primitive 组装各自的业务步骤。

use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};

use crate::i18n::{Lang, t, tpl};

/// 必填项为空时,用户的 2 选 1 决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequiredFieldEmptyAction {
    Retry,
    DisableChannel,
}

/// 解析一次必填项输入的最终结果。
///
/// `Value`：拿到了新的（或维持旧的）非空值;`Retry`：让外层循环再次 prompt;
/// `DisableChannel`：用户决定放弃填写并禁用对应渠道。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequiredFieldResolution {
    Value(String),
    Retry,
    DisableChannel,
}

/// 供应商(FMP / Tavily 等)API key 为空时的 2 选 1 决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderEmptyAction {
    Retry,
    Skip,
}

/// 把 secret / token 粘贴值统一 trim,避免首尾空格被写进 yaml。
pub(crate) fn normalize_credential_value(raw: &str) -> String {
    raw.trim().to_string()
}

/// `normalize_credential_value` 的 `Option<&str>` 版:空串直接当 `None`。
pub(crate) fn normalize_credential_value_opt(raw: Option<&str>) -> Option<String> {
    raw.map(normalize_credential_value)
        .filter(|value| !value.is_empty())
}

/// 统一解析「必填项输入」。
///
/// 优先使用本次 prompt 的返回值;若为空则退回到已有配置值;若仍为空则按
/// `on_empty` 决定重试或禁用。用于同步(非交互)上下文;interactive 场景
/// 见 [`resolve_required_secret_attempt`] 接 FnOnce 的版本。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn resolve_required_field_attempt(
    attempted: Option<String>,
    current: &str,
    on_empty: RequiredFieldEmptyAction,
) -> RequiredFieldResolution {
    if let Some(value) = normalize_credential_value_opt(attempted.as_deref()) {
        return RequiredFieldResolution::Value(value);
    }
    if let Some(value) = normalize_credential_value_opt(Some(current)) {
        return RequiredFieldResolution::Value(value);
    }
    match on_empty {
        RequiredFieldEmptyAction::Retry => RequiredFieldResolution::Retry,
        RequiredFieldEmptyAction::DisableChannel => RequiredFieldResolution::DisableChannel,
    }
}

/// [`resolve_required_field_attempt`] 的 interactive 变体：`on_empty` 是一个
/// 在真的需要决策时才被调用的 `FnOnce`,避免无意义地 prompt。
pub(crate) fn resolve_required_secret_attempt<F>(
    attempted: Option<String>,
    current: &str,
    on_empty: F,
) -> Result<RequiredFieldResolution, String>
where
    F: FnOnce() -> Result<RequiredFieldEmptyAction, String>,
{
    if let Some(value) = normalize_credential_value_opt(attempted.as_deref()) {
        return Ok(RequiredFieldResolution::Value(value));
    }
    if let Some(value) = normalize_credential_value_opt(Some(current)) {
        return Ok(RequiredFieldResolution::Value(value));
    }
    Ok(match on_empty()? {
        RequiredFieldEmptyAction::Retry => RequiredFieldResolution::Retry,
        RequiredFieldEmptyAction::DisableChannel => RequiredFieldResolution::DisableChannel,
    })
}

pub(crate) fn prompt_text(
    theme: &ColorfulTheme,
    prompt: &str,
    current: &str,
) -> Result<String, String> {
    let mut input = Input::<String>::with_theme(theme);
    input = input.with_prompt(prompt.to_string());
    if !current.is_empty() {
        input = input.with_initial_text(current.to_string());
    }
    input.interact_text().map_err(|e| e.to_string())
}

pub(crate) fn prompt_bool(
    theme: &ColorfulTheme,
    prompt: &str,
    current: bool,
) -> Result<bool, String> {
    Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(current)
        .interact()
        .map_err(|e| e.to_string())
}

/// 密码风格的 prompt:输入时隐藏,允许空值(代表「保持现有」)。
/// `keep_note=true` 会在提示里追加「留空保持现有值」。
pub(crate) fn prompt_secret(
    theme: &ColorfulTheme,
    lang: Lang,
    prompt: &str,
    keep_note: bool,
) -> Result<Option<String>, String> {
    let prompt = decorate_keep_note_prompt(lang, prompt, keep_note);
    let value = Password::with_theme(theme)
        .with_prompt(prompt)
        .allow_empty_password(true)
        .interact()
        .map_err(|e| e.to_string())?;
    Ok(normalize_credential_value_opt(Some(&value)))
}

/// 可见字段的 secret prompt(例如 bot token 那种不算完全机密、需要肉眼核对的)。
/// 与 [`prompt_secret`] 的区别：输入时显示、允许预填 `current`。
pub(crate) fn prompt_visible_credential(
    theme: &ColorfulTheme,
    lang: Lang,
    prompt: &str,
    keep_note: bool,
    current: &str,
) -> Result<Option<String>, String> {
    let prompt = decorate_keep_note_prompt(lang, prompt, keep_note);
    let mut input = Input::<String>::with_theme(theme);
    input = input.with_prompt(prompt).allow_empty(true);
    if !current.is_empty() {
        input = input.with_initial_text(current.to_string());
    }
    let value = input.interact_text().map_err(|e| e.to_string())?;
    Ok(normalize_credential_value_opt(Some(&value)))
}

/// `prompt` 后追加一段「留空保持现有值」的提示。
fn decorate_keep_note_prompt(lang: Lang, prompt: &str, keep_note: bool) -> String {
    if !keep_note {
        return prompt.to_string();
    }
    let suffix = match lang {
        Lang::Zh => "（留空保持现有值）",
        Lang::En => " (leave blank to keep existing)",
    };
    format!("{prompt}{suffix}")
}

pub(crate) fn prompt_select_index(
    theme: &ColorfulTheme,
    prompt: &str,
    items: &[String],
    default: usize,
) -> Result<usize, String> {
    Select::with_theme(theme)
        .with_prompt(prompt)
        .items(items)
        .default(default.min(items.len().saturating_sub(1)))
        .interact()
        .map_err(|e| e.to_string())
}

/// 必填项为空时让用户选:重试 / 回退并禁用整个渠道。
pub(crate) fn prompt_channel_recovery_action(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
    field_label: &str,
) -> Result<RequiredFieldEmptyAction, String> {
    let items = vec![
        t(lang, "recovery.option_retry").to_string(),
        tpl(
            t(lang, "recovery.option_disable_channel"),
            &[("label", &channel_label)],
        ),
    ];
    let idx = prompt_select_index(
        theme,
        &tpl(
            t(lang, "recovery.channel_required_empty_prompt"),
            &[("label", &channel_label), ("field", &field_label)],
        ),
        &items,
        0,
    )?;
    Ok(match idx {
        0 => RequiredFieldEmptyAction::Retry,
        _ => RequiredFieldEmptyAction::DisableChannel,
    })
}

/// Provider API key 为空时让用户选:重试 / 本轮跳过。
pub(crate) fn prompt_provider_recovery_action(
    theme: &ColorfulTheme,
    lang: Lang,
    provider_label: &str,
) -> Result<ProviderEmptyAction, String> {
    let items = vec![
        t(lang, "recovery.option_retry").to_string(),
        tpl(
            t(lang, "recovery.option_provider_skip"),
            &[("label", &provider_label)],
        ),
    ];
    let idx = prompt_select_index(
        theme,
        &tpl(
            t(lang, "recovery.provider_empty_prompt"),
            &[("label", &provider_label)],
        ),
        &items,
        0,
    )?;
    Ok(match idx {
        0 => ProviderEmptyAction::Retry,
        _ => ProviderEmptyAction::Skip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_required_field_attempt_disables_channel_when_empty_and_no_current_value() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "",
            RequiredFieldEmptyAction::DisableChannel,
        );

        assert_eq!(resolution, RequiredFieldResolution::DisableChannel);
    }

    #[test]
    fn resolve_required_field_attempt_retries_when_empty_and_no_current_value() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "",
            RequiredFieldEmptyAction::Retry,
        );

        assert_eq!(resolution, RequiredFieldResolution::Retry);
    }

    #[test]
    fn resolve_required_field_attempt_keeps_existing_value_on_empty_input() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "existing-secret",
            RequiredFieldEmptyAction::DisableChannel,
        );

        assert_eq!(
            resolution,
            RequiredFieldResolution::Value("existing-secret".to_string())
        );
    }

    #[test]
    fn resolve_required_field_attempt_trims_secret_values() {
        let resolution = resolve_required_field_attempt(
            Some("  new-secret  ".to_string()),
            "",
            RequiredFieldEmptyAction::DisableChannel,
        );

        assert_eq!(
            resolution,
            RequiredFieldResolution::Value("new-secret".to_string())
        );
    }

    #[test]
    fn resolve_required_secret_attempt_skips_empty_recovery_for_non_empty_input() {
        let mut on_empty_called = false;
        let resolution = resolve_required_secret_attempt(Some("token".to_string()), "", || {
            on_empty_called = true;
            Ok(RequiredFieldEmptyAction::Retry)
        })
        .unwrap();

        assert_eq!(
            resolution,
            RequiredFieldResolution::Value("token".to_string())
        );
        assert!(!on_empty_called);
    }
}
