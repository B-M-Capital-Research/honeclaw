import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const source = await readFile(new URL('../../packages/app/public/trade-manager/model.js', import.meta.url), 'utf8');
const M = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
const close = (actual, expected) => assert.ok(Math.abs(actual - expected) < 1e-7, `${actual} != ${expected}`);
const fullChecks = { thesis: true, value: true, risk: true, evidence: true };
const today = new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date());

function fixture() {
  const s = M.createDemoState();
  s.settings.cash = 1000;
  s.settings.lifeCash = 50000;
  s.holdings = [{ ...s.holdings[0], id: 'h', symbol: 'TEST', quantity: 10, cost: 50, price: 100 }];
  s.plans = [{ ...s.plans[0], id: 'p', holdingId: 'h', symbol: 'TEST', quantity: 2, price: 100, fee: 2, status: 'reviewed', checklist: fullChecks }];
  return M.reviewPlanState(s, 'p', fullChecks);
}

test('empty and synthetic data pass deep validation', () => {
  assert.deepEqual(M.validateState(M.createState()), { valid: true, errors: [] });
  assert.deepEqual(M.validateState(M.createDemoState()), { valid: true, errors: [] });
  assert.equal(M.createDemoState().plans.some(p => p.status === 'executed'), false);
});

test('portfolio excludes household reserve and includes defensive equity losses', () => {
  const s = fixture();
  s.holdings.push({ ...s.holdings[0], id: 'd', symbol: 'DEF', quantity: 5, bucket: 'defense', theme: '防守' });
  const p = M.portfolio(s);
  close(p.total, 2500); close(p.growth, 1000); close(p.defense, 500); close(p.cash, 1000);
  close(p.stressLoss, 575); close(p.stressPct, 23);
  close(p.growthPct + p.defensePct + p.cashPct, 100);
  close(p.holdings[0].pnl, 500);
  close(p.themeWeights.find(t => t.theme === 'AI').weight, 40);
});

test('zero account never yields NaN or fabricated exposure', () => {
  const p = M.portfolio(M.createState());
  assert.equal(p.total, 0); assert.equal(p.stressPct, 0); assert.equal(p.cashPct, 0);
  assert.ok(p.alerts.some(a => a.title.includes('基线')));
});

test('two-asset ceiling accounts for defensive losses and infeasibility', () => {
  close(M.allocationCeiling(30, 50, 0), 60);
  close(M.allocationCeiling(30, 50, 15), 100 * 15 / 35);
  assert.equal(M.allocationCeiling(10, 50, 15), null);
  assert.equal(M.allocationCeiling(50, 50, 50), 100);
  assert.equal(M.allocationCeiling(40, 50, 50), null);
  assert.equal(M.allocationCeiling(0, 50, 0), 0);
  assert.equal(M.allocationCeiling(NaN, 50, 0), null);
});

test('draft simulation does not mutate state and incorporates fees', () => {
  const s = fixture(); s.plans[0].status = 'draft';
  const original = JSON.stringify(s);
  const sim = M.simulatePlan(s, s.plans[0]);
  assert.equal(sim.valid, true); assert.equal(JSON.stringify(s), original);
  close(sim.after.total, 1998); close(sim.after.cash, 798); close(sim.impactPct, 10);
  close(sim.after.holdings[0].cost, 702 / 12);
});

test('cash and share constraints reject invalid simulations', () => {
  const s = fixture(), p = s.plans[0];
  assert.equal(M.simulatePlan(s, { ...p, quantity: 11 }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, side: 'sell', quantity: 11 }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, quantity: -1 }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, price: Infinity }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, fee: NaN }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, holdingId: 'missing' }).valid, false);
  assert.equal(M.simulatePlan(s, { ...p, symbol: 'OTHER' }).valid, false);
});

test('executed buy preserves source, captures actual fill and adjusts weighted cost', () => {
  const s = fixture(), original = JSON.stringify(s);
  const next = M.executePlan(s, 'p', { quantity: 3, price: 90, fee: 3, date: today });
  assert.equal(JSON.stringify(s), original);
  assert.equal(next.plans[0].status, 'executed');
  close(next.holdings[0].quantity, 13); close(next.holdings[0].cost, 773 / 13);
  close(next.settings.cash, 727); assert.equal(next.holdings[0].price, 100);
  close(next.plans[0].execution.cashFlow, -273);
  close(next.plans[0].execution.quantity, 3);
  assert.equal(next.plans[0].quantity, 2, 'original plan retained beside actual fill');
  assert.equal(M.validateState(next).valid, true);
  assert.throws(() => M.executePlan(next, 'p', { quantity: 3, price: 90, fee: 3, date: today }), /重复/);
});

