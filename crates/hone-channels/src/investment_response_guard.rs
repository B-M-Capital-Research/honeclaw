use std::collections::HashSet;
use std::sync::Arc;

use hone_core::ActorIdentity;
use hone_core::agent::ToolCallMade;
use hone_llm::Message;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::HoneBotCore;
use crate::agent_session::AgentTurnOrigin;

const EVIDENCE_ITEM_CHAR_LIMIT: usize = 6_000;
const CONTRACT_FAILURE_MESSAGE: &str =
    "这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。";
const CURRENT_PRICE_INTENT_MARKERS: &[&str] = &[
    "多少钱",
    "股价",
    "价格",
    "现价",
    "当前价",
    "最新价",
    "实时价",
    "当前报价",
    "最新报价",
    "实时报价",
    "报价",
    "行情",
    "price",
    "quote",
    "last price",
    "current price",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeepAnalysisKind {
    None,
    Equity,
    Fund,
    Crypto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvestmentResponseContract {
    pub entities: Vec<ResolvedSecurityEntity>,
    pub deep_analysis: DeepAnalysisKind,
    pub deep_comparison: bool,
    pub requires_verified_price: bool,
    pub needs_outlook_evidence: bool,
    pub comparison: bool,
    pub origin: AgentTurnOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedSecurityEntity {
    pub mention: String,
    pub symbol: String,
    pub name: String,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub asset_type: Option<String>,
    pub profile_verified: bool,
    pub verified_price: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityMention {
    mention: String,
    search_query: String,
    explicit_symbol: Option<String>,
    tentative_symbol: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityCandidate {
    symbol: String,
    name: String,
    exchange: Option<String>,
    currency: Option<String>,
    asset_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EntityMatch {
    Resolved(ResolvedSecurityEntity),
    Ambiguous(Vec<EntityCandidate>),
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetEvidenceRoute {
    Equity,
    Fund,
    Crypto,
}

#[derive(Debug, Deserialize)]
struct EntityExtractionPayload {
    entities: Vec<EntityExtractionItem>,
}

#[derive(Debug, Deserialize)]
struct EntityExtractionItem {
    mention: String,
    search_query: String,
    #[serde(default)]
    explicit_symbol: Option<String>,
}

impl InvestmentResponseContract {
    fn symbols(&self) -> Vec<&str> {
        self.entities
            .iter()
            .map(|entity| entity.symbol.as_str())
            .collect()
    }

    pub(crate) fn enforcement_block(&self) -> String {
        let entity_map = self
            .entities
            .iter()
            .map(|entity| format!("{} → {} ({})", entity.mention, entity.name, entity.symbol))
            .collect::<Vec<_>>()
            .join("；");
        if self.origin != AgentTurnOrigin::Interactive {
            return format!(
                "\n\n【本轮代码级证券实体与数据门禁】\n已确认实体：{entity_map}。任务来源为结构化 {:?}，不得从任务 envelope、repeat 配置或报告缩写推断其它证券。价格、估值、财务、新闻和日期数字只能使用本轮同标的证据。",
                self.origin
            );
        }
        if self.comparison {
            if !self.deep_comparison {
                return format!(
                    "\n\n【本轮代码级多证券行情门禁】\n已确认实体：{entity_map}。必须逐一覆盖 {}，先用独立一行写“数据时间：北京时间 YYYY-MM-DD”，再为每个标的单独一行使用“现价”或“当前价”写出本轮同 symbol 价格；不得用一个标的的数据代替另一个标的。",
                    self.symbols().join("、")
                );
            }
            return format!(
                "\n\n【本轮代码级多证券比较门禁】\n已确认实体：{entity_map}。必须逐一覆盖 {}，每个标的的数值都只能来自本轮同 symbol 证据；不得用一个标的的数据代替另一个标的。公司使用公司概况与财务证据，ETF/基金使用基金概况与持仓证据，加密资产使用同代码行情与网络/代币口径，不得混用。回答先用独立一行写“数据时间：北京时间 YYYY-MM-DD”，再给比较结论，并严格使用独立 Markdown 标题 `### SYMBOL` 为每个标的建立小节；每个标的小节必须写出本轮已核验同代码现价、适配资产类型的事实与估值/风险差异，最后给动作条件与证伪条件。",
                self.symbols().join("、")
            );
        }
        match self.deep_analysis {
            DeepAnalysisKind::None => {
                let price_requirement = if self.requires_verified_price {
                    "回答必须使用“现价”或“当前价”明确写出本轮已核验同代码价格。"
                } else {
                    ""
                };
                format!(
                    "\n\n【本轮代码级证券数据门禁】\n已确认实体：{entity_map}。价格、估值、财务、新闻和日期数字只能使用本轮同标的证据；不得从历史对话或模型记忆补数。{price_requirement}"
                )
            }
            DeepAnalysisKind::Fund => format!(
                "\n\n【本轮代码级投研路由：ETF / 基金深度分析，必须完整执行】\n已确认实体：{entity_map}。该标的是 ETF 或基金，不得套用单一公司的商业模式、利润表或 DCF 口径。最终答案先用独立一行写“数据时间：北京时间 YYYY-MM-DD”，再按以下九个编号章节逐项回答，不得合并或省略：\n1. 结论（必须写出本轮已核验同代码现价）\n2. 基金目标、策略与跟踪对象\n3. 持仓、集中度与主要暴露\n4. 地域、行业与货币风险\n5. 流动性、规模与交易特征\n6. 费用、跟踪误差与底层资产估值口径\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议（买、等、减、卖、观察之一，并给触发条件）\n明确区分本轮已核验事实、推断和动作。持仓、费用或规模证据为空时必须逐项写“本轮未核验”，不得从历史对话或模型记忆补数。"
            ),
            DeepAnalysisKind::Equity => format!(
                "\n\n【本轮代码级投研路由：单股深度分析，必须完整执行】\n已确认实体：{entity_map}。这不是简短行情问答。最终答案先用独立一行写“数据时间：北京时间 YYYY-MM-DD”，再按以下九个编号章节逐项回答，不得合并或省略：\n1. 结论（必须写出本轮已核验同代码现价）\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量\n6. 估值（至少两种适配方法或“倍数法 + 情景法”，写清假设）\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议（买、等、减、卖、观察之一，并给触发条件）\n明确区分本轮已核验事实、推断和动作。证据没有的数字明确写“本轮未核验”，不得从历史对话或模型记忆补数。"
            ),
            DeepAnalysisKind::Crypto => format!(
                "\n\n【本轮代码级投研路由：加密资产深度分析，必须完整执行】\n已确认实体：{entity_map}。该标的是加密资产，不得套用公司利润表、公司财报日历、ETF 持仓或单一公司 DCF 口径。最终答案先用独立一行写“数据时间：北京时间 YYYY-MM-DD”，再按以下九个编号章节逐项回答，不得合并或省略：\n1. 结论（必须写出本轮已核验同代码现价）\n2. 资产、网络与核心用途\n3. 供给机制、代币经济与集中度\n4. 采用、流动性与市场结构\n5. 链上、网络与生态数据\n6. 估值框架与关键假设\n7. Bull / Bear / Base Case\n8. 催化剂、监管与风险、证伪条件\n9. 动作建议（买、等、减、卖、观察之一，并给触发条件）\n明确区分本轮已核验事实、推断和动作。链上、供给或生态数据未提供时必须逐项写“本轮未核验”，不得从模型记忆补数。"
            ),
        }
    }

    pub(crate) fn retry_block(&self, missing: &[&'static str]) -> String {
        if self.comparison {
            if !self.deep_comparison {
                return format!(
                    "\n\n【上一版多标的行情草稿已被代码级完整性检查拒绝】\n缺失或不合格项：{}。重新生成并逐一覆盖 {}，每个标的单独一行写出本轮同代码现价，并标明数据时间；不得解释检查过程。",
                    missing.join("、"),
                    self.symbols().join("、")
                );
            }
            return format!(
                "\n\n【上一版多标的比较草稿已被代码级完整性检查拒绝】\n缺失或不合格项：{}。重新生成完整比较，必须逐一覆盖 {}，标明数据时间；使用独立 `### SYMBOL` 小节，在对应小节写出本轮同代码现价与适配资产类型的证据，并区分事实、推断、动作和证伪条件；不得解释检查过程。",
                missing.join("、"),
                self.symbols().join("、")
            );
        }
        if self.deep_analysis == DeepAnalysisKind::Fund {
            return format!(
                "\n\n【上一版 ETF / 基金草稿已被代码级完整性检查拒绝】\n缺失或不合格章节：{}。重新生成完整最终答案，开头必须独立写一行“数据时间：北京时间 YYYY-MM-DD”，严格使用 ETF / 基金九个编号章节，并在第 1 节写出本轮已核验同代码现价；不得解释检查过程，不得虚构持仓、费用、规模或公司财务，不得用追问持仓成本代替动作建议。",
                missing.join("、")
            );
        }
        if self.deep_analysis == DeepAnalysisKind::Crypto {
            return format!(
                "\n\n【上一版加密资产草稿已被代码级完整性检查拒绝】\n缺失或不合格章节：{}。重新生成完整最终答案，开头必须独立写一行“数据时间：北京时间 YYYY-MM-DD”，严格使用加密资产九个编号章节，并在第 1 节写出本轮已核验同代码现价；不得解释检查过程，不得调用或引用公司财务、公司财报日历或 ETF 持仓。",
                missing.join("、")
            );
        }
        if self.deep_analysis == DeepAnalysisKind::None {
            if !self.requires_verified_price {
                return format!(
                    "\n\n【上一版证券草稿已被代码级数据检查拒绝】\n缺失或不合格项：{}。重新回答时严格使用本轮已核验实体与资产类型；ETF / 基金不得调用或引用公司财务与公司财报日历；不得解释检查过程。",
                    missing.join("、")
                );
            }
            return format!(
                "\n\n【上一版证券行情草稿已被代码级数据检查拒绝】\n缺失或不合格项：{}。重新回答时使用“现价”或“当前价”明确写出本轮已核验同代码价格，并标明数据时间；不得解释检查过程。",
                missing.join("、")
            );
        }
        format!(
            "\n\n【上一版草稿已被代码级完整性检查拒绝】\n缺失或不合格章节：{}。重新生成完整最终答案，开头必须独立写一行“数据时间：北京时间 YYYY-MM-DD”，严格使用九个编号章节，并在第 1 节写出本轮已核验同代码现价；不得解释检查过程，不得用追问持仓成本代替动作建议。",
            missing.join("、")
        )
    }
}

pub(crate) fn contract_failure_message() -> &'static str {
    CONTRACT_FAILURE_MESSAGE
}

pub(crate) fn forbidden_investment_tool_calls(
    contract: &InvestmentResponseContract,
    calls: &[ToolCallMade],
) -> Vec<&'static str> {
    let mut violations = Vec::new();
    for entity in &contract.entities {
        let forbidden_types: &[&str] = if entity_is_fund(entity) {
            &["financials", "earnings_calendar"]
        } else if entity_is_crypto(entity) {
            &["financials", "earnings_calendar", "etf_holdings"]
        } else {
            continue;
        };
        let violated = calls.iter().any(|call| {
            call.name.to_ascii_lowercase().contains("data_fetch")
                && call
                    .arguments
                    .get("data_type")
                    .and_then(Value::as_str)
                    .is_some_and(|data_type| {
                        forbidden_types
                            .iter()
                            .any(|forbidden| data_type.eq_ignore_ascii_case(forbidden))
                    })
                && tool_call_targets_entity(&call.arguments, &entity.symbol)
        });
        let label = if entity_is_crypto(entity) {
            "加密资产不得调用公司财务、公司财报日历或 ETF 持仓"
        } else {
            "ETF / 基金不得调用公司财务或公司财报日历"
        };
        if violated && !violations.contains(&label) {
            violations.push(label);
        }
    }
    violations
}

fn tool_call_targets_entity(arguments: &Value, symbol: &str) -> bool {
    let target = arguments
        .get("ticker")
        .or_else(|| arguments.get("symbol"))
        .and_then(Value::as_str)
        .unwrap_or("");
    target.is_empty()
        || target
            .split([',', ';', ' ', '、'])
            .any(|candidate| candidate.eq_ignore_ascii_case(symbol))
}

fn response_intent(input: &str) -> (bool, bool) {
    let normalized = input.to_ascii_lowercase();
    let deep = [
        "分析",
        "研究",
        "怎么看",
        "怎么样",
        "值不值得",
        "能不能买",
        "能否买",
        "起飞",
        "前景",
        "估值",
        "目标价",
        "未来",
        "财报",
        "业绩",
        "基本面",
        "财务",
        "营收",
        "利润",
        "现金流",
        "持仓",
        "成分股",
        "集中度",
        "费率",
        "跟踪误差",
        "holdings",
        "expense ratio",
        "cash flow",
        "比较",
        "对比",
        "compare",
        "versus",
        " vs ",
        "哪个好",
        "哪一个好",
        "哪个更好",
        "谁更好",
        "二选一",
        "选哪个",
        "bull",
        "bear",
        "case",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
        || Regex::new(r"(?i)\bq[1-4]\b")
            .expect("quarter regex")
            .is_match(input);
    let needs_outlook_evidence = deep
        && [
            "起飞", "前景", "未来", "财报", "业绩", "催化", "q1", "q2", "q3", "q4",
        ]
        .iter()
        .any(|keyword| normalized.contains(keyword));
    (deep, needs_outlook_evidence)
}

fn response_requires_verified_price(input: &str, deep: bool, comparison: bool) -> bool {
    let normalized = input.to_ascii_lowercase();
    deep || comparison || has_current_price_intent(&normalized)
}

fn has_current_price_intent(normalized_input: &str) -> bool {
    CURRENT_PRICE_INTENT_MARKERS
        .iter()
        .any(|marker| normalized_input.contains(marker))
}

fn asset_evidence_route(profile: &Value, symbol: &str) -> Option<AssetEvidenceRoute> {
    profile_asset_route(profile, symbol)
}

fn profile_asset_route(value: &Value, symbol: &str) -> Option<AssetEvidenceRoute> {
    match value {
        Value::Object(map) => {
            let object_symbol = map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str);
            let belongs_to_symbol = object_symbol
                .map(|candidate| candidate.eq_ignore_ascii_case(symbol))
                .unwrap_or(true);
            if object_symbol.is_some() && belongs_to_symbol {
                if map.get("isEtf").and_then(Value::as_bool) == Some(true)
                    || map.get("isFund").and_then(Value::as_bool) == Some(true)
                {
                    return Some(AssetEvidenceRoute::Fund);
                }
                if let Some(route) = map
                    .get("type")
                    .or_else(|| map.get("assetType"))
                    .and_then(Value::as_str)
                    .and_then(asset_route_from_label)
                {
                    return Some(route);
                }
                if map.get("isEtf").and_then(Value::as_bool) == Some(false)
                    && map.get("isFund").and_then(Value::as_bool) == Some(false)
                {
                    return Some(AssetEvidenceRoute::Equity);
                }
            }
            map.values()
                .find_map(|child| profile_asset_route(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| profile_asset_route(child, symbol)),
        _ => None,
    }
}

fn asset_route_from_label(label: &str) -> Option<AssetEvidenceRoute> {
    let normalized = label.to_ascii_lowercase();
    if normalized.contains("crypto") || normalized.contains("digital asset") || normalized == "ccc"
    {
        return Some(AssetEvidenceRoute::Crypto);
    }
    if normalized.contains("etf") || normalized.contains("fund") {
        Some(AssetEvidenceRoute::Fund)
    } else if normalized.contains("stock")
        || normalized.contains("equity")
        || normalized.contains("company")
    {
        Some(AssetEvidenceRoute::Equity)
    } else {
        None
    }
}

fn set_verified_asset_type(entity: &mut ResolvedSecurityEntity, route: AssetEvidenceRoute) {
    entity.asset_type = Some(
        match route {
            AssetEvidenceRoute::Equity => "equity",
            AssetEvidenceRoute::Fund => "etf_or_fund",
            AssetEvidenceRoute::Crypto => "crypto",
        }
        .to_string(),
    );
    entity.profile_verified = true;
}

fn entity_is_fund(entity: &ResolvedSecurityEntity) -> bool {
    entity
        .asset_type
        .as_deref()
        .and_then(asset_route_from_label)
        == Some(AssetEvidenceRoute::Fund)
}

fn entity_is_equity(entity: &ResolvedSecurityEntity) -> bool {
    entity
        .asset_type
        .as_deref()
        .and_then(asset_route_from_label)
        == Some(AssetEvidenceRoute::Equity)
}

fn entity_is_crypto(entity: &ResolvedSecurityEntity) -> bool {
    entity
        .asset_type
        .as_deref()
        .and_then(asset_route_from_label)
        == Some(AssetEvidenceRoute::Crypto)
}

fn should_fetch_earnings_calendar(entity: &ResolvedSecurityEntity) -> bool {
    entity.profile_verified && entity_is_equity(entity)
}

pub(crate) async fn prepare_verified_investment_turn(
    core: &Arc<HoneBotCore>,
    actor: &ActorIdentity,
    channel_target: &str,
    allow_cron: bool,
    user_input: &str,
    origin: AgentTurnOrigin,
    runtime_input: &mut String,
) -> Result<Option<InvestmentResponseContract>, String> {
    let entity_stage_ran = should_run_entity_stage(user_input, origin);
    let mentions = extract_entity_mentions(core, user_input, origin).await?;
    if mentions.is_empty() {
        if entity_stage_ran {
            runtime_input.push_str("\n\n【本轮实体解析结果】\n当前请求未识别到明确公司或证券实体；按宏观、行业或一般金融问题处理。不得从历史对话补入旧 ticker，也不得生成公司特定价格或财务数字。\n");
        }
        return Ok(None);
    }
    let registry = core.create_tool_registry(Some(actor), channel_target, allow_cron);
    let mut entities = Vec::new();
    let mut seen_symbols = HashSet::new();
    let mut unresolved_ticker_candidates = Vec::new();
    for mention in mentions {
        let search = registry
            .execute_tool(
                "data_fetch",
                json!({"data_type": "search", "query": mention.search_query}),
            )
            .await
            .map_err(|_| "证券实体查询暂时不可用，请稍后重试。".to_string())?;
        if value_has_error(&search) {
            return Err("证券实体查询暂时不可用，请稍后重试。".to_string());
        }
        match resolve_entity_match(&mention, &search) {
            EntityMatch::Resolved(entity) => {
                if seen_symbols.insert(entity.symbol.clone()) {
                    entities.push(entity);
                }
            }
            EntityMatch::Ambiguous(candidates) => {
                let choices = candidates
                    .iter()
                    .take(4)
                    .map(|c| format!("{}（{}）", c.name, c.symbol))
                    .collect::<Vec<_>>()
                    .join("、");
                return Err(format!(
                    "你提到的“{}”对应多个可能的证券实体：{}。请补充公司全名或确认 ticker。",
                    mention.mention, choices
                ));
            }
            EntityMatch::Unresolved => {
                if mention.tentative_symbol {
                    unresolved_ticker_candidates.push(mention.mention);
                    continue;
                }
                return Err(format!(
                    "我暂时无法确认你提到的“{}”对应哪家上市公司或证券。请补充公司全名或 ticker。",
                    mention.mention
                ));
            }
        }
    }
    if origin == AgentTurnOrigin::Interactive && !unresolved_ticker_candidates.is_empty() {
        return Err(format!(
            "我没有在当前证券数据中精确核验到代码 {}。请检查 ticker 是否正确，或补充公司全名。",
            unresolved_ticker_candidates.join("、")
        ));
    }
    if entities.is_empty() {
        if !unresolved_ticker_candidates.is_empty() {
            runtime_input.push_str(
                "\n\n【本轮实体核验结果】\n当前请求中的证券代码候选均未通过同代码精确核验；不得生成任何候选对应的公司特定价格、财务数字或事件结论。\n",
            );
        }
        return Ok(None);
    }
    let (deep_intent, needs_outlook_evidence) = response_intent(user_input);
    let comparison = entities.len() > 1;
    let mut contract = InvestmentResponseContract {
        deep_analysis: if origin == AgentTurnOrigin::Interactive && deep_intent && !comparison {
            DeepAnalysisKind::Equity
        } else {
            DeepAnalysisKind::None
        },
        deep_comparison: origin == AgentTurnOrigin::Interactive && deep_intent && comparison,
        requires_verified_price: origin == AgentTurnOrigin::Interactive
            && response_requires_verified_price(user_input, deep_intent, comparison),
        needs_outlook_evidence,
        comparison,
        origin,
        entities,
    };
    let symbols = contract
        .entities
        .iter()
        .map(|entity| entity.symbol.clone())
        .collect::<Vec<_>>();
    let quote_type = if symbols.len() > 1 {
        "quote_short"
    } else {
        "quote"
    };
    let quote = registry
        .execute_tool(
            "data_fetch",
            json!({"data_type": quote_type, "ticker": symbols.join(",")}),
        )
        .await
        .map_err(|_| "最新证券行情查询暂时不可用，请稍后重试。".to_string())?;
    for index in 0..contract.entities.len() {
        let symbol = &contract.entities[index].symbol;
        let Some(price) = matching_quote_price(&quote, symbol) else {
            return Err(format!(
                "{symbol} 的最新同标的行情尚未完成确认。本轮不会基于不确定价格给出投资结论。"
            ));
        };
        contract.entities[index].verified_price = Some(price.to_string());
    }

    let mut evidence = vec![("最新行情", quote)];

    // 资产类型是所有后续数据路由的先决条件，不只是深度分析的可选步骤。
    // 这里对每个 exact-symbol 实体先做 profile 核验，后面才允许选择公司财务
    // 或 ETF/基金持仓路线，避免模型在浅层问题中重新把基金当公司。
    for index in 0..contract.entities.len() {
        let symbol = contract.entities[index].symbol.clone();
        if entity_is_crypto(&contract.entities[index]) {
            set_verified_asset_type(&mut contract.entities[index], AssetEvidenceRoute::Crypto);
            evidence.push((
                "逐标的已核验加密资产类型",
                json!({
                    "symbol": symbol,
                    "name": contract.entities[index].name.clone(),
                    "exchange": contract.entities[index].exchange.clone(),
                    "asset_type": "crypto"
                }),
            ));
            continue;
        }
        let profile = registry
            .execute_tool(
                "data_fetch",
                json!({"data_type": "profile", "ticker": symbol}),
            )
            .await
            .map_err(|err| format!("{symbol} 的资产类型与基本资料核验失败：{err}"))?;
        if !has_matching_symbol_data(&profile, &symbol) {
            return Err(format!(
                "{symbol} 的同代码资产类型与基本资料本轮未能确认，已停止生成可能套用错误数据口径的回答。"
            ));
        }
        let route = asset_evidence_route(&profile, &symbol).ok_or_else(|| {
            format!(
                "{symbol} 的 profile 未返回可确认的 isEtf/isFund 或资产类型字段，已停止生成可能套用错误数据口径的分析。"
            )
        })?;
        set_verified_asset_type(&mut contract.entities[index], route);
        evidence.push(("逐标的已核验资产类型与基本资料", profile));
    }

    if contract.deep_analysis == DeepAnalysisKind::Equity {
        let symbol = contract.entities[0].symbol.clone();
        let route = if entity_is_crypto(&contract.entities[0]) {
            AssetEvidenceRoute::Crypto
        } else if entity_is_fund(&contract.entities[0]) {
            AssetEvidenceRoute::Fund
        } else {
            AssetEvidenceRoute::Equity
        };
        match route {
            AssetEvidenceRoute::Fund => {
                contract.deep_analysis = DeepAnalysisKind::Fund;
                let (holdings, news) = tokio::join!(
                    registry.execute_tool(
                        "data_fetch",
                        json!({"data_type": "etf_holdings", "ticker": symbol}),
                    ),
                    registry.execute_tool(
                        "data_fetch",
                        json!({"data_type": "news", "ticker": symbol}),
                    ),
                );
                evidence.push((
                    "ETF / 基金持仓（为空或报错时必须写本轮未核验）",
                    result_or_error_value(holdings),
                ));
                evidence.push(("ETF / 基金相关新闻", result_or_error_value(news)));
            }
            AssetEvidenceRoute::Equity => {
                let (financials, news) = tokio::join!(
                    registry.execute_tool(
                        "data_fetch",
                        json!({"data_type": "financials", "ticker": symbol}),
                    ),
                    registry.execute_tool(
                        "data_fetch",
                        json!({"data_type": "news", "ticker": symbol}),
                    ),
                );
                let financials = financials
                    .map_err(|err| format!("{symbol} 的公司年度利润表查询失败：{err}"))?;
                if !has_matching_financial_data(&financials, &symbol) {
                    let reason = if value_has_error(&financials) {
                        "公司年度利润表查询返回 provider error"
                    } else if has_nonempty_data(&financials) {
                        "公司年度利润表未返回同代码数据"
                    } else {
                        "公司年度利润表返回空数据"
                    };
                    return Err(format!(
                        "{symbol} 已核验为公司，但本轮{reason}；在财务证据补齐前不生成完整公司估值结论。"
                    ));
                }
                evidence.push(("公司年度利润表（最近四期）", financials));
                evidence.push(("公司新闻", result_or_error_value(news)));
            }
            AssetEvidenceRoute::Crypto => {
                contract.deep_analysis = DeepAnalysisKind::Crypto;
                let news = registry
                    .execute_tool("data_fetch", json!({"data_type": "news", "ticker": symbol}))
                    .await;
                evidence.push(("加密资产相关新闻", result_or_error_value(news)));
            }
        }
    }
    if contract.deep_comparison {
        for index in 0..contract.entities.len() {
            let symbol = contract.entities[index].symbol.clone();
            let route = if entity_is_crypto(&contract.entities[index]) {
                AssetEvidenceRoute::Crypto
            } else if entity_is_fund(&contract.entities[index]) {
                AssetEvidenceRoute::Fund
            } else {
                AssetEvidenceRoute::Equity
            };
            match route {
                AssetEvidenceRoute::Fund => {
                    let holdings = registry
                        .execute_tool(
                            "data_fetch",
                            json!({"data_type": "etf_holdings", "ticker": symbol}),
                        )
                        .await;
                    evidence.push((
                        "逐标的 ETF / 基金持仓（为空或报错时必须写本轮未核验）",
                        result_or_error_value(holdings),
                    ));
                }
                AssetEvidenceRoute::Equity => {
                    let financials = registry
                        .execute_tool(
                            "data_fetch",
                            json!({"data_type": "financials", "ticker": symbol}),
                        )
                        .await
                        .map_err(|err| format!("{symbol} 的公司年度利润表查询失败：{err}"))?;
                    if !has_matching_financial_data(&financials, &symbol) {
                        return Err(format!(
                            "{symbol} 已核验为公司，但本轮公司年度利润表为空、查询失败或未返回同代码数据，暂不能进行可靠的多标的公司估值比较。"
                        ));
                    }
                    evidence.push(("逐标的公司年度利润表（最近四期）", financials));
                }
                AssetEvidenceRoute::Crypto => {
                    let news = registry
                        .execute_tool("data_fetch", json!({"data_type": "news", "ticker": symbol}))
                        .await;
                    evidence.push(("逐标的加密资产相关新闻", result_or_error_value(news)));
                }
            }
        }
    }
    if contract.needs_outlook_evidence && contract.entities.len() <= 5 {
        let from = hone_core::beijing_now().date_naive();
        let to = from + chrono::Duration::days(120);
        for entity in &contract.entities {
            if !should_fetch_earnings_calendar(entity) {
                continue;
            }
            let symbol = &entity.symbol;
            let calendar = registry.execute_tool("data_fetch", json!({"data_type": "earnings_calendar", "ticker": symbol, "from": from.format("%Y-%m-%d").to_string(), "to": to.format("%Y-%m-%d").to_string()})).await;
            evidence.push((
                "未来 120 天财报日历（仅当前标的）",
                matching_symbol_objects_or_error(&result_or_error_value(calendar), symbol),
            ));
        }
    }

    runtime_input.push_str(&contract.enforcement_block());
    runtime_input.push_str("\n\n【本轮已核验数据证据】\n");
    for (label, value) in evidence {
        runtime_input.push_str(&format!(
            "- {label}：{}\n",
            truncate_chars(&value.to_string(), EVIDENCE_ITEM_CHAR_LIMIT)
        ));
    }
    runtime_input
        .push_str("以上证据是本轮运行时注入，不得向用户暴露工具名、原始 JSON 或内部检查流程。\n");
    Ok(Some(contract))
}

pub(crate) fn missing_deep_single_stock_sections(content: &str) -> Vec<&'static str> {
    let text = content.to_ascii_lowercase();
    let mut missing = Vec::new();
    require_any(&text, &["结论"], "1. 结论", &mut missing);
    require_any(
        &text,
        &["靠什么赚钱", "商业模式", "公司是什么"],
        "2. 公司与商业模式",
        &mut missing,
    );
    require_any(
        &text,
        &["护城河", "竞争壁垒", "壁垒"],
        "3. 护城河与壁垒",
        &mut missing,
    );
    require_any(
        &text,
        &["行业位置", "关键对手", "竞争对手"],
        "4. 行业位置与对手",
        &mut missing,
    );
    require_any(
        &text,
        &["财务质量", "毛利率", "自由现金流"],
        "5. 财务质量",
        &mut missing,
    );
    require_any(&text, &["估值"], "6. 估值", &mut missing);
    if !(text.contains("bull") && text.contains("bear") && text.contains("base")) {
        missing.push("7. Bull / Bear / Base Case");
    }
    if !(text.contains("催化") && text.contains("风险") && text.contains("证伪")) {
        missing.push("8. 催化、风险与证伪");
    }
    require_any(
        &text,
        &["动作建议", "行动建议", "操作建议"],
        "9. 动作建议",
        &mut missing,
    );
    for (number, label) in [
        (1, "1. 结论"),
        (2, "2. 公司与商业模式"),
        (3, "3. 护城河与壁垒"),
        (4, "4. 行业位置与对手"),
        (5, "5. 财务质量"),
        (6, "6. 估值"),
        (7, "7. Bull / Bear / Base Case"),
        (8, "8. 催化、风险与证伪"),
        (9, "9. 动作建议"),
    ] {
        if !has_numbered_section(content, number) && !missing.contains(&label) {
            missing.push(label);
        }
    }
    for (number, label) in [
        (2, "2. 公司与商业模式"),
        (3, "3. 护城河与壁垒"),
        (4, "4. 行业位置与对手"),
        (5, "5. 财务质量"),
        (6, "6. 估值"),
        (7, "7. Bull / Bear / Base Case"),
        (8, "8. 催化、风险与证伪"),
        (9, "9. 动作建议"),
    ] {
        if !numbered_section_has_substance(content, number) {
            push_missing(&mut missing, label);
        }
    }
    let section_2 = numbered_section(content, 2)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_3 = numbered_section(content, 3)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_4 = numbered_section(content, 4)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_5 = numbered_section(content, 5)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_6 = numbered_section(content, 6)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_7 = numbered_section(content, 7)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_8 = numbered_section(content, 8)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_9 = numbered_section(content, 9)
        .unwrap_or("")
        .to_ascii_lowercase();
    require_any(
        &section_2,
        &["靠什么赚钱", "商业模式", "公司是什么"],
        "2. 公司与商业模式",
        &mut missing,
    );
    require_any(
        &section_3,
        &["护城河", "竞争壁垒", "壁垒"],
        "3. 护城河与壁垒",
        &mut missing,
    );
    require_any(
        &section_4,
        &["行业位置", "关键对手", "竞争对手"],
        "4. 行业位置与对手",
        &mut missing,
    );
    require_any(
        &section_5,
        &["财务质量", "毛利率", "自由现金流"],
        "5. 财务质量",
        &mut missing,
    );
    require_any(&section_6, &["估值"], "6. 估值", &mut missing);
    if !(section_7.contains("bull") && section_7.contains("bear") && section_7.contains("base")) {
        push_missing(&mut missing, "7. Bull / Bear / Base Case");
    }
    if !(section_8.contains("催化") && section_8.contains("风险") && section_8.contains("证伪"))
    {
        push_missing(&mut missing, "8. 催化、风险与证伪");
    }
    if !has_action_and_trigger(&section_9) {
        push_missing(&mut missing, "9. 动作建议与触发条件");
    }
    if !has_data_time_context(content) {
        missing.push("数据时间口径");
    }
    // Do not require the model to repeat the exact words “事实 / 推断”. A draft has
    // already separated the two when it labels source-backed statements as verified
    // and forward-looking statements as assumptions, estimates, or judgments.
    let has_fact_marker = ["事实", "已核验", "实际", "本轮数据"]
        .iter()
        .any(|marker| text.contains(marker));
    let has_inference_marker = ["推断", "假设", "估算", "判断", "预期", "情景"]
        .iter()
        .any(|marker| text.contains(marker));
    if !(has_fact_marker && has_inference_marker) {
        missing.push("事实 / 推断标识");
    }
    let valuation_method_count = usize::from(has_pe_valuation_method(&section_6))
        + [
            ["p/s", "ps 倍", "ps估值"].as_slice(),
            ["ev/ebitda", "ev / ebitda"].as_slice(),
            ["fcf yield", "自由现金流收益率"].as_slice(),
            ["dcf", "现金流折现"].as_slice(),
            ["sotp", "分部估值"].as_slice(),
            ["情景法", "情景分析"].as_slice(),
        ]
        .iter()
        .filter(|aliases| aliases.iter().any(|alias| section_6.contains(alias)))
        .count();
    if valuation_method_count < 2 {
        missing.push("至少两种估值方法");
    }
    missing
}

fn has_pe_valuation_method(section: &str) -> bool {
    Regex::new(r"(?i)(?:^|[^a-z0-9])p\s*/?\s*e(?:$|[^a-z0-9])")
        .expect("P/E valuation method regex")
        .is_match(section)
}

pub(crate) fn missing_deep_fund_sections(content: &str) -> Vec<&'static str> {
    let text = content.to_ascii_lowercase();
    let mut missing = Vec::new();
    require_any(&text, &["结论"], "1. 结论", &mut missing);
    require_any(
        &text,
        &["基金目标", "投资目标", "跟踪对象", "基金策略"],
        "2. 基金目标与策略",
        &mut missing,
    );
    require_any(
        &text,
        &["持仓", "集中度", "主要暴露"],
        "3. 持仓与主要暴露",
        &mut missing,
    );
    require_any(
        &text,
        &["地域", "行业", "货币风险", "汇率风险"],
        "4. 地域、行业与货币风险",
        &mut missing,
    );
    require_any(
        &text,
        &["流动性", "基金规模", "交易特征", "成交"],
        "5. 流动性、规模与交易特征",
        &mut missing,
    );
    require_any(
        &text,
        &["费用", "费率", "跟踪误差", "底层资产估值", "底层估值"],
        "6. 费用、跟踪误差与底层估值",
        &mut missing,
    );
    if !(text.contains("bull") && text.contains("bear") && text.contains("base")) {
        missing.push("7. Bull / Bear / Base Case");
    }
    if !(text.contains("催化") && text.contains("风险") && text.contains("证伪")) {
        missing.push("8. 催化、风险与证伪");
    }
    require_any(
        &text,
        &["动作建议", "行动建议", "操作建议"],
        "9. 动作建议",
        &mut missing,
    );
    for (number, label) in [
        (1, "1. 结论"),
        (2, "2. 基金目标与策略"),
        (3, "3. 持仓与主要暴露"),
        (4, "4. 地域、行业与货币风险"),
        (5, "5. 流动性、规模与交易特征"),
        (6, "6. 费用、跟踪误差与底层估值"),
        (7, "7. Bull / Bear / Base Case"),
        (8, "8. 催化、风险与证伪"),
        (9, "9. 动作建议"),
    ] {
        if !has_numbered_section(content, number) && !missing.contains(&label) {
            missing.push(label);
        }
    }
    for (number, label) in [
        (2, "2. 基金目标与策略"),
        (3, "3. 持仓与主要暴露"),
        (4, "4. 地域、行业与货币风险"),
        (5, "5. 流动性、规模与交易特征"),
        (6, "6. 费用、跟踪误差与底层估值"),
        (7, "7. Bull / Bear / Base Case"),
        (8, "8. 催化、风险与证伪"),
        (9, "9. 动作建议"),
    ] {
        if !numbered_section_has_substance(content, number) {
            push_missing(&mut missing, label);
        }
    }
    let section_2 = numbered_section(content, 2)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_3 = numbered_section(content, 3)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_4 = numbered_section(content, 4)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_5 = numbered_section(content, 5)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_6 = numbered_section(content, 6)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_7 = numbered_section(content, 7)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_8 = numbered_section(content, 8)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_9 = numbered_section(content, 9)
        .unwrap_or("")
        .to_ascii_lowercase();
    require_any(
        &section_2,
        &["基金目标", "投资目标", "跟踪对象", "基金策略"],
        "2. 基金目标与策略",
        &mut missing,
    );
    require_any(
        &section_3,
        &["持仓", "集中度", "主要暴露"],
        "3. 持仓与主要暴露",
        &mut missing,
    );
    require_any(
        &section_4,
        &["地域", "行业", "货币风险", "汇率风险"],
        "4. 地域、行业与货币风险",
        &mut missing,
    );
    require_any(
        &section_5,
        &["流动性", "基金规模", "交易特征", "成交"],
        "5. 流动性、规模与交易特征",
        &mut missing,
    );
    require_any(
        &section_6,
        &["费用", "费率", "跟踪误差", "底层资产估值", "底层估值"],
        "6. 费用、跟踪误差与底层估值",
        &mut missing,
    );
    if !(section_7.contains("bull") && section_7.contains("bear") && section_7.contains("base")) {
        push_missing(&mut missing, "7. Bull / Bear / Base Case");
    }
    if !(section_8.contains("催化") && section_8.contains("风险") && section_8.contains("证伪"))
    {
        push_missing(&mut missing, "8. 催化、风险与证伪");
    }
    if !has_action_and_trigger(&section_9) {
        push_missing(&mut missing, "9. 动作建议与触发条件");
    }
    if !has_data_time_context(content) {
        missing.push("数据时间口径");
    }
    let has_fact_marker = ["事实", "已核验", "实际", "本轮数据"]
        .iter()
        .any(|marker| text.contains(marker));
    let has_inference_marker = ["推断", "假设", "估算", "判断", "预期", "情景"]
        .iter()
        .any(|marker| text.contains(marker));
    if !(has_fact_marker && has_inference_marker) {
        missing.push("事实 / 推断标识");
    }
    missing
}

pub(crate) fn missing_deep_crypto_sections(content: &str) -> Vec<&'static str> {
    let text = content.to_ascii_lowercase();
    let mut missing = Vec::new();
    let labels = [
        "1. 结论",
        "2. 资产、网络与核心用途",
        "3. 供给机制、代币经济与集中度",
        "4. 采用、流动性与市场结构",
        "5. 链上、网络与生态数据",
        "6. 估值框架与关键假设",
        "7. Bull / Bear / Base Case",
        "8. 催化、监管、风险与证伪",
        "9. 动作建议",
    ];
    for (index, label) in labels.iter().enumerate() {
        let number = (index + 1) as u8;
        if !has_numbered_section(content, number)
            || (number >= 2 && !numbered_section_has_substance(content, number))
        {
            push_missing(&mut missing, label);
        }
    }
    let section_2 = numbered_section(content, 2)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_3 = numbered_section(content, 3)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_4 = numbered_section(content, 4)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_5 = numbered_section(content, 5)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_6 = numbered_section(content, 6)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_7 = numbered_section(content, 7)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_8 = numbered_section(content, 8)
        .unwrap_or("")
        .to_ascii_lowercase();
    let section_9 = numbered_section(content, 9)
        .unwrap_or("")
        .to_ascii_lowercase();
    require_any(
        &section_2,
        &["资产", "网络", "核心用途", "use case"],
        labels[1],
        &mut missing,
    );
    require_any(
        &section_3,
        &["供给", "代币经济", "集中度", "tokenomics"],
        labels[2],
        &mut missing,
    );
    require_any(
        &section_4,
        &["采用", "流动性", "市场结构", "adoption"],
        labels[3],
        &mut missing,
    );
    require_any(
        &section_5,
        &["链上", "网络", "生态", "on-chain"],
        labels[4],
        &mut missing,
    );
    require_any(
        &section_6,
        &["估值", "假设", "valuation"],
        labels[5],
        &mut missing,
    );
    if !(section_7.contains("bull") && section_7.contains("bear") && section_7.contains("base")) {
        push_missing(&mut missing, labels[6]);
    }
    if !(section_8.contains("催化") && section_8.contains("风险") && section_8.contains("证伪"))
    {
        push_missing(&mut missing, labels[7]);
    }
    if !has_action_and_trigger(&section_9) {
        push_missing(&mut missing, "9. 动作建议与触发条件");
    }
    if !has_data_time_context(content) {
        push_missing(&mut missing, "数据时间口径");
    }
    let has_fact = ["事实", "已核验", "实际", "本轮数据"]
        .iter()
        .any(|marker| text.contains(marker));
    let has_inference = ["推断", "假设", "估算", "判断", "预期", "情景"]
        .iter()
        .any(|marker| text.contains(marker));
    if !(has_fact && has_inference) {
        push_missing(&mut missing, "事实 / 推断标识");
    }
    missing
}

pub(crate) fn missing_investment_response_sections(
    contract: &InvestmentResponseContract,
    content: &str,
) -> Vec<&'static str> {
    match contract.deep_analysis {
        DeepAnalysisKind::Equity => {
            let mut missing = missing_deep_single_stock_sections(content);
            let conclusion = numbered_section(content, 1).unwrap_or("");
            if !entity_verified_price_appears(&contract.entities[0], conclusion)
                || !entity_verified_price_appears(&contract.entities[0], content)
            {
                push_missing(&mut missing, "1. 已核验同代码现价");
            }
            return missing;
        }
        DeepAnalysisKind::Fund => {
            let mut missing = missing_deep_fund_sections(content);
            let conclusion = numbered_section(content, 1).unwrap_or("");
            if !entity_verified_price_appears(&contract.entities[0], conclusion)
                || !entity_verified_price_appears(&contract.entities[0], content)
            {
                push_missing(&mut missing, "1. 已核验同代码现价");
            }
            return missing;
        }
        DeepAnalysisKind::Crypto => {
            let mut missing = missing_deep_crypto_sections(content);
            let conclusion = numbered_section(content, 1).unwrap_or("");
            if !entity_verified_price_appears(&contract.entities[0], conclusion)
                || !entity_verified_price_appears(&contract.entities[0], content)
            {
                push_missing(&mut missing, "1. 已核验同代码现价");
            }
            return missing;
        }
        DeepAnalysisKind::None => {}
    }
    if !contract.comparison {
        let mut missing = Vec::new();
        if contract.requires_verified_price
            && !entity_verified_price_appears(&contract.entities[0], content)
        {
            missing.push("已核验同代码现价");
        }
        return missing;
    }
    let normalized = content.to_ascii_uppercase();
    let mut missing = Vec::new();
    if contract
        .entities
        .iter()
        .any(|entity| !normalized.contains(&entity.symbol.to_ascii_uppercase()))
    {
        missing.push("逐标的覆盖");
    }
    let lower = content.to_ascii_lowercase();
    require_any(
        &lower,
        &["数据时间", "北京时间", "美东时间"],
        "数据时间",
        &mut missing,
    );
    if !contract.deep_comparison {
        if contract.requires_verified_price
            && contract.entities.iter().any(|entity| {
                !entity_line_verified_price_appears(entity, &contract.entities, content)
            })
        {
            push_missing(&mut missing, "逐标的已核验同代码现价");
        }
        return missing;
    }
    require_any(
        &lower,
        &["比较结论", "对比结论", "综合结论", "comparison conclusion"],
        "比较结论",
        &mut missing,
    );
    for entity in &contract.entities {
        let Some(section) = symbol_section(content, &entity.symbol, &contract.entities) else {
            push_missing(&mut missing, "逐标的独立小节");
            continue;
        };
        if !entity_verified_price_appears(entity, section) {
            push_missing(&mut missing, "逐标的已核验同代码现价");
        }
        let section_lower = section.to_ascii_lowercase();
        if entity_is_fund(entity)
            && ![
                "持仓",
                "集中度",
                "暴露",
                "费用",
                "holdings",
                "exposure",
                "fee",
            ]
            .iter()
            .any(|keyword| section_lower.contains(keyword))
        {
            push_missing(&mut missing, "ETF / 基金小节证据口径");
        }
        if entity_is_equity(entity)
            && ![
                "财务",
                "商业模式",
                "估值",
                "financial",
                "business model",
                "valuation",
            ]
            .iter()
            .any(|keyword| section_lower.contains(keyword))
        {
            push_missing(&mut missing, "公司小节证据口径");
        }
        if entity_is_crypto(entity)
            && ![
                "代币",
                "网络",
                "链上",
                "供给",
                "流动性",
                "token",
                "network",
                "on-chain",
                "liquidity",
            ]
            .iter()
            .any(|keyword| section_lower.contains(keyword))
        {
            push_missing(&mut missing, "加密资产小节证据口径");
        }
    }
    if !(lower.contains("风险") || lower.contains("risk"))
        || !(lower.contains("证伪") || lower.contains("失效") || lower.contains("falsif"))
    {
        missing.push("风险与证伪条件");
    }
    let has_action = ["动作建议", "行动建议", "操作建议", "action"]
        .iter()
        .any(|marker| lower.contains(marker));
    let has_trigger = ["触发条件", "触发点", "条件", "trigger"]
        .iter()
        .any(|marker| lower.contains(marker));
    if !(has_action && has_trigger) {
        missing.push("动作与触发条件");
    }
    let has_fact_marker = ["事实", "已核验", "实际", "本轮数据", "verified fact"]
        .iter()
        .any(|marker| lower.contains(marker));
    let has_inference_marker = ["推断", "假设", "估算", "判断", "预期", "情景", "inference"]
        .iter()
        .any(|marker| lower.contains(marker));
    if !(has_fact_marker && has_inference_marker) {
        missing.push("事实 / 推断标识");
    }
    missing
}

fn has_numbered_section(content: &str, number: u8) -> bool {
    Regex::new(&format!(
        r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{number}\s*[.、)]"
    ))
    .expect("numbered section regex")
    .is_match(content)
}

fn has_data_time_context(content: &str) -> bool {
    let section_two = Regex::new(r"(?m)^\s*(?:#{1,6}\s*)?(?:\*\*)?\s*2\s*[.、)]")
        .expect("second numbered section regex");
    let fallback_end = content
        .char_indices()
        .nth(1_200)
        .map(|(index, _)| index)
        .unwrap_or(content.len());
    let scope = section_two
        .find(content)
        .map(|matched| &content[..matched.start()])
        .unwrap_or(&content[..fallback_end]);
    let lower = scope.to_ascii_lowercase();
    if ["数据时间", "北京时间", "美东时间", "data time"]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return true;
    }

    let date = r"(?:20\d{2}[-/.]\d{1,2}[-/.]\d{1,2}|20\d{2}年\d{1,2}月\d{1,2}日)";
    let explicit_as_of = Regex::new(&format!(
        r"(?i)(?:数据口径|截至|核验(?:时间|日期)|as\s+of)[^。；;\r\n]{{0,64}}{date}"
    ))
    .expect("explicit data date regex");
    if explicit_as_of.is_match(scope) {
        return true;
    }

    // A quote may carry its provider date directly, for example
    // “当前报价 $30.495（2026-07-16）”. Keep the date on the same sentence and
    // close to a current-price marker so an unrelated listing or historical date
    // elsewhere in the analysis cannot satisfy the freshness contract.
    Regex::new(&format!(
        r"(?i)(?:现价|当前价(?:格)?|最新价(?:格)?|实时价(?:格)?|(?:当前|最新|实时)?股价|当前报价|最新报价|实时报价|current\s+price|last\s+price|quote)[^。；;\r\n]{{0,96}}{date}"
    ))
    .expect("current quote data date regex")
    .is_match(scope)
}

fn numbered_section(content: &str, number: u8) -> Option<&str> {
    let start_regex = Regex::new(&format!(
        r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{number}\s*[.、)]"
    ))
    .expect("numbered section start regex");
    let start = start_regex.find(content)?.start();
    let end = if number < 9 {
        Regex::new(&format!(
            r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{}\s*[.、)]",
            number + 1
        ))
        .expect("numbered section end regex")
        .find(&content[start + 1..])
        .map(|matched| start + 1 + matched.start())
        .unwrap_or(content.len())
    } else {
        content.len()
    };
    Some(&content[start..end])
}

