use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use hone_core::ActorIdentity;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use super::markdown::{base_profile_sections, sanitize_id};
use super::transfer::COMPANY_PROFILE_BUNDLE_VERSION;
use super::*;

fn test_actor(channel: &str, user_id: &str, scope: Option<&str>) -> ActorIdentity {
    ActorIdentity::new(channel, user_id, scope).expect("actor")
}

fn scoped_storage(
    dir: &Path,
    channel: &str,
    user_id: &str,
    scope: Option<&str>,
) -> CompanyProfileStorage {
    let actor = test_actor(channel, user_id, scope);
    CompanyProfileStorage::new(dir).for_actor(&actor)
}

fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn create_profile_with_thesis(
    storage: &CompanyProfileStorage,
    company_name: &str,
    stock_code: &str,
    thesis: &str,
) -> CompanyProfileDocument {
    let (document, _) = storage
        .create_profile(CreateProfileInput {
            company_name: company_name.to_string(),
            stock_code: Some(stock_code.to_string()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::General,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("create profile");

    let mut sections = BTreeMap::new();
    sections.insert("投资主线".to_string(), thesis.to_string());
    storage
        .rewrite_sections(&document.profile_id, &sections)
        .expect("rewrite mainline")
        .expect("profile exists")
}

fn write_plain_markdown_profile(
    root: &Path,
    actor: &ActorIdentity,
    profile_id: &str,
    profile_markdown: &str,
    events: &[(&str, &str)],
) {
    let profile_dir = root
        .join(actor.channel_fs_component())
        .join(actor.scoped_user_fs_key())
        .join("company_profiles")
        .join(profile_id);
    fs::create_dir_all(profile_dir.join("events")).expect("create raw profile dirs");
    fs::write(profile_dir.join("profile.md"), profile_markdown).expect("write raw profile");
    for (filename, markdown) in events {
        fs::write(profile_dir.join("events").join(filename), markdown).expect("write raw event");
    }
}

fn build_invalid_bundle(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = FileOptions::default();
    for (path, content) in entries {
        writer.start_file(*path, options).expect("start file");
        writer
            .write_all(content.as_bytes())
            .expect("write bundle file");
    }
    writer.finish().expect("finish bundle").into_inner()
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
    let actor = test_actor("discord", "alice", Some("watchlist"));
    let storage = CompanyProfileStorage::new(&dir).for_actor(&actor);

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
    assert!(document.markdown.contains("## 投资主线"));
    assert!(document.markdown.contains("## 关键经营指标"));
    assert!(document.markdown.contains("ARR"));
    assert!(
        dir.join(actor.channel_fs_component())
            .join(actor.scoped_user_fs_key())
            .join("company_profiles")
            .join("SNOW")
            .join("profile.md")
            .exists()
    );
}

#[test]
fn create_profile_reuses_existing_profile_and_merges_aliases() {
    let dir = make_temp_dir("company_profile_alias");
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));

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
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));

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
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));

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
        mainline_impact: "mixed".to_string(),
        changed_sections: vec!["财务质量".to_string(), "关键跟踪清单".to_string()],
        refs: vec!["earnings-call".to_string()],
        what_happened: "毛利率承压，但储能业务增长延续。".to_string(),
        why_it_matters: "汽车与储能盈利结构的分化，决定市场是否继续给予成长溢价。".to_string(),
        mainline_effect: "汽车业务压力仍需观察，储能继续改善长期结构。".to_string(),
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
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));

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
fn profiles_are_isolated_by_actor_space() {
    let dir = make_temp_dir("company_profile_isolation");
    let alice = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    let bob = scoped_storage(&dir, "discord", "bob", Some("watchlist"));

    let (alice_profile, _) = alice
        .create_profile(CreateProfileInput {
            company_name: "ServiceNow".to_string(),
            stock_code: Some("NOW".to_string()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::Saas,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("alice profile");

    let (bob_profile, _) = bob
        .create_profile(CreateProfileInput {
            company_name: "CrowdStrike".to_string(),
            stock_code: Some("CRWD".to_string()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::Saas,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("bob profile");

    let alice_profiles = alice.list_profiles();
    let bob_profiles = bob.list_profiles();

    assert_eq!(alice_profiles.len(), 1);
    assert_eq!(bob_profiles.len(), 1);
    assert_eq!(alice_profiles[0].profile_id, alice_profile.profile_id);
    assert_eq!(bob_profiles[0].profile_id, bob_profile.profile_id);
    assert!(
        alice
            .get_profile(&bob_profile.profile_id)
            .expect("alice load")
            .is_none()
    );
    assert!(
        bob.get_profile(&alice_profile.profile_id)
            .expect("bob load")
            .is_none()
    );
}

#[test]
fn list_profile_spaces_summarizes_actor_roots() {
    let dir = make_temp_dir("company_profile_spaces");
    let base = CompanyProfileStorage::new(&dir);
    let alice_actor = test_actor("discord", "alice", Some("watchlist"));
    let bob_actor = test_actor("telegram", "bob", None::<&str>);
    let alice = base.for_actor(&alice_actor);
    let bob = base.for_actor(&bob_actor);

    let _ = alice
        .create_profile(CreateProfileInput {
            company_name: "Microsoft".to_string(),
            stock_code: Some("MSFT".to_string()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::General,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("alice profile");
    let _ = bob
        .create_profile(CreateProfileInput {
            company_name: "Visa".to_string(),
            stock_code: Some("V".to_string()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::Financials,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("bob profile");

    let spaces = base.list_profile_spaces();
    assert_eq!(spaces.len(), 2);
    assert!(spaces.iter().any(|space| {
        space.channel == alice_actor.channel
            && space.user_id == alice_actor.user_id
            && space.channel_scope == alice_actor.channel_scope
            && space.profile_count == 1
    }));
    assert!(spaces.iter().any(|space| {
        space.channel == bob_actor.channel
            && space.user_id == bob_actor.user_id
            && space.channel_scope == bob_actor.channel_scope
            && space.profile_count == 1
    }));
}

#[test]
fn raw_listing_reads_plain_markdown_profiles() {
    let dir = make_temp_dir("company_profile_raw_listing");
    let base = CompanyProfileStorage::new(&dir);
    let actor = test_actor("telegram", "alice", None::<&str>);
    write_plain_markdown_profile(
        &dir,
        &actor,
        "plain-profile",
        "# Plain Profile\n\n## Notes\nhello raw world\n",
        &[("2026-04-13-update.md", "# Fresh Event\n\nbody\n")],
    );

    let spaces = base.list_profile_spaces_raw();
    assert_eq!(spaces.len(), 1);
    assert_eq!(spaces[0].channel, actor.channel);
    assert_eq!(spaces[0].user_id, actor.user_id);
    assert_eq!(spaces[0].profile_count, 1);

    let profiles = base.for_actor(&actor).list_profiles_raw();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].profile_id, "plain-profile");
    assert_eq!(profiles[0].title, "Plain Profile");
    assert_eq!(profiles[0].event_count, 1);

    let detail = base
        .for_actor(&actor)
        .get_profile_raw("plain-profile")
        .expect("load raw profile")
        .expect("raw profile exists");
    assert_eq!(detail.title, "Plain Profile");
    assert_eq!(detail.events.len(), 1);
    assert_eq!(detail.events[0].title, "Fresh Event");
}

#[test]
fn export_bundle_normalizes_plain_markdown_profiles_without_frontmatter() {
    let dir = make_temp_dir("company_profile_export_plain_markdown");
    let base = CompanyProfileStorage::new(&dir);
    let actor = test_actor("discord", "alice", Some("watchlist"));
    write_plain_markdown_profile(
        &dir,
        &actor,
        "AAPL",
        "# Apple Inc.\n\n## Thesis\nlegacy plain markdown thesis\n",
        &[(
            "2026-04-13-update.md",
            "# Earnings Update\n\nplain event body\n",
        )],
    );

    let bytes = base
        .for_actor(&actor)
        .export_bundle()
        .expect("export plain markdown bundle");
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes.clone())).expect("archive");

    let mut profile_markdown = String::new();
    archive
        .by_name("company_profiles/AAPL/profile.md")
        .expect("profile entry")
        .read_to_string(&mut profile_markdown)
        .expect("read profile entry");
    assert!(profile_markdown.starts_with("---\n"));
    assert!(profile_markdown.contains("company_name: Apple Inc."));

    let mut event_markdown = String::new();
    archive
        .by_name("company_profiles/AAPL/events/2026-04-13-update.md")
        .expect("event entry")
        .read_to_string(&mut event_markdown)
        .expect("read event entry");
    assert!(event_markdown.starts_with("---\n"));
    assert!(event_markdown.contains("# Earnings Update"));

    let target = scoped_storage(&dir, "discord", "bob", Some("watchlist"));
    let preview = target
        .preview_import_bundle(&bytes)
        .expect("preview normalized bundle");
    assert_eq!(preview.conflict_count, 0);
    assert_eq!(preview.profiles.len(), 1);
    assert_eq!(preview.profiles[0].company_name, "Apple Inc.");
    assert_eq!(preview.profiles[0].stock_code, "AAPL");
}

#[test]
fn list_and_get_profile_tolerate_plain_markdown_without_frontmatter() {
    let dir = make_temp_dir("company_profile_plain_markdown_storage");
    let actor = test_actor("discord", "alice", Some("watchlist"));
    let storage = CompanyProfileStorage::new(&dir).for_actor(&actor);
    write_plain_markdown_profile(
        &dir,
        &actor,
        "AAPL",
        "# Apple Inc.\n\n## Thesis\nlegacy plain markdown thesis\n",
        &[(
            "2026-04-13-update.md",
            "# Earnings Update\n\nplain event body\n",
        )],
    );

    let profiles = storage.list_profiles();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].profile_id, "AAPL");
    assert_eq!(profiles[0].company_name, "Apple Inc.");
    assert_eq!(profiles[0].stock_code, "AAPL");
    assert_eq!(profiles[0].event_count, 1);

    let document = storage
        .get_profile("AAPL")
        .expect("load plain markdown profile")
        .expect("plain markdown profile exists");
    assert_eq!(document.metadata.company_name, "Apple Inc.");
    assert_eq!(document.metadata.stock_code, "AAPL");
    assert!(document.markdown.starts_with("# Apple Inc."));
    assert_eq!(document.events.len(), 1);
    assert_eq!(document.events[0].title, "Earnings Update");
    assert_eq!(document.events[0].metadata.event_type, "update");
    assert!(document.events[0].markdown.starts_with("# Earnings Update"));
}

#[test]
fn preview_import_bundle_accepts_plain_markdown_profile_without_frontmatter() {
    let dir = make_temp_dir("company_profile_preview_plain_bundle");
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    let manifest = CompanyProfileTransferManifest {
        version: COMPANY_PROFILE_BUNDLE_VERSION.to_string(),
        exported_at: "2026-04-19T12:00:00Z".to_string(),
        profile_count: 1,
        event_count: 1,
        profiles: vec![CompanyProfileTransferManifestProfile {
            profile_id: "AAPL".to_string(),
            company_name: "Apple Inc.".to_string(),
            stock_code: "AAPL".to_string(),
            event_count: 1,
            updated_at: "2026-04-19T12:00:00Z".to_string(),
        }],
    };
    let bundle = build_invalid_bundle(&[
        (
            "manifest.json",
            &serde_json::to_string(&manifest).expect("manifest json"),
        ),
        (
            "company_profiles/AAPL/profile.md",
            "# Apple Inc.\n\n## Thesis\nplain imported thesis\n",
        ),
        (
            "company_profiles/AAPL/events/2026-04-13-update.md",
            "# Earnings Update\n\nplain imported event\n",
        ),
    ]);

    let preview = storage
        .preview_import_bundle(&bundle)
        .expect("preview plain markdown bundle");
    assert_eq!(preview.conflict_count, 0);
    assert_eq!(preview.profiles.len(), 1);
    assert_eq!(preview.profiles[0].company_name, "Apple Inc.");
    assert_eq!(preview.profiles[0].stock_code, "AAPL");
    assert_eq!(preview.profiles[0].event_count, 1);
}

#[test]
fn apply_import_bundle_can_replace_plain_markdown_target_profile() {
    let dir = make_temp_dir("company_profile_apply_plain_target");
    let source = scoped_storage(&dir, "discord", "source", Some("watchlist"));
    let target_actor = test_actor("discord", "target", Some("watchlist"));
    let target = CompanyProfileStorage::new(&dir).for_actor(&target_actor);

    let _ = create_profile_with_thesis(&source, "Apple Inc.", "AAPL", "结构化导入 thesis");
    let bundle = source.export_bundle().expect("export bundle");

    write_plain_markdown_profile(
        &dir,
        &target_actor,
        "AAPL",
        "# Apple Inc.\n\n## Thesis\nlegacy plain markdown thesis\n",
        &[],
    );

    let preview = target.preview_import_bundle(&bundle).expect("preview");
    assert_eq!(preview.conflict_count, 1);
    assert!(
        preview.conflicts[0]
            .reasons
            .iter()
            .any(|reason| reason == "目录名相同")
    );

    let result = target
        .apply_import_bundle(
            &bundle,
            CompanyProfileImportApplyInput {
                mode: Some(CompanyProfileImportMode::ReplaceAll),
                decisions: BTreeMap::new(),
            },
        )
        .expect("apply replace all");
    assert_eq!(result.replaced_count, 1);

    let replaced = target
        .get_profile("AAPL")
        .expect("load replaced profile")
        .expect("replaced profile exists");
    assert!(replaced.markdown.contains("结构化导入 thesis"));
    assert_eq!(replaced.metadata.company_name, "Apple Inc.");
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

#[test]
fn sanitize_id_rejects_dot_components_but_keeps_safe_ticker_chars() {
    assert_eq!(sanitize_id(".."), "");
    assert_eq!(sanitize_id("../secret"), "secret");
    assert_eq!(sanitize_id("BRK.B"), "BRK.B");
}

#[test]
fn get_profile_rejects_parent_dir_component() {
    let dir = make_temp_dir("company_profile_invalid_component");
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    assert!(
        storage
            .get_profile("..")
            .expect("load invalid component")
            .is_none()
    );
}

#[test]
fn export_bundle_contains_manifest_and_markdown_files() {
    let dir = make_temp_dir("company_profile_export_bundle");
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    let document =
        create_profile_with_thesis(&storage, "NVIDIA", "NVDA", "AI 基础设施继续驱动长期需求。");
    storage
        .append_event(
            &document.profile_id,
            AppendEventInput {
                title: "Q1 更新".to_string(),
                event_type: "earnings".to_string(),
                occurred_at: "2026-04-19T10:00:00Z".to_string(),
                mainline_impact: "positive".to_string(),
                changed_sections: vec!["投资主线".to_string()],
                refs: vec!["earnings-call".to_string()],
                what_happened: "数据中心需求继续走强。".to_string(),
                why_it_matters: "决定估值溢价是否继续扩张。".to_string(),
                mainline_effect: "长期主线强化。".to_string(),
                evidence: "管理层确认订单能见度。".to_string(),
                research_log: "checked earnings call".to_string(),
                follow_up: "继续跟踪 capex。".to_string(),
            },
        )
        .expect("append event")
        .expect("event exists");

    let bytes = storage.export_bundle().expect("export bundle");
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).expect("open archive");
    let mut entry_names = (0..archive.len())
        .map(|index| {
            archive
                .by_index(index)
                .expect("archive entry")
                .name()
                .to_string()
        })
        .collect::<Vec<_>>();
    entry_names.sort();

    assert!(entry_names.contains(&"manifest.json".to_string()));
    assert!(entry_names.contains(&format!(
        "company_profiles/{}/profile.md",
        document.profile_id
    )));
    assert!(entry_names.iter().any(|name| {
        name.starts_with(&format!("company_profiles/{}/events/", document.profile_id))
            && name.ends_with(".md")
    }));
}

#[test]
fn preview_import_bundle_detects_conflicts_by_stock_name_alias_and_profile_id() {
    let dir = make_temp_dir("company_profile_preview_bundle");
    let target = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    let source = scoped_storage(&dir, "discord", "bob", Some("watchlist"));

    let (existing, _) = target
        .create_profile(CreateProfileInput {
            company_name: "Alphabet".to_string(),
            stock_code: Some("GOOGL".to_string()),
            sector: None,
            aliases: vec!["Google".to_string(), "GOOG".to_string()],
            industry_template: IndustryTemplate::General,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("create target profile");

    let _ = source
        .create_profile(CreateProfileInput {
            company_name: "Google".to_string(),
            stock_code: Some("GOOGL".to_string()),
            sector: None,
            aliases: vec!["Alphabet Inc.".to_string()],
            industry_template: IndustryTemplate::General,
            tracking: None,
            initial_sections: BTreeMap::new(),
        })
        .expect("create source profile");

    let mut bundle = source.export_bundle().expect("export bundle");
    let preview = target.preview_import_bundle(&bundle).expect("preview");
    assert_eq!(preview.conflict_count, 1);
    let reasons = &preview.conflicts[0].reasons;
    assert!(reasons.iter().any(|reason| reason == "股票代码相同"));
    assert!(reasons.iter().any(|reason| reason == "别名命中"));

    let manifest = CompanyProfileTransferManifest {
        version: COMPANY_PROFILE_BUNDLE_VERSION.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        profile_count: 1,
        event_count: 0,
        profiles: vec![CompanyProfileTransferManifestProfile {
            profile_id: existing.profile_id.clone(),
            company_name: "Different Name".to_string(),
            stock_code: "".to_string(),
            event_count: 0,
            updated_at: Utc::now().to_rfc3339(),
        }],
    };
    bundle = build_invalid_bundle(&[
        (
            "manifest.json",
            &serde_json::to_string(&manifest).expect("manifest json"),
        ),
        (
            &format!("company_profiles/{}/profile.md", existing.profile_id),
            "---\ncompany_name: Different Name\nstock_code: \"\"\naliases: []\nsector: \"\"\nindustry_template: general\nstatus: active\ntracking:\n  enabled: false\n  cadence: weekly\n  focus_metrics: []\ncreated_at: 2026-04-19T00:00:00Z\nupdated_at: 2026-04-19T00:00:00Z\n---\n\n## Thesis\n不同公司\n",
        ),
    ]);
    let err = target
        .preview_import_bundle(&bundle)
        .expect_err("profile id conflict");
    assert!(err.contains("manifest 与内容不一致"));
}

#[test]
fn apply_import_bundle_supports_keep_replace_and_interactive_modes() {
    let dir = make_temp_dir("company_profile_apply_bundle");
    let source = scoped_storage(&dir, "discord", "source", Some("watchlist"));
    let target_keep = scoped_storage(&dir, "discord", "target-keep", Some("watchlist"));
    let target_replace = scoped_storage(&dir, "discord", "target-replace", Some("watchlist"));
    let target_interactive =
        scoped_storage(&dir, "discord", "target-interactive", Some("watchlist"));

    let _ = create_profile_with_thesis(&source, "Apple Inc.", "AAPL", "导入版本 thesis");
    let _ = create_profile_with_thesis(&source, "Snowflake", "SNOW", "新的 SaaS thesis");
    let bundle = source.export_bundle().expect("export bundle");

    let _ = create_profile_with_thesis(&target_keep, "Apple", "AAPL", "保留版本 thesis");
    let keep_result = target_keep
        .apply_import_bundle(
            &bundle,
            CompanyProfileImportApplyInput {
                mode: Some(CompanyProfileImportMode::KeepExisting),
                decisions: BTreeMap::new(),
            },
        )
        .expect("apply keep");
    assert_eq!(keep_result.imported_count, 1);
    assert_eq!(keep_result.skipped_count, 1);
    let kept = target_keep
        .get_profile("AAPL")
        .expect("load kept")
        .expect("kept profile");
    assert!(kept.markdown.contains("保留版本 thesis"));
    assert!(
        target_keep
            .get_profile("SNOW")
            .expect("load imported")
            .is_some()
    );

    let _ = create_profile_with_thesis(&target_replace, "Apple", "AAPL", "旧 thesis");
    let replace_result = target_replace
        .apply_import_bundle(
            &bundle,
            CompanyProfileImportApplyInput {
                mode: Some(CompanyProfileImportMode::ReplaceAll),
                decisions: BTreeMap::new(),
            },
        )
        .expect("apply replace all");
    assert_eq!(replace_result.imported_count, 1);
    assert_eq!(replace_result.replaced_count, 1);
    let replaced = target_replace
        .get_profile("AAPL")
        .expect("load replaced")
        .expect("replaced profile");
    assert!(replaced.markdown.contains("导入版本 thesis"));

    let _ = create_profile_with_thesis(&target_interactive, "Apple", "AAPL", "交互旧 thesis");
    let err = target_interactive
        .apply_import_bundle(
            &bundle,
            CompanyProfileImportApplyInput {
                mode: Some(CompanyProfileImportMode::Interactive),
                decisions: BTreeMap::new(),
            },
        )
        .expect_err("missing interactive decision");
    assert!(err.contains("缺少冲突决策"));

    let mut decisions = BTreeMap::new();
    decisions.insert("AAPL".to_string(), CompanyProfileConflictDecision::Replace);
    let interactive_result = target_interactive
        .apply_import_bundle(
            &bundle,
            CompanyProfileImportApplyInput {
                mode: Some(CompanyProfileImportMode::Interactive),
                decisions,
            },
        )
        .expect("apply interactive");
    assert_eq!(interactive_result.replaced_count, 1);
    let interactive = target_interactive
        .get_profile("AAPL")
        .expect("load interactive")
        .expect("interactive profile");
    assert!(interactive.markdown.contains("导入版本 thesis"));
}

#[test]
fn preview_import_bundle_rejects_invalid_bundle_paths() {
    let dir = make_temp_dir("company_profile_invalid_bundle");
    let storage = scoped_storage(&dir, "discord", "alice", Some("watchlist"));
    let manifest = CompanyProfileTransferManifest {
        version: COMPANY_PROFILE_BUNDLE_VERSION.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        profile_count: 1,
        event_count: 0,
        profiles: vec![CompanyProfileTransferManifestProfile {
            profile_id: "AAPL".to_string(),
            company_name: "Apple".to_string(),
            stock_code: "AAPL".to_string(),
            event_count: 0,
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        }],
    };
    let bundle = build_invalid_bundle(&[
        (
            "manifest.json",
            &serde_json::to_string(&manifest).expect("manifest json"),
        ),
        (
            "../company_profiles/AAPL/profile.md",
            "---\ncompany_name: Apple\nstock_code: AAPL\naliases: []\nsector: \"\"\nindustry_template: general\nstatus: active\ntracking:\n  enabled: false\n  cadence: weekly\n  focus_metrics: []\ncreated_at: 2026-04-19T00:00:00Z\nupdated_at: 2026-04-19T00:00:00Z\n---\n\n## Thesis\nbad\n",
        ),
    ]);

    let err = storage
        .preview_import_bundle(&bundle)
        .expect_err("invalid bundle should fail");
    assert!(err.contains("画像包路径非法") || err.contains("不支持的路径"));
}

#[test]
fn describe_import_conflict_returns_section_and_event_diffs() {
    let dir = make_temp_dir("company_profile_conflict_detail");
    let source = scoped_storage(&dir, "discord", "source", Some("watchlist"));
    let target = scoped_storage(&dir, "discord", "target", Some("watchlist"));

    let source_doc = create_profile_with_thesis(&source, "Apple Inc.", "AAPL", "导入版本 thesis");
    let mut source_updates = BTreeMap::new();
    source_updates.insert("风险台账".to_string(), "导入版风险台账".to_string());
    source
        .rewrite_sections(&source_doc.profile_id, &source_updates)
        .expect("rewrite source")
        .expect("source exists");
    source
        .append_event(
            &source_doc.profile_id,
            AppendEventInput {
                title: "导入事件".to_string(),
                event_type: "earnings".to_string(),
                occurred_at: "2026-04-19T10:00:00Z".to_string(),
                mainline_impact: "positive".to_string(),
                changed_sections: vec!["投资主线".to_string()],
                refs: vec![],
                what_happened: "导入事件正文".to_string(),
                why_it_matters: "导入事件影响".to_string(),
                mainline_effect: "导入事件主线".to_string(),
                evidence: String::new(),
                research_log: String::new(),
                follow_up: String::new(),
            },
        )
        .expect("append source event")
        .expect("source event");

    let target_doc = create_profile_with_thesis(&target, "Apple", "AAPL", "本地旧 thesis");
    let mut target_updates = BTreeMap::new();
    target_updates.insert("风险台账".to_string(), "本地旧风险台账".to_string());
    target
        .rewrite_sections(&target_doc.profile_id, &target_updates)
        .expect("rewrite target")
        .expect("target exists");

    let bundle = source.export_bundle().expect("export bundle");
    let detail = target
        .describe_import_conflict(&bundle, "AAPL", None)
        .expect("describe conflict");
    assert!(
        detail
            .available_section_titles
            .iter()
            .any(|title| title == "投资主线")
    );
    assert!(
        detail
            .section_diffs
            .iter()
            .any(|section| section.section_title == "投资主线")
    );
    let thesis = detail
        .section_diffs
        .iter()
        .find(|section| section.section_title == "投资主线")
        .expect("mainline diff");
    assert!(
        thesis
            .line_diff
            .iter()
            .any(|line| line.kind == CompanyProfileImportDiffLineKind::Added)
    );
    assert_eq!(detail.event_diff.imported_only_event_ids.len(), 1);

    let focused = target
        .describe_import_conflict(&bundle, "AAPL", Some("风险台账"))
        .expect("focused section diff");
    assert_eq!(focused.section_diffs.len(), 1);
    assert_eq!(focused.section_diffs[0].section_title, "风险台账");
}

#[test]
fn apply_import_resolution_merges_selected_sections_and_imports_missing_events() {
    let dir = make_temp_dir("company_profile_merge_resolution");
    let source = scoped_storage(&dir, "discord", "source", Some("watchlist"));
    let target = scoped_storage(&dir, "discord", "target", Some("watchlist"));

    let source_doc = create_profile_with_thesis(&source, "Apple Inc.", "AAPL", "导入版本 thesis");
    let mut source_updates = BTreeMap::new();
    source_updates.insert("风险台账".to_string(), "导入版风险台账".to_string());
    source
        .rewrite_sections(&source_doc.profile_id, &source_updates)
        .expect("rewrite source")
        .expect("source exists");
    let source_event = source
        .append_event(
            &source_doc.profile_id,
            AppendEventInput {
                title: "导入事件".to_string(),
                event_type: "earnings".to_string(),
                occurred_at: "2026-04-19T10:00:00Z".to_string(),
                mainline_impact: "positive".to_string(),
                changed_sections: vec!["投资主线".to_string()],
                refs: vec![],
                what_happened: "导入事件正文".to_string(),
                why_it_matters: "导入事件影响".to_string(),
                mainline_effect: "导入事件主线".to_string(),
                evidence: String::new(),
                research_log: String::new(),
                follow_up: String::new(),
            },
        )
        .expect("append source event")
        .expect("source event");

    let target_doc = create_profile_with_thesis(&target, "Apple", "AAPL", "本地旧 thesis");
    let mut target_updates = BTreeMap::new();
    target_updates.insert("风险台账".to_string(), "本地旧风险台账".to_string());
    target
        .rewrite_sections(&target_doc.profile_id, &target_updates)
        .expect("rewrite target")
        .expect("target exists");

    let bundle = source.export_bundle().expect("export bundle");
    let result = target
        .apply_import_resolution(
            &bundle,
            CompanyProfileImportResolutionInput {
                imported_profile_id: "AAPL".to_string(),
                strategy: CompanyProfileImportResolutionStrategy::MergeSections,
                section_titles: vec!["投资主线".to_string()],
                import_missing_events: true,
            },
        )
        .expect("merge resolution");

    assert!(result.merged_existing_profile);
    assert_eq!(result.changed_sections, vec!["投资主线".to_string()]);
    assert_eq!(result.imported_event_ids, vec![source_event.id.clone()]);

    let merged = target
        .get_profile("AAPL")
        .expect("load merged")
        .expect("merged exists");
    assert!(merged.markdown.contains("导入版本 thesis"));
    assert!(merged.markdown.contains("本地旧风险台账"));
    assert_eq!(merged.events.len(), 1);
    assert_eq!(merged.events[0].id, source_event.id);
}
