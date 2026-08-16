import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicKeyEventChains } from "@/lib/api";
import { ResearchPanel, ResearchPanelHead } from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type { KeyEventChainSnapshot } from "@/lib/types";
import "./key-event-chain-dashboard.css";

type Props = {
  onClose: () => void;
  onAsk?: (message: string) => void;
};

type EvidenceFilter = "all" | "confirmed";

const statusLabel = (value: string) => ({
  live: "影响分析已更新",
  partial: "部分完成分析",
  source_only: "原链已更新 · 影响待验证",
  no_updates: "当前来源无命中",
  source_unconfigured: "来源待配置",
  data_unavailable: "来源读取失败",
  stale: "上次成功快照",
}[value] ?? "等待数据");

/** Traffic light for the panel head: green only once impact analysis ran. */
const statusSignal = (value: string) => ({
  live: "green",
  partial: "yellow",
  source_only: "yellow",
  no_updates: "yellow",
  stale: "yellow",
  source_unconfigured: "orange",
  data_unavailable: "red",
}[value] ?? "yellow");

const changeLabel = (value: string) => ({
  schedule: "时间表 / 路线图",
  specification: "参数 / 规格",
  launch: "发布",
  qualification: "认证 / 验证",
  mass_production: "量产",
  order: "订单 / 合同",
  capacity: "扩产 / 建设",
  deployment: "部署 / 交付",
  financial: "财务兑现",
  policy: "政策变化",
  viewpoint: "观点变化",
  unclear: "待判断",
}[value] ?? "待分类");

const directionLabel = (value: string) => ({
  positive: "正向",
  negative: "负向",
  mixed: "多空混合",
  neutral: "中性",
  unclear: "待验证",
}[value] ?? "待验证");

const sourceTierLabel = (value: string) => ({
  primary: "公司官方",
  regulatory: "监管原文",
  research: "研究资料",
  opinion: "观点线索",
  secondary: "二手来源",
}[value] ?? "来源待分类");

const verificationLabel = (value: string) => (value === "confirmed" ? "一手确认" : "待核实线索");

