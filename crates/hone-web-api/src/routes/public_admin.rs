use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone};
use serde::Deserialize;
use serde_json::json;

use hone_memory::cron_job::{CronJobExecutionRecord, ExecutionFilter};
use hone_memory::session::Session;
use hone_memory::{
    WEB_ADMIN_DAILY_INVITE_LIMIT, WebAdminInviteCreateOutcome, WebAdminInviteDisableOutcome,
    WebAdminInviteSummary, WebInviteUser,
};

use crate::state::AppState;
use crate::types::{
    PublicAdminCreateInviteRequest, PublicAdminInviteInfo, PublicAdminInviteList,
    PublicAdminInviteMutation, PublicAdminUsageQuestion, PublicAdminUsageReport,
    PublicAdminUsageRow, PublicAdminUsageSummary,
};

const ADMIN_ACTION_HEADER: &str = "x-hone-admin-action";
const ADMIN_ACTION_HEADER_VALUE: &str = "whitelist";
const DEFAULT_USAGE_REPORT_DAYS: i64 = 14;
const ALLOWED_USAGE_REPORT_DAYS: [i64; 3] = [14, 30, 90];
const USAGE_EXECUTION_LIMIT_PER_DAY: usize = 5_000;
const USAGE_QUESTION_PREVIEW_CHARS: usize = 1_000;

#[derive(Default)]
struct UsageAccumulator {
    questions: Vec<PublicAdminUsageQuestion>,
    scheduled_run_count: u32,
    delivered_push_count: u32,
    failed_delivery_count: u32,
    latest_activity_at: String,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PublicAdminUsageQuery {
    days: Option<i64>,
}

impl PublicAdminUsageQuery {
    fn report_days(&self) -> Option<i64> {
        let days = self.days.unwrap_or(DEFAULT_USAGE_REPORT_DAYS);
        ALLOWED_USAGE_REPORT_DAYS.contains(&days).then_some(days)
    }
}

pub(crate) async fn handle_list_invites(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let admin = match require_public_admin(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let invites = state_for_worker.web_auth.list_web_admin_invite_summaries();
        let created_today = state_for_worker
            .web_auth
            .web_admin_create_count_today(&admin_user_id);
        (invites, created_today)
    })
    .await;
    let (invites, created_today) = match result {
        Ok((Ok(invites), created_today)) => {
            let created_today = match created_today {
                Ok(created_today) => created_today,
                Err(error) => {
                    tracing::warn!(
                        admin_user_id = %admin.user_id,
                        error = %error,
                        "public admin daily whitelist count failed; disabling creation conservatively"
                    );
                    WEB_ADMIN_DAILY_INVITE_LIMIT
                }
            };
            (invites, created_today)
        }
        Ok((Err(error), _)) => {
            tracing::error!(
                admin_user_id = %admin.user_id,
                error = %error,
                "public admin whitelist list failed"
            );
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "会员白名单暂时无法读取，请稍后重试",
            );
        }
        Err(error) => {
            tracing::error!(
                admin_user_id = %admin.user_id,
                error = %error,
                "public admin whitelist list task failed"
            );
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "会员白名单暂时无法读取，请稍后重试",
            );
        }
    };
    tracing::info!(
        admin_user_id = %admin.user_id,
        invite_count = invites.len(),
        created_today,
        "public admin whitelist list loaded"
    );
    let response = PublicAdminInviteList {
        invites: invites
            .into_iter()
            .map(|invite| to_public_admin_summary(&admin.user_id, invite))
            .collect(),
        daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
        created_today,
        remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
    };
    Json(response).into_response()
}

