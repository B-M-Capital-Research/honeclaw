use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use hone_core::ActorIdentity;

use super::markdown::{
    build_initial_sections, create_profile_body, extract_title_from_markdown,
    file_modified_at_rfc3339, normalize_company_name, normalize_event_date, normalize_stock_code,
    parse_event_markdown_relaxed, parse_profile_metadata_relaxed, parse_profile_sections,
    render_event_markdown, render_profile_markdown, safe_component_join, sanitize_id, slugify,
};
use super::types::default_profile_status;
use super::{
    CompanyProfileDocument, CompanyProfileEventDocument, CompanyProfileStorage, CreateProfileInput,
    ProfileEventMetadata, ProfileMetadata, ProfileSpaceSummary, ProfileSummary, RawProfileDocument,
    RawProfileEventDocument, RawProfileSummary, TrackingConfig,
};

impl CompanyProfileStorage {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        let root_dir = root_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&root_dir);
        Self {
            root_dir,
            actor: None,
        }
    }

    pub fn for_actor(&self, actor: &ActorIdentity) -> Self {
        Self {
            root_dir: self.root_dir.clone(),
            actor: Some(actor.clone()),
        }
    }

    pub fn profile_id(company_name: &str, stock_code: &str) -> String {
        if !stock_code.trim().is_empty() {
            let sanitized = sanitize_id(&normalize_stock_code(stock_code));
            if sanitized.is_empty() {
                slugify(company_name)
            } else {
                sanitized
            }
        } else {
            slugify(company_name)
        }
    }

    pub fn find_profile_id(
        &self,
        company_name: Option<&str>,
        stock_code: Option<&str>,
    ) -> Option<String> {
        let root_dir = self.scoped_root().ok()?;
        let company_name = company_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_company_name);
        let stock_code = stock_code
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_stock_code);

        let entries = match fs::read_dir(&root_dir) {
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
        let Ok(root_dir) = self.scoped_root() else {
            return Vec::new();
        };
        let entries = match fs::read_dir(&root_dir) {
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

    pub fn list_profile_spaces(&self) -> Vec<ProfileSpaceSummary> {
        let mut spaces = Vec::new();
        let channels = match fs::read_dir(&self.root_dir) {
            Ok(entries) => entries,
            Err(_) => return spaces,
        };

        for channel_entry in channels.flatten() {
            let channel_path = channel_entry.path();
            if !channel_path.is_dir() {
                continue;
            }
            let Some(channel_component) = channel_path.file_name().and_then(|value| value.to_str())
            else {
                continue;
            };
            let channel = decode_component(channel_component);
            if channel.is_empty() {
                continue;
            }

            let scoped_users = match fs::read_dir(&channel_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for scoped_user_entry in scoped_users.flatten() {
                let scoped_user_path = scoped_user_entry.path();
                if !scoped_user_path.is_dir() {
                    continue;
                }
                let Some(scoped_user_key) = scoped_user_path
                    .file_name()
                    .and_then(|value| value.to_str())
                else {
                    continue;
                };
                let Some((channel_scope, user_id)) = actor_from_scoped_user_key(scoped_user_key)
                else {
                    continue;
                };
                let Ok(actor) = ActorIdentity::new(channel.clone(), user_id, channel_scope.clone())
                else {
                    continue;
                };
                let profiles_dir = scoped_user_path.join("company_profiles");
                if !profiles_dir.exists() {
                    continue;
                }
                let profiles = self.for_actor(&actor).list_profiles();
                if profiles.is_empty() {
                    continue;
                }
                spaces.push(ProfileSpaceSummary {
                    channel: actor.channel.clone(),
                    user_id: actor.user_id.clone(),
                    channel_scope: actor.channel_scope.clone(),
                    profile_count: profiles.len(),
                    updated_at: profiles.first().map(|profile| profile.updated_at.clone()),
                });
            }
        }

        spaces.sort_by(|a, b| {
            b.updated_at
                .as_deref()
                .unwrap_or_default()
                .cmp(a.updated_at.as_deref().unwrap_or_default())
                .then_with(|| a.channel.cmp(&b.channel))
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        spaces
    }

    pub fn list_profile_spaces_raw(&self) -> Vec<ProfileSpaceSummary> {
        let mut spaces = Vec::new();
        let channels = match fs::read_dir(&self.root_dir) {
            Ok(entries) => entries,
            Err(_) => return spaces,
        };

        for channel_entry in channels.flatten() {
            let channel_path = channel_entry.path();
            if !channel_path.is_dir() {
                continue;
            }
            let Some(channel_component) = channel_path.file_name().and_then(|value| value.to_str())
            else {
                continue;
            };
            let channel = decode_component(channel_component);
            if channel.is_empty() {
                continue;
            }

            let scoped_users = match fs::read_dir(&channel_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for scoped_user_entry in scoped_users.flatten() {
                let scoped_user_path = scoped_user_entry.path();
                if !scoped_user_path.is_dir() {
                    continue;
                }
                let Some(scoped_user_key) = scoped_user_path
                    .file_name()
                    .and_then(|value| value.to_str())
                else {
                    continue;
                };
                let Some((channel_scope, user_id)) = actor_from_scoped_user_key(scoped_user_key)
                else {
                    continue;
                };
                let Ok(actor) = ActorIdentity::new(channel.clone(), user_id, channel_scope.clone())
                else {
                    continue;
                };
                let profiles = self.for_actor(&actor).list_profiles_raw();
                if profiles.is_empty() {
                    continue;
                }
                spaces.push(ProfileSpaceSummary {
                    channel: actor.channel.clone(),
                    user_id: actor.user_id.clone(),
                    channel_scope: actor.channel_scope.clone(),
                    profile_count: profiles.len(),
                    updated_at: profiles
                        .first()
                        .and_then(|profile| profile.updated_at.clone()),
                });
            }
        }

        spaces.sort_by(|a, b| {
            b.updated_at
                .as_deref()
                .unwrap_or_default()
                .cmp(a.updated_at.as_deref().unwrap_or_default())
                .then_with(|| a.channel.cmp(&b.channel))
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        spaces
    }

    pub fn list_profiles_raw(&self) -> Vec<RawProfileSummary> {
        let Ok(root_dir) = self.scoped_root() else {
            return Vec::new();
        };
        let entries = match fs::read_dir(&root_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        let mut profiles = Vec::new();
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let Some(summary) = self.load_raw_profile_summary_by_dir(&dir).ok().flatten() else {
                continue;
            };
            profiles.push(summary);
        }

        profiles.sort_by(|a, b| {
            b.updated_at
                .as_deref()
                .unwrap_or_default()
                .cmp(a.updated_at.as_deref().unwrap_or_default())
                .then_with(|| a.title.cmp(&b.title))
        });
        profiles
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<CompanyProfileDocument>, String> {
        let root_dir = self.scoped_root()?;
        let Some(profile_dir) = safe_component_join(&root_dir, profile_id) else {
            return Ok(None);
        };
        self.load_profile_by_dir(&profile_dir)
    }

    pub fn get_profile_raw(&self, profile_id: &str) -> Result<Option<RawProfileDocument>, String> {
        let root_dir = self.scoped_root()?;
        let Some(profile_dir) = safe_component_join(&root_dir, profile_id) else {
            return Ok(None);
        };
        self.load_raw_profile_by_dir(&profile_dir)
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
        if profile_id.is_empty() {
            return Err("profile_id 非法".to_string());
        }
        let profile_dir = self.scoped_root()?.join(&profile_id);
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
        input: super::AppendEventInput,
    ) -> Result<Option<CompanyProfileEventDocument>, String> {
        let Some(mut document) = self.get_profile(profile_id)? else {
            return Ok(None);
        };
        if input.what_happened.trim().is_empty()
            && input.why_it_matters.trim().is_empty()
            && input.mainline_effect.trim().is_empty()
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
            .scoped_root()?
            .join(&document.profile_id)
            .join("events")
            .join(&event_filename);
        if event_path.exists() {
            let content = fs::read_to_string(&event_path)
                .map_err(|err| format!("读取现有事件失败: {err}"))?;
            return Ok(Some(parse_event_markdown_relaxed(
                &event_id,
                &event_filename,
                &content,
                file_modified_at_rfc3339(&event_path),
            )?));
        }

        let metadata = ProfileEventMetadata {
            event_type: input.event_type.trim().to_string(),
            occurred_at: input.occurred_at.trim().to_string(),
            captured_at: Utc::now().to_rfc3339(),
            mainline_impact: input.mainline_impact.trim().to_string(),
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
        let root_dir = self.scoped_root()?;
        let Some(profile_dir) = safe_component_join(&root_dir, profile_id) else {
            return Ok(false);
        };
        if !profile_dir.exists() {
            return Ok(false);
        }
        fs::remove_dir_all(&profile_dir).map_err(|err| format!("删除画像目录失败: {err}"))?;
        Ok(true)
    }

    pub(super) fn write_profile(
        &self,
        document: &CompanyProfileDocument,
        sections: Option<Vec<(String, String)>>,
    ) -> Result<(), String> {
        let profile_dir = self.scoped_root()?.join(&document.profile_id);
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

    pub(super) fn touch_profile_updated_at(
        &self,
        profile_id: &str,
    ) -> Result<Option<CompanyProfileDocument>, String> {
        let Some(mut document) = self.get_profile(profile_id)? else {
            return Ok(None);
        };
        let sections = parse_profile_sections(&document.markdown).0;
        document.metadata.updated_at = Utc::now().to_rfc3339();
        document.markdown = render_profile_markdown(
            &document.metadata,
            &sections,
            &create_profile_body(&sections),
        )
        .map_err(|err| format!("更新 profile.md 时间戳失败: {err}"))?;
        self.write_profile(&document, Some(sections))?;
        Ok(Some(document))
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
        let metadata = parse_profile_metadata_relaxed(
            &profile_id,
            &markdown,
            file_modified_at_rfc3339(&profile_path),
        )?;
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
            events.push(parse_event_markdown_relaxed(
                &id,
                &filename,
                &markdown,
                file_modified_at_rfc3339(&path),
            )?);
        }

        events.sort_by(|a, b| {
            b.metadata
                .occurred_at
                .cmp(&a.metadata.occurred_at)
                .then_with(|| b.filename.cmp(&a.filename))
        });
        Ok(events)
    }

    fn load_raw_profile_summary_by_dir(
        &self,
        dir: &Path,
    ) -> Result<Option<RawProfileSummary>, String> {
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
        let title = extract_title_from_markdown(&markdown, &profile_id);
        let updated_at = file_modified_at_rfc3339(&profile_path);
        let event_count = self.load_raw_events(dir)?.len();

        Ok(Some(RawProfileSummary {
            profile_id,
            title,
            updated_at,
            event_count,
        }))
    }

    fn load_raw_profile_by_dir(&self, dir: &Path) -> Result<Option<RawProfileDocument>, String> {
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
        let title = extract_title_from_markdown(&markdown, &profile_id);
        let updated_at = file_modified_at_rfc3339(&profile_path);
        let events = self.load_raw_events(dir)?;

        Ok(Some(RawProfileDocument {
            profile_id,
            title,
            updated_at,
            markdown,
            events,
        }))
    }

    fn load_raw_events(&self, profile_dir: &Path) -> Result<Vec<RawProfileEventDocument>, String> {
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
            let title = extract_title_from_markdown(&markdown, &id);
            let updated_at = file_modified_at_rfc3339(&path);
            events.push(RawProfileEventDocument {
                id,
                filename,
                title,
                updated_at,
                markdown,
            });
        }

        events.sort_by(|a, b| {
            b.updated_at
                .as_deref()
                .unwrap_or_default()
                .cmp(a.updated_at.as_deref().unwrap_or_default())
                .then_with(|| b.filename.cmp(&a.filename))
        });
        Ok(events)
    }

    pub(super) fn scoped_root(&self) -> Result<PathBuf, String> {
        let Some(actor) = &self.actor else {
            return Err("company profile storage requires actor scope".to_string());
        };
        Ok(self
            .root_dir
            .join(actor.channel_fs_component())
            .join(actor.scoped_user_fs_key())
            .join("company_profiles"))
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

fn actor_from_scoped_user_key(key: &str) -> Option<(Option<String>, String)> {
    let parts: Vec<&str> = key.splitn(2, "__").collect();
    if parts.len() != 2 {
        return None;
    }
    let scope_raw = decode_component(parts[0]);
    let user_id = decode_component(parts[1]);
    if user_id.is_empty() {
        return None;
    }
    let channel_scope = if scope_raw == "direct" {
        None
    } else {
        Some(scope_raw)
    };
    Some((channel_scope, user_id))
}

fn decode_component(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && i + 2 < bytes.len() {
            let hi = bytes[i + 1];
            let lo = bytes[i + 2];
            if let (Some(h), Some(l)) = (hex_digit(hi), hex_digit(lo)) {
                out.push(char::from(h * 16 + l));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
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
