//! Prompt helpers for channel-specific guidance.

use chrono::{DateTime, FixedOffset};
use hone_core::config::HoneConfig;
use hone_memory::SessionStorage;
use hone_memory::session::SessionPromptState;
use serde::Deserialize;
use std::sync::OnceLock;

pub const DEFAULT_GROUP_PRIVACY_GUARD: &str = "【群聊隐私约束】在群聊中禁止要求用户提供持仓、成交价、交易单等敏感信息；如需明细，引导其转为私聊提交。";

/// The language the user is reading the product in. It is a user-interface
/// fact carried per request, not a server-wide setting: two users on the same
/// deployment can be reading different languages at the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyLanguage {
    Chinese,
    English,
}

impl ReplyLanguage {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "zh" => Some(Self::Chinese),
            "en" => Some(Self::English),
            _ => None,
        }
    }
}

const COMPANY_RESEARCH_CARDS_JSON: &str =
    include_str!("../../../skills/company-thesis-ratings/references/company-cards.json");
const COMPANY_RESEARCH_INDEX_JSON: &str =
    include_str!("../../../skills/company-thesis-ratings/references/company-index.json");
const MAX_PROJECTED_COMPANY_CARDS: usize = 8;

/// 行业块是公司卡之外的第二份每轮注入，两块叠加就是这一轮的固定开销。一家公司常常
/// 同时落在两行（博通既是 AI 芯片也是网络），第三行开始基本是噪声。
const MAX_PROJECTED_INDUSTRIES: usize = 2;
/// 只带最该盯的那几条；`core_watch` 完整清单在 `industry-map` skill 里，模型要展开时自己加载。
const MAX_PROJECTED_INDUSTRY_WATCH: usize = 3;
/// 上游信号只注入两条，NVDA 永远在前：它是本体里最粗的那条边，也是用户点名的失败点。
const MAX_PROJECTED_UPSTREAM_SIGNALS: usize = 2;

/// 按字符截断并补省略号；注入面里的散文只保留一句，完整版留给页面与 skill。
fn clip(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    trimmed.chars().take(limit).collect::<String>() + "…"
}

#[derive(Debug, Deserialize)]
struct CompanyResearchCorpus {
    companies: Vec<CompanyResearchCard>,
}

#[derive(Debug, Deserialize)]
struct CompanyResearchCard {
    name: String,
    symbol: String,
    business_model: String,
    moat: String,
    thesis_summary: String,
    valuation_method: String,
    confidence: String,
    #[serde(default)]
    watch_items: Vec<String>,
    #[serde(default)]
    risks: Vec<String>,
    #[serde(default)]
    falsifiers: Vec<String>,
    source_updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CompanyResearchIndex {
    companies: Vec<CompanyAliasEntry>,
}

#[derive(Debug, Deserialize)]
struct CompanyAliasEntry {
    symbol: String,
    aliases: Vec<String>,
}

fn company_research_resources() -> &'static (CompanyResearchCorpus, CompanyResearchIndex) {
    static RESOURCES: OnceLock<(CompanyResearchCorpus, CompanyResearchIndex)> = OnceLock::new();
    RESOURCES.get_or_init(|| {
        let cards = serde_json::from_str(COMPANY_RESEARCH_CARDS_JSON)
            .expect("embedded company research cards must remain valid JSON");
        let index = serde_json::from_str(COMPANY_RESEARCH_INDEX_JSON)
            .expect("embedded company research index must remain valid JSON");
        (cards, index)
    })
}

fn ascii_term_match(input: &str, term: &str) -> bool {
    if term.is_empty() {
        return false;
    }
    let input_lower = input.to_ascii_lowercase();
    let term_lower = term.to_ascii_lowercase();
    input_lower.match_indices(&term_lower).any(|(start, _)| {
        let end = start + term_lower.len();
        let before_ok = input_lower[..start]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_ascii_alphanumeric());
        let after_ok = input_lower[end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_ascii_alphanumeric());
        before_ok && after_ok
    })
}

fn explicit_symbol_match(input: &str, symbol: &str) -> bool {
    // Short tickers normally demand exact case, because standalone lowercase
    // words like "app" or "be" are ordinary English prose. A query written in
    // Chinese has no such prose, so a bare ASCII token there is the ticker —
    // and users type it lowercase ("mu 现在的估值贵不贵"), which used to miss
    // the company card entirely.
    let cjk_context = input.chars().any(
        |character| matches!(character as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF),
    );
    input
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .any(|token| {
            token.eq_ignore_ascii_case(symbol)
                && (symbol.len() >= 4
                    || token == symbol
                    || cjk_context
                    || input.contains(&format!("${token}")))
        })
}

fn alias_match(input: &str, alias: &str) -> bool {
    if alias.chars().any(|character| !character.is_ascii()) {
        input.contains(alias)
    } else {
        ascii_term_match(input, alias)
    }
}