test('sell realizes P&L with fees, keeps remaining cost and supports complete exit', () => {
  const s = fixture(); s.plans[0].side = 'sell';
  const next = M.executePlan(M.reviewPlanState(s, 'p', fullChecks), 'p', { quantity: 4, price: 90, fee: 2, date: today });
  close(next.settings.cash, 1358); close(next.holdings[0].quantity, 6); close(next.holdings[0].cost, 50);
  close(next.plans[0].execution.realizedPnl, 158); close(next.plans[0].execution.costBasis, 200);
  const full = M.executePlan(M.reviewPlanState(s, 'p', fullChecks), 'p', { quantity: 10, price: 90, fee: 2, date: today });
  assert.equal(full.holdings[0].quantity, 0);
  assert.equal(M.validateState(full).valid, true);
});

test('execution requires review, all checklist items, and a valid actual date', () => {
  const fill = { quantity: 1, price: 90, fee: 0, date: today };
  const draft = fixture(); draft.plans[0].status = 'draft';
  assert.throws(() => M.executePlan(draft, 'p', fill), /复核/);
  const incomplete = fixture(); incomplete.plans[0].checklist = { ...fullChecks, evidence: false };
  assert.throws(() => M.executePlan(incomplete, 'p', fill), /检查/);
  assert.throws(() => M.executePlan(fixture(), 'p', { ...fill, date: '2099-01-01' }), /日期/);
  assert.throws(() => M.executePlan(fixture(), 'p', { ...fill, date: '2026-02-30' }), /日期/);
});

test('import rejects schema errors, duplicate IDs, malformed links and invalid numeric content', () => {
  for (const mutate of [
    s => s.settings.cash = -1,
    s => s.settings.targetCash = 20,
    s => s.settings.growthShock = Infinity,
    s => s.holdings[0].quantity = '10',
    s => s.holdings.push({ ...s.holdings[0] }),
    s => s.plans[0].holdingId = 'missing',
    s => s.holdings[0].low = 9999,
    s => s.events[0].date = '2026-02-30',
    s => s.checks = { [today]: { thesis: 'yes' } },
    s => s.schemaVersion = 2,
    s => { s.holdings[0].id = 'bad\" onclick=\"alert(1)'; s.plans[0].holdingId = s.holdings[0].id; },
  ]) {
    const s = fixture(); mutate(s); assert.equal(M.validateState(s).valid, false);
  }
  const polluted = JSON.parse(JSON.stringify(fixture()).replace('"checks":{}', '"checks":{"__proto__":{"polluted":true}}'));
  assert.equal(M.validateState(polluted).valid, false);
  assert.equal(M.validateState(null).valid, false);
});

test('CSV escapes quotes, newlines and formula injection', () => {
  const s = fixture(); s.holdings[0].thesis = '=HYPERLINK("https://example.org")\nnext';
  const csv = M.csvExport(s);
  assert.ok(csv.startsWith('\uFEFF'));
  assert.ok(csv.includes('"\'=HYPERLINK(""https://example.org"")\nnext"'));
  assert.ok(csv.includes('"USD"'));
  assert.ok(csv.includes('"单位成本"')); assert.ok(csv.includes('"TEST"'));
  assert.ok(csv.includes('"1000"'), 'holding market value is exported');
});

test('stale price and thesis warnings are visible without executing a trade', () => {
  const s = fixture(); s.holdings[0].priceDate = '2020-01-01'; s.holdings[0].differentiation = 'weaker';
  const p = M.portfolio(s);
  assert.equal(p.holdings[0].priceStale, true);
  assert.ok(p.alerts.some(a => a.title.includes('投资逻辑')));
  assert.equal(s.plans[0].status, 'reviewed');
});

