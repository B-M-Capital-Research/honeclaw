use std::collections::HashSet;
use std::sync::Arc;

use hone_core::ActorIdentity;
use hone_llm::Message;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::HoneBotCore;
use crate::agent_session::AgentTurnOrigin;

const EVIDENCE_ITEM_CHAR_LIMIT: usize = 6_000;
const CONTRACT_FAILURE_MESSAGE: &str =
    "这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvestmentResponseContract {
    pub entities: Vec<ResolvedSecurityEntity>,
    pub deep_single_stock: bool,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityMention {
    mention: String,
    search_query: String,
    explicit_symbol: Option<String>,
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

#[derive(Debug, Deserialize)]
struct EntityExtractionPayload {
    #[serde(default)]
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
            return format!(
                "\n\n【本轮代码级多证券比较门禁】\n已确认实体：{entity_map}。必须逐一覆盖 {}，每个标的的数值都只能来自本轮同 symbol 证据；不得用一个标的的数据代替另一个标的。回答先给比较结论，再给逐标的事实、估值/风险差异、动作条件与证伪条件。",
                self.symbols().join("、")
            );
        }
        if !self.deep_single_stock {
            return format!(
                "\n\n【本轮代码级证券数据门禁】\n已确认实体：{entity_map}。价格、估值、财务、新闻和日期数字只能使用本轮同标的证据；不得从历史对话或模型记忆补数。"
            );
        }
        format!(
            "\n\n【本轮代码级投研路由：单股深度分析，必须完整执行】\n已确认实体：{entity_map}。这不是简短行情问答。最终答案必须按以下九个编号章节逐项回答，不得合并或省略：\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量\n6. 估值（至少两种适配方法或“倍数法 + 情景法”，写清假设）\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议（买、等、减、卖、观察之一，并给触发条件）\n先给结论并标明数据时间；明确区分本轮已核验事实、推断和动作。证据没有的数字明确写“本轮未核验”，不得从历史对话或模型记忆补数。"
        )
    }

    pub(crate) fn retry_block(&self, missing: &[&'static str]) -> String {
        if self.comparison {
            return format!(
                "\n\n【上一版多标的比较草稿已被代码级完整性检查拒绝】\n缺失或不合格项：{}。重新生成完整比较，必须逐一覆盖 {}，标明数据时间，并区分事实、推断、动作和证伪条件；不得解释检查过程。",
                missing.join("、"),
                self.symbols().join("、")
            );
        }
        format!(
            "\n\n【上一版草稿已被代码级完整性检查拒绝】\n缺失或不合格章节：{}。重新生成完整最终答案，严格使用九个编号章节；不得解释检查过程，不得用追问持仓成本代替动作建议。",
            missing.join("、")
        )
    }
}

pub(crate) fn contract_failure_message() -> &'static str {
    CONTRACT_FAILURE_MESSAGE
}

