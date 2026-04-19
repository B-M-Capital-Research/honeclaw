use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path};

use chrono::Utc;
use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::FileOptions;

use super::markdown::{
    file_modified_at_rfc3339, normalize_company_name, normalize_stock_code,
    parse_event_markdown_for_transfer, parse_profile_markdown_for_transfer, parse_profile_sections,
    safe_component_join, validate_storage_component,
};
use super::{
    CompanyProfileConflictDecision, CompanyProfileDocument, CompanyProfileEventDocument,
    CompanyProfileImportApplyInput, CompanyProfileImportApplyResult, CompanyProfileImportConflict,
    CompanyProfileImportConflictDetail, CompanyProfileImportDiffLine,
    CompanyProfileImportDiffLineKind, CompanyProfileImportEventDiff, CompanyProfileImportMode,
    CompanyProfileImportPreview, CompanyProfileImportProfileSummary,
    CompanyProfileImportResolutionInput, CompanyProfileImportResolutionResult,
    CompanyProfileImportResolutionStrategy, CompanyProfileImportSectionChangeType,
    CompanyProfileImportSectionDiff, CompanyProfileStorage, CompanyProfileTransferManifest,
    CompanyProfileTransferManifestProfile, ProfileMetadata,
};

pub(super) const COMPANY_PROFILE_BUNDLE_VERSION: &str = "company-profile-bundle-v1";

impl CompanyProfileStorage {
    pub fn export_bundle(&self) -> Result<Vec<u8>, String> {
        let documents = self.load_transfer_documents()?;
        let manifest = CompanyProfileTransferManifest {
            version: COMPANY_PROFILE_BUNDLE_VERSION.to_string(),
            exported_at: Utc::now().to_rfc3339(),
            profile_count: documents.len(),
            event_count: documents.iter().map(|document| document.events.len()).sum(),
            profiles: documents
                .iter()
                .map(|document| CompanyProfileTransferManifestProfile {
                    profile_id: document.profile_id.clone(),
                    company_name: document.metadata.company_name.clone(),
                    stock_code: document.metadata.stock_code.clone(),
                    event_count: document.events.len(),
                    updated_at: document.metadata.updated_at.clone(),
                })
                .collect(),
        };

        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = FileOptions::default();

        writer
            .start_file("manifest.json", options)
            .map_err(|err| format!("写入 manifest 失败: {err}"))?;
        let manifest_json = serde_json::to_vec_pretty(&manifest)
            .map_err(|err| format!("序列化 manifest 失败: {err}"))?;
        writer
            .write_all(&manifest_json)
            .map_err(|err| format!("写入 manifest 内容失败: {err}"))?;

        for document in &documents {
            writer
                .start_file(
                    format!("company_profiles/{}/profile.md", document.profile_id),
                    options,
                )
                .map_err(|err| format!("写入 profile.md 失败: {err}"))?;
            writer
                .write_all(document.markdown.as_bytes())
                .map_err(|err| format!("写入 profile.md 内容失败: {err}"))?;

            let mut events = document.events.clone();
            events.sort_by(|left, right| left.filename.cmp(&right.filename));
            for event in events {
                writer
                    .start_file(
                        format!(
                            "company_profiles/{}/events/{}",
                            document.profile_id, event.filename
                        ),
                        options,
                    )
                    .map_err(|err| format!("写入事件文件失败: {err}"))?;
                writer
                    .write_all(event.markdown.as_bytes())
                    .map_err(|err| format!("写入事件文件内容失败: {err}"))?;
            }
        }

        writer
            .finish()
            .map_err(|err| format!("关闭画像包 zip 失败: {err}"))
            .map(|cursor| cursor.into_inner())
    }

    pub fn preview_import_bundle(
        &self,
        bundle_bytes: &[u8],
    ) -> Result<CompanyProfileImportPreview, String> {
        let bundle = parse_company_profile_bundle(bundle_bytes)?;
        let existing_documents = self.load_transfer_documents()?;
        let mut profiles = Vec::with_capacity(bundle.documents.len());
        let mut conflicts = Vec::new();

        for document in &bundle.documents {
            let imported = import_profile_summary(document);
            profiles.push(imported.clone());
            if let Some((existing_document, reasons)) =
                find_best_conflict(&existing_documents, document)
            {
                conflicts.push(CompanyProfileImportConflict {
                    imported,
                    existing: import_profile_summary(existing_document),
                    reasons,
                });
            }
        }

        Ok(CompanyProfileImportPreview {
            manifest: bundle.manifest,
            profiles,
            importable_count: bundle.documents.len().saturating_sub(conflicts.len()),
            conflict_count: conflicts.len(),
            suggested_mode: if conflicts.is_empty() {
                CompanyProfileImportMode::KeepExisting
            } else {
                CompanyProfileImportMode::Interactive
            },
            conflicts,
        })
    }