fn numbered_section_has_substance(content: &str, number: u8) -> bool {
    let Some(section) = numbered_section(content, number) else {
        return false;
    };
    let marker = Regex::new(&format!(
        r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{number}\s*[.、)]"
    ))
    .expect("numbered section substance regex");
    let Some(marker) = marker.find(section) else {
        return false;
    };
    let remainder = section[marker.end()..].trim();
    let body_after_line = remainder
        .split_once('\n')
        .map(|(_, body)| body)
        .unwrap_or("");
    let body_after_colon = remainder
        .find(|character: char| matches!(character, '：' | ':'))
        .map(|index| &remainder[index + remainder[index..].chars().next().unwrap().len_utf8()..])
        .unwrap_or("");
    let meaningful_chars = |value: &str| {
        value
            .chars()
            .filter(|character| !character.is_whitespace() && !"-*#_`|".contains(*character))
            .count()
    };
    meaningful_chars(body_after_line) >= 6
        || meaningful_chars(body_after_colon) >= 6
        || meaningful_chars(remainder) >= 32
}

fn has_action_and_trigger(section: &str) -> bool {
    let has_action = [
        "买", "等", "减", "卖", "观察", "buy", "wait", "reduce", "sell",
    ]
    .iter()
    .any(|marker| section.contains(marker));
    let has_trigger = [
        "触发", "条件", "如果", "若", "当", "区间", "阈值", "跌破", "突破", "trigger",
    ]
    .iter()
    .any(|marker| section.contains(marker));
    has_action && has_trigger
}

