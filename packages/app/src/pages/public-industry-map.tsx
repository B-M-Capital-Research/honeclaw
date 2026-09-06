import { Title } from "@solidjs/meta";
import { useNavigate, useSearchParams } from "@solidjs/router";
import {
  For,
  Show,
  batch,
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  onMount,
} from "solid-js";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import {
  ApiError,
  getPublicAuthMe,
  getPublicIndustryMap,
  postPublicIndustryMapEdit,
} from "@/lib/api";
import { resolveIndustryMapSelection } from "@/lib/industry-map-navigation";
import {
  INFERRED_MEMBER_TIP,
  LATEST_STALE_AFTER_DAYS,
  askHoneHref,
  askHonePrompt,
  findMember,
  isInferredMember,
  isLatestStale,
  isSubtypeId,
  matchLabel,
  methodologyOf,
  splitSymbols,
  subtypeOf,
  valuationOf,
} from "@/lib/industry-valuation";
import type { MemberMatch } from "@/lib/industry-valuation";
import { cachedPublicUser, setCachedPublicUser } from "@/lib/public-session-cache";
import type {
  Industry,
  IndustryEditField,
  IndustryEditOp,
  IndustryKeyVariable,
  IndustryMapSnapshot,
  IndustryMember,
  IndustryMethodRule,
  IndustryMethodology,
  IndustrySubtype,
  IndustryUpstreamRelation,
  IndustryUpstreamSignal,
  IndustryValuation,
  IndustryValuationListField,
  IndustryValuationTextField,
  PublicAuthUserInfo,
} from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-industry-map.css";

type ViewState = "loading" | "ready" | "login" | "forbidden" | "error";

/** 一次保存的回执：成功回显后端的一行摘要，失败原样回显后端给的拒绝理由。 */
type Flash = { kind: "ok" | "error"; text: string };

/**
 * 详情面板里每个可改块共用的一组回调，由页面组件注入。改动说明与「保存中」是全页共用的
 * 状态，所以各块只问「现在能不能存」「存」，不各自握着 note。
 */
type Editor = {
  /** 编辑模式开着、改动说明已填、且没有别的保存在跑。 */
  canSave: () => boolean;
  busy: () => boolean;
  noteMissing: () => boolean;
  /** 成功返回 true；页面已用返回的快照整体替换本地状态并回显 applied。 */
  submit: (industry: string, op: IndustryEditOp) => Promise<boolean>;
};

const RELATION_LABELS: Record<IndustryUpstreamRelation, string> = {
  demand_source: "需求来源",
  capex_source: "资本开支来源",
  supply_gate: "供给卡口",
  peer_signal: "同业信号",
};

function relationLabel(value: string) {
  return (RELATION_LABELS as Record<string, string>)[value] ?? value;
}

/** 行业树只收美股与 ADR：带交易所后缀（0700.HK）或前缀（NYSE:TSM）的代码在前端就拒掉。 */
const NON_US_SYMBOL_MESSAGE = "只收美股与 ADR";

function isNonUsSymbol(symbol: string) {
  return symbol.includes(".") || symbol.includes(":");
}

/** 后端与前端分开上线；旧后端还没带这块时按空列表渲染，而不是整页报错。 */
function upstreamSignals(industry: Industry): IndustryUpstreamSignal[] {
  return industry.upstream_signals ?? [];
}