pub(crate) async fn handle_create_invite(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicAdminCreateInviteRequest>,
) -> Response {
    let admin = match require_public_admin_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(phone_number) => phone_number,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        state_for_worker
            .web_auth
            .create_invite_user_by_admin(&admin_user_id, &phone_number)
    })
    .await;
    match result {
        Ok(Ok(WebAdminInviteCreateOutcome::Created { invite, used_today })) => {
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_mutation_invite(&admin.user_id, invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today: used_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(used_today),
                cleared_session_count: 0,
                message: "已加入会员白名单".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteCreateOutcome::NotAdmin)) => {
            crate::routes::json_error(StatusCode::FORBIDDEN, "当前账号没有管理权限")
        }
        Ok(Ok(WebAdminInviteCreateOutcome::LimitReached { used_today })) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "今日新增白名单已达到 5 人上限",
                "daily_create_limit": WEB_ADMIN_DAILY_INVITE_LIMIT,
                "created_today": used_today,
                "remaining_today": 0,
            })),
        )
            .into_response(),
        Ok(Ok(WebAdminInviteCreateOutcome::DuplicatePhone)) => {
            crate::routes::json_error(StatusCode::CONFLICT, "该手机号已在会员白名单中")
        }
        Ok(Err(error)) if error.to_string().contains("手机号格式不合法") => {
            crate::routes::json_error(StatusCode::BAD_REQUEST, "手机号格式不合法")
        }
        Ok(Err(error)) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("新增会员白名单失败: {error}"),
        ),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("新增会员白名单任务失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_disable_invite(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(target_user_id): Path<String>,
) -> Response {
    let admin = match require_public_admin_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        state_for_worker
            .web_auth
            .disable_invite_user_by_admin(&admin_user_id, &target_user_id)
    })
    .await;
    match result {
        Ok(Ok(WebAdminInviteDisableOutcome::Disabled(result))) => {
            let created_today = state
                .web_auth
                .web_admin_create_count_today(&admin.user_id)
                .unwrap_or(WEB_ADMIN_DAILY_INVITE_LIMIT);
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_mutation_invite(&admin.user_id, result.invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
                cleared_session_count: result.cleared_session_count,
                message: "已禁用会员白名单，并清理该用户登录态".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteDisableOutcome::AlreadyDisabled(invite))) => {
            let created_today = state
                .web_auth
                .web_admin_create_count_today(&admin.user_id)
                .unwrap_or(WEB_ADMIN_DAILY_INVITE_LIMIT);
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_mutation_invite(&admin.user_id, invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
                cleared_session_count: 0,
                message: "该用户已处于禁用状态".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteDisableOutcome::NotAdmin)) => {
            crate::routes::json_error(StatusCode::FORBIDDEN, "当前账号没有管理权限")
        }
        Ok(Ok(WebAdminInviteDisableOutcome::NotFound)) => {
            crate::routes::json_error(StatusCode::NOT_FOUND, "会员白名单用户不存在")
        }
        Ok(Ok(WebAdminInviteDisableOutcome::ProtectedAdmin)) => {
            crate::routes::json_error(StatusCode::CONFLICT, "不能禁用当前管理员或其他管理员")
        }
        Ok(Err(error)) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("禁用会员白名单失败: {error}"),
        ),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("禁用会员白名单任务失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_usage_report(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PublicAdminUsageQuery>,
    headers: HeaderMap,
) -> Response {
    let admin = match require_public_admin(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let Some(report_days) = query.report_days() else {
        return crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "统计周期仅支持 14、30 或 90 天",
        );
    };
    let now = hone_core::beijing_now();
    let period_start_date = now.date_naive() - Duration::days(report_days - 1);
    let period_start = start_of_beijing_day(period_start_date, now.offset());
    let execution_limit = usize::try_from(report_days)
        .unwrap_or_default()
        .saturating_mul(USAGE_EXECUTION_LIMIT_PER_DAY);
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = state_for_worker.core.session_storage.list_sessions()?;
        let executions = state_for_worker
            .core
            .cron_job_storage()
            .list_recent_executions(&ExecutionFilter {
                since: Some(period_start.to_rfc3339()),
                until: Some(now.to_rfc3339()),
                limit: execution_limit,
                ..ExecutionFilter::default()
            })?;
        if executions.len() >= execution_limit {
            return Err(hone_core::HoneError::Config(format!(
                "public admin usage execution query reached safety limit {execution_limit}"
            )));
        }
        let users = state_for_worker.web_auth.list_invite_users()?;
        Ok::<_, hone_core::HoneError>(build_usage_report(
            now,
            report_days,
            sessions,
            executions,
            users,
        ))
    })
    .await;

    match result {
        Ok(Ok(report)) => {
            tracing::info!(
                admin_user_id = %admin.user_id,
                report_days,
                row_count = report.rows.len(),
                today_questions = report.summary.today_question_count,
                "public admin usage report loaded"
            );
            Json(report).into_response()
        }
        Ok(Err(error)) => {
            tracing::error!(
                admin_user_id = %admin.user_id,
                error = %error,
                "public admin usage report failed"
            );
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "使用统计暂时无法读取，请稍后重试",
            )
        }
        Err(error) => {
            tracing::error!(
                admin_user_id = %admin.user_id,
                error = %error,
                "public admin usage report task failed"
            );
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "使用统计暂时无法读取，请稍后重试",
            )
        }
    }
}