    pub fn apply_import_bundle(
        &self,
        bundle_bytes: &[u8],
        input: CompanyProfileImportApplyInput,
    ) -> Result<CompanyProfileImportApplyResult, String> {
        let preview = self.preview_import_bundle(bundle_bytes)?;
        let bundle = parse_company_profile_bundle(bundle_bytes)?;
        let mode = input.mode.unwrap_or(CompanyProfileImportMode::Interactive);
        let conflict_decisions = input.decisions;
        let conflict_by_imported = preview
            .conflicts
            .iter()
            .map(|conflict| (conflict.imported.profile_id.clone(), conflict.clone()))
            .collect::<HashMap<_, _>>();

        if matches!(mode, CompanyProfileImportMode::Interactive) {
            for conflict in &preview.conflicts {
                if !conflict_decisions.contains_key(&conflict.imported.profile_id) {
                    return Err(format!(
                        "interactive 模式缺少冲突决策: {}",
                        conflict.imported.profile_id
                    ));
                }
            }
        }

        let mut imported_profile_ids = Vec::new();
        let mut replaced_profile_ids = Vec::new();
        let mut skipped_profile_ids = Vec::new();

        for document in &bundle.documents {
            let Some(conflict) = conflict_by_imported.get(&document.profile_id) else {
                self.write_imported_document(&document.profile_id, document, false)?;
                imported_profile_ids.push(document.profile_id.clone());
                continue;
            };

            let decision = match mode {
                CompanyProfileImportMode::KeepExisting => CompanyProfileConflictDecision::Skip,
                CompanyProfileImportMode::ReplaceAll => CompanyProfileConflictDecision::Replace,
                CompanyProfileImportMode::Interactive => conflict_decisions
                    .get(&document.profile_id)
                    .cloned()
                    .ok_or_else(|| {
                        format!("interactive 模式缺少冲突决策: {}", document.profile_id)
                    })?,
            };

            match decision {
                CompanyProfileConflictDecision::Skip => {
                    skipped_profile_ids.push(document.profile_id.clone());
                }
                CompanyProfileConflictDecision::Replace => {
                    self.write_imported_document(&conflict.existing.profile_id, document, true)?;
                    replaced_profile_ids.push(conflict.existing.profile_id.clone());
                }
            }
        }

        let changed_profile_ids = imported_profile_ids
            .iter()
            .chain(replaced_profile_ids.iter())
            .cloned()
            .collect::<Vec<_>>();

        Ok(CompanyProfileImportApplyResult {
            imported_count: imported_profile_ids.len(),
            replaced_count: replaced_profile_ids.len(),
            skipped_count: skipped_profile_ids.len(),
            imported_profile_ids,
            replaced_profile_ids,
            skipped_profile_ids,
            changed_profile_ids,
        })
    }

    pub fn describe_import_conflict(
        &self,
        bundle_bytes: &[u8],
        imported_profile_id: &str,
        section_title: Option<&str>,
    ) -> Result<CompanyProfileImportConflictDetail, String> {
        let preview = self.preview_import_bundle(bundle_bytes)?;
        let bundle = parse_company_profile_bundle(bundle_bytes)?;
        let conflict = preview
            .conflicts
            .into_iter()
            .find(|conflict| conflict.imported.profile_id == imported_profile_id)
            .ok_or_else(|| format!("未找到冲突公司: {imported_profile_id}"))?;
        let imported_document = bundle
            .documents
            .iter()
            .find(|document| document.profile_id == imported_profile_id)
            .ok_or_else(|| format!("画像包中不存在公司画像: {imported_profile_id}"))?;
        let existing_document = self
            .get_profile(&conflict.existing.profile_id)?
            .ok_or_else(|| format!("目标画像不存在: {}", conflict.existing.profile_id))?;
        build_conflict_detail(
            conflict,
            imported_document,
            &existing_document,
            section_title,
        )
    }

