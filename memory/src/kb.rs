//! 知识库存储 — 附件原件 + 解析文本 + 索引
//!
//! 目录结构：
//! ```text
//! data/kb/
//!   <uuid>/
//!     index.json      # 元数据
//!     original.<ext>  # 原件文件（复制）
//!     parsed.txt      # 完整提取文本（无字数上限）
//!   stock_table.json  # 全局股票信息表（按公司/代码去重）
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

/// 知识库条目元数据（对应 index.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbEntry {
    pub id: String,
    pub filename: String,
    /// 附件分类标签（"Pdf" / "Image" / "Text" 等）
    pub kind: String,
    pub size: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub channel: String,
    pub user_id: String,
    pub session_id: String,
    pub uploaded_at: String,
    /// 相对于项目根目录的原件路径
    pub original_path: String,
    /// 相对于项目根目录的解析文本路径
    pub parsed_path: String,
    /// "ok" | "failed" | "empty" | "skipped"
    pub parse_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
    /// 最近一次成功同步到知识表的时间（ISO 8601），None 表示从未同步
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyzed_at: Option<String>,
}

/// 保存请求（由 channel bin 填充所有字段后传入）
pub struct KbSaveRequest {
    pub filename: String,
    /// 附件分类标签
    pub kind: String,
    pub size: u32,
    pub content_type: Option<String>,
    pub channel: String,
    pub user_id: String,
    pub session_id: String,
    /// 已下载到本地的文件绝对路径
    pub source_path: PathBuf,
    /// 完整解析文本（无截断），None 表示未提取或不适用
    pub parsed_text: Option<String>,
    /// 提取失败时的错误信息
    pub parse_error: Option<String>,
}

/// 知识库存储管理器
#[derive(Clone)]
pub struct KbStorage {
    kb_dir: PathBuf,
}

