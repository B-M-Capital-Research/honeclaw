use glob::Pattern;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::skill_registry::{default_skill_registry_path, load_skill_registry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSource {
    System,
    Custom,
    Dynamic,
}

impl SkillSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Custom => "custom",
            Self::Dynamic => "dynamic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillExecutionContext {
    Inline,
    Fork,
}

impl SkillExecutionContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Fork => "fork",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub allowed_tools: Vec<String>,
    pub aliases: Vec<String>,
    pub user_invocable: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub context: SkillExecutionContext,
    pub agent: Option<String>,
    pub paths: Vec<String>,
    pub hooks: Option<Value>,
    pub arguments: Vec<String>,
    pub script: Option<String>,
    pub shell: Option<String>,
    pub source: SkillSource,
    pub enabled: bool,
    pub skill_dir: PathBuf,
    pub skill_path: PathBuf,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct SkillSummary {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub allowed_tools: Vec<String>,
    pub aliases: Vec<String>,
    pub user_invocable: bool,
    pub context: SkillExecutionContext,
    pub script: Option<String>,
    pub loaded_from: String,
    pub enabled: bool,
    pub paths: Vec<String>,
    pub detail_path: String,
}

impl From<&SkillDefinition> for SkillSummary {
    fn from(value: &SkillDefinition) -> Self {
        Self {
            id: value.id.clone(),
            display_name: value.display_name.clone(),
            description: value.description.clone(),
            when_to_use: value.when_to_use.clone(),
            allowed_tools: value.allowed_tools.clone(),
            aliases: value.aliases.clone(),
            user_invocable: value.user_invocable,
            context: value.context.clone(),
            script: value.script.clone(),
            loaded_from: value.source.as_str().to_string(),
            enabled: value.enabled,
            paths: value.paths.clone(),
            detail_path: value.skill_path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct SkillFrontmatter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    when_to_use: Option<String>,
    #[serde(rename = "allowed-tools", default)]
    allowed_tools: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(rename = "user-invocable")]
    user_invocable: Option<bool>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    effort: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    hooks: Option<Value>,
    #[serde(default)]
    arguments: Vec<String>,
    #[serde(default)]
    script: Option<String>,
    #[serde(default)]
    shell: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SkillRuntime {
    system_dir: PathBuf,
    custom_dir: PathBuf,
    cwd: PathBuf,
    registry_path: PathBuf,
}

impl SkillRuntime {
    pub fn new(system_dir: PathBuf, custom_dir: PathBuf, cwd: PathBuf) -> Self {
        let registry_path = default_skill_registry_path(&custom_dir);
        Self {
            system_dir,
            custom_dir,
            cwd,
            registry_path,
        }
    }

    pub fn with_registry_path(mut self, registry_path: PathBuf) -> Self {
        self.registry_path = registry_path;
        self
    }

    pub fn list_summaries(&self) -> Vec<SkillSummary> {
        self.load_active_skills(&[])
            .iter()
            .map(SkillSummary::from)
            .collect()
    }

    pub fn list_registered_summaries(&self) -> Vec<SkillSummary> {
        self.load_registered_skills()
            .iter()
            .map(SkillSummary::from)
            .collect()
    }

    pub fn list_all_summaries(&self) -> Vec<SkillSummary> {
        self.load_registered_skills()
            .iter()
            .map(SkillSummary::from)
            .collect()
    }

    pub fn build_skill_listing(&self, max_chars: usize) -> String {
        let listing = self
            .list_summaries()
            .into_iter()
            .map(|skill| {
                let mut line = format!("- {}: {}", skill.id, skill.description);
                if let Some(when_to_use) = skill
                    .when_to_use
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                {
                    line.push_str(" - ");
                    line.push_str(when_to_use.trim());
                }
                line
            })
            .collect::<Vec<_>>();

        truncate_listing(&listing.join("\n"), max_chars)
    }

    pub fn search(&self, query: &str, file_paths: &[String], limit: usize) -> Vec<SkillSummary> {
        let normalized_query = normalize_skill_text(query);
        let mut scored = self
            .load_active_skills(file_paths)
            .into_iter()
            .filter_map(|skill| {
                let summary = SkillSummary::from(&skill);
                let score = if normalized_query.is_empty() {
                    1
                } else {
                    score_skill(&summary, &normalized_query)
                };
                (score > 0).then_some((score, summary))
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.display_name.cmp(&right.1.display_name))
                .then_with(|| left.1.id.cmp(&right.1.id))
        });

        let take_n = if limit == 0 {
            scored.len()
        } else {
            scored.len().min(limit)
        };
        scored
            .into_iter()
            .take(take_n)
            .map(|(_, skill)| skill)
            .collect()
    }

    pub fn load_skill(
        &self,
        skill_id: &str,
        file_paths: &[String],
    ) -> Result<SkillDefinition, String> {
        let normalized = skill_id.trim();
        if normalized.is_empty() {
            return Err("skill_name 不能为空".to_string());
        }

        if let Some(skill) = self
            .load_registered_skills()
            .into_iter()
            .find(|skill| skill_matches_reference(skill, normalized))
        {
            if !skill.enabled {
                return Err(skill_disabled_error(normalized));
            }
        }

        self.load_active_skills(file_paths)
            .into_iter()
            .find(|skill| skill_matches_reference(skill, normalized))
            .ok_or_else(|| format!("技能 '{normalized}' 不存在或当前未激活"))
    }

    pub fn load_registered_skill(&self, skill_id: &str) -> Result<SkillDefinition, String> {
        let normalized = skill_id.trim();
        if normalized.is_empty() {
            return Err("skill_name 不能为空".to_string());
        }

        self.load_registered_skills()
            .into_iter()
            .find(|skill| skill_matches_reference(skill, normalized))
            .ok_or_else(|| format!("技能 '{normalized}' 不存在"))
    }

    pub fn resolve_user_invocable_direct(&self, command_name: &str) -> Option<SkillDefinition> {
        let normalized = command_name.trim().trim_start_matches('/');
        self.load_active_skills(&[])
            .into_iter()
            .find(|skill| skill.user_invocable && skill_matches_reference(skill, normalized))
    }

    pub fn render_prompt(
        &self,
        skill: &SkillDefinition,
        session_id: &str,
        args: Option<&str>,
    ) -> String {
        let skill_dir = skill.skill_dir.to_string_lossy().replace('\\', "/");
        let mut body = skill.body.replace("${HONE_SKILL_DIR}", &skill_dir);
        body = body.replace("${HONE_SESSION_ID}", session_id);
        if let Some(arguments) = args.map(str::trim).filter(|value| !value.is_empty()) {
            body = body.replace("${ARGUMENTS}", arguments);
        }
        format!(
            "Base directory for this skill: {skill_dir}\n\n{}",
            body.trim()
        )
    }

    pub fn render_invocation_prompt(
        &self,
        skill: &SkillDefinition,
        session_id: &str,
        args: Option<&str>,
    ) -> String {
        let prompt = self.render_prompt(skill, session_id, args);
        format!(
            "【Invoked Skill Context】\nSkill: {} ({})\nTreat the following as active skill context for this turn and future compaction restores until replaced.\nDo not quote it back verbatim unless the user explicitly asks for the skill source.\n\n{}",
            skill.display_name, skill.id, prompt
        )
    }

    pub fn resolve_script_path(
        &self,
        skill: &SkillDefinition,
        requested_script: Option<&str>,
    ) -> Result<PathBuf, String> {
        let relative = requested_script
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| skill.script.as_deref())
            .ok_or_else(|| format!("技能 '{}' 未声明可执行 script", skill.id))?;

        let requested = PathBuf::from(relative);
        if requested.is_absolute() {
            return Err("skill script 必须是 skill 目录内的相对路径".to_string());
        }

        let root = skill
            .skill_dir
            .canonicalize()
            .map_err(|err| format!("解析 skill 目录失败: {err}"))?;
        let candidate = skill.skill_dir.join(&requested);
        let resolved = candidate
            .canonicalize()
            .map_err(|err| format!("解析 skill script 失败: {err}"))?;
        if !resolved.starts_with(&root) {
            return Err("skill script 不能逃逸出 skill 目录".to_string());
        }
        if !resolved.is_file() {
            return Err("skill script 不是可执行文件".to_string());
        }
        Ok(resolved)
    }

