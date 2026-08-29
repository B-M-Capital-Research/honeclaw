import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicCompanyRatings } from "@/lib/api";
import {
  coverageLabel,
  filterRatings,
  ratingCounts,
  type CompanyRatingFilter,
} from "@/lib/company-rating-model";
import type { CompanyRating, CompanyRatingSnapshot } from "@/lib/types";
import "./company-rating-dashboard.css";

const FILTER_LABELS: Record<CompanyRatingFilter, string> = {
  all: "全部",
  green: "绿灯",
  yellow: "黄灯",
  red: "红灯",
  unknown: "研究基线",
};

const DIMENSIONS = [
  ["商业/护城河", "moat"],
  ["产品/地位", "scarcity"],
  ["增长质量", "growth_quality"],
  ["定价/毛利", "pricing_power"],
  ["盈利/财务", "financial_quality"],
  ["前瞻能见度", "visibility"],
  ["估值", "valuation"],
  ["时点确认", "timing"],
] as const;

function formatPrice(value?: number | null) {
  if (value == null) return "—";
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: value >= 100 ? 0 : 2,
  }).format(value);
}

function statusLabel(status: string) {
  if (status === "simulation") return "Codex 模拟预览";
  if (status === "live") return "行情 + 财报已更新";
  if (status === "partial") return "部分数据已更新";
  if (status === "stale") return "上次成功快照";
  return "演讲研究基线";
}

function scoreBasisLabel(item: CompanyRating) {
  const coverage = item.factor_coverage ?? (item.valuation ? 8 : 3);
  if (item.data_status === "simulation") return `${coverage}/8 因子 · Codex 模拟`;
  if (item.valuation) return `${coverage}/8 因子 · 含当日估值`;
  if (item.data_status === "transcript_only") return `${coverage}/8 因子 · 研究基线`;
  return `${coverage}/8 因子 · 估值待核验`;
}

function scoreDisplay(item: CompanyRating) {
  if (item.light === "unknown" || item.data_status === "transcript_only") {
    return {
      value: "—",
      basis: `研究结构分 ${item.score.toFixed(1)} · ${item.factor_coverage ?? 3}/8 因子`,
    };
  }
  return { value: item.score.toFixed(1), basis: scoreBasisLabel(item) };
}

function metric(value?: number | null, suffix = "%") {
  if (value == null) return "—";
  return `${value > 0 ? "+" : ""}${value.toFixed(1)}${suffix}`;
}

function financialBasisSummary(item: CompanyRating) {
  const claims = item.metrics?.financial_source_claims ?? [];
  const standards = [...new Set(claims.map((claim) => claim.metric_basis.split(":", 1)[0]))];
  const units = [...new Set(claims.map((claim) => claim.unit))];
  return [
    standards.length > 0 ? `会计口径 ${standards.join(" / ")}` : undefined,
    units.length > 0 ? `原始单位 ${units.join(" / ")}` : undefined,
  ].filter(Boolean).join(" · ");
}

