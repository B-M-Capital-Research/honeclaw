use async_trait::async_trait;
use glob::Pattern;
use hone_core::{ActorIdentity, HoneError, HoneResult, truncate_chars_append};
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::base::{Tool, ToolParameter};

const DEFAULT_LIST_MAX_DEPTH: usize = 3;
const DEFAULT_LIST_MAX_RESULTS: usize = 100;
const DEFAULT_SEARCH_MAX_RESULTS: usize = 20;
const DEFAULT_READ_START_LINE: usize = 1;
const DEFAULT_READ_MAX_LINES: usize = 200;
const MAX_READ_CHARS: usize = 12_000;
const MAX_SEARCH_FILE_BYTES: u64 = 512 * 1024;
const MAX_SEARCH_EXCERPT_CHARS: usize = 240;

#[derive(Clone)]
struct LocalSandboxAccess {
    sandbox_base_dir: PathBuf,
    actor: ActorIdentity,
}

impl LocalSandboxAccess {
    fn new(sandbox_base_dir: PathBuf, actor: ActorIdentity) -> Self {
        Self {
            sandbox_base_dir,
            actor,
        }
    }

    fn sandbox_root(&self) -> PathBuf {
        self.sandbox_base_dir
            .join(self.actor.channel_fs_component())
            .join(self.actor.scoped_user_fs_key())
    }

    fn normalize_relative_path(&self, raw: &str) -> HoneResult<PathBuf> {
        let trimmed = raw.trim();
        let input = if trimmed.is_empty() { "." } else { trimmed };
        let path = Path::new(input);
        if path.is_absolute() {
            return Err(HoneError::Tool(
                "只允许访问当前 actor sandbox 内的相对路径".to_string(),
            ));
        }

        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => normalized.push(part),
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(HoneError::Tool(
                        "路径不能包含绝对路径、.. 或跨 sandbox 的前缀".to_string(),
                    ));
                }
            }
        }

        if normalized.as_os_str().is_empty() {
            Ok(PathBuf::from("."))
        } else {
            Ok(normalized)
        }
    }

    fn pattern(&self, raw: Option<&str>) -> HoneResult<Option<Pattern>> {
        let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(None);
        };
        Pattern::new(raw)
            .map(Some)
            .map_err(|err| HoneError::Tool(format!("glob 模式无效: {err}")))
    }

    fn ensure_path_within_root(&self, path: &Path) -> HoneResult<(PathBuf, PathBuf)> {
        let root = self.sandbox_root();
        let root_canonical = if root.exists() {
            fs::canonicalize(&root)?
        } else {
            return Err(HoneError::Tool("当前 actor sandbox 不存在".to_string()));
        };
        let candidate = fs::canonicalize(path)?;
        if !candidate.starts_with(&root_canonical) {
            return Err(HoneError::Tool(
                "目标路径超出当前 actor sandbox 范围".to_string(),
            ));
        }
        Ok((root_canonical, candidate))
    }

    fn resolve_directory(&self, raw: &str) -> HoneResult<(PathBuf, PathBuf, PathBuf)> {
        let relative = self.normalize_relative_path(raw)?;
        let joined = self.sandbox_root().join(&relative);
        if !joined.exists() {
            return Err(HoneError::Tool(format!(
                "目录不存在: {}",
                relative.display()
            )));
        }
        if fs::symlink_metadata(&joined)?.file_type().is_symlink() {
            return Err(HoneError::Tool("不允许通过符号链接访问目录".to_string()));
        }
        let (root, resolved) = self.ensure_path_within_root(&joined)?;
        if !resolved.is_dir() {
            return Err(HoneError::Tool(format!(
                "目标不是目录: {}",
                relative.display()
            )));
        }
        Ok((relative, root, resolved))
    }

    fn resolve_file(&self, raw: &str) -> HoneResult<(PathBuf, PathBuf)> {
        let relative = self.normalize_relative_path(raw)?;
        if relative == PathBuf::from(".") {
            return Err(HoneError::Tool("请提供具体文件路径".to_string()));
        }
        let joined = self.sandbox_root().join(&relative);
        if !joined.exists() {
            return Err(HoneError::Tool(format!(
                "文件不存在: {}",
                relative.display()
            )));
        }
        if fs::symlink_metadata(&joined)?.file_type().is_symlink() {
            return Err(HoneError::Tool("不允许通过符号链接读取文件".to_string()));
        }
        let (_, resolved) = self.ensure_path_within_root(&joined)?;
        if !resolved.is_file() {
            return Err(HoneError::Tool(format!(
                "目标不是文件: {}",
                relative.display()
            )));
        }
        Ok((relative, resolved))
    }

    fn relative_to_root(root: &Path, path: &Path) -> HoneResult<String> {
        let rel = path
            .strip_prefix(root)
            .map_err(|_| HoneError::Tool("无法生成相对路径".to_string()))?;
        Ok(if rel.as_os_str().is_empty() {
            ".".to_string()
        } else {
            rel.to_string_lossy().to_string()
        })
    }
}