/// 本轮问题命中行业树时，给出这一行的传导链、倍数锚与反模式。
///
/// 命中方式有两条：用户直接说行业（「光通信现在怎么样」），或者说了这一行里的某家公司
/// （「闪迪的估值」）。后一条是这块内容的主要用途——个股分析里「需求从哪来」那一段
/// 本该用行业的量价驱动来写，而不是每家公司重讲一遍 AI 故事。
///
/// 传导链还没定稿的行业不注入：空块占着 token 却什么都没说，还会让模型以为这一行没有逻辑。
pub(crate) fn industry_baseline(user_input: &str, data_root: &std::path::Path) -> Option<String> {
    // 底稿 + 管理员在对话里改出来的那份日志：研究台看到什么，这里就注入什么。
    // 数据根由调用方给：这个模块没有状态，而改动日志和研究台读的必须是同一份。
    let (map, _) = hone_core::industry_map::load(data_root);
    let mut sections = Vec::new();
    for industry in &map.industries {
        if industry.ai_valuation_logic.driver_chain.trim().is_empty()
            && industry.valuation.logic.summary.trim().is_empty()
        {
            continue;
        }
        let matched = industry
            .members
            .iter()
            .filter(|member| {
                explicit_symbol_match(user_input, &member.symbol)
                    || alias_match(user_input, &member.name)
            })
            .collect::<Vec<_>>();
        let by_alias = industry
            .aliases
            .iter()
            .any(|alias| alias_match(user_input, alias));
        if matched.is_empty() && !by_alias {
            continue;
        }

        let logic = &industry.ai_valuation_logic;
        let valuation = &industry.valuation;
        // 命中公司所属的子类型：知识被消费的最小单位。多家命中时取第一家的子类型，
        // 其余公司若在别的子类型里，只点名它们的子类型名，不整段重复。
        let primary_member = matched.first();
        let subtype = primary_member.and_then(|member| valuation.subtype_of(&member.symbol));
        let mut head = format!("- {}（{}）", industry.name, industry.id);
        if !matched.is_empty() {
            head.push_str(&format!(
                " · 本轮命中：{}",
                matched
                    .iter()
                    .map(|member| {
                        let tag = valuation
                            .subtype_of(&member.symbol)
                            .map(|s| format!("｜{}", s.name))
                            .unwrap_or_default();
                        format!("{}（{}{}）", member.symbol, member.name, tag)
                    })
                    .collect::<Vec<_>>()
                    .join("、")
            ));
        }
        let mut lines = vec![head];

        // 1) 本体的边：上游最近动作先给事实（带日期），再给核对取法。只带两条，NVDA 永远第一。
        let mut signals = industry.upstream_signals.iter().collect::<Vec<_>>();
        signals.sort_by_key(|signal| signal.symbol != "NVDA");
        for signal in signals.into_iter().take(MAX_PROJECTED_UPSTREAM_SIGNALS) {
            if !signal.latest.trim().is_empty() {
                let as_of = if signal.latest_as_of.trim().is_empty() {
                    String::new()
                } else {
                    format!("，截至 {}", signal.latest_as_of.trim())
                };
                lines.push(format!(
                    "  上游最近动作 · {}（{}，{}{}）：{}",
                    signal.symbol,
                    signal.name,
                    signal.relation,
                    as_of,
                    clip(&signal.latest, 240)
                ));
            }
            let pull = if signal.pull.is_empty() {
                "最新一季财报与指引".to_string()
            } else {
                signal
                    .pull
                    .iter()
                    .take(3)
                    .map(|item| clip(item, 40))
                    .collect::<Vec<_>>()
                    .join("；")
            };
            lines.push(format!(
                "  写这家之前先核对 {} 有没有更新的一季：`data_fetch(data_type=\"earnings_outlook\", ticker=\"{}\")`（评级与目标价 `analyst_actions`，原话 `transcript` / `press_releases`）；要读的是 {}。{}",
                signal.symbol,
                signal.symbol,
                pull,
                clip(&signal.why, 80)
            ));
        }

        // 2) 估值执行卡：这家公司该用哪个前瞻财年、哪一族倍数、区间由什么决定、什么被禁止。
        //    先给子类型（命中公司专属），子类型缺失时退到行级倍数锚。
        if let Some(subtype) = subtype {
            let mut card = format!(
                "  估值执行卡 · 子类型「{}」：主锚 {}",
                subtype.name,
                clip(&subtype.primary, 160)
            );
            if !subtype.secondary.trim().is_empty() {
                card.push_str(&format!("；次锚 {}", clip(&subtype.secondary, 120)));
            }
            if !subtype.when.trim().is_empty() {
                card.push_str(&format!("；适用阶段：{}", clip(&subtype.when, 120)));
            }
            if !subtype.note.trim().is_empty() {
                card.push_str(&format!("；这家的特别提醒：{}", clip(&subtype.note, 200)));
            }
            lines.push(card);
        } else if !valuation.anchor.paragraphs.is_empty() {
            lines.push(format!(
                "  估值执行卡 · 行级倍数锚：{}",
                clip(&valuation.anchor.paragraphs[0], 300)
            ));
        } else {
            let anchor = if logic.multiple_anchor_short.trim().is_empty() {
                &logic.multiple_anchor
            } else {
                &logic.multiple_anchor_short
            };
            if !anchor.trim().is_empty() {
                lines.push(format!("  倍数锚：{anchor}"));
            }
        }
        if !valuation.anchor.upper_range_drivers.trim().is_empty() {
            lines.push(format!(
                "  倍数上沿由什么决定：{}",
                clip(&valuation.anchor.upper_range_drivers, 220)
            ));
        }
        if !valuation.anchor.revision_optionality.trim().is_empty() {
            lines.push(format!(
                "  盈利上修期权（提高 FY+2 权重或取区间上半部的条件）：{}",
                clip(&valuation.anchor.revision_optionality, 260)
            ));
        }
        let mut forbidden = valuation
            .anchor
            .forbidden
            .iter()
            .take(4)
            .map(|item| clip(item, 60))
            .collect::<Vec<_>>();
        let anti = if logic.anti_pattern_short.trim().is_empty() {
            &logic.anti_pattern
        } else {
            &logic.anti_pattern_short
        };
        if forbidden.is_empty() && !anti.trim().is_empty() {
            forbidden.push(anti.clone());
        }
        if !forbidden.is_empty() {
            lines.push(format!("  这一行禁止的估值动作：{}", forbidden.join("；")));
        }

        // 3) 底层估值逻辑：第一性原理一句 + 公式 + 前瞻重点；再带上带日期的需求传导链。
        if !valuation.logic.summary.trim().is_empty() {
            lines.push(format!(
                "  底层估值逻辑：{}",
                clip(&valuation.logic.summary, 260)
            ));
        }
        let formulas = valuation
            .logic
            .formulas
            .iter()
            .take(2)
            .map(|f| clip(&f.formula, 110))
            .collect::<Vec<_>>();
        if !formulas.is_empty() {
            lines.push(format!("  量化关系：{}", formulas.join("；")));
        }
        let focus = valuation
            .logic
            .forward_focus
            .iter()
            .take(3)
            .map(|item| clip(item, 90))
            .collect::<Vec<_>>();
        if !focus.is_empty() {
            lines.push(format!("  未来 1–3 年先看：{}", focus.join("；")));
        }
        if !valuation.logic.state_note.trim().is_empty() {
            lines.push(format!(
                "  这一行典型的 State：{}",
                clip(&valuation.logic.state_note, 200)
            ));
        }
        if !valuation.upstream_summary.trim().is_empty() {
            lines.push(format!(
                "  上游信号（需求怎么传到本行）：{}",
                clip(&valuation.upstream_summary, 220)
            ));
        }
        if !valuation.transmission.trim().is_empty() {
            lines.push(format!(
                "  变量传导与证伪：{}",
                clip(&valuation.transmission, 200)
            ));
        }
        let observables = valuation
            .observables
            .iter()
            .take(5)
            .map(|item| clip(&item.name, 16))
            .collect::<Vec<_>>();
        if !observables.is_empty() {
            lines.push(format!(
                "  可观测变量（每条绑定公司、期间、单位、出处，未公开的标缺失，不拿行业代理量冒充本公司实测）：{}",
                observables.join("、")
            ));
        }
        if !logic.driver_chain.trim().is_empty() {
            lines.push(format!(
                "  需求传导链（带日期的量）：{}",
                logic.driver_chain
            ));
        }
        let watch = industry
            .core_watch
            .iter()
            .take(MAX_PROJECTED_INDUSTRY_WATCH)
            .map(|item| {
                if item.cadence.trim().is_empty() {
                    item.what.clone()
                } else {
                    format!("{}（{}）", item.what, item.cadence)
                }
            })
            .collect::<Vec<_>>();
        if !watch.is_empty() {
            lines.push(format!("  核心关注点：{}", watch.join("；")));
        }
        sections.push(lines.join("\n"));
        if sections.len() >= MAX_PROJECTED_INDUSTRIES {
            break;
        }
    }
    if sections.is_empty() {
        return None;
    }
    // 通用执行规则的压缩版：只带能改变分母与倍数选择的那几条；全文在 `industry-map` skill。
    let rules = map
        .methodology
        .execution_rules
        .iter()
        .filter(|rule| {
            matches!(
                rule.rule.as_str(),
                "远期收入的证据等级"
                    | "共识与市场起点"
                    | "利润与普通股资本桥"
                    | "估值贡献拆解"
                    | "双重上修与不重估检验"
                    | "结论与红绿灯边界"
            )
        })
        .map(|rule| format!("{}：{}", rule.rule, clip(&rule.requirement, 170)))
        .collect::<Vec<_>>();
    let rules_block = if rules.is_empty() {
        String::new()
    } else {
        format!(
            "\n全局规则（压缩版，全文在 `industry-map` skill）：{}。",
            rules.join("；")
        )
    };
    let fields_block = if map.methodology.output_fields.is_empty() {
        String::new()
    } else {
        format!(
            "\n估值类终稿按 V5.3 的十二个执行字段落笔（就落在对账表、三问、三情景、反向估值、结论的现有位置里，不另起一套；缺的写「未取得 / 未完成」，不填虚构数字）：{}。",
            map.methodology
                .output_fields
                .iter()
                .map(|field| field.rule.clone())
                .collect::<Vec<_>>()
                .join("、")
        )
    };
    Some(format!(
        "【本轮相关行业 · HOne Industry Map V5.3】\n以下来自 `industry-map` 的行业本体（AI 数据中心八行 + 商业太空 + AI 应用与数据服务），是这一行的**结构与先验**，不是当前事实。\
需求侧照底层估值逻辑与传导链写，不要另起一套 AI 叙事；链条上游的量对同一行所有公司共用，差异出现在份额、认证、产能或合约这些闸门上，\
不得把行业增速直接当成公司增速。\
带「上游最近动作」的行，那一行是本文的起点事实，要落进终稿已有的位置而不是另起一段：公司研究稿落在「行业位置与关键对手」一节的第一句，估值稿落在对账表的「上游最近动作」行、一问的增长来源第一句和基准情景的第一项经营输入——每处都就从那条最近动作写起（日期和数字都带上），再写它沿传导链怎么到这家公司，最后用本轮取到的更新一季覆盖它；没有更新就照本体这条写并注明截至日期。上游不得略过，也不得用记忆里的旧季度代替。\
估值执行卡决定分母与倍数：先按这一行的典型经营状态判断这家处在哪个阶段；远期收入按项目 / 批次 / 客户群逐项评 A / B / C 级（A 按履约曲线进基准，B 部分进附条件基准，C 只进条件期权或乐观情景），FY+2 / FY+3 进基准要分别核需求、资格、供应链、建设验收、单位经济性、融资、许可，不因价格更低就换到更远财年；主锚次锚照子类型卡给，不强制 PE 优先，也不因利润暂低自动抬 Sales 倍数，EV/Sales 必须旁列同年度隐含 EV/EBIT；当前共识倍数与当前模型倍数分别标注，缺共识就把相对共识的上修标「未知」；盈利上修与倍数扩张分别举证并写交互项（总价差 = e + r + e×r），保留倍数不变结果；资本桥不闭合就只给条件 EV 与融资敏感性，不发正式每股目标价；DCF 定价权重为零但时间换算与资本回报检查照做。公司卡指定了估值框架时以公司卡为准，子类型卡做交叉检查。\
永久规则不存放最新一季数字：下面带日期的量与上游最近动作是已核验的事实层，规则是方法层，两者不混，也不把行业代理量当成本公司的实测。{rules_block}{fields_block}\n\n{}",
        sections.join("\n\n")
    ))
}