    pub fn apply_import_resolution(
        &self,
        bundle_bytes: &[u8],
        input: CompanyProfileImportResolutionInput,
    ) -> Result<CompanyProfileImportResolutionResult, String> {
        let preview = self.preview_import_bundle(bundle_bytes)?;
        let bundle = parse_company_profile_bundle(bundle_bytes)?;
        let imported_document = bundle
            .documents
            .iter()
            .find(|document| document.profile_id == input.imported_profile_id)
            .ok_or_else(|| format!("画像包中不存在公司画像: {}", input.imported_profile_id))?;
        let conflict = preview
            .conflicts
            .iter()
            .find(|conflict| conflict.imported.profile_id == input.imported_profile_id)
            .cloned();

        match conflict {
            None => match input.strategy {
                CompanyProfileImportResolutionStrategy::Skip => {
                    Ok(CompanyProfileImportResolutionResult {
                        imported_profile_id: imported_document.profile_id.clone(),
                        target_profile_id: imported_document.profile_id.clone(),
                        strategy: CompanyProfileImportResolutionStrategy::Skip,
                        created_new_profile: false,
                        replaced_existing_profile: false,
                        merged_existing_profile: false,
                        skipped: true,
                        changed_sections: Vec::new(),
                        imported_event_ids: Vec::new(),
                        skipped_event_ids: imported_document
                            .events
                            .iter()
                            .map(|event| event.id.clone())
                            .collect(),
                    })
                }
                CompanyProfileImportResolutionStrategy::Replace
                | CompanyProfileImportResolutionStrategy::MergeSections => {
                    self.write_imported_document(
                        &imported_document.profile_id,
                        imported_document,
                        false,
                    )?;
                    Ok(CompanyProfileImportResolutionResult {
                        imported_profile_id: imported_document.profile_id.clone(),
                        target_profile_id: imported_document.profile_id.clone(),
                        strategy: input.strategy,
                        created_new_profile: true,
                        replaced_existing_profile: false,
                        merged_existing_profile: false,
                        skipped: false,
                        changed_sections: changed_imported_section_titles(imported_document),
                        imported_event_ids: imported_document
                            .events
                            .iter()
                            .map(|event| event.id.clone())
                            .collect(),
                        skipped_event_ids: Vec::new(),
                    })
                }
            },
            Some(conflict) => match input.strategy {
                CompanyProfileImportResolutionStrategy::Skip => {
                    Ok(CompanyProfileImportResolutionResult {
                        imported_profile_id: imported_document.profile_id.clone(),
                        target_profile_id: conflict.existing.profile_id,
                        strategy: CompanyProfileImportResolutionStrategy::Skip,
                        created_new_profile: false,
                        replaced_existing_profile: false,
                        merged_existing_profile: false,
                        skipped: true,
                        changed_sections: Vec::new(),
                        imported_event_ids: Vec::new(),
                        skipped_event_ids: imported_document
                            .events
                            .iter()
                            .map(|event| event.id.clone())
                            .collect(),
                    })
                }
                CompanyProfileImportResolutionStrategy::Replace => {
                    self.write_imported_document(
                        &conflict.existing.profile_id,
                        imported_document,
                        true,
                    )?;
                    Ok(CompanyProfileImportResolutionResult {
                        imported_profile_id: imported_document.profile_id.clone(),
                        target_profile_id: conflict.existing.profile_id,
                        strategy: CompanyProfileImportResolutionStrategy::Replace,
                        created_new_profile: false,
                        replaced_existing_profile: true,
                        merged_existing_profile: false,
                        skipped: false,
                        changed_sections: changed_imported_section_titles(imported_document),
                        imported_event_ids: imported_document
                            .events
                            .iter()
                            .map(|event| event.id.clone())
                            .collect(),
                        skipped_event_ids: Vec::new(),
                    })
                }
                CompanyProfileImportResolutionStrategy::MergeSections => self
                    .merge_imported_sections(
                        &conflict.existing.profile_id,
                        imported_document,
                        &input,
                    ),
            },
        }
    }