test('market options and real calendar dates are accepted or rejected accurately', () => {
  for (const market of ['positive', 'neutral', 'negative', 'unknown']) {
    const s = fixture(); s.settings.market = market; assert.equal(M.validateState(s).valid, true);
  }
  const leap = fixture(); leap.holdings[0].reviewDate = '2024-02-29';
  assert.equal(M.validateState(leap).valid, true);
  leap.holdings[0].reviewDate = '2025-02-29';
  assert.equal(M.validateState(leap).valid, false);
  const future = fixture(); future.holdings[0].priceDate = '2099-01-01';
  assert.ok(M.portfolio(future).alerts.some(a => a.detail.includes('价格日期在未来')));
});

test('tactical changes accumulate absolute growth-weight changes, not net changes', () => {
  const s = fixture(); s.plans[0].reason = 'risk';
  const bought = M.executePlan(M.reviewPlanState(s, 'p', fullChecks), 'p', { quantity: 2, price: 100, fee: 0, date: today });
  close(bought.plans[0].execution.growthPctBefore, 50);
  close(bought.plans[0].execution.growthPctAfter, 60);
  bought.plans.push({ ...s.plans[0], id: 'p-sell', side: 'sell' });
  const sold = M.executePlan(M.reviewPlanState(bought, 'p-sell', fullChecks), 'p-sell', { quantity: 2, price: 100, fee: 0, date: today });
  const p = M.portfolio(sold);
  close(p.growthPct, 50); close(p.monthlyTacticalChangePct, 20);
  assert.equal(p.monthlyTacticalCount, 2); assert.equal(p.monthlyTacticalKnownCount, 2); assert.equal(p.monthlyTacticalUnknownCount, 0);
  assert.ok(p.alerts.some(a => a.title.includes('战术调整超过')));
});

test('legacy tactical snapshots remain unknown and unrelated trades do not count', () => {
  const s = fixture(); s.plans[0].reason = 'risk';
  const executed = M.executePlan(M.reviewPlanState(s, 'p', fullChecks), 'p', { quantity: 2, price: 100, fee: 0, date: today });
  delete executed.plans[0].execution.growthPctBefore;
  delete executed.plans[0].execution.growthPctAfter;
  const legacy = M.portfolio(executed);
  assert.equal(legacy.monthlyTacticalChangePct, 0); assert.equal(legacy.monthlyTacticalUnknownCount, 1);
  assert.ok(legacy.alerts.some(a => a.title.includes('统计不完整')));
  assert.equal(M.validateState(executed).valid, true);
  executed.plans[0].reason = 'opportunity';
  assert.equal(M.portfolio(executed).monthlyTacticalCount, 0);
  executed.plans[0].reason = 'risk'; executed.plans[0].execution.date = '2020-01-01';
  assert.equal(M.portfolio(executed).monthlyTacticalCount, 0);
});

test('ledger rejects out-of-order dates while allowing same-day sequential fills', () => {
  const s = fixture();
  const first = M.executePlan(s, 'p', { quantity: 2, price: 100, fee: 0, date: today });
  first.plans.push({ ...s.plans[0], id: 'p-second', side: 'sell' });
  const yesterday = new Date(`${today}T00:00:00Z`); yesterday.setUTCDate(yesterday.getUTCDate() - 1);
  assert.throws(() => M.executePlan(M.reviewPlanState(first, 'p-second', fullChecks), 'p-second', { quantity: 2, price: 110, fee: 0, date: yesterday.toISOString().slice(0, 10) }), /按成交日期顺序录入/);
  const sameDay = M.executePlan(M.reviewPlanState(first, 'p-second', fullChecks), 'p-second', { quantity: 2, price: 110, fee: 0, date: today });
  assert.equal(sameDay.plans[1].status, 'executed');
  close(sameDay.holdings[0].quantity, 10);
  close(sameDay.plans[1].execution.costBasis, 2 * 700 / 12);
});