function splitLines(text: string) {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function splitAliases(text: string) {
  return text
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

/** 市值只用于排序与规模感，给到两位有效小数就够，不做币种换算（树里全是美元计价的美股）。 */
function marketCap(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "—";
  if (value >= 1e12) return `${(value / 1e12).toFixed(2)} 万亿`;
  if (value >= 1e8) return `${(value / 1e8).toFixed(0)} 亿`;
  return `${(value / 1e8).toFixed(2)} 亿`;
}

/** 改动时间只给到分钟：卡片要的是「什么时候改的」，不是精确时刻。 */
function editedAt(value: string | undefined) {
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

function changePercent(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "";
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
}

/**
 * 一段文本的就地编辑：textarea + 保存。草稿只在源值变了（切换行业、或这一段刚被保存）时
 * 跟着重置，别的块保存引起的快照替换不会冲掉正在改的内容。
 */
function FieldEditor(props: {
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
function ListEditor(props: {
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
function LatestEditor(props: {
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

function VariablesTable(props: { variables: IndustryKeyVariable[] }) {
  return (
    <table class="industry-variables">
      <thead>
        <tr>
          <th>可观测变量</th>
          <th>它在链条哪一环</th>
          <th>去哪取</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.variables}>
          {(variable) => (
            <tr>
              <td>{variable.name}</td>
              <td>{variable.why}</td>
              <td class="industry-where">{variable.where}</td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}

/** 公司表表尾的「加入公司」。symbol 带 . 或 : 的在这里就拒掉，不必等后端。 */
function AddMemberRow(props: { industry: string; editor: Editor }) {
  const [symbol, setSymbol] = createSignal("");
  const [name, setName] = createSignal("");
  const [role, setRole] = createSignal("");
  const [error, setError] = createSignal("");
  const ready = () => symbol().trim() !== "" && name().trim() !== "";
  const add = async () => {
    const code = symbol().trim().toUpperCase();
    if (isNonUsSymbol(code)) {
      setError(NON_US_SYMBOL_MESSAGE);
      return;
    }
    setError("");
    const ok = await props.editor.submit(props.industry, {
      kind: "add_member",
      member: { symbol: code, name: name().trim(), role: role().trim() },
    });
    if (ok) {
      setSymbol("");
      setName("");
      setRole("");
    }
  };
  return (
    <tr class="industry-members-add">
      <td>
        <input
          class="industry-input"
          aria-label="代码"
          placeholder="代码"
          value={symbol()}
          disabled={props.editor.busy()}
          onInput={(event) => {
            setSymbol(event.currentTarget.value);
            setError("");
          }}
        />
        <Show when={error()}>
          <span class="industry-members-error" role="alert">
            {error()}
          </span>
        </Show>
      </td>
      <td>
        <input
          class="industry-input"
          aria-label="公司"
          placeholder="公司"
          value={name()}
          disabled={props.editor.busy()}
          onInput={(event) => setName(event.currentTarget.value)}
        />
      </td>
      <td class="industry-members-add-hint">子类型稍后指定</td>
      <td class="industry-members-add-hint">加入公司</td>
      <td class="industry-members-add-hint">市值与现价由行情补齐</td>
      <td>
        <input
          class="industry-input"
          aria-label="在这一行的位置"
          placeholder="在这一行的位置"
          value={role()}
          disabled={props.editor.busy()}
          onInput={(event) => setRole(event.currentTarget.value)}
        />
      </td>
      <td>
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !ready()}
          onClick={() => void add()}
        >
          加入
        </button>
      </td>
    </tr>
  );
}

function SignalForm(props: { industry: string; editor: Editor }) {
  const [symbol, setSymbol] = createSignal("");
  const [name, setName] = createSignal("");
  const [relation, setRelation] = createSignal<IndustryUpstreamRelation>("demand_source");
  const [why, setWhy] = createSignal("");
  const [pull, setPull] = createSignal("");
  const [cadence, setCadence] = createSignal("");
  const [latest, setLatest] = createSignal("");
  const [latestAsOf, setLatestAsOf] = createSignal("");
  const ready = () => symbol().trim() !== "";
  const add = async () => {
    const ok = await props.editor.submit(props.industry, {
      kind: "add_upstream_signal",
      signal: {
        symbol: symbol().trim().toUpperCase(),
        name: name().trim(),
        relation: relation(),
        why: why().trim(),
        pull: splitLines(pull()),
        cadence: cadence().trim(),
        latest: latest().trim(),
        latest_as_of: latestAsOf().trim(),
      },
    });
    if (ok) {
      setSymbol("");
      setName("");
      setRelation("demand_source");
      setWhy("");
      setPull("");
      setCadence("");
      setLatest("");
      setLatestAsOf("");
    }
  };
  return (
    <div class="industry-form" role="group" aria-label="新增上游信号">
      <p class="industry-form-title">新增上游信号</p>
      <label>
        代码
        <input
          class="industry-input"
          placeholder="如 NVDA"
          value={symbol()}
          disabled={props.editor.busy()}
          onInput={(event) => setSymbol(event.currentTarget.value)}
        />
      </label>
      <label>
        公司
        <input
          class="industry-input"
          value={name()}
          disabled={props.editor.busy()}
          onInput={(event) => setName(event.currentTarget.value)}
        />
      </label>
      <label>
        关系
        <select
          class="industry-select"
          value={relation()}
          disabled={props.editor.busy()}
          onChange={(event) =>
            setRelation(event.currentTarget.value as IndustryUpstreamRelation)
          }
        >
          <option value="demand_source">{RELATION_LABELS.demand_source}（它买本行的东西）</option>
          <option value="capex_source">{RELATION_LABELS.capex_source}（它的资本开支是需求源头）</option>
          <option value="supply_gate">{RELATION_LABELS.supply_gate}（本行供给受它卡口）</option>
          <option value="peer_signal">{RELATION_LABELS.peer_signal}（同业龙头，最早的景气读数）</option>
        </select>
      </label>
      <label>
        节奏
        <input
          class="industry-input"
          placeholder="如 每季财报后"
          value={cadence()}
          disabled={props.editor.busy()}
          onInput={(event) => setCadence(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        为什么看它
        <textarea
          class="industry-textarea"
          rows={2}
          value={why()}
          disabled={props.editor.busy()}
          onInput={(event) => setWhy(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        最近动作
        <textarea
          class="industry-textarea"
          rows={3}
          placeholder="它最近一次有日期的动作：哪一期、何时发布、关键数字与下季指引"
          value={latest()}
          disabled={props.editor.busy()}
          onInput={(event) => setLatest(event.currentTarget.value)}
        />
      </label>
      <label>
        截至
        <input
          class="industry-input"
          placeholder="2026-08-26"
          value={latestAsOf()}
          disabled={props.editor.busy()}
          onInput={(event) => setLatestAsOf(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        去取它的哪几个读数（一行一条）
        <textarea
          class="industry-textarea"
          rows={3}
          value={pull()}
          disabled={props.editor.busy()}
          onInput={(event) => setPull(event.currentTarget.value)}
        />
      </label>
      <div class="industry-form-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !ready()}
          onClick={() => void add()}
        >
          新增
        </button>
      </div>
    </div>
  );
}

function WatchForm(props: { industry: string; editor: Editor }) {
  const [what, setWhat] = createSignal("");
  const [why, setWhy] = createSignal("");
  const [cadence, setCadence] = createSignal("");
  const ready = () => what().trim() !== "";
  const add = async () => {
    const ok = await props.editor.submit(props.industry, {
      kind: "add_watch",
      watch: { what: what().trim(), why: why().trim(), cadence: cadence().trim() },
    });
    if (ok) {
      setWhat("");
      setWhy("");
      setCadence("");
    }
  };
  return (
    <div class="industry-form" role="group" aria-label="新增关注点">
      <p class="industry-form-title">新增关注点</p>
      <label>
        看什么
        <input
          class="industry-input"
          value={what()}
          disabled={props.editor.busy()}
          onInput={(event) => setWhat(event.currentTarget.value)}
        />
      </label>
      <label>
        节奏
        <input
          class="industry-input"
          placeholder="如 每季 / 每月"
          value={cadence()}
          disabled={props.editor.busy()}
          onInput={(event) => setCadence(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        为什么
        <textarea
          class="industry-textarea"
          rows={2}
          value={why()}
          disabled={props.editor.busy()}
          onInput={(event) => setWhy(event.currentTarget.value)}
        />
      </label>
      <div class="industry-form-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !ready()}
          onClick={() => void add()}
        >
          新增
        </button>
      </div>
    </div>
  );
}

function SourceForm(props: { industry: string; editor: Editor }) {
  const [house, setHouse] = createSignal("");
  const [title, setTitle] = createSignal("");
  const [date, setDate] = createSignal("");
  const [url, setUrl] = createSignal("");
  const [takeaway, setTakeaway] = createSignal("");
  // url 是这条来源的身份（移除按它找），所以和机构、标题一起必填。
  const ready = () => house().trim() !== "" && title().trim() !== "" && url().trim() !== "";
  const add = async () => {
    const ok = await props.editor.submit(props.industry, {
      kind: "add_source",
      source: {
        house: house().trim(),
        title: title().trim(),
        date: date().trim(),
        url: url().trim(),
        takeaway: takeaway().trim(),
      },
    });
    if (ok) {
      setHouse("");
      setTitle("");
      setDate("");
      setUrl("");
      setTakeaway("");
    }
  };
  return (
    <div class="industry-form" role="group" aria-label="新增来源">
      <p class="industry-form-title">新增来源</p>
      <label>
        机构
        <input
          class="industry-input"
          value={house()}
          disabled={props.editor.busy()}
          onInput={(event) => setHouse(event.currentTarget.value)}
        />
      </label>
      <label>
        标题
        <input
          class="industry-input"
          value={title()}
          disabled={props.editor.busy()}
          onInput={(event) => setTitle(event.currentTarget.value)}
        />
      </label>
      <label>
        日期
        <input
          class="industry-input"
          placeholder="YYYY-MM-DD"
          value={date()}
          disabled={props.editor.busy()}
          onInput={(event) => setDate(event.currentTarget.value)}
        />
      </label>
      <label>
        链接
        <input
          class="industry-input"
          placeholder="https://"
          value={url()}
          disabled={props.editor.busy()}
          onInput={(event) => setUrl(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        要点
        <textarea
          class="industry-textarea"
          rows={2}
          value={takeaway()}
          disabled={props.editor.busy()}
          onInput={(event) => setTakeaway(event.currentTarget.value)}
        />
      </label>
      <div class="industry-form-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !ready()}
          onClick={() => void add()}
        >
          新增
        </button>
      </div>
    </div>
  );
}

/** 树顶部的「新增行业」。请求体的 industry 就是新 id，成功后页面切到它。 */
function IndustryForm(props: { editor: Editor }) {
  const [open, setOpen] = createSignal(false);
  const [id, setId] = createSignal("");
  const [name, setName] = createSignal("");
  const [oneLiner, setOneLiner] = createSignal("");
  const [aliases, setAliases] = createSignal("");
  const ready = () => id().trim() !== "" && name().trim() !== "";
  const add = async () => {
    const newId = id().trim();
    const ok = await props.editor.submit(newId, {
      kind: "add_industry",
      industry: {
        id: newId,
        name: name().trim(),
        one_liner: oneLiner().trim(),
        aliases: splitAliases(aliases()),
      },
    });
    if (ok) {
      setId("");
      setName("");
      setOneLiner("");
      setAliases("");
      setOpen(false);
    }
  };
  return (
    <div class="industry-tree-add">
      <button
        type="button"
        class="industry-btn"
        aria-expanded={open()}
        onClick={() => setOpen((value) => !value)}
      >
        {open() ? "收起" : "新增行业"}
      </button>
      <Show when={open()}>
        <div class="industry-form" role="group" aria-label="新增行业">
          <label>
            id
            <input
              class="industry-input"
              placeholder="如 optics"
              value={id()}
              disabled={props.editor.busy()}
              onInput={(event) => setId(event.currentTarget.value)}
            />
          </label>
          <label>
            名称
            <input
              class="industry-input"
              value={name()}
              disabled={props.editor.busy()}
              onInput={(event) => setName(event.currentTarget.value)}
            />
          </label>
          <label>
            一句话
            <textarea
              class="industry-textarea"
              rows={2}
              value={oneLiner()}
              disabled={props.editor.busy()}
              onInput={(event) => setOneLiner(event.currentTarget.value)}
            />
          </label>
          <label>
            别名（逗号分隔）
            <input
              class="industry-input"
              value={aliases()}
              disabled={props.editor.busy()}
              onInput={(event) => setAliases(event.currentTarget.value)}
            />
          </label>
          <div class="industry-form-actions">
            <button
              type="button"
              class="industry-btn is-primary"
              disabled={!props.editor.canSave() || !ready()}
              onClick={() => void add()}
            >
              保存
            </button>
            <Show when={props.editor.noteMissing()}>
              <span class="industry-form-hint">先在右侧面板顶部填改动说明</span>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}

/**
 * 方法论条：前瞻估值执行版的通用部分，所有行共用，所以只在详情区顶部放一次。
 * 旧后端没带方法论（执行规则为空）时整条不渲染，页面退回逐行内容。
 */
function MethodologyStrip(props: { methodology: IndustryMethodology }) {
  const method = () => props.methodology;
  return (
    <Show when={method().execution_rules.length > 0}>
      <section class="industry-method" aria-label="估值方法论">
        <h2>HOne 前瞻估值执行版 {method().version}</h2>
        <Show when={method().positioning}>
          <p class="industry-method-lead">{method().positioning}</p>
        </Show>
        <Show when={method().demand_chain.length > 0}>
          <ol class="industry-method-chain" aria-label="需求链">
            <For each={method().demand_chain}>
              {(step) => (
                <li>
                  <span class="industry-method-step">{step}</span>
                </li>
              )}
            </For>
          </ol>
        </Show>
        <Show when={method().core_principle}>
          <p class="industry-callout">
            <span class="industry-callout-label">核心原则</span>
            {method().core_principle}
          </p>
        </Show>
        <details class="industry-details">
          <summary>通用执行规则</summary>
          <RuleTable head={["规则", "执行要求"]} rows={method().execution_rules} />
        </details>
        <Show when={method().output_fields.length > 0}>
          <details class="industry-details">
            <summary>强制输出字段</summary>
            <RuleTable head={["字段", "要求"]} rows={method().output_fields} />
          </details>
        </Show>
        <Show when={method().hindsight_error}>
          <p class="industry-callout is-warn">
            <span class="industry-callout-label">最常见的后视镜错误</span>
            {method().hindsight_error}
          </p>
        </Show>
      </section>
    </Show>
  );
}

/** 「规则 → 要求」两列表，执行规则与输出字段共用；在自己的容器里横向滚动。 */
function RuleTable(props: { head: [string, string]; rows: IndustryMethodRule[] }) {
  return (
    <div class="industry-table-scroll">
      <table class="industry-rules">
        <thead>
          <tr>
            <th>{props.head[0]}</th>
            <th>{props.head[1]}</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.rows}>
            {(row) => (
              <tr>
                <td>{row.rule}</td>
                <td>{row.requirement}</td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}

/**
 * 子类型表单：新增（seed 为 null）与编辑（seed 为现有子类型，预填）共用一份。
 * 提交的是整条 upsert：编辑时把底稿推断的成员原样带回去（去掉这次已明确归入的），
 * 否则整体替换会把推断成员丢掉。id 是子类型的身份，编辑时锁住——改 id 等于新建。
 */
function SubtypeForm(props: {
  industry: string;
  seed: IndustrySubtype | null;
  editor: Editor;
  onSaved: (id: string) => void;
  onCancel: () => void;
}) {
  const [id, setId] = createSignal(props.seed?.id ?? "");
  const [name, setName] = createSignal(props.seed?.name ?? "");
  const [members, setMembers] = createSignal((props.seed?.members ?? []).join("\n"));
  const [primary, setPrimary] = createSignal(props.seed?.primary ?? "");
  const [secondary, setSecondary] = createSignal(props.seed?.secondary ?? "");
  const [when, setWhen] = createSignal(props.seed?.when ?? "");
  const [note, setNote] = createSignal(props.seed?.note ?? "");
  const [error, setError] = createSignal("");
  // 「编辑」换了一张卡时整张表单跟着换种子，不留上一张的草稿。
  createEffect(
    on(
      () => props.seed,
      (seed) => {
        setId(seed?.id ?? "");
        setName(seed?.name ?? "");
        setMembers((seed?.members ?? []).join("\n"));
        setPrimary(seed?.primary ?? "");
        setSecondary(seed?.secondary ?? "");
        setWhen(seed?.when ?? "");
        setNote(seed?.note ?? "");
        setError("");
      },
      { defer: true },
    ),
  );
  const editing = () => props.seed !== null;
  const ready = () => id().trim() !== "" && name().trim() !== "";
  const save = async () => {
    const code = id().trim();
    if (!isSubtypeId(code)) {
      setError("id 只能用小写字母、数字与连字符，如 pure-nand");
      return;
    }
    const symbols = splitSymbols(members());
    const foreign = symbols.find(isNonUsSymbol);
    if (foreign) {
      setError(`${foreign}：${NON_US_SYMBOL_MESSAGE}`);
      return;
    }
    setError("");
    const ok = await props.editor.submit(props.industry, {
      kind: "upsert_subtype",
      subtype: {
        id: code,
        name: name().trim(),
        members: symbols,
        inferred_members: (props.seed?.inferred_members ?? []).filter(
          (symbol) => !symbols.includes(symbol.trim().toUpperCase()),
        ),
        primary: primary().trim(),
        secondary: secondary().trim(),
        when: when().trim(),
        note: note().trim(),
      },
    });
    if (ok) props.onSaved(code);
  };
  return (
    <div class="industry-form" role="group" aria-label={editing() ? "编辑子类型" : "新增子类型"}>
      <p class="industry-form-title">
        {editing() ? `编辑子类型「${props.seed?.name ?? ""}」` : "新增子类型"}
      </p>
      <label>
        id
        <input
          class="industry-input"
          placeholder="如 pure-nand"
          value={id()}
          disabled={props.editor.busy() || editing()}
          title={editing() ? "id 是子类型的身份，改 id 等于新建一个" : undefined}
          onInput={(event) => {
            setId(event.currentTarget.value);
            setError("");
          }}
        />
      </label>
      <label>
        名称
        <input
          class="industry-input"
          placeholder="如 纯NAND/eSSD"
          value={name()}
          disabled={props.editor.busy()}
          onInput={(event) => setName(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        成员代码（一行一个，或逗号分隔）
        <textarea
          class="industry-textarea"
          rows={3}
          placeholder="SNDK, WDC"
          value={members()}
          disabled={props.editor.busy()}
          onInput={(event) => {
            setMembers(event.currentTarget.value);
            setError("");
          }}
        />
      </label>
      <label class="is-wide">
        主锚
        <textarea
          class="industry-textarea"
          rows={2}
          placeholder="哪个前瞻财年、哪一族倍数"
          value={primary()}
          disabled={props.editor.busy()}
          onInput={(event) => setPrimary(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        次锚
        <textarea
          class="industry-textarea"
          rows={2}
          value={secondary()}
          disabled={props.editor.busy()}
          onInput={(event) => setSecondary(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        适用阶段
        <textarea
          class="industry-textarea"
          rows={2}
          placeholder="在哪个 State 下用它"
          value={when()}
          disabled={props.editor.busy()}
          onInput={(event) => setWhen(event.currentTarget.value)}
        />
      </label>
      <label class="is-wide">
        提醒
        <textarea
          class="industry-textarea"
          rows={2}
          value={note()}
          disabled={props.editor.busy()}
          onInput={(event) => setNote(event.currentTarget.value)}
        />
      </label>
      <div class="industry-form-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !ready()}
          onClick={() => void save()}
        >
          {editing() ? "保存" : "新增"}
        </button>
        <button
          type="button"
          class="industry-btn"
          disabled={props.editor.busy()}
          onClick={() => props.onCancel()}
        >
          取消
        </button>
        <Show when={error()}>
          <span class="industry-form-error" role="alert">
            {error()}
          </span>
        </Show>
        <Show when={props.editor.noteMissing()}>
          <span class="industry-form-hint">先在面板顶部填改动说明</span>
        </Show>
      </div>
    </div>
  );
}

/**
 * 估值执行卡：这一行拆成哪些子类型，选中的那一个用什么主锚 / 次锚、适用什么阶段、
 * 有哪些成员；下面三行是行级锚——倍数上沿由什么决定、盈利上修期权在哪、什么被禁止。
 * 读者先在这里定位公司，再去看上游最近做了什么。
 */
function ValuationCard(props: {
  industry: Industry;
  valuation: IndustryValuation;
  active: IndustrySubtype | undefined;
  onSelect: (id: string) => void;
  editMode: boolean;
  editor: Editor;
}) {
  // 表单种子：undefined 关着；null 新增；对象 = 预填编辑。
  const [seed, setSeed] = createSignal<IndustrySubtype | null | undefined>(undefined);
  const subtypes = () => props.valuation.subtypes;
  const anchor = () => props.valuation.anchor;
  const isActive = (subtype: IndustrySubtype) => props.active?.id === subtype.id;
  const memberCount = (subtype: IndustrySubtype) =>
    subtype.members.length;
  const setText = (field: IndustryValuationTextField, value: string) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_field", field, value });
  const setList = (field: IndustryValuationListField, items: string[]) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_list", field, items });
  const remove = async (subtype: IndustrySubtype) => {
    const confirmed = window.confirm(
      `确定移除子类型「${subtype.name}」？它的成员会回到未归入状态；这次移除会记进改动日志。`,
    );
    if (!confirmed) return;
    const ok = await props.editor.submit(props.industry.id, {
      kind: "remove_subtype",
      id: subtype.id,
    });
    if (ok && seed()?.id === subtype.id) setSeed(undefined);
  };
  // 编辑模式关掉时表单也收起，别把一张半填的表单留到读模式。
  createEffect(
    on(
      () => props.editMode,
      (enabled) => {
        if (!enabled) setSeed(undefined);
      },
      { defer: true },
    ),
  );
  return (
    <div class="industry-valuation-card">
      <Show when={subtypes().length === 0}>
        <p class="industry-detail-note">这一行尚未拆子类型，按下面的行级锚执行。</p>
      </Show>
      <Show when={subtypes().length > 0 || props.editMode}>
        <div class="industry-subtype-chips" aria-label="子类型">
          <For each={subtypes()}>
            {(subtype) => (
              <button
                type="button"
                class="industry-subtype-chip"
                classList={{ "is-active": isActive(subtype) }}
                aria-pressed={isActive(subtype)}
                onClick={() => props.onSelect(subtype.id)}
              >
                {subtype.name}
                <span class="industry-subtype-count">{memberCount(subtype)}</span>
              </button>
            )}
          </For>
          <Show when={props.editMode}>
            <button
              type="button"
              class="industry-btn"
              aria-expanded={seed() === null}
              onClick={() => setSeed((value) => (value === null ? undefined : null))}
            >
              {seed() === null ? "收起" : "新增子类型"}
            </button>
          </Show>
        </div>
      </Show>
      <Show when={props.active}>
        {(subtype) => (
          <div class="industry-subtype-card">
            <div class="industry-subtype-head">
              <strong>{subtype().name}</strong>
              <code class="industry-subtype-id">{subtype().id}</code>
              <Show when={props.editMode}>
                <span class="industry-subtype-actions">
                  <button
                    type="button"
                    class="industry-btn"
                    disabled={props.editor.busy()}
                    onClick={() => setSeed(subtype())}
                  >
                    编辑
                  </button>
                  <button
                    type="button"
                    class="industry-btn is-danger"
                    disabled={!props.editor.canSave()}
                    onClick={() => void remove(subtype())}
                  >
                    移除子类型
                  </button>
                </span>
              </Show>
            </div>
            <dl class="industry-subtype-grid">
              <dt>主锚</dt>
              <dd>{subtype().primary || "—"}</dd>
              <dt>次锚</dt>
              <dd>{subtype().secondary || "—"}</dd>
              <dt>适用阶段</dt>
              <dd>{subtype().when || "—"}</dd>
              <dt>提醒</dt>
              <dd>{subtype().note || "—"}</dd>
            </dl>
            <Show when={memberCount(subtype()) > 0}>
              <div class="industry-subtype-members" aria-label="成员">
                <For each={subtype().members}>
                  {(symbol) => <span class="industry-member-chip">{symbol}</span>}
                </For>
                <For each={subtype().inferred_members}>
                  {(symbol) => (
                    <span class="industry-member-chip is-inferred">
                      {symbol}
                      <span class="industry-inferred-tag" title={INFERRED_MEMBER_TIP}>
                        推断
                      </span>
                    </span>
                  )}
                </For>
              </div>
            </Show>
          </div>
        )}
      </Show>
      <Show when={props.editMode && seed() !== undefined}>
        <SubtypeForm
          industry={props.industry.id}
          seed={seed() ?? null}
          editor={props.editor}
          onSaved={(id) => {
            setSeed(undefined);
            props.onSelect(id);
          }}
          onCancel={() => setSeed(undefined)}
        />
      </Show>
      <Show
        when={props.editMode}
        fallback={
          <dl class="industry-anchor-rows">
            <dt>倍数上沿由</dt>
            <dd>{anchor().upper_range_drivers || "—"}</dd>
            <dt>盈利上修期权</dt>
            <dd>{anchor().revision_optionality || "—"}</dd>
            <dt>禁止</dt>
            <dd>
              <Show when={anchor().forbidden.length > 0} fallback="—">
                <ul class="industry-forbidden">
                  <For each={anchor().forbidden}>{(item) => <li>{item}</li>}</For>
                </ul>
              </Show>
            </dd>
          </dl>
        }
      >
        <FieldEditor
          label="倍数上沿由"
          value={anchor().upper_range_drivers}
          rows={2}
          editor={props.editor}
          onSave={(value) => setText("anchor.upper_range_drivers", value)}
        />
        <FieldEditor
          label="盈利上修期权"
          value={anchor().revision_optionality}
          rows={2}
          editor={props.editor}
          onSave={(value) => setText("anchor.revision_optionality", value)}
        />
        <ListEditor
          label="禁止（一行一条）"
          items={anchor().forbidden}
          rows={3}
          editor={props.editor}
          onSave={(items) => setList("anchor.forbidden", items)}
        />
      </Show>
    </div>
  );
}

/**
 * 底层估值逻辑：一段话、公式、未来 1–3 年先看什么；原文与倍数锚原文折叠在后面，
 * 读者要核出处时再展开。公式没有改动入口，改它要动底稿。
 */
function ValuationLogicBlock(props: {
  industry: Industry;
  valuation: IndustryValuation;
  editMode: boolean;
  editor: Editor;
}) {
  const logic = () => props.valuation.logic;
  const anchor = () => props.valuation.anchor;
  const setText = (field: IndustryValuationTextField, value: string) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_field", field, value });
  const setList = (field: IndustryValuationListField, items: string[]) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_list", field, items });
  return (
    <div class="industry-valuation-logic">
      <Show
        when={props.editMode}
        fallback={
          <Show
            when={logic().summary}
            fallback={<p class="industry-detail-note">这一行的前瞻估值逻辑尚未定稿。</p>}
          >
            <p class="industry-valuation-summary">{logic().summary}</p>
          </Show>
        }
      >
        <FieldEditor
          label="一段话"
          value={logic().summary}
          rows={4}
          editor={props.editor}
          onSave={(value) => setText("logic.summary", value)}
        />
      </Show>
      <Show when={logic().formulas.length > 0}>
        <ul class="industry-formulas" aria-label="公式">
          <For each={logic().formulas}>
            {(item) => (
              <li>
                <code class="industry-formula">{item.formula}</code>
                <Show when={item.note}>
                  <span class="industry-formula-note">{item.note}</span>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <Show
        when={props.editMode}
        fallback={
          <Show when={logic().forward_focus.length > 0}>
            <h4 class="industry-subhead">未来 1–3 年先看</h4>
            <ol class="industry-forward-focus">
              <For each={logic().forward_focus}>{(item) => <li>{item}</li>}</For>
            </ol>
          </Show>
        }
      >
        <ListEditor
          label="未来 1–3 年先看（一行一条）"
          items={logic().forward_focus}
          editor={props.editor}
          onSave={(items) => setList("logic.forward_focus", items)}
        />
      </Show>
      <details class="industry-details">
        <summary>原文</summary>
        <Show
          when={props.editMode}
          fallback={
            <Show
              when={logic().paragraphs.length > 0}
              fallback={<p class="industry-detail-note">尚未定稿。</p>}
            >
              <For each={logic().paragraphs}>
                {(paragraph) => <p class="industry-paragraph">{paragraph}</p>}
              </For>
            </Show>
          }
        >
          <ListEditor
            label="原文（一行一段）"
            items={logic().paragraphs}
            rows={8}
            editor={props.editor}
            onSave={(items) => setList("logic.paragraphs", items)}
          />
        </Show>
      </details>
      <details class="industry-details">
        <summary>倍数锚原文</summary>
        <Show
          when={props.editMode}
          fallback={
            <Show
              when={anchor().paragraphs.length > 0}
              fallback={<p class="industry-detail-note">尚未定稿。</p>}
            >
              <For each={anchor().paragraphs}>
                {(paragraph) => <p class="industry-paragraph">{paragraph}</p>}
              </For>
            </Show>
          }
        >
          <ListEditor
            label="倍数锚原文（一行一段）"
            items={anchor().paragraphs}
            rows={8}
            editor={props.editor}
            onSave={(items) => setList("anchor.paragraphs", items)}
          />
        </Show>
      </details>
    </div>
  );
}

/** 公司表编辑列的子类型下拉：只认明确归入；推断成员显示为「未确认」，管理员选一下就是确认。 */
function MemberSubtypeSelect(props: { industry: Industry; symbol: string; editor: Editor }) {
  const subtypes = () => valuationOf(props.industry).subtypes;
  const current = () => subtypeOf(props.industry, props.symbol);
  const inferred = () => {
    const subtype = current();
    return subtype && isInferredMember(subtype, props.symbol) ? subtype : undefined;
  };
  const value = () => {
    const subtype = current();
    return subtype && !isInferredMember(subtype, props.symbol) ? subtype.id : "";
  };
  return (
    <select
      class="industry-select"
      aria-label={`${props.symbol} 子类型`}
      value={value()}
      disabled={!props.editor.canSave() || subtypes().length === 0}
      onChange={(event) => {
        const id = event.currentTarget.value;
        if (!id) return;
        void props.editor.submit(props.industry.id, {
          kind: "set_member_subtype",
          symbol: props.symbol,
          subtype: id,
        });
      }}
    >
      <option value="">{inferred() ? `未确认（推断为 ${inferred()?.name ?? ""}）` : "未归入"}</option>
      <For each={subtypes()}>{(subtype) => <option value={subtype.id}>{subtype.name}</option>}</For>
    </select>
  );
}

export default function PublicIndustryMapPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [view, setView] = createSignal<ViewState>("loading");
  const [snapshot, setSnapshot] = createSignal<IndustryMapSnapshot>();
  // The URL owns selection so incoming 3D links, refresh and browser history agree.
  const selected = createMemo(() =>
    resolveIndustryMapSelection(snapshot()?.industries ?? [], searchParams.industry),
  );
  const selectIndustry = (id: string | undefined, replace = false) => {
    const next = resolveIndustryMapSelection(snapshot()?.industries ?? [], id);
    if (searchParams.industry === next) return;
    setSearchParams({ industry: next }, { replace, scroll: false });
  };
  // Normalize stale links only after an authorized snapshot is available. Missing
  // parameters keep the existing default view without adding a history entry.
  createEffect(() => {
    if (!snapshot() || searchParams.industry === undefined) return;
    if (searchParams.industry !== selected()) {
      setSearchParams({ industry: selected() }, { replace: true, scroll: false });
    }
  });
  const [error, setError] = createSignal("");
  // 编辑本体：开关、所有保存共用的改动说明、保存中、最近一次保存的回执。
  const [editing, setEditing] = createSignal(false);
  const [note, setNote] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [flash, setFlash] = createSignal<Flash>();
  let controller: AbortController | undefined;
  // 找公司：输入即匹配；命中后 URL 跟着切到那一行（replace，不堆历史），估值执行卡切到它的
  // 子类型，公司表里那一行亮几秒。
  const [query, setQuery] = createSignal("");
  const [match, setMatch] = createSignal<MemberMatch>();
  const [highlight, setHighlight] = createSignal<string>();
  // 估值执行卡里选中的子类型，按行业记：切到别的行业时自然失效，回到第一个。
  const [subtypePick, setSubtypePick] = createSignal<{ industry: string; id: string }>();
  let highlightTimer: ReturnType<typeof setTimeout> | undefined;
  const pickSubtype = (industry: string, id: string) => setSubtypePick({ industry, id });
  const flashRow = (symbol: string) => {
    setHighlight(symbol);
    clearTimeout(highlightTimer);
    highlightTimer = setTimeout(() => setHighlight(undefined), 4000);
  };
  const runSearch = (text: string) => {
    setQuery(text);
    const hit = findMember(snapshot()?.industries ?? [], text);
    setMatch(hit);
    if (!hit) return;
    batch(() => {
      selectIndustry(hit.industry.id, true);
      if (hit.subtype) pickSubtype(hit.industry.id, hit.subtype.id);
      flashRow(hit.member.symbol);
    });
    // 行业切换后公司表才重画，等一拍再把那一行滚进视野。
    setTimeout(() => {
      document
        .querySelector<HTMLElement>(`.industry-members tr[data-symbol="${hit.member.symbol}"]`)
        ?.scrollIntoView({ block: "center", behavior: "smooth" });
    }, 60);
  };
  /** 「问 HONE」：带着这一行的本体去开一轮前瞻估值；没归入子类型的公司明说，让模型按行级锚走。 */
  const askHone = (industry: Industry, member: IndustryMember) => {
    const subtype = subtypeOf(industry, member.symbol);
    navigate(
      askHoneHref(askHonePrompt(member.symbol, member.name, subtype?.name ?? "未分子类型")),
    );
  };

  const bootstrap = async () => {
    try {
      const me = await getPublicAuthMe();
      setUser(me);
      setCachedPublicUser(me);
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setUser(null);
      setCachedPublicUser(null);
      setView("login");
      return;
    }
    await load();
  };

  const load = async () => {
    if (!user()) {
      setView("login");
      return;
    }
    controller?.abort();
    controller = new AbortController();
    setError("");
    try {
      const data = await getPublicIndustryMap(controller.signal);
      setSnapshot(data);
      setView("ready");
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      if (cause instanceof ApiError && cause.status === 401) setView("login");
      else if (cause instanceof ApiError && cause.status === 403) setView("forbidden");
      else {
        setError(cause instanceof Error ? cause.message : String(cause));
        setView("error");
      }
    }
  };

  onMount(() => void bootstrap());
  onCleanup(() => {
    controller?.abort();
    clearTimeout(highlightTimer);
  });

  const current = createMemo<Industry | undefined>(() =>
    snapshot()?.industries.find((item) => item.id === selected()),
  );
  const valuation = createMemo(() => valuationOf(current()));
  /** 估值执行卡里选中的子类型：按行业记的选择只在还是这一行时生效，否则回到第一个。 */
  const activeSubtype = createMemo(() => {
    const industry = current();
    const subtypes = valuation().subtypes;
    const pick = subtypePick();
    const picked =
      industry && pick && pick.industry === industry.id
        ? subtypes.find((subtype) => subtype.id === pick.id)
        : undefined;
    return picked ?? subtypes[0];
  });

  /** 开关只在后端说 is_admin 时渲染，这里再核一次快照，而不是只信本地开关。 */
  const editMode = () => editing() && snapshot()?.is_admin === true;

  const editor: Editor = {
    canSave: () => editMode() && note().trim() !== "" && !busy(),
    busy,
    noteMissing: () => note().trim() === "",
    submit: async (industry, op) => {
      const why = note().trim();
      if (!editMode() || !why || busy()) return false;
      setBusy(true);
      setFlash(undefined);
      try {
        const result = await postPublicIndustryMapEdit({ industry, note: why, op });
        // 整体替换：「最近改动」卡片与树上的红点跟着一起更新。保存成功本身就证明是管理员，
        // 快照万一漏了 is_admin 也不能让开关消失。快照与选中项一批提交：移除行业时若分两步，
        // 中间那一刻选中项指向已不存在的行业，整个详情面板会先卸载再重建。
        batch(() => {
          setSnapshot({ ...result.snapshot, is_admin: result.snapshot.is_admin ?? true });
          if (op.kind === "add_industry") selectIndustry(op.industry.id);
          else if (op.kind === "remove_industry") selectIndustry(result.snapshot.industries[0]?.id, true);
          setFlash({ kind: "ok", text: result.applied });
        });
        return true;
      } catch (cause) {
        if (cause instanceof Error && cause.name === "AbortError") return false;
        if (cause instanceof ApiError && cause.status === 401) {
          setView("login");
          return false;
        }
        setFlash({
          kind: "error",
          text: cause instanceof Error ? cause.message : String(cause),
        });
        return false;
      } finally {
        setBusy(false);
      }
    },
  };

  const setField = (industry: string, field: IndustryEditField, value: string) =>
    editor.submit(industry, { kind: "set_field", field, value });

  const removeIndustry = async (industry: Industry) => {
    const confirmed = window.confirm(
      `确定从行业树移除「${industry.name}」？底稿不动，这次移除会记进改动日志，之后仍可恢复。`,
    );
    if (!confirmed) return;
    await editor.submit(industry.id, { kind: "remove_industry" });
  };

  return (
    <>
      <Title>行业分析 · HONE</Title>
      <Show
        when={view() !== "loading"}
        fallback={<div class="industry-map-loading" role="status">正在读取行业树…</div>}
      >
        <Show
          when={view() !== "login"}
          fallback={
            <PublicLoginForm
              title="登录后查看行业分析"
              subtitle="行业树是研究结构与先验，不是当前事实或买卖建议。"
              onLogin={() => void bootstrap()}
            />
          }
        >
          <PublicWorkspaceShell active="research" topbarLabel="行业分析">
            <Show
              when={view() !== "forbidden"}
              fallback={
                <p class="industry-map-empty">行业分析仅管理员可见，当前账号没有查看权限。</p>
              }
            >
              <Show
                when={view() !== "error"}
                fallback={<p class="industry-map-empty">读取失败：{error()}</p>}
              >
                <Show when={snapshot()}>
                  {(data) => (
                    <div class="industry-map">
                      <header class="industry-map-head">
                        <h1>{data().root.name}</h1>
                        <p>{data().root.summary}</p>
                        <p class="industry-map-meta">
                          研究底稿更新：{data().generated_at}
                          <Show when={!data().market_data_available}>
                            <span class="industry-map-warn">
                              本次未取到行情，公司暂按维护顺序排列
                            </span>
                          </Show>
                        </p>
                      </header>

                      <Show when={data().is_admin === true}>
                        <div class="industry-map-adminbar">
                          <button
                            type="button"
                            role="switch"
                            aria-checked={editing()}
                            class="industry-edit-toggle"
                            classList={{ "is-on": editing() }}
                            onClick={() => setEditing((value) => !value)}
                          >
                            <span class="industry-edit-toggle-track" aria-hidden="true" />
                            编辑本体
                          </button>
                          <p>
                            {editing()
                              ? "改动直接写进研究底稿，研究台与后续对话的行业注入同时生效；每次保存都要写明为什么改。"
                              : "打开后可就地改这一页的每一块：一句话、公司、上游信号、估值逻辑、关注点与来源。"}
                          </p>
                        </div>
                      </Show>

                      <Show when={data().is_admin === true && data().recent_edits.length > 0}>
                        <section class="industry-edits" aria-label="最近改动">
                          <h2>
                            最近改动
                            <span class="industry-edits-count">
                              共 {data().edit_count} 次
                            </span>
                          </h2>
                          <p class="industry-edits-note">
                            管理员在对话里改的行业内容会记在这里，研究台与后续对话的行业注入同时生效。
                          </p>
                          <ul>
                            <For each={data().recent_edits}>
                              {(edit) => (
                                <li>
                                  <button
                                    type="button"
                                    class="industry-edits-jump"
                                    onClick={() => selectIndustry(edit.industry)}
                                  >
                                    {edit.industry_name}
                                  </button>
                                  <span class="industry-edits-summary">{edit.summary}</span>
                                  <span class="industry-edits-meta">
                                    <span>{editedAt(edit.at)}</span>
                                    <span class="industry-edits-by">{edit.by}</span>
                                  </span>
                                  <Show when={edit.note}>
                                    <p class="industry-edits-why">{edit.note}</p>
                                  </Show>
                                </li>
                              )}
                            </For>
                          </ul>
                        </section>
                      </Show>

                      <div class="industry-map-body">
                        <nav class="industry-tree" aria-label="行业树">
                          <div class="industry-search">
                            <input
                              class="industry-input"
                              type="search"
                              aria-label="找公司"
                              placeholder="找公司：SNDK / 闪迪"
                              autocomplete="off"
                              value={query()}
                              onInput={(event) => {
                                if (event.isComposing) return;
                                runSearch(event.currentTarget.value);
                              }}
                              onKeyDown={(event) => {
                                if (event.key === "Enter") runSearch(event.currentTarget.value);
                              }}
                            />
                            <Show when={query().trim()}>
                              <p
                                class="industry-search-result"
                                classList={{ "is-hit": match() !== undefined }}
                                role="status"
                              >
                                <Show when={match()} fallback="没有匹配的公司">
                                  {(hit) => matchLabel(hit())}
                                </Show>
                              </p>
                            </Show>
                          </div>
                          <div class="industry-tree-root">{data().root.name}</div>
                          <Show when={editMode()}>
                            <IndustryForm editor={editor} />
                          </Show>
                          <ul>
                            <For each={data().industries}>
                              {(industry) => (
                                <li>
                                  <button
                                    type="button"
                                    class="industry-tree-node"
                                    classList={{ "is-active": industry.id === selected() }}
                                    aria-current={industry.id === selected() ? "true" : undefined}
                                    onClick={() => selectIndustry(industry.id)}
                                  >
                                    <span class="industry-tree-name">
                                      {industry.name}
                                      <Show when={industry.last_edited_at}>
                                        <span class="industry-tree-dot" title="有管理员改动" />
                                      </Show>
                                    </span>
                                    <span class="industry-tree-count">
                                      {industry.members.length}
                                    </span>
                                  </button>
                                </li>
                              )}
                            </For>
                          </ul>
                        </nav>

                        <div class="industry-detail-column">
                          <MethodologyStrip methodology={methodologyOf(data())} />
                          <Show
                            when={current()}
                            fallback={<p class="industry-map-empty">选择左侧的一个行业。</p>}
                          >
                            {(industry) => (
                              <section class="industry-detail" id="industry-detail">
                                <h2>
                                  {industry().name}
                                  <Show when={industry().last_edited_at}>
                                    <span class="industry-detail-edited">
                                      最近改动 {editedAt(industry().last_edited_at)}
                                    </span>
                                  </Show>
                                  <Show when={editMode()}>
                                    <button
                                      type="button"
                                      class="industry-btn is-danger industry-detail-remove"
                                      disabled={!editor.canSave()}
                                      onClick={() => void removeIndustry(industry())}
                                    >
                                      移除此行业
                                    </button>
                                  </Show>
                                </h2>

                                <Show when={editMode()}>
                                  <div class="industry-editbar">
                                    <label>
                                      改动说明
                                      <input
                                        class="industry-input"
                                        placeholder="为什么改（必填，展示给其它管理员）"
                                        value={note()}
                                        onInput={(event) => setNote(event.currentTarget.value)}
                                      />
                                    </label>
                                    <Show when={flash()}>
                                      {(item) => (
                                        <p
                                          class="industry-flash"
                                          classList={{
                                            "is-ok": item().kind === "ok",
                                            "is-error": item().kind === "error",
                                          }}
                                          role="status"
                                        >
                                          {item().text}
                                        </p>
                                      )}
                                    </Show>
                                    <Show when={editor.noteMissing()}>
                                      <p class="industry-editbar-hint">
                                        先写明为什么改，各块的保存按钮才会亮；说明会和改动一起记进「最近改动」。
                                      </p>
                                    </Show>
                                  </div>
                                </Show>

                                <Show
                                  when={editMode()}
                                  fallback={<p class="industry-detail-lead">{industry().one_liner}</p>}
                                >
                                  <FieldEditor
                                    label="一句话"
                                    value={industry().one_liner}
                                    rows={2}
                                    editor={editor}
                                    onSave={(value) => setField(industry().id, "one_liner", value)}
                                  />
                                </Show>
                                <Show
                                  when={editMode()}
                                  fallback={
                                    <Show when={valuation().logic.state_note}>
                                      <p class="industry-state-note">
                                        <span class="industry-state-label">典型 State</span>
                                        {valuation().logic.state_note}
                                      </p>
                                    </Show>
                                  }
                                >
                                  <FieldEditor
                                    label="典型 State"
                                    value={valuation().logic.state_note}
                                    rows={2}
                                    editor={editor}
                                    onSave={(value) =>
                                      editor.submit(industry().id, {
                                        kind: "set_valuation_field",
                                        field: "logic.state_note",
                                        value,
                                      })
                                    }
                                  />
                                </Show>

                                <h3>估值执行卡</h3>
                                <p class="industry-detail-note">先在这里定位：公司在哪个子类型、用哪个前瞻财年与哪一族倍数、什么被禁止；再去看上游最近做了什么。</p>
                                <ValuationCard
                                  industry={industry()}
                                  valuation={valuation()}
                                  active={activeSubtype()}
                                  onSelect={(id) => pickSubtype(industry().id, id)}
                                  editMode={editMode()}
                                  editor={editor}
                                />

                                <h3>上游信号</h3>
                                <p class="industry-detail-note">这一行的收入最终由哪家上市公司的最近行为决定，以及写这一行的公司之前该先去取它的哪几个读数。</p>
                                <Show
                                  when={upstreamSignals(industry()).length > 0}
                                  fallback={<p class="industry-detail-note">尚未定稿。</p>}
                                >
                                  <ul class="industry-signals">
                                    <For each={upstreamSignals(industry())}>
                                      {(signal) => (
                                        <li>
                                          <div class="industry-signal-head">
                                            <strong>{signal.symbol}</strong>
                                            <Show when={signal.name}>
                                              <span class="industry-signal-name">{signal.name}</span>
                                            </Show>
                                            <Show when={signal.relation}>
                                              <span class="industry-relation">
                                                {relationLabel(signal.relation)}
                                              </span>
                                            </Show>
                                            <Show when={signal.cadence}>
                                              <span class="industry-cadence">{signal.cadence}</span>
                                            </Show>
                                            <Show when={editMode()}>
                                              <button
                                                type="button"
                                                class="industry-btn is-danger"
                                                disabled={!editor.canSave()}
                                                onClick={() =>
                                                  void editor.submit(industry().id, {
                                                    kind: "remove_upstream_signal",
                                                    symbol: signal.symbol,
                                                  })
                                                }
                                              >
                                                移除
                                              </button>
                                            </Show>
                                          </div>
                                          <Show
                                            when={editMode()}
                                            fallback={
                                              <Show when={signal.latest}>
                                                <div class="industry-signal-latest">
                                                  <div class="industry-signal-latest-head">
                                                    <span class="industry-signal-latest-label">
                                                      最近动作
                                                    </span>
                                                    <Show when={signal.latest_as_of}>
                                                      <span class="industry-signal-asof">
                                                        截至 {signal.latest_as_of}
                                                      </span>
                                                      <Show when={isLatestStale(signal.latest_as_of)}>
                                                        <span
                                                          class="industry-signal-stale"
                                                          title={`截至日期已超过 ${LATEST_STALE_AFTER_DAYS} 天，引用前先核最近一期`}
                                                        >
                                                          可能已过期
                                                        </span>
                                                      </Show>
                                                    </Show>
                                                  </div>
                                                  <p class="industry-signal-latest-text">{signal.latest}</p>
                                                </div>
                                              </Show>
                                            }
                                          >
                                            <LatestEditor
                                              symbol={signal.symbol}
                                              latest={signal.latest ?? ""}
                                              asOf={signal.latest_as_of ?? ""}
                                              editor={editor}
                                              onSave={(latest, asOf) =>
                                                editor.submit(industry().id, {
                                                  kind: "set_upstream_latest",
                                                  symbol: signal.symbol,
                                                  latest,
                                                  as_of: asOf,
                                                })
                                              }
                                            />
                                          </Show>
                                          <Show when={signal.why}>
                                            <p>{signal.why}</p>
                                          </Show>
                                          <Show when={(signal.pull ?? []).length > 0}>
                                            <ul class="industry-signal-pull">
                                              <For each={signal.pull}>{(item) => <li>{item}</li>}</For>
                                            </ul>
                                          </Show>
                                        </li>
                                      )}
                                    </For>
                                  </ul>
                                </Show>
                                <Show when={editMode()}>
                                  <SignalForm industry={industry().id} editor={editor} />
                                </Show>

                                <h3>底层估值逻辑</h3>
                                <ValuationLogicBlock
                                  industry={industry()}
                                  valuation={valuation()}
                                  editMode={editMode()}
                                  editor={editor}
                                />
                                <h4 class="industry-subhead">传导链与旧版锚（带日期的量）</h4>
                                <Show
                                  when={editMode()}
                                  fallback={
                                    <Show
                                      when={industry().ai_valuation_logic.driver_chain}
                                      fallback={<p class="industry-detail-note">这一行的传导链尚未定稿。</p>}
                                    >
                                      <p class="industry-chain">
                                        {industry().ai_valuation_logic.driver_chain}
                                      </p>
                                      <Show when={industry().ai_valuation_logic.key_variables.length > 0}>
                                        <VariablesTable
                                          variables={industry().ai_valuation_logic.key_variables}
                                        />
                                      </Show>
                                      <dl class="industry-anchor">
                                        <dt>倍数锚</dt>
                                        <dd>{industry().ai_valuation_logic.multiple_anchor || "—"}</dd>
                                        <dt>这一行最常见的估值错法</dt>
                                        <dd>{industry().ai_valuation_logic.anti_pattern || "—"}</dd>
                                      </dl>
                                    </Show>
                                  }
                                >
                                  <FieldEditor
                                    label="传导链"
                                    value={industry().ai_valuation_logic.driver_chain}
                                    rows={4}
                                    editor={editor}
                                    onSave={(value) => setField(industry().id, "driver_chain", value)}
                                  />
                                  <Show when={industry().ai_valuation_logic.key_variables.length > 0}>
                                    <p class="industry-detail-note">可观测变量表暂不在页面上改。</p>
                                    <VariablesTable
                                      variables={industry().ai_valuation_logic.key_variables}
                                    />
                                  </Show>
                                  <FieldEditor
                                    label="倍数锚（长版，研究台看）"
                                    value={industry().ai_valuation_logic.multiple_anchor}
                                    editor={editor}
                                    onSave={(value) => setField(industry().id, "multiple_anchor", value)}
                                  />
                                  <FieldEditor
                                    label="倍数锚（短版，每轮注入模型，110 字内）"
                                    value={industry().ai_valuation_logic.multiple_anchor_short ?? ""}
                                    rows={2}
                                    editor={editor}
                                    onSave={(value) =>
                                      setField(industry().id, "multiple_anchor_short", value)
                                    }
                                  />
                                  <FieldEditor
                                    label="这一行最常见的估值错法（长版，研究台看）"
                                    value={industry().ai_valuation_logic.anti_pattern}
                                    editor={editor}
                                    onSave={(value) => setField(industry().id, "anti_pattern", value)}
                                  />
                                  <FieldEditor
                                    label="估值错法（短版，每轮注入模型，110 字内）"
                                    value={industry().ai_valuation_logic.anti_pattern_short ?? ""}
                                    rows={2}
                                    editor={editor}
                                    onSave={(value) =>
                                      setField(industry().id, "anti_pattern_short", value)
                                    }
                                  />
                                </Show>

                                <h3>相关公司</h3>
                                <p class="industry-detail-note">按市值降序；本轮未取到行情的排在最后。树里只收美股与 ADR。标着「官方股本口径」的行，市值是现价 × 最近一期定期报告封面上的官方股本；提供方的股本会整整落后一份申报，所以并列给出提供方市值供对照。「问 HONE」会带着这一行的本体去开一轮前瞻估值。</p>
                                <table class="industry-members">
                                  <thead>
                                    <tr>
                                      <th>代码</th>
                                      <th>公司</th>
                                      <th>子类型</th>
                                      <th>市值（美元）</th>
                                      <th>现价</th>
                                      <th>在这一行的位置</th>
                                      <th>操作</th>
                                    </tr>
                                  </thead>
                                  <tbody>
                                    <For each={industry().members}>
                                      {(member) => (
                                        <tr
                                          data-symbol={member.symbol}
                                          classList={{ "is-hit": highlight() === member.symbol }}
                                        >
                                          <td class="industry-symbol">{member.symbol}</td>
                                          <td>{member.name}</td>
                                          <td class="industry-subtype-cell">
                                            <Show
                                              when={editMode()}
                                              fallback={
                                                <Show
                                                  when={subtypeOf(industry(), member.symbol)}
                                                  fallback={<span class="industry-subtype-none">—</span>}
                                                >
                                                  {(subtype) => (
                                                    <button
                                                      type="button"
                                                      class="industry-subtype-tag"
                                                      classList={{
                                                        "is-inferred": isInferredMember(subtype(), member.symbol),
                                                      }}
                                                      title={
                                                        isInferredMember(subtype(), member.symbol)
                                                          ? INFERRED_MEMBER_TIP
                                                          : `查看子类型「${subtype().name}」`
                                                      }
                                                      onClick={() => pickSubtype(industry().id, subtype().id)}
                                                    >
                                                      {subtype().name}
                                                    </button>
                                                  )}
                                                </Show>
                                              }
                                            >
                                              <MemberSubtypeSelect
                                                industry={industry()}
                                                symbol={member.symbol}
                                                editor={editor}
                                              />
                                            </Show>
                                          </td>
                                          <td>
                                            {marketCap(member.market_cap)}
                                            <Show
                                              when={member.market_cap_basis === "price_x_official_shares"}
                                            >
                                              <span class="industry-basis" title="提供方的 sharesOutstanding 会整整落后一份申报，这里按最近一期定期报告封面上的官方股本重算；括号里是提供方原样的市值，便于与外部站点对照。">
                                                官方股本口径
                                                <Show when={member.provider_market_cap != null}>
                                                  {" · 提供方 "}
                                                  {marketCap(member.provider_market_cap)}
                                                </Show>
                                              </span>
                                            </Show>
                                          </td>
                                          <td>
                                            <Show when={member.price != null} fallback="—">
                                              {member.price?.toFixed(2)}
                                              <span
                                                class="industry-change"
                                                classList={{ "is-down": (member.change_percent ?? 0) < 0 }}
                                              >
                                                {changePercent(member.change_percent)}
                                              </span>
                                            </Show>
                                          </td>
                                          <td class="industry-role">
                                            <Show when={editMode()} fallback={member.role}>
                                              <FieldEditor
                                                ariaLabel={`${member.symbol} 在这一行的位置`}
                                                value={member.role}
                                                rows={2}
                                                editor={editor}
                                                onSave={(role) =>
                                                  editor.submit(industry().id, {
                                                    kind: "set_member_role",
                                                    symbol: member.symbol,
                                                    role,
                                                  })
                                                }
                                              />
                                            </Show>
                                          </td>
                                          <td>
                                            <div class="industry-members-actions">
                                              <button
                                                type="button"
                                                class="industry-btn"
                                                title="带着这一行的本体去开一轮前瞻估值"
                                                onClick={() => askHone(industry(), member)}
                                              >
                                                问 HONE
                                              </button>
                                              <Show when={editMode()}>
                                                <button
                                                  type="button"
                                                  class="industry-btn is-danger"
                                                  disabled={!editor.canSave()}
                                                  onClick={() =>
                                                    void editor.submit(industry().id, {
                                                      kind: "remove_member",
                                                      symbol: member.symbol,
                                                    })
                                                  }
                                                >
                                                  移出
                                                </button>
                                              </Show>
                                            </div>
                                          </td>
                                        </tr>
                                      )}
                                    </For>
                                  </tbody>
                                  <Show when={editMode()}>
                                    <tfoot>
                                      <AddMemberRow industry={industry().id} editor={editor} />
                                    </tfoot>
                                  </Show>
                                </table>

                                <h3>核心关注点</h3>
                                <Show
                                  when={industry().core_watch.length > 0}
                                  fallback={<p class="industry-detail-note">尚未定稿。</p>}
                                >
                                  <ul class="industry-watch">
                                    <For each={industry().core_watch}>
                                      {(watch) => (
                                        <li>
                                          <Show when={editMode()}>
                                            <button
                                              type="button"
                                              class="industry-btn is-danger industry-item-remove"
                                              disabled={!editor.canSave()}
                                              onClick={() =>
                                                void editor.submit(industry().id, {
                                                  kind: "remove_watch",
                                                  what: watch.what,
                                                })
                                              }
                                            >
                                              移除
                                            </button>
                                          </Show>
                                          <strong>{watch.what}</strong>
                                          <span class="industry-cadence">{watch.cadence}</span>
                                          <p>{watch.why}</p>
                                        </li>
                                      )}
                                    </For>
                                  </ul>
                                </Show>
                                <Show when={editMode()}>
                                  <WatchForm industry={industry().id} editor={editor} />
                                </Show>

                                <h3>研报与数据来源</h3>
                                <Show
                                  when={industry().sources.length > 0}
                                  fallback={<p class="industry-detail-note">尚未定稿。</p>}
                                >
                                  <ul class="industry-sources">
                                    <For each={industry().sources}>
                                      {(source) => (
                                        <li>
                                          <Show when={editMode()}>
                                            <button
                                              type="button"
                                              class="industry-btn is-danger industry-item-remove"
                                              disabled={!editor.canSave()}
                                              onClick={() =>
                                                void editor.submit(industry().id, {
                                                  kind: "remove_source",
                                                  url: source.url,
                                                })
                                              }
                                            >
                                              移除
                                            </button>
                                          </Show>
                                          <a href={source.url} target="_blank" rel="noreferrer">
                                            {source.house}｜{source.title}
                                          </a>
                                          <span class="industry-source-date">{source.date}</span>
                                          <p>{source.takeaway}</p>
                                        </li>
                                      )}
                                    </For>
                                  </ul>
                                </Show>
                                <Show when={editMode()}>
                                  <SourceForm industry={industry().id} editor={editor} />
                                </Show>
                              </section>
                            )}
                          </Show>
                        </div>
                      </div>
                    </div>
                  )}
                </Show>
              </Show>
            </Show>
          </PublicWorkspaceShell>
        </Show>
      </Show>
    </>
  );
}
