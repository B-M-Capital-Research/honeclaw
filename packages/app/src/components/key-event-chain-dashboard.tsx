import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicKeyEventChains } from "@/lib/api";
import {
  ResearchPanel,
  ResearchPanelHead,
  shortLocalTimestamp,
} from "@/components/research/research-panel";
import { ResearchFeed, ResearchFeedItem } from "@/components/research/research-feed";
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

type KeyEvent = NonNullable<KeyEventChainSnapshot["topics"][number]["events"]>[number];

/**
 * The event as its source published it: headline, then the source's own text.
 * They are one block of writing rather than a heading plus a folded body, so
 * the change reads at a glance instead of asking for two clicks. The headline
 * is dropped when the excerpt already opens with it.
 */
const eventText = (event: KeyEvent) => {
  const title = event.title.trim();
  const excerpt = event.excerpt.trim();
  if (!excerpt) return title;
  return excerpt.startsWith(title) ? excerpt : [title, excerpt].filter(Boolean).join("\n");
};

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

        {/* One compact chip per line. The card used to stack layer, name and a
            spelled-out count on three rows, so six of twelve lines filled the
            row and the rest needed a long drag. The layer is not lost: the
            selected topic prints it directly below. */}
        <nav
          class="key-chain-topics research-scroller"
          aria-label="产业主线：模型 → 应用 → 数据中心 → 算力 → 光互连 → 存储 → 电力"
        >
          <For each={snapshot()?.topics ?? []}>
            {(item) => (
              <button
                classList={{ active: topicId() === item.id }}
                aria-label={`${item.name} · 一手确认 ${item.confirmed_count ?? 0} 条，共 ${item.event_count} 条`}
                onClick={() => setTopicId(item.id)}
              >
                <strong>{item.name}</strong>
                <small>{item.confirmed_count ?? 0}/{item.event_count}</small>
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
                  <nav class="key-chain-evidence-filter research-scroller" aria-label="证据筛选">
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
                  <ResearchFeed>
                    <For each={visibleEvents()}>
                      {(event) => (
                        <ResearchFeedItem
                          author={event.source_name}
                          meta={[changeLabel(event.change_type), directionLabel(event.direction)]}
                          // The head's meta line already names the run timezone;
                          // repeating it per row bought width and no information.
                          time={shortLocalTimestamp(event.published_at_local)}
                          // Green only for a first-hand confirmation; a clue
                          // stays yellow. One edge, no chip row.
                          accent={event.verification_status === "confirmed" ? "green" : "yellow"}
                          links={[
                            {
                              href: event.source_url,
                              label:
                                event.verification_status === "confirmed"
                                  ? "查看一手原文"
                                  : "查看线索原文",
                            },
                          ]}
                          analysisLabel="HONE 解读"
                          analysis={
                            <>
                              <p>
                                <b>证据口径：</b>
                                {verificationLabel(event.verification_status)} ·{" "}
                                {event.verification_note}
                              </p>
                              <p>
                                <b>影响：</b>
                                {event.impact}
                              </p>
                              <p>
                                <b>下一验证点：</b>
                                {event.next_watch}
                              </p>
                              <p class="key-chain-analysis-terms">
                                {[
                                  sourceTierLabel(event.source_tier),
                                  ...event.tickers.map((ticker) => `$${ticker}`),
                                ].join(" · ")}
                              </p>
                            </>
                          }
                        >
                          {/* The change in the source's own words, open. It used
                              to sit folded under a restated headline, below the
                              chips and above three of our own paragraphs. */}
                          <p>{eventText(event)}</p>
                        </ResearchFeedItem>
                      )}
                    </For>
                  </ResearchFeed>
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
