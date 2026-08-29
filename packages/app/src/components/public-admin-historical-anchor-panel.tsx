import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import {
  createHistoricalDecisionAnchorCandidate,
  getHistoricalAnchorDiscovery,
  getHistoricalDecisionAnchors,
  reviewHistoricalDecisionAnchor,
  screenHistoricalAnchorDiscovery,
} from "@/lib/api";
import type {
  HistoricalAnchorDiscoveryScreeningVerdict,
  HistoricalDecisionAnchorAction,
  HistoricalAnchorDiscoveryResponse,
  HistoricalDecisionAnchorRegistry,
  HistoricalDecisionAnchorReviewVerdict,
} from "@/lib/types";

const ACTION_LABELS: Record<HistoricalDecisionAnchorAction, string> = {
  increase: "增加暴露",
  maintain: "维持",
  reduce: "减少暴露",
  exit: "退出",
  research_only: "仅研究",
};

const SCREENING_LABELS: Record<HistoricalAnchorDiscoveryScreeningVerdict, string> = {
  continue_candidate_review: "值得继续",
  not_decision_context: "不是判断语境",
  needs_more_context: "需要更多上下文",
};

export function PublicAdminHistoricalAnchorPanel() {
  const [registry, setRegistry] = createSignal<HistoricalDecisionAnchorRegistry>();
  const [discovery, setDiscovery] = createSignal<HistoricalAnchorDiscoveryResponse>();
  const [discoverySelection, setDiscoverySelection] =
    createSignal<"active_batch" | "shortlist" | "full_queue">("active_batch");
  const [suggestionId, setSuggestionId] = createSignal("");
  const [sourceId, setSourceId] = createSignal("");
  const [symbol, setSymbol] = createSignal("");
  const [locator, setLocator] = createSignal("");
  const [excerpt, setExcerpt] = createSignal("");
  const [action, setAction] = createSignal<HistoricalDecisionAnchorAction>("research_only");
  const [thesis, setThesis] = createSignal("");
  const [reviewCandidateId, setReviewCandidateId] = createSignal("");
  const [reviewVerdict, setReviewVerdict] =
    createSignal<HistoricalDecisionAnchorReviewVerdict>("confirmed");
  const [reviewStatement, setReviewStatement] = createSignal("");
  const [decisionAvailableAt, setDecisionAvailableAt] = createSignal("");
  const [sourceTimeConfirmed, setSourceTimeConfirmed] = createSignal(false);
  const [speakerConfirmed, setSpeakerConfirmed] = createSignal(false);
  const [noHindsightConfirmed, setNoHindsightConfirmed] = createSignal(false);
  const [screeningCorrectionReason, setScreeningCorrectionReason] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const selectedSource = createMemo(() =>
    registry()?.sources.find((source) => source.source_item_id === sourceId()),
  );
  const selectedCandidate = createMemo(() =>
    registry()?.anchors.find((item) => item.candidate.candidate_id === reviewCandidateId()),
  );
  const selectedSuggestion = createMemo(() =>
    discovery()?.suggestions.find((item) => item.suggestion_id === suggestionId()),
  );
  const visibleSuggestions = createMemo(() => {
    const found = discovery();
    if (!found) return [];
    if (discoverySelection() === "active_batch") return found.active_review_batch;
    if (discoverySelection() === "shortlist") return found.shortlisted_review;
    return found.suggestions;
  });

  const selectDiscoveryView = (selection: "active_batch" | "shortlist" | "full_queue") => {
    setDiscoverySelection(selection);
    const found = discovery();
    const options = selection === "active_batch"
      ? found?.active_review_batch ?? []
      : selection === "shortlist"
        ? found?.shortlisted_review ?? []
        : found?.suggestions ?? [];
    setSuggestionId(options[0]?.suggestion_id ?? "");
  };

  const load = async () => {
    const [next, discovered] = await Promise.all([
      getHistoricalDecisionAnchors(),
      getHistoricalAnchorDiscovery(),
    ]);
    setRegistry(next);
    setDiscovery(discovered);
    if (!sourceId() && next.sources.length) {
      setSourceId(next.sources[0].source_item_id);
      setSymbol(next.sources[0].tickers[0] ?? "");
    }
    if (!reviewCandidateId() && next.anchors.length) {
      setReviewCandidateId(
        next.anchors.find((item) => !item.latest_review)?.candidate.candidate_id
          ?? next.anchors[0].candidate.candidate_id,
      );
    }
    if (!suggestionId()) {
      const preferred = discoverySelection() === "active_batch"
        ? discovered.active_review_batch
        : discoverySelection() === "shortlist"
          ? discovered.shortlisted_review
          : discovered.suggestions;
      setSuggestionId(preferred[0]?.suggestion_id ?? "");
    }
  };

  onMount(() => void load().catch((cause) => setError(cause instanceof Error ? cause.message : "历史锚点读取失败")));

  const prefillSuggestion = () => {
    const suggestion = selectedSuggestion();
    if (!suggestion) return;
    setSourceId(suggestion.source_item_id);
    setSymbol(suggestion.tickers[0] ?? "");
    setLocator(suggestion.source_locator);
    setExcerpt(suggestion.verbatim_excerpt);
    setAction(suggestion.suggested_action ?? "research_only");
    setThesis("");
    setError("");
    setNotice("只预填完整原文、来源和动作词；请人工核对说话人并填写候选判断。系统尚未保存任何候选。");
  };

  const submitDiscoveryScreening = async (
    verdict: HistoricalAnchorDiscoveryScreeningVerdict,
  ) => {
    const suggestion = selectedSuggestion();
    if (!suggestion) return;
    const isCorrection = suggestion.screening_status !== "pending";
    if (isCorrection && !screeningCorrectionReason().trim()) {
      setError("修正已完成的筛选时，请先填写修正原因。");
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await screenHistoricalAnchorDiscovery(suggestion.suggestion_id, {
        expected_source_sha256: suggestion.source_sha256,
        expected_screening_id: suggestion.screening_record_id,
        verdict,
        correction_reason: isCorrection ? screeningCorrectionReason().trim() : undefined,
      });
      await load();
      setScreeningCorrectionReason("");
      setNotice(isCorrection
        ? "修正已作为新记录追加，旧记录仍完整保留；这一步仍不确认说话人、动作或投资逻辑，也不进入训练。"
        : "筛选已写入不可覆盖记录；这一步只决定是否继续建立候选，不确认说话人、动作或投资逻辑，也不进入训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存历史原话筛选失败");
    } finally {
      setBusy(false);
    }
  };

  const submitCandidate = async () => {
    const source = selectedSource();
    if (!source) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const candidate = await createHistoricalDecisionAnchorCandidate({
        source_item_id: source.source_item_id,
        expected_source_sha256: source.source_sha256,
        symbol: symbol(),
        source_locator: locator(),
        verbatim_excerpt: excerpt(),
        candidate_action: action(),
        candidate_thesis: thesis(),
      });
      setReviewCandidateId(candidate.candidate_id);
      setLocator("");
      setExcerpt("");
      setThesis("");
      await load();
      setNotice("候选已与完整原文哈希绑定；尚未视为老王确认，也不进入训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存历史判断候选失败");
    } finally {
      setBusy(false);
    }
  };

  const submitReview = async () => {
    const item = selectedCandidate();
    if (!item) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await reviewHistoricalDecisionAnchor(item.candidate.candidate_id, {
        expected_review_id: item.latest_review?.review_id,
        verdict: reviewVerdict(),
        confirmation_statement: reviewStatement(),
        decision_available_at:
          reviewVerdict() === "rejected" || !decisionAvailableAt()
            ? undefined
            : new Date(decisionAvailableAt()).toISOString(),
        source_time_confirmed: sourceTimeConfirmed(),
        speaker_identity_confirmed: speakerConfirmed(),
        later_evidence_excluded_confirmed: noHindsightConfirmed(),
        revised_action:
          reviewVerdict() === "revised" ? action() : undefined,
        revised_thesis:
          reviewVerdict() === "revised" ? thesis() : undefined,
      });
      setReviewStatement("");
      setDecisionAvailableAt("");
      setSourceTimeConfirmed(false);
      setSpeakerConfirmed(false);
      setNoHindsightConfirmed(false);
      await load();
      setNotice("复核已写入不可覆盖记录；确认项目前仍只属于历史基准，不进入训练或奖励门槛。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存历史锚点复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(value) => (
        <section class="public-admin-historical-anchors" aria-label="历史判断锚点">
          <header>
            <div>
              <h3>历史判断锚点（候选 → 老王确认 → 回测）</h3>
              <p>{value().scope}</p>
            </div>
            <span>训练隔离</span>
          </header>
          <div class="public-admin-decision-metrics">
            <div><span>完整原文</span><strong>{value().source_count}</strong></div>
            <div><span>覆盖公司</span><strong>{value().source_symbol_count}</strong></div>
            <div><span>待确认</span><strong>{value().pending_candidate_count}</strong></div>
            <div><span>历史基准锚点</span><strong>{value().confirmed_anchor_count}</strong></div>
          </div>
          <p class="public-admin-anchor-boundary">
            原文范围 {value().earliest_source_date ?? "—"} 至 {value().latest_source_date ?? "—"}；自动提取关闭、自动确认关闭、结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭。
          </p>

          <Show when={discovery()}>{(found) => (
            <details class="public-admin-reward-governance">
              <summary>从逐字稿定位待确认原话</summary>
              <p>{found().scope}</p>
              <div class="public-admin-decision-metrics">
                <div><span>扫描原文</span><strong>{found().source_count}</strong></div>
                <div><span>命中原文</span><strong>{found().matched_source_count}</strong></div>
                <div><span>待筛片段</span><strong>{found().pending_screening_count}</strong></div>
                <div><span>优先复核</span><strong>{found().active_review_batch_size}</strong></div>
                <div><span>继续建候选</span><strong>{found().shortlisted_review_count}</strong></div>
              </div>
              <div class="public-admin-financial-review-selection">
                <div>
                  <strong>人工复核范围</strong>
                  <span>{discoverySelection() === "active_batch" ? "本轮优先复核" : discoverySelection() === "shortlist" ? "已筛出的候选短名单" : "完整发现队列"}</span>
                  <small>每批最多 5 条，只按逐字稿主要说话人标签、第一人称语境、风险排除和公司多样性降噪；身份未确认，绝不代表老王确认。</small>
                </div>
                <nav aria-label="历史原话复核范围">
                  <button type="button" classList={{ "is-active": discoverySelection() === "active_batch" }} onClick={() => selectDiscoveryView("active_batch")}>优先 {found().active_review_batch_size} 条</button>
                  <button type="button" classList={{ "is-active": discoverySelection() === "shortlist" }} onClick={() => selectDiscoveryView("shortlist")}>短名单 {found().shortlisted_review_count} 条</button>
                  <button type="button" classList={{ "is-active": discoverySelection() === "full_queue" }} onClick={() => selectDiscoveryView("full_queue")}>查看全部</button>
                </nav>
              </div>
              <Show when={found().suggestions.length > 0} fallback={<p class="public-admin-decision-notice">完整逐字稿中暂未定位到明确动作词，请继续使用下方手工入口。</p>}>
                <label>
                  <span>待筛原话</span>
                  <select value={suggestionId()} onChange={(event) => setSuggestionId(event.currentTarget.value)}>
                    <For each={visibleSuggestions()}>{(suggestion) => (
                      <option value={suggestion.suggestion_id}>
                        {suggestion.source_date} · {suggestion.tickers.join("/")} · {suggestion.speaker_label ?? "说话人未识别"} · {suggestion.matched_action_cues.join("/")} · {suggestion.screening_status === "pending" ? "待筛" : suggestion.screening_status === "continue_candidate_review" ? "继续建候选" : suggestion.screening_status === "needs_more_context" ? "需更多上下文" : "非判断语境"}
                      </option>
                    )}</For>
                  </select>
                </label>
                <Show when={selectedSuggestion()}>{(suggestion) => (
                  <>
                    <blockquote>
                      “{suggestion().verbatim_excerpt}”
                      <small>说话人标签：{suggestion().speaker_label ?? "未识别"}（身份未确认） · {suggestion().dominant_source_speaker ? "该逐字稿主要说话人" : "非主要说话人"} · {suggestion().source_title} · {suggestion().source_locator}</small>
                      <small>{suggestion().review_priority_reasons.join("；") || "未进入优先复核规则"} · {suggestion().suggested_action ? `第一人称动作预填：${ACTION_LABELS[suggestion().suggested_action!]}` : "动作归属或方向不够明确，必须人工选择"}</small>
                    </blockquote>
                    <details class="public-admin-anchor-context">
                      <summary>查看前后原文（第 {suggestion().context_window.start_line}–{suggestion().context_window.end_line} 行）</summary>
                      <pre>{suggestion().context_window.verbatim_context}</pre>
                      <small>上下文 SHA-256：{suggestion().context_window.context_sha256.slice(0, 16)}…{suggestion().context_window.truncated ? " · 已按长度边界截取" : ""}</small>
                    </details>
                  </>
                )}</Show>
                <Show when={selectedSuggestion()}>{(suggestion) => (
                  <div class="public-admin-anchor-screening-question">
                    <strong>单问：这条原话是否值得继续建立历史判断候选？</strong>
                    <small>{suggestion().screening_status === "pending"
                      ? "这里只做管理员分流，不确认说话人、动作或投资逻辑。"
                      : `当前筛选：${SCREENING_LABELS[suggestion().screening_status as HistoricalAnchorDiscoveryScreeningVerdict] ?? suggestion().screening_status}。如需纠错，只追加修正记录，不覆盖旧记录。`}</small>
                    <Show when={suggestion().screening_status !== "pending"}>
                      <textarea
                        value={screeningCorrectionReason()}
                        maxlength={400}
                        onInput={(event) => setScreeningCorrectionReason(event.currentTarget.value)}
                        placeholder="填写修正原因（必填）"
                      />
                    </Show>
                    <div>
                      <button type="button" disabled={busy() || suggestion().screening_status === "continue_candidate_review"} onClick={() => void submitDiscoveryScreening("continue_candidate_review")}>值得继续</button>
                      <button type="button" disabled={busy() || suggestion().screening_status === "not_decision_context"} onClick={() => void submitDiscoveryScreening("not_decision_context")}>不是判断语境</button>
                      <button type="button" disabled={busy() || suggestion().screening_status === "needs_more_context"} onClick={() => void submitDiscoveryScreening("needs_more_context")}>需要更多上下文</button>
                    </div>
                  </div>
                )}</Show>
                <Show when={selectedSuggestion()?.screening_status === "continue_candidate_review"}>
                  <button type="button" class="public-admin-decision-submit" disabled={busy() || !selectedSuggestion()} onClick={prefillSuggestion}>
                    预填到人工候选表单（不保存）
                  </button>
                </Show>
              </Show>
            </details>
          )}</Show>

          <details class="public-admin-reward-governance">
            <summary>从完整逐字稿建立一个可核验候选</summary>
            <div class="public-admin-anchor-form">
              <label>
                <span>来源逐字稿</span>
                <select value={sourceId()} onChange={(event) => {
                  const id = event.currentTarget.value;
                  setSourceId(id);
                  const source = value().sources.find((item) => item.source_item_id === id);
                  setSymbol(source?.tickers[0] ?? "");
                }}>
                  <For each={value().sources}>{(source) => (
                    <option value={source.source_item_id}>{source.source_date} · {source.title}</option>
                  )}</For>
                </select>
              </label>
              <label>
                <span>公司</span>
                <select value={symbol()} onChange={(event) => setSymbol(event.currentTarget.value)}>
                  <For each={selectedSource()?.tickers ?? []}>{(ticker) => <option value={ticker}>{ticker}</option>}</For>
                </select>
              </label>
              <label>
                <span>原话定位</span>
                <input value={locator()} maxlength={160} onInput={(event) => setLocator(event.currentTarget.value)} placeholder="例如 00:27:18 或第 61 行" />
              </label>
              <label>
                <span>动作候选</span>
                <select value={action()} onChange={(event) => setAction(event.currentTarget.value as HistoricalDecisionAnchorAction)}>
                  <For each={Object.entries(ACTION_LABELS)}>{([key, label]) => <option value={key}>{label}</option>}</For>
                </select>
              </label>
              <label class="is-wide">
                <span>逐字复制原话（服务器会对照完整文件）</span>
                <textarea value={excerpt()} maxlength={2400} onInput={(event) => setExcerpt(event.currentTarget.value)} />
              </label>
              <label class="is-wide">
                <span>AI/管理员候选归纳（不是老王确认）</span>
                <textarea value={thesis()} maxlength={1200} onInput={(event) => setThesis(event.currentTarget.value)} />
              </label>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={busy() || !selectedSource() || !symbol() || !locator().trim() || !excerpt().trim() || !thesis().trim()} onClick={() => void submitCandidate()}>
              保存候选（不进入训练）
            </button>
          </details>

          <Show when={value().anchors.length > 0}>
            <details class="public-admin-reward-governance">
              <summary>逐条确认历史判断</summary>
              <label>
                <span>待复核候选</span>
                <select value={reviewCandidateId()} onChange={(event) => setReviewCandidateId(event.currentTarget.value)}>
                  <For each={value().anchors}>{(item) => (
                    <option value={item.candidate.candidate_id}>
                      {item.candidate.claimed_source_date} · {item.candidate.symbol} · {item.latest_review?.verdict ?? "待确认"}
                    </option>
                  )}</For>
                </select>
              </label>
              <Show when={selectedCandidate()}>{(item) => (
                <blockquote>
                  “{item().candidate.verbatim_excerpt}”
                  <small>{item().candidate.source_locator} · 候选：{ACTION_LABELS[item().candidate.candidate_action]} · {item().candidate.candidate_thesis}</small>
                </blockquote>
              )}</Show>
              <label>
                <span>复核结论</span>
                <select value={reviewVerdict()} onChange={(event) => setReviewVerdict(event.currentTarget.value as HistoricalDecisionAnchorReviewVerdict)}>
                  <option value="confirmed">确认这是当时判断</option>
                  <option value="revised">修订后确认</option>
                  <option value="rejected">否决候选归纳</option>
                </select>
              </label>
              <label>
                <span>老王确认说明</span>
                <textarea value={reviewStatement()} maxlength={800} onInput={(event) => setReviewStatement(event.currentTarget.value)} placeholder="说明当时实际判断、动作和必要边界。" />
              </label>
              <Show when={reviewVerdict() !== "rejected"}>
                <label>
                  <span>当时判断可被市场使用的精确时间</span>
                  <input type="datetime-local" value={decisionAvailableAt()} onInput={(event) => setDecisionAvailableAt(event.currentTarget.value)} />
                  <small>必须是来源日期当天的北京时间，用于区分盘前、盘中与盘后，不能只填日期。</small>
                </label>
                <label class="public-admin-reward-confirm"><input type="checkbox" checked={sourceTimeConfirmed()} onChange={(event) => setSourceTimeConfirmed(event.currentTarget.checked)} /> 已核对原始发布时间</label>
                <label class="public-admin-reward-confirm"><input type="checkbox" checked={speakerConfirmed()} onChange={(event) => setSpeakerConfirmed(event.currentTarget.checked)} /> 已确认原话确为老王本人表达</label>
                <label class="public-admin-reward-confirm"><input type="checkbox" checked={noHindsightConfirmed()} onChange={(event) => setNoHindsightConfirmed(event.currentTarget.checked)} /> 候选没有混入事后信息</label>
              </Show>
              <button type="button" class="public-admin-decision-submit" disabled={busy() || !selectedCandidate() || !reviewStatement().trim() || (reviewVerdict() !== "rejected" && (!decisionAvailableAt() || !(sourceTimeConfirmed() && speakerConfirmed() && noHindsightConfirmed())))} onClick={() => void submitReview()}>
                写入不可覆盖的确认记录
              </button>
            </details>
          </Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
        </section>
      )}
    </Show>
  );
}
