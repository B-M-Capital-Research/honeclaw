//! AI 数据中心行业树：编译进二进制的研究底稿 + 运行时的管理员改动。
//!
//! 底稿（`skills/industry-map/references/industry-map.json`）与公司卡同一处理方式：
//! `include_str!` 编进二进制，改它要重新构建。但管理员需要在对话里直接修数据，
//! 而重新构建镜像再发一次版对「把某一行的反模式改一句话」来说太重了。
//!
//! 所以运行时改动走一份**追加式的改动日志**，存在数据目录里，读取时按顺序重放到底稿上。
//! 选日志而不是「整棵树的可写副本」有三个理由：底稿升级时改动不会被整体覆盖回去；
//! 「谁在什么时候改了什么」不需要另建一张审计表；面板要的「最近改了什么」直接就是日志尾部。
//!
//! 两个进程都读这份日志——web 进程渲染研究台，channels 进程做每轮注入——它们共享
//! `HONE_DATA_DIR`，所以看到的是同一份。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const BASE_JSON: &str = include_str!("../../../skills/industry-map/references/industry-map.json");

/// 面板与注入都只关心最近的几条；日志本身不截断，读的时候取尾部。
pub const RECENT_EDIT_LIMIT: usize = 8;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustryRoot {
    pub id: String,
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AiValuationLogic {
    #[serde(default)]
    pub driver_chain: String,
    #[serde(default)]
    pub key_variables: Vec<Value>,
    #[serde(default)]
    pub multiple_anchor: String,
    #[serde(default)]
    pub anti_pattern: String,
    /// 注入专用的压缩版：长版是给研究台页面看的，整段注入每轮要花上千 token。
    #[serde(default)]
    pub multiple_anchor_short: String,
    #[serde(default)]
    pub anti_pattern_short: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustryMember {
    pub symbol: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CoreWatch {
    pub what: String,
    #[serde(default)]
    pub why: String,
    #[serde(default)]
    pub cadence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustrySource {
    pub house: String,
    pub title: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub takeaway: String,
}

/// 这一行的收入最终由哪家上市公司的最近行为决定，以及写这一行的公司之前该去取它的哪几个读数。
/// 它是行业树从「一段说明」变成「本体」的那条边：存储、光通信、新云都挂在英伟达的财报上，
/// 但如果只写在 `core_watch` 的散文里，模型不会真的去取。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct UpstreamSignal {
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    /// demand_source（它买本行的东西）/ capex_source（它的资本开支是需求源头）/
    /// supply_gate（本行供给受它卡口）/ peer_signal（同业龙头，最早的景气读数）。
    #[serde(default)]
    pub relation: String,
    #[serde(default)]
    pub why: String,
    /// 去取它的哪几个读数，每条都是现有工具取得到的量。
    #[serde(default)]
    pub pull: Vec<String>,
    #[serde(default)]
    pub cadence: String,
}

pub const UPSTREAM_RELATIONS: &[&str] = &[
    "demand_source",
    "capex_source",
    "supply_gate",
    "peer_signal",
];

/// 管理员在线新增一个行业时提交的骨架；其余字段留空，之后用别的改动填。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NewIndustry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub one_liner: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Industry {
    pub id: String,
    pub name: String,
    pub parent: String,
    #[serde(default)]
    pub one_liner: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub ai_valuation_logic: AiValuationLogic,
    #[serde(default)]
    pub core_watch: Vec<CoreWatch>,
    #[serde(default)]
    pub members: Vec<IndustryMember>,
    #[serde(default)]
    pub sources: Vec<IndustrySource>,
    #[serde(default)]
    pub upstream_signals: Vec<UpstreamSignal>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustryMap {
    pub schema_version: u32,
    pub generated_at: String,
    pub root: IndustryRoot,
    pub industries: Vec<Industry>,
}

impl IndustryMap {
    pub fn industry(&self, id: &str) -> Option<&Industry> {
        self.industries.iter().find(|item| item.id == id)
    }

    fn industry_mut(&mut self, id: &str) -> Option<&mut Industry> {
        self.industries.iter_mut().find(|item| item.id == id)
    }
}

/// 一次管理员改动。`op` 决定还要读哪几个字段，未用到的留空。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustryEdit {
    /// RFC3339，服务端写入时间。
    pub at: String,
    /// 改动者的 user_id，用于面板展示与追责。
    pub by: String,
    pub industry: String,
    pub op: EditOp,
    /// 管理员说明这次为什么改；面板上和改动并排显示。
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EditOp {
    /// 改一段文本字段：one_liner / driver_chain / multiple_anchor / anti_pattern /
    /// multiple_anchor_short / anti_pattern_short。
    SetField {
        field: String,
        value: String,
    },
    AddMember {
        member: IndustryMember,
    },
    RemoveMember {
        symbol: String,
    },
    /// 改一家已有成员的 `role`（它是这家和同行的区别，最常需要微调）。
    SetMemberRole {
        symbol: String,
        role: String,
    },
    AddSource {
        source: IndustrySource,
    },
    RemoveSource {
        url: String,
    },
    AddWatch {
        watch: CoreWatch,
    },
    RemoveWatch {
        what: String,
    },
    AddUpstreamSignal {
        signal: UpstreamSignal,
    },
    RemoveUpstreamSignal {
        symbol: String,
    },
    /// 新增一个行业。`IndustryEdit.industry` 就是新行业的 id。
    AddIndustry {
        industry: NewIndustry,
    },
    /// 从树里移除一个行业。底稿不动，重放时跳过它——底稿升级后仍然可以恢复。
    RemoveIndustry,
}

impl EditOp {
    /// 面板上的一行摘要；不含正文，避免把整段改动铺在卡片里。
    pub fn summary(&self) -> String {
        match self {
            EditOp::SetField { field, value } => {
                format!("改写 {field}（{} 字）", value.chars().count())
            }
            EditOp::AddMember { member } => format!("加入公司 {}", member.symbol),
            EditOp::RemoveMember { symbol } => format!("移出公司 {symbol}"),
            EditOp::SetMemberRole { symbol, .. } => format!("改写 {symbol} 的行业位置"),
            EditOp::AddSource { source } => format!("新增来源 {}", source.house),
            EditOp::RemoveSource { .. } => "移除一条来源".to_string(),
            EditOp::AddWatch { watch } => format!("新增关注点「{}」", truncate(&watch.what, 18)),
            EditOp::RemoveWatch { what } => format!("移除关注点「{}」", truncate(what, 18)),
            EditOp::AddUpstreamSignal { signal } => format!("新增上游信号 {}", signal.symbol),
            EditOp::RemoveUpstreamSignal { symbol } => format!("移除上游信号 {symbol}"),
            EditOp::AddIndustry { industry } => format!("新增行业「{}」", industry.name),
            EditOp::RemoveIndustry => "移除整个行业".to_string(),
        }
    }
}

fn truncate(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    trimmed.chars().take(limit).collect::<String>() + "…"
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EditLog {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub edits: Vec<IndustryEdit>,
}

/// 可改写的文本字段白名单。放开任意字段会让 `key_variables` 这种结构化内容被写成一段散文。
pub const EDITABLE_FIELDS: &[&str] = &[
    "one_liner",
    "driver_chain",
    "multiple_anchor",
    "anti_pattern",
    "multiple_anchor_short",
    "anti_pattern_short",
];

pub fn base_map() -> IndustryMap {
    serde_json::from_str(BASE_JSON).expect("embedded industry map must remain valid JSON")
}

pub fn log_path(data_root: &Path) -> PathBuf {
    data_root.join("industry_map").join("edits.json")
}

pub fn load_log(data_root: &Path) -> EditLog {
    let path = log_path(data_root);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return EditLog::default();
    };
    match serde_json::from_str::<EditLog>(&text) {
        Ok(log) => log,
        Err(error) => {
            // 日志损坏时退回底稿而不是让整棵树消失：树的研究内容本身是完好的。
            tracing::warn!("industry map edit log unreadable at {path:?}: {error}");
            EditLog::default()
        }
    }
}

/// 底稿 + 改动日志。日志按顺序重放，指向不存在的行业或成员的那条被跳过
/// （底稿升级后可能删过某一行，旧改动不应让整棵树读不出来）。
pub fn load(data_root: &Path) -> (IndustryMap, Vec<IndustryEdit>) {
    let mut map = base_map();
    let log = load_log(data_root);
    let mut applied = Vec::new();
    for edit in log.edits {
        if apply(&mut map, &edit).is_ok() {
            applied.push(edit);
        }
    }
    (map, applied)
}

#[derive(Debug, PartialEq)]
pub enum ApplyError {
    UnknownIndustry(String),
    DuplicateIndustry(String),
    InvalidIndustryId(String),
    UnknownField(String),
    UnknownMember(String),
    DuplicateMember(String),
    UnknownSignal(String),
    DuplicateSignal(String),
    InvalidRelation(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::UnknownIndustry(id) => write!(formatter, "没有这个行业：{id}"),
            ApplyError::UnknownField(field) => write!(
                formatter,
                "不能改这个字段：{field}（可改：{}）",
                EDITABLE_FIELDS.join(" / ")
            ),
            ApplyError::UnknownMember(symbol) => write!(formatter, "这一行里没有 {symbol}"),
            ApplyError::DuplicateMember(symbol) => write!(formatter, "{symbol} 已经在这一行里了"),
            ApplyError::DuplicateIndustry(id) => write!(formatter, "已经有这个行业了：{id}"),
            ApplyError::InvalidIndustryId(id) => {
                write!(formatter, "行业 id 只能用小写字母、数字和连字符：{id}")
            }
            ApplyError::UnknownSignal(symbol) => {
                write!(formatter, "这一行的上游信号里没有 {symbol}")
            }
            ApplyError::DuplicateSignal(symbol) => {
                write!(formatter, "{symbol} 已经是这一行的上游信号了")
            }
            ApplyError::InvalidRelation(relation) => write!(
                formatter,
                "relation 不合法：{relation}（可用：{}）",
                UPSTREAM_RELATIONS.join(" / ")
            ),
        }
    }
}

pub fn apply(map: &mut IndustryMap, edit: &IndustryEdit) -> Result<(), ApplyError> {
    // 行业级的两个动作先处理：它们的前提恰好和其它动作相反（新增要求不存在）。
    match &edit.op {
        EditOp::AddIndustry { industry } => {
            let id = edit.industry.trim();
            if id.is_empty()
                || !id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(ApplyError::InvalidIndustryId(edit.industry.clone()));
            }
            if map.industry(id).is_some() {
                return Err(ApplyError::DuplicateIndustry(id.to_string()));
            }
            map.industries.push(Industry {
                id: id.to_string(),
                name: industry.name.clone(),
                parent: map.root.id.clone(),
                one_liner: industry.one_liner.clone(),
                aliases: industry.aliases.clone(),
                ai_valuation_logic: AiValuationLogic::default(),
                core_watch: Vec::new(),
                members: Vec::new(),
                sources: Vec::new(),
                upstream_signals: Vec::new(),
            });
            return Ok(());
        }
        EditOp::RemoveIndustry => {
            let before = map.industries.len();
            map.industries.retain(|item| item.id != edit.industry);
            if map.industries.len() == before {
                return Err(ApplyError::UnknownIndustry(edit.industry.clone()));
            }
            return Ok(());
        }
        _ => {}
    }
    let industry = map
        .industry_mut(&edit.industry)
        .ok_or_else(|| ApplyError::UnknownIndustry(edit.industry.clone()))?;
    match &edit.op {
        EditOp::AddIndustry { .. } | EditOp::RemoveIndustry => unreachable!("handled above"),
        EditOp::AddUpstreamSignal { signal } => {
            if !UPSTREAM_RELATIONS.contains(&signal.relation.as_str()) {
                return Err(ApplyError::InvalidRelation(signal.relation.clone()));
            }
            if industry
                .upstream_signals
                .iter()
                .any(|item| item.symbol == signal.symbol)
            {
                return Err(ApplyError::DuplicateSignal(signal.symbol.clone()));
            }
            industry.upstream_signals.push(signal.clone());
        }
        EditOp::RemoveUpstreamSignal { symbol } => {
            let before = industry.upstream_signals.len();
            industry
                .upstream_signals
                .retain(|item| &item.symbol != symbol);
            if industry.upstream_signals.len() == before {
                return Err(ApplyError::UnknownSignal(symbol.clone()));
            }
        }
        EditOp::SetField { field, value } => match field.as_str() {
            "one_liner" => industry.one_liner = value.clone(),
            "driver_chain" => industry.ai_valuation_logic.driver_chain = value.clone(),
            "multiple_anchor" => industry.ai_valuation_logic.multiple_anchor = value.clone(),
            "anti_pattern" => industry.ai_valuation_logic.anti_pattern = value.clone(),
            "multiple_anchor_short" => {
                industry.ai_valuation_logic.multiple_anchor_short = value.clone();
            }
            "anti_pattern_short" => {
                industry.ai_valuation_logic.anti_pattern_short = value.clone();
            }
            other => return Err(ApplyError::UnknownField(other.to_string())),
        },
        EditOp::AddMember { member } => {
            if industry
                .members
                .iter()
                .any(|item| item.symbol == member.symbol)
            {
                return Err(ApplyError::DuplicateMember(member.symbol.clone()));
            }
            industry.members.push(member.clone());
        }
        EditOp::RemoveMember { symbol } => {
            let before = industry.members.len();
            industry.members.retain(|item| &item.symbol != symbol);
            if industry.members.len() == before {
                return Err(ApplyError::UnknownMember(symbol.clone()));
            }
        }
        EditOp::SetMemberRole { symbol, role } => {
            let member = industry
                .members
                .iter_mut()
                .find(|item| &item.symbol == symbol)
                .ok_or_else(|| ApplyError::UnknownMember(symbol.clone()))?;
            member.role = role.clone();
        }
        EditOp::AddSource { source } => industry.sources.push(source.clone()),
        EditOp::RemoveSource { url } => industry.sources.retain(|item| &item.url != url),
        EditOp::AddWatch { watch } => industry.core_watch.push(watch.clone()),
        EditOp::RemoveWatch { what } => industry.core_watch.retain(|item| &item.what != what),
    }
    Ok(())
}

/// 把一次改动追加到日志。先在底稿的重放结果上试跑一遍：改不动的（行业不存在、
/// 字段不可改、成员重复）当场返回错误，不写进日志——否则每次读取都要重放一条注定失败的记录。
pub fn append(data_root: &Path, edit: IndustryEdit) -> Result<IndustryMap, ApplyError> {
    let (mut map, _) = load(data_root);
    apply(&mut map, &edit)?;
    let mut log = load_log(data_root);
    log.schema_version = 1;
    log.edits.push(edit);
    let path = log_path(data_root);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&log) {
        Ok(text) => {
            let temp = path.with_extension("json.tmp");
            if std::fs::write(&temp, text).is_ok() {
                let _ = std::fs::rename(&temp, &path);
            }
        }
        Err(error) => tracing::warn!("industry map edit log not serializable: {error}"),
    }
    Ok(map)
}