fn symbol_section<'a>(
    content: &'a str,
    symbol: &str,
    entities: &[ResolvedSecurityEntity],
) -> Option<&'a str> {
    let heading = symbol_heading_regex(symbol);
    let start = heading.find(content)?.start();
    let end = entities
        .iter()
        .filter(|entity| !entity.symbol.eq_ignore_ascii_case(symbol))
        .filter_map(|entity| {
            symbol_heading_regex(&entity.symbol)
                .find(&content[start + 1..])
                .map(|matched| start + 1 + matched.start())
        })
        .min()
        .unwrap_or(content.len());
    Some(&content[start..end])
}

fn symbol_heading_regex(symbol: &str) -> Regex {
    Regex::new(&format!(
        r"(?im)^\s*#{{1,6}}\s*(?:\*\*)?\s*{}(?:\s|$|[（(\[|:：—-])",
        regex::escape(symbol)
    ))
    .expect("symbol heading regex")
}

fn entity_line_verified_price_appears(
    entity: &ResolvedSecurityEntity,
    entities: &[ResolvedSecurityEntity],
    content: &str,
) -> bool {
    content.split(['\n', '。', '；', ';', '，']).any(|segment| {
        symbol_appears_in_text(segment, &entity.symbol)
            && !entities.iter().any(|other| {
                !other.symbol.eq_ignore_ascii_case(&entity.symbol)
                    && symbol_appears_in_text(segment, &other.symbol)
            })
            && entity_verified_price_appears(entity, segment)
    })
}