fn build_usage_report(
    now: DateTime<FixedOffset>,
    report_days: i64,
    sessions: Vec<Session>,
    executions: Vec<CronJobExecutionRecord>,
    users: Vec<WebInviteUser>,
) -> PublicAdminUsageReport {
    let today = now.date_naive();
    let period_start_date = today - Duration::days(report_days - 1);
    let period_start = start_of_beijing_day(period_start_date, now.offset());
    let web_phone_numbers = users
        .into_iter()
        .map(|user| (user.user_id, user.phone_number))
        .collect::<HashMap<_, _>>();
    let mut rows = BTreeMap::<(NaiveDate, String, String), UsageAccumulator>::new();

    for session in sessions {
        let Some((channel, user_id)) = usage_session_actor(&session) else {
            continue;
        };
        if usage_user_is_automation(&user_id) {
            continue;
        }
        for message in session.messages {
            if message.role != "user" {
                continue;
            }
            let text = hone_memory::session_message_text(&message);
            if usage_message_is_automation(&text, message.metadata.as_ref()) {
                continue;
            }
            let Some(asked_at) = parse_beijing_time(&message.timestamp, now.offset()) else {
                continue;
            };
            if asked_at < period_start || asked_at > now {
                continue;
            }
            let question_text = if text.trim().is_empty() {
                "[附件或图片提问]".to_string()
            } else {
                hone_core::truncate_chars_append(text.trim(), USAGE_QUESTION_PREVIEW_CHARS, "…")
            };
            let entry = rows
                .entry((asked_at.date_naive(), channel.clone(), user_id.clone()))
                .or_default();
            entry.questions.push(PublicAdminUsageQuestion {
                asked_at: asked_at.to_rfc3339(),
                text: question_text,
            });
            update_latest_activity(&mut entry.latest_activity_at, &asked_at.to_rfc3339());
        }
    }

    for execution in executions {
        let Some(channel) = normalized_usage_channel(&execution.channel) else {
            continue;
        };
        if usage_user_is_automation(&execution.user_id) {
            continue;
        }
        let Some(executed_at) = parse_beijing_time(&execution.executed_at, now.offset()) else {
            continue;
        };
        if executed_at < period_start || executed_at > now {
            continue;
        }
        let entry = rows
            .entry((
                executed_at.date_naive(),
                channel.to_string(),
                execution.user_id,
            ))
            .or_default();
        entry.scheduled_run_count = entry.scheduled_run_count.saturating_add(1);
        if execution.delivered {
            entry.delivered_push_count = entry.delivered_push_count.saturating_add(1);
        } else if execution.should_deliver {
            entry.failed_delivery_count = entry.failed_delivery_count.saturating_add(1);
        }
        update_latest_activity(&mut entry.latest_activity_at, &executed_at.to_rfc3339());
    }

    let mut report_rows = rows
        .into_iter()
        .map(|((date, channel, user_id), mut entry)| {
            entry
                .questions
                .sort_by(|left, right| right.asked_at.cmp(&left.asked_at));
            let question_count = u32::try_from(entry.questions.len()).unwrap_or(u32::MAX);
            PublicAdminUsageRow {
                date: date.to_string(),
                user_label: usage_user_label(
                    &channel,
                    &user_id,
                    web_phone_numbers
                        .get(&user_id)
                        .map(String::as_str)
                        .unwrap_or(""),
                ),
                channel,
                user_id,
                question_count,
                questions: entry.questions,
                scheduled_run_count: entry.scheduled_run_count,
                delivered_push_count: entry.delivered_push_count,
                failed_delivery_count: entry.failed_delivery_count,
                latest_activity_at: entry.latest_activity_at,
            }
        })
        .collect::<Vec<_>>();
    report_rows.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| right.question_count.cmp(&left.question_count))
            .then_with(|| left.channel.cmp(&right.channel))
            .then_with(|| left.user_label.cmp(&right.user_label))
    });

    let summary = build_usage_summary(now, &report_rows);
    PublicAdminUsageReport {
        generated_at: now.to_rfc3339(),
        period_days: u32::try_from(report_days).unwrap_or_default(),
        period_start: period_start_date.to_string(),
        period_end: today.to_string(),
        summary,
        rows: report_rows,
    }
}

