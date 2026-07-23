use std::path::{Path, PathBuf};

use hone_core::ActorIdentity;
use hone_core::agent::{AgentResponse, ToolCallMade};
use hone_core::cloud_runtime::{CloudCompanyProfileFileRecord, CloudPgRuntime};
use serde_json::Value;

use crate::HoneBotCore;
use crate::outbound::{ResponseContentSegment, split_response_content_segments};
use crate::runtime::{
    is_transitional_planning_sentence, sanitize_agent_owned_user_visible_output,
    sanitize_user_visible_output,
};
use crate::sandbox::sandbox_base_dir;
use crate::tool_trace::{
    PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE, PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE,
    UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE, response_has_only_known_read_only_calls,
};

pub(crate) const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";
const MISSING_LOCAL_IMAGE_FALLBACK_MESSAGE: &str = "（图表文件不可用，请重新生成）";
const CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE: &str = "定时任务管理暂时不可用，请稍后再试。";

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FinalizeResponseOutcome {
    pub(crate) fallback_reason: Option<&'static str>,
}

pub(crate) fn finalize_agent_response(
    core: &HoneBotCore,
    session_id: &str,
    runner_name: &str,
    response: &mut AgentResponse,
) -> FinalizeResponseOutcome {
    let mut outcome = FinalizeResponseOutcome::default();
    if !response.success {
        return outcome;
    }

    if response_leaks_system_prompt(&response.content) {
        tracing::error!(
            "[AgentSession] blocked echoed system prompt runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned leaked system instructions".to_string());
        response.content.clear();
        return outcome;
    }

    let sanitized = sanitize_user_visible_output(&response.content);
    if sanitized.only_internal {
        tracing::error!(
            "[AgentSession] blocked internal-only assistant output runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned internal-only output".to_string());
        response.content.clear();
        return outcome;
    }

    if sanitized.content.trim().is_empty() {
        if let Some(recovered) = recover_user_facing_tool_outcome(&response.tool_calls_made) {
            tracing::info!(
                "[AgentSession] recovered user-facing tool outcome from sanitized-empty output runner={} session_id={}",
                runner_name,
                session_id
            );
            response.content = recovered;
            response.error = None;
            sync_company_profiles_to_cloud(core, session_id);
            response.content =
                normalize_local_image_references(core, session_id, &response.content);
            return outcome;
        }
        tracing::warn!(
            "[AgentSession] empty visible output after sanitization runner={} session_id={} removed_internal={}",
            runner_name,
            session_id,
            sanitized.removed_internal
        );
        response.success = false;
        response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
        response.error = Some(EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string());
        outcome.fallback_reason = Some("sanitized_empty_success");
    } else if is_transitional_planning_sentence(sanitized.content.trim()) {
        if let Some(recovered) = recover_user_facing_tool_outcome(&response.tool_calls_made) {
            tracing::info!(
                "[AgentSession] recovered user-facing tool outcome from planning sentence runner={} session_id={}",
                runner_name,
                session_id
            );
            response.content = recovered;
            response.error = None;
            return outcome;
        }
        tracing::warn!(
            "[AgentSession] transitional planning sentence detected, treating as empty runner={} session_id={} chars={}",
            runner_name,
            session_id,
            sanitized.content.trim().chars().count()
        );
        response.success = false;
        response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
        response.error = Some(EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string());
        outcome.fallback_reason = Some("planning_sentence_suppressed");
    } else {
        if sanitized.content.trim() == CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE
            && let Some(recovered) = recover_user_facing_tool_outcome(&response.tool_calls_made)
        {
            tracing::info!(
                "[AgentSession] recovered user-facing tool outcome from generic cron unavailable copy runner={} session_id={}",
                runner_name,
                session_id
            );
            response.content = recovered;
            response.error = None;
            sync_company_profiles_to_cloud(core, session_id);
            response.content =
                normalize_local_image_references(core, session_id, &response.content);
            return outcome;
        }
        response.content = sanitized.content;
    }

    sync_company_profiles_to_cloud(core, session_id);
    response.content = normalize_local_image_references(core, session_id, &response.content);
    outcome
}

