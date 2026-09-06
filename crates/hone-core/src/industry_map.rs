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
    /// `why` 里那些数字截至哪一天（或哪个月，"2026-06"）；空串 = 未标注。
    #[serde(default)]
    pub as_of: String,
}

/// 行业简报：页面第一块「当前重点」。管理员每季财报或口径变化后先改它。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct IndustryBrief {
    /// 现在值得研究的问题，一句。
    #[serde(default)]
    pub question: String,
    /// 为什么是现在，一段。
    #[serde(default)]
    pub body: String,
    /// 接下来要确认什么，一条一件事，建议以日期或事件开头。
    #[serde(default)]
    pub next: Vec<String>,
    /// 这份简报截至哪一天；SetBrief 要求非空且可解析。
    #[serde(default)]
    pub as_of: String,
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
    /// 它最近一季实际做了什么（带数字、带日期的一段）。这是注入时排在最前的一行：模型拿到的是
    /// 事实而不是「去取」的指令，所以能直接写进需求侧第一段；管理员每季财报后更新。
    #[serde(default)]
    pub latest: String,
    /// `latest` 截至哪一天（或哪个月），注入时随行带出，也让页面上一眼看出是否过期。
    #[serde(default)]
    pub latest_as_of: String,
}

/// HOne 前瞻估值执行版（V3.0）的通用部分：定位、原则、七段需求链、通用执行规则、强制输出字段。
/// 挂在树根上，所有行共用；注入时只带执行规则与输出字段的压缩版，全文留给页面与 skill。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Methodology {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub positioning: String,
    #[serde(default)]
    pub principle: String,
    #[serde(default)]
    pub demand_chain: Vec<String>,
    #[serde(default)]
    pub core_principle: String,
    #[serde(default)]
    pub execution_rules: Vec<MethodRule>,
    #[serde(default)]
    pub hindsight_error: String,
    #[serde(default)]
    pub output_fields: Vec<MethodRule>,
}

/// 「规则名 → 执行要求」或「输出字段 → 输出要求」，两张表同一形状。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct MethodRule {
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub requirement: String,
}

/// 一条公式：原文 + 一句说明。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Formula {
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub note: String,
}

/// 底层估值逻辑：未来 1–3 年收入、利润、现金流为什么会变。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ValuationLogic {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub paragraphs: Vec<String>,
    #[serde(default)]
    pub formulas: Vec<Formula>,
    #[serde(default)]
    pub forward_focus: Vec<String>,
    #[serde(default)]
    pub state_note: String,
}

/// 倍数锚：在什么阶段用哪个前瞻财年、哪一族倍数、区间由什么决定、什么被禁止。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ValuationAnchor {
    #[serde(default)]
    pub paragraphs: Vec<String>,
    #[serde(default)]
    pub upper_range_drivers: String,
    #[serde(default)]
    pub revision_optionality: String,
    #[serde(default)]
    pub forbidden: Vec<String>,
}

/// 子类型：同一行里价值链位置不同的公司，各自的主锚 / 次锚与适用阶段。
/// 这是「这些知识被消费」的最小单位——注入时只带命中公司所属的那一条。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Subtype {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub inferred_members: Vec<String>,
    #[serde(default)]
    pub primary: String,
    #[serde(default)]
    pub secondary: String,
    #[serde(default)]
    pub when: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct IndustryValuation {
    #[serde(default)]
    pub logic: ValuationLogic,
    #[serde(default)]
    pub anchor: ValuationAnchor,
    #[serde(default)]
    pub subtypes: Vec<Subtype>,
}

impl IndustryValuation {
    /// 一家公司在这一行里属于哪个子类型；不在任何子类型里就返回 None。
    pub fn subtype_of(&self, symbol: &str) -> Option<&Subtype> {
        self.subtypes
            .iter()
            .find(|subtype| subtype.members.iter().any(|member| member == symbol))
    }
}

/// `SetValuationField` 能改的文本字段。
pub const VALUATION_TEXT_FIELDS: &[&str] = &[
    "logic.summary",
    "logic.state_note",
    "anchor.upper_range_drivers",
    "anchor.revision_optionality",
];
/// `SetValuationList` 能改的列表字段（整表替换）。
pub const VALUATION_LIST_FIELDS: &[&str] = &[
    "logic.paragraphs",
    "logic.forward_focus",
    "anchor.paragraphs",
    "anchor.forbidden",
];

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
    /// HOne 前瞻估值执行版：底层估值逻辑、倍数锚、子类型。
    #[serde(default)]
    pub valuation: IndustryValuation,
    #[serde(default)]
    pub brief: Option<IndustryBrief>,
}