fn symbol_appears_in_text(content: &str, symbol: &str) -> bool {
    Regex::new(&format!(
        r"(?i)(?:^|[^A-Z0-9.\-]){}(?:$|[^A-Z0-9.\-])",
        regex::escape(symbol)
    ))
    .expect("symbol occurrence regex")
    .is_match(content)
}

fn entity_verified_price_appears(entity: &ResolvedSecurityEntity, content: &str) -> bool {
    let Some(price) = entity
        .verified_price
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|price| price.is_finite() && *price > 0.0)
    else {
        return false;
    };
    // This is a restatement of the same current-turn quote, so only display rounding
    // is allowed. A percentage tolerance would admit materially wrong high prices.
    let tolerance = 0.011;
    let claims = Regex::new(
        r"(?i)(?:本轮(?:已核验)?同代码\s*)?(?P<label>现价|当前价(?:格)?|最新价(?:格)?|实时价(?:格)?|(?:当前|最新|实时)?股价|报价|current\s+price|last\s+price|quote)\s*(?:\*\*|__|`|\|)?\s*(?:(?:（截至[^）\r\n]{0,60}）)|(?:\(\s*as\s+of[^)\r\n]{0,60}\)))?\s*(?:\*\*|__|`|\|)?\s*(?:约为?|为|是|报)?\s*[:：=]?\s*(?:\*\*|__|`|\|)?\s*(?P<prefix>us\$|hk\$|c\$|a\$|s\$|\$|€|£|¥|￥|₩|₽|₹|[a-z]{3})?\s*(?P<number>\d[\d,]*(?:\.\d+)?)\s*(?P<suffix>美元|美金|欧元|港元|港币|人民币|加元|日元|英镑|澳元|新加坡元|瑞郎|韩元|卢布|新台币|纽元|泰铢|印度卢比|瑞典克朗|挪威克朗|丹麦克朗|南非兰特|巴西雷亚尔|墨西哥比索|[a-z]{3})?",
    )
    .expect("current price claim regex")
    .captures_iter(content)
    .filter_map(|capture| {
        let label = capture.name("label")?;
        if label.as_str().eq_ignore_ascii_case("股价") {
            let context = content[..label.start()].trim_end();
            if ["对应", "对应的", "目标", "目标的", "隐含", "隐含的", "折算", "折算的"]
                .iter()
                .any(|qualifier| context.ends_with(qualifier))
            {
                return None;
            }
        }
        let candidate = capture
            .name("number")
            .map(|value| value.as_str().replace(',', ""))
            .and_then(|value| value.parse::<f64>().ok())?;
        let stated_currencies = [capture.name("prefix"), capture.name("suffix")]
            .into_iter()
            .flatten()
            .map(|value| normalize_price_currency(value.as_str()))
            .collect::<Option<Vec<_>>>()?;
        let tail = capture
            .get(0)
            .map(|matched| &content[matched.end()..])
            .unwrap_or("")
            .trim_start();
        if stated_currencies.is_empty()
            && ["日均线", "日线", "年", "月", "日", "%"]
                .iter()
                .any(|unit| tail.starts_with(unit))
        {
            return None;
        }
        let currencies_agree = stated_currencies
            .windows(2)
            .all(|pair| pair[0] == pair[1]);
        let currency_matches = currencies_agree
            && entity
                .currency
                .as_deref()
                .map(str::to_ascii_uppercase)
                .map(|expected| {
                    stated_currencies
                        .iter()
                        .all(|stated| stated == &expected)
                })
                .unwrap_or(true);
        Some((candidate, currency_matches))
    })
    .collect::<Vec<_>>();
    !claims.is_empty()
        && claims.into_iter().all(|(candidate, currency_matches)| {
            currency_matches && (candidate - price).abs() <= tolerance
        })
}

fn normalize_price_currency(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "$" | "us$" | "usd" | "美元" | "美金" => Some("USD".to_string()),
        "€" | "eur" | "欧元" => Some("EUR".to_string()),
        "hk$" | "hkd" | "港元" | "港币" => Some("HKD".to_string()),
        "¥" | "￥" | "cny" | "rmb" | "人民币" => Some("CNY".to_string()),
        "c$" | "cad" | "加元" => Some("CAD".to_string()),
        "jpy" | "日元" => Some("JPY".to_string()),
        "£" | "gbp" | "英镑" => Some("GBP".to_string()),
        "a$" | "aud" | "澳元" => Some("AUD".to_string()),
        "s$" | "sgd" | "新加坡元" => Some("SGD".to_string()),
        "chf" | "瑞郎" => Some("CHF".to_string()),
        "₩" | "krw" | "韩元" => Some("KRW".to_string()),
        "₽" | "rub" | "卢布" => Some("RUB".to_string()),
        "twd" | "新台币" => Some("TWD".to_string()),
        "nzd" | "纽元" => Some("NZD".to_string()),
        "thb" | "泰铢" => Some("THB".to_string()),
        "₹" | "inr" | "印度卢比" => Some("INR".to_string()),
        "sek" | "瑞典克朗" => Some("SEK".to_string()),
        "nok" | "挪威克朗" => Some("NOK".to_string()),
        "dkk" | "丹麦克朗" => Some("DKK".to_string()),
        "zar" | "南非兰特" => Some("ZAR".to_string()),
        "brl" | "巴西雷亚尔" => Some("BRL".to_string()),
        "mxn" | "墨西哥比索" => Some("MXN".to_string()),
        code if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) => {
            Some(code.to_ascii_uppercase())
        }
        _ => None,
    }
}

fn push_missing(missing: &mut Vec<&'static str>, label: &'static str) {
    if !missing.contains(&label) {
        missing.push(label);
    }
}

fn require_any(
    content: &str,
    keywords: &[&str],
    label: &'static str,
    missing: &mut Vec<&'static str>,
) {
    if !keywords.iter().any(|keyword| content.contains(keyword)) {
        push_missing(missing, label);
    }
}