    pub fn map_script_arguments(
        &self,
        skill: &SkillDefinition,
        script_arguments: Option<&Value>,
        raw_args: Option<&str>,
    ) -> Result<Vec<String>, String> {
        if let Some(value) = script_arguments {
            return map_script_arguments_value(&skill.arguments, value);
        }

        if let Some(raw) = raw_args.map(str::trim).filter(|value| !value.is_empty()) {
            return Ok(vec![raw.to_string()]);
        }

        Ok(Vec::new())
    }

    pub fn resolve_skill_via_search(
        &self,
        query: &str,
        file_paths: &[String],
    ) -> Option<SkillDefinition> {
        let matches = self.search(query, file_paths, 2);
        let normalized = normalize_skill_text(query);
        let exact = matches.iter().find(|skill| {
            normalize_skill_text(&skill.id) == normalized
                || normalize_skill_text(&skill.display_name) == normalized
                || skill
                    .aliases
                    .iter()
                    .any(|alias| normalize_skill_text(alias) == normalized)
        });
        if let Some(skill) = exact {
            return self.load_skill(&skill.id, file_paths).ok();
        }
        if matches.len() == 1 {
            return self.load_skill(&matches[0].id, file_paths).ok();
        }
        None
    }

    fn load_active_skills(&self, file_paths: &[String]) -> Vec<SkillDefinition> {
        self.load_registered_skills()
            .into_iter()
            .filter(|skill| skill.enabled)
            .filter(|skill| skill.paths.is_empty() || skill_matches_paths(&skill.paths, file_paths))
            .collect()
    }

