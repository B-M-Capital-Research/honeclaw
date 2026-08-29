//! Hone Agent — Function Calling Agent 核心
//!
//! 基于 `OpenAI` Function Calling 模式的 legacy Agent 适配器。
//! 这里负责多轮工具调用循环，并把最终结果聚合成 `AgentResponse`；
//! 渠道级流式输出由 `hone-channels` 的 runner 层处理。

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Weekday};
use futures::StreamExt;
use hone_core::agent::{
    Agent, AgentContext, AgentResponse, RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY, ToolCallMade,
};
use hone_core::tool_effect::{
    persistent_tool_reconciliation_call, tool_call_has_persistent_side_effect,
    tool_call_is_known_read_only,
};
use hone_core::{
    LlmAuditRecord, LlmAuditSink, ToolExecutionObserver, is_context_overflow_error,
    provider_canonical_key, provider_symbols_equivalent, truncate_chars,
};
use hone_llm::provider::ChatStreamFinishReason;
use hone_llm::{
    ChatResponse, ChatStreamEvent, FunctionCall, LlmProvider, Message, ToolCall, ToolChoiceMode,
};
use hone_tools::ToolRegistry;
use hone_tools::data_fetch::{
    data_fetch_data_type_uses_security_target, effective_data_fetch_data_type,
    effective_data_fetch_security_target, validated_data_fetch_search_query,
    validated_data_fetch_symbols,
};
use regex::Regex;
#[cfg(test)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

const REASONING_CONTENT_METADATA_KEY: &str = "reasoning_content";
#[cfg(not(test))]
const FALLBACK_ACTIVE_BUSINESS_TIMEOUT: Duration = Duration::from_secs(20);
#[cfg(test)]
const FALLBACK_ACTIVE_BUSINESS_TIMEOUT: Duration = Duration::from_millis(25);
#[cfg(test)]
const FINISH_RESEARCH_TOOL_NAME: &str = "finish_research";
const ACTIVE_BUSINESS_FAILURE_RETRY_LIMIT: u32 = 1;
const MAX_AGENT_OWNED_HISTORY_USER_TURNS: usize = 4;
const MAX_AGENT_OWNED_HISTORY_CHARS: usize = 4_000;
/// Three rounds is enough to confirm a price and stop. Answering why a business
/// moved needs the release, the comparison and the reaction, which is a round
/// each — the budget, not the model, was what kept answers at headline depth.
/// These constants are the conservative defaults; deployments raise them via
/// `FinanceResearchBudget` (config `agent.finance_research`) for gap-driven
/// research without changing the tested legacy behavior.
const MAX_AGENT_OWNED_FINANCE_TOOL_ROUNDS: u32 = 5;
/// Entity identity resolution is a precondition of research, not research
/// itself. A turn whose first rounds only ask "which security is this?" must
/// not arrive at the forced final having never looked at the user's actual
/// question, so identity-only rounds draw on this separate small budget.
const MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUNDS: u32 = 2;
/// A forced final with zero substantive evidence is indistinguishable from an
/// offline assistant. When the research budget runs out before any non-identity
/// tool call happened, the same Agent gets this many extra tool rounds aimed at
/// the user's original question.
const MAX_AGENT_OWNED_FINANCE_EVIDENCE_RESCUE_ROUNDS: u32 = 1;
/// Global tool-call slots that only open web research may spend. Reserving them
/// keeps a phantom or unresolvable entity route from consuming the entire turn.
const AGENT_OWNED_FINANCE_RESERVED_OPEN_RESEARCH_CALLS: u32 = 2;
const AGENT_OWNED_OPEN_RESEARCH_TOOL: &str = "web_search";
const MAX_AGENT_OWNED_SCREENING_CANDIDATE_ROUTES: usize = 4;
const MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES: usize = 6;
const MAX_AGENT_OWNED_FINANCE_TOOL_CALLS: u32 = 24;
const MAX_AGENT_OWNED_FINANCE_DATA_FETCH_CALLS: u32 = 20;
const MAX_AGENT_OWNED_FINANCE_WEB_SEARCH_CALLS: u32 = 6;
// One retry was not enough in practice: production drafts kept failing the same
// "cite the target date and the source URL in the cause paragraph" rule on the
// single correction round and degraded to the deterministic gap answer. The
// citation itself must stay the model's job — stapling an eligible URL onto a
// cause paragraph mechanically would manufacture exactly the evidence link this
// check exists to verify — so the only honest lever is one more attempt. Both
// rounds still sit far below the agent's 18-iteration ceiling.
const MAX_MARKET_MOVE_FINAL_CORRECTIONS: u32 = 2;
const MAX_MARKET_MOVE_DRAFT_FEEDBACK_CHARS: usize = 3_000;
const MAX_LISTING_FINAL_CORRECTIONS: u32 = 1;
const MAX_LANGUAGE_FINAL_CORRECTIONS: u32 = 1;
/// The one structural marker an investment answer must open with. It is the
/// existing answer contract, not a classifier vocabulary.
const CANONICAL_RESEARCH_ANSWER_HEADER: &str = "数据时间：";
const MAX_LISTING_DRAFT_FEEDBACK_CHARS: usize = 3_000;
const CONTEXT_OVERFLOW_RECOVERY_TOTAL_TOOL_CHARS: usize = 48_000;
const CONTEXT_OVERFLOW_RECOVERY_MAX_RESULT_CHARS: usize = 6_000;
const CONTEXT_OVERFLOW_RECOVERY_MIN_RESULT_CHARS: usize = 800;
const AGENT_OVERALL_TIMEOUT_ERROR: &str =
    "agent_timeout: function-calling overall deadline exceeded";
const AGENT_STEP_TIMEOUT_ERROR: &str = "agent_timeout: function-calling step deadline exceeded";
const AGENT_OWNED_FINANCE_FORCED_FINAL_TOOL_ERROR: &str =
    "agent_owned_finance_forced_final_returned_tool_call";
const AGENT_OWNED_FINANCE_FORCED_FINAL_SYSTEM_INSTRUCTION: &str = "【本轮研究预算已完成】当前轮次不再提供工具。请由同一 Agent 仅根据本轮已经取得的真实工具结果，直接生成一次完整自然终稿；已有证据不足的项目如实披露具体缺口，不得补写模型记忆、不得要求用户重试，也不要提及预算、内部轮次或工具已关闭。";
const READ_ONLY_TRACE_FINALIZATION_INSTRUCTION: &str = "【内部只读证据收口】当前轮次不再提供工具。本轮已经取得真实只读工具结果，但工具轮在形成终稿前结束。请由同一 Agent 继续回答用户原问题：只使用本轮已经取得的真实工具结果；缺少的数据做最小、具体披露，不得把整轮改写成“研究未完成”“请稍后再试”或其它通用拒答；不得声称未实际执行的查询已经完成，也不要向用户提及工具、轮次上限、内部状态或本说明。";
const AGENT_OWNED_OPEN_RESEARCH_RESCUE_INSTRUCTION: &str = "【本轮尚未取得任何实质证据】到目前为止本轮只做过证券身份查询，没有取得任何回答用户原问题所需的实质资料。请立刻停止继续解析、补查或重试证券代码：本轮出现的大写缩写可能根本不是证券代码，而是宏观、策略、指标或行业术语（例如 CTA、RSI、QT、TTM 这类）。本轮请重新完整阅读用户原话，判断用户真正想知道什么，并用 `web_search` 等开放检索工具，围绕用户原问题的真实主题发起检索；需要时可并行多条查询。取得资料后再按用户原问题作答；确实检索不到时，如实说明检索过的方向与缺口，不得直接回答“无法提供具体数字”或要求用户重试，也不要向用户提及内部轮次、预算或本说明。";
const BLOCKED_TOOL_FINALIZATION_INSTRUCTION: &str = "【内部安全收口】上一批工具调用没有执行，也没有形成任何工具结果。当前轮次不再提供工具。请由同一 Agent 继续回答用户原问题：只使用本轮已经取得的真实证据；缺少的数据做最小、具体披露或确认，不得把整轮改写成“研究未完成”“请稍后再试”或其它通用拒答；不得声称被拦截的查询或操作已经执行，也不要向用户提及工具、安全边界、内部轮次或本说明。";
const CONTEXT_OVERFLOW_FINALIZATION_INSTRUCTION: &str = "【内部有界证据收口】当前轮次不再提供工具。上一阶段已经取得的完整工具结果仍保留在内部审计中；为保证本轮继续执行，下面只提供这些结果的机械有界副本。每条记录保留真实 tool_call_id、工具名、参数和可容纳的结果字段；`result_compacted=true` 表示部分长字符串或数组尾部未进入本副本，不代表原结果为空或事实不存在。请由同一 Agent 直接回答原问题：只能使用副本中实际可见的事实，未出现的字段作具体缺口披露，不得凭模型记忆补齐，不得要求用户重试或切换会话，也不要向用户提及上下文、压缩、载荷、内部轮次、工具关闭或本说明。";
const PERSISTENT_MUTATION_FINALIZATION_INSTRUCTION: &str = "【内部写后核验收口】上一项持久化操作返回了不确定错误，系统没有重放该写操作，而是通过同一用户范围内的只读查询取得了操作后的权威状态。当前轮次不再提供工具。请只根据原请求和只读核验结果回答操作是否已经生效；已生效就明确确认，未生效就明确说明当前状态，无法从核验结果判断的字段只披露该具体缺口。不得再次调用或建议重放写操作，不得使用“可能已执行”“状态不确定”“请重试”等模糊结论，也不要向用户提及内部错误、工具、核验机制或本说明。";
const OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION: &str = "【本轮 Agent 工具决策】先完整阅读本轮用户原话，再决定是否调用工具。若问题明确点名任何公司、证券、基金、指数或加密资产，第一轮优先完成结构化实体发现，不写最终正文：为你识别出的每个点名标的分别并行调用一次 DataFetch search，每个调用都填写互不复用且后续原样复用的 `entity_route`，并填写本次调用自己的 `identity_match`（ticker 用 `exact_symbol`，公司名、中文名或别名用 `name_or_alias`）。search 返回标准 symbol 后，下一步优先为每条路线调用 DataFetch snapshot；它是 quote + profile + news + 可得盘后字段的聚合行情入口。snapshot 不适用时再组合 quote/profile；用户询问盘前、盘后或常规盘与扩展时段对比时补 extended_hours。结构化行情的工具选择优先级高于开放 Web 搜索；Web/news/filing/industry 用于随后补充公告、监管文件、关系、事件和因果等行情工具不能证明的内容。这里描述的是优先级而不是完成门禁：provider 无覆盖、工具失败或问题本身不需要行情时，不得反复补取或拒绝回答，可继续使用当前证据和开放检索并自然披露缺口。quote/profile/snapshot 等依赖标准 ticker 的调用必须等待 search 返回标准 symbol 后再执行，禁止猜测、补全或凭记忆构造代码。用户可能用小写、混合大小写或带市场常用分隔符书写 ticker；证券语境里的代码仍按 ticker 处理并用标准代码精确查询，不能因为写成小写就先改走公司别名搜索。不要只处理第一个标的，也不要等服务端按字符串拆分问题。若并非证券/公司研究问题，则按用户实际意图正常处理，不要生造证券实体。";
const POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION: &str = "【内部研究取证轮】当前轮已进入金融研究工具链，但证券实体、行情或资产路由证据仍未完整。先由你完整分析用户实际点名的全部公司/证券，不要依赖固定问法扫描器。为每个标的分配一个本轮稳定且互不复用的 `entity_route`（内部短键，不是用户可见结论）；每个标的分别发起一个 search（可在同一轮并行，禁止把多个标的拼成一个 query），并由你依据完整语义在每一次 search 调用里明确填写 call-scoped `identity_match`：query 是 ticker 时用 `exact_symbol`，是公司名、中文名或别名时用 `name_or_alias`；用户书写的 ticker 不要求大写，证券语境里的小写或混合大小写代码应先规范成标准代码并走 `exact_symbol`，不能仅因大小写改走别名 refinement；前一次声明不会授权后一次 search，也不要让服务端按大小写或长度猜。标准 symbol 已确认的路线优先调用 snapshot，一次取得 quote、profile、报价源时间、服务端涨跌口径和可得盘后字段；snapshot 不适用时组合 quote/profile，问题涉及盘前、盘后或常规盘与扩展时段对比时补 extended_hours。完成或实际尝试这组结构化行情读取后，再用 Web/news/filing/industry 检索补充行情工具不能证明的公告、关系、事件与因果；互不依赖且确有时效需要时可以与行情调用同批，但应把行情调用列在 Web 之前。这个顺序是 Agent 选择信号，不是缺失即拒答的门禁；provider 无覆盖或调用失败时继续搜索和回答，不反复重试。quote/profile/snapshot 等依赖某条路线标准 symbol 的调用必须等待该路线 search 结果返回；若当前上下文已经有该路线的标准 symbol，可在本轮执行，否则禁止猜测、补全或凭记忆构造代码。后续 refinement、quote、profile/snapshot 与其它该标的调用都原样携带同一路线键。显式 ticker 路线的同代码约束在后续公司名补查中仍持续有效，不能切换成名字里提到该代码的其它产品；有限 provider 分隔写法可等价。若此前调用缺少路线键，补查时重复原 query，或用 `supersedes_query` 逐字指向那次旧 query，以便只迁移该路线。`refines_query` 与 `supersedes_query` 严格互斥，每次 search 最多填写一个：前者只连接同路线的空结果补查，后者只迁移一条漏写路线键的旧 query。若中文名、别名或代码搜索为空，在同一 `entity_route` 下换用公司正式英文名或标准 ticker 做精确补查；可在 `refines_query` 中逐字填写原始空 query，但不得另建或复用其它实体的路线来抵消。随后按用户原始问题继续取得财务、新闻、网页、公告、持仓或其它业务证据。尽量在同一轮批量或并行调用互不依赖的工具。不得把 data_fetch(search) 或 profile 当成公司关系、事件或因果证据。合理取证已经完成或必要来源经实际尝试后明确不可得时，由同一 Agent 直接形成一次自然终稿。";
const AGENT_OWNED_RESEARCH_SYSTEM_INSTRUCTION: &str = "【同一 Agent 自然研究轮】继续阅读完整用户原话和本轮真实工具结果，自主决定是补充当前问题真正需要的业务工具，还是直接形成一次完整终稿。明确公司/证券路线已有标准 symbol、但尚未尝试完整行情时，优先调用 snapshot；snapshot 不适用时组合 quote/profile，扩展时段问题补 extended_hours，之后再用 Web 搜索补事件和因果。当前时刻处于美股盘前、盘后、隔夜（夜盘）或休市而用户问最新涨跌或行情时，extended_hours（个股）与指数期货或 SPY/QQQ 扩展时段（宽基）就是主数据源：取到后直接围绕该窗口作答，常规时段口径一句带过即可，不必反复对照或解释时段差异。这个行情优先顺序只是工具选择信号，不是终稿门禁；provider 无覆盖或调用失败时不反复补取，继续使用当前证据回答并披露具体缺口。证据不足时只调用当前需要的真实工具；合理取证已经完成，或必要来源经实际尝试后明确不可得并可如实披露时，直接返回自然语言最终回答。实体 search/profile 只证明身份或公司自述，不证明关系、事件和因果；宽泛关系问题通常分别核查商业/客户供应/技术合同与投资持股，优先 SEC、公司 IR 或双方公告。所有事实使用当前工具结果；`hone_security_listing_evidence.status=active_listing` 是本轮同代码当前上市证据，不得用旧收购或退市记忆否认；单项数据缺失时如实披露，并继续完成当前证据能够支持的部分。";
#[cfg(test)]
const ACTIVE_RESEARCH_SYSTEM_INSTRUCTION: &str = "【内部研究工具轮】当前仍是工具轮，同时提供真实业务工具和 `finish_research`。请由同一 Agent 重新阅读完整用户原话与本轮真实工具结果；当前结构状态只覆盖 Agent 已声明的路线，不证明点名实体集合完整、工具调用成功或业务证据充分。证据不足时本轮只调用当前最需要的真实业务工具；合理研究已经完成，或必要来源经实际尝试后明确不可得并可如实披露时，本轮只提交 `finish_research` 的结构化证据交接进入无工具终稿。不要把完成信号与业务工具混用，也不要在工具轮写最终正文。实体 search/profile 只证明身份，不证明公司关系；关系、事件和因果结论必须先取得本轮 web/news/公告证据。对宽泛关系问题，由你从完整语义自主枚举与当前问题有关的关系轴；通常至少分别核查商业/客户供应/技术合同与投资持股，优先查 SEC、公司 IR 或双方公告，不能用一次泛搜索或“没有搜到”推出否定事实。准备写入终稿的每个外部事实都要在 finish 交接里引用本轮 tool call 与逐字 excerpt/JSON 字段；其余内容进入 gaps，不得从模型记忆补齐。";
#[cfg(test)]
const FINISH_RESEARCH_SYSTEM_INSTRUCTION: &str = "【显式完成后的终稿阶段】Agent 已在同一业务工具循环中提交本轮结构化证据交接，现由同一 Agent 和同一上下文进入无工具终稿阶段。这是证据整理而不是新的研究规划：直接组织终稿，不要重新展开工具决策、套用与问题无关的深度模板或冗长隐藏推演。克制的对象是结论与篇幅膨胀，不是覆盖面——交接中已核验的口径、趋势、对比与风险应当写足，不要把已取得的证据留在交接里不用。只有服务端注入的 Session 时间前缀本身不需要外部证据；行情口径中的报价、币种、涨跌与报价源时间仍必须来自交接中的 resolved_evidence 或 fallback_evidence。外部事实只能由这些机械解析出的原文或字段自行归纳；交接不包含任何已验证的自由文本 claim。推断只能来自交接中 inferences 并明确标记，缺失维度只能按 gaps 披露。不得在终稿新增交接外事实。`reasoning_content`、隐藏思考、未采用草稿和内部状态文本都不是事实证据。缺失证据不构成拒答。";
const FINAL_ANSWER_EVIDENCE_CONTRACT: &str = concat!(
    "`reasoning_content`、隐藏思考、未采用草稿、内部状态文本以及模型记忆都不是事实证据，不得从中提取或补齐关系、日期、行情、财务或估值事实。",
    "当前上市状态必须服从本轮同代码结构化证据。若 `hone_security_listing_evidence.status=active_listing`，必须视为当前上市交易；不得再回答该证券未上市、已退市或应替换成旧母公司。公司过去被收购或退市的历史不能覆盖其后续分拆、重新上市事实。若本轮当前权威监管文件与 provider 证据冲突，继续取得并披露监管证据与冲突，不得凭模型记忆裁决。",
    "数据时间只能采用本轮 Session 北京时间；quote 的 provider timestamp 只能写在‘行情口径’里，绝不能冒充数据时间。用户可见的报价源时间优先使用 `hone_quote_time.beijing`；该字段缺失时才能如实使用其它已核验时间并明确时区。`hone_quote_time.market_date_new_york` 只是纽约时区的日历日期，`hone_quote_time.new_york` 也只是纽约时区的时间；二者都不证明交易所、交易时段或已经收盘，绝不能据此写‘纽交所’或‘收盘价’。证券所属交易所只能来自 quote/profile 的 `exchange` 或 `exchangeShortName` 字段。没有行情证据时仍保留‘行情口径’字段并说明范围，不得伪造报价时间或盘前/盘后时段。",
    "回答“为什么大跌/暴跌/下跌”前，先锁定用户所指对象或市场范围与目标时段，并先核验该对象在该时段是否真的发生所述波动。用户明确给出的日期、星期或时段优先；用户未指明时段时，以当前运行时刻所处的市场时段为默认目标时段——处于盘前、盘后、隔夜（夜盘）或休市时，默认指最新可得的延长时段行情，个股直接以 extended_hours 的最新时段窗口为主叙事，宽基优先指数期货或 SPY/QQQ 扩展时段；主窗口锁定后直接进入原因分析，其它口径最多一句带过，不必展开多窗口对照（这是默认锚定引导，不是核验门禁）。不得因为 latest quote、当前日历日或另一日新闻更容易取得，就静默改答前一日、后一日或其它波动。latest quote 的涨跌幅只证明其 provider timestamp 对应的快照，不能证明另一历史交易日。宽基指数、行业板块与单只证券必须分开；若宽基不支持用户的“大跌”前提，明确说明观察范围不一致并继续核验板块/个股或做最小澄清，不得擅自挑另一天的大跌替换问题。原因事实必须由本轮明确覆盖同一对象与同一绝对市场本地日期的 Web/news/公告原文支持；否则先回答已核验的实际波动与范围，再写“原因本轮未完全核验”。",
    "逐项复核所有公司关系、新闻因果、日期、行情、财务与估值数字：实体 search/profile 只证明标的身份，不证明公司关系；关系、事件与因果结论必须有当前 web/news/公告或工具原文明确支持，并在相关事实同句或紧邻句末使用本轮工具实际返回的来源标题与原始 URL 做内联引用。URL 只用于定位来源，不证明句中内容；外部事实里的数字、排名或角色、合同权利义务、产品或芯片型号、估值标签都必须直接出现在该 URL 本轮返回的 title/content/snippet 中，否则删除。不得只写来源名、域名或与事实脱节的文末来源清单，也不得使用历史会话或模型记忆中的 URL。基于已核验事实形成的判断必须另起句并以‘推断：’开头。只有二级摘要时应继续找公司公告、监管文件或其它一手来源，若仍不可得则明确披露证据层级。未找到证据不等于事实不存在；否定某种关系同样需要本轮来源直接支持，否则只能披露本轮检索边界。",
    "年度数据不得写成 TTM；单季数据必须标明季度与报告期，年化时必须显示是“单季×4”还是“最近四季求和”及算术、分子分母口径，并披露季节性限制。",
    "准备在终稿引用营收、净利润、EPS、EBIT、EBITA、EBITDA、利润率、经营现金流、自由现金流或资产负债表数字时，先核对本轮 financials / earnings_outlook 中最新已披露报告期，把每个数字绑定到对应 date/period、hone_latest_quarter、hone_ttm.period_ends 或 hone_forward.forward_period_ends；“最新”只表示工具可见的最新已披露报告期，不得暗示未发布季度。检查季度、年度、最近四季与 forward 是否混用；EBIT、EBITA、EBITDA 与营业利润不得互相代替，来源未明确给出时不得自行补算。若关键数字的 provider 窗口可能滞后或来源冲突，优先针对性查公司 IR、财报公告或监管文件核对报告日期与关键数字。这是生成前的时效和准确性复核引导，不是逐数字双来源、缺项拒答或自动改写门禁；官方二次核对不可得时，按当前结构化证据标明截至日、来源层级和具体缺口后继续回答。",
    "未取得净债务或企业价值时不得使用 EV 或 EV/EBITDA 标签，也不得把市值/EBITDA 写成 EV/EBITDA。quote 返回的 PE 未明确标注 forward 时不得称为 Forward PE；已核验期间 EBITDA 为正时不得声称公司需到未来才转正。",
    "没有直接证据与完整输入时，不得给出目标价、概率、仓位比例、止损位或精确支撑位；第三方分析师目标价必须标注为第三方聚合口径与对应时间，不得直接作为交易锚点。",
    "某项证据不可得时，披露缺项并继续完成能够被当前证据支持的分析。回答范围和篇幅跟随用户原问题：关系问答只回答已核验关系、必要推断和关键缺口，不得为凑单股模板扩写公司介绍、风险清单或交易建议。最终回答只面向用户问题与本轮证据。"
);
#[cfg(test)]
const FINAL_RELATIONSHIP_DELETION_CHECK: &str = "【最后一步：严格服从结构化交接】逐句对照下方 facts / resolved_evidence / inferences / gaps。每条外部关系事实都必须来自同一 fact 的已解析证据，并在句旁内联该证据的标题与原始 URL；客户/供应商方向、核心/最大/头部、持股或无股权、具体产品型号、合同数量、议价权与估值标签都不得从常识、搜索顺序或其它来源扩写。否定关系必须有直接否定 excerpt；gap 或未检索到绝不等于不存在。任何超出来源字面的判断只能使用交接里已有 inference，另起句并以‘推断：’开头。";
const DIRECT_FINAL_RELATIONSHIP_CHECK: &str = "【关系回答最后一步】逐句对照本轮真实工具结果。每条外部关系事实都必须由当前 Web/news/公告原文明示，并在句旁内联该来源标题与原始 URL；客户/供应商方向、核心/最大/头部、大客户、持股或无股权、具体产品型号、合同数量、议价权、高度依赖、锁定和多重绑定都不得从常识、搜索顺序或其它来源扩写。否定关系必须有直接否定原文；未检索到绝不等于不存在。任何超出来源字面的判断必须另起句并以‘推断：’开头；若没有足够前提，就保持中性事实归纳，不生成该判断。";

#[cfg(test)]
const MAX_RESEARCH_HANDOFF_BYTES: usize = 32 * 1024;
#[cfg(test)]
const MAX_RESEARCH_FACTS: usize = 24;
#[cfg(test)]
const MAX_RESEARCH_INFERENCES: usize = 16;
#[cfg(test)]
const MAX_RESEARCH_GAPS: usize = 16;
#[cfg(test)]
const MAX_RESEARCH_REFS_PER_FACT: usize = 6;
#[cfg(test)]
const MAX_RESEARCH_TEXT_CHARS: usize = 1200;
#[cfg(test)]
const MIN_WEB_EXCERPT_CHARS: usize = 8;
#[cfg(test)]
const MAX_FALLBACK_EVIDENCE_ITEMS: usize = 64;
#[cfg(test)]
const MAX_FALLBACK_SCANNED_SCALARS_PER_TOOL: usize = 256;
#[cfg(test)]
const MAX_FALLBACK_ITEMS_PER_TOOL: usize = 32;
#[cfg(test)]
const MAX_INTERNAL_VALIDATION_WARNINGS: usize = 8;
#[cfg(test)]
const MAX_UNAVAILABLE_FINISH_CORRECTIONS: u32 = 1;
#[cfg(test)]
const MAX_INVALID_FINISH_CORRECTIONS: u32 = 1;
#[cfg(test)]
const MAX_RESEARCH_SOURCE_CATALOG_ITEMS: usize = 32;
#[cfg(test)]
const EMPTY_TERMINAL_VISIBLE_CONTENT_ERROR: &str =
    "terminal synthesis returned empty visible content";

#[cfg(test)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ResearchHandoff {
    answer_scope: String,
    facts: Vec<ResearchHandoffFact>,
    inferences: Vec<ResearchHandoffInference>,
    gaps: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ResearchHandoffFact {
    id: String,
    evidence: Vec<ResearchEvidenceRef>,
}

#[cfg(test)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ResearchHandoffInference {
    claim: String,
    premise_fact_ids: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ResearchEvidenceRef {
    tool_call_id: String,
    result_number: Option<usize>,
    exact_excerpt: Option<String>,
    json_pointer: Option<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResearchEvidenceSource {
    tool_call_id: String,
    kind: ResearchEvidenceSourceKind,
    description: String,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResearchEvidenceSourceKind {
    DataFetch,
    WebSearch,
}

/// Mechanically narrows fallback extraction to the source locations that the
/// Agent selected in its finish handoff. This is intentionally not an entity
/// or answer-quality classifier: it only prevents unrelated current-turn tool
/// results from being replayed merely because they happened to succeed.
#[cfg(test)]
#[derive(Debug, Clone, Default)]
struct FallbackEvidenceScope {
    web_result_numbers: BTreeMap<String, BTreeSet<usize>>,
    data_json_pointers: BTreeMap<String, BTreeSet<String>>,
}

#[cfg(test)]
impl FallbackEvidenceScope {
    fn observe_reference(&mut self, reference: &ResearchEvidenceRef) {
        let tool_call_id = reference.tool_call_id.trim();
        if tool_call_id.is_empty() {
            return;
        }
        match (reference.result_number, reference.json_pointer.as_deref()) {
            (Some(result_number), None) if result_number > 0 => {
                self.web_result_numbers
                    .entry(tool_call_id.to_string())
                    .or_default()
                    .insert(result_number);
            }
            (None, Some(json_pointer))
                if json_pointer.trim().starts_with("/data/")
                    && json_pointer.trim().chars().count() <= MAX_RESEARCH_TEXT_CHARS =>
            {
                self.data_json_pointers
                    .entry(tool_call_id.to_string())
                    .or_default()
                    .insert(json_pointer.trim().to_string());
            }
            _ => {}
        }
    }

    fn is_empty(&self) -> bool {
        self.web_result_numbers.is_empty() && self.data_json_pointers.is_empty()
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Serialize)]
struct ValidatedResearchHandoff {
    answer_scope: String,
    facts: Vec<ValidatedResearchFact>,
    inferences: Vec<ResearchHandoffInference>,
    gaps: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fallback_evidence: Vec<Value>,
    #[serde(skip)]
    validation_warnings: Vec<String>,
    #[serde(skip)]
    unresolved_reference_count: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Serialize)]
struct ValidatedResearchFact {
    id: String,
    resolved_evidence: Vec<Value>,
}

#[async_trait]
pub trait FunctionCallingStreamObserver: Send + Sync {
    async fn on_content_delta(&self, content: &str);

    /// A delta from a tool-free terminal synthesis round. The default keeps
    /// existing observers source-compatible; channel adapters may override it
    /// when they need to distinguish draft-capable tool rounds from a final
    /// stream that can no longer be followed by another tool call.
    async fn on_final_content_delta(&self, content: &str) {
        self.on_content_delta(content).await;
    }

    /// Returns an exact user-visible prefix that has already crossed an
    /// irreversible channel boundary. Most observers buffer/reset all output
    /// and therefore return `None`; canonical terminal observers use this to
    /// permit a terminal-only transport recovery without rerunning tools.
    fn committed_visible_prefix(&self) -> Option<String> {
        None
    }

    /// Ask the unique downstream sink to ACK a typed service-owned finance
    /// prefix after a read-only DataFetch batch has mechanically activated the
    /// protocol. Ordinary observers fail closed.
    async fn commit_service_owned_prefix(&self, _prefix: &str) -> bool {
        false
    }

    /// A provider reasoning/thinking delta (deepseek `reasoning_content`,
    /// or adapters mapping upstream reasoning summaries onto that field).
    /// Never part of the visible answer; observers may surface it as a live
    /// thinking track. Default keeps existing observers source-compatible.
    async fn on_reasoning_delta(&self, _content: &str) {}

    async fn on_content_reset(&self);
}

#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug, Default)]
struct ResearchIdentityRouteEvidence {
    explicit: bool,
    identity_match_declared: bool,
    search_attempts: u32,
    empty_search_results: u32,
    post_identity_attempts: u32,
    query_aliases: BTreeSet<String>,
    exact_symbol_constraint: Option<String>,
    candidates: BTreeSet<String>,
    quote_symbols: BTreeSet<String>,
    asset_route_symbols: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentitySearchMatchMode {
    ExactSymbol,
    NameOrAlias,
}

impl ResearchIdentityRouteEvidence {
    fn symbol_matches_constraint(&self, symbol: &str) -> bool {
        self.exact_symbol_constraint
            .as_deref()
            .map_or(true, |constraint| {
                provider_symbols_equivalent(constraint, symbol)
            })
    }

    fn retain_symbols_matching_constraint(&mut self) {
        let Some(constraint) = self.exact_symbol_constraint.clone() else {
            return;
        };
        self.candidates
            .retain(|symbol| provider_symbols_equivalent(&constraint, symbol));
        self.quote_symbols
            .retain(|symbol| provider_symbols_equivalent(&constraint, symbol));
        self.asset_route_symbols
            .retain(|symbol| provider_symbols_equivalent(&constraint, symbol));
    }

    fn retain_symbols_matching_candidates(&mut self) {
        let candidates = self.candidates.clone();
        self.quote_symbols.retain(|symbol| {
            candidates
                .iter()
                .any(|candidate| provider_symbols_equivalent(candidate, symbol))
        });
        self.asset_route_symbols.retain(|symbol| {
            candidates
                .iter()
                .any(|candidate| provider_symbols_equivalent(candidate, symbol))
        });
        self.retain_symbols_matching_constraint();
    }

    fn is_covered(&self) -> bool {
        !self.candidates.is_empty()
            && self.quote_symbols.iter().any(|quote_symbol| {
                self.symbol_matches_constraint(quote_symbol)
                    && self.asset_route_symbols.iter().any(|asset_symbol| {
                        self.symbol_matches_constraint(asset_symbol)
                            && provider_symbols_equivalent(quote_symbol, asset_symbol)
                            && self.candidates.iter().any(|candidate| {
                                self.symbol_matches_constraint(candidate)
                                    && provider_symbols_equivalent(candidate, quote_symbol)
                            })
                    })
            })
    }

    fn has_bounded_no_coverage(&self) -> bool {
        self.candidates.is_empty()
            && self.search_attempts >= 2
            && self.empty_search_results >= 2
            && self.post_identity_attempts > 0
    }
}

#[derive(Debug, Default)]
struct ResearchEvidenceLedger {
    identity_only_attempts: u32,
    unscoped_identity_search_attempts: u32,
    broad_data_attempts: u32,
    symbol_scoped_attempts: u32,
    broad_market_mode: bool,
    broad_market_quote_groups: BTreeSet<String>,
    post_activation_attempts: u32,
    post_identity_attempts: u32,
    post_identity_quote_attempts: u32,
    post_identity_asset_route_attempts: u32,
    identity_routes: BTreeMap<String, ResearchIdentityRouteEvidence>,
    identity_route_limit: Option<usize>,
    rejected_identity_routes: u32,
}

impl ResearchEvidenceLedger {
    fn with_identity_route_limit(limit: Option<usize>) -> Self {
        Self {
            identity_route_limit: limit,
            ..Self::default()
        }
    }

    fn active_route_keys(&self) -> Vec<String> {
        self.identity_routes.keys().cloned().collect()
    }

    /// Only a valid identity-search route may enter the evidence floor. The
    /// complete turn has a hard route ceiling: the service cannot prove that
    /// every search in the first model batch came from the user rather than
    /// model-memory screening, so initial and later routes share one bound.
    fn identity_search_admission_error(&mut self, tool_call: &ToolCall) -> Option<Value> {
        if !is_identity_only_search_call(tool_call) {
            return None;
        }
        if !data_fetch_identity_search_shape_is_valid(tool_call) {
            self.rejected_identity_routes = self.rejected_identity_routes.saturating_add(1);
            return Some(serde_json::json!({
                "error": "invalid_identity_search_shape",
                "message": "DataFetch search 必须包含有效 query；显式 entity_route 还必须在本次调用声明 exact_symbol 或 name_or_alias，且 refines_query/supersedes_query 不能并用"
            }));
        }
        let limit = self.identity_route_limit?;
        let (route_key, _) = self.resolve_identity_route_key(tool_call)?;
        if self.identity_routes.contains_key(&route_key) {
            return None;
        }
        let migrates_existing = data_fetch_identity_migration_source(tool_call)
            .map(|query| self.identity_routes.contains_key(&format!("query:{query}")))
            .unwrap_or(false);
        if migrates_existing || self.identity_routes.len() < limit {
            return None;
        }
        self.rejected_identity_routes = self.rejected_identity_routes.saturating_add(1);
        Some(serde_json::json!({
            "error": "identity_route_limit_reached",
            "limit": limit,
            "screening_candidate_limit": MAX_AGENT_OWNED_SCREENING_CANDIDATE_ROUTES,
            "message": "本轮实体路线已达上限；优先保留用户点名标的，复用已接纳的 entity_route 并完成其 snapshot/证据，不要新增候选"
        }))
    }

    fn agent_guidance_summary(&self) -> String {
        if self.identity_routes.is_empty() {
            if self.broad_market_mode {
                if self.broad_market_quote_groups.len() >= 2 {
                    return format!(
                        "宽市场代表性行情已覆盖 {} 组（{}）：无需为宽基 ETF/指数补证券身份 search；读取实际 quote 结果，并只在需要解释目标日期原因时补同日 Web 来源后完成回答。",
                        self.broad_market_quote_groups.len(),
                        self.broad_market_quote_groups
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("、")
                    );
                }
                return format!(
                    "这是宽市场问题：已尝试 {} 组代表性行情，至少用 data_fetch quote 的 symbol/ticker（或带 identity_match=exact_symbol 的 query 兼容形态）覆盖两个不同宽基/风格组；无需为宽基 ETF/指数执行证券身份 search。",
                    self.broad_market_quote_groups.len()
                );
            }
            return "尚未建立任何证券实体路线：重新阅读完整用户原话，并为每个点名标的分别执行带 entity_route 与 identity_match 的 search。"
                .to_string();
        }

        self.identity_routes
            .iter()
            .map(|(key, route)| {
                let mut pending = Vec::new();
                if route.is_covered() {
                    pending.push("结构调用已按同一候选代码成对尝试；成功、空结果、失败与证据质量仍须读取 tool result 判断");
                } else if route.has_bounded_no_coverage() {
                    pending.push("有界无覆盖调用已尝试；须读取 tool result 并在终稿准确披露具体缺口");
                } else {
                    if !route.explicit {
                        pending.push("需要用显式 entity_route 绑定该 search（必要时用逐字 supersedes_query）");
                    }
                    if route.search_attempts == 0
                        || (route.explicit && !route.identity_match_declared)
                    {
                        pending.push("缺少带 call-scoped identity_match 的有效 search");
                    } else if route.candidates.is_empty() {
                        pending.push("当前 search 无有效候选，需要在同一路线 refinement 或完成有界无覆盖尝试");
                    } else {
                        let has_candidate_quote = route.quote_symbols.iter().any(|symbol| {
                            route.symbol_matches_constraint(symbol)
                                && route.candidates.iter().any(|candidate| {
                                    provider_symbols_equivalent(candidate, symbol)
                                })
                        });
                        let has_candidate_asset_route =
                            route.asset_route_symbols.iter().any(|symbol| {
                                route.symbol_matches_constraint(symbol)
                                    && route.candidates.iter().any(|candidate| {
                                        provider_symbols_equivalent(candidate, symbol)
                                    })
                            });
                        if !has_candidate_quote {
                            pending.push("缺同路线同代码 quote");
                        }
                        if !has_candidate_asset_route {
                            pending.push("缺同路线同代码 profile/snapshot（crypto 用 crypto_quote）");
                        }
                        if has_candidate_quote
                            && has_candidate_asset_route
                            && !route.is_covered()
                        {
                            pending.push("quote 与 profile/asset-route 尚未落在同一候选代码");
                        }
                    }
                }

                let candidates = serde_json::to_string(
                    &route.candidates.iter().cloned().collect::<Vec<_>>(),
                )
                .unwrap_or_else(|_| "[]".to_string());
                let state = pending.join("；");
                let route_label = if let Some(raw_route) = key.strip_prefix("route:") {
                    format!(
                        "entity_route={}",
                        serde_json::to_string(raw_route).unwrap_or_else(|_| "\"\"".to_string())
                    )
                } else if let Some(query) = key.strip_prefix("query:") {
                    format!(
                        "未绑定的 provisional query={}",
                        serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string())
                    )
                } else {
                    "未知内部路线".to_string()
                };
                format!("- {route_label}: candidates={candidates}；{state}")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn resolve_identity_route_key(&self, tool_call: &ToolCall) -> Option<(String, bool)> {
        if !is_identity_only_search_call(tool_call) {
            return None;
        }
        if let Some(route_key) = data_fetch_explicit_entity_route_key(tool_call) {
            return Some((route_key, true));
        }
        // An untagged call may bind back to an explicit route only when its
        // actual executed query is an exact known alias. Self-labelled
        // refines/supersedes metadata cannot rewrite another explicit route.
        let aliases = data_fetch_search_query(tool_call)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let matching_explicit_routes = self
            .identity_routes
            .iter()
            .filter(|(_, route)| {
                route.explicit
                    && route
                        .query_aliases
                        .iter()
                        .any(|alias| aliases.contains(alias))
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if let [route_key] = matching_explicit_routes.as_slice() {
            return Some((route_key.clone(), false));
        }
        data_fetch_identity_route_key(tool_call)
    }

    fn migrate_implicit_routes_for_explicit_search(
        &mut self,
        tool_call: &ToolCall,
        explicit_route_key: &str,
    ) {
        if let Some(implicit_query) = data_fetch_identity_migration_source(tool_call) {
            let implicit_key = format!("query:{implicit_query}");
            let implicit = (implicit_key != explicit_route_key)
                .then(|| self.identity_routes.remove(&implicit_key))
                .flatten();
            if let Some(implicit) = implicit {
                let explicit = self
                    .identity_routes
                    .entry(explicit_route_key.to_string())
                    .or_default();
                explicit.explicit = true;
                explicit.search_attempts = explicit
                    .search_attempts
                    .saturating_add(implicit.search_attempts);
                explicit.empty_search_results = explicit
                    .empty_search_results
                    .saturating_add(implicit.empty_search_results);
                // Migration carries attempt history, exact-text aliases, and a
                // previously Agent-declared exact constraint. Candidate,
                // quote/profile, and untyped follow-up evidence remain
                // provisional; the explicit route must establish its own
                // candidate and then collect route-correct evidence.
                explicit.query_aliases.extend(implicit.query_aliases);
                if explicit.exact_symbol_constraint.is_none() {
                    explicit.exact_symbol_constraint = implicit.exact_symbol_constraint;
                }
                explicit.retain_symbols_matching_constraint();
            }
        }
        let explicit = self
            .identity_routes
            .entry(explicit_route_key.to_string())
            .or_default();
        explicit.explicit = true;
        explicit.query_aliases.extend(
            data_fetch_search_query(tool_call)
                .into_iter()
                .chain(data_fetch_refines_query(tool_call))
                .chain(data_fetch_supersedes_query(tool_call)),
        );
    }

    fn observe_route_symbols(
        &mut self,
        tool_call: &ToolCall,
        symbols: &BTreeSet<String>,
        quote: bool,
        asset_route: bool,
    ) {
        if symbols.is_empty() {
            return;
        }
        if let Some(route_key) = data_fetch_explicit_entity_route_key(tool_call)
            && let Some(route) = self.identity_routes.get_mut(&route_key)
        {
            if route.search_attempts == 0
                || !route.identity_match_declared
                || route.candidates.is_empty()
            {
                return;
            }
            let matching_symbols = symbols
                .iter()
                .filter(|symbol| {
                    route.symbol_matches_constraint(symbol)
                        && route
                            .candidates
                            .iter()
                            .any(|candidate| provider_symbols_equivalent(candidate, symbol))
                })
                .cloned()
                .collect::<BTreeSet<_>>();
            if quote {
                route.quote_symbols.extend(matching_symbols.iter().cloned());
            }
            if asset_route {
                route.asset_route_symbols.extend(matching_symbols);
            }
            return;
        }

        // Untagged calls are backward compatible only when a symbol belongs
        // to exactly one active route. Provider noise or overlapping aliases
        // cannot let one company's quote/profile unlock another route.
        let active_route_keys = self.active_route_keys();
        for symbol in symbols {
            let matching_routes = active_route_keys
                .iter()
                .filter(|key| {
                    self.identity_routes.get(*key).is_some_and(|route| {
                        route
                            .candidates
                            .iter()
                            .any(|candidate| provider_symbols_equivalent(candidate, symbol))
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            if let [route_key] = matching_routes.as_slice() {
                if let Some(route) = self.identity_routes.get_mut(route_key) {
                    if quote {
                        route.quote_symbols.insert(symbol.clone());
                    }
                    if asset_route {
                        route.asset_route_symbols.insert(symbol.clone());
                    }
                }
            }
        }
    }

    fn observe_route_non_search_attempt(
        &mut self,
        tool_call: &ToolCall,
        symbols: &BTreeSet<String>,
    ) {
        if let Some(route_key) = data_fetch_explicit_entity_route_key(tool_call)
            && let Some(route) = self.identity_routes.get_mut(&route_key)
        {
            if route.search_attempts == 0 || !route.identity_match_declared {
                return;
            }
            let symbol_matches_route = if route.candidates.is_empty() {
                route.empty_search_results >= 2
                    && !symbols.is_empty()
                    && route
                        .exact_symbol_constraint
                        .as_deref()
                        .map_or(true, |constraint| {
                            symbols
                                .iter()
                                .any(|symbol| provider_symbols_equivalent(constraint, symbol))
                        })
            } else {
                symbols.iter().any(|symbol| {
                    route.symbol_matches_constraint(symbol)
                        && route
                            .candidates
                            .iter()
                            .any(|candidate| provider_symbols_equivalent(candidate, symbol))
                })
            };
            if !symbol_matches_route {
                return;
            }
            route.post_identity_attempts = route.post_identity_attempts.saturating_add(1);
            return;
        }
        let active_route_keys = self.active_route_keys();
        let matching_routes = active_route_keys
            .iter()
            .filter(|key| {
                self.identity_routes.get(*key).is_some_and(|route| {
                    symbols.iter().any(|symbol| {
                        route
                            .candidates
                            .iter()
                            .any(|candidate| provider_symbols_equivalent(candidate, symbol))
                    })
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        if let [route_key] = matching_routes.as_slice() {
            if let Some(route) = self.identity_routes.get_mut(route_key) {
                route.post_identity_attempts = route.post_identity_attempts.saturating_add(1);
            }
            return;
        }
        if symbols.is_empty()
            && active_route_keys.len() == 1
            && tool_call.function.name == "web_search"
        {
            // An unscoped Web/news follow-up can be attributed only when
            // exactly one route is still preparing bounded no-coverage. With
            // two empty routes the service cannot guess which entity it serves.
            let empty_routes = active_route_keys
                .iter()
                .filter(|key| {
                    self.identity_routes.get(*key).is_some_and(|route| {
                        route.candidates.is_empty()
                            && route.search_attempts >= 2
                            && route.empty_search_results >= 2
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            if let [route_key] = empty_routes.as_slice() {
                if let Some(route) = self.identity_routes.get_mut(route_key) {
                    route.post_identity_attempts = route.post_identity_attempts.saturating_add(1);
                }
            }
        }
    }

    fn observe_business_call(&mut self, tool_call: &ToolCall, active_business_round: bool) {
        // A malformed function payload never counts as an evidence attempt.
        // The normal execution path will return its parse error to the Agent,
        // which can then issue a corrected business call.
        if serde_json::from_str::<Value>(&tool_call.function.arguments).is_err() {
            return;
        }
        if active_business_round {
            self.post_activation_attempts = self.post_activation_attempts.saturating_add(1);
        }
        if is_identity_only_search_call(tool_call) {
            if !data_fetch_identity_search_shape_is_valid(tool_call) {
                return;
            }
            if let Some((route_key, explicit)) = self.resolve_identity_route_key(tool_call) {
                // `identity_match` is call-scoped. This applies both to calls
                // carrying `entity_route` and to later untagged calls that
                // resolve back to an already-explicit route by exact alias.
                let targets_explicit_route = explicit
                    || self
                        .identity_routes
                        .get(&route_key)
                        .is_some_and(|route| route.explicit);
                if targets_explicit_route && data_fetch_identity_match_mode(tool_call).is_none() {
                    return;
                }
                self.identity_only_attempts = self.identity_only_attempts.saturating_add(1);
                if explicit {
                    // An explicit route retires only the provisional route
                    // named by this exact query/refines_query. One tagged
                    // entity must never hide every other untagged entity.
                    self.migrate_implicit_routes_for_explicit_search(tool_call, &route_key);
                }
                let route = self.identity_routes.entry(route_key).or_default();
                route.explicit |= explicit;
                if let Some(match_mode) = data_fetch_identity_match_mode(tool_call) {
                    route.identity_match_declared = true;
                    if match_mode == IdentitySearchMatchMode::ExactSymbol {
                        if let Some(query) = data_fetch_search_query_raw(tool_call) {
                            // The first explicit ticker fixes this route's
                            // identity. A later different ticker cannot widen
                            // it; bounded provider separator variants normalize
                            // to the same value.
                            if route.exact_symbol_constraint.is_none() {
                                route.exact_symbol_constraint = provider_canonical_key(&query);
                            }
                            route.retain_symbols_matching_constraint();
                        }
                    }
                }
                if let Some(query) = data_fetch_search_query(tool_call) {
                    route.query_aliases.insert(query);
                }
                route.search_attempts = route.search_attempts.saturating_add(1);
            } else {
                self.identity_only_attempts = self.identity_only_attempts.saturating_add(1);
                self.unscoped_identity_search_attempts =
                    self.unscoped_identity_search_attempts.saturating_add(1);
            }
            return;
        }

        let data_type = data_fetch_data_type(tool_call);
        let symbols = data_fetch_target_symbols(tool_call);
        if self.broad_market_mode && matches!(data_type.as_deref(), Some("quote" | "snapshot")) {
            self.broad_market_quote_groups.extend(
                symbols
                    .iter()
                    .filter_map(|symbol| broad_us_market_symbol_group(symbol))
                    .map(str::to_string),
            );
        }
        if !symbols.is_empty() {
            self.symbol_scoped_attempts = self.symbol_scoped_attempts.saturating_add(1);
        } else if data_type.as_deref().is_some_and(is_broad_data_type) {
            self.broad_data_attempts = self.broad_data_attempts.saturating_add(1);
        }

        // Evidence gathered before the first identity-search attempt cannot
        // satisfy the post-identity floor. This keeps an out-of-order quote or
        // profile from silently replacing the entity-resolution step, while
        // still allowing the same assistant turn to batch search first and
        // then exact-symbol evidence calls.
        if self.identity_only_attempts > 0 {
            self.post_identity_attempts = self.post_identity_attempts.saturating_add(1);
            self.observe_route_non_search_attempt(tool_call, &symbols);
            match data_type.as_deref() {
                Some("quote" | "quote_short") => {
                    self.post_identity_quote_attempts =
                        self.post_identity_quote_attempts.saturating_add(1);
                    self.observe_route_symbols(tool_call, &symbols, true, false);
                }
                Some("crypto_quote") => {
                    // A structured crypto search followed by crypto_quote is
                    // the complete price + asset-route path. Requiring a stock
                    // profile here would deadlock a valid crypto request.
                    self.post_identity_quote_attempts =
                        self.post_identity_quote_attempts.saturating_add(1);
                    self.post_identity_asset_route_attempts =
                        self.post_identity_asset_route_attempts.saturating_add(1);
                    self.observe_route_symbols(tool_call, &symbols, true, true);
                }
                Some("profile") => {
                    self.post_identity_asset_route_attempts =
                        self.post_identity_asset_route_attempts.saturating_add(1);
                    self.observe_route_symbols(tool_call, &symbols, false, true);
                }
                Some("snapshot" | "earnings_outlook") => {
                    // DataFetch snapshot and earnings_outlook are canonical
                    // combined quote/profile routes. One real attempt therefore
                    // proves both structural steps even when the provider
                    // reports a field-level error that the Agent must disclose.
                    self.post_identity_quote_attempts =
                        self.post_identity_quote_attempts.saturating_add(1);
                    self.post_identity_asset_route_attempts =
                        self.post_identity_asset_route_attempts.saturating_add(1);
                    self.observe_route_symbols(tool_call, &symbols, true, true);
                }
                _ => {}
            }
        }
    }

    #[cfg(test)]
    fn completion_signal_available(&self, active_business_round: bool) -> bool {
        self.evidence_floor_satisfied(active_business_round)
    }

    fn evidence_floor_satisfied(&self, active_business_round: bool) -> bool {
        if !active_business_round {
            return false;
        }

        if self.broad_market_mode && self.broad_market_quote_groups.len() >= 2 {
            return true;
        }

        let route_keys = self.active_route_keys();
        let security_path = self.identity_only_attempts > 0
            || self.symbol_scoped_attempts > 0
            || !route_keys.is_empty();
        if !security_path {
            return self.broad_data_attempts > 0;
        }
        if self.identity_only_attempts == 0 || self.post_identity_attempts == 0 {
            return false;
        }
        !route_keys.is_empty()
            && route_keys.iter().all(|key| {
                self.identity_routes.get(key).is_some_and(|route| {
                    route.search_attempts > 0
                        && (!route.explicit || route.identity_match_declared)
                        && (route.is_covered() || route.has_bounded_no_coverage())
                })
            })
    }

    fn observe_business_result(
        &mut self,
        tool_call: &ToolCall,
        tool_result: &Value,
        _active_business_round: bool,
    ) {
        if !is_identity_only_search_call(tool_call) {
            return;
        }
        if !data_fetch_identity_search_shape_is_valid(tool_call) {
            return;
        }
        let Some((route_key, explicit)) = self.resolve_identity_route_key(tool_call) else {
            return;
        };
        let query = data_fetch_search_query_raw(tool_call).unwrap_or_default();
        let match_mode = data_fetch_identity_match_mode(tool_call);
        if (explicit
            || self
                .identity_routes
                .get(&route_key)
                .is_some_and(|route| route.explicit))
            && match_mode.is_none()
        {
            return;
        }
        let exact_symbol_constraint = self
            .identity_routes
            .get(&route_key)
            .and_then(|route| route.exact_symbol_constraint.clone());
        let mut candidates = identity_search_route_candidates(tool_result, &query, match_mode);
        if let Some(exact_symbol_constraint) = exact_symbol_constraint.as_deref() {
            candidates
                .retain(|symbol| provider_symbols_equivalent(exact_symbol_constraint, symbol));
        }
        let route = self.identity_routes.entry(route_key).or_default();
        route.explicit |= explicit;
        if candidates.is_empty() {
            route.empty_search_results = route.empty_search_results.saturating_add(1);
            // Bounded no-coverage requires a real follow-up after the latest
            // unsuccessful identity attempt. Evidence collected for an older
            // candidate (or between two empty attempts) cannot satisfy a later
            // empty generation.
            route.post_identity_attempts = 0;
            if match_mode == Some(IdentitySearchMatchMode::ExactSymbol)
                || exact_symbol_constraint.is_some()
            {
                route.candidates.clear();
                route.quote_symbols.clear();
                route.asset_route_symbols.clear();
            }
        } else {
            // A later exact/refined result is authoritative for this declared
            // route. Replace earlier broad/noisy candidates instead of unioning
            // them into a permanently ambiguous set.
            route.empty_search_results = 0;
            route.candidates = candidates;
            route.retain_symbols_matching_candidates();
        }
    }

    fn observe_business_failure(&mut self, tool_call: &ToolCall) {
        if !is_identity_only_search_call(tool_call) {
            return;
        }
        if !data_fetch_identity_search_shape_is_valid(tool_call) {
            return;
        }
        if let Some((route_key, explicit)) = self.resolve_identity_route_key(tool_call) {
            if (explicit
                || self
                    .identity_routes
                    .get(&route_key)
                    .is_some_and(|route| route.explicit))
                && data_fetch_identity_match_mode(tool_call).is_none()
            {
                return;
            }
            if let Some(route) = self.identity_routes.get_mut(&route_key) {
                route.empty_search_results = route.empty_search_results.saturating_add(1);
                if route.candidates.is_empty() {
                    route.post_identity_attempts = 0;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StreamToolChoiceTelemetry {
    requested: ToolChoiceMode,
    effective: Option<ToolChoiceMode>,
    fallback: Option<bool>,
}

impl StreamToolChoiceTelemetry {
    fn new(requested: ToolChoiceMode) -> Self {
        Self {
            requested,
            effective: None,
            fallback: None,
        }
    }

    fn observe(
        &mut self,
        requested: ToolChoiceMode,
        effective: ToolChoiceMode,
        fallback: bool,
    ) -> hone_core::HoneResult<()> {
        if self.effective.is_some() {
            return Err(hone_core::HoneError::Llm(
                "stream returned duplicate tool choice metadata".to_string(),
            ));
        }
        if requested != self.requested {
            return Err(hone_core::HoneError::Llm(format!(
                "stream tool choice metadata mismatch: requested {}, expected {}",
                tool_choice_mode_name(requested),
                tool_choice_mode_name(self.requested),
            )));
        }
        self.effective = Some(effective);
        self.fallback = Some(fallback);
        Ok(())
    }
}

fn tool_choice_mode_name(mode: ToolChoiceMode) -> &'static str {
    match mode {
        ToolChoiceMode::Auto => "auto",
        ToolChoiceMode::Required => "required",
    }
}

fn trace_has_only_known_read_only_calls(tool_calls: &[ToolCallMade]) -> bool {
    !tool_calls.is_empty()
        && tool_calls
            .iter()
            .all(|call| tool_call_is_known_read_only(&call.name, &call.arguments))
}

fn observe_stream_finish(
    finish: &mut Option<ChatStreamFinishReason>,
    reason: ChatStreamFinishReason,
) -> hone_core::HoneResult<()> {
    if finish.is_some() {
        return Err(hone_core::HoneError::Llm(
            "stream returned duplicate finish reason".to_string(),
        ));
    }
    match reason {
        ChatStreamFinishReason::Stop | ChatStreamFinishReason::ToolCalls => {
            *finish = Some(reason);
            Ok(())
        }
        ChatStreamFinishReason::Length => Err(hone_core::HoneError::Llm(
            "stream completion was truncated (finish reason: length)".to_string(),
        )),
        ChatStreamFinishReason::ContentFilter => Err(hone_core::HoneError::Llm(
            "stream completion was blocked (finish reason: content_filter)".to_string(),
        )),
        ChatStreamFinishReason::Error => Err(hone_core::HoneError::Llm(
            "stream completion failed (finish reason: error)".to_string(),
        )),
        ChatStreamFinishReason::Other(reason) => Err(hone_core::HoneError::Llm(format!(
            "stream completion ended with unsupported finish reason: {reason}"
        ))),
    }
}

fn require_complete_stream(
    telemetry: &StreamToolChoiceTelemetry,
    finish: Option<ChatStreamFinishReason>,
    done: bool,
    expected_finish: ChatStreamFinishReason,
    operation: &str,
) -> hone_core::HoneResult<()> {
    if telemetry.effective.is_none() {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream ended without tool choice metadata"
        )));
    }
    if !done {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream ended before Done"
        )));
    }
    let Some(actual_finish) = finish else {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream reached Done without a finish reason"
        )));
    };
    if actual_finish != expected_finish {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream finish mismatch: expected {expected_finish:?}, got {actual_finish:?}"
        )));
    }
    Ok(())
}

/// Function Calling Agent
/// Lines in a would-be final answer that self-report missing first-party
/// evidence ("本轮未读到…原文" style). They are the gap checklist a
/// gap-closure round hands back to the same agent.
/// A Chinese question answered by an essentially English body. Soft prompt
/// guidance alone proved insufficient for long structured answers built from
/// English sources (audited in production: a Chinese question about MRVL came
/// back with an English report carrying only the injected Chinese first line),
/// so the final draft gets one language-repair round before publication.
/// Thresholds are deliberately conservative: ordinary Chinese finance answers
/// are CJK-dense even when packed with tickers and tables.
fn final_language_mismatch(user_input: &str, draft: &str) -> bool {
    let cjk = |text: &str| {
        text.chars()
            .filter(|ch| matches!(*ch as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF))
            .count()
    };
    let question_cjk = cjk(user_input);
    if question_cjk < 2 {
        return false;
    }
    let draft_letters = draft.chars().filter(char::is_ascii_alphabetic).count();
    if draft_letters < 600 {
        return false;
    }
    let draft_cjk = cjk(draft);
    draft_cjk * 100 < draft_letters * 15
}

fn language_final_correction_prompt(draft: &str) -> String {
    let bounded: String = draft.chars().take(24_000).collect();
    format!(
        "【内部终稿语言纠正】上一版终稿尚未发布正文。用户以中文提问，终稿必须全程使用简体中文。\
         把 <unpublished_draft> 完整改写为简体中文成品：所有事实、数字、百分比、表格结构、ticker、交易所代码、链接与引用来源逐项保留，不新增、不删减、不重算任何事实；英文引用的原文标题可保留原文并附中文说明；小标题与结论一并译为中文。\
         重写稿仍须从精确数据时间首行开始。不要向用户提及本检查。\n<unpublished_draft>\n{bounded}\n</unpublished_draft>"
    )
}

fn research_gap_lines(content: &str) -> Vec<String> {
    const GAP_MARKERS: [&str; 8] = [
        "未读到",
        "未能读到",
        "未核验",
        "无法确认",
        "未能确认",
        "没有直接原文",
        "未见一手",
        "未取得一手",
    ];
    let mut lines: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && GAP_MARKERS.iter().any(|marker| line.contains(marker)))
        .map(|line| line.chars().take(200).collect())
        .collect();
    lines.truncate(8);
    lines
}

fn gap_closure_feedback_prompt(gap_lines: &[String], draft: &str) -> String {
    let checklist = gap_lines
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let draft_excerpt: String = draft
        .chars()
        .take(MAX_MARKET_MOVE_DRAFT_FEEDBACK_CHARS)
        .collect();
    format!(
        "【内部缺口闭环轮】上一版终稿尚未发布，其中仍自述缺少下列一手证据：\n{checklist}\n\
         本轮工具重新开放。请针对上述每一项分别继续取证，每项至少换用两种不同策略：\
         `web_search` 换英文或本地化关键词（可用 site: 限定市政府、公司 IR、SEC、监管机构等官方域名）、\
         `data_fetch` 的 sec_filings / press_releases / transcript / analyst_actions 等一手类型，必要时并行多条查询。\
         取得的证据融入终稿并附来源链接；经两种以上策略实际尝试仍不可得的项目，才保留为缺口并写明尝试过的方向。\
         终稿仍从精确数据时间首行开始。不要向用户提及本说明、内部轮次或工具预算。\n\
         <unpublished_draft>\n{draft_excerpt}\n</unpublished_draft>"
    )
}

pub struct FunctionCallingAgent {
    pub llm: Arc<dyn LlmProvider>,
    pub tools: Arc<ToolRegistry>,
    pub system_prompt: String,
    pub max_iterations: u32,
    pub debug_log: bool,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub tool_observer: Option<Arc<dyn ToolExecutionObserver>>,
    pub stream_observer: Option<Arc<dyn FunctionCallingStreamObserver>>,
    pub max_tool_calls: Option<u32>,
    pub tool_call_limits: HashMap<String, u32>,
    pub agent_owned_finance_loop: bool,
    /// Business evidence the service loaded into this turn's input before the
    /// first model call. Context only — it never fixes an entity or constrains
    /// the answer; it only means the turn did not start evidence-free.
    pub preloaded_evidence_calls: u32,
    pub service_owned_initial_prefix: Option<String>,
    pub precommitted_service_prefix: Option<String>,
    /// Whether the service authorized committing its owned prefix before the
    /// model's own final answer reproduces it (the runner's before-model
    /// pre-commit and this agent's post-evidence mid-loop commit). When false,
    /// the only commit source is real final-answer bytes flowing through the
    /// stream observer.
    pub allow_pre_final_prefix_commit: bool,
    /// Research budget for the agent-owned finance loop. Defaults to the
    /// conservative constants; deployments raise it (and enable gap-closure
    /// rounds) via config so "最新进展"-style questions can keep chasing
    /// self-reported evidence gaps instead of settling at the first draft.
    pub finance_research_budget: FinanceResearchBudget,
    /// Images attached to the current user turn. Handed to the model as real
    /// image parts on the turn's user message, so a vision-capable model sees
    /// the attachment instead of reasoning from its filename.
    pub turn_images: Vec<hone_llm::MessageImage>,
    #[cfg(test)]
    pub finish_research_terminal_synthesis: bool,
    pub step_timeout: Option<Duration>,
    pub overall_timeout: Option<Duration>,
}

/// Tool budget for one agent-owned finance turn plus how many gap-closure
/// rounds a self-reported-incomplete final draft may trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinanceResearchBudget {
    pub tool_rounds: u32,
    pub tool_calls: u32,
    pub data_fetch_calls: u32,
    pub web_search_calls: u32,
    pub gap_closure_rounds: u32,
}

impl Default for FinanceResearchBudget {
    fn default() -> Self {
        Self {
            tool_rounds: MAX_AGENT_OWNED_FINANCE_TOOL_ROUNDS,
            tool_calls: MAX_AGENT_OWNED_FINANCE_TOOL_CALLS,
            data_fetch_calls: MAX_AGENT_OWNED_FINANCE_DATA_FETCH_CALLS,
            web_search_calls: MAX_AGENT_OWNED_FINANCE_WEB_SEARCH_CALLS,
            gap_closure_rounds: 0,
        }
    }
}

impl FunctionCallingAgent {
    pub fn new(
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        system_prompt: String,
        max_iterations: u32,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        let debug_log = std::env::var("HONE_AGENT_DEBUG")
            .map(|v| matches!(v.trim(), "1" | "true" | "True"))
            .unwrap_or(false);

        Self {
            llm,
            tools,
            system_prompt,
            max_iterations,
            debug_log,
            llm_audit,
            tool_observer: None,
            stream_observer: None,
            max_tool_calls: None,
            tool_call_limits: HashMap::new(),
            agent_owned_finance_loop: false,
            finance_research_budget: FinanceResearchBudget::default(),
            turn_images: Vec::new(),
            preloaded_evidence_calls: 0,
            service_owned_initial_prefix: None,
            precommitted_service_prefix: None,
            allow_pre_final_prefix_commit: false,
            #[cfg(test)]
            finish_research_terminal_synthesis: false,
            step_timeout: None,
            overall_timeout: None,
        }
    }

    pub fn with_tool_observer(mut self, observer: Option<Arc<dyn ToolExecutionObserver>>) -> Self {
        self.tool_observer = observer;
        self
    }

    pub fn with_stream_observer(
        mut self,
        observer: Option<Arc<dyn FunctionCallingStreamObserver>>,
    ) -> Self {
        self.stream_observer = observer;
        self
    }

    pub fn with_tool_call_budget(
        mut self,
        max_tool_calls: Option<u32>,
        tool_call_limits: HashMap<String, u32>,
    ) -> Self {
        self.max_tool_calls = max_tool_calls;
        self.tool_call_limits = tool_call_limits;
        self
    }

    /// Test-only coverage for the retired research terminal protocol. Once the Agent has
    /// actually attempted DataFetch in an eligible turn, the same business
    /// loop first requires a post-identity evidence attempt, then exposes the
    /// real actor-bound tools together with a sole `finish_research` signal.
    /// DataFetch is the structural finance-evidence boundary already required
    /// by the investment prompt; using it avoids a question-phrase classifier
    /// and does not force unrelated Web/file/skill tool turns into the
    /// canonical investment answer format. A sole finish signal performs one
    /// final tool-free streamed completion using the same in-memory context.
    /// Direct answers before finance research remain exact one-shot answers.
    #[cfg(test)]
    pub fn with_finish_research_terminal_synthesis(mut self, enabled: bool) -> Self {
        self.finish_research_terminal_synthesis = enabled;
        if enabled {
            self.agent_owned_finance_loop = false;
        }
        self
    }

    #[cfg(test)]
    fn finish_research_terminal_synthesis_enabled(&self) -> bool {
        self.finish_research_terminal_synthesis
    }

    /// Keep Interactive finance research in the ordinary function-calling
    /// loop. DataFetch activates the request-local entity/evidence ledger;
    /// before its structural floor the Agent must keep using real tools, and
    /// after the floor it may either use another real tool or return one
    /// natural `Stop + Done` answer. This mode never exposes a retired control
    /// tool and never starts a tool-free rewrite.
    pub fn with_preloaded_evidence_calls(mut self, calls: u32) -> Self {
        self.preloaded_evidence_calls = calls;
        self
    }

    pub fn with_agent_owned_finance_loop(mut self, enabled: bool) -> Self {
        self.agent_owned_finance_loop = enabled;
        #[cfg(test)]
        if enabled {
            self.finish_research_terminal_synthesis = false;
        }
        self
    }

    /// Override the agent-owned finance research budget (config-driven).
    /// Values below the compiled defaults are honored as tighter budgets.
    pub fn with_finance_research_budget(mut self, budget: FinanceResearchBudget) -> Self {
        self.finance_research_budget = budget;
        self
    }

    /// Attach this turn's user images. They ride the current user message only;
    /// replayed history stays text so old turns cannot resend media.
    pub fn with_turn_images(mut self, images: Vec<hone_llm::MessageImage>) -> Self {
        self.turn_images = images;
        self
    }

    /// Attach a trusted service-owned first line and separately record whether
    /// the Web publication sink ACKed it. An unacknowledged prefix still
    /// constrains the final model output, but never counts as visible bytes.
    pub fn with_service_owned_initial_prefix(
        mut self,
        required: Option<String>,
        precommitted: Option<String>,
    ) -> Self {
        self.service_owned_initial_prefix = required;
        self.precommitted_service_prefix = precommitted;
        self
    }

    pub fn with_pre_final_prefix_commit(mut self, allow: bool) -> Self {
        self.allow_pre_final_prefix_commit = allow;
        self
    }

    /// Apply one absolute deadline to the complete Agent loop. The deadline is
    /// created once at `run` entry and is never reset between model, tool,
    /// terminal, or recovery phases.
    pub fn with_overall_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.overall_timeout = timeout.filter(|timeout| !timeout.is_zero());
        self
    }

    /// Apply a fresh per-step deadline to each model, tool, and observer
    /// await. This bounds a single stalled phase while `overall_timeout`
    /// remains one absolute, non-resetting deadline for the whole Agent run.
    pub fn with_step_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.step_timeout = timeout.filter(|timeout| !timeout.is_zero());
        self
    }

    fn dbg(&self, msg: &str) {
        if self.debug_log {
            tracing::debug!("{msg}");
        }
    }

    async fn reset_emitted_content(&self, emitted: bool) {
        if emitted && let Some(observer) = &self.stream_observer {
            // A committed canonical prefix is irreversible. Resetting after it
            // would make a successful buffered recovery impossible to append
            // byte-for-byte and can cause visible flicker in non-deferred
            // adapters.
            if observer.committed_visible_prefix().is_none() {
                observer.on_content_reset().await;
            }
        }
    }

    /// 构建完整消息列表（system prompt + context messages）
    fn build_messages(
        &self,
        context: &AgentContext,
        additional_system_instruction: Option<&str>,
    ) -> Vec<Message> {
        self.build_messages_from_index(context, additional_system_instruction, 0)
    }

    fn build_messages_from_index(
        &self,
        context: &AgentContext,
        additional_system_instruction: Option<&str>,
        message_start: usize,
    ) -> Vec<Message> {
        let message_start = message_start.min(context.messages.len());
        let mut messages =
            Vec::with_capacity(context.messages.len().saturating_sub(message_start) + 1);

        if !self.system_prompt.is_empty() || additional_system_instruction.is_some() {
            let system_prompt = match (self.system_prompt.is_empty(), additional_system_instruction)
            {
                (false, Some(instruction)) => {
                    format!("{}\n\n{}", self.system_prompt, instruction)
                }
                (true, Some(instruction)) => instruction.to_string(),
                (_, None) => self.system_prompt.clone(),
            };
            messages.push(Message {
                images: Vec::new(),
                role: "system".to_string(),
                content: Some(system_prompt),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        for msg in &context.messages[message_start..] {
            messages.push(Message {
                images: Vec::new(),
                role: msg.role.clone(),
                content: msg.content.clone(),
                reasoning_content: msg
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string),
                tool_calls: msg.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                        .collect()
                }),
                tool_call_id: msg.tool_call_id.clone(),
                name: msg.name.clone(),
            });
        }

        // This turn's attachments ride the most recent user message. Replayed
        // history stays text-only, so an old turn can never resend its media
        // and re-bill it on every later round.
        if !self.turn_images.is_empty()
            && let Some(last_user) = messages.iter_mut().rev().find(|m| m.role == "user")
        {
            last_user.images = self.turn_images.clone();
        }

        messages
    }

    /// Preserve a small conversational window for follow-up references without
    /// replaying historical assistant claims, tool protocol, prices, or
    /// reasoning into a new research ledger. Previous user wording may explain
    /// pronouns such as "它" or "第二个"; only the current runtime input defines
    /// what must be researched in this turn.
    fn build_agent_owned_messages(
        &self,
        context: &AgentContext,
        additional_system_instruction: Option<&str>,
        turn_message_start: usize,
    ) -> Vec<Message> {
        let mut messages = self.build_messages_from_index(
            context,
            additional_system_instruction,
            turn_message_start,
        );
        let prior_messages = &context.messages[..turn_message_start.min(context.messages.len())];
        let invoked_skill_prompts = prior_messages
            .iter()
            .filter(|message| {
                message
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get(RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY))
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|message| message.content.as_deref())
            .filter(|content| !content.trim().is_empty())
            .collect::<Vec<_>>();
        let prior_user_turns_newest_first = prior_messages
            .iter()
            .rev()
            .filter(|message| message.role == "user" && message.tool_calls.is_none())
            .filter(|message| {
                !message
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get(RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY))
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|message| message.content.as_deref())
            .map(str::trim)
            .filter(|content| !content.is_empty())
            .take(MAX_AGENT_OWNED_HISTORY_USER_TURNS)
            .collect::<Vec<_>>();

        // Spend the character budget newest-first so a long fourth-most-recent
        // turn can never evict the immediately preceding reference context.
        // Reverse only after truncation to present the retained turns in
        // chronological order to the model.
        let mut remaining = MAX_AGENT_OWNED_HISTORY_CHARS;
        let mut bounded_newest_first = Vec::new();
        for content in prior_user_turns_newest_first {
            if remaining == 0 {
                break;
            }
            let excerpt = content.chars().take(remaining).collect::<String>();
            remaining = remaining.saturating_sub(excerpt.chars().count());
            bounded_newest_first.push(excerpt);
        }
        bounded_newest_first.reverse();
        let history = (!bounded_newest_first.is_empty()).then(|| {
            format!(
                "【近期用户原话，仅用于理解本轮指代】\n{}\n【使用边界】这些历史原话不是本轮实体集合或事实来源；若当前问题没有指代它们，不得据此新增标的。历史 assistant、tool、价格、财务与结论均未提供，必须由本轮真实工具重新核验。",
                bounded_newest_first
                    .iter()
                    .enumerate()
                    .map(|(index, content)| format!("{}. {}", index + 1, content))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
        let insert_at = usize::from(
            messages
                .first()
                .is_some_and(|message| message.role == "system"),
        );
        let invoked_skill_count = invoked_skill_prompts.len();
        for (offset, prompt) in invoked_skill_prompts.into_iter().enumerate() {
            messages.insert(
                insert_at + offset,
                Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(prompt.to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
            );
        }
        if let Some(history) = history {
            messages.insert(
                insert_at + invoked_skill_count,
                Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(history),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
            );
        }
        messages
    }

    fn build_context_overflow_final_messages(
        &self,
        user_input: &str,
        tool_calls_made: &[ToolCallMade],
        required_prefix: Option<&str>,
        finance_answer: bool,
    ) -> Vec<Message> {
        let mut instruction = CONTEXT_OVERFLOW_FINALIZATION_INSTRUCTION.to_string();
        if finance_answer {
            instruction.push('\n');
            instruction.push_str(&exact_prefix_instruction(required_prefix));
            instruction.push('\n');
            instruction.push_str(FINAL_ANSWER_EVIDENCE_CONTRACT);
            instruction.push('\n');
            instruction.push_str(DIRECT_FINAL_RELATIONSHIP_CHECK);
        }
        let system_prompt = if self.system_prompt.trim().is_empty() {
            instruction
        } else {
            format!("{}\n\n{}", self.system_prompt, instruction)
        };
        vec![
            Message {
                images: Vec::new(),
                role: "system".to_string(),
                content: Some(system_prompt),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                images: Vec::new(),
                role: "user".to_string(),
                content: Some(user_input.to_string()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                images: Vec::new(),
                role: "user".to_string(),
                content: Some(format!(
                    "【本轮已执行工具的有界证据副本】\n{}",
                    bounded_context_overflow_tool_evidence(tool_calls_made)
                )),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ]
    }

    async fn record_audit(
        &self,
        context: &AgentContext,
        operation: &str,
        request: Value,
        response: Option<Value>,
        error: Option<String>,
        latency_ms: u128,
        metadata: Value,
        usage: Option<hone_llm::provider::TokenUsage>,
    ) {
        let Some(sink) = &self.llm_audit else {
            return;
        };
        let mut record = LlmAuditRecord::new(
            context.session_id.clone(),
            context.actor_identity(),
            "agent.function_calling",
            operation.to_string(),
            "openrouter",
            None,
            request,
        );
        record.success = error.is_none();
        record.response = response;
        record.error = error;
        record.latency_ms = Some(latency_ms);
        record.metadata = metadata;
        if let Some(u) = usage {
            record.prompt_tokens = u.prompt_tokens;
            record.completion_tokens = u.completion_tokens;
            record.total_tokens = u.total_tokens;
        }
        if let Err(err) = sink.record(record).await {
            tracing::warn!(
                "[LlmAudit] failed to persist function_calling audit: {}",
                err
            );
        }
    }

    async fn chat_with_tools_streaming(
        &self,
        messages: &[Message],
        tools: &[Value],
        tool_choice_mode: ToolChoiceMode,
        emit_speculative_content: bool,
        telemetry: &mut StreamToolChoiceTelemetry,
    ) -> hone_core::HoneResult<ChatResponse> {
        let mut stream = self
            .llm
            .chat_with_tools_stream(messages, tools, None, tool_choice_mode);
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_calls = BTreeMap::<u32, PendingToolCall>::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut emitted_visible_content = false;
        let mut finish = None;
        let mut done = false;

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    self.reset_emitted_content(emitted_visible_content).await;
                    return Err(error);
                }
            };
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                return Err(hone_core::HoneError::Llm(
                    "chat_with_tools stream emitted payload before tool choice metadata"
                        .to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => {
                    if let Err(error) = telemetry.observe(requested, effective, fallback) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::ContentDelta(delta) => {
                    content.push_str(&delta);
                    if emit_speculative_content {
                        let visible = formatter.push(&delta);
                        if !visible.is_empty() && tool_calls.is_empty() {
                            if let Some(observer) = &self.stream_observer {
                                observer.on_content_delta(&visible).await;
                                emitted_visible_content = true;
                            }
                        }
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => {
                    if let Some(observer) = &self.stream_observer {
                        observer.on_reasoning_delta(&delta).await;
                    }
                    reasoning_content.push_str(&delta);
                }
                ChatStreamEvent::ToolCallDelta {
                    index,
                    id,
                    name,
                    arguments,
                } => {
                    if emit_speculative_content && tool_calls.is_empty() && emitted_visible_content
                    {
                        if let Some(observer) = &self.stream_observer {
                            observer.on_content_reset().await;
                        }
                        emitted_visible_content = false;
                    }
                    let pending = tool_calls.entry(index).or_default();
                    if let Some(id) = id {
                        pending.id.push_str(&id);
                    }
                    if let Some(name) = name {
                        pending.name.push_str(&name);
                    }
                    pending.arguments.push_str(&arguments);
                }
                ChatStreamEvent::Usage(value) => usage = Some(value),
                ChatStreamEvent::Finish(reason) => {
                    if let Err(error) = observe_stream_finish(&mut finish, reason) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }

        let has_tool_calls = !tool_calls.is_empty();
        if let Err(error) = require_complete_stream(
            telemetry,
            finish,
            done,
            if has_tool_calls {
                ChatStreamFinishReason::ToolCalls
            } else {
                ChatStreamFinishReason::Stop
            },
            "chat_with_tools",
        ) {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(error);
        }

        if emit_speculative_content && !has_tool_calls {
            let visible = formatter.finish();
            if !visible.is_empty()
                && let Some(observer) = &self.stream_observer
            {
                observer.on_content_delta(&visible).await;
            }
        }

        let tool_calls = (!tool_calls.is_empty()).then(|| {
            tool_calls
                .into_values()
                .map(|pending| ToolCall {
                    id: pending.id,
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: pending.name,
                        arguments: pending.arguments,
                    },
                })
                .collect()
        });

        Ok(ChatResponse {
            content,
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls,
            usage,
        })
    }

    async fn chat_active_business_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        tool_choice_mode: ToolChoiceMode,
        eligible_final_prefix: Option<&str>,
        session_id: &str,
        iteration: u32,
        telemetry: &mut StreamToolChoiceTelemetry,
        precommitted_service_prefix: Option<&str>,
    ) -> hone_core::HoneResult<ActiveBusinessStreamOutcome> {
        let mut stream = self
            .llm
            .chat_with_tools_stream(messages, tools, None, tool_choice_mode);
        let mut reasoning_content = String::new();
        let mut tool_calls = BTreeMap::<u32, PendingToolCall>::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut visible_content = String::new();
        let mut finish = None;
        let mut done = false;
        let stream_started = std::time::Instant::now();
        let early_final_eligible = eligible_final_prefix.is_some();
        let mut forwarded_final_delta = false;
        let mut eligible_visible_candidate = String::new();
        let mut early_final_rejected = false;
        let mut canonical_prefix_committed = precommitted_service_prefix.is_some();
        let mut forwarded_prefix_bytes = 0usize;
        let mut first_provider_delta_ms = None;
        let mut first_committed_prefix_ms = None;

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    self.reset_emitted_content(forwarded_final_delta).await;
                    return Err(error);
                }
            };
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                self.reset_emitted_content(forwarded_final_delta).await;
                return Err(hone_core::HoneError::Llm(
                    "active business stream emitted payload before tool choice metadata"
                        .to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => {
                    if let Err(error) = telemetry.observe(requested, effective, fallback) {
                        self.reset_emitted_content(forwarded_final_delta).await;
                        return Err(error);
                    }
                }
                // Some supported providers can still emit a short preamble
                // before a timely tool call, including after Required falls
                // back from a provider capability error. Keep it silent and
                // out of context, but continue polling for the tool call. The
                // outer ACTIVE_BUSINESS_TIMEOUT bounds a long/hung bypass.
                ChatStreamEvent::ContentDelta(delta) => {
                    if early_final_eligible
                        && first_provider_delta_ms.is_none()
                        && !delta.is_empty()
                    {
                        let elapsed_ms =
                            u64::try_from(stream_started.elapsed().as_millis()).unwrap_or(u64::MAX);
                        first_provider_delta_ms = Some(elapsed_ms);
                        tracing::info!(
                            target: "hone_agent::ttft",
                            session_id = %session_id,
                            iteration,
                            elapsed_ms,
                            "agent-owned finance received first provider content delta"
                        );
                    }
                    let visible = formatter.push(&delta);
                    visible_content.push_str(&visible);
                    if early_final_eligible
                        && tool_calls.is_empty()
                        && !early_final_rejected
                        && !visible.is_empty()
                    {
                        eligible_visible_candidate.push_str(&visible);
                        let expected_prefix = eligible_final_prefix
                            .expect("early-final eligibility requires an exact prefix");
                        let candidate = eligible_visible_candidate.trim_start();
                        let prefix_delta = canonical_prefix_delta(
                            expected_prefix,
                            candidate,
                            forwarded_prefix_bytes,
                        );
                        if prefix_delta.is_err() {
                            self.reset_emitted_content(forwarded_final_delta).await;
                            forwarded_final_delta = false;
                            forwarded_prefix_bytes = 0;
                            eligible_visible_candidate.clear();
                            early_final_rejected = true;
                        } else if precommitted_service_prefix.is_none()
                            && let Some(observer) = &self.stream_observer
                        {
                            let prefix_delta =
                                prefix_delta.expect("compatible prefix must produce a delta");
                            if !prefix_delta.is_empty() {
                                observer.on_final_content_delta(&prefix_delta).await;
                                forwarded_prefix_bytes =
                                    forwarded_prefix_bytes.saturating_add(prefix_delta.len());
                                forwarded_final_delta = true;
                            }
                            if first_committed_prefix_ms.is_none()
                                && let Some(prefix) = observer.committed_visible_prefix()
                                && committed_prefix_matches_required(&prefix, eligible_final_prefix)
                            {
                                canonical_prefix_committed = true;
                                let elapsed_ms =
                                    u64::try_from(stream_started.elapsed().as_millis())
                                        .unwrap_or(u64::MAX);
                                first_committed_prefix_ms = Some(elapsed_ms);
                                tracing::info!(
                                    target: "hone_agent::ttft",
                                    session_id = %session_id,
                                    iteration,
                                    elapsed_ms,
                                    prefix_bytes = prefix.len(),
                                    "agent-owned finance committed canonical header"
                                );
                            }
                        }
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => {
                    if let Some(observer) = &self.stream_observer {
                        observer.on_reasoning_delta(&delta).await;
                    }
                    reasoning_content.push_str(&delta);
                }
                ChatStreamEvent::ToolCallDelta {
                    index,
                    id,
                    name,
                    arguments,
                } => {
                    if early_final_eligible {
                        if self
                            .stream_observer
                            .as_ref()
                            .and_then(|observer| observer.committed_visible_prefix())
                            .is_some_and(|prefix| {
                                committed_prefix_matches_required(&prefix, eligible_final_prefix)
                            })
                        {
                            canonical_prefix_committed = true;
                        }
                        if forwarded_final_delta && !canonical_prefix_committed {
                            self.reset_emitted_content(true).await;
                            forwarded_final_delta = false;
                        }
                        eligible_visible_candidate.clear();
                        early_final_rejected = true;
                    }
                    let pending = tool_calls.entry(index).or_default();
                    if let Some(id) = id {
                        pending.id.push_str(&id);
                    }
                    if let Some(name) = name {
                        pending.name.push_str(&name);
                    }
                    pending.arguments.push_str(&arguments);
                }
                ChatStreamEvent::Usage(value) => usage = Some(value),
                ChatStreamEvent::Finish(reason) => {
                    if let Err(error) = observe_stream_finish(&mut finish, reason) {
                        self.reset_emitted_content(forwarded_final_delta).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }
        let tail = formatter.finish();
        visible_content.push_str(&tail);
        if early_final_eligible
            && tool_calls.is_empty()
            && !early_final_rejected
            && !tail.is_empty()
        {
            eligible_visible_candidate.push_str(&tail);
            let expected_prefix =
                eligible_final_prefix.expect("early-final eligibility requires an exact prefix");
            let candidate = eligible_visible_candidate.trim_start();
            let prefix_delta =
                canonical_prefix_delta(expected_prefix, candidate, forwarded_prefix_bytes);
            if prefix_delta.is_err() {
                self.reset_emitted_content(forwarded_final_delta).await;
                forwarded_final_delta = false;
                eligible_visible_candidate.clear();
            } else if precommitted_service_prefix.is_none()
                && let Some(observer) = &self.stream_observer
            {
                let prefix_delta = prefix_delta.expect("compatible prefix must produce a delta");
                if !prefix_delta.is_empty() {
                    observer.on_final_content_delta(&prefix_delta).await;
                    forwarded_final_delta = true;
                }
                if first_committed_prefix_ms.is_none()
                    && let Some(prefix) = observer.committed_visible_prefix()
                    && committed_prefix_matches_required(&prefix, eligible_final_prefix)
                {
                    let elapsed_ms =
                        u64::try_from(stream_started.elapsed().as_millis()).unwrap_or(u64::MAX);
                    first_committed_prefix_ms = Some(elapsed_ms);
                    tracing::info!(
                        target: "hone_agent::ttft",
                        session_id = %session_id,
                        iteration,
                        elapsed_ms,
                        prefix_bytes = prefix.len(),
                        "agent-owned finance committed canonical header"
                    );
                }
            }
        }

        if tool_calls.is_empty() {
            if let Err(error) = require_complete_stream(
                telemetry,
                finish,
                done,
                ChatStreamFinishReason::Stop,
                "active business",
            ) {
                self.reset_emitted_content(forwarded_final_delta).await;
                return Err(error);
            }
            return Ok(if visible_content.trim().is_empty() {
                ActiveBusinessStreamOutcome::Empty
            } else {
                let elapsed_ms =
                    u64::try_from(stream_started.elapsed().as_millis()).unwrap_or(u64::MAX);
                if early_final_eligible {
                    tracing::info!(
                        target: "hone_agent::ttft",
                        session_id = %session_id,
                        iteration,
                        elapsed_ms,
                        first_provider_delta_ms = ?first_provider_delta_ms,
                        first_committed_prefix_ms = ?first_committed_prefix_ms,
                        content_bytes = visible_content.len(),
                        "agent-owned finance completed natural direct final"
                    );
                }
                ActiveBusinessStreamOutcome::DirectFinal(ChatResponse {
                    content: visible_content,
                    reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
                    tool_calls: None,
                    usage,
                })
            });
        }

        if let Err(error) = require_complete_stream(
            telemetry,
            finish,
            done,
            ChatStreamFinishReason::ToolCalls,
            "active business",
        ) {
            self.reset_emitted_content(forwarded_final_delta).await;
            return Err(error);
        }

        let tool_calls = tool_calls
            .into_values()
            .map(|pending| ToolCall {
                id: pending.id,
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: pending.name,
                    arguments: pending.arguments,
                },
            })
            .collect::<Vec<_>>();
        Ok(ActiveBusinessStreamOutcome::Tools(ChatResponse {
            content: String::new(),
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls: Some(tool_calls),
            usage,
        }))
    }

    #[cfg(test)]
    async fn chat_terminal_streaming(
        &self,
        messages: &[Message],
        telemetry: &mut StreamToolChoiceTelemetry,
        emit_to_observer: bool,
    ) -> hone_core::HoneResult<ChatResponse> {
        let empty_tools = Vec::<Value>::new();
        let mut stream =
            self.llm
                .chat_with_tools_stream(messages, &empty_tools, None, ToolChoiceMode::Auto);
        let mut visible_content = String::new();
        let mut reasoning_content = String::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut unexpected_tool_call = false;
        let mut emitted_visible_content = false;
        let mut finish = None;
        let mut done = false;

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    self.reset_emitted_content(emitted_visible_content).await;
                    return Err(error);
                }
            };
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                return Err(hone_core::HoneError::Llm(
                    "terminal stream emitted payload before tool choice metadata".to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => {
                    if let Err(error) = telemetry.observe(requested, effective, fallback) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::ContentDelta(delta) => {
                    let visible = formatter.push(&delta);
                    visible_content.push_str(&visible);
                    if emit_to_observer
                        && !visible.is_empty()
                        && let Some(observer) = &self.stream_observer
                    {
                        observer.on_final_content_delta(&visible).await;
                        emitted_visible_content = true;
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => {
                    if let Some(observer) = &self.stream_observer {
                        observer.on_reasoning_delta(&delta).await;
                    }
                    reasoning_content.push_str(&delta);
                }
                ChatStreamEvent::ToolCallDelta { .. } => unexpected_tool_call = true,
                ChatStreamEvent::Usage(value) => usage = Some(value),
                ChatStreamEvent::Finish(reason) => {
                    if let Err(error) = observe_stream_finish(&mut finish, reason) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }

        if unexpected_tool_call {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(hone_core::HoneError::Llm(
                "tool-free terminal synthesis returned a tool call".to_string(),
            ));
        }

        if let Err(error) = require_complete_stream(
            telemetry,
            finish,
            done,
            ChatStreamFinishReason::Stop,
            "terminal synthesis",
        ) {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(error);
        }

        let visible = formatter.finish();
        visible_content.push_str(&visible);
        if emit_to_observer
            && !visible.is_empty()
            && let Some(observer) = &self.stream_observer
        {
            observer.on_final_content_delta(&visible).await;
        }

        if emit_to_observer
            && let Some(committed_prefix) = self
                .stream_observer
                .as_ref()
                .and_then(|observer| observer.committed_visible_prefix())
        {
            // A header-only terminal is not a complete answer. Treat this as
            // an interrupted terminal transport so run_terminal_synthesis can
            // use its one buffered, empty-tools recovery rather than publish a
            // bare timestamp line as success.
            validate_terminal_recovery_content(&visible_content, &committed_prefix)?;
        }

        if visible_content.trim().is_empty() {
            return Err(hone_core::HoneError::Llm(
                EMPTY_TERMINAL_VISIBLE_CONTENT_ERROR.to_string(),
            ));
        }

        Ok(ChatResponse {
            // Some compatible providers encode hidden reasoning inside the
            // content stream as <think> blocks. Return the same formatter-
            // reduced bytes that the observer sees so prefix validation,
            // persistence, and terminal recovery operate on one canonical
            // user-visible representation.
            content: visible_content,
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls: None,
            usage,
        })
    }

    #[cfg(test)]
    async fn run_terminal_synthesis(
        &self,
        context: &mut AgentContext,
        tool_calls_made: Vec<ToolCallMade>,
        completed_iterations: u32,
        turn_message_start: usize,
        handoff: &ValidatedResearchHandoff,
        required_prefix: Option<&str>,
        overall_deadline: Option<tokio::time::Instant>,
    ) -> AgentResponse {
        let iterations = completed_iterations.saturating_add(1);
        // The initial discovery round may use bounded conversation history to
        // resolve a follow-up pronoun. Once this turn has produced concrete
        // entity routes and evidence, neither business follow-ups nor terminal
        // synthesis need old user requests. Starting at turn_message_start
        // prevents a stale ticker or old requested format from contaminating
        // the current answer.
        let mut terminal_messages = self.build_messages_from_index(
            context,
            Some(FINISH_RESEARCH_SYSTEM_INSTRUCTION),
            turn_message_start,
        );
        // The validated handoff is the compact current-turn evidence boundary.
        // Do not replay assistant drafts, tool-call protocol frames, or full raw
        // tool payloads into the terminal model: doing so both weakens that
        // boundary and duplicates the largest part of the prompt. Retain only
        // system/user intent; resolved evidence is appended below.
        terminal_messages.retain(|message| matches!(message.role.as_str(), "system" | "user"));
        let terminal_prompt = terminal_synthesis_prompt(required_prefix, handoff);
        terminal_messages.push(Message {
            images: Vec::new(),
            role: "user".to_string(),
            content: Some(terminal_prompt),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        let terminal_request_payload = serde_json::json!({
            "messages": terminal_messages.clone(),
            "tools": Vec::<Value>::new(),
        });
        let terminal_started = std::time::Instant::now();
        let mut terminal_tool_choice = StreamToolChoiceTelemetry::new(ToolChoiceMode::Auto);
        let (terminal_deadline, terminal_timeout_error) =
            step_deadline(overall_deadline, self.step_timeout);
        let terminal_result = match await_before_deadline(
            terminal_deadline,
            terminal_timeout_error,
            self.chat_terminal_streaming(&terminal_messages, &mut terminal_tool_choice, true),
        )
        .await
        {
            Ok(response) => {
                self.record_audit(
                    context,
                    "chat_terminal_without_tools",
                    terminal_request_payload,
                    Some(serde_json::json!({
                        "content": response.content.clone(),
                        "tool_calls": response.tool_calls.clone(),
                    })),
                    None,
                    terminal_started.elapsed().as_millis(),
                    serde_json::json!({
                        "iteration": iterations,
                        "has_tools": false,
                        "finish_research": true,
                        "terminal_reason": "explicit_finish",
                        "terminal_recovery_eligible": false,
                        "requested_tool_choice": tool_choice_mode_name(terminal_tool_choice.requested),
                        "effective_tool_choice": terminal_tool_choice.effective.map(tool_choice_mode_name),
                        "tool_choice_fallback": terminal_tool_choice.fallback,
                    }),
                    response.usage.clone(),
                ).await;
                response
            }
            Err(error) => {
                let committed_prefix = self
                    .stream_observer
                    .as_ref()
                    .and_then(|observer| observer.committed_visible_prefix());
                self.record_audit(
                    context,
                    "chat_terminal_without_tools",
                    terminal_request_payload.clone(),
                    None,
                    Some(error.to_string()),
                    terminal_started.elapsed().as_millis(),
                    serde_json::json!({
                        "iteration": iterations,
                        "has_tools": false,
                        "finish_research": true,
                        "terminal_reason": "explicit_finish",
                        "terminal_recovery_eligible": committed_prefix.is_some()
                            || error.to_string().contains(EMPTY_TERMINAL_VISIBLE_CONTENT_ERROR),
                        "requested_tool_choice": tool_choice_mode_name(terminal_tool_choice.requested),
                        "effective_tool_choice": terminal_tool_choice.effective.map(tool_choice_mode_name),
                        "tool_choice_fallback": terminal_tool_choice.fallback,
                    }),
                    None,
                ).await;
                let empty_terminal = error
                    .to_string()
                    .contains(EMPTY_TERMINAL_VISIBLE_CONTENT_ERROR);
                if committed_prefix.is_none() && !empty_terminal {
                    return AgentResponse {
                        content: String::new(),
                        tool_calls_made,
                        iterations,
                        success: false,
                        error: Some(error.to_string()),
                    };
                }

                if canonical_agent_timeout(&error).is_some() {
                    return AgentResponse {
                        content: String::new(),
                        tool_calls_made,
                        iterations,
                        success: false,
                        error: Some(error.to_string()),
                    };
                }

                // The canonical header has already reached the user, so an
                // outer Agent/runner retry would either duplicate it or rerun
                // business tools. Retry this terminal transport exactly once,
                // buffered, against the same evidence and with tools disabled.
                let recovery_messages =
                    terminal_recovery_messages(&terminal_messages, committed_prefix.as_deref());
                let recovery_request_payload = serde_json::json!({
                    "messages": recovery_messages.clone(),
                    "tools": Vec::<Value>::new(),
                });
                let recovery_started = std::time::Instant::now();
                let mut recovery_tool_choice = StreamToolChoiceTelemetry::new(ToolChoiceMode::Auto);
                let (recovery_deadline, recovery_timeout_error) =
                    step_deadline(overall_deadline, self.step_timeout);
                let recovery_result = await_before_deadline(
                    recovery_deadline,
                    recovery_timeout_error,
                    self.chat_terminal_streaming(
                        &recovery_messages,
                        &mut recovery_tool_choice,
                        false,
                    ),
                )
                .await
                .and_then(|response| {
                    if let Some(committed_prefix) = committed_prefix.as_deref() {
                        validate_terminal_recovery_content(&response.content, committed_prefix)?;
                    }
                    Ok(response)
                });

                match recovery_result {
                    Ok(response) => {
                        self.record_audit(
                            context,
                            "chat_terminal_recovery_without_tools",
                            recovery_request_payload,
                            Some(serde_json::json!({
                                "content": response.content.clone(),
                                "tool_calls": response.tool_calls.clone(),
                            })),
                            None,
                            recovery_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "finish_research": true,
                                "terminal_reason": "explicit_finish",
                                "terminal_recovery": true,
                                "recovery_attempt": 1,
                                "committed_prefix_bytes": committed_prefix.as_deref().map_or(0, str::len),
                                "requested_tool_choice": tool_choice_mode_name(recovery_tool_choice.requested),
                                "effective_tool_choice": recovery_tool_choice.effective.map(tool_choice_mode_name),
                                "tool_choice_fallback": recovery_tool_choice.fallback,
                            }),
                            response.usage.clone(),
                        ).await;
                        response
                    }
                    Err(recovery_error) => {
                        self.record_audit(
                            context,
                            "chat_terminal_recovery_without_tools",
                            recovery_request_payload,
                            None,
                            Some(recovery_error.to_string()),
                            recovery_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "finish_research": true,
                                "terminal_reason": "explicit_finish",
                                "terminal_recovery": true,
                                "recovery_attempt": 1,
                                "committed_prefix_bytes": committed_prefix.as_deref().map_or(0, str::len),
                                "initial_terminal_error": error.to_string(),
                                "requested_tool_choice": tool_choice_mode_name(recovery_tool_choice.requested),
                                "effective_tool_choice": recovery_tool_choice.effective.map(tool_choice_mode_name),
                                "tool_choice_fallback": recovery_tool_choice.fallback,
                            }),
                            None,
                        ).await;
                        return AgentResponse {
                            content: String::new(),
                            tool_calls_made,
                            iterations,
                            success: false,
                            error: Some(format!(
                                "terminal synthesis recovery failed: {recovery_error}"
                            )),
                        };
                    }
                }
            }
        };

        // Terminal reasoning is neither user-visible output nor fact evidence.
        // Do not persist it into context, where a later turn could replay it.
        context.add_assistant_message_with_metadata(&terminal_result.content, None, None);
        AgentResponse {
            content: terminal_result.content,
            tool_calls_made,
            iterations,
            success: true,
            error: None,
        }
    }
}

#[cfg(test)]
fn terminal_recovery_messages(
    messages: &[Message],
    committed_prefix: Option<&str>,
) -> Vec<Message> {
    let mut recovery_messages = messages.to_vec();
    let recovery_constraint = match committed_prefix {
        Some(committed_prefix) => {
            let encoded_prefix = Value::String(committed_prefix.to_string()).to_string();
            format!(
                "\n【终稿传输恢复】上一次终稿流在已提交首行后中断。请基于完全相同的事实证据重新生成完整终稿；第一个字节起必须逐字输出以下 JSON 字符串解码后的已提交前缀，前面不得有任何字符：{encoded_prefix}。前缀后必须继续输出非空正文，其余事实边界与格式要求不变。不要提及本次传输恢复。"
            )
        }
        None => "\n【终稿传输恢复】上一次终稿流正常结束但没有可见正文。请基于完全相同的用户问题与事实证据直接生成一次非空完整终稿；不得调用工具、重新研究或提及本次传输恢复。".to_string(),
    };
    if let Some(prompt) = recovery_messages
        .last_mut()
        .and_then(|message| message.content.as_mut())
    {
        prompt.push_str(&recovery_constraint);
    } else {
        recovery_messages.push(Message {
            images: Vec::new(),
            role: "user".to_string(),
            content: Some(recovery_constraint),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }
    recovery_messages
}

#[cfg(test)]
fn validate_terminal_recovery_content(
    content: &str,
    committed_prefix: &str,
) -> hone_core::HoneResult<()> {
    let Some(tail) = content.strip_prefix(committed_prefix) else {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content does not start with the committed visible prefix"
                .to_string(),
        ));
    };
    if tail.trim().is_empty() {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content contains no body after the committed visible prefix"
                .to_string(),
        ));
    }
    if tail.trim_start().starts_with(committed_prefix) {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content repeats the committed visible prefix".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
fn finish_research_tool_schema(sources: &[ResearchEvidenceSource]) -> Value {
    let mut schema = serde_json::json!({
        "type": "function",
        "function": {
            "name": FINISH_RESEARCH_TOOL_NAME,
            "description": "Agent-owned structured evidence handoff. Call it by itself only after rereading the complete original question and all current tool results. Group current-turn evidence in facts without writing a claim: web evidence cites the exact tool_call_id, 1-based result_number, and verbatim excerpt; Hone injects the title and URL from that result. Structured DataFetch evidence cites the tool_call_id and one scalar JSON Pointer. Put only evidence-derived judgments in inferences and reference their premise fact IDs. Put attempted but unresolved dimensions in gaps; absence is never a negative fact. For a broad relationship question, first investigate the relevant commercial/customer-supplier/technology-contract and investment/ownership dimensions, preferably through SEC, company IR, or both parties' announcements. Hone checks only this provenance protocol, never the semantic answer, and then asks the same Agent for one tool-free final. Never mix this call with another function and never place final prose in the tool round.",
            "parameters": {
                "type": "object",
                "properties": {
                    "answer_scope": {
                        "type": "string",
                        "maxLength": MAX_RESEARCH_TEXT_CHARS,
                        "description": "Concise description of exactly what the user's original question asks; use it to keep the final answer minimal and on-scope."
                    },
                    "facts": {
                        "type": "array",
                        "maxItems": MAX_RESEARCH_FACTS,
                        "description": "Evidence groups available to the final answer; do not put a free-text factual claim here.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string", "maxLength": 64, "description": "Unique short evidence-group ID such as F1; this is not a factual claim."},
                                "evidence": {
                                    "type": "array",
                                    "maxItems": MAX_RESEARCH_REFS_PER_FACT,
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "tool_call_id": {"type": "string", "maxLength": 512, "description": "Copy one exact current-turn ID from the runtime source catalog. Never use a tool name such as web_search, quote, or profile as an ID."},
                                            "result_number": {"type": "integer", "minimum": 1, "description": "Required only for web_excerpt."},
                                            "exact_excerpt": {"type": "string", "maxLength": MAX_RESEARCH_TEXT_CHARS, "description": "For Web evidence: a verbatim substring from that result's title/content/snippet. Use this together with result_number and omit json_pointer."},
                                            "json_pointer": {"type": "string", "maxLength": MAX_RESEARCH_TEXT_CHARS, "description": "For DataFetch evidence: RFC 6901 pointer to one scalar field, for example /data/0/price. Omit result_number and exact_excerpt."}
                                        },
                                        "required": ["tool_call_id"],
                                        "additionalProperties": false
                                    }
                                }
                            },
                            "required": ["id", "evidence"],
                            "additionalProperties": false
                        }
                    },
                    "inferences": {
                        "type": "array",
                        "maxItems": MAX_RESEARCH_INFERENCES,
                        "description": "Judgments derived only from submitted fact IDs; these will be labeled as inference.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "claim": {"type": "string", "maxLength": MAX_RESEARCH_TEXT_CHARS},
                                "premise_fact_ids": {"type": "array", "maxItems": MAX_RESEARCH_FACTS, "items": {"type": "string", "maxLength": 64}}
                            },
                            "required": ["claim", "premise_fact_ids"],
                            "additionalProperties": false
                        }
                    },
                    "gaps": {
                        "type": "array",
                        "maxItems": MAX_RESEARCH_GAPS,
                        "description": "Material dimensions actually attempted but not verified; never convert a gap into a negative fact.",
                        "items": {"type": "string", "maxLength": MAX_RESEARCH_TEXT_CHARS}
                    }
                },
                "required": ["answer_scope", "facts", "inferences", "gaps"],
                "additionalProperties": false
            }
        }
    });
    if !sources.is_empty() {
        schema["function"]["parameters"]["properties"]["facts"]["items"]["properties"]["evidence"]
            ["items"]["properties"]["tool_call_id"]["enum"] = Value::Array(
            sources
                .iter()
                .map(|source| Value::String(source.tool_call_id.clone()))
                .collect(),
        );
    }
    schema
}

#[cfg(test)]
fn is_finish_research_call(tool_call: &ToolCall) -> bool {
    tool_call.function.name == FINISH_RESEARCH_TOOL_NAME
}

#[cfg(test)]
fn parse_finish_research_handoff(tool_call: &ToolCall) -> Result<ResearchHandoff, String> {
    if !is_finish_research_call(tool_call) {
        return Err("not a finish_research call".to_string());
    }
    if tool_call.function.arguments.len() > MAX_RESEARCH_HANDOFF_BYTES {
        return Err(format!(
            "handoff exceeds {MAX_RESEARCH_HANDOFF_BYTES} bytes"
        ));
    }
    serde_json::from_str::<ResearchHandoff>(&tool_call.function.arguments)
        .map_err(|error| format!("handoff JSON does not match the schema: {error}"))
}

#[cfg(test)]
fn json_pointer_targets_error_field(json_pointer: &str) -> bool {
    json_pointer.split('/').skip(1).any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "error" | "errors" | "iserror"
        ) || segment.to_ascii_lowercase().ends_with("_error")
    })
}

#[cfg(test)]
fn fallback_data_value_prefix(payload: &Value, json_pointer: &str) -> Option<String> {
    let pointer = json_pointer.trim();
    if !pointer.starts_with("/data/")
        || pointer.chars().count() > MAX_RESEARCH_TEXT_CHARS
        || json_pointer_targets_error_field(pointer)
    {
        return None;
    }
    match payload.pointer(pointer) {
        Some(Value::Object(_)) => return Some(pointer.to_string()),
        Some(Value::Bool(_) | Value::Number(_) | Value::String(_)) | None => {}
        // An explicitly selected array or null is not a leaf typo and must
        // never widen to its parent object. In particular, `/data/quote`
        // cannot authorize unrelated `/data/profile` or `/data/news` fields.
        Some(Value::Array(_) | Value::Null) => return None,
    }
    let parent = pointer.rsplit_once('/').map(|(parent, _)| parent)?;
    if parent.is_empty() || !payload.pointer(parent).is_some_and(Value::is_object) {
        return None;
    }
    // A selected scalar, or a misspelled leaf beneath a real selected object,
    // may recover sibling fields from that exact object. Arrays and roots are
    // never expanded, which isolates batched entities and financial periods.
    Some(parent.to_string())
}

#[cfg(test)]
fn fallback_scope_from_handoff(handoff: &ResearchHandoff) -> FallbackEvidenceScope {
    let mut scope = FallbackEvidenceScope::default();
    for fact in handoff.facts.iter().take(MAX_RESEARCH_FACTS) {
        for reference in fact.evidence.iter().take(MAX_RESEARCH_REFS_PER_FACT) {
            scope.observe_reference(reference);
        }
    }
    scope
}

#[cfg(test)]
fn fallback_scope_from_unvalidated_value(value: &Value) -> FallbackEvidenceScope {
    let mut scope = FallbackEvidenceScope::default();
    let Some(facts) = value.get("facts").and_then(Value::as_array) else {
        return scope;
    };
    for fact in facts.iter().take(MAX_RESEARCH_FACTS) {
        let Some(evidence) = fact.get("evidence").and_then(Value::as_array) else {
            continue;
        };
        for reference in evidence.iter().take(MAX_RESEARCH_REFS_PER_FACT) {
            let Some(tool_call_id) = reference.get("tool_call_id").and_then(Value::as_str) else {
                continue;
            };
            let recovered = ResearchEvidenceRef {
                tool_call_id: tool_call_id.to_string(),
                result_number: reference
                    .get("result_number")
                    .and_then(Value::as_u64)
                    .and_then(|number| usize::try_from(number).ok()),
                exact_excerpt: None,
                json_pointer: reference
                    .get("json_pointer")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
            };
            scope.observe_reference(&recovered);
        }
    }
    scope
}

#[cfg(test)]
fn fallback_scope_from_finish_calls(finish_calls: &[&ToolCall]) -> FallbackEvidenceScope {
    for finish_call in finish_calls {
        if finish_call.function.arguments.len() > MAX_RESEARCH_HANDOFF_BYTES {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&finish_call.function.arguments) else {
            continue;
        };
        let recovered = fallback_scope_from_unvalidated_value(&value);
        if !recovered.is_empty() {
            // Duplicated provider controls are alternatives, not additive
            // evidence authority. Use the first syntactically readable scoped
            // submission and never union mutually inconsistent duplicates.
            return recovered;
        }
    }
    FallbackEvidenceScope::default()
}

#[cfg(test)]
fn bounded_research_text(label: &str, value: &str, allow_empty: bool) -> Result<String, String> {
    let trimmed = value.trim();
    if !allow_empty && trimmed.is_empty() {
        return Err(format!("{label} is empty"));
    }
    if trimmed.chars().count() > MAX_RESEARCH_TEXT_CHARS {
        return Err(format!(
            "{label} exceeds {MAX_RESEARCH_TEXT_CHARS} characters"
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
fn current_turn_tool_result<'a>(
    context: &'a AgentContext,
    turn_message_start: usize,
    tool_call_id: &str,
) -> Option<(&'a str, &'a str)> {
    context.messages[turn_message_start.min(context.messages.len())..]
        .iter()
        .find(|message| {
            message.role == "tool"
                && message.tool_call_id.as_deref() == Some(tool_call_id)
                && message.name.as_deref().is_some()
                && message.content.as_deref().is_some()
        })
        .and_then(|message| Some((message.name.as_deref()?, message.content.as_deref()?)))
}

#[cfg(test)]
fn current_turn_tool_call(
    context: &AgentContext,
    turn_message_start: usize,
    tool_call_id: &str,
) -> Option<ToolCall> {
    context.messages[turn_message_start.min(context.messages.len())..]
        .iter()
        .filter_map(|message| message.tool_calls.as_ref())
        .flatten()
        .filter_map(|tool_call| serde_json::from_value::<ToolCall>(tool_call.clone()).ok())
        .find(|tool_call| tool_call.id == tool_call_id)
}

#[cfg(test)]
fn current_turn_data_fetch_type(
    context: &AgentContext,
    turn_message_start: usize,
    tool_call_id: &str,
) -> Option<String> {
    let tool_call = current_turn_tool_call(context, turn_message_start, tool_call_id)?;
    (tool_call.function.name == "data_fetch")
        .then(|| data_fetch_data_type(&tool_call))
        .flatten()
}

#[cfg(test)]
fn value_contains_research_scalar(value: &Value, remaining_depth: usize) -> bool {
    if remaining_depth == 0 {
        return false;
    }
    match value {
        Value::Bool(_) | Value::Number(_) => true,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => items
            .iter()
            .any(|item| value_contains_research_scalar(item, remaining_depth - 1)),
        Value::Object(fields) => fields.iter().any(|(key, item)| {
            !matches!(
                key.to_ascii_lowercase().as_str(),
                "error" | "errors" | "iserror"
            ) && !key.to_ascii_lowercase().ends_with("_error")
                && value_contains_research_scalar(item, remaining_depth - 1)
        }),
        Value::Null => false,
    }
}

#[cfg(test)]
fn current_turn_research_source_catalog(
    context: &AgentContext,
    turn_message_start: usize,
) -> Vec<ResearchEvidenceSource> {
    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();
    for message in &context.messages[turn_message_start.min(context.messages.len())..] {
        if sources.len() >= MAX_RESEARCH_SOURCE_CATALOG_ITEMS || message.role != "tool" {
            continue;
        }
        let (Some(tool_call_id), Some(tool_name), Some(content)) = (
            message.tool_call_id.as_deref().map(str::trim),
            message.name.as_deref(),
            message.content.as_deref(),
        ) else {
            continue;
        };
        if tool_call_id.is_empty() || !seen.insert(tool_call_id.to_string()) {
            continue;
        }
        let Some(tool_call) = current_turn_tool_call(context, turn_message_start, tool_call_id)
        else {
            continue;
        };
        if tool_call.function.name != tool_name {
            continue;
        }
        let Ok(payload) = serde_json::from_str::<Value>(content) else {
            continue;
        };
        if tool_result_is_failure(&payload) {
            continue;
        }
        let (kind, description) = if tool_name == "web_search" {
            let result_numbers = payload
                .get("results")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
                .filter_map(|(index, result)| {
                    let has_url = result
                        .get("url")
                        .and_then(Value::as_str)
                        .is_some_and(|url| !url.trim().is_empty());
                    let has_excerpt = ["title", "content", "snippet"].into_iter().any(|field| {
                        result
                            .get(field)
                            .and_then(Value::as_str)
                            .is_some_and(|text| {
                                text.trim().chars().count() >= MIN_WEB_EXCERPT_CHARS
                            })
                    });
                    (has_url && has_excerpt).then_some(index + 1)
                })
                .collect::<Vec<_>>();
            if result_numbers.is_empty() {
                continue;
            }
            (
                ResearchEvidenceSourceKind::WebSearch,
                format!(
                    "web_search; citable result_number={}",
                    result_numbers
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )
        } else if tool_name == "data_fetch" {
            let Some(data_type) = data_fetch_data_type(&tool_call) else {
                continue;
            };
            if data_type == "search"
                || !payload
                    .get("data")
                    .is_some_and(|data| value_contains_research_scalar(data, 8))
            {
                continue;
            }
            let targets = data_fetch_target_symbols(&tool_call);
            if targets.is_empty() {
                (
                    ResearchEvidenceSourceKind::DataFetch,
                    format!("data_fetch; data_type={data_type}"),
                )
            } else {
                (
                    ResearchEvidenceSourceKind::DataFetch,
                    format!(
                        "data_fetch; data_type={data_type}; target={}",
                        targets.into_iter().collect::<Vec<_>>().join(",")
                    ),
                )
            }
        } else {
            continue;
        };
        sources.push(ResearchEvidenceSource {
            tool_call_id: tool_call_id.to_string(),
            kind,
            description,
        });
    }
    sources
}

#[cfg(test)]
fn research_source_catalog_prompt(sources: &[ResearchEvidenceSource]) -> String {
    if sources.is_empty() {
        return "本轮尚无可作为 finish facts 的成功来源；不要编造 tool_call_id，能确认的缺项写入 gaps。"
            .to_string();
    }
    let entries = sources
        .iter()
        .map(|source| {
            format!(
                "- {} = {}",
                Value::String(source.tool_call_id.clone()),
                source.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "【本轮 finish_research 可引用来源目录】\n{entries}\n`tool_call_id` 必须逐字复制上面一个完整 ID；`web_search`、`quote`、`profile` 等工具名或 data_type 不是 ID。"
    )
}

#[cfg(test)]
fn validate_web_evidence_ref(
    tool_name: &str,
    tool_content: &str,
    tool_call_id: &str,
    result_number: usize,
    exact_excerpt: &str,
) -> Result<Value, String> {
    if tool_name != "web_search" {
        return Err(format!(
            "tool_call_id {tool_call_id} is {tool_name}, not web_search"
        ));
    }
    let parsed: Value = serde_json::from_str(tool_content)
        .map_err(|_| format!("tool_call_id {tool_call_id} did not return JSON"))?;
    if tool_result_is_failure(&parsed) {
        return Err(format!(
            "tool_call_id {tool_call_id} returned a failed Web result"
        ));
    }
    let results = parsed
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("tool_call_id {tool_call_id} has no results array"))?;
    let result_index = result_number
        .checked_sub(1)
        .ok_or_else(|| "result_number is 1-based and must be at least 1".to_string())?;
    let result = results.get(result_index).ok_or_else(|| {
        format!("tool_call_id {tool_call_id} has no result_number {result_number}")
    })?;
    let actual_url = result
        .get("url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!("tool_call_id {tool_call_id} result_number {result_number} has no citable URL")
        })?;
    let excerpt = bounded_research_text("exact_excerpt", exact_excerpt, false)?;
    if excerpt.chars().count() < MIN_WEB_EXCERPT_CHARS {
        return Err(format!(
            "exact_excerpt must contain at least {MIN_WEB_EXCERPT_CHARS} characters"
        ));
    }
    let matching_field = ["title", "content", "snippet"]
        .into_iter()
        .find(|field| {
            result
                .get(*field)
                .and_then(Value::as_str)
                .is_some_and(|value| value.contains(&excerpt))
        })
        .ok_or_else(|| {
            format!(
                "exact_excerpt is not a verbatim substring of tool_call_id {tool_call_id} result_number {result_number} title/content/snippet"
            )
        })?;
    Ok(serde_json::json!({
        "source_kind": "web_excerpt",
        "tool_call_id": tool_call_id,
        "result_number": result_number,
        "field": matching_field,
        "title": result.get("title").and_then(Value::as_str),
        "url": actual_url,
        "exact_excerpt": excerpt,
    }))
}

#[cfg(test)]
fn resolve_web_evidence_ref(
    context: &AgentContext,
    turn_message_start: usize,
    submitted_tool_call_id: &str,
    result_number: usize,
    exact_excerpt: &str,
) -> Result<Value, String> {
    let submitted_tool_call_id = submitted_tool_call_id.trim();
    if let Some((tool_name, content)) =
        current_turn_tool_result(context, turn_message_start, submitted_tool_call_id)
    {
        let invocation = current_turn_tool_call(
            context,
            turn_message_start,
            submitted_tool_call_id,
        )
        .ok_or_else(|| {
            format!("tool_call_id {submitted_tool_call_id} has no matching current-turn invocation")
        })?;
        if invocation.function.name != tool_name {
            return Err(format!(
                "tool_call_id {submitted_tool_call_id} invocation/result tool names do not match"
            ));
        }
        return validate_web_evidence_ref(
            tool_name,
            content,
            submitted_tool_call_id,
            result_number,
            exact_excerpt,
        );
    }

    // Some OpenAI-compatible providers treat opaque call IDs as disposable
    // protocol details and return the tool name instead. The Agent still made
    // an explicit evidence selection when it supplied a verbatim excerpt and
    // 1-based result number. Recover that locator only when the excerpt maps
    // to exactly one current-turn Web invocation; ambiguity authorizes
    // nothing and never widens to every successful result.
    let citable_web_ids = current_turn_research_source_catalog(context, turn_message_start)
        .into_iter()
        .filter(|source| source.kind == ResearchEvidenceSourceKind::WebSearch)
        .map(|source| source.tool_call_id)
        .collect::<BTreeSet<_>>();
    let mut matches = Vec::new();
    for message in &context.messages[turn_message_start.min(context.messages.len())..] {
        if message.role != "tool" || message.name.as_deref() != Some("web_search") {
            continue;
        }
        let (Some(actual_id), Some(content)) =
            (message.tool_call_id.as_deref(), message.content.as_deref())
        else {
            continue;
        };
        if !citable_web_ids.contains(actual_id) {
            continue;
        }
        if !current_turn_tool_call(context, turn_message_start, actual_id)
            .is_some_and(|tool_call| tool_call.function.name == "web_search")
        {
            continue;
        }
        if let Ok(resolved) = validate_web_evidence_ref(
            "web_search",
            content,
            actual_id,
            result_number,
            exact_excerpt,
        ) {
            matches.push(resolved);
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(format!(
            "tool_call_id {submitted_tool_call_id} is not a current-turn result and the selected excerpt did not identify a unique current-turn Web result"
        )),
        _ => Err(format!(
            "tool_call_id {submitted_tool_call_id} is not a current-turn result and the selected excerpt is ambiguous across current-turn Web results"
        )),
    }
}

#[cfg(test)]
fn validate_data_evidence_ref(
    tool_name: &str,
    tool_content: &str,
    tool_call_id: &str,
    json_pointer: &str,
    data_type: &str,
) -> Result<Value, String> {
    if tool_name != "data_fetch" {
        return Err(format!(
            "tool_call_id {tool_call_id} is {tool_name}, not data_fetch"
        ));
    }
    if data_type == "search" {
        return Err(
            "DataFetch search results are identity candidates and cannot be terminal facts"
                .to_string(),
        );
    }
    let pointer = bounded_research_text("json_pointer", json_pointer, false)?;
    if !pointer.starts_with('/') {
        return Err("json_pointer must be a non-root RFC 6901 pointer".to_string());
    }
    let parsed: Value = serde_json::from_str(tool_content)
        .map_err(|_| format!("tool_call_id {tool_call_id} did not return JSON"))?;
    if tool_result_is_failure(&parsed) {
        return Err(format!(
            "tool_call_id {tool_call_id} returned a failed/error result"
        ));
    }
    if json_pointer_targets_error_field(&pointer) {
        return Err("json_pointer must not resolve an error field".to_string());
    }
    let resolved = parsed.pointer(&pointer).ok_or_else(|| {
        format!("json_pointer {pointer} does not exist in tool_call_id {tool_call_id}")
    })?;
    if !matches!(
        resolved,
        Value::Bool(_) | Value::Number(_) | Value::String(_)
    ) || resolved
        .as_str()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(format!(
            "json_pointer {pointer} must resolve to one non-null scalar field"
        ));
    }
    let encoded = serde_json::to_string(resolved).unwrap_or_default();
    if encoded.chars().count() > MAX_RESEARCH_TEXT_CHARS {
        return Err(format!(
            "json_pointer {pointer} resolves to more than {MAX_RESEARCH_TEXT_CHARS} characters; point to a narrower field"
        ));
    }
    Ok(serde_json::json!({
        "source_kind": "data_field",
        "tool_call_id": tool_call_id,
        "tool_name": tool_name,
        "json_pointer": pointer,
        "value": resolved,
    }))
}

#[cfg(test)]
fn tool_result_is_failure(value: &Value) -> bool {
    if value.get("isError").and_then(Value::as_bool) == Some(true) {
        return true;
    }
    if value
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| {
            matches!(
                status.trim().to_ascii_lowercase().as_str(),
                "failed" | "failure" | "error"
            )
        })
    {
        return true;
    }
    value.get("error").is_some_and(|error| match error {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
        _ => true,
    })
}

#[cfg(test)]
fn push_validation_warning(warnings: &mut Vec<String>, warning: impl Into<String>) {
    if warnings.len() < MAX_INTERNAL_VALIDATION_WARNINGS {
        warnings.push(warning.into());
    }
}

#[cfg(test)]
fn truncate_research_text(value: &str) -> String {
    value.chars().take(MAX_RESEARCH_TEXT_CHARS).collect()
}

#[cfg(test)]
fn json_pointer_segment(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
fn collect_scalar_fallback_evidence(
    value: &Value,
    path: &str,
    tool_call_id: &str,
    output: &mut Vec<Value>,
) {
    if output.len() >= MAX_FALLBACK_SCANNED_SCALARS_PER_TOOL {
        return;
    }
    match value {
        Value::Bool(_) | Value::Number(_) => output.push(serde_json::json!({
            "source_kind": "data_field",
            "tool_call_id": tool_call_id,
            "json_pointer": path,
            "value": value,
        })),
        Value::String(text) if !text.trim().is_empty() => {
            output.push(serde_json::json!({
                "source_kind": "data_field",
                "tool_call_id": tool_call_id,
                "json_pointer": path,
                "value": truncate_research_text(text.trim()),
            }));
        }
        Value::Array(values) => {
            for (index, item) in values.iter().enumerate() {
                let child = format!("{path}/{index}");
                collect_scalar_fallback_evidence(item, &child, tool_call_id, output);
                if output.len() >= MAX_FALLBACK_SCANNED_SCALARS_PER_TOOL {
                    break;
                }
            }
        }
        Value::Object(values) => {
            // DataFetch snapshots can contain large profile/news branches.
            // Traverse quote/time/valuation branches and important scalar
            // leaves first so the bounded scan cannot lose a quote merely
            // because the provider serialized `news` before `quote`.
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| fallback_object_key_priority(key));
            for (key, item) in entries {
                let child = format!("{path}/{}", json_pointer_segment(key));
                collect_scalar_fallback_evidence(item, &child, tool_call_id, output);
                if output.len() >= MAX_FALLBACK_SCANNED_SCALARS_PER_TOOL {
                    break;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
fn fallback_object_key_priority(key: &str) -> usize {
    match key {
        "quote" | "quotes" | "hone_quote_time" => 0,
        "symbol" => 1,
        "beijing" => 2,
        "price" => 3,
        "currency" => 4,
        "marketCap" | "pe" | "enterpriseValue" => 5,
        "revenue"
        | "ebitda"
        | "netIncome"
        | "operatingIncome"
        | "freeCashFlow"
        | "operatingCashFlow"
        | "cashAndCashEquivalents"
        | "totalDebt" => 6,
        "profile" | "metrics" | "keyMetrics" | "ratios" => 7,
        "date" | "period" | "calendarYear" | "reportedCurrency" => 8,
        "changesPercentage" | "change" => 9,
        "exchange" | "exchangeShortName" | "name" => 10,
        "financials" | "incomeStatement" | "balanceSheet" | "cashFlow" => 11,
        "news" | "articles" => 1_000,
        _ => 100,
    }
}

#[cfg(test)]
fn fallback_data_field_priority(item: &Value) -> usize {
    let pointer = item
        .get("json_pointer")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let leaf = pointer.rsplit('/').next().unwrap_or_default();
    match leaf {
        "symbol" => 0,
        "beijing" => 1,
        "price" => 2,
        "currency" => 3,
        "marketCap" => 4,
        "pe"
        | "enterpriseValue"
        | "enterpriseValueOverEBITDA"
        | "enterpriseValueOverRevenue"
        | "priceToSalesRatio"
        | "priceToBookRatio"
        | "priceEarningsRatio" => 5,
        "revenue" => 6,
        "ebitda" | "operatingIncome" => 7,
        "netIncome" | "freeCashFlow" | "operatingCashFlow" => 8,
        "cashAndCashEquivalents" | "cashAndShortTermInvestments" | "totalDebt" => 9,
        "date" | "period" | "calendarYear" | "reportedCurrency" => 10,
        "changesPercentage" => 11,
        "change" => 12,
        "exchange" | "exchangeShortName" => 13,
        "name" => 14,
        _ => 100,
    }
}

#[cfg(test)]
#[derive(Debug)]
struct FallbackEvidenceCandidate {
    priority: usize,
    evidence: Value,
}

#[cfg(test)]
fn fallback_data_candidate_priority(data_type: Option<&str>, item: &Value) -> usize {
    let pointer = item
        .get("json_pointer")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let base: usize = match data_type {
        Some(
            "quote" | "quote_short" | "crypto_quote" | "extended_hours" | "snapshot"
            | "earnings_outlook",
        ) => 5,
        Some("financials") | None => 20,
        Some("news") => 50,
        Some("profile") => 80,
        Some("search") => 1_000,
        Some(_) => 30,
    };
    let nested_news_penalty = usize::from(pointer.contains("/news/")) * 150;
    let period_penalty = if data_type == Some("financials") {
        pointer
            .strip_prefix("/data/")
            .and_then(|remainder| remainder.split('/').next())
            .and_then(|index| index.parse::<usize>().ok())
            .unwrap_or_default()
            .saturating_mul(20)
    } else {
        0
    };
    base.saturating_add(fallback_data_field_priority(item))
        .saturating_add(nested_news_penalty)
        .saturating_add(period_penalty)
}

#[cfg(test)]
fn current_turn_fallback_evidence_catalog_excluding(
    context: &AgentContext,
    turn_message_start: usize,
    scope: &FallbackEvidenceScope,
    covered_locators: &BTreeSet<String>,
) -> Vec<Value> {
    if scope.is_empty() {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for message in context.messages[turn_message_start.min(context.messages.len())..]
        .iter()
        .rev()
    {
        if message.role != "tool" {
            continue;
        }
        let (Some(tool_name), Some(tool_call_id), Some(content)) = (
            message.name.as_deref(),
            message.tool_call_id.as_deref(),
            message.content.as_deref(),
        ) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<Value>(content) else {
            continue;
        };
        if tool_result_is_failure(&parsed) {
            continue;
        }
        match tool_name {
            "web_search" => {
                let Some(result_numbers) = scope.web_result_numbers.get(tool_call_id) else {
                    continue;
                };
                let Some(results) = parsed.get("results").and_then(Value::as_array) else {
                    continue;
                };
                for result_number in result_numbers
                    .iter()
                    .copied()
                    .take(MAX_FALLBACK_ITEMS_PER_TOOL)
                {
                    let Some(index) = result_number.checked_sub(1) else {
                        continue;
                    };
                    let Some(result) = results.get(index) else {
                        continue;
                    };
                    let Some(url) = result
                        .get("url")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    else {
                        continue;
                    };
                    let Some(excerpt) = ["content", "snippet", "title"]
                        .into_iter()
                        .find_map(|field| result.get(field).and_then(Value::as_str))
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    else {
                        continue;
                    };
                    candidates.push(FallbackEvidenceCandidate {
                        priority: 0,
                        evidence: serde_json::json!({
                            "source_kind": "web_result",
                            "tool_call_id": tool_call_id,
                            "result_number": result_number,
                            "title": result.get("title").and_then(Value::as_str).map(truncate_research_text),
                            "url": url,
                            "exact_excerpt": truncate_research_text(excerpt),
                        }),
                    });
                }
            }
            "data_fetch" => {
                let Some(json_pointers) = scope.data_json_pointers.get(tool_call_id) else {
                    continue;
                };
                let Some(data_type) =
                    current_turn_data_fetch_type(context, turn_message_start, tool_call_id)
                else {
                    continue;
                };
                // Search results are an identity candidate list, not a selected
                // company fact. Letting a malformed handoff replay that list
                // can revive derivative/near matches such as CWY for CRWV.
                if data_type == "search" {
                    continue;
                }
                let mut tool_catalog = Vec::new();
                for json_pointer in json_pointers {
                    let Some(prefix) = fallback_data_value_prefix(&parsed, json_pointer) else {
                        continue;
                    };
                    let Some(value) = parsed.pointer(&prefix) else {
                        continue;
                    };
                    collect_scalar_fallback_evidence(
                        value,
                        &prefix,
                        tool_call_id,
                        &mut tool_catalog,
                    );
                    if tool_catalog.len() >= MAX_FALLBACK_SCANNED_SCALARS_PER_TOOL {
                        break;
                    }
                }
                // Scope selection happens before traversal, so unrelated
                // batched entities and older financial rows cannot consume the
                // scan/output bounds. Within that scope, retain quote identity,
                // price, time and valuation fields ahead of descriptive noise.
                tool_catalog.sort_by_key(fallback_data_field_priority);
                let mut seen_locators = BTreeSet::new();
                candidates.extend(
                    tool_catalog
                        .into_iter()
                        .filter(|evidence| {
                            evidence_locator(evidence)
                                .is_some_and(|locator| seen_locators.insert(locator))
                        })
                        .take(MAX_FALLBACK_ITEMS_PER_TOOL)
                        .map(|evidence| FallbackEvidenceCandidate {
                            priority: fallback_data_candidate_priority(
                                Some(data_type.as_str()),
                                &evidence,
                            ),
                            evidence,
                        }),
                );
            }
            _ => {}
        }
    }
    // Select across all successful tools only after every tool had a chance to
    // contribute. A large late financial/news payload therefore cannot evict
    // an earlier batched quote, and dual-ticker marketCap/PE fields survive the
    // same global bound as relationship excerpts and period-tagged financials.
    candidates.retain(|item| {
        evidence_locator(&item.evidence).is_none_or(|locator| !covered_locators.contains(&locator))
    });
    candidates.sort_by_key(|item| item.priority);
    candidates.truncate(MAX_FALLBACK_EVIDENCE_ITEMS);
    candidates.into_iter().map(|item| item.evidence).collect()
}

#[cfg(test)]
fn current_turn_fallback_evidence_catalog(
    context: &AgentContext,
    turn_message_start: usize,
    scope: &FallbackEvidenceScope,
) -> Vec<Value> {
    current_turn_fallback_evidence_catalog_excluding(
        context,
        turn_message_start,
        scope,
        &BTreeSet::new(),
    )
}

#[cfg(test)]
fn evidence_locator(item: &Value) -> Option<String> {
    let tool_call_id = item.get("tool_call_id")?.as_str()?;
    if let Some(pointer) = item.get("json_pointer").and_then(Value::as_str) {
        return Some(format!("data:{tool_call_id}:{pointer}"));
    }
    item.get("result_number")
        .and_then(Value::as_u64)
        .map(|result_number| format!("web:{tool_call_id}:{result_number}"))
}

#[cfg(test)]
fn fallback_research_handoff(
    context: &AgentContext,
    turn_message_start: usize,
    scope: &FallbackEvidenceScope,
    warning: impl Into<String>,
) -> ValidatedResearchHandoff {
    let fallback_evidence =
        current_turn_fallback_evidence_catalog(context, turn_message_start, scope);
    let unresolved_reference_count = usize::from(fallback_evidence.is_empty());
    let gaps = fallback_evidence.is_empty().then(|| {
        vec![
            "本轮工具结果未包含可机械提取的成功事实字段；仅披露具体缺项并回答仍可回答的部分。"
                .to_string(),
        ]
    });
    ValidatedResearchHandoff {
        answer_scope: "回答用户当前问题".to_string(),
        facts: Vec::new(),
        inferences: Vec::new(),
        gaps: gaps.unwrap_or_default(),
        fallback_evidence,
        validation_warnings: vec![warning.into()],
        unresolved_reference_count,
    }
}

#[cfg(test)]
fn validate_finish_research_handoff(
    handoff: ResearchHandoff,
    context: &AgentContext,
    turn_message_start: usize,
) -> ValidatedResearchHandoff {
    let fallback_scope = fallback_scope_from_handoff(&handoff);
    let mut warnings = Vec::new();
    let mut unresolved_reference_count = 0usize;
    let answer_scope = match bounded_research_text("answer_scope", &handoff.answer_scope, false) {
        Ok(value) => value,
        Err(warning) => {
            push_validation_warning(&mut warnings, warning);
            "回答用户当前问题".to_string()
        }
    };
    if handoff.facts.len() > MAX_RESEARCH_FACTS {
        push_validation_warning(
            &mut warnings,
            format!("facts exceeds {MAX_RESEARCH_FACTS} items; extra items were dropped"),
        );
    }
    if handoff.inferences.len() > MAX_RESEARCH_INFERENCES {
        push_validation_warning(
            &mut warnings,
            format!("inferences exceeds {MAX_RESEARCH_INFERENCES} items; extra items were dropped"),
        );
    }
    if handoff.gaps.len() > MAX_RESEARCH_GAPS {
        push_validation_warning(
            &mut warnings,
            format!("gaps exceeds {MAX_RESEARCH_GAPS} items; extra items were dropped"),
        );
    }

    let mut valid_fact_ids = BTreeSet::new();
    let mut facts = Vec::new();
    for (fact_index, fact) in handoff
        .facts
        .into_iter()
        .take(MAX_RESEARCH_FACTS)
        .enumerate()
    {
        let label = format!("facts[{fact_index}]");
        let id = match bounded_research_text(&format!("{label}.id"), &fact.id, false) {
            Ok(value) => value,
            Err(warning) => {
                push_validation_warning(&mut warnings, warning);
                continue;
            }
        };
        if valid_fact_ids.contains(&id) {
            push_validation_warning(&mut warnings, format!("duplicate fact id {id}"));
            continue;
        }
        if fact.evidence.is_empty() {
            push_validation_warning(&mut warnings, format!("{label}.evidence is empty"));
            continue;
        }
        if fact.evidence.len() > MAX_RESEARCH_REFS_PER_FACT {
            push_validation_warning(
                &mut warnings,
                format!(
                    "{label}.evidence exceeds {MAX_RESEARCH_REFS_PER_FACT} items; extra items were dropped"
                ),
            );
        }
        let mut resolved_evidence = Vec::new();
        for (reference_index, reference) in fact
            .evidence
            .into_iter()
            .take(MAX_RESEARCH_REFS_PER_FACT)
            .enumerate()
        {
            let tool_call_id = reference.tool_call_id;
            let resolved = match (
                reference.result_number,
                reference.exact_excerpt.as_deref(),
                reference.json_pointer.as_deref(),
            ) {
                (Some(result_number), Some(exact_excerpt), None) => resolve_web_evidence_ref(
                    context,
                    turn_message_start,
                    tool_call_id.trim(),
                    result_number,
                    exact_excerpt,
                ),
                (None, None, Some(json_pointer)) => {
                    current_turn_tool_result(context, turn_message_start, tool_call_id.trim())
                        .ok_or_else(|| {
                            format!(
                                "tool_call_id {} is not a current-turn result",
                                tool_call_id.trim()
                            )
                        })
                        .and_then(|(tool_name, content)| {
                            let data_type = current_turn_data_fetch_type(
                        context,
                        turn_message_start,
                        tool_call_id.trim(),
                    )
                    .ok_or_else(|| {
                        format!(
                            "tool_call_id {} has no matching current-turn DataFetch invocation",
                            tool_call_id.trim()
                        )
                    })?;
                            validate_data_evidence_ref(
                                tool_name,
                                content,
                                tool_call_id.trim(),
                                json_pointer,
                                &data_type,
                            )
                        })
                }
                _ => Err(
                    "evidence must provide either result_number + exact_excerpt or json_pointer"
                        .to_string(),
                ),
            };
            match resolved {
                Ok(value) => resolved_evidence.push(value),
                Err(warning) => {
                    unresolved_reference_count = unresolved_reference_count.saturating_add(1);
                    push_validation_warning(
                        &mut warnings,
                        format!("{label}.evidence[{reference_index}] ({tool_call_id}): {warning}"),
                    );
                }
            }
        }
        if !resolved_evidence.is_empty() {
            valid_fact_ids.insert(id.clone());
            facts.push(ValidatedResearchFact {
                id,
                resolved_evidence,
            });
        }
    }

    let mut inferences = Vec::new();
    for (index, inference) in handoff
        .inferences
        .into_iter()
        .take(MAX_RESEARCH_INFERENCES)
        .enumerate()
    {
        let claim = match bounded_research_text(
            &format!("inferences[{index}].claim"),
            &inference.claim,
            false,
        ) {
            Ok(value) => value,
            Err(warning) => {
                push_validation_warning(&mut warnings, warning);
                continue;
            }
        };
        if inference.premise_fact_ids.is_empty() {
            push_validation_warning(
                &mut warnings,
                format!("inferences[{index}].premise_fact_ids is empty"),
            );
            continue;
        }
        let missing = inference
            .premise_fact_ids
            .iter()
            .filter(|id| !valid_fact_ids.contains(id.trim()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            push_validation_warning(
                &mut warnings,
                format!(
                    "inferences[{index}] references unresolved fact IDs: {}",
                    missing.join(", ")
                ),
            );
            continue;
        }
        inferences.push(ResearchHandoffInference {
            claim,
            premise_fact_ids: inference.premise_fact_ids,
        });
    }

    let mut gaps = Vec::new();
    for (index, gap) in handoff.gaps.into_iter().take(MAX_RESEARCH_GAPS).enumerate() {
        match bounded_research_text(&format!("gaps[{index}]"), &gap, false) {
            Ok(value) => gaps.push(value),
            Err(warning) => push_validation_warning(&mut warnings, warning),
        }
    }

    // The handoff is a compact Agent-authored grouping, not an all-or-nothing
    // gate. Supplement only source locations the Agent explicitly selected,
    // including a selected object/result whose submitted leaf/excerpt was
    // malformed. Never scan every successful current-turn tool result: an
    // earlier wrong-entity call is still current-turn data but is not part of
    // the Agent's final evidence scope. De-duplicate provider-owned locators.
    let covered_locators = facts
        .iter()
        .flat_map(|fact| fact.resolved_evidence.iter())
        .filter_map(evidence_locator)
        .collect::<BTreeSet<_>>();
    let fallback_evidence = current_turn_fallback_evidence_catalog_excluding(
        context,
        turn_message_start,
        &fallback_scope,
        &covered_locators,
    );
    if facts.is_empty() && fallback_evidence.is_empty() && gaps.is_empty() {
        gaps.push(
            "本轮工具结果未包含可机械提取的成功事实字段；仅披露具体缺项并回答仍可回答的部分。"
                .to_string(),
        );
    }
    ValidatedResearchHandoff {
        answer_scope,
        facts,
        inferences,
        gaps,
        fallback_evidence,
        validation_warnings: warnings,
        unresolved_reference_count,
    }
}

fn is_identity_only_search_call(tool_call: &ToolCall) -> bool {
    data_fetch_data_type(tool_call).is_some_and(|data_type| data_type == "search")
}

fn data_fetch_data_type(tool_call: &ToolCall) -> Option<String> {
    let arguments = data_fetch_arguments(tool_call)?;
    Some(effective_data_fetch_data_type(&arguments).to_string())
}

fn data_fetch_arguments(tool_call: &ToolCall) -> Option<Value> {
    tool_call
        .function
        .name
        .eq("data_fetch")
        .then(|| serde_json::from_str::<Value>(&tool_call.function.arguments).ok())
        .flatten()
}

fn normalized_data_fetch_string_arg(tool_call: &ToolCall, keys: &[&str]) -> Option<String> {
    data_fetch_string_arg_raw(tool_call, keys).map(|value| value.to_lowercase())
}

fn data_fetch_string_arg_raw(tool_call: &ToolCall, keys: &[&str]) -> Option<String> {
    if tool_call.function.name != "data_fetch" {
        return None;
    }
    let arguments = serde_json::from_str::<Value>(&tool_call.function.arguments).ok()?;
    keys.iter()
        .find_map(|key| arguments.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn data_fetch_search_query(tool_call: &ToolCall) -> Option<String> {
    data_fetch_search_query_raw(tool_call)
}

fn data_fetch_refines_query(tool_call: &ToolCall) -> Option<String> {
    is_identity_only_search_call(tool_call)
        .then(|| data_fetch_string_arg_raw(tool_call, &["refines_query"]))
        .flatten()
}

fn data_fetch_supersedes_query(tool_call: &ToolCall) -> Option<String> {
    is_identity_only_search_call(tool_call)
        .then(|| data_fetch_string_arg_raw(tool_call, &["supersedes_query"]))
        .flatten()
}

fn data_fetch_identity_match_mode(tool_call: &ToolCall) -> Option<IdentitySearchMatchMode> {
    if !is_identity_only_search_call(tool_call) {
        return None;
    }
    match normalized_data_fetch_string_arg(tool_call, &["identity_match"])?.as_str() {
        "exact_symbol" => Some(IdentitySearchMatchMode::ExactSymbol),
        "name_or_alias" => Some(IdentitySearchMatchMode::NameOrAlias),
        _ => None,
    }
}

fn data_fetch_search_query_raw(tool_call: &ToolCall) -> Option<String> {
    if !is_identity_only_search_call(tool_call) {
        return None;
    }
    let arguments = data_fetch_arguments(tool_call)?;
    let target = effective_data_fetch_security_target(&arguments)?;
    validated_data_fetch_search_query(target).ok()
}

fn data_fetch_optional_metadata_string_is_valid(tool_call: &ToolCall, key: &str) -> bool {
    let Some(arguments) = data_fetch_arguments(tool_call) else {
        return false;
    };
    match arguments.get(key) {
        None => true,
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(_) => false,
    }
}

fn data_fetch_identity_search_shape_is_valid(tool_call: &ToolCall) -> bool {
    if !is_identity_only_search_call(tool_call) {
        return false;
    }
    if [
        "entity_route",
        "identity_match",
        "refines_query",
        "supersedes_query",
    ]
    .into_iter()
    .any(|key| !data_fetch_optional_metadata_string_is_valid(tool_call, key))
    {
        return false;
    }
    if data_fetch_refines_query(tool_call).is_some()
        && data_fetch_supersedes_query(tool_call).is_some()
    {
        return false;
    }
    let raw_match = data_fetch_string_arg_raw(tool_call, &["identity_match"]);
    let match_mode = data_fetch_identity_match_mode(tool_call);
    if raw_match.is_some() && match_mode.is_none() {
        return false;
    }
    if data_fetch_explicit_entity_route_key(tool_call).is_some() && match_mode.is_none() {
        return false;
    }
    let Some(query) = data_fetch_search_query_raw(tool_call) else {
        return false;
    };
    match match_mode {
        Some(IdentitySearchMatchMode::ExactSymbol) => provider_canonical_key(&query).is_some(),
        Some(IdentitySearchMatchMode::NameOrAlias) | None => {
            !normalized_identity_search_text(&query).is_empty()
        }
    }
}

fn data_fetch_identity_migration_source(tool_call: &ToolCall) -> Option<String> {
    data_fetch_supersedes_query(tool_call)
        .or_else(|| data_fetch_refines_query(tool_call))
        .or_else(|| data_fetch_search_query(tool_call))
}

fn data_fetch_explicit_entity_route_key(tool_call: &ToolCall) -> Option<String> {
    if tool_call.function.name != "data_fetch" {
        return None;
    }
    data_fetch_string_arg_raw(tool_call, &["entity_route"]).map(|route| format!("route:{route}"))
}

fn data_fetch_identity_route_key(tool_call: &ToolCall) -> Option<(String, bool)> {
    if !is_identity_only_search_call(tool_call) {
        return None;
    }
    if let Some(route_key) = data_fetch_explicit_entity_route_key(tool_call) {
        return Some((route_key, true));
    }
    // `refines_query` links a legacy/unscoped refinement to the original
    // query-derived route. Otherwise each separate Agent search call declares
    // one provisional route without parsing the natural-language query.
    data_fetch_refines_query(tool_call)
        .or_else(|| data_fetch_search_query(tool_call))
        .map(|query| (format!("query:{query}"), false))
}

fn normalized_identity_search_text(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

fn identity_search_route_candidates(
    tool_result: &Value,
    query: &str,
    match_mode: Option<IdentitySearchMatchMode>,
) -> BTreeSet<String> {
    let normalized_query = normalized_identity_search_text(query);
    let rows = tool_result
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let all_candidates = rows
        .iter()
        .filter_map(|row| row.get("symbol").and_then(Value::as_str))
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_ascii_uppercase)
        .collect::<BTreeSet<_>>();
    if all_candidates.is_empty() {
        return all_candidates;
    }

    if match_mode == Some(IdentitySearchMatchMode::ExactSymbol) {
        return rows
            .iter()
            .filter_map(|row| row.get("symbol").and_then(Value::as_str))
            .map(str::trim)
            .filter(|symbol| provider_symbols_equivalent(query, symbol))
            .map(str::to_ascii_uppercase)
            .collect();
    }
    if normalized_query.is_empty() {
        return BTreeSet::new();
    }

    let grounded = rows
        .iter()
        .filter(|row| {
            identity_search_row_is_grounded_in_query(
                row,
                query,
                match_mode != Some(IdentitySearchMatchMode::NameOrAlias),
            )
        })
        .filter_map(|row| row.get("symbol").and_then(Value::as_str))
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_ascii_uppercase)
        .collect::<BTreeSet<_>>();
    if grounded.is_empty() && match_mode != Some(IdentitySearchMatchMode::NameOrAlias) {
        all_candidates
    } else {
        // When at least one row directly matches the query, provider noise is
        // excluded from the route instead of becoming an alternate symbol.
        grounded
    }
}

fn identity_search_ascii_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn identity_search_name_starts_with_query(name: &str, query: &str) -> bool {
    if query.is_ascii() {
        let query_tokens = identity_search_ascii_tokens(query);
        let name_tokens = identity_search_ascii_tokens(name);
        !query_tokens.is_empty()
            && name_tokens.len() >= query_tokens.len()
            && name_tokens[..query_tokens.len()] == query_tokens
    } else {
        let normalized_query = normalized_identity_search_text(query);
        let normalized_name = normalized_identity_search_text(name);
        normalized_query.len() >= 2
            && (normalized_query == normalized_name
                || normalized_name.starts_with(&normalized_query))
    }
}

fn identity_search_row_is_grounded_in_query(
    row: &Value,
    query: &str,
    allow_exact_symbol: bool,
) -> bool {
    let exact_symbol_match = allow_exact_symbol
        && row
            .get("symbol")
            .and_then(Value::as_str)
            .is_some_and(|symbol| provider_symbols_equivalent(query, symbol));
    let name_match = ["name", "companyName"]
        .into_iter()
        .filter_map(|key| row.get(key).and_then(Value::as_str))
        .any(|name| identity_search_name_starts_with_query(name, query));
    exact_symbol_match || name_match
}

fn data_fetch_target_symbols(tool_call: &ToolCall) -> BTreeSet<String> {
    let Some(arguments) = data_fetch_arguments(tool_call) else {
        return BTreeSet::new();
    };
    effective_data_fetch_security_target(&arguments)
        .and_then(|target| validated_data_fetch_symbols(target).ok())
        .into_iter()
        .flatten()
        .map(|symbol| symbol.to_ascii_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect()
}

fn is_broad_data_type(data_type: &str) -> bool {
    matches!(
        data_type,
        "gainers_losers" | "sector_performance" | "earnings_calendar"
    )
}

fn explicit_data_fetch_data_type(arguments: &Value) -> Option<&str> {
    let data_type = arguments.get("data_type")?.as_str()?;
    (!data_type.is_empty() && data_type.trim() == data_type).then_some(data_type)
}

/// Validate the same coarse request shape that the DataFetch registry tool
/// will consume. This is deliberately structural: a provider no-coverage or
/// transport error remains a real research attempt, while an unsupported type,
/// malformed identity declaration, or unusable security target never opens the
/// finance protocol or crosses an already-visible prefix boundary.
fn data_fetch_call_is_structurally_valid(tool_call: &ToolCall) -> bool {
    let Some(arguments) = data_fetch_arguments(tool_call) else {
        return false;
    };
    let Some(data_type) = explicit_data_fetch_data_type(&arguments) else {
        return false;
    };

    match data_type {
        "search" => data_fetch_identity_search_shape_is_valid(tool_call),
        "news" => {
            let selected_target = arguments.get("ticker").or_else(|| arguments.get("symbol"));
            match selected_target {
                None => true,
                Some(Value::String(target)) if target.trim().is_empty() => true,
                Some(Value::String(_)) => effective_data_fetch_security_target(&arguments)
                    .is_some_and(|target| validated_data_fetch_symbols(target).is_ok()),
                Some(_) => false,
            }
        }
        "gainers_losers" | "sector_performance" | "earnings_calendar" => true,
        data_type if data_fetch_data_type_uses_security_target(data_type) => {
            effective_data_fetch_security_target(&arguments)
                .is_some_and(|target| validated_data_fetch_symbols(target).is_ok())
        }
        _ => false,
    }
}

fn starts_investment_research_protocol(tool_call: &ToolCall) -> bool {
    if !data_fetch_call_is_structurally_valid(tool_call) {
        return false;
    }
    let Some(arguments) = data_fetch_arguments(tool_call) else {
        return false;
    };
    let Some(data_type) = explicit_data_fetch_data_type(&arguments) else {
        return false;
    };
    // An unscoped news feed is a valid DataFetch request but does not prove
    // that an ordinary Interactive turn is security research.
    data_type != "news" || effective_data_fetch_security_target(&arguments).is_some()
}

fn registered_read_only_tool_call_is_well_formed(
    tool_call: &ToolCall,
    registered_tool_names: &BTreeSet<String>,
) -> bool {
    if !registered_tool_names.contains(&tool_call.function.name) {
        return false;
    }
    serde_json::from_str::<Value>(&tool_call.function.arguments).is_ok_and(|arguments| {
        tool_call_is_known_read_only(&tool_call.function.name, &arguments)
            && (tool_call.function.name != "data_fetch"
                || data_fetch_call_is_structurally_valid(tool_call))
    })
}

fn registered_tool_call_is_known_read_only(
    tool_call: &ToolCall,
    registered_tool_names: &BTreeSet<String>,
) -> bool {
    registered_tool_names.contains(&tool_call.function.name)
        && serde_json::from_str::<Value>(&tool_call.function.arguments).is_ok_and(|arguments| {
            tool_call_is_known_read_only(&tool_call.function.name, &arguments)
        })
}

fn malformed_read_only_tool_result(tool_call: &ToolCall) -> Value {
    let arguments = serde_json::from_str::<Value>(&tool_call.function.arguments).ok();
    let data_type = arguments
        .as_ref()
        .and_then(|arguments| explicit_data_fetch_data_type(arguments));
    let message = if tool_call.function.name == "data_fetch"
        && data_type.is_some_and(data_fetch_data_type_uses_security_target)
    {
        format!(
            "DataFetch {} 必须在实体 search 返回标准代码后，使用 ticker 或 symbol 携带该代码重新调用；query 只用于 data_type=search，不能替代证券目标字段",
            data_type.unwrap_or("证券级接口")
        )
    } else if tool_call.function.name == "data_fetch" {
        "DataFetch 参数结构无效；请依据工具 schema 修正本次只读调用后重试".to_string()
    } else {
        format!(
            "只读工具 {} 的参数结构无效；请依据工具 schema 修正后重试",
            tool_call.function.name
        )
    };
    serde_json::json!({
        "error": "invalid_read_only_tool_shape",
        "status": "rejected_before_execution",
        "isError": true,
        "tool": tool_call.function.name,
        "data_type": data_type,
        "message": message,
    })
}

fn committed_prefix_matches_required(committed: &str, required: Option<&str>) -> bool {
    required.is_some_and(|required| committed == required || committed.starts_with(required))
}

fn original_market_move_request(runtime_input: &str) -> &str {
    let current_turn_and_context = [
        "\n\n【本轮证券实体发现：主 Agent 工具循环】",
        "\n\n【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】",
        "\n\n【本轮最终回答契约：由主 Agent 一次完成】",
    ]
    .into_iter()
    .filter_map(|marker| runtime_input.find(marker))
    .min()
    .map(|end| runtime_input[..end].trim())
    .unwrap_or_else(|| runtime_input.trim());
    current_turn_and_context
        .rfind("【本轮用户输入】")
        .map(|start| current_turn_and_context[start + "【本轮用户输入】".len()..].trim())
        .unwrap_or(current_turn_and_context)
}

fn is_market_move_final_check_enabled(runtime_input: &str) -> bool {
    if !runtime_input
        .contains("【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】")
    {
        return false;
    }
    let normalized = original_market_move_request(runtime_input).to_ascii_lowercase();
    [
        "大跌",
        "暴跌",
        "大涨",
        "暴涨",
        "下跌原因",
        "上涨原因",
        "跌的原因",
        "涨的原因",
        "为什么跌",
        "为什么涨",
        "why did",
        "why is",
        "selloff",
        "rally",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn market_move_starts_investment_research(
    runtime_input: &str,
    agent_owned_finance_loop: bool,
) -> bool {
    agent_owned_finance_loop && is_market_move_final_check_enabled(runtime_input)
}

fn chinese_weekday_label(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "周一",
        Weekday::Tue => "周二",
        Weekday::Wed => "周三",
        Weekday::Thu => "周四",
        Weekday::Fri => "周五",
        Weekday::Sat => "周六",
        Weekday::Sun => "周日",
    }
}

fn parse_chinese_weekday(value: &str) -> Option<Weekday> {
    match value {
        "周一" => Some(Weekday::Mon),
        "周二" => Some(Weekday::Tue),
        "周三" => Some(Weekday::Wed),
        "周四" => Some(Weekday::Thu),
        "周五" => Some(Weekday::Fri),
        "周六" => Some(Weekday::Sat),
        "周日" | "周天" => Some(Weekday::Sun),
        _ => None,
    }
}

fn anchored_market_move_date(runtime_input: &str) -> Option<NaiveDate> {
    let re =
        Regex::new(r"最近同名候选日期是\s+([0-9]{4}-[0-9]{2}-[0-9]{2})\s+周[一二三四五六日天]")
            .expect("market-move candidate date regex");
    re.captures(runtime_input)
        .and_then(|captures| captures.get(1))
        .and_then(|date| NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").ok())
}

fn first_absolute_market_date(content: &str) -> Option<NaiveDate> {
    let re = Regex::new(r"([0-9]{4})(?:年|-|/)([0-9]{1,2})(?:月|-|/)([0-9]{1,2})日?")
        .expect("absolute market date regex");
    re.captures_iter(content)
        .find_map(|captures| date_from_capture(&captures, 1, 2, 3))
}

fn date_from_capture(
    captures: &regex::Captures<'_>,
    year: usize,
    month: usize,
    day: usize,
) -> Option<NaiveDate> {
    let year = captures.get(year)?.as_str().parse::<i32>().ok()?;
    let month = captures.get(month)?.as_str().parse::<u32>().ok()?;
    let day = captures.get(day)?.as_str().parse::<u32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn first_body_market_date(content: &str) -> Option<NaiveDate> {
    let body = content.split_once('\n').map(|(_, body)| body).unwrap_or("");
    first_absolute_market_date(body)
}

fn content_contains_date(content: &str, date: NaiveDate) -> bool {
    let normalized = content.to_ascii_lowercase();
    let month_name = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ][date.month0() as usize];
    [
        date.format("%Y-%m-%d").to_string(),
        date.format("%Y/%m/%d").to_string(),
        date.format("%m/%d/%Y").to_string(),
        format!("{}年{}月{}日", date.year(), date.month(), date.day()),
        format!("{month_name} {}, {}", date.day(), date.year()),
        format!("{month_name} {} {}", date.day(), date.year()),
        format!("{} {month_name} {}", date.day(), date.year()),
    ]
    .iter()
    .any(|candidate| normalized.contains(candidate))
}

#[derive(Debug, Clone, PartialEq)]
struct MarketMoveQuoteEvidence {
    symbol: String,
    change_percentage: f64,
    market_date: NaiveDate,
    beijing_time: Option<String>,
    exchange: Option<String>,
}

fn collect_market_move_quote_evidence(value: &Value, evidence: &mut Vec<MarketMoveQuoteEvidence>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_market_move_quote_evidence(item, evidence);
            }
        }
        Value::Object(fields) => {
            let quote_time = fields.get("hone_quote_time").and_then(Value::as_object);
            let quote = fields
                .get("symbol")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|symbol| !symbol.is_empty())
                .zip(
                    fields
                        .get("hone_change_basis")
                        .and_then(Value::as_object)
                        .and_then(|basis| basis.get("pct"))
                        .and_then(|value| {
                            value
                                .as_f64()
                                .or_else(|| value.as_str()?.parse::<f64>().ok())
                        }),
                )
                .zip(
                    quote_time
                        .and_then(|time| time.get("market_date_new_york"))
                        .and_then(Value::as_str)
                        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()),
                );
            if let Some(((symbol, change_percentage), market_date)) = quote {
                evidence.push(MarketMoveQuoteEvidence {
                    symbol: symbol.to_ascii_uppercase(),
                    change_percentage,
                    market_date,
                    beijing_time: quote_time
                        .and_then(|time| time.get("beijing"))
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    exchange: fields
                        .get("exchange")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|exchange| !exchange.is_empty())
                        .map(str::to_ascii_uppercase),
                });
            }
            for value in fields.values() {
                collect_market_move_quote_evidence(value, evidence);
            }
        }
        _ => {}
    }
}

fn current_turn_market_move_quotes(
    context: &AgentContext,
    turn_message_start: usize,
) -> Vec<MarketMoveQuoteEvidence> {
    let mut by_symbol_and_date = BTreeMap::new();
    for message in &context.messages[turn_message_start.min(context.messages.len())..] {
        if message.role != "tool" || message.name.as_deref() != Some("data_fetch") {
            continue;
        }
        let Some(content) = message.content.as_deref() else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(content) else {
            continue;
        };
        let mut evidence = Vec::new();
        collect_market_move_quote_evidence(&value, &mut evidence);
        for quote in evidence {
            by_symbol_and_date.insert((quote.symbol.clone(), quote.market_date), quote);
        }
    }
    let mut evidence = by_symbol_and_date.into_values().collect::<Vec<_>>();
    evidence.sort_by_key(|quote| {
        let rank = match quote.symbol.as_str() {
            "SPY" => 0,
            "QQQ" => 1,
            "DIA" => 2,
            "IWM" => 3,
            _ => 4,
        };
        (rank, quote.symbol.clone(), quote.market_date)
    });
    evidence
}

fn current_turn_verified_broad_market_quote_groups(
    context: &AgentContext,
    turn_message_start: usize,
) -> BTreeSet<String> {
    current_turn_market_move_quotes(context, turn_message_start)
        .into_iter()
        .filter_map(|quote| broad_us_market_symbol_group(&quote.symbol).map(str::to_string))
        .collect()
}

fn current_turn_market_move_quote_date(
    context: &AgentContext,
    turn_message_start: usize,
) -> Option<NaiveDate> {
    let mut counts = BTreeMap::<NaiveDate, usize>::new();
    for quote in current_turn_market_move_quotes(context, turn_message_start) {
        *counts.entry(quote.market_date).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|(date_a, count_a), (date_b, count_b)| {
            count_a.cmp(count_b).then_with(|| date_a.cmp(date_b))
        })
        .map(|(date, _)| date)
}

fn resolved_market_move_date(
    runtime_input: &str,
    content: &str,
    context: &AgentContext,
    turn_message_start: usize,
) -> Option<NaiveDate> {
    first_absolute_market_date(original_market_move_request(runtime_input))
        .or_else(|| anchored_market_move_date(runtime_input))
        .or_else(|| current_turn_market_move_quote_date(context, turn_message_start))
        .or_else(|| first_body_market_date(content))
}

fn append_date_weekday_violations(content: &str, violations: &mut Vec<String>) {
    let re = Regex::new(
        r"([0-9]{4})(?:年|-|/)([0-9]{1,2})(?:月|-|/)([0-9]{1,2})日?\s*[（(]?(周[一二三四五六日天])",
    )
    .expect("date weekday pair regex");
    for captures in re.captures_iter(content) {
        let Some(date) = date_from_capture(&captures, 1, 2, 3) else {
            violations.push("终稿包含无效的绝对日期".to_string());
            continue;
        };
        let Some(label) = captures.get(4).map(|value| value.as_str()) else {
            continue;
        };
        let Some(weekday) = parse_chinese_weekday(label) else {
            continue;
        };
        if date.weekday() != weekday {
            violations.push(format!(
                "{} 的星期应为{}，不能写成{}",
                date.format("%Y-%m-%d"),
                chinese_weekday_label(date.weekday()),
                label
            ));
        }
    }
}

fn append_declared_target_date_violations(
    content: &str,
    target_date: NaiveDate,
    violations: &mut Vec<String>,
) {
    for paragraph in content.split("\n\n").skip(1) {
        if !["目标时段", "目标日期", "解释日期", "本轮解释"]
            .iter()
            .any(|marker| paragraph.contains(marker))
        {
            continue;
        }
        let Some(declared_date) = first_absolute_market_date(paragraph) else {
            continue;
        };
        if declared_date != target_date {
            violations.push(format!(
                "终稿声明的目标日期 {} 与本轮证据锚定日期 {} 不一致",
                declared_date.format("%Y-%m-%d"),
                target_date.format("%Y-%m-%d")
            ));
        }
    }
}

fn collect_tool_result_urls(value: &Value, urls: &mut BTreeMap<String, String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_tool_result_urls(item, urls);
            }
        }
        Value::Object(fields) => {
            for url in fields.iter().filter_map(|(key, value)| {
                matches!(
                    key.to_ascii_lowercase().as_str(),
                    "url" | "link" | "source_url" | "original_url"
                )
                .then(|| value.as_str())
                .flatten()
                .map(str::trim)
                .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
            }) {
                urls.insert(url.to_string(), value.to_string());
            }
            for value in fields.values() {
                collect_tool_result_urls(value, urls);
            }
        }
        _ => {}
    }
}

fn current_turn_tool_result_urls(
    context: &AgentContext,
    turn_message_start: usize,
) -> BTreeMap<String, String> {
    let mut urls = BTreeMap::new();
    for message in &context.messages[turn_message_start.min(context.messages.len())..] {
        if message.role != "tool" {
            continue;
        }
        let Some(content) = message.content.as_deref() else {
            continue;
        };
        if let Ok(value) = serde_json::from_str::<Value>(content) {
            let snippet_only = message.name.as_deref() == Some("web_search")
                && (value
                    .pointer("/hone_search_contract/evidence_scope/full_page_content")
                    .and_then(Value::as_bool)
                    == Some(false)
                    || value
                        .pointer("/hone_search_contract/evidence_scope/kind")
                        .and_then(Value::as_str)
                        == Some("search_snippets"));
            if snippet_only {
                continue;
            }
            collect_tool_result_urls(&value, &mut urls);
        }
    }
    urls
}

fn current_turn_source_urls_for_date(
    context: &AgentContext,
    turn_message_start: usize,
    date: NaiveDate,
) -> Vec<String> {
    current_turn_tool_result_urls(context, turn_message_start)
        .into_iter()
        .filter_map(|(url, evidence)| content_contains_date(&evidence, date).then_some(url))
        .collect()
}

fn paragraph_has_definitive_market_cause(paragraph: &str) -> bool {
    let normalized = paragraph.to_ascii_lowercase();
    if [
        "原因本轮未完全核验",
        "原因尚未核验",
        "无法确认",
        "未能确认",
        "不能确认",
        "证据不足",
        "候选原因",
        "如果",
        "可能",
        "或许",
        "预计",
        "假设",
        "情景",
        "bull",
        "bear",
        "base case",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
        || normalized.contains("推断：")
    {
        return false;
    }
    [
        "已核验原因",
        "原因（已核验",
        "原因(已核验",
        "主要原因",
        "直接原因",
        "下跌原因",
        "上涨原因",
        "已核验新闻",
        "原因是",
        "原因包括",
        "核心驱动",
        "主要驱动",
        "最强驱动",
        "公开新闻指向",
        "公开归因",
        "导致",
        "直接导致",
        "直接压制",
        "拖累",
        "打压",
        "归因于",
        "源于",
        "触发",
        "引发",
        "催化剂",
        "共振",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn append_market_move_quote_fact_violations(
    content: &str,
    context: &AgentContext,
    turn_message_start: usize,
    violations: &mut Vec<String>,
) {
    let quotes = current_turn_market_move_quotes(context, turn_message_start);
    if quotes.is_empty() {
        return;
    }

    // What this has to catch is a draft presenting the plain quote price as a
    // closing price. It used to fire on the bare words 收盘 / 收市 anywhere in
    // the answer, which also hit the comparison the change-basis rules require
    // ("较上一常规交易日收盘价下跌 6.22%"), and on `收` + digit, which hit
    // 营收 6.5 亿 / 回收 3 亿. Require a close word attached to a price-shaped
    // number — a percentage after it means it is a change, not a price.
    let close_value_claim = Regex::new(
        r"(?i)(?:收报|收于|收在|收盘价?\s*(?:为|是|报)?|closing price(?:\s+(?:of|was|is))?|closed at|close price(?:\s+(?:of|was|is))?)\s*[:：]?\s*[$¥€£]?\s*[0-9]+(?:[.,][0-9]+)?\s*(?:%|％)?",
    )
    .expect("market-move close value regex");
    if close_value_claim
        .find_iter(content)
        .any(|hit| !hit.as_str().ends_with('%') && !hit.as_str().ends_with('％'))
    {
        violations.push(
            "普通 quote 的时间字段不证明交易时段或收盘，不能把其 price 写成收盘价".to_string(),
        );
    }

    for quote in &quotes {
        let escaped_symbol = regex::escape(&quote.symbol);
        let symbol_pattern = format!(r"(?i)(?:^|[^A-Z0-9]){escaped_symbol}(?:[^A-Z0-9]|$)");
        let Ok(symbol_regex) = Regex::new(&symbol_pattern) else {
            continue;
        };
        let percentage_pattern = format!(
            r"(?i)(?:^|[^A-Z0-9]){escaped_symbol}(?:[^A-Z0-9]|$)[^%A-Z\n]{{0,80}}?([+-]?[0-9]+(?:\.[0-9]+)?)\s*%"
        );
        if let Ok(percentage_regex) = Regex::new(&percentage_pattern) {
            for line in content.lines() {
                let verified_symbols_on_line = quotes
                    .iter()
                    .filter(|candidate| {
                        Regex::new(&format!(
                            r"(?i)(?:^|[^A-Z0-9]){}(?:[^A-Z0-9]|$)",
                            regex::escape(&candidate.symbol)
                        ))
                        .is_ok_and(|candidate_regex| candidate_regex.is_match(line))
                    })
                    .count();
                if line.contains("分别") && verified_symbols_on_line > 1 {
                    continue;
                }
                for captures in percentage_regex.captures_iter(line) {
                    let Some(displayed) = captures
                        .get(1)
                        .and_then(|value| value.as_str().parse::<f64>().ok())
                    else {
                        continue;
                    };
                    // A move-attribution answer is supposed to quantify the
                    // event (dilution, margin, share, implied upside). Those
                    // percentages sit on the same line as the ticker but are
                    // not change claims, so only reconcile a percentage whose
                    // lead-in does not name a different metric.
                    let lead_in = captures
                        .get(0)
                        .zip(captures.get(1))
                        .map(|(whole, number)| &line[whole.start()..number.start().min(line.len())])
                        .unwrap_or("");
                    if percentage_names_another_metric(lead_in) {
                        continue;
                    }
                    if (displayed - quote.change_percentage).abs() > 0.015 {
                        violations.push(format!(
                            "{} 的涨跌幅应来自服务端 hone_change_basis.pct（约 {:+.2}%），不能使用 provider changesPercentage 或把 change 等其它字段写成百分比",
                            quote.symbol, quote.change_percentage
                        ));
                    }
                }
            }
        }

        let Some(exchange) = quote.exchange.as_deref() else {
            continue;
        };
        for line in content.lines().filter(|line| symbol_regex.is_match(line)) {
            let normalized = line.to_ascii_lowercase();
            let wrong_exchange = (normalized.contains("纽交所") || normalized.contains("nyse"))
                && !exchange.contains("NYSE")
                || (normalized.contains("纳斯达克") || normalized.contains("nasdaq"))
                    && !exchange.contains("NASDAQ")
                || (normalized.contains("美交所") || normalized.contains("amex"))
                    && !exchange.contains("AMEX");
            if wrong_exchange {
                violations.push(format!(
                    "{} 的交易所声明与本轮 quote 的 exchange={} 不一致",
                    quote.symbol, exchange
                ));
            }
        }
    }
}

fn broad_us_market_scope_coverage(content: &str) -> usize {
    let normalized = content.to_ascii_lowercase();
    [
        ["标普", "s&p", "spy"],
        ["纳斯达克", "nasdaq", "qqq"],
        ["道琼斯", "dow jones", "dia"],
        ["罗素", "russell", "iwm"],
    ]
    .iter()
    .filter(|markers| markers.iter().any(|marker| normalized.contains(marker)))
    .count()
}

fn broad_us_market_symbol_group(symbol: &str) -> Option<&'static str> {
    match symbol.trim().to_ascii_uppercase().as_str() {
        "SPY" | "SPX" | "^SPX" | "GSPC" | "^GSPC" => Some("标普500"),
        "QQQ" | "NDX" | "^NDX" | "IXIC" | "^IXIC" | "COMP" | "^COMP" => Some("纳斯达克"),
        "DIA" | "DJI" | "^DJI" => Some("道琼斯"),
        "IWM" | "RUT" | "^RUT" => Some("罗素2000"),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct ListingEvidenceState {
    active: bool,
    inactive: bool,
    current_quote: bool,
    active_profile: bool,
    /// Provider company names for this symbol. A user asking about
    /// `coreweave` reads an answer written about "CoreWeave", not "CRWV", so
    /// symbol-only matching leaves the most natural denial phrasing unguarded.
    names: BTreeSet<String>,
}

/// Every string a denial line could use to name this security: the symbol and
/// the provider company name, plus the segment before the first comma so
/// `CoreWeave, Inc.` also matches prose that writes just `CoreWeave`. All of it
/// comes from the provider payload; nothing here is a maintained word list.
fn listing_entity_labels(symbol: &str, state: &ListingEvidenceState) -> Vec<String> {
    let mut labels = vec![symbol.to_ascii_uppercase()];
    for name in &state.names {
        let upper = name.to_ascii_uppercase();
        let head = upper
            .split(',')
            .next()
            .unwrap_or(&upper)
            .trim()
            .trim_end_matches('.')
            .to_string();
        for label in [upper, head] {
            if label.len() >= 3 && !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    labels
}

fn canonical_listing_symbol(value: &str) -> Option<String> {
    provider_canonical_key(value).or_else(|| {
        let normalized = value.trim().to_ascii_uppercase();
        (!normalized.is_empty()).then_some(normalized)
    })
}

fn collect_listing_markers(value: &Value, states: &mut BTreeMap<String, ListingEvidenceState>) {
    match value {
        Value::Object(fields) => {
            if let Some(marker) = fields.get("hone_security_listing_evidence")
                && let Some(marker_fields) = marker.as_object()
            {
                let symbol = ["requested_symbol", "quote_symbol", "profile_symbol"]
                    .into_iter()
                    .find_map(|key| marker_fields.get(key).and_then(Value::as_str))
                    .and_then(canonical_listing_symbol);
                if let Some(symbol) = symbol {
                    let state = states.entry(symbol).or_default();
                    match marker_fields.get("status").and_then(Value::as_str) {
                        Some("active_listing") => state.active = true,
                        Some("inactive_listing") => state.inactive = true,
                        _ => {}
                    }
                    if let Some(company_name) = marker_fields
                        .get("company_name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                    {
                        state.names.insert(company_name.to_string());
                    }
                }
            }
            for child in fields.values() {
                collect_listing_markers(child, states);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_listing_markers(item, states);
            }
        }
        _ => {}
    }
}

fn current_turn_listing_states(
    tool_calls_made: &[ToolCallMade],
) -> BTreeMap<String, ListingEvidenceState> {
    let mut states = BTreeMap::new();
    for call in tool_calls_made {
        collect_listing_markers(&call.result, &mut states);
        if call.name != "data_fetch" {
            continue;
        }
        let Some(data_type) = call.arguments.get("data_type").and_then(Value::as_str) else {
            continue;
        };
        if !matches!(data_type, "quote" | "profile") {
            continue;
        }
        let Some(data) = call.result.get("data") else {
            continue;
        };
        let rows = data
            .as_array()
            .map(|items| items.iter().collect::<Vec<_>>())
            .unwrap_or_else(|| vec![data]);
        for row in rows {
            let Some(fields) = row.as_object() else {
                continue;
            };
            let Some(symbol) = fields
                .get("symbol")
                .and_then(Value::as_str)
                .and_then(canonical_listing_symbol)
            else {
                continue;
            };
            let state = states.entry(symbol).or_default();
            if let Some(company_name) = fields
                .get("companyName")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
            {
                state.names.insert(company_name.to_string());
            }
            if data_type == "quote" {
                state.current_quote = fields
                    .get("price")
                    .and_then(Value::as_f64)
                    .is_some_and(|price| price.is_finite() && price > 0.0);
            } else {
                let active = fields.get("isActivelyTrading").and_then(Value::as_bool) == Some(true);
                let exchange_present = fields
                    .get("exchangeShortName")
                    .or_else(|| fields.get("exchange"))
                    .and_then(Value::as_str)
                    .is_some_and(|exchange| !exchange.trim().is_empty());
                state.active_profile = active && exchange_present;
            }
        }
    }
    for state in states.values_mut() {
        state.active |= state.current_quote && state.active_profile;
    }
    states
}

fn explicit_security_symbols(runtime_input: &str) -> BTreeSet<String> {
    let request = original_market_move_request(runtime_input);
    let token =
        Regex::new(r"\$?[A-Za-z][A-Za-z0-9./-]*").expect("explicit security candidate regex");
    let code_shape =
        Regex::new(r"^[A-Za-z]{1,5}(?:[./-][A-Za-z])?$").expect("security code shape regex");
    token
        .find_iter(request)
        .filter_map(|matched| {
            let raw = matched.as_str();
            let value = raw.strip_prefix('$').unwrap_or(raw);
            if !code_shape.is_match(value) {
                return None;
            }
            let uppercase = value.chars().any(|ch| ch.is_ascii_uppercase())
                && value
                    .chars()
                    .filter(|ch| ch.is_ascii_alphabetic())
                    .all(|ch| ch.is_ascii_uppercase());
            let adjacent_non_ascii = request[..matched.start()]
                .chars()
                .next_back()
                .is_some_and(|ch| !ch.is_ascii())
                || request[matched.end()..]
                    .chars()
                    .next()
                    .is_some_and(|ch| !ch.is_ascii());
            (raw.starts_with('$') || uppercase || adjacent_non_ascii)
                .then(|| canonical_listing_symbol(value))
                .flatten()
        })
        .collect()
}

fn listing_denial_line(line: &str) -> bool {
    let normalized = line.to_ascii_lowercase();
    let denial = [
        "已退市",
        "退市",
        "未上市",
        "没有上市",
        "并未上市",
        "不再上市",
        "不是当前交易代码",
        "is delisted",
        "delisted",
        "was delisted",
        "has been delisted",
        "not listed",
        "no longer listed",
        "not publicly traded",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    let rebuttal = [
        "并非已退市",
        "不是已退市",
        "没有退市",
        "尚未退市",
        "不能断言",
        "不得断言",
        "不能用历史",
        "不得用历史",
        "错误声称",
        "错误地声称",
        "退市风险",
        "是否退市",
        "会不会退市",
        "可能退市",
        "重新上市",
        "恢复上市",
        "当前上市",
        "目前上市",
        "not delisted",
        "incorrectly",
        "delisting risk",
        "could be delisted",
        "may be delisted",
        "relisted",
        "currently listed",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    denial && !rebuttal
}

fn listing_final_violations(
    runtime_input: &str,
    content: &str,
    tool_calls_made: &[ToolCallMade],
) -> Vec<String> {
    let states = current_turn_listing_states(tool_calls_made);
    let mut symbols = explicit_security_symbols(runtime_input);
    symbols.extend(states.keys().cloned());
    let mut violations = BTreeSet::new();
    for line in content.split(['\n', '。', '！', '？']) {
        if !listing_denial_line(line) {
            continue;
        }
        let upper = line.to_ascii_uppercase();
        for symbol in &symbols {
            let state = states.get(symbol);
            // A denial may name the security by symbol or by provider company
            // name; `CoreWeave 尚未上市` and `CRWV 尚未上市` are the same claim.
            let named = state
                .map(|state| listing_entity_labels(symbol, state))
                .unwrap_or_else(|| vec![symbol.to_ascii_uppercase()])
                .into_iter()
                .any(|label| upper.contains(&label));
            if !named {
                continue;
            }
            if state.is_some_and(|state| state.inactive && !state.active) {
                continue;
            }
            let reason = if state.is_some_and(|state| state.active) {
                "本轮结构化证据已确认 active_listing，终稿却断言退市或未上市"
            } else {
                "本轮没有 inactive_listing 结构化证据，终稿却用历史记忆断言退市或未上市"
            };
            violations.insert(format!("{symbol}: {reason}"));
        }
    }
    violations.into_iter().collect()
}

/// The service already requires an investment answer to open with the exact
/// `数据时间：北京时间 …；行情口径：` line, and the same contract tells the Agent to
/// skip that format entirely when the turn is not an investment request. So the
/// header is the Agent's own declaration of what it is about to publish — no
/// server-side classifier and no keyword list is involved in reading it.
///
/// A turn carrying that declaration while having called nothing has answered a
/// research question from model memory. That is the whole defect class behind
/// `SNDK 未上市`, `coreweave 还没 IPO`, `无法提供具体的数字`, and any fabricated
/// price or multiple: not a vocabulary to enumerate, one precondition to hold.
fn research_answer_without_evidence(
    content: &str,
    tool_calls_made: &[ToolCallMade],
    preloaded_evidence_calls: u32,
) -> bool {
    // Service-preloaded evidence sits in the turn input rather than in
    // `tool_calls_made`, so a turn that answered straight from a rich
    // preloaded context is evidence-backed, not evidence-free.
    if preloaded_evidence_calls > 0 || !tool_calls_made.is_empty() {
        return false;
    }
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .is_some_and(|line| line.starts_with(CANONICAL_RESEARCH_ANSWER_HEADER))
}

fn research_evidence_precondition_prompt() -> String {
    format!(
        "【内部取证前置】上一版终稿尚未发布正文。它以 `{CANONICAL_RESEARCH_ANSWER_HEADER}` 开头，即由你自己声明这是一次投研回答，但本轮没有执行过任何业务工具，因此其中的上市状态、价格、估值、财务、事件与结论都只来自模型记忆，不是本轮核验结果。本轮请先取证再作答：按用户原问题的真实主题，用 `data_fetch` 确认实体与同代码行情/资料，并用 `web_search` 取当前公开来源；两者可在同一轮并行。取得真实结果后再按用户原问题重写终稿。若某项确实查不到，只写该项未核验，不得把缺失写成事实。不要向用户提及本检查。"
    )
}

fn listing_final_correction_prompt(
    violations: &[String],
    draft: &str,
    tool_calls_made: &[ToolCallMade],
) -> String {
    let states = current_turn_listing_states(tool_calls_made);
    let active = states
        .iter()
        .filter(|(_, state)| state.active)
        .map(|(symbol, _)| symbol.as_str())
        .collect::<Vec<_>>();
    let active = if active.is_empty() {
        "无；必须继续取得同代码 quote/profile 或 earnings_outlook 后再判断上市状态".to_string()
    } else {
        active.join("、")
    };
    let bounded_draft = draft
        .chars()
        .take(MAX_LISTING_DRAFT_FEEDBACK_CHARS)
        .collect::<String>();
    format!(
        "【内部终稿上市状态纠正】上一版终稿尚未发布正文。逐项修正：{}。本轮 active_listing 标的：{active}。不得用历史收购或旧退市记忆断言当前未上市，也不得引导用户改问旧母公司；已有 active_listing 时直接按当前上市证券完成原问题。尚无当前状态证据时，优先继续调用同代码 quote/profile、snapshot 或 earnings_outlook；若工具已关闭，只能披露当前状态未核验，绝不能把证据缺失写成退市事实。不要向用户提及本检查。重写稿仍须遵守精确数据时间首行。\n<unpublished_draft>\n{bounded_draft}\n</unpublished_draft>",
        violations.join("；")
    )
}

fn deterministic_listing_gap_response(
    required_prefix: Option<&str>,
    tool_calls_made: &[ToolCallMade],
    violations: &[String],
) -> String {
    let first_line = match required_prefix {
        Some(prefix) if prefix.ends_with("；行情口径：") => {
            format!("{prefix}本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露")
        }
        Some(prefix) => prefix.to_string(),
        None => "本轮仅使用可核验资料。".to_string(),
    };
    let states = current_turn_listing_states(tool_calls_made);
    let active = states
        .iter()
        .filter(|(_, state)| state.active)
        .map(|(symbol, _)| symbol.as_str())
        .collect::<Vec<_>>();
    if !active.is_empty() {
        return format!(
            "{first_line}\n\n{} 的本轮同代码结构化行情与公司资料已确认其当前上市交易；不能用历史收购或旧退市记录否认当前上市。财报前瞻的其它项目本轮未形成足够可靠的完整证据，因此不补写未经核验的数字。",
            active.join("、")
        );
    }
    let symbols = violations
        .iter()
        .filter_map(|violation| violation.split(':').next())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("、");
    format!(
        "{first_line}\n\n本轮没有取得足以判断 {symbols} 当前上市状态的结构化证据，因此不能用历史并购或旧代码记忆断言其已退市或未上市。财报前瞻的其它项目也暂不补写未经核验的数字。"
    )
}

fn is_broad_us_market_request(runtime_input: &str) -> bool {
    let original = original_market_move_request(runtime_input).to_ascii_lowercase();
    ["美股", "美国股市", "u.s. market", "us market"]
        .iter()
        .any(|marker| original.contains(marker))
}

fn market_move_final_violations(
    runtime_input: &str,
    content: &str,
    context: &AgentContext,
    turn_message_start: usize,
) -> Vec<String> {
    if !is_market_move_final_check_enabled(runtime_input) {
        return Vec::new();
    }

    let mut violations = Vec::new();
    append_date_weekday_violations(content, &mut violations);
    append_market_move_quote_fact_violations(content, context, turn_message_start, &mut violations);

    let target_date =
        resolved_market_move_date(runtime_input, content, context, turn_message_start);
    if let Some(target_date) = target_date {
        append_declared_target_date_violations(content, target_date, &mut violations);
    }
    match target_date {
        Some(date) if !content_contains_date(content, date) => violations.push(format!(
            "终稿没有保留本轮证据锚定的目标日期 {} {}",
            date.format("%Y-%m-%d"),
            chinese_weekday_label(date.weekday())
        )),
        None => violations.push("终稿没有披露本轮实际解释的绝对市场本地日期".to_string()),
        _ => {}
    }

    if is_broad_us_market_request(runtime_input) && broad_us_market_scope_coverage(content) < 2 {
        violations.push(
            "美股宽基问题至少要分开核验两个代表性宽基/风格指数，不能用单一板块或个股替代"
                .to_string(),
        );
    }

    let current_urls = current_turn_tool_result_urls(context, turn_message_start);
    for paragraph in content.split("\n\n") {
        if !paragraph_has_definitive_market_cause(paragraph) {
            continue;
        }
        let matching_sources = current_urls
            .iter()
            .filter(|(url, _)| paragraph.contains(url.as_str()))
            .collect::<Vec<_>>();
        let has_current_url = !matching_sources.is_empty();
        let has_target_date =
            target_date.is_some_and(|date| content_contains_date(paragraph, date));
        let source_record_has_target_date = target_date.is_some_and(|date| {
            matching_sources
                .iter()
                .any(|(_, evidence)| content_contains_date(evidence, date))
        });
        if !has_current_url || !has_target_date || !source_record_has_target_date {
            violations.push(
                "每个确定性原因段落都必须在同一段内给出目标日期和本轮工具实际返回的原始 URL，且该 URL 的本轮结果记录本身也须覆盖目标日期；否则降级为“原因本轮未完全核验”"
                    .to_string(),
            );
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

fn bounded_market_move_draft(content: &str) -> String {
    content
        .chars()
        .take(MAX_MARKET_MOVE_DRAFT_FEEDBACK_CHARS)
        .collect()
}

/// The mechanical round repairs the named facts, not the answer's depth. An
/// earlier version also capped the body at ~350 characters, capped causes at
/// two, and banned scenarios and recommendations, so one wrong percentage
/// permanently flattened every move answer; do not restore any of those.
fn market_move_final_correction_prompt_with_sources(
    violations: &[String],
    draft: &str,
    target_date: Option<NaiveDate>,
    eligible_source_urls: &[String],
) -> String {
    let target = target_date.map_or_else(
        || "未形成唯一绝对日期".to_string(),
        |date| {
            format!(
                "{}（{}）",
                date.format("%Y-%m-%d"),
                chinese_weekday_label(date.weekday())
            )
        },
    );
    let sources = if eligible_source_urls.is_empty() {
        "无".to_string()
    } else {
        eligible_source_urls.join("、")
    };
    format!(
        "【内部终稿机械一致性纠正】上一版终稿尚未发布正文，只保留了服务端精确首行。请由同一 Agent 根据本轮已有真实工具结果，对本条消息末尾附上的那份原稿做定点修正，整篇输出完整终稿，不要只回复改动处或修改说明，也不要向用户提及本检查。逐项修正：{}。机械目标日期：{target}。当前工具结果中记录本身覆盖该日期的可用原始 URL 仅限：{sources}。除被点名的这几处及依赖它们的句子外，原稿其余部分保留原意与篇幅，不因这次修正而删减或压缩。原稿若还没走完 `market_analysis`「归因之后的三段」（事件换算成数字、长期公式动没动、落到条件化应对），这一轮按该 skill 的口径补上；全篇数值一律取自本轮工具结果或由其推导，推导值在同句交代输入项或算式，第三段照旧按 `market_analysis` 第 10 条处理买卖点。这三段属于推断，按该 skill 的「事实/推断分离」与已核验事实分段写；确定性归因措辞只出现在同段内已给出目标绝对日期和上述可用 URL 的那几段里，不写其它日期的原因。百分比命名与区间涨跌幅的写法按 `market_analysis` 那条。若可用 URL 为“无”或不足以支持原因，只把原因那一段降级为“原因本轮未完全核验”，已核验行情与上述三段照常保留。修订稿仍须从同一条精确数据时间首行开始；草稿若在句中断开，说明后面被截去，按同一结构写完整，不要停在断开处。\n<unpublished_draft>\n{}\n</unpublished_draft>",
        violations.join("；"),
        bounded_market_move_draft(draft)
    )
}

/// True when the text between a ticker and a percentage names a metric other
/// than the security's price change, so the percentage must not be reconciled
/// against `hone_change_basis.pct`.
fn percentage_names_another_metric(lead_in: &str) -> bool {
    const OTHER_METRICS: &[&str] = &[
        "毛利率",
        "利润率",
        "净利率",
        "税率",
        "费用率",
        "占比",
        "比例",
        "份额",
        "稀释",
        "摊薄",
        "收益率",
        "增速",
        "增长",
        "复合",
        "折价",
        "溢价",
        "覆盖",
        "良率",
        "利用率",
        "分位",
        "概率",
        "中位",
        "权重",
        "仓位",
        "转化",
        "合理价",
        "目标价",
        "估值",
        "隐含",
        "空间",
        "回报",
        "利差",
        "换手",
        "股息",
    ];
    const OTHER_METRICS_ASCII: &[&str] = &[
        "margin",
        "share",
        "dilut",
        "yield",
        "growth",
        "cagr",
        "ratio",
        "coverage",
        "percentile",
        "probability",
        "discount",
        "premium",
        "allocation",
        "weight",
        "utilization",
        "conversion",
        "upside",
        "downside",
    ];
    if OTHER_METRICS.iter().any(|word| lead_in.contains(word)) {
        return true;
    }
    let lowered = lead_in.to_ascii_lowercase();
    OTHER_METRICS_ASCII
        .iter()
        .any(|word| lowered.contains(word))
}

fn format_market_move_change(change_percentage: f64) -> String {
    if change_percentage >= 0.0 {
        format!("+{change_percentage:.2}%")
    } else {
        format!("{change_percentage:.2}%")
    }
}

fn deterministic_market_move_gap_response(
    runtime_input: &str,
    rejected_content: &str,
    required_prefix: &str,
    context: &AgentContext,
    turn_message_start: usize,
) -> String {
    let first_line = if required_prefix.ends_with("；行情口径：") {
        format!("{required_prefix}本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露")
    } else {
        required_prefix.to_string()
    };
    let target_date =
        resolved_market_move_date(runtime_input, rejected_content, context, turn_message_start);
    let quotes = current_turn_market_move_quotes(context, turn_message_start)
        .into_iter()
        .filter(|quote| target_date.is_none_or(|date| quote.market_date == date))
        .collect::<Vec<_>>();
    let target_line = target_date.map_or_else(
        || "目标时段：当前原话未形成可安全发布的唯一绝对市场日期。".to_string(),
        |date| {
            let target_kind = if first_absolute_market_date(original_market_move_request(
                runtime_input,
            ))
            .is_some()
                || anchored_market_move_date(runtime_input).is_some()
            {
                "用户原话对应的市场本地民用日期候选"
            } else if quotes.iter().any(|quote| quote.market_date == date) {
                "本轮最近可得 quote 的纽约日历日期"
            } else {
                "市场本地民用日期候选"
            };
            format!(
                "目标时段：{}（{}，{}；是否开市及具体交易时段仍以本轮行情证据为准）。",
                date.format("%Y-%m-%d"),
                chinese_weekday_label(date.weekday()),
                target_kind
            )
        },
    );
    let quote_line = if quotes.is_empty() {
        "本轮没有取得与该日期一致、可安全复述的 quote 快照。".to_string()
    } else {
        let changes = quotes
            .iter()
            .map(|quote| {
                format!(
                    "{} {}",
                    quote.symbol,
                    format_market_move_change(quote.change_percentage)
                )
            })
            .collect::<Vec<_>>()
            .join("、");
        let quote_time = quotes
            .iter()
            .filter_map(|quote| quote.beijing_time.as_deref())
            .next()
            .map_or_else(
                || "provider 未返回可统一复述的北京时间".to_string(),
                |time| format!("provider 北京时间 {time}"),
            );
        format!(
            "本轮最近可得 quote（{quote_time}；该时间字段本身不额外证明交易时段）显示：{changes}。"
        )
    };
    let scope_line = if is_broad_us_market_request(runtime_input) && quotes.len() >= 2 {
        let positive = quotes
            .iter()
            .filter(|quote| quote.change_percentage > 0.0)
            .count();
        let negative = quotes
            .iter()
            .filter(|quote| quote.change_percentage < 0.0)
            .count();
        if positive > 0 && negative > 0 {
            "按这组代表性宽基/风格 ETF，本轮可核验范围呈分化，不支持“美股宽基全面大跌”的前提；若你观察的是科技或其它具体板块，需要按那个范围另行归因。"
        } else if negative == quotes.len() {
            "按这组代表性宽基/风格 ETF，本轮可核验方向一致偏弱；“大跌”的程度仍以各自披露的具体幅度为准。"
        } else {
            "按这组代表性宽基/风格 ETF，本轮可核验范围不支持“美股宽基全面大跌”的前提。"
        }
    } else {
        "以上只回答本轮 quote 实际覆盖的对象，不把单股、板块与宽基市场互相替代。"
    };
    format!(
        "{first_line}\n\n{target_line}\n\n{quote_line}\n\n{scope_line}\n\n本轮原因证据未能同时满足同一对象、同一目标日期和当前来源原文核对，因此不把候选新闻、标题摘要或其它交易日叙事写成已确认触发因素。**原因本轮未完全核验。**"
    )
}

fn canonical_prefix_delta(
    expected_prefix: &str,
    visible_candidate: &str,
    forwarded_bytes: usize,
) -> Result<String, ()> {
    if !(expected_prefix.starts_with(visible_candidate)
        || visible_candidate.starts_with(expected_prefix))
    {
        return Err(());
    }
    let mut target_bytes = visible_candidate.len().min(expected_prefix.len());
    // Once the candidate moves past the expected header line, forward its
    // terminating newline as well: that single byte is what lets the stream
    // observer validate the completed canonical header and commit it while
    // the retry-unstable body stays deferred. Without it the header could
    // only ever commit through a service-side pre-commit.
    if visible_candidate.len() > expected_prefix.len()
        && visible_candidate.as_bytes()[expected_prefix.len()] == b'\n'
    {
        target_bytes = expected_prefix.len() + 1;
    }
    if target_bytes < forwarded_bytes {
        return Err(());
    }
    Ok(visible_candidate[forwarded_bytes..target_bytes].to_string())
}

fn exact_final_answer_prefix(user_input: &str) -> Option<String> {
    const REQUIREMENT: &str = "第一条非空行必须严格以 `";
    let start = user_input.rfind(REQUIREMENT)? + REQUIREMENT.len();
    let remainder = &user_input[start..];
    let end = remainder.find('`')?;
    let prefix = remainder[..end].trim();
    (prefix.starts_with("数据时间：北京时间 ") && prefix.ends_with("；行情口径："))
        .then(|| prefix.to_string())
}

fn exact_prefix_instruction(required_prefix: Option<&str>) -> String {
    match required_prefix {
        Some(prefix) => format!(
            "第一条非空行必须逐字以 `{prefix}` 开头，第一可见字符必须是‘数’。这是服务端为本轮生成的精确 Session 时间锚点，不得改用 quote 的报价源时间。"
        ),
        None => "第一条非空行必须严格使用 `数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：`，其中 YYYY-MM-DD HH:MM 只能取当前 runtime user message 的本轮 Session 北京时间，第一可见字符必须是‘数’。".to_string(),
    }
}

#[cfg(test)]
fn terminal_synthesis_prompt(
    required_prefix: Option<&str>,
    handoff: &ValidatedResearchHandoff,
) -> String {
    let handoff_json = serde_json::to_string(handoff).unwrap_or_else(|_| {
        "{\"facts\":[],\"inferences\":[],\"gaps\":[\"证据交接序列化失败\"]}".to_string()
    });
    let internal_warning_constraint = if handoff.validation_warnings.is_empty() {
        ""
    } else {
        "内部来源定位时已有无法解析的候选项被机械丢弃；它们不属于证据。不要向用户提及校验、协议、交接、丢弃或内部错误，也绝不能因此拒绝回答。"
    };
    format!(
        "【终局回答阶段】\n{}\n{}\n{}\n{}\n{}\n{}\n【Agent 本轮结构化证据交接；只有 resolved_evidence / fallback_evidence 是已机械定位的外部证据】\n{}",
        "Agent 已结束本轮合理的研究与工具尝试；服务端没有审核或否决答案语义。",
        "当前阶段不再提供任何工具；请直接生成一次完整、可见的最终回答。facts 只是证据分组，必须由其中 resolved_evidence 的逐字原文/标量字段自行归纳，不存在可照抄的自由文本 claim。fallback_evidence 是服务端从本轮成功工具结果机械压缩出的同等级证据目录。只有 Session 时间前缀可直接使用；行情口径中的价格、币种、涨跌和报价源时间也必须来自上述证据。inferences 必须明确写‘推断：’；gaps 只能披露未知，不能写成否定事实。",
        exact_prefix_instruction(required_prefix),
        FINAL_ANSWER_EVIDENCE_CONTRACT,
        FINAL_RELATIONSHIP_DELETION_CHECK,
        internal_warning_constraint,
        handoff_json
    )
}

#[cfg(test)]
fn active_business_turn_prompt(
    evidence_floor_satisfied: bool,
    route_guidance: &str,
    source_catalog: &str,
    finish_feedback: Option<&str>,
) -> String {
    let finish_feedback = finish_feedback
        .map(|feedback| {
            format!(
                "\n上一次内部 finish_research 交接未通过协议级来源定位，请在同一 Agent 内修正后继续；这不是用户可见错误，也不是答案审查：\n{feedback}"
            )
        })
        .unwrap_or_default();
    if evidence_floor_satisfied {
        format!(
            "【本轮仍是工具轮，不写终稿】下面仅是 Agent 已声明路线的结构调用状态，不证明用户点名实体已经完整、工具调用成功或问题所需业务证据充分：\n{}\n{}\n重新阅读完整用户原话与每条 tool result。若仍需更多业务证据，本轮只调用所需真实工具；若合理取证已经完成，本轮唯一动作是单独提交带 answer_scope / facts / inferences / gaps 的 `finish_research` 结构化交接。fact 不写 claim，只把相关证据分组：Web 使用来源目录中的完整 tool_call_id + result_number + 逐字 excerpt（标题和 URL 由服务端注入），DataFetch 使用来源目录中的完整 tool_call_id + 标量 JSON Pointer。未核验内容写入 gaps，绝不能由缺失推出否定事实。宽泛公司关系须由你按完整语义分别核查相关的商业/客户供应/技术合同与投资持股维度，优先一手来源，不得一次泛搜索后凭记忆收口。不要在这个仍提供工具的轮次输出数据时间、摘要、解释或最终正文；交接后的无工具阶段会生成唯一可见终稿。{}",
            route_guidance, source_catalog, finish_feedback,
        )
    } else {
        format!(
            "【本轮只取证，不作答】下面仅是 Agent 已声明路线的结构调用状态，不证明用户点名实体已经完整、工具调用成功或问题所需业务证据充分：\n{}\n重新阅读完整用户原话。本轮必须只返回一个或多个真实业务工具调用，禁止输出数据时间、摘要、解释、草稿或最终正文。先补齐上面逐路线列出的 search；标准 symbol 已确认后优先调用 snapshot 获取 quote、profile、报价时间、服务端涨跌口径和可得盘后字段，snapshot 不适用时再组合 quote/profile，扩展时段问题补 extended_hours。完成或实际尝试结构化行情后，再按用户原始问题补关系、财务、新闻、网页或公告证据；这是工具选择优先级，不是缺行情即拒答的门禁。每个 search 都重新填写本次调用自己的 entity_route 与 identity_match；关系问题的 Web 结果只是摘要证据，不能靠模型记忆补故事。宽泛关系要由你自主拆出相关的商业/客户供应/技术合同与投资持股待证维度，并尽量并行查一手来源。完成这些调用并读取结果后，再由下一轮决定继续取证还是提交结构化 finish_research。{}",
            route_guidance, finish_feedback,
        )
    }
}

fn agent_owned_business_turn_prompt(
    evidence_floor_satisfied: bool,
    route_guidance: &str,
    required_prefix: Option<&str>,
    force_final: bool,
    open_research_rescue: bool,
) -> String {
    if open_research_rescue {
        // The evidence floor is unsatisfied precisely because identity
        // resolution consumed the turn. Repeating the entity-first
        // instructions here would spend the rescue the same way.
        return format!(
            "【本轮改为开放检索取证】本轮已经用完常规研究预算，但仍未取得任何回答用户原问题所需的实质资料。下面的实体路线状态只说明此前尝试过什么，不是本轮任务：\n{route_guidance}\n不要再调用实体注册表、不要再解析或补查证券代码。重新完整阅读用户原话，判断用户真正想知道的主题（可能是宏观、资金流、仓位、策略、指标或行业问题，其中的大写缩写未必是证券代码），并本轮只返回围绕该真实主题的开放检索工具调用，可并行多条查询。不要在本轮输出数据时间、摘要或最终正文；真实检索结果进入上下文后，由下一轮同一 Agent 直接作答。"
        );
    }
    if force_final {
        return format!(
            "【本轮由同一 Agent 有界收口】研究工具批次已经完整结束，本轮没有任何可用工具。重新阅读完整用户原话和本轮真实 tool result，直接生成一次完整自然终稿。只使用已经取得的证据；未覆盖的候选、行情或业务事实如实写入缺口，不得继续规划工具、凭模型记忆补齐或提及内部预算。\n{}\n{}\n{}\n{}",
            exact_prefix_instruction(required_prefix),
            FINAL_ANSWER_EVIDENCE_CONTRACT,
            DIRECT_FINAL_RELATIONSHIP_CHECK,
            route_guidance,
        );
    }
    let route_guidance = format!(
        "{route_guidance}\n【有界筛选约束】本轮最多接纳 {MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES} 条实体路线，优先用户明确点名标的，筛选结果通常最多保留 {MAX_AGENT_OWNED_SCREENING_CANDIDATE_ROUTES} 条新候选；达到上限后只完成已接纳路线，不再扩展公司清单。已有 Web 结果能命名候选时，当前批次直接为候选执行 identity search，不要重复泛搜。后续 snapshot / quote / profile 的 entity_route 必须逐字复制上方 guidance 中的原键，不得翻译、缩写或重命名；支持 snapshot 时优先一次完成 quote、profile、报价时间、服务端涨跌口径和可得盘后字段，扩展时段问题另补 extended_hours。结构化行情的工具选择优先级高于 Web 搜索，但这不是完成门禁；provider 无覆盖或调用失败时不反复补取，继续使用当前证据和开放检索。同一工具与相同参数已有成功结果时不得重复调用。"
    );
    if !evidence_floor_satisfied {
        return format!(
            "【本轮只取证，不作答】下面只是同一 Agent 当前建立的实体路线与结构调用状态，不证明工具结果成功或问题所需证据充分：\n{}\n重新阅读完整用户原话。本轮只返回一个或多个真实业务工具调用，不输出数据时间、摘要、解释、草稿或最终正文。先为尚缺身份候选的每条路线分别调用 search；每个 search 都携带自己的 entity_route 与 identity_match，用户书写的 ticker 不要求大写，小写或混合大小写代码应先规范成标准代码并走 exact_symbol。标准 symbol 已确认后优先调用 snapshot 获取完整行情与资产路由，snapshot 不适用时再组合 quote/profile（crypto 用 crypto_quote），扩展时段问题补 extended_hours；之后再按用户原始公司名与问题主题调用 Web/news/filing/industry 检索一手来源。互不依赖且确有时效需要时，行情与 Web 可同批，但应把行情调用列在 Web 之前。quote、profile/snapshot 等依赖标准 symbol 的调用必须等待对应 search 结果返回；当前上下文已有该路线标准 symbol 时才可执行，禁止猜测、补全或凭记忆构造代码。行情优先只是工具选择信号，不是终稿门禁；provider 无覆盖或调用失败时继续搜索和回答，不反复补取。关系题不能把 search/profile 当关系证据，应按完整语义核查商业/客户供应/技术合同与投资持股等相关维度。真实工具结果进入当前上下文后，由下一轮同一 Agent 继续取证或直接自然作答。",
            route_guidance,
        );
    }

    format!(
        "【本轮由同一 Agent 自然收口】下面只是当前实体路线的结构调用状态，不证明所有业务证据均已取得：\n{}\n重新阅读完整用户原话与本轮每条真实 tool result。若当前问题仍缺少关键证据，本轮只调用需要的真实业务工具；若合理取证已经完成，或必要来源经实际尝试后明确不可得，本轮直接生成一次完整自然终稿，并让回答范围跟随用户原问题。\n{}\n{}\n{}",
        route_guidance,
        exact_prefix_instruction(required_prefix),
        FINAL_ANSWER_EVIDENCE_CONTRACT,
        DIRECT_FINAL_RELATIONSHIP_CHECK,
    )
}

fn scrub_research_evidence_messages(messages: &mut [Message], strip_reasoning: bool) {
    for message in messages {
        if strip_reasoning {
            message.reasoning_content = None;
        }
        if message.role == "assistant"
            && message
                .tool_calls
                .as_ref()
                .is_some_and(|tool_calls| !tool_calls.is_empty())
        {
            message.content = Some(String::new());
        }
    }
}

fn truncate_context_json_strings(value: &Value, max_chars: usize) -> Value {
    match value {
        Value::String(text) => Value::String(truncate_chars(text, max_chars)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| truncate_context_json_strings(item, max_chars))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, item)| (key.clone(), truncate_context_json_strings(item, max_chars)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn bounded_context_json_value(value: &Value, max_chars: usize) -> (Value, bool) {
    let serialized = value.to_string();
    let original_chars = serialized.chars().count();
    if original_chars <= max_chars {
        return (value.clone(), false);
    }

    // Never trim a large result one array element at a time. Serializing the
    // complete value after every pop makes calendar/search payloads O(n²) and
    // can monopolize a Tokio worker for minutes exactly when overflow recovery
    // needs to remain cheap. Values far beyond the budget go directly to one
    // bounded preview; near-budget values may still retain their structure by
    // shortening strings in a fixed number of linear passes.
    if original_chars <= max_chars.saturating_mul(4) {
        for string_limit in [1_000, 512, 256, 128, 64] {
            let compact = truncate_context_json_strings(value, string_limit);
            if compact.to_string().chars().count() <= max_chars {
                return (compact, true);
            }
        }
    }

    let mut preview_chars = max_chars.saturating_sub(160).min(1_000);
    loop {
        let fallback = serde_json::json!({
            "status": "tool_result_compacted",
            "original_chars": original_chars,
            "preview": truncate_chars(&serialized, preview_chars),
        });
        if fallback.to_string().chars().count() <= max_chars || preview_chars == 0 {
            return (fallback, true);
        }
        preview_chars /= 2;
    }
}

fn compact_context_overflow_tool_records(
    tool_calls_made: &[ToolCallMade],
    result_budget: usize,
    arguments_budget: usize,
) -> Value {
    Value::Array(
        tool_calls_made
            .iter()
            .map(|call| {
                let (arguments, arguments_compacted) =
                    bounded_context_json_value(&call.arguments, arguments_budget);
                let (result, result_compacted) =
                    bounded_context_json_value(&call.result, result_budget);
                serde_json::json!({
                    "tool_call_id": call.tool_call_id,
                    "tool_name": call.name,
                    "arguments": arguments,
                    "arguments_compacted": arguments_compacted,
                    "result": result,
                    "result_compacted": result_compacted,
                })
            })
            .collect(),
    )
}

fn bounded_context_overflow_tool_evidence(tool_calls_made: &[ToolCallMade]) -> String {
    if tool_calls_made.is_empty() {
        return "[]".to_string();
    }

    let call_count = tool_calls_made.len();
    let overhead = call_count.saturating_mul(384);
    let available = CONTEXT_OVERFLOW_RECOVERY_TOTAL_TOOL_CHARS.saturating_sub(overhead);
    let initial_result_budget = (available / call_count).clamp(
        CONTEXT_OVERFLOW_RECOVERY_MIN_RESULT_CHARS,
        CONTEXT_OVERFLOW_RECOVERY_MAX_RESULT_CHARS,
    );
    for (result_budget, arguments_budget) in [
        (initial_result_budget, 512),
        (
            initial_result_budget
                .saturating_mul(3)
                .checked_div(4)
                .unwrap_or_default()
                .max(512),
            256,
        ),
        (CONTEXT_OVERFLOW_RECOVERY_MIN_RESULT_CHARS, 128),
    ] {
        let records =
            compact_context_overflow_tool_records(tool_calls_made, result_budget, arguments_budget);
        let serialized = records.to_string();
        if serialized.chars().count() <= CONTEXT_OVERFLOW_RECOVERY_TOTAL_TOOL_CHARS {
            return serialized;
        }
    }

    // Extremely high call counts still retain every call's source identity.
    // The result body is deliberately unavailable instead of dropping a call
    // or sending a partial JSON fragment that the model could misread.
    Value::Array(
        tool_calls_made
            .iter()
            .map(|call| {
                serde_json::json!({
                    "tool_call_id": call.tool_call_id,
                    "tool_name": call.name,
                    "arguments": {"status": "omitted_from_bounded_copy"},
                    "arguments_compacted": true,
                    "result": {"status": "omitted_from_bounded_copy"},
                    "result_compacted": true,
                })
            })
            .collect(),
    )
    .to_string()
}

enum ActiveBusinessStreamOutcome {
    Tools(ChatResponse),
    DirectFinal(ChatResponse),
    Empty,
}

fn consume_active_business_retry(failures: &mut u32) -> bool {
    if *failures >= ACTIVE_BUSINESS_FAILURE_RETRY_LIMIT {
        return false;
    }
    *failures = failures.saturating_add(1);
    true
}

fn is_recoverable_openai_compatible_protocol_error_text(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    lowered.contains("tool call result does not follow tool call")
        || lowered.contains("stream ended before done")
        || (lowered.contains("stream transport error")
            && (lowered.contains("error decoding response body")
                || lowered.contains("response body decode")
                || lowered.contains("decode failure")))
}

fn failed_agent_response(
    tool_calls_made: Vec<ToolCallMade>,
    iterations: u32,
    error: impl Into<String>,
) -> AgentResponse {
    AgentResponse {
        content: String::new(),
        tool_calls_made,
        iterations,
        success: false,
        error: Some(error.into()),
    }
}

async fn await_before_deadline<T, F>(
    deadline: Option<tokio::time::Instant>,
    timeout_error: &'static str,
    future: F,
) -> hone_core::HoneResult<T>
where
    F: Future<Output = hone_core::HoneResult<T>>,
{
    match deadline {
        Some(deadline) => tokio::time::timeout_at(deadline, future)
            .await
            .map_err(|_| hone_core::HoneError::Llm(timeout_error.to_string()))?,
        None => future.await,
    }
}

async fn await_unit_before_deadline<F>(
    deadline: Option<tokio::time::Instant>,
    timeout_error: &'static str,
    future: F,
) -> hone_core::HoneResult<()>
where
    F: Future<Output = ()>,
{
    match deadline {
        Some(deadline) => tokio::time::timeout_at(deadline, future)
            .await
            .map_err(|_| hone_core::HoneError::Llm(timeout_error.to_string())),
        None => {
            future.await;
            Ok(())
        }
    }
}

fn step_deadline(
    overall_deadline: Option<tokio::time::Instant>,
    step_timeout: Option<Duration>,
) -> (Option<tokio::time::Instant>, &'static str) {
    let step_deadline = step_timeout.map(|timeout| tokio::time::Instant::now() + timeout);
    match (overall_deadline, step_deadline) {
        (Some(overall), Some(step)) if overall <= step => {
            (Some(overall), AGENT_OVERALL_TIMEOUT_ERROR)
        }
        (Some(_), Some(step)) => (Some(step), AGENT_STEP_TIMEOUT_ERROR),
        (Some(overall), None) => (Some(overall), AGENT_OVERALL_TIMEOUT_ERROR),
        (None, Some(step)) => (Some(step), AGENT_STEP_TIMEOUT_ERROR),
        (None, None) => (None, AGENT_STEP_TIMEOUT_ERROR),
    }
}

fn active_business_deadline(
    overall_deadline: Option<tokio::time::Instant>,
    step_timeout: Option<Duration>,
) -> (tokio::time::Instant, &'static str) {
    let (configured_deadline, configured_error) = step_deadline(overall_deadline, step_timeout);
    match configured_deadline {
        Some(configured_deadline) => (configured_deadline, configured_error),
        None => (
            tokio::time::Instant::now() + FALLBACK_ACTIVE_BUSINESS_TIMEOUT,
            "active business stream timed out",
        ),
    }
}

fn canonical_agent_timeout(error: &impl std::fmt::Display) -> Option<&'static str> {
    let error = error.to_string();
    if error.contains(AGENT_OVERALL_TIMEOUT_ERROR) {
        Some(AGENT_OVERALL_TIMEOUT_ERROR)
    } else if error.contains(AGENT_STEP_TIMEOUT_ERROR) {
        Some(AGENT_STEP_TIMEOUT_ERROR)
    } else {
        None
    }
}

// Keep the agent crate independent from channel presentation code while using
// the same hidden-tag semantics for incremental model output.
mod hone_channels_compat {
    #[derive(Default)]
    pub(super) struct HiddenStreamFormatter {
        pending: String,
        hidden: Option<&'static str>,
    }

    impl HiddenStreamFormatter {
        pub(super) fn push(&mut self, chunk: &str) -> String {
            self.pending.push_str(chunk);
            let mut visible = String::new();
            loop {
                if let Some(close) = self.hidden {
                    let Some(end) = self.pending.find(close) else {
                        break;
                    };
                    self.pending.drain(..end + close.len());
                    self.hidden = None;
                    continue;
                }
                let markers = [
                    ("<think>", "</think>"),
                    ("<tool_code>", "</tool_code>"),
                    ("<tool_call>", "</tool_call>"),
                    ("<tool_result>", "</tool_result>"),
                    ("<tool_use>", "</tool_use>"),
                    ("<minimax:tool_call>", "</minimax:tool_call>"),
                    ("<minimax:tool_result>", "</minimax:tool_result>"),
                    ("<minimax:tool_use>", "</minimax:tool_use>"),
                ];
                if let Some((start, open, close)) = markers
                    .iter()
                    .filter_map(|(open, close)| {
                        self.pending.find(open).map(|start| (start, *open, *close))
                    })
                    .min_by_key(|(start, _, _)| *start)
                {
                    visible.push_str(&self.pending[..start]);
                    self.pending.drain(..start + open.len());
                    self.hidden = Some(close);
                    continue;
                }
                let keep = markers
                    .iter()
                    .map(|(open, _)| trailing_prefix_len(&self.pending, open))
                    .max()
                    .unwrap_or(0);
                let emit_len = self.pending.len().saturating_sub(keep);
                visible.push_str(&self.pending[..emit_len]);
                self.pending.drain(..emit_len);
                break;
            }
            visible
        }

        pub(super) fn finish(&mut self) -> String {
            if self.hidden.is_some() {
                self.pending.clear();
                return String::new();
            }
            std::mem::take(&mut self.pending)
        }
    }

    fn trailing_prefix_len(text: &str, marker: &str) -> usize {
        (1..marker.len())
            .rev()
            .find(|length| text.ends_with(&marker[..*length]))
            .unwrap_or(0)
    }
}

/// The global slots a tool may actually spend. Inside the bounded finance loop
/// the last few slots are reserved for open web research so a turn can never
/// spend its entire budget on entity resolution and then answer from memory.
fn effective_global_tool_budget(
    tool_name: &str,
    max_tool_calls: Option<u32>,
    reserve_open_research: bool,
) -> Option<u32> {
    let limit = max_tool_calls?;
    if !reserve_open_research || tool_name == AGENT_OWNED_OPEN_RESEARCH_TOOL {
        return Some(limit);
    }
    Some(limit.saturating_sub(AGENT_OWNED_FINANCE_RESERVED_OPEN_RESEARCH_CALLS))
}

/// Pure budget check: two of the three call sites use it as a feasibility probe
/// (`.is_none()` / `.is_some()`) rather than to reject anything, so this must
/// stay side-effect free. Logging it here reported a rejection that never
/// happened, and at heartbeat budgets (global 3 / web_search 1 / data_fetch 2)
/// that buried real errors in the channel journals. The site that actually
/// rejects a call does the logging.
fn tool_budget_error(
    tool_name: &str,
    max_tool_calls: Option<u32>,
    tool_call_limits: &HashMap<String, u32>,
    total_tool_calls: u32,
    tool_call_counts: &HashMap<String, u32>,
) -> Option<Value> {
    if let Some(limit) = max_tool_calls
        && total_tool_calls >= limit
    {
        return Some(serde_json::json!({
            "error": format!("tool call limit reached ({limit})")
        }));
    }

    let Some(limit) = tool_call_limits.get(tool_name).copied() else {
        return None;
    };
    let used = tool_call_counts.get(tool_name).copied().unwrap_or(0);
    if used >= limit {
        return Some(serde_json::json!({
            "error": format!("tool `{tool_name}` call limit reached ({limit})")
        }));
    }
    None
}

#[async_trait]
impl Agent for FunctionCallingAgent {
    /// 运行一次非流式 Agent turn，直到没有新的工具调用或达到迭代上限。
    ///
    /// 1. 接收用户输入
    /// 2. 调用 LLM，传入可用工具列表
    /// 3. 如果 LLM 返回 `tool_calls`，执行对应工具
    /// 4. 将工具结果反馈给 LLM
    /// 5. 重复 2-4 直到 LLM 返回最终答案
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse {
        let turn_message_start = context.messages.len();
        context.add_user_message(user_input);
        let required_final_answer_prefix = self
            .service_owned_initial_prefix
            .clone()
            .or_else(|| exact_final_answer_prefix(user_input));
        let mut precommitted_service_prefix = self
            .precommitted_service_prefix
            .as_ref()
            .filter(|prefix| Some(*prefix) == required_final_answer_prefix.as_ref())
            .cloned();
        let overall_deadline = self
            .overall_timeout
            .map(|timeout| tokio::time::Instant::now() + timeout);

        let business_tools: Vec<Value> = self.tools.get_tools_schema();
        let registered_tool_names = self
            .tools
            .list_tool_names()
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut tool_call_counts: HashMap<String, u32> = HashMap::new();
        let finance_budget = self.finance_research_budget;
        let finance_max_tool_calls = Some(
            self.max_tool_calls
                .unwrap_or(finance_budget.tool_calls)
                .min(finance_budget.tool_calls),
        );
        let mut finance_tool_call_limits = self.tool_call_limits.clone();
        for (name, cap) in [
            ("data_fetch", finance_budget.data_fetch_calls),
            ("web_search", finance_budget.web_search_calls),
        ] {
            finance_tool_call_limits
                .entry(name.to_string())
                .and_modify(|limit| *limit = (*limit).min(cap))
                .or_insert(cap);
        }
        let mut total_tool_calls = 0u32;
        let mut iterations: u32 = 0;
        // A market-move request already carries a service-owned date anchor.
        // Enter the existing finance loop from the first model round so a
        // Web-only answer cannot bypass DataFetch identity/quote evidence.
        let mut investment_research_started =
            market_move_starts_investment_research(user_input, self.agent_owned_finance_loop);
        let mut research_evidence = ResearchEvidenceLedger::with_identity_route_limit(
            self.agent_owned_finance_loop
                .then_some(MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES),
        );
        research_evidence.broad_market_mode = is_market_move_final_check_enabled(user_input)
            && is_broad_us_market_request(user_input);
        let mut finance_tool_rounds = 0u32;
        let mut finance_identity_rounds = 0u32;
        // Executed business calls that are not identity-only searches. Identity
        // resolution answers "which security is this?"; only these calls can
        // answer what the user actually asked. Counted at the execution site so
        // a Web-only turn that never touches DataFetch still registers.
        let mut substantive_business_calls = 0u32;
        let mut evidence_rescue_rounds = 0u32;
        let mut tool_budget_exhausted = false;
        let mut service_prefix_commit_eligible = true;
        let mut active_business_failures = 0u32;
        let mut pending_market_move_final_correction: Option<String> = None;
        let mut market_move_final_corrections = 0u32;
        let mut pending_gap_closure_feedback: Option<String> = None;
        let mut gap_closure_rounds = 0u32;
        let mut pending_listing_final_correction: Option<String> = None;
        let mut pending_language_final_correction: Option<String> = None;
        let mut research_evidence_retries = 0u32;
        let mut research_evidence_retry_pending = false;
        let mut listing_final_corrections = 0u32;
        let mut language_final_corrections = 0u32;
        let mut blocked_tool_finalization: Option<String> = None;
        let mut read_only_trace_finalization: Option<String> = None;
        let mut context_overflow_finalization = false;
        let mut persistent_mutation_finalization = false;
        #[cfg(test)]
        let mut pending_finish_feedback: Option<String> = None;
        #[cfg(test)]
        let mut unavailable_finish_corrections = 0u32;
        #[cfg(test)]
        let mut invalid_finish_corrections = 0u32;

        self.dbg(&format!(
            "[Agent] start tools={:?}",
            self.tools.list_tool_names()
        ));

        loop {
            if overall_deadline.is_some_and(|deadline| deadline <= tokio::time::Instant::now()) {
                return failed_agent_response(
                    tool_calls_made,
                    iterations,
                    AGENT_OVERALL_TIMEOUT_ERROR,
                );
            }
            #[cfg(test)]
            let legacy_finish_terminal = self.finish_research_terminal_synthesis_enabled();
            #[cfg(test)]
            let finance_protocol_active = (self.agent_owned_finance_loop || legacy_finish_terminal)
                && investment_research_started;
            #[cfg(not(test))]
            let finance_protocol_active =
                self.agent_owned_finance_loop && investment_research_started;
            let bounded_finance_research_active = self.agent_owned_finance_loop
                && (finance_protocol_active
                    || precommitted_service_prefix.is_some()
                    || blocked_tool_finalization.is_some());
            let market_move_bounded_final_ready = bounded_finance_research_active
                && research_evidence.broad_market_mode
                && finance_tool_rounds >= 1
                && tool_call_counts
                    .get("web_search")
                    .is_some_and(|attempts| *attempts >= 1)
                && current_turn_verified_broad_market_quote_groups(context, turn_message_start)
                    .len()
                    >= 2;
            let finance_budget_final_due = bounded_finance_research_active
                && (finance_tool_rounds >= finance_budget.tool_rounds
                    || tool_budget_exhausted
                    || market_move_bounded_final_ready
                    || blocked_tool_finalization.is_some());
            // Structural invariant: the loop never closes a finance turn with a
            // tools-disabled final while it has spent the whole turn resolving
            // identifiers and never looked at what the user asked. Blocked
            // batches keep their own safety finalization; everything else gets
            // one bounded open-research round instead of an offline answer.
            let evidence_rescue_due = finance_budget_final_due
                && blocked_tool_finalization.is_none()
                && substantive_business_calls == 0
                && evidence_rescue_rounds < MAX_AGENT_OWNED_FINANCE_EVIDENCE_RESCUE_ROUNDS
                && registered_tool_names.contains(AGENT_OWNED_OPEN_RESEARCH_TOOL)
                && iterations + 1 < self.max_iterations
                && tool_budget_error(
                    AGENT_OWNED_OPEN_RESEARCH_TOOL,
                    finance_max_tool_calls,
                    &finance_tool_call_limits,
                    total_tool_calls,
                    &tool_call_counts,
                )
                .is_none();
            if evidence_rescue_due {
                evidence_rescue_rounds = evidence_rescue_rounds.saturating_add(1);
                tracing::warn!(
                    session_id = %context.session_id,
                    iteration = iterations,
                    finance_tool_rounds,
                    finance_identity_rounds,
                    total_tool_calls,
                    substantive_business_calls,
                    "agent-owned finance budget reached with no substantive evidence; granting an open-research round"
                );
            }
            let force_finance_final = finance_budget_final_due && !evidence_rescue_due;
            let force_context_overflow_final = context_overflow_finalization;

            if iterations >= self.max_iterations
                && blocked_tool_finalization.is_none()
                && read_only_trace_finalization.is_none()
                && !force_context_overflow_final
                && !persistent_mutation_finalization
            {
                if trace_has_only_known_read_only_calls(&tool_calls_made) {
                    read_only_trace_finalization =
                        Some(format!("max_iterations_exceeded:{}", self.max_iterations));
                    continue;
                }
                // The iteration bound is a normal failed run, never implicit
                // finish authority. A bounded finance final receives its own
                // tools-disabled iteration before reaching this guard.
                return AgentResponse {
                    content: String::new(),
                    tool_calls_made,
                    iterations,
                    success: false,
                    error: Some(format!("max_iterations_exceeded:{}", self.max_iterations)),
                };
            }

            iterations += 1;
            self.dbg(&format!("[Agent] iter={iterations}"));

            // A T0 prefix starts the delivery/budget boundary immediately, but
            // does not force finance-specific discovery instructions onto the
            // first Web-only rounds. The same Agent enters the active final path
            // only once DataFetch activates or the bounded final is due.
            let active_business_round = finance_protocol_active || force_finance_final;
            let force_persistent_mutation_final = persistent_mutation_finalization;
            #[cfg(test)]
            let finish_research_available = legacy_finish_terminal
                && research_evidence.completion_signal_available(active_business_round);
            let evidence_floor_satisfied =
                research_evidence.evidence_floor_satisfied(active_business_round);
            #[cfg(test)]
            let research_sources = finish_research_available
                .then(|| current_turn_research_source_catalog(context, turn_message_start))
                .unwrap_or_default();
            let mut round_tools = business_tools.clone();
            #[cfg(test)]
            if finish_research_available {
                round_tools.push(finish_research_tool_schema(&research_sources));
            }
            if force_finance_final
                || read_only_trace_finalization.is_some()
                || force_context_overflow_final
                || force_persistent_mutation_final
            {
                round_tools.clear();
            } else if evidence_rescue_due {
                // Take the identity registry off the table for this round so
                // the rescue cannot be spent re-resolving the same identifier
                // that already consumed the turn.
                round_tools.retain(|tool| {
                    tool.get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some("data_fetch")
                });
            }
            let has_tools = !round_tools.is_empty();
            let tool_choice_mode = if force_context_overflow_final {
                ToolChoiceMode::Auto
            } else if active_business_round && !evidence_floor_satisfied && !force_finance_final {
                ToolChoiceMode::Required
            } else if research_evidence_retry_pending && has_tools {
                // The previous round asserted a current listing status having
                // called nothing. Asking again without requiring a tool call
                // just invites the same memory answer.
                ToolChoiceMode::Required
            } else {
                ToolChoiceMode::Auto
            };
            research_evidence_retry_pending = false;
            let round_instruction = Some(if force_context_overflow_final {
                CONTEXT_OVERFLOW_FINALIZATION_INSTRUCTION
            } else if force_persistent_mutation_final {
                PERSISTENT_MUTATION_FINALIZATION_INSTRUCTION
            } else if force_finance_final {
                AGENT_OWNED_FINANCE_FORCED_FINAL_SYSTEM_INSTRUCTION
            } else if read_only_trace_finalization.is_some() {
                READ_ONLY_TRACE_FINALIZATION_INSTRUCTION
            } else if evidence_rescue_due {
                AGENT_OWNED_OPEN_RESEARCH_RESCUE_INSTRUCTION
            } else if active_business_round {
                #[cfg(not(test))]
                {
                    if evidence_floor_satisfied {
                        AGENT_OWNED_RESEARCH_SYSTEM_INSTRUCTION
                    } else {
                        POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION
                    }
                }
                #[cfg(test)]
                {
                    if self.agent_owned_finance_loop && !legacy_finish_terminal {
                        if evidence_floor_satisfied {
                            AGENT_OWNED_RESEARCH_SYSTEM_INSTRUCTION
                        } else {
                            POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION
                        }
                    } else if finish_research_available {
                        ACTIVE_RESEARCH_SYSTEM_INSTRUCTION
                    } else {
                        POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION
                    }
                }
            } else {
                OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION
            });
            let mut messages = if force_context_overflow_final {
                self.build_context_overflow_final_messages(
                    user_input,
                    &tool_calls_made,
                    required_final_answer_prefix.as_deref(),
                    investment_research_started || self.agent_owned_finance_loop,
                )
            } else if active_business_round
                || self.agent_owned_finance_loop
                || read_only_trace_finalization.is_some()
            {
                // Keep only bounded historical user wording for pronoun and
                // follow-up resolution. Old assistant/tool traces never enter
                // a new research ledger, so stale prices or entities cannot be
                // mistaken for current-turn evidence.
                self.build_agent_owned_messages(context, round_instruction, turn_message_start)
            } else {
                self.build_messages(context, round_instruction)
            };
            if active_business_round && !force_context_overflow_final {
                // Keep provider-issued reasoning signatures during live tool
                // follow-up rounds (MiniMax/Mimo compatibility), while
                // removing assistant prose drafts beside tool calls. The
                // shared final contract explicitly excludes hidden reasoning
                // as evidence; explicit terminal synthesis additionally
                // strips it from the transcript altogether.
                scrub_research_evidence_messages(&mut messages, false);
                #[cfg(not(test))]
                let active_turn_prompt = agent_owned_business_turn_prompt(
                    evidence_floor_satisfied,
                    &research_evidence.agent_guidance_summary(),
                    required_final_answer_prefix.as_deref(),
                    force_finance_final,
                    evidence_rescue_due,
                );
                #[cfg(test)]
                let active_turn_prompt = if self.agent_owned_finance_loop && !legacy_finish_terminal
                {
                    agent_owned_business_turn_prompt(
                        evidence_floor_satisfied,
                        &research_evidence.agent_guidance_summary(),
                        required_final_answer_prefix.as_deref(),
                        force_finance_final,
                        evidence_rescue_due,
                    )
                } else {
                    active_business_turn_prompt(
                        evidence_floor_satisfied,
                        &research_evidence.agent_guidance_summary(),
                        &research_source_catalog_prompt(&research_sources),
                        pending_finish_feedback.as_deref(),
                    )
                };
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(active_turn_prompt),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if active_business_round
                && let Some(feedback) = pending_market_move_final_correction.as_deref()
            {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(feedback.to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if active_business_round && let Some(feedback) = pending_gap_closure_feedback.take() {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(feedback),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if let Some(feedback) = pending_listing_final_correction.as_deref() {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(feedback.to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if let Some(feedback) = pending_language_final_correction.as_deref() {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(feedback.to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if force_finance_final && let Some(blocked_call) = blocked_tool_finalization.as_deref()
            {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(format!(
                        "{BLOCKED_TOOL_FINALIZATION_INSTRUCTION}\n内部未完成步骤（不要对用户披露）：{blocked_call}"
                    )),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            if read_only_trace_finalization.is_some()
                && let Some(reason) = read_only_trace_finalization.as_deref()
            {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(format!(
                        "{READ_ONLY_TRACE_FINALIZATION_INSTRUCTION}\n内部收口原因（不要对用户披露）：{reason}"
                    )),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            #[cfg(test)]
            if !active_business_round && let Some(feedback) = pending_finish_feedback.as_deref() {
                messages.push(Message {
                    images: Vec::new(),
                    role: "user".to_string(),
                    content: Some(format!(
                        "【内部工具协议纠正】{feedback} 不要向用户提及这条内部纠正；请继续正常调用可用业务工具，或直接给出非空的自然回答。"
                    )),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            let request_payload = serde_json::json!({
                "messages": messages.clone(),
                "tools": if has_tools
                    || force_finance_final
                    || force_context_overflow_final
                    || force_persistent_mutation_final
                {
                    Some(round_tools.clone())
                } else {
                    None
                },
                "tool_choice_mode": format!("{tool_choice_mode:?}"),
            });
            let call_started = std::time::Instant::now();
            let mut stream_tool_choice = StreamToolChoiceTelemetry::new(tool_choice_mode);
            let mut active_business_outcome = active_business_round.then_some("tools");
            let eligible_active_final_prefix = (active_business_round
                && self.agent_owned_finance_loop
                && (evidence_floor_satisfied
                    || force_finance_final
                    || force_context_overflow_final)
                && tool_choice_mode == ToolChoiceMode::Auto)
                .then_some(required_final_answer_prefix.as_deref())
                .flatten();

            // 如果有工具，使用 chat_with_tools；否则使用 chat
            let result: ChatResponse = if has_tools
                || force_finance_final
                || force_context_overflow_final
                || force_persistent_mutation_final
            {
                if active_business_round {
                    let (active_deadline, active_timeout_error) =
                        active_business_deadline(overall_deadline, self.step_timeout);
                    match tokio::time::timeout_at(
                        active_deadline,
                        self.chat_active_business_tools(
                            &messages,
                            &round_tools,
                            tool_choice_mode,
                            eligible_active_final_prefix,
                            &context.session_id,
                            iterations,
                            &mut stream_tool_choice,
                            precommitted_service_prefix.as_deref(),
                        ),
                    )
                    .await
                    {
                        Ok(Ok(ActiveBusinessStreamOutcome::Tools(response))) => {
                            active_business_failures = 0;
                            response
                        }
                        Ok(Ok(ActiveBusinessStreamOutcome::DirectFinal(mut response))) => {
                            // A complete Stop + Done body is always the same
                            // Agent's natural final answer. The evidence ledger
                            // controls only when the internal finish signal is
                            // offered. A narrow pre-publication market-move
                            // check below verifies only civil-calendar
                            // consistency, requested broad scope, and
                            // current-turn source locality; it never chooses or
                            // rewrites a cause.
                            active_business_failures = 0;
                            active_business_outcome = Some("direct_final");
                            if research_answer_without_evidence(
                                &response.content,
                                &tool_calls_made,
                                self.preloaded_evidence_calls,
                            ) && blocked_tool_finalization.is_none()
                                && research_evidence_retries < MAX_LISTING_FINAL_CORRECTIONS
                                && iterations < self.max_iterations
                            {
                                tracing::warn!(
                                    session_id = %context.session_id,
                                    iteration = iterations,
                                    "research answer published no business evidence; requiring tools"
                                );
                                research_evidence_retries =
                                    research_evidence_retries.saturating_add(1);
                                research_evidence_retry_pending = true;
                                pending_listing_final_correction =
                                    Some(research_evidence_precondition_prompt());
                                continue;
                            }
                            // Gap-closure: a natural final that still self-reports
                            // missing first-party evidence goes back into the tool
                            // loop with a targeted checklist while round budget
                            // remains, instead of publishing "本轮未读到" gaps the
                            // agent never actually chased. Disabled unless the
                            // deployment budget grants gap-closure rounds; the
                            // market-move flow keeps its own deterministic gap
                            // machinery.
                            // Only before any user-visible byte is irreversible: this
                            // draft was already streamed, and re-entering the tool loop
                            // appends the next rounds' narration onto the same visible
                            // body instead of replacing it. A committed canonical prefix
                            // cannot be retracted, so a gap there stays a disclosed gap.
                            let visible_stream_is_retractable =
                                self.stream_observer.as_ref().is_none_or(|observer| {
                                    observer.committed_visible_prefix().is_none()
                                });
                            if gap_closure_rounds < finance_budget.gap_closure_rounds
                                && !is_market_move_final_check_enabled(user_input)
                                && blocked_tool_finalization.is_none()
                                && iterations < self.max_iterations
                                && finance_tool_rounds + 1 < finance_budget.tool_rounds
                                && visible_stream_is_retractable
                                && finance_max_tool_calls
                                    .is_none_or(|cap| total_tool_calls + 4 <= cap)
                            {
                                let gap_lines = research_gap_lines(&response.content);
                                if !gap_lines.is_empty() {
                                    tracing::info!(
                                        session_id = %context.session_id,
                                        iteration = iterations,
                                        gap_closure_rounds,
                                        gaps = gap_lines.len(),
                                        finance_tool_rounds,
                                        total_tool_calls,
                                        "agent-owned final self-reported evidence gaps; granting a gap-closure research round"
                                    );
                                    if let Some(observer) = &self.stream_observer {
                                        observer
                                            .on_reasoning_delta(&format!(
                                                "终稿自检：仍有 {} 项一手证据缺口，重新开放工具进行第 {} 轮针对性补证。\n",
                                                gap_lines.len(),
                                                gap_closure_rounds + 1
                                            ))
                                            .await;
                                    }
                                    // Retract the speculative draft so the next round
                                    // starts from a clean visible stream rather than
                                    // appending to a published one.
                                    self.reset_emitted_content(true).await;
                                    gap_closure_rounds = gap_closure_rounds.saturating_add(1);
                                    pending_gap_closure_feedback = Some(
                                        gap_closure_feedback_prompt(&gap_lines, &response.content),
                                    );
                                    continue;
                                }
                            }
                            let listing_violations = listing_final_violations(
                                user_input,
                                &response.content,
                                &tool_calls_made,
                            );
                            if !listing_violations.is_empty() {
                                tracing::warn!(
                                    session_id = %context.session_id,
                                    iteration = iterations,
                                    violations = %listing_violations.join(" | "),
                                    listing_final_corrections,
                                    "agent-owned listing final contradicted or exceeded current-turn listing evidence"
                                );
                                if listing_final_corrections < MAX_LISTING_FINAL_CORRECTIONS
                                    && iterations < self.max_iterations
                                {
                                    listing_final_corrections =
                                        listing_final_corrections.saturating_add(1);
                                    pending_listing_final_correction =
                                        Some(listing_final_correction_prompt(
                                            &listing_violations,
                                            &response.content,
                                            &tool_calls_made,
                                        ));
                                    continue;
                                }
                                response.content = deterministic_listing_gap_response(
                                    required_final_answer_prefix.as_deref(),
                                    &tool_calls_made,
                                    &listing_violations,
                                );
                            }
                            let violations = market_move_final_violations(
                                user_input,
                                &response.content,
                                context,
                                turn_message_start,
                            );
                            if !violations.is_empty() {
                                let target_date = resolved_market_move_date(
                                    user_input,
                                    &response.content,
                                    context,
                                    turn_message_start,
                                );
                                let eligible_source_urls =
                                    target_date.map_or_else(Vec::new, |date| {
                                        current_turn_source_urls_for_date(
                                            context,
                                            turn_message_start,
                                            date,
                                        )
                                    });
                                let cause_evidence_missing = violations.iter().any(|violation| {
                                    violation
                                        .contains("每个确定性原因段落都必须在同一段内给出目标日期")
                                }) && eligible_source_urls.is_empty();
                                tracing::warn!(
                                    session_id = %context.session_id,
                                    iteration = iterations,
                                    violations = %violations.join(" | "),
                                    market_move_final_corrections,
                                    target_date = ?target_date,
                                    eligible_source_count = eligible_source_urls.len(),
                                    cause_evidence_missing,
                                    "agent-owned market-move final failed mechanical pre-publication checks"
                                );
                                if market_move_final_corrections < MAX_MARKET_MOVE_FINAL_CORRECTIONS
                                    && iterations < self.max_iterations
                                    && !cause_evidence_missing
                                {
                                    if precommitted_service_prefix.is_none()
                                        && let Some(prefix) = self
                                            .stream_observer
                                            .as_ref()
                                            .and_then(|observer| {
                                                observer.committed_visible_prefix()
                                            })
                                            .filter(|prefix| {
                                                committed_prefix_matches_required(
                                                    prefix,
                                                    required_final_answer_prefix.as_deref(),
                                                )
                                            })
                                    {
                                        precommitted_service_prefix = Some(prefix);
                                    }
                                    market_move_final_corrections =
                                        market_move_final_corrections.saturating_add(1);
                                    pending_market_move_final_correction =
                                        Some(market_move_final_correction_prompt_with_sources(
                                            &violations,
                                            &response.content,
                                            target_date,
                                            &eligible_source_urls,
                                        ));
                                    continue;
                                }
                                if let Some(prefix) = required_final_answer_prefix.as_deref() {
                                    response.content = deterministic_market_move_gap_response(
                                        user_input,
                                        &response.content,
                                        prefix,
                                        context,
                                        turn_message_start,
                                    );
                                }
                            }
                            // Language repair runs last so it rewrites the
                            // already-corrected draft rather than racing the
                            // factual correction rounds above.
                            if final_language_mismatch(user_input, &response.content)
                                && language_final_corrections < MAX_LANGUAGE_FINAL_CORRECTIONS
                                && iterations < self.max_iterations
                            {
                                tracing::warn!(
                                    session_id = %context.session_id,
                                    iteration = iterations,
                                    "agent-owned final answered a Chinese question with an English body; granting one language-repair round"
                                );
                                if let Some(observer) = &self.stream_observer {
                                    observer
                                        .on_reasoning_delta(
                                            "终稿自检：正文语言与提问语言不一致，正在整体改写为中文。\n",
                                        )
                                        .await;
                                }
                                language_final_corrections =
                                    language_final_corrections.saturating_add(1);
                                pending_language_final_correction =
                                    Some(language_final_correction_prompt(&response.content));
                                continue;
                            }
                            response
                        }
                        Ok(Ok(ActiveBusinessStreamOutcome::Empty)) => {
                            let error = "active business stream returned no tool call";
                            let retrying =
                                consume_active_business_retry(&mut active_business_failures);
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.to_string()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_outcome": "empty",
                                    "terminal_authorized": false,
                                    "retrying": retrying,
                                    "tool_choice_mode": tool_choice_mode_name(tool_choice_mode),
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            ).await;
                            if retrying {
                                continue;
                            }
                            if self.agent_owned_finance_loop && blocked_tool_finalization.is_none()
                            {
                                blocked_tool_finalization = Some(error.to_string());
                                continue;
                            }
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                        Ok(Err(error)) => {
                            let error = error.to_string();
                            let context_overflow_recovery = is_context_overflow_error(&error)
                                && !context_overflow_finalization
                                && !tool_calls_made.is_empty()
                                && tool_calls_made.iter().all(|call| {
                                    tool_call_is_known_read_only(&call.name, &call.arguments)
                                });
                            let tools_disabled_recovery = !is_context_overflow_error(&error)
                                && self.agent_owned_finance_loop
                                && blocked_tool_finalization.is_none()
                                && !context_overflow_finalization;
                            let retrying = context_overflow_recovery || tools_disabled_recovery;
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.clone()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_outcome": "error",
                                    "terminal_authorized": false,
                                    "retrying": retrying,
                                    "tools_disabled_recovery": tools_disabled_recovery,
                                    "context_overflow_bounded_recovery": context_overflow_recovery,
                                    "tool_choice_mode": tool_choice_mode_name(tool_choice_mode),
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            ).await;
                            self.dbg(&format!(
                                "[Agent] active business stream failed without terminal authorization: {error}"
                            ));
                            if context_overflow_recovery {
                                context_overflow_finalization = true;
                                continue;
                            }
                            if tools_disabled_recovery {
                                blocked_tool_finalization = Some(error);
                                continue;
                            }
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                        Err(_) => {
                            let error = active_timeout_error;
                            let retrying = self.agent_owned_finance_loop
                                && error != AGENT_OVERALL_TIMEOUT_ERROR
                                && blocked_tool_finalization.is_none();
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.to_string()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_timeout": true,
                                    "overall_timeout": error == AGENT_OVERALL_TIMEOUT_ERROR,
                                    "active_business_outcome": "timeout",
                                    "terminal_authorized": false,
                                    "retrying": retrying,
                                    "tools_disabled_recovery": retrying,
                                    "tool_choice_mode": tool_choice_mode_name(tool_choice_mode),
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            ).await;
                            if retrying {
                                blocked_tool_finalization = Some(error.to_string());
                                continue;
                            }
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                    }
                } else {
                    let (initial_deadline, initial_timeout_error) =
                        step_deadline(overall_deadline, self.step_timeout);
                    match await_before_deadline(
                        initial_deadline,
                        initial_timeout_error,
                        self.chat_with_tools_streaming(
                            &messages,
                            &round_tools,
                            tool_choice_mode,
                            true,
                            &mut stream_tool_choice,
                        ),
                    )
                    .await
                    {
                        Ok(response) => response,
                        Err(error) => {
                            let error_text = error.to_string();
                            let context_overflow_recovery = is_context_overflow_error(&error_text)
                                && !context_overflow_finalization
                                && !tool_calls_made.is_empty()
                                && tool_calls_made.iter().all(|call| {
                                    tool_call_is_known_read_only(&call.name, &call.arguments)
                                });
                            let tools_disabled_recovery =
                                is_recoverable_openai_compatible_protocol_error_text(&error_text)
                                    && self.agent_owned_finance_loop
                                    && blocked_tool_finalization.is_none()
                                    && !tool_calls_made.is_empty()
                                    && tool_calls_made.iter().all(|call| {
                                        tool_call_is_known_read_only(&call.name, &call.arguments)
                                    });
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error_text.clone()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "retrying": context_overflow_recovery || tools_disabled_recovery,
                                    "context_overflow_bounded_recovery": context_overflow_recovery,
                                    "tools_disabled_recovery": tools_disabled_recovery,
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            ).await;
                            if context_overflow_recovery {
                                context_overflow_finalization = true;
                                continue;
                            }
                            if tools_disabled_recovery {
                                blocked_tool_finalization = Some(error_text);
                                continue;
                            }
                            return AgentResponse {
                                content: String::new(),
                                tool_calls_made,
                                iterations,
                                success: false,
                                error: Some(error_text),
                            };
                        }
                    }
                }
            } else {
                let (chat_deadline, chat_timeout_error) =
                    step_deadline(overall_deadline, self.step_timeout);
                match await_before_deadline(
                    chat_deadline,
                    chat_timeout_error,
                    self.llm.chat(&messages, None),
                )
                .await
                {
                    Ok(r) => ChatResponse {
                        content: r.content,
                        reasoning_content: None,
                        tool_calls: None,
                        usage: r.usage,
                    },
                    Err(e) => {
                        let error_text = e.to_string();
                        let context_overflow_recovery = is_context_overflow_error(&error_text)
                            && !context_overflow_finalization
                            && !tool_calls_made.is_empty()
                            && tool_calls_made.iter().all(|call| {
                                tool_call_is_known_read_only(&call.name, &call.arguments)
                            });
                        self.record_audit(
                            context,
                            "chat",
                            request_payload,
                            None,
                            Some(error_text.clone()),
                            call_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "retrying": context_overflow_recovery,
                                "context_overflow_bounded_recovery": context_overflow_recovery,
                            }),
                            None,
                        )
                        .await;
                        if context_overflow_recovery {
                            context_overflow_finalization = true;
                            continue;
                        }
                        return AgentResponse {
                            content: String::new(),
                            tool_calls_made,
                            iterations,
                            success: false,
                            error: Some(error_text),
                        };
                    }
                }
            };

            #[cfg(test)]
            let mut audit_metadata = serde_json::json!({
                "iteration": iterations,
                "has_tools": has_tools,
                "active_business_outcome": active_business_outcome,
                "evidence_floor_satisfied": evidence_floor_satisfied,
                "identity_only_attempts": research_evidence.identity_only_attempts,
                "broad_data_attempts": research_evidence.broad_data_attempts,
                "symbol_scoped_attempts": research_evidence.symbol_scoped_attempts,
                "broad_market_mode": research_evidence.broad_market_mode,
                "broad_market_quote_groups": research_evidence.broad_market_quote_groups.iter().cloned().collect::<Vec<_>>(),
                "post_activation_attempts": research_evidence.post_activation_attempts,
                "post_identity_attempts": research_evidence.post_identity_attempts,
                "post_identity_quote_attempts": research_evidence.post_identity_quote_attempts,
                "post_identity_asset_route_attempts": research_evidence.post_identity_asset_route_attempts,
                "identity_route_count": research_evidence.identity_routes.len(),
                "active_identity_route_count": research_evidence.active_route_keys().len(),
                "explicit_identity_route_count": research_evidence.identity_routes.values().filter(|route| route.explicit).count(),
                "unscoped_identity_search_attempts": research_evidence.unscoped_identity_search_attempts,
                "unresolved_identity_route_count": research_evidence.active_route_keys().iter().filter(|key| research_evidence.identity_routes.get(*key).is_some_and(|route| route.candidates.is_empty())).count(),
                "finance_tool_rounds": finance_tool_rounds,
                "finance_identity_rounds": finance_identity_rounds,
                "evidence_rescue_rounds": evidence_rescue_rounds,
                "force_finance_final": force_finance_final,
                "force_context_overflow_final": force_context_overflow_final,
                "tool_budget_exhausted": tool_budget_exhausted,
                "identity_route_limit": research_evidence.identity_route_limit,
                "rejected_identity_routes": research_evidence.rejected_identity_routes,
                "requested_tool_choice": has_tools.then_some(tool_choice_mode_name(stream_tool_choice.requested)),
                "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                "tool_choice_fallback": stream_tool_choice.fallback,
            });
            #[cfg(not(test))]
            let audit_metadata = serde_json::json!({
                "iteration": iterations,
                "has_tools": has_tools,
                "active_business_outcome": active_business_outcome,
                "evidence_floor_satisfied": evidence_floor_satisfied,
                "identity_only_attempts": research_evidence.identity_only_attempts,
                "broad_data_attempts": research_evidence.broad_data_attempts,
                "symbol_scoped_attempts": research_evidence.symbol_scoped_attempts,
                "broad_market_mode": research_evidence.broad_market_mode,
                "broad_market_quote_groups": research_evidence.broad_market_quote_groups.iter().cloned().collect::<Vec<_>>(),
                "post_activation_attempts": research_evidence.post_activation_attempts,
                "post_identity_attempts": research_evidence.post_identity_attempts,
                "post_identity_quote_attempts": research_evidence.post_identity_quote_attempts,
                "post_identity_asset_route_attempts": research_evidence.post_identity_asset_route_attempts,
                "identity_route_count": research_evidence.identity_routes.len(),
                "active_identity_route_count": research_evidence.active_route_keys().len(),
                "explicit_identity_route_count": research_evidence.identity_routes.values().filter(|route| route.explicit).count(),
                "unscoped_identity_search_attempts": research_evidence.unscoped_identity_search_attempts,
                "unresolved_identity_route_count": research_evidence.active_route_keys().iter().filter(|key| research_evidence.identity_routes.get(*key).is_some_and(|route| route.candidates.is_empty())).count(),
                "finance_tool_rounds": finance_tool_rounds,
                "finance_identity_rounds": finance_identity_rounds,
                "evidence_rescue_rounds": evidence_rescue_rounds,
                "force_finance_final": force_finance_final,
                "force_context_overflow_final": force_context_overflow_final,
                "tool_budget_exhausted": tool_budget_exhausted,
                "identity_route_limit": research_evidence.identity_route_limit,
                "rejected_identity_routes": research_evidence.rejected_identity_routes,
                "requested_tool_choice": has_tools.then_some(tool_choice_mode_name(stream_tool_choice.requested)),
                "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                "tool_choice_fallback": stream_tool_choice.fallback,
            });
            #[cfg(test)]
            if let Some(object) = audit_metadata.as_object_mut() {
                object.insert(
                    "finish_research_available".to_string(),
                    Value::Bool(finish_research_available),
                );
            }
            self.record_audit(
                context,
                if has_tools || force_finance_final || force_context_overflow_final {
                    "chat_with_tools"
                } else {
                    "chat"
                },
                request_payload,
                Some(serde_json::json!({
                    "content": result.content.clone(),
                    "tool_calls": result.tool_calls.clone()
                })),
                None,
                call_started.elapsed().as_millis(),
                audit_metadata,
                result.usage.clone(),
            )
            .await;

            // 检查是否有工具调用
            if let Some(ref tcs) = result.tool_calls {
                let tcs: &Vec<hone_llm::ToolCall> = tcs;
                if !tcs.is_empty() {
                    self.dbg(&format!("[Agent] tool_calls n={}", tcs.len()));

                    if force_finance_final
                        || force_context_overflow_final
                        || force_persistent_mutation_final
                    {
                        return failed_agent_response(
                            tool_calls_made,
                            iterations,
                            AGENT_OWNED_FINANCE_FORCED_FINAL_TOOL_ERROR,
                        );
                    }

                    // The provider may finish the exact canonical first line
                    // and then decide that one more read-only lookup is needed
                    // in the same stream. Treat that line as an irreversible
                    // service boundary, not as a terminal decision. The full
                    // completed tool batch is still validated below before any
                    // assistant frame, observer notification, or execution.
                    let visible_service_prefix_committed = precommitted_service_prefix.is_some()
                        || self
                            .stream_observer
                            .as_ref()
                            .and_then(|observer| observer.committed_visible_prefix())
                            .is_some_and(|prefix| {
                                committed_prefix_matches_required(
                                    &prefix,
                                    required_final_answer_prefix.as_deref(),
                                )
                            });

                    #[cfg(not(test))]
                    let actionable_tool_calls = tcs.iter().collect::<Vec<_>>();
                    #[cfg(test)]
                    let actionable_tool_calls = tcs
                        .iter()
                        .filter(|tool_call| {
                            !legacy_finish_terminal || !is_finish_research_call(tool_call)
                        })
                        .collect::<Vec<_>>();
                    #[cfg(test)]
                    let finish_calls = if legacy_finish_terminal {
                        tcs.iter()
                            .filter(|tool_call| is_finish_research_call(tool_call))
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };

                    let round_starts_investment_research =
                        actionable_tool_calls.iter().any(|tool_call| {
                            registered_tool_names.contains(&tool_call.function.name)
                                && starts_investment_research_protocol(tool_call)
                        });
                    if self.agent_owned_finance_loop
                        && (visible_service_prefix_committed
                            || investment_research_started
                            || round_starts_investment_research)
                    {
                        // A round that only asks "which security is this?" is
                        // not research into the user's question. Charging it to
                        // the research budget is what lets one unresolvable or
                        // phantom identifier consume the whole turn and reach
                        // the forced final with nothing to answer from.
                        let round_is_identity_only = !actionable_tool_calls.is_empty()
                            && actionable_tool_calls
                                .iter()
                                .all(|tool_call| is_identity_only_search_call(tool_call));
                        if round_is_identity_only
                            && finance_identity_rounds < MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUNDS
                        {
                            finance_identity_rounds = finance_identity_rounds.saturating_add(1);
                        } else {
                            finance_tool_rounds = finance_tool_rounds.saturating_add(1);
                        }
                    }

                    // Once the service date anchor or DataFetch has made this
                    // a finance-research loop, it is read-only. Reject
                    // write-capable calls before adding a
                    // tool trace, notifying observers, or entering the
                    // registry. An initial non-finance portfolio/cron request
                    // remains unaffected until the DataFetch boundary exists.
                    let finance_round_is_read_only = self.agent_owned_finance_loop
                        && (visible_service_prefix_committed
                            || investment_research_started
                            || round_starts_investment_research);
                    let finance_round_is_known_read_only =
                        actionable_tool_calls.iter().all(|tool_call| {
                            registered_tool_call_is_known_read_only(
                                tool_call,
                                &registered_tool_names,
                            )
                        });
                    if finance_round_is_read_only && !finance_round_is_known_read_only {
                        service_prefix_commit_eligible = false;
                    }

                    // A finance/read-only turn must never execute an
                    // unregistered, unknown-effect, or write-capable batch.
                    // A registered, known-read-only batch is handled per call
                    // below: malformed siblings receive a structured rejection
                    // while valid searches and evidence reads still execute.
                    // Blocking an unsafe batch is not authority to refuse the
                    // user's business question, so the same Agent receives one
                    // tools-disabled continuation from evidence already held.
                    if finance_round_is_read_only && !finance_round_is_known_read_only {
                        let blocked_calls = actionable_tool_calls
                            .iter()
                            .filter(|tool_call| {
                                !registered_read_only_tool_call_is_well_formed(
                                    tool_call,
                                    &registered_tool_names,
                                )
                            })
                            .map(|tool_call| tool_call.function.name.as_str())
                            .collect::<Vec<_>>()
                            .join(",");
                        tracing::warn!(
                            target: "hone_agent::ttft",
                            session_id = %context.session_id,
                            iteration = iterations,
                            blocked_tools = %blocked_calls,
                            visible_service_prefix_committed,
                            "agent-owned finance blocked unsafe tool batch before execution and will continue with a tools-disabled answer"
                        );
                        blocked_tool_finalization = Some(if blocked_calls.is_empty() {
                            "unknown".to_string()
                        } else {
                            blocked_calls
                        });
                        continue;
                    }

                    // A finish-only round can enter the terminal even when a
                    // compatible provider duplicates the internal control
                    // call. Select the first parseable handoff; if none parse,
                    // recover only the Agent-selected current-turn locations.
                    // An unresolvable locator stays inside this same Agent loop
                    // for one bounded protocol correction; it never authorizes
                    // an unrestricted empty-evidence terminal.
                    #[cfg(test)]
                    if finish_research_available
                        && actionable_tool_calls.is_empty()
                        && !finish_calls.is_empty()
                    {
                        let mut parse_warnings = Vec::new();
                        let parsed_handoff = finish_calls.iter().find_map(|finish_call| {
                            match parse_finish_research_handoff(finish_call) {
                                Ok(handoff) => Some(handoff),
                                Err(warning) => {
                                    push_validation_warning(&mut parse_warnings, warning);
                                    None
                                }
                            }
                        });
                        let mut handoff = match parsed_handoff {
                            Some(handoff) => validate_finish_research_handoff(
                                handoff,
                                context,
                                turn_message_start,
                            ),
                            None => {
                                let warning = if parse_warnings.is_empty() {
                                    "finish_research handoff was unavailable".to_string()
                                } else {
                                    parse_warnings.join("; ")
                                };
                                let fallback_scope =
                                    fallback_scope_from_finish_calls(&finish_calls);
                                tracing::warn!(
                                    session_id = %context.session_id,
                                    selected_fallback_sources = fallback_scope
                                        .web_result_numbers
                                        .len()
                                        .saturating_add(fallback_scope.data_json_pointers.len()),
                                    "finish_research handoff could not be parsed; using only mechanically recoverable referenced evidence: {warning}"
                                );
                                fallback_research_handoff(
                                    context,
                                    turn_message_start,
                                    &fallback_scope,
                                    warning,
                                )
                            }
                        };
                        if finish_calls.len() > 1 {
                            push_validation_warning(
                                &mut handoff.validation_warnings,
                                format!(
                                    "provider emitted {} finish_research calls; used the first parseable handoff",
                                    finish_calls.len()
                                ),
                            );
                        }
                        for warning in parse_warnings {
                            push_validation_warning(&mut handoff.validation_warnings, warning);
                        }
                        let finish_evidence_empty =
                            handoff.facts.is_empty() && handoff.fallback_evidence.is_empty();
                        let needs_evidence_correction = !research_sources.is_empty()
                            && (handoff.unresolved_reference_count > 0 || finish_evidence_empty);
                        if needs_evidence_correction
                            && invalid_finish_corrections < MAX_INVALID_FINISH_CORRECTIONS
                        {
                            invalid_finish_corrections =
                                invalid_finish_corrections.saturating_add(1);
                            let locator_issue = if handoff.unresolved_reference_count > 0 {
                                format!(
                                    "上一次交接有 {} 条 evidence locator 无法解析",
                                    handoff.unresolved_reference_count
                                )
                            } else {
                                "上一次交接没有形成任何可解析的 fact evidence".to_string()
                            };
                            pending_finish_feedback = Some(format!(
                                "{locator_issue}。只修正来源地址，不重做答案：本轮已有可引用来源，至少把与回答有关的来源放入 facts；逐字复制本轮来源目录中的完整 tool_call_id；Web 仅填写 result_number + exact_excerpt，DataFetch 仅填写 json_pointer，二者不得混填。无法核验的维度仍写入 gaps。",
                            ));
                            tracing::warn!(
                                session_id = %context.session_id,
                                unresolved_references = handoff.unresolved_reference_count,
                                "finish_research lacks resolvable evidence and requires one bounded same-Agent correction"
                            );
                            continue;
                        }
                        if finish_evidence_empty
                            && (handoff.unresolved_reference_count > 0
                                || !research_sources.is_empty())
                        {
                            return failed_agent_response(
                                tool_calls_made,
                                iterations,
                                "finish_research_evidence_locators_repeatedly_unresolvable",
                            );
                        }
                        if !handoff.validation_warnings.is_empty() {
                            tracing::warn!(
                                session_id = %context.session_id,
                                dropped_handoff_items = handoff.validation_warnings.len(),
                                "finish_research handoff contained unresolved items; continuing with valid evidence"
                            );
                        }
                        return self
                            .run_terminal_synthesis(
                                context,
                                tool_calls_made,
                                iterations,
                                turn_message_start,
                                &handoff,
                                required_final_answer_prefix.as_deref(),
                                overall_deadline,
                            )
                            .await;
                    }

                    // A mixed finish never substitutes for real business calls.
                    // An unavailable finish-only hallucination gets one hidden,
                    // bounded correction and can never become an empty success.
                    // The internal signal does not consume tool budget or notify
                    // business-tool observers.

                    // Every nonempty Interactive turn enters the open Agent
                    // discovery path, including non-finance questions that may
                    // use Web/file/skill tools. A service-owned market-move
                    // date anchor activates finance from the first round;
                    // other security turns still activate only at the
                    // structural DataFetch boundary.
                    if actionable_tool_calls.is_empty() {
                        #[cfg(test)]
                        {
                            self.dbg("[Agent] ignored malformed or unavailable finish signal");
                            if unavailable_finish_corrections < MAX_UNAVAILABLE_FINISH_CORRECTIONS {
                                unavailable_finish_corrections =
                                    unavailable_finish_corrections.saturating_add(1);
                                pending_finish_feedback = Some(
                                    "finish_research 当前尚不可用；先完成所需真实工具调用。"
                                        .to_string(),
                                );
                                continue;
                            }
                            return failed_agent_response(
                                tool_calls_made,
                                iterations,
                                "unavailable_finish_research_repeated",
                            );
                        }
                        #[cfg(not(test))]
                        unreachable!("a nonempty provider tool-call list must remain actionable");
                    } else {
                        #[cfg(test)]
                        {
                            pending_finish_feedback = None;
                            unavailable_finish_corrections = 0;
                            invalid_finish_corrections = 0;
                        }
                        // 记录 assistant 消息（只含真实业务 tool_calls）
                        let tc_values: Vec<Value> = actionable_tool_calls
                            .iter()
                            .filter_map(|tc| serde_json::to_value(*tc).ok())
                            .collect();
                        let metadata = result.reasoning_content.as_ref().map(|reasoning| {
                            std::collections::HashMap::from([(
                                REASONING_CONTENT_METADATA_KEY.to_string(),
                                Value::String(reasoning.clone()),
                            )])
                        });
                        #[cfg(not(test))]
                        let finance_round_owns_tool_content = self.agent_owned_finance_loop;
                        #[cfg(test)]
                        let finance_round_owns_tool_content =
                            self.agent_owned_finance_loop || legacy_finish_terminal;
                        let assistant_tool_content =
                            if finance_round_owns_tool_content && finance_round_is_read_only {
                                ""
                            } else {
                                &result.content
                            };
                        context.add_assistant_message_with_metadata(
                            assistant_tool_content,
                            Some(tc_values),
                            metadata,
                        );

                        // 逐个执行真实业务工具
                        for tc in actionable_tool_calls {
                            let tool_name = &tc.function.name;
                            let tool_call_id = &tc.id;
                            let tool_args_str = &tc.function.arguments;
                            let notify_tool_observer = registered_tool_names.contains(tool_name);

                            match serde_json::from_str::<Value>(tool_args_str) {
                                Ok(tool_args) => {
                                    self.dbg(&format!("[Agent] tool_call name={tool_name}"));
                                    if finance_round_is_read_only
                                        && !registered_read_only_tool_call_is_well_formed(
                                            tc,
                                            &registered_tool_names,
                                        )
                                    {
                                        let error_result = malformed_read_only_tool_result(tc);
                                        tracing::warn!(
                                            session_id = %context.session_id,
                                            iteration = iterations,
                                            tool = %tool_name,
                                            "rejected one malformed read-only finance call while preserving the valid batch siblings"
                                        );
                                        tool_calls_made.push(ToolCallMade {
                                            name: tool_name.clone(),
                                            arguments: tool_args,
                                            result: error_result.clone(),
                                            tool_call_id: Some(tool_call_id.clone()),
                                        });
                                        context.add_tool_result(
                                            tool_call_id,
                                            tool_name,
                                            &serde_json::to_string(&error_result)
                                                .unwrap_or_default(),
                                        );
                                        continue;
                                    }
                                    let (round_max_tool_calls, effective_tool_call_limits) =
                                        if finance_round_is_read_only {
                                            (finance_max_tool_calls, &finance_tool_call_limits)
                                        } else {
                                            (self.max_tool_calls, &self.tool_call_limits)
                                        };
                                    let effective_max_tool_calls = effective_global_tool_budget(
                                        tool_name,
                                        round_max_tool_calls,
                                        finance_round_is_read_only,
                                    );
                                    if let Some(error_result) = tool_budget_error(
                                        tool_name,
                                        effective_max_tool_calls,
                                        effective_tool_call_limits,
                                        total_tool_calls,
                                        &tool_call_counts,
                                    ) {
                                        // Reaching a configured budget is the
                                        // designed stop, not a fault: the model
                                        // gets the error and finalizes.
                                        tracing::debug!(
                                            session_id = %context.session_id,
                                            iteration = iterations,
                                            tool = tool_name,
                                            total_tool_calls,
                                            detail = %error_result
                                                .get("error")
                                                .and_then(|value| value.as_str())
                                                .unwrap_or_default(),
                                            "function_calling tool call rejected by budget"
                                        );
                                        // Reserved open-research slots are not
                                        // exhaustion: the turn can still search
                                        // the user's actual question, so the
                                        // forced final must not trigger yet.
                                        tool_budget_exhausted |= tool_budget_error(
                                            AGENT_OWNED_OPEN_RESEARCH_TOOL,
                                            round_max_tool_calls,
                                            effective_tool_call_limits,
                                            total_tool_calls,
                                            &tool_call_counts,
                                        )
                                        .is_some();
                                        let result_str = serde_json::to_string(&error_result)
                                            .unwrap_or_default();
                                        context.add_tool_result(
                                            tool_call_id,
                                            tool_name,
                                            &result_str,
                                        );
                                        continue;
                                    }
                                    if self.agent_owned_finance_loop
                                        && is_identity_only_search_call(tc)
                                        && let Some(error_result) =
                                            research_evidence.identity_search_admission_error(tc)
                                    {
                                        let result_str = serde_json::to_string(&error_result)
                                            .unwrap_or_default();
                                        context.add_tool_result(
                                            tool_call_id,
                                            tool_name,
                                            &result_str,
                                        );
                                        continue;
                                    }
                                    total_tool_calls += 1;
                                    *tool_call_counts.entry(tool_name.clone()).or_insert(0) += 1;
                                    if notify_tool_observer && !is_identity_only_search_call(tc) {
                                        substantive_business_calls =
                                            substantive_business_calls.saturating_add(1);
                                    }
                                    if notify_tool_observer
                                        && starts_investment_research_protocol(tc)
                                    {
                                        // Activate only at the same boundary as
                                        // a syntactically valid, budget-accepted
                                        // registry attempt. A malformed or
                                        // rejected DataFetch must not trap an
                                        // ordinary turn in the finance loop.
                                        investment_research_started = true;
                                    }
                                    if investment_research_started {
                                        // Count only a real registry attempt:
                                        // malformed arguments and calls
                                        // rejected by the request-local tool
                                        // budget cannot satisfy the evidence
                                        // floor. Provider no-coverage/errors
                                        // still count because execution below
                                        // is genuinely attempted and the
                                        // Agent can disclose that gap.
                                        research_evidence
                                            .observe_business_call(tc, active_business_round);
                                    }
                                    if notify_tool_observer
                                        && let Some(observer) = &self.tool_observer
                                    {
                                        let (observer_deadline, observer_timeout_error) =
                                            step_deadline(overall_deadline, self.step_timeout);
                                        if let Err(error) = await_unit_before_deadline(
                                            observer_deadline,
                                            observer_timeout_error,
                                            observer.on_tool_start(tool_name, &tool_args, None),
                                        )
                                        .await
                                        {
                                            return failed_agent_response(
                                                tool_calls_made,
                                                iterations,
                                                canonical_agent_timeout(&error)
                                                    .unwrap_or(observer_timeout_error),
                                            );
                                        }
                                    }

                                    let (tool_deadline, tool_timeout_error) =
                                        step_deadline(overall_deadline, self.step_timeout);
                                    match await_before_deadline(
                                        tool_deadline,
                                        tool_timeout_error,
                                        self.tools.execute_tool(tool_name, tool_args.clone()),
                                    )
                                    .await
                                    {
                                        Ok(tool_result) => {
                                            self.dbg(&format!(
                                                "[Agent] tool_result name={tool_name}"
                                            ));

                                            if investment_research_started {
                                                research_evidence.observe_business_result(
                                                    tc,
                                                    &tool_result,
                                                    active_business_round,
                                                );
                                            }

                                            let tr: Value = tool_result.clone();
                                            tool_calls_made.push(ToolCallMade {
                                                name: tool_name.clone(),
                                                arguments: tool_args.clone(),
                                                result: tr,
                                                tool_call_id: Some(tool_call_id.clone()),
                                            });

                                            let result_str = serde_json::to_string(&tool_result)
                                                .unwrap_or_default();
                                            context.add_tool_result(
                                                tool_call_id,
                                                tool_name,
                                                &result_str,
                                            );
                                            if notify_tool_observer
                                                && let Some(observer) = &self.tool_observer
                                            {
                                                let (observer_deadline, observer_timeout_error) =
                                                    step_deadline(
                                                        overall_deadline,
                                                        self.step_timeout,
                                                    );
                                                if let Err(error) = await_unit_before_deadline(
                                                    observer_deadline,
                                                    observer_timeout_error,
                                                    observer.on_tool_finish(
                                                        tool_name, &tool_args, true,
                                                    ),
                                                )
                                                .await
                                                {
                                                    return failed_agent_response(
                                                        tool_calls_made,
                                                        iterations,
                                                        canonical_agent_timeout(&error)
                                                            .unwrap_or(observer_timeout_error),
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            self.dbg(&format!(
                                                "[Agent] tool_error name={tool_name} error={e}"
                                            ));
                                            if investment_research_started {
                                                research_evidence.observe_business_failure(tc);
                                            }
                                            let err_str = e.to_string();
                                            let timeout_error = canonical_agent_timeout(&e);
                                            let error_result: Value = serde_json::json!({
                                                "error": err_str,
                                                "status": "failed",
                                                "isError": true,
                                                "timeout": timeout_error.is_some(),
                                            });
                                            tool_calls_made.push(ToolCallMade {
                                                name: tool_name.clone(),
                                                arguments: tool_args.clone(),
                                                result: error_result.clone(),
                                                tool_call_id: Some(tool_call_id.clone()),
                                            });
                                            let result_str = serde_json::to_string(&error_result)
                                                .unwrap_or_default();
                                            context.add_tool_result(
                                                tool_call_id,
                                                tool_name,
                                                &result_str,
                                            );
                                            if notify_tool_observer
                                                && let Some(observer) = &self.tool_observer
                                            {
                                                let (observer_deadline, observer_timeout_error) =
                                                    step_deadline(
                                                        overall_deadline,
                                                        self.step_timeout,
                                                    );
                                                if let Err(error) = await_unit_before_deadline(
                                                    observer_deadline,
                                                    observer_timeout_error,
                                                    observer.on_tool_finish(
                                                        tool_name, &tool_args, false,
                                                    ),
                                                )
                                                .await
                                                {
                                                    return failed_agent_response(
                                                        tool_calls_made,
                                                        iterations,
                                                        canonical_agent_timeout(&error)
                                                            .unwrap_or(observer_timeout_error),
                                                    );
                                                }
                                            }
                                            if timeout_error.is_none()
                                                && let Some((read_tool_name, read_args)) =
                                                    persistent_tool_reconciliation_call(
                                                        tool_name, &tool_args,
                                                    )
                                            {
                                                // The write may have committed
                                                // before the error became
                                                // visible. Never replay it.
                                                // Instead issue the central
                                                // actor-scoped read contract,
                                                // record the observed state,
                                                // and give the same Agent one
                                                // tools-disabled final turn.
                                                let read_call_id =
                                                    format!("{tool_call_id}__read_after_write");
                                                let read_call = ToolCall {
                                                    id: read_call_id.clone(),
                                                    call_type: "function".to_string(),
                                                    function: FunctionCall {
                                                        name: read_tool_name.to_string(),
                                                        arguments: serde_json::to_string(
                                                            &read_args,
                                                        )
                                                        .unwrap_or_else(|_| "{}".to_string()),
                                                    },
                                                };
                                                context.add_assistant_message(
                                                    "",
                                                    Some(vec![
                                                        serde_json::to_value(&read_call).expect(
                                                            "serialize reconciliation read",
                                                        ),
                                                    ]),
                                                );
                                                if let Some(observer) = &self.tool_observer {
                                                    let (observer_deadline, observer_timeout_error) =
                                                        step_deadline(
                                                            overall_deadline,
                                                            self.step_timeout,
                                                        );
                                                    if let Err(error) = await_unit_before_deadline(
                                                        observer_deadline,
                                                        observer_timeout_error,
                                                        observer.on_tool_start(
                                                            read_tool_name,
                                                            &read_args,
                                                            None,
                                                        ),
                                                    )
                                                    .await
                                                    {
                                                        return failed_agent_response(
                                                            tool_calls_made,
                                                            iterations,
                                                            canonical_agent_timeout(&error)
                                                                .unwrap_or(observer_timeout_error),
                                                        );
                                                    }
                                                }
                                                let (read_deadline, read_timeout_error) =
                                                    step_deadline(
                                                        overall_deadline,
                                                        self.step_timeout,
                                                    );
                                                match await_before_deadline(
                                                    read_deadline,
                                                    read_timeout_error,
                                                    self.tools.execute_tool(
                                                        read_tool_name,
                                                        read_args.clone(),
                                                    ),
                                                )
                                                .await
                                                {
                                                    Ok(observed_state) => {
                                                        let read_result = serde_json::json!({
                                                            "status": "observed_after_ambiguous_write",
                                                            "mutation_replayed": false,
                                                            "observed_state": observed_state,
                                                        });
                                                        tool_calls_made.push(ToolCallMade {
                                                            name: read_tool_name.to_string(),
                                                            arguments: read_args.clone(),
                                                            result: read_result.clone(),
                                                            tool_call_id: Some(
                                                                read_call_id.clone(),
                                                            ),
                                                        });
                                                        context.add_tool_result(
                                                            &read_call_id,
                                                            read_tool_name,
                                                            &serde_json::to_string(&read_result)
                                                                .unwrap_or_default(),
                                                        );
                                                        if let Some(observer) = &self.tool_observer
                                                        {
                                                            let (
                                                                observer_deadline,
                                                                observer_timeout_error,
                                                            ) = step_deadline(
                                                                overall_deadline,
                                                                self.step_timeout,
                                                            );
                                                            if let Err(error) =
                                                                await_unit_before_deadline(
                                                                    observer_deadline,
                                                                    observer_timeout_error,
                                                                    observer.on_tool_finish(
                                                                        read_tool_name,
                                                                        &read_args,
                                                                        true,
                                                                    ),
                                                                )
                                                                .await
                                                            {
                                                                return failed_agent_response(
                                                                    tool_calls_made,
                                                                    iterations,
                                                                    canonical_agent_timeout(&error)
                                                                        .unwrap_or(
                                                                            observer_timeout_error,
                                                                        ),
                                                                );
                                                            }
                                                        }
                                                        total_tool_calls =
                                                            total_tool_calls.saturating_add(1);
                                                        *tool_call_counts
                                                            .entry(read_tool_name.to_string())
                                                            .or_insert(0) += 1;
                                                        persistent_mutation_finalization = true;
                                                        break;
                                                    }
                                                    Err(read_error) => {
                                                        return failed_agent_response(
                                                            tool_calls_made,
                                                            iterations,
                                                            canonical_agent_timeout(&read_error)
                                                                .or(timeout_error)
                                                                .unwrap_or(
                                                                    "persistent_tool_failure: read-after-write reconciliation failed",
                                                                ),
                                                        );
                                                    }
                                                }
                                            }
                                            if let Some(timeout_error) = timeout_error {
                                                return failed_agent_response(
                                                    tool_calls_made,
                                                    iterations,
                                                    timeout_error,
                                                );
                                            }
                                            if tool_call_has_persistent_side_effect(
                                                tool_name, &tool_args,
                                            ) {
                                                return failed_agent_response(
                                                    tool_calls_made,
                                                    iterations,
                                                    "persistent_tool_failure: no safe read-after-write reconciliation is available",
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.dbg(&format!(
                                        "[Agent] json parse error for {tool_name}: {e}"
                                    ));
                                    let err_str = format!("参数解析失败: {e}");
                                    let error_result: Value = serde_json::json!({"error": err_str});
                                    let result_str =
                                        serde_json::to_string(&error_result).unwrap_or_default();
                                    context.add_tool_result(tool_call_id, tool_name, &result_str);
                                }
                            }
                        }
                        if persistent_mutation_finalization {
                            continue;
                        }
                        if finance_round_is_read_only
                            && investment_research_started
                            && service_prefix_commit_eligible
                            && self.allow_pre_final_prefix_commit
                            && precommitted_service_prefix.is_none()
                            && let (Some(prefix), Some(observer)) = (
                                self.service_owned_initial_prefix.as_deref(),
                                self.stream_observer.as_ref(),
                            )
                        {
                            let already_committed = observer
                                .committed_visible_prefix()
                                .as_deref()
                                .is_some_and(|committed| committed == prefix);
                            if already_committed
                                || (observer.commit_service_owned_prefix(prefix).await
                                    && observer.committed_visible_prefix().as_deref()
                                        == Some(prefix))
                            {
                                precommitted_service_prefix = Some(prefix.to_string());
                            }
                        }
                        // 继续循环 — 把真实工具结果送回 LLM
                        continue;
                    }
                }
            }

            // Before finance research starts, preserve ordinary direct answers.
            // After the structural evidence floor, a complete Stop + Done body
            // is likewise the same Agent's natural final answer and is not sent
            // through another terminal generation or a service semantic gate.
            self.dbg("[Agent] done (no more tool_calls)");
            if research_answer_without_evidence(
                &result.content,
                &tool_calls_made,
                self.preloaded_evidence_calls,
            ) && blocked_tool_finalization.is_none()
                && research_evidence_retries < MAX_LISTING_FINAL_CORRECTIONS
                && iterations < self.max_iterations
            {
                tracing::warn!(
                    session_id = %context.session_id,
                    iteration = iterations,
                    "research answer published no business evidence; requiring tools"
                );
                research_evidence_retries = research_evidence_retries.saturating_add(1);
                research_evidence_retry_pending = true;
                pending_listing_final_correction = Some(research_evidence_precondition_prompt());
                continue;
            }
            let listing_violations =
                listing_final_violations(user_input, &result.content, &tool_calls_made);
            if !listing_violations.is_empty() {
                tracing::warn!(
                    session_id = %context.session_id,
                    iteration = iterations,
                    violations = %listing_violations.join(" | "),
                    listing_final_corrections,
                    "listing final contradicted or exceeded current-turn listing evidence"
                );
                if listing_final_corrections < MAX_LISTING_FINAL_CORRECTIONS
                    && iterations < self.max_iterations
                {
                    listing_final_corrections = listing_final_corrections.saturating_add(1);
                    pending_listing_final_correction = Some(listing_final_correction_prompt(
                        &listing_violations,
                        &result.content,
                        &tool_calls_made,
                    ));
                    continue;
                }
                let content = deterministic_listing_gap_response(
                    required_final_answer_prefix.as_deref(),
                    &tool_calls_made,
                    &listing_violations,
                );
                context.add_assistant_message_with_metadata(&content, None, None);
                return AgentResponse {
                    content,
                    tool_calls_made,
                    iterations,
                    success: true,
                    error: None,
                };
            }
            let metadata = if active_business_round {
                None
            } else {
                result.reasoning_content.as_ref().map(|reasoning| {
                    std::collections::HashMap::from([(
                        REASONING_CONTENT_METADATA_KEY.to_string(),
                        Value::String(reasoning.clone()),
                    )])
                })
            };
            context.add_assistant_message_with_metadata(&result.content, None, metadata);
            return AgentResponse {
                content: result.content,
                tool_calls_made,
                iterations,
                success: true,
                error: None,
            };
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn final_language_mismatch_flags_english_body_for_chinese_question() {
        let question = "mrvl昨晚财报后为什么跌";
        // The audited production shape: injected Chinese first line, then a
        // long English report with a couple of Chinese marker words.
        let english_body = format!(
            "数据时间：运行时时区 2026-08-28 09:02；行情口径：本轮仅使用可核验资料\n\n### Conclusion\n{}\n*推断：* priced-for-perfection expectations.",
            "Marvell delivered a fundamentally robust quarter with record revenue and accelerating data center momentum. ".repeat(12)
        );
        assert!(super::final_language_mismatch(question, &english_body));

        // A normal Chinese finance answer stays untouched even when dense
        // with tickers, numbers, and table markup.
        let chinese_body = format!(
            "数据时间：运行时时区 2026-08-28 09:02；行情口径：最新可得\n\n### 结论\n{}\n| MRVL | $222.62 | -7.80% |",
            "美满电子本季度数据中心收入同比增长46%，指引超一致预期，盘后下跌主要来自高估值下的获利了结与情绪回落。".repeat(10)
        );
        assert!(!super::final_language_mismatch(question, &chinese_body));

        // English question keeps English answers.
        assert!(!super::final_language_mismatch(
            "why did mrvl drop after earnings",
            &english_body
        ));

        // Short English fragments never trigger a rewrite.
        assert!(!super::final_language_mismatch(
            question,
            "Double beat; guidance raised."
        ));
    }

    #[test]
    fn language_final_correction_prompt_carries_draft_and_rules() {
        let prompt = super::language_final_correction_prompt("### Conclusion\nEnglish body");
        assert!(prompt.contains("内部终稿语言纠正"));
        assert!(prompt.contains("<unpublished_draft>"));
        assert!(prompt.contains("English body"));
        assert!(prompt.contains("精确数据时间首行"));
    }
    use super::*;
    use async_trait::async_trait;
    use futures::stream::{self, BoxStream};
    use hone_core::ToolExecutionObserver;
    use hone_core::agent::{AgentContext, AgentMessage};
    use hone_tools::{Tool, ToolParameter};
    use serde_json::{Value, json};
    use std::collections::VecDeque;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn tool_budget_error_is_a_pure_probe_over_its_inputs() {
        // Two of the three call sites use this as a feasibility probe rather
        // than to reject a call, so it must decide purely from its arguments
        // and never report a rejection on its own.
        let limits = HashMap::from([("web_search".to_string(), 1)]);
        let counts = HashMap::from([("web_search".to_string(), 1)]);
        let empty_counts: HashMap<String, u32> = HashMap::new();

        // Under both budgets: nothing to reject.
        assert!(tool_budget_error("web_search", Some(3), &limits, 0, &empty_counts).is_none());
        // Per-tool budget spent, global budget still open.
        let per_tool = tool_budget_error("web_search", Some(3), &limits, 1, &counts)
            .expect("per-tool budget should reject");
        assert_eq!(
            per_tool["error"],
            json!("tool `web_search` call limit reached (1)")
        );
        // Global budget takes precedence over the per-tool one.
        let global = tool_budget_error("web_search", Some(3), &limits, 3, &empty_counts)
            .expect("global budget should reject");
        assert_eq!(global["error"], json!("tool call limit reached (3)"));
        // A tool without its own limit rides only the global budget.
        assert!(tool_budget_error("data_fetch", Some(3), &limits, 2, &counts).is_none());
        // No global budget configured means no global rejection.
        assert!(tool_budget_error("data_fetch", None, &limits, 999, &counts).is_none());

        // Repeating a probe must return the same answer.
        assert_eq!(
            tool_budget_error("web_search", Some(3), &limits, 1, &counts),
            tool_budget_error("web_search", Some(3), &limits, 1, &counts)
        );
    }

    #[test]
    fn hidden_stream_formatter_suppresses_minimax_tool_protocol() {
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let first = formatter.push("前文<minimax:tool_");
        let second = formatter
            .push("call><invoke name=\"skill_tool\">payload</invoke></minimax:tool_call>后文");
        let rendered = format!("{first}{second}{}", formatter.finish());
        assert_eq!(rendered, "前文后文");

        let mut unfinished = hone_channels_compat::HiddenStreamFormatter::default();
        assert_eq!(
            unfinished.push("<minimax:tool_call><invoke name=\"skill_tool\">"),
            ""
        );
        assert_eq!(unfinished.finish(), "");
    }

    #[derive(Clone)]
    struct StreamingMockLlmProvider {
        rounds: Arc<Mutex<VecDeque<Vec<ChatStreamEvent>>>>,
        seen_tool_counts: Arc<Mutex<Vec<usize>>>,
        seen_tool_names: Arc<Mutex<Vec<Vec<String>>>>,
        seen_tool_choice_modes: Arc<Mutex<Vec<ToolChoiceMode>>>,
        seen_messages: Arc<Mutex<Vec<Vec<Message>>>>,
        delivered_events: Arc<AtomicUsize>,
        stream_calls: Arc<AtomicUsize>,
        failed_stream_calls: Arc<Mutex<Vec<usize>>>,
        failed_stream_errors: Arc<Mutex<HashMap<usize, String>>>,
        pending_stream_calls: Arc<Mutex<Vec<usize>>>,
        hang_after_first_event_stream_calls: Arc<Mutex<Vec<usize>>>,
    }

    impl StreamingMockLlmProvider {
        fn with_rounds(rounds: Vec<Vec<ChatStreamEvent>>) -> Self {
            Self {
                rounds: Arc::new(Mutex::new(rounds.into())),
                seen_tool_counts: Arc::new(Mutex::new(Vec::new())),
                seen_tool_names: Arc::new(Mutex::new(Vec::new())),
                seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
                seen_messages: Arc::new(Mutex::new(Vec::new())),
                delivered_events: Arc::new(AtomicUsize::new(0)),
                stream_calls: Arc::new(AtomicUsize::new(0)),
                failed_stream_calls: Arc::new(Mutex::new(Vec::new())),
                failed_stream_errors: Arc::new(Mutex::new(HashMap::new())),
                pending_stream_calls: Arc::new(Mutex::new(Vec::new())),
                hang_after_first_event_stream_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn failing_on_stream_calls(self, calls: &[usize]) -> Self {
            self.failed_stream_calls
                .lock()
                .expect("failed stream calls lock")
                .extend_from_slice(calls);
            self
        }

        fn failing_on_stream_call_with_error(self, call: usize, error: &str) -> Self {
            self.failed_stream_errors
                .lock()
                .expect("failed stream errors lock")
                .insert(call, error.to_string());
            self
        }

        fn pending_on_stream_calls(self, calls: &[usize]) -> Self {
            self.pending_stream_calls
                .lock()
                .expect("pending stream calls lock")
                .extend_from_slice(calls);
            self
        }
    }

    #[async_trait]
    impl LlmProvider for StreamingMockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            unreachable!("streaming test uses tools")
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            unreachable!("native streaming override should be used")
        }

        fn chat_with_tools_stream<'a>(
            &'a self,
            messages: &'a [Message],
            tools: &'a [Value],
            _model: Option<&'a str>,
            tool_choice_mode: ToolChoiceMode,
        ) -> BoxStream<'a, hone_core::HoneResult<ChatStreamEvent>> {
            self.seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .push(tools.len());
            self.seen_tool_names
                .lock()
                .expect("stream tool names lock")
                .push(
                    tools
                        .iter()
                        .filter_map(|tool| {
                            tool.get("function")
                                .and_then(|function| function.get("name"))
                                .and_then(Value::as_str)
                                .map(ToString::to_string)
                        })
                        .collect(),
                );
            self.seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .push(tool_choice_mode);
            self.seen_messages
                .lock()
                .expect("stream messages lock")
                .push(messages.to_vec());
            let stream_call = self.stream_calls.fetch_add(1, Ordering::SeqCst) + 1;
            let mut events = self
                .rounds
                .lock()
                .expect("stream rounds lock")
                .pop_front()
                .expect("stream round");
            let should_fail = self
                .failed_stream_calls
                .lock()
                .expect("failed stream calls lock")
                .contains(&stream_call);
            let failed_stream_error = self
                .failed_stream_errors
                .lock()
                .expect("failed stream errors lock")
                .get(&stream_call)
                .cloned();
            let should_pending = self
                .pending_stream_calls
                .lock()
                .expect("pending stream calls lock")
                .contains(&stream_call);
            if should_pending {
                return Box::pin(stream::pending());
            }
            // Most tests describe only payload deltas. Mirror the provider
            // contract by adding the lifecycle envelope automatically. A
            // round that contains any lifecycle event is intentionally kept
            // raw so protocol-negative tests can model missing/mismatched
            // Finish or Done boundaries precisely.
            let explicit_lifecycle = events.iter().any(|event| {
                matches!(
                    event,
                    ChatStreamEvent::ToolChoiceMetadata { .. }
                        | ChatStreamEvent::Finish(_)
                        | ChatStreamEvent::Done
                )
            });
            if !explicit_lifecycle {
                let finish_reason = if events
                    .iter()
                    .any(|event| matches!(event, ChatStreamEvent::ToolCallDelta { .. }))
                {
                    ChatStreamFinishReason::ToolCalls
                } else {
                    ChatStreamFinishReason::Stop
                };
                events.insert(
                    0,
                    ChatStreamEvent::ToolChoiceMetadata {
                        requested: tool_choice_mode,
                        effective: tool_choice_mode,
                        fallback: false,
                    },
                );
                events.push(ChatStreamEvent::Finish(finish_reason));
                events.push(ChatStreamEvent::Done);
            }
            let hang_take = if matches!(
                events.first(),
                Some(ChatStreamEvent::ToolChoiceMetadata { .. })
            ) {
                2
            } else {
                1
            };
            let items: Vec<hone_core::HoneResult<ChatStreamEvent>> =
                if let Some(error) = failed_stream_error {
                    vec![Err(hone_core::HoneError::Llm(error))]
                } else if should_fail {
                    vec![Err(hone_core::HoneError::Llm(format!(
                        "mock stream failure {stream_call}"
                    )))]
                } else {
                    events.into_iter().map(Ok).collect()
                };
            let delivered_events = self.delivered_events.clone();
            let should_hang_after_first = self
                .hang_after_first_event_stream_calls
                .lock()
                .expect("hang after first event calls lock")
                .contains(&stream_call);
            if should_hang_after_first {
                return Box::pin(
                    stream::iter(items.into_iter().take(hang_take))
                        .inspect(move |_| {
                            delivered_events.fetch_add(1, Ordering::SeqCst);
                        })
                        .chain(stream::pending()),
                );
            }
            Box::pin(stream::iter(items).inspect(move |_| {
                delivered_events.fetch_add(1, Ordering::SeqCst);
            }))
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    #[derive(Default)]
    struct RecordingStreamObserver {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl FunctionCallingStreamObserver for RecordingStreamObserver {
        async fn on_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("delta:{content}"));
        }

        async fn on_final_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("final:{content}"));
        }

        async fn on_content_reset(&self) {
            self.events
                .lock()
                .expect("stream events lock")
                .push("reset".to_string());
        }
    }

    struct CommittedPrefixStreamObserver {
        prefix: String,
        accumulated: Mutex<String>,
        events: Mutex<Vec<String>>,
    }

    impl CommittedPrefixStreamObserver {
        fn new(prefix: impl Into<String>) -> Self {
            Self {
                prefix: prefix.into(),
                accumulated: Mutex::new(String::new()),
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl FunctionCallingStreamObserver for CommittedPrefixStreamObserver {
        async fn on_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("delta:{content}"));
        }

        async fn on_final_content_delta(&self, content: &str) {
            self.accumulated
                .lock()
                .expect("accumulated stream content")
                .push_str(content);
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("final:{content}"));
        }

        fn committed_visible_prefix(&self) -> Option<String> {
            self.accumulated
                .lock()
                .expect("accumulated stream content")
                .starts_with(&self.prefix)
                .then(|| self.prefix.clone())
        }

        async fn commit_service_owned_prefix(&self, prefix: &str) -> bool {
            if prefix != self.prefix {
                return false;
            }
            let mut accumulated = self.accumulated.lock().expect("accumulated stream content");
            if accumulated.is_empty() {
                accumulated.push_str(prefix);
                self.events
                    .lock()
                    .expect("stream events lock")
                    .push(format!("commit:{prefix}"));
            }
            true
        }

        async fn on_content_reset(&self) {
            self.events
                .lock()
                .expect("stream events lock")
                .push("reset".to_string());
        }
    }

    #[derive(Clone)]
    struct MockLlmProvider {
        state: Arc<Mutex<MockState>>,
    }

    struct MockState {
        chat_calls: usize,
        chat_with_tools_calls: usize,
        next_chat_response: Option<String>,
        next_tool_responses: VecDeque<ChatResponse>,
        seen_tool_messages: Vec<Vec<Message>>,
    }

    impl MockLlmProvider {
        fn with_chat_response(content: &str) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: Some(content.to_string()),
                    next_tool_responses: VecDeque::new(),
                    seen_tool_messages: Vec::new(),
                })),
            }
        }

        fn with_chat_and_tool_responses(chat_response: &str, responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: Some(chat_response.to_string()),
                    next_tool_responses: responses.into(),
                    seen_tool_messages: Vec::new(),
                })),
            }
        }

        fn with_tool_responses(responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: None,
                    next_tool_responses: responses.into(),
                    seen_tool_messages: Vec::new(),
                })),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            let mut state = self.state.lock().expect("mock state lock");
            state.chat_calls += 1;
            Ok(hone_llm::provider::ChatResult {
                content: state
                    .next_chat_response
                    .clone()
                    .unwrap_or_else(|| "mock chat".to_string()),
                usage: None,
            })
        }

        async fn chat_with_tools(
            &self,
            messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            let mut state = self.state.lock().expect("mock state lock");
            state.chat_with_tools_calls += 1;
            state.seen_tool_messages.push(messages.to_vec());
            match state.next_tool_responses.pop_front() {
                Some(mock_tool_response) => Ok(mock_tool_response),
                None => Err(hone_core::HoneError::Llm(
                    "no more mock tool responses".to_string(),
                )),
            }
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    struct EchoTool;

    #[async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo_tool"
        }

        fn description(&self) -> &str {
            "echo"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "text".to_string(),
                required: true,
                r#enum: None,
                items: None,
            }]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "echo": args.get("text").and_then(|v| v.as_str()).unwrap_or_default()
            }))
        }
    }

    struct FinanceEvidenceTool;

    #[async_trait]
    impl Tool for FinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "text".to_string(),
                required: false,
                r#enum: None,
                items: None,
            }]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            if args.get("data_type").and_then(Value::as_str) == Some("search") {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                let data = match query.as_str() {
                    "CRWV" => json!([{"symbol":"CRWV","name":"CoreWeave, Inc."}]),
                    "NVIDIA" | "NVDA" => {
                        json!([{"symbol":"NVDA","name":"NVIDIA Corporation"}])
                    }
                    _ => json!([]),
                };
                return Ok(json!({"data_type":"search","data":data}));
            }
            Ok(json!({
                "evidence": args.get("text").and_then(|v| v.as_str()).unwrap_or_default()
            }))
        }
    }

    struct OversizedFinanceEvidenceTool;

    #[async_trait]
    impl Tool for OversizedFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "oversized finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            FinanceEvidenceTool.parameters()
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "data_type": "search",
                "data": [{"symbol":"NVDA","name":"NVIDIA Corporation"}],
                "provider_time": "2026-07-26T04:00:00-04:00",
                "oversized_blob": "OVERSIZED_SENTINEL".repeat(10_000),
            }))
        }
    }

    struct GroundedFinanceEvidenceTool;

    #[async_trait]
    impl Tool for GroundedFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "entity-bound quote and profile evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            let data_type = args
                .get("data_type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if data_type == "search" {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                let data = match query.as_str() {
                    "CRWV" => json!([{"symbol":"CRWV","name":"CoreWeave, Inc."}]),
                    "NVIDIA" | "NVDA" => {
                        json!([{"symbol":"NVDA","name":"NVIDIA Corporation"}])
                    }
                    _ => json!([]),
                };
                return Ok(json!({"data_type":"search","data":data}));
            }
            let symbols = args
                .get("ticker")
                .or_else(|| args.get("symbol"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|symbol| !symbol.is_empty())
                .map(str::to_ascii_uppercase)
                .collect::<Vec<_>>();
            let data = symbols
                .iter()
                .map(|symbol| {
                    if data_type == "quote" {
                        json!({
                            "symbol": symbol,
                            "price": if symbol == "CRWV" { 73.21 } else { 172.05 },
                            "currency": "USD",
                            "exchange": "NASDAQ Global Market",
                            "exchangeShortName": "NASDAQ",
                            "hone_quote_time": {
                                "beijing": "2026-07-18 04:00:00",
                                "new_york": "2026-07-17 16:00:00",
                                "market_date_new_york": "2026-07-17"
                            }
                        })
                    } else {
                        json!({
                            "symbol": symbol,
                            "companyName": if symbol == "CRWV" { "CoreWeave, Inc." } else { "NVIDIA Corporation" },
                            "exchange": "NASDAQ Global Market",
                            "exchangeShortName": "NASDAQ"
                        })
                    }
                })
                .collect::<Vec<_>>();
            Ok(json!({"data_type":data_type,"data":data}))
        }
    }

    struct SndkFinanceEvidenceTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for SndkFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "SNDK listing and earnings evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match args.get("data_type").and_then(Value::as_str) {
                Some("search") => Ok(json!({
                    "data_type": "search",
                    "data": [{
                        "symbol": "SNDK",
                        "name": "Sandisk Corporation",
                        "exchangeShortName": "NASDAQ"
                    }]
                })),
                Some("earnings_outlook") => Ok(json!({
                    "data_type": "earnings_outlook",
                    "ticker": "SNDK",
                    "data": {
                        "quote": [{"symbol":"SNDK","price":238.15,"exchange":"NASDAQ"}],
                        "profile": [{
                            "symbol":"SNDK",
                            "companyName":"Sandisk Corporation",
                            "exchangeShortName":"NASDAQ",
                            "isActivelyTrading":true
                        }],
                        "earnings": [{"date":"2026-08-05"}]
                    },
                    "hone_security_listing_evidence": {
                        "status":"active_listing",
                        "requested_symbol":"SNDK",
                        "quote_symbol":"SNDK",
                        "profile_symbol":"SNDK",
                        "exchange":"NASDAQ",
                        "profile_is_actively_trading":true,
                        "current_quote_available":true
                    }
                })),
                _ => Ok(json!({"data":[]})),
            }
        }
    }

    struct EntityRouteFinanceEvidenceTool;

    #[async_trait]
    impl Tool for EntityRouteFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "entity-route finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            let data_type = args
                .get("data_type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if data_type == "search" {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                let data = match query.as_str() {
                    "CRWV" => json!([
                        {"symbol":"CRWV","name":"CoreWeave, Inc."},
                        {"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF"}
                    ]),
                    "NVIDIA" | "NVDA" => json!([
                        {"symbol":"NVDA","name":"NVIDIA Corporation"},
                        {"symbol":"NVD.DE","name":"NVIDIA Corporation"}
                    ]),
                    _ => json!([]),
                };
                return Ok(json!({"data_type":"search","data":data}));
            }

            let symbols = args
                .get("ticker")
                .or_else(|| args.get("symbol"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|symbol| !symbol.is_empty())
                .map(|symbol| json!({"symbol":symbol.to_ascii_uppercase()}))
                .collect::<Vec<_>>();
            let data = match data_type {
                "quote" => symbols
                    .iter()
                    .filter_map(|item| item.get("symbol").and_then(Value::as_str))
                    .map(|symbol| {
                        json!({
                            "symbol": symbol,
                            "price": if symbol == "CRWV" { 73.21 } else { 172.05 },
                            "currency": "USD",
                            "exchange": "NASDAQ Global Market",
                            "exchangeShortName": "NASDAQ",
                            "hone_quote_time": {
                                "beijing": "2026-07-18 04:00:00",
                                "new_york": "2026-07-17 16:00:00",
                                "market_date_new_york": "2026-07-17"
                            }
                        })
                    })
                    .collect::<Vec<_>>(),
                "profile" => symbols
                    .iter()
                    .filter_map(|item| item.get("symbol").and_then(Value::as_str))
                    .map(|symbol| {
                        json!({
                            "symbol": symbol,
                            "companyName": if symbol == "CRWV" { "CoreWeave, Inc." } else { "NVIDIA Corporation" },
                            "exchange": "NASDAQ Global Market",
                            "exchangeShortName": "NASDAQ"
                        })
                    })
                    .collect::<Vec<_>>(),
                _ => symbols,
            };
            Ok(json!({"data_type":data_type,"data":data}))
        }
    }

    struct FailingFinanceEvidenceTool;

    #[async_trait]
    impl Tool for FailingFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "unavailable finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Err(hone_core::HoneError::Tool(
                "finance provider unavailable".to_string(),
            ))
        }
    }

    struct WebSearchEvidenceTool;

    #[async_trait]
    impl Tool for WebSearchEvidenceTool {
        fn name(&self) -> &str {
            "web_search"
        }

        fn description(&self) -> &str {
            "relationship evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "results": [
                    {
                        "title": "Capacity purchase announcement",
                        "url": "https://example.test/capacity",
                        "published_date": "2026-07-24",
                        "content": "The buyer agreed to purchase $6.3B of unused capacity."
                    },
                    {
                        "title": "Most-favored-nation relationship",
                        "url": "https://example.test/mfn",
                        "content": "The filing describes a most-favored-nation relationship."
                    }
                ]
            }))
        }
    }

    struct MarketMoveCanaryFinanceTool;

    #[async_trait]
    impl Tool for MarketMoveCanaryFinanceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "broad-market quote evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            let symbol = args
                .get("ticker")
                .or_else(|| args.get("symbol"))
                .or_else(|| args.get("query"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_uppercase();
            let (change, change_percentage, exchange) = match symbol.as_str() {
                "SPY" => (0.75, 0.1016, "AMEX"),
                "QQQ" => (-7.73, -1.11712, "NASDAQ"),
                _ => (0.0, 0.0, "UNKNOWN"),
            };
            Ok(json!({
                "data_type": "quote",
                "ticker": symbol,
                "data": [{
                    "symbol": symbol,
                    "change": change,
                    "changesPercentage": change_percentage,
                    "hone_change_basis": {
                        "pct": change_percentage,
                        "label": "常规时段涨跌（最新价较上一交易日收盘）"
                    },
                    "exchange": exchange,
                    "hone_quote_time": {
                        "beijing": "2026-07-25 04:00:00 +08:00",
                        "market_date_new_york": "2026-07-24"
                    }
                }]
            }))
        }
    }

    struct SnippetOnlyMarketMoveWebTool;

    #[async_trait]
    impl Tool for SnippetOnlyMarketMoveWebTool {
        fn name(&self) -> &str {
            "web_search"
        }

        fn description(&self) -> &str {
            "snippet-only market news"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "hone_search_contract": {
                    "evidence_scope": {
                        "full_page_content": false,
                        "kind": "search_snippets"
                    }
                },
                "results": [{
                    "title": "Why Is Stock Market Down Today, July 24, 2026?",
                    "url": "https://example.test/july-24-snippet",
                    "content": "renewed geopolitical tensions and a spike in oil"
                }]
            }))
        }
    }

    struct GroundedRelationshipEvidenceTool;

    #[async_trait]
    impl Tool for GroundedRelationshipEvidenceTool {
        fn name(&self) -> &str {
            "web_search"
        }

        fn description(&self) -> &str {
            "entity-bound relationship evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "results": [
                    {
                        "title": "CoreWeave filing describes NVIDIA capacity purchase",
                        "url": "https://example.test/capacity",
                        "content": "CoreWeave disclosed that NVIDIA agreed to purchase $6.3B of unused CoreWeave capacity."
                    },
                    {
                        "title": "CoreWeave filing describes NVIDIA investment terms",
                        "url": "https://example.test/mfn",
                        "content": "CoreWeave's filing describes NVIDIA as an investor and states that the parties have a most-favored-nation relationship."
                    }
                ]
            }))
        }
    }

    struct HangingPortfolioTool;

    #[async_trait]
    impl Tool for HangingPortfolioTool {
        fn name(&self) -> &str {
            "portfolio"
        }

        fn description(&self) -> &str {
            "persistent tool that never returns"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            std::future::pending().await
        }
    }

    struct FailingPortfolioTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for FailingPortfolioTool {
        fn name(&self) -> &str {
            "portfolio"
        }

        fn description(&self) -> &str {
            "persistent tool that reports an uncertain failure"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if args.get("action").and_then(Value::as_str) == Some("view") {
                return Ok(json!({
                    "holdings": [{"symbol":"CRWV"}],
                    "watchlist": []
                }));
            }
            Err(hone_core::HoneError::Tool(
                "portfolio write acknowledgement lost".to_string(),
            ))
        }
    }

    struct CountingTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for CountingTool {
        fn name(&self) -> &str {
            "counting_tool"
        }

        fn description(&self) -> &str {
            "count"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            let calls = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(json!({ "calls": calls }))
        }
    }

    struct CountingDataFetchTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for CountingDataFetchTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "count DataFetch registry executions"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            let calls = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(json!({ "calls": calls, "data": [] }))
        }
    }

    struct CountingSkillTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for CountingSkillTool {
        fn name(&self) -> &str {
            "skill_tool"
        }

        fn description(&self) -> &str {
            "count executable skill invocations"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            let calls = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(json!({ "calls": calls }))
        }
    }

    #[derive(Default)]
    struct MockToolObserver {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl ToolExecutionObserver for MockToolObserver {
        async fn on_tool_start(
            &self,
            tool_name: &str,
            _arguments: &Value,
            _reasoning: Option<String>,
        ) {
            self.events
                .lock()
                .expect("observer lock")
                .push(format!("start:{tool_name}"));
        }

        async fn on_tool_finish(&self, tool_name: &str, _arguments: &Value, success: bool) {
            self.events
                .lock()
                .expect("observer lock")
                .push(format!("done:{tool_name}:{success}"));
        }
    }

    struct HangingStartObserver;

    #[async_trait]
    impl ToolExecutionObserver for HangingStartObserver {
        async fn on_tool_start(
            &self,
            _tool_name: &str,
            _arguments: &Value,
            _reasoning: Option<String>,
        ) {
            std::future::pending().await
        }

        async fn on_tool_finish(&self, _tool_name: &str, _arguments: &Value, _success: bool) {}
    }

    #[derive(Default)]
    struct RecordingAuditSink {
        operations: Mutex<Vec<String>>,
        records: Mutex<Vec<LlmAuditRecord>>,
    }

    #[async_trait]
    impl LlmAuditSink for RecordingAuditSink {
        async fn record(&self, record: LlmAuditRecord) -> hone_core::HoneResult<()> {
            self.operations
                .lock()
                .expect("audit operations lock")
                .push(record.operation.clone());
            self.records
                .lock()
                .expect("audit records lock")
                .push(record);
            Ok(())
        }
    }

    fn test_finish_arguments() -> String {
        json!({
            "answer_scope": "回答当前测试请求",
            "facts": [],
            "inferences": [],
            "gaps": ["测试夹具未声明外部事实"]
        })
        .to_string()
    }

    fn data_finish_arguments(tool_call_ids: &[&str]) -> String {
        let facts = tool_call_ids
            .iter()
            .enumerate()
            .map(|(index, tool_call_id)| {
                json!({
                    "id": format!("F{}", index + 1),
                    "evidence": [{
                        "tool_call_id": tool_call_id,
                        "json_pointer": "/data/0/symbol"
                    }]
                })
            })
            .collect::<Vec<_>>();
        json!({
            "answer_scope": "回答当前测试请求",
            "facts": facts,
            "inferences": [],
            "gaps": []
        })
        .to_string()
    }

    fn relationship_finish_arguments(tool_call_id: &str) -> String {
        json!({
            "answer_scope": "回答 CoreWeave 与 NVIDIA 的已核验关系",
            "facts": [
                {
                    "id": "F1",
                    "evidence": [{
                        "tool_call_id": tool_call_id,
                        "result_number": 1,
                        "exact_excerpt": "The buyer agreed to purchase $6.3B of unused capacity."
                    }]
                },
                {
                    "id": "F2",
                    "evidence": [{
                        "tool_call_id": tool_call_id,
                        "result_number": 2,
                        "exact_excerpt": "The filing describes a most-favored-nation relationship."
                    }]
                }
            ],
            "inferences": [{
                "claim": "该关系不只是单向 GPU 采购。",
                "premise_fact_ids": ["F1", "F2"]
            }],
            "gaps": ["本轮测试来源未核验双方持股关系，不能写成无股权关系"]
        })
        .to_string()
    }

    fn test_validated_handoff() -> ValidatedResearchHandoff {
        ValidatedResearchHandoff {
            answer_scope: "回答当前测试请求".to_string(),
            facts: Vec::new(),
            inferences: Vec::new(),
            gaps: vec!["测试夹具未声明外部事实".to_string()],
            fallback_evidence: Vec::new(),
            validation_warnings: Vec::new(),
            unresolved_reference_count: 0,
        }
    }

    fn assert_explicit_terminal_messages(seen_messages: &Arc<Mutex<Vec<Vec<Message>>>>) {
        let terminal_messages = seen_messages
            .lock()
            .expect("stream messages lock")
            .last()
            .cloned()
            .expect("terminal messages");
        let system = terminal_messages
            .first()
            .and_then(|message| message.content.as_deref())
            .expect("terminal system instruction");
        let prompt = terminal_messages
            .last()
            .and_then(|message| message.content.as_deref())
            .expect("terminal user prompt");

        assert!(
            terminal_messages.iter().all(|message| {
                message.reasoning_content.is_none()
                    || (message.role == "assistant"
                        && message
                            .tool_calls
                            .as_ref()
                            .is_some_and(|calls| !calls.is_empty()))
            }),
            "provider reasoning may only survive as a tool-followup wire signature"
        );
        assert!(
            terminal_messages
                .iter()
                .all(|message| matches!(message.role.as_str(), "system" | "user")),
            "terminal synthesis must receive only system/user intent plus the compact handoff"
        );
        assert!(prompt.starts_with("【终局回答阶段】"));
        assert!(prompt.contains("【Agent 本轮结构化证据交接"));
        assert!(prompt.contains("Agent 已结束本轮合理的研究与工具尝试"));
        assert!(prompt.contains("`reasoning_content`、隐藏思考、未采用草稿"));
        assert!(system.contains(FINISH_RESEARCH_SYSTEM_INSTRUCTION));
        assert!(!prompt.contains("上一内部步骤未产出可用的新事实证据"));
    }

    #[test]
    fn terminal_prompt_is_authorized_only_by_explicit_finish() {
        let explicit = terminal_synthesis_prompt(None, &test_validated_handoff());
        assert!(explicit.contains("Agent 已结束本轮合理的研究与工具尝试"));
        assert!(!explicit.contains("上一内部步骤未产出可用的新事实证据"));
        assert!(explicit.contains("resolved_evidence"));
    }

    #[test]
    fn structured_finish_schema_is_flat_and_contains_no_free_text_fact_claim_or_url() {
        let schema = finish_research_tool_schema(&[]);
        let fact_properties =
            &schema["function"]["parameters"]["properties"]["facts"]["items"]["properties"];
        assert!(fact_properties.get("claim").is_none());
        assert!(fact_properties.get("proposed_claim").is_none());
        let evidence_schema = &fact_properties["evidence"]["items"];
        assert!(evidence_schema.get("oneOf").is_none());
        assert!(evidence_schema["properties"].get("url").is_none());
        assert_eq!(evidence_schema["required"], json!(["tool_call_id"]));
        assert!(
            evidence_schema["properties"]["tool_call_id"]
                .get("enum")
                .is_none(),
            "an empty runtime catalog must not emit a provider-hostile empty enum"
        );
    }

    #[test]
    fn finish_schema_enumerates_only_citable_current_turn_sources() {
        let mut context = AgentContext::new("dynamic-finish-source-catalog".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVDA");
        let calls = vec![
            ToolCall {
                id: "tc_search".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            },
            ToolCall {
                id: "tc_quote".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV"}"#.to_string(),
                },
            },
            ToolCall {
                id: "tc_web".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "web_search".to_string(),
                    arguments: r#"{"query":"relationship"}"#.to_string(),
                },
            },
            ToolCall {
                id: "tc_failed".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"profile","symbol":"NVDA"}"#.to_string(),
                },
            },
            ToolCall {
                id: "tc_other".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "echo_tool".to_string(),
                    arguments: r#"{"text":"ignore"}"#.to_string(),
                },
            },
        ];
        context.add_assistant_message(
            "",
            Some(
                calls
                    .iter()
                    .map(|call| serde_json::to_value(call).expect("serialize catalog call"))
                    .collect(),
            ),
        );
        context.add_tool_result("tc_search", "data_fetch", r#"{"data":[{"symbol":"CRWV"}]}"#);
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21}]}"#,
        );
        context.add_tool_result(
            "tc_web",
            "web_search",
            r#"{"results":[{"title":"Relationship filing","url":"https://example.test/filing","content":"NVIDIA agreed to purchase unused CoreWeave capacity."},{"title":"Missing URL","content":"This row is not citable."}]}"#,
        );
        context.add_tool_result(
            "tc_failed",
            "data_fetch",
            r#"{"error":"provider unavailable"}"#,
        );
        context.add_tool_result("tc_other", "echo_tool", r#"{"echo":"ignore"}"#);
        context.add_tool_result(
            "tc_without_invocation",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","price":15}]}"#,
        );

        let sources = current_turn_research_source_catalog(&context, turn_message_start);
        assert_eq!(
            sources
                .iter()
                .map(|source| source.tool_call_id.as_str())
                .collect::<Vec<_>>(),
            ["tc_quote", "tc_web"]
        );
        assert_eq!(
            sources[0].description,
            "data_fetch; data_type=quote; target=CRWV"
        );
        assert_eq!(
            sources[1].description,
            "web_search; citable result_number=1"
        );

        let schema = finish_research_tool_schema(&sources);
        let evidence = &schema["function"]["parameters"]["properties"]["facts"]["items"]["properties"]
            ["evidence"]["items"];
        assert_eq!(
            evidence["properties"]["tool_call_id"]["enum"],
            json!(["tc_quote", "tc_web"])
        );
        let catalog = research_source_catalog_prompt(&sources);
        assert!(catalog.contains("tc_quote"));
        assert!(catalog.contains("tc_web"));
        assert!(!catalog.contains("provider unavailable"));
        assert!(!catalog.contains("https://example.test/filing"));
        assert!(!catalog.contains("NVIDIA agreed"));

        let empty_schema = finish_research_tool_schema(&[]);
        assert!(
            empty_schema["function"]["parameters"]["properties"]["facts"]["items"]
                ["properties"]["evidence"]["items"]["properties"]["tool_call_id"]
                .get("enum")
                .is_none(),
            "an empty source catalog must not emit enum: [], which some compatible providers reject"
        );
    }

    #[test]
    fn generic_web_tool_name_recovers_only_one_uniquely_selected_excerpt() {
        let mut context = AgentContext::new("unique-web-excerpt-source".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVIDIA 的关系");
        let web_call = ToolCall {
            id: "tc_real_web".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"CoreWeave NVIDIA relationship"}"#.to_string(),
            },
        };
        context.add_assistant_message(
            "",
            Some(vec![
                serde_json::to_value(web_call).expect("serialize Web call"),
            ]),
        );
        context.add_tool_result(
            "tc_real_web",
            "web_search",
            r#"{"results":[{"title":"Capacity agreement","url":"https://example.test/agreement","content":"NVIDIA agreed to purchase unused CoreWeave capacity through 2032."}]}"#,
        );
        let failed_web_call = ToolCall {
            id: "tc_failed_web".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"failed duplicate"}"#.to_string(),
            },
        };
        context.add_assistant_message(
            "",
            Some(vec![
                serde_json::to_value(failed_web_call).expect("serialize failed Web call"),
            ]),
        );
        context.add_tool_result(
            "tc_failed_web",
            "web_search",
            r#"{"error":"provider unavailable","results":[{"title":"Residual result","url":"https://example.test/failed","content":"NVIDIA agreed to purchase unused CoreWeave capacity through 2032."}]}"#,
        );
        let handoff = ResearchHandoff {
            answer_scope: "回答双方关系".to_string(),
            facts: vec![ResearchHandoffFact {
                id: "F1".to_string(),
                evidence: vec![ResearchEvidenceRef {
                    tool_call_id: "web_search".to_string(),
                    result_number: Some(1),
                    exact_excerpt: Some(
                        "NVIDIA agreed to purchase unused CoreWeave capacity through 2032."
                            .to_string(),
                    ),
                    json_pointer: None,
                }],
            }],
            inferences: Vec::new(),
            gaps: Vec::new(),
        };

        let validated =
            validate_finish_research_handoff(handoff.clone(), &context, turn_message_start);
        assert_eq!(validated.facts.len(), 1);
        assert_eq!(
            validated.facts[0].resolved_evidence[0]["tool_call_id"],
            "tc_real_web"
        );
        assert_eq!(validated.unresolved_reference_count, 0);
        assert_eq!(
            current_turn_research_source_catalog(&context, turn_message_start)
                .iter()
                .map(|source| source.tool_call_id.as_str())
                .collect::<Vec<_>>(),
            ["tc_real_web"],
            "a failed response with residual results is neither catalogued nor eligible for generic Web recovery"
        );

        let duplicate_call = ToolCall {
            id: "tc_duplicate_web".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"duplicate"}"#.to_string(),
            },
        };
        context.add_assistant_message(
            "",
            Some(vec![
                serde_json::to_value(duplicate_call).expect("serialize duplicate Web call"),
            ]),
        );
        context.add_tool_result(
            "tc_duplicate_web",
            "web_search",
            r#"{"results":[{"title":"Duplicate agreement","url":"https://example.test/duplicate","content":"NVIDIA agreed to purchase unused CoreWeave capacity through 2032."}]}"#,
        );
        let ambiguous = validate_finish_research_handoff(handoff, &context, turn_message_start);
        assert!(ambiguous.facts.is_empty());
        assert_eq!(ambiguous.unresolved_reference_count, 1);
        assert!(ambiguous.fallback_evidence.is_empty());
        assert!(
            ambiguous
                .validation_warnings
                .iter()
                .any(|warning| { warning.contains("ambiguous across current-turn Web results") })
        );
    }

    fn test_fallback_scope(
        data_refs: &[(&str, &str)],
        web_refs: &[(&str, usize)],
    ) -> FallbackEvidenceScope {
        let mut scope = FallbackEvidenceScope::default();
        for (tool_call_id, json_pointer) in data_refs {
            scope.observe_reference(&ResearchEvidenceRef {
                tool_call_id: (*tool_call_id).to_string(),
                result_number: None,
                exact_excerpt: None,
                json_pointer: Some((*json_pointer).to_string()),
            });
        }
        for (tool_call_id, result_number) in web_refs {
            scope.observe_reference(&ResearchEvidenceRef {
                tool_call_id: (*tool_call_id).to_string(),
                result_number: Some(*result_number),
                exact_excerpt: None,
                json_pointer: None,
            });
        }
        scope
    }

    fn add_test_data_fetch_calls(context: &mut AgentContext, calls: &[(&str, &str)]) {
        let tool_calls = calls
            .iter()
            .map(|(tool_call_id, data_type)| {
                serde_json::to_value(ToolCall {
                    id: (*tool_call_id).to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: json!({"data_type": data_type}).to_string(),
                    },
                })
                .expect("serialize test DataFetch call")
            })
            .collect();
        context.add_assistant_message("", Some(tool_calls));
    }

    fn add_test_web_calls(context: &mut AgentContext, calls: &[(&str, &str)]) {
        let tool_calls = calls
            .iter()
            .map(|(tool_call_id, query)| {
                serde_json::to_value(ToolCall {
                    id: (*tool_call_id).to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "web_search".to_string(),
                        arguments: json!({"query": query}).to_string(),
                    },
                })
                .expect("serialize test Web call")
            })
            .collect();
        context.add_assistant_message("", Some(tool_calls));
    }

    #[test]
    fn data_field_provenance_accepts_only_successful_scalar_data_fetch_values() {
        let failed = validate_data_evidence_ref(
            "data_fetch",
            r#"{"error":"provider unavailable"}"#,
            "tc_failed",
            "/error",
            "quote",
        )
        .expect_err("failed DataFetch must not become evidence");
        assert!(failed.contains("failed/error"));

        let partial_error = validate_data_evidence_ref(
            "data_fetch",
            r#"{"data":{"price":73.21},"errors":{"profile":"unavailable"}}"#,
            "tc_partial",
            "/errors/profile",
            "quote",
        )
        .expect_err("an error field in a partially successful payload is not fact evidence");
        assert!(partial_error.contains("must not resolve an error field"));

        let object = validate_data_evidence_ref(
            "data_fetch",
            r#"{"data":{"quote":{"price":73.21}}}"#,
            "tc_object",
            "/data/quote",
            "quote",
        )
        .expect_err("object-valued pointers must be narrowed to a scalar");
        assert!(object.contains("non-null scalar"));

        let wrong_tool = validate_data_evidence_ref(
            "echo_tool",
            r#"{"data":{"price":73.21}}"#,
            "tc_echo",
            "/data/price",
            "quote",
        )
        .expect_err("non-DataFetch tools must not satisfy data_field provenance");
        assert!(wrong_tool.contains("not data_fetch"));

        let scalar = validate_data_evidence_ref(
            "data_fetch",
            r#"{"data":{"price":73.21}}"#,
            "tc_scalar",
            "/data/price",
            "quote",
        )
        .expect("successful scalar DataFetch value");
        assert_eq!(scalar["value"], 73.21);

        let search_candidate = validate_data_evidence_ref(
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}"#,
            "tc_search",
            "/data/1/symbol",
            "search",
        )
        .expect_err("identity search candidates must never become terminal facts");
        assert!(search_candidate.contains("identity candidates"));
    }

    #[test]
    fn structured_data_search_reference_is_never_terminal_evidence() {
        let mut context = AgentContext::new("structured-search-rejection".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_search", "search")]);
        context.add_tool_result(
            "tc_search",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","name":"CoreWeave"},{"symbol":"CWY","name":"YieldBOOST CRWV ETF"}]}"#,
        );
        let handoff = ResearchHandoff {
            answer_scope: "回答 CRWV".to_string(),
            facts: vec![ResearchHandoffFact {
                id: "F1".to_string(),
                evidence: vec![ResearchEvidenceRef {
                    tool_call_id: "tc_search".to_string(),
                    result_number: None,
                    exact_excerpt: None,
                    json_pointer: Some("/data/1/symbol".to_string()),
                }],
            }],
            inferences: Vec::new(),
            gaps: Vec::new(),
        };

        let validated = validate_finish_research_handoff(handoff, &context, turn_message_start);
        assert!(validated.facts.is_empty());
        assert!(validated.fallback_evidence.is_empty());
        assert!(
            validated
                .validation_warnings
                .iter()
                .any(|warning| { warning.contains("identity candidates") })
        );
        let serialized = serde_json::to_string(&validated).expect("serialize rejected search");
        assert!(!serialized.contains("CWY"));
    }

    #[test]
    fn data_reference_without_matching_invocation_is_rejected() {
        let mut context = AgentContext::new("missing-data-invocation".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        context.add_tool_result(
            "tc_spoofed_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21}]}"#,
        );
        let handoff = ResearchHandoff {
            answer_scope: "回答 CRWV".to_string(),
            facts: vec![ResearchHandoffFact {
                id: "F1".to_string(),
                evidence: vec![ResearchEvidenceRef {
                    tool_call_id: "tc_spoofed_quote".to_string(),
                    result_number: None,
                    exact_excerpt: None,
                    json_pointer: Some("/data/0/price".to_string()),
                }],
            }],
            inferences: Vec::new(),
            gaps: Vec::new(),
        };

        let validated = validate_finish_research_handoff(handoff, &context, turn_message_start);
        assert!(validated.facts.is_empty());
        assert!(validated.fallback_evidence.is_empty());
        assert!(
            validated.validation_warnings.iter().any(|warning| {
                warning.contains("no matching current-turn DataFetch invocation")
            })
        );
    }

    #[test]
    fn gaps_only_handoff_does_not_replay_unselected_current_turn_evidence() {
        let mut context = AgentContext::new("gap-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(
            &mut context,
            &[("tc_failed", "quote"), ("tc_quote", "quote")],
        );
        context.add_tool_result(
            "tc_failed",
            "data_fetch",
            r#"{"error":"provider unavailable"}"#,
        );
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21}]}"#,
        );
        let handoff = ResearchHandoff {
            answer_scope: "回答 CRWV 当前情况".to_string(),
            facts: Vec::new(),
            inferences: Vec::new(),
            gaps: vec!["估值分母期间未核验".to_string()],
        };

        let validated = validate_finish_research_handoff(handoff, &context, turn_message_start);

        assert!(validated.facts.is_empty());
        assert!(validated.fallback_evidence.is_empty());
        let encoded = serde_json::to_string(&validated).expect("serialize fallback handoff");
        assert!(!encoded.contains("tc_quote"));
        assert!(!encoded.contains("73.21"));
        assert!(!encoded.contains("tc_failed"));
    }

    #[test]
    fn compact_fallback_prioritizes_quote_identity_price_and_source_time() {
        let mut context = AgentContext::new("quote-fallback-priority".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_quote", "quote")]);
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"avgVolume":100,"change":0.3,"changesPercentage":0.41,"dayHigh":74,"dayLow":71,"earningsAnnouncement":"2026-08-01","eps":1.1,"exchange":"NASDAQ","exchangeShortName":"NASDAQ","hone_quote_time":{"beijing":"2026-07-18 04:00"},"marketCap":35000000000,"name":"CoreWeave","open":72,"pe":55,"previousClose":72.91,"price":73.21,"sharesOutstanding":470000000,"symbol":"CRWV","timestamp":1784328000,"volume":1234567,"yearHigh":187,"yearLow":33}]}"#,
        );

        let scope = test_fallback_scope(&[("tc_quote", "/data/0/price")], &[]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        let pointers = catalog
            .iter()
            .filter_map(|item| item.get("json_pointer").and_then(Value::as_str))
            .collect::<Vec<_>>();

        assert!(pointers.contains(&"/data/0/symbol"));
        assert!(pointers.contains(&"/data/0/price"));
        assert!(pointers.contains(&"/data/0/hone_quote_time/beijing"));
        assert!(catalog.len() <= MAX_FALLBACK_ITEMS_PER_TOOL);
    }

    #[test]
    fn batched_dual_ticker_quote_fallback_keeps_market_cap_and_pe_for_both() {
        let mut context = AgentContext::new("dual-quote-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVDA 的估值怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_dual_quote", "quote")]);
        context.add_tool_result(
            "tc_dual_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","hone_quote_time":{"beijing":"2026-07-18 04:00"},"price":73.21,"currency":"USD","changesPercentage":0.41,"change":0.30,"marketCap":35000000000,"pe":55},{"symbol":"NVDA","hone_quote_time":{"beijing":"2026-07-18 04:00"},"price":172.00,"currency":"USD","changesPercentage":1.25,"change":2.12,"marketCap":4200000000000,"pe":48}]}"#,
        );

        let scope = test_fallback_scope(
            &[
                ("tc_dual_quote", "/data/0/price"),
                ("tc_dual_quote", "/data/1/price"),
            ],
            &[],
        );
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        let pointers = catalog
            .iter()
            .filter_map(|item| item.get("json_pointer").and_then(Value::as_str))
            .collect::<BTreeSet<_>>();

        for pointer in [
            "/data/0/symbol",
            "/data/0/price",
            "/data/0/marketCap",
            "/data/0/pe",
            "/data/1/symbol",
            "/data/1/price",
            "/data/1/marketCap",
            "/data/1/pe",
        ] {
            assert!(pointers.contains(pointer), "missing {pointer}: {catalog:?}");
        }
    }

    #[test]
    fn batched_quote_fallback_does_not_cross_the_selected_object() {
        let mut context = AgentContext::new("single-object-quote-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_dual_quote", "quote")]);
        context.add_tool_result(
            "tc_dual_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21,"marketCap":35000000000},{"symbol":"NBIS","price":52.40,"marketCap":12600000000}]}"#,
        );

        let scope = test_fallback_scope(&[("tc_dual_quote", "/data/0/missing_pe")], &[]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);

        assert!(catalog.iter().any(|item| item["value"] == "CRWV"));
        assert!(catalog.iter().all(|item| {
            item["json_pointer"]
                .as_str()
                .is_some_and(|pointer| pointer.starts_with("/data/0/"))
        }));
        let serialized = serde_json::to_string(&catalog).expect("serialize selected quote row");
        assert!(!serialized.contains("NBIS"));
        assert!(!serialized.contains("12600000000"));
    }

    #[test]
    fn snapshot_array_or_null_pointer_does_not_expand_the_parent_object() {
        let mut context = AgentContext::new("snapshot-container-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_snapshot", "snapshot")]);
        context.add_tool_result(
            "tc_snapshot",
            "data_fetch",
            r#"{"data":{"quote":[{"symbol":"CRWV","price":73.21}],"profile":null,"news":[{"symbol":"NBIS","title":"unselected sibling"}]}}"#,
        );

        for pointer in ["/data/quote", "/data/profile"] {
            let scope = test_fallback_scope(&[("tc_snapshot", pointer)], &[]);
            let catalog =
                current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
            assert!(
                catalog.is_empty(),
                "{pointer} widened fallback: {catalog:?}"
            );
        }
    }

    #[test]
    fn error_field_pointer_produces_no_fallback_evidence() {
        let mut context = AgentContext::new("error-field-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_partial", "profile")]);
        context.add_tool_result(
            "tc_partial",
            "data_fetch",
            r#"{"data":{"symbol":"CRWV","price":73.21,"errors":{"profile":"unavailable"},"provider_error":"slow"}}"#,
        );

        let scope = test_fallback_scope(
            &[
                ("tc_partial", "/data/errors/profile"),
                ("tc_partial", "/data/provider_error"),
            ],
            &[],
        );
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);

        assert!(
            catalog.is_empty(),
            "error paths widened fallback: {catalog:?}"
        );
    }

    #[test]
    fn four_period_financial_fallback_uses_only_the_selected_row() {
        let mut context = AgentContext::new("selected-financial-row".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 估值怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_financials", "financials")]);
        context.add_tool_result(
            "tc_financials",
            "data_fetch",
            r#"{"data":[{"period":"FY","calendarYear":"2025","revenue":5000},{"period":"FY","calendarYear":"2024","revenue":4000},{"period":"FY","calendarYear":"2023","revenue":3000},{"period":"FY","calendarYear":"2022","revenue":2000}]}"#,
        );

        let scope = test_fallback_scope(&[("tc_financials", "/data/2/missing_ebitda")], &[]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);

        assert!(catalog.iter().any(|item| item["value"] == "2023"));
        assert!(catalog.iter().any(|item| item["value"] == 3000));
        assert!(catalog.iter().all(|item| {
            item["json_pointer"]
                .as_str()
                .is_some_and(|pointer| pointer.starts_with("/data/2/"))
        }));
    }

    #[test]
    fn invalid_web_excerpt_falls_back_to_only_the_selected_result_number() {
        let mut context = AgentContext::new("selected-web-result".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVIDIA 有什么关系");
        context.add_tool_result(
            "tc_web",
            "web_search",
            r#"{"results":[{"title":"Unselected one","url":"https://example.test/1","content":"first unrelated result"},{"title":"Selected two","url":"https://example.test/2","content":"second selected relationship result"},{"title":"Unselected three","url":"https://example.test/3","content":"third unrelated result"}]}"#,
        );

        let scope = test_fallback_scope(&[], &[("tc_web", 2)]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);

        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0]["result_number"], 2);
        assert_eq!(catalog[0]["title"], "Selected two");
        let serialized = serde_json::to_string(&catalog).expect("serialize selected web result");
        assert!(!serialized.contains("Unselected one"));
        assert!(!serialized.contains("Unselected three"));
    }

    #[test]
    fn schema_invalid_finish_recovers_only_fixed_path_references() {
        let mut context = AgentContext::new("schema-invalid-scoped-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(
            &mut context,
            &[
                ("tc_selected_quote", "quote"),
                ("tc_unselected_nbis", "quote"),
            ],
        );
        context.add_tool_result(
            "tc_selected_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21}]}"#,
        );
        context.add_tool_result(
            "tc_unselected_nbis",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","price":52.40}]}"#,
        );
        let finish = ToolCall {
            id: "tc_finish".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: FINISH_RESEARCH_TOOL_NAME.to_string(),
                // Valid JSON but intentionally missing the typed handoff's
                // required top-level fields. A stray root locator must not be
                // discovered recursively.
                arguments: json!({
                    "facts": [{
                        "id": "F1",
                        "evidence": [{
                            "tool_call_id": "tc_selected_quote",
                            "json_pointer": "/data/0/missing_pe"
                        }]
                    }],
                    "tool_call_id": "tc_unselected_nbis",
                    "json_pointer": "/data/0/price"
                })
                .to_string(),
            },
        };
        assert!(parse_finish_research_handoff(&finish).is_err());

        let scope = fallback_scope_from_finish_calls(&[&finish]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        let serialized = serde_json::to_string(&catalog).expect("serialize recovered scope");
        assert!(serialized.contains("tc_selected_quote"));
        assert!(serialized.contains("CRWV"));
        assert!(!serialized.contains("tc_unselected_nbis"));
        assert!(!serialized.contains("NBIS"));
    }

    #[test]
    fn multi_tool_dual_ticker_valuation_fallback_keeps_each_entitys_core_inputs() {
        let mut context = AgentContext::new("multi-tool-valuation-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NBIS 的估值怎么看");
        add_test_data_fetch_calls(
            &mut context,
            &[
                ("tc_quote", "quote"),
                ("tc_crwv_financials", "financials"),
                ("tc_nbis_financials", "financials"),
            ],
        );
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21,"marketCap":35000000000,"pe":55},{"symbol":"NBIS","price":52.40,"marketCap":12600000000,"pe":-8}]}"#,
        );
        context.add_tool_result(
            "tc_crwv_financials",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","date":"2025-12-31","period":"FY","reportedCurrency":"USD","revenue":5100000000,"ebitda":620000000,"netIncome":-410000000,"freeCashFlow":-1200000000}]}"#,
        );
        context.add_tool_result(
            "tc_nbis_financials",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","date":"2025-12-31","period":"FY","reportedCurrency":"USD","revenue":920000000,"ebitda":-450000000,"netIncome":-610000000,"freeCashFlow":-700000000}]}"#,
        );

        let scope = test_fallback_scope(
            &[
                ("tc_quote", "/data/0/marketCap"),
                ("tc_quote", "/data/1/marketCap"),
                ("tc_crwv_financials", "/data/0/revenue"),
                ("tc_nbis_financials", "/data/0/revenue"),
            ],
            &[],
        );
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        for (tool_call_id, pointer, expected) in [
            ("tc_quote", "/data/0/marketCap", json!(35000000000_u64)),
            ("tc_quote", "/data/1/marketCap", json!(12600000000_u64)),
            (
                "tc_crwv_financials",
                "/data/0/revenue",
                json!(5100000000_u64),
            ),
            ("tc_crwv_financials", "/data/0/ebitda", json!(620000000_u64)),
            (
                "tc_nbis_financials",
                "/data/0/revenue",
                json!(920000000_u64),
            ),
            (
                "tc_nbis_financials",
                "/data/0/ebitda",
                json!(-450000000_i64),
            ),
        ] {
            assert!(
                catalog.iter().any(|item| {
                    item["tool_call_id"] == tool_call_id
                        && item["json_pointer"] == pointer
                        && item["value"] == expected
                }),
                "missing {tool_call_id} {pointer}: {catalog:?}"
            );
        }
    }

    #[test]
    fn realistic_dual_valuation_fallback_excludes_search_candidates_and_keeps_latest_inputs() {
        let mut context = AgentContext::new("realistic-dual-valuation-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NBIS 的估值怎么看");
        let data_call = |id: &str, arguments: Value| {
            serde_json::to_value(ToolCall {
                id: id.to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: arguments.to_string(),
                },
            })
            .expect("serialize DataFetch call")
        };
        let web_call = |id: &str, query: &str| {
            serde_json::to_value(ToolCall {
                id: id.to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "web_search".to_string(),
                    arguments: json!({"query": query}).to_string(),
                },
            })
            .expect("serialize Web call")
        };
        context.add_assistant_message(
            "",
            Some(vec![
                data_call(
                    "tc_search_crwv",
                    json!({"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}),
                ),
                data_call(
                    "tc_search_nbis",
                    json!({"data_type":"search","query":"NBIS","entity_route":"nbis","identity_match":"exact_symbol"}),
                ),
                data_call(
                    "tc_quote",
                    json!({"data_type":"quote","ticker":"CRWV,NBIS"}),
                ),
                data_call(
                    "tc_profile_crwv",
                    json!({"data_type":"profile","ticker":"CRWV"}),
                ),
                data_call(
                    "tc_profile_nbis",
                    json!({"data_type":"profile","ticker":"NBIS"}),
                ),
                data_call(
                    "tc_crwv_financials",
                    json!({"data_type":"financials","ticker":"CRWV"}),
                ),
                data_call(
                    "tc_nbis_financials",
                    json!({"data_type":"financials","ticker":"NBIS"}),
                ),
                web_call("tc_web_business", "CoreWeave NBIS business relationship"),
                web_call("tc_web_ownership", "CoreWeave NBIS investment ownership"),
            ]),
        );
        context.add_tool_result(
            "tc_search_crwv",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","name":"CoreWeave"},{"symbol":"CWY","name":"YieldBOOST CRWV ETF"}]}"#,
        );
        context.add_tool_result(
            "tc_search_nbis",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","name":"Nebius Group"},{"symbol":"NBIX","name":"Neurocrine Biosciences"}]}"#,
        );
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","hone_quote_time":{"beijing":"2026-07-18 04:00"},"price":73.21,"currency":"USD","changesPercentage":0.41,"change":0.30,"marketCap":35000000000,"pe":55},{"symbol":"NBIS","hone_quote_time":{"beijing":"2026-07-18 04:00"},"price":52.40,"currency":"USD","changesPercentage":-0.8,"change":-0.42,"marketCap":12600000000,"pe":-8}]}"#,
        );
        context.add_tool_result(
            "tc_profile_crwv",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","name":"CoreWeave","currency":"USD","exchange":"NASDAQ","industry":"Software - Infrastructure","description":"Cloud infrastructure profile text","price":73.21,"marketCap":35000000000}]}"#,
        );
        context.add_tool_result(
            "tc_profile_nbis",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","name":"Nebius Group","currency":"USD","exchange":"NASDAQ","industry":"Information Technology Services","description":"AI infrastructure profile text","price":52.40,"marketCap":12600000000}]}"#,
        );
        context.add_tool_result(
            "tc_crwv_financials",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","date":"2025-12-31","calendarYear":"2025","period":"FY","reportedCurrency":"USD","revenue":5100000000,"ebitda":620000000,"netIncome":-410000000},{"symbol":"CRWV","date":"2024-12-31","calendarYear":"2024","period":"FY","reportedCurrency":"USD","revenue":1900000000,"ebitda":180000000,"netIncome":-860000000},{"symbol":"CRWV","date":"2023-12-31","calendarYear":"2023","period":"FY","reportedCurrency":"USD","revenue":1200000000,"ebitda":90000000,"netIncome":-590000000},{"symbol":"CRWV","date":"2022-12-31","calendarYear":"2022","period":"FY","reportedCurrency":"USD","revenue":500000000,"ebitda":20000000,"netIncome":-300000000}]}"#,
        );
        context.add_tool_result(
            "tc_nbis_financials",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","date":"2025-12-31","calendarYear":"2025","period":"FY","reportedCurrency":"USD","revenue":920000000,"ebitda":-450000000,"netIncome":-610000000},{"symbol":"NBIS","date":"2024-12-31","calendarYear":"2024","period":"FY","reportedCurrency":"USD","revenue":550000000,"ebitda":-370000000,"netIncome":-641400000},{"symbol":"NBIS","date":"2023-12-31","calendarYear":"2023","period":"FY","reportedCurrency":"USD","revenue":300000000,"ebitda":-250000000,"netIncome":-500000000},{"symbol":"NBIS","date":"2022-12-31","calendarYear":"2022","period":"FY","reportedCurrency":"USD","revenue":180000000,"ebitda":-180000000,"netIncome":-400000000}]}"#,
        );
        for (tool_call_id, topic) in [
            ("tc_web_business", "business"),
            ("tc_web_ownership", "ownership"),
        ] {
            let results = (1..=3)
                .map(|index| {
                    json!({
                        "title": format!("{topic} source {index}"),
                        "url": format!("https://example.test/{topic}/{index}"),
                        "content": format!("Current sourced {topic} excerpt number {index}.")
                    })
                })
                .collect::<Vec<_>>();
            context.add_tool_result(
                tool_call_id,
                "web_search",
                &json!({"results": results}).to_string(),
            );
        }

        let mut scope = test_fallback_scope(
            &[
                ("tc_search_crwv", "/data/1/symbol"),
                ("tc_search_nbis", "/data/1/symbol"),
                ("tc_quote", "/data/0/marketCap"),
                ("tc_quote", "/data/1/marketCap"),
                ("tc_profile_crwv", "/data/0/name"),
                ("tc_profile_nbis", "/data/0/name"),
            ],
            &[
                ("tc_web_business", 1),
                ("tc_web_business", 2),
                ("tc_web_business", 3),
                ("tc_web_ownership", 1),
                ("tc_web_ownership", 2),
                ("tc_web_ownership", 3),
            ],
        );
        for tool_call_id in ["tc_crwv_financials", "tc_nbis_financials"] {
            for row in 0..4 {
                scope.observe_reference(&ResearchEvidenceRef {
                    tool_call_id: tool_call_id.to_string(),
                    result_number: None,
                    exact_excerpt: None,
                    json_pointer: Some(format!("/data/{row}/revenue")),
                });
            }
        }
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        assert_eq!(catalog.len(), MAX_FALLBACK_EVIDENCE_ITEMS);
        let serialized = serde_json::to_string(&catalog).expect("serialize realistic fallback");
        assert!(!serialized.contains("CWY"));
        assert!(!serialized.contains("NBIX"));
        assert!(catalog.iter().all(|item| {
            !matches!(
                item["tool_call_id"].as_str(),
                Some("tc_search_crwv" | "tc_search_nbis")
            )
        }));
        for (tool_call_id, pointer) in [
            ("tc_quote", "/data/0/marketCap"),
            ("tc_quote", "/data/0/pe"),
            ("tc_quote", "/data/1/marketCap"),
            ("tc_quote", "/data/1/pe"),
            ("tc_crwv_financials", "/data/0/date"),
            ("tc_crwv_financials", "/data/0/period"),
            ("tc_crwv_financials", "/data/0/revenue"),
            ("tc_crwv_financials", "/data/0/ebitda"),
            ("tc_nbis_financials", "/data/0/date"),
            ("tc_nbis_financials", "/data/0/period"),
            ("tc_nbis_financials", "/data/0/revenue"),
            ("tc_nbis_financials", "/data/0/ebitda"),
        ] {
            assert!(
                catalog.iter().any(|item| {
                    item["tool_call_id"] == tool_call_id && item["json_pointer"] == pointer
                }),
                "missing {tool_call_id} {pointer}: {catalog:?}"
            );
        }
    }

    #[test]
    fn covered_fallback_locators_are_removed_before_the_global_cap() {
        let mut context = AgentContext::new("fallback-exclusion-before-cap".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_older_quote", "quote")]);
        context.add_tool_result(
            "tc_older_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21,"marketCap":35000000000,"pe":55}]}"#,
        );
        let mut covered = BTreeSet::new();
        let mut scope = test_fallback_scope(&[("tc_older_quote", "/data/0/price")], &[]);
        for tool_index in 0..3 {
            let tool_call_id = format!("tc_covered_{tool_index}");
            add_test_data_fetch_calls(&mut context, &[(tool_call_id.as_str(), "financials")]);
            let data = (0..MAX_FALLBACK_ITEMS_PER_TOOL)
                .map(|item_index| json!({"symbol": format!("NOISE_{tool_index}_{item_index}")}))
                .collect::<Vec<_>>();
            let payload = json!({"data": data}).to_string();
            context.add_tool_result(&tool_call_id, "data_fetch", &payload);
            for item_index in 0..MAX_FALLBACK_ITEMS_PER_TOOL {
                scope.observe_reference(&ResearchEvidenceRef {
                    tool_call_id: tool_call_id.clone(),
                    result_number: None,
                    exact_excerpt: None,
                    json_pointer: Some(format!("/data/{item_index}/symbol")),
                });
                covered.insert(format!("data:{tool_call_id}:/data/{item_index}/symbol"));
            }
        }

        let catalog = current_turn_fallback_evidence_catalog_excluding(
            &context,
            turn_message_start,
            &scope,
            &covered,
        );

        assert!(
            catalog.iter().any(|item| {
                item["tool_call_id"] == "tc_older_quote"
                    && item["json_pointer"] == "/data/0/price"
                    && item["value"] == 73.21
            }),
            "covered recent items consumed the cap before exclusion: {catalog:?}"
        );
        assert!(catalog.iter().all(|item| {
            item["tool_call_id"]
                .as_str()
                .is_none_or(|tool_call_id| !tool_call_id.starts_with("tc_covered_"))
        }));
    }

    #[test]
    fn snapshot_fallback_scans_quote_before_a_large_news_branch() {
        let mut context = AgentContext::new("snapshot-quote-first".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 怎么看");
        add_test_data_fetch_calls(&mut context, &[("tc_snapshot", "snapshot")]);
        let news = (0..300)
            .map(|index| {
                json!({
                    "date": format!("2026-07-{:02}", (index % 28) + 1),
                    "title": format!("unrelated news item {index}"),
                    "text": format!("large branch scalar {index}")
                })
            })
            .collect::<Vec<_>>();
        let payload = json!({
            "data": {
                "news": news,
                "quote": [{
                    "symbol": "CRWV",
                    "price": 73.21,
                    "currency": "USD",
                    "hone_quote_time": {"beijing": "2026-07-18 04:00"}
                }]
            }
        })
        .to_string();
        context.add_tool_result("tc_snapshot", "data_fetch", &payload);

        let scope = test_fallback_scope(&[("tc_snapshot", "/data/quote/0/price")], &[]);
        let catalog = current_turn_fallback_evidence_catalog(&context, turn_message_start, &scope);
        let pointers = catalog
            .iter()
            .filter_map(|item| item.get("json_pointer").and_then(Value::as_str))
            .collect::<Vec<_>>();

        assert!(pointers.contains(&"/data/quote/0/symbol"));
        assert!(pointers.contains(&"/data/quote/0/price"));
        assert!(pointers.contains(&"/data/quote/0/hone_quote_time/beijing"));
    }

    #[test]
    fn valid_web_fact_is_supplemented_when_a_quote_reference_is_bad() {
        let mut context = AgentContext::new("partial-handoff-fallback".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVIDIA 有什么关系");
        add_test_data_fetch_calls(
            &mut context,
            &[("tc_stale_nbis", "quote"), ("tc_quote", "quote")],
        );
        add_test_web_calls(&mut context, &[("tc_web", "CoreWeave NVIDIA relationship")]);
        context.add_tool_result(
            "tc_stale_nbis",
            "data_fetch",
            r#"{"data":[{"symbol":"NBIS","price":52.40,"marketCap":12600000000}]}"#,
        );
        context.add_tool_result(
            "tc_web",
            "web_search",
            r#"{"results":[{"title":"Commercial agreement","url":"https://example.test/agreement","content":"NVIDIA and CoreWeave announced a commercial agreement."}]}"#,
        );
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21,"currency":"USD"}]}"#,
        );
        let handoff = ResearchHandoff {
            answer_scope: "回答双方关系".to_string(),
            facts: vec![
                ResearchHandoffFact {
                    id: "F1".to_string(),
                    evidence: vec![ResearchEvidenceRef {
                        tool_call_id: "tc_web".to_string(),
                        result_number: Some(1),
                        exact_excerpt: Some(
                            "NVIDIA and CoreWeave announced a commercial agreement.".to_string(),
                        ),
                        json_pointer: None,
                    }],
                },
                ResearchHandoffFact {
                    id: "F2".to_string(),
                    evidence: vec![ResearchEvidenceRef {
                        tool_call_id: "tc_quote".to_string(),
                        result_number: None,
                        exact_excerpt: None,
                        json_pointer: Some("/data/0/missing_price".to_string()),
                    }],
                },
            ],
            inferences: Vec::new(),
            gaps: Vec::new(),
        };

        let validated = validate_finish_research_handoff(handoff, &context, turn_message_start);

        assert_eq!(validated.facts.len(), 1);
        assert!(validated.validation_warnings.iter().any(|warning| {
            warning.contains("json_pointer /data/0/missing_price does not exist")
        }));
        assert!(validated.fallback_evidence.iter().any(|item| {
            item["tool_call_id"] == "tc_quote"
                && item["json_pointer"] == "/data/0/price"
                && item["value"] == 73.21
        }));
        let serialized = serde_json::to_string(&validated).expect("serialize scoped handoff");
        assert!(!serialized.contains("tc_stale_nbis"));
        assert!(!serialized.contains("NBIS"));
    }

    #[test]
    fn structured_finish_handoff_resolves_only_current_turn_exact_provenance() {
        let mut context = AgentContext::new("structured-handoff".to_string());
        context.add_tool_result(
            "tc_old_web",
            "web_search",
            r#"{"results":[{"title":"Old","url":"https://old.test","content":"old relationship"}]}"#,
        );
        let turn_message_start = context.messages.len();
        context.add_user_message("CRWV 和 NVIDIA 有什么关系");
        add_test_data_fetch_calls(&mut context, &[("tc_quote", "quote")]);
        add_test_web_calls(
            &mut context,
            &[("tc_web_relationship", "CoreWeave NVIDIA relationship")],
        );
        context.add_tool_result(
            "tc_quote",
            "data_fetch",
            r#"{"data":[{"symbol":"CRWV","price":73.21}]}"#,
        );
        context.add_tool_result(
            "tc_web_relationship",
            "web_search",
            r#"{"results":[{"title":"Capacity purchase announcement","url":"https://example.test/capacity","content":"The buyer agreed to purchase $6.3B of unused capacity."},{"title":"Most-favored-nation relationship","url":"https://example.test/mfn","content":"The filing describes a most-favored-nation relationship."}]}"#,
        );
        let call = ToolCall {
            id: "tc_finish".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: FINISH_RESEARCH_TOOL_NAME.to_string(),
                arguments: json!({
                    "answer_scope": "回答双方关系",
                    "facts": [
                        {
                            "id": "F1",
                            "evidence": [
                                {
                                    "tool_call_id": "tc_web_relationship",
                                    "result_number": 1,
                                    "exact_excerpt": "The buyer agreed to purchase $6.3B of unused capacity."
                                },
                                {
                                    "tool_call_id": "tc_web_relationship",
                                    "result_number": 2,
                                    "exact_excerpt": "fabricated second excerpt"
                                }
                            ]
                        },
                        {
                            "id": "F2",
                            "evidence": [{
                                "tool_call_id": "tc_quote",
                                "json_pointer": "/data/0/price"
                            }]
                        }
                    ],
                    "inferences": [{
                        "claim": "双方关系不止一个维度。",
                        "premise_fact_ids": ["F1", "F2"]
                    }],
                    "gaps": ["持股比例未核验"]
                })
                .to_string(),
            },
        };

        let handoff = parse_finish_research_handoff(&call).expect("parse handoff");
        let validated = validate_finish_research_handoff(handoff, &context, turn_message_start);
        assert_eq!(validated.facts.len(), 2);
        assert_eq!(validated.facts[0].resolved_evidence[0]["result_number"], 1);
        assert_eq!(validated.facts[0].resolved_evidence.len(), 1);
        assert_eq!(validated.facts[1].resolved_evidence[0]["value"], 73.21);
        assert!(
            validated
                .validation_warnings
                .iter()
                .any(|warning| warning.contains("not a verbatim substring")),
            "one bad reference must be dropped without rejecting the good reference or handoff"
        );

        let stale = ToolCall {
            id: "tc_finish_stale".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: FINISH_RESEARCH_TOOL_NAME.to_string(),
                arguments: json!({
                    "answer_scope": "回答双方关系",
                    "facts": [{
                        "id": "F1",
                        "evidence": [{
                            "tool_call_id": "tc_old_web",
                            "result_number": 1,
                            "exact_excerpt": "old relationship"
                        }]
                    }],
                    "inferences": [],
                    "gaps": []
                })
                .to_string(),
            },
        };
        let stale_validated = validate_finish_research_handoff(
            parse_finish_research_handoff(&stale).expect("parse stale handoff"),
            &context,
            turn_message_start,
        );
        assert!(stale_validated.facts.is_empty());
        assert!(
            stale_validated
                .validation_warnings
                .iter()
                .any(|warning| warning.contains("not a current-turn result"))
        );
    }

    #[test]
    fn structured_finish_handoff_rejects_empty_or_fabricated_web_provenance() {
        let empty = ToolCall {
            id: "tc_empty".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: FINISH_RESEARCH_TOOL_NAME.to_string(),
                arguments: "{}".to_string(),
            },
        };
        assert!(parse_finish_research_handoff(&empty).is_err());

        let mut context = AgentContext::new("fabricated-handoff".to_string());
        let turn_message_start = context.messages.len();
        context.add_user_message("核验双方关系");
        add_test_web_calls(&mut context, &[("tc_web", "company relationship filing")]);
        context.add_tool_result(
            "tc_web",
            "web_search",
            r#"{"results":[{"title":"Partnership","url":"https://example.test/partnership","content":"The companies announced a collaboration."}]}"#,
        );
        let fabricated = ToolCall {
            id: "tc_finish".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: FINISH_RESEARCH_TOOL_NAME.to_string(),
                arguments: json!({
                    "answer_scope": "回答双方关系",
                    "facts": [{
                        "id": "F1",
                        "evidence": [{
                            "tool_call_id": "tc_web",
                            "result_number": 1,
                            "exact_excerpt": "no equity relationship"
                        }]
                    }],
                    "inferences": [],
                    "gaps": []
                })
                .to_string(),
            },
        };
        let validated = validate_finish_research_handoff(
            parse_finish_research_handoff(&fabricated).expect("parse fabricated handoff"),
            &context,
            turn_message_start,
        );
        assert!(validated.facts.is_empty());
        assert!(!validated.fallback_evidence.is_empty());
        assert!(
            validated
                .validation_warnings
                .iter()
                .any(|warning| warning.contains("not a verbatim substring"))
        );
    }

    #[test]
    fn tool_rounds_defer_prose_and_explicit_finish_owns_exact_final_contract() {
        let runtime_input = concat!(
            "【Session 上下文】\n当前时间：2026-07-19 09:31:42 (北京时间)\n\n",
            "【本轮用户输入】\ncrwv和英伟达有什么关系\n\n",
            "【本轮最终回答契约：由主 Agent 一次完成】\n",
            "第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。"
        );
        let prefix = exact_final_answer_prefix(runtime_input).expect("exact runtime prefix");
        assert_eq!(prefix, "数据时间：北京时间 2026-07-19 09:31；行情口径：");

        let route_guidance = "- entity_route=\"coreweave\": candidates=CRWV；结构取证已覆盖";
        let direct = active_business_turn_prompt(true, route_guidance, "source catalog", None);
        let evidence_pending =
            active_business_turn_prompt(false, route_guidance, "source catalog", None);
        let explicit = terminal_synthesis_prompt(Some(&prefix), &test_validated_handoff());
        assert!(direct.contains("本轮仍是工具轮，不写终稿"));
        assert!(direct.contains("answer_scope / facts / inferences / gaps"));
        assert!(direct.contains(route_guidance));
        assert!(evidence_pending.contains("本轮只取证，不作答"));
        assert!(evidence_pending.contains("本轮必须只返回一个或多个真实业务工具调用"));
        assert!(evidence_pending.contains("禁止输出数据时间、摘要、解释、草稿或最终正文"));
        assert!(evidence_pending.contains(route_guidance));
        for required in [
            prefix.as_str(),
            "quote 的 provider timestamp 只能写在‘行情口径’里",
            "来源标题与原始 URL 做内联引用",
            "URL 只用于定位来源，不证明句中内容",
            "title/content/snippet",
            "不得使用历史会话或模型记忆中的 URL",
            "以‘推断：’开头",
            "否定某种关系同样需要本轮来源直接支持",
            "严格服从结构化交接",
            "披露缺项并继续完成能够被当前证据支持的分析",
        ] {
            assert!(explicit.contains(required), "terminal missing {required}");
        }
        assert!(!direct.contains(prefix.as_str()));
        assert!(!evidence_pending.contains(prefix.as_str()));
        assert!(!direct.contains("不得由交易事实推导排名"));
        assert!(!evidence_pending.contains("不得由交易事实推导排名"));
        assert!(!direct.contains("数据时间：北京时间 2026-07-18 04:00；"));
        assert!(!evidence_pending.contains("数据时间：北京时间 2026-07-18 04:00；"));
        assert!(!explicit.contains("数据时间：北京时间 2026-07-18 04:00；"));
    }

    #[test]
    fn agent_owned_prompts_use_lowercase_tickers_and_natural_final_without_rewrite() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：";
        let route_guidance =
            "- entity_route=\"coreweave\": candidates=[\"CRWV\"]；结构调用已按同一候选代码成对尝试";
        let pending =
            agent_owned_business_turn_prompt(false, route_guidance, Some(prefix), false, false);
        let eligible =
            agent_owned_business_turn_prompt(true, route_guidance, Some(prefix), false, false);
        let forced =
            agent_owned_business_turn_prompt(true, route_guidance, Some(prefix), true, false);

        assert!(
            OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION
                .contains("用户可能用小写、混合大小写或带市场常用分隔符书写 ticker")
        );
        assert!(
            OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION
                .contains("结构化行情的工具选择优先级高于开放 Web 搜索")
        );
        assert!(
            OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION
                .contains("quote/profile/snapshot 等依赖标准 ticker 的调用必须等待 search")
        );
        assert!(OPEN_AGENT_ENTITY_DISCOVERY_SYSTEM_INSTRUCTION.contains("不是完成门禁"));
        assert!(pending.contains("小写或混合大小写代码应先规范成标准代码并走 exact_symbol"));
        assert!(pending.contains("优先调用 snapshot 获取完整行情与资产路由"));
        assert!(pending.contains("Web/news/filing/industry 检索"));
        assert!(pending.contains("行情优先只是工具选择信号，不是终稿门禁"));
        assert!(pending.contains("禁止猜测、补全或凭记忆构造代码"));
        assert!(pending.contains("由下一轮同一 Agent 继续取证或直接自然作答"));
        assert!(!pending.contains(prefix));
        assert!(eligible.contains(prefix));
        assert!(eligible.contains("直接生成一次完整自然终稿"));
        assert!(eligible.contains("让回答范围跟随用户原问题"));
        assert!(eligible.contains("`hone_quote_time.market_date_new_york`"));
        assert!(eligible.contains("绝不能据此写‘纽交所’或‘收盘价’"));
        assert!(eligible.contains("`hone_security_listing_evidence.status=active_listing`"));
        assert!(eligible.contains("不得再回答该证券未上市、已退市或应替换成旧母公司"));
        assert!(eligible.contains("高度依赖、锁定和多重绑定"));
        assert!(eligible.contains("EBIT、EBITA、EBITDA 与营业利润不得互相代替"));
        assert!(eligible.contains("不是逐数字双来源、缺项拒答或自动改写门禁"));
        for final_prompt in [&eligible, &forced] {
            assert!(final_prompt.contains("先锁定用户所指对象或市场范围与目标时段"));
            assert!(final_prompt.contains("不得因为 latest quote"));
            assert!(final_prompt.contains("宽基指数、行业板块与单只证券必须分开"));
            assert!(final_prompt.contains("不得擅自挑另一天的大跌替换问题"));
            assert!(final_prompt.contains("原因本轮未完全核验"));
        }
        for internal_marker in ["finish_research", "locator 纠正", "固定拒答", "回写阶段"]
        {
            assert!(!pending.contains(internal_marker));
            assert!(!eligible.contains(internal_marker));
        }
        assert!(!eligible.contains("answer_scope / facts / inferences / gaps"));
    }

    #[test]
    fn route_guidance_uses_raw_agent_keys_and_reports_same_symbol_gaps() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.identity_routes.insert(
            "route:coreweave".to_string(),
            ResearchIdentityRouteEvidence {
                explicit: true,
                identity_match_declared: true,
                search_attempts: 1,
                candidates: ["CRWV".to_string(), "CWY".to_string()]
                    .into_iter()
                    .collect(),
                quote_symbols: ["CRWV".to_string()].into_iter().collect(),
                asset_route_symbols: ["CWY".to_string()].into_iter().collect(),
                ..ResearchIdentityRouteEvidence::default()
            },
        );
        ledger.identity_routes.insert(
            "query:Ford".to_string(),
            ResearchIdentityRouteEvidence {
                search_attempts: 1,
                candidates: ["F".to_string()].into_iter().collect(),
                ..ResearchIdentityRouteEvidence::default()
            },
        );
        ledger.identity_routes.insert(
            "route:missing".to_string(),
            ResearchIdentityRouteEvidence {
                explicit: true,
                identity_match_declared: true,
                search_attempts: 2,
                empty_search_results: 2,
                post_identity_attempts: 1,
                ..ResearchIdentityRouteEvidence::default()
            },
        );

        let summary = ledger.agent_guidance_summary();

        assert!(summary.contains("entity_route=\"coreweave\""));
        assert!(!summary.contains("entity_route=\"route:coreweave\""));
        assert!(summary.contains("quote 与 profile/asset-route 尚未落在同一候选代码"));
        assert!(summary.contains("未绑定的 provisional query=\"Ford\""));
        assert!(summary.contains("需要用显式 entity_route 绑定该 search"));
        assert!(summary.contains("entity_route=\"missing\""));
        assert!(summary.contains("有界无覆盖调用已尝试"));
        let missing_line = summary
            .lines()
            .find(|line| line.contains("entity_route=\"missing\""))
            .expect("bounded no-coverage route line");
        assert!(!missing_line.contains("需要在同一路线 refinement"));
    }

    #[test]
    fn route_guidance_replays_crwv_nvidia_canary_missing_calls_concretely() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.identity_routes.insert(
            "route:coreweave".to_string(),
            ResearchIdentityRouteEvidence {
                explicit: true,
                identity_match_declared: true,
                search_attempts: 1,
                candidates: ["CRWV".to_string()].into_iter().collect(),
                quote_symbols: ["CRWV".to_string()].into_iter().collect(),
                ..ResearchIdentityRouteEvidence::default()
            },
        );
        ledger.identity_routes.insert(
            "route:nvidia".to_string(),
            ResearchIdentityRouteEvidence {
                explicit: true,
                identity_match_declared: true,
                search_attempts: 1,
                candidates: ["NVDA".to_string()].into_iter().collect(),
                ..ResearchIdentityRouteEvidence::default()
            },
        );

        let summary = ledger.agent_guidance_summary();
        let crwv = summary
            .lines()
            .find(|line| line.contains("entity_route=\"coreweave\""))
            .expect("CRWV route");
        let nvidia = summary
            .lines()
            .find(|line| line.contains("entity_route=\"nvidia\""))
            .expect("NVIDIA route");

        assert!(!crwv.contains("缺同路线同代码 quote"));
        assert!(crwv.contains("缺同路线同代码 profile/snapshot"));
        assert!(nvidia.contains("缺同路线同代码 quote"));
        assert!(nvidia.contains("缺同路线同代码 profile/snapshot"));
        assert!(!ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn quote_only_does_not_unlock_finish_for_discovered_security() {
        let search = ToolCall {
            id: "search".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "data_fetch".to_string(),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            },
        };
        let quote = ToolCall {
            id: "quote".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "data_fetch".to_string(),
                arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
            },
        };
        let profile = ToolCall {
            id: "profile".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "data_fetch".to_string(),
                arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
            },
        };
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}),
            false,
        );
        ledger.observe_business_call(&quote, true);
        assert!(!ledger.evidence_floor_satisfied(true));
        ledger.observe_business_call(&profile, true);
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn every_successful_identity_candidate_set_needs_quote_and_profile_coverage() {
        let crwv_search = evidence_call("search-crwv", r#"{"data_type":"search","query":"CRWV"}"#);
        let nvda_search =
            evidence_call("search-nvda", r#"{"data_type":"search","query":"NVIDIA"}"#);
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&crwv_search, false);
        ledger.observe_business_result(
            &crwv_search,
            &json!({"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}),
            false,
        );
        ledger.observe_business_call(&nvda_search, false);
        ledger.observe_business_result(
            &nvda_search,
            &json!({"data":[{"symbol":"NVDA"},{"symbol":"NVD.DE"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call("quote-crwv", r#"{"data_type":"quote","symbol":"CRWV"}"#),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile-crwv", r#"{"data_type":"profile","symbol":"CRWV"}"#),
            true,
        );

        assert!(!ledger.evidence_floor_satisfied(true));
        assert!(!ledger.completion_signal_available(true));

        ledger.observe_business_call(
            &evidence_call("quote-nvda", r#"{"data_type":"quote","symbol":"NVDA"}"#),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile-nvda", r#"{"data_type":"profile","symbol":"NVDA"}"#),
            true,
        );

        assert!(ledger.evidence_floor_satisfied(true));
        assert!(ledger.completion_signal_available(true));
    }

    #[test]
    fn agent_declared_routes_prevent_cross_entity_and_wrong_product_unlocks() {
        let crwv_search = evidence_call(
            "search-crwv",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let nvidia_search = evidence_call(
            "search-nvidia",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&crwv_search, false);
        ledger.observe_business_result(
            &crwv_search,
            &json!({"data":[
                {"symbol":"CRWV","name":"CoreWeave, Inc."},
                {"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF"}
            ]}),
            false,
        );
        ledger.observe_business_call(&nvidia_search, false);
        ledger.observe_business_result(
            &nvidia_search,
            &json!({"data":[
                {"symbol":"CRWV","name":"unrelated provider noise"},
                {"symbol":"CWY","name":"unrelated provider noise"},
                {"symbol":"NVDA","name":"NVIDIA Corporation"},
                {"symbol":"NVD.DE","name":"NVIDIA Corporation"}
            ]}),
            false,
        );

        let crwv_route = ledger
            .identity_routes
            .get("route:coreweave")
            .expect("coreweave route");
        assert_eq!(crwv_route.candidates, BTreeSet::from(["CRWV".to_string()]));
        let nvidia_route = ledger
            .identity_routes
            .get("route:nvidia")
            .expect("nvidia route");
        assert_eq!(
            nvidia_route.candidates,
            BTreeSet::from(["NVDA".to_string(), "NVD.DE".to_string()])
        );

        // A complete CWY quote/profile pair is still the wrong asset for the
        // exact CRWV route and must not unlock finish.
        ledger.observe_business_call(
            &evidence_call(
                "quote-cwy",
                r#"{"data_type":"quote","symbol":"CWY","entity_route":"coreweave"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-cwy",
                r#"{"data_type":"profile","symbol":"CWY","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "CWY evidence cannot satisfy the exact CRWV route"
        );

        ledger.observe_business_call(
            &evidence_call(
                "quote-crwv",
                r#"{"data_type":"quote","symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-crwv",
                r#"{"data_type":"profile","symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "CoreWeave coverage cannot satisfy the separate NVIDIA route"
        );

        ledger.observe_business_call(
            &evidence_call(
                "quote-nvda",
                r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-nvda",
                r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));

        let only_wrong_product = evidence_call(
            "search-only-cwy",
            r#"{"data_type":"search","query":"crwv","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let mut exact_only = ResearchEvidenceLedger::default();
        exact_only.observe_business_call(&only_wrong_product, false);
        exact_only.observe_business_result(
            &only_wrong_product,
            &json!({"data":[{
                "symbol":"CWY",
                "name":"GraniteShares YieldBOOST CRWV ETF"
            }]}),
            false,
        );
        assert!(
            exact_only
                .identity_routes
                .get("route:coreweave")
                .expect("coreweave route")
                .candidates
                .is_empty(),
            "an embedded-name CWY result cannot replace a missing exact CRWV result"
        );
        let name_refinement = evidence_call(
            "search-coreweave-only-cwy",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","refines_query":"crwv"}"#,
        );
        exact_only.observe_business_call(&name_refinement, true);
        exact_only.observe_business_result(
            &name_refinement,
            &json!({"data":[{
                "symbol":"CWY",
                "name":"GraniteShares YieldBOOST CRWV ETF"
            }]}),
            true,
        );
        let wrong_exact_retry = evidence_call(
            "search-cwy-wrong-exact",
            r#"{"data_type":"search","query":"CWY","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        exact_only.observe_business_call(&wrong_exact_retry, true);
        exact_only.observe_business_result(
            &wrong_exact_retry,
            &json!({"data":[{"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF"}]}),
            true,
        );
        let exact_route = exact_only
            .identity_routes
            .get("route:coreweave")
            .expect("coreweave exact route");
        assert_eq!(
            exact_route.exact_symbol_constraint.as_deref(),
            Some("CRWV"),
            "a later different exact query cannot widen the first ticker constraint"
        );
        assert!(
            exact_route.candidates.is_empty(),
            "company-name refinement or a different exact ticker cannot revive CWY"
        );
        exact_only.observe_business_call(
            &evidence_call(
                "quote-only-cwy",
                r#"{"data_type":"quote","symbol":"CWY","entity_route":"coreweave"}"#,
            ),
            true,
        );
        exact_only.observe_business_call(
            &evidence_call(
                "profile-only-cwy",
                r#"{"data_type":"profile","symbol":"CWY","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(!exact_only.evidence_floor_satisfied(true));

        let coreweave_name = evidence_call(
            "search-coreweave-name",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias"}"#,
        );
        let mut explicit_name = ResearchEvidenceLedger::default();
        explicit_name.observe_business_call(&coreweave_name, false);
        explicit_name.observe_business_result(
            &coreweave_name,
            &json!({"data":[{
                "symbol":"CWY",
                "name":"GraniteShares YieldBOOST CRWV ETF"
            }]}),
            false,
        );
        assert!(
            explicit_name
                .identity_routes
                .get("route:coreweave")
                .expect("name route")
                .candidates
                .is_empty(),
            "explicit name matching cannot fall back to an ungrounded product"
        );

        // Query strings are never split into entities, so legal names/share
        // classes remain one Agent-declared route.
        for query in [
            "AT&T",
            "S&P Global",
            "M&T Bank",
            "H&R Block",
            "BRK/B",
            "Berkshire Hathaway, Class B",
            "NVIDIA and valuation",
        ] {
            let call = evidence_call(
                "legal-name",
                &json!({
                    "data_type":"search",
                    "query":query,
                    "entity_route":"single-company",
                    "identity_match":"name_or_alias"
                })
                .to_string(),
            );
            let mut single = ResearchEvidenceLedger::default();
            single.observe_business_call(&call, false);
            single.observe_business_result(
                &call,
                &json!({"data":[{"symbol":"T","name":"AT&T Inc."}]}),
                false,
            );
            assert_eq!(single.identity_routes.len(), 1, "query: {query}");
        }
    }

    #[test]
    fn agent_declared_match_mode_preserves_short_company_names_and_provider_symbol_aliases() {
        for (query, expected, rows) in [
            (
                "ford",
                "F",
                json!({"data":[
                    {"symbol":"FORD","name":"Forward Industries, Inc."},
                    {"symbol":"F","name":"Ford Motor Company"}
                ]}),
            ),
            (
                "apple",
                "AAPL",
                json!({"data":[
                    {"symbol":"AAPL","name":"Apple Inc."},
                    {"symbol":"APPLX","name":"Appleseed Fund"}
                ]}),
            ),
            (
                "tesla",
                "TSLA",
                json!({"data":[
                    {"symbol":"TSLA","name":"Tesla, Inc."},
                    {"symbol":"TSLZ","name":"T-Rex 2X Inverse Tesla Daily Target ETF"}
                ]}),
            ),
        ] {
            let call = evidence_call(
                "short-name",
                &json!({
                    "data_type":"search",
                    "query":query,
                    "entity_route":"company",
                    "identity_match":"name_or_alias"
                })
                .to_string(),
            );
            let mut ledger = ResearchEvidenceLedger::default();
            ledger.observe_business_call(&call, false);
            ledger.observe_business_result(&call, &rows, false);
            assert_eq!(
                ledger
                    .identity_routes
                    .get("route:company")
                    .expect("company route")
                    .candidates,
                BTreeSet::from([expected.to_string()]),
                "query: {query}"
            );
        }

        let ford_ticker = evidence_call(
            "ford-ticker",
            r#"{"data_type":"search","query":"FORD","entity_route":"forward-industries","identity_match":"exact_symbol"}"#,
        );
        let mut exact_ford = ResearchEvidenceLedger::default();
        exact_ford.observe_business_call(&ford_ticker, false);
        exact_ford.observe_business_result(
            &ford_ticker,
            &json!({"data":[
                {"symbol":"FORD","name":"Forward Industries, Inc."},
                {"symbol":"F","name":"Ford Motor Company"}
            ]}),
            false,
        );
        assert_eq!(
            exact_ford
                .identity_routes
                .get("route:forward-industries")
                .expect("FORD route")
                .candidates,
            BTreeSet::from(["FORD".to_string()])
        );

        let brk = evidence_call(
            "brk-class-b",
            r#"{"data_type":"search","query":"BRK/B","entity_route":"berkshire-b","identity_match":"exact_symbol"}"#,
        );
        let mut share_class = ResearchEvidenceLedger::default();
        share_class.observe_business_call(&brk, false);
        share_class.observe_business_result(
            &brk,
            &json!({"data":[{"symbol":"BRK-B","name":"Berkshire Hathaway Inc."}]}),
            false,
        );
        share_class.observe_business_call(
            &evidence_call(
                "brk-quote",
                r#"{"data_type":"quote","symbol":"BRK/B","entity_route":"berkshire-b"}"#,
            ),
            true,
        );
        share_class.observe_business_call(
            &evidence_call(
                "brk-profile",
                r#"{"data_type":"profile","symbol":"BRK.B","entity_route":"berkshire-b"}"#,
            ),
            true,
        );
        assert!(
            share_class.evidence_floor_satisfied(true),
            "bounded provider separators must preserve one BRK/B route"
        );

        let valid_nvidia = evidence_call(
            "nvidia-valid-name",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias"}"#,
        );
        let missing_match_mode = evidence_call(
            "same-route-missing-match-mode",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"nvidia"}"#,
        );
        let mut call_scoped_mode = ResearchEvidenceLedger::default();
        call_scoped_mode.observe_business_call(&valid_nvidia, false);
        call_scoped_mode.observe_business_result(
            &valid_nvidia,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            false,
        );
        call_scoped_mode.observe_business_call(&missing_match_mode, true);
        call_scoped_mode.observe_business_result(
            &missing_match_mode,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        let route = call_scoped_mode
            .identity_routes
            .get("route:nvidia")
            .expect("NVIDIA route");
        assert_eq!(route.search_attempts, 1);
        assert_eq!(route.candidates, BTreeSet::from(["NVDA".to_string()]));
        call_scoped_mode.observe_business_call(
            &evidence_call(
                "wrong-quote-after-missing-mode",
                r#"{"data_type":"quote","symbol":"CRWV","entity_route":"nvidia"}"#,
            ),
            true,
        );
        call_scoped_mode.observe_business_call(
            &evidence_call(
                "wrong-profile-after-missing-mode",
                r#"{"data_type":"profile","symbol":"CRWV","entity_route":"nvidia"}"#,
            ),
            true,
        );
        assert!(
            !call_scoped_mode.evidence_floor_satisfied(true),
            "a sticky old match declaration cannot authorize a later malformed search"
        );

        let missing_mode_untagged_refinement = evidence_call(
            "untagged-missing-mode-refinement",
            r#"{"data_type":"search","query":"CoreWeave","refines_query":"NVIDIA"}"#,
        );
        call_scoped_mode.observe_business_call(&missing_mode_untagged_refinement, true);
        call_scoped_mode.observe_business_result(
            &missing_mode_untagged_refinement,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        let route = call_scoped_mode
            .identity_routes
            .get("route:nvidia")
            .expect("NVIDIA route after malformed untagged refinement");
        assert_eq!(route.search_attempts, 1);
        assert_eq!(route.candidates, BTreeSet::from(["NVDA".to_string()]));
    }

    #[test]
    fn explicit_route_does_not_hide_an_unrelated_implicit_route() {
        let crwv_search = evidence_call(
            "search-crwv-legacy",
            r#"{"data_type":"search","query":"CRWV"}"#,
        );
        let nvidia_search = evidence_call(
            "search-nvidia",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&crwv_search, false);
        ledger.observe_business_result(
            &crwv_search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            false,
        );
        ledger.observe_business_call(&nvidia_search, false);
        ledger.observe_business_result(
            &nvidia_search,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "quote-nvda",
                r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-nvda",
                r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "one explicit NVIDIA route must not globally hide an untagged CRWV route"
        );

        ledger.observe_business_call(
            &evidence_call("quote-crwv", r#"{"data_type":"quote","symbol":"CRWV"}"#),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile-crwv", r#"{"data_type":"profile","symbol":"CRWV"}"#),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));

        let explicit_crwv = evidence_call(
            "search-crwv-explicit",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        ledger.observe_business_call(&explicit_crwv, true);
        assert!(
            !ledger.identity_routes.contains_key("query:CRWV"),
            "only the exact provisional CRWV route should migrate"
        );
        assert!(ledger.identity_routes.contains_key("route:coreweave"));
        assert!(ledger.identity_routes.contains_key("route:nvidia"));

        let late_untagged_crwv = evidence_call(
            "search-crwv-late-untagged",
            r#"{"data_type":"search","query":"CRWV","identity_match":"exact_symbol"}"#,
        );
        ledger.observe_business_call(&late_untagged_crwv, true);
        ledger.observe_business_result(
            &late_untagged_crwv,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        assert!(
            !ledger.identity_routes.contains_key("query:CRWV"),
            "a later untagged exact alias must bind back to the unique explicit route"
        );

        let legacy_alias = evidence_call(
            "search-nvidia-legacy-cn",
            r#"{"data_type":"search","query":"英伟达"}"#,
        );
        let explicit_alias = evidence_call(
            "search-nvidia-explicit-en",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias","supersedes_query":"英伟达"}"#,
        );
        let mut alias_migration = ResearchEvidenceLedger::default();
        alias_migration.observe_business_call(&legacy_alias, false);
        alias_migration.observe_business_result(
            &legacy_alias,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            false,
        );
        alias_migration.observe_business_call(&explicit_alias, true);
        alias_migration.observe_business_result(
            &explicit_alias,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            true,
        );
        assert!(
            !alias_migration.identity_routes.contains_key("query:英伟达"),
            "supersedes_query must migrate one successful provisional alias"
        );
        assert_eq!(alias_migration.identity_routes.len(), 1);
        alias_migration.observe_business_call(
            &evidence_call(
                "quote-nvda-migrated",
                r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        alias_migration.observe_business_call(
            &evidence_call(
                "profile-nvda-migrated",
                r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        assert!(alias_migration.evidence_floor_satisfied(true));
    }

    #[test]
    fn exact_route_migration_drops_wrong_provisional_evidence_even_when_retry_fails() {
        let exact_crwv = evidence_call(
            "search-crwv-exact-empty",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let provisional_cwy = evidence_call(
            "search-cwy-provisional",
            r#"{"data_type":"search","query":"CWY"}"#,
        );
        let superseding_retry = evidence_call(
            "search-coreweave-supersedes-cwy",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","supersedes_query":"CWY"}"#,
        );

        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&exact_crwv, false);
        ledger.observe_business_result(&exact_crwv, &json!({"data":[]}), false);
        ledger.observe_business_call(&provisional_cwy, true);
        ledger.observe_business_result(
            &provisional_cwy,
            &json!({"data":[{"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF"}]}),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "quote-cwy-provisional",
                r#"{"data_type":"quote","symbol":"CWY"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-cwy-provisional",
                r#"{"data_type":"profile","symbol":"CWY"}"#,
            ),
            true,
        );
        assert!(
            ledger
                .identity_routes
                .get("query:CWY")
                .expect("provisional CWY route")
                .is_covered()
        );

        ledger.observe_business_call(&superseding_retry, true);
        ledger.observe_business_failure(&superseding_retry);

        let route = ledger
            .identity_routes
            .get("route:coreweave")
            .expect("migrated CoreWeave route");
        assert_eq!(route.exact_symbol_constraint.as_deref(), Some("CRWV"));
        assert!(route.candidates.is_empty());
        assert!(route.quote_symbols.is_empty());
        assert!(route.asset_route_symbols.is_empty());
        assert_eq!(route.post_identity_attempts, 0);
        assert!(!route.is_covered());
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "a failed superseding search cannot convert provisional CWY evidence into CRWV coverage"
        );

        let name_superseding_retry = evidence_call(
            "search-coreweave-name-supersedes-cwy",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","supersedes_query":"CWY"}"#,
        );
        let mut name_route = ResearchEvidenceLedger::default();
        name_route.observe_business_call(&provisional_cwy, false);
        name_route.observe_business_result(
            &provisional_cwy,
            &json!({"data":[{"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF"}]}),
            false,
        );
        name_route.observe_business_call(
            &evidence_call(
                "quote-cwy-name-route",
                r#"{"data_type":"quote","symbol":"CWY"}"#,
            ),
            true,
        );
        name_route.observe_business_call(
            &evidence_call(
                "profile-cwy-name-route",
                r#"{"data_type":"profile","symbol":"CWY"}"#,
            ),
            true,
        );
        name_route.observe_business_call(&name_superseding_retry, true);
        name_route.observe_business_failure(&name_superseding_retry);
        let route = name_route
            .identity_routes
            .get("route:coreweave")
            .expect("failed name route");
        assert!(route.candidates.is_empty());
        assert!(!route.is_covered());
        assert!(
            !name_route.evidence_floor_satisfied(true),
            "a failed first explicit name search cannot inherit a provisional product candidate"
        );
    }

    #[test]
    fn explicit_refinement_inherits_only_a_declared_exact_constraint_from_provisional_route() {
        let provisional_exact = evidence_call(
            "search-crwv-provisional-exact",
            r#"{"data_type":"search","query":"CRWV","identity_match":"exact_symbol"}"#,
        );
        let explicit_refinement = evidence_call(
            "search-coreweave-explicit-refinement",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","supersedes_query":"CRWV"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&provisional_exact, false);
        ledger.observe_business_result(
            &provisional_exact,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "quote-crwv-provisional",
                r#"{"data_type":"quote","ticker":"CRWV"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-crwv-provisional",
                r#"{"data_type":"profile","ticker":"CRWV"}"#,
            ),
            true,
        );
        assert!(
            ledger
                .identity_routes
                .get("query:CRWV")
                .expect("provisional exact route")
                .is_covered()
        );

        ledger.observe_business_call(&explicit_refinement, true);
        ledger.observe_business_result(
            &explicit_refinement,
            &json!({"data":[{
                "symbol":"CWY",
                "name":"GraniteShares YieldBOOST CRWV ETF"
            }]}),
            true,
        );

        let route = ledger
            .identity_routes
            .get("route:coreweave")
            .expect("explicit CoreWeave route");
        assert_eq!(route.exact_symbol_constraint.as_deref(), Some("CRWV"));
        assert!(route.candidates.is_empty());
        assert!(route.quote_symbols.is_empty());
        assert!(route.asset_route_symbols.is_empty());
        assert_eq!(route.post_identity_attempts, 0);
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "migration may retain the Agent-declared ticker constraint, but never provisional candidates or coverage"
        );
    }

    #[test]
    fn ambiguous_migration_links_leave_every_provisional_route_untouched() {
        let provisional_crwv = evidence_call(
            "search-crwv-provisional",
            r#"{"data_type":"search","query":"CRWV","identity_match":"exact_symbol"}"#,
        );
        let provisional_nvidia = evidence_call(
            "search-nvidia-provisional",
            r#"{"data_type":"search","query":"NVIDIA","identity_match":"name_or_alias"}"#,
        );
        let invalid_multi_link = evidence_call(
            "search-invalid-multi-link",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","refines_query":"CRWV","supersedes_query":"NVIDIA"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        for (search, symbol, name) in [
            (&provisional_crwv, "CRWV", "CoreWeave, Inc."),
            (&provisional_nvidia, "NVDA", "NVIDIA Corporation"),
        ] {
            ledger.observe_business_call(search, false);
            ledger.observe_business_result(
                search,
                &json!({"data":[{"symbol":symbol,"name":name}]}),
                false,
            );
            ledger.observe_business_call(
                &evidence_call(
                    &format!("quote-{symbol}"),
                    &json!({"data_type":"quote","ticker":symbol}).to_string(),
                ),
                true,
            );
            ledger.observe_business_call(
                &evidence_call(
                    &format!("profile-{symbol}"),
                    &json!({"data_type":"profile","ticker":symbol}).to_string(),
                ),
                true,
            );
        }
        assert!(ledger.evidence_floor_satisfied(true));

        ledger.observe_business_call(&invalid_multi_link, true);
        ledger.observe_business_result(
            &invalid_multi_link,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        assert!(ledger.identity_routes.contains_key("query:CRWV"));
        assert!(ledger.identity_routes.contains_key("query:NVIDIA"));
        assert!(!ledger.identity_routes.contains_key("route:coreweave"));
        assert!(
            ledger.evidence_floor_satisfied(true),
            "a malformed route declaration must neither migrate nor poison covered entities"
        );

        let same_text_double_link = evidence_call(
            "search-invalid-same-text-double-link",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"same-text","identity_match":"name_or_alias","refines_query":"CRWV","supersedes_query":"CRWV"}"#,
        );
        assert!(!data_fetch_identity_search_shape_is_valid(
            &same_text_double_link
        ));
        let mut same_text = ResearchEvidenceLedger::default();
        same_text.observe_business_call(&provisional_crwv, false);
        same_text.observe_business_result(
            &provisional_crwv,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            false,
        );
        same_text.observe_business_call(&same_text_double_link, true);
        assert!(same_text.identity_routes.contains_key("query:CRWV"));
        assert!(!same_text.identity_routes.contains_key("route:same-text"));
        assert_eq!(same_text.identity_routes.len(), 1);
    }

    #[test]
    fn exact_text_migration_keeps_ford_company_and_ford_ticker_routes_distinct() {
        let ford_company = evidence_call(
            "search-ford-company",
            r#"{"data_type":"search","query":"Ford","identity_match":"name_or_alias"}"#,
        );
        let ford_ticker = evidence_call(
            "search-ford-ticker",
            r#"{"data_type":"search","query":"FORD","identity_match":"exact_symbol"}"#,
        );
        let explicit_company = evidence_call(
            "search-ford-motor-explicit",
            r#"{"data_type":"search","query":"Ford Motor","entity_route":"ford-motor","identity_match":"name_or_alias","supersedes_query":"Ford"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&ford_company, false);
        ledger.observe_business_result(
            &ford_company,
            &json!({"data":[{"symbol":"F","name":"Ford Motor Company"}]}),
            false,
        );
        ledger.observe_business_call(&ford_ticker, false);
        ledger.observe_business_result(
            &ford_ticker,
            &json!({"data":[{"symbol":"FORD","name":"Forward Industries, Inc."}]}),
            false,
        );
        assert!(ledger.identity_routes.contains_key("query:Ford"));
        assert!(ledger.identity_routes.contains_key("query:FORD"));

        ledger.observe_business_call(&explicit_company, true);
        ledger.observe_business_result(
            &explicit_company,
            &json!({"data":[{"symbol":"F","name":"Ford Motor Company"}]}),
            true,
        );
        assert!(!ledger.identity_routes.contains_key("query:Ford"));
        assert!(
            ledger.identity_routes.contains_key("query:FORD"),
            "case-sensitive exact text linkage must not merge a company name with a different ticker"
        );

        let explicit_ford_company = evidence_call(
            "search-explicit-ford-company",
            r#"{"data_type":"search","query":"Ford","entity_route":"Ford","identity_match":"name_or_alias"}"#,
        );
        let explicit_ford_ticker = evidence_call(
            "search-explicit-ford-ticker",
            r#"{"data_type":"search","query":"FORD","entity_route":"FORD","identity_match":"exact_symbol"}"#,
        );
        let mut explicit_case = ResearchEvidenceLedger::default();
        explicit_case.observe_business_call(&explicit_ford_company, false);
        explicit_case.observe_business_result(
            &explicit_ford_company,
            &json!({"data":[{"symbol":"F","name":"Ford Motor Company"}]}),
            false,
        );
        explicit_case.observe_business_call(&explicit_ford_ticker, false);
        explicit_case.observe_business_result(
            &explicit_ford_ticker,
            &json!({"data":[{"symbol":"FORD","name":"Forward Industries, Inc."}]}),
            false,
        );
        assert!(explicit_case.identity_routes.contains_key("route:Ford"));
        assert!(explicit_case.identity_routes.contains_key("route:FORD"));
        assert_eq!(explicit_case.identity_routes.len(), 2);
    }

    #[test]
    fn stable_entity_route_refinement_replaces_empty_or_noisy_candidates() {
        let crwv_search = evidence_call(
            "search-crwv",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let nvidia_alias_search = evidence_call(
            "search-nvidia-cn",
            r#"{"data_type":"search","query":"英伟达","entity_route":"nvidia","identity_match":"name_or_alias"}"#,
        );
        let wrong_nvidia_refinement = evidence_call(
            "search-nvidia-wrong",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"nvidia","identity_match":"name_or_alias"}"#,
        );
        let nvidia_refinement = evidence_call(
            "search-nvidia",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias","refines_query":"英伟达"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&crwv_search, false);
        ledger.observe_business_result(
            &crwv_search,
            &json!({"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}),
            false,
        );
        ledger.observe_business_call(&nvidia_alias_search, false);
        ledger.observe_business_result(&nvidia_alias_search, &json!({"data":[]}), false);
        ledger.observe_business_call(
            &evidence_call(
                "quote-crwv",
                r#"{"data_type":"quote","symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-crwv",
                r#"{"data_type":"profile","symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "an empty NVIDIA route cannot disappear behind CoreWeave coverage"
        );

        ledger.observe_business_call(&wrong_nvidia_refinement, true);
        ledger.observe_business_result(
            &wrong_nvidia_refinement,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "a noisy route result has no NVIDIA quote/profile coverage"
        );

        ledger.observe_business_call(&nvidia_refinement, true);
        ledger.observe_business_result(
            &nvidia_refinement,
            &json!({"data":[
                {"symbol":"NVDA","name":"NVIDIA Corporation"},
                {"symbol":"NVD.DE","name":"NVIDIA Corporation"},
                {"symbol":"TSLA","name":"unrelated provider noise"}
            ]}),
            true,
        );
        assert_eq!(
            ledger
                .identity_routes
                .get("route:nvidia")
                .expect("nvidia route")
                .candidates,
            BTreeSet::from(["NVDA".to_string(), "NVD.DE".to_string()])
        );
        ledger.observe_business_call(
            &evidence_call(
                "quote-nvda",
                r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-nvda",
                r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn each_identity_route_needs_one_same_symbol_quote_and_profile_pair() {
        let search = evidence_call("search", r#"{"data_type":"search","query":"CRWV"}"#);
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call("quote", r#"{"data_type":"quote","symbol":"CRWV"}"#),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("wrong-profile", r#"{"data_type":"profile","symbol":"CWY"}"#),
            true,
        );
        assert!(!ledger.evidence_floor_satisfied(true));

        ledger.observe_business_call(
            &evidence_call(
                "right-profile",
                r#"{"data_type":"profile","symbol":"CRWV"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn finance_identity_route_limit_applies_to_the_first_batch_and_rejects_without_poisoning() {
        let mut ledger = ResearchEvidenceLedger::with_identity_route_limit(Some(6));
        let mut rejected = Vec::new();

        for index in 0..10 {
            let symbol = format!("SYM{index}");
            let route = format!("candidate-{index}");
            let call = evidence_call(
                &format!("search-{index}"),
                &json!({
                    "data_type": "search",
                    "query": symbol,
                    "entity_route": route,
                    "identity_match": "exact_symbol"
                })
                .to_string(),
            );
            if let Some(error) = ledger.identity_search_admission_error(&call) {
                rejected.push((route, error));
                continue;
            }
            ledger.observe_business_call(&call, false);
            ledger.observe_business_result(&call, &json!({"data":[{"symbol":symbol}]}), false);
        }

        assert_eq!(ledger.identity_routes.len(), 6);
        assert_eq!(ledger.rejected_identity_routes, 4);
        assert_eq!(rejected.len(), 4);
        for (route, error) in rejected {
            assert_eq!(error["error"], "identity_route_limit_reached");
            assert_eq!(error["limit"], 6);
            assert!(
                !ledger
                    .identity_routes
                    .contains_key(&format!("route:{route}"))
            );
        }
    }

    #[test]
    fn unknown_route_aliases_merge_by_unique_symbol_without_creating_orphans() {
        let search = evidence_call(
            "search-runze",
            r#"{"data_type":"search","query":"润泽科技","entity_route":"润泽科技","identity_match":"name_or_alias"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"300442.SZ","name":"润泽科技"}]}),
            false,
        );

        ledger.observe_business_call(
            &evidence_call(
                "quote-runze-alias",
                r#"{"data_type":"quote","ticker":"300442.SZ","entity_route":"runze_tech"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-runze-alias",
                r#"{"data_type":"profile","ticker":"300442.SZ","entity_route":"runze"}"#,
            ),
            true,
        );

        assert_eq!(ledger.identity_routes.len(), 1);
        assert!(ledger.identity_routes.contains_key("route:润泽科技"));
        assert!(!ledger.identity_routes.contains_key("route:runze_tech"));
        assert!(!ledger.identity_routes.contains_key("route:runze"));
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn route_bound_evidence_before_that_routes_search_never_preloads_coverage() {
        let search_a = evidence_call(
            "search-a",
            r#"{"data_type":"search","query":"AAAA","entity_route":"a","identity_match":"exact_symbol"}"#,
        );
        let search_b = evidence_call(
            "search-b",
            r#"{"data_type":"search","query":"BBBB","entity_route":"b","identity_match":"exact_symbol"}"#,
        );
        let pre_search_b = evidence_call(
            "snapshot-b-before-search",
            r#"{"data_type":"snapshot","ticker":"BBBB","entity_route":"b"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search_a, false);
        ledger.observe_business_result(
            &search_a,
            &json!({"data":[{"symbol":"AAAA","name":"AAAA Corp."}]}),
            false,
        );
        ledger.observe_business_call(&pre_search_b, true);
        assert!(
            !ledger.identity_routes.contains_key("route:b"),
            "out-of-order evidence must not create an orphan route"
        );
        ledger.observe_business_call(
            &evidence_call(
                "quote-a",
                r#"{"data_type":"quote","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "profile-a",
                r#"{"data_type":"profile","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        assert!(
            ledger.evidence_floor_satisfied(true),
            "unknown route evidence cannot poison a covered route"
        );
        ledger.observe_business_call(&search_b, true);
        ledger.observe_business_result(
            &search_b,
            &json!({"data":[{"symbol":"BBBB","name":"BBBB Corp."}]}),
            true,
        );
        let b = ledger
            .identity_routes
            .get("route:b")
            .expect("searched B route");
        assert!(b.quote_symbols.is_empty());
        assert!(b.asset_route_symbols.is_empty());
        assert_eq!(b.post_identity_attempts, 0);
        assert!(
            !b.is_covered(),
            "B evidence observed before B search cannot become later B coverage"
        );
        assert!(!ledger.evidence_floor_satisfied(true));
        ledger.observe_business_call(&pre_search_b, true);
        assert!(ledger.evidence_floor_satisfied(true));

        let mut empty_b = ResearchEvidenceLedger::default();
        empty_b.observe_business_call(&search_a, false);
        empty_b.observe_business_result(
            &search_a,
            &json!({"data":[{"symbol":"AAAA","name":"AAAA Corp."}]}),
            false,
        );
        empty_b.observe_business_call(&pre_search_b, true);
        for _ in 0..2 {
            empty_b.observe_business_call(&search_b, true);
            empty_b.observe_business_result(&search_b, &json!({"data":[]}), true);
        }
        let b = empty_b
            .identity_routes
            .get("route:b")
            .expect("empty B route");
        assert_eq!(b.post_identity_attempts, 0);
        assert!(
            !b.has_bounded_no_coverage(),
            "pre-search evidence cannot satisfy an empty route's post-search attempt"
        );

        let mut before_any_search = ResearchEvidenceLedger::default();
        before_any_search.observe_business_call(&pre_search_b, false);
        before_any_search.observe_business_call(&search_a, true);
        before_any_search.observe_business_result(
            &search_a,
            &json!({"data":[{"symbol":"AAAA","name":"AAAA Corp."}]}),
            true,
        );
        before_any_search.observe_business_call(
            &evidence_call(
                "quote-a-after-pending-b",
                r#"{"data_type":"quote","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        before_any_search.observe_business_call(
            &evidence_call(
                "profile-a-after-pending-b",
                r#"{"data_type":"profile","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        assert!(!before_any_search.identity_routes.contains_key("route:b"));
        assert!(
            before_any_search.evidence_floor_satisfied(true),
            "only a valid identity search may create a route in the evidence floor"
        );
    }

    #[test]
    fn malformed_or_missing_mode_searches_do_not_create_routes_or_poison_floor() {
        let valid_a = evidence_call(
            "search-a-valid",
            r#"{"data_type":"search","query":"AAAA","entity_route":"a","identity_match":"exact_symbol"}"#,
        );
        let missing_query_b = evidence_call(
            "search-b-null-query",
            r#"{"data_type":"search","query":null,"ticker":"BBBB","entity_route":"b","identity_match":"exact_symbol"}"#,
        );
        let missing_mode_c = evidence_call(
            "search-c-missing-mode",
            r#"{"data_type":"search","query":"CCCC","entity_route":"c"}"#,
        );
        let invalid_untagged = evidence_call(
            "search-d-invalid-mode",
            r#"{"data_type":"search","query":"DDDD","identity_match":"tickerish"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&valid_a, false);
        ledger.observe_business_result(
            &valid_a,
            &json!({"data":[{"symbol":"AAAA","name":"AAAA Corp."}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "snapshot-a",
                r#"{"data_type":"snapshot","ticker":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));

        for invalid in [&missing_query_b, &missing_mode_c, &invalid_untagged] {
            ledger.observe_business_call(invalid, true);
            ledger.observe_business_result(
                invalid,
                &json!({"data":[{"symbol":"DDDD","name":"wrongly supplied row"}]}),
                true,
            );
        }
        for key in ["route:b", "route:c", "query:DDDD"] {
            assert!(
                !ledger.identity_routes.contains_key(key),
                "malformed search created {key}"
            );
        }
        assert!(
            ledger.evidence_floor_satisfied(true),
            "malformed declarations must not poison another covered entity"
        );

        let numeric_match_mode = evidence_call(
            "numeric-match-mode",
            r#"{"data_type":"search","query":"CRWV","identity_match":7}"#,
        );
        let mut malformed_metadata = ResearchEvidenceLedger::default();
        malformed_metadata.observe_business_call(&numeric_match_mode, true);
        malformed_metadata.observe_business_result(
            &numeric_match_mode,
            &json!({"data":[{
                "symbol":"CWY",
                "name":"GraniteShares YieldBOOST CRWV ETF"
            }]}),
            true,
        );
        malformed_metadata.observe_business_call(
            &evidence_call("quote-cwy", r#"{"data_type":"quote","ticker":"CWY"}"#),
            true,
        );
        malformed_metadata.observe_business_call(
            &evidence_call("profile-cwy", r#"{"data_type":"profile","ticker":"CWY"}"#),
            true,
        );
        assert!(malformed_metadata.identity_routes.is_empty());
        assert!(
            !malformed_metadata.evidence_floor_satisfied(true),
            "wrongly typed identity metadata cannot establish a security evidence floor"
        );

        let provisional_nvidia = evidence_call(
            "provisional-nvidia",
            r#"{"data_type":"search","query":"NVIDIA","identity_match":"name_or_alias"}"#,
        );
        let malformed_link = evidence_call(
            "malformed-link",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","refines_query":7,"supersedes_query":"NVIDIA"}"#,
        );
        let mut malformed_migration = ResearchEvidenceLedger::default();
        malformed_migration.observe_business_call(&provisional_nvidia, false);
        malformed_migration.observe_business_result(
            &provisional_nvidia,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            false,
        );
        malformed_migration.observe_business_call(&malformed_link, true);
        assert!(
            malformed_migration
                .identity_routes
                .contains_key("query:NVIDIA")
        );
        assert!(
            !malformed_migration
                .identity_routes
                .contains_key("route:coreweave")
        );
    }

    #[test]
    fn ledger_uses_the_executor_target_and_rejects_spoofed_or_malformed_symbol_fields() {
        let search = evidence_call(
            "search-crwv",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            false,
        );

        for spoofed in [
            evidence_call(
                "ticker-wins-over-symbol",
                r#"{"data_type":"quote","ticker":"CWY","symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            evidence_call(
                "wrongly-typed-ticker-does-not-fall-through",
                r#"{"data_type":"profile","ticker":["CRWV"],"symbol":"CRWV","entity_route":"coreweave"}"#,
            ),
            evidence_call(
                "malformed-batch-symbol",
                r#"{"data_type":"snapshot","ticker":"CRWV,","entity_route":"coreweave"}"#,
            ),
            evidence_call(
                "broad-call-ignores-ticker",
                r#"{"data_type":"gainers_losers","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
        ] {
            ledger.observe_business_call(&spoofed, true);
        }
        let route = ledger
            .identity_routes
            .get("route:coreweave")
            .expect("CoreWeave route");
        assert!(route.quote_symbols.is_empty());
        assert!(route.asset_route_symbols.is_empty());
        assert!(!ledger.evidence_floor_satisfied(true));

        ledger.observe_business_call(
            &evidence_call(
                "real-quote",
                r#"{"data_type":"quote","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "real-profile",
                r#"{"data_type":"profile","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn evidence_for_an_old_candidate_cannot_preload_a_later_candidate_replacement() {
        let first_search = evidence_call(
            "search-nvidia",
            r#"{"data_type":"search","query":"NVIDIA","entity_route":"company","identity_match":"name_or_alias"}"#,
        );
        let replacement_search = evidence_call(
            "search-coreweave",
            r#"{"data_type":"search","query":"CoreWeave","entity_route":"company","identity_match":"name_or_alias"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&first_search, false);
        ledger.observe_business_result(
            &first_search,
            &json!({"data":[{"symbol":"NVDA","name":"NVIDIA Corporation"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "wrong-quote-before-replacement",
                r#"{"data_type":"quote","ticker":"CRWV","entity_route":"company"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call(
                "wrong-profile-before-replacement",
                r#"{"data_type":"profile","ticker":"CRWV","entity_route":"company"}"#,
            ),
            true,
        );
        ledger.observe_business_call(&replacement_search, true);
        ledger.observe_business_result(
            &replacement_search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        let route = ledger
            .identity_routes
            .get("route:company")
            .expect("replaced route");
        assert_eq!(route.candidates, BTreeSet::from(["CRWV".to_string()]));
        assert!(route.quote_symbols.is_empty());
        assert!(route.asset_route_symbols.is_empty());
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "wrong-symbol calls made before replacement cannot become evidence for the new candidate"
        );
    }

    #[test]
    fn old_candidate_followup_cannot_satisfy_a_later_empty_identity_generation() {
        let search = evidence_call(
            "search-crwv",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "old-snapshot",
                r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));

        for _ in 0..2 {
            ledger.observe_business_call(&search, true);
            ledger.observe_business_result(&search, &json!({"data":[]}), true);
        }
        let route = ledger
            .identity_routes
            .get("route:coreweave")
            .expect("empty CoreWeave generation");
        assert_eq!(route.empty_search_results, 2);
        assert_eq!(route.post_identity_attempts, 0);
        assert!(route.quote_symbols.is_empty());
        assert!(route.asset_route_symbols.is_empty());
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "old candidate follow-up cannot satisfy the current empty generation"
        );

        ledger.observe_business_call(
            &evidence_call(
                "current-empty-followup",
                r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));

        // A later successful result starts a new streak. One subsequent empty
        // result cannot reuse the two historical empties as bounded failure.
        ledger.observe_business_call(&search, true);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        assert_eq!(
            ledger
                .identity_routes
                .get("route:coreweave")
                .expect("successful generation")
                .empty_search_results,
            0
        );
        ledger.observe_business_call(&search, true);
        ledger.observe_business_result(&search, &json!({"data":[]}), true);
        ledger.observe_business_call(
            &evidence_call(
                "single-empty-followup",
                r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"coreweave"}"#,
            ),
            true,
        );
        assert!(
            !ledger.evidence_floor_satisfied(true),
            "successful identity evidence must reset the empty-attempt streak"
        );
    }

    #[test]
    fn wrongly_cased_tool_names_never_activate_or_satisfy_the_research_ledger() {
        let mut uppercase_data_fetch = evidence_call(
            "uppercase-data-fetch",
            r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#,
        );
        uppercase_data_fetch.function.name = "DATA_FETCH".to_string();
        assert!(!starts_investment_research_protocol(&uppercase_data_fetch));
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&uppercase_data_fetch, true);
        ledger.observe_business_result(
            &uppercase_data_fetch,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        assert!(ledger.identity_routes.is_empty());
        assert!(!ledger.evidence_floor_satisfied(true));

        let empty_search = evidence_call(
            "empty-search",
            r#"{"data_type":"search","query":"UNKNOWN","identity_match":"exact_symbol"}"#,
        );
        for _ in 0..2 {
            ledger.observe_business_call(&empty_search, true);
            ledger.observe_business_result(&empty_search, &json!({"data":[]}), true);
        }
        let mut uppercase_web = ToolCall {
            id: "uppercase-web-search".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "WEB_SEARCH".to_string(),
                arguments: r#"{"query":"UNKNOWN company"}"#.to_string(),
            },
        };
        ledger.observe_business_call(&uppercase_web, true);
        assert!(!ledger.evidence_floor_satisfied(true));
        uppercase_web.function.name = "web_search".to_string();
        ledger.observe_business_call(&uppercase_web, true);
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn unrelated_extra_quote_does_not_block_a_covered_identity_route() {
        let search = evidence_call("search", r#"{"data_type":"search","query":"CRWV"}"#);
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV"},{"symbol":"CWY"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "quotes",
                r#"{"data_type":"quote","ticker":"CRWV,UNRELATED"}"#,
            ),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile", r#"{"data_type":"profile","ticker":"CRWV"}"#),
            true,
        );

        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn empty_identity_coverage_requires_linked_refinement_and_a_real_followup() {
        let search = evidence_call(
            "search-unknown",
            r#"{"data_type":"search","query":"UNKNOWN"}"#,
        );
        let refinement = evidence_call(
            "search-unknown-name",
            r#"{"data_type":"search","query":"Unknown Holdings Inc","refines_query":"UNKNOWN"}"#,
        );
        let web_followup = ToolCall {
            id: "web-unknown".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"UNKNOWN company"}"#.to_string(),
            },
        };

        let mut web_only = ResearchEvidenceLedger::default();
        web_only.observe_business_call(&search, false);
        web_only.observe_business_result(&search, &json!({"data":[]}), false);
        web_only.observe_business_call(&web_followup, true);
        assert!(
            !web_only.evidence_floor_satisfied(true),
            "a non-search follow-up cannot replace the linked refinement"
        );

        let mut refinement_only = ResearchEvidenceLedger::default();
        refinement_only.observe_business_call(&search, false);
        refinement_only.observe_business_result(&search, &json!({"data":[]}), false);
        refinement_only.observe_business_call(&refinement, true);
        refinement_only.observe_business_result(&refinement, &json!({"data":[]}), true);
        assert!(
            !refinement_only.evidence_floor_satisfied(true),
            "the linked refinement cannot replace a real non-search follow-up"
        );

        refinement_only.observe_business_call(&web_followup, true);
        assert!(refinement_only.evidence_floor_satisfied(true));
        assert!(refinement_only.completion_signal_available(true));

        let empty_a = evidence_call(
            "empty-a",
            r#"{"data_type":"search","query":"AAAA","entity_route":"a","identity_match":"exact_symbol"}"#,
        );
        let empty_b = evidence_call(
            "empty-b",
            r#"{"data_type":"search","query":"BBBB","entity_route":"b","identity_match":"exact_symbol"}"#,
        );
        let mut two_empty_routes = ResearchEvidenceLedger::default();
        for call in [&empty_a, &empty_b, &empty_a, &empty_b] {
            two_empty_routes.observe_business_call(call, true);
            two_empty_routes.observe_business_result(call, &json!({"data":[]}), true);
        }
        two_empty_routes.observe_business_call(
            &evidence_call(
                "attempt-a",
                r#"{"data_type":"snapshot","ticker":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        assert!(
            !two_empty_routes.evidence_floor_satisfied(true),
            "a route-bound attempt for A cannot unlock empty route B"
        );
        two_empty_routes.observe_business_call(
            &evidence_call(
                "attempt-b",
                r#"{"data_type":"snapshot","ticker":"BBBB","entity_route":"b"}"#,
            ),
            true,
        );
        assert!(two_empty_routes.evidence_floor_satisfied(true));

        let covered_a = evidence_call(
            "covered-a",
            r#"{"data_type":"search","query":"AAAA","entity_route":"a","identity_match":"exact_symbol"}"#,
        );
        let mut covered_plus_empty = ResearchEvidenceLedger::default();
        covered_plus_empty.observe_business_call(&covered_a, false);
        covered_plus_empty.observe_business_result(
            &covered_a,
            &json!({"data":[{"symbol":"AAAA","name":"AAAA Corp."}]}),
            false,
        );
        covered_plus_empty.observe_business_call(
            &evidence_call(
                "quote-covered-a",
                r#"{"data_type":"quote","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        covered_plus_empty.observe_business_call(
            &evidence_call(
                "profile-covered-a",
                r#"{"data_type":"profile","symbol":"AAAA","entity_route":"a"}"#,
            ),
            true,
        );
        for _ in 0..2 {
            covered_plus_empty.observe_business_call(&empty_b, true);
            covered_plus_empty.observe_business_result(&empty_b, &json!({"data":[]}), true);
        }
        covered_plus_empty.observe_business_call(&web_followup, true);
        assert!(
            !covered_plus_empty.evidence_floor_satisfied(true),
            "an unscoped Web call cannot be guessed as the only empty route while another route exists"
        );
    }

    fn evidence_call(id: &str, arguments: &str) -> ToolCall {
        ToolCall {
            id: id.to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "data_fetch".to_string(),
                arguments: arguments.to_string(),
            },
        }
    }

    #[test]
    fn unsearched_symbol_scoped_data_fetch_does_not_unlock_finish() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(
            &evidence_call("quote", r#"{"data_type":"quote","ticker":"CRWV"}"#),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile", r#"{"data_type":"profile","ticker":"CRWV"}"#),
            true,
        );
        assert!(!ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn pre_search_quote_does_not_satisfy_post_search_floor() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(
            &evidence_call("early-quote", r#"{"data_type":"quote","ticker":"CRWV"}"#),
            true,
        );
        let search = evidence_call("search", r#"{"data_type":"search","query":"CRWV"}"#);
        ledger.observe_business_call(&search, true);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"CRWV","name":"CoreWeave, Inc."}]}),
            true,
        );
        ledger.observe_business_call(
            &evidence_call("profile", r#"{"data_type":"profile","ticker":"CRWV"}"#),
            true,
        );
        assert!(!ledger.evidence_floor_satisfied(true));
        ledger.observe_business_call(
            &evidence_call(
                "post-search-quote",
                r#"{"data_type":"quote","ticker":"CRWV"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn security_level_earnings_outlook_satisfies_quote_and_asset_route_floor() {
        let mut ledger = ResearchEvidenceLedger::default();
        let search = evidence_call(
            "search",
            r#"{"data_type":"search","query":"COHR","entity_route":"coherent","identity_match":"exact_symbol"}"#,
        );
        ledger.observe_business_call(&search, true);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"COHR","name":"Coherent Corp."}]}),
            true,
        );
        let outlook = evidence_call(
            "earnings",
            r#"{"data_type":"earnings_outlook","ticker":"COHR","entity_route":"coherent"}"#,
        );
        assert!(data_fetch_call_is_structurally_valid(&outlook));
        ledger.observe_business_call(&outlook, true);

        assert!(ledger.evidence_floor_satisfied(true));
        let route = ledger
            .identity_routes
            .get("route:coherent")
            .expect("earnings route");
        assert_eq!(route.quote_symbols, BTreeSet::from(["COHR".to_string()]));
        assert_eq!(
            route.asset_route_symbols,
            BTreeSet::from(["COHR".to_string()])
        );
    }

    #[test]
    fn broad_market_data_fetch_can_finish_without_security_search() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(
            &evidence_call("sector", r#"{"data_type":"sector_performance"}"#),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn broad_market_exact_symbol_query_quotes_open_floor_after_two_groups() {
        let spy = evidence_call(
            "spy",
            r#"{"data_type":"quote","query":"SPY","identity_match":"exact_symbol"}"#,
        );
        let qqq = evidence_call(
            "qqq",
            r#"{"data_type":"quote","query":"QQQ","identity_match":"exact_symbol"}"#,
        );
        let registered = BTreeSet::from(["data_fetch".to_string()]);
        assert!(data_fetch_call_is_structurally_valid(&spy));
        assert!(registered_read_only_tool_call_is_well_formed(
            &spy,
            &registered
        ));

        let mut ledger = ResearchEvidenceLedger {
            broad_market_mode: true,
            ..ResearchEvidenceLedger::default()
        };
        ledger.observe_business_call(&spy, true);
        assert!(!ledger.evidence_floor_satisfied(true));
        let pending_guidance = ledger.agent_guidance_summary();
        assert!(pending_guidance.contains("至少"));
        assert!(!pending_guidance.contains("每个点名标的"));

        ledger.observe_business_call(&qqq, true);
        assert!(ledger.evidence_floor_satisfied(true));
        assert_eq!(
            ledger.broad_market_quote_groups,
            BTreeSet::from(["标普500".to_string(), "纳斯达克".to_string()])
        );
        let complete_guidance = ledger.agent_guidance_summary();
        assert!(complete_guidance.contains("无需为宽基 ETF/指数补证券身份 search"));
        assert!(!complete_guidance.contains("每个点名标的"));

        let mut ordinary_security_ledger = ResearchEvidenceLedger::default();
        ordinary_security_ledger.observe_business_call(&spy, true);
        ordinary_security_ledger.observe_business_call(&qqq, true);
        assert!(
            !ordinary_security_ledger.evidence_floor_satisfied(true),
            "ordinary securities still require identity-search coverage"
        );

        let mut short_quote_ledger = ResearchEvidenceLedger {
            broad_market_mode: true,
            ..ResearchEvidenceLedger::default()
        };
        for (id, symbol) in [("spy-short", "SPY"), ("qqq-short", "QQQ")] {
            short_quote_ledger.observe_business_call(
                &evidence_call(
                    id,
                    &format!(r#"{{"data_type":"quote_short","ticker":"{symbol}"}}"#),
                ),
                true,
            );
        }
        assert!(
            !short_quote_ledger.evidence_floor_satisfied(true),
            "quote_short can omit percentage, exchange, and timestamp, so it cannot open a target-date market-move floor"
        );
        assert!(short_quote_ledger.broad_market_quote_groups.is_empty());
    }

    #[test]
    fn exact_symbol_query_alias_rejects_missing_or_non_exact_match_modes() {
        for arguments in [
            r#"{"data_type":"quote","query":"SPY"}"#,
            r#"{"data_type":"quote","query":"SPY","identity_match":"name_or_alias"}"#,
            r#"{"data_type":"quote","query":["SPY"],"identity_match":"exact_symbol"}"#,
        ] {
            assert!(!data_fetch_call_is_structurally_valid(&evidence_call(
                "invalid-query-alias",
                arguments
            )));
        }
    }

    #[tokio::test]
    async fn broad_market_query_alias_batch_executes_and_can_finish_naturally() {
        let answer = concat!(
            "数据时间：北京时间 2026-07-26 17:30；行情口径：本轮用两组宽基行情尝试核验\n\n",
            "目标时段：2026-07-24（周五，美东市场日）。标普500（SPY）与纳斯达克100（QQQ）应分别判断，不能把单一风格回撤写成整个美股同步大跌。\n\n",
            "原因本轮未完全核验：本测试仅证明宽市场代表性行情调用可以完成，不补写没有同日来源的原因。"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_spy_query_alias".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments:
                        r#"{"data_type":"quote","query":"SPY","identity_match":"exact_symbol"}"#
                            .to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_qqq_query_alias".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments:
                        r#"{"data_type":"quote","query":"QQQ","identity_match":"exact_symbol"}"#
                            .to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingDataFetchTool {
            calls: calls.clone(),
        }));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            3,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("broad-market-query-alias".to_string());
        let response = agent
            .run(
                concat!(
                    "【Session 上下文】\n当前时间：2026-07-26 17:30:00 (北京时间)\n\n",
                    "【本轮用户输入】\n美股为什么大跌\n\n",
                    "【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n",
                    "最近同名候选日期是 2026-07-24 周五。\n\n",
                    "【本轮最终回答契约：由主 Agent 一次完成】\n",
                    "第一条非空行必须严格以 `数据时间：北京时间 2026-07-26 17:30；行情口径：` 开头。"
                ),
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            [ToolChoiceMode::Required, ToolChoiceMode::Auto],
            "a market-move turn must require initial research, then two representative broad-market groups open natural-final mode without identity-search deadlock"
        );
        let records = audit.records.lock().expect("audit records");
        let direct_final = records.last().expect("direct final audit");
        assert_eq!(direct_final.metadata["evidence_floor_satisfied"], true);
        assert_eq!(direct_final.metadata["broad_market_mode"], true);
        assert_eq!(
            direct_final.metadata["broad_market_quote_groups"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
    }

    #[tokio::test]
    async fn broad_market_verified_quotes_and_one_web_attempt_force_bounded_final() {
        let answer = concat!(
            "数据时间：北京时间 2026-07-26 18:30；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露\n\n",
            "目标时段：2026-07-24（周五）。SPY +0.10%，QQQ -1.12%；两组宽基/风格代表走势分化，不支持“美股全面暴跌”的前提。\n\n",
            "本轮搜索只返回标题摘要，没有取得目标日期的来源原文，因此不写成已确认触发因素。**原因本轮未完全核验。**"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_spy".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments:
                        r#"{"data_type":"quote","query":"SPY","identity_match":"exact_symbol"}"#
                            .to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_qqq".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments:
                        r#"{"data_type":"quote","query":"QQQ","identity_match":"exact_symbol"}"#
                            .to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_web".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"US stock market selloff July 24 2026 reasons"}"#
                        .to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(MarketMoveCanaryFinanceTool));
        registry.register(Box::new(SnippetOnlyMarketMoveWebTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            3,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("broad-market-bounded-final".to_string());
        let response = agent
            .run(
                concat!(
                    "【Session 上下文】\n当前时间：2026-07-26 18:30:00 (北京时间)\n\n",
                    "【本轮用户输入】\n周五美股为什么暴跌\n\n",
                    "【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n",
                    "最近同名候选日期是 2026-07-24 周五。\n\n",
                    "【本轮最终回答契约：由主 Agent 一次完成】\n",
                    "第一条非空行必须严格以 `数据时间：北京时间 2026-07-26 18:30；行情口径：` 开头。"
                ),
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_messages.lock().expect("seen messages").len(),
            2,
            "one source attempt plus two verified broad-market quote groups must go directly to a tools-disabled final instead of spending another network-search round"
        );
        let records = audit.records.lock().expect("audit records");
        let final_record = records.last().expect("forced final audit");
        assert_eq!(final_record.metadata["force_finance_final"], true);
        assert_eq!(final_record.metadata["finance_tool_rounds"], 1);
        assert_eq!(
            final_record.metadata["broad_market_quote_groups"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn crypto_search_plus_crypto_quote_unlocks_without_stock_profile() {
        let mut ledger = ResearchEvidenceLedger::default();
        let search = evidence_call(
            "search",
            r#"{"data_type":"search","query":"BTCUSD","entity_route":"bitcoin","identity_match":"exact_symbol"}"#,
        );
        ledger.observe_business_call(&search, false);
        ledger.observe_business_result(
            &search,
            &json!({"data":[{"symbol":"BTCUSD","name":"Bitcoin USD"}]}),
            false,
        );
        ledger.observe_business_call(
            &evidence_call(
                "crypto-quote",
                r#"{"data_type":"crypto_quote","ticker":"BTCUSD","entity_route":"bitcoin"}"#,
            ),
            true,
        );
        assert!(ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn crypto_quote_without_identity_search_does_not_unlock_finish() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(
            &evidence_call(
                "crypto-quote",
                r#"{"data_type":"crypto_quote","ticker":"BTCUSD"}"#,
            ),
            true,
        );
        assert!(!ledger.evidence_floor_satisfied(true));
    }

    #[test]
    fn web_only_after_identity_search_does_not_unlock_finish() {
        let mut ledger = ResearchEvidenceLedger::default();
        ledger.observe_business_call(
            &evidence_call("search", r#"{"data_type":"search","query":"CRWV"}"#),
            false,
        );
        ledger.observe_business_call(
            &ToolCall {
                id: "web".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "web_search".to_string(),
                    arguments: r#"{"query":"company relationship"}"#.to_string(),
                },
            },
            true,
        );
        assert!(!ledger.evidence_floor_satisfied(true));
    }

    #[tokio::test]
    async fn run_without_tools_uses_chat_once() {
        let llm = MockLlmProvider::with_chat_response("plain response");
        let tools = Arc::new(ToolRegistry::new());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm.clone()), tools, "system".to_string(), 3, None);
        let mut context = AgentContext::new("s1".to_string());

        let response = agent.run("hello", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "plain response");
        assert_eq!(response.iterations, 1);
        assert!(response.tool_calls_made.is_empty());

        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.chat_calls, 1);
        assert_eq!(state.chat_with_tools_calls, 0);
    }

    #[tokio::test]
    async fn run_with_tool_call_executes_tool_and_returns_final_answer() {
        let tool_call = hone_llm::ToolCall {
            id: "tc_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"text":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "let me call tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm.clone()),
            Arc::new(registry),
            "system".to_string(),
            4,
            None,
        );
        let mut context = AgentContext::new("s2".to_string());

        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "done");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "echo_tool");
        assert_eq!(response.tool_calls_made[0].result["echo"], "abc");

        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.chat_calls, 0);
        assert_eq!(state.chat_with_tools_calls, 2);
    }

    #[tokio::test]
    async fn native_stream_resets_tool_preamble_and_hides_reasoning_from_final_deltas() {
        let llm = StreamingMockLlmProvider {
            rounds: Arc::new(Mutex::new(VecDeque::from([
                vec![
                    ChatStreamEvent::ContentDelta("I will check".to_string()),
                    ChatStreamEvent::ToolCallDelta {
                        index: 0,
                        id: Some("tc_stream".to_string()),
                        name: Some("echo_tool".to_string()),
                        arguments: "{\"text\":".to_string(),
                    },
                    ChatStreamEvent::ToolCallDelta {
                        index: 0,
                        id: None,
                        name: None,
                        arguments: "\"abc\"}".to_string(),
                    },
                ],
                vec![
                    ChatStreamEvent::ContentDelta("<thi".to_string()),
                    ChatStreamEvent::ContentDelta("nk>secret</think>最终".to_string()),
                    ChatStreamEvent::ContentDelta("答案".to_string()),
                ],
            ]))),
            seen_tool_counts: Arc::new(Mutex::new(Vec::new())),
            seen_tool_names: Arc::new(Mutex::new(Vec::new())),
            seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
            seen_messages: Arc::new(Mutex::new(Vec::new())),
            delivered_events: Arc::new(AtomicUsize::new(0)),
            stream_calls: Arc::new(AtomicUsize::new(0)),
            failed_stream_calls: Arc::new(Mutex::new(Vec::new())),
            failed_stream_errors: Arc::new(Mutex::new(HashMap::new())),
            pending_stream_calls: Arc::new(Mutex::new(Vec::new())),
            hang_after_first_event_stream_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("native-stream".to_string());

        let response = agent.run("stream", &mut context).await;

        assert!(response.success);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].result["echo"], "abc");
        assert_eq!(response.content, "<think>secret</think>最终答案");
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:I will check", "reset", "delta:最终", "delta:答案"]
        );
    }

    #[tokio::test]
    async fn sole_finish_research_runs_one_tool_free_terminal_stream_in_the_same_agent_run() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ReasoningDelta(
                    "hidden draft must not become terminal evidence".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvidia".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: relationship_finish_arguments("tc_web_relationship"),
            }],
            vec![
                ChatStreamEvent::ReasoningDelta("terminal reasoning".to_string()),
                ChatStreamEvent::ContentDelta("最终".to_string()),
                ChatStreamEvent::ContentDelta("答案".to_string()),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            4,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(stream_observer.clone()))
        .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finish-research-terminal".to_string());
        context.add_user_message("旧问题：NBIS 估值");
        context.add_assistant_message(
            "旧草稿：NBIS 价格 15 USD；不要把它当成本轮事实。",
            Some(vec![
                serde_json::to_value(ToolCall {
                    id: "tc_stale_nbis".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"ticker":"NBIS"}"#.to_string(),
                    },
                })
                .expect("stale tool call"),
            ]),
        );
        context.add_tool_result(
            "tc_stale_nbis",
            "data_fetch",
            r#"{"symbol":"NBIS","price":15,"stale":true}"#,
        );

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "最终答案");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 6);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].arguments["data_type"], "search");
        assert_eq!(response.tool_calls_made[2].arguments["data_type"], "quote");
        assert_eq!(
            response.tool_calls_made[3].arguments["data_type"],
            "profile"
        );
        assert_eq!(
            response.tool_calls_made[4].arguments["data_type"],
            "profile"
        );
        assert_eq!(response.tool_calls_made[5].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 0],
            "search-only evidence must force one post-identity business round before the same Agent can select finish and enter the empty-tools terminal"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ],
            "the first turn is open, evidence acquisition is required, completion is Agent-owned Auto, and terminal synthesis has no tools"
        );
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:最终", "final:答案"]
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:web_search",
                "done:web_search:true",
            ]
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
        let terminal_assistant = context.messages.last().expect("terminal assistant message");
        assert_eq!(terminal_assistant.role, "assistant");
        assert_eq!(terminal_assistant.content.as_deref(), Some("最终答案"));
        assert!(
            terminal_assistant.metadata.is_none(),
            "terminal reasoning must not persist into cross-turn context"
        );
        assert_explicit_terminal_messages(&seen_messages);
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let terminal_messages = seen_messages.last().expect("terminal messages");
        assert!(terminal_messages.iter().any(|message| {
            message
                .content
                .as_deref()
                .is_some_and(|content| content.contains("relationship"))
        }));
        assert!(
            terminal_messages.iter().all(|message| {
                message.content.as_deref().is_none_or(|content| {
                    !content.contains("15 USD")
                        && !content.contains("\"price\":15")
                        && !content.contains("NBIS")
                })
            }),
            "any stale prior-turn ticker, request, assistant draft, or price reached terminal synthesis"
        );
    }

    #[tokio::test]
    async fn generic_finish_ids_are_corrected_before_one_grounded_terminal() {
        let generic_finish = json!({
            "answer_scope": "回答 CoreWeave 与 NVIDIA 的关系",
            "facts": [
                {
                    "id": "F1",
                    "evidence": [{
                        "tool_call_id": "web_search",
                        "result_number": 1,
                        "exact_excerpt": "The buyer agreed to purchase $6.3B of unused capacity."
                    }]
                },
                {
                    "id": "F2",
                    "evidence": [{
                        "tool_call_id": "quote",
                        "exact_excerpt": "/data/0/symbol",
                        "json_pointer": "/data/0/symbol"
                    }]
                },
                {
                    "id": "F3",
                    "evidence": [{
                        "tool_call_id": "profile",
                        "exact_excerpt": "/data/0/symbol",
                        "json_pointer": "/data/0/symbol"
                    }]
                }
            ],
            "inferences": [{
                "claim": "双方存在已披露的容量购买安排。",
                "premise_fact_ids": ["F1"]
            }],
            "gaps": ["持股关系未核验"]
        })
        .to_string();
        let corrected_finish = json!({
            "answer_scope": "回答 CoreWeave 与 NVIDIA 的关系",
            "facts": [
                {
                    "id": "F1",
                    "evidence": [{
                        "tool_call_id": "tc_web_relationship",
                        "result_number": 1,
                        "exact_excerpt": "The buyer agreed to purchase $6.3B of unused capacity."
                    }]
                },
                {
                    "id": "F2",
                    "evidence": [{
                        "tool_call_id": "tc_crwv_quote",
                        "json_pointer": "/data/0/symbol"
                    }]
                },
                {
                    "id": "F3",
                    "evidence": [{
                        "tool_call_id": "tc_nvda_quote",
                        "json_pointer": "/data/0/symbol"
                    }]
                }
            ],
            "inferences": [{
                "claim": "双方存在已披露的容量购买安排。",
                "premise_fact_ids": ["F1"]
            }],
            "gaps": ["持股关系未核验"]
        })
        .to_string();
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvda".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_crwv_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 4,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_generic_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: generic_finish,
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_corrected_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: corrected_finish,
            }],
            vec![ChatStreamEvent::ContentDelta("有证据的唯一终稿".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EntityRouteFinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            6,
            None,
        )
        .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("generic-finish-id-correction".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "有证据的唯一终稿");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 7);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 3, 0]
        );
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let correction_round =
            serde_json::to_string(&seen_messages[3]).expect("serialize correction round messages");
        assert!(correction_round.contains("上一次交接有 2 条 evidence locator 无法解析"));
        for id in [
            "tc_crwv_quote",
            "tc_crwv_profile",
            "tc_nvda_quote",
            "tc_nvda_profile",
            "tc_web_relationship",
        ] {
            assert!(correction_round.contains(id), "missing catalog ID {id}");
        }
        let terminal = seen_messages
            .last()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("terminal prompt");
        assert!(terminal.contains("tc_web_relationship"));
        assert!(terminal.contains("tc_crwv_quote"));
        assert!(terminal.contains("tc_nvda_quote"));
        assert!(!terminal.contains("\"facts\":[]"));
    }

    #[tokio::test]
    async fn repeatedly_invalid_finish_ids_never_authorize_empty_evidence_terminal() {
        let invalid_finish = json!({
            "answer_scope": "回答 CoreWeave 的当前情况",
            "facts": [{
                "id": "F1",
                "evidence": [{
                    "tool_call_id": "quote",
                    "json_pointer": "/data/0/symbol"
                }]
            }],
            "inferences": [],
            "gaps": []
        })
        .to_string();
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_crwv_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_invalid_finish_1".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: invalid_finish.clone(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_invalid_finish_2".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: invalid_finish,
            }],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EntityRouteFinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            6,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("repeated-invalid-finish-id".to_string());

        let response = agent.run("crwv最近怎么样", &mut context).await;

        assert!(!response.success);
        assert_eq!(
            response.error.as_deref(),
            Some("finish_research_evidence_locators_repeatedly_unresolvable")
        );
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 2],
            "one hidden correction is allowed, but a second invalid locator must fail before any tool-free terminal call"
        );
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let correction_round = serde_json::to_string(&seen_messages[3])
            .expect("serialize repeated invalid correction round");
        assert!(correction_round.contains("tc_crwv_quote"));
        assert!(correction_round.contains("tc_crwv_profile"));
        assert!(correction_round.contains("上一次交接有 1 条 evidence locator 无法解析"));
        drop(seen_messages);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "protocol failure must not flash or rewrite a user-visible answer"
        );
    }

    #[tokio::test]
    async fn repeated_gaps_only_finish_with_citable_sources_never_enters_empty_terminal() {
        let gaps_only_finish = json!({
            "answer_scope": "回答 CoreWeave 的当前情况",
            "facts": [],
            "inferences": [],
            "gaps": ["当前关系证据仍不足"]
        })
        .to_string();
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_crwv_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_gaps_only_finish_1".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: gaps_only_finish.clone(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_gaps_only_finish_2".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: gaps_only_finish,
            }],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EntityRouteFinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            6,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("repeated-gaps-only-with-sources".to_string());

        let response = agent.run("crwv最近怎么样", &mut context).await;

        assert!(!response.success);
        assert_eq!(
            response.error.as_deref(),
            Some("finish_research_evidence_locators_repeatedly_unresolvable")
        );
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 2],
            "citable current-turn sources require at least one resolved fact before terminal synthesis"
        );
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let correction_round =
            serde_json::to_string(&seen_messages[3]).expect("serialize gaps-only correction round");
        assert!(correction_round.contains("上一次交接没有形成任何可解析的 fact evidence"));
        assert!(correction_round.contains("tc_crwv_quote"));
        assert!(correction_round.contains("tc_crwv_profile"));
        drop(seen_messages);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "a gaps-only handoff must not flash a terminal body when citable evidence was omitted"
        );
    }

    #[tokio::test]
    async fn mixed_finish_keeps_business_tools_in_the_same_agent_loop() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_hallucinated_finish_with_data".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_entity_snapshot".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("不应发布的业务轮草稿".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_echo".to_string()),
                    name: Some("echo_tool".to_string()),
                    arguments: r#"{"text":"relationship evidence"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_hallucinated_finish_with_echo".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![ChatStreamEvent::ContentDelta("最终研究答案".to_string())],
        ]);
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            5,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_tool_call_budget(Some(2), HashMap::new())
        .with_stream_observer(Some(stream_observer.clone()))
        .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("same-agent-business-finish".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "最终研究答案");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].name, "data_fetch");
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        let tool_names = seen_tool_names.lock().expect("stream tool names lock");
        assert!(
            tool_names[0]
                .iter()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[1]
                .iter()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[2].iter().any(|name| name == "data_fetch"));
        assert!(tool_names[2].iter().any(|name| name == "echo_tool"));
        assert!(
            tool_names[2]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[3]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[4].is_empty());
        drop(tool_names);
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:最终研究答案"],
            "active business drafts and internal finish signals must remain invisible"
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
            ],
            "finish signals and budget-rejected mixed calls must not enter the business tool trace"
        );
        assert_eq!(
            audit
                .operations
                .lock()
                .expect("audit operations lock")
                .as_slice(),
            [
                "chat_with_tools",
                "chat_with_tools",
                "chat_with_tools",
                "chat_with_tools",
                "chat_terminal_without_tools",
            ],
            "business calls and Agent-owned finish decisions stay in one audited loop"
        );
        assert!(context.messages.iter().all(|message| {
            message.content.as_deref() != Some("不应发布的业务轮草稿")
                && message.tool_calls.as_ref().is_none_or(|tool_calls| {
                    tool_calls.iter().all(|tool_call| {
                        let name = tool_call
                            .get("function")
                            .and_then(|function| function.get("name"))
                            .and_then(Value::as_str);
                        name != Some(FINISH_RESEARCH_TOOL_NAME)
                    })
                })
        }));
    }

    #[tokio::test]
    async fn natural_direct_final_before_finish_signal_is_preserved_without_service_veto() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "provider bypass draft".to_string(),
            )],
        ]);
        let delivered_events = llm.delivered_events.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-content-bypass".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "provider bypass draft");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(response.error.is_none());
        assert_eq!(
            delivered_events.load(Ordering::SeqCst),
            8,
            "the complete active content stream must be consumed through Finish + Done"
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1],
            "a natural direct final must never trigger an empty-tools terminal call"
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(
            context
                .messages
                .iter()
                .any(|message| message.content.as_deref() == Some("provider bypass draft"))
        );
        let requests = seen_messages.lock().expect("stream messages lock");
        assert_eq!(requests.len(), 2);
        let pending_reminder = requests
            .last()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("evidence-pending reminder");
        assert!(pending_reminder.contains("本轮只取证，不作答"));
        assert!(pending_reminder.contains("本轮必须只返回一个或多个真实业务工具调用"));
        assert!(pending_reminder.contains("禁止输出数据时间、摘要、解释、草稿或最终正文"));
        assert!(!pending_reminder.contains("自然输出完整正文"));
        assert!(!pending_reminder.contains("该正文会原样成为最终回答"));
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes lock")
                .as_slice(),
            [ToolChoiceMode::Auto, ToolChoiceMode::Required]
        );
        drop(requests);
        let records = audit.records.lock().expect("audit records lock");
        let direct_finals = records
            .iter()
            .filter(|record| {
                record.metadata["active_business_outcome"].as_str() == Some("direct_final")
            })
            .collect::<Vec<_>>();
        assert_eq!(direct_finals.len(), 1);
        assert!(direct_finals[0].success);
        assert!(direct_finals[0].error.is_none());
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn eligible_direct_final_is_preserved_without_terminal_or_second_generation() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：报价源时间：北京时间 2026-07-18 04:00（最新可得、非逐笔）\n\nCoreWeave 与 NVIDIA 的关系仅按本轮网页来源直接支持的范围表述。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvidia".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA","entity_route":"nvidia","identity_match":"name_or_alias"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_crwv_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"NVDA","entity_route":"nvidia"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"NVDA","entity_route":"nvidia"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 4,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ReasoningDelta("未采用的隐藏关系推演不能进入后续会话".to_string()),
                ChatStreamEvent::ContentDelta(answer.to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("eligible-direct-final".to_string());
        context.add_user_message("旧问题：NBIS 的估值");
        context.add_assistant_message("旧答案：NBIS 价格是 15 USD", None);

        let response = agent
            .run(
                concat!(
                    "【Session 上下文】\n当前时间：2026-07-19 09:31:42 (北京时间)\n\n",
                    "【本轮用户输入】\ncrwv和英伟达有什么关系\n\n",
                    "【本轮最终回答契约：由主 Agent 一次完成】\n",
                    "第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。"
                ),
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 7);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 2],
            "production natural-final mode must expose only real business tools and never start an empty-tools second generation"
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names lock")
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME),
            "finish_research must not be present in the production Interactive Agent loop"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:数据时间：北京时间 2026-07-19 09:31；行情口径："],
            "only the canonical header reaches the observer before the accepted body is finalized"
        );
        assert_eq!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref()),
            Some(answer)
        );
        assert!(
            context
                .messages
                .last()
                .and_then(|message| message.metadata.as_ref())
                .is_none(),
            "finance direct-final reasoning must not persist into a later turn"
        );
        let records = audit.records.lock().expect("audit records lock");
        let direct_final = records.last().expect("direct final audit");
        assert!(direct_final.success);
        assert_eq!(
            direct_final.metadata["active_business_outcome"].as_str(),
            Some("direct_final")
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
        drop(records);
        let requests = seen_messages.lock().expect("stream messages lock");
        let initial_request = serde_json::to_string(requests.first().expect("initial request"))
            .expect("serialize initial request");
        assert!(initial_request.contains("近期用户原话，仅用于理解本轮指代"));
        assert!(initial_request.contains("旧问题：NBIS 的估值"));
        assert!(!initial_request.contains("15 USD"));
        let direct_request = requests.last().expect("direct final request");
        let last_reminder = direct_request
            .last()
            .and_then(|message| message.content.as_deref())
            .expect("last-mile reminder");
        assert!(last_reminder.contains("本轮由同一 Agent 自然收口"));
        assert!(last_reminder.contains("直接生成一次完整自然终稿"));
        assert!(last_reminder.contains("让回答范围跟随用户原问题"));
        assert!(!last_reminder.contains("finish_research"));
        assert!(!last_reminder.contains("固定拒答"));
        assert!(last_reminder.contains("数据时间：北京时间 2026-07-19 09:31；行情口径："));
        assert!(last_reminder.contains("`hone_quote_time.market_date_new_york`"));
        assert!(last_reminder.contains("绝不能据此写‘纽交所’或‘收盘价’"));
        assert!(last_reminder.contains("核心/最大/头部、大客户"));
        assert!(last_reminder.contains("高度依赖、锁定和多重绑定"));
        let serialized = serde_json::to_string(direct_request).expect("serialize direct request");
        assert!(serialized.contains("近期用户原话，仅用于理解本轮指代"));
        assert!(serialized.contains("历史 assistant、tool、价格、财务与结论均未提供"));
        assert!(!serialized.contains("15 USD"));
        assert!(serialized.contains("CoreWeave NVIDIA relationship filing"));
        assert!(serialized.contains("2026-07-18 04:00:00"));
        assert!(serialized.contains("NASDAQ Global Market"));
        assert!(
            serialized.contains("NVIDIA agreed to purchase $6.3B of unused CoreWeave capacity")
        );
        assert!(serialized.contains("NVIDIA as an investor"));
        assert!(serialized.contains("most-favored-nation relationship"));
        assert!(!last_reminder.contains("若 provider 仍自然输出完整正文"));
    }

    struct UncoveredIdentityDataFetchTool;

    #[async_trait]
    impl Tool for UncoveredIdentityDataFetchTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "registry with no coverage for the requested token"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            let data_type = args
                .get("data_type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Ok(json!({"data_type": data_type, "data": []}))
        }
    }

    #[tokio::test]
    async fn memory_only_listing_claim_is_sent_back_with_tools_required() {
        let denial = "数据时间：北京时间 2026-07-19 09:31；行情口径：暂无\n\nCoreWeave 目前尚未上市，仍是一家私营公司。";
        let corrected = "数据时间：北京时间 2026-07-19 09:31；行情口径：最新可得、非逐笔\n\nCoreWeave（CRWV）当前在 NASDAQ 上市交易。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ContentDelta(denial.to_string())],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CoreWeave","entity_route":"crwv","identity_match":"name_or_alias"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(corrected.to_string())],
        ]);
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 6, None)
                .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("memory-only-listing".to_string());

        let response = agent
            .run(
                "【本轮用户输入】coreweave 现在怎么样，值得买吗\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, corrected);
        assert!(!response.content.contains("尚未上市"));
        // The retry must actually require a tool call; asking again politely is
        // how the same memory answer comes back.
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            // Round 2 is forced by this guard; round 3 stays forced by the
            // ordinary evidence floor, which now has an identity search but no
            // post-identity quote/profile yet.
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
            ]
        );
        let requests = seen_messages.lock().expect("stream messages");
        let retry_prompt = requests
            .get(1)
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("retry prompt");
        assert!(
            retry_prompt.contains("本轮没有执行过任何业务工具"),
            "{retry_prompt}"
        );
        assert!(
            retry_prompt.contains("不得把缺失写成事实"),
            "{retry_prompt}"
        );
    }

    /// Hole 2: the turn did fetch CRWV and got `active_listing`, but the denial
    /// names the company rather than the symbol — the natural phrasing.
    #[test]
    fn listing_denial_by_company_name_is_caught_like_a_symbol_denial() {
        let call = ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: json!({"data_type": "snapshot", "ticker": "CRWV"}),
            result: json!({
                "hone_security_listing_evidence": {
                    "status": "active_listing",
                    "requested_symbol": "CRWV",
                    "company_name": "CoreWeave, Inc.",
                    "exchange": "NASDAQ"
                }
            }),
            tool_call_id: Some("tc1".to_string()),
        };
        let input = "【本轮用户输入】coreweave怎么样";
        for draft in [
            "CoreWeave 目前尚未上市，还没有公开交易的股票。",
            "CRWV 目前尚未上市。",
            "CoreWeave 已退市。",
        ] {
            let violations = listing_final_violations(input, draft, std::slice::from_ref(&call));
            assert_eq!(violations.len(), 1, "{draft}");
            assert!(
                violations[0].starts_with("CRWV:"),
                "{draft} -> {violations:?}"
            );
        }
        // A true statement about the current listing stays publishable.
        assert!(
            listing_final_violations(
                input,
                "CoreWeave（CRWV）当前在 NASDAQ 上市交易。",
                std::slice::from_ref(&call),
            )
            .is_empty()
        );
    }

    /// Labels are derived from the provider payload, never from a maintained
    /// list of corporate-form words: the segment before the first comma is
    /// enough to turn `CoreWeave, Inc.` into a matchable `CoreWeave`.
    #[test]
    fn company_name_labels_come_from_the_provider_payload_only() {
        let mut state = ListingEvidenceState {
            active: true,
            ..Default::default()
        };
        state.names.insert("CoreWeave, Inc.".to_string());
        let labels = listing_entity_labels("CRWV", &state);
        assert!(labels.contains(&"CRWV".to_string()));
        assert!(labels.contains(&"COREWEAVE".to_string()));
        assert!(!labels.iter().any(|label| label == "INC"));
    }

    /// The precondition reads one thing: did the Agent declare an investment
    /// answer, and did it call anything. No claim vocabulary is involved, so a
    /// fabricated price is caught by exactly the same rule as a listing denial
    /// — the case a hand-maintained denial list structurally could not cover.
    #[test]
    fn research_evidence_precondition_ignores_what_the_claim_says() {
        let header = "数据时间：北京时间 2026-07-19 09:31；行情口径：最新可得、非逐笔";
        for body in [
            "CoreWeave 目前尚未上市，仍是一家私营公司。",
            "CoreWeave 还没 IPO。",
            "CRWV 当前股价 142.30 美元，Forward PE 约 38 倍，目标价 175 美元。",
            "CRWV 三季度营收同比增长 210%，自由现金流转正。",
            "本轮无法提供具体的数字。",
        ] {
            assert!(
                research_answer_without_evidence(&format!("{header}\n\n{body}"), &[], 0),
                "missed: {body}"
            );
        }

        // An ordinary chat answer never carries the contract header, so it is
        // untouched: the gate keys on the Agent's own declaration, not on the
        // user's wording.
        assert!(!research_answer_without_evidence(
            "你好，有什么可以帮你的？",
            &[],
            0
        ));
        assert!(!research_answer_without_evidence(
            "IPO 是指公司首次公开发行股票。未上市公司不能公开交易。",
            &[],
            0
        ));

        // Service-preloaded evidence stands the guard down: the turn is not
        // evidence-free just because the Agent itself called nothing.
        assert!(!research_answer_without_evidence(
            &format!("{header}\n\nCoreWeave（CRWV）当前在 NASDAQ 上市交易。"),
            &[],
            3
        ));

        // Once any tool ran, the evidence-backed guards own the answer.
        let call = ToolCallMade {
            name: "web_search".to_string(),
            arguments: json!({"query": "CoreWeave IPO"}),
            result: json!({"results": []}),
            tool_call_id: Some("tc1".to_string()),
        };
        assert!(!research_answer_without_evidence(
            &format!("{header}\n\nCoreWeave 目前尚未上市。"),
            std::slice::from_ref(&call),
            0
        ));
    }

    #[tokio::test]
    async fn unresolvable_identity_never_closes_a_finance_turn_without_open_research() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料\n\n根据本轮检索到的持仓与资金流资料回答。";
        let identity_round = || {
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_cta".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CTA","entity_route":"cta","identity_match":"exact_symbol"}"#.to_string(),
            }]
        };
        // Enough identity-only rounds to drain the identity budget and then the
        // whole research budget, which is what triggers the rescue round.
        assert_eq!(MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUNDS, 2);
        assert_eq!(MAX_AGENT_OWNED_FINANCE_TOOL_ROUNDS, 5);
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            identity_round(),
            identity_round(),
            identity_round(),
            identity_round(),
            identity_round(),
            identity_round(),
            identity_round(),
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_cta".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CTA positioning US tech semiconductors"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(UncoveredIdentityDataFetchTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            11,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("phantom-identity-rescue".to_string());

        let response = agent
            .run(
                "【本轮用户输入】美股科技股和半导体股票方面的CTA是多少\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);

        let tool_names = seen_tool_names.lock().expect("stream tool names");
        // The rescue round takes the identity registry off the table and keeps
        // open research available, so the turn cannot spend it re-resolving the
        // same token that already consumed the budget.
        let rescue_round = tool_names
            .iter()
            .position(|names| !names.contains(&"data_fetch".to_string()) && !names.is_empty())
            .expect("an open-research rescue round is offered");
        assert!(tool_names[rescue_round].contains(&"web_search".to_string()));
        // Only after open research actually ran may the tools-disabled final
        // happen.
        assert!(tool_names.last().expect("final round").is_empty());
        drop(tool_names);

        let requests = seen_messages.lock().expect("stream messages");
        let rescue_prompt = requests
            .get(rescue_round)
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("rescue prompt");
        assert!(rescue_prompt.contains("本轮改为开放检索取证"));
        assert!(rescue_prompt.contains("不要再调用实体注册表"));
        drop(requests);

        let records = audit.records.lock().expect("audit records");
        let rescue = records.get(rescue_round).expect("rescue audit record");
        assert_eq!(rescue.metadata["force_finance_final"], false);
        let forced = records.last().expect("forced-final audit");
        assert_eq!(forced.metadata["force_finance_final"], true);
        assert_eq!(forced.metadata["evidence_rescue_rounds"], 1);
    }

    #[tokio::test]
    async fn finance_loop_keeps_identity_rounds_off_the_research_budget_before_forcing_final() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：最新可得、非逐笔\n\n只根据三轮已取得证据完成回答。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_snapshot_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"crwv"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_crwv".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave data center filing"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_crwv_2".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave capacity agreement"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_crwv_3".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave backlog conversion"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_crwv_4".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave margin trend"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        // The script above holds exactly one identity round plus a full
        // research budget, so raising the budget without extending it would
        // silently stop testing the forced final.
        assert_eq!(MAX_AGENT_OWNED_FINANCE_TOOL_ROUNDS, 5);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            8,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("bounded-finance-final".to_string());

        let response = agent
            .run(
                "【本轮用户输入】CRWV 怎么看\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        // The identity-only first batch draws on the identity budget, so the
        // full research budget still runs before the tools-disabled final.
        assert_eq!(response.iterations, 7);
        assert_eq!(response.tool_calls_made.len(), 6);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts")
                .as_slice(),
            [2, 2, 2, 2, 2, 2, 0]
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names")
                .last()
                .expect("forced-final tool list")
                .is_empty()
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        assert_eq!(
            observer.events.lock().expect("stream events").as_slice(),
            ["final:数据时间：北京时间 2026-07-19 09:31；行情口径："]
        );
        let requests = seen_messages.lock().expect("stream messages");
        let forced_prompt = requests
            .last()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("forced-final prompt");
        assert!(forced_prompt.contains("本轮由同一 Agent 有界收口"));
        assert!(forced_prompt.contains("本轮没有任何可用工具"));
        drop(requests);
        let records = audit.records.lock().expect("audit records");
        assert_eq!(records.len(), 7);
        assert!(
            records
                .iter()
                .all(|record| record.operation == "chat_with_tools")
        );
        let forced = records.last().expect("forced-final audit");
        assert_eq!(forced.request["tools"], json!([]));
        assert_eq!(forced.metadata["has_tools"], false);
        assert_eq!(forced.metadata["force_finance_final"], true);
        assert_eq!(forced.metadata["finance_identity_rounds"], 1);
        assert_eq!(forced.metadata["finance_tool_rounds"], 5);
        assert_eq!(forced.metadata["evidence_rescue_rounds"], 0);
        assert_eq!(forced.metadata["active_business_outcome"], "direct_final");
    }

    #[tokio::test]
    async fn finance_loop_enforces_intrinsic_web_limit_without_request_budget_then_forces_final() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：最新可得、非逐笔\n\n只使用已取得的六次网页证据完成回答。";
        let web_calls = (0..7)
            .map(|index| ChatStreamEvent::ToolCallDelta {
                index,
                id: Some(format!("tc_web_{}", index + 1)),
                name: Some("web_search".to_string()),
                arguments: json!({"query": format!("CoreWeave evidence {}", index + 1)})
                    .to_string(),
            })
            .collect::<Vec<_>>();
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            web_calls,
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("intrinsic-web-budget".to_string());

        let response = agent
            .run(
                "【本轮用户输入】分析 CRWV\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 7);
        assert_eq!(
            response
                .tool_calls_made
                .iter()
                .filter(|call| call.name == "web_search")
                .count(),
            MAX_AGENT_OWNED_FINANCE_WEB_SEARCH_CALLS as usize
        );
        let rejected = context
            .messages
            .iter()
            .find(|message| message.tool_call_id.as_deref() == Some("tc_web_7"))
            .expect("seventh Web call budget result");
        assert!(
            rejected
                .content
                .as_deref()
                .is_some_and(|content| content.contains("call limit reached (6)"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts")
                .as_slice(),
            [2, 2, 0]
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names")
                .last()
                .expect("forced-final tool list")
                .is_empty()
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
        );
        assert_eq!(
            observer.events.lock().expect("stream events").as_slice(),
            ["final:数据时间：北京时间 2026-07-19 09:31；行情口径："]
        );
        let requests = seen_messages.lock().expect("stream messages");
        let forced_prompt = requests
            .last()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("forced-final prompt");
        assert!(forced_prompt.contains("本轮由同一 Agent 有界收口"));
        drop(requests);
        let records = audit.records.lock().expect("audit records");
        let forced = records.last().expect("forced-final audit");
        assert_eq!(forced.request["tools"], json!([]));
        assert_eq!(forced.metadata["tool_budget_exhausted"], true);
        assert_eq!(forced.metadata["force_finance_final"], true);
        assert_eq!(forced.metadata["active_business_outcome"], "direct_final");
    }

    #[tokio::test]
    async fn gap_closure_never_reopens_a_turn_whose_visible_prefix_is_committed() {
        // The draft has already crossed an irreversible publication boundary.
        // Re-entering the tool loop there appends the next rounds' narration
        // onto the published body instead of replacing it — the regression that
        // published 23k chars of inter-round narration as the answer.
        let prefix =
            "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料".to_string();
        let gap_draft = format!("{prefix}\n\n项目按期推进。本轮未读到市政停工令原文。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_committed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"NBIS","entity_route":"nbis","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(gap_draft.clone())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            10,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_finance_research_budget(FinanceResearchBudget {
            gap_closure_rounds: 2,
            ..FinanceResearchBudget::default()
        })
        .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()))
        .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("gap-closure-committed".to_string());

        let response = agent
            .run("某数据中心项目最近有什么进展", &mut context)
            .await;

        assert!(response.success, "{:?}", response.error);
        // The gap draft is published as-is; its disclosed gap stays disclosed.
        assert_eq!(response.content, gap_draft);
        let all = seen_messages.lock().expect("seen messages");
        assert!(
            !all.iter().flatten().any(|message| message
                .content
                .as_deref()
                .is_some_and(|c| c.contains("内部缺口闭环轮"))),
            "no gap-closure round may be granted once the prefix is committed"
        );
    }

    #[tokio::test]
    async fn gap_closure_budget_sends_incomplete_finals_back_to_the_tool_loop() {
        let prefix =
            "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料".to_string();
        let gap_draft = format!("{prefix}\n\n项目按期推进。本轮未读到市政停工令原文。");
        let closed_final = format!("{prefix}\n\n已取得市政停工令原文，项目于 8 月复工。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_gap".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"NBIS","entity_route":"nbis","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_gap_1".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"vineland stop work order"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(gap_draft.clone())],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_gap_2".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"site:vinelandcity.org stop work order data center"}"#
                    .to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(closed_final.clone())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            10,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_finance_research_budget(FinanceResearchBudget {
            gap_closure_rounds: 1,
            ..FinanceResearchBudget::default()
        })
        .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()));
        let mut context = AgentContext::new("gap-closure-budget".to_string());

        let response = agent
            .run("某数据中心项目最近有什么进展", &mut context)
            .await;

        assert!(response.success, "{:?}", response.error);
        // The gap draft is not published; the closed final wins.
        assert_eq!(response.content, closed_final);
        let all_messages = seen_messages.lock().expect("seen messages");
        let gap_prompts: Vec<&Message> = all_messages
            .iter()
            .flatten()
            .filter(|message| {
                message
                    .content
                    .as_deref()
                    .is_some_and(|content| content.contains("内部缺口闭环轮"))
            })
            .collect();
        assert_eq!(
            gap_prompts.len(),
            1,
            "exactly one gap-closure feedback round"
        );
        assert!(
            gap_prompts[0]
                .content
                .as_deref()
                .is_some_and(|content| content.contains("本轮未读到市政停工令原文")),
            "gap checklist carries the self-reported missing evidence line"
        );
        // With the default budget (gap closure disabled) the same draft would
        // have been accepted; guard the default so tests keep meaning.
        assert_eq!(FinanceResearchBudget::default().gap_closure_rounds, 0);
    }

    #[tokio::test]
    async fn precommitted_prefix_forces_empty_tools_final_after_the_web_only_research_budget() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\n连续三轮网页取证后的同 Agent 自然终稿。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_1".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave data center filing"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_2".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"Nebius data center filing"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_3".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"China listed data center operators filing"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_4".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"China data center operator capacity disclosure"}"#
                    .to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_5".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"China data center operator margin disclosure"}"#
                    .to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        // One web-only round per unit of research budget, then the final.
        assert_eq!(MAX_AGENT_OWNED_FINANCE_TOOL_ROUNDS, 5);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let stream_calls = llm.stream_calls.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedRelationshipEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            8,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()))
        .with_tool_observer(Some(tool_observer.clone()))
        .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("precommitted-web-only-budget".to_string());

        let response = agent.run("筛选数据中心标的", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 6);
        assert_eq!(response.tool_calls_made.len(), 5);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 6);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts")
                .as_slice(),
            [1, 1, 1, 1, 1, 0]
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names")
                .last()
                .expect("forced-final tool list")
                .is_empty()
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .len(),
            10
        );
        assert!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .is_empty(),
            "the T0 prefix is already visible and the natural final must not restream it"
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
        let requests = seen_messages.lock().expect("stream messages");
        let forced_prompt = requests
            .last()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .expect("forced-final prompt");
        assert!(forced_prompt.contains("本轮由同一 Agent 有界收口"));
        drop(requests);
        let records = audit.records.lock().expect("audit records");
        let forced = records.last().expect("forced-final audit");
        assert_eq!(forced.request["tools"], json!([]));
        assert_eq!(forced.metadata["finance_tool_rounds"], 5);
        assert_eq!(forced.metadata["force_finance_final"], true);
        assert_eq!(forced.metadata["active_business_outcome"], "direct_final");
    }

    #[tokio::test]
    async fn precommitted_prefix_blocks_unregistered_mcp_datafetch_and_still_answers() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer =
            format!("{prefix}\n\nCRWV：未执行未注册的数据请求；仅按当前已有信息给出风险框架。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_unregistered_mcp_datafetch".to_string()),
                name: Some("mcp__hone__data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let registry_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingTool {
            calls: registry_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 2, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()))
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("precommitted-unregistered-mcp-datafetch".to_string());

        let response = agent.run("分析 CRWV", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(registry_calls.load(Ordering::SeqCst), 0);
        assert!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .is_empty()
        );
        assert!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .is_empty()
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
        assert!(context.messages.iter().all(|message| {
            message.role != "tool"
                && message.tool_calls.as_ref().is_none_or(|tool_calls| {
                    tool_calls.iter().all(|tool_call| {
                        tool_call.get("id").and_then(Value::as_str)
                            != Some("tc_unregistered_mcp_datafetch")
                    })
                })
        }));
    }

    #[tokio::test]
    async fn deferred_prefix_ignores_structurally_invalid_datafetch_activation() {
        for (case, arguments) in [
            (
                "unsupported-data-type",
                r#"{"data_type":"unsupported_type","ticker":"CRWV"}"#,
            ),
            ("missing-security-target", r#"{"data_type":"quote"}"#),
        ] {
            let prefix =
                format!("数据时间：北京时间 2026-07-19 09:31；行情口径：{case} 仅使用可核验资料");
            let llm =
                StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some(format!("tc_{case}")),
                    name: Some("data_fetch".to_string()),
                    arguments: arguments.to_string(),
                }]]);
            let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
            let audit = Arc::new(RecordingAuditSink::default());
            let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
            let calls = Arc::new(AtomicUsize::new(0));
            let mut registry = ToolRegistry::new();
            registry.register(Box::new(CountingDataFetchTool {
                calls: calls.clone(),
            }));
            let agent = FunctionCallingAgent::new(
                Arc::new(llm),
                Arc::new(registry),
                String::new(),
                1,
                Some(audit.clone()),
            )
            .with_agent_owned_finance_loop(true)
            .with_service_owned_initial_prefix(Some(prefix), None)
            .with_stream_observer(Some(stream_observer.clone()));
            let mut context = AgentContext::new(format!("deferred-{case}"));

            let response = agent.run("分析 CRWV", &mut context).await;

            assert!(!response.success, "{case}");
            assert_eq!(
                response.error.as_deref(),
                Some("max_iterations_exceeded:1"),
                "{case}"
            );
            assert_eq!(calls.load(Ordering::SeqCst), 1, "{case}");
            assert_eq!(
                seen_tool_choice_modes
                    .lock()
                    .expect("tool choice modes")
                    .as_slice(),
                [ToolChoiceMode::Auto],
                "{case}"
            );
            assert!(
                stream_observer
                    .events
                    .lock()
                    .expect("stream events")
                    .is_empty(),
                "{case} must not ACK the deferred prefix"
            );
            assert!(
                stream_observer.committed_visible_prefix().is_none(),
                "{case}"
            );
            let records = audit.records.lock().expect("audit records");
            let attempt = records.last().expect("DataFetch attempt audit");
            assert_eq!(attempt.metadata["finance_tool_rounds"], 0, "{case}");
            assert_eq!(attempt.metadata["force_finance_final"], false, "{case}");
            assert_eq!(
                attempt.metadata["active_business_outcome"],
                Value::Null,
                "{case}"
            );
        }
    }

    #[tokio::test]
    async fn first_batch_identity_route_limit_executes_only_six_valid_routes() {
        let identity_calls = (0..7)
            .map(|index| ChatStreamEvent::ToolCallDelta {
                index,
                id: Some(format!("tc_route_{index}")),
                name: Some("data_fetch".to_string()),
                arguments: json!({
                    "data_type": "search",
                    "query": format!("SYM{index}"),
                    "entity_route": format!("candidate-{index}"),
                    "identity_match": "exact_symbol"
                })
                .to_string(),
            })
            .collect::<Vec<_>>();
        let llm = StreamingMockLlmProvider::with_rounds(vec![identity_calls]);
        let registry_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingDataFetchTool {
            calls: registry_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 1, None)
                .with_agent_owned_finance_loop(true)
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("first-batch-route-limit".to_string());

        let response = agent.run("筛选七个合法候选", &mut context).await;

        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("max_iterations_exceeded:1"));
        assert_eq!(
            registry_calls.load(Ordering::SeqCst),
            MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES
        );
        assert_eq!(
            response.tool_calls_made.len(),
            MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .len(),
            MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES * 2
        );
        let rejected = context
            .messages
            .iter()
            .find(|message| message.tool_call_id.as_deref() == Some("tc_route_6"))
            .expect("seventh route rejection");
        let rejected: Value = serde_json::from_str(
            rejected
                .content
                .as_deref()
                .expect("seventh route rejection content"),
        )
        .expect("parse seventh route rejection");
        assert_eq!(rejected["error"], "identity_route_limit_reached");
        assert_eq!(rejected["limit"], MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUTES);
        assert!(
            response
                .tool_calls_made
                .iter()
                .all(|call| { call.tool_call_id.as_deref() != Some("tc_route_6") })
        );
    }

    #[tokio::test]
    async fn invalid_explicit_identity_searches_never_execute_or_ack_deferred_prefix() {
        for (case, arguments) in [
            (
                "missing-identity-match",
                r#"{"data_type":"search","query":"CRWV","entity_route":"crwv"}"#,
            ),
            (
                "invalid-identity-match",
                r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"tickerish"}"#,
            ),
        ] {
            let prefix =
                format!("数据时间：北京时间 2026-07-19 09:31；行情口径：{case} 仅使用可核验资料");
            let llm =
                StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some(format!("tc_{case}")),
                    name: Some("data_fetch".to_string()),
                    arguments: arguments.to_string(),
                }]]);
            let registry_calls = Arc::new(AtomicUsize::new(0));
            let tool_observer = Arc::new(MockToolObserver::default());
            let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
            let mut registry = ToolRegistry::new();
            registry.register(Box::new(CountingDataFetchTool {
                calls: registry_calls.clone(),
            }));
            let agent = FunctionCallingAgent::new(
                Arc::new(llm),
                Arc::new(registry),
                String::new(),
                1,
                None,
            )
            .with_agent_owned_finance_loop(true)
            .with_service_owned_initial_prefix(Some(prefix), None)
            .with_tool_observer(Some(tool_observer.clone()))
            .with_stream_observer(Some(stream_observer.clone()));
            let mut context = AgentContext::new(format!("invalid-explicit-{case}"));

            let response = agent.run("分析 CRWV", &mut context).await;

            assert!(!response.success, "{case}");
            assert_eq!(
                response.error.as_deref(),
                Some("max_iterations_exceeded:1"),
                "{case}"
            );
            assert!(response.tool_calls_made.is_empty(), "{case}");
            assert_eq!(registry_calls.load(Ordering::SeqCst), 0, "{case}");
            assert!(
                tool_observer
                    .events
                    .lock()
                    .expect("tool observer events")
                    .is_empty(),
                "{case}"
            );
            assert!(
                stream_observer
                    .events
                    .lock()
                    .expect("stream events")
                    .is_empty(),
                "{case} must not ACK the deferred prefix"
            );
            assert!(
                stream_observer.committed_visible_prefix().is_none(),
                "{case}"
            );
            let rejected = context
                .messages
                .iter()
                .find(|message| message.tool_call_id.as_deref() == Some(&format!("tc_{case}")))
                .expect("invalid identity rejection");
            let rejected: Value = serde_json::from_str(
                rejected
                    .content
                    .as_deref()
                    .expect("invalid identity rejection content"),
            )
            .expect("parse invalid identity rejection");
            assert_eq!(rejected["error"], "invalid_identity_search_shape", "{case}");
        }
    }

    #[tokio::test]
    async fn precommitted_service_prefix_allows_known_read_only_finance_tools() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\nCRWV 的本轮核验结果。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_snapshot_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"crwv"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()))
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("precommitted-read-only".to_string());

        let response = agent.run("分析 CRWV", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
            ]
        );
        assert!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .is_empty(),
            "the runner already published the prefix; model body remains deferred until the final suffix"
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
    }

    #[tokio::test]
    async fn precommitted_service_prefix_allows_non_executable_image_skill_loading() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!(
            "{prefix}\n\n已读取本轮图片文字，先按截图中可确认的持仓和盈亏给出风险处置顺序。"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_image_skill".to_string()),
                name: Some("skill_tool".to_string()),
                arguments: r#"{"skill_name":"image_understanding","execute_script":false}"#
                    .to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let skill_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingSkillTool {
            calls: skill_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), Some(prefix.clone()))
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("precommitted-image-skill".to_string());

        let response = agent.run("分析持仓截图", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(skill_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .as_slice(),
            ["start:skill_tool", "done:skill_tool:true"]
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
    }

    #[tokio::test]
    async fn deferred_service_prefix_commits_after_first_valid_datafetch_batch() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\nNebius 的本轮核验结果。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_nebius".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"NVIDIA","entity_route":"nebius","identity_match":"name_or_alias"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_snapshot_nebius".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"NVDA","entity_route":"nebius"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), None)
                .with_pre_final_prefix_commit(true)
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("deferred-service-prefix".to_string());

        let response = agent.run("Nebius 有哪些大A类似标的", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .as_slice(),
            [format!("commit:{prefix}")],
            "the service prefix is ACKed after DataFetch activation and the later model body stays deferred"
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
    }

    #[tokio::test]
    async fn zero_budget_datafetch_does_not_execute_or_commit_deferred_service_prefix() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![
            ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_budget_zero".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            },
        ]]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 1, None)
                .with_agent_owned_finance_loop(true)
                .with_tool_call_budget(Some(0), HashMap::new())
                .with_service_owned_initial_prefix(Some(prefix), None)
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("zero-budget-deferred-prefix".to_string());

        let response = agent.run("分析 CRWV", &mut context).await;

        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("max_iterations_exceeded:1"));
        assert_eq!(response.iterations, 1);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts")
                .as_slice(),
            [1]
        );
        assert!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .is_empty(),
            "a budget-rejected DataFetch must never enter registry execution"
        );
        assert!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .is_empty(),
            "a budget-rejected DataFetch must not publish the deferred prefix"
        );
        assert!(stream_observer.committed_visible_prefix().is_none());
        let rejected = context
            .messages
            .iter()
            .find(|message| message.tool_call_id.as_deref() == Some("tc_budget_zero"))
            .expect("budget rejection tool result");
        assert!(
            rejected
                .content
                .as_deref()
                .is_some_and(|content| content.contains("tool call limit reached (0)"))
        );
    }

    #[tokio::test]
    async fn deferred_service_prefix_blocks_unknown_tool_after_commit_and_still_answers() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\nCoreWeave：未知工具未执行；以下仅使用已取得的实体证据。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_unknown_after_commit".to_string()),
                name: Some("counting_tool".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let unknown_calls = Arc::new(AtomicUsize::new(0));
        let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(CountingTool {
            calls: unknown_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), None)
                .with_pre_final_prefix_commit(true)
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("deferred-prefix-unknown-after".to_string());

        let response = agent.run("分析 CoreWeave", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(unknown_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .as_slice(),
            [format!("commit:{prefix}")]
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) != Some("tc_unknown_after_commit")
                })
            })
        }));
    }

    #[tokio::test]
    async fn unknown_tool_in_initial_finance_batch_is_blocked_before_all_execution_and_answers() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\nCoreWeave：本轮没有执行不安全批次，先给出信息核对清单。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_unknown_same_batch".to_string()),
                    name: Some("counting_tool".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let unknown_calls = Arc::new(AtomicUsize::new(0));
        let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.clone()));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(CountingTool {
            calls: unknown_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 1, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(prefix.clone()), None)
                .with_pre_final_prefix_commit(true)
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("deferred-prefix-mixed-initial".to_string());

        let response = agent.run("分析 CoreWeave", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(unknown_calls.load(Ordering::SeqCst), 0);
        // The header line forwards together with its terminating newline so
        // the downstream observer can validate and commit the completed line.
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .as_slice(),
            [format!("final:{prefix}\n")]
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(prefix.clone())
        );
    }

    #[tokio::test]
    async fn precommitted_service_prefix_blocks_unknown_first_batch_and_still_answers() {
        let prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露".to_string();
        let answer = format!("{prefix}\n\nCRWV：未知工具没有执行；当前证据不足处已单独标出。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_unknown".to_string()),
                name: Some("counting_tool".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver {
            prefix: prefix.clone(),
            accumulated: Mutex::new(prefix.clone()),
            events: Mutex::new(Vec::new()),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 2, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(
                    Some(prefix),
                    Some(stream_observer.prefix.clone()),
                )
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer));
        let mut context = AgentContext::new("precommitted-unknown-tool".to_string());

        let response = agent.run("分析 CRWV", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .is_empty()
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) != Some("tc_unknown")
                })
            })
        }));
    }

    #[tokio::test]
    async fn agent_owned_required_round_never_forwards_final_deltas() {
        let answer =
            "数据时间：北京时间 2026-07-19 09:31；行情口径：不应提前可见\n\n结构取证尚未完成。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_agent_owned_finance_loop(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("required-final-remains-hidden".to_string());

        let response = agent
            .run(
                "【本轮用户输入】CRWV 怎么看\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .as_slice(),
            [ToolChoiceMode::Auto, ToolChoiceMode::Required]
        );
        assert!(
            observer.events.lock().expect("stream events").is_empty(),
            "a Required round below the evidence floor is never an early-final source"
        );
    }

    #[tokio::test]
    async fn agent_owned_tool_round_blocks_unknown_call_and_resets_uncommitted_candidate() {
        let required_prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：";
        let compatible_fragment = "数据时间：北京时间 2026-07-19 09:";
        let answer = format!("{required_prefix}最新可得、非逐笔\n\n最终回答");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta(compatible_fragment.to_string()),
                ChatStreamEvent::ContentDelta("32；行情口径：错误时间".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_count".to_string()),
                    name: Some("counting_tool".to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let calls = Arc::new(AtomicUsize::new(0));
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_agent_owned_finance_loop(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("uncommitted-final-candidate".to_string());

        let response = agent
            .run(
                &format!("【本轮用户输入】CRWV 怎么看\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `{required_prefix}` 开头。"),
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            observer.events.lock().expect("stream events").as_slice(),
            [
                format!("final:{compatible_fragment}"),
                "reset".to_string(),
                format!("final:{required_prefix}"),
            ],
            "a wrong Session timestamp resets the partial candidate; the accepted body stays buffered behind the canonical header"
        );
    }

    #[tokio::test]
    async fn committed_finance_header_allows_registered_read_only_continuation_without_reset() {
        let required_prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露";
        let header_lead = "数据时间：北京时间 2026-07-19 09:31；行情口径：";
        let header_basis = "本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露";
        let answer = format!("{required_prefix}\n\nCRWV 的同日行情与原因核验结果。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta(header_lead.to_string()),
                ChatStreamEvent::ContentDelta(header_basis.to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_late_web".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CRWV same-day move cause"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let tool_observer = Arc::new(MockToolObserver::default());
        let stream_observer = Arc::new(CommittedPrefixStreamObserver::new(
            required_prefix.to_string(),
        ));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_agent_owned_finance_loop(true)
                .with_service_owned_initial_prefix(Some(required_prefix.to_string()), None)
                .with_tool_observer(Some(tool_observer.clone()))
                .with_stream_observer(Some(stream_observer.clone()));
        let mut context = AgentContext::new("committed-header-read-only-tool".to_string());

        let response = agent.run("CRWV 周五为什么大跌", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 4);
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:web_search",
                "done:web_search:true",
            ]
        );
        assert_eq!(
            stream_observer.committed_visible_prefix(),
            Some(required_prefix.to_string())
        );
        assert!(
            stream_observer
                .events
                .lock()
                .expect("stream events")
                .iter()
                .all(|event| event != "reset"),
            "an irreversible header must remain visible across the safe continuation"
        );
        assert!(context.messages.iter().any(|message| {
            message.tool_calls.as_ref().is_some_and(|tool_calls| {
                tool_calls.iter().any(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) == Some("tc_late_web")
                })
            })
        }));
    }

    #[test]
    fn listing_final_requires_current_inactive_evidence_before_delisting_claim() {
        let runtime_input =
            "【本轮用户输入】\nsndk财报前瞻\n\n【本轮最终回答契约：由主 Agent 一次完成】";
        let denied = "数据时间：北京时间 2026-08-03 12:52；行情口径：可核验\n\nSNDK（Sandisk Corporation）已退市，无法提供当前财报前瞻。";

        assert_eq!(
            explicit_security_symbols(runtime_input),
            BTreeSet::from(["SNDK".to_string()])
        );
        assert!(explicit_security_symbols("Sandisk公司财报前瞻").is_empty());

        let unsupported = listing_final_violations(runtime_input, denied, &[]);
        assert_eq!(unsupported.len(), 1);
        assert!(unsupported[0].contains("没有 inactive_listing"));

        let active = vec![ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: json!({"data_type":"earnings_outlook","ticker":"SNDK"}),
            result: json!({
                "hone_security_listing_evidence": {
                    "status":"active_listing",
                    "requested_symbol":"SNDK"
                }
            }),
            tool_call_id: Some("tc_sndk_outlook".to_string()),
        }];
        let contradicted = listing_final_violations(runtime_input, denied, &active);
        assert_eq!(contradicted.len(), 1);
        assert!(contradicted[0].contains("已确认 active_listing"));
        let deterministic = deterministic_listing_gap_response(
            Some("数据时间：北京时间 2026-08-03 12:52；行情口径："),
            &active,
            &contradicted,
        );
        assert!(deterministic.contains("SNDK"));
        assert!(deterministic.contains("当前上市交易"));
        assert!(!deterministic.contains("WDC"));
        assert!(
            listing_final_violations(
                runtime_input,
                "SNDK 当前在 Nasdaq 上市交易；历史收购不能覆盖后续重新上市事实。",
                &active,
            )
            .is_empty()
        );

        let inactive = vec![ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: json!({"data_type":"snapshot","ticker":"SNDK"}),
            result: json!({
                "hone_security_listing_evidence": {
                    "status":"inactive_listing",
                    "requested_symbol":"SNDK"
                }
            }),
            tool_call_id: Some("tc_sndk_inactive".to_string()),
        }];
        assert!(listing_final_violations(runtime_input, denied, &inactive).is_empty());
    }

    #[test]
    fn separate_quote_and_profile_form_active_listing_evidence() {
        let calls = vec![
            ToolCallMade {
                name: "data_fetch".to_string(),
                arguments: json!({"data_type":"quote","ticker":"SNDK"}),
                result: json!({"data":[{"symbol":"SNDK","price":238.15}]}),
                tool_call_id: Some("tc_quote".to_string()),
            },
            ToolCallMade {
                name: "data_fetch".to_string(),
                arguments: json!({"data_type":"profile","ticker":"SNDK"}),
                result: json!({"data":[{
                    "symbol":"SNDK",
                    "exchangeShortName":"NASDAQ",
                    "isActivelyTrading":true
                }]}),
                tool_call_id: Some("tc_profile".to_string()),
            },
        ];
        let states = current_turn_listing_states(&calls);
        assert!(states.get("SNDK").is_some_and(|state| state.active));
        assert!(
            listing_final_violations(
                "【本轮用户输入】SNDK 财报前瞻",
                "SNDK 当前上市交易。",
                &calls,
            )
            .is_empty()
        );
        assert_eq!(
            listing_final_violations("【本轮用户输入】SNDK 财报前瞻", "SNDK 已退市。", &calls,)
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn sndk_malformed_outlook_keeps_valid_search_and_corrects_delisting_final() {
        let wrong = concat!(
            "数据时间：北京时间 2026-08-03 13:19；行情口径：本轮仅使用可核验资料\n\n",
            "SNDK 已于 2016 年退市，无法提供财报前瞻，请改看 WDC。"
        );
        let corrected = concat!(
            "数据时间：北京时间 2026-08-03 13:19；行情口径：DataFetch 最新可得、非逐笔\n\n",
            "SNDK（Sandisk Corporation）当前在 Nasdaq 上市交易。本轮 earnings_outlook 已取得 2026-08-05 财报日历覆盖，以下按当前上市公司完成前瞻。"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_sndk".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"SNDK","entity_route":"sndk","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_bad_outlook".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"earnings_outlook","query":"SNDK","entity_route":"sndk"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_web_sndk".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"Sandisk SNDK earnings August 2026 investor relations"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_outlook_sndk".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"earnings_outlook","ticker":"SNDK","entity_route":"sndk"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(wrong.to_string())],
            vec![ChatStreamEvent::ContentDelta(corrected.to_string())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let data_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(SndkFinanceEvidenceTool {
            calls: data_calls.clone(),
        }));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_agent_owned_finance_loop(true)
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("sndk-malformed-outlook-recovery".to_string());

        let response = agent
            .run(
                "【本轮用户输入】\nsndk财报前瞻\n\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-08-03 13:19；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, corrected);
        assert_eq!(response.iterations, 4);
        assert_eq!(data_calls.load(Ordering::SeqCst), 2);
        assert_eq!(response.tool_calls_made.len(), 4);
        let rejected = response
            .tool_calls_made
            .iter()
            .find(|call| call.tool_call_id.as_deref() == Some("tc_bad_outlook"))
            .expect("malformed outlook should remain auditable");
        assert_eq!(rejected.result["error"], "invalid_read_only_tool_shape");
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:web_search",
                "done:web_search:true",
                "start:data_fetch",
                "done:data_fetch:true",
            ]
        );
        let messages = seen_messages.lock().expect("seen messages");
        let correction = messages
            .last()
            .and_then(|round| round.last())
            .and_then(|message| message.content.as_deref())
            .expect("listing correction prompt");
        assert!(correction.contains("内部终稿上市状态纠正"));
        assert!(correction.contains("active_listing 标的：SNDK"));
    }

    #[test]
    fn market_move_final_check_rejects_wrong_weekday_and_unlocalized_cause() {
        let runtime_input = "美股周五为什么暴跌\n\n\
            【本轮证券实体发现：主 Agent 工具循环】\n候选仅用于取证。\n\n\
            【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
            当前原话提到周五；不晚于当前市场本地日历的最近同名候选日期是 2026-07-24 周五。\n\n\
            【本轮最终回答契约：由主 Agent 一次完成】";
        let mut context = AgentContext::new("market-move-final-check".to_string());
        context.add_tool_result(
            "tc_web",
            "web_search",
            &json!({
                "results": [{
                    "title": "same-day report",
                    "url": "https://example.test/capacity",
                    "published_date": "2026-07-24",
                    "content": "same-day market report"
                }]
            })
            .to_string(),
        );

        let invalid = "数据时间：北京时间 2026-07-26 16:00；行情口径：可核验\n\n\
            目标时段是 2026-07-24（周四）。标普与纳斯达克走势分化。\n\n\
            核心驱动是候选新闻摘要。";
        let violations = market_move_final_violations(runtime_input, invalid, &context, 0);
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("星期应为周五"))
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("目标日期和本轮工具实际返回的原始 URL"))
        );

        let valid = "数据时间：北京时间 2026-07-26 16:00；行情口径：可核验\n\n\
            目标时段是 2026-07-24（周五）。标普与纳斯达克走势分化。\n\n\
            已核验原因：2026-07-24（周五）的同日来源支持该事件。\
            [same-day report](https://example.test/capacity)";
        assert!(market_move_final_violations(runtime_input, valid, &context, 0).is_empty());

        let single_stock_runtime = "【近期用户原话，仅用于理解本轮指代】\n美股为什么大跌\n\n\
            【本轮用户输入】\nHIMS 周五为什么大跌\n\n\
            【本轮证券实体发现：主 Agent 工具循环】\n候选仅用于取证。\n\n\
            【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
            当前原话提到周五；不晚于当前市场本地日历的最近同名候选日期是 2026-07-24 周五。";
        assert_eq!(
            original_market_move_request(single_stock_runtime),
            "HIMS 周五为什么大跌"
        );
        let single_stock_gap = "数据时间：北京时间 2026-07-26 16:00；行情口径：可核验\n\n\
            目标时段是 2026-07-24（周五），HIMS 的行情范围已核验。\n\n\
            原因本轮未完全核验。";
        assert!(
            market_move_final_violations(single_stock_runtime, single_stock_gap, &context, 0)
                .is_empty(),
            "an older broad-market reference must not impose broad-index coverage on the current single-stock question"
        );
    }

    #[test]
    fn market_move_date_anchor_starts_existing_investment_research_flow() {
        let runtime_input = "mrvl下跌原因是啥呢\n\n\
            【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
            目标日期候选是 2026-08-21 周五。";

        assert!(market_move_starts_investment_research(runtime_input, true));
        assert!(!market_move_starts_investment_research(
            runtime_input,
            false
        ));
        assert!(!market_move_starts_investment_research(
            "mrvl最近怎么看",
            true
        ));
    }

    #[test]
    fn close_price_claim_does_not_fire_on_revenue_or_change_wording() {
        // `收` used to be its own alternative (hitting 营收 / 回收 / 吸收) and the
        // bare words 收盘 / 收市 fired anywhere, including the comparison the
        // change-basis rules themselves require.
        let close_claim = |content: &str| {
            regex::Regex::new(
                r"(?i)(?:收报|收于|收在|收盘价?\s*(?:为|是|报)?|closing price(?:\s+(?:of|was|is))?|closed at|close price(?:\s+(?:of|was|is))?)\s*[:：]?\s*[$¥€£]?\s*[0-9]+(?:[.,][0-9]+)?\s*(?:%|％)?",
            )
            .expect("close value regex")
            .find_iter(content)
            .any(|hit| !hit.as_str().ends_with('%') && !hit.as_str().ends_with('％'))
        };
        for benign in [
            "AAOI 单季营收 6.5 亿美元，同比下滑。",
            "本季度净营收 8000 万美元。",
            "公司回收 3 亿美元现金用于偿债。",
            "AAOI 较上一常规交易日收盘价下跌 6.22%。",
            "常规日涨跌用 regular 窗口的 close-to-close 口径，收盘价对比为 -5.57%。",
        ] {
            assert!(!close_claim(benign), "should not fire: {benign}");
        }
        for claim in [
            "AAOI 收于 $18.20。",
            "股价收报 18.20 美元。",
            "常规时段收盘价 18.20 美元。",
            "NVDA closed at 217.55 on Friday.",
        ] {
            assert!(close_claim(claim), "should fire: {claim}");
        }
    }

    #[test]
    fn change_basis_reconciliation_skips_percentages_that_are_other_metrics() {
        // A move answer has to quantify the event; those percentages sit on the
        // same line as the ticker but are not change claims.
        for other in [
            " 的毛利率从 ",
            " 本次增发的稀释比例约 ",
            " 的数据中心收入占比 ",
            " implied upside of ",
            " 的合理价下移约 ",
        ] {
            assert!(
                percentage_names_another_metric(other),
                "should be skipped: {other}"
            );
        }
        for change in [" 下跌 ", " 当日 ", "：", " closed down "] {
            assert!(
                !percentage_names_another_metric(change),
                "should still be reconciled: {change}"
            );
        }
    }

    #[test]
    fn market_move_quote_evidence_prefers_server_change_basis() {
        let mut context = AgentContext::new("aaoi-change-basis".to_string());
        context.add_tool_result(
            "tc_aaoi",
            "data_fetch",
            &json!({
                "data": [{
                    "symbol": "AAOI",
                    "price": 124.82,
                    "previousClose": 129.10,
                    "changesPercentage": -3.46,
                    "hone_change_basis": {
                        "pct": -3.32,
                        "label": "常规时段涨跌（最新价较上一交易日收盘）",
                        "from": 129.10,
                        "to": 124.82
                    },
                    "hone_quote_time": {
                        "market_date_new_york": "2026-08-21"
                    }
                }]
            })
            .to_string(),
        );

        let quotes = current_turn_market_move_quotes(&context, 0);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].symbol, "AAOI");
        assert_eq!(quotes[0].change_percentage, -3.32);

        let mut raw_only = AgentContext::new("aaoi-raw-provider-change".to_string());
        raw_only.add_tool_result(
            "tc_aaoi_raw",
            "data_fetch",
            &json!({
                "data": [{
                    "symbol": "AAOI",
                    "changesPercentage": -3.46,
                    "hone_quote_time": {
                        "market_date_new_york": "2026-08-21"
                    }
                }]
            })
            .to_string(),
        );
        assert!(
            current_turn_market_move_quotes(&raw_only, 0).is_empty(),
            "raw provider changesPercentage must not become authoritative quote evidence"
        );
    }

    #[test]
    fn market_move_final_uses_quote_date_and_preserves_verified_scope_in_gap_response() {
        let runtime_input = "美股为什么大跌\n\n\
            【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
            Session 北京时间：2026-07-26 16:51 周日；对应市场本地民用时间：2026-07-26 04:51 周日。\n\n\
            【本轮最终回答契约：由主 Agent 一次完成】";
        let mut context = AgentContext::new("market-move-quote-date".to_string());
        for (call_id, symbol, change) in [
            ("tc_spy", "SPY", 0.1016),
            ("tc_qqq", "QQQ", -1.11712),
            ("tc_dia", "DIA", 0.48425),
            ("tc_iwm", "IWM", -0.31497),
        ] {
            context.add_tool_result(
                call_id,
                "data_fetch",
                &json!({
                    "data": [{
                        "symbol": symbol,
                        "changesPercentage": change,
                        "hone_change_basis": {
                            "pct": change,
                            "label": "常规时段涨跌（最新价较上一交易日收盘）"
                        },
                        "hone_quote_time": {
                            "beijing": "2026-07-25 04:00:00 +08:00",
                            "market_date_new_york": "2026-07-24"
                        }
                    }]
                })
                .to_string(),
            );
        }
        context.add_tool_result(
            "tc_same_day",
            "web_search",
            &json!({
                "results": [{
                    "title": "Markets News, July 24, 2026",
                    "url": "https://example.test/july-24",
                    "content": "Mixed session on Friday as chip stocks dropped."
                }]
            })
            .to_string(),
        );

        assert_eq!(
            current_turn_market_move_quote_date(&context, 0),
            NaiveDate::from_ymd_opt(2026, 7, 24)
        );
        assert!(content_contains_date(
            "Markets News, July 24, 2026",
            NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date")
        ));
        assert_eq!(
            current_turn_source_urls_for_date(
                &context,
                0,
                NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date")
            ),
            ["https://example.test/july-24"]
        );

        let invalid = "数据时间：北京时间 2026-07-26 16:51；行情口径：可核验\n\n\
            目标时段：2026-07-26（周日）。SPY 与 QQQ 走势分化。\n\n\
            本轮 quote 证据的纽约日历日期为 2026-07-24。\n\n\
            原因本轮未完全核验。";
        let violations = market_move_final_violations(runtime_input, invalid, &context, 0);
        assert!(violations.iter().any(|violation| {
            violation.contains("声明的目标日期 2026-07-26 与本轮证据锚定日期 2026-07-24 不一致")
        }));

        let gap = deterministic_market_move_gap_response(
            runtime_input,
            invalid,
            "数据时间：北京时间 2026-07-26 16:51；行情口径：",
            &context,
            0,
        );
        assert!(gap.contains("2026-07-24（周五，本轮最近可得 quote 的纽约日历日期"));
        assert!(gap.contains("SPY +0.10%"));
        assert!(gap.contains("QQQ -1.12%"));
        assert!(gap.contains("DIA +0.48%"));
        assert!(gap.contains("IWM -0.31%"));
        assert!(gap.contains("不支持“美股宽基全面大跌”的前提"));
        assert!(gap.contains("原因本轮未完全核验"));
        assert!(!gap.contains("目标时段：2026-07-26"));
    }

    #[test]
    fn market_move_final_rejects_quote_field_confusion_and_snippet_only_causes() {
        let runtime_input = "周五美股大跌是不是因为一个没有来源的市场传言？只说本轮能核验的\n\n\
            【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
            当前原话提到周五；不晚于当前市场本地日历的最近同名候选日期是 2026-07-24 周五。\n\n\
            【本轮最终回答契约：由主 Agent 一次完成】";
        let mut context = AgentContext::new("market-move-snippet-cause".to_string());
        for (call_id, symbol, change, exchange) in [
            ("tc_spy", "SPY", 0.1016, "AMEX"),
            ("tc_qqq", "QQQ", -1.11712, "NASDAQ"),
        ] {
            context.add_tool_result(
                call_id,
                "data_fetch",
                &json!({
                    "data": [{
                        "symbol": symbol,
                        "change": if symbol == "SPY" { 0.75 } else { -7.73 },
                        "changesPercentage": change,
                        "hone_change_basis": {
                            "pct": change,
                            "label": "常规时段涨跌（最新价较上一交易日收盘）"
                        },
                        "exchange": exchange,
                        "hone_quote_time": {
                            "beijing": "2026-07-25 04:00:00 +08:00",
                            "market_date_new_york": "2026-07-24"
                        }
                    }]
                })
                .to_string(),
            );
        }
        context.add_tool_result(
            "tc_snippets",
            "web_search",
            &json!({
                "hone_search_contract": {
                    "evidence_scope": {
                        "full_page_content": false,
                        "kind": "search_snippets"
                    }
                },
                "results": [{
                    "title": "Why Is Stock Market Down Today, July 24, 2026?",
                    "url": "https://example.test/july-24-snippet",
                    "content": "renewed geopolitical tensions and a spike in oil"
                }]
            })
            .to_string(),
        );

        let invalid = "数据时间：北京时间 2026-07-26 17:56；行情口径：本轮仅使用可核验资料\n\n\
            目标时段：2026-07-24（周五）。SPY 与 QQQ 走势分化。\n\n\
            本轮普通 quote 被错误写成纽约收盘报价。\n\n\
            | 标的 | 收盘价 | 涨跌幅 |\n\
            | SPY | $738.93 | +0.75% |\n\
            | QQQ | $684.23 | -1.12% |\n\n\
            SPY 收 $738.93，QQQ 收于 $684.23。\n\n\
            DataFetch SPY/QQQ quote 的 timestamp 对应纽交所 2026-07-24 16:00。\n\n\
            **2026-07-24 下跌原因的已核验新闻来源**：地缘风险与油价冲击。\
            https://example.test/july-24-snippet";
        let violations = market_move_final_violations(runtime_input, invalid, &context, 0);

        assert!(violations.iter().any(|violation| {
            violation.contains("SPY 的涨跌幅应来自服务端 hone_change_basis.pct（约 +0.10%）")
        }));
        assert!(violations.iter().any(|violation| {
            violation.contains("普通 quote 的时间字段不证明交易时段或收盘")
        }));
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("SPY 的交易所声明")
                    && violation.contains("exchange=AMEX"))
        );
        assert!(violations.iter().any(|violation| {
            violation.contains("目标日期和本轮工具实际返回的原始 URL")
        }));
        assert!(
            current_turn_source_urls_for_date(
                &context,
                0,
                NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date")
            )
            .is_empty(),
            "snippet-only search results must not satisfy original-source cause evidence"
        );

        let gap = deterministic_market_move_gap_response(
            runtime_input,
            invalid,
            "数据时间：北京时间 2026-07-26 17:56；行情口径：",
            &context,
            0,
        );
        assert!(gap.contains("SPY +0.10%"));
        assert!(gap.contains("QQQ -1.12%"));
        assert!(gap.contains("原因本轮未完全核验"));
        assert!(!gap.contains("july-24-snippet"));
        assert!(!gap.contains("收盘价"));
        assert!(!gap.contains("纽交所"));

        let mut paired_percentage_violations = Vec::new();
        append_market_move_quote_fact_violations(
            "SPY 与 QQQ 分别 +0.10% 和 -1.12%。",
            &context,
            0,
            &mut paired_percentage_violations,
        );
        assert!(
            paired_percentage_violations.is_empty(),
            "a compact ordered comparison is ambiguous to the per-symbol parser and must not be falsely rejected: {paired_percentage_violations:?}"
        );
    }

    #[tokio::test]
    async fn snippet_only_market_cause_uses_verified_quote_gap_without_another_generation() {
        let prefix = "数据时间：北京时间 2026-07-26 17:56；行情口径：";
        let invalid = format!(
            "{prefix}本轮仅使用可核验资料\n\n\
             目标时段：2026-07-24（周五）。SPY 与 QQQ 走势分化。\n\n\
             | 标的 | 收盘价 | 涨跌幅 |\n\
             | SPY | $738.93 | +0.75% |\n\
             | QQQ | $684.23 | -1.12% |\n\n\
             DataFetch SPY/QQQ quote 的 timestamp 对应纽交所 2026-07-24 16:00。\n\n\
             **2026-07-24 下跌原因的已核验新闻来源**：地缘风险与油价冲击。\
             https://example.test/july-24-snippet"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_spy".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","query":"SPY","entity_route":"spy","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_qqq".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","query":"QQQ","entity_route":"qqq","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_web".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"US stock market selloff July 24 2026 reasons"}"#
                        .to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(invalid)],
        ]);
        let calls = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(MarketMoveCanaryFinanceTool));
        registry.register(Box::new(SnippetOnlyMarketMoveWebTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("market-move-snippet-gap-e2e".to_string());
        let runtime_input = format!(
            "周五美股大跌是不是因为一个没有来源的市场传言？只说本轮能核验的\n\n\
             【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
             当前原话提到周五；不晚于当前市场本地日历的最近同名候选日期是 2026-07-24 周五。\n\n\
             【本轮最终回答契约：由主 Agent 一次完成】\
             第一条非空行必须严格以 `{prefix}` 开头。"
        );

        let response = agent.run(&runtime_input, &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            calls.lock().expect("seen calls").len(),
            2,
            "missing original-source cause evidence must select the deterministic quote gap instead of asking the model to rewrite"
        );
        assert!(response.content.starts_with(prefix));
        assert!(response.content.contains("2026-07-24（周五"));
        assert!(response.content.contains("SPY +0.10%"));
        assert!(response.content.contains("QQQ -1.12%"));
        assert!(response.content.contains("原因本轮未完全核验"));
        assert!(!response.content.contains("+0.75%"));
        assert!(!response.content.contains("收盘价"));
        assert!(!response.content.contains("纽交所"));
        assert!(!response.content.contains("july-24-snippet"));
    }

    #[tokio::test]
    async fn market_move_final_correction_keeps_only_header_visible_until_acceptance() {
        let prefix = "数据时间：北京时间 2026-07-26 16:00；行情口径：";
        let invalid = format!(
            "{prefix}本轮仅使用可核验资料\n\n\
             目标时段是 2026-07-24（周四）。标普与纳斯达克走势分化。\n\n\
             核心驱动是候选新闻摘要。"
        );
        let corrected = format!(
            "{prefix}本轮仅使用可核验资料\n\n\
             目标时段是 2026-07-24（周五）。标普与纳斯达克走势分化。\n\n\
             已核验原因：2026-07-24（周五）的同日来源支持该事件。\
             [same-day report](https://example.test/capacity)"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"market","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_web_market".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"2026-07-24 US market move"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_snapshot_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments:
                    r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"market"}"#
                        .to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(invalid)],
            vec![ChatStreamEvent::ContentDelta(corrected.clone())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let observer = Arc::new(CommittedPrefixStreamObserver::new(prefix.to_string()));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_agent_owned_finance_loop(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("market-move-final-correction".to_string());
        let runtime_input = format!(
            "美股周五为什么暴跌\n\n\
             【本轮证券实体发现：主 Agent 工具循环】\n候选仅用于取证。\n\n\
             【本轮涨跌归因日期锚点：只指导主 Agent 取证，不是行情或交易日事实】\n\
             当前原话提到周五；不晚于当前市场本地日历的最近同名候选日期是 2026-07-24 周五。\n\n\
             【本轮最终回答契约：由主 Agent 一次完成】\
             第一条非空行必须严格以 `{prefix}` 开头。"
        );

        let response = agent.run(&runtime_input, &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, corrected);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("tool choice modes")
                .first(),
            Some(&ToolChoiceMode::Required),
            "market-move turns must enter the existing investment research flow before any model answer"
        );
        let first_research_prompt = seen_messages
            .lock()
            .expect("seen messages")
            .first()
            .and_then(|messages| messages.last())
            .and_then(|message| message.content.as_deref())
            .map(str::to_string)
            .expect("first research prompt");
        assert!(first_research_prompt.contains("本轮只取证，不作答"));
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream observer events")
                .as_slice(),
            [format!("final:{prefix}")],
            "the rejected body must never flash, reset, or duplicate the durable header"
        );
        assert_eq!(
            observer.committed_visible_prefix(),
            Some(prefix.to_string())
        );
        let correction_request = seen_messages
            .lock()
            .expect("seen messages")
            .last()
            .cloned()
            .expect("correction request");
        let correction_prompt = correction_request
            .last()
            .and_then(|message| message.content.as_deref())
            .expect("correction prompt");
        assert!(correction_prompt.contains("内部终稿机械一致性纠正"));
        assert!(correction_prompt.contains("2026-07-24 的星期应为周五"));
        assert!(correction_prompt.contains("https://example.test/capacity"));
        let correction_template = correction_prompt
            .split("<unpublished_draft>")
            .next()
            .expect("correction template");
        assert!(correction_template.contains("整篇输出完整终稿"));
        assert!(correction_template.contains("归因之后的三段"));
        assert!(
            !correction_template.contains("个汉字") && !correction_template.contains("Bull/Bear"),
            "the mechanical correction round must not cap length or ban the answer's own analysis"
        );
        assert!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref())
                .is_some_and(|content| content == corrected)
        );
    }

    #[tokio::test]
    async fn committed_finance_header_blocks_a_late_tool_and_still_answers() {
        let required_prefix = "数据时间：北京时间 2026-07-19 09:31；行情口径：";
        let basis = "最新可得、非逐笔\n";
        let committed_header = required_prefix.to_string();
        let answer = format!(
            "{required_prefix}{basis}\nCRWV：未执行未知工具；以下结论仅使用已取得的行情与实体证据。"
        );
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta(required_prefix.to_string()),
                ChatStreamEvent::ContentDelta(basis.to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_late_count".to_string()),
                    name: Some("counting_tool".to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ]);
        let calls = Arc::new(AtomicUsize::new(0));
        let observer = Arc::new(CommittedPrefixStreamObserver::new(committed_header.clone()));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(GroundedFinanceEvidenceTool));
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_agent_owned_finance_loop(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("committed-header-late-tool".to_string());

        let response = agent
            .run(
                &format!("【本轮用户输入】CRWV 怎么看\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `{required_prefix}` 开头。"),
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(observer.committed_visible_prefix(), Some(committed_header));
        assert!(
            observer
                .events
                .lock()
                .expect("stream events")
                .iter()
                .all(|event| event != "reset"),
            "an irreversible header must never be reset"
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) != Some("tc_late_count")
                })
            })
        }));
    }

    #[tokio::test]
    async fn agent_owned_finance_loop_blocks_persistent_tool_and_still_answers() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮只使用已取得的实体证据\n\nCRWV 尚未加入持仓；先核对标的和数量，再执行写入。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_portfolio_add".to_string()),
                name: Some("portfolio".to_string()),
                arguments: r#"{"action":"add","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let portfolio_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(FailingPortfolioTool {
            calls: portfolio_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_agent_owned_finance_loop(true)
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finance-persistent-preflight".to_string());

        let response = agent.run("研究 CRWV 后把它加入持仓", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(portfolio_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .as_slice(),
            ["start:data_fetch", "done:data_fetch:true"],
            "the persistent call must be rejected before observer or registry execution"
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) != Some("tc_portfolio_add")
                })
            })
        }));
    }

    #[tokio::test]
    async fn finance_discovery_round_blocks_executable_skill_and_still_answers() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮没有执行脚本\n\nCRWV：先给出不依赖脚本的核对框架。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_executable_skill".to_string()),
                    name: Some("skill_tool".to_string()),
                    arguments: r#"{"skill_name":"stock_research","execute_script":true}"#
                        .to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let skill_calls = Arc::new(AtomicUsize::new(0));
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(CountingSkillTool {
            calls: skill_calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_agent_owned_finance_loop(true)
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finance-executable-skill-preflight".to_string());

        let response = agent.run("研究 CRWV", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(skill_calls.load(Ordering::SeqCst), 0);
        assert!(
            tool_observer
                .events
                .lock()
                .expect("tool observer events")
                .is_empty(),
            "the mixed discovery/write round must fail before any observer or registry call"
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call.get("id").and_then(Value::as_str) != Some("tc_executable_skill")
                })
            })
        }));
    }

    #[test]
    fn agent_owned_history_prioritizes_nearest_user_turns_before_budgeting() {
        let agent = FunctionCallingAgent::new(
            Arc::new(StreamingMockLlmProvider::with_rounds(vec![])),
            Arc::new(ToolRegistry::new()),
            String::new(),
            1,
            None,
        );
        let mut context = AgentContext::new("bounded-history-priority".to_string());
        context.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some("RESTORED_INVOKED_SKILL_PROMPT".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: Some(HashMap::from([(
                RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY.to_string(),
                Value::Bool(true),
            )])),
        });
        context.add_user_message("excluded-fifth-most-recent");
        context.add_assistant_message("旧助手错误行情：NBIS 15 USD", None);
        context.add_user_message(&format!("long-fourth-most-recent:{}", "x".repeat(4_100)));
        context.add_user_message("third-most-recent");
        context.add_user_message("second-most-recent");
        context.add_user_message("nearest-reference: 第二个呢");
        let turn_message_start = context.messages.len();
        context.add_user_message("它和英伟达是什么关系？");

        let messages = agent.build_agent_owned_messages(&context, None, turn_message_start);
        let history = messages
            .iter()
            .filter_map(|message| message.content.as_deref())
            .find(|content| content.contains("近期用户原话，仅用于理解本轮指代"))
            .expect("bounded prior-user context");

        assert!(history.contains("nearest-reference: 第二个呢"));
        assert!(history.contains("second-most-recent"));
        assert!(history.contains("third-most-recent"));
        assert!(history.contains("long-fourth-most-recent:"));
        assert!(!history.contains("excluded-fifth-most-recent"));
        assert!(!history.contains("15 USD"));
        assert!(messages.iter().any(|message| {
            message.content.as_deref() == Some("RESTORED_INVOKED_SKILL_PROMPT")
        }));
        let long_index = history
            .find("long-fourth-most-recent:")
            .expect("fourth-most-recent marker");
        let third_index = history.find("third-most-recent").expect("third marker");
        let second_index = history.find("second-most-recent").expect("second marker");
        let nearest_index = history
            .find("nearest-reference: 第二个呢")
            .expect("nearest marker");
        assert!(long_index < third_index);
        assert!(third_index < second_index);
        assert!(second_index < nearest_index);
    }

    #[tokio::test]
    async fn production_natural_mode_blocks_unregistered_batch_and_still_answers() {
        const INVENTED_TOOL_NAME: &str = "invented_research_gate";
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：本轮报价最新可得、非逐笔\n\nCRWV 的关系结论仅按本轮证据表述。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"crwv","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","symbol":"CRWV","entity_route":"crwv"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_hallucinated_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_invented_gate".to_string()),
                    name: Some(INVENTED_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta(answer.to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ],
        ]);
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_messages = llm.seen_messages.clone();
        let tool_observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true)
                .with_agent_owned_finance_loop(true)
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("natural-hallucinated-finish".to_string());

        let response = agent
            .run(
                "【本轮用户输入】crwv和英伟达有什么关系\n【本轮最终回答契约：由主 Agent 一次完成】第一条非空行必须严格以 `数据时间：北京时间 2026-07-19 09:31；行情口径：` 开头。",
                &mut context,
            )
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(response.tool_calls_made[0].arguments["query"], "crwv");
        assert_eq!(
            response.tool_calls_made[0].arguments["identity_match"],
            "exact_symbol"
        );
        assert_eq!(
            response.tool_calls_made[0].result["data"][0]["symbol"],
            "CRWV"
        );
        assert!(
            response
                .tool_calls_made
                .iter()
                .all(|call| call.name != FINISH_RESEARCH_TOOL_NAME
                    && call.name != INVENTED_TOOL_NAME)
        );
        let observer_events = tool_observer.events.lock().expect("tool observer events");
        assert!(
            observer_events
                .iter()
                .all(|event| !event.contains(FINISH_RESEARCH_TOOL_NAME)
                    && !event.contains(INVENTED_TOOL_NAME)),
            "an unregistered model-invented tool must not flash progress or failure status"
        );
        assert!(
            observer_events
                .iter()
                .any(|event| event == "start:data_fetch")
        );
        drop(observer_events);
        assert!(
            seen_tool_names
                .lock()
                .expect("tool schemas")
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME),
            "natural mode must never expose the retired control schema"
        );
        let requests = seen_messages.lock().expect("requests");
        let final_request = serde_json::to_string(requests.last().expect("final request"))
            .expect("serialize request");
        assert!(!final_request.contains("tc_hallucinated_finish"));
        assert!(!final_request.contains("tc_invented_gate"));
        assert!(final_request.contains(INVENTED_TOOL_NAME));
        assert!(!final_request.contains("内部工具协议纠正"));
        assert!(!final_request.contains("finish_research 当前尚不可用"));
    }

    #[tokio::test]
    async fn finish_stays_hidden_until_each_nonempty_entity_route_is_covered() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvda".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote_nvda".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile_nvda".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: data_finish_arguments(&["tc_quote_crwv", "tc_quote_nvda"]),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "CRWV 与 NVDA 均已按各自实体路线核验。".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EntityRouteFinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer));
        let mut context = AgentContext::new("per-entity-route-finish".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "CRWV 与 NVDA 均已按各自实体路线核验。");
        assert_eq!(response.tool_calls_made.len(), 6);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 1, 2, 0],
            "finish must stay hidden after CRWV-only coverage and appear only after NVDA is covered"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
    }

    #[tokio::test]
    async fn relationship_search_does_not_offer_finish_until_post_identity_evidence() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvidia".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta("discarded unavailable-finish preamble".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_unavailable_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: relationship_finish_arguments("tc_web_relationship"),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "continue preamble terminal".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("relationship-stage-gate".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "continue preamble terminal");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 6);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].arguments["data_type"], "search");
        assert_eq!(response.tool_calls_made[2].arguments["data_type"], "quote");
        assert_eq!(
            response.tool_calls_made[3].arguments["data_type"],
            "profile"
        );
        assert_eq!(
            response.tool_calls_made[4].arguments["data_type"],
            "profile"
        );
        assert_eq!(response.tool_calls_made[5].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 2, 3, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        let tool_names = seen_tool_names.lock().expect("stream tool names lock");
        assert!(
            tool_names[..3]
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[3]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[4].is_empty());
        drop(tool_names);
        assert_explicit_terminal_messages(&seen_messages);
        let terminal_transcript = serde_json::to_string(
            seen_messages
                .lock()
                .expect("stream messages lock")
                .last()
                .expect("terminal transcript"),
        )
        .expect("serialize terminal transcript");
        for required in [
            "$6.3B of unused capacity",
            "most-favored-nation relationship",
            "https://example.test/capacity",
            "https://example.test/mfn",
            "严格服从结构化交接",
            "URL 只用于定位来源，不证明句中内容",
        ] {
            assert!(
                terminal_transcript.contains(required),
                "missing {required}: {terminal_transcript}"
            );
        }
        assert!(!terminal_transcript.contains(r#"\"data_type\":\"quote\""#));
        assert!(!terminal_transcript.contains(r#"\"data_type\":\"profile\""#));
        assert!(!terminal_transcript.contains("CoreWeave NVIDIA relationship filing"));
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:continue preamble terminal"]
        );
        assert!(context.messages.iter().all(|message| {
            message.content.as_deref() != Some("discarded unavailable-finish preamble")
        }));
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
    }

    #[tokio::test]
    async fn sole_finish_preamble_is_hidden_before_terminal_synthesis() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("discarded finish preamble".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(
                "finish preamble terminal".to_string(),
            )],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finish-visible-preamble".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "finish preamble terminal");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_explicit_terminal_messages(&seen_messages);
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:finish preamble terminal"]
        );
        assert!(
            context
                .messages
                .iter()
                .all(|message| { message.content.as_deref() != Some("discarded finish preamble") })
        );
    }

    #[tokio::test]
    async fn malformed_finish_gets_one_hidden_same_agent_locator_correction() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_malformed_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{bad".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "malformed finish 后的唯一终稿".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EntityRouteFinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("malformed-finish".to_string());

        let response = agent.run("CRWV research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "malformed finish 后的唯一终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 2],
            "malformed handoff JSON must remain in the same Agent loop for one source-locator correction; it must not enter an empty-evidence terminal"
        );
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let correction_transcript =
            serde_json::to_string(seen_messages.last().expect("correction messages"))
                .expect("serialize correction transcript");
        assert!(correction_transcript.contains("上一次交接有 1 条 evidence locator 无法解析"));
        assert!(correction_transcript.contains("tc_quote"));
        assert!(correction_transcript.contains("tc_profile"));
        assert!(correction_transcript.contains("本轮 finish_research 可引用来源目录"));
        drop(seen_messages);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "the corrected active-round DirectFinal is returned once by the Agent runner, not streamed as a second terminal generation"
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
    }

    #[tokio::test]
    async fn fragmented_hidden_thinking_stays_internal_during_business_evidence_round() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_1".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("<thi".to_string()),
                ChatStreamEvent::ContentDelta("nk>private business thought</think>".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"relationship evidence"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_post_identity_snapshot".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: relationship_finish_arguments("tc_web_relationship"),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "隐藏思考后的终稿".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("hidden-business-thinking".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "隐藏思考后的终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 0],
            "fragmented hidden thinking must not replace the business evidence call or the later sole finish"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:隐藏思考后的终稿"]
        );
        assert!(context.messages.iter().all(|message| {
            message
                .content
                .as_deref()
                .is_none_or(|content| !content.contains("private business thought"))
        }));
    }

    #[tokio::test]
    async fn active_step_timeout_recovers_with_a_tools_disabled_answer() {
        let answer =
            "数据时间：北京时间 2026-07-26 20:00；行情口径：已取得资料\n\n基于现有证据的回答";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ])
        .pending_on_stream_calls(&[2]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-timeout".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 0]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        let records = audit.records.lock().expect("audit records lock");
        let timeout = records
            .iter()
            .find(|record| record.metadata["active_business_outcome"] == "timeout")
            .expect("timeout audit");
        assert_eq!(timeout.metadata["active_business_outcome"], "timeout");
        assert_eq!(timeout.metadata["retrying"].as_bool(), Some(true));
        assert_eq!(
            timeout.metadata["tools_disabled_recovery"].as_bool(),
            Some(true)
        );
        assert_eq!(
            timeout.metadata["terminal_authorized"].as_bool(),
            Some(false)
        );
    }

    #[test]
    fn configured_step_deadline_replaces_legacy_active_phase_cap() {
        let before = tokio::time::Instant::now();
        let (deadline, error) = active_business_deadline(None, Some(Duration::from_millis(100)));

        assert_eq!(error, AGENT_STEP_TIMEOUT_ERROR);
        assert!(
            deadline.saturating_duration_since(before) >= Duration::from_millis(80),
            "configured step deadline must not be shortened to the 25ms test fallback"
        );
    }

    #[tokio::test]
    async fn initial_stream_respects_one_overall_agent_deadline() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![]]).pending_on_stream_calls(&[1]);
        let stream_calls = llm.stream_calls.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_overall_timeout(Some(Duration::from_millis(10)))
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("initial-overall-timeout".to_string());

        let response = agent.run("hello", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains(AGENT_OVERALL_TIMEOUT_ERROR)),
            "{:?}",
            response.error
        );
        assert!(response.content.is_empty());
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(response.iterations, 1);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn initial_stream_respects_configured_step_deadline() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![]]).pending_on_stream_calls(&[1]);
        let stream_calls = llm.stream_calls.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_step_timeout(Some(Duration::from_millis(10)))
                .with_overall_timeout(Some(Duration::from_secs(1)));
        let mut context = AgentContext::new("initial-step-timeout".to_string());

        let response = agent.run("hello", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains(AGENT_STEP_TIMEOUT_ERROR)),
            "{:?}",
            response.error
        );
        assert_eq!(response.iterations, 1);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn hanging_tool_observer_is_bounded_before_execution() {
        let llm =
            StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_count".to_string()),
                name: Some("counting_tool".to_string()),
                arguments: "{}".to_string(),
            }]]);
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_tool_observer(Some(Arc::new(HangingStartObserver)))
                .with_step_timeout(Some(Duration::from_millis(10)))
                .with_overall_timeout(Some(Duration::from_secs(1)));
        let mut context = AgentContext::new("hanging-tool-observer".to_string());

        let response = agent.run("count once", &mut context).await;

        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some(AGENT_STEP_TIMEOUT_ERROR));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(response.tool_calls_made.is_empty());
    }

    #[tokio::test]
    async fn persistent_tool_error_reconciles_by_read_without_replaying_write() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_portfolio_add_1".to_string()),
                name: Some("portfolio".to_string()),
                arguments: r#"{"action":"add","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "CRWV 已加入持仓。".to_string(),
            )],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FailingPortfolioTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None);
        let mut context = AgentContext::new("persistent-tool-error".to_string());

        let response = agent.run("把 CRWV 加入持仓", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "CRWV 已加入持仓。");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 2);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(response.tool_calls_made[0].result["status"], "failed");
        assert_eq!(response.tool_calls_made[0].result["timeout"], false);
        assert_eq!(
            response.tool_calls_made[1].arguments,
            json!({"action":"view"})
        );
        assert_eq!(
            response.tool_calls_made[1].result["mutation_replayed"],
            false
        );
        assert_eq!(
            response.tool_calls_made[1].result["observed_state"]["holdings"][0]["symbol"],
            "CRWV"
        );
        assert_eq!(
            seen_tool_counts.lock().expect("tool counts").as_slice(),
            [1, 0]
        );
        let final_messages = seen_messages.lock().expect("seen messages");
        assert!(
            final_messages[1].iter().any(|message| {
                message.content.as_deref().is_some_and(|content| {
                    content.contains(PERSISTENT_MUTATION_FINALIZATION_INSTRUCTION)
                })
            }),
            "the tools-disabled final must be told to use the observed state"
        );
    }

    #[tokio::test]
    async fn persistent_tool_timeout_keeps_uncertain_trace_and_stops_the_agent() {
        let llm =
            StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_portfolio_add".to_string()),
                name: Some("portfolio".to_string()),
                arguments: r#"{"action":"add","ticker":"CRWV"}"#.to_string(),
            }]]);
        let stream_calls = llm.stream_calls.clone();
        let observer = Arc::new(MockToolObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(HangingPortfolioTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_overall_timeout(Some(Duration::from_millis(10)))
                .with_tool_observer(Some(observer.clone()));
        let mut context = AgentContext::new("persistent-tool-overall-timeout".to_string());

        let response = agent.run("把 CRWV 加入持仓", &mut context).await;

        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some(AGENT_OVERALL_TIMEOUT_ERROR));
        assert_eq!(response.iterations, 1);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 1);
        assert_eq!(response.tool_calls_made.len(), 1);
        let call = &response.tool_calls_made[0];
        assert_eq!(call.name, "portfolio");
        assert_eq!(call.arguments["action"], "add");
        assert_eq!(call.result["status"], "failed");
        assert_eq!(call.result["isError"], true);
        assert_eq!(call.result["timeout"], true);
        assert!(
            call.result["error"]
                .as_str()
                .is_some_and(|error| error.contains(AGENT_OVERALL_TIMEOUT_ERROR))
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            ["start:portfolio", "done:portfolio:false"]
        );
        assert!(context.messages.iter().any(|message| {
            message.role == "tool"
                && message
                    .content
                    .as_deref()
                    .is_some_and(|content| content.contains(AGENT_OVERALL_TIMEOUT_ERROR))
        }));
    }

    #[tokio::test]
    async fn successful_tools_reset_the_consecutive_active_failure_counter() {
        let first_business_empty = vec![ChatStreamEvent::ReasoningDelta(
            "first hidden-only business thought".to_string(),
        )];
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            first_business_empty,
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"relationship evidence"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_post_identity_snapshot".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ReasoningDelta(
                "second hidden-only business thought".to_string(),
            )],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: relationship_finish_arguments("tc_web_relationship"),
            }],
            vec![ChatStreamEvent::ContentDelta("唯一可见终稿".to_string())],
        ]);
        let delivered_events = llm.delivered_events.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            5,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-failure-counter-reset".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "唯一可见终稿");
        assert_eq!(response.iterations, 6);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(response.tool_calls_made[1].name, "web_search");
        assert_eq!(
            delivered_events.load(Ordering::SeqCst),
            25,
            "all six completed streams must be consumed through their lifecycle boundaries"
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 2, 3, 3, 0]
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:唯一可见终稿"]
        );
        assert!(context.messages.iter().all(|message| {
            message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                .and_then(Value::as_str)
                .is_none_or(|reasoning| !reasoning.contains("hidden-only business thought"))
        }));
        assert_explicit_terminal_messages(&seen_messages);
        let records = audit.records.lock().expect("audit records lock");
        let empties = records
            .iter()
            .filter(|record| record.metadata["active_business_outcome"].as_str() == Some("empty"))
            .collect::<Vec<_>>();
        assert_eq!(empties.len(), 2);
        assert!(
            empties
                .iter()
                .all(|record| record.metadata["retrying"].as_bool() == Some(true))
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.operation == "chat_terminal_without_tools")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn fallback_direct_final_is_preserved_without_terminal_synthesis() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Auto,
                    fallback: true,
                },
                ChatStreamEvent::ContentDelta("finite active draft".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finite-active-content-bypass".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "finite active draft");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(
            context
                .messages
                .iter()
                .any(|message| { message.content.as_deref() == Some("finite active draft") })
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 2);

        let records = audit.records.lock().expect("audit records lock");
        let direct_finals = records
            .iter()
            .filter(|record| {
                record.metadata["active_business_outcome"].as_str() == Some("direct_final")
            })
            .collect::<Vec<_>>();
        assert_eq!(direct_finals.len(), 1);
        let direct_final = direct_finals[0];
        assert!(direct_final.success);
        assert!(direct_final.error.is_none());
        assert_eq!(
            direct_final.metadata["requested_tool_choice"].as_str(),
            Some("required")
        );
        assert_eq!(
            direct_final.metadata["effective_tool_choice"].as_str(),
            Some("auto")
        );
        assert_eq!(
            direct_final.metadata["tool_choice_fallback"].as_bool(),
            Some(true)
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_empty_retries_once_then_recovers_with_a_tools_disabled_answer() {
        let answer = "数据时间：北京时间 2026-07-26 20:00；行情口径：已取得资料\n\n继续回答";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ReasoningDelta(
                "hidden-only active thought".to_string(),
            )],
            vec![ChatStreamEvent::ReasoningDelta(
                "second hidden-only active thought".to_string(),
            )],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-empty".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 1, 0]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 4);
        assert!(context.messages.iter().all(|message| {
            message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                .and_then(Value::as_str)
                .is_none_or(|reasoning| !reasoning.contains("hidden-only active thought"))
        }));
        let records = audit.records.lock().expect("audit records lock");
        let empties = records
            .iter()
            .filter(|record| record.metadata["active_business_outcome"].as_str() == Some("empty"))
            .collect::<Vec<_>>();
        assert_eq!(empties.len(), 2);
        assert_eq!(empties[0].metadata["retrying"].as_bool(), Some(true));
        assert_eq!(empties[1].metadata["retrying"].as_bool(), Some(false));
        assert!(
            empties
                .iter()
                .all(|record| { record.metadata["terminal_authorized"].as_bool() == Some(false) })
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn completion_auto_empty_retries_once_then_preserves_direct_final() {
        let answer = "数据时间：北京时间 2026-07-19 09:31；行情口径：最新可得、非逐笔\n\n正常终稿";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_search_crwv".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_search_nvidia".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"NVIDIA"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_snapshot".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"snapshot","ticker":"CRWV,NVDA"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ReasoningDelta(
                "provider returned no visible payload on the first Auto completion".to_string(),
            )],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            5,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("completion-auto-empty".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 4);
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 2]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "the empty Auto attempt and completed DirectFinal must remain one deferred answer"
        );
        assert_eq!(
            context
                .messages
                .iter()
                .filter(|message| message.role == "assistant" && message.tool_calls.is_none())
                .count(),
            1
        );
        let records = audit.records.lock().expect("audit records lock");
        let empty = records
            .iter()
            .find(|record| record.metadata["active_business_outcome"].as_str() == Some("empty"))
            .expect("empty Auto audit");
        assert_eq!(empty.metadata["tool_choice_mode"].as_str(), Some("auto"));
        assert_eq!(
            empty.metadata["requested_tool_choice"].as_str(),
            Some("auto")
        );
        assert_eq!(empty.metadata["retrying"].as_bool(), Some(true));
        assert!(records.iter().any(|record| {
            record.metadata["active_business_outcome"].as_str() == Some("direct_final")
        }));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_provider_error_recovers_with_a_tools_disabled_answer() {
        let answer = "数据时间：北京时间 2026-07-26 20:00；行情口径：已取得资料\n\n继续回答";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ])
        .failing_on_stream_calls(&[2]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-business-provider-error".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 0]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        let records = audit.records.lock().expect("audit records lock");
        let error = records
            .iter()
            .find(|record| record.metadata["active_business_outcome"] == "error")
            .expect("active error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(true));
        assert_eq!(
            error.metadata["tools_disabled_recovery"].as_bool(),
            Some(true)
        );
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
    }

    #[tokio::test]
    async fn initial_protocol_mismatch_recovers_with_a_tools_disabled_answer() {
        let answer = "数据时间：北京时间 2026-08-13 20:00；行情口径：已取得资料\n\n根据已完成的身份检索继续收口。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_crwv".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"crwv","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ])
        .failing_on_stream_call_with_error(
            2,
            "LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)",
        );
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("initial-protocol-mismatch-recovery".to_string());

        let response = agent.run("分析 CRWV", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 0]
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records
            .iter()
            .find(|record| {
                record.error.as_deref().is_some_and(|message| {
                    message.contains("tool call result does not follow tool call")
                })
            })
            .expect("protocol mismatch audit");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(true));
        assert_eq!(
            error.metadata["tools_disabled_recovery"].as_bool(),
            Some(true)
        );
        assert_eq!(
            error.metadata["context_overflow_bounded_recovery"].as_bool(),
            Some(false)
        );
    }

    #[tokio::test]
    async fn initial_stream_decode_failure_recovers_with_a_tools_disabled_answer() {
        let answer = "数据时间：北京时间 2026-08-13 20:30；行情口径：已取得资料\n\n已基于当前轮只读证据完成收口。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_search_nvda".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"NVDA","entity_route":"nvda","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![],
            vec![ChatStreamEvent::ContentDelta(answer.to_string())],
        ])
        .failing_on_stream_call_with_error(
            2,
            "LLM 错误: stream transport error: error decoding response body",
        );
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true);
        let mut context = AgentContext::new("initial-stream-decode-recovery".to_string());

        let response = agent.run("分析 NVDA", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 0]
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records
            .iter()
            .find(|record| {
                record
                    .error
                    .as_deref()
                    .is_some_and(|message| message.contains("error decoding response body"))
            })
            .expect("decode failure audit");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(true));
        assert_eq!(
            error.metadata["tools_disabled_recovery"].as_bool(),
            Some(true)
        );
        assert_eq!(
            error.metadata["context_overflow_bounded_recovery"].as_bool(),
            Some(false)
        );
    }

    #[tokio::test]
    async fn context_overflow_after_oversized_read_only_results_uses_bounded_same_agent_final() {
        let prefix = "数据时间：北京时间 2026-07-26 21:00；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露";
        let answer = format!("{prefix}\n\n结论：基于已取得证据继续完成本轮分析。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_oversized_search".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"NVDA","entity_route":"nvda","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![],
            vec![ChatStreamEvent::ContentDelta(answer.clone())],
        ])
        .failing_on_stream_call_with_error(
            2,
            "LLM 错误: invalid params, context window exceeds limit (2013)",
        );
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(OversizedFinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_agent_owned_finance_loop(true)
        .with_service_owned_initial_prefix(Some(prefix.to_string()), None);
        let mut context = AgentContext::new("context-overflow-bounded-final".to_string());

        let response = agent.run("分析 NVDA", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(
            response.tool_calls_made[0].result["oversized_blob"]
                .as_str()
                .is_some_and(|value| value.chars().count() > 100_000),
            "the complete tool result must remain available for audit"
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 0],
            "context recovery must not execute or offer tools again"
        );
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let recovery_messages = seen_messages.last().expect("bounded recovery request");
        let recovery_payload = serde_json::to_string(recovery_messages).expect("serialize request");
        assert!(recovery_payload.contains("本轮已执行工具的有界证据副本"));
        let recovery_evidence = recovery_messages
            .iter()
            .filter_map(|message| message.content.as_deref())
            .find(|content| content.contains("本轮已执行工具的有界证据副本"))
            .expect("bounded evidence message");
        assert!(recovery_evidence.contains("\"result_compacted\":true"));
        assert!(recovery_payload.contains("tc_oversized_search"));
        assert!(
            recovery_payload.chars().count() < CONTEXT_OVERFLOW_RECOVERY_TOTAL_TOOL_CHARS + 40_000
        );
        assert!(
            !recovery_payload.contains(&"OVERSIZED_SENTINEL".repeat(10_000)),
            "the complete oversized result must not be replayed to the provider"
        );
        let records = audit.records.lock().expect("audit records lock");
        let overflow = records
            .iter()
            .find(|record| {
                record.metadata["context_overflow_bounded_recovery"].as_bool() == Some(true)
            })
            .expect("overflow recovery audit");
        assert_eq!(overflow.metadata["retrying"], true);
        assert!(records.iter().any(|record| {
            record.metadata["force_context_overflow_final"].as_bool() == Some(true)
        }));
    }

    #[test]
    fn bounded_context_overflow_evidence_is_valid_and_keeps_every_call_identity() {
        let calls = (0..24)
            .map(|index| ToolCallMade {
                name: "data_fetch".to_string(),
                arguments: json!({"data_type":"earnings_calendar","ticker":format!("T{index}")}),
                result: json!({
                    "data": (0..100)
                        .map(|row| json!({
                            "symbol": format!("T{index}_{row}"),
                            "date": "2026-08-01",
                            "description": "OVERSIZED_SENTINEL".repeat(100),
                        }))
                        .collect::<Vec<_>>()
                }),
                tool_call_id: Some(format!("call_{index}")),
            })
            .collect::<Vec<_>>();

        let evidence = bounded_context_overflow_tool_evidence(&calls);

        assert!(evidence.chars().count() <= CONTEXT_OVERFLOW_RECOVERY_TOTAL_TOOL_CHARS);
        let parsed = serde_json::from_str::<Value>(&evidence).expect("valid bounded JSON");
        let records = parsed.as_array().expect("records");
        assert_eq!(records.len(), calls.len());
        for index in 0..calls.len() {
            let expected = format!("call_{index}");
            assert!(
                records
                    .iter()
                    .any(|record| { record["tool_call_id"].as_str() == Some(expected.as_str()) })
            );
        }
        assert!(
            records
                .iter()
                .any(|record| record["result_compacted"].as_bool() == Some(true))
        );
    }

    #[test]
    fn bounded_context_overflow_large_calendar_is_fast_and_bounded() {
        let calendar = json!({
            "data": (0..12_000)
                .map(|row| json!({
                    "symbol": format!("AI{row}"),
                    "date": "2026-08-01",
                    "description": "large earnings calendar evidence row ".repeat(8),
                }))
                .collect::<Vec<_>>()
        });
        let started = std::time::Instant::now();

        let (bounded, compacted) =
            bounded_context_json_value(&calendar, CONTEXT_OVERFLOW_RECOVERY_MAX_RESULT_CHARS);

        assert!(compacted);
        assert_eq!(bounded["status"], "tool_result_compacted");
        assert!(bounded.to_string().chars().count() <= CONTEXT_OVERFLOW_RECOVERY_MAX_RESULT_CHARS);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "large-result recovery compaction must stay linear and non-blocking; elapsed={:?}",
            started.elapsed()
        );
    }

    #[tokio::test]
    async fn data_fetch_starts_same_agent_research_before_finish_is_available() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ContentDelta("首轮隐藏工具草稿".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![ChatStreamEvent::ContentDelta("行情分析终稿".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("data-fetch-starts-agent-research".to_string());

        let response = agent.run("CRWV 最新行情", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "行情分析终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].name, "data_fetch");
        assert_eq!(response.tool_calls_made[2].name, "data_fetch");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 0],
            "the same Agent must complete a post-identity business round before finish becomes available"
        );
        // Keep this fixture on the retired finish-only protocol: its contract
        // is finish availability after same-Agent evidence rounds, while draft
        // ownership/scrubbing belongs to the separate agent-owned finance path.
    }

    #[tokio::test]
    async fn terminal_scrubs_tool_round_drafts_that_precede_data_fetch_activation() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ContentDelta(
                    "未经证据支持的早期关系草稿：CRWV 是 NVIDIA 子公司。".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_web_search".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta(
                    "未经采用的行情草稿：CRWV 市值已经核验。".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_post_identity_web".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_post_identity_snapshot".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: relationship_finish_arguments("tc_post_identity_web"),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "基于两项工具证据的终稿".to_string(),
            )],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(WebSearchEvidenceTool));
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("pre-data-fetch-draft-scrub".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "基于两项工具证据的终稿");
        assert_eq!(response.tool_calls_made.len(), 4);
        assert_explicit_terminal_messages(&seen_messages);
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let terminal_messages = seen_messages.last().expect("terminal messages");
        assert!(terminal_messages.iter().any(|message| {
            message
                .content
                .as_deref()
                .is_some_and(|content| content.contains("relationship"))
        }));
        let terminal_transcript =
            serde_json::to_string(terminal_messages).expect("serialize terminal transcript");
        assert!(!terminal_transcript.contains(r#"\"query\":\"CRWV\""#));
        assert!(terminal_transcript.contains("fallback_evidence"));
        assert!(terminal_transcript.contains("Capacity purchase announcement"));
        assert!(terminal_messages.iter().all(|message| {
            message.content.as_deref().is_none_or(|content| {
                !content.contains("CRWV 是 NVIDIA 子公司") && !content.contains("CRWV 市值已经核验")
            })
        }));
    }

    #[tokio::test]
    async fn non_finance_web_search_does_not_activate_the_investment_terminal_protocol() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_search".to_string()),
                name: Some("web_search".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "这是普通网页检索后的直接回答。".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("non-finance-web-search".to_string());

        let response = agent.run("查一下普通网页资料", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "这是普通网页检索后的直接回答。");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1],
            "non-finance tools must keep the ordinary Agent loop without an internal finish signal or terminal completion"
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names lock")
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
    }

    #[tokio::test]
    async fn iteration_limit_fails_without_terminal_call_when_trace_is_not_known_read_only() {
        let llm =
            StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_echo".to_string()),
                name: Some("echo_tool".to_string()),
                arguments: r#"{"text":"abc"}"#.to_string(),
            }]]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            1,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("iteration-limit-failure".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 1);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.error.as_deref(), Some("max_iterations_exceeded:1"));
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [ToolChoiceMode::Auto]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(
            audit
                .operations
                .lock()
                .expect("audit operations lock")
                .iter()
                .all(|operation| operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn iteration_limit_with_known_read_only_trace_gets_one_tools_disabled_final_answer() {
        let answer = "数据时间：北京时间 2026-08-26 09:31；行情口径：已取得当前轮只读证据\n\n基于现有 quote 和检索结果，先给出可执行结论。";
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            answer,
            vec![ChatResponse {
                content: String::new(),
                reasoning_content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "tc_data_fetch".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
                usage: None,
            }],
        );
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm.clone()),
            Arc::new(registry),
            String::new(),
            1,
            None,
        )
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("iteration-limit-read-only-recovery".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.chat_with_tools_calls, 1);
        assert_eq!(state.chat_calls, 1);
        assert_eq!(state.seen_tool_messages.len(), 1);
        drop(state);
        assert!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref())
                == Some(answer)
        );
    }

    #[tokio::test]
    async fn direct_answer_fallback_does_not_start_a_second_terminal_generation() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ContentDelta(
            "直接答案".to_string(),
        )]]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finish-research-direct".to_string());

        let response = agent.run("answer directly", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "直接答案");
        assert_eq!(response.iterations, 1);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1],
            "a direct answer must not see finish_research or be followed by an empty-tools rewrite"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [ToolChoiceMode::Auto],
            "a turn that has not entered the finance tool chain must preserve ordinary direct answers"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:直接答案"]
        );
    }

    #[tokio::test]
    async fn direct_stream_requires_stop_and_done() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![
            ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Auto,
                effective: ToolChoiceMode::Auto,
                fallback: false,
            },
            ChatStreamEvent::ContentDelta("partial direct answer".to_string()),
            ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
        ]]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("direct-missing-done".to_string());

        let response = agent.run("answer", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:partial direct answer", "reset"]
        );
    }

    #[tokio::test]
    async fn tool_stream_requires_tool_calls_finish_reason() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![
            ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Auto,
                effective: ToolChoiceMode::Auto,
                fallback: false,
            },
            ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_wrong_finish".to_string()),
                name: Some("echo_tool".to_string()),
                arguments: r#"{"text":"never execute"}"#.to_string(),
            },
            ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ChatStreamEvent::Done,
        ]]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None);
        let mut context = AgentContext::new("tool-wrong-finish".to_string());

        let response = agent.run("tool", &mut context).await;

        assert!(!response.success);
        assert!(response.tool_calls_made.is_empty());
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("expected ToolCalls, got Stop"))
        );
    }

    #[tokio::test]
    async fn active_finish_stream_missing_done_fails_immediately_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_incomplete_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
                ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-finish-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records.last().expect("active finish error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_stream_missing_done_fails_immediately_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_incomplete_data".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"text":"incomplete"}"#.to_string(),
                },
                ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records.last().expect("active business error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn terminal_stream_requires_stop_and_done() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta("incomplete terminal".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
        ]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("terminal synthesis stream ended before Done"))
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:incomplete terminal", "reset"]
        );
    }

    #[tokio::test]
    async fn empty_completed_terminal_recovers_once_without_rerunning_business_tools() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            Vec::new(),
            vec![ChatStreamEvent::ContentDelta(
                "同一证据生成的唯一非空终稿".to_string(),
            )],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("empty-terminal-recovery".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "同一证据生成的唯一非空终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 5);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 0, 0],
            "empty terminal recovery must reuse the same evidence with tools disabled"
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "an empty first terminal and buffered recovery must not flash or reset"
        );
        let messages = seen_messages.lock().expect("stream messages lock");
        assert!(
            messages[4]
                .last()
                .and_then(|message| message.content.as_deref())
                .is_some_and(|prompt| prompt.contains("正常结束但没有可见正文"))
        );
        drop(messages);
        let records = audit.records.lock().expect("audit records lock");
        let initial = records
            .iter()
            .find(|record| record.operation == "chat_terminal_without_tools")
            .expect("initial terminal audit");
        assert!(!initial.success);
        assert_eq!(initial.metadata["terminal_recovery_eligible"], true);
        let recovery = records
            .iter()
            .find(|record| record.operation == "chat_terminal_recovery_without_tools")
            .expect("terminal recovery audit");
        assert!(recovery.success, "{:?}", recovery.error);
        assert_eq!(recovery.metadata["committed_prefix_bytes"], 0);
    }

    #[tokio::test]
    async fn committed_terminal_prefix_recovers_once_without_restreaming_or_rerunning_tools() {
        let prefix = concat!(
            "数据时间：北京时间 2026-07-18 21:05；行情口径：",
            "报价源最新可得、非逐笔\n"
        );
        let incomplete = format!("{prefix}未完成的正文");
        let recovered = format!("{prefix}\n## 结论\n基于本轮工具证据完成回答。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta(incomplete.clone()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
            vec![ChatStreamEvent::ContentDelta(format!(
                "<think>recovery reasoning is not visible evidence</think>{recovered}"
            ))],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(CommittedPrefixStreamObserver::new(prefix));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-recovery-success".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, recovered);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 5);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 0, 0],
            "recovery must stay in the same terminal phase with tools disabled"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            [format!("final:{incomplete}")],
            "the recovery response must remain buffered and must not reset the committed prefix"
        );
        let messages = seen_messages.lock().expect("stream messages lock");
        assert_eq!(messages.len(), 5);
        assert!(
            messages[4]
                .last()
                .and_then(|message| message.content.as_deref())
                .is_some_and(|prompt| {
                    prompt.contains("【终稿传输恢复】")
                        && prompt.contains("前缀后必须继续输出非空正文")
                })
        );
        assert!(
            messages[4]
                .iter()
                .all(|message| message.reasoning_content.is_none())
        );
        drop(messages);

        let records = audit.records.lock().expect("audit records lock");
        let initial = records
            .iter()
            .find(|record| record.operation == "chat_terminal_without_tools")
            .expect("initial terminal audit");
        assert!(!initial.success);
        assert_eq!(
            initial.metadata["terminal_recovery_eligible"],
            Value::Bool(true)
        );
        let recovery = records
            .iter()
            .find(|record| record.operation == "chat_terminal_recovery_without_tools")
            .expect("terminal recovery audit");
        assert!(recovery.success, "{:?}", recovery.error);
        assert_eq!(recovery.metadata["recovery_attempt"], 1);
        assert_eq!(recovery.metadata["has_tools"], Value::Bool(false));
        assert_eq!(
            recovery.metadata["effective_tool_choice"],
            Value::String("auto".to_string())
        );
        assert_eq!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref()),
            Some(response.content.as_str())
        );
        assert!(
            context
                .messages
                .last()
                .expect("terminal message")
                .metadata
                .is_none()
        );
    }

    #[tokio::test]
    async fn committed_terminal_prefix_recovery_mismatch_fails_after_exactly_one_attempt() {
        let prefix = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
        let incomplete = format!("{prefix}未完成的正文");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta(incomplete.clone()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
            vec![ChatStreamEvent::ContentDelta(
                "数据时间：北京时间 2026-07-18 21:06；行情口径：不同前缀\n正文".to_string(),
            )],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(CommittedPrefixStreamObserver::new(prefix));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-recovery-mismatch".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains(
                    "terminal recovery content does not start with the committed visible prefix"
                )),
            "{:?}",
            response.error
        );
        assert_eq!(stream_calls.load(Ordering::SeqCst), 5);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            [format!("final:{incomplete}")]
        );
        let records = audit.records.lock().expect("audit records lock");
        let recovery_records = records
            .iter()
            .filter(|record| record.operation == "chat_terminal_recovery_without_tools")
            .collect::<Vec<_>>();
        assert_eq!(recovery_records.len(), 1);
        assert!(!recovery_records[0].success);
    }

    #[test]
    fn terminal_content_rejects_header_only_and_duplicate_committed_prefix() {
        let prefix = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
        let header_only = validate_terminal_recovery_content(prefix, prefix)
            .expect_err("a canonical header without a body is incomplete");
        assert!(header_only.to_string().contains("contains no body"));

        let duplicated = format!("{prefix}\n{prefix}正文");
        let duplicate_error = validate_terminal_recovery_content(&duplicated, prefix)
            .expect_err("replaying the committed header would duplicate visible output");
        assert!(
            duplicate_error
                .to_string()
                .contains("repeats the committed visible prefix")
        );
    }

    #[test]
    fn non_success_stream_finish_reasons_are_errors() {
        for reason in [
            ChatStreamFinishReason::Length,
            ChatStreamFinishReason::ContentFilter,
            ChatStreamFinishReason::Error,
            ChatStreamFinishReason::Other("provider_specific".to_string()),
        ] {
            let mut finish = None;
            assert!(
                observe_stream_finish(&mut finish, reason).is_err(),
                "non-success finish reason must fail"
            );
            assert!(finish.is_none());
        }
    }

    #[tokio::test]
    async fn unavailable_finance_evidence_can_finish_with_a_disclosed_gap() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_failed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV","entity_route":"coreweave","identity_match":"exact_symbol"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_refine_failed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CoreWeave","entity_route":"coreweave","identity_match":"name_or_alias","refines_query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_quote_failed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV","entity_route":"coreweave"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish_after_gap".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "本轮财务源不可用；以下仅分析已核验部分。".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FailingFinanceEvidenceTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            5,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer));
        let mut context = AgentContext::new("finish-research-after-gap".to_string());

        let response = agent
            .run("research with unavailable evidence", &mut context)
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "本轮财务源不可用；以下仅分析已核验部分。");
        assert_eq!(response.tool_calls_made.len(), 3);
        assert!(response.tool_calls_made.iter().all(|call| {
            call.name == "data_fetch"
                && call.result["status"] == "failed"
                && call.result["isError"] == true
                && call.result["timeout"] == false
        }));
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 1, 2, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
    }

    #[tokio::test]
    async fn duplicate_finish_calls_use_the_first_parseable_handoff_without_a_model_retry() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"snapshot","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_finish_1".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_finish_2".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta("唯一终稿".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("duplicate-finish".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "唯一终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
                ToolChoiceMode::Auto,
            ]
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
        assert_explicit_terminal_messages(&seen_messages);
    }

    #[tokio::test]
    async fn hallucinated_unknown_finish_uses_standard_tool_error_without_hidden_correction() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_unavailable_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: test_finish_arguments(),
            }],
            vec![ChatStreamEvent::ContentDelta("自然非空回答".to_string())],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None);
        let mut context = AgentContext::new("unavailable-sole-finish".to_string());

        let response = agent.run("普通问题", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "自然非空回答");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, FINISH_RESEARCH_TOOL_NAME);
        assert_eq!(response.tool_calls_made[0].result["isError"], true);
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let followup = serde_json::to_string(&seen_messages[1]).expect("followup transcript");
        assert!(followup.contains("tc_unavailable_finish"));
        assert!(!followup.contains("内部工具协议纠正"));
        assert!(!followup.contains("finish_research 当前尚不可用"));
    }

    #[tokio::test]
    async fn unknown_finish_name_follows_ordinary_tool_budget_when_policy_is_disabled() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_echo_mixed".to_string()),
                    name: Some("echo_tool".to_string()),
                    arguments: r#"{"text":"mixed"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_finish_mixed".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: test_finish_arguments(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta("完成".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_tool_call_budget(Some(1), HashMap::new())
                .with_stream_observer(Some(stream_observer.clone()))
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finish-research-mixed".to_string());

        let response = agent.run("mixed", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "完成");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "echo_tool");
        assert_eq!(response.tool_calls_made[0].result["echo"], "mixed");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1],
            "the retired name is absent from the schema and follows the ordinary tool budget if hallucinated"
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            ["start:echo_tool", "done:echo_tool:true"],
            "the budgeted real tool executes normally"
        );
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:完成"]
        );
        assert!(context.messages.iter().any(|message| {
            message.tool_calls.as_ref().is_some_and(|tool_calls| {
                tool_calls.iter().any(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        == Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
    }

    #[tokio::test]
    async fn run_handles_invalid_tool_arguments_and_continues() {
        let invalid_tool_call = hone_llm::ToolCall {
            id: "tc_bad".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: "{not json}".to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "try tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![invalid_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "fallback final".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None);
        let mut context = AgentContext::new("s3".to_string());

        let response = agent.run("bad args", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "fallback final");
        assert!(response.tool_calls_made.is_empty());
        let tool_msgs: Vec<_> = context
            .messages
            .iter()
            .filter(|m| m.role == "tool")
            .collect();
        assert_eq!(tool_msgs.len(), 1);
        let tool_msg_content = tool_msgs[0].content.clone().unwrap_or_default();
        assert!(tool_msg_content.contains("参数解析失败"));
    }

    #[tokio::test]
    async fn run_rejects_tool_calls_after_per_tool_budget() {
        let first_tool_call = hone_llm::ToolCall {
            id: "tc_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "counting_tool".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let second_tool_call = hone_llm::ToolCall {
            id: "tc_2".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "counting_tool".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "call once".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![first_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "call twice".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![second_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_tool_call_budget(None, HashMap::from([("counting_tool".to_string(), 1)]));
        let mut context = AgentContext::new("budget".to_string());

        let response = agent.run("budget", &mut context).await;

        assert!(response.success);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(response.tool_calls_made.len(), 1);
        let tool_messages = context
            .messages
            .iter()
            .filter(|message| message.role == "tool")
            .collect::<Vec<_>>();
        assert_eq!(tool_messages.len(), 2);
        assert!(
            tool_messages[1]
                .content
                .as_deref()
                .unwrap_or_default()
                .contains("call limit reached")
        );
    }

    #[tokio::test]
    async fn run_notifies_tool_observer_on_execution() {
        let tool_call = hone_llm::ToolCall {
            id: "call_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"echo":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "let me call tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(MockToolObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            3,
            None,
        )
        .with_tool_observer(Some(observer.clone()));

        let actor = hone_core::ActorIdentity::new("web", "u1", None::<String>).expect("actor");
        let mut context = AgentContext::new("s1".to_string());
        context.set_actor_identity(&actor);
        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        let events = observer.events.lock().expect("observer lock").clone();
        assert_eq!(events, vec!["start:echo_tool", "done:echo_tool:true"]);
    }

    #[tokio::test]
    async fn run_replays_reasoning_content_into_followup_tool_round() {
        let tool_call = hone_llm::ToolCall {
            id: "tc_reason".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"text":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: String::new(),
                reasoning_content: Some("need tool lookup first".to_string()),
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm.clone()),
            Arc::new(registry),
            String::new(),
            4,
            None,
        );
        let mut context = AgentContext::new("s_reason".to_string());

        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.seen_tool_messages.len(), 2);
        let assistant = state.seen_tool_messages[1]
            .iter()
            .find(|message| message.role == "assistant")
            .expect("assistant followup message");
        assert_eq!(
            assistant.reasoning_content.as_deref(),
            Some("need tool lookup first")
        );
    }
}