test('cash flow ledger separates deposits, withdrawals, dividends and fees from trade gains', () => {
  const s = fixture(), original = JSON.stringify(s);
  let n = M.recordCashFlow(s, { id: 'cash-1', date: today, type: 'deposit', amount: 200, note: '银行转入测试' });
  n = M.recordCashFlow(n, { id: 'cash-2', date: today, type: 'withdrawal', amount: 100, note: '银行转出测试' });
  n = M.recordCashFlow(n, { id: 'cash-3', date: today, type: 'dividend', amount: 25, holdingId: 'h', note: '已核对净到账股息' });
  n = M.recordCashFlow(n, { id: 'cash-4', date: today, type: 'fee', amount: 5, note: '托管费用' });
  close(n.settings.cash, 1120); close(n.settings.lifeCash, 50000);
  assert.deepEqual(n.holdings, s.holdings); assert.equal(n.plans[0].status, 'reviewed');
  assert.equal(n.cashFlows.length, 4); assert.equal(n.cashFlows[2].symbol, 'TEST');
  assert.equal(JSON.stringify(s), original); assert.equal(M.validateState(n).valid, true);
  assert.throws(() => M.recordCashFlow(n, { id: 'cash-1', date: today, type: 'deposit', amount: 10, note: 'duplicate' }), /重复/);
  assert.throws(() => M.recordCashFlow(n, { id: 'cash-5', date: today, type: 'withdrawal', amount: 1121, note: 'too much' }), /现金不足/);
});

test('cash flow rejects invalid inputs and mixed ledger out-of-order entries', () => {
  const s = fixture(), good = { id: 'cash', date: today, type: 'deposit', amount: 100, note: 'verified' };
  for (const change of [{ amount: 0 }, { amount: Infinity }, { amount: -1 }, { type: 'other' }, { date: '2099-01-01' }, { date: '2026-02-30' }, { note: '' }, { holdingId: 'missing' }]) assert.throws(() => M.recordCashFlow(s, { ...good, ...change }));
  const next = M.recordCashFlow(s, good);
  assert.throws(() => M.executePlan(M.reviewPlanState(next, 'p', fullChecks), 'p', { quantity: 1, price: 100, fee: 0, date: '2020-01-01' }), /按成交日期顺序录入/);
  assert.throws(() => M.recordCashFlow(next, { ...good, id: 'earlier', date: '2020-01-01' }), /按成交日期顺序录入/);
});

test('split preserves aggregate cost and value, updates valuation units and cancels active plans', () => {
  const s = fixture(), original = JSON.stringify(s), before = M.portfolio(s);
  const n = M.applySplit(s, { id: 'split', date: today, holdingId: 'h', ratio: 8, note: '公告确认 1 拆 8' });
  close(n.holdings[0].quantity, 80); close(n.holdings[0].cost, 6.25); close(n.holdings[0].price, 12.5);
  close(n.holdings[0].base, s.holdings[0].base / 8);
  close(M.portfolio(n).total, before.total); close(M.portfolio(n).holdings[0].pnl, before.holdings[0].pnl);
  assert.equal(n.settings.cash, s.settings.cash); assert.equal(n.plans[0].status, 'cancelled');
  assert.match(n.plans[0].cancelReason, /拆并股/); assert.deepEqual(n.corporateActions[0].cancelledPlanIds, ['p']);
  assert.equal(JSON.stringify(s), original); assert.equal(M.validateState(n).valid, true);
  const reverse = M.applySplit(n, { id: 'reverse', date: today, holdingId: 'h', ratio: 0.125, note: '反向单位调整测试' });
  close(reverse.holdings[0].quantity, 10); close(reverse.holdings[0].cost, 50);
});

test('split refuses invalid ratios and preserves prior executions unchanged', () => {
  const s = fixture(), good = { id: 'split', date: today, holdingId: 'h', ratio: 2, note: 'verified' };
  for (const change of [{ ratio: 0 }, { ratio: 1 }, { ratio: -2 }, { ratio: Infinity }, { holdingId: 'missing' }, { note: '' }, { date: '2099-01-01' }]) assert.throws(() => M.applySplit(s, { ...good, ...change }));
  const executed = M.executePlan(M.reviewPlanState(s, 'p', fullChecks), 'p', { quantity: 2, price: 100, fee: 0, date: today });
  const n = M.applySplit(executed, good);
  assert.deepEqual(n.plans[0], executed.plans[0]); assert.equal(M.validateState(n).valid, true);
});

