import { Show, createEffect, createSignal, on } from "solid-js";

import { LATEST_STALE_AFTER_DAYS, isLatestStale } from "@/lib/industry-valuation";
import type {
  Industry,
  IndustryEditOp,
  IndustryMember,
  IndustryUpstreamSignal,
} from "@/lib/types";

/**
 * 行业分析页各模块共用的类型、纯助手与三个就地编辑器。页面入口 `../public-industry-map.tsx`
 * 负责状态与顺序，这里不持有任何状态。
 */

/** 一次保存的回执：成功回显后端的一行摘要，失败原样回显后端给的拒绝理由。 */
export type Flash = { kind: "ok" | "error"; text: string };

/**
 * 详情面板里每个可改块共用的一组回调，由页面组件注入。改动说明与「保存中」是全页共用的
 * 状态，所以各块只问「现在能不能存」「存」，不各自握着 note。
 */
export type Editor = {
  /** 编辑模式开着、改动说明已填、且没有别的保存在跑。 */
  canSave: () => boolean;
  busy: () => boolean;
  noteMissing: () => boolean;
  /** 成功返回 true；页面已用返回的快照整体替换本地状态并回显 applied。 */
  submit: (industry: string, op: IndustryEditOp) => Promise<boolean>;
};

/** 行业树只收美股与 ADR：带交易所后缀（0700.HK）或前缀（NYSE:TSM）的代码在前端就拒掉。 */
export const NON_US_SYMBOL_MESSAGE = "只收美股与 ADR";

export function isNonUsSymbol(symbol: string) {
  return symbol.includes(".") || symbol.includes(":");
}

/** 后端与前端分开上线；旧后端还没带这块时按空列表渲染，而不是整页报错。 */
export function upstreamSignals(industry: Industry): IndustryUpstreamSignal[] {
  return industry.upstream_signals ?? [];
}

