//! SkillTool — 技能管理工具
//!
//! 在运行时提供查询、添加、更新、删除自定义技能的能力。
//! 系统内置技能（存放在 system_skills_dir）不可通过此工具修改或删除。

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};
use crate::load_skill::LoadSkillTool;

/// SkillTool — 自定义技能管理工具
pub struct SkillTool {
    system_skills_dir: PathBuf,
    custom_skills_dir: PathBuf,
}

impl SkillTool {
    pub fn new(system_skills_dir: PathBuf, custom_skills_dir: PathBuf) -> Self {
        Self {
            system_skills_dir,
            custom_skills_dir,
        }
    }

    /// 判断一个技能名称是否属于系统内置技能
    fn is_system_skill(&self, name: &str) -> bool {
        let dir = self.system_skills_dir.join(name);
        dir.exists() && dir.is_dir()
    }
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill_tool"
    }

    fn description(&self) -> &str {
        "管理用户的自定义技能。支持操作：list（列出所有技能，含 display_name/description/type）、add（添加新自定义技能）、update（更新已有自定义技能的字段）、remove（删除自定义技能）。系统内置技能（type=system）不可通过此工具修改或删除。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型".to_string(),
                required: true,
                r#enum: Some(vec![
                    "list".into(),
                    "add".into(),
                    "update".into(),
                    "remove".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "技能英文代号，只能包含大小写字母、数字和下划线，且以字母开头（add/update/remove 时必填）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "display_name".to_string(),
                param_type: "string".to_string(),
                description: "技能的中文或展示名称（add 时必填，update 时可选）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "description".to_string(),
                param_type: "string".to_string(),
                description: "技能作用的一句话描述（add 时必填，update 时可选）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "aliases".to_string(),
                param_type: "array".to_string(),
                description: "触发该技能的别名关键词列表，例如 [\"新闻\", \"今日资讯\"]（add/update 时可选）".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({
                    "type": "string"
                })),
            },
            ToolParameter {
                name: "tools".to_string(),
                param_type: "array".to_string(),
                description: "技能需要使用到的底层工具列表（如 web_search, data_fetch 等）（add/update 时可选）".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({
                    "type": "string"
                })),
            },
            ToolParameter {
                name: "prompt".to_string(),
                param_type: "string".to_string(),
                description: "技能详细提示词/执行逻辑，采用 markdown 格式（add 时必填，update 时可选）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        match action {
            "list" => {
                // 使用 list_skills_with_meta 获取富信息
                let load_tool_system = LoadSkillTool::new(vec![self.system_skills_dir.clone()]);
                let load_tool_custom = LoadSkillTool::new(vec![self.custom_skills_dir.clone()]);

                let system_metas = load_tool_system.list_skills_with_meta();
                let custom_metas = load_tool_custom.list_skills_with_meta();

                let mut all_skills: Vec<Value> = system_metas
                    .into_iter()
                    .map(|m| {
                        serde_json::json!({
                            "name": m.name,
                            "display_name": m.display_name,
                            "description": m.description,
                            "aliases": m.aliases,
                            "tools": m.tools,
                            "type": "system",
                            "can_delete": false,
                        })
                    })
                    .collect();

                for m in custom_metas {
                    // 自定义技能同名时覆盖系统技能的展示
                    all_skills.retain(|v| v.get("name").and_then(|n| n.as_str()) != Some(&m.name));
                    all_skills.push(serde_json::json!({
                        "name": m.name,
                        "display_name": m.display_name,
                        "description": m.description,
                        "aliases": m.aliases,
                        "tools": m.tools,
                        "type": "custom",
                        "can_delete": true,
                    }));
                }

                Ok(serde_json::json!({
                    "action": "list",
                    "skills": all_skills,
                    "note": "type=system 的技能不可通过 skill_tool 修改或删除"
                }))
            }

            "add" => {
                let name = match args.get("name").and_then(|v| v.as_str()) {
                    Some(n) if !n.is_empty() => n,
                    _ => {
                        return Ok(serde_json::json!({"success": false, "error": "name 不能为空"}));
                    }
                };

                // 格式校验
                if !name
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_alphabetic())
                    .unwrap_or(false)
                    || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "name 只能包含字母、数字、下划线，且必须以字母开头"
                    }));
                }

                // 不允许与系统技能同名
                if self.is_system_skill(name) {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("技能 '{}' 是系统内置技能，不可覆盖。请换一个名称", name)
                    }));
                }

                let display_name = args
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(name);
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");

                let aliases_yaml = build_yaml_list(&args, "aliases", "  ");
                let tools_yaml = build_yaml_list(&args, "tools", "  ");

                let skill_dir = self.custom_skills_dir.join(name);
                if let Err(e) = std::fs::create_dir_all(&skill_dir) {
                    return Ok(
                        serde_json::json!({"success": false, "error": format!("创建技能目录失败: {}", e)}),
                    );
                }

                let content = build_skill_md(
                    display_name,
                    description,
                    &aliases_yaml,
                    &tools_yaml,
                    prompt,
                );

                if let Err(e) = std::fs::write(skill_dir.join("SKILL.md"), content) {
                    return Ok(
                        serde_json::json!({"success": false, "error": format!("写入技能文件失败: {}", e)}),
                    );
                }

                Ok(serde_json::json!({
                    "success": true,
                    "message": format!("自定义技能 '{}' 已成功添加", name),
                    "name": name,
                    "display_name": display_name,
                }))
            }

            "update" => {
                let name = match args.get("name").and_then(|v| v.as_str()) {
                    Some(n) if !n.is_empty() => n,
                    _ => {
                        return Ok(serde_json::json!({"success": false, "error": "name 不能为空"}));
                    }
                };

                // 不允许修改系统技能
                if self.is_system_skill(name) {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("技能 '{}' 是系统内置技能（type=system），不可通过聊天修改。如需修改请直接编辑 skills/{}/SKILL.md", name, name)
                    }));
                }

                let skill_dir = self.custom_skills_dir.join(name);
                let skill_md_path = skill_dir.join("SKILL.md");

                if !skill_md_path.exists() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("自定义技能 '{}' 不存在", name)
                    }));
                }

                // 读取现有内容
                let existing = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
                let (existing_fm, existing_body) = parse_existing_skill(&existing);

                // 用传入的字段覆盖，未传入的保持原值
                let display_name = args
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&existing_fm.display_name);
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&existing_fm.description);
                let prompt = args
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&existing_body);

                let aliases_yaml = if args.get("aliases").is_some() {
                    build_yaml_list(&args, "aliases", "  ")
                } else {
                    existing_fm.aliases_yaml.clone()
                };

                let tools_yaml = if args.get("tools").is_some() {
                    build_yaml_list(&args, "tools", "  ")
                } else {
                    existing_fm.tools_yaml.clone()
                };

                let content = build_skill_md(
                    display_name,
                    description,
                    &aliases_yaml,
                    &tools_yaml,
                    prompt,
                );

                if let Err(e) = std::fs::write(&skill_md_path, content) {
                    return Ok(
                        serde_json::json!({"success": false, "error": format!("更新技能文件失败: {}", e)}),
                    );
                }

                Ok(serde_json::json!({
                    "success": true,
                    "message": format!("自定义技能 '{}' 已更新", name),
                    "name": name,
                }))
            }

            "remove" => {
                let name = match args.get("name").and_then(|v| v.as_str()) {
                    Some(n) if !n.is_empty() => n,
                    _ => {
                        return Ok(serde_json::json!({"success": false, "error": "name 不能为空"}));
                    }
                };

                // 不允许删除系统技能
                if self.is_system_skill(name) {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("技能 '{}' 是系统内置技能（type=system），不可删除", name)
                    }));
                }

                let custom_skill_dir = self.custom_skills_dir.join(name);
                if !custom_skill_dir.exists() {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": format!("自定义技能 '{}' 不存在", name)
                    }));
                }

                if let Err(e) = std::fs::remove_dir_all(&custom_skill_dir) {
                    return Ok(
                        serde_json::json!({"success": false, "error": format!("删除技能失败: {}", e)}),
                    );
                }

                Ok(serde_json::json!({
                    "success": true,
                    "message": format!("自定义技能 '{}' 已移除", name)
                }))
            }

            _ => Ok(
                serde_json::json!({"error": format!("不支持的操作: {}，可用操作：list / add / update / remove", action)}),
            ),
        }
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