export function KeyEventChainPanel(props: Props) {
  const [snapshot, setSnapshot] = createSignal<KeyEventChainSnapshot>();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [topicId, setTopicId] = createSignal("rubin");
  const [evidenceFilter, setEvidenceFilter] = createSignal<EvidenceFilter>("all");
  const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      const next = await getPublicKeyEventChains(controller.signal);
      setSnapshot(next);
      if (!next.topics.some((topic) => topic.id === topicId())) {
        setTopicId(next.topics[0]?.id ?? "rubin");
      }
    } catch (value) {
      if (value instanceof Error && value.name === "AbortError") return;
      setError(value instanceof Error ? value.message : "关键事件链暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const topic = createMemo(
    () => snapshot()?.topics.find((item) => item.id === topicId()) ?? snapshot()?.topics[0],
  );
  // Panel-level verdict: how much moved across every industry line, and how
  // much of it is first-hand rather than a clue.
  const totals = createMemo(() => {
    const topics = snapshot()?.topics ?? [];
    return {
      topics: topics.length,
      events: topics.reduce((sum, item) => sum + item.event_count, 0),
      confirmed: topics.reduce((sum, item) => sum + (item.confirmed_count ?? 0), 0),
    };
  });
  const metaLine = createMemo(() => {
    const current = snapshot();
    if (!current) return undefined;
    // Provenance is fail-closed: a field the snapshot did not carry is
    // dropped rather than printed as an `undefined` hole.
    return [
      `报告日 ${current.report_date}`,
      `近 ${current.lookback_days} 天`,
      `${totals().topics} 条产业主线`,
      `一手确认 ${totals().confirmed}`,
      "每日 19:55 更新",
      current.generated_at_local
        ? [current.generated_at_local, current.timezone].filter(Boolean).join(" ")
        : undefined,
    ]
      .filter(Boolean)
      .join(" · ");
  });
  const visibleEvents = createMemo(() => {
    const events = topic()?.events ?? [];
    return evidenceFilter() === "confirmed"
      ? events.filter((event) => event.verification_status === "confirmed")
      : events;
  });

  const send = () => {
    const report = snapshot();
    const current = topic();
    const query = question().trim();
    if (!report || !current || !query || !props.onAsk) return;
    const saved = {
      reportDate: report.report_date,
      generatedAtLocal: report.generated_at_local,
      status: report.status,
      evidenceFilter: evidenceFilter(),
      topic: current,
    };
    props.onAsk(
      buildSavedReportPrompt({
        marker: "HONE_SAVED_KEY_EVENT_CHAIN",
        payload: saved,
        subject: "已保存的关键事件链",
        question: query,
        rules: [
          "逐条附原文链接和运行时时区",
          "只有 verification_status=confirmed 才可称为已确认里程碑，clue 只能作为待核实线索",
          "区分来源事实、作者观点与 HONE 推断",
          "聚合翻译和管理员研究资料不是一手事实",
          "影响待验证时不得补造结论，不得直接转成买卖或仓位动作",
        ],
      }),
    );
    setQuestion("");
    props.onClose();
  };

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="key-chain-title"
      backdropClass="key-chain-backdrop"
      dialogClass="key-chain-dialog"
    >
      <>
        <ResearchPanelHead
          id="key-chain-title"
          kicker="第一性原理产业图谱"
          title="关键事件链"
          headline={snapshot() ? `${totals().events} 条变化` : undefined}
          signal={snapshot() ? statusSignal(snapshot()!.status) : undefined}
          signalLabel={snapshot() ? statusLabel(snapshot()!.status) : undefined}
          summary={snapshot()?.summary}
          meta={metaLine()}
          onClose={props.onClose}
          action={
            <button type="button" disabled={loading()} onClick={() => void load()}>
              {loading() ? "读取中…" : "重新读取"}
            </button>
          }
        />

        <nav class="key-chain-topics" aria-label="产业主线：模型 → 应用 → 数据中心 → 算力 → 光互连 → 存储 → 电力">
          <For each={snapshot()?.topics ?? []}>
            {(item) => (
              <button classList={{ active: topicId() === item.id }} onClick={() => setTopicId(item.id)}>
                <em>{item.layer || "产业主线"}</em>
                <strong>{item.name}</strong>
                <small>{item.confirmed_count ?? 0} 确认 · {item.clue_count ?? item.event_count} 线索</small>
              </button>
            )}
          </For>
        </nav>

        <div class="key-chain-body">
          <Show when={loading() && !snapshot()}>
            <ResearchState kind="loading" message="正在整理模型到电力的产业主线" detail="只读取已保存的事件链快照，不会触发重新计算。" />
          </Show>
          <Show when={error()}>
            <ResearchState kind="error" message="关键事件链读取失败" detail={error()} onRetry={() => void load()} />
          </Show>
          <Show when={!error() && topic()}>
            {(current) => (
              <>
                <section class="key-chain-topic-head">
                  <div>
                    <span>{current().layer || "产业主线"}</span>
                    <h3>{current().name}</h3>
                  </div>
                  <p>{current().description}</p>
                  <aside>
                    <strong>第一性原理：</strong>
                    {current().first_principle}
                  </aside>
                  <strong>{current().latest_change}</strong>
                  <nav class="key-chain-evidence-filter" aria-label="证据筛选">
                    <button
                      classList={{ active: evidenceFilter() === "all" }}
                      onClick={() => setEvidenceFilter("all")}
                    >
                      全部原链 {current().event_count}
                    </button>
                    <button
                      classList={{ active: evidenceFilter() === "confirmed" }}
                      onClick={() => setEvidenceFilter("confirmed")}
                    >
                      只看一手确认 {current().confirmed_count ?? 0}
                    </button>
                  </nav>
                </section>
                <Show
                  when={visibleEvents().length}
                  fallback={
                    <ResearchState
                      kind="empty"
                      message={evidenceFilter() === "confirmed" ? "近 30 天没有一手确认里程碑" : "近 30 天当前来源没有命中事件"}
                      detail="没有事件不等于主题没有变化；待核实线索也不会被自动升级为事实。"
                    />
                  }
                >
                  <div class="key-chain-timeline">
                    <For each={visibleEvents()}>
                      {(event) => (
                        <article data-verification={event.verification_status}>
                          <time>{event.published_at_local} {snapshot()?.timezone}</time>
                          <div>
                            <div class="key-chain-event-meta">
                              <b>{event.source_name}</b>
                              <span>{changeLabel(event.change_type)}</span>
                              <span>{sourceTierLabel(event.source_tier)}</span>
                              <span data-verification={event.verification_status}>
                                {verificationLabel(event.verification_status)}
                              </span>
                              <span data-direction={event.direction}>{directionLabel(event.direction)}</span>
                            </div>
                            <h4>{event.title}</h4>
                            <p>{event.excerpt}</p>
                            <aside class="key-chain-verification">
                              <strong>证据口径：</strong>
                              {event.verification_note}
                            </aside>
                            <aside>
                              <strong>影响：</strong>
                              {event.impact}
                            </aside>
                            <aside>
                              <strong>下一验证点：</strong>
                              {event.next_watch}
                            </aside>
                            <div class="key-chain-tickers">
                              <For each={event.tickers}>{(ticker) => <span>${ticker}</span>}</For>
                            </div>
                            <a href={event.source_url} target="_blank" rel="noreferrer">
                              {event.verification_status === "confirmed" ? "查看一手原文" : "查看线索原文"} ↗
                            </a>
                          </div>
                        </article>
                      )}
                    </For>
                  </div>
                </Show>
              </>
            )}
          </Show>
        </div>

        <Show when={props.onAsk}>
          <footer>
            <p>{snapshot()?.disclaimer}</p>
            <div>
              <input
                aria-label="基于关键事件链提问"
                value={question()}
                onInput={(event) => setQuestion(event.currentTarget.value)}
                placeholder="这条变化会影响哪些公司？"
              />
              <button disabled={!question().trim()} onClick={send}>发送到对话</button>
            </div>
          </footer>
        </Show>
      </>
    </ResearchPanel>
  );
}