fn build_usage_summary(
    now: DateTime<FixedOffset>,
    rows: &[PublicAdminUsageRow],
) -> PublicAdminUsageSummary {
    let today = now.date_naive();
    let last_week_same_day = today - Duration::days(7);
    let today_rows = rows.iter().filter(|row| row.date == today.to_string());
    let today_question_count = today_rows
        .clone()
        .map(|row| row.question_count)
        .fold(0_u32, u32::saturating_add);
    let today_delivered_push_count = today_rows
        .clone()
        .map(|row| row.delivered_push_count)
        .fold(0_u32, u32::saturating_add);
    let today_users = today_rows
        .filter(|row| row.question_count > 0)
        .map(|row| (row.channel.as_str(), row.user_id.as_str()))
        .collect::<HashSet<_>>();
    let last_week_users = rows
        .iter()
        .filter(|row| row.date == last_week_same_day.to_string() && row.question_count > 0)
        .map(|row| (row.channel.as_str(), row.user_id.as_str()))
        .collect::<HashSet<_>>();
    let today_active_users = u32::try_from(today_users.len()).unwrap_or(u32::MAX);
    let last_week_same_day_active_users = u32::try_from(last_week_users.len()).unwrap_or(u32::MAX);
    let active_user_change = i64::from(today_active_users)
        .saturating_sub(i64::from(last_week_same_day_active_users))
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;

    let current_week_start =
        today - Duration::days(i64::from(now.weekday().num_days_from_monday()));
    let previous_week_start = current_week_start - Duration::days(7);
    let comparison_date_end = today - Duration::days(7);
    let comparison_time = now.time();
    let mut current_counts = HashMap::<(&str, &str), u32>::new();
    let mut previous_counts = HashMap::<(&str, &str), u32>::new();
    let mut labels = HashMap::<(&str, &str), &str>::new();
    for row in rows {
        let Ok(date) = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d") else {
            continue;
        };
        let actor_key = (row.channel.as_str(), row.user_id.as_str());
        labels.insert(actor_key, row.user_label.as_str());
        let include_current = date >= current_week_start
            && date <= today
            && (date < today
                || row.questions.iter().any(|question| {
                    parse_beijing_time(&question.asked_at, now.offset())
                        .is_some_and(|asked_at| asked_at.time() <= comparison_time)
                }));
        if include_current {
            let count = row
                .questions
                .iter()
                .filter(|question| {
                    parse_beijing_time(&question.asked_at, now.offset())
                        .is_some_and(|asked_at| asked_at <= now)
                })
                .count();
            let count = u32::try_from(count).unwrap_or(u32::MAX);
            current_counts
                .entry(actor_key)
                .and_modify(|value| *value = value.saturating_add(count))
                .or_insert(count);
        }
        let include_previous = date >= previous_week_start && date <= comparison_date_end;
        if include_previous {
            let previous_end = now - Duration::days(7);
            let count = row
                .questions
                .iter()
                .filter(|question| {
                    parse_beijing_time(&question.asked_at, now.offset())
                        .is_some_and(|asked_at| asked_at <= previous_end)
                })
                .count();
            let count = u32::try_from(count).unwrap_or(u32::MAX);
            previous_counts
                .entry(actor_key)
                .and_modify(|value| *value = value.saturating_add(count))
                .or_insert(count);
        }
    }
    let leading_decline = previous_counts
        .iter()
        .filter_map(|(actor_key, previous)| {
            let current = current_counts.get(actor_key).copied().unwrap_or_default();
            previous
                .checked_sub(current)
                .filter(|drop| *drop > 0)
                .map(|drop| (*actor_key, drop))
        })
        .max_by(|(left_actor, left_drop), (right_actor, right_drop)| {
            left_drop
                .cmp(right_drop)
                .then_with(|| right_actor.cmp(left_actor))
        });
    let (leading_decline_user_label, leading_decline_question_delta) = leading_decline
        .map(|(actor_key, drop)| {
            (
                Some(
                    labels
                        .get(&actor_key)
                        .copied()
                        .unwrap_or(actor_key.1)
                        .to_string(),
                ),
                drop,
            )
        })
        .unwrap_or((None, 0));
    let comparison = match active_user_change.cmp(&0) {
        std::cmp::Ordering::Less => {
            format!("比上周同日少 {} 人", active_user_change.unsigned_abs())
        }
        std::cmp::Ordering::Greater => format!("比上周同日多 {active_user_change} 人"),
        std::cmp::Ordering::Equal => "与上周同日持平".to_string(),
    };
    let decline = leading_decline_user_label
        .as_ref()
        .map(|label| {
            format!(
                "主要是 {label} 本周使用频率降低（较上周同期少 {leading_decline_question_delta} 次）"
            )
        })
        .unwrap_or_else(|| "本周暂无明显降频用户".to_string());
    let text = format!(
        "今日 HONE 总使用人数 {today_active_users} 人，提问问题总共 {today_question_count} 个，定时任务成功推送 {today_delivered_push_count} 条，{comparison}；{decline}。"
    );

    PublicAdminUsageSummary {
        today: today.to_string(),
        today_active_users,
        today_question_count,
        today_delivered_push_count,
        last_week_same_day_active_users,
        active_user_change,
        leading_decline_user_label,
        leading_decline_question_delta,
        text,
    }
}

fn usage_session_actor(session: &Session) -> Option<(String, String)> {
    if let Some(actor) = session
        .actor
        .as_ref()
        .filter(|actor| !actor.user_id.trim().is_empty())
    {
        let channel = normalized_usage_channel(&actor.channel)?;
        return Some((channel.to_string(), actor.user_id.clone()));
    }
    session.session_identity.as_ref().and_then(|identity| {
        let user_id = identity.user_id.as_ref()?;
        let channel = normalized_usage_channel(&identity.channel)?;
        (!user_id.trim().is_empty()).then(|| (channel.to_string(), user_id.clone()))
    })
}