    fn load_registered_skills(&self) -> Vec<SkillDefinition> {
        let registry = load_skill_registry(&self.registry_path);
        self.load_all_skills()
            .into_iter()
            .map(|mut skill| {
                skill.enabled = registry.is_enabled(&skill.id);
                skill
            })
            .collect()
    }

    fn load_all_skills(&self) -> Vec<SkillDefinition> {
        let mut seen = HashSet::new();
        let mut skills = Vec::new();

        for dir in self.dynamic_skill_dirs() {
            skills.extend(load_from_root(&dir, SkillSource::Dynamic, &mut seen));
        }
        skills.extend(load_from_root(
            &self.custom_dir,
            SkillSource::Custom,
            &mut seen,
        ));
        skills.extend(load_from_root(
            &self.system_dir,
            SkillSource::System,
            &mut seen,
        ));
        skills.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then_with(|| left.id.cmp(&right.id))
        });
        skills
    }

    fn dynamic_skill_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = WalkDir::new(&self.cwd)
            .into_iter()
            .filter_entry(is_dynamic_walk_entry)
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
            .filter_map(|entry| {
                let file_name = entry.file_name().to_string_lossy();
                if file_name == "skills" {
                    let parent = entry.path().parent()?;
                    if parent.file_name().and_then(|name| name.to_str()) == Some(".hone") {
                        return Some(entry.into_path());
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        dirs.sort_by(|left, right| {
            let left_depth = left.components().count();
            let right_depth = right.components().count();
            right_depth.cmp(&left_depth).then_with(|| left.cmp(right))
        });
        dirs
    }
}

fn is_dynamic_walk_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git" | "node_modules" | "target" | "dist" | "build" | ".next"
    )
}

