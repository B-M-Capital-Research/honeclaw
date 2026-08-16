//! Login-free unsubscribe landing page.
//!
//! Someone who receives a push in Feishu, Discord or email has no session here
//! and usually cannot get one, so the signed token in the link is the whole
//! authorisation. Two consequences shape this module:
//!
//! * The page is server-rendered HTML with a plain form. Chat clients open
//!   links in embedded webviews where a single-page app may not boot; a page
//!   that needs JavaScript to work is a page that sometimes does not.
//! * `GET` never changes anything. Feishu, Discord and mail clients fetch
//!   links to build previews, so a `GET` that unsubscribed would silently
//!   unsubscribe people who never clicked. The confirmation button `POST`s.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use hone_core::unsubscribe_token::{UnsubscribeTokenError, verify_unsubscribe_token};
use hone_memory::cron_job::CronJobUpdate;

use crate::state::AppState;

/// What the visitor should see. Every refusal collapses to `Invalid`: telling
/// a caller apart "bad signature" from "no such job" would let them map which
/// job ids exist.
enum Outcome {
    Confirm { token: String, job_name: String },
    Done { job_name: String },
    Already { job_name: String },
    Invalid,
}

fn secret(state: &AppState) -> String {
    state.core.config.email.resolved_unsubscribe_secret()
}

async fn resolve(state: &AppState, token: &str) -> Result<(String, String, bool), ()> {
    let job_id = match verify_unsubscribe_token(&secret(state), token) {
        Ok(job_id) => job_id,
        // A deployment with no secret trusts nothing, and a forged or
        // malformed token is refused the same way a valid one for a deleted
        // job is.
        Err(UnsubscribeTokenError::Malformed)
        | Err(UnsubscribeTokenError::BadSignature)
        | Err(UnsubscribeTokenError::NotConfigured) => return Err(()),
    };
    // The link carries no identity, so the job is looked up across actors —
    // the signature is what proves the caller was given this link.
    let (_, job) = state
        .core
        .cron_job_storage()
        .get_job(&job_id, None)
        .await
        .ok_or(())?;
    Ok((job_id, job.name.clone(), job.enabled))
}

pub(crate) async fn handle_unsubscribe_page(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let outcome = match resolve(&state, &token).await {
        Ok((_, job_name, true)) => Outcome::Confirm { token, job_name },
        Ok((_, job_name, false)) => Outcome::Already { job_name },
        Err(()) => Outcome::Invalid,
    };
    render(outcome)
}

pub(crate) async fn handle_unsubscribe_submit(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let outcome = match resolve(&state, &token).await {
        Ok((_, job_name, false)) => Outcome::Already { job_name },
        Ok((job_id, job_name, true)) => {
            let updates = CronJobUpdate {
                // Disabled, not deleted: the person may want it back, and the
                // delivery history stays readable in the push centre.
                enabled: Some(false),
                ..Default::default()
            };
            match state
                .core
                .cron_job_storage()
                .update_job(&job_id, None, updates, true)
                .await
            {
                Ok(Some(_)) => Outcome::Done { job_name },
                // Someone else disabled it between the page load and the click.
                Ok(None) => Outcome::Already { job_name },
                Err(error) => {
                    tracing::warn!(%job_id, %error, "unsubscribe failed to disable job");
                    Outcome::Invalid
                }
            }
        }
        Err(()) => Outcome::Invalid,
    };
    render(outcome)
}

