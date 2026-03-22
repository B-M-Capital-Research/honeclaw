//! KbSearchTool — 知识记忆查询工具
//!
//! 提供两个操作：
//! - `search`：按公司名称或股票代码检索 stock_table.json，返回匹配的公司行及各文件摘要
//! - `load_file`：按 kb_id 加载对应附件的完整解析文本（适合需要深度阅读时使用）

use std::collections::HashSet;
use std::path::PathBuf;

use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_memory::{KbStorage, StockRow, StockTableStorage};
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// 知识记忆查询 & 编辑工具
pub struct KbSearchTool {
    kb_dir: PathBuf,
    actor: Option<ActorIdentity>,
    isolate_by_actor: bool,
    stock_table: StockTableStorage,
}

impl KbSearchTool {
    /// `kb_dir` 通常为 `data/kb`
    pub fn new(kb_dir: PathBuf, actor: Option<ActorIdentity>, isolate_by_actor: bool) -> Self {
        let stock_table = StockTableStorage::new(&kb_dir);
        Self {
            kb_dir,
            actor,
            isolate_by_actor,
            stock_table,
        }
    }
}

#[async_trait]
impl Tool for KbSearchTool {
    fn name(&self) -> &str {
        "kb_search"
    }

    fn description(&self) -> &str {
        "知识记忆操作工具。\
        action=search：按公司名称或股票代码检索知识表，返回相关文件列表、摘要及重点知识；\
        action=load_file：按 kb_id 加载某个文件的完整解析文本（仅在摘要不足时使用）；\
        action=update_knowledge：向指定标的追加一条重点知识（用于录入用户明确表达的重要信息）。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型：search / load_file / update_knowledge".to_string(),
                required: true,
                r#enum: Some(vec!["search".into(), "load_file".into(), "update_knowledge".into()]),
                items: None,
            },
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "搜索关键词：公司名称（中英文均可）或股票代码（action=search 时必填）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "kb_id".to_string(),
                param_type: "string".to_string(),
                description: "知识库条目 ID，来自 search 结果中的 related_files[].kb_id（action=load_file 时必填）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "company_name".to_string(),
                param_type: "string".to_string(),
                description: "公司/标的名称（action=update_knowledge 时用于定位或新建标的行）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "stock_code".to_string(),
                param_type: "string".to_string(),
                description: "股票代码，如 AAPL、600519.SH（action=update_knowledge 时优先用于定位标的）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "knowledge_text".to_string(),
                param_type: "string".to_string(),
                description: "要录入的重点知识内容（单条文本，action=update_knowledge 时必填）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "search" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if query.is_empty() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "action=search 时 query 不能为空"
                    }));
                }

                let session_id = if self.isolate_by_actor {
                    match self.actor.as_ref() {
                        Some(actor) => Some(actor.session_id()),
                        None => {
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": "缺少 actor 身份，无法进行隔离查询"
                            }));
                        }
                    }
                } else {
                    None
                };

                let stock_table_path = self.kb_dir.join("stock_table.json");
                let raw = match tokio::fs::read_to_string(&stock_table_path).await {
                    Ok(s) => s,
                    Err(_) => {
                        return Ok(serde_json::json!({
                            "success": true,
                            "query": query,
                            "matches": [],
                            "note": "知识记忆表尚未生成，请先对知识库文件执行「同步到知识」操作"
                        }));
                    }
                };

                let rows: Vec<StockRow> =
                    serde_json::from_str::<Vec<StockRow>>(&raw).unwrap_or_default();

                let allowed_ids: Option<HashSet<String>> = if self.isolate_by_actor {
                    let kb_storage = KbStorage::new(&self.kb_dir);
                    let entries = kb_storage.list_entries().await;
                    let allow: HashSet<String> = entries
                        .into_iter()
                        .filter(|entry| entry.session_id == session_id.as_deref().unwrap_or(""))
                        .map(|entry| entry.id)
                        .collect();
                    Some(allow)
                } else {
                    None
                };

                let mut visible_rows: Vec<StockRow> = Vec::new();
                for mut row in rows {
                    if let Some(allow) = &allowed_ids {
                        row.related_files.retain(|file| allow.contains(&file.kb_id));
                        if row.related_files.is_empty() {
                            continue;
                        }
                    }
                    visible_rows.push(row);
                }

                let query_lower = query.to_lowercase();
                let matches: Vec<StockRow> = visible_rows
                    .iter()
                    .filter(|row| {
                        let company = row.company_name.to_lowercase();
                        let code = row.stock_code.to_lowercase();
                        company.contains(&query_lower) || code.contains(&query_lower)
                    })
                    .cloned()
                    .collect();

                let total = visible_rows.len();
                let matched = matches.len();

                Ok(serde_json::json!({
                    "success": true,
                    "query": query,
                    "total_companies_in_table": total,
                    "matched": matched,
                    "matches": serde_json::to_value(&matches).unwrap_or_default(),
                    "hint": if matched == 0 {
                        "未找到匹配条目。可尝试不同的关键词（中文公司名或英文股票代码）。如知识表条目较少，可在知识库页面上传更多文件并同步。"
                    } else {
                        "找到匹配条目。请查阅 related_files[].summary 判断是否需要加载全文（action=load_file）。"
                    }
                }))
            }

            "load_file" => {
                let kb_id = args
                    .get("kb_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if kb_id.is_empty() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "action=load_file 时 kb_id 不能为空"
                    }));
                }

                // 防止路径穿越
                if kb_id.contains('/') || kb_id.contains('\\') || kb_id.contains("..") {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "kb_id 包含非法字符"
                    }));
                }

                let kb_storage = KbStorage::new(&self.kb_dir);
                let Some(entry) = kb_storage.get_entry(&kb_id).await else {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("KB 条目 '{kb_id}' 不存在或无法读取")
                    }));
                };

                if self.isolate_by_actor {
                    let Some(session_id) = self.actor.as_ref().map(|actor| actor.session_id())
                    else {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": "缺少 actor 身份，无法进行隔离查询"
                        }));
                    };
                    if entry.session_id != session_id {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": "无权访问该 KB 条目"
                        }));
                    }
                }

                let filename = entry.filename.clone();
                let parse_status = entry.parse_status.clone();

                // 读取解析文本
                let parsed_path = self.kb_dir.join(&kb_id).join("parsed.txt");
                let raw_text = match tokio::fs::read_to_string(&parsed_path).await {
                    Ok(s) => s,
                    Err(_) => {
                        return Ok(serde_json::json!({
                            "success": false,
                            "kb_id": kb_id,
                            "filename": filename,
                            "parse_status": parse_status,
                            "error": "该文件无解析文本（可能是图片/音视频，或解析失败）"
                        }));
                    }
                };

                const MAX_CHARS: usize = 20_000;
                let (text, truncated) = if raw_text.chars().count() > MAX_CHARS {
                    let truncated_text: String = raw_text.chars().take(MAX_CHARS).collect();
                    (truncated_text, true)
                } else {
                    (raw_text.clone(), false)
                };

                let total_chars = raw_text.chars().count();

                Ok(serde_json::json!({
                    "success": true,
                    "kb_id": kb_id,
                    "filename": filename,
                    "parse_status": parse_status,
                    "total_chars": total_chars,
                    "truncated": truncated,
                    "truncated_at": if truncated { MAX_CHARS } else { total_chars },
                    "text": text,
                    "hint": if truncated {
                        format!("文本已截断至前 {MAX_CHARS} 字符（原文共 {total_chars} 字符）。如需后续内容，可再次调用并说明需要后半部分。")
                    } else {
                        "已返回完整解析文本。".to_string()
                    }
                }))
            }

            "update_knowledge" => {
                let company_name = args
                    .get("company_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let stock_code = args
                    .get("stock_code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let knowledge_text = args
                    .get("knowledge_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if company_name.is_empty() && stock_code.is_empty() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "company_name 和 stock_code 至少填一个"
                    }));
                }
                if knowledge_text.is_empty() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "knowledge_text 不能为空"
                    }));
                }

                match self
                    .stock_table
                    .append_knowledge(&company_name, &stock_code, knowledge_text.clone())
                    .await
                {
                    Ok(()) => Ok(serde_json::json!({
                        "success": true,
                        "message": format!(
                            "已将重点知识录入「{}{}」",
                            if !company_name.is_empty() { company_name.as_str() } else { stock_code.as_str() },
                            if !stock_code.is_empty() && !company_name.is_empty() {
                                format!("（{}）", stock_code)
                            } else {
                                String::new()
                            }
                        ),
                        "knowledge_text": knowledge_text
                    })),
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("写入失败: {e}")
                    })),
                }
            }

            _ => Ok(serde_json::json!({
                "success": false,
                "error": format!("不支持的操作 '{action}'，可用操作：search / load_file / update_knowledge")
            })),
        }
    }
}