    fn load_transfer_documents(&self) -> Result<Vec<CompanyProfileDocument>, String> {
        let root_dir = self.scoped_root()?;
        if !root_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&root_dir).map_err(|err| format!("读取画像目录失败: {err}"))?;
        let mut documents = Vec::new();
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            if let Some(document) = self.load_transfer_profile_by_dir(&dir)? {
                documents.push(document);
            }
        }
        documents.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(documents)
    }

    fn write_imported_document(
        &self,
        target_profile_id: &str,
        document: &CompanyProfileDocument,
        replace_existing: bool,
    ) -> Result<(), String> {
        let root_dir = self.scoped_root()?;
        let Some(profile_dir) = safe_component_join(&root_dir, target_profile_id) else {
            return Err("目标画像目录非法".to_string());
        };

        if replace_existing && profile_dir.exists() {
            fs::remove_dir_all(&profile_dir).map_err(|err| format!("替换画像目录失败: {err}"))?;
        } else if !replace_existing && profile_dir.exists() {
            return Err(format!("目标画像目录已存在: {target_profile_id}"));
        }

        fs::create_dir_all(profile_dir.join("events"))
            .map_err(|err| format!("创建画像目录失败: {err}"))?;
        fs::write(profile_dir.join("profile.md"), document.markdown.as_bytes())
            .map_err(|err| format!("写 profile.md 失败: {err}"))?;

        for event in &document.events {
            let Some(filename) = validate_storage_component(&event.filename) else {
                return Err(format!("事件文件名非法: {}", event.filename));
            };
            fs::write(
                profile_dir.join("events").join(filename),
                event.markdown.as_bytes(),
            )
            .map_err(|err| format!("写事件文件失败: {err}"))?;
        }

        Ok(())
    }

    fn write_event_document(
        &self,
        profile_id: &str,
        event: &CompanyProfileEventDocument,
        overwrite: bool,
    ) -> Result<(), String> {
        let root_dir = self.scoped_root()?;
        let Some(profile_dir) = safe_component_join(&root_dir, profile_id) else {
            return Err("目标画像目录非法".to_string());
        };
        fs::create_dir_all(profile_dir.join("events"))
            .map_err(|err| format!("创建事件目录失败: {err}"))?;
        let Some(filename) = validate_storage_component(&event.filename) else {
            return Err(format!("事件文件名非法: {}", event.filename));
        };
        let event_path = profile_dir.join("events").join(filename);
        if event_path.exists() && !overwrite {
            return Ok(());
        }
        fs::write(&event_path, event.markdown.as_bytes())
            .map_err(|err| format!("写事件文件失败: {err}"))?;
        Ok(())
    }

    fn load_transfer_profile_by_dir(
        &self,
        dir: &Path,
    ) -> Result<Option<CompanyProfileDocument>, String> {
        let profile_path = dir.join("profile.md");
        if !profile_path.exists() {
            return Ok(None);
        }

        let profile_id = dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let raw_markdown = fs::read_to_string(&profile_path)
            .map_err(|err| format!("读取 profile.md 失败: {err}"))?;
        let updated_at = file_modified_at_rfc3339(&profile_path);
        let (metadata, markdown) =
            parse_profile_markdown_for_transfer(&profile_id, &raw_markdown, updated_at)?;
        let events = self.load_transfer_events(dir)?;

        Ok(Some(CompanyProfileDocument {
            profile_id,
            metadata,
            markdown,
            events,
        }))
    }

    fn load_transfer_events(
        &self,
        profile_dir: &Path,
    ) -> Result<Vec<CompanyProfileEventDocument>, String> {
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
            let raw_markdown =
                fs::read_to_string(&path).map_err(|err| format!("读取事件文件失败: {err}"))?;
            let updated_at = file_modified_at_rfc3339(&path);
            events.push(parse_event_markdown_for_transfer(
                &id,
                &filename,
                &raw_markdown,
                updated_at,
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

    fn merge_imported_sections(
        &self,
        target_profile_id: &str,
        imported_document: &CompanyProfileDocument,
        input: &CompanyProfileImportResolutionInput,
    ) -> Result<CompanyProfileImportResolutionResult, String> {
        let existing_document = self
            .get_profile(target_profile_id)?
            .ok_or_else(|| format!("目标画像不存在: {target_profile_id}"))?;
        let mergeable_titles = mergeable_section_titles(imported_document, &existing_document);
        let selected_titles = if input.section_titles.is_empty() {
            mergeable_titles.clone()
        } else {
            let requested = normalize_section_titles(&input.section_titles);
            let available = mergeable_titles
                .iter()
                .map(|title| normalize_company_name(title))
                .collect::<HashSet<_>>();
            for title in &requested {
                if !available.contains(&normalize_company_name(title)) {
                    return Err(format!("不能合并不存在的导入 section: {title}"));
                }
            }
            requested
        };

        let imported_sections = parse_profile_sections(&imported_document.markdown).0;
        let imported_by_title = imported_sections
            .into_iter()
            .map(|(title, content)| (normalize_company_name(&title), (title, content)))
            .collect::<HashMap<_, _>>();
        let mut updates = BTreeMap::new();
        let mut changed_sections = Vec::new();
        for title in &selected_titles {
            let normalized = normalize_company_name(title);
            let Some((canonical_title, content)) = imported_by_title.get(&normalized) else {
                continue;
            };
            updates.insert(canonical_title.clone(), content.clone());
            changed_sections.push(canonical_title.clone());
        }
        if !updates.is_empty() {
            self.rewrite_sections(target_profile_id, &updates)?
                .ok_or_else(|| format!("回写画像失败: {target_profile_id}"))?;
        }

        let existing_event_ids = existing_document
            .events
            .iter()
            .map(|event| event.filename.clone())
            .collect::<HashSet<_>>();
        let mut imported_event_ids = Vec::new();
        let mut skipped_event_ids = Vec::new();
        if input.import_missing_events {
            for event in &imported_document.events {
                if existing_event_ids.contains(&event.filename) {
                    skipped_event_ids.push(event.id.clone());
                    continue;
                }
                self.write_event_document(target_profile_id, event, false)?;
                imported_event_ids.push(event.id.clone());
            }
        } else {
            skipped_event_ids = imported_document
                .events
                .iter()
                .map(|event| event.id.clone())
                .collect();
        }

        if updates.is_empty() && !imported_event_ids.is_empty() {
            let _ = self.touch_profile_updated_at(target_profile_id)?;
        }

        Ok(CompanyProfileImportResolutionResult {
            imported_profile_id: imported_document.profile_id.clone(),
            target_profile_id: target_profile_id.to_string(),
            strategy: CompanyProfileImportResolutionStrategy::MergeSections,
            created_new_profile: false,
            replaced_existing_profile: false,
            merged_existing_profile: true,
            skipped: updates.is_empty() && imported_event_ids.is_empty(),
            changed_sections,
            imported_event_ids,
            skipped_event_ids,
        })
    }
}

#[derive(Debug)]
struct ParsedCompanyProfileBundle {
    manifest: CompanyProfileTransferManifest,
    documents: Vec<CompanyProfileDocument>,
}

enum BundleEntryPath {
    Profile {
        profile_id: String,
    },
    Event {
        profile_id: String,
        filename: String,
    },
}

fn parse_company_profile_bundle(bundle_bytes: &[u8]) -> Result<ParsedCompanyProfileBundle, String> {
    let reader = Cursor::new(bundle_bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|err| format!("读取画像包 zip 失败: {err}"))?;
    let mut manifest: Option<CompanyProfileTransferManifest> = None;
    let mut profiles = BTreeMap::<String, String>::new();
    let mut events = HashMap::<String, Vec<(String, String)>>::new();

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|err| format!("读取画像包条目失败: {err}"))?;
        if file.is_dir() {
            continue;
        }

        let name = normalize_zip_entry_name(file.name());
        if name == "manifest.json" {
            if manifest.is_some() {
                return Err("画像包包含重复 manifest.json".to_string());
            }
            let content = read_zip_text(&mut file, "manifest.json")?;
            let parsed: CompanyProfileTransferManifest = serde_json::from_str(&content)
                .map_err(|err| format!("解析 manifest.json 失败: {err}"))?;
            if parsed.version.trim() != COMPANY_PROFILE_BUNDLE_VERSION {
                return Err(format!("不支持的画像包版本: {}", parsed.version.trim()));
            }
            manifest = Some(parsed);
            continue;
        }

        match parse_bundle_entry_path(&name)? {
            BundleEntryPath::Profile { profile_id } => {
                if profiles.contains_key(&profile_id) {
                    return Err(format!("画像包包含重复 profile.md: {profile_id}"));
                }
                let content = read_zip_text(&mut file, &name)?;
                profiles.insert(profile_id, content);
            }
            BundleEntryPath::Event {
                profile_id,
                filename,
            } => {
                let content = read_zip_text(&mut file, &name)?;
                events
                    .entry(profile_id)
                    .or_default()
                    .push((filename, content));
            }
        }
    }

    let manifest = manifest.ok_or_else(|| "画像包缺少 manifest.json".to_string())?;
    if profiles.len() != manifest.profile_count {
        return Err(format!(
            "画像包画像数与 manifest 不一致: manifest={} actual={}",
            manifest.profile_count,
            profiles.len()
        ));
    }

    let manifest_profiles = manifest
        .profiles
        .iter()
        .map(|profile| (profile.profile_id.clone(), profile.clone()))
        .collect::<HashMap<_, _>>();
    let mut documents = Vec::with_capacity(profiles.len());
    for (profile_id, markdown) in profiles {
        let profile_manifest = manifest_profiles
            .get(&profile_id)
            .ok_or_else(|| format!("画像包 manifest 中缺少画像条目: {profile_id}"))?;
        let (metadata, markdown) = parse_profile_markdown_for_transfer(
            &profile_id,
            &markdown,
            Some(profile_manifest.updated_at.clone()),
        )?;
        let mut parsed_events = events.remove(&profile_id).unwrap_or_default();
        let mut event_documents = Vec::with_capacity(parsed_events.len());
        for (filename, content) in parsed_events.drain(..) {
            let id = filename.trim_end_matches(".md").to_string();
            event_documents.push(parse_event_markdown_for_transfer(
                &id,
                &filename,
                &content,
                Some(profile_manifest.updated_at.clone()),
            )?);
        }
        event_documents.sort_by(|left, right| {
            right
                .metadata
                .occurred_at
                .cmp(&left.metadata.occurred_at)
                .then_with(|| right.filename.cmp(&left.filename))
        });
        documents.push(CompanyProfileDocument {
            profile_id,
            metadata,
            markdown,
            events: event_documents,
        });
    }

    if !events.is_empty() {
        let orphan = events.keys().next().cloned().unwrap_or_default();
        return Err(format!("画像包包含孤立事件目录: {orphan}"));
    }

    documents.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
    validate_bundle_manifest(&manifest, &documents)?;
    validate_bundle_duplicates(&documents)?;

    Ok(ParsedCompanyProfileBundle {
        manifest,
        documents,
    })
}