fn load_from_root(
    root: &Path,
    source: SkillSource,
    seen: &mut HashSet<String>,
) -> Vec<SkillDefinition> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut skills = Vec::new();
    for entry in entries.filter_map(|item| item.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_path = path.join("SKILL.md");
        if !skill_path.exists() {
            continue;
        }
        let skill_id = entry.file_name().to_string_lossy().to_string();
        if seen.contains(&skill_id) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&skill_path) else {
            continue;
        };
        if let Ok(skill) =
            parse_skill_definition(&skill_id, &path, &skill_path, &content, source.clone())
        {
            seen.insert(skill_id);
            skills.push(skill);
        }
    }
    skills
}

fn parse_skill_definition(
    skill_id: &str,
    skill_dir: &Path,
    skill_path: &Path,
    content: &str,
    source: SkillSource,
) -> Result<SkillDefinition, String> {
    let (frontmatter, body) = parse_skill_md(content)?;
    let allowed_tools = if frontmatter.allowed_tools.is_empty() {
        frontmatter.tools.clone()
    } else {
        frontmatter.allowed_tools.clone()
    };
    let context = match frontmatter.context.as_deref().map(str::trim) {
        Some("fork") => SkillExecutionContext::Fork,
        _ => SkillExecutionContext::Inline,
    };
    Ok(SkillDefinition {
        id: skill_id.to_string(),
        display_name: if frontmatter.name.trim().is_empty() {
            skill_id.to_string()
        } else {
            frontmatter.name.trim().to_string()
        },
        description: frontmatter.description.trim().to_string(),
        when_to_use: frontmatter
            .when_to_use
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        allowed_tools,
        aliases: frontmatter.aliases,
        user_invocable: frontmatter.user_invocable.unwrap_or(true),
        model: frontmatter
            .model
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        effort: frontmatter
            .effort
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        context,
        agent: frontmatter
            .agent
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        paths: frontmatter.paths,
        hooks: frontmatter.hooks,
        arguments: frontmatter.arguments,
        script: frontmatter
            .script
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        shell: frontmatter
            .shell
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        source,
        enabled: true,
        skill_dir: skill_dir.to_path_buf(),
        skill_path: skill_path.to_path_buf(),
        body,
    })
}

fn skill_disabled_error(skill_id: &str) -> String {
    format!("技能 '{skill_id}' 已被管理员禁用")
}

fn skill_matches_reference(skill: &SkillDefinition, reference: &str) -> bool {
    let normalized = normalize_skill_text(reference);
    if normalized.is_empty() {
        return false;
    }

    normalize_skill_text(&skill.id) == normalized
        || normalize_skill_text(&skill.display_name) == normalized
        || skill
            .aliases
            .iter()
            .any(|alias| normalize_skill_text(alias) == normalized)
}

fn parse_skill_md(content: &str) -> Result<(SkillFrontmatter, String), String> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
        .ok_or("SKILL.md 缺少 YAML frontmatter（应以 '---' 开头）")?;

    let end_marker = rest
        .find("\n---\n")
        .or_else(|| rest.find("\n---\r\n"))
        .ok_or("SKILL.md frontmatter 未正确关闭（缺少结束 '---'）")?;

    let yaml_part = &rest[..end_marker];
    let body_start = end_marker + "\n---\n".len();
    let body = rest.get(body_start..).unwrap_or("").trim().to_string();

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_part)
        .map_err(|err| format!("解析 SKILL.md frontmatter 失败: {err}"))?;

    Ok((frontmatter, body))
}

fn truncate_listing(content: &str, max_chars: usize) -> String {
    if max_chars == 0 || content.chars().count() <= max_chars {
        return content.to_string();
    }
    content.chars().take(max_chars).collect::<String>() + "..."
}

