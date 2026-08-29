import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import {
  createHistoricalStateReconstruction,
  getHistoricalDecisionAnchors,
  getHistoricalStateReconstructions,
  reviewHistoricalStateReconstruction,
} from "@/lib/api";
import type {
  HistoricalDecisionAnchorRegistry,
  HistoricalStateComponentId,
  HistoricalStateComponentStatus,
  HistoricalStateReconstructionRegistry,
  HistoricalStateReviewVerdict,
} from "@/lib/types";

type ComponentDraft = {
  status: HistoricalStateComponentStatus;
  sourceId: string;
  availableAt: string;
  locator: string;
  excerpt: string;
  claim: string;
  missingReason: string;
};

const COMPONENT_IDS: HistoricalStateComponentId[] = [
  "industry_thesis",
  "company_fundamentals",
  "financial_verification",
  "valuation",
  "crowding",
  "market_regime",
  "portfolio_context",
];

const COMPANY_COMPONENTS = new Set<HistoricalStateComponentId>([
  "company_fundamentals",
  "financial_verification",
  "valuation",
  "crowding",
]);

function emptyDrafts(): Record<HistoricalStateComponentId, ComponentDraft> {
  return Object.fromEntries(COMPONENT_IDS.map((id) => [id, {
    status: "explicitly_missing",
    sourceId: "",
    availableAt: "",
    locator: "",
    excerpt: "",
    claim: "",
    missingReason: "当时可用资料尚未恢复，保留为明确缺失。",
  }])) as Record<HistoricalStateComponentId, ComponentDraft>;
}

