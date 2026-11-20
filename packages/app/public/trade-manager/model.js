/** 巴芒交易管理 · 本地记账与情景演算。所有价格和假设均由使用者录入。 */
const EPS = 1e-8;
const BUCKETS = ['core', 'flex', 'defense'];
const TRENDS = ['stronger', 'stable', 'weaker', 'unknown'];
const clone = value => JSON.parse(JSON.stringify(value));
const finite = value => typeof value === 'number' && Number.isFinite(value);
const ratio = (part, whole) => whole > 0 ? part / whole * 100 : 0;
const day = () => new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date());
const daysAgo = (n) => { const d = new Date(`${day()}T00:00:00Z`); d.setUTCDate(d.getUTCDate() - n); return d.toISOString().slice(0, 10); };
const validId = value => typeof value === 'string' && /^[A-Za-z0-9_-]{1,100}$/.test(value);
const near = (a, b) => finite(a) && finite(b) && Math.abs(a - b) <= 1e-8 + 1e-10 * Math.max(Math.abs(a), Math.abs(b));
const isDate = value => typeof value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(value) && !Number.isNaN(Date.parse(value)) && new Date(`${value}T00:00:00Z`).toISOString().slice(0, 10) === value;

export function createState() {
  return {
    schemaVersion: 1,
    settings: { currency: 'USD', cash: 0, lifeCash: 0, budget: 30, singleLimit: 20, themeLimit: 60, targetGrowth: 60, targetDefense: 30, targetCash: 10, buffer: 5, maxMonthly: 5, growthShock: 50, defenseShock: 15, cashShock: 0, market: 'neutral', marketNote: '' },
    holdings: [], events: [], plans: [], journal: [], checks: {}, cashFlows: [], corporateActions: [],
  };
}

export function createDemoState() {
  const s = createState();
  s.settings.cash = 120000;
  s.settings.lifeCash = 60000;
  s.settings.marketNote = '演示观察：关注利率、流动性与 AI 资本开支。没有接入实时市场信号。';
  const makeHolding = (id, symbol, name, sector, theme, bucket, quantity, cost, price, extra = {}) => ({
    id, symbol, name, sector, theme, bucket, quantity, cost, price, priceDate: day(), scarcity: 'stable', differentiation: 'stable', thesis: '演示假设：研究结构性供需、相对竞争优势与长期现金流。', invalidation: '演示条件：相对毛利优势持续收窄；客户份额下降；正常化盈利下修。', evidence: '合成示例，尚未录入官方研究来源。', reviewDate: daysAgo(10), nextReview: daysAgo(-20), valuation: 'fair', low: price * 0.8, base: price * 1.15, high: price * 1.4, ...extra,
  });
  s.holdings = [
    makeHolding('h-nvda', 'NVDA', '英伟达（演示）', '芯片', 'AI', 'core', 1500, 110, 140, { scarcity: 'stronger', differentiation: 'stronger' }),
    makeHolding('h-mu', 'MU', '美光（演示）', '存储', 'AI', 'core', 1800, 90, 110, { nextReview: daysAgo(1), priceDate: daysAgo(5) }),
    makeHolding('h-googl', 'GOOGL', 'Alphabet（演示）', '软件 / 云', 'AI', 'core', 900, 130, 150),
    makeHolding('h-be', 'BE', 'Bloom Energy（演示）', '电力', 'AI', 'flex', 1000, 25, 35, { valuation: 'unknown' }),
    makeHolding('h-def', 'DEF-DEMO', '防守资产组合（演示）', '防守资产', '防守', 'defense', 2000, 100, 100, { thesis: '演示防守篮子；假设压力损失 15%，不是无风险现金。', invalidation: '流动性、久期或信用风险发生变化。' }),
  ];
  s.events = [{ id: 'e-1', date: day(), symbol: 'MU', type: '竞争与供给', title: '演示：竞争供给扩张消息需要核实', source: '', verification: 'pending', impact: 'negative', note: '先核实原始披露、产能规模及投产时间，再判断对正常化盈利的影响。', reviewed: false }];
  s.plans = [
    { id: 'p-1', date: day(), holdingId: 'h-mu', symbol: 'MU', side: 'buy', quantity: 50, price: 110, fee: 2, reason: 'opportunity', rationale: '演示草稿：仅在证据核实且组合风险允许时评估。', status: 'draft', checklist: { thesis: false, value: false, risk: false, evidence: false } },
    { id: 'p-2', date: day(), holdingId: 'h-nvda', symbol: 'NVDA', side: 'sell', quantity: 50, price: 140, fee: 2, reason: 'risk', rationale: '演示草稿：讨论集中度再平衡，不代表看空企业。', status: 'draft', checklist: { thesis: false, value: false, risk: false, evidence: false } },
  ];
  return s;
}