fn skill_matches_paths(patterns: &[String], file_paths: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    if file_paths.is_empty() {
        return false;
    }

    let compiled = patterns
        .iter()
        .filter_map(|pattern| Pattern::new(pattern).ok())
        .collect::<Vec<_>>();
    if compiled.is_empty() {
        return false;
    }

    file_paths.iter().any(|path| {
        let normalized = path.replace('\\', "/");
        compiled.iter().any(|pattern| pattern.matches(&normalized))
    })
}

fn normalize_skill_text(value: &str) -> String {
    value.trim().to_lowercase()
}

fn map_script_arguments_value(
    declared_arguments: &[String],
    value: &Value,
) -> Result<Vec<String>, String> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Array(values) => values
            .iter()
            .map(json_value_to_argument)
            .collect::<Result<Vec<_>, _>>(),
        Value::Object(map) => {
            if declared_arguments.is_empty() {
                return Err(
                    "脚本参数使用对象形式时，SKILL.md 必须先声明 arguments 顺序".to_string()
                );
            }

            let mut ordered = Vec::new();
            for key in declared_arguments {
                if let Some(argument) = map.get(key) {
                    if argument.is_null() {
                        continue;
                    }
                    ordered.push(json_value_to_argument(argument)?);
                }
            }
            Ok(ordered)
        }
        _ => Ok(vec![json_value_to_argument(value)?]),
    }
}

fn json_value_to_argument(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(boolean) => Ok(boolean.to_string()),
        Value::Null => Err("脚本参数不能为 null".to_string()),
        Value::Array(_) | Value::Object(_) => {
            Err("脚本参数必须是字符串、数字、布尔值，或这些值构成的数组".to_string())
        }
    }
}

fn score_skill(skill: &SkillSummary, query: &str) -> i32 {
    let mut best = score_field(&skill.id, query, 130);
    best = best.max(score_field(&skill.display_name, query, 120));
    best = best.max(score_field(&skill.description, query, 40));
    if let Some(when_to_use) = &skill.when_to_use {
        best = best.max(score_field(when_to_use, query, 80));
    }
    for alias in &skill.aliases {
        best = best.max(score_field(alias, query, 110));
    }
    for tool in &skill.allowed_tools {
        best = best.max(score_field(tool, query, 20));
    }
    best
}