pub(crate) fn recover_failed_read_only_user_visible_output(
    response: &AgentResponse,
) -> Option<String> {
    if response.success
        || response.tool_calls_made.is_empty()
        || !response_has_only_known_read_only_calls(&response.tool_calls_made)
    {
        return None;
    }

    let sanitized = sanitize_user_visible_output(&response.content);
    if sanitized.only_internal {
        return None;
    }
    let content = sanitized.content.trim().to_string();
    if content.is_empty()
        || is_transitional_planning_sentence(&content)
        || looks_like_failed_response_fallback_copy(&content)
    {
        return None;
    }
    Some(content)
}

/// Finalize a completed Interactive Agent body without applying legacy
/// business-copy normalization, planning-sentence vetoes, or tool-result
/// reconstruction. The Agent owns the answer; this boundary is limited to
/// system-prompt/protocol/path safety and local media stabilization.
pub(crate) fn finalize_agent_owned_interactive_response(
    core: &HoneBotCore,
    session_id: &str,
    runner_name: &str,
    response: &mut AgentResponse,
) -> FinalizeResponseOutcome {
    let outcome = FinalizeResponseOutcome::default();
    if !response.success {
        return outcome;
    }

    if response_leaks_system_prompt(&response.content) {
        tracing::error!(
            "[AgentSession] blocked echoed system prompt runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned leaked system instructions".to_string());
        response.content.clear();
        return outcome;
    }

    let sanitized = sanitize_agent_owned_user_visible_output(&response.content);
    if sanitized.only_internal {
        tracing::error!(
            "[AgentSession] blocked internal-only Interactive Agent output runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned internal-only output".to_string());
        response.content.clear();
        return outcome;
    }
    if sanitized.content.trim().is_empty() {
        tracing::warn!(
            "[AgentSession] empty Interactive Agent output after security cleanup runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned empty user-visible output".to_string());
        response.content.clear();
        return outcome;
    }

    response.content = sanitized.content;
    sync_company_profiles_to_cloud(core, session_id);
    response.content = normalize_local_image_references(core, session_id, &response.content);
    outcome
}

fn sync_company_profiles_to_cloud(core: &HoneBotCore, session_id: &str) {
    if !core.config.cloud.effective_mode().is_cloud_authoritative()
        || !core.config.cloud.postgres.is_configured()
    {
        return;
    }
    let Some(actor) = ActorIdentity::from_session_id(session_id) else {
        return;
    };
    let root = sandbox_base_dir()
        .join(actor.channel_fs_component())
        .join(actor.scoped_user_fs_key())
        .join("company_profiles");
    if !root.is_dir() {
        return;
    }
    let Some(postgres) = CloudPgRuntime::from_cloud_config(&core.config.cloud) else {
        return;
    };
    let Ok(actor_value) = serde_json::to_value(&actor) else {
        return;
    };
    let mut records = Vec::new();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };
    for entry in entries.flatten() {
        let profile_dir = entry.path();
        if !profile_dir.is_dir() {
            continue;
        }
        let Some(profile_id) = profile_dir
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        push_company_profile_file_record(
            &mut records,
            &actor,
            &actor_value,
            &profile_id,
            "profile.md",
            &profile_dir.join("profile.md"),
        );
        let events_dir = profile_dir.join("events");
        let Ok(event_entries) = std::fs::read_dir(&events_dir) else {
            continue;
        };
        for event_entry in event_entries.flatten() {
            let path = event_entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            push_company_profile_file_record(
                &mut records,
                &actor,
                &actor_value,
                &profile_id,
                &format!("events/{filename}"),
                &path,
            );
        }
    }
    if records.is_empty() {
        return;
    }
    let import_result = run_cloud_company_profile_sync(async move {
        postgres.import_company_profile_files(&records).await
    });
    if let Err(error) = import_result {
        tracing::warn!(session_id, "cloud company profile sync failed: {error}");
    }
}