pub(crate) fn company_research_baseline(user_input: &str) -> Option<String> {
    let (corpus, index) = company_research_resources();
    let mut symbols = index
        .companies
        .iter()
        .filter(|entry| {
            explicit_symbol_match(user_input, &entry.symbol)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| alias_match(user_input, alias))
        })
        .map(|entry| entry.symbol.as_str())
        .take(MAX_PROJECTED_COMPANY_CARDS)
        .collect::<Vec<_>>();
    symbols.sort_unstable();
    symbols.dedup();
    if symbols.is_empty() {
        return None;
    }

    let mut sections = Vec::new();
    for symbol in symbols {
        let Some(card) = corpus.companies.iter().find(|card| card.symbol == symbol) else {
            continue;
        };
        let joined = |values: &[String]| {
            if values.is_empty() {
                "未记录".to_string()
            } else {
                values.join("；")
            }
        };
        sections.push(format!(
            "- {symbol} / {name}（历史研究基线更新：{updated}；基线置信度：{confidence}）\n  商业模式：{business}\n  护城河：{moat}\n  长期主线：{thesis}\n  估值框架：{valuation}\n  重点验证：{watch}\n  风险：{risks}\n  证伪条件：{falsifiers}",
            symbol = card.symbol,
            name = card.name,
            updated = card.source_updated_at,
            confidence = card.confidence,
            business = card.business_model,
            moat = card.moat,
            thesis = card.thesis_summary,
            valuation = card.valuation_method,
            watch = joined(&card.watch_items),
            risks = joined(&card.risks),
            falsifiers = joined(&card.falsifiers),
        ));
    }
    if sections.is_empty() {
        return None;
    }

    Some(format!(
        "【历史公司研究基线】\n以下内容来自此前授权研究材料形成的 `company-thesis-ratings` 压缩公司卡，只用于商业模式、基本面结构、护城河、产业链位置、估值框架与方法选择、风险与证伪条件的历史判断基线。它不是当前事实、最新观点或交易指令。必须实际加载 `company-thesis-ratings` 和 `hari-invest` 后再回答。股价、最新财报、指引、订单、新闻、产业状态与估值输入继续使用本轮原有证据工具核验；冲突时以最新一手证据为准，并说明历史逻辑加强、削弱或失效。不得向用户暴露内部逐字稿、文件名或 Skill 路径。\n\n{}",
        sections.join("\n\n")
    ))
}

