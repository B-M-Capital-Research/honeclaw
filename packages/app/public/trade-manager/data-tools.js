import { portfolio } from './model.js';

const MAX_BYTES = 1024 * 1024;
const MAX_ROWS = 5000;
const today = () => new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date());
const realDate = value => typeof value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(value) && !Number.isNaN(Date.parse(value)) && new Date(`${value}T00:00:00Z`).toISOString().slice(0, 10) === value;
const finite = value => typeof value === 'number' && Number.isFinite(value);
const clone = value => JSON.parse(JSON.stringify(value));
const normalized = value => String(value ?? '').trim().toUpperCase();
const safeCSV = value => {
  let text = String(value ?? '');
  if (typeof value === 'string' && /^[\s\uFEFF]*[=+@-]/.test(text)) text = `'${text}`;
  return `"${text.replaceAll('"', '""')}"`;
};

// Parse CSV as data only: quoted commas, escaped quotes, CRLF and quoted newlines.
function csvRecords(text) {
  const records = [];
  let row = [], value = '', quoted = false, closed = false, start = true;
  const cell = () => { row.push(value); value = ''; closed = false; start = true; };
  const line = () => { cell(); if (row.some(v => v.trim() !== '')) records.push(row); row = []; if (records.length > MAX_ROWS + 1) throw Error(`最多导入 ${MAX_ROWS} 条报价。`); };
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"') {
        if (text[i + 1] === '"') { value += '"'; i++; }
        else { quoted = false; closed = true; }
      } else value += c;
    } else if (closed) {
      if (c === ',') cell();
      else if (c === '\r' || c === '\n') { if (c === '\r' && text[i + 1] === '\n') i++; line(); }
      else if (c !== ' ' && c !== '\t') throw Error('引号结束后存在无效字符，请检查 CSV 格式。');
    } else if (c === '"') {
      if (!start) throw Error('未正确引用的双引号，请使用双引号包围含逗号或换行的字段。');
      quoted = true; start = false;
    } else if (c === ',') cell();
    else if (c === '\r' || c === '\n') { if (c === '\r' && text[i + 1] === '\n') i++; line(); }
    else { value += c; start = false; }
    if (value.length > 20000) throw Error('CSV 单个字段过长。');
  }
  if (quoted) throw Error('CSV 双引号未闭合。');
  if (row.length || value.length || closed) line();
  return records;
}

function quoteError(holding, row, seen, checkPreview) {
  if (!holding) return '代码不存在，请先添加对应标的。';
  if (seen.has(holding.id)) return '存在重复代码，每个标的只保留一条报价。';
  seen.add(holding.id);
  if (row.holdingId !== holding.id || normalized(row.symbol) !== normalized(holding.symbol)) return '代码与持仓标识不一致。';
  if (!finite(row.price) || row.price <= 0 || row.price > 1e12) return '价格必须是大于 0 且不超过 1 万亿的有限数字。';
  if (!realDate(row.date)) return '日期必须是有效的 YYYY-MM-DD。';
  if (row.date > today()) return '价格日期不能晚于今天（北京时间）。';
  if (realDate(holding.priceDate) && row.date < holding.priceDate) return '报价日期早于现有价格日期，不允许覆盖。';
  if (checkPreview && (row.oldPrice !== holding.price || row.oldDate !== holding.priceDate)) return '持仓价格在预览后已变化，请重新导入预览。';
  return '';
}

