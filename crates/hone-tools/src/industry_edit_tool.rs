//! IndustryMapEditTool —— 管理员在对话里直接改行业树。
//!
//! 树的研究底稿编译在二进制里，改一句话本来要重新构建镜像再发一次版。这个工具让管理员
//! 用一句话完成同样的事：改动写进数据目录里的追加式日志，web 进程与 channels 进程下一次
//! 读取时重放，研究台和每轮注入同时生效，不需要重启。
//!
//! 只注册给管理员（`bot_core` 里按 `is_admin_actor` 判断），普通用户的工具集里不存在这个名字。
//!
//! 刻意不开放 `key_variables`（结构化内容，用一段散文覆盖它会毁掉表格）。行业本身可以在线
//! 新增与移除：移除只是让重放跳过它，底稿不动，底稿升级后仍然可以恢复。

use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::PathBuf;

use hone_core::industry_map::{
    CoreWatch, EDITABLE_FIELDS, EditOp, IndustryEdit, IndustryMember, IndustrySource, NewIndustry,
    UPSTREAM_RELATIONS, UpstreamSignal,
};

use crate::base::{Tool, ToolParameter};

pub struct IndustryMapEditTool {
    data_root: PathBuf,
    /// 记进改动日志的 `by`，面板上会显示它。
    actor_user_id: String,
}

impl IndustryMapEditTool {
    pub fn new(data_root: PathBuf, actor_user_id: impl Into<String>) -> Self {
        Self {
            data_root,
            actor_user_id: actor_user_id.into(),
        }
    }
}