fn render(outcome: Outcome) -> Response {
    let (status, body) = match outcome {
        Outcome::Confirm { token, job_name } => (
            StatusCode::OK,
            page(
                "退订推送",
                &format!("确认不再接收「{}」的推送吗？", escape(&job_name)),
                "确认后这条定时推送会停止发送。你随时可以在 HONE 里重新开启它。",
                Some(&format!("/api/public/unsubscribe/{}", escape(&token))),
            ),
        ),
        Outcome::Done { job_name } => (
            StatusCode::OK,
            page(
                "已退订",
                &format!("已退订「{}」", escape(&job_name)),
                "这条推送不会再发送了。你随时可以在 HONE 的推送页面重新开启它。",
                None,
            ),
        ),
        Outcome::Already { job_name } => (
            StatusCode::OK,
            page(
                "已退订",
                &format!("「{}」此前已经退订过了", escape(&job_name)),
                "无需再次操作。你随时可以在 HONE 的推送页面重新开启它。",
                None,
            ),
        ),
        Outcome::Invalid => (
            StatusCode::NOT_FOUND,
            page(
                "链接无效",
                "这个退订链接无法使用",
                "它可能已经过期，或者这条推送已经被删除。你可以在 HONE 的推送页面直接管理订阅。",
                None,
            ),
        ),
    };
    (status, Html(body)).into_response()
}

/// Self-contained: no scripts, no external stylesheet, no fonts. It has to
/// render inside whatever embedded browser the chat client happens to use.
fn page(title: &str, heading: &str, detail: &str, action: Option<&str>) -> String {
    let button = action.map_or(String::new(), |action| {
        format!(
            r#"<form method="post" action="{action}"><button type="submit">确认退订</button></form>"#
        )
    });
    format!(
        r#"<!doctype html>
<html lang="zh-CN"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>{title} · HONE</title>
<style>
:root {{ color-scheme: light dark; }}
body {{ margin:0; min-height:100vh; display:flex; align-items:center; justify-content:center;
  background:#fffdf8; color:#17201f; font:16px/1.6 -apple-system,"PingFang SC",system-ui,sans-serif; }}
main {{ max-width:34rem; padding:2.5rem 1.5rem; text-align:center; }}
h1 {{ font-size:1.35rem; margin:0 0 .75rem; }}
p {{ margin:0 0 1.75rem; color:#606c68; }}
button {{ font:inherit; padding:.7rem 1.6rem; border:0; border-radius:999px;
  background:#b94432; color:#fff; cursor:pointer; }}
button:hover {{ filter:brightness(1.06); }}
small {{ display:block; margin-top:2rem; color:#8b8e89; }}
@media (prefers-color-scheme: dark) {{
  body {{ background:#171a18; color:#f3f5f2; }}
  p {{ color:#b5bcb6; }}
  small {{ color:#8b8e89; }}
}}
</style></head>
<body><main>
<h1>{heading}</h1>
<p>{detail}</p>
{button}
<small>HONE · 你的 AI 投资助手</small>
</main></body></html>"#
    )
}

/// The job name is user-authored and reaches an HTML document.
fn escape(raw: &str) -> String {
    raw.chars()
        .map(|character| match character {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_name_cannot_inject_markup() {
        // Job names are written by users and land inside an HTML document.
        let rendered = page("t", &escape("<img src=x onerror=alert(1)>"), "d", None);
        assert!(!rendered.contains("<img"));
        assert!(rendered.contains("&lt;img"));
    }

    #[test]
    fn the_confirmation_page_posts_and_the_result_pages_do_not() {
        // A GET that unsubscribed would fire on every link preview, so the
        // only mutation path is behind a form submit.
        let confirm = page("t", "h", "d", Some("/api/public/unsubscribe/tok"));
        assert!(confirm.contains(r#"method="post""#));
        assert!(confirm.contains("确认退订"));

        let done = page("t", "h", "d", None);
        assert!(!done.contains("<form"));
        assert!(!done.contains("确认退订"));
    }

    #[test]
    fn the_page_needs_no_javascript_or_external_asset() {
        // Chat clients open links in embedded webviews; anything fetched from
        // elsewhere may simply not arrive.
        let rendered = page("t", "h", "d", Some("/x"));
        assert!(!rendered.contains("<script"));
        assert!(!rendered.contains("http://"));
        assert!(!rendered.contains("https://"));
    }

    #[test]
    fn the_page_asks_not_to_be_indexed() {
        // These URLs carry a working capability; they must not end up in a
        // search index.
        assert!(page("t", "h", "d", None).contains(r#"name="robots" content="noindex""#));
    }
}
