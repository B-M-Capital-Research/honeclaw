import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicInfluencerDigest } from "@/lib/api";
import {
  ResearchPanel,
  ResearchPanelHead,
  shortLocalTimestamp,
} from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type { InfluencerDigestItem, InfluencerDigestSnapshot } from "@/lib/types";
import "./influencer-digest-dashboard.css";

type Props = {
  onClose: () => void;
  onAsk?: (message: string) => void;
};

const stanceLabel = (v: string) =>
  ({ bullish: "偏多", bearish: "偏空", mixed: "多空混合", neutral: "中性", unclear: "待判断" }[v] ?? "待判断");

const postKindLabel = (v: string) =>
  ({ original: "原创", reply: "回复", quote: "引用", retweet: "转推", article: "文章" }[v] ?? "公开内容");

const statusLabel = (v: string) =>
  ({
    live: "来源与模型已更新",
    partial: "部分更新",
    source_only: "仅原文",
    no_updates: "今日无更新",
    source_unconfigured: "部分来源待配置",
    data_unavailable: "来源读取失败",
    stale: "上次成功快照",
  }[v] ?? "等待数据");

/** Author text as published: translation first, English when a post was never
 *  translated, and the legacy short excerpt for pre-full-text snapshots. */
const sourceText = (item: InfluencerDigestItem) =>
  (item.source_text_cn || "").trim() ||
  (item.source_text_en || "").trim() ||
  (item.source_excerpt || "").trim();

/** Only a secondary fold: an untranslated post is already shown in English. */
const englishOriginal = (item: InfluencerDigestItem) => {
  const english = (item.source_text_en || "").trim();
  return english === sourceText(item) ? "" : english;
};

const reach = (item: InfluencerDigestItem) => {
  const views = item.metrics?.views ?? 0;
  const likes = item.metrics?.likes ?? 0;
  if (!views && !likes) return undefined;
  const compact = (value: number) =>
    value >= 10000 ? `${(value / 1000).toFixed(1)}k` : `${value}`;
  return [views ? `阅读 ${compact(views)}` : undefined, likes ? `赞 ${compact(likes)}` : undefined]
    .filter(Boolean)
    .join(" · ");
};

/** Traffic light for the panel head: green only when sources and model both ran. */
const statusSignal = (v: string) =>
  ({
    live: "green",
    partial: "yellow",
    source_only: "yellow",
    no_updates: "yellow",
    stale: "yellow",
    source_unconfigured: "orange",
    data_unavailable: "red",
  }[v] ?? "yellow");

