//! Signed, login-free unsubscribe links for scheduled pushes.
//!
//! A push that arrives in Feishu, Discord or email reaches someone who has no
//! session on the web app and often cannot get one. The link therefore has to
//! carry its own authority. It carries exactly one thing — permission to stop
//! this one job — and nothing about who the recipient is.
//!
//! Two properties matter and both are structural rather than advisory:
//!
//! * **Unforgeable.** The job id is signed with a server-held secret, so a
//!   link cannot be constructed or enumerated. Without this, anyone could walk
//!   ids and silently switch off other people's pushes.
//! * **Not self-executing.** Verifying a token never changes anything. Feishu,
//!   Discord and mail clients all fetch links to build previews, so a GET that
//!   unsubscribed would unsubscribe people who never clicked. The caller is
//!   expected to render a confirmation and act only on an explicit POST.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Why a token was refused. The caller must render the same page for every
/// variant: distinguishing "bad signature" from "unknown job" would tell an
/// attacker which ids exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsubscribeTokenError {
    Malformed,
    BadSignature,
    /// No secret configured, so no token can be trusted. Failing closed here
    /// keeps a misconfigured deployment from accepting unsigned links.
    NotConfigured,
}

/// Length of the hex signature carried in the link. Sixteen bytes of HMAC is
/// far past brute force for a value that only ever disables one push, and it
/// keeps the URL short enough to survive chat clients that shorten or wrap.
const SIGNATURE_BYTES: usize = 16;

fn signature_for(secret: &str, job_id: &str) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts a key of any length");
    mac.update(b"hone.unsubscribe.v1:");
    mac.update(job_id.as_bytes());
    mac.finalize().into_bytes()[..SIGNATURE_BYTES].to_vec()
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || text.is_empty() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).ok())
        .collect()
}

/// `{job_id}.{signature}`. The job id stays readable so an operator reading a
/// log can tell which push a complaint is about without reversing anything.
pub fn issue_unsubscribe_token(secret: &str, job_id: &str) -> Option<String> {
    let secret = secret.trim();
    let job_id = job_id.trim();
    if secret.is_empty() || job_id.is_empty() || job_id.contains('.') {
        return None;
    }
    Some(format!(
        "{job_id}.{}",
        to_hex(&signature_for(secret, job_id))
    ))
}

/// Returns the job this token authorises. Pure: it never mutates anything, so
/// a link preview cannot unsubscribe anyone.
pub fn verify_unsubscribe_token(
    secret: &str,
    token: &str,
) -> Result<String, UnsubscribeTokenError> {
    let secret = secret.trim();
    if secret.is_empty() {
        return Err(UnsubscribeTokenError::NotConfigured);
    }
    let token = token.trim();
    let (job_id, signature) = token
        .rsplit_once('.')
        .ok_or(UnsubscribeTokenError::Malformed)?;
    if job_id.is_empty() || signature.is_empty() {
        return Err(UnsubscribeTokenError::Malformed);
    }
    let provided = from_hex(signature).ok_or(UnsubscribeTokenError::Malformed)?;
    let expected = signature_for(secret, job_id);
    // Constant-time: a length-independent early return would leak how much of
    // a guessed signature was correct.
    if provided.len() != expected.len() || !constant_time_eq(&provided, &expected) {
        return Err(UnsubscribeTokenError::BadSignature);
    }
    Ok(job_id.to_string())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.iter()
        .zip(right.iter())
        .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// The link put at the bottom of every delivered push.
pub fn unsubscribe_link(base_url: &str, token: &str) -> String {
    format!(
        "{}/unsubscribe/{token}",
        base_url.trim().trim_end_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret";

    #[test]
    fn a_token_round_trips_to_the_job_it_authorises() {
        let token = issue_unsubscribe_token(SECRET, "job-123").expect("token");
        assert!(token.starts_with("job-123."));
        assert_eq!(
            verify_unsubscribe_token(SECRET, &token),
            Ok("job-123".into())
        );
    }

    #[test]
    fn a_link_cannot_be_forged_or_walked() {
        let token = issue_unsubscribe_token(SECRET, "job-123").expect("token");
        // Someone who knows a job id still cannot build a working link.
        assert_eq!(
            verify_unsubscribe_token(SECRET, "job-123.00000000000000000000000000000000"),
            Err(UnsubscribeTokenError::BadSignature)
        );
        // Nor can they reuse one job's signature for another job.
        let swapped = token.replace("job-123", "job-124");
        assert_eq!(
            verify_unsubscribe_token(SECRET, &swapped),
            Err(UnsubscribeTokenError::BadSignature)
        );
        // A different deployment secret does not accept it either.
        assert_eq!(
            verify_unsubscribe_token("other-secret", &token),
            Err(UnsubscribeTokenError::BadSignature)
        );
    }

    #[test]
    fn a_deployment_without_a_secret_trusts_nothing() {
        // Failing open here would accept unsigned links from anyone.
        assert_eq!(issue_unsubscribe_token("", "job-123"), None);
        assert_eq!(
            verify_unsubscribe_token("  ", "job-123.aabb"),
            Err(UnsubscribeTokenError::NotConfigured)
        );
    }

    #[test]
    fn malformed_tokens_are_refused_without_panicking() {
        for bad in [
            "",
            "no-separator",
            "job-123.",
            ".signature",
            "job-123.zz",
            "job-123.abc",
        ] {
            assert!(
                matches!(
                    verify_unsubscribe_token(SECRET, bad),
                    Err(UnsubscribeTokenError::Malformed | UnsubscribeTokenError::BadSignature)
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_job_id_containing_the_separator_is_refused_at_issue_time() {
        // `rsplit_once` would otherwise recover a truncated id and the token
        // would authorise a job nobody meant.
        assert_eq!(issue_unsubscribe_token(SECRET, "job.with.dots"), None);
    }

    #[test]
    fn the_job_id_stays_readable_and_the_recipient_stays_anonymous() {
        let token = issue_unsubscribe_token(SECRET, "job-abc").expect("token");
        assert!(token.contains("job-abc"));
        // Nothing about the person receiving the push is encoded in the link.
        assert_eq!(token.matches('.').count(), 1);
    }

    #[test]
    fn the_link_joins_cleanly_whatever_the_base_url_looks_like() {
        for base in ["https://hone-claw.com", "https://hone-claw.com/"] {
            assert_eq!(
                unsubscribe_link(base, "job-1.abcd"),
                "https://hone-claw.com/unsubscribe/job-1.abcd"
            );
        }
    }
}