export function parseQuoteCSV(text, holdings) {
  const errors = [], rows = [];
  if (typeof text !== 'string') return { rows, errors: ['请选择 UTF-8 CSV 文本。'] };
  if (new TextEncoder().encode(text).byteLength > MAX_BYTES) return { rows, errors: ['CSV 文件超过 1MB。'] };
  if (!Array.isArray(holdings)) return { rows, errors: ['持仓数据无效。'] };
  try {
    const records = csvRecords(text.replace(/^\uFEFF/, ''));
    if (records.length < 2) throw Error('CSV 需包含表头和至少一条报价。');
    const aliases = { symbol: 'symbol', price: 'price', date: 'date', '代码': 'symbol', '价格': 'price', '日期': 'date' };
    const header = records.shift().map(s => aliases[s.trim().toLowerCase()]);
    if (header.length !== 3 || new Set(header).size !== 3 || header.some(k => !k)) throw Error('表头应为 symbol,price,date 或 代码,价格,日期；可调整列顺序。');
    const seen = new Set();
    for (let i = 0; i < records.length; i++) {
      const line = records[i], number = i + 2;
      if (line.length !== header.length) { errors.push(`第 ${number} 条记录列数不正确。`); continue; }
      const raw = Object.fromEntries(header.map((k, j) => [k, line[j].trim()]));
      let symbol = normalized(raw.symbol);
      // Our Excel-safe template prefixes formula-like codes with an apostrophe.
      if (/^'[=+@-]/.test(symbol) && holdings.some(h => normalized(h.symbol) === symbol.slice(1))) symbol = symbol.slice(1);
      const h = holdings.find(h => normalized(h.symbol) === symbol);
      const row = { symbol: h?.symbol || symbol, holdingId: h?.id, oldPrice: h?.price, oldDate: h?.priceDate, price: /^(?:\d+(?:\.\d*)?|\.\d+)$/.test(raw.price) ? Number(raw.price) : NaN, date: raw.date };
      const problem = quoteError(h, row, seen, false);
      if (problem) errors.push(`第 ${number} 条记录（${symbol || '空代码'}）：${problem}`);
      else rows.push(row);
      if (errors.length >= 50) { errors.push('错误超过 50 项，请修正后重新导入。'); break; }
    }
  } catch (error) { errors.push(error.message); }
  return { rows, errors };
}

export function quoteTemplate(holdings) {
  return '\uFEFF' + [['symbol', 'price', 'date'], ...holdings.map(h => [h.symbol, h.price > 0 ? h.price : '', realDate(h.priceDate) ? h.priceDate : today()])].map(row => row.map(safeCSV).join(',')).join('\r\n');
}

export function applyQuoteRows(state, rows) {
  if (!Array.isArray(rows) || !rows.length || rows.length > MAX_ROWS) throw Error('请先导入 1 至 5000 条有效报价并预览。');
  const seen = new Set();
  for (const row of rows) {
    if (!row || typeof row !== 'object') throw Error('报价记录无效。');
    const h = state.holdings.find(h => h.id === row.holdingId);
    const problem = quoteError(h, row, seen, true);
    if (problem) throw Error(`${row.symbol || '报价'}：${problem}`);
  }
  const next = clone(state);
  for (const row of rows) {
    const h = next.holdings.find(h => h.id === row.holdingId);
    h.price = row.price; h.priceDate = row.date;
  }
  return next;
}

function dateRange(from, to) {
  if ((from && !realDate(from)) || (to && !realDate(to))) throw Error('筛选日期必须为有效的 YYYY-MM-DD。');
  if (from && to && from > to) throw Error('开始日期不能晚于结束日期。');
  return date => (!from || date >= from) && (!to || date <= to);
}

/** Observed trade totals only, not account returns; slippage compares actual price to the written plan. */
export function decisionMetrics(state, { from = '', to = '', reason = 'all' } = {}) {
  const inRange = dateRange(from, to);
  if (!['all', 'thesis', 'valuation', 'risk', 'opportunity'].includes(reason)) throw Error('交易原因筛选无效。');
  const records = [];
  for (const p of state.plans) {
    if (p.status !== 'executed') continue;
    const e = p.execution;
    if (!e || !realDate(e.date) || ![e.quantity, e.price, e.fee, e.realizedPnl, p.price].every(finite) || e.quantity <= 0 || e.price <= 0 || p.price <= 0 || e.fee < 0) throw Error('存在不完整成交记录，请先校验或修复备份。');
    if (!inRange(e.date) || (reason !== 'all' && p.reason !== reason)) continue;
    records.push({ ...clone(p), date: e.date, amount: e.quantity * e.price, plannedPrice: p.price, actualPrice: e.price, slippageCost: e.quantity * (p.side === 'buy' ? e.price - p.price : p.price - e.price) });
  }
  records.sort((a, b) => b.date.localeCompare(a.date) || String(b.executedAt || '').localeCompare(String(a.executedAt || '')));
  const reasonCounts = { thesis: 0, valuation: 0, risk: 0, opportunity: 0 };
  let buyAmount = 0, sellAmount = 0, fees = 0, realizedPnl = 0, slippageCost = 0;
  for (const p of records) {
    if (p.side === 'buy') buyAmount += p.amount;
    else if (p.side === 'sell') { sellAmount += p.amount; realizedPnl += p.execution.realizedPnl; }
    else throw Error('存在无效成交方向。');
    fees += p.execution.fee;
    slippageCost += p.slippageCost;
    reasonCounts[p.reason] = (reasonCounts[p.reason] || 0) + 1;
  }
  let externalFlows = 0, dividends = 0, accountFees = 0;
  for (const flow of state.cashFlows || []) {
    if (!realDate(flow.date) || !finite(flow.amount) || flow.amount <= 0) throw Error('存在无效资金流水，请先核对。');
    if (!inRange(flow.date)) continue;
    if (flow.type === 'deposit') externalFlows += flow.amount;
    else if (flow.type === 'withdrawal') externalFlows -= flow.amount;
    else if (flow.type === 'dividend') dividends += flow.amount;
    else if (flow.type === 'fee') accountFees += flow.amount;
  }
  return { count: records.length, buyAmount, sellAmount, fees, realizedPnl, slippageCost, records, reasonCounts, externalFlows, dividends, accountFees };
}

/** Preserve numeric losses for spreadsheet calculations; sanitize formula-like text only. */
export function tradeCSV(state, filters = {}) {
  const records = decisionMetrics(state, filters).records;
  const rows = [['日期', '代码', '方向', '实际数量', '实际价格', '费用', '已实现盈亏', '原因', '依据', '币种']];
  for (const p of records) rows.push([p.execution.date, p.symbol, p.side === 'buy' ? '买入' : '卖出', p.execution.quantity, p.execution.price, p.execution.fee, p.execution.realizedPnl, reasonName[p.reason] || p.reason, p.rationale, state.settings.currency]);
  return '\uFEFF' + rows.map(row => row.map(safeCSV).join(',')).join('\r\n');
}

const md = value => String(value ?? '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/([\\`*_{}\[\]()#+.!|])/g, '\\$1').replace(/[\r\n]+/g, ' / ');
const number = (n, places = 2) => Number(n).toLocaleString('zh-CN', { maximumFractionDigits: places, minimumFractionDigits: places });
const reasonName = { thesis: '逻辑变化', valuation: '估值变化', risk: '风险再平衡', opportunity: '新增机会' };
const statusName = { pending: '待核实', verified: '已核实', refuted: '已证伪' };

export function dailyReport(state, reportDay = today()) {
  if (!realDate(reportDay) || reportDay > today()) throw Error('日报日期必须有效且不能晚于今天。');
  const p = portfolio(state), m = decisionMetrics(state, { from: reportDay, to: reportDay });
  const pending = state.events.filter(e => !e.reviewed);
  const journals = state.journal.filter(j => j.date === reportDay);
  const checks = state.checks?.[reportDay] || {};
  const money = n => `${md(state.settings.currency)} ${number(n)}`;
  const lines = [
    `# 巴芒交易日报 · ${reportDay}`, '',
    `生成时间：${new Intl.DateTimeFormat('zh-CN', { timeZone: 'Asia/Shanghai', dateStyle: 'medium', timeStyle: 'short' }).format(new Date())}（北京时间）`, '',
    '本报告由用户录入数据生成，未接入实时行情或券商。交易与复盘按所选日期筛选；账户、持仓与待办为生成时的当前快照，并非该日期的历史重建。', '',
    '## 当前账户快照', '',
    `- 可投资资产：${money(p.total)}；账户现金：${money(p.cash)}。`,
    `- 生活保障现金：${money(state.settings.lifeCash)}，未计入可投资资产。`,
    `- 成长／防守／现金占比：${number(p.growthPct, 1)}%／${number(p.defensePct, 1)}%／${number(p.cashPct, 1)}%。`,
    `- 人工目标：${number(state.settings.targetGrowth, 1)}%／${number(state.settings.targetDefense, 1)}%／${number(state.settings.targetCash, 1)}%。`, '',
    '| 标的 | 数量 | 手工参考价 | 参考价日期 | 市值 | 占比 |',
    '| --- | ---: | ---: | --- | ---: | ---: |',
    ...p.holdings.map(h => `| ${md(h.symbol)} | ${number(h.quantity, 4)} | ${number(h.price)} | ${md(h.priceDate || '未填写')} | ${number(h.value)} | ${number(h.weight, 1)}% |`),
    ...(p.holdings.length ? [] : ['| 暂无持仓 | — | — | — | — | — |']), '',
    '## 当前风险假设与提醒', '',
    `假设成长资产下跌 ${state.settings.growthShock}%，防守资产下跌 ${state.settings.defenseShock}%，现金损失 ${state.settings.cashShock}%；线性情景损失为 ${money(p.stressLoss)}（${number(p.stressPct, 1)}%），设定预算 ${state.settings.budget}%。这是情景演算，不是预测最大回撤或收益。`, '',
    ...(p.alerts.length ? p.alerts.map(a => `- ${md(a.title)}：${md(a.detail)}`) : ['- 当前输入未触发既有阈值，仍需核对数据完整性。']), '',
    '## 当前待复核事件', '',
    ...(pending.length ? pending.map(e => `- ${md(e.date)} · ${md(e.symbol || '宏观')} · ${statusName[e.verification] || '待判断'}：${md(e.title)}。影响记录：${md(e.note || '未填写')}。来源（原样文本）：${md(e.source || '未填写')}。`) : ['- 暂无未完成事件。']), '',
    `## ${reportDay} 成交与资金记录`, '',
    `已记账 ${m.count} 笔；买入金额 ${money(m.buyAmount)}；卖出金额 ${money(m.sellAmount)}；成交费用 ${money(m.fees)}；卖出已实现盈亏 ${money(m.realizedPnl)}。`, '',
    '已实现盈亏只覆盖工具内登记的卖出，已含成交费用，不能代表完整账户收益；不得再次减去上述成交费用。', '',
    ...(m.records.length ? m.records.map(t => `- ${md(t.symbol)} ${t.side === 'buy' ? '买入' : '卖出'} ${number(t.execution.quantity, 4)} 股 × ${number(t.actualPrice)}，费用 ${money(t.execution.fee)}；${reasonName[t.reason] || '其他'}：${md(t.rationale)}。`) : ['- 当日没有登记成交。']), '',
    `与计划价格的成交金额差额合计：${money(m.slippageCost)}（正数表示比计划更不利，负数表示更有利；按实际数量比较，不含费用，不等于市场冲击或完整执行质量）。`, '',
    `期间净入金：${money(m.externalFlows)}；现金分红：${money(m.dividends)}；独立账户费用：${money(m.accountFees)}。资金流入不计作投资盈利。`, '',
    `## ${reportDay} 检查与复盘`, '',
    ...[['quotes', '持仓与价格'], ['events', '事件与证伪条件'], ['risk', '风险预算'], ['decision', '今日决定']].map(([key, label]) => `- [${checks[key] ? 'x' : ' '}] ${label}`), '',
    ...(journals.length ? journals.map(j => `- ${j.discipline === 'yes' ? '遵守纪律' : j.discipline === 'no' ? '存在偏离' : '持有不动'}：${md(j.lesson)}。下一步：${md(j.next || '暂不调整')}。`) : ['- 尚未填写当日复盘。']), '',
    '报告为个人决策记录；不构成交易指令、自动订单、策略业绩或净值发布。', '',
  ];
  return lines.join('\n');
}