export function portfolio(state) {
  const settings = state.settings;
  const cash = settings.cash;
  const invested = state.holdings.reduce((sum, h) => sum + h.quantity * h.price, 0);
  const total = invested + cash;
  const holdings = state.holdings.map(h => {
    const value = h.quantity * h.price;
    const age = isDate(h.priceDate) ? Math.floor((Date.parse(`${day()}T00:00:00Z`) - Date.parse(`${h.priceDate}T00:00:00Z`)) / 86400000) : null;
    return { ...h, value, weight: ratio(value, total), pnl: h.quantity * (h.price - h.cost), pnlPct: h.cost > 0 ? (h.price / h.cost - 1) * 100 : null, priceAgeDays: age, priceStale: h.quantity > 0 && (age === null || age > 3 || age < 0), reviewDue: h.quantity > 0 && Boolean(h.nextReview) && h.nextReview <= day() };
  });
  const growth = holdings.filter(h => h.bucket !== 'defense').reduce((sum, h) => sum + h.value, 0);
  const defense = invested - growth;
  const growthPct = ratio(growth, total), defensePct = ratio(defense, total), cashPct = ratio(cash, total);
  const stressLoss = growth * settings.growthShock / 100 + defense * settings.defenseShock / 100 + cash * settings.cashShock / 100;
  const stressPct = ratio(stressLoss, total);
  const themeMap = new Map();
  for (const h of holdings) if (h.value > 0) themeMap.set(h.theme || '未分类', (themeMap.get(h.theme || '未分类') || 0) + h.value);
  const themeWeights = [...themeMap.entries()].map(([theme, value]) => ({ theme, value, weight: ratio(value, total) })).sort((a, b) => b.weight - a.weight);
  const drift = { growth: growthPct - settings.targetGrowth, defense: defensePct - settings.targetDefense, cash: cashPct - settings.targetCash };
  const tacticalTrades = state.plans.filter(p => p.status === 'executed' && p.reason === 'risk' && isDate(p.execution?.date) && p.execution.date.slice(0, 7) === day().slice(0, 7));
  const tacticalKnown = tacticalTrades.filter(p => finite(p.execution.growthPctBefore) && finite(p.execution.growthPctAfter));
  const monthlyTacticalChangePct = tacticalKnown.reduce((sum, p) => sum + Math.abs(p.execution.growthPctAfter - p.execution.growthPctBefore), 0);
  const monthlyTacticalCount = tacticalTrades.length;
  const monthlyTacticalKnownCount = tacticalKnown.length;
  const monthlyTacticalUnknownCount = monthlyTacticalCount - monthlyTacticalKnownCount;
  const alerts = [];
  if (!total) alerts.push({ level: 'info', title: '先建立账户基线', detail: '录入投资账户现金与持仓；生活保障金单独记录，不进入投资资产分母。' });
  if (stressPct > settings.budget + EPS) alerts.push({ level: 'danger', title: '压力损失超过预算', detail: `当前情景损失 ${stressPct.toFixed(1)}%，预算 ${settings.budget.toFixed(1)}%。复核假设与风险敞口。` });
  const targetStressPct = (settings.targetGrowth * settings.growthShock + settings.targetDefense * settings.defenseShock + settings.targetCash * settings.cashShock) / 100;
  if (targetStressPct > settings.budget + EPS) alerts.push({ level: 'warning', title: '目标配置在当前情景下也超预算', detail: `按自设目标权重演算的损失为 ${targetStressPct.toFixed(1)}%，高于预算 ${settings.budget.toFixed(1)}%。请核对目标或情景假设；工具不会自动修改配置。` });
  for (const h of holdings) {
    if (h.weight > settings.singleLimit + EPS) alerts.push({ level: 'warning', title: `${h.symbol} 单票占比超限`, detail: `占投资资产 ${h.weight.toFixed(1)}%，自设上限 ${settings.singleLimit.toFixed(1)}%。` });
    if (h.priceStale) alerts.push({ level: 'warning', title: `${h.symbol} 价格待更新`, detail: h.priceAgeDays === null ? '尚未填写价格日期。' : h.priceAgeDays < 0 ? '价格日期在未来，请核对。' : `价格日期为 ${h.priceDate}，已超过 3 个自然日；风险计算依赖该录入价格。` });
    if (h.reviewDue) alerts.push({ level: 'info', title: `${h.symbol} 到期复核`, detail: '检查稀缺性、差异化、证伪条件与新证据，再决定是否更新估值。' });
    if (h.quantity > 0 && (h.scarcity === 'weaker' || h.differentiation === 'weaker')) alerts.push({ level: 'warning', title: `${h.symbol} 投资逻辑待复核`, detail: '已记录竞争优势转弱。区分行业周期与企业差异化，并检查证据。' });
  }
  for (const t of themeWeights) if (t.weight > settings.themeLimit + EPS) alerts.push({ level: 'warning', title: `${t.theme} 主题集中度超限`, detail: `相关主题合计 ${t.weight.toFixed(1)}%，自设上限 ${settings.themeLimit.toFixed(1)}%。不同股票不等于风险独立。` });
  const followUps = state.events.filter(e => !e.reviewed && isDate(e.followUp) && e.followUp <= day());
  if (followUps.length) alerts.push({ level: 'info', title: `${followUps.length} 条事件已到跟进日期`, detail: '检查待核实证据与后续影响，更新跟进日期或完成复核。' });
  const pending = state.events.filter(e => !e.reviewed && e.verification === 'pending');
  if (pending.length) alerts.push({ level: 'info', title: `${pending.length} 条消息待核实`, detail: '未核实消息进入研究队列，不自动改变持仓或执行交易。' });
  const adverse = state.events.filter(e => !e.reviewed && e.verification === 'verified' && e.impact === 'negative');
  if (adverse.length) alerts.push({ level: 'warning', title: `${adverse.length} 条负面证据待复核`, detail: '检查是否触及原先写下的证伪条件，并单独记录判断。' });
  if (total && Object.values(drift).some(v => Math.abs(v) > settings.buffer + EPS)) alerts.push({ level: 'info', title: '配置偏离缓冲带', detail: `至少一类资产偏离手动目标超过 ${settings.buffer} 个百分点，可评估再平衡。` });
  if (monthlyTacticalChangePct > settings.maxMonthly + EPS) alerts.push({ level: 'warning', title: '本月战术调整超过试用阈值', detail: `已知风险再平衡成交的成长仓占比变化绝对值累计为 ${monthlyTacticalChangePct.toFixed(1)} 个百分点，自设阈值 ${settings.maxMonthly} 个百分点；累计绝对变化并非月末净变化。` });
  if (monthlyTacticalUnknownCount) alerts.push({ level: 'info', title: '本月战术调整统计不完整', detail: `${monthlyTacticalUnknownCount} 笔旧成交缺少交易前后成长仓占比快照，未计入累计变化；没有按当前持仓补算。` });
  return { invested, total, cash, growth, defense, growthPct, defensePct, cashPct, stressLoss, stressPct, holdings, themeWeights, alerts, drift, targetStressPct, monthlyTacticalChangePct, monthlyTacticalCount, monthlyTacticalKnownCount, monthlyTacticalUnknownCount };
}