pub struct LocalListFilesTool {
    access: LocalSandboxAccess,
}

impl LocalListFilesTool {
    pub fn new(sandbox_base_dir: PathBuf, actor: ActorIdentity) -> Self {
        Self {
            access: LocalSandboxAccess::new(sandbox_base_dir, actor),
        }
    }
}

#[async_trait]
impl Tool for LocalListFilesTool {
    fn name(&self) -> &str {
        "local_list_files"
    }

    fn description(&self) -> &str {
        "列出当前 actor sandbox 内的文件和目录。只读，仅支持相对路径。适合检查 company_profiles、uploads、runtime 等本地持久化信息。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "要列出的相对目录，默认 .".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "glob".to_string(),
                param_type: "string".to_string(),
                description: "可选。按相对路径过滤，例如 company_profiles/**/*.md".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "max_depth".to_string(),
                param_type: "number".to_string(),
                description: "递归深度，默认 3。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "max_results".to_string(),
                param_type: "number".to_string(),
                description: "最多返回多少条，默认 100。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> HoneResult<Value> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let max_depth = args
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_LIST_MAX_DEPTH as u64) as usize;
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_LIST_MAX_RESULTS as u64) as usize;
        let pattern = self
            .access
            .pattern(args.get("glob").and_then(|v| v.as_str()))?;
        let (display_path, root, dir) = self.access.resolve_directory(path)?;

        let mut entries = Vec::new();
        for entry in WalkDir::new(&dir)
            .follow_links(false)
            .min_depth(1)
            .max_depth(max_depth)
        {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => continue,
            };
            if entry.file_type().is_symlink() {
                continue;
            }
            let rel = LocalSandboxAccess::relative_to_root(&root, entry.path())?;
            if let Some(pattern) = &pattern {
                if !pattern.matches_path(Path::new(&rel)) {
                    continue;
                }
            }
            let kind = if entry.file_type().is_dir() {
                "dir"
            } else if entry.file_type().is_file() {
                "file"
            } else {
                continue;
            };
            let size_bytes = if entry.file_type().is_file() {
                entry.metadata().ok().map(|meta| meta.len())
            } else {
                None
            };
            entries.push(json!({
                "path": rel,
                "kind": kind,
                "size_bytes": size_bytes,
            }));
        }

        entries.sort_by(|a, b| {
            a["path"]
                .as_str()
                .unwrap_or_default()
                .cmp(b["path"].as_str().unwrap_or_default())
        });
        let truncated = entries.len() > max_results;
        if truncated {
            entries.truncate(max_results);
        }

        Ok(json!({
            "path": display_path.to_string_lossy(),
            "entries": entries,
            "count": entries.len(),
            "truncated": truncated,
        }))
    }
}

pub struct LocalSearchFilesTool {
    access: LocalSandboxAccess,
}

impl LocalSearchFilesTool {
    pub fn new(sandbox_base_dir: PathBuf, actor: ActorIdentity) -> Self {
        Self {
            access: LocalSandboxAccess::new(sandbox_base_dir, actor),
        }
    }
}

#[async_trait]
impl Tool for LocalSearchFilesTool {
    fn name(&self) -> &str {
        "local_search_files"
    }

