import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicPositionManagement } from "@/lib/api";
import { ResearchPanel } from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type {
  PositionAdviceItem,
  PositionManagementAction,
  PositionManagementSnapshot,
} from "@/lib/types";
import "./position-management-dashboard.css";

type Props = {
  onClose: () => void;
  onAsk?: (message: string) => void;
};
type Filter = "all" | PositionManagementAction;

const FILTERS: Array<{ id: Filter; label: string }> = [
  { id: "all", label: "全部" },
  { id: "review", label: "立即复核" },
  { id: "reduce", label: "降低暴露" },
  { id: "increase_candidate", label: "加仓候选" },
  { id: "hold", label: "持有" },
  { id: "insufficient_data", label: "数据不足" },
];

function statusLabel(status: string) {
  if (status === "live") return "当日证据完整";
  if (status === "partial") return "部分持仓证据不足";
  if (status === "data_unavailable") return "当前证据不足";
  if (status === "no_portfolio") return "尚无持仓";
  if (status === "waiting_refresh") return "等待首次生成";
  if (status === "portfolio_changed") return "持仓已变化";
  if (status === "stale") return "上次成功快照";
  return "等待数据";
}

function concentrationLabel(level: string) {
  if (level === "high") return "集中度高";
  if (level === "elevated") return "集中度偏高";
  if (level === "balanced") return "结构相对均衡";
  return "待评估";
}

function formatPrice(value?: number | null) {
  return value == null ? "—" : `$${value.toLocaleString("en-US", { maximumFractionDigits: 2 })}`;
}

function savedReportContext(snapshot: PositionManagementSnapshot, question: string) {
  const compact = {
    reportDate: snapshot.report_date,
    generatedAtLocal: snapshot.generated_at_local,
    status: snapshot.status,
    frameworkVersion: snapshot.framework_version,
    macro: snapshot.macro_context,
    concentration: snapshot.concentration,
    summary: snapshot.summary,
    items: snapshot.items.map((item) => ({
      symbol: item.symbol,
      weight: item.weight,
      action: item.action,
      confidence: item.confidence,
      ratingScore: item.rating_score,
      ratingStatus: item.rating_status,
      valuationPosition: item.valuation_position,
      newsImpact: item.news_impact,
      rationale: item.rationale,
      risks: item.risks,
      falsifiers: item.falsifiers,
      frameworkLogic: item.framework_logic,
      evidenceAsOf: item.evidence_as_of,
      evidenceSources: item.evidence_sources,
    })),
  };
  return buildSavedReportPrompt({
    marker: "HONE_SAVED_POSITION_MANAGEMENT_REPORT",
    payload: compact,
    subject: `已保存的仓位管理建议（报告日 ${snapshot.report_date}）`,
    question,
    rules: [
      "严格区分当前事实、Hari 逻辑、HONE 集中度规则和推断",
      "不得补造缺失行情、财务、估值或新闻",
      "先说结论，再说明反方与证伪条件",
      "不得自动修改持仓或声称已经下单",
    ],
  });
}