test('legacy backups migrate without mandatory new arrays and tampered financial snapshots are rejected', () => {
  const old = fixture(); delete old.cashFlows; delete old.corporateActions;
  assert.equal(M.validateState(old).valid, true);
  const flow = M.recordCashFlow(old, { id: 'cash', date: today, type: 'deposit', amount: 100, note: 'verified' });
  flow.cashFlows[0].cashAfter += 1; assert.equal(M.validateState(flow).valid, false);
  const split = M.applySplit(old, { id: 'split', date: today, holdingId: 'h', ratio: 2, note: 'verified' });
  split.corporateActions[0].after.quantity += 1; assert.equal(M.validateState(split).valid, false);
  const trade = M.executePlan(old, 'p', { quantity: 2, price: 100, fee: 0, date: today });
  trade.plans[0].execution.realizedPnl = 500; assert.equal(M.validateState(trade).valid, false);
  const malformed = fixture(); malformed.plans = {}; assert.equal(M.validateState(malformed).valid, false);
});


test('review requires written thesis, invalidation and evidence, and detects relevant changes only', () => {
  const s = fixture(); assert.equal(M.planReviewFresh(s, s.plans[0]), true);
  for (const key of ['thesis', 'invalidation', 'evidence']) {
    const incomplete = fixture(); incomplete.holdings[0][key] = '  ';
    assert.throws(() => M.reviewPlanState(incomplete, 'p', fullChecks), /补充/);
  }
  for (const change of [
    n => n.settings.cash += 1,
    n => n.settings.budget += 1,
    n => n.holdings[0].price += 1,
    n => n.holdings[0].thesis += ' changed',
    n => n.events[0].note += ' new evidence',
    n => n.events[0].reviewed = true,
    n => n.plans[0].quantity += 1,
  ]) { const n = fixture(); change(n); assert.equal(M.planReviewFresh(n, n.plans[0]), false); }
  const routine = fixture(); routine.journal.push({ id: 'j', date: today, discipline: 'none', lesson: 'review', next: '' }); routine.checks[today] = { risk: true };
  assert.equal(M.planReviewFresh(routine, routine.plans[0]), true);
  assert.throws(() => M.reviewPlanState(s, 'p', { ...fullChecks, risk: false }), /全部四项/);
});

test('legacy or stale review must be refreshed before fill bookkeeping', () => {
  const fill = { quantity: 1, price: 100, fee: 0, date: today };
  const legacy = fixture(); delete legacy.plans[0].reviewFingerprint; delete legacy.plans[0].reviewedAt;
  assert.equal(M.validateState(legacy).valid, true); assert.equal(M.planReviewFresh(legacy, legacy.plans[0]), false);
  assert.throws(() => M.executePlan(legacy, 'p', fill), /重新审查/);
  const refreshed = M.reviewPlanState(legacy, 'p', fullChecks);
  assert.equal(M.executePlan(refreshed, 'p', fill).plans[0].status, 'executed');
  const stale = fixture(); stale.holdings[0].price = 101;
  assert.throws(() => M.executePlan(stale, 'p', fill), /重新审查/);
});

test('editing a reviewed plan resets every check and cannot edit an executed record', () => {
  const s = fixture(), original = JSON.stringify(s);
  const n = M.updatePlan(s, 'p', { quantity: 3, price: 90, rationale: 'revised plan' });
  assert.equal(n.plans[0].status, 'draft'); assert.deepEqual(Object.values(n.plans[0].checklist), [false, false, false, false]);
  assert.equal(n.plans[0].reviewFingerprint, undefined); assert.equal(n.plans[0].reviewedAt, undefined);
  assert.equal(JSON.stringify(s), original);
  assert.throws(() => M.updatePlan(s, 'p', { status: 'executed' }), /字段/);
  assert.throws(() => M.updatePlan(s, 'p', { quantity: -1 }), /数量/);
  const done = M.executePlan(s, 'p', { quantity: 1, price: 100, fee: 0, date: today });
  assert.throws(() => M.updatePlan(done, 'p', { price: 90 }), /只能修改/);
});

