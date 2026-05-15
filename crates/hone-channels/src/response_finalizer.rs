use std::path::{Path, PathBuf};

use hone_core::agent::{AgentResponse, ToolCallMade};

use crate::HoneBotCore;
use crate::outbound::{ResponseContentSegment, split_response_content_segments};
use crate::runtime::{is_transitional_planning_sentence, sanitize_user_visible_output};
use crate::sandbox::sandbox_base_dir;

pub(crate) const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";
const MISSING_LOCAL_IMAGE_FALLBACK_MESSAGE: &str = "（图表文件不可用，请重新生成）";

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
        if let Some(recovered) =
            recover_successful_side_effect_confirmation(&response.tool_calls_made)
        {
            tracing::info!(
                "[AgentSession] recovered side-effect confirmation from tool result runner={} session_id={}",
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
        response.content = sanitized.content;
    }

    response.content = normalize_local_image_references(core, session_id, &response.content);
    outcome
}

fn recover_successful_side_effect_confirmation(tool_calls: &[ToolCallMade]) -> Option<String> {
    tool_calls.iter().rev().find_map(|call| {
        if call.result.get("success").and_then(|value| value.as_bool()) != Some(true) {
            return None;
        }
        match call.name.as_str() {
            "cron_job" => recover_cron_job_confirmation(call),
            "portfolio" => recover_portfolio_confirmation(call),
            _ => None,
        }
    })
}

fn recover_cron_job_confirmation(call: &ToolCallMade) -> Option<String> {
    let action = tool_action(call);
    match action.as_deref() {
        Some("add") => cron_job_confirmation_message("已创建定时任务", call),
        Some("update") => cron_job_confirmation_message("已更新定时任务", call),
        Some("remove") => call
            .result
            .get("removed_job_id")
            .and_then(|value| value.as_str())
            .map(|job_id| format!("已删除定时任务：{job_id}。")),
        _ => None,
    }
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

fn push_portfolio_ticker(tickers: &mut Vec<String>, value: &serde_json::Value) {
    let Some(ticker) = value
        .get("ticker")
        .or_else(|| value.get("symbol"))
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    if !tickers.iter().any(|existing| existing == &ticker) {
        tickers.push(ticker);
    }
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
    let trimmed = content.trim_start_matches(char::is_whitespace);
    trimmed.starts_with("### System Instructions ###")
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
                    normalized.push_str("file://");
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
    if source.starts_with(&gen_images_root) {
        return Some(source.to_string_lossy().to_string());
    }

    let sandbox_root = sandbox_base_dir();
    if !source.starts_with(&sandbox_root) {
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