function FundamentalMetrics(props: { item: CompanyRating }) {
  const values = () => props.item.metrics ?? {};
  const reviewOnly = () =>
    Boolean(values().financial_as_of) && values().financial_score_eligible !== true;
  return (
    <section class="company-rating-fundamentals">
      <h3>财务兑现数据</h3>
      <dl>
        <div><dt>营收同比</dt><dd>{metric(values().revenue_growth_percent)}</dd></div>
        <div><dt>前瞻营收</dt><dd>{metric(values().forward_revenue_growth_percent)}</dd></div>
        <div><dt>毛利率</dt><dd>{metric(values().gross_margin_percent)}</dd></div>
        <div><dt>毛利变化</dt><dd>{metric(values().gross_margin_change_pp, " 个百分点")}</dd></div>
        <div><dt>经营利润率</dt><dd>{metric(values().ebit_margin_percent)}</dd></div>
        <div><dt>自由现金流率</dt><dd>{metric(values().fcf_margin_percent)}</dd></div>
        <div><dt>净现金/营收</dt><dd>{metric(values().net_cash_to_revenue_percent)}</dd></div>
        <div><dt>应收同比</dt><dd>{metric(values().accounts_receivable_growth_percent)}</dd></div>
        <div><dt>应付同比</dt><dd>{metric(values().accounts_payable_growth_percent)}</dd></div>
        <div><dt>库存同比</dt><dd>{metric(values().inventory_growth_percent)}</dd></div>
        <div><dt>固定资产同比</dt><dd>{metric(values().property_plant_equipment_growth_percent)}</dd></div>
        <div><dt>经营现金流同比</dt><dd>{metric(values().operating_cash_flow_growth_percent)}</dd></div>
        <div><dt>资本开支同比</dt><dd>{metric(values().capital_expenditure_growth_percent)}</dd></div>
        <div><dt>自由现金流同比</dt><dd>{metric(values().free_cash_flow_growth_percent)}</dd></div>
        <Show when={values().forward_metric_label}>
          <div><dt>{values().forward_metric_label}</dt><dd>{values().forward_metric_value ?? "—"}<Show when={values().forward_metric_growth_percent != null}> · {metric(values().forward_metric_growth_percent)}</Show></dd></div>
        </Show>
      </dl>
      <small classList={{ "is-review-only": reviewOnly() }}>
        {values().financial_as_of
          ? reviewOnly()
            ? `SEC 财务证据截至 ${values().financial_as_of} · 待人工质量复核，暂不计分`
            : `财务截至 ${values().financial_as_of} · 已通过评分门槛`
          : "未取得通过日期与来源校验的财务数据，相关因子不计分"}
      </small>
      <Show when={financialBasisSummary(props.item)}>
        <small class="company-rating-fundamentals__basis">{financialBasisSummary(props.item)}</small>
      </Show>
      <Show when={(values().financial_quality_warnings?.length ?? 0) > 0}>
        <ul class="company-rating-fundamentals__warnings">
          <For each={values().financial_quality_warnings}>{(warning) => <li>{warning}</li>}</For>
        </ul>
      </Show>
      <Show when={
        (values().financial_calculations?.length ?? 0) > 0
        || (values().financial_source_claims?.length ?? 0) > 0
      }>
        <details class="company-rating-fundamentals__trace">
          <summary>查看逐项口径、可复算过程与 SEC 来源</summary>
          <ul>
            <For each={values().financial_calculations}>{(calculation) => <li>{calculation}</li>}</For>
          </ul>
          <div class="company-rating-fundamentals__claims">
            <For each={values().financial_source_claims ?? []}>
              {(claim) => (
                <p>
                  <strong>{claim.metric_id}</strong> · {claim.period} · {claim.numeric_value.toLocaleString()} {claim.unit}<br />
                  <small>{claim.metric_basis} · 发布 {new Date(claim.published_at).toLocaleString("zh-CN", { hour12: false })}</small><br />
                  <a href={claim.source_url} target="_blank" rel="noreferrer">对应 SEC 原文</a>
                </p>
              )}
            </For>
          </div>
          <p>
            <For each={values().financial_source_urls}>
              {(url, index) => <><a href={url} target="_blank" rel="noreferrer">SEC 原文 {index() + 1}</a>{" "}</>}
            </For>
          </p>
        </details>
      </Show>
      <Show when={values().forward_metric_as_of}>
        <small>前瞻指标截至 {values().forward_metric_as_of} · 仅已核验增速调整能见度分</small>
      </Show>
    </section>
  );
}

function valuationSyncMessage(snapshot: CompanyRatingSnapshot) {
  const { companies, quotes, financials, valuations } = snapshot.coverage;
  if (snapshot.data_status === "simulation") {
    return `当前 ${companies} 家均使用 Codex 情景模拟填充八因子；模拟估值分不生成虚构目标价，也不会写成真实财报。`;
  }
  if (valuations === companies && companies > 0) {
    return `当日估值已同步 ${valuations}/${companies} 家。`;
  }
  if (quotes === 0 && financials === 0) {
    return `当日估值 ${valuations}/${companies}：本次没有取得行情与财务输入，因此没有生成或沿用旧目标价。`;
  }
  return `当日估值 ${valuations}/${companies}：其余公司未同时通过数据日期、至少两种方法和方法离散度门槛。`;
}

