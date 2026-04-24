//! 管理员口令注册 + `/report` 研报 workflow bridge 的拦截层。
//!
//! 这两个功能在语义上是「聊天里敲一条特殊命令 → HoneBot 代理一次
//! HTTP 调用或内部状态翻转 → 回一条用户可读文案」。两者都不走 LLM
//! runner,也不进 session 消息流,属于独立「bot → workflow service」
//! 的 command bridge,所以整体从 `bot_core.rs` 抽出来放这里。
//!
//! 结构:
//! - `impl HoneBotCore { try_intercept_admin_registration / try_handle_intercept_command /
//!   handle_report_* / workflow_runner_* }`:对外入口 + HTTP 调用;
//! - 顶层 helper (`parse_*` / `format_*` / `build_report_run_input` / `constant_time_str_eq` 等):
//!   纯函数,测试里直接断言;
//! - `WorkflowRun*` / `ReportIntercept` / `WorkflowRunnerHttpResponse`:只在本模块内用的
//!   数据类型,保持为 `pub(super)` 不外泄。

use hone_core::ActorIdentity;
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde_json::json;
use subtle::ConstantTimeEq;
use tokio::process::Command;

use super::bot_core::HoneBotCore;
use super::logging::truncate_for_log;

pub const REGISTER_ADMIN_INTERCEPT_PREFIX: &str = "/register-admin";
pub const REGISTER_ADMIN_INTERCEPT_ACK: &str = "已将当前 identity 升级为管理员。";
pub const REGISTER_ADMIN_INTERCEPT_DENY_ACK: &str =
    "管理员注册失败：当前 identity 不在 admins 白名单中。";
pub const REGISTER_ADMIN_INTERCEPT_DISABLED_ACK: &str =
    "管理员注册失败：当前未配置 runtime 管理员注册口令。";
pub const REGISTER_ADMIN_INTERCEPT_INVALID_ACK: &str = "管理员注册失败：口令无效。";

pub(super) const REPORT_INTERCEPT_PREFIX: &str = "/report";
pub(super) const REPORT_PROGRESS_COMMAND: &str = "进度";
pub(super) const REPORT_PROGRESS_COMMAND_ALIAS: &str = "progress";
pub(super) const REPORT_WORKFLOW_ID: &str = "company_report";
pub(super) const REPORT_DEFAULT_MODE: &str = "全跑完-美";
pub(super) const REPORT_DEFAULT_RESEARCH_TOPIC: &str = "新闻";

impl HoneBotCore {
    pub fn try_intercept_admin_registration(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        let Some(passphrase) = parse_admin_registration_passphrase(input) else {
            return None;
        };

        if !self.is_admin(&actor.user_id, &actor.channel) {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=not_whitelisted",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_DENY_ACK.to_string());
        }

        let expected = self
            .config
            .admins
            .resolved_runtime_admin_registration_passphrase();
        if expected.is_empty() {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=passphrase_disabled",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_DISABLED_ACK.to_string());
        }

        if !constant_time_str_eq(&passphrase, &expected) {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=invalid_passphrase",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_INVALID_ACK.to_string());
        }

        let inserted = self
            .runtime_admin_overrides
            .write()
            .map(|mut overrides| overrides.insert(actor.clone()))
            .unwrap_or(false);

        tracing::warn!(
            "[HoneBotCore] runtime_admin_override actor={} inserted={}",
            actor.session_id(),
            inserted
        );
        Some(REGISTER_ADMIN_INTERCEPT_ACK.to_string())
    }

    pub async fn try_handle_intercept_command(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        if let Some(reply) = self.try_intercept_admin_registration(actor, input) {
            return Some(reply);
        }

        match parse_report_intercept(input) {
            Some(ReportIntercept::Start { company_name }) => {
                Some(self.handle_report_start(actor, &company_name).await)
            }
            Some(ReportIntercept::Progress) => Some(self.handle_report_progress(actor).await),
            None => None,
        }
    }