/** 两资产、无杠杆情景；百分比输入输出。null 表示该假设下无可行成长权重。 */
export function allocationCeiling(budget, growthShock, defenseShock) {
  if (![budget, growthShock, defenseShock].every(n => finite(n) && n >= 0 && n <= 100)) return null;
  if (growthShock <= budget) return 100;
  if (defenseShock > budget || growthShock <= defenseShock) return null;
  return Math.max(0, Math.min(100, (budget - defenseShock) / (growthShock - defenseShock) * 100));
}

function tradeErrors(state, plan) {
  const errors = [];
  const h = state.holdings.find(item => item.id === plan.holdingId);
  if (!h) errors.push('请先在持仓与研究中建立该标的。');
  if (h?.archived) errors.push('请先恢复归档标的，再建立或执行交易计划。');
  if (!['buy', 'sell'].includes(plan.side)) errors.push('买卖方向无效。');
  if (!finite(plan.quantity) || plan.quantity <= 0) errors.push('成交数量必须大于零。');
  if (!finite(plan.price) || plan.price <= 0) errors.push('成交价格必须大于零。');
  if (!finite(plan.fee) || plan.fee < 0) errors.push('费用必须是非负数。');
  if (h && plan.symbol !== h.symbol) errors.push('交易标的与持仓不一致。');
  const amount = plan.quantity * plan.price;
  if (!Number.isFinite(amount)) errors.push('成交金额超出可计算范围。');
  if (!errors.length && plan.side === 'buy' && amount + plan.fee > state.settings.cash + EPS) errors.push('账户现金不足，不能使用生活保障金或借款。');
  if (!errors.length && plan.side === 'sell' && plan.quantity > h.quantity + EPS) errors.push('卖出数量超过当前持股，首版不支持做空。');
  if (!errors.length && plan.side === 'sell' && state.settings.cash + amount - plan.fee < -EPS) errors.push('卖出后现金不足以支付费用。');
  return errors;
}

function applyTrade(state, trade) {
  const result = clone(state);
  const h = result.holdings.find(item => item.id === trade.holdingId);
  const amount = trade.quantity * trade.price;
  const costBasis = h.cost * trade.quantity;
  if (trade.side === 'buy') {
    h.cost = (h.quantity * h.cost + amount + trade.fee) / (h.quantity + trade.quantity);
    h.quantity += trade.quantity;
    result.settings.cash -= amount + trade.fee;
  } else {
    h.quantity -= trade.quantity;
    if (Math.abs(h.quantity) < EPS) h.quantity = 0;
    result.settings.cash += amount - trade.fee;
  }
  if (Math.abs(result.settings.cash) < EPS) result.settings.cash = 0;
  return { result, costBasis, amount };
}

export function simulatePlan(state, plan) {
  const errors = tradeErrors(state, plan);
  const before = portfolio(state);
  if (errors.length) return { valid: false, errors, before, after: null, impactPct: 0 };
  const { result, amount } = applyTrade(state, plan);
  return { valid: true, errors: [], before, after: portfolio(result), impactPct: ratio(amount, before.total) };
}

const REVIEW_KEYS = ['thesis', 'value', 'risk', 'evidence'];
const PLAN_FIELDS = ['date', 'holdingId', 'symbol', 'side', 'quantity', 'price', 'fee', 'reason', 'rationale'];
const blankChecklist = () => Object.fromEntries(REVIEW_KEYS.map(key => [key, false]));
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;