fn text(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn missing(field: &str) -> hone_core::HoneResult<Value> {
    Ok(json!({ "ok": false, "error": format!("缺少参数 {field}") }))
}

#[async_trait]
impl Tool for IndustryMapEditTool {
    fn name(&self) -> &str {
        "industry_map_edit"
    }

    fn description(&self) -> &str {
        "【仅管理员可用】读写 AI 数据中心行业树（研究台「行业分析」用的那棵）。\
        改动立即对研究台和后续对话的行业注入生效，不需要重启或发版，并会记进改动日志（谁、何时、改了什么、为什么），\
        在研究台顶部的「最近改动」卡片里显示。\n\
        `action=\"show\"` 先读当前内容再改——不要凭记忆改，树的内容会被别的管理员改动。\n\
        可改字段：`one_liner`（这一行是什么，一句话）、`driver_chain`（从 AI 侧可观测量到这一行收入/价格的传导链，是这一行的第一性公式）、\
        `multiple_anchor` 与 `anti_pattern`（研究台页面看的长版）、`multiple_anchor_short` 与 `anti_pattern_short`（每轮注入模型的压缩版，各控制在 110 字以内）。\n\
        成员公司只收美股与 ADR：带交易所后缀的代码（如 `000660.KS`）会被拒绝——它们取不到行情，也不在本产品的判断范围内。\n\
        **上游信号**（`add_upstream_signal` / `remove_upstream_signal`）是这棵树的本体边：这一行的收入由哪家上市公司的最近行为决定、写这一行的公司之前该先取它的哪几个读数（例如存储 → NVDA 的数据中心收入与毛利率指引）。relation 只能是 demand_source / capex_source / supply_gate / peer_signal。\n\
        行业可以在线新增（`add_industry`，id 只用小写字母数字连字符）与移除（`remove_industry`，只是从树里隐藏，底稿不动）。不能改 `key_variables`（结构化表格，用散文覆盖会毁掉它）。\n\
        每次改动都要写 `note` 说明依据，例如引用的研报或财报口径变化；它会和改动一起展示给其它管理员。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "show（读当前内容）/ set_field / add_member / remove_member / set_member_role / add_source / remove_source / add_watch / remove_watch / add_upstream_signal / remove_upstream_signal / add_industry / remove_industry".to_string(),
                required: true,
                r#enum: Some(vec![
                    "show".into(),
                    "set_field".into(),
                    "add_member".into(),
                    "remove_member".into(),
                    "set_member_role".into(),
                    "add_source".into(),
                    "remove_source".into(),
                    "add_watch".into(),
                    "remove_watch".into(),
                    "add_upstream_signal".into(),
                    "remove_upstream_signal".into(),
                    "add_industry".into(),
                    "remove_industry".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "industry".to_string(),
                param_type: "string".to_string(),
                description: "行业 id：ai-chip / storage / optical / power / neocloud / equipment / server-oem / hyperscaler。action=show 时省略则列出整棵树的概览。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "field".to_string(),
                param_type: "string".to_string(),
                description: format!("set_field 用：{}", EDITABLE_FIELDS.join(" / ")),
                required: false,
                r#enum: Some(EDITABLE_FIELDS.iter().map(|item| item.to_string()).collect()),
                items: None,
            },
            ToolParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "set_field 的新内容（整段替换，不是追加）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "symbol".to_string(),
                param_type: "string".to_string(),
                description: "成员公司的美股代码，用于 add_member / remove_member / set_member_role".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "add_member 用：公司名".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "role".to_string(),
                param_type: "string".to_string(),
                description: "add_member / set_member_role 用：它在这一行里的位置，一句话说清它和同行的差别".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "house".to_string(),
                param_type: "string".to_string(),
                description: "add_source 用：出具机构".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "add_source 用：材料标题".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "date".to_string(),
                param_type: "string".to_string(),
                description: "add_source 用：YYYY-MM-DD 或 YYYY-MM".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "add_source / remove_source 用：原文链接".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "takeaway".to_string(),
                param_type: "string".to_string(),
                description: "add_source 用：这份材料给这一行贡献了哪个具体数字或口径".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "what".to_string(),
                param_type: "string".to_string(),
                description: "add_watch / remove_watch 用：关注点本身".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "why".to_string(),
                param_type: "string".to_string(),
                description: "add_watch 用：为什么它重要、怎么传导".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "cadence".to_string(),
                param_type: "string".to_string(),
                description: "add_watch 用：什么频率出现（季度财报 / 月度出货 / 拍卖结果…）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "relation".to_string(),
                param_type: "string".to_string(),
                description: "add_upstream_signal 用：demand_source / capex_source / supply_gate / peer_signal".to_string(),
                required: false,
                r#enum: Some(UPSTREAM_RELATIONS.iter().map(|s| s.to_string()).collect()),
                items: None,
            },
            ToolParameter {
                name: "pull".to_string(),
                param_type: "string".to_string(),
                description: "add_upstream_signal 用：要去取的读数，多条用「；」分隔，每条都是现有工具取得到的量".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "id".to_string(),
                param_type: "string".to_string(),
                description: "add_industry 用：新行业 id（小写字母、数字、连字符），也可直接放在 industry 参数里".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "one_liner".to_string(),
                param_type: "string".to_string(),
                description: "add_industry 用：这一行是什么，一句话".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "aliases".to_string(),
                param_type: "string".to_string(),
                description: "add_industry 用：召回别名，多条用「；」分隔".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "note".to_string(),
                param_type: "string".to_string(),
                description: "这次为什么改。会和改动一起展示给其它管理员，写清依据。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let action = text(&args, "action").unwrap_or_default();
        let (map, edits) = hone_core::industry_map::load(&self.data_root);

        if action == "show" {
            return Ok(match text(&args, "industry") {
                Some(id) => match map.industry(&id) {
                    Some(industry) => json!({ "ok": true, "industry": industry }),
                    None => json!({
                        "ok": false,
                        "error": format!("没有这个行业：{id}"),
                        "available": map.industries.iter().map(|item| &item.id).collect::<Vec<_>>(),
                    }),
                },
                None => json!({
                    "ok": true,
                    "generated_at": map.generated_at,
                    "industries": map.industries.iter().map(|item| json!({
                        "id": item.id,
                        "name": item.name,
                        "one_liner": item.one_liner,
                        "members": item.members.len(),
                        "sources": item.sources.len(),
                    })).collect::<Vec<_>>(),
                    "recent_edits": edits.iter().rev().take(5).map(|edit| json!({
                        "at": edit.at, "by": edit.by, "industry": edit.industry,
                        "summary": edit.op.summary(), "note": edit.note,
                    })).collect::<Vec<_>>(),
                }),
            });
        }

        let Some(industry) = text(&args, "industry") else {
            return missing("industry");
        };

        let op = match action.as_str() {
            "set_field" => {
                let (Some(field), Some(value)) = (text(&args, "field"), text(&args, "value"))
                else {
                    return missing("field / value");
                };
                EditOp::SetField { field, value }
            }
            "add_member" => {
                let (Some(symbol), Some(name), Some(role)) = (
                    text(&args, "symbol"),
                    text(&args, "name"),
                    text(&args, "role"),
                ) else {
                    return missing("symbol / name / role");
                };
                // 非美股的代码带交易所后缀。它们取不到行情、也不在本产品的判断范围内，
                // 在这里拒绝比让它进树之后在页面上显示「—」要清楚。
                if symbol.contains('.') || symbol.contains(':') {
                    return Ok(json!({
                        "ok": false,
                        "error": format!("{symbol} 不是美股代码：行业树只收美股与 ADR。非美股同行的作用由这一行的传导链与关注点承载。"),
                    }));
                }
                EditOp::AddMember {
                    member: IndustryMember {
                        symbol: symbol.to_ascii_uppercase(),
                        name,
                        role,
                    },
                }
            }
            "remove_member" => {
                let Some(symbol) = text(&args, "symbol") else {
                    return missing("symbol");
                };
                EditOp::RemoveMember {
                    symbol: symbol.to_ascii_uppercase(),
                }
            }
            "set_member_role" => {
                let (Some(symbol), Some(role)) = (text(&args, "symbol"), text(&args, "role"))
                else {
                    return missing("symbol / role");
                };
                EditOp::SetMemberRole {
                    symbol: symbol.to_ascii_uppercase(),
                    role,
                }
            }
            "add_source" => {
                let (Some(house), Some(title)) = (text(&args, "house"), text(&args, "title"))
                else {
                    return missing("house / title");
                };
                EditOp::AddSource {
                    source: IndustrySource {
                        house,
                        title,
                        date: text(&args, "date").unwrap_or_default(),
                        url: text(&args, "url").unwrap_or_default(),
                        takeaway: text(&args, "takeaway").unwrap_or_default(),
                    },
                }
            }
            "remove_source" => {
                let Some(url) = text(&args, "url") else {
                    return missing("url");
                };
                EditOp::RemoveSource { url }
            }
            "add_watch" => {
                let Some(what) = text(&args, "what") else {
                    return missing("what");
                };
                EditOp::AddWatch {
                    watch: CoreWatch {
                        what,
                        why: text(&args, "why").unwrap_or_default(),
                        cadence: text(&args, "cadence").unwrap_or_default(),
                    },
                }
            }
            "remove_watch" => {
                let Some(what) = text(&args, "what") else {
                    return missing("what");
                };
                EditOp::RemoveWatch { what }
            }
            "add_upstream_signal" => {
                let (Some(symbol), Some(relation)) =
                    (text(&args, "symbol"), text(&args, "relation"))
                else {
                    return missing("symbol / relation");
                };
                if symbol.contains('.') || symbol.contains(':') {
                    return Ok(json!({ "ok": false,
                        "error": format!("{symbol} 不是美股代码：上游信号必须是能用工具取到财报的美股。") }));
                }
                EditOp::AddUpstreamSignal {
                    signal: UpstreamSignal {
                        symbol: symbol.to_ascii_uppercase(),
                        name: text(&args, "name").unwrap_or_default(),
                        relation,
                        why: text(&args, "why").unwrap_or_default(),
                        pull: text(&args, "pull")
                            .map(|v| {
                                v.split(['；', ';'])
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        cadence: text(&args, "cadence").unwrap_or_default(),
                    },
                }
            }
            "remove_upstream_signal" => {
                let Some(symbol) = text(&args, "symbol") else {
                    return missing("symbol");
                };
                EditOp::RemoveUpstreamSignal {
                    symbol: symbol.to_ascii_uppercase(),
                }
            }
            "add_industry" => {
                let Some(name) = text(&args, "name") else {
                    return missing("name");
                };
                let id = text(&args, "id").unwrap_or_else(|| industry.clone());
                EditOp::AddIndustry {
                    industry: NewIndustry {
                        id: id.clone(),
                        name,
                        one_liner: text(&args, "one_liner").unwrap_or_default(),
                        aliases: text(&args, "aliases")
                            .map(|v| {
                                v.split(['；', ';'])
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect()
                            })
                            .unwrap_or_default(),
                    },
                }
            }
            "remove_industry" => EditOp::RemoveIndustry,
            other => {
                return Ok(json!({ "ok": false, "error": format!("不支持的 action：{other}") }));
            }
        };

        let industry = match &op {
            EditOp::AddIndustry { industry: new } => new.id.clone(),
            _ => industry,
        };
        let edit = IndustryEdit {
            at: hone_core::local_now().to_rfc3339(),
            by: self.actor_user_id.clone(),
            industry: industry.clone(),
            op,
            note: text(&args, "note").unwrap_or_default(),
        };
        let summary = edit.op.summary();
        Ok(
            match hone_core::industry_map::append(&self.data_root, edit) {
                Ok(updated) => json!({
                    "ok": true,
                    "applied": summary,
                    "industry": industry,
                    "industry_after": updated.industry(&industry),
                    "note": "改动已生效：研究台「行业分析」与后续对话的行业注入都会读到新内容，不需要重启。",
                }),
                Err(error) => json!({ "ok": false, "error": error.to_string() }),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(dir: &std::path::Path) -> IndustryMapEditTool {
        IndustryMapEditTool::new(dir.to_path_buf(), "web-user-admin")
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hone-ind-tool-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[tokio::test]
    async fn show_without_an_industry_lists_the_tree_and_recent_edits() {
        let dir = temp("show");
        let out = tool(&dir)
            .execute(json!({ "action": "show" }))
            .await
            .expect("tool");
        assert_eq!(out["ok"], true);
        assert!(out["industries"].as_array().unwrap().len() >= 8);
        assert!(out["recent_edits"].as_array().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn set_field_takes_effect_on_the_next_read() {
        let dir = temp("set");
        let out = tool(&dir)
            .execute(json!({
                "action": "set_field",
                "industry": "storage",
                "field": "anti_pattern_short",
                "value": "不要把峰值毛利率上的低 P/E 当便宜。",
                "note": "按 2026-08 财报口径收紧"
            }))
            .await
            .expect("tool");
        assert_eq!(out["ok"], true, "{out}");
        let (map, edits) = hone_core::industry_map::load(&dir);
        assert_eq!(
            map.industry("storage")
                .unwrap()
                .ai_valuation_logic
                .anti_pattern_short,
            "不要把峰值毛利率上的低 P/E 当便宜。"
        );
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].by, "web-user-admin");
        assert_eq!(edits[0].note, "按 2026-08 财报口径收紧");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn a_non_us_symbol_is_refused_with_the_reason() {
        let dir = temp("nonus");
        let out = tool(&dir)
            .execute(json!({
                "action": "add_member",
                "industry": "storage",
                "symbol": "000660.KS",
                "name": "SK 海力士",
                "role": "HBM 份额领先"
            }))
            .await
            .expect("tool");
        assert_eq!(out["ok"], false);
        assert!(out["error"].as_str().unwrap().contains("只收美股"), "{out}");
        // 被拒的改动不能留在日志里，否则每次读取都要重放一条注定失败的记录。
        assert!(hone_core::industry_map::load_log(&dir).edits.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn structured_fields_stay_out_of_reach() {
        let dir = temp("struct");
        let out = tool(&dir)
            .execute(json!({
                "action": "set_field",
                "industry": "storage",
                "field": "key_variables",
                "value": "随便写点什么"
            }))
            .await
            .expect("tool");
        assert_eq!(out["ok"], false);
        assert!(
            out["error"].as_str().unwrap().contains("不能改这个字段"),
            "{out}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn upstream_signal_and_industry_lifecycle_work_through_the_tool() {
        let dir = temp("lifecycle");
        let t = tool(&dir);
        let added = t
            .execute(json!({
                "action": "add_industry", "industry": "cooling", "name": "散热",
                "one_liner": "机柜功率密度决定的液冷与风冷", "aliases": "散热；液冷",
                "note": "本地验证"
            }))
            .await
            .expect("tool");
        assert_eq!(added["ok"], true, "{added}");
        let sig = t
            .execute(json!({
                "action": "add_upstream_signal", "industry": "cooling", "symbol": "nvda",
                "name": "英伟达", "relation": "demand_source",
                "why": "单机柜功率由 GPU 平台定", "pull": "下一代平台机柜功率；数据中心收入指引",
                "cadence": "GTC / 季度财报"
            }))
            .await
            .expect("tool");
        assert_eq!(sig["ok"], true, "{sig}");
        let (map, edits) = hone_core::industry_map::load(&dir);
        let cooling = map.industry("cooling").expect("cooling exists");
        assert_eq!(cooling.aliases, vec!["散热", "液冷"]);
        assert_eq!(cooling.upstream_signals[0].symbol, "NVDA");
        assert_eq!(cooling.upstream_signals[0].pull.len(), 2);
        assert_eq!(edits.len(), 2);
        // 非法 relation 在核心层被拒，且不写日志
        let bad = t
            .execute(
                json!({ "action": "add_upstream_signal", "industry": "cooling",
                "symbol": "MSFT", "relation": "friend" }),
            )
            .await
            .expect("tool");
        assert_eq!(bad["ok"], false);
        assert!(bad["error"].as_str().unwrap().contains("relation"), "{bad}");
        assert_eq!(hone_core::industry_map::load_log(&dir).edits.len(), 2);
        let removed = t
            .execute(json!({ "action": "remove_industry", "industry": "cooling", "note": "撤回" }))
            .await
            .expect("tool");
        assert_eq!(removed["ok"], true, "{removed}");
        assert!(
            hone_core::industry_map::load(&dir)
                .0
                .industry("cooling")
                .is_none()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn member_symbols_are_normalised_to_upper_case() {
        let dir = temp("case");
        let added = tool(&dir)
            .execute(json!({
                "action": "add_member",
                "industry": "storage",
                "symbol": "kioxia",
                "name": "铠侠美国存托凭证",
                "role": "NAND 第三供给"
            }))
            .await
            .expect("tool");
        assert_eq!(added["ok"], true, "{added}");
        let (map, _) = hone_core::industry_map::load(&dir);
        assert!(
            map.industry("storage")
                .unwrap()
                .members
                .iter()
                .any(|member| member.symbol == "KIOXIA")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
