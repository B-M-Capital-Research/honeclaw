//! 公司画像存储 - Markdown 主文件 + 事件目录
//!
//! 目录结构：
//! ```text
//! data/company_profiles/
//!   <profile_id>/
//!     profile.md
//!     events/
//!       2026-04-12-earnings-q1-update.md
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndustryTemplate {
    General,
    Saas,
    SemiconductorHardware,
    Consumer,
    IndustrialDefense,
    Financials,
}

impl Default for IndustryTemplate {
    fn default() -> Self {
        Self::General
    }
}

impl IndustryTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Saas => "saas",
            Self::SemiconductorHardware => "semiconductor_hardware",
            Self::Consumer => "consumer",
            Self::IndustrialDefense => "industrial_defense",
            Self::Financials => "financials",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_tracking_cadence")]
    pub cadence: String,
    #[serde(default)]
    pub focus_metrics: Vec<String>,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cadence: default_tracking_cadence(),
            focus_metrics: Vec::new(),
        }
    }
}

fn default_tracking_cadence() -> String {
    "weekly".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMetadata {
    pub company_name: String,
    #[serde(default)]
    pub stock_code: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sector: String,
    #[serde(default)]
    pub industry_template: IndustryTemplate,
    #[serde(default = "default_profile_status")]
    pub status: String,
    #[serde(default)]
    pub tracking: TrackingConfig,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reviewed_at: Option<String>,
}

fn default_profile_status() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileEventMetadata {
    pub event_type: String,
    pub occurred_at: String,
    pub captured_at: String,
    #[serde(default = "default_thesis_impact")]
    pub thesis_impact: String,
    #[serde(default)]
    pub changed_sections: Vec<String>,
    #[serde(default)]
    pub refs: Vec<String>,
}

fn default_thesis_impact() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileEventDocument {
    pub id: String,
    pub filename: String,
    pub title: String,
    pub metadata: ProfileEventMetadata,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileDocument {
    pub profile_id: String,
    pub metadata: ProfileMetadata,
    pub markdown: String,
    pub events: Vec<CompanyProfileEventDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub profile_id: String,
    pub company_name: String,
    pub stock_code: String,
    pub sector: String,
    pub industry_template: IndustryTemplate,
    pub status: String,
    pub tracking_enabled: bool,
    pub tracking_cadence: String,
    pub updated_at: String,
    pub last_reviewed_at: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone)]
pub struct CreateProfileInput {
    pub company_name: String,
    pub stock_code: Option<String>,
    pub sector: Option<String>,
    pub aliases: Vec<String>,
    pub industry_template: IndustryTemplate,
    pub tracking: Option<TrackingConfig>,
    pub initial_sections: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AppendEventInput {
    pub title: String,
    pub event_type: String,
    pub occurred_at: String,
    pub thesis_impact: String,
    pub changed_sections: Vec<String>,
    pub refs: Vec<String>,
    pub what_happened: String,
    pub why_it_matters: String,
    pub thesis_effect: String,
    pub evidence: String,
    pub research_log: String,
    pub follow_up: String,
}

pub struct CompanyProfileStorage {
    root_dir: PathBuf,
}

impl CompanyProfileStorage {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        let root_dir = root_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&root_dir);
        Self { root_dir }
    }

    pub fn profile_id(company_name: &str, stock_code: &str) -> String {
        if !stock_code.trim().is_empty() {
            sanitize_id(&normalize_stock_code(stock_code))
        } else {
            slugify(company_name)
        }
    }

    pub fn find_profile_id(
        &self,
        company_name: Option<&str>,
        stock_code: Option<&str>,
    ) -> Option<String> {
        let company_name = company_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_company_name);
        let stock_code = stock_code
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_stock_code);

        let entries = match fs::read_dir(&self.root_dir) {
            Ok(entries) => entries,
            Err(_) => return None,
        };

        for entry in entries.flatten() {
            let dir = entry.path();
            let Some(document) = self.load_profile_by_dir(&dir).ok().flatten() else {
                continue;
            };
            let alias_match = company_name.as_ref().map(|needle| {
                document
                    .metadata
                    .aliases
                    .iter()
                    .any(|alias| normalize_company_name(alias) == *needle)
            });
            let code_match = stock_code
                .as_ref()
                .map(|value| normalize_stock_code(&document.metadata.stock_code) == *value)
                .unwrap_or(false);
            let name_match = company_name
                .as_ref()
                .map(|value| normalize_company_name(&document.metadata.company_name) == *value)
                .unwrap_or(false);

            if code_match || name_match || alias_match.unwrap_or(false) {
                return Some(document.profile_id);
            }
        }

        None
    }

    pub fn list_profiles(&self) -> Vec<ProfileSummary> {
        let entries = match fs::read_dir(&self.root_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        let mut profiles = Vec::new();
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            if let Some(document) = self.load_profile_by_dir(&dir).ok().flatten() {
                profiles.push(ProfileSummary {
                    profile_id: document.profile_id,
                    company_name: document.metadata.company_name.clone(),
                    stock_code: document.metadata.stock_code.clone(),
                    sector: document.metadata.sector.clone(),
                    industry_template: document.metadata.industry_template.clone(),
                    status: document.metadata.status.clone(),
                    tracking_enabled: document.metadata.tracking.enabled,
                    tracking_cadence: document.metadata.tracking.cadence.clone(),
                    updated_at: document.metadata.updated_at.clone(),
                    last_reviewed_at: document.metadata.last_reviewed_at.clone(),
                    event_count: document.events.len(),
                });
            }
        }

        profiles.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.company_name.cmp(&b.company_name))
        });
        profiles
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<CompanyProfileDocument>, String> {
        let profile_id = sanitize_id(profile_id.trim());
        if profile_id.is_empty() {
            return Ok(None);
        }
        self.load_profile_by_dir(&self.root_dir.join(profile_id))
    }

    pub fn create_profile(
        &self,
        input: CreateProfileInput,
    ) -> Result<(CompanyProfileDocument, bool), String> {
        let company_name = input.company_name.trim();
        if company_name.is_empty() {
            return Err("company_name 不能为空".to_string());
        }

        let stock_code = input
            .stock_code
            .as_deref()
            .map(normalize_stock_code)
            .unwrap_or_default();

        if let Some(existing_id) = self.find_profile_id(Some(company_name), Some(&stock_code)) {
            let mut document = self
                .get_profile(&existing_id)?
                .ok_or_else(|| "画像已存在但读取失败".to_string())?;
            let mut changed = false;

            if !stock_code.is_empty() && document.metadata.stock_code.is_empty() {
                document.metadata.stock_code = stock_code.clone();
                changed = true;
            }
            let mut aliases = alias_union(
                &document.metadata.aliases,
                &input.aliases,
                company_name,
                &stock_code,
                &document.metadata.company_name,
            );
            if aliases != document.metadata.aliases {
                document.metadata.aliases = std::mem::take(&mut aliases);
                changed = true;
            }
            if changed {
                document.metadata.updated_at = Utc::now().to_rfc3339();
                self.write_profile(&document, None)?;
            }
            return Ok((document, false));
        }

        let created_at = Utc::now().to_rfc3339();
        let profile_id = Self::profile_id(company_name, &stock_code);
        let profile_dir = self.root_dir.join(&profile_id);
        fs::create_dir_all(profile_dir.join("events"))
            .map_err(|err| format!("创建画像目录失败: {err}"))?;

        let metadata = ProfileMetadata {
            company_name: company_name.to_string(),
            stock_code: stock_code.clone(),
            aliases: alias_union(&[], &input.aliases, company_name, &stock_code, company_name),
            sector: input.sector.unwrap_or_default().trim().to_string(),
            industry_template: input.industry_template,
            status: default_profile_status(),
            tracking: input.tracking.unwrap_or_default(),
            created_at: created_at.clone(),
            updated_at: created_at.clone(),
            last_reviewed_at: None,
        };

        let sections = build_initial_sections(&metadata.industry_template, &input.initial_sections);
        let markdown =
            render_profile_markdown(&metadata, &sections, &create_profile_body(&sections))
                .map_err(|err| format!("渲染 profile.md 失败: {err}"))?;
        let document = CompanyProfileDocument {
            profile_id,
            metadata,
            markdown,
            events: Vec::new(),
        };

        self.write_profile(&document, Some(sections))?;
        Ok((document, true))
    }

    pub fn rewrite_sections(
        &self,
        profile_id: &str,
        sections: &BTreeMap<String, String>,
    ) -> Result<Option<CompanyProfileDocument>, String> {
        let Some(mut document) = self.get_profile(profile_id)? else {
            return Ok(None);
        };

        let (mut ordered_sections, _) = parse_profile_sections(&document.markdown);
        let mut index_by_title = HashMap::new();
        for (index, (title, _)) in ordered_sections.iter().enumerate() {
            index_by_title.insert(title.clone(), index);
        }

        for (title, content) in sections {
            let normalized_title = title.trim().to_string();
            if normalized_title.is_empty() {
                continue;
            }
            if let Some(index) = index_by_title.get(&normalized_title).copied() {
                ordered_sections[index].1 = content.trim().to_string();
            } else {
                ordered_sections.push((normalized_title.clone(), content.trim().to_string()));
                index_by_title.insert(normalized_title, ordered_sections.len() - 1);
            }
        }

        document.metadata.updated_at = Utc::now().to_rfc3339();
        document.markdown = render_profile_markdown(
            &document.metadata,
            &ordered_sections,
            &create_profile_body(&ordered_sections),
        )
        .map_err(|err| format!("回写 profile.md 失败: {err}"))?;
        self.write_profile(&document, Some(ordered_sections))?;
        Ok(Some(document))
    }

    pub fn set_tracking(
        &self,
        profile_id: &str,
        tracking: TrackingConfig,
    ) -> Result<Option<CompanyProfileDocument>, String> {
        let Some(mut document) = self.get_profile(profile_id)? else {
            return Ok(None);
        };
        document.metadata.tracking = tracking;
        document.metadata.updated_at = Utc::now().to_rfc3339();
        let sections = parse_profile_sections(&document.markdown).0;
        document.markdown = render_profile_markdown(
            &document.metadata,
            &sections,
            &create_profile_body(&sections),
        )
        .map_err(|err| format!("回写 profile.md 失败: {err}"))?;
        self.write_profile(&document, Some(sections))?;
        Ok(Some(document))
    }

    pub fn append_event(
        &self,
        profile_id: &str,
        input: AppendEventInput,
    ) -> Result<Option<CompanyProfileEventDocument>, String> {
        let Some(mut document) = self.get_profile(profile_id)? else {
            return Ok(None);
        };
        if input.what_happened.trim().is_empty()
            && input.why_it_matters.trim().is_empty()
            && input.thesis_effect.trim().is_empty()
            && input.evidence.trim().is_empty()
            && input.research_log.trim().is_empty()
            && input.follow_up.trim().is_empty()
        {
            return Err("事件内容不能为空".to_string());
        }

        let occurred_date = normalize_event_date(&input.occurred_at);
        let event_filename = format!(
            "{}-{}-{}.md",
            occurred_date,
            sanitize_id(&input.event_type),
            slugify(&input.title)
        );
        let event_id = event_filename.trim_end_matches(".md").to_string();
        let event_path = self
            .root_dir
            .join(&document.profile_id)
            .join("events")
            .join(&event_filename);
        if event_path.exists() {
            let content = fs::read_to_string(&event_path)
                .map_err(|err| format!("读取现有事件失败: {err}"))?;
            return Ok(Some(parse_event_markdown(
                &event_id,
                &event_filename,
                &content,
            )?));
        }

        let metadata = ProfileEventMetadata {
            event_type: input.event_type.trim().to_string(),
            occurred_at: input.occurred_at.trim().to_string(),
            captured_at: Utc::now().to_rfc3339(),
            thesis_impact: input.thesis_impact.trim().to_string(),
            changed_sections: unique_strings(&input.changed_sections),
            refs: unique_strings(&input.refs),
        };

        let markdown = render_event_markdown(&input.title, &metadata, &input);
        fs::write(&event_path, markdown.as_bytes())
            .map_err(|err| format!("写事件文件失败: {err}"))?;

        document.metadata.updated_at = Utc::now().to_rfc3339();
        let sections = parse_profile_sections(&document.markdown).0;
        document.markdown = render_profile_markdown(
            &document.metadata,
            &sections,
            &create_profile_body(&sections),
        )
        .map_err(|err| format!("更新 profile.md 时间戳失败: {err}"))?;
        self.write_profile(&document, Some(sections))?;

        Ok(Some(CompanyProfileEventDocument {
            id: event_id,
            filename: event_filename,
            title: input.title.trim().to_string(),
            metadata,
            markdown,
        }))
    }

    pub fn delete_profile(&self, profile_id: &str) -> Result<bool, String> {
        let profile_id = sanitize_id(profile_id.trim());
        if profile_id.is_empty() {
            return Ok(false);
        }
        let profile_dir = self.root_dir.join(&profile_id);
        if !profile_dir.exists() {
            return Ok(false);
        }
        fs::remove_dir_all(&profile_dir).map_err(|err| format!("删除画像目录失败: {err}"))?;
        Ok(true)
    }

    fn write_profile(
        &self,
        document: &CompanyProfileDocument,
        sections: Option<Vec<(String, String)>>,
    ) -> Result<(), String> {
        let profile_dir = self.root_dir.join(&document.profile_id);
        fs::create_dir_all(profile_dir.join("events"))
            .map_err(|err| format!("创建画像目录失败: {err}"))?;

        let profile_path = profile_dir.join("profile.md");
        let markdown = if let Some(sections) = sections {
            render_profile_markdown(
                &document.metadata,
                &sections,
                &create_profile_body(&sections),
            )
            .map_err(|err| format!("渲染 profile.md 失败: {err}"))?
        } else {
            let (parsed_sections, _) = parse_profile_sections(&document.markdown);
            render_profile_markdown(
                &document.metadata,
                &parsed_sections,
                &create_profile_body(&parsed_sections),
            )
            .map_err(|err| format!("渲染 profile.md 失败: {err}"))?
        };
        fs::write(profile_path, markdown.as_bytes())
            .map_err(|err| format!("写 profile.md 失败: {err}"))?;
        Ok(())
    }

    fn load_profile_by_dir(&self, dir: &Path) -> Result<Option<CompanyProfileDocument>, String> {
        let profile_path = dir.join("profile.md");
        if !profile_path.exists() {
            return Ok(None);
        }

        let profile_id = dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let markdown = fs::read_to_string(&profile_path)
            .map_err(|err| format!("读取 profile.md 失败: {err}"))?;
        let (metadata, _) = parse_profile_markdown(&markdown)?;
        let events = self.load_events(dir)?;

        Ok(Some(CompanyProfileDocument {
            profile_id,
            metadata,
            markdown,
            events,
        }))
    }

    fn load_events(&self, profile_dir: &Path) -> Result<Vec<CompanyProfileEventDocument>, String> {
        let events_dir = profile_dir.join("events");
        if !events_dir.exists() {
            return Ok(Vec::new());
        }

        let entries =
            fs::read_dir(&events_dir).map_err(|err| format!("读取事件目录失败: {err}"))?;
        let mut events = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            let id = filename.trim_end_matches(".md").to_string();
            let markdown =
                fs::read_to_string(&path).map_err(|err| format!("读取事件文件失败: {err}"))?;
            events.push(parse_event_markdown(&id, &filename, &markdown)?);
        }

        events.sort_by(|a, b| {
            b.metadata
                .occurred_at
                .cmp(&a.metadata.occurred_at)
                .then_with(|| b.filename.cmp(&a.filename))
        });
        Ok(events)
    }
}

fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = trimmed.to_lowercase();
        if seen.insert(normalized) {
            unique.push(trimmed.to_string());
        }
    }
    unique
}

fn alias_union(
    existing: &[String],
    extra: &[String],
    company_name: &str,
    stock_code: &str,
    metadata_company_name: &str,
) -> Vec<String> {
    let mut combined = Vec::new();
    let mut seen = HashSet::new();
    let extras = [
        company_name.to_string(),
        stock_code.to_string(),
        metadata_company_name.to_string(),
    ];
    for candidate in existing.iter().chain(extra.iter()).chain(extras.iter()) {
        let trimmed = candidate.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = trimmed.to_lowercase();
        if seen.insert(normalized) {
            combined.push(trimmed.to_string());
        }
    }
    combined
}

fn parse_frontmatter(content: &str) -> Result<(String, String), String> {
    if !content.starts_with("---\n") {
        return Err("缺少 frontmatter".to_string());
    }
    let remainder = &content[4..];
    let Some(end) = remainder.find("\n---\n") else {
        return Err("frontmatter 未正确结束".to_string());
    };
    let frontmatter = remainder[..end].to_string();
    let body = remainder[end + 5..].to_string();
    Ok((frontmatter, body))
}

fn parse_profile_markdown(
    content: &str,
) -> Result<(ProfileMetadata, Vec<(String, String)>), String> {
    let (frontmatter, body) = parse_frontmatter(content)?;
    let metadata: ProfileMetadata = serde_yaml::from_str(&frontmatter)
        .map_err(|err| format!("解析画像 frontmatter 失败: {err}"))?;
    let (sections, _) = parse_profile_sections(&body);
    Ok((metadata, sections))
}