// Compact change detection only: this is not a cryptographic authorization token.
function reviewFingerprint(state, plan) {
  const ordered = items => items.slice().sort((a, b) => a.id.localeCompare(b.id));
  const payload = JSON.stringify(canonical({
    plan: Object.fromEntries(['id', ...PLAN_FIELDS].map(k => [k, plan[k]])),
    settings: state.settings,
    holdings: ordered(state.holdings),
    events: ordered(state.events.filter(e => !e.reviewed)),
    executions: ordered(state.plans.filter(p => p.status === 'executed')).map(p => ({ id: p.id, reason: p.reason, execution: p.execution })),
  }));
  let a = 2166136261, b = 5381;
  for (let i = 0; i < payload.length; i++) { const code = payload.charCodeAt(i); a = Math.imul(a ^ code, 16777619); b = Math.imul(b, 33) ^ code; }
  return `review-v1-${payload.length}-${(a >>> 0).toString(16)}-${(b >>> 0).toString(16)}`;
}

export function planReviewFresh(state, plan) {
  return Boolean(plan && plan.status === 'reviewed' && REVIEW_KEYS.every(k => plan.checklist?.[k] === true) && typeof plan.reviewFingerprint === 'string' && plan.reviewFingerprint === reviewFingerprint(state, plan));
}

/** Draft edits invalidate all previous checks, including on previously reviewed plans. */
export function updatePlan(state, id, patch) {
  const original = state.plans.find(p => p.id === id);
  if (!original || !['draft', 'reviewed'].includes(original.status)) throw new Error('只能修改待审查或已审查且尚未成交的计划。');
  if (!patch || typeof patch !== 'object' || Array.isArray(patch) || Object.keys(patch).some(k => !PLAN_FIELDS.includes(k))) throw new Error('计划修改字段无效。');
  const result = clone(state), plan = result.plans.find(p => p.id === id);
  for (const k of PLAN_FIELDS) if (Object.hasOwn(patch, k)) plan[k] = patch[k];
  const holding = result.holdings.find(h => h.id === plan.holdingId);
  if (!holding) throw new Error('交易标的不存在。');
  plan.symbol = holding.symbol;
  const errors = tradeErrors(result, plan);
  if (errors.length) throw new Error(errors.join(' '));
  plan.status = 'draft'; plan.checklist = blankChecklist();
  delete plan.reviewFingerprint; delete plan.reviewedAt;
  return assertResult(result);
}

export function reviewPlanState(state, id, checklist) {
  const original = state.plans.find(p => p.id === id);
  if (!original || !['draft', 'reviewed'].includes(original.status)) throw new Error('该计划当前不能审查。');
  if (!REVIEW_KEYS.every(k => checklist?.[k] === true)) throw new Error('请完成全部四项交易前检查。');
  const holding = state.holdings.find(h => h.id === original.holdingId);
  if (!holding || ['thesis', 'invalidation', 'evidence'].some(k => typeof holding[k] !== 'string' || !holding[k].trim())) throw new Error('请先补充标的投资逻辑、证伪条件及关键证据。');
  if (typeof original.rationale !== 'string' || !original.rationale.trim()) throw new Error('请写明本次操作依据。');
  const errors = tradeErrors(state, original);
  if (errors.length) throw new Error(errors.join(' '));
  const result = clone(state), plan = result.plans.find(p => p.id === id);
  plan.status = 'reviewed'; plan.checklist = { ...checklist }; plan.reviewedAt = new Date().toISOString();
  plan.reviewFingerprint = reviewFingerprint(result, plan);
  return assertResult(result);
}

/** Partial fills remain immutable; a remainder creates a separately reviewable draft once. */
export function makeRemainingPlan(state, id, newId) {
  const original = state.plans.find(p => p.id === id);
  if (!original || original.status !== 'executed' || !original.execution) throw new Error('只有已记账的部分成交可以建立剩余计划。');
  if ((state.corporateActions || []).some(a => a.holdingId === original.holdingId && a.type === 'split' && a.date >= original.execution.date && Date.parse(a.recordedAt) >= Date.parse(original.executedAt))) throw new Error('原成交后发生拆并股，剩余数量口径已变化，请按券商调整后的数量手工建立新计划。');
  const remainingQuantity = Math.max(0, original.quantity - original.execution.quantity);
  if (remainingQuantity <= EPS) throw new Error('这笔计划没有剩余未成交数量。');
  if (state.plans.some(p => p.sourcePlanId === id)) throw new Error('此笔剩余数量已建立计划，不能重复生成。');
  assertNewLedgerId(state, newId);
  const result = clone(state);
  const plan = Object.fromEntries(PLAN_FIELDS.map(k => [k, original[k]]));
  Object.assign(plan, { id: newId, date: day(), quantity: remainingQuantity, fee: 0, rationale: `部分成交后的剩余计划。原依据：${original.rationale}`, status: 'draft', checklist: blankChecklist(), sourcePlanId: id });
  result.plans.push(plan);
  return assertResult(result);
}

