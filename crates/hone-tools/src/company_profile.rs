//! CompanyProfileTool - 公司画像管理工具

use std::collections::BTreeMap;
use std::path::PathBuf;

use async_trait::async_trait;
use hone_memory::{
    AppendEventInput, CompanyProfileStorage, CreateProfileInput, IndustryTemplate, TrackingConfig,
};
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

pub struct CompanyProfileTool {
    root_dir: PathBuf,
}

impl CompanyProfileTool {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    fn storage(&self) -> CompanyProfileStorage {
        CompanyProfileStorage::new(&self.root_dir)
    }
}

#[async_trait]
impl Tool for CompanyProfileTool {
    fn name(&self) -> &str {
        "company_profile"
    }

    fn description(&self) -> &str {
        "管理公司画像 Markdown 文档。支持 exists、create、get_profile、list_profiles、append_event、rewrite_sections、set_tracking，用于建立长期画像、维护 thesis、追加事件时间线，以及保存 why / evidence / research trail。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型：exists / create / get_profile / list_profiles / append_event / rewrite_sections / set_tracking".to_string(),
                required: true,
                r#enum: Some(vec![
                    "exists".into(),
                    "create".into(),
                    "get_profile".into(),
                    "list_profiles".into(),
                    "append_event".into(),
                    "rewrite_sections".into(),
                    "set_tracking".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "profile_id".to_string(),
                param_type: "string".to_string(),
                description: "画像 ID（get_profile / append_event / rewrite_sections / set_tracking 时使用）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "company_name".to_string(),
                param_type: "string".to_string(),
                description: "公司名称（exists / create 时用于定位或创建画像）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "stock_code".to_string(),
                param_type: "string".to_string(),
                description: "股票代码（exists / create 时优先定位）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "sector".to_string(),
                param_type: "string".to_string(),
                description: "所属 sector，create 时可选。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "industry_template".to_string(),
                param_type: "string".to_string(),
                description: "行业模板：general / saas / semiconductor_hardware / consumer / industrial_defense / financials".to_string(),
                required: false,
                r#enum: Some(vec![
                    "general".into(),
                    "saas".into(),
                    "semiconductor_hardware".into(),
                    "consumer".into(),
                    "industrial_defense".into(),
                    "financials".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "aliases".to_string(),
                param_type: "array".to_string(),
                description: "画像别名列表，create 时可选。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({"type": "string"})),
            },
            ToolParameter {
                name: "sections".to_string(),
                param_type: "object".to_string(),
                description: "section -> markdown 内容映射，rewrite_sections 或 create 初始内容时使用。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "事件标题，append_event 时必填。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "event_type".to_string(),
                param_type: "string".to_string(),
                description: "事件类型，如 earnings / filing / review / management_change。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "occurred_at".to_string(),
                param_type: "string".to_string(),
                description: "事件发生时间（建议 ISO 8601）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "thesis_impact".to_string(),
                param_type: "string".to_string(),
                description: "事件对 thesis 的影响：positive / neutral / negative / mixed / unknown".to_string(),
                required: false,
                r#enum: Some(vec![
                    "positive".into(),
                    "neutral".into(),
                    "negative".into(),
                    "mixed".into(),
                    "unknown".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "changed_sections".to_string(),
                param_type: "array".to_string(),
                description: "受影响的画像 section 列表。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({"type": "string"})),
            },
            ToolParameter {
                name: "refs".to_string(),
                param_type: "array".to_string(),
                description: "引用来源列表。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({"type": "string"})),
            },
            ToolParameter {
                name: "what_happened".to_string(),
                param_type: "string".to_string(),
                description: "事件发生了什么。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "thesis_effect".to_string(),
                param_type: "string".to_string(),
                description: "事件如何影响当前 thesis。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "why_it_matters".to_string(),
                param_type: "string".to_string(),
                description: "为什么这件事重要，为什么值得写进长期画像。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "evidence".to_string(),
                param_type: "string".to_string(),
                description: "支撑判断的事实、引用摘要或证据说明。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "research_log".to_string(),
                param_type: "string".to_string(),
                description: "本轮研究路径，如看了哪些资料、用了哪些查询。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "follow_up".to_string(),
                param_type: "string".to_string(),
                description: "后续需要继续跟踪的内容。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "tracking_enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "是否开启长期追踪。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "tracking_cadence".to_string(),
                param_type: "string".to_string(),
                description: "追踪频率，默认 weekly。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "focus_metrics".to_string(),
                param_type: "array".to_string(),
                description: "重点跟踪指标列表。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({"type": "string"})),
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let action = args
            .get("action")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let storage = self.storage();

        match action {
            "exists" => {
                let company_name = args.get("company_name").and_then(|value| value.as_str());
                let stock_code = args.get("stock_code").and_then(|value| value.as_str());
                let profile_id = storage.find_profile_id(company_name, stock_code);
                Ok(serde_json::json!({
                    "success": true,
                    "exists": profile_id.is_some(),
                    "profile_id": profile_id
                }))
            }
            "list_profiles" => Ok(serde_json::json!({
                "success": true,
                "profiles": storage.list_profiles()
            })),
            "get_profile" => {
                let profile_id = required_string(&args, "profile_id")?;
                let profile = storage
                    .get_profile(profile_id)
                    .map_err(hone_core::HoneError::Tool)?
                    .map(|document| serde_json::to_value(document).unwrap_or_default());
                Ok(serde_json::json!({
                    "success": profile.is_some(),
                    "profile": profile
                }))
            }
            "create" => {
                let company_name = required_string(&args, "company_name")?;
                let stock_code = optional_string(&args, "stock_code");
                let sector = optional_string(&args, "sector");
                let aliases = string_array(&args, "aliases");
                let sections = string_map(&args, "sections");
                let industry_template =
                    parse_template(optional_string(&args, "industry_template").as_deref());
                let tracking = parse_tracking(&args);
                let (profile, created) = storage
                    .create_profile(CreateProfileInput {
                        company_name: company_name.to_string(),
                        stock_code,
                        sector,
                        aliases,
                        industry_template,
                        tracking: Some(tracking),
                        initial_sections: sections,
                    })
                    .map_err(hone_core::HoneError::Tool)?;

                Ok(serde_json::json!({
                    "success": true,
                    "created": created,
                    "profile_id": profile.profile_id,
                    "profile": profile
                }))
            }
            "append_event" => {
                let profile_id = required_string(&args, "profile_id")?;
                let title = required_string(&args, "title")?;
                let event_type = required_string(&args, "event_type")?;
                let occurred_at = required_string(&args, "occurred_at")?;
                let event = storage
                    .append_event(
                        profile_id,
                        AppendEventInput {
                            title: title.to_string(),
                            event_type: event_type.to_string(),
                            occurred_at: occurred_at.to_string(),
                            thesis_impact: optional_string(&args, "thesis_impact")
                                .unwrap_or_else(|| "unknown".to_string()),
                            changed_sections: string_array(&args, "changed_sections"),
                            refs: string_array(&args, "refs"),
                            what_happened: optional_string(&args, "what_happened")
                                .unwrap_or_default(),
                            why_it_matters: optional_string(&args, "why_it_matters")
                                .unwrap_or_default(),
                            thesis_effect: optional_string(&args, "thesis_effect")
                                .unwrap_or_default(),
                            evidence: optional_string(&args, "evidence").unwrap_or_default(),
                            research_log: optional_string(&args, "research_log")
                                .unwrap_or_default(),
                            follow_up: optional_string(&args, "follow_up").unwrap_or_default(),
                        },
                    )
                    .map_err(hone_core::HoneError::Tool)?;

                Ok(serde_json::json!({
                    "success": event.is_some(),
                    "event": event
                }))
            }
            "rewrite_sections" => {
                let profile_id = required_string(&args, "profile_id")?;
                let sections = string_map(&args, "sections");
                let profile = storage
                    .rewrite_sections(profile_id, &sections)
                    .map_err(hone_core::HoneError::Tool)?;
                Ok(serde_json::json!({
                    "success": profile.is_some(),
                    "profile": profile
                }))
            }
            "set_tracking" => {
                let profile_id = required_string(&args, "profile_id")?;
                let profile = storage
                    .set_tracking(profile_id, parse_tracking(&args))
                    .map_err(hone_core::HoneError::Tool)?;
                Ok(serde_json::json!({
                    "success": profile.is_some(),
                    "profile": profile
                }))
            }
            _ => Ok(serde_json::json!({
                "success": false,
                "error": format!("不支持的操作: {action}")
            })),
        }
    }
}