fn parse_event_markdown(
    id: &str,
    filename: &str,
    content: &str,
) -> Result<CompanyProfileEventDocument, String> {
    let (frontmatter, body) = parse_frontmatter(content)?;
    let metadata: ProfileEventMetadata = serde_yaml::from_str(&frontmatter)
        .map_err(|err| format!("解析事件 frontmatter 失败: {err}"))?;
    let title = extract_title_from_markdown(&body);
    Ok(CompanyProfileEventDocument {
        id: id.to_string(),
        filename: filename.to_string(),
        title,
        metadata,
        markdown: content.to_string(),
    })
}

fn extract_title_from_markdown(body: &str) -> String {
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            return rest.trim().to_string();
        }
    }
    "未命名事件".to_string()
}

fn render_profile_markdown(
    metadata: &ProfileMetadata,
    _sections: &[(String, String)],
    body: &str,
) -> Result<String, serde_yaml::Error> {
    let frontmatter = serde_yaml::to_string(metadata)?;
    Ok(format!("---\n{}---\n\n{}", frontmatter, body.trim()))
}

fn render_event_markdown(
    title: &str,
    metadata: &ProfileEventMetadata,
    input: &AppendEventInput,
) -> String {
    let frontmatter =
        serde_yaml::to_string(metadata).unwrap_or_else(|_| "event_type: unknown\n".to_string());
    format!(
        "---\n{}---\n\n# {}\n\n## 发生了什么\n{}\n\n## 为什么重要\n{}\n\n## 影响哪些画像 section\n{}\n\n## 对 thesis 的影响\n{}\n\n## 证据与来源\n{}\n\n## 本轮研究路径\n{}\n\n## 需要继续跟踪什么\n{}\n",
        frontmatter,
        title.trim(),
        fallback_markdown(&input.what_happened),
        fallback_markdown(&input.why_it_matters),
        render_list_or_placeholder(&input.changed_sections, "暂无"),
        fallback_markdown(&input.thesis_effect),
        render_evidence_markdown(&input.evidence, &input.refs),
        fallback_markdown(&input.research_log),
        fallback_markdown(&input.follow_up),
    )
}