impl KbStorage {
    /// 创建实例，`kb_dir` 通常为 `data/kb`
    pub fn new(kb_dir: impl AsRef<Path>) -> Self {
        let dir = kb_dir.as_ref().to_path_buf();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("[KbStorage] 创建 KB 目录失败: {e}");
        }
        Self { kb_dir: dir }
    }

    /// 保存一个附件到知识库，返回生成的 KbEntry
    pub async fn save_attachment(&self, req: KbSaveRequest) -> Result<KbEntry, String> {
        let id = Uuid::new_v4().to_string();
        let entry_dir = self.kb_dir.join(&id);
        tokio::fs::create_dir_all(&entry_dir)
            .await
            .map_err(|e| format!("创建 KB 条目目录失败: {e}"))?;

        // 确定原件目标文件名（保留扩展名）
        let ext = Path::new(&req.filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        let original_filename = format!("original.{ext}");
        let original_dest = entry_dir.join(&original_filename);

        tokio::fs::copy(&req.source_path, &original_dest)
            .await
            .map_err(|e| format!("复制原件失败: {e}"))?;

        // 写解析文本
        let parsed_filename = "parsed.txt";
        let parsed_dest = entry_dir.join(parsed_filename);
        let parse_status;
        let parse_error;
        if let Some(ref text) = req.parsed_text {
            tokio::fs::write(&parsed_dest, text.as_bytes())
                .await
                .map_err(|e| format!("写解析文本失败: {e}"))?;
            parse_status = "ok".to_string();
            parse_error = None;
        } else if let Some(ref err) = req.parse_error {
            // 把错误信息也写进去方便排查
            let _ = tokio::fs::write(&parsed_dest, err.as_bytes()).await;
            parse_status = "failed".to_string();
            parse_error = Some(err.clone());
        } else {
            parse_status = "skipped".to_string();
            parse_error = None;
        }

        let original_path = entry_dir.join(&original_filename).display().to_string();
        let parsed_path = entry_dir.join(parsed_filename).display().to_string();

        let entry = KbEntry {
            id: id.clone(),
            filename: req.filename,
            kind: req.kind,
            size: req.size,
            content_type: req.content_type,
            channel: req.channel,
            user_id: req.user_id,
            session_id: req.session_id,
            uploaded_at: Utc::now().to_rfc3339(),
            original_path,
            parsed_path,
            parse_status,
            parse_error,
            analyzed_at: None,
        };

        // 写 index.json
        let index_path = entry_dir.join("index.json");
        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| format!("序列化 index.json 失败: {e}"))?;
        tokio::fs::write(&index_path, json.as_bytes())
            .await
            .map_err(|e| format!("写 index.json 失败: {e}"))?;

        info!(
            "[KB] 已保存附件: id={} filename={} channel={} user={}",
            id, entry.filename, entry.channel, entry.user_id
        );
        Ok(entry)
    }

    /// 列出所有知识库条目（按上传时间倒序）
    pub async fn list_entries(&self) -> Vec<KbEntry> {
        let mut entries = Vec::new();
        let mut read_dir = match tokio::fs::read_dir(&self.kb_dir).await {
            Ok(rd) => rd,
            Err(_) => return entries,
        };
        while let Ok(Some(de)) = read_dir.next_entry().await {
            let index_path = de.path().join("index.json");
            if !index_path.exists() {
                continue;
            }
            match tokio::fs::read_to_string(&index_path).await {
                Ok(raw) => {
                    if let Ok(entry) = serde_json::from_str::<KbEntry>(&raw) {
                        entries.push(entry);
                    }
                }
                Err(e) => {
                    warn!("[KB] 读取 index.json 失败 {:?}: {e}", index_path);
                }
            }
        }
        // 按上传时间倒序
        entries.sort_by(|a, b| b.uploaded_at.cmp(&a.uploaded_at));
        entries
    }

    /// 列出某个 actor 的知识库条目（按上传时间倒序）
    pub async fn list_entries_for_actor(&self, actor: &ActorIdentity) -> Vec<KbEntry> {
        let session_id = actor.session_id();
        self.list_entries()
            .await
            .into_iter()
            .filter(|entry| entry.session_id == session_id)
            .collect()
    }

    /// 获取单个条目（含 index.json 内容）
    pub async fn get_entry(&self, id: &str) -> Option<KbEntry> {
        // 防止路径穿越
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return None;
        }
        let index_path = self.kb_dir.join(id).join("index.json");
        let raw = tokio::fs::read_to_string(&index_path).await.ok()?;
        serde_json::from_str::<KbEntry>(&raw).ok()
    }

    /// 获取某个 actor 的条目（不匹配则返回 None）
    pub async fn get_entry_for_actor(&self, actor: &ActorIdentity, id: &str) -> Option<KbEntry> {
        let entry = self.get_entry(id).await?;
        if entry.session_id == actor.session_id() {
            Some(entry)
        } else {
            None
        }
    }

    /// 读取某条目的完整解析文本
    pub async fn get_parsed_text(&self, id: &str) -> Option<String> {
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return None;
        }
        let parsed_path = self.kb_dir.join(id).join("parsed.txt");
        tokio::fs::read_to_string(&parsed_path).await.ok()
    }

    /// 标记一个条目为"已分析"（写入 analyzed_at 时间戳）
    pub async fn mark_analyzed(&self, id: &str) -> Result<(), String> {
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err("非法 id".to_string());
        }
        let index_path = self.kb_dir.join(id).join("index.json");
        let raw = tokio::fs::read_to_string(&index_path)
            .await
            .map_err(|e| format!("读取 index.json 失败: {e}"))?;
        let mut entry: KbEntry =
            serde_json::from_str(&raw).map_err(|e| format!("解析 index.json 失败: {e}"))?;
        entry.analyzed_at = Some(Utc::now().to_rfc3339());
        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| format!("序列化 index.json 失败: {e}"))?;
        tokio::fs::write(&index_path, json.as_bytes())
            .await
            .map_err(|e| format!("写 index.json 失败: {e}"))?;
        info!("[KB] 已标记 analyzed_at: id={id}");
        Ok(())
    }

    /// 删除一个知识库条目（目录及其所有内容）
    pub async fn delete_entry(&self, id: &str) -> Result<(), String> {
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err("非法 id".to_string());
        }
        let entry_dir = self.kb_dir.join(id);
        if !entry_dir.exists() {
            return Err("条目不存在".to_string());
        }
        tokio::fs::remove_dir_all(&entry_dir)
            .await
            .map_err(|e| format!("删除条目目录失败: {e}"))?;
        info!("[KB] 已删除条目: id={id}");
        Ok(())
    }

    /// 删除某个 actor 的条目（不匹配则拒绝）
    pub async fn delete_entry_for_actor(
        &self,
        actor: &ActorIdentity,
        id: &str,
    ) -> Result<(), String> {
        let Some(entry) = self.get_entry_for_actor(actor, id).await else {
            return Err("KB 条目不存在或无权访问".to_string());
        };
        self.delete_entry(&entry.id).await
    }
}

