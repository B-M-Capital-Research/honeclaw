import test from 'node:test';
import assert from 'node:assert/strict';
import { createDemoState } from '../../packages/app/public/trade-manager/model.js';
import { parseQuoteCSV, quoteTemplate, applyQuoteRows, decisionMetrics, dailyReport, tradeCSV } from '../../packages/app/public/trade-manager/data-tools.js';
const today = new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date());
const oldDate = '2024-02-29';
const fixtures = () => [{ id: 'h-a', symbol: 'TEST', price: 100, priceDate: oldDate }, { id: 'h-b', symbol: 'BRK.B', price: 80, priceDate: oldDate }];

test('quote template roundtrips BOM, header order and exact holding references', () => {
  const hs = fixtures();
  const parsed = parseQuoteCSV(quoteTemplate(hs), hs);
  assert.deepEqual(parsed.errors, []); assert.equal(parsed.rows.length, 2);
  assert.deepEqual(parsed.rows[0], { symbol: 'TEST', holdingId: 'h-a', oldPrice: 100, oldDate, price: 100, date: oldDate });
  assert.equal(parseQuoteCSV(`日期,代码,价格\r\n${today},test,123.45`, hs).rows[0].price, 123.45);
});

test('RFC 4180 quoting handles quoted commas, escaped quotes and quoted newlines without code execution', () => {
  const hs = [{ id: 'q', symbol: 'AB,"C\nD', price: 1, priceDate: oldDate }];
  const result = parseQuoteCSV(quoteTemplate(hs), hs);
  assert.deepEqual(result.errors, []); assert.equal(result.rows[0].symbol, hs[0].symbol);
  assert.equal(parseQuoteCSV('symbol,price,date\n"TEST,2,2024-02-29', fixtures()).errors.length, 1);
  assert.equal(parseQuoteCSV('symbol,price,date\n"TEST"oops,2,2024-02-29', fixtures()).errors.length, 1);
});

test('quote import rejects bad headers, missing columns, unknown and duplicate symbols', () => {
  const hs = fixtures();
  for (const text of ['name,price,date\nTEST,2,2024-02-29', 'symbol,price,date\nTEST,2', `symbol,price,date\nUNKNOWN,2,${today}`, `symbol,price,date\nTEST,2,${today}\n test ,3,${today}`]) assert.ok(parseQuoteCSV(text, hs).errors.length, text);
});

test('quote prices reject NaN, Infinity, formulas, zero and negative values', () => {
  for (const price of ['NaN', 'Infinity', '=1+2', '-1', '0', '1e999', '0x100', '1000000000001']) assert.ok(parseQuoteCSV(`symbol,price,date\nTEST,${price},${today}`, fixtures()).errors.length, price);
});

test('quote dates validate leap day, reject impossible dates, future and stale replacements', () => {
  assert.deepEqual(parseQuoteCSV('symbol,price,date\nTEST,1,2024-02-29', fixtures()).errors, []);
  for (const date of ['2023-02-29', '2024-02-30', '2024-13-01', '2024/02/29', '2999-01-01', '2024-02-28']) assert.ok(parseQuoteCSV(`symbol,price,date\nTEST,1,${date}`, fixtures()).errors.length, date);
});

test('quote limits stop large or excessive input', () => {
  assert.match(parseQuoteCSV('x'.repeat(1024 * 1024 + 1), fixtures()).errors[0], /1MB/);
  assert.match(parseQuoteCSV('symbol,price,date\n' + `TEST,1,${today}\n`.repeat(5001), fixtures()).errors[0], /5000/);
});

test('quote application is immutable, all-or-nothing and guards a stale preview', () => {
  const state = createDemoState(); state.holdings = fixtures();
  const before = JSON.stringify(state);
  const result = parseQuoteCSV(`symbol,price,date\nTEST,150,${today}\nBRK.B,90,${today}`, state.holdings);
  const next = applyQuoteRows(state, result.rows);
  assert.equal(next.holdings[0].price, 150); assert.equal(next.holdings[1].price, 90);
  assert.equal(JSON.stringify(state), before);
  assert.throws(() => applyQuoteRows(state, [...result.rows, {...result.rows[0], price: 0}]), /重复|价格/);
  assert.equal(JSON.stringify(state), before);
  const updated = structuredClone(state); updated.holdings[0].price = 101;
  assert.throws(() => applyQuoteRows(updated, result.rows), /预览后已变化/);
  assert.throws(() => applyQuoteRows(state, [{...result.rows[0], holdingId:'missing'}]), /代码不存在/);
  assert.throws(() => applyQuoteRows(state, [{...result.rows[0], symbol:'OTHER'}]), /不一致/);
});

test('Excel-safe template escapes formula-like stock symbols and roundtrips them as data', () => {
  const hs = [{ id: 'f', symbol: '=HYPERLINK("evil")', price: 10, priceDate: oldDate }];
  const csv = quoteTemplate(hs);
  assert.ok(csv.includes("'=HYPERLINK"));
  const parsed = parseQuoteCSV(csv, hs);
  assert.deepEqual(parsed.errors, []); assert.equal(parsed.rows[0].symbol, hs[0].symbol);
});

