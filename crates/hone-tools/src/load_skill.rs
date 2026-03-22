//! LoadSkillTool — 技能加载工具
//!
//! 从 skills/ 目录读取技能定义，格式固定为 `skills/{name}/SKILL.md`：
//! 文件顶部为 YAML frontmatter，正文为 Markdown prompt。

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};

/// 宏：统一技能加载日志格式（`[LoadSkillTool] skill_load name=xxx ...`）
macro_rules! skill_log {
    ($($arg:tt)*) => {
        tracing::info!(target: "hone_tools::load_skill", $($arg)*)
    };
}

/// 技能定义（从 YAML frontmatter 解析）
#[derive(Debug, Deserialize, Default)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tools: Vec<String>,
    /// 触发该技能的别名关键词列表
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// 技能元信息（用于列表展示，不含完整 prompt body）
#[derive(Debug, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub tools: Vec<String>,
}

/// LoadSkillTool — 加载指定技能
pub struct LoadSkillTool {
    skills_dirs: Vec<PathBuf>,
}

impl LoadSkillTool {
    pub fn new(skills_dirs: Vec<PathBuf>) -> Self {
        Self { skills_dirs }
    }

    /// 列出所有可用技能名称
    ///
    /// 遍历所有技能目录，识别包含 `SKILL.md` 的子目录。
    fn list_skills(&self) -> Vec<String> {
        let mut names = Vec::new();

        for dir in &self.skills_dirs {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };

            for entry in entries.filter_map(|e| e.ok()) {
                let Ok(ft) = entry.file_type() else { continue };
                let file_name = entry.file_name();
                let s = file_name.to_string_lossy();

                if ft.is_dir() {
                    // 新格式：目录内需要有 SKILL.md
                    let skill_md = entry.path().join("SKILL.md");
                    if skill_md.exists() && !names.contains(&s.to_string()) {
                        names.push(s.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    /// 列出所有可用技能并附带元信息（display_name / description / aliases / tools）。
    ///
    /// 尝试读取每个技能的 SKILL.md frontmatter；若读取失败则仅返回目录名。
    pub fn list_skills_with_meta(&self) -> Vec<SkillMeta> {
        let names = self.list_skills();
        let mut result = Vec::new();

        for name in names {
            let meta = match self.load(&name) {
                Ok((frontmatter, _body)) => SkillMeta {
                    name: name.clone(),
                    display_name: if frontmatter.name.is_empty() {
                        name.clone()
                    } else {
                        frontmatter.name
                    },
                    description: frontmatter.description,
                    aliases: frontmatter.aliases,
                    tools: frontmatter.tools,
                },
                Err(_) => SkillMeta {
                    name: name.clone(),
                    display_name: name.clone(),
                    description: String::new(),
                    aliases: vec![],
                    tools: vec![],
                },
            };
            result.push(meta);
        }

        result
    }

    /// 根据查询词搜索技能，按相关性排序返回。
    ///
    /// 匹配范围包含：技能 id、展示名、aliases、description、tools。
    pub fn search_skills_with_meta(&self, query: &str, limit: usize) -> Vec<SkillMeta> {
        let normalized_query = normalize_skill_search_text(query);
        let mut metas = self.list_skills_with_meta();

        if normalized_query.is_empty() {
            metas.sort_by(|a, b| {
                a.display_name
                    .cmp(&b.display_name)
                    .then_with(|| a.name.cmp(&b.name))
            });
            if limit == 0 || metas.len() <= limit {
                return metas;
            }
            metas.truncate(limit);
            return metas;
        }

        let mut scored = metas
            .into_iter()
            .filter_map(|meta| {
                let score = score_skill_meta(&meta, &normalized_query)?;
                Some((score, meta))
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.display_name.cmp(&right.1.display_name))
                .then_with(|| left.1.name.cmp(&right.1.name))
        });

        let take_n = if limit == 0 {
            scored.len()
        } else {
            scored.len().min(limit)
        };

        scored
            .into_iter()
            .take(take_n)
            .map(|(_, meta)| meta)
            .collect()
    }

    /// 读取并解析指定技能
    ///
    /// 读取并解析 `skills/{name}/SKILL.md`。
    fn load(&self, skill_name: &str) -> Result<(SkillFrontmatter, String), String> {
        for dir in &self.skills_dirs {
            let skill_md = dir.join(skill_name).join("SKILL.md");
            if skill_md.exists() {
                let content = std::fs::read_to_string(&skill_md)
                    .map_err(|e| format!("读取 SKILL.md 失败: {e}"))?;
                return parse_skill_md(&content);
            }
        }

        // 找不到
        let available = self.list_skills().join(", ");
        Err(format!("技能 '{skill_name}' 不存在。可用技能：{available}"))
    }
}

fn normalize_skill_search_text(value: &str) -> String {
    value.trim().to_lowercase()
}

fn score_skill_meta(meta: &SkillMeta, query: &str) -> Option<i32> {
    let query = query.trim();
    if query.is_empty() {
        return Some(0);
    }

    let mut best = score_text_field(&meta.name, query, 130)?;
    best = best.max(score_text_field(&meta.display_name, query, 120).unwrap_or(0));
    best = best.max(score_text_field(&meta.description, query, 40).unwrap_or(0));

    for alias in &meta.aliases {
        best = best.max(score_text_field(alias, query, 110).unwrap_or(0));
    }
    for tool in &meta.tools {
        best = best.max(score_text_field(tool, query, 20).unwrap_or(0));
    }

    Some(best)
}

fn score_text_field(field: &str, query: &str, base: i32) -> Option<i32> {
    let normalized = normalize_skill_search_text(field);
    if normalized.is_empty() {
        return None;
    }
    if normalized == query {
        return Some(base + 1000);
    }
    if normalized.starts_with(query) {
        return Some(base + 800);
    }
    if normalized.contains(query) {
        return Some(base + 600);
    }

    let query_tokens = query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if query_tokens.is_empty() {
        return None;
    }
    if query_tokens.iter().all(|token| normalized.contains(token)) {
        return Some(base + 400 - query_tokens.len() as i32);
    }

    None
}

/// 解析 SKILL.md 文件：拆分 YAML frontmatter 与 Markdown 正文
///
/// 格式：
/// ```text
/// ---
/// name: ...
/// description: ...
/// ---
///
/// ## Markdown 正文（作为 prompt 注入）
/// ```
fn parse_skill_md(content: &str) -> Result<(SkillFrontmatter, String), String> {
    // frontmatter 必须以 "---\n" 开头
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
        .ok_or("SKILL.md 缺少 YAML frontmatter（应以 '---' 开头）")?;

    // 找到结束的 "---"
    let end_marker = rest
        .find("\n---\n")
        .or_else(|| rest.find("\n---\r\n"))
        .ok_or("SKILL.md frontmatter 未正确关闭（缺少结束 '---'）")?;

    let yaml_part = &rest[..end_marker];
    let body_start = end_marker + "\n---\n".len();
    let body = rest.get(body_start..).unwrap_or("").trim().to_string();

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_part)
        .map_err(|e| format!("解析 SKILL.md frontmatter 失败: {e}"))?;

    Ok((frontmatter, body))
}

/// 截取字符串到 `max_bytes` 字节附近的换行边界（UTF-8 安全）
fn truncate_to_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // 从 max_bytes 往回找到第一个合法 char boundary
    let safe_max = (0..=max_bytes)
        .rev()
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(0);
    // 在此范围内找最后一个换行，让截断更整齐
    let cut = s[..safe_max].rfind('\n').unwrap_or(safe_max);
    format!("{}...", s[..cut].trim_end())
}

#[async_trait]
impl Tool for LoadSkillTool {
    fn name(&self) -> &str {
        "load_skill"
    }

    fn description(&self) -> &str {
        "加载指定技能的系统提示词和工具列表。在执行专项任务前调用，例如：load_skill('stock_research')、load_skill('portfolio_management')。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "skill_name".to_string(),
            param_type: "string".to_string(),
            description: "技能名称，对应 skills/ 目录下的子目录名（目录内需包含 SKILL.md）。例如：stock_research、portfolio_management、market_analysis、scheduled_task、x_publish、image_generation、image_understanding、OWDR".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let skill_name = args
            .get("skill_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if skill_name.is_empty() {
            skill_log!("[LoadSkillTool] skill_load_error reason=empty_skill_name");
            return Ok(serde_json::json!({
                "success": false,
                "error": "skill_name 不能为空",
                "available_skills": self.list_skills()
            }));
        }

        skill_log!("[LoadSkillTool] skill_load_start name={}", skill_name);

        match self.load(skill_name) {
            Ok((skill, body)) => {
                // 取 Markdown body 的精简摘要（前 600 字节，截到换行边界），
                // 避免返回整个大 Markdown 导致 LLM 忘记用户的原始问题。
                let guide = truncate_to_boundary(&body, 600);

                skill_log!(
                    "[LoadSkillTool] skill_load_success name={} display_name={} tools={:?}",
                    skill_name,
                    skill.name,
                    skill.tools
                );

                Ok(serde_json::json!({
                    "success": true,
                    "skill_name": skill_name,
                    "skill_display_name": skill.name,
                    "skill_description": skill.description,
                    "available_tools": skill.tools,
                    "guide": guide,
                    "reminder": "现在请根据用户的原始问题调用合适的工具，记住用户问的是什么！"
                }))
            }
            Err(e) => {
                skill_log!(
                    "[LoadSkillTool] skill_load_failed name={} error={}",
                    skill_name,
                    e
                );
                Ok(serde_json::json!({
                    "success": false,
                    "error": e,
                    "available_skills": self.list_skills()
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[tokio::test]
    async fn load_skill_md_success_and_guide_truncated() {
        let skills_dir = make_temp_dir("hone_load_skill_md");
        let skill_dir = skills_dir.join("stock_research");
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");

        let long_body = "A".repeat(1000);
        let content = format!(
            "---\nname: \"个股研究\"\ndescription: \"desc\"\ntools:\n  - web_search\n---\n\n{}\n",
            long_body
        );
        std::fs::write(skill_dir.join("SKILL.md"), content).expect("write skill.md");

        let tool = LoadSkillTool::new(vec![skills_dir.clone()]);
        let result = tool
            .execute(serde_json::json!({"skill_name":"stock_research"}))
            .await
            .expect("execute load skill");

        assert_eq!(result["success"], true);
        assert_eq!(result["skill_display_name"], "个股研究");
        assert_eq!(result["available_tools"][0], "web_search");
        let guide = result["guide"].as_str().unwrap_or_default();
        assert!(!guide.is_empty());
        assert!(guide.len() <= 603);
    }

    #[tokio::test]
    async fn load_missing_skill_returns_available_skills() {
        let skills_dir = make_temp_dir("hone_load_skill_missing");

        let a_dir = skills_dir.join("a_skill");
        std::fs::create_dir_all(&a_dir).expect("create dir");
        std::fs::write(
            a_dir.join("SKILL.md"),
            "---\nname: \"A\"\ndescription: \"d\"\n---\n\na\n",
        )
        .expect("write a");
        let tool = LoadSkillTool::new(vec![skills_dir.clone()]);
        let result = tool
            .execute(serde_json::json!({"skill_name":"not_exists"}))
            .await
            .expect("execute missing");

        assert_eq!(result["success"], false);
        let list = result["available_skills"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let names: Vec<String> = list
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        assert!(names.contains(&"a_skill".to_string()));
        assert!(!names.contains(&"b_skill".to_string()));
    }

    #[test]
    fn search_skills_prefers_exact_and_alias_matches() {
        let skills_dir = make_temp_dir("hone_load_skill_search");
        let alpha_dir = skills_dir.join("stock_research");
        let beta_dir = skills_dir.join("macro_watch");
        std::fs::create_dir_all(&alpha_dir).expect("create alpha dir");
        std::fs::create_dir_all(&beta_dir).expect("create beta dir");

        std::fs::write(
            alpha_dir.join("SKILL.md"),
            r#"---
name: 个股研究
description: 深入研究单个股票
aliases:
  - stock
  - equity research
tools:
  - data_fetch
---
body
"#,
        )
        .expect("write alpha");
        std::fs::write(
            beta_dir.join("SKILL.md"),
            r#"---
name: 宏观观察
description: 跟踪宏观事件
aliases:
  - macro
tools:
  - web_search
---
body
"#,
        )
        .expect("write beta");

        let tool = LoadSkillTool::new(vec![skills_dir]);

        let exact = tool.search_skills_with_meta("stock_research", 5);
        assert_eq!(
            exact.first().map(|item| item.name.as_str()),
            Some("stock_research")
        );

        let alias = tool.search_skills_with_meta("macro", 5);
        assert_eq!(
            alias.first().map(|item| item.name.as_str()),
            Some("macro_watch")
        );
    }
}