fn create_profile_body(sections: &[(String, String)]) -> String {
    sections
        .iter()
        .map(|(title, content)| format!("## {}\n{}\n", title, content.trim()))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn fallback_markdown(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "暂无".to_string()
    } else {
        trimmed.to_string()
    }
}

fn render_list_or_placeholder(values: &[String], placeholder: &str) -> String {
    if values.is_empty() {
        placeholder.to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {}", value.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_evidence_markdown(evidence: &str, refs: &[String]) -> String {
    let mut blocks = Vec::new();
    if !evidence.trim().is_empty() {
        blocks.push(evidence.trim().to_string());
    }
    if !refs.is_empty() {
        blocks.push(
            refs.iter()
                .map(|value| format!("- {}", value.trim()))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    if blocks.is_empty() {
        "暂无".to_string()
    } else {
        blocks.join("\n\n")
    }
}

fn parse_profile_sections(content: &str) -> (Vec<(String, String)>, Vec<String>) {
    let body = if content.starts_with("---\n") {
        parse_frontmatter(content)
            .map(|(_, body)| body)
            .unwrap_or_else(|_| content.to_string())
    } else {
        content.to_string()
    };

    let mut sections = Vec::new();
    let mut extra_lines = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(title) = current_title.take() {
                sections.push((title, current_lines.join("\n").trim().to_string()));
                current_lines.clear();
            }
            current_title = Some(rest.trim().to_string());
        } else if current_title.is_some() {
            current_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            extra_lines.push(line.to_string());
        }
    }

    if let Some(title) = current_title.take() {
        sections.push((title, current_lines.join("\n").trim().to_string()));
    }

    (sections, extra_lines)
}

fn build_initial_sections(
    template: &IndustryTemplate,
    overrides: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let mut sections = base_profile_sections(template);
    let mut index = HashMap::new();
    for (position, (title, _)) in sections.iter().enumerate() {
        index.insert(title.clone(), position);
    }

    for (title, content) in overrides {
        let normalized_title = title.trim().to_string();
        if normalized_title.is_empty() {
            continue;
        }
        if let Some(position) = index.get(&normalized_title).copied() {
            sections[position].1 = content.trim().to_string();
        } else {
            sections.push((normalized_title.clone(), content.trim().to_string()));
            index.insert(normalized_title, sections.len() - 1);
        }
    }
    sections
}

fn base_profile_sections(template: &IndustryTemplate) -> Vec<(String, String)> {
    let mut sections = vec![
        (
            "投资主张".to_string(),
            "待补充：这家公司当前最核心的长期判断、为何值得跟踪，以及现阶段最重要的一句话结论。".to_string(),
        ),
        (
            "Thesis".to_string(),
            "待补充：当前多空要点、判断为什么成立、最关键的 3-5 个观察变量，以及什么事实会证伪或改写 thesis。".to_string(),
        ),
        (
            "商业模式".to_string(),
            "待补充：公司如何赚钱、收入结构、成本结构、单位经济与周期性特征。".to_string(),
        ),
        (
            "行业与竞争格局".to_string(),
            "待补充：行业空间、竞争者、替代品、上下游议价权、进入壁垒与监管环境。".to_string(),
        ),
        (
            "护城河".to_string(),
            "待补充：品牌、网络效应、切换成本、规模优势、成本优势、渠道控制或牌照壁垒，并标注 moat 趋势。".to_string(),
        ),
        (
            "管理层与治理".to_string(),
            "待补充：创始人/高管团队、激励机制、资本配置记录、治理质量与对外沟通可信度。".to_string(),
        ),
        (
            "财务质量".to_string(),
            "待补充：增长质量、利润率、ROIC、现金流、负债结构、再投资效率与会计质量。".to_string(),
        ),
        (
            "资本配置".to_string(),
            "待补充：分红、回购、并购、研发、产能投资、去杠杆等动作是否提升长期每股价值。".to_string(),
        ),
        (
            "关键经营指标".to_string(),
            template_operating_metrics_markdown(template),
        ),
        (
            "估值框架".to_string(),
            "待补充：估值方法、关键假设、敏感性、可比对象和当前估值区间。".to_string(),
        ),
        (
            "风险台账".to_string(),
            "待补充：监管、技术替代、客户集中、库存、地缘政治、融资、治理失误或财务失真等风险，并单列 disconfirming evidence。".to_string(),
        ),
        (
            "关键跟踪清单".to_string(),
            template_tracking_markdown(template),
        ),
        (
            "未决问题".to_string(),
            "待补充：当前还未验证、但会显著影响 thesis 的问题列表。".to_string(),
        ),
        (
            "行业模板附录".to_string(),
            template_appendix_markdown(template),
        ),
    ];

    if matches!(template, IndustryTemplate::General) {
        sections.retain(|(title, _)| title != "行业模板附录");
    }
    sections
}

fn template_tracking_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => {
            "- 季度至少 review 一次\n- 财报/业绩会后必更\n- 重大事件（管理层、监管、资本配置、行业格局变化）触发更新\n- 估值进入关键区间时重看 thesis / 赔率 / 风险回报".to_string()
        }
        IndustryTemplate::Saas => {
            "- 财报后核对 ARR / RPO / NRR / 留存 / deferred revenue 的方向是否改变 thesis\n- 观察 seat expansion、产品渗透与销售效率是否改善\n- 指引变化时同步检查估值假设与可持续增长判断".to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "- 跟踪 ASP、良率、产能利用率、库存周期与 capex\n- 设计 win / 产品 mix 变化若影响中期盈利能力，应更新 thesis\n- 行业景气和客户备货节奏变化时，重看估值框架和风险台账".to_string()
        }
        IndustryTemplate::Consumer => {
            "- 跟踪同店、复购率、客单价、渠道库存、促销强度\n- 品牌溢价与新品表现若出现拐点，应检查护城河与管理层判断\n- 观察库存/折扣是否正在侵蚀长期盈利质量".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "- 跟踪订单、积压订单、book-to-bill、交付节奏、产能利用率\n- 大客户签约/流失、项目延误、预算变化应写入事件并重看 thesis\n- 若订单质量或兑现节奏恶化，更新风险台账与估值假设".to_string()
        }
        IndustryTemplate::Financials => {
            "- 跟踪净息差、不良、拨备、资本充足率、负债成本\n- 若风险成本、资产质量或资本压力变化，应更新财务质量与 thesis\n- 利率环境或监管变化后，重看估值框架和核心风险".to_string()
        }
    }
}