fn normalized_usage_channel(channel: &str) -> Option<&'static str> {
    match channel.trim().to_ascii_lowercase().as_str() {
        "web" => Some("web"),
        "feishu" => Some("feishu"),
        "telegram" => Some("telegram"),
        "discord" => Some("discord"),
        "imessage" => Some("imessage"),
        _ => None,
    }
}

fn usage_message_is_automation(
    content: &str,
    metadata: Option<&HashMap<String, serde_json::Value>>,
) -> bool {
    let tagged_source = metadata
        .and_then(|items| items.get("source"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|source| {
            matches!(
                source.trim().to_ascii_lowercase().as_str(),
                "scheduler" | "heartbeat"
            )
        });
    let tagged_job = metadata
        .is_some_and(|items| items.contains_key("job_id") || items.contains_key("web_push_id"));
    tagged_source || tagged_job || content.trim_start().starts_with("[定时任务触发]")
}

fn usage_user_is_automation(user_id: &str) -> bool {
    user_id.trim().to_ascii_lowercase().starts_with("codex")
}

fn usage_user_label(channel: &str, user_id: &str, phone_number: &str) -> String {
    let phone = phone_number.trim();
    if channel == "web"
        && phone.len() == 11
        && phone.chars().all(|character| character.is_ascii_digit())
    {
        return format!("{}****{}", &phone[..3], &phone[7..]);
    }
    let normalized = user_id.trim();
    if let Some(suffix) = normalized.strip_prefix("web-user-") {
        let tail = suffix
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        return format!("HONE 用户 {tail}");
    }
    let channel_label = match channel {
        "web" => "网页",
        "feishu" => "飞书",
        "telegram" => "Telegram",
        "discord" => "Discord",
        "imessage" => "iMessage",
        _ => channel,
    };
    let tail = normalized
        .chars()
        .rev()
        .take(6)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{channel_label}用户 {tail}")
}

fn parse_beijing_time(value: &str, offset: &FixedOffset) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(offset))
}

fn start_of_beijing_day(date: NaiveDate, offset: &FixedOffset) -> DateTime<FixedOffset> {
    offset
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("valid start of day"))
        .single()
        .expect("fixed offset has one local datetime")
}

fn update_latest_activity(current: &mut String, candidate: &str) {
    if current.is_empty() || candidate > current.as_str() {
        *current = candidate.to_string();
    }
}

/// Read-only admin gate for other public modules. The mutation variant adds a
/// header marker on top of this; a report endpoint does not need it.
pub(crate) fn require_public_admin_for_read(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<WebInviteUser, Response> {
    require_public_admin(state, headers)
}

/// Shared mutation gate for administrator-owned public modules. Mutations must
/// carry the same explicit browser action marker as invite administration.
pub(crate) fn require_public_admin_for_mutation(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<WebInviteUser, Response> {
    require_public_admin_mutation(state, headers)
}

fn require_public_admin(state: &AppState, headers: &HeaderMap) -> Result<WebInviteUser, Response> {
    let user = crate::routes::public::require_public_session_user(state, headers)?;
    match state.web_auth.is_web_admin(&user.user_id) {
        Ok(true) => Ok(user),
        Ok(false) => Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "当前账号没有管理权限",
        )),
        Err(error) => Err(crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取管理员权限失败: {error}"),
        )),
    }
}

fn require_public_admin_mutation(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<WebInviteUser, Response> {
    let user = require_public_admin(state, headers)?;
    let marker = headers
        .get(ADMIN_ACTION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim);
    if marker != Some(ADMIN_ACTION_HEADER_VALUE) {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "管理操作校验失败，请刷新后重试",
        ));
    }
    Ok(user)
}

fn to_public_admin_summary(
    admin_user_id: &str,
    invite: WebAdminInviteSummary,
) -> PublicAdminInviteInfo {
    let enabled = invite.revoked_at.is_none();
    PublicAdminInviteInfo {
        can_disable: enabled && invite.user_id != admin_user_id && !invite.is_admin,
        user_id: invite.user_id,
        phone_number: invite.phone_number,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        enabled,
    }
}