fn response_intent(input: &str) -> (bool, bool) {
    let normalized = input.to_ascii_lowercase();
    let deep = [
        "分析",
        "研究",
        "怎么看",
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
                return Err(format!(
                    "我暂时无法确认你提到的“{}”对应哪家上市公司或证券。请补充公司全名或 ticker。",
                    mention.mention
                ));
            }
        }
    }
    if entities.is_empty() {
        return Ok(None);
    }
    let (deep_intent, needs_outlook_evidence) = response_intent(user_input);
    let comparison = entities.len() > 1;
    let contract = InvestmentResponseContract {
        deep_single_stock: origin == AgentTurnOrigin::Interactive && deep_intent && !comparison,
        needs_outlook_evidence,
        comparison,
        origin,
        entities,
    };
    let symbols = contract
        .entities
        .iter()
        .map(|entity| entity.symbol.as_str())
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
    for symbol in &symbols {
        if !quote_has_positive_matching_price(&quote, symbol) {
            return Err(format!(
                "{symbol} 的最新同标的行情尚未完成确认。本轮不会基于不确定价格给出投资结论。"
            ));
        }
    }

    let mut evidence = vec![("最新行情", quote)];
    if contract.deep_single_stock {
        let symbol = &contract.entities[0].symbol;
        let (profile, financials, news) = tokio::join!(
            registry.execute_tool(
                "data_fetch",
                json!({"data_type": "profile", "ticker": symbol}),
            ),
            registry.execute_tool(
                "data_fetch",
                json!({"data_type": "financials", "ticker": symbol}),
            ),
            registry.execute_tool("data_fetch", json!({"data_type": "news", "ticker": symbol}),),
        );
        let financials = financials.map_err(|err| format!("{symbol} 财务核验失败：{err}"))?;
        if !has_nonempty_data(&financials) {
            return Err(format!(
                "当前无法稳定核验 {symbol} 的本轮财务数据，已停止生成完整估值结论。"
            ));
        }
        evidence.push(("公司概况", result_or_error_value(profile)));
        evidence.push(("财务数据", financials));
        evidence.push(("公司新闻", result_or_error_value(news)));
    }
    if contract.comparison && origin == AgentTurnOrigin::Interactive && deep_intent {
        for entity in &contract.entities {
            let symbol = &entity.symbol;
            let (profile, financials) = tokio::join!(
                registry.execute_tool(
                    "data_fetch",
                    json!({"data_type": "profile", "ticker": symbol})
                ),
                registry.execute_tool(
                    "data_fetch",
                    json!({"data_type": "financials", "ticker": symbol})
                ),
            );
            let financials = financials
                .map_err(|_| format!("{symbol} 的财务数据查询暂时不可用，请稍后重试。"))?;
            if !has_nonempty_data(&financials) {
                return Err(format!(
                    "{symbol} 的本轮财务数据尚未完成确认，暂不能进行可靠的多标的估值比较。"
                ));
            }
            evidence.push(("公司概况", result_or_error_value(profile)));
            evidence.push(("财务数据", financials));
        }
    }
    if contract.needs_outlook_evidence && contract.entities.len() <= 5 {
        let from = hone_core::beijing_now().date_naive();
        let to = from + chrono::Duration::days(120);
        for entity in &contract.entities {
            let symbol = &entity.symbol;
            let calendar = registry.execute_tool("data_fetch", json!({"data_type": "earnings_calendar", "ticker": symbol, "from": from.format("%Y-%m-%d").to_string(), "to": to.format("%Y-%m-%d").to_string()})).await;
            evidence.push((
                "未来 120 天财报日历（仅当前标的）",
                matching_symbol_objects(&result_or_error_value(calendar), symbol),
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
    if !(text.contains("北京时间") || text.contains("数据时间")) {
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
    let valuation_method_count = [
        ["p/s", "ps 倍", "ps估值"].as_slice(),
        ["p/e", "pe 倍", "pe估值"].as_slice(),
        ["ev/ebitda", "ev / ebitda"].as_slice(),
        ["fcf yield", "自由现金流收益率"].as_slice(),
        ["dcf", "现金流折现"].as_slice(),
        ["sotp", "分部估值"].as_slice(),
        ["情景法", "情景分析"].as_slice(),
    ]
    .iter()
    .filter(|aliases| aliases.iter().any(|alias| text.contains(alias)))
    .count();
    if valuation_method_count < 2 {
        missing.push("至少两种估值方法");
    }
    missing
}

pub(crate) fn missing_investment_response_sections(
    contract: &InvestmentResponseContract,
    content: &str,
) -> Vec<&'static str> {
    if contract.deep_single_stock {
        return missing_deep_single_stock_sections(content);
    }
    if !contract.comparison {
        return Vec::new();
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
    require_any(&lower, &["风险", "证伪"], "风险与证伪条件", &mut missing);
    missing
}

fn has_numbered_section(content: &str, number: u8) -> bool {
    Regex::new(&format!(
        r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{number}\s*[.、)]"
    ))
    .expect("numbered section regex")
    .is_match(content)
}

fn require_any(
    content: &str,
    keywords: &[&str],
    label: &'static str,
    missing: &mut Vec<&'static str>,
) {
    if !keywords.iter().any(|keyword| content.contains(keyword)) {
        missing.push(label);
    }
}

async fn extract_entity_mentions(
    core: &Arc<HoneBotCore>,
    input: &str,
    origin: AgentTurnOrigin,
) -> Result<Vec<EntityMention>, String> {
    let explicit = explicit_dollar_mentions(input);
    if !explicit.is_empty() {
        return Ok(explicit);
    }
    if !should_run_entity_stage(input, origin) {
        return Ok(Vec::new());
    }
    let Some(llm) = core.auxiliary_llm.as_ref() else {
        return complete_entity_extraction(input, Vec::new());
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
        Ok(response) => {
            let entities = parse_entity_extraction(&response.content)
                .map_err(|_| "证券实体识别结果不完整。请补充公司全名或明确 ticker。".to_string())?;
            complete_entity_extraction(input, entities)
        }
        Err(_) => complete_entity_extraction(input, Vec::new()),
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
        })
        .collect()
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
        || analysis_has_named_subject(input)
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
    let normalized = input.to_ascii_lowercase();
    let broad_scope = [
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
        "sector",
        "industry",
        "macro",
        "index",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    if broad_scope {
        return Ok(Vec::new());
    }
    Err("我暂时无法从当前问题中确认具体公司或证券。请补充公司全名或 ticker。".to_string())
}

fn parse_entity_extraction(content: &str) -> Result<Vec<EntityMention>, serde_json::Error> {
    let trimmed = content.trim();
    let json_text = match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(start), Some(end)) if start <= end => &trimmed[start..=end],
        _ => trimmed,
    };
    let payload: EntityExtractionPayload = serde_json::from_str(json_text)?;
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
        .map(str::to_string);
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
    }
}