fn template_operating_metrics_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => {
            "待补充：列出这家公司真正决定长期判断的 3-7 个经营指标，并说明每个指标为什么重要。"
                .to_string()
        }
        IndustryTemplate::Saas => {
            "- ARR\n- RPO / cRPO\n- NRR / 客户留存\n- seat expansion / 产品渗透\n- deferred revenue"
                .to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "- ASP\n- 良率\n- 产能与 capex\n- 库存天数 / 渠道库存\n- 设计 win 与产品 mix"
                .to_string()
        }
        IndustryTemplate::Consumer => {
            "- 同店销售\n- 复购率\n- 客单价\n- 渠道库存\n- 品牌溢价与促销强度".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "- 新签订单\n- 积压订单\n- book-to-bill\n- 交付节奏\n- 产能利用率".to_string()
        }
        IndustryTemplate::Financials => {
            "- 净息差\n- 不良率\n- 拨备覆盖率\n- 资本充足率\n- 负债成本".to_string()
        }
    }
}

fn template_appendix_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => String::new(),
        IndustryTemplate::Saas => {
            "本模板重点关注 SaaS 公司常见核心变量：ARR、RPO、NRR、留存、产品渗透、deferred revenue。".to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "本模板重点关注半导体/硬件公司常见核心变量：ASP、良率、产能、库存、设计 win、capex。".to_string()
        }
        IndustryTemplate::Consumer => {
            "本模板重点关注消费公司常见核心变量：同店、复购、客单价、渠道库存、品牌溢价。".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "本模板重点关注工业/国防公司常见核心变量：订单、积压订单、book-to-bill、交付、产能。".to_string()
        }
        IndustryTemplate::Financials => {
            "本模板重点关注金融公司常见核心变量：净息差、不良、拨备、资本充足率、负债成本。".to_string()
        }
    }
}