async fn extract_entity_mentions(
    core: &Arc<HoneBotCore>,
    input: &str,
    origin: AgentTurnOrigin,
) -> Result<Vec<EntityMention>, String> {
    if !should_run_entity_stage(input, origin) {
        return Ok(Vec::new());
    }
    let explicit = explicit_dollar_mentions(input);
    let deterministic =
        merge_entity_mentions(explicit.clone(), plain_ticker_mentions(input, origin));
    let trusted_scheduled_subject = origin != AgentTurnOrigin::Interactive
        && scheduled_request_has_security_context(input)
        && deterministic.iter().any(|mention| mention.tentative_symbol);
    if ticker_mentions_cover_request(input, &deterministic) || trusted_scheduled_subject {
        return Ok(deterministic);
    }
    let Some(llm) = core.auxiliary_llm.as_ref() else {
        return complete_entity_extraction(input, explicit);
    };
    let prompt = format!(
        "你是证券实体识别器，只做实体提取，不回答投资问题。\n\
         从下方当前请求中提取所有明确提到的上市公司、股票、ETF、基金或加密资产。\n\
         不得把行业词、技术词、财务指标、季度、报告缩写、任务配置、repeat 值或普通英文单词当成证券。\n\
         中文名、别名或旧公司名需要给出适合证券搜索的标准英文查询词；只有用户明确写出代码时才填写 explicit_symbol。\n\
         如果是宏观、行业或板块问题且没有点名证券，entities 必须为空数组。保留多标的，不得只取一个。\n\
         只输出严格 JSON：{{\"entities\":[{{\"mention\":\"原文\",\"search_query\":\"标准英文公司名或代码\",\"explicit_symbol\":null}}]}}。\n\n\
         当前请求：\n{}",
        input.trim()
    );
    let messages = vec![Message {
        role: "user".to_string(),
        content: Some(prompt),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
    let model = core.auxiliary_model_name();
    match llm.chat(&messages, Some(&model)).await {
        Ok(response) => parse_entity_extraction(&response.content)
            .map(|extracted| merge_entity_mentions(explicit, extracted))
            .map_err(|_| {
                "证券实体解析服务暂时未返回可用结果。请稍后重试，或补充明确 ticker。".to_string()
            })
            .and_then(|entities| complete_entity_extraction(input, entities)),
        Err(_) => complete_entity_extraction(input, explicit),
    }
}

fn explicit_dollar_mentions(input: &str) -> Vec<EntityMention> {
    let regex = Regex::new(r"(?i)\$[a-z][a-z0-9.-]{0,9}").expect("dollar ticker regex");
    regex
        .find_iter(input)
        .map(|matched| {
            matched
                .as_str()
                .trim_start_matches('$')
                .to_ascii_uppercase()
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|symbol| EntityMention {
            mention: ["$", symbol.as_str()].concat(),
            search_query: symbol.clone(),
            explicit_symbol: Some(symbol),
            tentative_symbol: false,
        })
        .collect()
}

fn plain_ticker_mentions(input: &str, origin: AgentTurnOrigin) -> Vec<EntityMention> {
    let normalized = input.to_ascii_lowercase();
    let has_cjk = input
        .chars()
        .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    let ticker_context = [
        "今天",
        "最近",
        "近期",
        "现在",
        "目前",
        "怎么样",
        "怎么看",
        "如何",
        "股票",
        "股价",
        "价格",
        "现价",
        "分析",
        "研究",
        "比较",
        "对比",
        "能买吗",
        "能不能买",
        "能不能",
        "能否买",
        "起飞",
        "前景",
        "未来",
        "财报",
        "业绩",
        "财务",
        "营收",
        "利润",
        "现金流",
        "持仓",
        "成分股",
        "费率",
        "跟踪误差",
        "估值",
        "目标价",
        "ticker",
        "stock",
        "share",
        "price",
        "earnings",
        "valuation",
        "buy",
        "sell",
        "compare",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
        || has_current_price_intent(&normalized);
    let lowercase_ticker_context = [
        "股票",
        "股价",
        "价格",
        "现价",
        "分析",
        "研究",
        "比较",
        "对比",
        "能买吗",
        "能不能",
        "能否买",
        "起飞",
        "前景",
        "财报",
        "业绩",
        "财务",
        "营收",
        "利润",
        "现金流",
        "持仓",
        "成分股",
        "费率",
        "跟踪误差",
        "估值",
        "目标价",
        "ticker",
        "stock",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
        || has_current_price_intent(&normalized)
        || (["今天", "最近", "近期", "现在", "目前"]
            .iter()
            .any(|marker| normalized.contains(marker))
            && ["怎么样", "怎么看", "表现", "多少钱"]
                .iter()
                .any(|marker| normalized.contains(marker)));
    let broad_scope = is_broad_scope_request(input);
    let scheduled_subject_end = if origin == AgentTurnOrigin::Interactive {
        input.len()
    } else {
        input
            .char_indices()
            .find_map(|(index, c)| matches!(c, '。' | '；' | '\n').then_some(index))
            .unwrap_or(input.len())
            .min(
                input
                    .char_indices()
                    .nth(96)
                    .map(|(index, _)| index)
                    .unwrap_or(input.len()),
            )
    };

    let bytes = input.as_bytes();
    let mut index = 0;
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    let mut accepted_scheduled_subject = false;
    while index < bytes.len() {
        if !bytes[index].is_ascii_alphabetic() {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len()
            && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'.' | b'-'))
        {
            index += 1;
        }
        let mut end = index;
        while end > start && matches!(bytes[end - 1], b'.' | b'-') {
            end -= 1;
        }
        let token = &input[start..end];
        if token.is_empty() || token.len() > 10 {
            continue;
        }
        if is_report_period_token(token) {
            continue;
        }
        let previous = input[..start].chars().rev().find(|c| !c.is_whitespace());
        let next = input[end..].chars().find(|c| !c.is_whitespace());
        if matches!(previous, Some('$' | '=')) || next == Some('=') {
            continue;
        }
        let letters = token.chars().filter(|c| c.is_ascii_alphabetic());
        let uppercase = letters.clone().all(|c| c.is_ascii_uppercase());
        let lowercase = letters.clone().all(|c| c.is_ascii_lowercase());
        let exact_input = input.trim().eq_ignore_ascii_case(token);
        if !uppercase && !(lowercase && has_cjk && lowercase_ticker_context) {
            continue;
        }
        if token.len() == 1 && !(ticker_context || exact_input) {
            continue;
        }
        if broad_scope && token.len() <= 3 && !exact_input {
            continue;
        }
        if origin != AgentTurnOrigin::Interactive {
            if start >= scheduled_subject_end {
                continue;
            }
            if !accepted_scheduled_subject
                && input[..start].chars().any(|c| c.is_ascii_alphabetic())
            {
                continue;
            }
            accepted_scheduled_subject = true;
        } else if !(ticker_context || exact_input || uppercase) {
            continue;
        }

        let symbol = token.to_ascii_uppercase();
        if seen.insert(symbol.clone()) {
            candidates.push(EntityMention {
                mention: token.to_string(),
                search_query: symbol.clone(),
                explicit_symbol: Some(symbol),
                tentative_symbol: true,
            });
        }
    }
    candidates
}

fn merge_entity_mentions(
    mut mentions: Vec<EntityMention>,
    additional: Vec<EntityMention>,
) -> Vec<EntityMention> {
    for mention in additional {
        let duplicate = mentions.iter_mut().find(|existing| {
            match (
                existing.explicit_symbol.as_deref(),
                mention.explicit_symbol.as_deref(),
            ) {
                (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
                _ => {
                    existing.mention.eq_ignore_ascii_case(&mention.mention)
                        && existing
                            .search_query
                            .eq_ignore_ascii_case(&mention.search_query)
                }
            }
        });
        if let Some(existing) = duplicate {
            if existing.tentative_symbol && !mention.tentative_symbol {
                *existing = mention;
            }
        } else {
            mentions.push(mention);
        }
    }
    mentions
}

fn ticker_mentions_cover_request(input: &str, mentions: &[EntityMention]) -> bool {
    if mentions.is_empty() {
        return false;
    }
    let mut residual = input.to_ascii_lowercase();
    for mention in mentions {
        residual = residual.replace(&mention.mention.to_ascii_lowercase(), "");
    }
    for grammar in [
        "能不能买",
        "能不能",
        "最近怎么样",
        "我想了解",
        "今天",
        "最近",
        "近期",
        "现在",
        "目前",
        "怎么样",
        "怎么看",
        "怎样",
        "如何",
        "请",
        "帮我",
        "深入",
        "详细",
        "分析",
        "研究",
        "一下",
        "股票",
        "股价",
        "证券",
        "代码",
        "价格",
        "现价",
        "当前价",
        "最新价",
        "实时价",
        "当前报价",
        "最新报价",
        "实时报价",
        "报价",
        "多少钱",
        "能买吗",
        "能否买",
        "前景",
        "未来",
        "财报",
        "业绩",
        "财务",
        "营收",
        "利润",
        "现金流",
        "持仓",
        "成分股",
        "费率",
        "跟踪误差",
        "估值",
        "目标价",
        "基本面",
        "业务",
        "竞争力",
        "竞争优势",
        "公司",
        "比较",
        "对比",
        "起飞",
        "表现",
        "值得",
        "时候",
        "过去",
        "和",
        "与",
        "的",
        "吗",
        "呢",
        "today",
        "recently",
        "lately",
        "please",
        "stock",
        "share",
        "price",
        "analyze",
        "analysis",
        "compare",
        "outlook",
        "doing",
        "worth",
        "how",
        "what",
        "about",
        "now",
        "buy",
        "sell",
        "and",
        "the",
        "is",
        "vs",
        "can",
        "take",
        "off",
        "in",
        "q1",
        "q2",
        "q3",
        "q4",
    ] {
        residual = residual.replace(grammar, "");
    }
    !residual.chars().any(char::is_alphanumeric)
}

fn is_report_period_token(token: &str) -> bool {
    let normalized = token.to_ascii_uppercase();
    matches!(normalized.as_str(), "Q1" | "Q2" | "Q3" | "Q4")
}

fn scheduled_request_has_security_context(input: &str) -> bool {
    let normalized = input.to_ascii_lowercase();
    [
        "关键事件",
        "重大事件",
        "公司事件",
        "股票",
        "股价",
        "行情",
        "证券",
        "财报",
        "业绩",
        "估值",
        "持仓",
        "ticker",
        "stock",
        "earnings",
        "valuation",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn should_run_entity_stage(input: &str, origin: AgentTurnOrigin) -> bool {
    if origin != AgentTurnOrigin::Interactive {
        return true;
    }
    let normalized = input.to_ascii_lowercase();
    let has_security_shaped_token = Regex::new(r"\b[A-Z][A-Z0-9.-]{1,9}\b")
        .expect("security shaped token regex")
        .is_match(input);
    let portfolio_overview = !has_security_shaped_token
        && [
            "看持仓",
            "查看持仓",
            "我的持仓",
            "持仓列表",
            "所有持仓",
            "我的关注",
            "关注列表",
        ]
        .iter()
        .any(|marker| normalized.contains(marker));
    if portfolio_overview {
        return false;
    }
    has_security_shaped_token
        || !plain_ticker_mentions(input, origin).is_empty()
        || analysis_has_named_subject(input)
        || has_current_price_intent(&normalized)
        || [
            "股票",
            "股价",
            "公司",
            "财报",
            "估值",
            "目标价",
            "能买吗",
            "能不能买",
            "能否买",
            "怎么看",
            "怎么样",
            "多少钱",
            "现价",
            "价格",
            "前景",
            "未来",
            "持仓",
            "关注",
            "比较",
            "ticker",
            "stock",
            "share",
            "price",
            "earnings",
            "valuation",
            "buy",
            "sell",
            "compare",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn analysis_has_named_subject(input: &str) -> bool {
    if !input.contains("分析") && !input.contains("研究") {
        return false;
    }
    let mut residual = input.to_string();
    for generic in [
        "请",
        "帮我",
        "继续",
        "分析",
        "研究",
        "一下",
        "深入",
        "详细",
        "看看",
        "看一下",
        "这个",
        "那个",
        "话题",
        "问题",
        "当前",
        "最新",
        "未来",
        "现在",
        "怎么",
        "如何",
    ] {
        residual = residual.replace(generic, "");
    }
    residual
        .chars()
        .filter(|character| ('\u{4e00}'..='\u{9fff}').contains(character))
        .count()
        >= 2
}

fn complete_entity_extraction(
    input: &str,
    entities: Vec<EntityMention>,
) -> Result<Vec<EntityMention>, String> {
    if !entities.is_empty() {
        return Ok(entities);
    }
    if is_broad_scope_request(input) {
        return Ok(Vec::new());
    }
    Err("我暂时无法从当前问题中确认具体公司或证券。请补充公司全名或 ticker。".to_string())
}

fn is_broad_scope_request(input: &str) -> bool {
    let normalized = input.to_ascii_lowercase();
    [
        "行业",
        "板块",
        "产业链",
        "宏观",
        "指数",
        "经济数据",
        "技术路线",
        "市场整体",
        "有什么影响",
        "如何影响",
        "的变化",
        "主题",
        "持仓观察",
        "市场观察",
        "sector",
        "industry",
        "macro",
        "index",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn parse_entity_extraction(content: &str) -> Result<Vec<EntityMention>, serde_json::Error> {
    let trimmed = content.trim();
    let object_starts = trimmed
        .char_indices()
        .filter_map(|(index, character)| (character == '{').then_some(index))
        .collect::<Vec<_>>();
    let object_ends = trimmed
        .char_indices()
        .filter_map(|(index, character)| (character == '}').then_some(index + 1))
        .collect::<Vec<_>>();
    let mut parsed = None;
    for start in object_starts.into_iter().rev() {
        for end in object_ends.iter().copied().rev() {
            if end <= start || !trimmed[start..end].contains("\"entities\"") {
                continue;
            }
            if let Ok(payload) =
                serde_json::from_str::<EntityExtractionPayload>(&trimmed[start..end])
            {
                parsed = Some(payload);
                break;
            }
        }
        if parsed.is_some() {
            break;
        }
    }
    let payload = match parsed {
        Some(payload) => payload,
        None => serde_json::from_str::<EntityExtractionPayload>(trimmed)?,
    };
    let mut seen = HashSet::new();
    Ok(payload
        .entities
        .into_iter()
        .take(32)
        .filter_map(|item| {
            let mention = item.mention.trim().to_string();
            let search_query = item.search_query.trim().to_string();
            if mention.is_empty() || search_query.is_empty() {
                return None;
            }
            let explicit_symbol = item
                .explicit_symbol
                .map(|s| s.trim().trim_start_matches('$').to_ascii_uppercase())
                .filter(|s| !s.is_empty());
            let key = format!("{}|{}", mention.to_lowercase(), search_query.to_lowercase());
            seen.insert(key).then_some(EntityMention {
                mention,
                search_query,
                explicit_symbol,
                tentative_symbol: false,
            })
        })
        .collect())
}

fn resolve_entity_match(mention: &EntityMention, search: &Value) -> EntityMatch {
    let candidates = search
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(entity_candidate_from_value)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return EntityMatch::Unresolved;
    }
    if let Some(explicit_symbol) = mention.explicit_symbol.as_deref() {
        return candidates
            .into_iter()
            .find(|candidate| candidate.symbol.eq_ignore_ascii_case(explicit_symbol))
            .map(|candidate| EntityMatch::Resolved(resolved_entity(mention, candidate)))
            .unwrap_or(EntityMatch::Unresolved);
    }
    let mut scored = candidates
        .into_iter()
        .map(|c| (entity_candidate_score(mention, &c), c))
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0));
    let best_score = scored[0].0;
    if best_score < 700 {
        return EntityMatch::Ambiguous(scored.into_iter().map(|(_, c)| c).collect());
    }
    let tied = scored
        .iter()
        .take_while(|(score, _)| *score == best_score)
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>();
    if tied.len() != 1 {
        return EntityMatch::Ambiguous(tied);
    }
    EntityMatch::Resolved(resolved_entity(mention, tied[0].clone()))
}

fn entity_candidate_from_value(value: &Value) -> Option<EntityCandidate> {
    let symbol = value
        .get("symbol")
        .or_else(|| value.get("ticker"))
        .and_then(Value::as_str)?
        .trim()
        .to_ascii_uppercase();
    if symbol.is_empty() {
        return None;
    }
    let name = value
        .get("name")
        .or_else(|| value.get("companyName"))
        .and_then(Value::as_str)
        .unwrap_or(&symbol)
        .trim()
        .to_string();
    let exchange = value
        .get("stockExchange")
        .or_else(|| value.get("exchangeShortName"))
        .or_else(|| value.get("exchange"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let currency = value
        .get("currency")
        .and_then(Value::as_str)
        .map(str::to_string);
    let asset_type = value
        .get("type")
        .or_else(|| value.get("assetType"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            value
                .get("exchangeShortName")
                .or_else(|| value.get("stockExchange"))
                .and_then(Value::as_str)
                .filter(|market| {
                    market.eq_ignore_ascii_case("CRYPTO") || market.eq_ignore_ascii_case("CCC")
                })
                .map(|_| "crypto".to_string())
        });
    Some(EntityCandidate {
        symbol,
        name,
        exchange,
        currency,
        asset_type,
    })
}

fn entity_candidate_score(mention: &EntityMention, candidate: &EntityCandidate) -> u16 {
    let query = normalize_entity_text(&mention.search_query);
    let original = normalize_entity_text(&mention.mention);
    let symbol = normalize_entity_text(&candidate.symbol);
    let name = normalize_entity_text(&candidate.name);
    let base = if query == symbol || original == symbol {
        950
    } else if query == name || original == name {
        900
    } else if query.len() >= 3 && (name.contains(&query) || query.contains(&name)) {
        800
    } else if original.len() >= 3 && (name.contains(&original) || original.contains(&name)) {
        750
    } else {
        0
    };
    let bonus = candidate
        .exchange
        .as_deref()
        .is_some_and(|exchange| {
            ["NASDAQ", "NYSE", "AMEX", "NASDAQ GLOBAL SELECT"]
                .iter()
                .any(|market| exchange.eq_ignore_ascii_case(market))
        })
        .then_some(20)
        .unwrap_or(0);
    base + bonus
}

fn normalize_entity_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn resolved_entity(mention: &EntityMention, candidate: EntityCandidate) -> ResolvedSecurityEntity {
    ResolvedSecurityEntity {
        mention: mention.mention.clone(),
        symbol: candidate.symbol,
        name: candidate.name,
        exchange: candidate.exchange,
        currency: candidate.currency,
        asset_type: candidate.asset_type,
        profile_verified: false,
        verified_price: None,
    }
}

#[cfg(test)]
fn quote_has_positive_matching_price(value: &Value, symbol: &str) -> bool {
    matching_quote_price(value, symbol).is_some()
}

fn matching_quote_price(value: &Value, symbol: &str) -> Option<f64> {
    if value_has_error(value) {
        return None;
    }
    match value {
        Value::Object(map) => {
            let symbol_ok = map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol));
            let price_ok = map
                .get("price")
                .and_then(Value::as_f64)
                .is_some_and(|price| price.is_finite() && price > 0.0);
            if symbol_ok && price_ok {
                return map.get("price").and_then(Value::as_f64);
            }
            map.values()
                .find_map(|child| matching_quote_price(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| matching_quote_price(child, symbol)),
        _ => None,
    }
}

fn has_nonempty_data(value: &Value) -> bool {
    !value_has_error(value)
        && value.get("data").is_some_and(|data| match data {
            Value::Array(items) => !items.is_empty(),
            Value::Object(map) => !map.is_empty(),
            _ => !data.is_null(),
        })
}

fn has_matching_symbol_data(value: &Value, symbol: &str) -> bool {
    !value_has_error(value)
        && value
            .get("data")
            .is_some_and(|data| contains_matching_symbol_object(data, symbol))
}

fn has_matching_financial_data(value: &Value, symbol: &str) -> bool {
    !value_has_error(value)
        && value
            .get("data")
            .is_some_and(|data| contains_meaningful_financial_record(data, symbol))
}

fn contains_meaningful_financial_record(value: &Value, symbol: &str) -> bool {
    match value {
        Value::Object(map) => {
            let same_symbol = map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol));
            let has_period = ["date", "calendarYear", "period"]
                .iter()
                .any(|field| map.get(*field).is_some_and(|value| !value.is_null()));
            let has_core_financial = [
                "revenue",
                "netIncome",
                "operatingIncome",
                "grossProfit",
                "eps",
                "epsdiluted",
            ]
            .iter()
            .any(|field| map.get(*field).is_some_and(Value::is_number));
            (same_symbol && has_period && has_core_financial)
                || map
                    .values()
                    .any(|child| contains_meaningful_financial_record(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .any(|child| contains_meaningful_financial_record(child, symbol)),
        _ => false,
    }
}

fn contains_matching_symbol_object(value: &Value, symbol: &str) -> bool {
    match value {
        Value::Object(map) => {
            map.get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol))
                || map
                    .values()
                    .any(|child| contains_matching_symbol_object(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .any(|child| contains_matching_symbol_object(child, symbol)),
        _ => false,
    }
}

fn value_has_error(value: &Value) -> bool {
    value
        .get("error")
        .is_some_and(|error| !error.is_null() && error.as_str() != Some(""))
}

fn result_or_error_value(result: hone_core::HoneResult<Value>) -> Value {
    result.unwrap_or_else(|err| json!({"error": err.to_string()}))
}

fn matching_symbol_objects(value: &Value, symbol: &str) -> Value {
    let mut output = Vec::new();
    collect_matching_symbol_objects(value.get("data").unwrap_or(value), symbol, &mut output);
    Value::Array(output)
}

fn matching_symbol_objects_or_error(value: &Value, symbol: &str) -> Value {
    if value_has_error(value) {
        value.clone()
    } else {
        matching_symbol_objects(value, symbol)
    }
}

fn collect_matching_symbol_objects(value: &Value, symbol: &str, output: &mut Vec<Value>) {
    if output.len() >= 8 {
        return;
    }
    match value {
        Value::Object(map) => {
            if map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol))
            {
                output.push(value.clone());
                return;
            }
            for child in map.values() {
                collect_matching_symbol_objects(child, symbol, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_matching_symbol_objects(child, symbol, output);
            }
        }
        _ => {}
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        format!(
            "{}…",
            value
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssetEvidenceRoute, DeepAnalysisKind, EntityMatch, EntityMention,
        InvestmentResponseContract, ResolvedSecurityEntity, asset_evidence_route,
        complete_entity_extraction, entity_is_crypto, entity_is_fund, explicit_dollar_mentions,
        forbidden_investment_tool_calls, has_data_time_context, has_matching_financial_data,
        has_matching_symbol_data, matching_symbol_objects_or_error, missing_deep_crypto_sections,
        missing_deep_fund_sections, missing_deep_single_stock_sections,
        missing_investment_response_sections, parse_entity_extraction, plain_ticker_mentions,
        quote_has_positive_matching_price, resolve_entity_match, response_intent,
        response_requires_verified_price, set_verified_asset_type, should_fetch_earnings_calendar,
        should_run_entity_stage, ticker_mentions_cover_request,
    };
    use crate::agent_session::AgentTurnOrigin;
    use hone_core::agent::ToolCallMade;
    use serde_json::json;

    #[test]
    fn extraction_payload_keeps_chinese_alias_and_multiple_entities() {
        let entities = parse_entity_extraction(
            r#"{"entities":[
          {"mention":"英伟达","search_query":"NVIDIA","explicit_symbol":null},
          {"mention":"AMD","search_query":"AMD","explicit_symbol":"AMD"}
        ]}"#,
        )
        .expect("extraction");
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].search_query, "NVIDIA");
        assert_eq!(entities[1].explicit_symbol.as_deref(), Some("AMD"));
    }

    #[test]
    fn extraction_parser_uses_the_last_complete_entities_object() {
        let entities = parse_entity_extraction(
            r#"<think>{"diagnostic":"not the answer"}</think>
```json
{"entities":[{"mention":"NBIS","search_query":"NBIS","explicit_symbol":"NBIS"}]}
```"#,
        )
        .expect("extraction after reasoning object");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].explicit_symbol.as_deref(), Some("NBIS"));
    }

    #[test]
    fn macro_or_sector_extraction_can_return_no_company_entity() {
        assert!(
            parse_entity_extraction(r#"{"entities":[]}"#)
                .unwrap()
                .is_empty()
        );
        assert!(
            complete_entity_extraction("AI 行业未来怎么看", Vec::new())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn explicit_dollar_symbols_are_preserved_without_acronym_denylist() {
        let entities = explicit_dollar_mentions("比较 $AMD、$NVDA 和 $AI");
        let symbols = entities
            .iter()
            .filter_map(|e| e.explicit_symbol.as_deref())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(symbols.len(), 3);
        assert!(symbols.contains("AMD") && symbols.contains("NVDA") && symbols.contains("AI"));
    }

    #[test]
    fn ordinary_ticker_questions_are_deterministic_candidates() {
        for (input, symbol) in [
            ("今天nbis怎么样", "NBIS"),
            ("NBIS最近怎么样", "NBIS"),
            ("现在intl怎么看", "INTL"),
            ("intl当前价", "INTL"),
            ("intl最新报价", "INTL"),
            ("intl持仓如何", "INTL"),
            ("intl费率", "INTL"),
        ] {
            let entities = plain_ticker_mentions(input, AgentTurnOrigin::Interactive);
            assert_eq!(entities.len(), 1, "{input}");
            assert_eq!(entities[0].explicit_symbol.as_deref(), Some(symbol));
            assert!(entities[0].tentative_symbol);
            assert!(ticker_mentions_cover_request(input, &entities), "{input}");
            assert!(should_run_entity_stage(input, AgentTurnOrigin::Interactive));
        }
        assert!(
            plain_ticker_mentions("hello", AgentTurnOrigin::Interactive).is_empty(),
            "an isolated lowercase word is not enough to claim ticker intent"
        );
    }

    #[test]
    fn reporting_period_is_not_a_symbol_in_a_ticker_question() {
        let input = "我想了解Q3的时候nbis能不能起飞";
        let entities = plain_ticker_mentions(input, AgentTurnOrigin::Interactive);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].explicit_symbol.as_deref(), Some("NBIS"));
        assert!(ticker_mentions_cover_request(input, &entities));
    }

    #[test]
    fn ordinary_multi_ticker_comparison_keeps_every_symbol() {
        let entities = plain_ticker_mentions("比较 NBIS 和 NVDA", AgentTurnOrigin::Interactive);
        let symbols = entities
            .iter()
            .filter_map(|entity| entity.explicit_symbol.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(symbols, vec!["NBIS", "NVDA"]);
        assert!(ticker_mentions_cover_request(
            "比较 NBIS 和 NVDA",
            &entities
        ));
    }

    #[test]
    fn industry_and_scheduler_acronyms_are_not_plain_ticker_candidates() {
        for input in ["AI 行业未来怎么看", "GPU 和 HBM 行业未来怎么看"] {
            assert!(
                plain_ticker_mentions(input, AgentTurnOrigin::Interactive).is_empty(),
                "{input}"
            );
        }
        assert!(
            plain_ticker_mentions(
                "REPEAT=30m，检查 API 状态后生成 AI 主题摘要",
                AgentTurnOrigin::Scheduled,
            )
            .is_empty()
        );
    }

    #[test]
    fn scheduled_ticker_subject_is_available_without_parsing_the_envelope() {
        let input = "每 30 分钟检查一次 NBIS / Nebius Group 关键事件，只在出现高权重变化时提醒用户。监控财报、ARR、GPU 与 EBITDA。";
        let entities = plain_ticker_mentions(input, AgentTurnOrigin::Scheduled);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].explicit_symbol.as_deref(), Some("NBIS"));
    }

    #[test]
    fn uppercase_metadata_is_treated_as_a_non_security_scope() {
        let result =
            complete_entity_extraction("REPEAT=30m，检查 API 状态后生成 AI 主题摘要", Vec::new());
        assert!(result.expect("non-security scope").is_empty());
    }

    #[test]
    fn entity_stage_uses_structured_origin_and_skips_portfolio_overview() {
        assert!(should_run_entity_stage(
            "检查正文",
            AgentTurnOrigin::Scheduled
        ));
        assert!(should_run_entity_stage(
            "检查条件",
            AgentTurnOrigin::Heartbeat
        ));
        assert!(!should_run_entity_stage(
            "帮我看持仓",
            AgentTurnOrigin::Interactive
        ));
        assert!(!should_run_entity_stage(
            "请继续分析这个话题",
            AgentTurnOrigin::Interactive
        ));
        assert!(should_run_entity_stage(
            "请分析一下英伟达",
            AgentTurnOrigin::Interactive
        ));
    }

    #[test]
    fn exact_symbol_resolution_rejects_nearby_wrong_company() {
        let mention = EntityMention {
            mention: "NBIS".into(),
            search_query: "NBIS".into(),
            explicit_symbol: Some("NBIS".into()),
            tentative_symbol: false,
        };
        assert!(matches!(
            resolve_entity_match(&mention, &json!({"data":[{"symbol":"NBIS","name":"Nebius Group N.V."}]})),
            EntityMatch::Resolved(entity) if entity.symbol == "NBIS"
        ));
        assert_eq!(
            resolve_entity_match(
                &mention,
                &json!({"data":[{"symbol":"MBIS","name":"Mediobanca"}]})
            ),
            EntityMatch::Unresolved
        );
    }

    #[test]
    fn normalized_company_name_resolves_chinese_alias_search_query() {
        let mention = EntityMention {
            mention: "英伟达".into(),
            search_query: "NVIDIA".into(),
            explicit_symbol: None,
            tentative_symbol: false,
        };
        assert!(matches!(
            resolve_entity_match(&mention, &json!({"data":[
              {"symbol":"NVDA","name":"NVIDIA Corporation","stockExchange":"NASDAQ","currency":"USD","type":"stock"},
              {"symbol":"NVD","name":"NVIDIA Corporation","stockExchange":"Frankfurt","currency":"EUR","type":"stock"}
            ]})),
            EntityMatch::Resolved(entity) if entity.symbol == "NVDA"
        ));
    }

    #[test]
    fn dual_share_classes_remain_ambiguous_instead_of_taking_first_result() {
        let mention = EntityMention {
            mention: "Alphabet".into(),
            search_query: "Alphabet".into(),
            explicit_symbol: None,
            tentative_symbol: false,
        };
        let result = resolve_entity_match(
            &mention,
            &json!({"data":[
              {"symbol":"GOOGL","name":"Alphabet Inc.","stockExchange":"NASDAQ"},
              {"symbol":"GOOG","name":"Alphabet Inc.","stockExchange":"NASDAQ"}
            ]}),
        );
        assert!(matches!(result, EntityMatch::Ambiguous(candidates) if candidates.len() == 2));
    }

    #[test]
    fn response_intent_distinguishes_quote_from_deep_outlook() {
        assert_eq!(response_intent("NBIS现在多少钱"), (false, false));
        assert_eq!(response_intent("今天nbis怎么样"), (true, false));
        assert_eq!(response_intent("intl持仓如何"), (true, false));
        assert_eq!(response_intent("intl费率"), (true, false));
        assert_eq!(response_intent("比较 INTL 和 NBIS"), (true, false));
        assert_eq!(response_intent("INTL vs NBIS"), (true, false));
        assert_eq!(response_intent("INTL 和 NBIS 哪个好"), (true, false));
        assert_eq!(
            response_intent("我想了解Q3的时候NBIS能不能起飞"),
            (true, true)
        );
        assert!(response_requires_verified_price(
            "NBIS现在多少钱",
            false,
            false
        ));
        for input in ["intl当前价", "intl最新报价", "intl实时价"] {
            assert!(response_requires_verified_price(input, false, false));
        }
        assert!(!response_requires_verified_price(
            "NBIS 是什么公司",
            false,
            false
        ));
    }

    fn entities(symbols: &[&str]) -> Vec<ResolvedSecurityEntity> {
        symbols
            .iter()
            .map(|symbol| ResolvedSecurityEntity {
                mention: (*symbol).into(),
                symbol: (*symbol).into(),
                name: (*symbol).into(),
                exchange: Some("NASDAQ".into()),
                currency: Some("USD".into()),
                asset_type: Some("stock".into()),
                profile_verified: false,
                verified_price: Some("100.0".into()),
            })
            .collect()
    }

    #[test]
    fn multi_entity_contract_and_final_validator_cover_every_symbol() {
        let contract = InvestmentResponseContract {
            entities: entities(&["AMD", "NVDA"]),
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: true,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: true,
            origin: AgentTurnOrigin::Interactive,
        };
        assert!(contract.enforcement_block().contains("多证券比较门禁"));
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：今天。AMD 有数据。风险待确认"
            )
            .contains(&"逐标的覆盖")
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：今天。比较结论：AMD 与 NVDA 已逐一比较。已核验事实如下，推断情景另列。\n### AMD\n本轮同代码现价 100.0 美元；财务与估值如下。\n### NVDA\n本轮同代码现价 100.0 美元；财务与估值如下。\n风险与证伪条件如下。动作建议与触发条件如下。"
            )
            .is_empty()
        );
    }

    #[test]
    fn quote_only_contract_rejects_missing_wrong_or_conflicting_current_price() {
        let contract = InvestmentResponseContract {
            entities: entities(&["NBIS"]),
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        assert!(
            missing_investment_response_sections(&contract, "NBIS 今天震荡。")
                .contains(&"已核验同代码现价")
        );
        assert!(
            missing_investment_response_sections(&contract, "NBIS 现价 15 美元。")
                .contains(&"已核验同代码现价")
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "NBIS 现价 15 美元；本轮已核验同代码现价 100 美元。",
            )
            .contains(&"已核验同代码现价")
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：现在。NBIS 当前价 100.0 美元。",
            )
            .is_empty()
        );
        for formatted in [
            "NBIS **现价：** $100.00。",
            "NBIS 当前价格为 100.00 美元。",
            "NBIS 报价 USD 100.00。",
        ] {
            assert!(
                missing_investment_response_sections(&contract, formatted).is_empty(),
                "{formatted}"
            );
        }
        assert!(
            missing_investment_response_sections(
                &contract,
                "NBIS 当前价（截至北京时间 2026-07-16）：100.0 美元。",
            )
            .is_empty(),
            "an as-of date must not be parsed as the current price"
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "NBIS 现价相对 30 日均线偏强；当前价 100 美元。",
            )
            .is_empty(),
            "a moving-average period must not be parsed as the current price"
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "NBIS 股价 15 美元；当前价 100 美元。",
            )
            .contains(&"已核验同代码现价")
        );
        assert!(
            missing_investment_response_sections(&contract, "NBIS 报价 100 欧元。")
                .contains(&"已核验同代码现价"),
            "an explicitly wrong currency must not pass price grounding"
        );
        for wrong in [
            "NBIS 现价 100.50 美元。",
            "NBIS 报价 100 加元。",
            "NBIS 现价 $100 欧元。",
        ] {
            assert!(
                missing_investment_response_sections(&contract, wrong)
                    .contains(&"已核验同代码现价"),
                "{wrong}"
            );
        }
    }

    #[test]
    fn shallow_multi_quote_contract_validates_each_symbol_locally() {
        let contract = InvestmentResponseContract {
            entities: entities(&["AMD", "NVDA"]),
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: true,
            origin: AgentTurnOrigin::Interactive,
        };
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：北京时间。\n- AMD 现价 100 美元\n- NVDA 当前价 100 美元",
            )
            .is_empty()
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：北京时间。\n- AMD 现价 100 美元\n- NVDA 当前价 15 美元",
            )
            .contains(&"逐标的已核验同代码现价")
        );
        assert!(
            missing_investment_response_sections(
                &contract,
                "数据时间：北京时间。AMD 和 NVDA 当前价 100 美元。",
            )
            .contains(&"逐标的已核验同代码现价"),
            "one shared claim must not substitute for per-symbol price grounding"
        );
    }

    #[test]
    fn mixed_fund_equity_comparison_requires_both_asset_evidence_routes() {
        let mut mixed = entities(&["INTL", "NBIS"]);
        mixed[0].asset_type = Some("etf_or_fund".into());
        mixed[0].profile_verified = true;
        mixed[1].asset_type = Some("equity".into());
        mixed[1].profile_verified = true;
        let contract = InvestmentResponseContract {
            entities: mixed,
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: true,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: true,
            origin: AgentTurnOrigin::Interactive,
        };
        let incomplete = "数据时间：北京时间。比较结论：INTL 和 NBIS 各有风险与证伪条件。已核验事实与情景推断分开。\n### INTL\n本轮同代码现价 100 美元；这里只写公司财务。\n### NBIS\n本轮同代码现价 100 美元；这里只写基金持仓。\n动作建议与触发条件如下。";
        let missing = missing_investment_response_sections(&contract, incomplete);
        assert!(missing.contains(&"ETF / 基金小节证据口径"));
        assert!(missing.contains(&"公司小节证据口径"));

        let complete = "数据时间：北京时间。比较结论：INTL 和 NBIS 已逐一比较。已核验事实与情景推断分开。\n### INTL\n本轮同代码现价 100 美元；持仓集中度、主要暴露与费用已列。\n### NBIS\n本轮同代码现价 100 美元；财务与估值已列。\n风险与证伪条件如下。动作建议与触发条件如下。";
        assert!(missing_investment_response_sections(&contract, complete).is_empty());
    }

    #[test]
    fn mixed_crypto_equity_comparison_keeps_route_specific_evidence() {
        let mut mixed = entities(&["BTCUSD", "NBIS"]);
        mixed[0].asset_type = Some("crypto".into());
        mixed[0].profile_verified = true;
        mixed[1].asset_type = Some("equity".into());
        mixed[1].profile_verified = true;
        let contract = InvestmentResponseContract {
            entities: mixed,
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: true,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: true,
            origin: AgentTurnOrigin::Interactive,
        };
        let incomplete = "数据时间：北京时间。比较结论已列。已核验事实与情景推断分开。\n### BTCUSD\n本轮同代码现价 100 美元；这里只写公司财务。\n### NBIS\n本轮同代码现价 100 美元；财务与估值已列。\n风险与证伪条件如下。动作建议与触发条件如下。";
        assert!(
            missing_investment_response_sections(&contract, incomplete)
                .contains(&"加密资产小节证据口径")
        );
        let complete = "数据时间：北京时间。比较结论已列。已核验事实与情景推断分开。\n### BTCUSD\n本轮同代码现价 100 美元；网络、代币供给与流动性已列。\n### NBIS\n本轮同代码现价 100 美元；财务与估值已列。\n风险与证伪条件如下。动作建议与触发条件如下。";
        assert!(missing_investment_response_sections(&contract, complete).is_empty());
    }

    #[test]
    fn scheduler_contract_uses_typed_origin_not_envelope_text() {
        let contract = InvestmentResponseContract {
            entities: entities(&["NBIS"]),
            deep_analysis: DeepAnalysisKind::None,
            deep_comparison: false,
            requires_verified_price: false,
            needs_outlook_evidence: false,
            comparison: false,
            origin: AgentTurnOrigin::Scheduled,
        };
        let block = contract.enforcement_block();
        assert!(block.contains("结构化 Scheduled"));
        assert!(block.contains("repeat 配置"));
    }

    #[test]
    fn incomplete_deep_reply_is_rejected_and_complete_reply_passes() {
        let missing = missing_deep_single_stock_sections(
            "结论：可能上涨。Bull 看增长，Bear 看竞争。你成本多少？",
        );
        assert!(missing.contains(&"2. 公司与商业模式"));
        assert!(missing.contains(&"9. 动作建议"));
        let complete = "数据时间：北京时间 2026-07-16。已核验事实与情景推断分开。\n1. 结论：本轮数据支持保持审慎观察。\n2. 公司是什么、靠什么赚钱：商业模式为云服务收入。\n3. 护城河与竞争壁垒：壁垒来自资源与客户粘性。\n4. 行业位置与关键对手：竞争对手与行业位置待持续跟踪。\n5. 财务质量与自由现金流：自由现金流仍是核心验证项。\n6. 估值：使用 P/S 与情景法两种方法，假设如下。\n7. Bull / Bear / Base Case：Bull 看增长，Bear 看竞争，Base 看执行。\n8. 催化剂、风险点、证伪条件：催化是订单，风险是降速，证伪是增长失速。\n9. 动作建议：观察；若增长与现金流同时改善则触发重评。";
        assert!(missing_deep_single_stock_sections(complete).is_empty());
    }

    #[test]
    fn rmbs_forward_pe_and_target_prices_pass_but_conflicting_current_price_fails() {
        let mut rmbs = entities(&["RMBS"]).remove(0);
        rmbs.name = "Rambus Inc.".into();
        rmbs.verified_price = Some("102.89".into());
        let contract = InvestmentResponseContract {
            entities: vec![rmbs],
            deep_analysis: DeepAnalysisKind::Equity,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: true,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        let complete = "数据时间：北京时间 2026-07-16。以下区分本轮已核验事实与情景推断。\n1. 结论：RMBS 当前价 **$102.89**，估值偏高，动作上先观察。\n2. 公司是什么、靠什么赚钱：公司通过芯片接口及安全 IP 授权和相关产品收入赚钱，商业模式以授权为核心。\n3. 护城河与竞争壁垒：护城河来自接口 IP、专利组合和客户验证周期形成的竞争壁垒。\n4. 行业位置与关键对手：公司处于内存接口产业链，行业位置及竞争对手的份额变化需要持续核验。\n5. 财务质量：本轮数据反映毛利率较高，自由现金流及收入持续性仍是财务质量的核心验证项。\n6. 估值：方法一采用 Forward PE，假设目标 PE 40x，对应股价 $252；方法二采用 EV/EBITDA，在保守假设下对应股价 $126。上述均为情景估算，不是当前报价。\n7. Bull / Bear / Base Case：Bull 看新品放量，Bear 看估值压缩，Base 看收入按预期增长。\n8. 催化剂、风险点、证伪条件：催化是新品订单，风险是竞争加剧；若收入增长失速则构成证伪。\n9. 动作建议：观察；若盈利兑现且估值回落到目标区间则触发重新评估。";

        assert!(
            missing_investment_response_sections(&contract, complete).is_empty(),
            "Forward PE 与 EV/EBITDA 是两种方法，估值目标价不得冒充当前价"
        );

        let pe_only = complete.replace(
            "方法二采用 EV/EBITDA，在保守假设下对应股价 $126",
            "方法二仍采用 Forward P/E，并以 PE 40x 得到目标股价 $126",
        );
        assert!(
            missing_investment_response_sections(&contract, &pe_only).contains(&"至少两种估值方法"),
            "Forward PE、Forward P/E、目标 PE 与 PE 40x 都只能计为同一种 P/E 方法"
        );

        let conflicting = complete.replacen(
            "RMBS 当前价 **$102.89**",
            "RMBS 当前价 **$102.89**，但最新价 **$99.00**",
            1,
        );
        assert!(
            missing_investment_response_sections(&contract, &conflicting)
                .contains(&"1. 已核验同代码现价"),
            "明确的最新价冲突仍必须被拒绝"
        );
    }

    #[test]
    fn data_time_context_accepts_dated_quote_semantics_but_not_unrelated_dates() {
        for accepted in [
            "数据时间：北京时间 2026-07-16。\n1. 结论：现价 30.495 美元。\n2. 下一节",
            "数据口径（截至 2026-07-16）。\n1. 结论：现价 30.495 美元。\n2. 下一节",
            "As of 2026-07-16.\n1. 结论：current price USD 30.495。\n2. 下一节",
            "1. 结论：INTL 当前报价 $30.495（2026-07-16 核验）。\n2. 下一节",
        ] {
            assert!(has_data_time_context(accepted), "must accept: {accepted}");
        }
        for rejected in [
            "1. 结论：现价 30.495 美元。\n2. 基金成立于 2022-12-02。",
            "1. 结论：现价 30.495 美元。\n2. 基金目标。\n8. 催化日期 2026-09-01。",
            "1. 结论：本轮已核验，现价 30.495 美元。\n2. 下一节",
            "数据口径：截至目前。\n1. 结论：现价 30.495 美元。\n2. 下一节",
        ] {
            assert!(!has_data_time_context(rejected), "must reject: {rejected}");
        }
    }

    #[test]
    fn exact_profile_routes_intl_to_fund_evidence_and_nbis_to_equity() {
        let intl = ResolvedSecurityEntity {
            mention: "intl".into(),
            symbol: "INTL".into(),
            name: "Main International ETF".into(),
            exchange: Some("CBOE".into()),
            currency: Some("USD".into()),
            asset_type: None,
            profile_verified: false,
            verified_price: None,
        };
        let nbis = ResolvedSecurityEntity {
            mention: "nbis".into(),
            symbol: "NBIS".into(),
            name: "Nebius Group N.V.".into(),
            exchange: Some("NASDAQ".into()),
            currency: Some("USD".into()),
            asset_type: None,
            profile_verified: false,
            verified_price: None,
        };
        assert_eq!(
            asset_evidence_route(
                &json!({"data":[{"symbol":"INTL","isEtf":true,"isFund":false}]}),
                &intl.symbol
            ),
            Some(AssetEvidenceRoute::Fund)
        );
        assert_eq!(
            asset_evidence_route(
                &json!({"data":[{"symbol":"NBIS","isEtf":false,"isFund":false}]}),
                &nbis.symbol
            ),
            Some(AssetEvidenceRoute::Equity)
        );

        let mut verified_intl = intl;
        set_verified_asset_type(&mut verified_intl, AssetEvidenceRoute::Fund);
        assert!(verified_intl.profile_verified);
        assert!(!should_fetch_earnings_calendar(&verified_intl));
        let mut verified_nbis = nbis;
        set_verified_asset_type(&mut verified_nbis, AssetEvidenceRoute::Equity);
        assert!(should_fetch_earnings_calendar(&verified_nbis));
    }

    #[test]
    fn exact_crypto_market_search_routes_without_stock_profile_or_company_tools() {
        let mention = EntityMention {
            mention: "BTCUSD".into(),
            search_query: "BTCUSD".into(),
            explicit_symbol: Some("BTCUSD".into()),
            tentative_symbol: true,
        };
        let resolved = resolve_entity_match(
            &mention,
            &json!({"data":[{
                "symbol":"BTCUSD",
                "name":"Bitcoin USD",
                "currency":"USD",
                "stockExchange":"CCC",
                "exchangeShortName":"CRYPTO"
            }]}),
        );
        let EntityMatch::Resolved(mut entity) = resolved else {
            panic!("BTCUSD must resolve from its exact CRYPTO market record");
        };
        assert!(entity_is_crypto(&entity));
        set_verified_asset_type(&mut entity, AssetEvidenceRoute::Crypto);
        assert!(entity.profile_verified);
        assert!(!should_fetch_earnings_calendar(&entity));

        let contract = InvestmentResponseContract {
            entities: vec![entity],
            deep_analysis: DeepAnalysisKind::Crypto,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: true,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        let call = |data_type: &str| ToolCallMade {
            name: "data_fetch".into(),
            arguments: json!({"data_type":data_type,"ticker":"BTCUSD"}),
            result: json!({"data":[]}),
            tool_call_id: None,
        };
        for forbidden in ["financials", "earnings_calendar", "etf_holdings"] {
            assert!(
                !forbidden_investment_tool_calls(&contract, &[call(forbidden)]).is_empty(),
                "{forbidden}"
            );
        }
        assert!(forbidden_investment_tool_calls(&contract, &[call("news")]).is_empty());
    }

    #[test]
    fn crypto_contract_requires_substantive_crypto_sections() {
        let mut crypto = entities(&["BTCUSD"]).remove(0);
        crypto.asset_type = Some("crypto".into());
        crypto.profile_verified = true;
        let contract = InvestmentResponseContract {
            entities: vec![crypto],
            deep_analysis: DeepAnalysisKind::Crypto,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        let headings_only = "数据时间：北京时间。已核验事实与情景推断分开。\n1. 结论：现价 100 美元\n2. 资产、网络与核心用途\n3. 供给机制、代币经济与集中度\n4. 采用、流动性与市场结构\n5. 链上、网络与生态数据\n6. 估值框架与关键假设\n7. Bull / Bear / Base Case\n8. 催化、监管、风险与证伪\n9. 动作建议";
        assert!(!missing_deep_crypto_sections(headings_only).is_empty());
        let complete = "数据时间：北京时间。已核验事实与情景推断分开。\n1. 结论：本轮同代码现价 100 美元，先观察。\n2. 资产、网络与核心用途：网络用于价值转移与结算。\n3. 供给机制、代币经济与集中度：供给节奏与集中度是核心变量。\n4. 采用、流动性与市场结构：采用率与流动性决定交易质量。\n5. 链上、网络与生态数据：链上活跃与生态数据本轮未核验。\n6. 估值框架与关键假设：估值取决于采用、流动性与假设。\n7. Bull / Bear / Base Case：Bull 看采用，Bear 看监管，Base 看流动性。\n8. 催化、监管、风险与证伪：催化是采用，风险是监管，证伪是活跃度失速。\n9. 动作建议：观察；若流动性与采用同时改善则触发重评。";
        assert!(missing_deep_crypto_sections(complete).is_empty());
        assert!(missing_investment_response_sections(&contract, complete).is_empty());
    }

    #[test]
    fn profile_classification_ignores_fund_flags_for_a_different_symbol() {
        let entity = entities(&["NBIS"]).remove(0);
        assert_eq!(
            asset_evidence_route(
                &json!({"data":[
                    {"symbol":"INTL","isEtf":true},
                    {"symbol":"NBIS","isEtf":false,"isFund":false}
                ]}),
                &entity.symbol
            ),
            Some(AssetEvidenceRoute::Equity)
        );
        assert_eq!(
            asset_evidence_route(
                &json!({
                    "metadata":{"type":"fund","isEtf":true},
                    "data":[{"symbol":"NBIS","companyName":"Nebius Group N.V."}]
                }),
                &entity.symbol
            ),
            None,
            "unknown exact-symbol profile shape must fail closed instead of using metadata or companyName"
        );
        assert_eq!(
            asset_evidence_route(
                &json!({"data":[{"symbol":"NBIS","isEtf":null,"isFund":false}]}),
                &entity.symbol
            ),
            None,
            "partial or non-boolean profile flags must remain unknown"
        );
    }

    #[test]
    fn profile_and_financial_evidence_must_match_the_resolved_symbol() {
        assert!(has_matching_symbol_data(
            &json!({"data":[{"symbol":"NBIS","isEtf":false}]}),
            "NBIS"
        ));
        assert!(has_matching_symbol_data(
            &json!({"data":[{"symbol":"NBIS","date":"2025-12-31","revenue":100}]}),
            "NBIS"
        ));
        assert!(!has_matching_symbol_data(
            &json!({"data":[{"symbol":"MBIS","date":"2025-12-31","revenue":100}]}),
            "NBIS"
        ));
        assert!(!has_matching_symbol_data(
            &json!({"ticker":"NBIS","data":[{"symbol":"MBIS","revenue":100}]}),
            "NBIS"
        ));
        assert!(!has_matching_symbol_data(
            &json!({"data":{"Error Message":"temporary provider failure"}}),
            "NBIS"
        ));
        assert!(has_matching_financial_data(
            &json!({"data":[{"symbol":"NBIS","date":"2025-12-31","revenue":100}]}),
            "NBIS"
        ));
        assert!(!has_matching_financial_data(
            &json!({"data":[{"symbol":"NBIS"}]}),
            "NBIS"
        ));
        assert!(!has_matching_financial_data(
            &json!({"data":[{"symbol":"NBIS","revenue":100}]}),
            "NBIS"
        ));
        assert!(!has_matching_financial_data(
            &json!({"data":[{"symbol":"MBIS","date":"2025-12-31","revenue":100}]}),
            "NBIS"
        ));
    }

    #[test]
    fn fund_contract_uses_fund_sections_and_rejects_company_template() {
        let mut fund_entity = entities(&["INTL"]).remove(0);
        fund_entity.asset_type = Some("etf_or_fund".into());
        fund_entity.profile_verified = true;
        fund_entity.verified_price = Some("30.495".into());
        let contract = InvestmentResponseContract {
            entities: vec![fund_entity.clone()],
            deep_analysis: DeepAnalysisKind::Fund,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: true,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        let block = contract.enforcement_block();
        assert!(block.contains("ETF / 基金深度分析"));
        assert!(block.contains("持仓、集中度与主要暴露"));
        assert!(block.contains("不得套用单一公司的商业模式"));
        assert!(entity_is_fund(&fund_entity));

        let company_template = "数据时间：北京时间。事实与推断分开。\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量\n6. 估值：P/S + 情景法\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议";
        assert!(!missing_deep_fund_sections(company_template).is_empty());

        let complete = "数据时间：北京时间 2026-07-16。已核验事实与情景假设分开。\n1. 结论：本轮同代码现价 30.495 美元，暂以观察为主。\n2. 基金目标、基金策略与跟踪对象：跟踪国际市场暴露是核心目标。\n3. 持仓、集中度与主要暴露：持仓与集中度按本轮数据核验。\n4. 地域、行业与货币风险：地域与汇率风险需同时管理。\n5. 流动性、基金规模与交易特征：流动性与成交特征决定交易成本。\n6. 费用、跟踪误差与底层资产估值：费率与底层估值是关键变量。\n7. Bull / Bear / Base Case：Bull 看风险偏好，Bear 看汇率，Base 看基准收益。\n8. 催化剂、风险点、证伪条件：催化是宽松，风险是波动，证伪是暴露失效。\n9. 动作建议：观察；若费率、流动性与暴露均符合条件则再评估。";
        assert!(missing_deep_fund_sections(complete).is_empty());
        assert!(missing_investment_response_sections(&contract, complete).is_empty());
        let dated_quote_without_literal_time_label = complete
            .replacen("数据时间：北京时间 2026-07-16。", "", 1)
            .replacen(
                "本轮同代码现价 30.495 美元",
                "INTL 当前报价 $30.495（2026-07-16 核验）",
                1,
            );
        assert!(
            missing_investment_response_sections(
                &contract,
                &dated_quote_without_literal_time_label
            )
            .is_empty(),
            "a provider date attached to the current quote is an explicit data-time context"
        );
        for historical_context in ["股价在 2025 年一度大幅波动", "股价在 30 日均线附近震荡"]
        {
            let with_history = complete.replace(
                "6. 费用、跟踪误差与底层资产估值：费率与底层估值是关键变量。",
                &format!("6. 费用、跟踪误差与底层资产估值：费率与底层估值是关键变量；{historical_context}。"),
            );
            assert!(
                missing_investment_response_sections(&contract, &with_history).is_empty(),
                "historical years or moving-average periods are not current-price claims"
            );
        }
        let wrong_price = complete.replace("30.495", "15.00");
        assert!(
            missing_investment_response_sections(&contract, &wrong_price)
                .contains(&"1. 已核验同代码现价")
        );
        let conflicting_price = complete.replace(
            "本轮同代码现价 30.495 美元",
            "现价 15.00 美元；本轮已核验同代码现价 30.495 美元",
        );
        assert!(
            missing_investment_response_sections(&contract, &conflicting_price)
                .contains(&"1. 已核验同代码现价")
        );
        let later_conflicting_price = complete.replace(
            "6. 费用、跟踪误差与底层资产估值：费率与底层估值是关键变量。",
            "6. 费用、跟踪误差与底层资产估值：费率与底层估值是关键变量；股价 15.00 美元。",
        );
        assert!(
            missing_investment_response_sections(&contract, &later_conflicting_price)
                .contains(&"1. 已核验同代码现价"),
            "a conflicting price outside section 1 must not be hidden by a correct conclusion"
        );
    }

    #[test]
    fn fund_contract_rejects_runner_financials_and_earnings_calls_for_the_fund() {
        let mut fund = entities(&["INTL"]).remove(0);
        fund.asset_type = Some("etf_or_fund".into());
        fund.profile_verified = true;
        let contract = InvestmentResponseContract {
            entities: vec![fund],
            deep_analysis: DeepAnalysisKind::Fund,
            deep_comparison: false,
            requires_verified_price: true,
            needs_outlook_evidence: false,
            comparison: false,
            origin: AgentTurnOrigin::Interactive,
        };
        let call = |data_type: &str, ticker: &str| ToolCallMade {
            name: "data_fetch".into(),
            arguments: json!({"data_type":data_type,"ticker":ticker}),
            result: json!({"data":[]}),
            tool_call_id: None,
        };
        assert!(
            !forbidden_investment_tool_calls(&contract, &[call("financials", "INTL")]).is_empty()
        );
        assert!(
            !forbidden_investment_tool_calls(&contract, &[call("earnings_calendar", "INTL")])
                .is_empty()
        );
        assert!(
            forbidden_investment_tool_calls(&contract, &[call("financials", "NBIS")]).is_empty()
        );
    }

    #[test]
    fn quote_must_match_every_resolved_symbol() {
        let quote = json!({"data":[
          {"symbol":"NBIS","price":194.09},{"symbol":"NVDA","price":201.50}
        ]});
        assert!(quote_has_positive_matching_price(&quote, "NBIS"));
        assert!(quote_has_positive_matching_price(&quote, "NVDA"));
        assert!(!quote_has_positive_matching_price(
            &json!({"data":[{"symbol":"MBIS","price":15.0}]}),
            "NBIS"
        ));
        assert!(!quote_has_positive_matching_price(
            &json!({"error":"provider failure","data":[{"symbol":"NBIS","price":194.09}]}),
            "NBIS"
        ));
    }

    #[test]
    fn earnings_calendar_provider_error_is_not_rewritten_as_an_empty_calendar() {
        let provider_error = json!({"error":"FMP provider error（HTTP 500）"});
        assert_eq!(
            matching_symbol_objects_or_error(&provider_error, "NBIS"),
            provider_error
        );
        assert_eq!(
            matching_symbol_objects_or_error(
                &json!({"data":[{"symbol":"NBIS","date":"2026-08-01"},{"symbol":"AAPL","date":"2026-08-02"}]}),
                "NBIS"
            ),
            json!([{"symbol":"NBIS","date":"2026-08-01"}])
        );
    }
}