export function PublicAdminHistoricalStateReconstructionPanel() {
  const [registry, setRegistry] = createSignal<HistoricalStateReconstructionRegistry>();
  const [anchorRegistry, setAnchorRegistry] = createSignal<HistoricalDecisionAnchorRegistry>();
  const [anchorId, setAnchorId] = createSignal("");
  const [drafts, setDrafts] = createSignal(emptyDrafts());
  const [reviewId, setReviewId] = createSignal("");
  const [reviewVerdict, setReviewVerdict] = createSignal<HistoricalStateReviewVerdict>("approved_for_benchmark");
  const [reviewStatement, setReviewStatement] = createSignal("");
  const [checks, setChecks] = createSignal([false, false, false, false, false, false]);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    const [next, anchors] = await Promise.all([
      getHistoricalStateReconstructions(),
      getHistoricalDecisionAnchors(),
    ]);
    setRegistry(next);
    setAnchorRegistry(anchors);
    if (!anchorId() && next.confirmed_anchors.length) setAnchorId(next.confirmed_anchors[0].candidate_id);
    if (!reviewId() && next.reconstructions.length) setReviewId(next.reconstructions[0].candidate.reconstruction_id);
  };

  onMount(() => void load().catch((cause) => setError(cause instanceof Error ? cause.message : "历史点时状态读取失败")));

  const selectedAnchor = createMemo(() => registry()?.confirmed_anchors.find((item) => item.candidate_id === anchorId()));
  const selectedReconstruction = createMemo(() => registry()?.reconstructions.find((item) => item.candidate.reconstruction_id === reviewId()));

  const sourceOptions = (componentId: HistoricalStateComponentId) => {
    const symbol = selectedAnchor()?.symbol;
    const sources = anchorRegistry()?.sources ?? [];
    return COMPANY_COMPONENTS.has(componentId)
      ? sources.filter((source) => source.tickers.includes(symbol ?? ""))
      : sources;
  };

  const updateDraft = (componentId: HistoricalStateComponentId, patch: Partial<ComponentDraft>) => {
    setDrafts((current) => ({
      ...current,
      [componentId]: { ...current[componentId], ...patch },
    }));
  };

  const reconstructionReady = createMemo(() => {
    if (!selectedAnchor()) return false;
    return COMPONENT_IDS.every((id) => {
      const draft = drafts()[id];
      if (draft.status === "explicitly_missing") return Boolean(draft.missingReason.trim());
      return Boolean(
        draft.sourceId
        && draft.availableAt
        && draft.locator.trim()
        && draft.excerpt.trim()
        && draft.claim.trim(),
      );
    });
  });

  const submitReconstruction = async () => {
    const anchor = selectedAnchor();
    if (!anchor || !reconstructionReady()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const sources = anchorRegistry()?.sources ?? [];
      const candidate = await createHistoricalStateReconstruction({
        anchor_candidate_id: anchor.candidate_id,
        expected_anchor_candidate_sha256: anchor.candidate_sha256,
        expected_anchor_review_id: anchor.review_id,
        components: COMPONENT_IDS.map((componentId) => {
          const draft = drafts()[componentId];
          const source = sources.find((item) => item.source_item_id === draft.sourceId);
          return {
            component_id: componentId,
            status: draft.status,
            evidence: draft.status === "evidence_backed" && source ? [{
              source_item_id: source.source_item_id,
              expected_source_sha256: source.source_sha256,
              claimed_available_at: new Date(draft.availableAt).toISOString(),
              source_locator: draft.locator,
              verbatim_excerpt: draft.excerpt,
              normalized_claim: draft.claim,
            }] : [],
            missing_reason: draft.status === "explicitly_missing" ? draft.missingReason : undefined,
          };
        }),
      });
      setReviewId(candidate.reconstruction_id);
      await load();
      setNotice("点时重建候选已冻结；仍需逐项人工复核，且尚未生成未来收益标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存点时重建候选失败");
    } finally {
      setBusy(false);
    }
  };

  const submitReview = async () => {
    const item = selectedReconstruction();
    if (!item) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = checks();
      await reviewHistoricalStateReconstruction(item.candidate.reconstruction_id, {
        expected_review_id: item.latest_review?.review_id,
        verdict: reviewVerdict(),
        review_statement: reviewStatement(),
        anchor_binding_confirmed: values[0],
        source_bytes_confirmed: values[1],
        availability_times_confirmed: values[2],
        no_future_information_confirmed: values[3],
        missingness_preserved_confirmed: values[4],
        component_interpretations_confirmed: values[5],
      });
      setReviewStatement("");
      setChecks([false, false, false, false, false, false]);
      await load();
      setNotice("复核已写入不可覆盖记录；批准项仅成为历史基准状态，结果标签和训练仍关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存点时重建复核失败");
    } finally {
      setBusy(false);
    }
  };

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };

  return (
    <Show when={registry()}>{(value) => (
      <section class="public-admin-historical-anchors" aria-label="历史点时状态重建">
        <header>
          <div>
            <h3>历史点时状态重建</h3>
            <p>{value().scope}</p>
          </div>
          <span>未来数据隔离</span>
        </header>
        <div class="public-admin-decision-metrics">
          <div><span>已确认锚点</span><strong>{value().confirmed_anchor_count}</strong></div>
          <div><span>重建候选</span><strong>{value().reconstruction_candidate_count}</strong></div>
          <div><span>基准状态就绪</span><strong>{value().benchmark_ready_count}</strong></div>
          <div><span>上游已变化</span><strong>{value().stale_reconstruction_count}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">
          结果协议固定为 {value().outcome_protocol.horizons_market_sessions.join(" / ")} 个共同交易日、{value().outcome_protocol.benchmark_symbol} 基准和复权收盘价；自动重建关闭、结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭。
        </p>

        <Show when={value().confirmed_anchors.length === 0}>
          <p class="public-admin-decision-notice">先完成一条历史判断锚点的本人确认和精确可用时间，系统才允许建立点时状态。</p>
        </Show>

        <Show when={value().confirmed_anchors.length > 0}>
          <details class="public-admin-reward-governance">
            <summary>建立七层点时状态候选</summary>
            <label>
              <span>已确认历史判断</span>
              <select value={anchorId()} onChange={(event) => setAnchorId(event.currentTarget.value)}>
                <For each={value().confirmed_anchors}>{(anchor) => (
                  <option value={anchor.candidate_id}>{anchor.symbol} · {new Date(anchor.decision_available_at).toLocaleString("zh-CN")}</option>
                )}</For>
              </select>
            </label>
            <For each={value().required_components}>{(component) => {
              const draft = () => drafts()[component.component_id];
              return (
                <fieldset class="public-admin-anchor-component">
                  <legend>{component.label}</legend>
                  <p>{component.requirement}</p>
                  <label>
                    <span>当时状态</span>
                    <select value={draft().status} onChange={(event) => updateDraft(component.component_id, { status: event.currentTarget.value as HistoricalStateComponentStatus })}>
                      <option value="explicitly_missing">明确缺失，不补造</option>
                      <option value="evidence_backed">有完整原文证据</option>
                    </select>
                  </label>
                  <Show when={draft().status === "explicitly_missing"} fallback={<>
                    <label><span>完整资料来源</span><select value={draft().sourceId} onChange={(event) => updateDraft(component.component_id, { sourceId: event.currentTarget.value })}><option value="">请选择</option><For each={sourceOptions(component.component_id)}>{(source) => <option value={source.source_item_id}>{source.source_date} · {source.title}</option>}</For></select></label>
                    <label><span>当时可用时间</span><input type="datetime-local" value={draft().availableAt} onInput={(event) => updateDraft(component.component_id, { availableAt: event.currentTarget.value })} /></label>
                    <label><span>原文定位</span><input maxlength={160} value={draft().locator} onInput={(event) => updateDraft(component.component_id, { locator: event.currentTarget.value })} /></label>
                    <label class="is-wide"><span>逐字原文</span><textarea maxlength={2400} value={draft().excerpt} onInput={(event) => updateDraft(component.component_id, { excerpt: event.currentTarget.value })} /></label>
                    <label class="is-wide"><span>仅基于原文的点时事实归纳</span><textarea maxlength={1000} value={draft().claim} onInput={(event) => updateDraft(component.component_id, { claim: event.currentTarget.value })} /></label>
                  </>}>
                    <label class="is-wide"><span>当时为什么无法恢复</span><textarea maxlength={1000} value={draft().missingReason} onInput={(event) => updateDraft(component.component_id, { missingReason: event.currentTarget.value })} /></label>
                  </Show>
                </fieldset>
              );
            }}</For>
            <button type="button" class="public-admin-decision-submit" disabled={busy() || !reconstructionReady()} onClick={() => void submitReconstruction()}>
              冻结点时状态候选（不生成收益）
            </button>
          </details>
        </Show>

        <Show when={value().reconstructions.length > 0}>
          <details class="public-admin-reward-governance">
            <summary>人工复核点时状态</summary>
            <label><span>重建候选</span><select value={reviewId()} onChange={(event) => setReviewId(event.currentTarget.value)}><For each={value().reconstructions}>{(item) => <option value={item.candidate.reconstruction_id}>{item.candidate.symbol} · {new Date(item.candidate.decision_available_at).toLocaleString("zh-CN")} · {item.latest_review?.verdict ?? "待复核"}</option>}</For></select></label>
            <label><span>复核结论</span><select value={reviewVerdict()} onChange={(event) => setReviewVerdict(event.currentTarget.value as HistoricalStateReviewVerdict)}><option value="approved_for_benchmark">批准为历史基准状态</option><option value="changes_requested">要求修订</option><option value="rejected">拒绝</option></select></label>
            <label><span>复核说明</span><textarea maxlength={1200} value={reviewStatement()} onInput={(event) => setReviewStatement(event.currentTarget.value)} /></label>
            <Show when={reviewVerdict() === "approved_for_benchmark"}>
              <For each={["锚点和动作绑定准确", "完整来源字节与摘录已核对", "每条证据的当时可用时间已核对", "没有混入未来信息", "缺失项没有被补造", "七层事实归纳与原文一致"]}>{(label, index) => <label class="public-admin-reward-confirm"><input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} /> {label}</label>}</For>
            </Show>
            <button type="button" class="public-admin-decision-submit" disabled={busy() || !selectedReconstruction() || !reviewStatement().trim() || (reviewVerdict() === "approved_for_benchmark" && checks().some((item) => !item))} onClick={() => void submitReview()}>
              写入不可覆盖的点时复核
            </button>
          </details>
        </Show>
        <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
      </section>
    )}</Show>
  );
}