fn normalize_zip_entry_name(name: &str) -> String {
    name.replace('\\', "/")
}

fn read_zip_text<R: Read>(reader: &mut R, entry_name: &str) -> Result<String, String> {
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(|err| format!("读取画像包条目失败 ({entry_name}): {err}"))?;
    Ok(content)
}

fn parse_bundle_entry_path(path: &str) -> Result<BundleEntryPath, String> {
    let components = Path::new(path)
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(|value| value.to_string())
                .ok_or_else(|| format!("画像包路径不是有效 UTF-8: {path}")),
            _ => Err(format!("画像包路径非法: {path}")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    match components.as_slice() {
        [root, profile_id, filename] if root == "company_profiles" && filename == "profile.md" => {
            let profile_id = validate_storage_component(profile_id)
                .ok_or_else(|| format!("画像包 profile_id 非法: {profile_id}"))?;
            Ok(BundleEntryPath::Profile { profile_id })
        }
        [root, profile_id, events_dir, filename]
            if root == "company_profiles"
                && events_dir == "events"
                && filename.ends_with(".md") =>
        {
            let profile_id = validate_storage_component(profile_id)
                .ok_or_else(|| format!("画像包 profile_id 非法: {profile_id}"))?;
            let filename = validate_storage_component(filename)
                .ok_or_else(|| format!("画像包事件文件名非法: {filename}"))?;
            Ok(BundleEntryPath::Event {
                profile_id,
                filename,
            })
        }
        _ => Err(format!("画像包包含不支持的路径: {path}")),
    }
}

fn validate_bundle_manifest(
    manifest: &CompanyProfileTransferManifest,
    documents: &[CompanyProfileDocument],
) -> Result<(), String> {
    let actual_event_count = documents
        .iter()
        .map(|document| document.events.len())
        .sum::<usize>();
    if manifest.event_count != actual_event_count {
        return Err(format!(
            "画像包事件数与 manifest 不一致: manifest={} actual={}",
            manifest.event_count, actual_event_count
        ));
    }

    let actual_ids = documents
        .iter()
        .map(|document| document.profile_id.clone())
        .collect::<Vec<_>>();
    let manifest_ids = manifest
        .profiles
        .iter()
        .map(|profile| profile.profile_id.clone())
        .collect::<Vec<_>>();
    if actual_ids != manifest_ids {
        return Err("画像包 manifest 中的画像列表与实际内容不一致".to_string());
    }

    for (profile, document) in manifest.profiles.iter().zip(documents.iter()) {
        if profile.company_name.trim() != document.metadata.company_name.trim()
            || normalize_stock_code(&profile.stock_code)
                != normalize_stock_code(&document.metadata.stock_code)
            || profile.event_count != document.events.len()
            || profile.updated_at.trim() != document.metadata.updated_at.trim()
        {
            return Err(format!(
                "画像包 manifest 与内容不一致: {}",
                profile.profile_id
            ));
        }
    }

    Ok(())
}

fn validate_bundle_duplicates(documents: &[CompanyProfileDocument]) -> Result<(), String> {
    for left_index in 0..documents.len() {
        for right_index in (left_index + 1)..documents.len() {
            let reasons = conflict_reasons(&documents[left_index], &documents[right_index]);
            if !reasons.is_empty() {
                return Err(format!(
                    "画像包内存在重复公司画像: {} 与 {} ({})",
                    documents[left_index].profile_id,
                    documents[right_index].profile_id,
                    reasons.join("、")
                ));
            }
        }
    }
    Ok(())
}

fn import_profile_summary(document: &CompanyProfileDocument) -> CompanyProfileImportProfileSummary {
    CompanyProfileImportProfileSummary {
        profile_id: document.profile_id.clone(),
        company_name: document.metadata.company_name.clone(),
        stock_code: document.metadata.stock_code.clone(),
        updated_at: document.metadata.updated_at.clone(),
        event_count: document.events.len(),
        thesis_excerpt: thesis_excerpt_from_markdown(&document.markdown),
    }
}

fn thesis_excerpt_from_markdown(markdown: &str) -> String {
    let (sections, extra_lines) = parse_profile_sections(markdown);
    let content = sections
        .iter()
        .find(|(title, _)| title.trim().eq_ignore_ascii_case("Thesis"))
        .map(|(_, content)| content.trim().to_string())
        .or_else(|| {
            sections.iter().find_map(|(_, content)| {
                (!content.trim().is_empty()).then(|| content.trim().to_string())
            })
        })
        .or_else(|| {
            let joined = extra_lines.join("\n");
            (!joined.trim().is_empty()).then(|| joined.trim().to_string())
        })
        .unwrap_or_default();

    let lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    truncate_chars(&lines, 180)
}

fn build_conflict_detail(
    conflict: CompanyProfileImportConflict,
    imported_document: &CompanyProfileDocument,
    existing_document: &CompanyProfileDocument,
    section_title: Option<&str>,
) -> Result<CompanyProfileImportConflictDetail, String> {
    let imported_sections = parse_profile_sections(&imported_document.markdown).0;
    let existing_sections = parse_profile_sections(&existing_document.markdown).0;
    let all_titles = changed_section_titles(imported_document, existing_document);
    let filter = section_title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_company_name);
    let imported_by_title = imported_sections
        .iter()
        .map(|(title, content)| {
            (
                normalize_company_name(title),
                (title.clone(), content.clone()),
            )
        })
        .collect::<HashMap<_, _>>();
    let existing_by_title = existing_sections
        .iter()
        .map(|(title, content)| {
            (
                normalize_company_name(title),
                (title.clone(), content.clone()),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut section_diffs = Vec::new();
    for title in &all_titles {
        let normalized = normalize_company_name(title);
        if let Some(filter) = filter.as_ref()
            && &normalized != filter
        {
            continue;
        }
        let imported = imported_by_title.get(&normalized);
        let existing = existing_by_title.get(&normalized);
        let (change_type, line_diff) = match (imported, existing) {
            (Some((_, imported_content)), Some((_, existing_content))) => {
                if imported_content.trim() == existing_content.trim() {
                    continue;
                }
                (
                    CompanyProfileImportSectionChangeType::Modified,
                    diff_section_lines(existing_content, imported_content),
                )
            }
            (Some((_, imported_content)), None) => (
                CompanyProfileImportSectionChangeType::ImportedOnly,
                imported_content
                    .lines()
                    .map(|line| CompanyProfileImportDiffLine {
                        kind: CompanyProfileImportDiffLineKind::Added,
                        text: line.to_string(),
                    })
                    .collect(),
            ),
            (None, Some((_, existing_content))) => (
                CompanyProfileImportSectionChangeType::ExistingOnly,
                existing_content
                    .lines()
                    .map(|line| CompanyProfileImportDiffLine {
                        kind: CompanyProfileImportDiffLineKind::Removed,
                        text: line.to_string(),
                    })
                    .collect(),
            ),
            (None, None) => continue,
        };
        section_diffs.push(CompanyProfileImportSectionDiff {
            section_title: title.clone(),
            change_type,
            line_diff,
            imported_excerpt: imported
                .map(|(_, content)| truncate_chars(content.trim(), 240))
                .unwrap_or_default(),
            existing_excerpt: existing
                .map(|(_, content)| truncate_chars(content.trim(), 240))
                .unwrap_or_default(),
        });
    }

    if filter.is_some() && section_diffs.is_empty() {
        return Err("指定 section 没有发现可解释的冲突差异".to_string());
    }

    Ok(CompanyProfileImportConflictDetail {
        conflict,
        available_section_titles: all_titles,
        section_diffs,
        event_diff: event_diff(imported_document, existing_document),
    })
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let truncated = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn find_best_conflict<'a>(
    existing_documents: &'a [CompanyProfileDocument],
    imported_document: &CompanyProfileDocument,
) -> Option<(&'a CompanyProfileDocument, Vec<String>)> {
    existing_documents
        .iter()
        .filter_map(|existing_document| {
            let reasons = conflict_reasons(existing_document, imported_document);
            (!reasons.is_empty()).then_some((existing_document, reasons))
        })
        .max_by(
            |(left_document, left_reasons), (right_document, right_reasons)| {
                conflict_score(left_reasons)
                    .cmp(&conflict_score(right_reasons))
                    .then_with(|| right_document.profile_id.cmp(&left_document.profile_id))
            },
        )
}

fn changed_section_titles(
    imported_document: &CompanyProfileDocument,
    existing_document: &CompanyProfileDocument,
) -> Vec<String> {
    let imported_sections = parse_profile_sections(&imported_document.markdown).0;
    let existing_sections = parse_profile_sections(&existing_document.markdown).0;
    let existing_by_title = existing_sections
        .iter()
        .map(|(title, content)| (normalize_company_name(title), content.trim().to_string()))
        .collect::<HashMap<_, _>>();
    let mut titles = Vec::new();
    let mut seen = HashSet::new();

    for (title, content) in &imported_sections {
        let normalized = normalize_company_name(title);
        let changed = existing_by_title
            .get(&normalized)
            .map(|existing| existing != content.trim())
            .unwrap_or(true);
        if changed && seen.insert(normalized.clone()) {
            titles.push(title.clone());
        }
    }
    for (title, content) in &existing_sections {
        let normalized = normalize_company_name(title);
        let missing_from_imported = !imported_sections
            .iter()
            .any(|(imported_title, _)| normalize_company_name(imported_title) == normalized);
        if missing_from_imported && !content.trim().is_empty() && seen.insert(normalized) {
            titles.push(title.clone());
        }
    }
    titles
}

fn mergeable_section_titles(
    imported_document: &CompanyProfileDocument,
    existing_document: &CompanyProfileDocument,
) -> Vec<String> {
    let imported_sections = parse_profile_sections(&imported_document.markdown).0;
    let existing_sections = parse_profile_sections(&existing_document.markdown).0;
    let existing_by_title = existing_sections
        .iter()
        .map(|(title, content)| (normalize_company_name(title), content.trim().to_string()))
        .collect::<HashMap<_, _>>();
    let mut titles = Vec::new();
    for (title, content) in imported_sections {
        let normalized = normalize_company_name(&title);
        let changed = existing_by_title
            .get(&normalized)
            .map(|existing| existing != content.trim())
            .unwrap_or(true);
        if changed {
            titles.push(title);
        }
    }
    titles
}

fn normalize_section_titles(section_titles: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for title in section_titles {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = normalize_company_name(trimmed);
        if seen.insert(normalized) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn changed_imported_section_titles(document: &CompanyProfileDocument) -> Vec<String> {
    parse_profile_sections(&document.markdown)
        .0
        .into_iter()
        .map(|(title, _)| title)
        .collect()
}

fn event_diff(
    imported_document: &CompanyProfileDocument,
    existing_document: &CompanyProfileDocument,
) -> CompanyProfileImportEventDiff {
    let imported_ids = imported_document
        .events
        .iter()
        .map(|event| event.id.clone())
        .collect::<HashSet<_>>();
    let existing_ids = existing_document
        .events
        .iter()
        .map(|event| event.id.clone())
        .collect::<HashSet<_>>();
    let mut imported_only_event_ids = imported_ids
        .difference(&existing_ids)
        .cloned()
        .collect::<Vec<_>>();
    let mut existing_only_event_ids = existing_ids
        .difference(&imported_ids)
        .cloned()
        .collect::<Vec<_>>();
    let mut shared_event_ids = imported_ids
        .intersection(&existing_ids)
        .cloned()
        .collect::<Vec<_>>();
    imported_only_event_ids.sort();
    existing_only_event_ids.sort();
    shared_event_ids.sort();
    CompanyProfileImportEventDiff {
        imported_only_event_ids,
        existing_only_event_ids,
        shared_event_ids,
    }
}

fn diff_section_lines(existing: &str, imported: &str) -> Vec<CompanyProfileImportDiffLine> {
    let existing_lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let imported_lines = imported.lines().map(str::to_string).collect::<Vec<_>>();
    let n = existing_lines.len();
    let m = imported_lines.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if existing_lines[i] == imported_lines[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut i = 0usize;
    let mut j = 0usize;
    let mut out = Vec::new();
    while i < n && j < m {
        if existing_lines[i] == imported_lines[j] {
            out.push(CompanyProfileImportDiffLine {
                kind: CompanyProfileImportDiffLineKind::Context,
                text: existing_lines[i].clone(),
            });
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push(CompanyProfileImportDiffLine {
                kind: CompanyProfileImportDiffLineKind::Removed,
                text: existing_lines[i].clone(),
            });
            i += 1;
        } else {
            out.push(CompanyProfileImportDiffLine {
                kind: CompanyProfileImportDiffLineKind::Added,
                text: imported_lines[j].clone(),
            });
            j += 1;
        }
    }
    while i < n {
        out.push(CompanyProfileImportDiffLine {
            kind: CompanyProfileImportDiffLineKind::Removed,
            text: existing_lines[i].clone(),
        });
        i += 1;
    }
    while j < m {
        out.push(CompanyProfileImportDiffLine {
            kind: CompanyProfileImportDiffLineKind::Added,
            text: imported_lines[j].clone(),
        });
        j += 1;
    }
    out
}

fn conflict_score(reasons: &[String]) -> usize {
    reasons
        .iter()
        .map(|reason| match reason.as_str() {
            "股票代码相同" => 8,
            "公司名相同" => 4,
            "别名命中" => 2,
            "目录名相同" => 1,
            _ => 0,
        })
        .sum()
}

fn conflict_reasons(left: &CompanyProfileDocument, right: &CompanyProfileDocument) -> Vec<String> {
    let mut reasons = Vec::new();
    let left_stock = normalize_stock_code(&left.metadata.stock_code);
    let right_stock = normalize_stock_code(&right.metadata.stock_code);
    if !left_stock.is_empty() && left_stock == right_stock {
        reasons.push("股票代码相同".to_string());
    }

    if normalize_company_name(&left.metadata.company_name)
        == normalize_company_name(&right.metadata.company_name)
    {
        reasons.push("公司名相同".to_string());
    }

    if left.profile_id == right.profile_id {
        reasons.push("目录名相同".to_string());
    }

    let left_aliases = profile_identity_tokens(&left.metadata);
    let right_aliases = profile_identity_tokens(&right.metadata);
    if left_aliases.intersection(&right_aliases).next().is_some()
        && !reasons
            .iter()
            .any(|reason| reason == "公司名相同" || reason == "股票代码相同")
    {
        reasons.push("别名命中".to_string());
    } else if left_aliases.intersection(&right_aliases).next().is_some()
        && !reasons.iter().any(|reason| reason == "别名命中")
    {
        reasons.push("别名命中".to_string());
    }

    reasons
}

fn profile_identity_tokens(metadata: &ProfileMetadata) -> HashSet<String> {
    let mut tokens = HashSet::new();
    let company_name = normalize_company_name(&metadata.company_name);
    if !company_name.is_empty() {
        tokens.insert(company_name);
    }
    let stock_code = normalize_stock_code(&metadata.stock_code);
    if !stock_code.is_empty() {
        tokens.insert(stock_code.to_lowercase());
    }
    for alias in &metadata.aliases {
        let normalized = normalize_company_name(alias);
        if !normalized.is_empty() {
            tokens.insert(normalized);
        }
    }
    tokens
}
