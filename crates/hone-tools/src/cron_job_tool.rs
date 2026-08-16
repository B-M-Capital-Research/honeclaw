//! CronJobTool — 定时任务管理工具
//!
//! 通过 Agent 会话管理用户的定时任务。

use std::path::PathBuf;
use std::sync::OnceLock;

use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_event_engine::prefs::{FilePrefsStorage, PrefsProvider};
use hone_memory::cron_job::{CronJobUpdate, CronSchedule};
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// CronJobTool — 定时任务管理
pub struct CronJobTool {
    data_dir: String,
    actor: Option<ActorIdentity>,
    channel_target: String,
    admin_bypass: bool,
    postgres: Option<CloudPgRuntime>,
    storage: OnceLock<hone_memory::CronJobStorage>,
    /// Where per-actor notification prefs live. Cron jobs are only one of the
    /// stores that can push on a schedule; without this the tool can report an
    /// empty job list while daily digests keep firing.
    notif_prefs_dir: Option<PathBuf>,
    /// Digest slot times that fire when the actor has no explicit `digest_slots`.
    default_digest_slot_times: Vec<String>,
}

impl CronJobTool {
    pub fn new(
        data_dir: &str,
        actor: Option<ActorIdentity>,
        channel_target: &str,
        admin_bypass: bool,
    ) -> Self {
        Self {
            data_dir: data_dir.to_string(),
            actor,
            channel_target: channel_target.to_string(),
            admin_bypass,
            postgres: None,
            storage: OnceLock::new(),
            notif_prefs_dir: None,
            default_digest_slot_times: Vec::new(),
        }
    }

    pub fn new_cloud(
        data_dir: &str,
        actor: Option<ActorIdentity>,
        channel_target: &str,
        admin_bypass: bool,
        postgres: CloudPgRuntime,
    ) -> Self {
        Self {
            data_dir: data_dir.to_string(),
            actor,
            channel_target: channel_target.to_string(),
            admin_bypass,
            postgres: Some(postgres),
            storage: OnceLock::new(),
            notif_prefs_dir: None,
            default_digest_slot_times: Vec::new(),
        }
    }

    /// Give the tool the other scheduled-push store so `list` / `remove_all`
    /// can state whether anything still pushes on a schedule.
    pub fn with_push_context(
        mut self,
        notif_prefs_dir: impl Into<PathBuf>,
        default_digest_slot_times: Vec<String>,
    ) -> Self {
        self.notif_prefs_dir = Some(notif_prefs_dir.into());
        self.default_digest_slot_times = default_digest_slot_times;
        self
    }

    /// Automatic pushes that survive a cron-only change. `cron_job` owns one
    /// store; event pushes and daily digests live in notification prefs and
    /// keep firing after every cron job is deleted. Reporting only the cron
    /// store is what lets an honest tool result become a false "已全部关闭".
    fn remaining_automatic_push_sources(
        &self,
        actor: &ActorIdentity,
        remaining_cron_jobs: usize,
    ) -> Option<Value> {
        let prefs_dir = self.notif_prefs_dir.as_ref()?;
        let storage = FilePrefsStorage::new(prefs_dir).ok()?;
        let prefs = storage.load(actor);

        let (digest_source, digest_times) = match prefs.effective_digest_slots() {
            Some(slots) if slots.is_empty() => ("disabled", Vec::new()),
            Some(slots) => (
                "user",
                slots.into_iter().map(|slot| slot.time).collect::<Vec<_>>(),
            ),
            None => ("system_default", self.default_digest_slot_times.clone()),
        };
        let digest_active = !digest_times.is_empty();
        let all_stopped = remaining_cron_jobs == 0 && !prefs.enabled && !digest_active;

        let mut remaining = Vec::new();
        if prefs.enabled {
            remaining.push("事件即时推送".to_string());
        }
        if digest_active {
            remaining.push(format!("每日摘要推送（{}）", digest_times.join("、")));
        }
        if remaining_cron_jobs > 0 {
            remaining.push(format!("{remaining_cron_jobs} 个定时/心跳任务"));
        }

        Some(serde_json::json!({
            "all_automatic_push_stopped": all_stopped,
            "remaining_sources": remaining,
            "event_push_enabled": prefs.enabled,
            "digest_source": digest_source,
            "digest_times": digest_times,
            "remaining_cron_jobs": remaining_cron_jobs,
            "stop_all_action": "notification_prefs(action=\"disable_all\")",
            "disclosure": if all_stopped {
                "本 actor 当前没有任何自动推送来源。"
            } else {
                "定时/心跳任务之外仍有自动推送来源在按计划触发。回答不得表述为“已全部关闭”“没有任何自动提醒”；必须逐项说明上面 remaining_sources，并告诉用户可以用一句话要求关闭全部自动提醒。"
            }
        }))
    }

