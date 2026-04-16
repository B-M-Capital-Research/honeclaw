//! CronJobTool — 定时任务管理工具
//!
//! 通过 Agent 会话管理用户的定时任务。

use async_trait::async_trait;
use hone_core::ActorIdentity;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// CronJobTool — 定时任务管理
pub struct CronJobTool {
    data_dir: String,
    actor: Option<ActorIdentity>,
    channel_target: String,
    admin_bypass: bool,
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
        }
    }

    fn actor(&self) -> hone_core::HoneResult<&ActorIdentity> {
        self.actor
            .as_ref()
            .ok_or_else(|| hone_core::HoneError::Tool("缺少 actor 身份，无法管理定时任务".into()))
    }
}

#[async_trait]
impl Tool for CronJobTool {
    fn name(&self) -> &str {
        "cron_job"
    }

    fn description(&self) -> &str {
        "管理定时任务（每日/每周/工作日/交易日/心跳检测）。支持操作：list（列出所有任务）、add（添加任务）、remove（删除任务）、update（修改任务）。update/remove 可通过 job_id 或 name 定位任务，name 支持模糊匹配（含子串即可）。remove 属于破坏性操作：必须先拿到精确 job_id，再显式传入 confirm=\"yes\" 才会真正删除；未确认前工具只会返回候选任务和确认指引。对于没有具体执行时间、而是按条件轮询的任务，请使用 repeat=heartbeat；heartbeat 任务会每 30 分钟检查一次条件。"
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
                description: "触发小时 (0-23，北京时间)".to_string(),
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
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let storage = hone_memory::CronJobStorage::new(&self.data_dir);
        let actor = self.actor()?;
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        match action {
            "list" => {
                let jobs = storage.list_jobs(actor);
                Ok(serde_json::json!({
                    "action": "list",
                    "jobs": serde_json::to_value(&jobs).unwrap_or_default()
                }))
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
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
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

                let result = storage.add_job(
                    actor,
                    name,
                    hour,
                    minute,
                    repeat,
                    task_prompt,
                    &self.channel_target,
                    weekday,
                    None,
                    true,
                    tags,
                    self.admin_bypass,
                );
                Ok(result)
            }
            "remove" => {
                let job_id = args.get("job_id").and_then(|v| v.as_str()).unwrap_or("");
                let name_query = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let confirm = args.get("confirm").and_then(|v| v.as_str()).unwrap_or("");
                let data = storage.load_jobs(actor);

                let matched_job = if !job_id.is_empty() {
                    match data.jobs.iter().find(|j| j.id == job_id) {
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
                        .filter(|j| j.name.to_lowercase().contains(&name_lower))
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

                let result = storage.remove_job(actor, &matched_job.id);
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
                if let Some(prompt) = args.get("task_prompt") {
                    updates.insert("task_prompt".into(), prompt.clone());
                }
                let tags = args.get("tags").and_then(|v| v.as_array()).map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                });
                // Only treat `name` as a field to update when job_id is also provided;
                // otherwise `name` is the search query.
                let new_name: Option<String> = if !job_id.is_empty() {
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                };

                let mut data = storage.load_jobs(actor);

                // Resolve the target job: by job_id first, then by name fuzzy match.
                let resolved_id: String = if !job_id.is_empty()
                    && data.jobs.iter().any(|j| j.id == job_id)
                {
                    job_id.to_string()
                } else if !name_query.is_empty() {
                    let name_lower = name_query.to_lowercase();
                    let matches: Vec<_> = data
                        .jobs
                        .iter()
                        .filter(|j| j.enabled && j.name.to_lowercase().contains(&name_lower))
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
                            let names: Vec<_> = matches.iter().map(|j| &j.name).collect();
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

                let target = data.jobs.iter_mut().find(|j| j.id == resolved_id);
                match target {
                    Some(job) => {
                        if let Some(name) = new_name {
                            job.name = name;
                        }
                        if let Some(hour) = updates.get("hour").and_then(|v| v.as_u64()) {
                            job.schedule.hour = hour as u32;
                        }
                        if let Some(minute) = updates.get("minute").and_then(|v| v.as_u64()) {
                            job.schedule.minute = minute as u32;
                        }
                        if let Some(repeat) = updates.get("repeat").and_then(|v| v.as_str()) {
                            job.schedule.repeat = repeat.to_string();
                        }
                        if let Some(prompt) = updates.get("task_prompt").and_then(|v| v.as_str()) {
                            job.task_prompt = prompt.to_string();
                        }
                        if let Some(tags) = tags.clone() {
                            job.tags = tags;
                        }
                        if job.schedule.repeat.eq_ignore_ascii_case("heartbeat") {
                            if !job
                                .tags
                                .iter()
                                .any(|tag| tag.eq_ignore_ascii_case("heartbeat"))
                            {
                                job.tags.push("heartbeat".to_string());
                            }
                        } else if updates.contains_key("repeat") || tags.is_some() {
                            job.tags
                                .retain(|tag| !tag.eq_ignore_ascii_case("heartbeat"));
                        }
                        let job_val = serde_json::to_value(job.clone()).unwrap_or_default();
                        storage.save_jobs(actor, &data)?;
                        Ok(serde_json::json!({"success": true, "job": job_val}))
                    }
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

        let add_resp = tool
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
        assert_eq!(add_resp["success"], true);
        let job_id = add_resp["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(!job_id.is_empty());

        let list_resp = tool
            .execute(serde_json::json!({"action":"list"}))
            .await
            .expect("list jobs");
        let jobs = list_resp["jobs"].as_array().cloned().unwrap_or_default();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["name"], "morning report");

        // Update by explicit job_id
        let update_resp = tool
            .execute(serde_json::json!({
                "action":"update",
                "job_id":job_id,
                "hour":10
            }))
            .await
            .expect("update job by id");
        assert_eq!(update_resp["success"], true);
        assert_eq!(update_resp["job"]["schedule"]["hour"], 10);

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
            update_by_name["success"], true,
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
        assert_eq!(remove_preview["success"], false);
        assert_eq!(remove_preview["needs_confirmation"], true);

        let remove_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": job_id,
                "confirm":"yes"
            }))
            .await
            .expect("remove job with confirm");
        assert_eq!(remove_resp["success"], true);

        let list_resp = tool
            .execute(serde_json::json!({"action":"list"}))
            .await
            .expect("list jobs after remove");
        let jobs = list_resp["jobs"].as_array().cloned().unwrap_or_default();
        assert!(jobs.is_empty());
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

        let resp = tool
            .execute(serde_json::json!({
                "action":"update",
                "name":"nonexistent task",
                "hour":9
            }))
            .await
            .expect("update nonexistent");
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap_or("").contains("未找到"));
    }