export function executePlan(state, planId, fill) {
  const plan = state.plans.find(p => p.id === planId);
  if (!plan) throw new Error('找不到交易计划。');
  if (plan.status === 'executed') throw new Error('此计划已记账，不能重复登记。');
  if (plan.status !== 'reviewed') throw new Error('请先完成计划复核。');
  if (!['thesis', 'value', 'risk', 'evidence'].every(key => plan.checklist?.[key] === true)) throw new Error('请完成全部交易前检查。');
  if (!planReviewFresh(state, plan)) throw new Error('原审查已过期或缺少审查快照。请根据当前持仓、现金与证据重新审查，再登记成交。');
  if (!isDate(fill.date) || fill.date > day()) throw new Error('成交日期必须有效且不能在未来。');
  assertLedgerDate(state, fill.date);
  const trade = { ...plan, price: fill.price, quantity: fill.quantity, fee: fill.fee };
  const errors = tradeErrors(state, trade);
  if (errors.length) throw new Error(errors.join(' '));
  const before = portfolio(state);
  const { result, costBasis, amount } = applyTrade(state, trade);
  const after = portfolio(result);
  const recorded = result.plans.find(p => p.id === planId);
  recorded.status = 'executed';
  recorded.executedAt = new Date().toISOString();
  recorded.execution = { remainingQuantity: Math.max(0, plan.quantity - trade.quantity), date: fill.date, price: trade.price, quantity: trade.quantity, fee: trade.fee, amount, cashFlow: trade.side === 'buy' ? -amount - trade.fee : amount - trade.fee, costBasis: trade.side === 'sell' ? costBasis : amount + trade.fee, realizedPnl: trade.side === 'sell' ? amount - trade.fee - costBasis : 0, impactPct: ratio(amount, before.total), cashBefore: state.settings.cash, cashAfter: result.settings.cash, quantityBefore: state.holdings.find(h => h.id === trade.holdingId).quantity, quantityAfter: result.holdings.find(h => h.id === trade.holdingId).quantity, growthPctBefore: before.growthPct, growthPctAfter: after.growthPct };
  return result;
}


function latestLedgerDate(state) {
  return [...state.plans.filter(p => p.status === 'executed').map(p => p.execution?.date), ...(state.cashFlows || []).map(f => f.date), ...(state.corporateActions || []).map(a => a.date)].filter(isDate).reduce((latest, date) => date > latest ? date : latest, '');
}

function assertLedgerDate(state, date) {
  if (!isDate(date) || date > day()) throw new Error('记账日期必须有效且不能在未来。');
  if (date < latestLedgerDate(state)) throw new Error('请按成交日期顺序录入资金与公司行动；需补录历史请先对账恢复正确基线。同日记录请按实际先后顺序登记。');
}

function assertNewLedgerId(state, id) {
  if (!validId(id)) throw new Error('记录 ID 无效。');
  if ([...state.plans, ...(state.cashFlows || []), ...(state.corporateActions || [])].some(item => item.id === id)) throw new Error('该记录 ID 已存在，不能重复记账。');
}

function assertResult(result) {
  const check = validateState(result);
  if (!check.valid) throw new Error(check.errors.join(' '));
  return result;
}

/** 现金流水只改变账户现金；生活保障金始终在账户外。 */
export function recordCashFlow(state, flow) {
  assertNewLedgerId(state, flow.id);
  assertLedgerDate(state, flow.date);
  if (!['deposit', 'withdrawal', 'dividend', 'fee'].includes(flow.type)) throw new Error('资金流水类型无效。');
  if (!finite(flow.amount) || flow.amount <= 0 || flow.amount > Number.MAX_SAFE_INTEGER) throw new Error('流水金额必须为大于零的有限数字。');
  if (typeof flow.note !== 'string' || !flow.note.trim()) throw new Error('请写明资金流水依据。');
  const holding = flow.holdingId ? state.holdings.find(h => h.id === flow.holdingId) : null;
  if (flow.holdingId && !holding) throw new Error('关联标的不存在。');
  const signedAmount = ['withdrawal', 'fee'].includes(flow.type) ? -flow.amount : flow.amount;
  const cashAfter = state.settings.cash + signedAmount;
  if (!finite(cashAfter) || cashAfter < -EPS) throw new Error('账户现金不足，不能使用生活保障金或借款。');
  const result = clone(state);
  result.cashFlows ||= [];
  result.settings.cash = Math.abs(cashAfter) < EPS ? 0 : cashAfter;
  result.cashFlows.push({ id: flow.id, date: flow.date, type: flow.type, amount: flow.amount, holdingId: holding?.id || '', symbol: holding?.symbol || '', note: flow.note.trim(), recordedAt: new Date().toISOString(), cashBefore: state.settings.cash, cashAfter: result.settings.cash });
  return assertResult(result);
}

/** 手工拆并股只重述单位，不创造收益；不处理零碎股现金兑付。 */
export function applySplit(state, action) {
  assertNewLedgerId(state, action.id);
  assertLedgerDate(state, action.date);
  const holding = state.holdings.find(h => h.id === action.holdingId);
  if (!holding || holding.quantity <= 0) throw new Error('拆并股需要当前持有的标的。');
  if (!finite(action.ratio) || action.ratio <= 0 || action.ratio === 1) throw new Error('拆并股比例必须大于零且不能为 1。');
  if (typeof action.note !== 'string' || !action.note.trim()) throw new Error('请填写公司公告或券商对账依据。');
  const result = clone(state), h = result.holdings.find(h => h.id === action.holdingId);
  const snapshot = value => Object.fromEntries(['quantity', 'cost', 'price', 'priceDate', 'low', 'base', 'high'].map(k => [k, value[k]]));
  const before = snapshot(h);
  h.quantity *= action.ratio;
  for (const key of ['cost', 'price', 'low', 'base', 'high']) h[key] /= action.ratio;
  const after = snapshot(h);
  const cancelledPlanIds = [];
  for (const p of result.plans) if (p.holdingId === h.id && ['draft', 'reviewed'].includes(p.status)) {
    p.status = 'cancelled'; p.cancelReason = '拆并股改变数量与价格口径，请按调整后的单位重新建立计划。';
    cancelledPlanIds.push(p.id);
  }
  result.corporateActions ||= [];
  result.corporateActions.push({ id: action.id, type: 'split', date: action.date, holdingId: h.id, symbol: h.symbol, ratio: action.ratio, note: action.note.trim(), recordedAt: new Date().toISOString(), before, after, cancelledPlanIds });
  return assertResult(result);
}