fn push_company_profile_file_record(
    records: &mut Vec<CloudCompanyProfileFileRecord>,
    actor: &ActorIdentity,
    actor_value: &serde_json::Value,
    profile_id: &str,
    relative_path: &str,
    path: &Path,
) {
    if !path.is_file() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let updated_at = path
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(system_time_to_rfc3339)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    records.push(CloudCompanyProfileFileRecord {
        actor_storage_key: actor.storage_key(),
        actor: actor_value.clone(),
        profile_id: profile_id.to_string(),
        relative_path: relative_path.to_string(),
        content,
        updated_at,
    });
}

fn run_cloud_company_profile_sync<T, F>(future: F) -> Result<T, String>
where
    T: Send + 'static,
    F: std::future::Future<Output = hone_core::HoneResult<T>> + Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().map_err(|err| err.to_string())?;
            runtime.block_on(future).map_err(|err| err.to_string())
        })
        .join()
        .map_err(|_| "cloud company profile sync worker panicked".to_string())?;
    }
    let runtime = tokio::runtime::Runtime::new().map_err(|err| err.to_string())?;
    runtime.block_on(future).map_err(|err| err.to_string())
}

fn system_time_to_rfc3339(value: std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = value.into();
    datetime.to_rfc3339()
}

fn recover_user_facing_tool_outcome(tool_calls: &[ToolCallMade]) -> Option<String> {
    tool_calls
        .iter()
        .rev()
        .find_map(|call| match call.name.as_str() {
            "cron_job" => recover_cron_job_confirmation(call),
            "portfolio" => recover_portfolio_confirmation(call),
            _ => None,
        })
}

fn looks_like_failed_response_fallback_copy(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == EMPTY_SUCCESS_FALLBACK_MESSAGE
        || trimmed == CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE
        || trimmed == PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE
        || trimmed == PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE
        || trimmed == UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE
        || trimmed == "抱歉，这次处理失败了。请稍后再试。"
        || trimmed == "抱歉，处理超时了。请稍后再试。"
        || trimmed == "当前执行额度已用尽，暂时无法继续处理。请稍后再试。"
        || trimmed.contains("当前状态无法确定")
        || trimmed.contains("没有自动重试")
        || trimmed.contains("系统已记录失败并将在下一次触发时重试")
}

fn recover_cron_job_confirmation(call: &ToolCallMade) -> Option<String> {
    let action = tool_action(call);
    match action.as_deref() {
        Some("list") => recover_cron_job_list_confirmation(call),
        Some("add") => cron_job_confirmation_message("已创建定时任务", call),
        Some("update") => cron_job_confirmation_message("已更新定时任务", call),
        Some("remove") if call.result.get("needs_confirmation").is_some() => {
            recover_cron_job_remove_confirmation(call)
        }
        Some("remove") => call
            .result
            .get("removed_job_id")
            .and_then(|value| value.as_str())
            .map(|job_id| format!("已删除定时任务：{job_id}。")),
        _ => None,
    }
}

fn recover_cron_job_list_confirmation(call: &ToolCallMade) -> Option<String> {
    let jobs = call.result.get("jobs")?.as_array()?;
    if jobs.is_empty() {
        return Some("你当前没有定时任务。".to_string());
    }

    let shown: Vec<String> = jobs
        .iter()
        .take(5)
        .filter_map(format_cron_job_list_entry)
        .collect();
    if shown.is_empty() {
        return None;
    }

    let mut message = format!("你当前有 {} 个定时任务：{}", jobs.len(), shown.join("；"));
    if jobs.len() > shown.len() {
        message.push_str(&format!("；另有 {} 个任务", jobs.len() - shown.len()));
    }
    message.push('。');
    Some(message)
}