    fn description(&self) -> &str {
        "在当前 actor sandbox 的文本文件里搜索关键词。只读，仅支持相对路径，返回相对路径、行号和摘要。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "要搜索的关键词。".to_string(),
                required: true,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "要搜索的相对目录或文件，默认 .".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "glob".to_string(),
                param_type: "string".to_string(),
                description: "可选。按相对路径过滤，例如 company_profiles/**/*.md".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "max_results".to_string(),
                param_type: "number".to_string(),
                description: "最多返回多少条匹配，默认 20。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> HoneResult<Value> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if query.is_empty() {
            return Err(HoneError::Tool("query 不能为空".to_string()));
        }

        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_SEARCH_MAX_RESULTS as u64) as usize;
        let pattern = self
            .access
            .pattern(args.get("glob").and_then(|v| v.as_str()))?;
        let lower_query = query.to_lowercase();

        let mut skipped_binary_files = 0usize;
        let mut skipped_non_utf8_files = 0usize;
        let mut skipped_unreadable_files = 0usize;

        let matches = if let Ok((display_path, root, dir)) = self.access.resolve_directory(path) {
            let mut matches = Vec::new();
            'walk: for entry in WalkDir::new(&dir).follow_links(false) {
                let entry = match entry {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                if !entry.file_type().is_file() || entry.file_type().is_symlink() {
                    continue;
                }
                let rel = LocalSandboxAccess::relative_to_root(&root, entry.path())?;
                if let Some(pattern) = &pattern {
                    if !pattern.matches_path(Path::new(&rel)) {
                        continue;
                    }
                }
                if entry.metadata().map(|meta| meta.len()).unwrap_or(0) > MAX_SEARCH_FILE_BYTES {
                    continue;
                }
                let file = match fs::File::open(entry.path()) {
                    Ok(file) => file,
                    Err(_) => {
                        skipped_unreadable_files += 1;
                        continue;
                    }
                };
                let mut reader = BufReader::new(file);
                let mut line_no = 0usize;
                loop {
                    let mut bytes = Vec::new();
                    match reader.read_until(b'\n', &mut bytes) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(_) => {
                            skipped_unreadable_files += 1;
                            continue 'walk;
                        }
                    }
                    line_no += 1;
                    if bytes.contains(&0) {
                        skipped_binary_files += 1;
                        continue 'walk;
                    }
                    let line = match String::from_utf8(bytes) {
                        Ok(line) => line,
                        Err(_) => {
                            skipped_non_utf8_files += 1;
                            continue 'walk;
                        }
                    };
                    if !line.to_lowercase().contains(&lower_query) {
                        continue;
                    }
                    let excerpt = truncate_chars(line.trim(), MAX_SEARCH_EXCERPT_CHARS);
                    matches.push(json!({
                        "path": rel,
                        "line": line_no,
                        "excerpt": excerpt,
                    }));
                    if matches.len() >= max_results {
                        break 'walk;
                    }
                }
            }
            (display_path.to_string_lossy().to_string(), matches)
        } else {
            let (display_path, file) = self.access.resolve_file(path)?;
            let rel = display_path.to_string_lossy().to_string();
            if let Some(pattern) = &pattern {
                if !pattern.matches_path(Path::new(&rel)) {
                    return Ok(json!({
                        "query": query,
                        "path": rel,
                        "matches": [],
                        "count": 0,
                        "truncated": false,
                    }));
                }
            }
            if fs::metadata(&file)?.len() > MAX_SEARCH_FILE_BYTES {
                return Ok(json!({
                    "query": query,
                    "path": rel,
                    "matches": [],
                    "count": 0,
                    "truncated": false,
                }));
            }
            let reader = BufReader::new(fs::File::open(file)?);
            let mut matches = Vec::new();
            for (index, line) in reader.lines().enumerate() {
                let line = line?;
                if line.contains('\0') {
                    return Err(HoneError::Tool("只支持搜索文本文件".to_string()));
                }
                if !line.to_lowercase().contains(&lower_query) {
                    continue;
                }
                matches.push(json!({
                    "path": rel,
                    "line": index + 1,
                    "excerpt": truncate_chars(line.trim(), MAX_SEARCH_EXCERPT_CHARS),
                }));
                if matches.len() >= max_results {
                    break;
                }
            }
            (rel, matches)
        };

        let (display_path, matches) = matches;
        Ok(json!({
            "query": query,
            "path": display_path,
            "matches": matches,
            "count": matches.len(),
            "truncated": matches.len() >= max_results,
            "skipped_binary_files": skipped_binary_files,
            "skipped_non_utf8_files": skipped_non_utf8_files,
            "skipped_unreadable_files": skipped_unreadable_files,
        }))
    }
}

