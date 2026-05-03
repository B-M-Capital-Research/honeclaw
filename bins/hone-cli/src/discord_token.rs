//! Discord bot token 的格式校验 + doctor check。
//!
//! Discord bot token 的规则：
//! - `xxx.yyy.zzz` 三段式,中间用 `.` 分隔
//! - 每段都是 base64url 字符集（A-Z / a-z / 0-9 / `-` / `_`)
//! - 合理长度大致在 50~120 字节,偏离会触发 warn
//!
//! 本 module 只做**格式**校验,不会拿 token 去 Discord API 验证——那是 online
//! check,留给 runtime 起 channel 时发现真 token 无效再报错。

use dialoguer::theme::ColorfulTheme;

use crate::display::{fail_line, ok_line, warn_line};
use crate::i18n::{Lang, t, tpl};
use crate::prompts::{normalize_credential_value, prompt_bool, prompt_visible_credential};
use crate::reports::DoctorCheck;

/// Discord token 的格式校验结论。`Warn` 表示可能有问题但仍允许保存。
///
/// 嵌入的 `&'static str` 是 i18n 翻译键(例如 `"discord_token.too_short"`),由调用方
/// 通过 `t(lang, key)` 解析成对应语言的人类可读文案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscordTokenValidation {
    Valid,
    Warn(&'static str),
    Invalid(&'static str),
}

/// 单段是否由合法的 base64url 字符组成（且非空）。
fn is_base64url_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

/// 对 trim 后的 token 做格式校验（不访问网络）。
pub(crate) fn validate_discord_token(value: &str) -> DiscordTokenValidation {
    let token = value.trim();
    if token.is_empty() {
        return DiscordTokenValidation::Invalid("discord_token.empty");
    }

    let segments = token.split('.').collect::<Vec<_>>();
    if segments.len() != 3 {
        return DiscordTokenValidation::Invalid("discord_token.bad_segments");
    }
    if !segments.iter().all(|segment| is_base64url_segment(segment)) {
        return DiscordTokenValidation::Invalid("discord_token.bad_charset");
    }

    let len = token.len();
    if len < 50 {
        DiscordTokenValidation::Warn("discord_token.too_short")
    } else if len > 120 {
        DiscordTokenValidation::Warn("discord_token.too_long")
    } else {
        DiscordTokenValidation::Valid
    }
}

/// 生成一条可塞进 `doctor` 报告的 DoctorCheck，详情里带长度用于肉眼 sanity check。
pub(crate) fn discord_token_doctor_check(lang: Lang, token: &str) -> DoctorCheck {
    let token = token.trim();
    let len = token.len();
    let (status, detail) = match validate_discord_token(token) {
        DiscordTokenValidation::Valid => (
            "ok",
            tpl(t(lang, "discord_token.doctor_ok"), &[("len", &len)]),
        ),
        DiscordTokenValidation::Warn(key) => (
            "warn",
            tpl(
                t(lang, "discord_token.message_with_len"),
                &[("message", &t(lang, key)), ("len", &len)],
            ),
        ),
        DiscordTokenValidation::Invalid(key) => (
            "fail",
            tpl(
                t(lang, "discord_token.message_with_len"),
                &[("message", &t(lang, key)), ("len", &len)],
            ),
        ),
    };
    DoctorCheck {
        name: "discord-token-format".to_string(),
        status,
        detail,
    }
}

/// 可选型 Discord token prompt：用户可以留空表示「保留现有/跳过」。
/// 空值直接返回 `None`;有值则按格式校验分三档处理(Valid / Warn / Invalid)。
/// `configure` 流程调用这一版;onboard 走另一条严格必填的版本
/// (`onboard::prompt_onboard_required_discord_token`)。
pub(crate) fn prompt_optional_discord_token(
    theme: &ColorfulTheme,
    lang: Lang,
    prompt: &str,
    current: &str,
    keep_note: bool,
) -> Result<Option<String>, String> {
    loop {
        let Some(token) = prompt_visible_credential(theme, lang, prompt, keep_note, current)?
        else {
            return Ok(None);
        };
        let normalized_token = normalize_credential_value(&token);
        let len = normalized_token.len();
        match validate_discord_token(&normalized_token) {
            DiscordTokenValidation::Valid => {
                ok_line(&tpl(
                    t(lang, "discord_token.valid_with_len"),
                    &[("len", &len)],
                ));
                return Ok(Some(normalized_token));
            }
            DiscordTokenValidation::Warn(key) => {
                warn_line(&tpl(
                    t(lang, "discord_token.message_with_len"),
                    &[("message", &t(lang, key)), ("len", &len)],
                ));
                if prompt_bool(theme, t(lang, "discord_token.confirm_save"), false)? {
                    return Ok(Some(normalized_token));
                }
            }
            DiscordTokenValidation::Invalid(key) => {
                fail_line(&tpl(
                    t(lang, "discord_token.message_with_len"),
                    &[("message", &t(lang, key)), ("len", &len)],
                ));
                if !prompt_bool(theme, t(lang, "discord_token.confirm_retry"), true)? {
                    return Ok(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_discord_token_accepts_expected_shape_and_length() {
        let token = format!("{}.{}.{}", "A".repeat(24), "b".repeat(6), "C".repeat(36));
        assert_eq!(
            validate_discord_token(&token),
            DiscordTokenValidation::Valid
        );
    }

    #[test]
    fn validate_discord_token_warns_when_length_is_abnormally_long() {
        let token = format!("{}.{}.{}", "A".repeat(48), "b".repeat(6), "C".repeat(96));
        assert_eq!(
            validate_discord_token(&token),
            DiscordTokenValidation::Warn("discord_token.too_long")
        );
    }

    #[test]
    fn validate_discord_token_rejects_non_three_segment_shape() {
        let token = "not-a-discord-token";
        assert_eq!(
            validate_discord_token(token),
            DiscordTokenValidation::Invalid("discord_token.bad_segments")
        );
    }

    #[test]
    fn discord_token_doctor_check_reports_warning() {
        let token = format!("{}.{}.{}", "A".repeat(48), "b".repeat(6), "C".repeat(96));
        let check = discord_token_doctor_check(Lang::Zh, &token);
        assert_eq!(check.name, "discord-token-format");
        assert_eq!(check.status, "warn");
        assert!(check.detail.contains("长度异常偏长"));
    }
}