fn recover_cron_job_remove_confirmation(call: &ToolCallMade) -> Option<String> {
    if let Some(job) = call.result.get("job") {
        let entry = format_cron_job_list_entry(job)?;
        return Some(format!(
            "删除前需要你确认：{}。如果确认删除，请明确回复要删除这个任务。",
            entry
        ));
    }

    let candidates = call.result.get("candidates")?.as_array()?;
    let shown: Vec<String> = candidates
        .iter()
        .take(5)
        .filter_map(format_cron_job_list_entry)
        .collect();
    if shown.is_empty() {
        return None;
    }

    let mut message = format!(
        "匹配到多个定时任务：{}。请指定要删除的任务 ID。",
        shown.join("；")
    );
    if candidates.len() > shown.len() {
        message.push_str(&format!(
            " 另有 {} 个候选任务。",
            candidates.len() - shown.len()
        ));
    }
    Some(message)
}

fn format_cron_job_list_entry(job: &Value) -> Option<String> {
    let name = job.get("name").and_then(Value::as_str)?.trim();
    if name.is_empty() {
        return None;
    }
    let schedule = job
        .get("schedule")
        .map(format_cron_schedule)
        .filter(|value| !value.trim().is_empty());
    let job_id = job
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let enabled = job.get("enabled").and_then(Value::as_bool);

    let mut entry = name.to_string();
    if let Some(schedule) = schedule {
        entry.push_str("（");
        entry.push_str(&schedule);
        if matches!(enabled, Some(false)) {
            entry.push_str("，已停用");
        }
        entry.push('）');
    } else if matches!(enabled, Some(false)) {
        entry.push_str("（已停用）");
    }
    if let Some(job_id) = job_id {
        entry.push_str("，任务 ID：");
        entry.push_str(job_id);
    }
    Some(entry)
}

fn cron_job_confirmation_message(prefix: &str, call: &ToolCallMade) -> Option<String> {
    let job = call.result.get("job")?;
    let name = job
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("未命名任务");
    let job_id = job.get("id").and_then(|value| value.as_str()).unwrap_or("");
    let schedule = job.get("schedule").map(format_cron_schedule);
    let mut message = format!("{prefix}：{name}");
    if let Some(schedule) = schedule.filter(|value| !value.is_empty()) {
        message.push_str("（");
        message.push_str(&schedule);
        message.push('）');
    }
    if !job_id.is_empty() {
        message.push_str("。任务 ID：");
        message.push_str(job_id);
    }
    message.push('。');
    Some(message)
}

fn recover_portfolio_confirmation(call: &ToolCallMade) -> Option<String> {
    let action = tool_action(call)?;
    if action == "view" {
        return recover_portfolio_view_confirmation(call);
    }

    let tickers = portfolio_tickers(call);
    if tickers.is_empty() {
        return None;
    }
    let label = tickers.join("、");
    let message = match action.as_str() {
        "add" => {
            let mut message = format!("已记录持仓：{label}");
            if let Some(cost_basis) = portfolio_cost_basis(call) {
                message.push_str("，成本价 ");
                message.push_str(&cost_basis);
            }
            message.push_str("。后续跟踪会优先参考这条持仓记录。");
            message
        }
        "update" => format!("已更新持仓：{label}。后续跟踪会使用最新持仓记录。"),
        "remove" => format!("已处理持仓/关注删除请求：{label}。"),
        "watch" => match portfolio_first_result(call).as_deref() {
            Some("already_holding") => format!("{label} 已在持仓中，会继续按持仓跟踪。"),
            Some("already_watching") => format!("{label} 已在关注列表中，会继续跟踪。"),
            _ => format!("已加入关注列表：{label}。后续会继续跟踪。"),
        },
        "unwatch" => format!("已取消关注：{label}。"),
        _ => return None,
    };
    Some(message)
}