    fn actor(&self) -> hone_core::HoneResult<&ActorIdentity> {
        self.actor
            .as_ref()
            .ok_or_else(|| hone_core::HoneError::Tool("缺少 actor 身份，无法管理定时任务".into()))
    }

    fn storage(&self) -> hone_core::HoneResult<&hone_memory::CronJobStorage> {
        if let Some(storage) = self.storage.get() {
            return Ok(storage);
        }
        let storage = if let Some(postgres) = self.postgres.clone() {
            hone_memory::CronJobStorage::new_cloud(postgres)?
        } else {
            hone_memory::CronJobStorage::new(&self.data_dir)
        };
        let _ = self.storage.set(storage);
        self.storage.get().ok_or_else(|| {
            hone_core::HoneError::Storage("定时任务 PostgreSQL 存储初始化失败".to_string())
        })
    }
}

#[async_trait]
impl Tool for CronJobTool {
    fn name(&self) -> &str {
        "cron_job"
    }

    fn description(&self) -> &str {
        "管理定时任务（每日/每周/工作日/交易日/心跳检测）。支持操作：list（列出所有任务）、add（添加任务）、remove（删除单个任务）、remove_all（删除当前用户全部定时和心跳任务）、update（修改任务）。update/remove 可通过 job_id 或 name 定位任务，name 支持模糊匹配（含子串即可）。remove 属于破坏性操作：必须先拿到精确 job_id，再显式传入 confirm=\"yes\" 才会真正删除；未确认前工具只会返回候选任务和确认指引。用户已经明确说“取消/删除所有定时任务或心跳任务”时，直接调用 remove_all；当前这句话就是授权，不要再逐个确认或循环删除。若用户说的是“取消所有自动提醒/关闭所有自动推送”，应改用 notification_prefs(action=\"disable_all\")，它会同时关闭事件推送并删除全部定时/心跳任务。对于没有具体执行时间、而是按条件轮询的任务，请使用 repeat=heartbeat；heartbeat 任务会每 30 分钟检查一次条件。\n\n**与 quiet_hours 的关系**：用户在 notification_prefs 设了 quiet_hours 后，所有 cron 任务**默认遵守**该勿扰区间——区间内到点的任务会被静音跳过（cron_job_runs 落 metadata.skipped='quiet_hours'）。若某条 cron 必须严守原时刻不能被静音（如盘前 06:55 复盘），update 时传 bypass_quiet_hours=true。add 暂不接受该字段，新建任务默认遵守 quiet_hours。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型".to_string(),
                required: true,
                r#enum: Some(vec![
                    "list".into(),
                    "add".into(),
                    "remove".into(),
                    "remove_all".into(),
                    "update".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description:
                    "任务名称（add 时必填；update/remove 时若无 job_id 可用名称模糊匹配定位任务）"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "hour".to_string(),
                param_type: "number".to_string(),
                description: "触发小时 (0-23，运行时时区)".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "minute".to_string(),
                param_type: "number".to_string(),
                description: "触发分钟 (0-59)".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "repeat".to_string(),
                param_type: "string".to_string(),
                description: "重复类型".to_string(),
                required: false,
                r#enum: Some(vec![
                    "daily".into(),
                    "weekly".into(),
                    "once".into(),
                    "workday".into(),
                    "trading_day".into(),
                    "holiday".into(),
                    "heartbeat".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "weekday".to_string(),
                param_type: "number".to_string(),
                description: "每周几（仅 weekly 使用；0=周一，6=周日）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "date".to_string(),
                param_type: "string".to_string(),
                description: "一次性任务的绝对日期（仅 repeat=once 使用，格式 YYYY-MM-DD，运行时时区）"
                    .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "tags".to_string(),
                param_type: "array".to_string(),
                description: "任务标签；heartbeat 任务建议包含 heartbeat 标签".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({ "type": "string" })),
            },
            ToolParameter {
                name: "task_prompt".to_string(),
                param_type: "string".to_string(),
                description: "任务指令描述（add 时必填）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "job_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "任务 ID（remove/update 时优先使用；若未知可留空并改用 name 模糊匹配）"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "confirm".to_string(),
                param_type: "string".to_string(),
                description:
                    "仅 remove 使用；删除属于破坏性操作，必须显式传入 confirm=\"yes\" 才会真正执行"
                        .to_string(),
                required: false,
                r#enum: Some(vec!["yes".into()]),
                items: None,
            },
            ToolParameter {
                name: "bypass_quiet_hours".to_string(),
                param_type: "boolean".to_string(),
                description:
                    "仅 update 使用；true=该任务忽略用户的 quiet_hours 静音区间，到点照常执行（如 06:55 盘前复盘）；false（默认）=遵守 quiet_hours，区间内被静音跳过"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let storage = self.storage()?;
        let actor = self.actor()?;
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        match action {
            "list" => {
                let jobs = storage.list_jobs(actor).await;
                let mut result = serde_json::json!({
                    "action": "list",
                    "jobs": serde_json::to_value(&jobs).unwrap_or_default()
                });
                if let Some(sources) = self.remaining_automatic_push_sources(actor, jobs.len()) {
                    result["automatic_push"] = sources;
                }
                Ok(result)
            }
            "add" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未命名任务");
                let hour = args.get("hour").and_then(|v| v.as_u64()).map(|v| v as u32);
                let minute = args
                    .get("minute")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                let repeat = args
                    .get("repeat")
                    .and_then(|v| v.as_str())
                    .unwrap_or("daily");
                let tags = args.get("tags").and_then(|v| v.as_array()).map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(|tag| tag.to_string()))
                        .collect::<Vec<_>>()
                });
                let task_prompt = args
                    .get("task_prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let weekday = args
                    .get("weekday")
                    .and_then(|v| v.as_u64())
                    .map(|w| w as u32);
                let date = args
                    .get("date")
                    .and_then(|v| v.as_str())
                    .map(|date_text| date_text.to_string());

                let result = storage
                    .add_job(
                        actor,
                        name,
                        hour,
                        minute,
                        repeat,
                        task_prompt,
                        &self.channel_target,
                        weekday,
                        date,
                        None,
                        true,
                        tags,
                        self.admin_bypass,
                    )
                    .await;
                Ok(result)
            }
            "remove" => {
                let job_id = args.get("job_id").and_then(|v| v.as_str()).unwrap_or("");
                let name_query = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let confirm = args.get("confirm").and_then(|v| v.as_str()).unwrap_or("");
                let data = storage.load_jobs(actor).await;

                let matched_job = if !job_id.is_empty() {
                    match data.jobs.iter().find(|job| job.id == job_id) {
                        Some(job) => job.clone(),
                        None => {
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": format!("未找到任务 ID「{job_id}」，请先调用 list 确认任务 ID")
                            }));
                        }
                    }
                } else if !name_query.is_empty() {
                    let name_lower = name_query.to_lowercase();
                    let matches: Vec<_> = data
                        .jobs
                        .iter()
                        .filter(|job| job.name.to_lowercase().contains(&name_lower))
                        .collect();
                    match matches.len() {
                        0 => {
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": format!("未找到名称包含「{name_query}」的任务，请先用 list 确认任务名称")
                            }));
                        }
                        1 => (*matches[0]).clone(),
                        _ => {
                            let candidates: Vec<_> = matches
                                .iter()
                                .map(|job| {
                                    serde_json::json!({
                                        "job_id": job.id,
                                        "name": job.name,
                                        "schedule": job.schedule,
                                        "enabled": job.enabled,
                                    })
                                })
                                .collect();
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": format!("名称「{name_query}」匹配到多个任务；删除前请先让用户确认具体 job_id"),
                                "needs_confirmation": true,
                                "candidates": candidates
                            }));
                        }
                    }
                } else {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "remove 操作需要提供 job_id 或 name"
                    }));
                };

                if confirm != "yes" {
                    return Ok(serde_json::json!({
                        "success": false,
                        "needs_confirmation": true,
                        "job": serde_json::to_value(&matched_job).unwrap_or_default(),
                        "error": format!(
                            "删除定时任务属于破坏性操作。请先向用户确认；确认后再使用 cron_job(action=\"remove\", job_id=\"{}\", confirm=\"yes\") 执行删除",
                            matched_job.id
                        )
                    }));
                }

                let result = storage.remove_job(actor, &matched_job.id).await?;
                Ok(result)
            }
            "remove_all" => {
                let removed_jobs = storage.remove_all_jobs(actor).await?;
                let mut result = serde_json::json!({
                    "success": true,
                    "action": "remove_all",
                    "removed_count": removed_jobs.len(),
                    "removed_jobs": removed_jobs,
                    // Cron jobs only. `remaining_count: 0` never meant that
                    // nothing pushes any more; `automatic_push` says that.
                    "remaining_count": 0,
                });
                if let Some(sources) = self.remaining_automatic_push_sources(actor, 0) {
                    result["automatic_push"] = sources;
                }
                Ok(result)
            }
            "update" => {
                let job_id = args.get("job_id").and_then(|v| v.as_str()).unwrap_or("");
                let name_query = args.get("name").and_then(|v| v.as_str()).unwrap_or("");

                let mut updates = serde_json::Map::new();
                if let Some(hour) = args.get("hour") {
                    updates.insert("hour".into(), hour.clone());
                }
                if let Some(minute) = args.get("minute") {
                    updates.insert("minute".into(), minute.clone());
                }
                if let Some(repeat) = args.get("repeat") {
                    updates.insert("repeat".into(), repeat.clone());
                }
                if let Some(date) = args.get("date") {
                    updates.insert("date".into(), date.clone());
                }
                let weekday = args
                    .get("weekday")
                    .and_then(|v| v.as_u64())
                    .map(|w| w as u32);
                if let Some(prompt) = args.get("task_prompt") {
                    updates.insert("task_prompt".into(), prompt.clone());
                }
                let tags = args.get("tags").and_then(|v| v.as_array()).map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(|tag| tag.to_string()))
                        .collect::<Vec<_>>()
                });
                let bypass_quiet_hours = args.get("bypass_quiet_hours").and_then(|v| v.as_bool());
                // Only treat `name` as a field to update when job_id is also provided;
                // otherwise `name` is the search query.
                let new_name: Option<String> = if !job_id.is_empty() {
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .map(|name| name.to_string())
                } else {
                    None
                };

                let data = storage.load_jobs(actor).await;

                // Resolve the target job: by job_id first, then by name fuzzy match.
                let resolved_id: String = if !job_id.is_empty()
                    && data.jobs.iter().any(|job| job.id == job_id)
                {
                    job_id.to_string()
                } else if !name_query.is_empty() {
                    let name_lower = name_query.to_lowercase();
                    let matches: Vec<_> = data
                        .jobs
                        .iter()
                        .filter(|job| job.enabled && job.name.to_lowercase().contains(&name_lower))
                        .collect();
                    match matches.len() {
                        0 => {
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": format!("未找到名称包含「{name_query}」的任务，请先用 list 确认任务名称")
                            }));
                        }
                        1 => matches[0].id.clone(),
                        _ => {
                            let names: Vec<_> = matches.iter().map(|job| &job.name).collect();
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": format!("名称「{name_query}」匹配到多个任务：{names:?}，请提供 job_id 精确定位")
                            }));
                        }
                    }
                } else {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!(
                            "update 操作需要提供 job_id 或 name 来定位任务。\
                            当前任务列表请先调用 cron_job(action=\"list\") 查看。\
                            job_id 传入值为「{job_id}」"
                        )
                    }));
                };

                let Some(existing_job) =
                    data.jobs.iter().find(|job| job.id == resolved_id).cloned()
                else {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("未找到任务 ID「{resolved_id}」，请先调用 list 确认任务 ID")
                    }));
                };

                let schedule = if updates.contains_key("hour")
                    || updates.contains_key("minute")
                    || updates.contains_key("repeat")
                    || weekday.is_some()
                    || updates.contains_key("date")
                {
                    let repeat = updates
                        .get("repeat")
                        .and_then(|v| v.as_str())
                        .unwrap_or(existing_job.schedule.repeat.as_str());
                    let date = if repeat.eq_ignore_ascii_case("once") {
                        updates
                            .get("date")
                            .and_then(|v| v.as_str())
                            .map(|date_text| date_text.to_string())
                            .or(existing_job.schedule.date.clone())
                    } else {
                        None
                    };
                    Some(CronSchedule {
                        hour: updates
                            .get("hour")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32)
                            .unwrap_or(existing_job.schedule.hour),
                        minute: updates
                            .get("minute")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32)
                            .unwrap_or(existing_job.schedule.minute),
                        weekday: if repeat.eq_ignore_ascii_case("weekly") {
                            weekday.or(existing_job.schedule.weekday)
                        } else {
                            None
                        },
                        repeat: repeat.to_string(),
                        date,
                    })
                } else {
                    None
                };

                let update = CronJobUpdate {
                    name: new_name,
                    schedule,
                    task_prompt: updates
                        .get("task_prompt")
                        .and_then(|v| v.as_str())
                        .map(|prompt| prompt.to_string()),
                    push: None,
                    enabled: None,
                    channel_target: None,
                    tags,
                    bypass_quiet_hours,
                };

                match storage
                    .update_job(&resolved_id, Some(actor), update, self.admin_bypass)
                    .await?
                {
                    Some((_updated_actor, job)) => Ok(serde_json::json!({
                        "success": true,
                        "job": serde_json::to_value(job).unwrap_or_default()
                    })),
                    None => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("未找到任务 ID「{resolved_id}」，请先调用 list 确认任务 ID")
                    })),
                }
            }
            _ => Ok(serde_json::json!({"error": format!("不支持的操作: {action}")})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn cron_job_tool_add_list_update_remove_flow() {
        let data_dir = make_temp_dir("hone_cron_tool");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "u1", false);

        let add_response = tool
            .execute(serde_json::json!({
                "action":"add",
                "name":"morning report",
                "hour":9,
                "minute":30,
                "repeat":"daily",
                "task_prompt":"send report"
            }))
            .await
            .expect("add job");
        assert_eq!(add_response["success"].as_bool(), Some(true));
        let job_id = add_response["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(!job_id.is_empty());

        let list_response = tool
            .execute(serde_json::json!({"action":"list"}))
            .await
            .expect("list jobs");
        let jobs = list_response["jobs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["name"], "morning report");

        // Update by explicit job_id
        let update_response = tool
            .execute(serde_json::json!({
                "action":"update",
                "job_id":job_id,
                "hour":10
            }))
            .await
            .expect("update job by id");
        assert_eq!(update_response["success"].as_bool(), Some(true));
        assert_eq!(update_response["job"]["schedule"]["hour"], 10);

        // Update by name fuzzy match (no job_id)
        let update_by_name = tool
            .execute(serde_json::json!({
                "action":"update",
                "name":"morning",
                "minute":45
            }))
            .await
            .expect("update job by name");
        assert_eq!(
            update_by_name["success"].as_bool(),
            Some(true),
            "name fuzzy update failed: {update_by_name}"
        );
        assert_eq!(update_by_name["job"]["schedule"]["minute"], 45);

        let remove_preview = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id":job_id
            }))
            .await
            .expect("remove job");
        assert_eq!(remove_preview["success"].as_bool(), Some(false));
        assert_eq!(remove_preview["needs_confirmation"].as_bool(), Some(true));

        let remove_response = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": job_id,
                "confirm":"yes"
            }))
            .await
            .expect("remove job with confirm");
        assert_eq!(remove_response["success"].as_bool(), Some(true));

        let list_response = tool
            .execute(serde_json::json!({"action":"list"}))
            .await
            .expect("list jobs after remove");
        let jobs = list_response["jobs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn cron_job_tool_add_preserves_origin_channel_target() {
        let data_dir = make_temp_dir("hone_cron_tool_origin_target");
        let actor = ActorIdentity::new("telegram", "user_42", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "-1001234567890", false);

        let add_response = tool
            .execute(serde_json::json!({
                "action":"add",
                "name":"group heartbeat",
                "repeat":"heartbeat",
                "task_prompt":"check conditions"
            }))
            .await
            .expect("add job");

        assert_eq!(add_response["success"].as_bool(), Some(true));
        assert_eq!(add_response["job"]["channel"], "telegram");
        assert_eq!(add_response["job"]["channel_target"], "-1001234567890");
    }

    /// Feishu users kept asking to turn scheduled tasks off, were told they
    /// were off, and kept receiving pushes. Cron jobs are one store; the daily
    /// digest is another. `remove_all` must say what still fires instead of
    /// letting an empty job list read as "nothing pushes any more".
    #[tokio::test]
    async fn cron_removal_discloses_digest_and_event_pushes_that_still_fire() {
        let data_dir = make_temp_dir("hone_cron_tool_remaining_push");
        let prefs_dir = format!("{data_dir}/notif_prefs");
        std::fs::create_dir_all(&prefs_dir).expect("prefs dir");
        let actor = ActorIdentity::new("feishu", "ou_remaining", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "ou_remaining", false)
            .with_push_context(&prefs_dir, vec!["08:30".to_string(), "09:00".to_string()]);

        tool.execute(serde_json::json!({
            "action": "add",
            "name": "daily report",
            "hour": 9,
            "minute": 0,
            "repeat": "daily",
            "task_prompt": "task"
        }))
        .await
        .expect("add job");

        let removed = tool
            .execute(serde_json::json!({"action": "remove_all"}))
            .await
            .expect("remove_all");
        assert_eq!(removed["removed_count"], 1);
        // The cron store is empty, but the actor has no prefs file yet, so the
        // system-default digest slots still fire every morning.
        let summary = &removed["automatic_push"];
        assert_eq!(summary["all_automatic_push_stopped"], false);
        assert_eq!(summary["digest_source"], "system_default");
        let remaining = summary["remaining_sources"]
            .as_array()
            .expect("remaining sources")
            .iter()
            .filter_map(|item| item.as_str())
            .collect::<Vec<_>>()
            .join("|");
        assert!(remaining.contains("每日摘要推送"), "{remaining}");
        assert!(remaining.contains("08:30"), "{remaining}");
        assert!(remaining.contains("事件即时推送"), "{remaining}");

        let listed = tool
            .execute(serde_json::json!({"action": "list"}))
            .await
            .expect("list");
        assert!(listed["jobs"].as_array().expect("jobs").is_empty());
        assert_eq!(
            listed["automatic_push"]["all_automatic_push_stopped"],
            false
        );
    }

    #[tokio::test]
    async fn cron_removal_reports_a_full_stop_once_every_push_source_is_off() {
        use hone_event_engine::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let data_dir = make_temp_dir("hone_cron_tool_full_stop");
        let prefs_dir = format!("{data_dir}/notif_prefs");
        std::fs::create_dir_all(&prefs_dir).expect("prefs dir");
        let actor = ActorIdentity::new("feishu", "ou_full_stop", None::<String>).expect("actor");

        let storage = FilePrefsStorage::new(&prefs_dir).expect("prefs storage");
        storage
            .save(
                &actor,
                &NotificationPrefs {
                    enabled: false,
                    digest_slots: Some(Vec::new()),
                    ..Default::default()
                },
            )
            .expect("save prefs");

        let tool = CronJobTool::new(&data_dir, Some(actor), "ou_full_stop", false)
            .with_push_context(&prefs_dir, vec!["08:30".to_string()]);
        let removed = tool
            .execute(serde_json::json!({"action": "remove_all"}))
            .await
            .expect("remove_all");
        assert_eq!(
            removed["automatic_push"]["all_automatic_push_stopped"],
            true
        );
        assert!(
            removed["automatic_push"]["remaining_sources"]
                .as_array()
                .expect("remaining sources")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn remove_all_is_actor_scoped_and_idempotent() {
        let data_dir = make_temp_dir("hone_cron_tool_remove_all");
        let actor = ActorIdentity::new("feishu", "u1", None::<String>).expect("actor");
        let other_actor = ActorIdentity::new("feishu", "u2", None::<String>).expect("other actor");
        let tool = CronJobTool::new(&data_dir, Some(actor.clone()), "u1", false);
        let other_tool = CronJobTool::new(&data_dir, Some(other_actor.clone()), "u2", false);

        for (name, repeat) in [("daily report", "daily"), ("price heartbeat", "heartbeat")] {
            tool.execute(serde_json::json!({
                "action": "add",
                "name": name,
                "hour": 9,
                "minute": 0,
                "repeat": repeat,
                "task_prompt": "task"
            }))
            .await
            .expect("add actor job");
        }
        other_tool
            .execute(serde_json::json!({
                "action": "add",
                "name": "other actor report",
                "hour": 10,
                "minute": 0,
                "repeat": "daily",
                "task_prompt": "task"
            }))
            .await
            .expect("add other actor job");

        let removed = tool
            .execute(serde_json::json!({"action": "remove_all"}))
            .await
            .expect("remove all actor jobs");
        assert_eq!(removed["success"], true);
        assert_eq!(removed["action"], "remove_all");
        assert_eq!(removed["removed_count"], 2);
        assert_eq!(removed["remaining_count"], 0);

        let storage = hone_memory::CronJobStorage::new(&data_dir);
        assert!(storage.list_jobs(&actor).await.is_empty());
        assert_eq!(storage.list_jobs(&other_actor).await.len(), 1);

        let repeated = tool
            .execute(serde_json::json!({"action": "remove_all"}))
            .await
            .expect("remove all remains idempotent");
        assert_eq!(repeated["success"], true);
        assert_eq!(repeated["removed_count"], 0);
    }

    #[tokio::test]
    async fn update_by_name_no_match_returns_error() {
        let data_dir = make_temp_dir("hone_cron_tool_nomatch");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "u1", false);

        tool.execute(serde_json::json!({
            "action":"add",
            "name":"daily briefing",
            "hour":8,
            "minute":0,
            "repeat":"daily",
            "task_prompt":"send briefing"
        }))
        .await
        .expect("add");

        let update_response = tool
            .execute(serde_json::json!({
                "action":"update",
                "name":"nonexistent task",
                "hour":9
            }))
            .await
            .expect("update nonexistent");
        assert_eq!(update_response["success"].as_bool(), Some(false));
        assert!(
            update_response["error"]
                .as_str()
                .unwrap_or("")
                .contains("未找到")
        );
    }

    #[tokio::test]
    async fn remove_requires_explicit_confirmation_and_exact_job_id() {
        let data_dir = make_temp_dir("hone_cron_tool_confirm");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor.clone()), "u1", false);

        let add_response = tool
            .execute(serde_json::json!({
                "action":"add",
                "name":"night review",
                "hour":20,
                "minute":30,
                "repeat":"daily",
                "task_prompt":"send review"
            }))
            .await
            .expect("add job");
        let job_id = add_response["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let preview_response = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": job_id
            }))
            .await
            .expect("preview remove");
        assert_eq!(preview_response["success"].as_bool(), Some(false));
        assert_eq!(preview_response["needs_confirmation"].as_bool(), Some(true));
        assert_eq!(preview_response["job"]["id"], add_response["job"]["id"]);

        let jobs_after_preview = hone_memory::CronJobStorage::new(&data_dir)
            .list_jobs(&actor)
            .await;
        assert_eq!(jobs_after_preview.len(), 1);

        let confirmed_response = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": add_response["job"]["id"],
                "confirm":"yes"
            }))
            .await
            .expect("confirmed remove");
        assert_eq!(confirmed_response["success"].as_bool(), Some(true));

        let jobs_after_confirm = hone_memory::CronJobStorage::new(&data_dir)
            .list_jobs(&actor)
            .await;
        assert!(jobs_after_confirm.is_empty());
    }

    #[tokio::test]
    async fn remove_by_ambiguous_name_returns_candidates_without_deleting() {
        let data_dir = make_temp_dir("hone_cron_tool_ambiguous_remove");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor.clone()), "u1", false);

        for suffix in ["oil am", "oil pm"] {
            tool.execute(serde_json::json!({
                "action":"add",
                "name": format!("crude {suffix}"),
                "hour":8,
                "minute":0,
                "repeat":"daily",
                "task_prompt":"send oil update"
            }))
            .await
            .expect("add job");
        }

        let remove_response = tool
            .execute(serde_json::json!({
                "action":"remove",
                "name":"crude"
            }))
            .await
            .expect("remove by ambiguous name");
        assert_eq!(remove_response["success"].as_bool(), Some(false));
        assert_eq!(remove_response["needs_confirmation"].as_bool(), Some(true));
        assert_eq!(
            remove_response["candidates"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );

        let jobs = hone_memory::CronJobStorage::new(&data_dir)
            .list_jobs(&actor)
            .await;
        assert_eq!(jobs.len(), 2);
    }

    #[tokio::test]
    async fn weekly_jobs_can_be_added_and_updated_with_weekday() {
        let data_dir = make_temp_dir("hone_cron_tool_weekly");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "u1", false);

        let add_response = tool
            .execute(serde_json::json!({
                "action":"add",
                "name":"weekly sunday report",
                "hour":12,
                "minute":0,
                "repeat":"weekly",
                "weekday":6,
                "task_prompt":"send weekly report"
            }))
            .await
            .expect("add weekly job");
        assert_eq!(
            add_response["success"].as_bool(),
            Some(true),
            "weekly add failed: {add_response}"
        );
        assert_eq!(add_response["job"]["schedule"]["weekday"], 6);
        let job_id = add_response["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let update_response = tool
            .execute(serde_json::json!({
                "action":"update",
                "job_id":job_id,
                "repeat":"weekly",
                "weekday":0,
                "hour":9
            }))
            .await
            .expect("update weekly job");
        assert_eq!(
            update_response["success"].as_bool(),
            Some(true),
            "weekly update failed: {update_response}"
        );
        assert_eq!(update_response["job"]["schedule"]["weekday"], 0);
        assert_eq!(update_response["job"]["schedule"]["hour"], 9);

        let clear_weekday_response = tool
            .execute(serde_json::json!({
                "action":"update",
                "job_id":update_response["job"]["id"],
                "repeat":"daily"
            }))
            .await
            .expect("change weekly to daily");
        assert_eq!(clear_weekday_response["success"].as_bool(), Some(true));
        assert!(clear_weekday_response["job"]["schedule"]["weekday"].is_null());
        assert_eq!(clear_weekday_response["job"]["schedule"]["repeat"], "daily");
    }

    #[test]
    fn openai_schema_uses_object_items_for_tags_array() {
        let data_dir = make_temp_dir("hone_cron_tool_schema");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "u1", false);

        let schema = tool.to_openai_schema();
        let tags_items = schema["function"]["parameters"]["properties"]["tags"]["items"].clone();
        assert_eq!(tags_items["type"], "string");
        assert_eq!(
            schema["function"]["parameters"]["properties"]["weekday"]["type"],
            "number"
        );
    }
}
