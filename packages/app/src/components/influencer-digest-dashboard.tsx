import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicInfluencerDigest } from "@/lib/api";
import { ResearchPanel } from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type { InfluencerDigestSnapshot } from "@/lib/types";
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

  const send = () => {
    const current = snapshot();
    const query = question().trim();
    if (!current || !query || !props.onAsk) return;
    const report = {
      reportDate: current.report_date,
      generatedAtLocal: current.generated_at_local,
      status: current.status,
      authors: current.authors,
      items: current.items.map(
        ({
          author_name,
          public_handle,
          title,
          published_at_local,
          source_url,
          aggregation_source,
          aggregation_url,
          post_kind,
          summary,
          stance,
          horizon,
          content_type,
          topics,
          tickers,
          counterpoint,
          analysis_status,
        }) => ({
          author_name,
          public_handle,
          title,
          published_at_local,
          source_url,
          aggregation_source,
          aggregation_url,
          post_kind,
          summary,
          stance,
          horizon,
          content_type,
          topics,
          tickers,
          counterpoint,
          analysis_status,
        }),
      ),
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
        <header>
          <div>
            <p>先看来源，再看观点</p>
            <h2 id="influencer-title">大V速报</h2>
            <span>原作者观点 × HONE 摘要 × 反方提醒 · 每日 19:50 更新</span>
          </div>
          <button aria-label="关闭大V速报" onClick={() => props.onClose()}>×</button>
        </header>

        <div class="influencer-digest-meta">
          <b>{statusLabel(snapshot()?.status ?? "")}</b>
          <span>报告日 {snapshot()?.report_date ?? "—"}</span>
          <span>{snapshot()?.generated_at_local ?? "—"} {snapshot()?.timezone}</span>
          <button onClick={() => void load()}>重新读取</button>
        </div>

        <div class="influencer-authors">
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
                  detail={snapshot()?.summary}
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
                      <time>{item.published_at_local} {snapshot()?.timezone}</time>
                    </div>
                    <h3>{item.title}</h3>
                    <p>{item.summary || item.source_excerpt}</p>
                    <div class="influencer-tags">
                      <span>{stanceLabel(item.stance)}</span>
                      <span>
                        {item.content_type === "fact"
                          ? "事实陈述"
                          : item.content_type === "opinion"
                            ? "作者观点"
                            : "事实与观点混合"}
                      </span>
                      <For each={item.topics}>{(topic) => <span>{topic}</span>}</For>
                      <For each={item.tickers}>{(ticker) => <span>${ticker}</span>}</For>
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