test('partial execution preserves original plan and creates its unfilled remainder only once', () => {
  const s = fixture();
  const partial = M.executePlan(s, 'p', { quantity: 0.5, price: 95, fee: 1, date: today });
  close(partial.plans[0].execution.remainingQuantity, 1.5);
  const n = M.makeRemainingPlan(partial, 'p', 'remaining');
  assert.equal(n.plans.length, 2); assert.equal(n.plans[1].status, 'draft'); assert.equal(n.plans[1].sourcePlanId, 'p');
  close(n.plans[1].quantity, 1.5); close(n.plans[1].fee, 0);
  assert.deepEqual(n.plans[0], partial.plans[0]); assert.deepEqual(n.holdings, partial.holdings); assert.equal(n.settings.cash, partial.settings.cash);
  assert.throws(() => M.makeRemainingPlan(n, 'p', 'remaining-2'), /重复/);
  assert.equal(M.validateState(n).valid, true);
  const full = M.executePlan(s, 'p', { quantity: 2, price: 95, fee: 1, date: today });
  assert.throws(() => M.makeRemainingPlan(full, 'p', 'extra'), /没有剩余/);
  const wrong = structuredClone(n); wrong.plans[1].sourcePlanId = 'missing'; assert.equal(M.validateState(wrong).valid, false);
});


test('archived holdings cannot hide positions or unfinished plans and must be restored for trading', () => {
  const s = fixture(); s.holdings[0].archived = true;
  assert.equal(M.validateState(s).valid, false);
  s.holdings[0].quantity = 0; assert.equal(M.validateState(s).valid, false);
  s.plans[0].status = 'cancelled'; assert.equal(M.validateState(s).valid, true);
  const plan = { ...s.plans[0], status: 'draft', side: 'buy', quantity: 1 };
  assert.equal(M.simulatePlan(s, plan).valid, false);
  s.holdings[0].archived = 'true'; assert.equal(M.validateState(s).valid, false);
  delete s.holdings[0].archived; assert.equal(M.validateState(s).valid, true);
});

test('optional event follow-up and update timestamps preserve legacy compatibility', () => {
  const s = fixture();
  assert.equal(M.validateState(s).valid, true);
  s.events[0].followUp = '2024-02-29'; s.events[0].updatedAt = '2026-11-16T01:02:03.000Z';
  assert.equal(M.validateState(s).valid, true);
  s.events[0].followUp = ''; assert.equal(M.validateState(s).valid, true);
  s.events[0].followUp = '2025-02-29'; assert.equal(M.validateState(s).valid, false);
  s.events[0].followUp = ''; s.events[0].updatedAt = '2026-02-30T01:02:03.000Z'; assert.equal(M.validateState(s).valid, false);
  s.events[0].updatedAt = '2026-11-16'; assert.equal(M.validateState(s).valid, false);
});

test('remainder uses original share units and cannot be recreated after a split', () => {
  const s = fixture();
  const partial = M.executePlan(s, 'p', { quantity: 0.5, price: 95, fee: 1, date: today });
  const split = M.applySplit(partial, { id: 'split', date: today, holdingId: 'h', ratio: 2, note: 'verified split' });
  assert.throws(() => M.makeRemainingPlan(split, 'p', 'remaining'), /数量口径已变化/);
});


test('defensive theme concentration is checked and target stress warnings are independent of current holdings', () => {
  const s = fixture(); s.holdings[0].theme = '防守'; s.holdings[0].bucket = 'defense'; s.settings.themeLimit = 40;
  assert.ok(M.portfolio(s).alerts.some(a => a.title === '防守 主题集中度超限'));
  const empty = M.createState(); empty.settings.cash = 1000;
  const p = M.portfolio(empty); close(p.stressPct, 0); close(p.targetStressPct, 34.5);
  assert.ok(p.alerts.some(a => a.title === '目标配置在当前情景下也超预算'));
  assert.equal(empty.settings.targetGrowth, 60, 'target remains unchanged');
  empty.settings.defenseShock = 0;
  assert.equal(M.portfolio(empty).alerts.some(a => a.title === '目标配置在当前情景下也超预算'), false);
});

test('event follow-up alerts count due unreviewed events only', () => {
  const s = fixture();
  s.events = [
    { ...s.events[0], id: 'due', followUp: today, reviewed: false },
    { ...s.events[0], id: 'older', followUp: '2020-01-01', reviewed: false },
    { ...s.events[0], id: 'done', followUp: today, reviewed: true },
    { ...s.events[0], id: 'future', followUp: '2099-01-01', reviewed: false },
    { ...s.events[0], id: 'blank', followUp: '', reviewed: false },
  ];
  assert.ok(M.portfolio(s).alerts.some(a => a.title === '2 条事件已到跟进日期'));
  assert.equal(M.validateState(s).valid, true);
});
