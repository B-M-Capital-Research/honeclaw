use serde::{Deserialize, Serialize};
use serde_json::Value;

use hone_memory::cron_job::{CronJob, CronJobExecutionRecord};
#[derive(Deserialize)]
pub struct UserIdQuery {
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub channel_scope: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub message: Option<String>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct HistoryMsg {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(default)]
    pub synthetic: bool,
    #[serde(default)]
    pub transcript_only: bool,
    pub attachments: Vec<HistoryAttachment>,
}

#[derive(Serialize)]
pub struct HistoryAttachment {
    pub path: String,
    pub name: String,
    pub kind: String,
}

#[derive(Deserialize)]
pub struct ImageQuery {
    pub path: Option<String>,
}

/// GET /api/users 响应体中的单条用户信息
#[derive(Serialize)]
pub struct UserInfo {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub session_id: String,
    pub session_kind: String,
    pub session_label: String,
    pub actor_user_id: Option<String>,
    pub last_message: String,
    pub last_role: String,
    pub last_time: String,
    pub message_count: usize,
}

/// GET /api/skills 响应体中的单条技能信息
#[derive(Serialize)]
pub struct SkillInfo {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub aliases: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub user_invocable: bool,
    pub context: String,
    pub loaded_from: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    pub has_script: bool,
    pub has_path_gate: bool,
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub struct SkillDetailInfo {
    pub summary: SkillInfo,
    pub markdown: String,
    pub detail_path: String,
}

#[derive(Deserialize)]
pub struct SkillStateUpdateRequest {
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct MetaInfo {
    pub name: String,
    pub version: String,
    pub channel: String,
    pub supports_imessage: bool,
    pub api_version: String,
    pub capabilities: Vec<String>,
    pub deployment_mode: String,
}

#[derive(Serialize)]
pub struct SseTicketResponse {
    pub ticket: String,
    pub expires_at: String,
}

#[derive(Serialize)]
pub struct CronJobRecord {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    #[serde(flatten)]
    pub job: CronJob,
}

#[derive(Serialize)]
pub struct CronJobDetailRecord {
    #[serde(flatten)]
    pub job: CronJobRecord,
    pub executions: Vec<CronJobExecutionRecord>,
}

#[derive(Deserialize)]
pub struct CronJobUpsertRequest {
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub channel_scope: Option<String>,
    pub name: Option<String>,
    pub hour: Option<u32>,
    pub minute: Option<u32>,
    pub repeat: Option<String>,
    pub weekday: Option<u32>,
    pub task_prompt: Option<String>,
    pub push: Option<Value>,
    pub enabled: Option<bool>,
    pub channel_target: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct PortfolioHoldingRequest {
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub channel_scope: Option<String>,
    pub symbol: Option<String>,
    pub asset_type: Option<String>,
    pub shares: Option<f64>,
    pub avg_cost: Option<f64>,
    pub quantity: Option<f64>,
    pub cost_basis: Option<f64>,
    pub underlying: Option<String>,
    pub option_type: Option<String>,
    pub strike_price: Option<f64>,
    pub expiration_date: Option<String>,
    pub contract_multiplier: Option<f64>,
    #[serde(alias = "horizon")]
    pub holding_horizon: Option<String>,
    #[serde(alias = "strategy")]
    pub strategy_notes: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct PortfolioSummary {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub holdings_count: usize,
    pub total_shares: f64,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct ChannelProcessInfo {
    pub pid: u32,
    pub running: bool,
    pub started_at: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub managed_by_desktop: Option<bool>,
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct ChannelStatusInfo {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub running: bool,
    pub status: String,
    pub pid: Option<u32>,
    pub last_heartbeat_at: Option<String>,
    pub detail: String,
    pub processes: Vec<ChannelProcessInfo>,
}
