import { For, Show, Switch, Match, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { getPublicInfluencerDigest } from "@/lib/api";
import {
  ResearchPanel,
  ResearchPanelHead,
  shortLocalTimestamp,
} from "@/components/research/research-panel";
import {
  ResearchFeed,
  ResearchFeedDay,
  ResearchFeedItem,
} from "@/components/research/research-feed";
import { ResearchState } from "@/components/research/research-state";
import type { InfluencerDigestItem, InfluencerDigestSnapshot } from "@/lib/types";
import "./influencer-digest-dashboard.css";

type Props = {
  onClose: () => void;
};

type KindFilter = "all" | "original" | "media";

const stanceLabel = (v: string) =>
  ({ bullish: "偏多", bearish: "偏空", mixed: "多空混合", neutral: "中性", unclear: "待判断" }[v] ?? "待判断");

const postKindLabel = (v: string) =>
  ({ original: "原创", reply: "回复", quote: "引用", retweet: "转推", article: "文章" }[v] ?? "公开内容");

const statusLabel = (v: string) =>
  ({
    live: "来源与模型已更新",
    partial: "部分更新",
    source_only: "仅原文",
    no_updates: "近 7 天无更新",
    source_unconfigured: "来源待配置",
    data_unavailable: "来源读取失败",
    stale: "同步中断，沿用上次",
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

const compact = (value: number) =>
  value >= 10000 ? `${(value / 1000).toFixed(1)}k` : `${value}`;

/** Reach, as the quiet numbers under a post. */
const stats = (item: InfluencerDigestItem) => {
  const views = item.metrics?.views ?? 0;
  const likes = item.metrics?.likes ?? 0;
  return [views ? `阅读 ${compact(views)}` : undefined, likes ? `赞 ${compact(likes)}` : undefined]
    .filter(Boolean) as string[];
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

/** The circle at the left edge: 白 for 白毛, initials for the rest. */
const avatarOf = (item: InfluencerDigestItem) =>
  ({ serenity: "白", semianalysis: "SA", jukan: "JK" }[item.author_id] ??
    item.author_name.trim().slice(0, 1).toUpperCase());

/** `HH:MM` under a day divider; the divider already says which day. */
const clockOf = (stamp: string) => {
  const match = stamp.match(/(\d{1,2}:\d{2})$/);
  return match ? match[1] : stamp;
};

/** `MM-DD` of a `MM-DD HH:MM` stamp. */
const dayOf = (stamp: string) => shortLocalTimestamp(stamp).slice(0, 5);

/** `MM-DD` for today and yesterday in the runtime's calendar, from the
 *  snapshot's own clock rather than the reader's. */
function dayLabels(generatedAtLocal: string | undefined) {
  const iso = (generatedAtLocal ?? "").slice(0, 10);
  const today = Date.parse(`${iso}T00:00:00Z`);
  if (Number.isNaN(today)) return { today: "", yesterday: "" };
  const format = (ms: number) => new Date(ms).toISOString().slice(5, 10);
  return { today: format(today), yesterday: format(today - 86_400_000) };
}

type Row = { kind: "day"; label: string; count: number } | { kind: "post"; item: InfluencerDigestItem };

export function InfluencerDigestPanel(props: Props) {
  const navigate = useNavigate();
  const [snapshot, setSnapshot] = createSignal<InfluencerDigestSnapshot>();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [author, setAuthor] = createSignal("all");
  const [kind, setKind] = createSignal<KindFilter>("all");
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

  // Only authors with a working source are offered. An author whose bridge
  // does not exist yet is not a filter anyone can use, so it is not shown.
  const authors = createMemo(() =>
    (snapshot()?.authors ?? []).filter((author) => author.configured),
  );

  const visible = createMemo(() =>
    (snapshot()?.items ?? [])
      .filter((item) => author() === "all" || item.author_id === author())
      .filter((item) => {
        if (kind() === "original") return item.post_kind === "original" || item.post_kind === "article";
        if (kind() === "media") return (item.media_urls?.length ?? 0) > 0;
        return true;
      }),
  );

  // A timeline reads by day: 今天 / 昨天 / MM-DD, each divider counting the
  // posts beneath it, so a reader can see at a glance what is new since
  // they last looked.
  const rows = createMemo<Row[]>(() => {
    const labels = dayLabels(snapshot()?.generated_at_local);
    const rows: Row[] = [];
    let currentDay: string | undefined;
    let header: Extract<Row, { kind: "day" }> | undefined;
    for (const item of visible()) {
      const day = dayOf(item.published_at_local);
      if (day !== currentDay) {
        currentDay = day;
        const label = day === labels.today ? "今天" : day === labels.yesterday ? "昨天" : day;
        header = { kind: "day", label, count: 0 };
        rows.push(header);
      }
      if (header) header.count += 1;
      rows.push({ kind: "post", item });
    }
    return rows;
  });

  // Provenance collapses into one secondary line: sync clock, cadence, window
  // and translation source. It used to sit in a separate metadata strip that
  // pushed the actual posts below the fold on phones.
  const metaLine = createMemo(() => {
    const current = snapshot();
    if (!current) return undefined;
    const minutes = current.refresh_interval_minutes ?? 15;
    return [
      current.generated_at_local ? `最近同步 ${current.generated_at_local}` : undefined,
      `每 ${minutes} 分钟同步一次`,
      `近 ${Math.max(1, Math.round((current.lookback_hours ?? 168) / 24))} 天`,
      "中文译文来自 aichainmap 白毛速报，原文以 X 为准",
    ]
      .filter(Boolean)
      .join(" · ");
  });

  /**
   * One post, handed to the assistant with its date and its author's words,
   * so the conversation starts from the exact thing the reader is looking at
   * rather than from "what does Serenity think".
   */
  const askHone = (item: InfluencerDigestItem) => {
    const said = sourceText(item).replace(/\s+/g, " ").trim().slice(0, 160);
    const who = item.author_id === "serenity" ? "Serenity（白毛，@aleabitoreddit）" : item.author_name;
    const targets = item.tickers.length ? item.tickers.map((ticker) => `$${ticker}`).join("、") : "相关公司";
    const question =
      `${who} ${item.published_at_local} 发布：“${said}”。` +
      `请结合本轮可核验的行情、财报与新闻说明：这条在讲什么，哪些是事实、哪些是作者判断，` +
      `对 ${targets} 意味着什么，以及接下来需要验证的数据点。`;
    props.onClose();
    navigate(`/chat?q=${encodeURIComponent(question)}&send=1`);
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
        />

        <div class="influencer-authors research-scroller">
          <button classList={{ active: author() === "all" }} onClick={() => setAuthor("all")}>全部</button>
          <For each={authors()}>
            {(item) => (
              <button
                classList={{ active: author() === item.id, carried: item.carried_over === true }}
                onClick={() => setAuthor(item.id)}
                title={item.carried_over ? "来源本轮读取失败，沿用上次同步的内容" : item.focus}
              >
                {item.name}
                <small>{item.carried_over ? "沿用上次" : `${item.item_count} 条`}</small>
              </button>
            )}
          </For>
          <i class="influencer-authors__gap" aria-hidden="true" />
          <button class="filter" classList={{ active: kind() === "all" }} onClick={() => setKind("all")}>
            全部类型
          </button>
          <button class="filter" classList={{ active: kind() === "original" }} onClick={() => setKind("original")}>
            只看原创
          </button>
          <button class="filter" classList={{ active: kind() === "media" }} onClick={() => setKind("media")}>
            含图
          </button>
        </div>

        <div class="influencer-digest-body">
          <Show when={loading() && !snapshot()}>
            <ResearchState kind="loading" message="正在读取公开观点" detail="只读取已同步的速报，不会触发重新抓取。" />
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
                  detail="换一位作者或类型再看；未配置来源不会被补造内容。"
                />
              }
            >
              <ResearchFeed>
                <For each={rows()}>
                  {(row) => (
                    <Switch>
                      <Match when={row.kind === "day" && row}>
                        {(day) => <ResearchFeedDay label={day().label} count={day().count} />}
                      </Match>
                      <Match when={row.kind === "post" && row}>
                        {(post) => {
                          const item = () => post().item;
                          return (
                            <ResearchFeedItem
                              avatar={avatarOf(item())}
                              author={item().author_name}
                              handle={item().public_handle}
                              kind={item().post_kind === "original" ? undefined : postKindLabel(item().post_kind)}
                              time={clockOf(shortLocalTimestamp(item().published_at_local))}
                              quoted={
                                item().reply_context
                                  ? {
                                      label: `${item().post_kind === "quote" ? "引用" : "回复"} ${item().reply_context!.author}`,
                                      text: item().reply_context!.text,
                                    }
                                  : undefined
                              }
                              media={item().media_urls}
                              mediaAlt={`${item().author_name} 原文配图`}
                              stats={stats(item())}
                              links={[
                                { href: item().source_url, label: "查看作者原文" },
                                ...(item().aggregation_url
                                  ? [{ href: item().aggregation_url!, label: `${item().aggregation_source} · 翻译/聚合源` }]
                                  : []),
                              ]}
                              actions={[{ label: "问 HONE", onClick: () => askHone(item()) }]}
                              analysisLabel="HONE 解读"
                              analysis={
                                item().analysis_status === "model_analyzed" ? (
                                  <>
                                    <Show when={item().summary.trim()}>
                                      <p>{item().summary}</p>
                                    </Show>
                                    <Show when={item().counterpoint.trim()}>
                                      <p>
                                        <b>反方 / 未证实处：</b>
                                        {item().counterpoint}
                                      </p>
                                    </Show>
                                    <p class="influencer-analysis-terms">
                                      {[
                                        stanceLabel(item().stance),
                                        item().content_type === "fact"
                                          ? "事实陈述"
                                          : item().content_type === "opinion"
                                            ? "作者观点"
                                            : "事实与观点混合",
                                        ...item().tickers.map((ticker) => `$${ticker}`),
                                        ...item().topics,
                                      ].join(" · ")}
                                    </p>
                                  </>
                                ) : undefined
                              }
                            >
                              {/* The post itself, in the author's words. The title line
                                  used to repeat this text's first line above it. */}
                              <p>{sourceText(item()) || item().title}</p>
                              <Show when={englishOriginal(item())}>
                                <details class="influencer-source-en">
                                  <summary>English original</summary>
                                  <p>{englishOriginal(item())}</p>
                                </details>
                              </Show>
                            </ResearchFeedItem>
                          );
                        }}
                      </Match>
                    </Switch>
                  )}
                </For>
              </ResearchFeed>
            </Show>
          </Show>
        </div>

        <footer class="influencer-digest-footer">
          <p>{snapshot()?.disclaimer}</p>
        </footer>
      </>
    </ResearchPanel>
  );
}