fn recover_portfolio_view_confirmation(call: &ToolCallMade) -> Option<String> {
    let portfolio = call.result.get("portfolio")?;
    let holdings = portfolio_array(portfolio, "holdings");
    let watchlist = portfolio_array(portfolio, "watchlist");
    let mut entries: Vec<&Value> = holdings.iter().chain(watchlist.iter()).copied().collect();

    let requested_tickers = portfolio_argument_tickers(call);
    if !requested_tickers.is_empty() {
        entries.retain(|holding| {
            portfolio_ticker(holding)
                .map(|ticker| {
                    requested_tickers
                        .iter()
                        .any(|requested| requested == &ticker)
                })
                .unwrap_or(false)
        });
    }

    if entries.is_empty() {
        if portfolio
            .get("message")
            .and_then(|value| value.as_str())
            .map(|value| value.contains("暂无持仓"))
            .unwrap_or(false)
        {
            return Some("当前还没有记录持仓或关注标的。".to_string());
        }
        return None;
    }

    let total = entries.len();
    let shown: Vec<String> = entries
        .iter()
        .take(3)
        .filter_map(|holding| format_portfolio_view_entry(holding))
        .collect();
    if shown.is_empty() {
        return None;
    }

    let prefix = if requested_tickers.is_empty() {
        "已读取当前持仓/关注记录"
    } else {
        "已读取相关持仓记录"
    };
    let mut message = format!("{prefix}：{}", shown.join("；"));
    if total > shown.len() {
        message.push_str(&format!("；另有 {} 个标的", total - shown.len()));
    }
    message.push('。');
    Some(message)
}

fn tool_action(call: &ToolCallMade) -> Option<String> {
    call.result
        .get("action")
        .and_then(|value| value.as_str())
        .or_else(|| {
            call.arguments
                .get("action")
                .and_then(|value| value.as_str())
        })
        .map(str::to_string)
}

fn portfolio_tickers(call: &ToolCallMade) -> Vec<String> {
    let mut tickers = Vec::new();
    push_portfolio_ticker(&mut tickers, &call.result);
    if let Some(portfolio) = call.result.get("portfolio") {
        if let Some(holdings) = portfolio.get("holdings").and_then(|value| value.as_array()) {
            for holding in holdings {
                push_portfolio_ticker(&mut tickers, holding);
            }
        }
        if let Some(watchlist) = portfolio
            .get("watchlist")
            .and_then(|value| value.as_array())
        {
            for holding in watchlist {
                push_portfolio_ticker(&mut tickers, holding);
            }
        }
    }
    if let Some(holdings) = call
        .result
        .get("holdings")
        .and_then(|value| value.as_array())
    {
        for holding in holdings {
            push_portfolio_ticker(&mut tickers, holding);
        }
    }
    if let Some(holdings) = call
        .arguments
        .get("holdings")
        .and_then(|value| value.as_array())
    {
        for holding in holdings {
            push_portfolio_ticker(&mut tickers, holding);
        }
    }
    push_portfolio_ticker(&mut tickers, &call.arguments);
    tickers
}

fn portfolio_argument_tickers(call: &ToolCallMade) -> Vec<String> {
    let mut tickers = Vec::new();
    if let Some(holdings) = call
        .arguments
        .get("holdings")
        .and_then(|value| value.as_array())
    {
        for holding in holdings {
            push_portfolio_ticker(&mut tickers, holding);
        }
    }
    push_portfolio_ticker(&mut tickers, &call.arguments);
    tickers
}

fn portfolio_array<'a>(portfolio: &'a Value, key: &str) -> Vec<&'a Value> {
    portfolio
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn push_portfolio_ticker(tickers: &mut Vec<String>, value: &serde_json::Value) {
    let Some(ticker) = portfolio_ticker(value) else {
        return;
    };
    if !tickers.iter().any(|existing| existing == &ticker) {
        tickers.push(ticker);
    }
}

fn portfolio_ticker(value: &serde_json::Value) -> Option<String> {
    value
        .get("ticker")
        .or_else(|| value.get("symbol"))
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
}