#[derive(Debug, Clone)]
pub struct PromptOptions {
    pub is_admin: bool,
    pub admin_prompt: Option<String>,
    pub privacy_guard: Option<String>,
    pub model_hint: Option<String>,
    /// Language the user is reading the product in. `None` lets the Agent
    /// follow the conversation instead of being pinned.
    pub reply_language: Option<ReplyLanguage>,
    pub extra_sections: Vec<String>,
    pub include_format_guidance: bool,
}

impl Default for PromptOptions {
    fn default() -> Self {
        Self {
            is_admin: false,
            admin_prompt: None,
            privacy_guard: None,
            model_hint: None,
            reply_language: None,
            extra_sections: Vec::new(),
            include_format_guidance: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptBundle {
    pub static_system: String,
    /// Runtime/turn guidance moved out of the system prompt and injected into
    /// the user prompt (channel formatting, privacy guard, reply language,
    /// model hint, admin note, and per-turn extra sections such as baselines
    /// or skill hints).
    pub turn_guidance: String,
    pub session_context: String,
    /// One timestamp generated with the Session context and reused by every
    /// current-turn answer contract. Keeping it structured prevents a
    /// cross-minute second clock read from changing the required first line.
    pub answer_time_local: String,
    pub conversation_context: Option<String>,
}

impl PromptBundle {
    pub fn system_prompt(&self) -> String {
        self.static_system.clone()
    }

    pub fn compose_user_input(&self, user_input: &str) -> String {
        let mut sections = Vec::new();

        if let Some(guidance) = Some(self.turn_guidance.trim()).filter(|value| !value.is_empty()) {
            sections.push(guidance.to_string());
        }

        if let Some(context) = self
            .conversation_context
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            sections.push(context.to_string());
        }

        if let Some(session_context) =
            Some(self.session_context.trim()).filter(|value| !value.is_empty())
        {
            sections.push(session_context.to_string());
        }

        sections.push(format!("【本轮用户输入】\n{}", user_input.trim()));

        sections.join("\n\n")
    }
}

pub fn default_admin_prompt(project_root: &str) -> String {
    format!(
        "【管理员权限】\
        \n你正在与管理员用户交互。\
        \n1. 当前渠道运行在隔离沙盒内，默认不能直接浏览项目源码；如需运维动作，优先使用平台已暴露的工具。\
        \n2. 可调用 restart_hone(confirm=\"yes\") 工具重启 Hone（将重新编译并启动）\
        \n3. 如需人工排查源码，仓库根目录仍为：{project_root}\
        \n4. 管理员操作须谨慎，执行重启前应先确认影响范围\
        \n如需管理员操作指引，请使用 Hone Administrator 技能；技能正文由当前 Agent 的技能机制按需加载。"
    )
}

pub async fn build_prompt_bundle(
    config: &HoneConfig,
    storage: &SessionStorage,
    channel: &str,
    session_id: &str,
    _prompt_state: &SessionPromptState,
    options: &PromptOptions,
) -> PromptBundle {
    build_prompt_bundle_at(
        config,
        storage,
        channel,
        session_id,
        _prompt_state,
        options,
        hone_core::local_now(),
        true,
    )
    .await
}

pub(crate) async fn build_prompt_bundle_at(
    _config: &HoneConfig,
    storage: &SessionStorage,
    channel: &str,
    session_id: &str,
    _prompt_state: &SessionPromptState,
    options: &PromptOptions,
    now: DateTime<FixedOffset>,
    include_conversation_context: bool,
) -> PromptBundle {
    // Interactive/main harness leaves system_prompt empty; domain policies live
    // in on-demand skills, and per-turn runtime guidance is placed in the user
    // prompt via `turn_guidance`.
    let static_system = String::new();
    let mut guidance_sections: Vec<String> = Vec::new();

    if options.include_format_guidance {
        let format_guidance = channel_format_guidance(channel);
        if !format_guidance.is_empty() {
            guidance_sections.push(format_guidance.to_string());
        }
    }

    if let Some(guard) = options
        .privacy_guard
        .as_ref()
        .filter(|s| !s.trim().is_empty())
    {
        guidance_sections.push(guard.trim().to_string());
    }

    if let Some(model_hint) = options.model_hint.as_ref().filter(|s| !s.trim().is_empty()) {
        guidance_sections.push(format!("【基础模型】{}。", model_hint.trim()));
    }

    match options.reply_language {
        Some(ReplyLanguage::Chinese) => guidance_sections.push(
            "【语言要求】必须全程以中文回复，不得应答其他语言；但证券代码（NVDA、0700.HK）、财务与估值缩写（P/E、EV/EBITDA、Capex、TTM、GAAP / Non-GAAP、ARR、FCF）、技术与产业术语（GPU、HBM、CoWoS、ASIC）、阿拉伯数字与货币单位（$89.0B、1,553.40 美元、+117%）一律保持原样，不音译、不用汉字数字、不把缩写改写成中文全称。中文正文里夹这些原样实体不算混排。"
                .to_string(),
        ),
        Some(ReplyLanguage::English) => guidance_sections.push(
            "【语言要求】The user is reading this product in English. Reply entirely in English, including section headings and disclosures. Keep tickers, exchange codes and provider field names in their original form."
                .to_string(),
        ),
        None => {}
    }

    if options.is_admin {
        let admin_note = if let Some(custom) = options
            .admin_prompt
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            custom.trim().to_string()
        } else {
            let project_root = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string());
            default_admin_prompt(&project_root)
        };
        guidance_sections.push(admin_note);
    }

    for section in &options.extra_sections {
        if !section.trim().is_empty() {
            guidance_sections.push(section.trim().to_string());
        }
    }

    let turn_guidance = guidance_sections.join("\n\n");

    let session_context = format!(
        "【Session 上下文】\n当前时间：{} (运行时时区：{})\n当前日期：{}\n当前年份：{}\n会话 ID：{}",
        now.format("%Y-%m-%d %H:%M:%S"),
        hone_core::runtime_timezone_name(),
        now.format("%Y-%m-%d"),
        now.format("%Y"),
        session_id
    );

    let conversation_context = if include_conversation_context {
        storage
            .load_session(session_id)
            .await
            .ok()
            .flatten()
            .and_then(|session| {
                hone_memory::latest_compact_summary(&session.messages)
                    .map(|message| {
                        hone_memory::session_message_text(message)
                            .trim()
                            .to_string()
                    })
                    .or_else(|| {
                        session
                            .summary
                            .map(|summary| format!("【历史会话总结】\n{}", summary.content.trim()))
                    })
            })
            .filter(|value| !value.trim().is_empty())
    } else {
        None
    };

