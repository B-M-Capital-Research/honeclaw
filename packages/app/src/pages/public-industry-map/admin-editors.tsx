import { For, Show, createEffect, createSignal, on } from "solid-js";

import {
  RELATION_LABELS,
  isInferredMember,
  isSubtypeId,
  splitSymbols,
  subtypeOf,
  valuationOf,
} from "@/lib/industry-valuation";
import type { Industry, IndustrySubtype, IndustryUpstreamRelation } from "@/lib/types";

import {
  NON_US_SYMBOL_MESSAGE,
  isNonUsSymbol,
  splitAliases,
  splitLines,
  type Editor,
} from "./shared";

/** 管理员编辑态才会渲染的表单：加公司、加上游信号、加关注点、加来源、加行业、子类型、成员归类。 */

/** 公司表表尾的「加入公司」。symbol 带 . 或 : 的在这里就拒掉，不必等后端。 */
export function AddMemberRow(props: { industry: string; editor: Editor }) {
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

export function SignalForm(props: { industry: string; editor: Editor }) {
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

export function WatchForm(props: { industry: string; editor: Editor }) {
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

export function SourceForm(props: { industry: string; editor: Editor }) {
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
export function IndustryForm(props: { editor: Editor }) {
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
 * 子类型表单：新增（seed 为 null）与编辑（seed 为现有子类型，预填）共用一份。
 * 提交的是整条 upsert：编辑时把底稿推断的成员原样带回去（去掉这次已明确归入的），
 * 否则整体替换会把推断成员丢掉。id 是子类型的身份，编辑时锁住——改 id 等于新建。
 */
export function SubtypeForm(props: {
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
  const [scopeNote, setScopeNote] = createSignal(props.seed?.scope_note ?? "");
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
        setScopeNote(seed?.scope_note ?? "");
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
        scope_note: scopeNote().trim(),
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
        范围（收哪些公司、不收哪些）
        <textarea
          class="industry-textarea"
          rows={2}
          placeholder="如：只收以 NAND 为主的上市公司；SpaceX 未上市不收"
          value={scopeNote()}
          disabled={props.editor.busy()}
          onInput={(event) => setScopeNote(event.currentTarget.value)}
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

/** 公司表编辑列的子类型下拉：只认明确归入；推断成员显示为「未确认」，管理员选一下就是确认。 */
export function MemberSubtypeSelect(props: { industry: Industry; symbol: string; editor: Editor }) {
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