    async fn handle_report_start(&self, actor: &ActorIdentity, company_name: &str) -> String {
        let Some(base_url) = self.workflow_runner_base_url() else {
            return "未配置本地 workflow runner 地址，暂时无法启动研报任务。".to_string();
        };

        let request_body = build_report_run_input(company_name);
        let url = format!("{base_url}/api/runs");
        let validate_code = self.config.web.resolved_local_workflow_validate_code();
        let mut run_payload = json!({
            "workflowId": REPORT_WORKFLOW_ID,
            "input": request_body,
            "promptOverrides": {},
        });
        if !validate_code.is_empty() {
            run_payload["validateCode"] = serde_json::Value::String(validate_code);
        }
        let response = match self
            .workflow_runner_request(Method::POST, &url, Some(run_payload))
            .await
        {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    "[HoneBotCore] report start request failed actor={} error={}",
                    actor.session_id(),
                    err
                );
                return format!("研报任务启动失败：无法连接本地 workflow runner（{err}）。");
            }
        };

        if response.status == StatusCode::CONFLICT {
            let conflict = serde_json::from_str::<WorkflowConflictResponse>(&response.body).ok();
            if let Some(active_run_id) = conflict
                .as_ref()
                .and_then(|value| value.active_run_id.as_deref())
            {
                if let Ok(progress) = self.fetch_report_progress_by_run_id(active_run_id).await {
                    return format!(
                        "已有研报任务正在运行中：{}",
                        format_progress_message(&progress)
                    );
                }
            }

            let detail = conflict
                .and_then(|value| value.error)
                .unwrap_or_else(|| "已有研报任务正在运行中。".to_string());
            return format!("研报任务未重复启动：{detail}");
        }

        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return format!("研报任务启动失败：{status} {detail}");
        }

        match serde_json::from_str::<WorkflowRunCreatedResponse>(&response.body) {
            Ok(payload) => format!(
                "已启动公司研报：{}。研究倾向默认使用“{}”，任务正在运行中（run_id={}）。可发送 `/report 进度` 查看进度。",
                company_name.trim(),
                REPORT_DEFAULT_RESEARCH_TOPIC,
                payload.id
            ),
            Err(err) => format!("研报任务已提交，但解析启动响应失败：{err}"),
        }
    }

    async fn handle_report_progress(&self, actor: &ActorIdentity) -> String {
        let Some(base_url) = self.workflow_runner_base_url() else {
            return "未配置本地 workflow runner 地址，暂时无法查询研报进度。".to_string();
        };

        let url = format!("{base_url}/api/runs?workflowId={REPORT_WORKFLOW_ID}&limit=1");
        let response = match self.workflow_runner_request(Method::GET, &url, None).await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    "[HoneBotCore] report progress request failed actor={} error={}",
                    actor.session_id(),
                    err
                );
                return format!("查询研报进度失败：无法连接本地 workflow runner（{err}）。");
            }
        };

        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return format!("查询研报进度失败：{status} {detail}");
        }

        let payload = match serde_json::from_str::<WorkflowRunListResponse>(&response.body) {
            Ok(payload) => payload,
            Err(err) => return format!("查询研报进度失败：响应解析错误（{err}）。"),
        };

        let Some(run) = payload.runs.into_iter().next() else {
            return "当前还没有可查询的研报任务。可直接发送 `/report 公司名` 启动。".to_string();
        };

        if run.status == "running" {
            match self.fetch_report_progress_by_run_id(&run.id).await {
                Ok(progress) => format_progress_message(&progress),
                Err(err) => format!(
                    "研报任务正在运行中（run_id={}），但拉取实时进度失败：{}",
                    run.id, err
                ),
            }
        } else {
            format_recent_report_message(&run)
        }
    }

    fn workflow_runner_base_url(&self) -> Option<String> {
        let base = self.config.web.local_workflow_api_base.trim();
        if base.is_empty() {
            None
        } else {
            Some(base.trim_end_matches('/').to_string())
        }
    }

    async fn fetch_report_progress_by_run_id(
        &self,
        run_id: &str,
    ) -> Result<WorkflowProgressEnvelope, String> {
        let base_url = self
            .workflow_runner_base_url()
            .ok_or_else(|| "未配置本地 workflow runner 地址".to_string())?;
        let url = format!("{base_url}/api/runs/{run_id}/progress");
        let response = self
            .workflow_runner_request(Method::GET, &url, None)
            .await
            .map_err(|err| err.to_string())?;
        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return Err(format!("{status} {detail}"));
        }
        serde_json::from_str::<WorkflowProgressEnvelope>(&response.body)
            .map_err(|err| err.to_string())
    }

    async fn workflow_runner_request(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<WorkflowRunnerHttpResponse, String> {
        let mut request = self.workflow_runner_http.request(method.clone(), url);
        if let Some(payload) = body.as_ref() {
            request = request.json(payload);
        }

        match request.send().await {
            Ok(response) => WorkflowRunnerHttpResponse::from_reqwest(response).await,
            Err(err) => {
                if should_fallback_to_curl(url) {
                    tracing::warn!(
                        "[HoneBotCore] workflow runner reqwest failed, falling back to curl url={} error={:?}",
                        url,
                        err
                    );
                    self.workflow_runner_request_via_curl(method, url, body)
                        .await
                } else {
                    Err(err.to_string())
                }
            }
        }
    }

    /// 本地 loopback (`127.0.0.1` / `localhost` / `[::1]`) 时的 reqwest
    /// fallback:Mac 上偶发 `connection refused` 能用 curl 成功,原因怀疑是
    /// reqwest 默认走 IPv4/IPv6 dual-stack 选错地址,留一层「用 curl 再试」
    /// 来兜底。非 loopback 不触发,避免隐式 shell out。
    async fn workflow_runner_request_via_curl(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<WorkflowRunnerHttpResponse, String> {
        let mut command = Command::new("curl");
        command.arg("-sS");
        command.arg("-X").arg(method.as_str());
        command.arg(url);
        command.arg("-H").arg("Accept: application/json");
        command.arg("-w").arg("\n__HONE_STATUS__:%{http_code}");

        if let Some(payload) = body {
            let payload_text = serde_json::to_string(&payload).map_err(|err| err.to_string())?;
            command.arg("-H").arg("Content-Type: application/json");
            command.arg("--data").arg(payload_text);
        }

        let output = command.output().await.map_err(|err| err.to_string())?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "curl exited with status {}: {}",
                output.status,
                truncate_for_log(stderr.trim(), 240)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let (body_text, status_text) = stdout
            .rsplit_once("\n__HONE_STATUS__:")
            .ok_or_else(|| "curl response missing HTTP status marker".to_string())?;
        let status_code = status_text
            .trim()
            .parse::<u16>()
            .map_err(|err| format!("invalid curl status code: {err}"))?;
        let status = StatusCode::from_u16(status_code)
            .map_err(|err| format!("unsupported HTTP status code {status_code}: {err}"))?;

        Ok(WorkflowRunnerHttpResponse {
            status,
            body: body_text.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ReportIntercept {
    Start { company_name: String },
    Progress,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunCreatedResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowConflictResponse {
    error: Option<String>,
    active_run_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunListResponse {
    runs: Vec<WorkflowRunSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunSummary {
    id: String,
    workflow_id: String,
    workflow_name: Option<String>,
    status: String,
    ended_at: Option<String>,
    error: Option<String>,
    progress: Option<WorkflowProgressSnapshot>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowProgressEnvelope {
    id: String,
    progress: WorkflowProgressSnapshot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowProgressSnapshot {
    total_nodes: u32,
    terminal_nodes: u32,
    running_nodes: u32,
    pending_nodes: u32,
    percent: f64,
    #[serde(default)]
    active_nodes: Vec<WorkflowActiveNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowActiveNode {
    workflow_name: Option<String>,
    title: Option<String>,
    id: String,
}

#[derive(Debug, Clone)]
struct WorkflowRunnerHttpResponse {
    status: StatusCode,
    body: String,
}

impl WorkflowRunnerHttpResponse {
    async fn from_reqwest(response: reqwest::Response) -> Result<Self, String> {
        let status = response.status();
        let body = response.text().await.map_err(|err| err.to_string())?;
        Ok(Self { status, body })
    }
}

pub(super) fn build_report_run_input(company_name: &str) -> serde_json::Value {
    json!({
        "companyName": company_name.trim(),
        "genPost": REPORT_DEFAULT_MODE,
        "news": "",
        "task_id": "",
        "research_topic": REPORT_DEFAULT_RESEARCH_TOPIC,
    })
}

/// 剥掉单/双引号包裹(有些 shell 粘贴进来会带一层),
/// 再 trim。专门给拦截命令判定用,避免 `'/report 进度'` 这类误伤。
fn normalize_intercept_input(input: &str) -> String {
    let trimmed = input.trim();
    let normalized = trimmed
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            trimmed
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
        .unwrap_or(trimmed)
        .trim();
    normalized.to_string()
}

fn parse_admin_registration_passphrase(input: &str) -> Option<String> {
    let normalized = normalize_intercept_input(input);
    let remainder = normalized
        .strip_prefix(REGISTER_ADMIN_INTERCEPT_PREFIX)?
        .trim();
    if remainder.is_empty() {
        return None;
    }
    Some(remainder.to_string())
}

#[cfg(test)]
pub(super) fn matches_register_admin_intercept(input: &str) -> bool {
    parse_admin_registration_passphrase(input).is_some()
}

/// 用 `subtle::ConstantTimeEq` 做常数时间字符串比较。注册口令的比较
/// 走这里,避免被 timing side-channel 推断出前缀。
fn constant_time_str_eq(left: &str, right: &str) -> bool {
    left.as_bytes().ct_eq(right.as_bytes()).into()
}

pub(super) fn parse_report_intercept(input: &str) -> Option<ReportIntercept> {
    let normalized = normalize_intercept_input(input);
    let remainder = normalized.strip_prefix(REPORT_INTERCEPT_PREFIX)?.trim();
    if remainder.is_empty() {
        return None;
    }
    if remainder == REPORT_PROGRESS_COMMAND
        || remainder.eq_ignore_ascii_case(REPORT_PROGRESS_COMMAND_ALIAS)
    {
        return Some(ReportIntercept::Progress);
    }
    Some(ReportIntercept::Start {
        company_name: remainder.to_string(),
    })
}

fn format_progress_message(progress: &WorkflowProgressEnvelope) -> String {
    let active = summarize_active_nodes(&progress.progress.active_nodes);
    format!(
        "研报任务正在运行中：{:.1}%（{}/{} 节点已进入终态，{} 个节点运行中，{} 个节点待执行）。{} run_id={}",
        progress.progress.percent,
        progress.progress.terminal_nodes,
        progress.progress.total_nodes,
        progress.progress.running_nodes,
        progress.progress.pending_nodes,
        active,
        progress.id
    )
}

fn format_recent_report_message(run: &WorkflowRunSummary) -> String {
    let workflow_name = run
        .workflow_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&run.workflow_id);
    let progress = run.progress.as_ref();
    let percent = progress.map(|value| value.percent).unwrap_or(0.0);
    let terminal_nodes = progress.map(|value| value.terminal_nodes).unwrap_or(0);
    let total_nodes = progress.map(|value| value.total_nodes).unwrap_or(0);
    let status_label = match run.status.as_str() {
        "succeeded" => "已完成",
        "failed" => "已失败",
        "stopped" => "已停止",
        other => other,
    };
    let mut message = format!(
        "当前没有运行中的研报任务。最近一次任务：{}（{}，{:.1}% ，{}/{} 节点终态，run_id={}）。",
        workflow_name, status_label, percent, terminal_nodes, total_nodes, run.id
    );
    if let Some(ended_at) = run
        .ended_at
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        message.push_str(&format!(" 结束时间：{ended_at}。"));
    }
    if let Some(error) = run
        .error
        .as_deref()
        .map(first_non_empty_line)
        .filter(|value| !value.is_empty())
    {
        message.push_str(&format!(" 错误：{}。", truncate_for_log(&error, 120)));
    }
    message
}

fn summarize_active_nodes(nodes: &[WorkflowActiveNode]) -> String {
    if nodes.is_empty() {
        return "当前没有活跃节点。".to_string();
    }
    let labels = nodes
        .iter()
        .take(3)
        .map(|node| {
            let workflow_name = node
                .workflow_name
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("-");
            let title = node
                .title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(&node.id);
            format!("{workflow_name}/{title}")
        })
        .collect::<Vec<_>>();
    if nodes.len() > 3 {
        format!(
            "当前活跃节点：{} 等 {} 个。",
            labels.join("、"),
            nodes.len()
        )
    } else {
        format!("当前活跃节点：{}。", labels.join("、"))
    }
}

fn first_non_empty_line(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

fn should_fallback_to_curl(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://localhost:")
        || url.starts_with("http://[::1]:")
}