export function validateState(input) {
  const errors = [];
  const obj = value => value !== null && typeof value === 'object' && !Array.isArray(value);
  if (!obj(input)) return { valid: false, errors: ['备份必须是 JSON 对象。'] };
  function safeTree(value, path = '数据', depth = 0) {
    if (depth > 20) { errors.push(`${path} 嵌套过深。`); return; }
    if (typeof value === 'number' && !Number.isFinite(value)) errors.push(`${path} 包含无效数字。`);
    if (value && typeof value === 'object') for (const [k, v] of Object.entries(value)) {
      if (['__proto__', 'prototype', 'constructor'].includes(k)) errors.push(`${path} 包含禁止字段。`);
      else safeTree(v, `${path}.${k}`, depth + 1);
    }
  }
  safeTree(input);
  const num = (value, path, max = Number.MAX_SAFE_INTEGER) => { if (!finite(value) || value < 0 || value > max) errors.push(`${path} 必须是 0 至 ${max} 的有限数字。`); };
  const str = (value, path, required = false) => { if (typeof value !== 'string' || value.length > 20000 || (required && !value.trim())) errors.push(`${path} 文本无效。`); };
  const choice = (value, values, path) => { if (!values.includes(value)) errors.push(`${path} 选项无效。`); };
  const date = (value, path, empty = false) => { if (!(empty && value === '') && !isDate(value)) errors.push(`${path} 日期无效。`); };
  const list = (key, optional = false) => {
    if (optional && input[key] === undefined) return [];
    if (!Array.isArray(input[key]) || input[key].length > 10000) { errors.push(`${key} 必须是最多 10000 项的数组。`); return []; }
    const ids = new Set();
    return input[key].filter((v, index) => {
      if (!obj(v)) { errors.push(`${key} 第 ${index + 1} 项无效。`); return false; }
      str(v.id, `${key}.id`, true);
      if (typeof v.id !== 'string' || !/^[A-Za-z0-9_-]{1,100}$/.test(v.id)) errors.push(`${key}.id 仅允许 1 至 100 位英文字母、数字、下划线与短横线。`);
      if (ids.has(v.id)) errors.push(`${key} 存在重复 ID。`);
      ids.add(v.id); return true;
    });
  };
  if (input.schemaVersion !== 1) errors.push('不支持该备份版本。');
  if (!obj(input.settings)) errors.push('缺少账户设置。');
  else {
    const s = input.settings;
    choice(s.currency, ['USD', 'CNY', 'HKD', 'EUR', 'GBP', 'JPY'], '账户币种');
    for (const key of ['cash', 'lifeCash']) num(s[key], key);
    for (const key of ['budget', 'singleLimit', 'themeLimit', 'targetGrowth', 'targetDefense', 'targetCash', 'buffer', 'maxMonthly', 'growthShock', 'defenseShock', 'cashShock']) num(s[key], key, 100);
    if (Math.abs(s.targetGrowth + s.targetDefense + s.targetCash - 100) > 0.001) errors.push('成长、防守与现金目标必须合计 100%。');
    choice(s.market, ['positive', 'neutral', 'negative', 'unknown', 'cool', 'hot', 'risk', 'cold', 'warm'], '市场环境');
    str(s.marketNote, '市场备注');
  }
  const holdings = list('holdings');
  const symbols = new Set();
  for (const h of holdings) {
    for (const key of ['symbol', 'name', 'sector', 'theme', 'thesis', 'invalidation', 'evidence']) str(h[key], `持仓.${key}`, key === 'symbol');
    if (typeof h.symbol === 'string') { const normalized = h.symbol.trim().toUpperCase(); if (symbols.has(normalized)) errors.push('同一账户的标的代码不能重复。'); symbols.add(normalized); }
    choice(h.bucket, BUCKETS, '持仓分类');
    if (h.archived !== undefined && typeof h.archived !== 'boolean') errors.push('持仓归档状态必须是布尔值。');
    if (h.archived === true && (h.quantity > 0 || (Array.isArray(input.plans) ? input.plans : []).some(p => p?.holdingId === h.id && ['draft', 'reviewed'].includes(p.status)))) errors.push('有持仓或未完成交易计划的标的不能归档。');
    for (const key of ['quantity', 'cost', 'price', 'low', 'base', 'high']) num(h[key], `持仓.${key}`);
    if (h.quantity > 0 && h.price <= 0) errors.push('有持股的标的必须录入正数价格。');
    if (!Number.isFinite(h.quantity * h.price)) errors.push('持仓市值超出计算范围。');
    if ((h.low || h.base || h.high) && !(h.low <= h.base && h.base <= h.high)) errors.push('情景价值必须满足悲观 ≤ 基准 ≤ 乐观。');
    choice(h.scarcity, TRENDS, '稀缺性'); choice(h.differentiation, TRENDS, '差异化');
    choice(h.valuation, ['attractive', 'fair', 'expensive', 'unknown'], '估值判断');
    for (const key of ['priceDate', 'reviewDate', 'nextReview']) date(h[key], `持仓.${key}`, true);
  }
  for (const e of list('events')) {
    for (const key of ['symbol', 'type', 'title', 'source', 'note']) str(e[key], `事件.${key}`, key === 'title');
    date(e.date, '事件日期'); choice(e.verification, ['pending', 'verified', 'refuted'], '证据状态'); choice(e.impact, ['positive', 'neutral', 'negative'], '事件影响');
    if (typeof e.reviewed !== 'boolean') errors.push('事件复核状态无效。');
    if (e.followUp !== undefined) date(e.followUp, '事件跟进日期', true);
    if (e.updatedAt !== undefined && (typeof e.updatedAt !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?(?:Z|[+-]\d{2}:\d{2})$/.test(e.updatedAt) || !isDate(e.updatedAt.slice(0, 10)) || Number.isNaN(Date.parse(e.updatedAt)))) errors.push('事件更新时间必须是有效 ISO 日期时间。');
  }
  for (const p of list('plans')) {
    for (const key of ['holdingId', 'symbol', 'rationale']) str(p[key], `计划.${key}`, key !== 'rationale');
    date(p.date, '计划日期'); choice(p.side, ['buy', 'sell'], '买卖方向'); choice(p.reason, ['thesis', 'valuation', 'risk', 'opportunity'], '计划原因'); choice(p.status, ['draft', 'reviewed', 'executed', 'cancelled'], '计划状态');
    for (const key of ['quantity', 'price', 'fee']) num(p[key], `计划.${key}`);
    if (p.quantity <= 0 || p.price <= 0) errors.push('计划数量和价格必须大于零。');
    const holding = holdings.find(h => h.id === p.holdingId);
    if (!holding || holding.symbol !== p.symbol) errors.push('计划引用的标的不存在或代码不一致。');
    if (!obj(p.checklist) || !['thesis', 'value', 'risk', 'evidence'].every(k => typeof p.checklist[k] === 'boolean')) errors.push('计划检查项缺失。');
    if (p.reviewedAt !== undefined && (typeof p.reviewedAt !== 'string' || Number.isNaN(Date.parse(p.reviewedAt)))) errors.push('计划审查时间无效。');
    if (p.reviewFingerprint !== undefined && (typeof p.reviewFingerprint !== 'string' || !/^review-v1-\d+-[a-f0-9]+-[a-f0-9]+$/.test(p.reviewFingerprint))) errors.push('计划审查快照无效。');
    if (p.sourcePlanId !== undefined) {
      const source = (Array.isArray(input.plans) ? input.plans : []).find(x => x.id === p.sourcePlanId);
      if (!validId(p.sourcePlanId) || p.sourcePlanId === p.id || !source || source.status !== 'executed' || source.holdingId !== p.holdingId || source.side !== p.side) errors.push('剩余计划的来源无效。');
    }
    if (p.status === 'executed') {
      if (typeof p.executedAt !== 'string' || Number.isNaN(Date.parse(p.executedAt))) errors.push('成交登记时间无效。');
      if (!obj(p.execution)) errors.push('已登记交易缺少成交快照。');
      else {
        date(p.execution.date, '成交日期');
        for (const key of ['price', 'quantity', 'fee', 'amount', 'costBasis', 'impactPct', 'cashBefore', 'cashAfter', 'quantityBefore', 'quantityAfter']) num(p.execution[key], `成交.${key}`);
        for (const key of ['cashFlow', 'realizedPnl']) if (!finite(p.execution[key])) errors.push(`成交.${key} 无效。`);
        for (const key of ['growthPctBefore', 'growthPctAfter']) if (p.execution[key] !== undefined) num(p.execution[key], `成交.${key}`, 100);
        if (p.execution.quantity <= 0 || p.execution.price <= 0) errors.push('成交价格和数量必须大于零。');
        const x = p.execution, sign = p.side === 'buy' ? 1 : -1;
        if (!near(x.amount, x.price * x.quantity) || !near(x.cashAfter, x.cashBefore + x.cashFlow) || !near(x.quantityAfter, x.quantityBefore + sign * x.quantity)) errors.push('成交金额、资金或数量快照不一致。');
        if (!near(x.cashFlow, p.side === 'buy' ? -x.amount - x.fee : x.amount - x.fee)) errors.push('成交现金流与费用不一致。');
        if (!near(x.realizedPnl, p.side === 'sell' ? x.amount - x.fee - x.costBasis : 0) || (p.side === 'buy' && !near(x.costBasis, x.amount + x.fee))) errors.push('成交成本或盈亏快照不一致。');
        if (x.date > day()) errors.push('成交日期不能在未来。');
        if (x.remainingQuantity !== undefined && (!finite(x.remainingQuantity) || !near(x.remainingQuantity, Math.max(0, p.quantity - x.quantity)))) errors.push('剩余未成交数量与原计划不一致。');
      }
    }
  }
  for (const f of list('cashFlows', true)) {
    choice(f.type, ['deposit', 'withdrawal', 'dividend', 'fee'], '流水类型'); date(f.date, '流水日期');
    for (const key of ['amount', 'cashBefore', 'cashAfter']) num(f[key], `流水.${key}`);
    for (const key of ['note', 'holdingId', 'symbol']) str(f[key], `流水.${key}`, key === 'note');
    if (f.amount <= 0) errors.push('流水金额必须大于零。');
    if (f.date > day()) errors.push('流水日期不能在未来。');
    if (typeof f.recordedAt !== 'string' || Number.isNaN(Date.parse(f.recordedAt))) errors.push('流水登记时间无效。');
    if (f.holdingId && !holdings.some(h => h.id === f.holdingId && h.symbol === f.symbol)) errors.push('流水关联标的无效。');
    const signed = ['withdrawal', 'fee'].includes(f.type) ? -f.amount : f.amount;
    if (!near(f.cashAfter, f.cashBefore + signed)) errors.push('流水金额与现金快照不一致。');
  }
  for (const a of list('corporateActions', true)) {
    choice(a.type, ['split'], '公司行动类型'); date(a.date, '公司行动日期'); num(a.ratio, '拆并股比例');
    for (const key of ['holdingId', 'symbol', 'note']) str(a[key], `公司行动.${key}`, true);
    if (a.ratio <= 0 || a.ratio === 1) errors.push('拆并股比例必须大于零且不能为 1。');
    if (a.date > day()) errors.push('公司行动日期不能在未来。');
    if (!holdings.some(h => h.id === a.holdingId && h.symbol === a.symbol)) errors.push('公司行动关联标的无效。');
    if (typeof a.recordedAt !== 'string' || Number.isNaN(Date.parse(a.recordedAt))) errors.push('公司行动登记时间无效。');
    if (!obj(a.before) || !obj(a.after)) errors.push('公司行动缺少前后快照。');
    else {
      for (const snapshot of [a.before, a.after]) {
        for (const key of ['quantity', 'cost', 'price', 'low', 'base', 'high']) num(snapshot[key], `公司行动快照.${key}`);
        date(snapshot.priceDate, '公司行动价格日期', true);
      }
      if (a.before.quantity <= 0 || !near(a.after.quantity, a.before.quantity * a.ratio) || ['cost', 'price', 'low', 'base', 'high'].some(k => !near(a.after[k], a.before[k] / a.ratio))) errors.push('拆并股比例与单位调整不一致。');
      if (!near(a.before.quantity * a.before.cost, a.after.quantity * a.after.cost) || !near(a.before.quantity * a.before.price, a.after.quantity * a.after.price)) errors.push('拆并股不应改变持仓总成本与总市值。');
    }
    if (!Array.isArray(a.cancelledPlanIds) || !a.cancelledPlanIds.every(id => validId(id) && (Array.isArray(input.plans) ? input.plans : []).some(p => p.id === id))) errors.push('公司行动关联计划无效。');
  }
  const sourcePlanIds = (Array.isArray(input.plans) ? input.plans : []).filter(p => p?.sourcePlanId !== undefined).map(p => p.sourcePlanId);
  if (new Set(sourcePlanIds).size !== sourcePlanIds.length) errors.push('同一部分成交不能重复生成剩余计划。');
  const ledgerIds = [...(Array.isArray(input.plans) ? input.plans : []), ...(Array.isArray(input.cashFlows) ? input.cashFlows : []), ...(Array.isArray(input.corporateActions) ? input.corporateActions : [])].filter(obj).map(x => x.id);
  if (new Set(ledgerIds).size !== ledgerIds.length) errors.push('交易、资金与公司行动记录的 ID 不能重复。');
  for (const j of list('journal')) {
    date(j.date, '复盘日期'); choice(j.discipline, ['yes', 'no', 'none'], '纪律复盘'); str(j.lesson, '复盘记录'); str(j.next, '下一步');
  }
  if (!obj(input.checks)) errors.push('每日检查状态无效。');
  else for (const [dateKey, checks] of Object.entries(input.checks)) {
    date(dateKey, '每日检查日期');
    if (!obj(checks) || !Object.values(checks).every(v => typeof v === 'boolean')) errors.push('每日检查记录必须是布尔值。');
  }
  return { valid: errors.length === 0, errors: [...new Set(errors)].slice(0, 30) };
}

/** UTF-8 BOM + RFC 4180 escaping, guarding formula evaluation in spreadsheets. */
export function csvExport(state) {
  const safe = value => {
    let str = value === null || value === undefined ? '' : String(value);
    if (typeof value === 'string' && /^[\s\uFEFF]*[=+@-]/.test(str)) str = `'${str}`;
    return `"${str.replaceAll('"', '""')}"`;
  };
  const trends = { stronger: '增强', stable: '稳定', weaker: '减弱', unknown: '待判断' };
  const rows = [['代码', '名称', '仓位分类', '数量', '单位成本', '录入价格', '价格日期', '市值', '仓位占比(%)', '稀缺性', '差异化', '投资逻辑', '证伪条件', '币种']];
  for (const h of portfolio(state).holdings) rows.push([h.symbol, h.name, { core: '核心成长', flex: '弹性成长', defense: '防守资产' }[h.bucket], h.quantity, h.cost, h.price, h.priceDate, h.value, h.weight, trends[h.scarcity], trends[h.differentiation], h.thesis, h.invalidation, state.settings.currency]);
  return '\uFEFF' + rows.map(row => row.map(safe).join(',')).join('\r\n');
}
