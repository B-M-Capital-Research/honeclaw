import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicInfluencerDigest } from "@/lib/api";
import type { InfluencerDigestSnapshot } from "@/lib/types";
import "./influencer-digest-dashboard.css";
import "./model-analysis-health.css";

type Props = { onAsk: (message: string) => void };
const stanceLabel = (v: string) => ({ bullish: "偏多", bearish: "偏空", mixed: "多空混合", neutral: "中性", unclear: "待判断" }[v] ?? "待判断");
const postKindLabel = (v: string) => ({ original: "原创", reply: "回复", quote: "引用", retweet: "转推", article: "文章" }[v] ?? "公开内容");
const statusLabel = (v: string) => ({ live: "来源与模型已更新", partial: "部分更新", source_only: "仅原文", no_updates: "今日无更新", source_unconfigured: "部分来源待配置", data_unavailable: "来源读取失败", stale: "上次成功快照" }[v] ?? "等待数据");

export function InfluencerDigestDashboard(props: Props) {
  const [snapshot, setSnapshot] = createSignal<InfluencerDigestSnapshot>();
  const [open, setOpen] = createSignal(false); const [loading, setLoading] = createSignal(true); const [error, setError] = createSignal(""); const [author, setAuthor] = createSignal("all"); const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;
  const load = async () => { controller?.abort(); controller = new AbortController(); setLoading(true); setError(""); try { setSnapshot(await getPublicInfluencerDigest(controller.signal)); } catch (e) { if (e instanceof Error && e.name === "AbortError") return; setError(e instanceof Error ? e.message : "大V速报暂时无法加载"); } finally { setLoading(false); } };
  onMount(() => void load()); onCleanup(() => controller?.abort());
  const visible = createMemo(() => author() === "all" ? snapshot()?.items ?? [] : (snapshot()?.items ?? []).filter((item) => item.author_id === author()));
  const send = () => { const s = snapshot(); const q = question().trim(); if (!s || !q) return; const report = { reportDate: s.report_date, generatedAtBJT: s.generated_at_beijing, status: s.status, analysisHealth: s.analysis_health, authors: s.authors, items: s.items.map(({ author_name, public_handle, title, published_at_beijing, source_url, aggregation_source, aggregation_url, post_kind, summary, stance, horizon, content_type, topics, tickers, counterpoint, analysis_status }) => ({ author_name, public_handle, title, published_at_beijing, source_url, aggregation_source, aggregation_url, post_kind, summary, stance, horizon, content_type, topics, tickers, counterpoint, analysis_status })) }; props.onAsk(`<!-- HONE_SAVED_INFLUENCER_DIGEST\n${JSON.stringify(report)}\nEND_HONE_SAVED_INFLUENCER_DIGEST -->\n基于已保存的大V速报回答：${q}\n要求：作者观点不等于事实或 HONE 结论；附原文链接与北京时间；聚合翻译不是独立事实来源；analysis_status=source_only 时只可引用原文，不得补造立场、反方或投资含义；不得补造未配置作者内容，不得把观点直接转换为买卖或仓位动作。`); setOpen(false); setQuestion(""); };
  return <>
    <div class="influencer-digest-launcher"><button type="button" onClick={() => setOpen(true)}><span class="influencer-digest-icon">V</span><span><strong>大V速报</strong><small>{loading() ? "读取公开观点…" : snapshot()?.summary ?? "查看今日观点"}</small></span><b>{snapshot()?.coverage.items ?? 0}</b><i>›</i></button></div>
    <Show when={open()}><Portal><div class="influencer-digest-backdrop" onClick={() => setOpen(false)}><section class="influencer-digest-dialog" role="dialog" aria-modal="true" aria-labelledby="influencer-title" onClick={(e) => e.stopPropagation()}>
      <header><div><p>先看来源，再看观点</p><h2 id="influencer-title">大V速报</h2><span>原作者观点 × HONE 摘要 × 反方提醒 · 每日 19:50 更新</span></div><button aria-label="关闭大V速报" onClick={() => setOpen(false)}>×</button></header>
      <div class="influencer-digest-meta"><b>{statusLabel(snapshot()?.status ?? "")}</b><span>报告日 {snapshot()?.report_date ?? "—"}</span><span>{snapshot()?.generated_at_beijing ?? "—"} 北京时间</span><span classList={{ "is-blocked": snapshot()?.analysis_health?.decision_use_allowed === false }}>{snapshot()?.analysis_health?.decision_use_allowed ? `模型整理 ${snapshot()?.analysis_health?.analyzed_items}/${snapshot()?.analysis_health?.requested_items}` : "观点整理门禁关闭"}</span><button onClick={() => void load()}>重新读取</button></div>
      <div class="influencer-authors"><button classList={{active:author()==="all"}} onClick={() => setAuthor("all")}>全部</button><For each={snapshot()?.authors ?? []}>{a => <button classList={{active:author()===a.id,missing:!a.configured}} onClick={() => setAuthor(a.id)}>{a.name}<small>{a.configured ? `${a.item_count} 条` : "源待配置"}</small></button>}</For></div>
      <div class="influencer-digest-body"><Show when={!error()} fallback={<div class="influencer-empty">{error()}</div>}><Show when={visible().length} fallback={<div class="influencer-empty"><strong>当前没有可展示的原文更新</strong><p>{snapshot()?.summary}</p></div>}><For each={visible()}>{item => <article><div><b>{item.author_name}</b><span>{item.public_handle}</span><span>{postKindLabel(item.post_kind)}</span><Show when={item.analysis_status !== "model_analyzed"}><span data-analysis="blocked">仅原文</span></Show><time>{item.published_at_beijing} 北京时间</time></div><h3>{item.title}</h3><p>{item.summary || item.source_excerpt}</p><div class="influencer-tags"><span>{stanceLabel(item.stance)}</span><span>{item.content_type === "fact" ? "事实陈述" : item.content_type === "opinion" ? "作者观点" : item.analysis_status === "model_analyzed" ? "事实与观点混合" : "观点类型待分析"}</span><For each={item.topics}>{t => <span>{t}</span>}</For><For each={item.tickers}>{t => <span>${t}</span>}</For></div><aside><strong>反方 / 未证实处：</strong>{item.counterpoint}</aside><a href={item.source_url} target="_blank" rel="noreferrer">查看作者原文 ↗</a><Show when={item.aggregation_url}><a href={item.aggregation_url!} target="_blank" rel="noreferrer">{item.aggregation_source} · 翻译/聚合源 ↗</a></Show></article>}</For></Show></Show></div>
      <footer><p>{snapshot()?.disclaimer}</p><div><input aria-label="基于大V速报提问" value={question()} onInput={e=>setQuestion(e.currentTarget.value)} placeholder="基于今日观点继续提问…"/><button disabled={!question().trim()} onClick={send}>发送到对话</button></div></footer>
    </section></div></Portal></Show>
  </>;
}