    PromptBundle {
        static_system,
        turn_guidance,
        session_context,
        answer_time_local: now.format("%Y-%m-%d %H:%M").to_string(),
        conversation_context,
    }
}

pub fn channel_format_guidance(channel: &str) -> &'static str {
    match channel {
        "discord" => DISCORD_FORMAT_GUIDANCE,
        "telegram" => TELEGRAM_FORMAT_GUIDANCE,
        "imessage" => IMESSAGE_FORMAT_GUIDANCE,
        "feishu" => FEISHU_FORMAT_GUIDANCE,
        _ => "",
    }
}

const DISCORD_FORMAT_GUIDANCE: &str = "【输出格式-Discord】\n\
- Discord 支持 Markdown：标题（# / ## / ###）、粗体/斜体/下划线/删除线、代码块、引用、列表与 Spoiler。\n\
- 标题仅用 #/##/###；更高层级不在官方指引内，部分客户端也有渲染问题，避免使用。\n\
- 不支持 Markdown 表格；需要表格时改为列表或代码块。\n\
- 社区反馈列表缩进可能失效，避免嵌套列表。";

const TELEGRAM_FORMAT_GUIDANCE: &str = "【输出格式-Telegram】\n\
- 当前使用 HTML 解析（parse_mode=HTML）。\n\
- 必须输出纯 HTML，不要混入任何 Markdown 语法；禁止出现 #、##、###、**粗体**、__粗体__、*斜体*、_斜体_、```代码块```、`行内代码` 这类 Markdown 标记。\n\
- 支持的核心标签：<b>/<strong>、<i>/<em>、<u>/<ins>、<s>/<strike>/<del>、<code>、<pre>、<a href=\"...\">。\n\
- 还可用 <span class=\"tg-spoiler\"> 或 <tg-spoiler> 做 spoiler；<blockquote> / <blockquote expandable> 做引用与可折叠引用。\n\
- 可用 <tg-emoji emoji-id=\"...\"> 和 <tg-time unix=\"...\" format=\"...\"> 做自定义 emoji 与时间展示，但仅在确实有价值时使用。\n\
- 代码块建议写成 <pre><code class=\"language-python\">...</code></pre> 这类嵌套结构；单独 <code> 不支持指定语言。\n\
- 文本中的 <、>、& 必须转义为 &lt;、&gt;、&amp;；除了官方支持标签外，不要使用其他 HTML 标签。\n\
- 未文档化表格支持，避免表格；需要表格时用 <pre>/<code> 生成等宽伪表格，或改为分行列表。\n\
- 如果原本想写 Markdown 标题或列表，请改成 HTML：标题用 <b>...</b>，列表用纯文本项目符号或分行，不要输出 Markdown 列表标记。\n\
- 段落粒度保持轻快，优先用短标题 + 分行列表 + 代码块/引用，避免一段写得过长。这是排版粒度要求，不是篇幅上限：投研终稿该有的研究深度与字数按投研约束执行，不得为了“看起来简洁”削减已核验证据、算式或展开。";

const IMESSAGE_FORMAT_GUIDANCE: &str = "【输出格式-iMessage】\n\
- iMessage 文本格式依赖客户端（加粗/斜体/下划线/删除线等），其他设备可能仅显示纯文本。\n\
- 因此输出只用纯文本与清晰换行；不要依赖 Markdown 语法；列表用 1. 2. 3. 表达。";