fn format_portfolio_view_entry(holding: &Value) -> Option<String> {
    let ticker = portfolio_ticker(holding)?;
    let mut parts = vec![ticker];

    if let Some(shares) = holding.get("shares").and_then(format_number_value) {
        parts.push(format!("{shares} 股"));
    }
    if let Some(cost_basis) = holding.get("avg_cost").and_then(format_number_value) {
        parts.push(format!("成本价 {cost_basis}"));
    }

    let is_watchlist = holding
        .get("kind")
        .and_then(|value| value.as_str())
        .map(|value| value == "watchlist")
        .or_else(|| {
            holding
                .get("tracking_only")
                .and_then(|value| value.as_bool())
        })
        .unwrap_or(false);
    if is_watchlist {
        parts.push("关注中".to_string());
    }

    if let Some(note) = holding
        .get("notes")
        .or_else(|| holding.get("strategy_notes"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("备注：{}", truncate_portfolio_note(note)));
    }

    Some(parts.join("，"))
}

fn truncate_portfolio_note(note: &str) -> String {
    const MAX_CHARS: usize = 80;
    let mut truncated: String = note.chars().take(MAX_CHARS).collect();
    if note.chars().count() > MAX_CHARS {
        truncated.push('…');
    }
    truncated
}

fn portfolio_cost_basis(call: &ToolCallMade) -> Option<String> {
    call.arguments
        .get("cost_basis")
        .and_then(format_number_value)
        .or_else(|| {
            call.arguments
                .get("holdings")
                .and_then(|value| value.as_array())
                .and_then(|holdings| holdings.first())
                .and_then(|holding| holding.get("cost_basis"))
                .and_then(format_number_value)
        })
}

fn format_number_value(value: &serde_json::Value) -> Option<String> {
    if let Some(number) = value.as_f64() {
        return Some(if number.fract() == 0.0 {
            format!("{number:.0}")
        } else {
            format!("{number:.2}")
        });
    }
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn portfolio_first_result(call: &ToolCallMade) -> Option<String> {
    call.result
        .get("result")
        .and_then(|value| value.as_str())
        .or_else(|| {
            call.result
                .get("holdings")
                .and_then(|value| value.as_array())
                .and_then(|holdings| holdings.first())
                .and_then(|holding| holding.get("result"))
                .and_then(|value| value.as_str())
        })
        .map(str::to_string)
}

fn format_cron_schedule(schedule: &serde_json::Value) -> String {
    let repeat = schedule
        .get("repeat")
        .and_then(|value| value.as_str())
        .unwrap_or("daily");
    let hour = schedule
        .get("hour")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let minute = schedule
        .get("minute")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let time = format!("{hour:02}:{minute:02}");
    match repeat {
        "heartbeat" => "每 30 分钟条件轮询".to_string(),
        "workday" => format!("工作日 {time}"),
        "trading_day" => format!("交易日 {time}"),
        "weekly" => {
            let weekday = schedule
                .get("weekday")
                .and_then(|value| value.as_u64())
                .and_then(weekday_label)
                .unwrap_or("每周");
            format!("{weekday} {time}")
        }
        "once" => schedule
            .get("date")
            .and_then(|value| value.as_str())
            .filter(|date| !date.trim().is_empty())
            .map(|date| format!("{date} {time}"))
            .unwrap_or(time),
        _ => format!("每天 {time}"),
    }
}

fn weekday_label(value: u64) -> Option<&'static str> {
    match value {
        0 => Some("每周一"),
        1 => Some("每周二"),
        2 => Some("每周三"),
        3 => Some("每周四"),
        4 => Some("每周五"),
        5 => Some("每周六"),
        6 => Some("每周日"),
        _ => None,
    }
}

pub(crate) fn response_leaks_system_prompt(content: &str) -> bool {
    const INTERNAL_PROMPT_ECHO_MARKERS: &[&str] = &[
        "### System Instructions ###",
        "### System Prompt ###",
        "### Skill Context ###",
        "### Conversation Context ###",
        "### User Prompt ###",
        "### Available Skills ###",
        "【Session 上下文】",
        "【Invoked Skill Context】",
        "turn-0 可用技能索引",
        "Base directory for this skill:",
    ];

    INTERNAL_PROMPT_ECHO_MARKERS
        .iter()
        .any(|marker| content.contains(marker))
}