fn score_field(value: &str, query: &str, base: i32) -> i32 {
    let normalized = normalize_skill_text(value);
    if normalized.is_empty() || query.is_empty() {
        return 0;
    }
    if normalized == query {
        return base + 1000;
    }
    if normalized.starts_with(query) {
        return base + 800;
    }
    if normalized.contains(query) {
        return base + 600;
    }
    let tokens = query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if !tokens.is_empty() && tokens.iter().all(|token| normalized.contains(token)) {
        return base + 400 - tokens.len() as i32;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_registry::set_skill_enabled;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_runtime_prefers_dynamic_then_custom_then_system() {
        let root = make_temp_dir("hone_skill_runtime_precedence");
        let system = root.join("system");
        let custom = root.join("custom");
        let dynamic = root.join("nested/.hone/skills");
        fs::create_dir_all(system.join("alpha")).expect("system alpha");
        fs::create_dir_all(custom.join("alpha")).expect("custom alpha");
        fs::create_dir_all(&dynamic).expect("dynamic root");
        fs::create_dir_all(dynamic.join("alpha")).expect("dynamic alpha");

        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: System\ndescription: from system\n---\n\nsystem",
        )
        .expect("write system");
        fs::write(
            custom.join("alpha/SKILL.md"),
            "---\nname: Custom\ndescription: from custom\n---\n\ncustom",
        )
        .expect("write custom");
        fs::write(
            dynamic.join("alpha/SKILL.md"),
            "---\nname: Dynamic\ndescription: from dynamic\n---\n\ndynamic",
        )
        .expect("write dynamic");

        let runtime = SkillRuntime::new(system, custom, root.clone());
        let summaries = runtime.list_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].display_name, "Dynamic");
        assert_eq!(summaries[0].loaded_from, "dynamic");
    }

    #[test]
    fn path_gated_skill_only_activates_with_matching_file() {
        let root = make_temp_dir("hone_skill_runtime_paths");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: gated\npaths:\n  - src/**/*.rs\n---\n\nbody",
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system, custom, root);
        assert!(runtime.list_summaries().is_empty());
        let matches = runtime.search("alpha", &[String::from("src/lib.rs")], 5);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn disabled_skill_is_hidden_from_active_runtime_and_returns_explicit_error() {
        let root = make_temp_dir("hone_skill_runtime_disabled");
        let system = root.join("system");
        let custom = root.join("custom");
        let registry_path = root.join("runtime").join("skill_registry.json");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: disabled test\n---\n\nbody",
        )
        .expect("write skill");

        set_skill_enabled(&registry_path, "alpha", false).expect("disable alpha");

        let runtime = SkillRuntime::new(system, custom, root).with_registry_path(registry_path);
        let registered = runtime.list_registered_summaries();
        assert_eq!(registered.len(), 1);
        assert!(!registered[0].enabled);
        assert!(runtime.list_summaries().is_empty());
        assert!(runtime.search("alpha", &[], 5).is_empty());
        let error = runtime.load_skill("alpha", &[]).expect_err("disabled");
        assert!(error.contains("已被管理员禁用"));
        let registered_skill = runtime
            .load_registered_skill("alpha")
            .expect("registered alpha");
        assert!(!registered_skill.enabled);
    }

    #[test]
    fn parse_new_frontmatter_fields_and_legacy_tools_fallback() {
        let root = make_temp_dir("hone_skill_runtime_frontmatter");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(system.join("beta")).expect("beta dir");
        fs::create_dir_all(&custom).expect("custom dir");

        fs::write(
            system.join("alpha/SKILL.md"),
            concat!(
                "---\n",
                "name: Alpha\n",
                "description: full schema\n",
                "when_to_use: for alpha tasks\n",
                "allowed-tools:\n",
                "  - skill_tool\n",
                "  - discover_skills\n",
                "user-invocable: false\n",
                "model: openrouter/openai/gpt-5.4\n",
                "effort: high\n",
                "context: fork\n",
                "agent: worker\n",
                "paths:\n",
                "  - src/**/*.rs\n",
                "arguments:\n",
                "  - ticker\n",
                "script: scripts/run.sh\n",
                "shell: zsh\n",
                "---\n\n",
                "body"
            ),
        )
        .expect("write alpha");
        fs::write(
            system.join("beta/SKILL.md"),
            "---\nname: Beta\ndescription: legacy tools\ntools:\n  - web_search\n---\n\nbody",
        )
        .expect("write beta");

        let runtime = SkillRuntime::new(system, custom, root);
        let alpha = runtime
            .load_skill("alpha", &[String::from("src/lib.rs")])
            .expect("alpha");
        assert_eq!(alpha.display_name, "Alpha");
        assert_eq!(alpha.description, "full schema");
        assert_eq!(alpha.when_to_use.as_deref(), Some("for alpha tasks"));
        assert_eq!(
            alpha.allowed_tools,
            vec!["skill_tool".to_string(), "discover_skills".to_string()]
        );
        assert!(!alpha.user_invocable);
        assert_eq!(alpha.model.as_deref(), Some("openrouter/openai/gpt-5.4"));
        assert_eq!(alpha.effort.as_deref(), Some("high"));
        assert_eq!(alpha.context, SkillExecutionContext::Fork);
        assert_eq!(alpha.agent.as_deref(), Some("worker"));
        assert_eq!(alpha.paths, vec!["src/**/*.rs".to_string()]);
        assert_eq!(alpha.arguments, vec!["ticker".to_string()]);
        assert_eq!(alpha.script.as_deref(), Some("scripts/run.sh"));
        assert_eq!(alpha.shell.as_deref(), Some("zsh"));

        let beta = runtime.load_skill("beta", &[]).expect("beta");
        assert_eq!(beta.allowed_tools, vec!["web_search".to_string()]);
    }

    #[test]
    fn load_skill_and_direct_invocation_accept_aliases() {
        let root = make_temp_dir("hone_skill_runtime_aliases");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("one_sentence_memory")).expect("skill dir");
        fs::create_dir_all(&custom).expect("custom dir");

        fs::write(
            system.join("one_sentence_memory/SKILL.md"),
            concat!(
                "---\n",
                "name: One-sentence Memory\n",
                "description: alias resolution\n",
                "aliases:\n",
                "  - OWJY\n",
                "  - memory write\n",
                "---\n\n",
                "body"
            ),
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system, custom, root);
        let alias = runtime.load_skill("OWJY", &[]).expect("alias load");
        assert_eq!(alias.id, "one_sentence_memory");

        let display = runtime
            .load_skill("One-sentence Memory", &[])
            .expect("display load");
        assert_eq!(display.id, "one_sentence_memory");

        let direct = runtime
            .resolve_user_invocable_direct("/OWJY")
            .expect("direct alias");
        assert_eq!(direct.id, "one_sentence_memory");
    }

    #[test]
    fn render_prompt_replaces_runtime_placeholders() {
        let root = make_temp_dir("hone_skill_runtime_render");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: prompt render\n---\n\nDir=${HONE_SKILL_DIR}\nSession=${HONE_SESSION_ID}\nArgs=${ARGUMENTS}",
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system.clone(), custom, root);
        let skill = runtime.load_skill("alpha", &[]).expect("alpha");
        let rendered = runtime.render_prompt(&skill, "session-123", Some("AAPL"));

        assert!(rendered.contains("Base directory for this skill:"));
        assert!(rendered.contains(&system.join("alpha").to_string_lossy().replace('\\', "/")));
        assert!(rendered.contains("Session=session-123"));
        assert!(rendered.contains("Args=AAPL"));
        assert!(!rendered.contains("${HONE_SKILL_DIR}"));
        assert!(!rendered.contains("${HONE_SESSION_ID}"));
        assert!(!rendered.contains("${ARGUMENTS}"));
    }

    #[test]
    fn render_invocation_prompt_wraps_rendered_skill_context() {
        let root = make_temp_dir("hone_skill_runtime_invocation");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: invocation\n---\n\nbody",
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system, custom, root);
        let skill = runtime.load_skill("alpha", &[]).expect("alpha");
        let rendered = runtime.render_invocation_prompt(&skill, "session-456", None);

        assert!(rendered.contains("【Invoked Skill Context】"));
        assert!(rendered.contains("Skill: Alpha (alpha)"));
        assert!(rendered.contains("Base directory for this skill:"));
    }

    #[test]
    fn map_script_arguments_uses_declared_order_for_objects() {
        let root = make_temp_dir("hone_skill_runtime_script_args");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: script args\narguments:\n  - ticker\n  - days\nscript: scripts/run.sh\n---\n\nbody",
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system, custom, root);
        let skill = runtime.load_skill("alpha", &[]).expect("alpha");
        let args = runtime
            .map_script_arguments(
                &skill,
                Some(&serde_json::json!({"days": 5, "ticker": "AAPL"})),
                None,
            )
            .expect("map args");
        assert_eq!(args, vec!["AAPL".to_string(), "5".to_string()]);
    }

    #[test]
    fn resolve_script_path_rejects_escape() {
        let root = make_temp_dir("hone_skill_runtime_script_path");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("alpha")).expect("alpha dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            system.join("alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: script path\nscript: ../escape.sh\n---\n\nbody",
        )
        .expect("write skill");

        let runtime = SkillRuntime::new(system, custom, root);
        let skill = runtime.load_skill("alpha", &[]).expect("alpha");
        let error = runtime
            .resolve_script_path(&skill, None)
            .expect_err("should reject escape");
        assert!(error.contains("skill script"));
    }
}