impl Industry {
    /// 这一行内容里最新的那个日期，原样返回写法（"2026-06" 就还是 "2026-06"）。
    /// 候选：brief.as_of、upstream_signals[].latest_as_of、core_watch[].as_of、sources[].date、
    /// ai_valuation_logic.key_variables[] 的可选字符串 as_of。解析不了的忽略；
    /// 同一天时日精度优先于月精度。不看 last_edited（那是编辑时钟，不是事实截至日）。
    pub fn content_as_of(&self) -> Option<String> {
        newest_as_of(
            self.brief
                .iter()
                .map(|brief| brief.as_of.as_str())
                .chain(
                    self.upstream_signals
                        .iter()
                        .map(|signal| signal.latest_as_of.as_str()),
                )
                .chain(self.core_watch.iter().map(|watch| watch.as_of.as_str()))
                .chain(self.sources.iter().map(|source| source.date.as_str()))
                .chain(
                    self.ai_valuation_logic
                        .key_variables
                        .iter()
                        .filter_map(|variable| variable.get("as_of").and_then(Value::as_str)),
                ),
        )
        .map(str::to_string)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IndustryMap {
    pub schema_version: u32,
    pub generated_at: String,
    pub root: IndustryRoot,
    pub industries: Vec<Industry>,
    /// 全树共用的方法论（V3）。
    #[serde(default)]
    pub methodology: Methodology,
}

impl IndustryMap {
    /// 各行 content_as_of 的最大值，同一天时也保留日精度。
    pub fn content_as_of(&self) -> Option<String> {
        newest_as_of(self.industries.iter().filter_map(Industry::content_as_of))
    }

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
    /// 整条替换关注点，按现有 what 定位（可顺带改名）；不单独改日期，避免数字与日期脱节。
    SetWatch {
        what: String,
        watch: CoreWatch,
    },
    RemoveWatch {
        what: String,
    },
    /// 写入或整体替换行业简报。
    SetBrief {
        brief: IndustryBrief,
    },
    ClearBrief,
    AddUpstreamSignal {
        signal: UpstreamSignal,
    },
    RemoveUpstreamSignal {
        symbol: String,
    },
    /// 只改一条上游信号的「最近动作」与截至日期——每季财报后最常做的那个动作。
    SetUpstreamLatest {
        symbol: String,
        latest: String,
        #[serde(default)]
        as_of: String,
    },
    /// 改前瞻估值里的一个文本字段（见 `VALUATION_TEXT_FIELDS`）。
    SetValuationField {
        field: String,
        value: String,
    },
    /// 整表替换前瞻估值里的一个列表字段（见 `VALUATION_LIST_FIELDS`）。
    SetValuationList {
        field: String,
        items: Vec<String>,
    },
    /// 新增或整体替换一个子类型（按 id 匹配）。
    UpsertSubtype {
        subtype: Subtype,
    },
    RemoveSubtype {
        id: String,
    },
    /// 把一家公司挪到某个子类型（从其它子类型里移出）。
    SetMemberSubtype {
        symbol: String,
        subtype: String,
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
            EditOp::SetWatch { watch, .. } => {
                format!("改写关注点「{}」", truncate(&watch.what, 18))
            }
            EditOp::RemoveWatch { what } => format!("移除关注点「{}」", truncate(what, 18)),
            EditOp::SetBrief { brief } => format!("写入行业简报（截至 {}）", brief.as_of),
            EditOp::ClearBrief => "清空行业简报".to_string(),
            EditOp::AddUpstreamSignal { signal } => format!("新增上游信号 {}", signal.symbol),
            EditOp::RemoveUpstreamSignal { symbol } => format!("移除上游信号 {symbol}"),
            EditOp::SetUpstreamLatest { symbol, as_of, .. } => {
                if as_of.trim().is_empty() {
                    format!("更新上游动作 {symbol}")
                } else {
                    format!("更新上游动作 {symbol}（截至 {as_of}）")
                }
            }
            EditOp::SetValuationField { field, value } => {
                format!("改写估值字段 {field}（{} 字）", value.chars().count())
            }
            EditOp::SetValuationList { field, items } => {
                format!("替换估值列表 {field}（{} 条）", items.len())
            }
            EditOp::UpsertSubtype { subtype } => format!("写入子类型「{}」", subtype.name),
            EditOp::RemoveSubtype { id } => format!("移除子类型 {id}"),
            EditOp::SetMemberSubtype { symbol, subtype } => {
                format!("把 {symbol} 归到子类型 {subtype}")
            }
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
    UnknownSubtype(String),
    InvalidSubtypeId(String),
    UnknownWatch(String),
    DuplicateWatch(String),
    EmptyBrief,
    InvalidDate(String),
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
            ApplyError::UnknownSubtype(id) => write!(formatter, "没有这个子类型：{id}"),
            ApplyError::InvalidSubtypeId(id) => {
                write!(formatter, "子类型 id 只能用小写字母、数字和连字符：{id}")
            }
            ApplyError::UnknownWatch(what) => write!(formatter, "这一行的关注点里没有「{what}」"),
            ApplyError::DuplicateWatch(what) => write!(formatter, "关注点「{what}」已经存在"),
            ApplyError::EmptyBrief => write!(formatter, "简报的 question 不能为空"),
            ApplyError::InvalidDate(text) => {
                write!(
                    formatter,
                    "日期写法不对：{text}（要 2026-08-26 或 2026-06）"
                )
            }
        }
    }
}

/// 接受 YYYY-MM-DD 或 YYYY-MM 两种截至日期；月精度按月初比较，空白与坏日期均忽略。
pub fn parse_as_of(text: &str) -> Option<chrono::NaiveDate> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(date);
    }
    if text.len() == 7
        && text.bytes().enumerate().all(|(index, byte)| {
            if index == 4 {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
    {
        return chrono::NaiveDate::parse_from_str(&format!("{text}-01"), "%Y-%m-%d").ok();
    }
    None
}

fn newest_as_of<T: AsRef<str>>(dates: impl IntoIterator<Item = T>) -> Option<T> {
    dates
        .into_iter()
        .filter_map(|text| {
            parse_as_of(text.as_ref()).map(|date| {
                // 月份折到月初只为排序，同日时仍应展示更精确的原始日日期。
                let day_precision = text.as_ref().bytes().filter(|byte| *byte == b'-').count() > 1;
                ((date, day_precision), text)
            })
        })
        .max_by_key(|(key, _)| *key)
        .map(|(_, text)| text)
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
                valuation: IndustryValuation::default(),
                brief: None,
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
        EditOp::SetUpstreamLatest {
            symbol,
            latest,
            as_of,
        } => {
            let Some(signal) = industry
                .upstream_signals
                .iter_mut()
                .find(|item| &item.symbol == symbol)
            else {
                return Err(ApplyError::UnknownSignal(symbol.clone()));
            };
            signal.latest = latest.trim().to_string();
            signal.latest_as_of = as_of.trim().to_string();
        }
        EditOp::SetValuationField { field, value } => {
            let value = value.trim().to_string();
            let v = &mut industry.valuation;
            match field.as_str() {
                "logic.summary" => v.logic.summary = value,
                "logic.state_note" => v.logic.state_note = value,
                "anchor.upper_range_drivers" => v.anchor.upper_range_drivers = value,
                "anchor.revision_optionality" => v.anchor.revision_optionality = value,
                other => return Err(ApplyError::UnknownField(other.to_string())),
            }
        }
        EditOp::SetValuationList { field, items } => {
            let items = items
                .iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>();
            let v = &mut industry.valuation;
            match field.as_str() {
                "logic.paragraphs" => v.logic.paragraphs = items,
                "logic.forward_focus" => v.logic.forward_focus = items,
                "anchor.paragraphs" => v.anchor.paragraphs = items,
                "anchor.forbidden" => v.anchor.forbidden = items,
                other => return Err(ApplyError::UnknownField(other.to_string())),
            }
        }
        EditOp::UpsertSubtype { subtype } => {
            let id = subtype.id.trim();
            if id.is_empty()
                || !id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(ApplyError::InvalidSubtypeId(subtype.id.clone()));
            }
            let mut fresh = subtype.clone();
            fresh.id = id.to_string();
            fresh.members = fresh
                .members
                .iter()
                .map(|m| m.trim().to_ascii_uppercase())
                .filter(|m| !m.is_empty())
                .collect();
            // 一家公司只能在一个子类型里：别的子类型里的同名成员先移出。
            for other in industry.valuation.subtypes.iter_mut() {
                if other.id != fresh.id {
                    other.members.retain(|m| !fresh.members.contains(m));
                    other
                        .inferred_members
                        .retain(|m| !fresh.members.contains(m));
                }
            }
            match industry
                .valuation
                .subtypes
                .iter_mut()
                .find(|existing| existing.id == fresh.id)
            {
                Some(existing) => *existing = fresh,
                None => industry.valuation.subtypes.push(fresh),
            }
        }
        EditOp::RemoveSubtype { id } => {
            let before = industry.valuation.subtypes.len();
            industry.valuation.subtypes.retain(|s| &s.id != id);
            if industry.valuation.subtypes.len() == before {
                return Err(ApplyError::UnknownSubtype(id.clone()));
            }
        }
        EditOp::SetMemberSubtype { symbol, subtype } => {
            let symbol = symbol.trim().to_ascii_uppercase();
            if !industry.members.iter().any(|m| m.symbol == symbol) {
                return Err(ApplyError::UnknownMember(symbol));
            }
            if !industry.valuation.subtypes.iter().any(|s| &s.id == subtype) {
                return Err(ApplyError::UnknownSubtype(subtype.clone()));
            }
            for s in industry.valuation.subtypes.iter_mut() {
                s.members.retain(|m| m != &symbol);
                s.inferred_members.retain(|m| m != &symbol);
                if &s.id == subtype {
                    s.members.push(symbol.clone());
                }
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
        EditOp::AddSource { source } => {
            // 线上改动折回底稿后仍会重放，同一 URL 不应因此重复出现。
            if !source.url.is_empty() && industry.sources.iter().any(|item| item.url == source.url)
            {
                return Ok(());
            }
            industry.sources.push(source.clone());
        }
        EditOp::RemoveSource { url } => industry.sources.retain(|item| &item.url != url),
        EditOp::AddWatch { watch } => {
            if industry
                .core_watch
                .iter()
                .any(|item| item.what.trim() == watch.what.trim())
            {
                return Err(ApplyError::DuplicateWatch(watch.what.clone()));
            }
            if !watch.as_of.is_empty() && parse_as_of(&watch.as_of).is_none() {
                return Err(ApplyError::InvalidDate(watch.as_of.clone()));
            }
            industry.core_watch.push(watch.clone());
        }
        EditOp::SetWatch { what, watch } => {
            let position = industry
                .core_watch
                .iter()
                .position(|item| item.what.trim() == what.trim())
                .ok_or_else(|| ApplyError::UnknownWatch(what.clone()))?;
            if industry
                .core_watch
                .iter()
                .enumerate()
                .any(|(index, item)| index != position && item.what.trim() == watch.what.trim())
            {
                return Err(ApplyError::DuplicateWatch(watch.what.clone()));
            }
            if !watch.as_of.is_empty() && parse_as_of(&watch.as_of).is_none() {
                return Err(ApplyError::InvalidDate(watch.as_of.clone()));
            }
            industry.core_watch[position] = watch.clone();
        }
        EditOp::RemoveWatch { what } => industry.core_watch.retain(|item| &item.what != what),
        EditOp::SetBrief { brief } => {
            if brief.question.trim().is_empty() {
                return Err(ApplyError::EmptyBrief);
            }
            if parse_as_of(&brief.as_of).is_none() {
                return Err(ApplyError::InvalidDate(brief.as_of.clone()));
            }
            industry.brief = Some(IndustryBrief {
                question: brief.question.trim().to_string(),
                body: brief.body.trim().to_string(),
                next: brief
                    .next
                    .iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect(),
                as_of: brief.as_of.trim().to_string(),
            });
        }
        EditOp::ClearBrief => industry.brief = None,
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
        assert_eq!(map.schema_version, 3);
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
            for date in industry
                .core_watch
                .iter()
                .map(|item| &item.as_of)
                .chain(
                    industry
                        .upstream_signals
                        .iter()
                        .map(|item| &item.latest_as_of),
                )
                .chain(industry.sources.iter().map(|item| &item.date))
                .filter(|date| !date.is_empty())
            {
                assert!(
                    parse_as_of(date).is_some(),
                    "{} 的日期不可解析：{date}",
                    industry.id
                );
            }
            let mut urls = std::collections::HashSet::new();
            for source in industry
                .sources
                .iter()
                .filter(|source| !source.url.is_empty())
            {
                assert!(
                    urls.insert(&source.url),
                    "{} 的来源 URL 重复：{}",
                    industry.id,
                    source.url
                );
            }
        }
    }

    /// 线上 `data/industry_map/edits.json` 里的 4 条改动（AVGO / DELL 的 role 与来源）已折回底稿。
    /// 重放时 `SetMemberRole` 会用日志文本覆盖底稿，所以底稿里这两段必须逐字等于日志——否则底稿改了也白改；
    /// `AddSource` 按 url 幂等，所以来源只会出现一次。
    #[test]
    fn live_edits_are_folded_into_the_base_verbatim() {
        let map = base_map();
        let ai_chip = map.industry("ai-chip").expect("ai-chip");
        let avgo = ai_chip
            .members
            .iter()
            .find(|member| member.symbol == "AVGO")
            .expect("AVGO");
        assert_eq!(
            avgo.role,
            "自研 ASIC 路线上最大的设计与交付方，同时卖以太网交换/路由芯片（Tomahawk、Jericho），是唯一一家在同一份 AI 收入里同时吃「定制 XPU」和「AI 网络」两条子线的公司；与 Marvell 的差别在于它锁定的是超大规模客户的多代路线图而非单个项目（Counterpoint 估其 2027 年 ASIC 设计伙伴份额约 60%）。最新财务基线（2026-09-02 发布，FY26Q3 截至 2026-08-02）：总收入 295.91 亿美元，AI 半导体收入 167 亿美元；经营现金流 141.97 亿美元，资本开支 5.32 亿美元，自由现金流 136.65 亿美元、约占收入 46%。FY26Q4 指引总收入约 348 亿美元、AI 半导体收入约 217 亿美元。上述 Q3 为已实现数据、Q4 为管理层指引；原 FY26Q2 数据及 Q3 旧指引仅作历史对照。"
        );
        let avgo_source_url = "https://www.sec.gov/Archives/edgar/data/1730168/000173016826000076/avgo-08022026x8kxex99.htm";
        assert_eq!(
            ai_chip
                .sources
                .iter()
                .filter(|source| source.url == avgo_source_url)
                .count(),
            1
        );
        let watch = ai_chip
            .core_watch
            .iter()
            .find(|watch| watch.what.starts_with("Broadcom 每季的 AI 半导体收入"))
            .expect("Broadcom 关注点");
        assert_eq!(watch.as_of, "2026-09-02");
        assert!(watch.why.contains("295.91") && !watch.why.contains("108 亿"));
        let variable = ai_chip
            .ai_valuation_logic
            .key_variables
            .iter()
            .find(|variable| {
                variable["name"]
                    .as_str()
                    .is_some_and(|name| name.starts_with("Broadcom 单季 AI 半导体收入"))
            })
            .expect("Broadcom 变量");
        assert_eq!(variable["as_of"], "2026-09-02");
        assert!(ai_chip.content_as_of().as_deref() == Some("2026-09-02"));

        let server_oem = map.industry("server-oem").expect("server-oem");
        let dell = server_oem
            .members
            .iter()
            .find(|member| member.symbol == "DELL")
            .expect("DELL");
        assert_eq!(
            dell.role,
            "品牌 OEM 里 AI 服务器规模最大的一家，靠企业与主权客户加上存储、服务的组合来补 AI 服务器的薄利。最新财务基线（2026-09-01 发布，FY27Q2 截至 2026-07-31）：ISG 营收 317.82 亿美元，营业利润 47.81 亿美元、营益率 15.0%；其中 AI 优化服务器收入 164.01 亿美元、传统服务器与网络 105.31 亿美元、存储 48.50 亿美元。FY27 全年 AI 服务器收入指引由 600 亿美元上修至 740 亿美元（管理层指引，不是已实现收入）。原 FY27Q1 数字及全年旧指引仅作历史对照。ISG 内部混着三块利润率完全不同的生意，是这一行里最需要做分部估值的标的。"
        );
        assert_eq!(
            server_oem
                .sources
                .iter()
                .filter(|source| source
                    .url
                    .contains("dell-technologies-delivers-second-quarter-fiscal-2027"))
                .count(),
            1
        );
        // 重放线上那条 add_source 不会产生第二条同 url 的来源。
        let mut replayed = map.clone();
        let source = ai_chip
            .sources
            .iter()
            .find(|source| source.url == avgo_source_url)
            .cloned()
            .unwrap();
        apply(
            &mut replayed,
            &edit("ai-chip", EditOp::AddSource { source }),
        )
        .unwrap();
        assert_eq!(
            replayed.industry("ai-chip").unwrap().sources.len(),
            ai_chip.sources.len()
        );
    }

    #[test]
    fn old_add_watch_payload_without_as_of_still_replays() {
        let op: EditOp = serde_json::from_str(
            r#"{"kind":"add_watch","watch":{"what":"w","why":"y","cadence":"c"}}"#,
        )
        .expect("旧日志可读");
        let mut map = base_map();
        apply(&mut map, &edit("storage", op)).expect("旧日志可重放");
        let watch = map.industry("storage").unwrap().core_watch.last().unwrap();
        assert_eq!(watch.what, "w");
        assert_eq!(watch.as_of, "");
    }

    #[test]
    fn set_watch_replaces_by_what_and_rejects_unknown_or_duplicate() {
        let mut map = base_map();
        let original = CoreWatch {
            what: " 测试关注点 ".into(),
            why: "上一季读数".into(),
            cadence: "季度".into(),
            as_of: "2026-08-26".into(),
        };
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::AddWatch {
                    watch: original.clone(),
                },
            ),
        )
        .expect("新增关注点");
        let updated = CoreWatch {
            what: "测试关注点新版".into(),
            why: "新一季读数".into(),
            cadence: "月度".into(),
            as_of: "2026-09".into(),
        };
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetWatch {
                    what: "测试关注点".into(),
                    watch: updated.clone(),
                },
            ),
        )
        .expect("按去空白后的标题定位并整条替换");
        assert_eq!(
            map.industry("storage").unwrap().core_watch.last(),
            Some(&updated)
        );
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetWatch {
                    what: " 测试关注点新版 ".into(),
                    watch: updated.clone(),
                },
            ),
        )
        .expect("保留自己的标题不算重名");
        let before = map.clone();
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetWatch {
                        what: "不存在的关注点".into(),
                        watch: updated.clone(),
                    }
                )
            ),
            Err(ApplyError::UnknownWatch("不存在的关注点".into()))
        );
        let duplicate = CoreWatch {
            what: " 测试关注点新版 ".into(),
            ..updated.clone()
        };
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::AddWatch {
                        watch: duplicate.clone()
                    }
                )
            ),
            Err(ApplyError::DuplicateWatch(duplicate.what))
        );
        let invalid = CoreWatch {
            as_of: "not-a-date".into(),
            ..original.clone()
        };
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::AddWatch {
                        watch: invalid.clone()
                    }
                )
            ),
            Err(ApplyError::InvalidDate("not-a-date".into()))
        );
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetWatch {
                        what: updated.what.clone(),
                        watch: invalid,
                    }
                )
            ),
            Err(ApplyError::InvalidDate("not-a-date".into()))
        );
        assert_eq!(map, before, "拒绝的操作不能改动已有内容");

        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::AddWatch {
                    watch: original.clone(),
                },
            ),
        )
        .expect("另一条不同标题的关注点");
        let before = map.clone();
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetWatch {
                        what: original.what,
                        watch: updated.clone(),
                    }
                )
            ),
            Err(ApplyError::DuplicateWatch(updated.what))
        );
        assert_eq!(map, before);
    }

    #[test]
    fn brief_is_absent_in_base_and_round_trips_through_set_and_clear() {
        let mut map = base_map();
        assert!(
            map.industries
                .iter()
                .all(|industry| industry.brief.is_none())
        );
        assert_eq!(
            serde_json::from_str::<IndustryBrief>("{}").unwrap(),
            IndustryBrief::default()
        );
        let brief = IndustryBrief {
            question: " 现在研究什么？ ".into(),
            body: " 新一季带来了变化。 ".into(),
            next: vec![
                " 下季财报验证 ".into(),
                "  ".into(),
                "".into(),
                " 月度出货确认 ".into(),
            ],
            as_of: " 2026-08-26 ".into(),
        };
        let before = map.clone();
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetBrief {
                        brief: IndustryBrief {
                            question: " \t ".into(),
                            ..brief.clone()
                        },
                    }
                )
            ),
            Err(ApplyError::EmptyBrief)
        );
        for date in ["", " ", "not-a-date"] {
            assert_eq!(
                apply(
                    &mut map,
                    &edit(
                        "storage",
                        EditOp::SetBrief {
                            brief: IndustryBrief {
                                as_of: date.into(),
                                ..brief.clone()
                            },
                        }
                    )
                ),
                Err(ApplyError::InvalidDate(date.into()))
            );
        }
        assert_eq!(map, before);
        apply(&mut map, &edit("storage", EditOp::SetBrief { brief })).expect("写入简报");
        assert_eq!(
            map.industry("storage").unwrap().brief,
            Some(IndustryBrief {
                question: "现在研究什么？".into(),
                body: "新一季带来了变化。".into(),
                next: vec!["下季财报验证".into(), "月度出货确认".into()],
                as_of: "2026-08-26".into(),
            })
        );
        let replacement = IndustryBrief {
            question: "下一个问题？".into(),
            as_of: "2026-09".into(),
            ..Default::default()
        };
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetBrief {
                    brief: replacement.clone(),
                },
            ),
        )
        .expect("整体替换简报");
        assert_eq!(map.industry("storage").unwrap().brief, Some(replacement));
        for _ in 0..2 {
            apply(&mut map, &edit("storage", EditOp::ClearBrief)).expect("清空可重复");
            let industry = map.industry("storage").unwrap();
            assert!(industry.brief.is_none());
            assert_eq!(
                serde_json::to_value(industry).unwrap().get("brief"),
                Some(&Value::Null)
            );
        }
    }

    #[test]
    fn content_as_of_takes_the_newest_parsable_date_across_dated_fields() {
        let mut industry: Industry = serde_json::from_value(serde_json::json!({
            "id": "dated", "name": "测试", "parent": "root",
            "upstream_signals": [{"symbol": "TEST", "latest_as_of": "2026-06"}],
            "core_watch": [{"what": "关注点", "as_of": "2026-08-26"}],
            "sources": [{"house": "机构", "title": "材料", "date": "garbage"}],
            "ai_valuation_logic": {"key_variables": [
                {"as_of": ""}, {"as_of": 20260901}, {}, null
            ]},
            "brief": {"as_of": ""}
        }))
        .unwrap();
        assert_eq!(industry.content_as_of().as_deref(), Some("2026-08-26"));
        industry.core_watch[0].as_of.clear();
        assert_eq!(industry.content_as_of().as_deref(), Some("2026-06"));
        industry.upstream_signals[0].latest_as_of.clear();
        assert_eq!(industry.content_as_of(), None);
        for candidate in 0..5 {
            let mut dated = industry.clone();
            let field = match candidate {
                0 => &mut dated.brief.as_mut().unwrap().as_of,
                1 => &mut dated.upstream_signals[0].latest_as_of,
                2 => &mut dated.core_watch[0].as_of,
                3 => &mut dated.sources[0].date,
                _ => {
                    dated.ai_valuation_logic.key_variables[0]["as_of"] =
                        Value::String("2026-08-26".into());
                    assert_eq!(dated.content_as_of().as_deref(), Some("2026-08-26"));
                    continue;
                }
            };
            *field = "2026-08-26".into();
            assert_eq!(dated.content_as_of().as_deref(), Some("2026-08-26"));
        }
        let mut map = base_map();
        map.industries = vec![industry.clone()];
        map.generated_at = "2099-12-31".into();
        apply(
            &mut map,
            &edit(
                "dated",
                EditOp::SetField {
                    field: "one_liner".into(),
                    value: "文字更新".into(),
                },
            ),
        )
        .unwrap();
        assert_eq!(map.content_as_of(), None, "底稿和编辑时钟都不是事实截至日");

        industry.core_watch[0].as_of = "2026-08".into();
        industry.upstream_signals[0].latest_as_of = "2026-08-01".into();
        assert_eq!(industry.content_as_of().as_deref(), Some("2026-08-01"));
        std::mem::swap(
            &mut industry.core_watch[0].as_of,
            &mut industry.upstream_signals[0].latest_as_of,
        );
        assert_eq!(industry.content_as_of().as_deref(), Some("2026-08-01"));
        let mut month = industry.clone();
        month.core_watch[0].as_of.clear();
        map.industries = vec![industry, month];
        assert_eq!(map.content_as_of().as_deref(), Some("2026-08-01"));
        map.industries.reverse();
        assert_eq!(map.content_as_of().as_deref(), Some("2026-08-01"));
        map.industries[0].brief.as_mut().unwrap().as_of = "2026-09".into();
        assert_eq!(map.content_as_of().as_deref(), Some("2026-09"));
        map.industries.clear();
        assert_eq!(map.content_as_of(), None);
    }

    #[test]
    fn parse_as_of_accepts_day_and_month_only() {
        assert_eq!(
            parse_as_of("2026-08-26"),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 26)
        );
        assert_eq!(
            parse_as_of(" 2026-06 "),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1)
        );
        assert_eq!(
            parse_as_of("2024-02-29"),
            chrono::NaiveDate::from_ymd_opt(2024, 2, 29)
        );
        for invalid in [
            "2026/08/26",
            "2026-13",
            "",
            " ",
            "2026-02-29",
            "2026-6",
            "2026",
            "2026-08-26T00:00:00Z",
        ] {
            assert_eq!(parse_as_of(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn add_source_is_idempotent_by_url() {
        let mut map = base_map();
        map.industry_mut("storage").unwrap().sources.clear();
        let source = IndustrySource {
            house: "测试机构".into(),
            title: "原始材料".into(),
            date: "2026-08".into(),
            url: "https://example.com/industry-source".into(),
            takeaway: "读数".into(),
        };
        let add = edit(
            "storage",
            EditOp::AddSource {
                source: source.clone(),
            },
        );
        apply(&mut map, &add).expect("首次新增");
        apply(&mut map, &add).expect("重复重放仍成功");
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::AddSource {
                    source: IndustrySource {
                        title: "同 URL 的不同标题".into(),
                        ..source.clone()
                    },
                },
            ),
        )
        .expect("按 URL 去重，不覆盖底稿");
        assert_eq!(
            map.industry("storage").unwrap().sources,
            vec![source.clone()]
        );
        for _ in 0..2 {
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::AddSource {
                        source: IndustrySource {
                            url: String::new(),
                            ..source.clone()
                        },
                    },
                ),
            )
            .expect("空 URL 不去重");
        }
        assert_eq!(map.industry("storage").unwrap().sources.len(), 3);
        let dir = tempfile::tempdir().unwrap();
        append(dir.path(), add.clone()).expect("首次写入日志");
        append(dir.path(), add).expect("重复也写入日志");
        let (map, applied) = load(dir.path());
        assert_eq!(applied.len(), 2, "幂等重放不应被视作失败而跳过");
        assert_eq!(
            map.industry("storage")
                .unwrap()
                .sources
                .iter()
                .filter(|item| item.url == source.url)
                .count(),
            1
        );
    }

    #[test]
    fn new_ops_round_trip_through_the_edit_log() {
        let dir = tempfile::tempdir().unwrap();
        let base = base_map();
        let industry = base
            .industries
            .iter()
            .find(|industry| !industry.core_watch.is_empty())
            .unwrap();
        let mut watch = industry.core_watch[0].clone();
        let what = watch.what.clone();
        watch.why = "新季度的读数".into();
        watch.as_of = "2026-08-26".into();
        let ops = [
            EditOp::SetBrief {
                brief: IndustryBrief {
                    question: "本季要验证什么？".into(),
                    as_of: "2026-08".into(),
                    ..Default::default()
                },
            },
            EditOp::SetWatch {
                what,
                watch: watch.clone(),
            },
            EditOp::ClearBrief,
        ];
        for op in &ops {
            append(dir.path(), edit(&industry.id, op.clone())).expect("新操作写入日志");
        }
        let (map, applied) = load(dir.path());
        assert_eq!(
            applied.iter().map(|edit| &edit.op).collect::<Vec<_>>(),
            ops.iter().collect::<Vec<_>>()
        );
        assert_eq!(map.industry(&industry.id).unwrap().core_watch[0], watch);
        assert!(map.industry(&industry.id).unwrap().brief.is_none());
        let log = serde_json::to_value(load_log(dir.path())).unwrap();
        let kinds = log["edits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edit| edit["op"]["kind"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(kinds, ["set_brief", "set_watch", "clear_brief"]);
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
                        latest: String::new(),
                        latest_as_of: String::new(),
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
    fn upstream_latest_is_set_by_symbol_and_only_on_existing_signals() {
        let mut map = base_map();
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::AddUpstreamSignal {
                    signal: UpstreamSignal {
                        symbol: "ZZZZ".into(),
                        name: "测试上游".into(),
                        relation: "capex_source".into(),
                        why: String::new(),
                        pull: vec![],
                        cadence: String::new(),
                        latest: String::new(),
                        latest_as_of: String::new(),
                    },
                },
            ),
        )
        .expect("add signal");
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetUpstreamLatest {
                    symbol: "ZZZZ".into(),
                    latest: " Q2：收入 +X%，capex 指引上调 ".into(),
                    as_of: "2026-08-01".into(),
                },
            ),
        )
        .expect("set latest");
        let signal = map
            .industries
            .iter()
            .find(|i| i.id == "storage")
            .unwrap()
            .upstream_signals
            .iter()
            .find(|s| s.symbol == "ZZZZ")
            .unwrap();
        assert_eq!(signal.latest, "Q2：收入 +X%，capex 指引上调");
        assert_eq!(signal.latest_as_of, "2026-08-01");
        assert_eq!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetUpstreamLatest {
                        symbol: "YYYY".into(),
                        latest: "x".into(),
                        as_of: String::new(),
                    },
                ),
            ),
            Err(ApplyError::UnknownSignal("YYYY".into()))
        );
        assert_eq!(
            EditOp::SetUpstreamLatest {
                symbol: "ZZZZ".into(),
                latest: "x".into(),
                as_of: "2026-08-01".into(),
            }
            .summary(),
            "更新上游动作 ZZZZ（截至 2026-08-01）"
        );
    }

    #[test]
    fn v3_valuation_fields_subtypes_and_member_moves_apply_in_order() {
        let mut map = base_map();
        let storage = map.industries.iter().find(|i| i.id == "storage").unwrap();
        assert!(
            !storage.valuation.subtypes.is_empty(),
            "the base map ships HOne V3 subtypes"
        );
        assert!(!map.methodology.execution_rules.is_empty());
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetValuationField {
                    field: "logic.state_note".into(),
                    value: " 量增价平阶段 ".into(),
                },
            ),
        )
        .expect("text field");
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetValuationList {
                    field: "anchor.forbidden".into(),
                    items: vec!["峰值季度 EPS×4".into(), " ".into(), "DCF".into()],
                },
            ),
        )
        .expect("list field");
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::UpsertSubtype {
                    subtype: Subtype {
                        id: "test-sub".into(),
                        name: "测试子类型".into(),
                        members: vec!["sndk".into()],
                        primary: "FY+2 Forward PE".into(),
                        ..Default::default()
                    },
                },
            ),
        )
        .expect("upsert subtype");
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::SetMemberSubtype {
                    symbol: "wdc".into(),
                    subtype: "test-sub".into(),
                },
            ),
        )
        .expect("move member");
        let storage = map.industries.iter().find(|i| i.id == "storage").unwrap();
        assert_eq!(storage.valuation.logic.state_note, "量增价平阶段");
        assert_eq!(
            storage.valuation.anchor.forbidden,
            vec!["峰值季度 EPS×4", "DCF"]
        );
        let sub = storage.valuation.subtype_of("SNDK").expect("SNDK moved");
        assert_eq!(sub.id, "test-sub");
        assert!(sub.members.contains(&"WDC".to_string()));
        assert_eq!(
            storage
                .valuation
                .subtypes
                .iter()
                .filter(|s| s.members.contains(&"SNDK".to_string()))
                .count(),
            1,
            "a member lives in exactly one subtype"
        );
        assert!(matches!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::SetMemberSubtype {
                        symbol: "MU".into(),
                        subtype: "nope".into(),
                    },
                ),
            ),
            Err(ApplyError::UnknownSubtype(_))
        ));
        assert!(matches!(
            apply(
                &mut map,
                &edit(
                    "storage",
                    EditOp::UpsertSubtype {
                        subtype: Subtype {
                            id: "Bad Id".into(),
                            ..Default::default()
                        },
                    },
                ),
            ),
            Err(ApplyError::InvalidSubtypeId(_))
        ));
        apply(
            &mut map,
            &edit(
                "storage",
                EditOp::RemoveSubtype {
                    id: "test-sub".into(),
                },
            ),
        )
        .expect("remove subtype");
        assert_eq!(
            EditOp::SetMemberSubtype {
                symbol: "SNDK".into(),
                subtype: "nand-essd".into()
            }
            .summary(),
            "把 SNDK 归到子类型 nand-essd"
        );
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
            latest: String::new(),
            latest_as_of: String::new(),
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
                as_of: String::new(),
            },
        }
        .summary();
        assert!(summary.chars().count() <= 30, "{summary}");
        assert!(summary.ends_with("…」"));
        let summary = EditOp::SetWatch {
            what: "旧关注点".into(),
            watch: CoreWatch {
                what: long.into(),
                why: String::new(),
                cadence: String::new(),
                as_of: "2026-08-26".into(),
            },
        }
        .summary();
        assert_eq!(summary, format!("改写关注点「{}」", truncate(long, 18)));
        assert!(summary.chars().count() <= 30, "{summary}");
        let summary = EditOp::SetBrief {
            brief: IndustryBrief {
                as_of: "2026-08-26".into(),
                ..Default::default()
            },
        }
        .summary();
        assert_eq!(summary, "写入行业简报（截至 2026-08-26）");
        assert!(summary.chars().count() <= 30, "{summary}");
        assert_eq!(EditOp::ClearBrief.summary(), "清空行业简报");
    }
}