fn quote_has_positive_matching_price(value: &Value, symbol: &str) -> bool {
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
            (symbol_ok && price_ok)
                || map
                    .values()
                    .any(|child| quote_has_positive_matching_price(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .any(|child| quote_has_positive_matching_price(child, symbol)),
        _ => false,
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
        EntityMatch, EntityMention, InvestmentResponseContract, ResolvedSecurityEntity,
        complete_entity_extraction, explicit_dollar_mentions, missing_deep_single_stock_sections,
        missing_investment_response_sections, parse_entity_extraction,
        quote_has_positive_matching_price, resolve_entity_match, response_intent,
        should_run_entity_stage,
    };
    use crate::agent_session::AgentTurnOrigin;
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
    fn uppercase_metadata_is_not_used_as_a_ticker_fallback() {
        let result =
            complete_entity_extraction("REPEAT=30m，检查 API 状态后生成 AI 主题摘要", Vec::new());
        assert!(result.is_err());
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
        assert_eq!(
            response_intent("我想了解Q3的时候NBIS能不能起飞"),
            (true, true)
        );
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
            })
            .collect()
    }

    #[test]
    fn multi_entity_contract_and_final_validator_cover_every_symbol() {
        let contract = InvestmentResponseContract {
            entities: entities(&["AMD", "NVDA"]),
            deep_single_stock: false,
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
                "数据时间：今天。AMD 与 NVDA 已逐一比较。风险与证伪条件如下"
            )
            .is_empty()
        );
    }

    #[test]
    fn scheduler_contract_uses_typed_origin_not_envelope_text() {
        let contract = InvestmentResponseContract {
            entities: entities(&["NBIS"]),
            deep_single_stock: false,
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
        let complete = "数据时间：北京时间 2026-07-16。事实与推断分开。\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量与自由现金流\n6. 估值：P/S + 情景法，假设如下\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议";
        assert!(missing_deep_single_stock_sections(complete).is_empty());
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
    }
}