function ledger() {
  const s = createDemoState();
  const base = { ...s.plans[0], status: 'executed', executedAt: `${today}T03:00:00Z` };
  s.plans = [
    { ...base, id: 'buy', side: 'buy', price: 100, reason: 'opportunity', execution: { date: today, quantity: 3, price: 102, fee: 1, realizedPnl: 0 } },
    { ...base, id: 'sell', side: 'sell', price: 120, reason: 'risk', execution: { date: today, quantity: 2, price: 119, fee: 2, realizedPnl: 36 } },
    { ...base, id: 'before', side: 'sell', price: 110, reason: 'valuation', execution: { date: oldDate, quantity: 1, price: 115, fee: 3, realizedPnl: 62 } },
    { ...base, id: 'draft', status: 'draft' },
  ];
  s.cashFlows = [
    { date: today, type: 'deposit', amount: 500 },
    { date: today, type: 'withdrawal', amount: 100 },
    { date: today, type: 'dividend', amount: 12 },
    { date: today, type: 'fee', amount: 2 },
    { date: oldDate, type: 'deposit', amount: 999 },
  ];
  return s;
}

test('decision totals use actual fills, separate funding and never double-deduct fees', () => {
  const s = ledger(), before = JSON.stringify(s);
  const m = decisionMetrics(s, { from: today, to: today });
  assert.equal(m.count, 2); assert.equal(m.buyAmount, 306); assert.equal(m.sellAmount, 238);
  assert.equal(m.fees, 3); assert.equal(m.realizedPnl, 36); assert.equal(m.slippageCost, 8);
  assert.equal(m.externalFlows, 400); assert.equal(m.dividends, 12); assert.equal(m.accountFees, 2);
  assert.equal(m.reasonCounts.risk, 1); assert.equal(m.reasonCounts.opportunity, 1);
  assert.equal(JSON.stringify(s), before);
});

test('decision filters include boundary dates and correctly sign buy/sell plan difference', () => {
  const m = decisionMetrics(ledger(), { from: oldDate, to: oldDate, reason: 'valuation' });
  assert.equal(m.count, 1); assert.equal(m.slippageCost, -5); assert.equal(m.realizedPnl, 62);
  assert.equal(decisionMetrics(ledger(), { from: today, reason: 'risk' }).count, 1);
  assert.throws(() => decisionMetrics(ledger(), { from: '2023-02-29' }), /有效/);
  assert.throws(() => decisionMetrics(ledger(), { from: today, to: oldDate }), /晚于/);
  assert.throws(() => decisionMetrics(ledger(), { reason: 'unknown' }), /无效/);
});

test('missing legacy cashFlows stays valid while incomplete execution fails loudly', () => {
  const s = ledger(); delete s.cashFlows;
  assert.equal(decisionMetrics(s).externalFlows, 0);
  delete s.plans[0].execution.price;
  assert.throws(() => decisionMetrics(s), /不完整成交/);
});

test('daily report makes snapshot timing, manual data and non-return accounting explicit', () => {
  const s = ledger();
  s.journal = [{ id: 'j', date: today, discipline: 'none', lesson: '等待证据', next: '财报后复核' }];
  s.checks[today] = { quotes: true };
  const report = dailyReport(s, today);
  assert.match(report, /当前快照，并非该日期的历史重建/);
  assert.match(report, /未接入实时行情或券商/);
  assert.match(report, /不得再次减去上述成交费用/);
  assert.match(report, /资金流入不计作投资盈利/);
  assert.match(report, /等待证据/); assert.match(report, /\[x\] 持仓与价格/);
  assert.match(report, /NVDA/); assert.match(report, /价格日期|参考价日期/);
  assert.throws(() => dailyReport(s, '2999-01-01'), /晚于/);
});

test('daily report escapes user Markdown and HTML rather than creating links or embedded content', () => {
  const s = ledger();
  s.events[0].title = '<img src=x onerror=alert(1)> [open](javascript:bad) | #title';
  s.events[0].source = '[fake](https://evil.test)';
  const report = dailyReport(s);
  assert.ok(!report.includes('<img')); assert.ok(!report.includes('[open](javascript:bad)'));
  assert.ok(report.includes('&lt;img')); assert.ok(report.includes('\\[open\\]'));
  assert.ok(!report.includes('[fake](https://evil.test)'));
});

test('trade CSV retains negative realized P&L as numeric content and applies date filters', () => {
  const s = ledger(); s.plans.find(p => p.id === 'sell').execution.realizedPnl = -123;
  const csv = tradeCSV(s, { from: today, to: today });
  assert.ok(csv.startsWith('\uFEFF"日期","代码","方向"'));
  assert.ok(csv.includes(',"-123",'), 'negative number must remain numeric when imported');
  assert.ok(!csv.includes("'-123"), 'numeric losses must not get formula-text apostrophes');
  assert.equal(csv.split('\r\n').length, 3, 'header plus two selected-date executions');
  assert.ok(!csv.includes(oldDate), 'older executions are outside the selected period');
  assert.equal(tradeCSV(s, { from: today, to: today, reason: 'risk' }).split('\r\n').length, 2);
});

test('trade CSV protects formula-like text while escaping quotes, commas and newlines', () => {
  const s = ledger(); const p = s.plans.find(p => p.id === 'sell');
  p.symbol = '=HYPERLINK("https://evil.test")'; p.rationale = '-cmd,"quoted"\nsecond line';
  const csv = tradeCSV(s, { from: today, to: today, reason: 'risk' });
  assert.ok(csv.includes('"\'=HYPERLINK(""https://evil.test"")"'));
  assert.ok(csv.includes('"\'-cmd,""quoted""\nsecond line"'));
  assert.ok(csv.includes(',"36",'), 'positive numeric values also remain numeric');
  assert.throws(() => tradeCSV(s, { from: today, to: oldDate }), /晚于/);
});