export function InfluencerDigestPanel(props: Props) {
  const [snapshot, setSnapshot] = createSignal<InfluencerDigestSnapshot>();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [author, setAuthor] = createSignal("all");
  const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setSnapshot(await getPublicInfluencerDigest(controller.signal));
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "大V速报暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const visible = createMemo(() =>
    author() === "all"
      ? snapshot()?.items ?? []
      : (snapshot()?.items ?? []).filter((item) => item.author_id === author()),
  );

  // Provenance collapses into one secondary line: report day, author coverage,
  // refresh clock, run timezone and model version used to sit in a separate
  // metadata strip that pushed the actual posts below the fold on phones.
  const metaLine = createMemo(() => {
    const current = snapshot();
    if (!current) return undefined;
    return [
      `报告日 ${current.report_date}`,
      `已配置作者 ${current.coverage.configured}/${current.coverage.authors}`,
      "每日 19:50 更新",
      current.generated_at_local
        ? [current.generated_at_local, current.timezone].filter(Boolean).join(" ")
        : undefined,
      `模型 ${current.model_version}`,
    ]
      .filter(Boolean)
      .join(" · ");
  });

  const send = () => {
    const current = snapshot();
    const query = question().trim();
    if (!current || !query || !props.onAsk) return;
    const report = {
      reportDate: current.report_date,
      generatedAtLocal: current.generated_at_local,
      status: current.status,
      authors: current.authors,
      items: current.items.map((item) => ({
        author_name: item.author_name,
        public_handle: item.public_handle,
        title: item.title,
        published_at_local: item.published_at_local,
        source_url: item.source_url,
        aggregation_source: item.aggregation_source,
        aggregation_url: item.aggregation_url,
        post_kind: item.post_kind,
        summary: item.summary,
        stance: item.stance,
        horizon: item.horizon,
        content_type: item.content_type,
        topics: item.topics,
        tickers: item.tickers,
        counterpoint: item.counterpoint,
        analysis_status: item.analysis_status,
        // The model answers about what the author actually wrote, not only
        // about our 90-character digest of it.
        source_text_cn: sourceText(item).slice(0, 600),
        reply_context: item.reply_context ?? undefined,
      })),
    };
    props.onAsk(
      buildSavedReportPrompt({
        marker: "HONE_SAVED_INFLUENCER_DIGEST",
        payload: report,
        subject: "已保存的大V速报",
        question: query,
        rules: [
          "作者观点不等于事实或 HONE 结论",
          "附原文链接与运行时时区",
          "聚合翻译不是独立事实来源",
          "不得补造未配置作者内容，不得把观点直接转换为买卖或仓位动作",
        ],
      }),
    );
    setQuestion("");
    props.onClose();
  };

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="influencer-title"
      backdropClass="influencer-digest-backdrop"
      dialogClass="influencer-digest-dialog"
    >
      <>
        <ResearchPanelHead
          id="influencer-title"
          kicker="先看来源，再看观点"
          title="大V速报"
          headline={snapshot() ? `${visible().length} 条原文` : undefined}
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

        <div class="influencer-authors research-scroller">
          <button classList={{ active: author() === "all" }} onClick={() => setAuthor("all")}>全部</button>
          <For each={snapshot()?.authors ?? []}>
            {(item) => (
              <button
                classList={{ active: author() === item.id, missing: !item.configured }}
                onClick={() => setAuthor(item.id)}
              >
                {item.name}
                <small>{item.configured ? `${item.item_count} 条` : "源待配置"}</small>
              </button>
            )}
          </For>
        </div>

        <div class="influencer-digest-body">
          <Show when={loading() && !snapshot()}>
            <ResearchState kind="loading" message="正在读取公开观点" detail="只读取已保存的当日速报，不会触发重新生成。" />
          </Show>
          <Show when={error()}>
            <ResearchState kind="error" message="大V速报读取失败" detail={error()} onRetry={() => void load()} />
          </Show>
          <Show when={!error() && snapshot()}>
            <Show
              when={visible().length}
              fallback={
                <ResearchState
                  kind="empty"
                  message="当前没有可展示的原文更新"
                  detail="换一位作者或稍后重新读取；未配置来源不会被补造内容。"
                />
              }
            >
              <For each={visible()}>
                {(item) => (
                  <article>
                    <div>
                      <b>{item.author_name}</b>
                      <span>{item.public_handle}</span>
                      <span>{postKindLabel(item.post_kind)}</span>
                      <Show when={reach(item)}>
                        <span>{reach(item)}</span>
                      </Show>
                      {/* The head's meta line already names the run timezone. */}
                      <time>{shortLocalTimestamp(item.published_at_local)}</time>
                    </div>
                    <h3>{item.title}</h3>

                    {/* A reply without the post it answers reads as half a
                        conversation, so the quoted context comes first. */}
                    <Show when={item.reply_context}>
                      <blockquote class="influencer-quoted">
                        <cite>
                          {item.post_kind === "quote" ? "引用" : "回复"} {item.reply_context!.author}
                        </cite>
                        <p>{item.reply_context!.text}</p>
                      </blockquote>
                    </Show>

                    <Show when={item.analysis_status === "model_analyzed" && item.summary.trim()}>
                      <p class="influencer-summary">
                        <b>HONE 摘要</b>
                        {item.summary}
                      </p>
                    </Show>

                    <Show when={sourceText(item)}>
                      <details
                        class="influencer-source"
                        open={item.analysis_status !== "model_analyzed"}
                      >
                        <summary>作者原文</summary>
                        <p>{sourceText(item)}</p>
                        <Show when={englishOriginal(item)}>
                          <details class="influencer-source-en">
                            <summary>English original</summary>
                            <p>{englishOriginal(item)}</p>
                          </details>
                        </Show>
                      </details>
                    </Show>

                    <Show when={item.media_urls?.length}>
                      <div
                        class="influencer-media"
                        classList={{ single: item.media_urls!.length === 1 }}
                      >
                        <For each={item.media_urls}>
                          {(url) => (
                            <a href={url} target="_blank" rel="noreferrer">
                              <img
                                src={url}
                                alt={`${item.author_name} 原文配图`}
                                loading="lazy"
                                decoding="async"
                                referrerpolicy="no-referrer"
                              />
                            </a>
                          )}
                        </For>
                      </div>
                    </Show>

                    {/* Two chips carry the judgement — which way the author
                        leans, and whether this is fact or opinion. Topics and
                        tickers are index terms, so they read as one quiet line
                        instead of a dozen chips of equal weight. */}
                    <div class="influencer-tags">
                      <span>{stanceLabel(item.stance)}</span>
                      <span>
                        {item.content_type === "fact"
                          ? "事实陈述"
                          : item.content_type === "opinion"
                            ? "作者观点"
                            : "事实与观点混合"}
                      </span>
                      <Show when={item.tickers.length || item.topics.length}>
                        <small>
                          {[
                            ...item.tickers.map((ticker) => `$${ticker}`),
                            ...item.topics,
                          ].join(" · ")}
                        </small>
                      </Show>
                    </div>
                    <aside>
                      <strong>反方 / 未证实处：</strong>
                      {item.counterpoint}
                    </aside>
                    <a href={item.source_url} target="_blank" rel="noreferrer">查看作者原文 ↗</a>
                    <Show when={item.aggregation_url}>
                      <a href={item.aggregation_url!} target="_blank" rel="noreferrer">
                        {item.aggregation_source} · 翻译/聚合源 ↗
                      </a>
                    </Show>
                  </article>
                )}
              </For>
            </Show>
          </Show>
        </div>

        <Show when={props.onAsk}>
          <footer>
            <p>{snapshot()?.disclaimer}</p>
            <div>
              <input
                aria-label="基于大V速报提问"
                value={question()}
                onInput={(event) => setQuestion(event.currentTarget.value)}
                placeholder="基于今日观点继续提问…"
              />
              <button disabled={!question().trim()} onClick={send}>发送到对话</button>
            </div>
          </footer>
        </Show>
      </>
    </ResearchPanel>
  );
}