function financialSyncMessage(snapshot: CompanyRatingSnapshot) {
  const { companies, financials } = snapshot.coverage;
  const observed = snapshot.coverage.financial_observations ?? financials;
  const reviewRequired = snapshot.coverage.financials_review_required ?? 0;
  if (snapshot.data_status === "simulation") {
    return "当前为模拟财务因子，不代表真实财报。";
  }
  return `财务证据 ${observed}/${companies}：其中 ${financials} 家已通过评分门槛，${reviewRequired} 家仅展示 SEC 点时证据、等待人工质量复核。`;
}

function formatValuation(value: number, currency: string) {
  if (currency === "USD") return formatPrice(value);
  return `${value.toFixed(2)} ${currency}`;
}

function ValuationPanel(props: { item: CompanyRating }) {
  const marker = () => Math.max(0, Math.min(100, props.item.valuation?.position_percent ?? 0));
  return (
    <section class="company-rating-valuation">
      <h3>当日估值</h3>
      <Show
        when={props.item.valuation}
        fallback={
          <div class="company-rating-valuation__empty">
            <strong>{props.item.data_status === "simulation" ? `Codex 模拟估值分 ${Math.round(props.item.dimensions.valuation ?? 0)}` : "今日不计估值分"}</strong>
            <p>{props.item.valuation_unavailable_reason}</p>
            <small>方法框架：{props.item.valuation_method}</small>
          </div>
        }
      >
        {(valuation) => (
          <>
            <dl class="company-rating-valuation__cases">
              <div><dt>悲观</dt><dd>{formatValuation(valuation().bear_case, valuation().currency)}</dd></div>
              <div><dt>基准</dt><dd>{formatValuation(valuation().base_case, valuation().currency)}</dd></div>
              <div><dt>乐观</dt><dd>{formatValuation(valuation().bull_case, valuation().currency)}</dd></div>
              <div><dt>当前</dt><dd>{formatValuation(valuation().current_price, valuation().currency)}</dd></div>
              <div><dt>概率加权</dt><dd>{formatValuation(valuation().probability_weighted_value, valuation().currency)}</dd></div>
              <div><dt>预期空间</dt><dd>{valuation().expected_upside_percent >= 0 ? "+" : ""}{valuation().expected_upside_percent.toFixed(1)}%</dd></div>
            </dl>
            <div class="company-rating-valuation__range" aria-label={`当前位于${valuation().current_position}`}>
              <i /><b style={{ left: `${marker()}%` }} />
            </div>
            <p class="company-rating-valuation__position">
              {valuation().current_position} · 区间位置 {valuation().position_percent.toFixed(1)}%
            </p>
            <p>{valuation().method} · {valuation().method_count} 种方法 · {valuation().confidence === "high" ? "高" : "中"}置信度</p>
            <small>估值时间 {valuation().generated_at_beijing} 北京时间 · 数据日 {valuation().as_of}</small>
          </>
        )}
      </Show>
    </section>
  );
}

