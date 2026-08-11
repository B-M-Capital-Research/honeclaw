import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicKeyEventChains } from "@/lib/api";
import type { KeyEventChainSnapshot } from "@/lib/types";
import "./key-event-chain-dashboard.css";

type Props = { onAsk: (message: string) => void };
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
const verificationLabel = (value: string) => value === "confirmed" ? "一手确认" : "待核实线索";

export function KeyEventChainDashboard(props: Props) {
  const [snapshot, setSnapshot] = createSignal<KeyEventChainSnapshot>();
  const [open, setOpen] = createSignal(false);
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

  const topic = createMemo(() => snapshot()?.topics.find((item) => item.id === topicId()) ?? snapshot()?.topics[0]);
  const count = createMemo(() => snapshot()?.topics.reduce((sum, item) => sum + item.event_count, 0) ?? 0);
  const confirmedCount = createMemo(() => snapshot()?.topics.reduce((sum, item) => sum + (item.confirmed_count ?? 0), 0) ?? 0);
  const visibleEvents = createMemo(() => {
    const events = topic()?.events ?? [];
    return evidenceFilter() === "confirmed" ? events.filter((event) => event.verification_status === "confirmed") : events;
  });
  const send = () => {
    const report = snapshot();
    const current = topic();
    const query = question().trim();
    if (!report || !current || !query) return;
    const saved = {
      reportDate: report.report_date,
      generatedAtBJT: report.generated_at_beijing,
      status: report.status,
      evidenceFilter: evidenceFilter(),
      topic: current,
    };
    props.onAsk(`<!-- HONE_SAVED_KEY_EVENT_CHAIN
${JSON.stringify(saved)}
END_HONE_SAVED_KEY_EVENT_CHAIN -->
基于已保存的关键事件链回答：${query}
要求：逐条附原文链接和北京时间；只有 verification_status=confirmed 才可称为已确认里程碑，clue 只能作为待核实线索；区分来源事实、作者观点与 HONE 推断；聚合翻译和管理员研究资料不是一手事实；影响待验证时不得补造结论，不得直接转成买卖或仓位动作。`);
    setOpen(false);
    setQuestion("");
  };

  return <>
    <div class="key-chain-launcher">
      <button type="button" onClick={() => setOpen(true)}>
        <span class="key-chain-icon">↗</span>
        <span><strong>关键事件链</strong><small>{loading() ? "整理模型到电力的产业主线…" : snapshot()?.summary ?? "查看主题变化"}</small></span>
        <b>{confirmedCount()}/{count()}</b><i>›</i>
      </button>
    </div>
    <Show when={open()}><Portal><div class="key-chain-backdrop" onClick={() => setOpen(false)}>
      <section class="key-chain-dialog" role="dialog" aria-modal="true" aria-labelledby="key-chain-title" onClick={(event) => event.stopPropagation()}>
        <header><div><p>第一性原理产业图谱</p><h2 id="key-chain-title">关键事件链</h2><span>模型 → 应用 → 数据中心 → 算力 → 光互连 → 存储 → 电力 · 每日 19:55 更新</span></div><button aria-label="关闭关键事件链" onClick={() => setOpen(false)}>×</button></header>
        <div class="key-chain-meta"><b>{statusLabel(snapshot()?.status ?? "")}</b><span>近 {snapshot()?.lookback_days ?? 30} 天</span><span>{snapshot()?.generated_at_beijing ?? "—"} 北京时间</span><button onClick={() => void load()}>重新读取</button></div>
        <nav class="key-chain-topics"><For each={snapshot()?.topics ?? []}>{item => <button classList={{ active: topicId() === item.id }} onClick={() => setTopicId(item.id)}><em>{item.layer || "产业主线"}</em><strong>{item.name}</strong><small>{item.confirmed_count ?? 0} 确认 · {item.clue_count ?? item.event_count} 线索</small></button>}</For></nav>
        <div class="key-chain-body">
          <Show when={!error()} fallback={<div class="key-chain-empty">{error()}</div>}>
            <Show when={topic()}>{current => <>
                <section class="key-chain-topic-head"><div><span>{current().layer || "产业主线"}</span><h3>{current().name}</h3></div><p>{current().description}</p><aside><strong>第一性原理：</strong>{current().first_principle}</aside><strong>{current().latest_change}</strong><nav class="key-chain-evidence-filter" aria-label="证据筛选"><button classList={{ active: evidenceFilter() === "all" }} onClick={() => setEvidenceFilter("all")}>全部原链 {current().event_count}</button><button classList={{ active: evidenceFilter() === "confirmed" }} onClick={() => setEvidenceFilter("confirmed")}>只看一手确认 {current().confirmed_count ?? 0}</button></nav></section>
                <Show when={visibleEvents().length} fallback={<div class="key-chain-empty"><strong>{evidenceFilter() === "confirmed" ? "近 30 天没有一手确认里程碑" : "近 30 天当前来源没有命中事件"}</strong><p>没有事件不等于主题没有变化；待核实线索也不会被自动升级为事实。</p></div>}>
                  <div class="key-chain-timeline"><For each={visibleEvents()}>{event => <article data-verification={event.verification_status}><time>{event.published_at_beijing} 北京时间</time><div><div class="key-chain-event-meta"><b>{event.source_name}</b><span>{changeLabel(event.change_type)}</span><span>{sourceTierLabel(event.source_tier)}</span><span data-verification={event.verification_status}>{verificationLabel(event.verification_status)}</span><span data-direction={event.direction}>{directionLabel(event.direction)}</span></div><h4>{event.title}</h4><p>{event.excerpt}</p><aside class="key-chain-verification"><strong>证据口径：</strong>{event.verification_note}</aside><aside><strong>影响：</strong>{event.impact}</aside><aside><strong>下一验证点：</strong>{event.next_watch}</aside><div class="key-chain-tickers"><For each={event.tickers}>{ticker => <span>${ticker}</span>}</For></div><a href={event.source_url} target="_blank" rel="noreferrer">{event.verification_status === "confirmed" ? "查看一手原文" : "查看线索原文"} ↗</a></div></article>}</For></div>
                </Show>
            </>}</Show>
          </Show>
        </div>
        <footer><p>{snapshot()?.disclaimer}</p><div><input aria-label="基于关键事件链提问" value={question()} onInput={(event) => setQuestion(event.currentTarget.value)} placeholder="这条变化会影响哪些公司？"/><button disabled={!question().trim()} onClick={send}>发送到对话</button></div></footer>
      </section>
    </div></Portal></Show>
  </>;
}