/// 每个行业最近一次改动的时间，用于面板上给改过的行业打标记。
pub fn last_edited(edits: &[IndustryEdit]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for edit in edits {
        out.insert(edit.industry.clone(), edit.at.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(industry: &str, op: EditOp) -> IndustryEdit {
        IndustryEdit {
            at: "2026-08-30T12:00:00Z".to_string(),
            by: "web-user-test".to_string(),
            industry: industry.to_string(),
            op,
            note: "测试".to_string(),
        }
    }

    #[test]
    fn shipped_base_map_parses_and_carries_short_injection_fields() {
        let map = base_map();
        assert_eq!(map.schema_version, 1);
        assert!(!map.industries.is_empty());
        for industry in &map.industries {
            assert_eq!(industry.parent, map.root.id);
            assert!(!industry.members.is_empty(), "{} 没有成员", industry.id);
            // 注入读的是短版；长版留给研究台页面。
            assert!(
                !industry.ai_valuation_logic.multiple_anchor_short.is_empty(),
                "{} 缺短版倍数锚",
                industry.id
            );
            assert!(
                !industry.ai_valuation_logic.anti_pattern_short.is_empty(),
                "{} 缺短版反模式",
                industry.id
            );
        }
    }

    #[test]
    fn set_field_only_accepts_the_whitelisted_text_fields() {
        let mut map = base_map();
        let ok = apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetField {
                    field: "anti_pattern_short".into(),
                    value: "不要把峰值毛利率上的低 P/E 当便宜。".into(),
                },
            ),
        );
        assert!(ok.is_ok());
        assert_eq!(
            map.industry("storage")
                .unwrap()
                .ai_valuation_logic
                .anti_pattern_short,
            "不要把峰值毛利率上的低 P/E 当便宜。"
        );

        // key_variables 是结构化内容，放开会让它被写成一段散文。
        let rejected = apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetField {
                    field: "key_variables".into(),
                    value: "随便写点什么".into(),
                },
            ),
        );
        assert_eq!(
            rejected,
            Err(ApplyError::UnknownField("key_variables".to_string()))
        );
    }

    #[test]
    fn member_edits_reject_duplicates_and_unknown_symbols() {
        let mut map = base_map();
        let existing = map.industry("storage").unwrap().members[0].symbol.clone();
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::AddMember {
                        member: IndustryMember {
                            symbol: existing.clone(),
                            name: "重复".into(),
                            role: "重复".into(),
                        },
                    },
                ),
            ),
            Err(ApplyError::DuplicateMember(existing))
        );
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::RemoveMember {
                        symbol: "NOPE".into(),
                    },
                ),
            ),
            Err(ApplyError::UnknownMember("NOPE".to_string()))
        );
    }

    #[test]
    fn an_edit_that_no_longer_applies_is_skipped_instead_of_breaking_the_whole_tree() {
        // 底稿升级后可能删掉某一行；旧改动不应让整棵树读不出来。
        let mut map = base_map();
        let stale = edit(
            "an-industry-that-was-removed",
            EditOp::SetField {
                field: "one_liner".into(),
                value: "x".into(),
            },
        );
        assert!(apply(&mut map, &stale).is_err());
        assert_eq!(map.industries.len(), base_map().industries.len());
    }

    #[test]
    fn edits_replay_in_order_and_the_last_one_wins() {
        let dir = std::env::temp_dir().join(format!("hone-industry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for value in ["第一版", "第二版"] {
            append(
                &dir,
                edit(
                    "storage",
                    EditOp::SetField {
                        field: "one_liner".into(),
                        value: value.to_string(),
                    },
                ),
            )
            .expect("append");
        }
        let (map, applied) = load(&dir);
        assert_eq!(map.industry("storage").unwrap().one_liner, "第二版");
        assert_eq!(applied.len(), 2);
        assert_eq!(
            last_edited(&applied).get("storage").map(String::as_str),
            Some("2026-08-30T12:00:00Z")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_rejected_edit_never_reaches_the_log() {
        let dir = std::env::temp_dir().join(format!("hone-industry-rej-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let bad = append(
            &dir,
            edit(
                "storage",
                EditOp::SetField {
                    field: "members".into(),
                    value: "x".into(),
                },
            ),
        );
        assert!(bad.is_err());
        assert!(load_log(&dir).edits.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_industry_can_be_added_online_then_filled_then_removed() {
        let mut map = base_map();
        let before = map.industries.len();
        apply(
            &mut map,
            &edit(
                "cooling",
                EditOp::AddIndustry {
                    industry: NewIndustry {
                        id: "cooling".into(),
                        name: "散热".into(),
                        one_liner: "机柜功率密度决定的液冷与风冷".into(),
                        aliases: vec!["散热".into(), "液冷".into()],
                    },
                },
            ),
        )
        .expect("add industry");
        assert_eq!(map.industries.len(), before + 1);
        assert_eq!(map.industry("cooling").unwrap().parent, map.root.id);
        // 新行业上可以继续挂内容
        apply(
            &mut map,
            &edit(
                "cooling",
                EditOp::AddUpstreamSignal {
                    signal: UpstreamSignal {
                        symbol: "NVDA".into(),
                        name: "英伟达".into(),
                        relation: "demand_source".into(),
                        why: "单机柜功率由 GPU 平台定".into(),
                        pull: vec!["下一代平台的机柜功率".into()],
                        cadence: "GTC".into(),
                    },
                },
            ),
        )
        .expect("add signal");
        assert_eq!(map.industry("cooling").unwrap().upstream_signals.len(), 1);
        // 重复的 id 与非法的 id 都被挡住
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "cooling",
                    EditOp::AddIndustry {
                        industry: NewIndustry {
                            id: "cooling".into(),
                            name: "x".into(),
                            one_liner: String::new(),
                            aliases: vec![],
                        },
                    },
                ),
            ),
            Err(ApplyError::DuplicateIndustry("cooling".into()))
        );
        assert!(matches!(
            apply(
                &mut map,
                &edit(
                    "Bad Id",
                    EditOp::AddIndustry {
                        industry: NewIndustry {
                            id: "Bad Id".into(),
                            name: "x".into(),
                            one_liner: String::new(),
                            aliases: vec![],
                        },
                    },
                ),
            ),
            Err(ApplyError::InvalidIndustryId(_))
        ));
        apply(&mut map, &edit("cooling", EditOp::RemoveIndustry)).expect("remove");
        assert!(map.industry("cooling").is_none());
        assert_eq!(map.industries.len(), before);
    }

    #[test]
    fn upstream_signals_reject_unknown_relations_and_duplicates() {
        let mut map = base_map();
        let bad = UpstreamSignal {
            symbol: "NVDA".into(),
            name: "英伟达".into(),
            relation: "friend".into(),
            why: String::new(),
            pull: vec![],
            cadence: String::new(),
        };
        assert_eq!(
            apply(
                &mut map,
                &edit("storage", EditOp::AddUpstreamSignal { signal: bad })
            ),
            Err(ApplyError::InvalidRelation("friend".into()))
        );
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::RemoveUpstreamSignal {
                        symbol: "ZZZZ".into()
                    }
                )
            ),
            Err(ApplyError::UnknownSignal("ZZZZ".into()))
        );
    }

    #[test]
    fn summaries_stay_short_enough_for_a_card_row() {
        let long = "这是一条很长很长的关注点，长到不应该整条铺在卡片上，应该被截断";
        let summary = EditOp::AddWatch {
            watch: CoreWatch {
                what: long.to_string(),
                why: String::new(),
                cadence: String::new(),
            },
        }
        .summary();
        assert!(summary.chars().count() <= 30, "{summary}");
        assert!(summary.ends_with("…」"));
    }
}
