//! KB 股票信息分析 — 使用统一 runner contract 提取公司/股票信息
//!
//! 每次 KB 附件保存成功后，由渠道 bin 以 fire-and-forget 方式调用此模块。
//! Agent 配置：
//!   - system_prompt：KB 分析专用（不使用全局 config 的 system_prompt）
//!   - tools：仅 web_search

use async_trait::async_trait;
use hone_core::agent::AgentContext;
use hone_memory::{KbEntry, RelatedFileRef, StockTableStorage};
use hone_tools::{ToolRegistry, WebSearchTool};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use crate::HoneBotCore;
use crate::agent_session::GeminiStreamOptions;
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest};
use crate::sandbox::ensure_task_sandbox;

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

/// Agent 输出 JSON 数组中的单个条目
#[derive(Debug, Deserialize)]
struct ExtractedCompany {
    company_name: String,
    #[serde(default)]
    stock_code: String,
    #[serde(default)]
    summary: String,
}

const KB_ANALYSIS_SYSTEM_PROMPT: &str = r#"你是一名专业的金融文档分析助手。

你的任务是：
1. 仔细阅读用户提供的文档内容（包括文件名）
2. 识别文档中涉及的所有公司名称及其股票代码
3. 如果文档中未直接提供股票代码，可使用 web_search 工具查询（例如搜索 "公司名 stock ticker"）
4. 用一句话（不超过 60 字）概括该文档与每家公司的关系

输出格式要求（严格 JSON 数组，不要有任何 markdown 包裹或其他说明文字）：
[
  {
    "company_name": "公司全称",
    "stock_code": "股票代码（如 AAPL、600519.SH，不确定则为空字符串）",
    "summary": "一句话摘要"
  }
]

如果文档不涉及任何上市公司，输出空数组：[]
"#;

/// 从 agent 响应文本中提取 JSON 数组
fn parse_extracted_companies(content: &str) -> Vec<ExtractedCompany> {
    // 尝试提取 ```json ... ``` 块
    let json_str = if let Some(start) = content.find("```json") {
        let after = &content[start + 7..];
        if let Some(end) = after.find("```") {
            after[..end].trim()
        } else {
            content.trim()
        }
    } else if let Some(start) = content.find('[') {
        // 直接找第一个 '[' 到最后一个 ']'
        if let Some(end) = content.rfind(']') {
            &content[start..=end]
        } else {
            content.trim()
        }
    } else {
        content.trim()
    };

    match serde_json::from_str::<Vec<ExtractedCompany>>(json_str) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "[KB/Analysis] JSON 解析失败: {e} | raw={}",
                &content[..content.len().min(200)]
            );
            Vec::new()
        }
    }
}

/// 截取 parsed_text 前 N 字符（避免超 LLM context）
fn truncate_text(text: &str, max_chars: usize) -> &str {
    let mut idx = 0;
    for (i, _) in text.char_indices() {
        if idx >= max_chars {
            return &text[..i];
        }
        idx += 1;
    }
    text
}

/// 主入口：对 KB 条目运行分析 agent，提取公司/股票信息并 upsert 到 stock_table
///
/// 走统一 runner contract；若 parsed_text 为空，则静默跳过。
///
/// 返回 `true` 表示分析 agent 执行完成（无论是否提取到公司），调用方应据此调 `mark_analyzed`；
/// 返回 `false` 表示跳过或 agent 出错，不应标记已分析。
pub async fn run_kb_analysis(
    core: &HoneBotCore,
    entry: &KbEntry,
    parsed_text: &str,
    stock_storage: &StockTableStorage,
) -> bool {
    if parsed_text.trim().is_empty() {
        info!("[KB/Analysis] 跳过（无解析文本）: id={}", entry.id);
        return false;
    }

    if !core.runner_supports_strict_actor_sandbox() {
        warn!(
            "[KB/Analysis] 当前 runner 不支持严格 sandbox，跳过分析: id={} runner={}",
            entry.id, core.config.agent.runner
        );
        return false;
    }

    info!(
        "[KB/Analysis] 开始分析: id={} filename={}",
        entry.id, entry.filename
    );

    // 只注册 web_search
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(WebSearchTool::from_config(&core.config)));

    // 构造用户输入：文件名 + 截断后的文本（前 8000 字）
    let text_snippet = truncate_text(parsed_text, 8000);
    let user_input = format!("文件名：{}\n\n文档内容：\n{}", entry.filename, text_snippet);

    let session_id = format!("kb_analysis_{}", entry.id);
    let ctx = AgentContext::new(session_id.clone());
    let actor = HoneBotCore::create_actor("kb_analysis", &entry.filename, None::<&str>)
        .expect("kb_analysis actor should be valid");
    let runner = match core.create_runner(KB_ANALYSIS_SYSTEM_PROMPT, registry) {
        Ok(r) => r,
        Err(err) => {
            warn!(
                "[KB/Analysis] create_runner 失败，跳过分析: id={} error={}",
                entry.id, err
            );
            return false;
        }
    };
    let working_directory = match ensure_task_sandbox("kb-analysis", &entry.id) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            warn!(
                "[KB/Analysis] sandbox 初始化失败: id={} err={err}",
                entry.id
            );
            return false;
        }
    };
    let response = runner
        .run(
            AgentRunnerRequest {
                session_id,
                actor_label: entry.filename.clone(),
                actor,
                channel_target: "kb_analysis".to_string(),
                allow_cron: false,
                config_path: crate::core::runtime_config_path(),
                system_prompt: KB_ANALYSIS_SYSTEM_PROMPT.to_string(),
                runtime_input: user_input,
                context: ctx,
                timeout: Some(Duration::from_secs(
                    core.config.llm.openrouter.timeout.max(60),
                )),
                gemini_stream: GeminiStreamOptions::default(),
                session_metadata: HashMap::new(),
                working_directory,
                allowed_tools: None,
                max_tool_calls: None,
            },
            Arc::new(NoopEmitter),
        )
        .await
        .response;

    if !response.success && response.content.is_empty() {
        warn!(
            "[KB/Analysis] Agent 执行失败: id={} error={:?}",
            entry.id, response.error
        );
        return false;
    }

    let companies = parse_extracted_companies(&response.content);
    if companies.is_empty() {
        info!("[KB/Analysis] 未提取到公司信息: id={}", entry.id);
        // 分析完成但无公司 → 仍算"已分析"，避免用户反复同步
        return true;
    }

    for company in &companies {
        if company.company_name.trim().is_empty() {
            continue;
        }
        let file_ref = RelatedFileRef {
            kb_id: entry.id.clone(),
            filename: entry.filename.clone(),
            summary: company.summary.clone(),
        };
        if let Err(e) = stock_storage
            .upsert(&company.company_name, &company.stock_code, file_ref)
            .await
        {
            warn!("[KB/Analysis] upsert 失败: {e}");
        }
    }

    info!(
        "[KB/Analysis] 分析完成: id={} companies={}",
        entry.id,
        companies.len()
    );
    true
}