/// 将 JSON args 中的数组字段转换为 YAML 缩进列表字符串
fn build_yaml_list(args: &Value, field: &str, indent: &str) -> String {
    let mut lines = String::new();
    if let Some(arr) = args.get(field).and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(s) = item.as_str() {
                lines.push_str(&format!("{}- {}\n", indent, s));
            }
        }
    }
    lines
}

/// 生成 SKILL.md 内容
fn build_skill_md(
    display_name: &str,
    description: &str,
    aliases_yaml: &str,
    tools_yaml: &str,
    prompt: &str,
) -> String {
    let mut content = format!(
        "---\nname: {}\ndescription: {}\n",
        display_name, description
    );

    if !aliases_yaml.is_empty() {
        content.push_str("aliases:\n");
        content.push_str(aliases_yaml);
    }

    if !tools_yaml.is_empty() {
        content.push_str("tools:\n");
        content.push_str(tools_yaml);
    }

    content.push_str("---\n\n");
    content.push_str(prompt);
    content.push('\n');
    content
}

/// 解析现有 SKILL.md 中的 frontmatter 字段，用于 update 时保留未传入的字段
struct ExistingFrontmatter {
    display_name: String,
    description: String,
    aliases_yaml: String,
    tools_yaml: String,
}

fn parse_existing_skill(content: &str) -> (ExistingFrontmatter, String) {
    let rest = match content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    {
        Some(r) => r,
        None => {
            return (
                ExistingFrontmatter {
                    display_name: String::new(),
                    description: String::new(),
                    aliases_yaml: String::new(),
                    tools_yaml: String::new(),
                },
                content.to_string(),
            );
        }
    };

    let end_marker = match rest.find("\n---\n").or_else(|| rest.find("\n---\r\n")) {
        Some(pos) => pos,
        None => {
            return (
                ExistingFrontmatter {
                    display_name: String::new(),
                    description: String::new(),
                    aliases_yaml: String::new(),
                    tools_yaml: String::new(),
                },
                content.to_string(),
            );
        }
    };

    let yaml_part = &rest[..end_marker];
    let body_start = end_marker + "\n---\n".len();
    let body = rest.get(body_start..).unwrap_or("").trim().to_string();

    // 简单提取 name 和 description（单行值）
    let mut display_name = String::new();
    let mut description = String::new();
    let mut aliases_yaml = String::new();
    let mut tools_yaml = String::new();

    let mut in_aliases = false;
    let mut in_tools = false;

    for line in yaml_part.lines() {
        if line.starts_with("name:") {
            display_name = line["name:".len()..].trim().to_string();
            in_aliases = false;
            in_tools = false;
        } else if line.starts_with("description:") {
            description = line["description:".len()..].trim().to_string();
            in_aliases = false;
            in_tools = false;
        } else if line.starts_with("aliases:") {
            in_aliases = true;
            in_tools = false;
        } else if line.starts_with("tools:") {
            in_tools = true;
            in_aliases = false;
        } else if line.starts_with("  - ") || line.starts_with("- ") {
            if in_aliases {
                aliases_yaml.push_str(line);
                aliases_yaml.push('\n');
            } else if in_tools {
                tools_yaml.push_str(line);
                tools_yaml.push('\n');
            }
        } else if !line.starts_with(' ') && !line.is_empty() {
            // 新的非缩进字段，退出列表模式
            in_aliases = false;
            in_tools = false;
        }
    }

    (
        ExistingFrontmatter {
            display_name,
            description,
            aliases_yaml,
            tools_yaml,
        },
        body,
    )
}