const FEISHU_FORMAT_GUIDANCE: &str = "【输出格式-飞书】\n\
- 飞书富文本/卡片支持 Markdown 语法扩展，支持链接、图片、表格等元素，但有明确限制。\n\
- 标题仅支持一级与二级；列表不支持缩进；表格有列数与数量上限。\n\
- 当前渠道使用卡片渲染 Markdown，请保持结构简单：短段落 + 列表；表格尽量扁平，超过限制则改为列表或代码块。这是结构限制，不是篇幅上限：投研终稿的研究深度与字数按投研约束执行。\n\
- 正文和列表请只写普通 Markdown；确实需要表格时，只写标准 Markdown 表格（`| 列1 | 列2 |`）。\n\
- 不要手写飞书卡片标签或扩展组件，例如 `<table .../>`、`<chart .../>`、`<row>`、`<record .../>`、`<button ...>`。\n\
- 运行时会自动把标准 Markdown 表格转换成飞书 JSON 2.0 原生表格组件；如果你手写原始飞书标签，渠道会降级为普通文本。";

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::HoneConfig;
    use hone_memory::SessionStorage;
    use hone_memory::session::SessionPromptState;
    use std::fs;

    #[test]
    fn hari_invest_skill_defaults_every_investment_question_to_full_research_depth() {
        let skill = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills/hari-invest/SKILL.md"),
        )
        .expect("hari-invest SKILL.md should be readable");
        // A one-line question ("戴尔超预期吗") must still come back as a full
        // research note; brevity is reserved for greetings, ledger follow-ups,
        // entity clarification and product usage.
        assert!(skill.contains("研究深度默认"));
        assert!(skill.contains("篇幅由本轮取到的证据决定，不由问句字数决定"));
        assert!(skill.contains("不得以“要不要接着算”收尾"));
        assert!(skill.contains("不得整块省略"));
        assert!(skill.contains("只有问候、记账追问、实体澄清与产品使用类问题才真正简短"));
        // The floor is a hard requirement, and it may only be met with
        // evidence — never with restated conclusions or disclaimers.
        assert!(skill.contains("篇幅下限是硬要求，不是风格偏好"));
        assert!(skill.contains("中文正文不少于 1500 字"));
        assert!(skill.contains("2500–4000 字"));
        assert!(skill.contains("不得靠复述问题、重复结论、罗列免责声明"));
        assert!(skill.contains("禁止把内容留到下一轮"));
        assert!(skill.contains("内部 `laowang-investment-distiller`"));
    }

    #[test]
    fn finance_research_core_skill_prioritizes_structured_market_data_without_a_completion_gate() {
        let skill = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills/finance-research-core/SKILL.md"),
        )
        .expect("finance-research-core SKILL.md should be readable");
        assert!(skill.contains("结构化行情是明确公司/证券问题的第一事实来源"));
        assert!(skill.contains("在开放网页搜索之前，优先对选中的全部标准 symbol"));
        assert!(skill.contains("不是完成门禁"));
        // 公式必须是纯文本：渲染层没有数学引擎，而正文里的 $ 是货币符号。
        assert!(skill.contains("公式排版约束"));
        assert!(skill.contains("`$` 只表示货币"));
        assert!(skill.contains("应继续使用当前可得证据与公开搜索"));
        assert!(skill.contains("财报数字时效与准确性"));
        // Channel formatting trims paragraph size, never research depth.
        assert!(TELEGRAM_FORMAT_GUIDANCE.contains("这是排版粒度要求，不是篇幅上限"));
        assert!(FEISHU_FORMAT_GUIDANCE.contains("这是结构限制，不是篇幅上限"));
        assert!(skill.contains("EBIT、EBITA、EBITDA 与营业利润不是同一个指标"));
        assert!(skill.contains("不是逐数字双来源或缺项拒答门禁"));
    }

    /// A turn that fetched no market data must not publish a priced quote
    /// prefix, and a turn that fetched provider aggregates must not label them
    /// as primary filings it never opened. Both live in the on-demand skill
    /// rather than in a runtime validator (AGENTS.md 139-145).
    #[test]
    fn finance_research_core_skill_binds_quote_prefix_and_source_labels_to_this_turn() {
        let skill = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills/finance-research-core/SKILL.md"),
        )
        .expect("finance-research-core SKILL.md should be readable");
        assert!(skill.contains("首行的“无行情分支”"));
        assert!(skill.contains("行情口径只能写“本轮未取到 <代码> 行情”"));
        for banned in [
            "价格",
            "涨跌幅",
            "市值",
            "估值倍数",
            "财报科目金额",
            "价位型买卖区间",
        ] {
            assert!(
                skill.contains(banned),
                "no-quote branch stopped naming {banned}"
            );
        }
        assert!(skill.contains("只加载 skill、只读本地文件、只查技能索引都**不算**取到行情"));
        assert!(skill.contains("这条同样管表格的「来源」列"));
        for upgraded in ["10-Q", "SEC 官方披露文件", "交易所官方收盘价"] {
            assert!(
                skill.contains(upgraded),
                "source-tier rule stopped naming {upgraded}"
            );
        }
    }

    /// The audit found generic cycle-normalization advice actively wrong for
    /// storage names whose earnings are partly locked by long-term agreements,
    /// and RKLB reduced to a single Neutron option. Those company-specific
    /// drivers live in the card's valuation framing and must reach the prompt.
    /// The audit's own MU sample was typed "mu不是15pe？" — lowercase, so the
    /// card never loaded and the answer lost its company-specific framing.
    #[tokio::test]
    async fn company_research_baseline_matches_lowercase_short_ticker_in_chinese() {
        assert!(company_research_baseline("mu不是15pe？咋不建议买？").is_some());
        assert!(company_research_baseline("sndk，成本1500").is_some());
        // English prose keeps its guard: "be" and "app" stay ordinary words.
        assert!(company_research_baseline("build an app and be concise").is_none());
    }

    #[tokio::test]
    async fn industry_map_ships_aliases_and_members_for_every_industry() {
        let corpus = hone_core::industry_map::base_map();
        assert!(!corpus.industries.is_empty());
        for industry in &corpus.industries {
            assert!(!industry.aliases.is_empty(), "{} 没有别名", industry.id);
            assert!(!industry.members.is_empty(), "{} 没有成员", industry.id);
        }
    }

    #[tokio::test]
    async fn industry_baseline_stays_silent_until_a_driver_chain_is_written() {
        // 传导链或估值逻辑是这块内容存在的理由。只有成员名单的行业注入进去，等于用 token 换一句
        // 「这家公司属于存储」——模型本来就知道。V5.3 新加的行（商业太空、AI 应用）还没有带日期的
        // 传导链，但有估值逻辑，所以照样注入；这里只盯两样都没有的行。
        let corpus = hone_core::industry_map::base_map();
        for industry in &corpus.industries {
            if !industry.ai_valuation_logic.driver_chain.trim().is_empty()
                || !industry.valuation.logic.summary.trim().is_empty()
            {
                continue;
            }
            for member in &industry.members {
                let hit = industry_baseline(
                    &format!("{} 的估值", member.symbol),
                    std::path::Path::new("/nonexistent"),
                );
                if let Some(text) = hit {
                    assert!(
                        !text.contains(&industry.name),
                        "{} 还没有传导链却被注入了",
                        industry.id
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn industry_baseline_pulls_nvda_first_and_keeps_the_upstream_block_compact() {
        // 存储行：上游最近动作在前（英伟达第一），估值执行卡带 SNDK 所属子类型的主锚，通用规则压缩版在头部。
        let text = industry_baseline(
            "SNDK 的合理估值是多少",
            std::path::Path::new("/nonexistent"),
        )
        .expect("storage is in the tree");
        let upstream = text
            .lines()
            .filter(|line| line.contains("上游最近动作 ·"))
            .collect::<Vec<_>>();
        assert_eq!(upstream.len(), 2, "{text}");
        assert!(
            upstream[0].contains("上游最近动作 · NVDA"),
            "{}",
            upstream[0]
        );
        assert!(upstream[0].contains("截至 2026-08-26"), "{}", upstream[0]);
        assert!(text.contains("data_fetch(data_type=\"earnings_outlook\", ticker=\"NVDA\")"));
        assert!(
            text.find("上游最近动作 · NVDA").unwrap() < text.find("估值执行卡 · ").unwrap()
                && text.find("估值执行卡 · ").unwrap() < text.find("需求传导链").unwrap(),
            "{text}"
        );
        assert!(
            text.contains("SNDK（闪迪（SanDisk）｜"),
            "命中行要带子类型名：{text}"
        );
        assert!(text.contains("估值执行卡 · 子类型「"), "{text}");
        assert!(text.contains("Forward PE"), "{text}");
        assert!(text.contains("这一行禁止的估值动作"), "{text}");
        assert!(text.contains("A / B / C 级"), "{text}");
        assert!(text.contains("十二个执行字段"), "{text}");
        assert!(text.contains("变量传导与证伪"), "{text}");
        // 单行命中的注入体量守在 4,600 字以内（约 2,300 token）：头部规则约 1,400 字，行内约 3,000 字。
        assert!(
            text.chars().count() < 5600,
            "注入过长：{} 字",
            text.chars().count()
        );
    }

    #[tokio::test]
    async fn industry_baseline_falls_back_to_the_row_anchor_for_industry_level_questions() {
        let text = industry_baseline(
            "光通信这一行现在怎么看",
            std::path::Path::new("/nonexistent"),
        )
        .expect("optical alias");
        assert!(
            text.contains("估值执行卡 · 行级倍数锚") || text.contains("估值执行卡 · 子类型"),
            "{text}"
        );
        assert!(!text.contains("本轮命中："), "no member hit: {text}");
    }

    #[tokio::test]
    async fn industry_baseline_is_absent_for_questions_outside_the_tree() {
        assert!(
            industry_baseline(
                "帮我把这句话翻译成英文",
                std::path::Path::new("/nonexistent")
            )
            .is_none()
        );
        assert!(industry_baseline("你好", std::path::Path::new("/nonexistent")).is_none());
    }

    #[tokio::test]
    async fn company_research_baseline_carries_company_specific_valuation_drivers() {
        let mu = company_research_baseline("mu 现在的估值贵不贵").expect("MU card");
        assert!(mu.contains("长协"), "MU baseline missing LTA driver: {mu}");
        assert!(mu.contains("HBM"), "MU baseline missing HBM driver");

        let sndk = company_research_baseline("sndk 这家公司怎么样").expect("SNDK card");
        assert!(
            sndk.contains("NBM"),
            "SNDK baseline missing NBM driver: {sndk}"
        );

        let rklb = company_research_baseline("分析下 RKLB").expect("RKLB card");
        assert!(
            rklb.contains("Neutron"),
            "RKLB baseline missing Neutron milestones"
        );
        assert!(
            rklb.contains("稀释"),
            "RKLB baseline missing dilution driver"
        );
    }

    #[tokio::test]
    async fn company_research_baseline_matches_chinese_alias_and_keeps_current_fact_boundary() {
        let baseline = company_research_baseline("微软现在的护城河是否被 AI 削弱？")
            .expect("Microsoft should be covered by the historical corpus");
        assert!(baseline.contains("MSFT / Microsoft"));
        assert!(baseline.contains("商业模式："));
        assert!(baseline.contains("护城河："));
        assert!(baseline.contains("它不是当前事实"));
        assert!(baseline.contains("最新一手证据为准"));
        assert!(!baseline.contains("逐字稿文本"));
    }

    #[tokio::test]
    async fn company_research_baseline_matches_multiple_explicit_short_symbols() {
        let baseline = company_research_baseline("比较 APP 和 BE 的商业模式与护城河")
            .expect("explicit uppercase symbols should match");
        assert!(baseline.contains("APP / AppLovin"));
        assert!(baseline.contains("BE / Bloom Energy"));
    }

    #[tokio::test]
    async fn company_research_baseline_does_not_match_short_tickers_inside_plain_words() {
        assert!(company_research_baseline("build an app and be concise").is_none());
        assert!(company_research_baseline("分析 NVDA 的护城河").is_none());
    }

    #[tokio::test]
    async fn company_research_index_and_cards_cover_the_same_symbols() {
        let (corpus, index) = company_research_resources();
        let mut card_symbols = corpus
            .companies
            .iter()
            .map(|card| card.symbol.as_str())
            .collect::<Vec<_>>();
        let mut indexed_symbols = index
            .companies
            .iter()
            .map(|entry| entry.symbol.as_str())
            .collect::<Vec<_>>();
        card_symbols.sort_unstable();
        indexed_symbols.sort_unstable();
        assert_eq!(card_symbols, indexed_symbols);
        assert_eq!(card_symbols.len(), 52);
    }

    #[tokio::test]
    async fn build_prompt_bundle_leaves_system_prompt_empty_and_moves_guidance_to_user_prompt() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-test-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();
        let prompt_state = SessionPromptState::default();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "telegram",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        )
        .await;

        assert!(bundle.system_prompt().is_empty());
        let composed = bundle.compose_user_input("分析 NVDA");
        assert!(composed.contains("【输出格式-Telegram】"));
        assert!(composed.contains("【Session 上下文】"));
        assert!(composed.contains("【本轮用户输入】"));

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn domain_policies_live_in_on_demand_skills() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
        let finance_core = fs::read_to_string(root.join("finance-research-core/SKILL.md"))
            .expect("finance-research-core SKILL.md");
        let company_portrait = fs::read_to_string(root.join("company_portrait/SKILL.md"))
            .expect("company_portrait SKILL.md");
        let scheduled_task = fs::read_to_string(root.join("scheduled_task/SKILL.md"))
            .expect("scheduled_task SKILL.md");
        let notification_prefs = fs::read_to_string(root.join("notification_preferences/SKILL.md"))
            .expect("notification_preferences SKILL.md");

        for required in [
            "禁止荐股",
            "不得输出可照抄的单票排序",
            "集中仓位模板",
            "多标的最新行情约束",
            "非标准 ticker 约束",
            "副作用写入确认约束",
            "强时效行情建议约束",
            "可审计核验约束",
            "基金/ETF 披露口径约束",
            "原油与大宗商品归因约束",
            "portfolio(action=\"replace_all\")",
            "只有这些股票/持仓，其他没有了",
            "用户信息汇总边界",
            "open_id",
            "chat_id",
        ] {
            assert!(
                finance_core.contains(required),
                "finance-research-core missing: {required}"
            );
        }

        for required in [
            "用户自己的看法、偏好或约束",
            "相似公司里与当前问题相关的记录",
            "宏观叙事与行业框架应尽量保持一致",
            "长答去重约束",
        ] {
            assert!(
                company_portrait.contains(required),
                "company_portrait missing: {required}"
            );
        }

        for required in [
            "我的持仓 / 关注列表 / 定时任务 / 心跳任务",
            "holdings.json",
            "data/portfolio",
            "automatic_push",
            "Web 渠道送达边界",
        ] {
            assert!(
                scheduled_task.contains(required),
                "scheduled_task missing: {required}"
            );
        }

        assert!(notification_prefs.contains("disable_all"));
        assert!(notification_prefs.contains("get_overview"));
        assert!(notification_prefs.contains("set_quiet_hours"));
    }

    #[tokio::test]
    async fn telegram_format_guidance_mentions_supported_html_features() {
        let guidance = channel_format_guidance("telegram");
        assert!(guidance.contains("parse_mode=HTML"));
        assert!(guidance.contains("tg-spoiler"));
        assert!(guidance.contains("blockquote expandable"));
        assert!(guidance.contains("tg-time"));
        assert!(guidance.contains("&lt;、&gt;、&amp;"));
    }

    #[tokio::test]
    async fn feishu_format_guidance_requires_standard_markdown_tables() {
        let guidance = channel_format_guidance("feishu");
        assert!(guidance.contains("只写标准 Markdown 表格"));
        assert!(guidance.contains("不要手写飞书卡片标签"));
        assert!(guidance.contains("JSON 2.0 原生表格组件"));
    }

    #[tokio::test]
    async fn dynamic_session_context_stays_out_of_system_prompt_prefix() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-dynamic-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
        // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();
        let prompt_state = SessionPromptState::default();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "feishu",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        )
        .await;

        assert!(!bundle.system_prompt().contains("【Session 上下文】"));
        assert!(
            bundle
                .compose_user_input("你好")
                .contains("【Session 上下文】")
        );
        assert!(
            bundle
                .compose_user_input("你好")
                .contains("【本轮用户输入】")
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn current_turn_input_is_last_after_session_context() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-session-order-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
        // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "feishu",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions::default(),
        )
        .await;

        let composed = bundle.compose_user_input("今天公布的非农数据怎么样");
        let input_pos = composed
            .find("【本轮用户输入】")
            .expect("user input section");
        let session_pos = composed
            .find("【Session 上下文】")
            .expect("session section");

        assert!(session_pos < input_pos);

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn current_turn_input_is_last_after_historical_skill_context() {
        let bundle = PromptBundle {
            static_system: String::new(),
            turn_guidance: String::new(),
            conversation_context: Some(
                "【Invoked Skill Context】\nSkill: Stock Research (stock_research)\nLITE"
                    .to_string(),
            ),
            session_context: "【Session 上下文】\n当前时间：2026-05-01 12:00:00".to_string(),
            answer_time_local: "2026-05-01 12:00".to_string(),
        };

        let composed = bundle.compose_user_input("AMD的电脑CPU是什么名字");
        let skill_pos = composed
            .find("Skill: Stock Research")
            .expect("historical skill context");
        let session_pos = composed
            .find("【Session 上下文】")
            .expect("session section");
        let input_pos = composed
            .find("【本轮用户输入】")
            .expect("current input section");

        assert!(skill_pos < input_pos);
        assert!(session_pos < input_pos);
        assert!(composed.ends_with("AMD的电脑CPU是什么名字"));
    }

    #[tokio::test]
    async fn session_context_uses_current_time_instead_of_frozen_prompt_time() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-current-time-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
        // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();
        let prompt_state = SessionPromptState {
            frozen_time_local: "2026-03-17T22:01:00+08:00".to_string(),
        };

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "discord",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        )
        .await;

        let composed = bundle.compose_user_input("今天公布的非农数据怎么样");
        assert!(!composed.contains("2026-03-17 22:01:00"));
        assert!(composed.contains(&hone_core::local_now().format("%Y-%m-%d").to_string()));
        assert!(
            bundle
                .session_context
                .contains(&format!("当前时间：{}:", bundle.answer_time_local)),
            "the answer contract anchor must be derived from the exact same clock read as Session context: {:?}",
            bundle
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn prompt_bundle_can_skip_compact_summary_loading_at_the_source() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-no-conversation-context-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let session_id = storage
            .create_session(Some("session-no-summary-load"), None, None)
            .await
            .expect("create session");
        storage
            .add_message(
                &session_id,
                "system",
                "【Compact Summary】\nsummary-that-must-stay-unloaded",
                Some(hone_memory::build_compact_summary_metadata("auto")),
            )
            .await
            .expect("add compact summary");
        let config = HoneConfig::default();
        let prompt_state = SessionPromptState::default();

        let bundle = build_prompt_bundle_at(
            &config,
            &storage,
            "web",
            &session_id,
            &prompt_state,
            &PromptOptions::default(),
            hone_core::local_now(),
            false,
        )
        .await;

        assert!(bundle.conversation_context.is_none());
        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn prompt_options_append_admin_language_and_extra_sections() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-options-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
        // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "discord",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions {
                is_admin: true,
                admin_prompt: Some("【管理员覆写】请先确认影响范围。".to_string()),
                privacy_guard: Some(DEFAULT_GROUP_PRIVACY_GUARD.to_string()),
                model_hint: Some("gpt-5.4".to_string()),
                reply_language: Some(ReplyLanguage::Chinese),
                extra_sections: vec![
                    "【附加规则】先给结论再展开。".to_string(),
                    "   ".to_string(),
                ],
                include_format_guidance: true,
            },
        )
        .await;

        assert!(bundle.system_prompt().is_empty());
        let composed = bundle.compose_user_input("分析一下 NVDA");
        assert!(composed.contains("【管理员覆写】请先确认影响范围。"));
        assert!(composed.contains(DEFAULT_GROUP_PRIVACY_GUARD));
        assert!(composed.contains("【基础模型】gpt-5.4。"));
        assert!(composed.contains("【语言要求】必须全程以中文回复"));
        assert!(composed.contains("阿拉伯数字与货币单位"));
        assert!(!composed.contains("禁止中英文混排"));
        assert!(composed.contains("【附加规则】先给结论再展开。"));
        assert!(composed.contains("【输出格式-Discord】"));
        let guidance_pos = composed
            .find("【输出格式-Discord】")
            .expect("guidance section");
        let input_pos = composed
            .find("【本轮用户输入】")
            .expect("user input section");
        assert!(guidance_pos < input_pos);

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn prompt_can_skip_channel_format_guidance() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-no-format-{}-{}",
            std::process::id(),
            hone_core::local_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions")).await;
        let mut config = HoneConfig::default();
        // 测试必须钉住运行时时区:`HoneBotCore::new` 会用 config 里的时区重新配置
        // 进程级全局,任何未设时区的 config 都会把它重置成宿主时区,污染同进程后续测试。
        config.timezone = Some("Asia/Shanghai".to_string());
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "telegram",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions {
                include_format_guidance: false,
                ..PromptOptions::default()
            },
        )
        .await;

        assert!(bundle.system_prompt().is_empty());
        assert!(
            !bundle
                .compose_user_input("你好")
                .contains("【输出格式-Telegram】")
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[tokio::test]
    async fn repository_soul_is_empty_and_finance_contract_lives_in_skill() {
        let soul = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../soul.md"),
        )
        .expect("repository soul.md should be readable");
        assert!(
            soul.trim().is_empty(),
            "main harness soul.md should stay empty while domain rules live in skills"
        );

        let core_skill = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills/finance-research-core/SKILL.md"),
        )
        .expect("finance-research-core SKILL.md should be readable");
        for required in [
            "B. 单股深度分析",
            "B.1 ETF / 基金深度分析",
            "B.2 加密资产深度分析",
            "C. 板块 / 技术 / 产业链分析",
            "F. 财务对比",
            "强制输出顺序",
            "Bull 投资主线",
            "Bear 投资主线",
            "Base Case",
            "用户信息汇总边界",
            "hone_security_listing_evidence.status=active_listing",
        ] {
            assert!(
                core_skill.contains(required),
                "finance-research-core SKILL.md missing: {required}"
            );
        }
    }
}