fn parse_template(value: Option<&str>) -> IndustryTemplate {
    match value.unwrap_or("general").trim() {
        "saas" => IndustryTemplate::Saas,
        "semiconductor_hardware" => IndustryTemplate::SemiconductorHardware,
        "consumer" => IndustryTemplate::Consumer,
        "industrial_defense" => IndustryTemplate::IndustrialDefense,
        "financials" => IndustryTemplate::Financials,
        _ => IndustryTemplate::General,
    }
}

fn parse_tracking(args: &Value) -> TrackingConfig {
    TrackingConfig {
        enabled: args
            .get("tracking_enabled")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        cadence: optional_string(args, "tracking_cadence").unwrap_or_else(|| "weekly".to_string()),
        focus_metrics: string_array(args, "focus_metrics"),
    }
}

fn required_string<'a>(args: &'a Value, key: &str) -> hone_core::HoneResult<&'a str> {
    args.get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| hone_core::HoneError::Tool(format!("{key} 不能为空")))
}

fn optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn string_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn string_map(args: &Value, key: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(object) = args.get(key).and_then(|value| value.as_object()) {
        for (section, value) in object {
            if let Some(content) = value.as_str() {
                let section = section.trim();
                let content = content.trim();
                if !section.is_empty() && !content.is_empty() {
                    map.insert(section.to_string(), content.to_string());
                }
            }
        }
    }
    map
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

    #[tokio::test]
    async fn create_and_get_profile_round_trip() {
        let tool = CompanyProfileTool::new(make_temp_dir("company_profile_tool"));
        let created = tool
            .execute(serde_json::json!({
                "action": "create",
                "company_name": "ServiceNow",
                "stock_code": "NOW",
                "industry_template": "saas"
            }))
            .await
            .expect("create");
        assert_eq!(created["success"], true);
        assert_eq!(created["created"], true);

        let fetched = tool
            .execute(serde_json::json!({
                "action": "get_profile",
                "profile_id": created["profile_id"]
            }))
            .await
            .expect("fetch");
        assert_eq!(fetched["success"], true);
        assert_eq!(fetched["profile"]["metadata"]["stock_code"], "NOW");
    }
}