// ── 股票信息表 ─────────────────────────────────────────────────────────────────

/// 对应某个 KB 条目的文件引用及摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedFileRef {
    pub kb_id: String,
    pub filename: String,
    /// Agent 提取的一句话摘要
    pub summary: String,
}

/// 股票信息表中的一行（按 stock_code 或 company_name 去重）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockRow {
    pub company_name: String,
    /// 股票代码，若 agent 未能识别则为空字符串
    #[serde(default)]
    pub stock_code: String,
    pub related_files: Vec<RelatedFileRef>,
    /// 用户/AI 手动录入的重点知识条目（每条独立文本）
    #[serde(default)]
    pub key_knowledge: Vec<String>,
    pub updated_at: String,
}

/// 全局股票信息表存储（`data/kb/stock_table.json`）
///
/// 并发安全：内部持有 `Mutex` 保证 read-modify-write 原子性。
#[derive(Clone)]
pub struct StockTableStorage {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl StockTableStorage {
    /// `kb_dir` 即 `data/kb`，stock_table 文件存于其根目录
    pub fn new(kb_dir: impl AsRef<Path>) -> Self {
        Self {
            path: kb_dir.as_ref().join("stock_table.json"),
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// 读取全部行（文件不存在时返回空 Vec）
    pub async fn list(&self) -> Vec<StockRow> {
        match tokio::fs::read_to_string(&self.path).await {
            Ok(raw) => serde_json::from_str::<Vec<StockRow>>(&raw).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Upsert：按 stock_code（非空时）或 company_name 查找已有行；
    /// 若已存在该 kb_id 的文件引用则跳过（幂等），否则追加；若无匹配行则新增。
    pub async fn upsert(
        &self,
        company_name: &str,
        stock_code: &str,
        file_ref: RelatedFileRef,
    ) -> Result<(), String> {
        let _guard = self.lock.lock().await;

        let mut rows = match tokio::fs::read_to_string(&self.path).await {
            Ok(raw) => serde_json::from_str::<Vec<StockRow>>(&raw).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        // 查找匹配行（stock_code 非空时优先按代码匹配，否则按公司名匹配）
        let matched = if !stock_code.is_empty() {
            rows.iter_mut().find(|r| {
                !r.stock_code.is_empty() && r.stock_code.to_uppercase() == stock_code.to_uppercase()
            })
        } else {
            rows.iter_mut()
                .find(|r| r.company_name.to_lowercase() == company_name.to_lowercase())
        };

        if let Some(row) = matched {
            // 幂等：同一 kb_id 不重复写入
            if !row.related_files.iter().any(|f| f.kb_id == file_ref.kb_id) {
                row.related_files.push(file_ref);
                row.updated_at = Utc::now().to_rfc3339();
            }
        } else {
            rows.push(StockRow {
                company_name: company_name.to_string(),
                stock_code: stock_code.to_string(),
                related_files: vec![file_ref],
                key_knowledge: vec![],
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        let json = serde_json::to_string_pretty(&rows)
            .map_err(|e| format!("序列化 stock_table 失败: {e}"))?;
        tokio::fs::write(&self.path, json.as_bytes())
            .await
            .map_err(|e| format!("写 stock_table.json 失败: {e}"))?;

        info!(
            "[KB/StockTable] upsert company={company_name} code={stock_code} kb_id={}",
            rows.last()
                .map(|r| r
                    .related_files
                    .last()
                    .map(|f| f.kb_id.as_str())
                    .unwrap_or("-"))
                .unwrap_or("-")
        );
        Ok(())
    }

    /// 替换某个标的的完整重点知识列表（UI 直接编辑时使用）。
    /// 若标的不存在，则新建一行（仅含知识，无关联文件）。
    pub async fn update_key_knowledge(
        &self,
        company_name: &str,
        stock_code: &str,
        key_knowledge: Vec<String>,
    ) -> Result<(), String> {
        let _guard = self.lock.lock().await;

        let mut rows = match tokio::fs::read_to_string(&self.path).await {
            Ok(raw) => serde_json::from_str::<Vec<StockRow>>(&raw).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        let matched = if !stock_code.is_empty() {
            rows.iter_mut().find(|r| {
                !r.stock_code.is_empty() && r.stock_code.to_uppercase() == stock_code.to_uppercase()
            })
        } else {
            rows.iter_mut()
                .find(|r| r.company_name.to_lowercase() == company_name.to_lowercase())
        };

        if let Some(row) = matched {
            row.key_knowledge = key_knowledge;
            row.updated_at = Utc::now().to_rfc3339();
        } else {
            rows.push(StockRow {
                company_name: company_name.to_string(),
                stock_code: stock_code.to_string(),
                related_files: vec![],
                key_knowledge,
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        let json = serde_json::to_string_pretty(&rows)
            .map_err(|e| format!("序列化 stock_table 失败: {e}"))?;
        tokio::fs::write(&self.path, json.as_bytes())
            .await
            .map_err(|e| format!("写 stock_table.json 失败: {e}"))?;

        info!("[KB/StockTable] update_key_knowledge company={company_name} code={stock_code}");
        Ok(())
    }

    /// 追加单条重点知识（AI 工具调用时使用；幂等：内容完全相同则跳过）。
    pub async fn append_knowledge(
        &self,
        company_name: &str,
        stock_code: &str,
        knowledge_text: String,
    ) -> Result<(), String> {
        let _guard = self.lock.lock().await;

        let mut rows = match tokio::fs::read_to_string(&self.path).await {
            Ok(raw) => serde_json::from_str::<Vec<StockRow>>(&raw).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        let matched = if !stock_code.is_empty() {
            rows.iter_mut().find(|r| {
                !r.stock_code.is_empty() && r.stock_code.to_uppercase() == stock_code.to_uppercase()
            })
        } else {
            rows.iter_mut()
                .find(|r| r.company_name.to_lowercase() == company_name.to_lowercase())
        };

        if let Some(row) = matched {
            // 幂等：相同文本不重复写入
            if !row.key_knowledge.contains(&knowledge_text) {
                row.key_knowledge.push(knowledge_text);
                row.updated_at = Utc::now().to_rfc3339();
            }
        } else {
            rows.push(StockRow {
                company_name: company_name.to_string(),
                stock_code: stock_code.to_string(),
                related_files: vec![],
                key_knowledge: vec![knowledge_text],
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        let json = serde_json::to_string_pretty(&rows)
            .map_err(|e| format!("序列化 stock_table 失败: {e}"))?;
        tokio::fs::write(&self.path, json.as_bytes())
            .await
            .map_err(|e| format!("写 stock_table.json 失败: {e}"))?;

        info!("[KB/StockTable] append_knowledge company={company_name} code={stock_code}");
        Ok(())
    }
}