pub(crate) fn normalize_local_image_references(
    core: &HoneBotCore,
    session_id: &str,
    content: &str,
) -> String {
    let segments = split_response_content_segments(content);
    if !segments
        .iter()
        .any(|segment| matches!(segment, ResponseContentSegment::LocalImage(_)))
    {
        return content.to_string();
    }

    let mut normalized = String::new();
    for segment in segments {
        match segment {
            ResponseContentSegment::Text(text) => normalized.push_str(&text),
            ResponseContentSegment::LocalImage(marker) => {
                if let Some(stable_path) =
                    stabilize_local_image_path(core, session_id, &marker.path)
                {
                    if !stable_path.starts_with("oss://") {
                        normalized.push_str("file://");
                    }
                    normalized.push_str(&stable_path);
                } else {
                    normalized.push_str(MISSING_LOCAL_IMAGE_FALLBACK_MESSAGE);
                }
            }
        }
    }
    normalized
}

fn stabilize_local_image_path(core: &HoneBotCore, session_id: &str, path: &str) -> Option<String> {
    let source = Path::new(path);
    if !source.is_absolute() || !source.exists() {
        return None;
    }

    let gen_images_root = PathBuf::from(&core.config.storage.gen_images_dir);
    let sandbox_root = sandbox_base_dir();
    let is_generated_image = source.starts_with(&gen_images_root);
    let is_sandbox_image = source.starts_with(&sandbox_root);

    if core.config.cloud.effective_mode().is_cloud_authoritative()
        && (is_generated_image || is_sandbox_image)
        && let Some(oss) =
            hone_core::cloud_runtime::OssObjectStore::from_config(&core.config.cloud.oss)
        && let Ok(bytes) = std::fs::read(source)
    {
        let target_name = unique_stable_image_name(source);
        let key = format!(
            "migration/generated/images/{}/{}",
            hone_core::cloud_runtime::sanitize_key_component(session_id),
            hone_core::cloud_runtime::sanitize_key_component(&target_name)
        );
        let content_type = match source
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => "application/octet-stream",
        };
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(oss.put_object(&key, bytes, content_type))
            })
        } else {
            tokio::runtime::Runtime::new()
                .ok()
                .and_then(|rt| rt.block_on(oss.put_object(&key, bytes, content_type)).ok())
                .map(|_| ())
                .ok_or_else(|| "runtime unavailable".to_string())
        };
        match result {
            Ok(()) => return Some(oss.object_uri(&key)),
            Err(err) => tracing::warn!(
                "[AgentSession] failed to upload generated image to OSS session_id={} source={} err={}",
                session_id,
                source.display(),
                err
            ),
        }
    }

    if is_generated_image {
        return Some(source.to_string_lossy().to_string());
    }

    if !is_sandbox_image {
        return Some(source.to_string_lossy().to_string());
    }

    let target_dir = gen_images_root.join(session_id);
    if let Err(err) = std::fs::create_dir_all(&target_dir) {
        tracing::warn!(
            "[AgentSession] failed to create stable image dir session_id={} dir={} err={}",
            session_id,
            target_dir.display(),
            err
        );
        return Some(source.to_string_lossy().to_string());
    }

    let target_name = unique_stable_image_name(source);
    let target = target_dir.join(target_name);
    match std::fs::copy(source, &target) {
        Ok(_) => Some(target.to_string_lossy().to_string()),
        Err(err) => {
            tracing::warn!(
                "[AgentSession] failed to stabilize local image session_id={} source={} target={} err={}",
                session_id,
                source.display(),
                target.display(),
                err
            );
            Some(source.to_string_lossy().to_string())
        }
    }
}

fn unique_stable_image_name(source: &Path) -> String {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_filename_component)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "image".to_string());
    let ext = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "png".to_string());
    format!("{stem}-{}.{}", uuid::Uuid::new_v4().simple(), ext)
}

fn sanitize_filename_component(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
