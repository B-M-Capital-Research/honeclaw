//! Discord bot token 的格式校验 + doctor check。
//!
//! Discord bot token 的规则：
//! - `xxx.yyy.zzz` 三段式,中间用 `.` 分隔
//! - 每段都是 base64url 字符集（A-Z / a-z / 0-9 / `-` / `_`)
//! - 合理长度大致在 50~120 字节,偏离会触发 warn
//!
//! 本 module 只做**格式**校验,不会拿 token 去 Discord API 验证——那是 online
//! check,留给 runtime 起 channel 时发现真 token 无效再报错。

use crate::reports::DoctorCheck;

/// Discord token 的格式校验结论。`Warn` 表示可能有问题但仍允许保存。
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
        return DiscordTokenValidation::Invalid("Token 不能为空。");
    }

    let segments = token.split('.').collect::<Vec<_>>();
    if segments.len() != 3 {
        return DiscordTokenValidation::Invalid("Token 必须是三段结构（形如 xxx.yyy.zzz）。");
    }
    if !segments.iter().all(|segment| is_base64url_segment(segment)) {
        return DiscordTokenValidation::Invalid("Token 包含非法字符，应为 base64url 字符集。");
    }

    let len = token.len();
    if len < 50 {
        DiscordTokenValidation::Warn("Token 长度偏短，请确认是否粘贴完整。")
    } else if len > 120 {
        DiscordTokenValidation::Warn("Token 长度异常偏长，请检查是否重复粘贴。")
    } else {
        DiscordTokenValidation::Valid
    }
}

/// 生成一条可塞进 `doctor` 报告的 DoctorCheck，详情里带长度用于肉眼 sanity check。
pub(crate) fn discord_token_doctor_check(token: &str) -> DoctorCheck {
    let token = token.trim();
    let len = token.len();
    let (status, detail) = match validate_discord_token(token) {
        DiscordTokenValidation::Valid => {
            ("ok", format!("Discord token 基本格式有效（长度={len}）。"))
        }
        DiscordTokenValidation::Warn(message) => ("warn", format!("{message}（长度={len}）。")),
        DiscordTokenValidation::Invalid(message) => ("fail", format!("{message}（长度={len}）。")),
    };
    DoctorCheck {
        name: "discord-token-format".to_string(),
        status,
        detail,
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
            DiscordTokenValidation::Warn("Token 长度异常偏长，请检查是否重复粘贴。")
        );
    }

    #[test]
    fn validate_discord_token_rejects_non_three_segment_shape() {
        let token = "not-a-discord-token";
        assert_eq!(
            validate_discord_token(token),
            DiscordTokenValidation::Invalid("Token 必须是三段结构（形如 xxx.yyy.zzz）。")
        );
    }

    #[test]
    fn discord_token_doctor_check_reports_warning() {
        let token = format!("{}.{}.{}", "A".repeat(48), "b".repeat(6), "C".repeat(96));
        let check = discord_token_doctor_check(&token);
        assert_eq!(check.name, "discord-token-format");
        assert_eq!(check.status, "warn");
        assert!(check.detail.contains("长度异常偏长"));
    }
}