pub struct LocalReadFileTool {
    access: LocalSandboxAccess,
}

impl LocalReadFileTool {
    pub fn new(sandbox_base_dir: PathBuf, actor: ActorIdentity) -> Self {
        Self {
            access: LocalSandboxAccess::new(sandbox_base_dir, actor),
        }
    }
}

#[async_trait]
impl Tool for LocalReadFileTool {
    fn name(&self) -> &str {
        "local_read_file"
    }

    fn description(&self) -> &str {
        "读取当前 actor sandbox 内的文本文件。只读，仅支持相对路径，可按行范围截取。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "要读取的相对文件路径。".to_string(),
                required: true,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "start_line".to_string(),
                param_type: "number".to_string(),
                description: "起始行号，默认 1。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "end_line".to_string(),
                param_type: "number".to_string(),
                description: "结束行号。默认最多读取 200 行。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> HoneResult<Value> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let start_line = args
            .get("start_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_READ_START_LINE as u64) as usize;
        if start_line == 0 {
            return Err(HoneError::Tool("start_line 必须从 1 开始".to_string()));
        }
        let requested_end = args
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let default_end = start_line + DEFAULT_READ_MAX_LINES - 1;
        let end_line = requested_end.unwrap_or(default_end).min(default_end);
        if end_line < start_line {
            return Err(HoneError::Tool("end_line 不能小于 start_line".to_string()));
        }

        let (relative, file) = self.access.resolve_file(path)?;
        let bytes = fs::read(&file)?;
        if bytes.contains(&0) {
            return Err(HoneError::Tool("只支持读取文本文件".to_string()));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| HoneError::Tool("只支持 UTF-8 文本文件".to_string()))?;
        let all_lines: Vec<&str> = text.lines().collect();
        let start_idx = start_line.saturating_sub(1).min(all_lines.len());
        let end_idx = end_line.min(all_lines.len());
        let selected = if start_idx < end_idx {
            &all_lines[start_idx..end_idx]
        } else {
            &[]
        };

        let mut content = selected.join("\n");
        let mut truncated = requested_end.is_none() && all_lines.len() > end_line;
        if content.chars().count() > MAX_READ_CHARS {
            content = truncate_chars(&content, MAX_READ_CHARS);
            truncated = true;
        }

        Ok(json!({
            "path": relative.to_string_lossy(),
            "start_line": start_line,
            "end_line": if selected.is_empty() { start_line.saturating_sub(1) } else { start_line + selected.len() - 1 },
            "total_lines": all_lines.len(),
            "content": content,
            "truncated": truncated,
        }))
    }
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    truncate_chars_append(input, max_chars.saturating_sub(1), "…")
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn actor() -> ActorIdentity {
        ActorIdentity::new("telegram", "8039067465", None::<String>).expect("actor")
    }

    fn actor_root(base: &Path) -> PathBuf {
        let actor = actor();
        base.join(actor.channel_fs_component())
            .join(actor.scoped_user_fs_key())
    }

    fn setup_sandbox() -> PathBuf {
        let base = make_temp_dir("hone-local-files");
        let root = actor_root(&base);
        fs::create_dir_all(root.join("company_profiles/aaoi")).expect("profiles dir");
        fs::create_dir_all(root.join("uploads")).expect("uploads dir");
        fs::write(
            root.join("company_profiles/aaoi/profile.md"),
            "# AAOI\n\nTicker: AAOI\n\nThesis: optics vendor\n",
        )
        .expect("write profile");
        fs::write(
            root.join("uploads/note.txt"),
            "AAOI was reviewed in depth.\n",
        )
        .expect("note");
        base
    }

    #[test]
    fn list_search_and_read_work_with_relative_paths() {
        let base = setup_sandbox();
        let list_tool = LocalListFilesTool::new(base.clone(), actor());
        let search_tool = LocalSearchFilesTool::new(base.clone(), actor());
        let read_tool = LocalReadFileTool::new(base.clone(), actor());

        let list = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(
                list_tool
                    .execute(json!({"path":"company_profiles","glob":"company_profiles/**/*.md"})),
            )
            .expect("list");
        assert_eq!(
            list["entries"][0]["path"],
            "company_profiles/aaoi/profile.md"
        );

        let search = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(search_tool.execute(json!({"query":"AAOI","path":"company_profiles"})))
            .expect("search");
        assert_eq!(
            search["matches"][0]["path"],
            "company_profiles/aaoi/profile.md"
        );
        assert!(
            search["matches"][0]["excerpt"]
                .as_str()
                .expect("excerpt")
                .contains("AAOI")
        );

        let read = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(read_tool.execute(json!({"path":"company_profiles/aaoi/profile.md"})))
            .expect("read");
        assert_eq!(read["path"], "company_profiles/aaoi/profile.md");
        assert!(
            read["content"]
                .as_str()
                .expect("content")
                .contains("Ticker: AAOI")
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn parent_and_absolute_paths_are_rejected() {
        let base = setup_sandbox();
        let list_tool = LocalListFilesTool::new(base.clone(), actor());
        let read_tool = LocalReadFileTool::new(base.clone(), actor());

        let abs_err = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(list_tool.execute(json!({"path":"/tmp"})))
            .expect_err("absolute path should fail");
        assert!(abs_err.to_string().contains("相对路径"));

        let parent_err = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(read_tool.execute(json!({"path":"../secret.txt"})))
            .expect_err("parent path should fail");
        assert!(parent_err.to_string().contains(".."));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn binary_files_are_rejected() {
        let base = setup_sandbox();
        let root = actor_root(&base);
        fs::write(root.join("uploads/blob.bin"), [0, 159, 1, 2]).expect("binary");
        let read_tool = LocalReadFileTool::new(base.clone(), actor());

        let err = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(read_tool.execute(json!({"path":"uploads/blob.bin"})))
            .expect_err("binary should fail");
        assert!(err.to_string().contains("文本文件"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn directory_search_skips_non_text_files_without_aborting() {
        let base = setup_sandbox();
        let root = actor_root(&base);
        fs::write(root.join("uploads/bad-encoding.txt"), [0xff, b'\n']).expect("bad utf8");
        fs::write(root.join("uploads/blob.bin"), [0, b'A', b'A', b'O', b'I']).expect("binary");
        fs::write(
            root.join("uploads/hit.txt"),
            "AAOI hit zone is 12-14\nsecond line\n",
        )
        .expect("hit");
        let search_tool = LocalSearchFilesTool::new(base.clone(), actor());

        let search = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(search_tool.execute(json!({"query":"hit zone","path":"uploads"})))
            .expect("search should skip bad files");
        assert_eq!(search["matches"][0]["path"], "uploads/hit.txt");
        assert_eq!(search["matches"][0]["line"], 1);
        assert_eq!(search["skipped_binary_files"], 1);
        assert_eq!(search["skipped_non_utf8_files"], 1);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn read_truncates_default_window() {
        let base = setup_sandbox();
        let root = actor_root(&base);
        let long_file = (1..=260)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(root.join("uploads/long.txt"), long_file).expect("write long");
        let read_tool = LocalReadFileTool::new(base.clone(), actor());

        let read = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(read_tool.execute(json!({"path":"uploads/long.txt"})))
            .expect("read");
        assert_eq!(read["start_line"], 1);
        assert_eq!(read["end_line"], 200);
        assert_eq!(read["truncated"], true);

        let _ = fs::remove_dir_all(base);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let base = setup_sandbox();
        let root = actor_root(&base);
        let outside = make_temp_dir("hone-local-files-outside");
        let outside_file = outside.join("secret.txt");
        fs::write(&outside_file, "secret").expect("outside file");
        symlink(&outside_file, root.join("uploads/secret-link.txt")).expect("symlink");
        let read_tool = LocalReadFileTool::new(base.clone(), actor());

        let err = tokio::runtime::Runtime::new()
            .expect("rt")
            .block_on(read_tool.execute(json!({"path":"uploads/secret-link.txt"})))
            .expect_err("symlink should fail");
        assert!(err.to_string().contains("符号链接"));

        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(outside);
    }
}