fn normalize_stock_code(value: &str) -> String {
    value.trim().to_uppercase()
}

fn normalize_company_name(value: &str) -> String {
    value
        .trim()
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
}

fn sanitize_id(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if ch.is_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn normalize_event_date(value: &str) -> String {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value.trim()) {
        parsed.format("%Y-%m-%d").to_string()
    } else {
        value.trim().chars().take(10).collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn profile_id_prefers_stock_code_then_slugified_name() {
        assert_eq!(
            CompanyProfileStorage::profile_id("Apple Inc.", "AAPL"),
            "AAPL"
        );
        assert_eq!(
            CompanyProfileStorage::profile_id("Hermes Holdings", ""),
            "hermes-holdings"
        );
    }

    #[test]
    fn create_profile_generates_markdown_template_and_industry_sections() {
        let dir = make_temp_dir("company_profile_create");
        let storage = CompanyProfileStorage::new(&dir);

        let (document, created) = storage
            .create_profile(CreateProfileInput {
                company_name: "Snowflake".to_string(),
                stock_code: Some("SNOW".to_string()),
                sector: Some("Software".to_string()),
                aliases: vec!["Snowflake Inc.".to_string()],
                industry_template: IndustryTemplate::Saas,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("create profile");

        assert!(created);
        assert_eq!(document.profile_id, "SNOW");
        assert!(document.markdown.contains("## 投资主张"));
        assert!(document.markdown.contains("## Thesis"));
        assert!(document.markdown.contains("## 关键经营指标"));
        assert!(document.markdown.contains("ARR"));
        assert!(dir.join("SNOW").join("profile.md").exists());
    }

    #[test]
    fn create_profile_reuses_existing_profile_and_merges_aliases() {
        let dir = make_temp_dir("company_profile_alias");
        let storage = CompanyProfileStorage::new(&dir);

        let _ = storage
            .create_profile(CreateProfileInput {
                company_name: "Apple".to_string(),
                stock_code: Some("AAPL".to_string()),
                sector: None,
                aliases: vec![],
                industry_template: IndustryTemplate::General,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("create");

        let (document, created) = storage
            .create_profile(CreateProfileInput {
                company_name: "Apple Inc.".to_string(),
                stock_code: Some("AAPL".to_string()),
                sector: None,
                aliases: vec!["AAPL US".to_string()],
                industry_template: IndustryTemplate::General,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("recreate");

        assert!(!created);
        assert!(
            document
                .metadata
                .aliases
                .iter()
                .any(|item| item == "Apple Inc.")
        );
        assert!(
            document
                .metadata
                .aliases
                .iter()
                .any(|item| item == "AAPL US")
        );
    }

    #[test]
    fn rewrite_sections_only_touches_target_section() {
        let dir = make_temp_dir("company_profile_rewrite");
        let storage = CompanyProfileStorage::new(&dir);

        let (document, _) = storage
            .create_profile(CreateProfileInput {
                company_name: "NVIDIA".to_string(),
                stock_code: Some("NVDA".to_string()),
                sector: None,
                aliases: vec![],
                industry_template: IndustryTemplate::SemiconductorHardware,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("create");

        let mut updates = BTreeMap::new();
        updates.insert(
            "投资主张".to_string(),
            "AI 数据中心需求依旧是核心驱动。".to_string(),
        );
        let updated = storage
            .rewrite_sections(&document.profile_id, &updates)
            .expect("rewrite")
            .expect("profile exists");

        assert!(updated.markdown.contains("AI 数据中心需求依旧是核心驱动。"));
        assert!(updated.markdown.contains("## 财务质量"));
    }

    #[test]
    fn append_event_is_idempotent_by_filename() {
        let dir = make_temp_dir("company_profile_event");
        let storage = CompanyProfileStorage::new(&dir);

        let (document, _) = storage
            .create_profile(CreateProfileInput {
                company_name: "Tesla".to_string(),
                stock_code: Some("TSLA".to_string()),
                sector: None,
                aliases: vec![],
                industry_template: IndustryTemplate::Consumer,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("create");

        let input = AppendEventInput {
            title: "Q1 财报更新".to_string(),
            event_type: "earnings".to_string(),
            occurred_at: "2026-04-12T10:00:00Z".to_string(),
            thesis_impact: "mixed".to_string(),
            changed_sections: vec!["财务质量".to_string(), "关键跟踪清单".to_string()],
            refs: vec!["earnings-call".to_string()],
            what_happened: "毛利率承压，但储能业务增长延续。".to_string(),
            why_it_matters: "汽车与储能盈利结构的分化，决定市场是否继续给予成长溢价。".to_string(),
            thesis_effect: "汽车业务压力仍需观察，储能继续改善长期结构。".to_string(),
            evidence: "电话会确认管理层仍把储能和 AI 基础设施视作中期投入重点。".to_string(),
            research_log: "- query: Tesla Q1 earnings margin storage\n- reviewed: earnings release, earnings call transcript, shareholder deck".to_string(),
            follow_up: "观察下一季交付和毛利率修复节奏。".to_string(),
        };

        let first = storage
            .append_event(&document.profile_id, input.clone())
            .expect("append")
            .expect("event");
        let second = storage
            .append_event(&document.profile_id, input)
            .expect("append again")
            .expect("event");

        assert_eq!(first.filename, second.filename);
        let loaded = storage
            .get_profile(&document.profile_id)
            .expect("load")
            .expect("profile exists");
        assert_eq!(loaded.events.len(), 1);
        assert!(loaded.events[0].markdown.contains("## 为什么重要"));
        assert!(loaded.events[0].markdown.contains("## 本轮研究路径"));
    }

    #[test]
    fn delete_profile_removes_directory() {
        let dir = make_temp_dir("company_profile_delete");
        let storage = CompanyProfileStorage::new(&dir);

        let (document, _) = storage
            .create_profile(CreateProfileInput {
                company_name: "Adobe".to_string(),
                stock_code: Some("ADBE".to_string()),
                sector: None,
                aliases: vec![],
                industry_template: IndustryTemplate::Saas,
                tracking: None,
                initial_sections: BTreeMap::new(),
            })
            .expect("create");

        assert!(
            storage
                .delete_profile(&document.profile_id)
                .expect("delete")
        );
        assert!(
            storage
                .get_profile(&document.profile_id)
                .expect("load deleted")
                .is_none()
        );
    }

    #[test]
    fn template_appendix_matches_industry() {
        let sections = base_profile_sections(&IndustryTemplate::Financials);
        let appendix = sections
            .iter()
            .find(|(title, _)| title == "行业模板附录")
            .expect("appendix");
        assert!(appendix.1.contains("净息差"));
    }
}