export function splitLines(text: string) {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

export function splitAliases(text: string) {
  return text
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

/** 市值只用于排序与规模感，给到两位有效小数就够，不做币种换算（树里全是美元计价的美股）。 */
export function marketCap(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "—";
  if (value >= 1e12) return `${(value / 1e12).toFixed(2)} 万亿`;
  if (value >= 1e8) return `${(value / 1e8).toFixed(0)} 亿`;
  return `${(value / 1e8).toFixed(2)} 亿`;
}

/** 改动时间只给到分钟：卡片要的是「什么时候改的」，不是精确时刻。 */
export function editedAt(value: string | undefined) {
  if (!value) return "";
  const at = new Date(value);
  if (Number.isNaN(at.getTime())) return value;
  return at.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function changePercent(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "";
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
}

/**
 * 一段文本的就地编辑：textarea + 保存。草稿只在源值变了（切换行业、或这一段刚被保存）时
 * 跟着重置，别的块保存引起的快照替换不会冲掉正在改的内容。
 */
export function FieldEditor(props: {
  label?: string;
  ariaLabel?: string;
  value: string;
  rows?: number;
  editor: Editor;
  onSave: (value: string) => Promise<boolean>;
}) {
  const [draft, setDraft] = createSignal(props.value);
  createEffect(on(() => props.value, (value) => setDraft(value), { defer: true }));
  const dirty = () => draft().trim() !== props.value.trim();
  return (
    <div class="industry-field">
      <Show when={props.label}>
        <span class="industry-field-label">{props.label}</span>
      </Show>
      <textarea
        class="industry-textarea"
        rows={props.rows ?? 3}
        aria-label={props.ariaLabel ?? props.label}
        value={draft()}
        disabled={props.editor.busy()}
        onInput={(event) => setDraft(event.currentTarget.value)}
      />
      <div class="industry-field-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !dirty()}
          onClick={() => void props.onSave(draft().trim())}
        >
          保存
        </button>
        <Show when={dirty()}>
          <span class="industry-field-dirty">未保存</span>
        </Show>
      </div>
    </div>
  );
}

/**
 * 一组条目的就地编辑：textarea 一行一条 + 保存，整表替换，空行丢掉。
 * 草稿只在源值真的变了（切换行业、或这一列刚被保存）时重置：别的块保存引起的快照替换
 * 会让效果重跑，但拼出来的源文本没变，就不动正在改的内容。
 */
export function ListEditor(props: {
  label: string;
  ariaLabel?: string;
  items: string[];
  rows?: number;
  placeholder?: string;
  editor: Editor;
  onSave: (items: string[]) => Promise<boolean>;
}) {
  const source = () => props.items.join("\n");
  const [draft, setDraft] = createSignal(source());
  let last = source();
  createEffect(
    on(
      source,
      (value) => {
        if (value === last) return;
        last = value;
        setDraft(value);
      },
      { defer: true },
    ),
  );
  const dirty = () => splitLines(draft()).join("\n") !== source();
  return (
    <div class="industry-field">
      <span class="industry-field-label">{props.label}</span>
      <textarea
        class="industry-textarea"
        rows={props.rows ?? 4}
        aria-label={props.ariaLabel ?? props.label}
        placeholder={props.placeholder ?? "一行一条"}
        value={draft()}
        disabled={props.editor.busy()}
        onInput={(event) => setDraft(event.currentTarget.value)}
      />
      <div class="industry-field-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !dirty()}
          onClick={() => void props.onSave(splitLines(draft()))}
        >
          保存
        </button>
        <Show when={dirty()}>
          <span class="industry-field-dirty">未保存</span>
        </Show>
      </div>
    </div>
  );
}

/**
 * 上游信号「最近动作」的就地编辑：一段动作 + 截至日期，两个字段一起存成一个改动。
 * 草稿的重置规则与 FieldEditor 相同：只在源值变了时跟着重置。
 */
export function LatestEditor(props: {
  symbol: string;
  latest: string;
  asOf: string;
  editor: Editor;
  onSave: (latest: string, asOf: string) => Promise<boolean>;
}) {
  const [latest, setLatest] = createSignal(props.latest);
  const [asOf, setAsOf] = createSignal(props.asOf);
  createEffect(on(() => props.latest, (value) => setLatest(value), { defer: true }));
  createEffect(on(() => props.asOf, (value) => setAsOf(value), { defer: true }));
  const dirty = () =>
    latest().trim() !== props.latest.trim() || asOf().trim() !== props.asOf.trim();
  return (
    <div class="industry-field industry-signal-latest-editor">
      <span class="industry-field-label">最近动作</span>
      <textarea
        class="industry-textarea"
        rows={3}
        aria-label={`${props.symbol} 最近动作`}
        placeholder="它最近一次有日期的动作：哪一期、何时发布、关键数字与下季指引"
        value={latest()}
        disabled={props.editor.busy()}
        onInput={(event) => setLatest(event.currentTarget.value)}
      />
      <label class="industry-signal-latest-asof">
        截至
        <input
          class="industry-input"
          aria-label={`${props.symbol} 最近动作截至`}
          placeholder="2026-08-26"
          value={asOf()}
          disabled={props.editor.busy()}
          onInput={(event) => setAsOf(event.currentTarget.value)}
        />
      </label>
      <div class="industry-field-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !dirty()}
          onClick={() => void props.onSave(latest().trim(), asOf().trim())}
        >
          保存
        </button>
        <Show when={dirty()}>
          <span class="industry-field-dirty">未保存</span>
        </Show>
        <Show when={dirty() && props.editor.noteMissing()}>
          <span class="industry-form-hint">先在面板顶部填改动说明</span>
        </Show>
      </div>
    </div>
  );
}

/**
 * 公司视角：选中一家公司后整页围绕它重排。状态由页面入口持有（就是 URL 里的 `?symbol=`），
 * 各区块只读 `member()`、只触发 enter / exit。
 */
export type Lens = {
  member: () => IndustryMember | undefined;
  enter: (symbol: string, options?: { replace?: boolean }) => void;
  exit: () => void;
};

/** 跳到详情里的某个锚点；落在研究底稿里的锚点要先把 <details> 展开再滚，所以不能用裸 #hash。 */
export type DetailJump = (id: string) => void;

/**
 * 「截至 2026-08-26」+ 超过一个季度就标「可能已过期」。上游动作、关注点、简报、来源都用它，
 * 读者在任何一块看到的日期语义一致。空日期不渲染。
 */
export function AsOfTag(props: {
  asOf?: string | null;
  now?: Date;
  prefix?: string;
  staleText?: string;
}) {
  const asOf = () => (props.asOf ?? "").trim();
  return (
    <Show when={asOf()}>
      <span class="industry-asof">
        {props.prefix ?? "截至"} {asOf()}
        <Show when={isLatestStale(asOf(), props.now)}>
          <span
            class="industry-stale"
            title={`日期已超过 ${LATEST_STALE_AFTER_DAYS} 天，引用前先核最近一期`}
          >
            {props.staleText ?? "可能已过期"}
          </span>
        </Show>
      </span>
    </Show>
  );
}

/** 公司视角下，提到了选中公司的条目打这个标；只在读模式前置，编辑态只打标不重排。 */
export function RelatedTag(props: { symbol: string }) {
  return <span class="industry-related-tag">与 {props.symbol} 有关</span>;
}