export function CompanyRatingDashboard() {
  const [open, setOpen] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [snapshot, setSnapshot] = createSignal<CompanyRatingSnapshot>();
  const [filter, setFilter] = createSignal<CompanyRatingFilter>("all");
  const [query, setQuery] = createSignal("");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setSnapshot(await getPublicCompanyRatings(controller.signal));
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "评级暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  createEffect(() => {
    if (!open()) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  const counts = createMemo(() => ratingCounts(snapshot()?.items ?? []));
  const visible = createMemo(() =>
    filterRatings(snapshot()?.items ?? [], filter(), query()),
  );

  return (
    <>
      <div class="company-rating-launcher">
        <button
          type="button"
          class="company-rating-launcher__button"
          aria-haspopup="dialog"
          onClick={() => setOpen(true)}
        >
          <span class="company-rating-launcher__icon" aria-hidden="true">
            <i class="is-green" />
            <i class="is-yellow" />
            <i class="is-red" />
          </span>
          <span class="company-rating-launcher__copy">
            <strong>每日公司评级</strong>
            <small>
              <Show when={snapshot()} fallback={loading() ? "正在读取…" : "查看研究评分"}>
                {(value) => coverageLabel(value())}
              </Show>
            </small>
          </span>
          <Show when={snapshot()}>
            <span class="company-rating-launcher__counts" aria-label="评级数量">
              <Show
                when={counts().unknown < (snapshot()?.items.length ?? 0)}
                fallback={<b class="is-unknown">{counts().unknown} 基线</b>}
              >
                <b class="is-green">{counts().green}</b>
                <b class="is-yellow">{counts().yellow}</b>
                <b class="is-red">{counts().red}</b>
              </Show>
            </span>
          </Show>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="m9 18 6-6-6-6" />
          </svg>
        </button>
      </div>

      <Show when={open()}>
        <Portal>
          <div class="company-rating-backdrop" onClick={() => setOpen(false)}>
            <section
              class="company-rating-dialog"
              role="dialog"
              aria-modal="true"
              aria-labelledby="company-rating-title"
              onClick={(event) => event.stopPropagation()}
            >
              <header class="company-rating-dialog__header">
                <div>
                  <p class="company-rating-eyebrow">HONE 研究信号</p>
                  <h2 id="company-rating-title">每日公司评级</h2>
                  <p>
                    结构质量、财务兑现、前瞻能见度、估值与时点确认的八因子研究分；缺失数据不填零，也不自动给中性分。
                  </p>
                </div>
                <button type="button" aria-label="关闭评级" onClick={() => setOpen(false)}>
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M18 6 6 18M6 6l12 12" />
                  </svg>
                </button>
              </header>

              <Show when={snapshot()}>
                {(value) => (
                  <div class="company-rating-meta">
                    <span class={`is-${value().data_status}`}>
                      {coverageLabel(value())}
                    </span>
                    <span>北京时间 {value().generated_at_beijing} 更新</span>
                    <button type="button" disabled={loading()} onClick={() => void load()}>
                      {loading() ? "刷新中…" : "重新读取"}
                    </button>
                  </div>
                )}
              </Show>

              <Show when={snapshot()?.data_status === "simulation"}>
                <div class="company-rating-simulation-note">
                  <strong>模拟预览</strong>
                  <span>{snapshot()?.simulation_note}</span>
                </div>
              </Show>

              <Show when={snapshot()}>
                {(value) => (
                  <details class="company-rating-methodology">
                    <summary>
                      <span>评分标准与估值同步</span>
                      <strong>
                        {value().data_status === "simulation"
                          ? `${value().coverage.valuations}/${value().coverage.companies} 家含模拟估值因子`
                          : `${value().coverage.valuations}/${value().coverage.companies} 家含当日估值`}
                      </strong>
                      <i aria-hidden="true">⌄</i>
                    </summary>
                    <div>
                      <section>
                        <h3>统一标尺</h3>
                        <p>演讲研究材料中的 1–5 档依次映射为 25 / 45 / 60 / 75 / 90。5 档代表“现有证据中的最高档”，不是完美公司；研究基线不再直接显示 100 分。</p>
                      </section>
                      <section>
                        <h3>综合分公式</h3>
                        <p>商业模式与护城河 20% + 产品能力与市场地位 10% + 增长质量 15% + 定价权与毛利趋势 10% + 盈利和财务健康 15% + 前瞻能见度 10% + 当日估值 15% + 时点确认 5%。</p>
                      </section>
                      <section>
                        <h3>可比与降档</h3>
                        <p>财务因子采用“60% 绝对门槛 + 40% 同主题可比排名”；同主题样本不足 5 家时使用全覆盖公司。缺失因子移除权重并显示覆盖数；财务质量进入高风险区会封顶红灯，护城河、产品地位或估值严重不足会封顶黄灯。</p>
                      </section>
                      <section>
                        <h3>当前同步状态</h3>
                        <p>{financialSyncMessage(value())}</p>
                        <p>{valuationSyncMessage(value())}</p>
                      </section>
                    </div>
                  </details>
                )}
              </Show>

              <div class="company-rating-toolbar">
                <label>
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <circle cx="11" cy="11" r="7" />
                    <path d="m20 20-3.6-3.6" />
                  </svg>
                  <input
                    type="search"
                    value={query()}
                    onInput={(event) => setQuery(event.currentTarget.value)}
                    placeholder="搜索公司、Ticker 或主题"
                    aria-label="搜索公司评级"
                  />
                </label>
                <div class="company-rating-filters" role="group" aria-label="按红绿灯筛选">
                  <For each={["all", "green", "yellow", "red", "unknown"] as CompanyRatingFilter[]}>
                    {(item) => (
                      <button
                        type="button"
                        classList={{ "is-active": filter() === item, [`is-${item}`]: item !== "all" }}
                        aria-pressed={filter() === item}
                        onClick={() => setFilter(item)}
                      >
                        {FILTER_LABELS[item]}
                        <Show when={item !== "all"}>
                          <span>{counts()[item as keyof ReturnType<typeof counts>]}</span>
                        </Show>
                      </button>
                    )}
                  </For>
                </div>
              </div>

              <div class="company-rating-dialog__body">
                <Show when={!loading() || snapshot()} fallback={<div class="company-rating-state">正在生成评级列表…</div>}>
                  <Show when={!error()} fallback={<div class="company-rating-state is-error">{error()}<button type="button" onClick={() => void load()}>重试</button></div>}>
                    <Show when={visible().length} fallback={<div class="company-rating-state">没有符合筛选条件的公司</div>}>
                      <div class="company-rating-list">
                        <div class="company-rating-list__head" aria-hidden="true">
                          <span>公司</span><span>当前数据</span><span>正式评级 / 研究分</span>
                        </div>
                        <For each={visible()}>
                          {(item) => (
                            <details class={`company-rating-row is-${item.light}`}>
                              <summary>
                                <span class={`company-rating-light is-${item.light}`} aria-label={`${FILTER_LABELS[item.light]}`} />
                                <span class="company-rating-company">
                                  <strong>{item.symbol}</strong>
                                  <small>{item.name} · {item.theme}</small>
                                </span>
                                <span class="company-rating-market">
                                  <strong>{formatPrice(item.price)}</strong>
                                  <small classList={{ "is-up": (item.change_percent ?? 0) > 0, "is-down": (item.change_percent ?? 0) < 0 }}>
                                    {item.change_percent == null ? statusLabel(item.data_status) : `${item.change_percent > 0 ? "+" : ""}${item.change_percent.toFixed(2)}%`}
                                  </small>
                                </span>
                                <span class="company-rating-score">
                                  <strong>{scoreDisplay(item).value}</strong>
                                  <small>{scoreDisplay(item).basis}</small>
                                </span>
                                <svg class="company-rating-chevron" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
                              </summary>
                              <div class="company-rating-detail">
                                <p class="company-rating-thesis">{item.thesis_summary}</p>
                                <Show when={item.score_cap_reason}>
                                  <p class="company-rating-cap"><strong>评级被降档：</strong>{item.score_cap_reason}</p>
                                </Show>
                                <div class="company-rating-dimensions">
                                  <For each={DIMENSIONS}>
                                    {([label, key]) => (
                                      <div>
                                        <span>{label}</span>
                                        <i classList={{ "is-unavailable": item.dimensions[key] == null }}><b style={{ width: `${item.dimensions[key] ?? 0}%` }} /></i>
                                        <strong>{item.dimensions[key] == null ? "—" : Math.round(item.dimensions[key] ?? 0)}</strong>
                                      </div>
                                    )}
                                  </For>
                                </div>
                                <div class="company-rating-detail__grid">
                                  <section><h3>护城河</h3><p>{item.moat}</p></section>
                                  <ValuationPanel item={item} />
                                  <FundamentalMetrics item={item} />
                                  <section><h3>重点跟踪</h3><ul><For each={item.watch_items}>{(text) => <li>{text}</li>}</For></ul></section>
                                  <section><h3>风险与证伪</h3><ul><For each={[...item.risks, ...item.falsifiers].slice(0, 4)}>{(text) => <li>{text}</li>}</For></ul></section>
                                </div>
                                <p class="company-rating-source">
                                  {statusLabel(item.data_status)} · 置信度 {item.confidence === "low" ? "低" : item.confidence === "high" ? "高" : "中"}
                                  <Show when={item.financial_as_of}> · 财务截至 {item.financial_as_of}</Show>
                                  <Show when={item.market_as_of}> · 行情截至 {item.market_as_of}</Show>
                                </p>
                              </div>
                            </details>
                          )}
                        </For>
                      </div>
                    </Show>
                  </Show>
                </Show>
              </div>
              <footer>{snapshot()?.disclaimer ?? "研究排序工具，不构成投资建议。"}</footer>
            </section>
          </div>
        </Portal>
      </Show>
    </>
  );
}