export function PositionManagementPanel(props: Props) {
  const [snapshot, setSnapshot] = createSignal<PositionManagementSnapshot>();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [filter, setFilter] = createSignal<Filter>("all");
  const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setSnapshot(await getPublicPositionManagement(controller.signal));
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "仓位建议暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  // Panel mounts only while open, so loading on mount means "fetch when the
  // panel is opened" — one cached snapshot read, no recomputation.
  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const visible = createMemo(() => {
    const items = snapshot()?.items ?? [];
    return filter() === "all" ? items : items.filter((item) => item.action === filter());
  });

  const sendQuestion = () => {
    const current = snapshot();
    const text = question().trim();
    if (!current || !text || !props.onAsk) return;
    props.onAsk(savedReportContext(current, text));
    setQuestion("");
    props.onClose();
  };

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="position-management-title"
      backdropClass="position-management-backdrop"
      dialogClass="position-management-dialog"
    >
      <>
        <header>
          <div>
            <p>HARI 仓位纪律</p>
            <h2 id="position-management-title">仓位管理建议</h2>
            <span>组合结构 × 公司质量 × 当日估值 × 宏观与重点新闻 · 每日 20:00 更新</span>
          </div>
          <button type="button" aria-label="关闭仓位建议" onClick={() => props.onClose()}>
            <svg viewBox="0 0 24 24"><path d="M18 6 6 18M6 6l12 12" /></svg>
          </button>
        </header>

        <Show when={snapshot()}>
          {(value) => (
            <div class="position-management-meta">
              <b class={`is-${value().status}`}>{statusLabel(value().status)}</b>
              <span>报告日 {value().report_date}</span>
              <span>{value().generated_at_local} {value().timezone}</span>
              <span>{value().framework_version}</span>
              <button type="button" onClick={() => void load()} disabled={loading()}>{loading() ? "读取中…" : "重新读取"}</button>
            </div>
          )}
        </Show>

        <div class="position-management-overview">
          <div><strong>{snapshot()?.total_weight.toFixed(1) ?? "0.0"}%</strong><span>已配置仓位</span></div>
          <div><strong>{snapshot()?.unallocated_weight.toFixed(1) ?? "100.0"}%</strong><span>未分配</span></div>
          <div class={`is-${snapshot()?.concentration.level ?? "unknown"}`}><strong>{concentrationLabel(snapshot()?.concentration.level ?? "unknown")}</strong><span>最大 {snapshot()?.concentration.largest_symbol || "—"} {snapshot()?.concentration.largest_weight.toFixed(1) ?? "0.0"}%</span></div>
          <div><strong>{snapshot()?.macro_context.signal === "green" ? "绿灯" : snapshot()?.macro_context.signal === "red" ? "红灯" : snapshot()?.macro_context.signal === "yellow" ? "黄灯" : "待定"}</strong><span>宏观 {snapshot()?.macro_context.score?.toFixed(1) ?? "—"}</span></div>
        </div>

        <div class="position-management-summary">
          <div class="is-review"><strong>{snapshot()?.counts.review ?? 0}</strong><span>立即复核</span></div>
          <div class="is-reduce"><strong>{snapshot()?.counts.reduce ?? 0}</strong><span>降低暴露</span></div>
          <div class="is-increase_candidate"><strong>{snapshot()?.counts.increase_candidate ?? 0}</strong><span>加仓候选</span></div>
          <div class="is-hold"><strong>{snapshot()?.counts.hold ?? 0}</strong><span>持有</span></div>
          <p>{snapshot()?.summary ?? (error() || "正在读取当天快照…")}</p>
        </div>

        <div class="position-management-filters" role="group" aria-label="按仓位建议筛选">
          <For each={FILTERS}>{(item) => (
            <button type="button" classList={{ "is-active": filter() === item.id }} aria-pressed={filter() === item.id} onClick={() => setFilter(item.id)}>{item.label}</button>
          )}</For>
        </div>

        <div class="position-management-body">
          <Show when={loading() && !snapshot()}>
            <ResearchState
              kind="loading"
              message="正在读取仓位建议"
              detail="只读取当日快照，不会触发重新计算。"
            />
          </Show>
          <Show when={error() && !snapshot()}>
            <ResearchState
              kind="error"
              message="仓位建议读取失败"
              detail={error()}
              onRetry={() => void load()}
            />
          </Show>
          <Show when={snapshot()}>
            <Show
              when={visible().length > 0}
              fallback={
                <ResearchState
                  kind="empty"
                  message={snapshot()?.status === "no_portfolio" ? "先添加持仓" : "当前没有可展示的仓位建议"}
                  detail={snapshot()?.summary}
                />
              }
            >
              <div class="position-management-list">
                <For each={visible()}>{(item: PositionAdviceItem) => (
                  <details class={`is-${item.action}`} open={item.action === "review" || item.action === "reduce"}>
                    <summary>
                      <span class={`position-management-action is-${item.action}`}>{item.action_label}</span>
                      <span><strong>{item.symbol}</strong><small>{item.name} · {item.theme}</small></span>
                      <span><strong>{item.weight.toFixed(1)}%</strong><small>当前仓位</small></span>
                      <span><strong>{item.rating_score?.toFixed(1) ?? "—"}</strong><small>当日评级</small></span>
                      <i>⌄</i>
                    </summary>
                    <div class="position-management-detail">
                      <div class="position-management-facts">
                        <span>现价 {formatPrice(item.current_price)}</span>
                        <span>成本 {formatPrice(item.avg_cost)}</span>
                        <span>浮动 {item.unrealized_return_percent == null ? "—" : `${item.unrealized_return_percent > 0 ? "+" : ""}${item.unrealized_return_percent.toFixed(1)}%`}</span>
                        <span>估值 {item.valuation_position}</span>
                        <span>新闻 {item.news_attention}</span>
                        <span>置信度 {item.confidence === "high" ? "高" : item.confidence === "medium" ? "中" : "低"}</span>
                      </div>
                      <section><h3>为什么这样判断</h3><ul><For each={item.rationale}>{(value) => <li>{value}</li>}</For></ul></section>
                      <div class="position-management-columns">
                        <section><h3>主要风险</h3><ul><For each={item.risks.length ? item.risks : ["当前无新增可核实风险项"]}>{(value) => <li>{value}</li>}</For></ul></section>
                        <section><h3>证伪条件</h3><ul><For each={item.falsifiers.length ? item.falsifiers : ["当前证据不足，需补充公司级证伪条件"]}>{(value) => <li>{value}</li>}</For></ul></section>
                      </div>
                      <p class="position-management-evidence">逻辑 {item.framework_logic.join(" / ")} · 数据日期 {item.evidence_as_of.join("、") || "待补充"} · 来源 {item.evidence_sources.join("、") || "待补充"}</p>
                    </div>
                  </details>
                )}</For>
              </div>
            </Show>
          </Show>
        </div>

        <Show when={props.onAsk}>
          <footer>
            <p class="position-management-method">{snapshot()?.methodology_note}</p>
            <div><input value={question()} onInput={(event) => setQuestion(event.currentTarget.value)} onKeyDown={(event) => event.key === "Enter" && sendQuestion()} placeholder="基于这份仓位建议继续提问…" aria-label="基于仓位建议提问" /><button type="button" disabled={!question().trim() || !snapshot()} onClick={sendQuestion}>发送到对话</button></div>
            <p>{snapshot()?.disclaimer ?? "仓位建议仅供研究与复核，不构成投资建议。"}</p>
          </footer>
        </Show>
      </>
    </ResearchPanel>
  );
}