    #[tokio::test]
    async fn remove_requires_explicit_confirmation_and_exact_job_id() {
        let data_dir = make_temp_dir("hone_cron_tool_confirm");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor.clone()), "u1", false);

        let add_resp = tool
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
        let job_id = add_resp["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let preview_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": job_id
            }))
            .await
            .expect("preview remove");
        assert_eq!(preview_resp["success"], false);
        assert_eq!(preview_resp["needs_confirmation"], true);
        assert_eq!(preview_resp["job"]["id"], add_resp["job"]["id"]);

        let jobs_after_preview = hone_memory::CronJobStorage::new(&data_dir).list_jobs(&actor);
        assert_eq!(jobs_after_preview.len(), 1);

        let confirmed_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "job_id": add_resp["job"]["id"],
                "confirm":"yes"
            }))
            .await
            .expect("confirmed remove");
        assert_eq!(confirmed_resp["success"], true);

        let jobs_after_confirm = hone_memory::CronJobStorage::new(&data_dir).list_jobs(&actor);
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

        let resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "name":"crude"
            }))
            .await
            .expect("remove by ambiguous name");
        assert_eq!(resp["success"], false);
        assert_eq!(resp["needs_confirmation"], true);
        assert_eq!(
            resp["candidates"].as_array().map(|items| items.len()),
            Some(2)
        );

        let jobs = hone_memory::CronJobStorage::new(&data_dir).list_jobs(&actor);
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn openai_schema_uses_object_items_for_tags_array() {
        let data_dir = make_temp_dir("hone_cron_tool_schema");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = CronJobTool::new(&data_dir, Some(actor), "u1", false);

        let schema = tool.to_openai_schema();
        let tags_items = schema["function"]["parameters"]["properties"]["tags"]["items"].clone();
        assert_eq!(tags_items["type"], "string");
    }
}