fn to_public_admin_mutation_invite(
    admin_user_id: &str,
    invite: WebInviteUser,
) -> PublicAdminInviteInfo {
    let enabled = invite.revoked_at.is_none();
    PublicAdminInviteInfo {
        can_disable: enabled && invite.user_id != admin_user_id,
        user_id: invite.user_id,
        phone_number: invite.phone_number,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ADMIN_ACTION_HEADER, ADMIN_ACTION_HEADER_VALUE, DEFAULT_USAGE_REPORT_DAYS,
        PublicAdminUsageQuery, build_usage_report, to_public_admin_mutation_invite,
        to_public_admin_summary, usage_user_is_automation,
    };
    use axum::http::{HeaderMap, HeaderValue};
    use chrono::DateTime;
    use hone_core::{ActorIdentity, SessionIdentity};
    use hone_memory::cron_job::CronJobExecutionRecord;
    use hone_memory::session::{Session, SessionRuntimeState};
    use hone_memory::session_message_from_text;
    use hone_memory::{WebAdminInviteSummary, WebInviteUser};
    use serde_json::json;
    use std::collections::HashMap;

    fn invite(user_id: &str, revoked: bool) -> WebInviteUser {
        WebInviteUser {
            user_id: user_id.to_string(),
            invite_code: "HONE-TEST".to_string(),
            phone_number: "13800138000".to_string(),
            created_at: "2026-07-31T00:00:00+08:00".to_string(),
            last_login_at: None,
            revoked_at: revoked.then(|| "2026-07-31T01:00:00+08:00".to_string()),
            password_hash: None,
            password_set_at: None,
            tos_accepted_at: None,
            tos_version: None,
            api_key_prefix: None,
            api_key_created_at: None,
            api_key_last_used_at: None,
            api_key_plaintext: None,
        }
    }

    fn summary(user_id: &str, revoked: bool, is_admin: bool) -> WebAdminInviteSummary {
        WebAdminInviteSummary {
            user_id: user_id.to_string(),
            phone_number: "13800138000".to_string(),
            created_at: "2026-07-31T00:00:00+08:00".to_string(),
            last_login_at: None,
            revoked_at: revoked.then(|| "2026-07-31T01:00:00+08:00".to_string()),
            is_admin,
        }
    }

    fn usage_session(
        user_id: &str,
        messages: Vec<hone_memory::session::SessionMessage>,
    ) -> Session {
        usage_session_for("web", user_id, None, messages)
    }

    fn usage_session_for(
        channel: &str,
        user_id: &str,
        channel_scope: Option<&str>,
        messages: Vec<hone_memory::session::SessionMessage>,
    ) -> Session {
        let actor = ActorIdentity::new(channel, user_id, channel_scope).expect("actor");
        Session {
            version: 4,
            id: actor.session_id(),
            session_identity: Some(SessionIdentity::from_actor(&actor).expect("identity")),
            actor: Some(actor),
            created_at: "2026-07-20T00:00:00+08:00".to_string(),
            updated_at: "2026-08-02T12:00:00+08:00".to_string(),
            messages,
            metadata: HashMap::new(),
            runtime: SessionRuntimeState::default(),
            summary: None,
        }
    }

    fn usage_execution(
        user_id: &str,
        executed_at: &str,
        should_deliver: bool,
        delivered: bool,
    ) -> CronJobExecutionRecord {
        usage_execution_for("web", user_id, None, executed_at, should_deliver, delivered)
    }

    fn usage_execution_for(
        channel: &str,
        user_id: &str,
        channel_scope: Option<&str>,
        executed_at: &str,
        should_deliver: bool,
        delivered: bool,
    ) -> CronJobExecutionRecord {
        CronJobExecutionRecord {
            run_id: 1,
            job_id: "job-1".to_string(),
            job_name: "每日复盘".to_string(),
            channel: channel.to_string(),
            user_id: user_id.to_string(),
            channel_scope: channel_scope.map(str::to_string),
            channel_target: user_id.to_string(),
            heartbeat: false,
            executed_at: executed_at.to_string(),
            execution_status: "completed".to_string(),
            message_send_status: if delivered { "sent" } else { "send_failed" }.to_string(),
            should_deliver,
            delivered,
            response_preview: None,
            error_message: None,
            detail: json!({}),
        }
    }

    #[test]
    fn public_projection_never_exposes_invite_or_api_credentials() {
        let value = serde_json::to_value(to_public_admin_mutation_invite(
            "admin",
            invite("member", false),
        ))
        .expect("serialize");
        assert!(value.get("invite_code").is_none());
        assert!(value.get("api_key").is_none());
        assert!(value.get("password_hash").is_none());
        assert_eq!(value["can_disable"], true);
    }

    #[test]
    fn public_projection_protects_self_and_disabled_rows() {
        assert!(!to_public_admin_mutation_invite("admin", invite("admin", false)).can_disable);
        assert!(!to_public_admin_mutation_invite("admin", invite("member", true)).can_disable);
    }

    #[test]
    fn list_projection_protects_all_admins() {
        assert!(!to_public_admin_summary("admin", summary("admin", false, true)).can_disable);
        assert!(!to_public_admin_summary("admin", summary("other-admin", false, true)).can_disable);
        assert!(to_public_admin_summary("admin", summary("member", false, false)).can_disable);
    }

    #[test]
    fn mutation_marker_requires_exact_custom_header() {
        let mut headers = HeaderMap::new();
        assert!(headers.get(ADMIN_ACTION_HEADER).is_none());
        headers.insert(
            ADMIN_ACTION_HEADER,
            HeaderValue::from_static(ADMIN_ACTION_HEADER_VALUE),
        );
        assert_eq!(
            headers
                .get(ADMIN_ACTION_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some(ADMIN_ACTION_HEADER_VALUE)
        );
    }

    #[test]
    fn usage_report_counts_real_questions_in_beijing_and_excludes_automation() {
        let now = DateTime::parse_from_rfc3339("2026-08-02T12:00:00+08:00").expect("now");
        let mut scheduler_metadata = HashMap::new();
        scheduler_metadata.insert("source".to_string(), json!(" Scheduler "));
        let sessions = vec![usage_session(
            "web-user-alpha1234",
            vec![
                session_message_from_text("user", "凌晨的问题", "2026-08-01T16:30:00Z", None),
                session_message_from_text(
                    "user",
                    "[定时任务触发] 每日复盘",
                    "2026-08-02T08:00:00+08:00",
                    Some(scheduler_metadata),
                ),
                session_message_from_text(
                    "user",
                    "今天还能买吗？",
                    "2026-08-02T10:00:00+08:00",
                    None,
                ),
            ],
        )];
        let report = build_usage_report(
            now,
            DEFAULT_USAGE_REPORT_DAYS,
            sessions,
            vec![
                usage_execution(
                    "web-user-alpha1234",
                    "2026-08-02T09:00:00+08:00",
                    true,
                    true,
                ),
                usage_execution(
                    "web-user-alpha1234",
                    "2026-08-02T11:00:00+08:00",
                    true,
                    false,
                ),
            ],
            vec![invite("web-user-alpha1234", false)],
        );

        assert_eq!(report.summary.today_active_users, 1);
        assert_eq!(report.summary.today_question_count, 2);
        assert_eq!(report.summary.today_delivered_push_count, 1);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].channel, "web");
        assert_eq!(report.rows[0].question_count, 2);
        assert_eq!(report.rows[0].scheduled_run_count, 2);
        assert_eq!(report.rows[0].failed_delivery_count, 1);
        assert!(
            report.rows[0]
                .questions
                .iter()
                .all(|question| !question.text.contains("定时任务触发"))
        );
        assert_eq!(report.rows[0].user_label, "138****8000");
    }

    #[test]
    fn usage_report_excludes_codex_prefixed_automation_users_and_executions() {
        let now = DateTime::parse_from_rfc3339("2026-08-02T12:00:00+08:00").expect("now");
        let sessions = vec![
            usage_session(
                "Codex-nightly-01",
                vec![session_message_from_text(
                    "user",
                    "自动化提问",
                    "2026-08-02T10:00:00+08:00",
                    None,
                )],
            ),
            usage_session(
                "web-user-real1234",
                vec![session_message_from_text(
                    "user",
                    "真实用户提问",
                    "2026-08-02T10:30:00+08:00",
                    None,
                )],
            ),
        ];
        let report = build_usage_report(
            now,
            DEFAULT_USAGE_REPORT_DAYS,
            sessions,
            vec![
                usage_execution(
                    " codex-push-worker ",
                    "2026-08-02T11:00:00+08:00",
                    true,
                    true,
                ),
                usage_execution("web-user-real1234", "2026-08-02T11:30:00+08:00", true, true),
            ],
            vec![invite("web-user-real1234", false)],
        );

        assert!(usage_user_is_automation("codex-worker"));
        assert!(usage_user_is_automation("  CODEX-worker  "));
        assert!(!usage_user_is_automation("web-user-codex-worker"));
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].user_id, "web-user-real1234");
        assert_eq!(report.rows[0].question_count, 1);
        assert_eq!(report.rows[0].delivered_push_count, 1);
        assert_eq!(report.summary.today_active_users, 1);
    }

    #[test]
    fn usage_report_includes_supported_channels_and_keeps_actor_namespaces_separate() {
        let now = DateTime::parse_from_rfc3339("2026-08-02T12:00:00+08:00").expect("now");
        let question = |text: &str| {
            vec![session_message_from_text(
                "user",
                text,
                "2026-08-02T10:00:00+08:00",
                None,
            )]
        };
        let report = build_usage_report(
            now,
            DEFAULT_USAGE_REPORT_DAYS,
            vec![
                usage_session_for("web", "shared-user", None, question("网页问题")),
                usage_session_for("feishu", "shared-user", None, question("飞书问题")),
                usage_session_for(
                    "discord",
                    "discord-user",
                    Some("guild:1:channel:2"),
                    question("Discord 群问题"),
                ),
                usage_session_for("sms", "sms-user", None, question("不支持渠道问题")),
            ],
            vec![
                usage_execution_for(
                    "feishu",
                    "shared-user",
                    None,
                    "2026-08-02T11:00:00+08:00",
                    true,
                    true,
                ),
                usage_execution_for(
                    "discord",
                    "discord-user",
                    Some("guild:1:channel:2"),
                    "2026-08-02T11:30:00+08:00",
                    true,
                    true,
                ),
                usage_execution_for(
                    "sms",
                    "sms-user",
                    None,
                    "2026-08-02T11:30:00+08:00",
                    true,
                    true,
                ),
            ],
            Vec::new(),
        );

        assert_eq!(report.rows.len(), 3);
        assert_eq!(report.summary.today_active_users, 3);
        assert_eq!(report.summary.today_question_count, 3);
        assert_eq!(report.summary.today_delivered_push_count, 2);
        assert!(report.rows.iter().any(|row| {
            row.channel == "feishu"
                && row.user_id == "shared-user"
                && row.user_label.starts_with("飞书用户 ")
                && row.delivered_push_count == 1
        }));
        assert!(report.rows.iter().any(|row| {
            row.channel == "discord"
                && row.user_id == "discord-user"
                && row.user_label.starts_with("Discord用户 ")
                && row.delivered_push_count == 1
        }));
        assert!(report.rows.iter().all(|row| row.channel != "sms"));
    }

    #[test]
    fn usage_summary_compares_same_day_and_names_the_largest_weekly_decline() {
        let now = DateTime::parse_from_rfc3339("2026-08-02T12:00:00+08:00").expect("now");
        let alpha = usage_session(
            "web-user-alpha1234",
            vec![
                session_message_from_text("user", "上周问题一", "2026-07-21T09:00:00+08:00", None),
                session_message_from_text("user", "上周问题二", "2026-07-22T09:00:00+08:00", None),
                session_message_from_text("user", "上周问题三", "2026-07-26T09:00:00+08:00", None),
                session_message_from_text("user", "本周问题", "2026-08-02T09:00:00+08:00", None),
            ],
        );
        let beta = usage_session(
            "web-user-beta5678",
            vec![session_message_from_text(
                "user",
                "上周同日问题",
                "2026-07-26T10:00:00+08:00",
                None,
            )],
        );
        let mut alpha_invite = invite("web-user-alpha1234", false);
        alpha_invite.phone_number = "13871396421".to_string();
        let report = build_usage_report(
            now,
            DEFAULT_USAGE_REPORT_DAYS,
            vec![alpha, beta],
            Vec::new(),
            vec![alpha_invite, invite("web-user-beta5678", false)],
        );

        assert_eq!(report.summary.today_active_users, 1);
        assert_eq!(report.summary.last_week_same_day_active_users, 2);
        assert_eq!(report.summary.active_user_change, -1);
        assert_eq!(
            report.summary.leading_decline_user_label.as_deref(),
            Some("138****6421")
        );
        assert_eq!(report.summary.leading_decline_question_delta, 2);
        assert!(report.summary.text.contains("比上周同日少 1 人"));
        assert!(report.summary.text.contains("138****6421 本周使用频率降低"));
    }

    #[test]
    fn usage_query_accepts_only_supported_report_ranges() {
        assert_eq!(PublicAdminUsageQuery::default().report_days(), Some(14));
        assert_eq!(
            PublicAdminUsageQuery { days: Some(30) }.report_days(),
            Some(30)
        );
        assert_eq!(
            PublicAdminUsageQuery { days: Some(90) }.report_days(),
            Some(90)
        );
        assert_eq!(PublicAdminUsageQuery { days: Some(7) }.report_days(), None);
    }

    #[test]
    fn usage_report_uses_requested_period_instead_of_a_fixed_two_weeks() {
        let now = DateTime::parse_from_rfc3339("2026-08-02T12:00:00+08:00").expect("now");
        let report = build_usage_report(
            now,
            30,
            vec![usage_session(
                "web-user-history",
                vec![session_message_from_text(
                    "user",
                    "三周前的问题",
                    "2026-07-12T10:00:00+08:00",
                    None,
                )],
            )],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(report.period_days, 30);
        assert_eq!(report.period_start, "2026-07-04");
        assert_eq!(report.period_end, "2026-08-02");
        assert_eq!(report.rows.len(), 1);
    }
}
