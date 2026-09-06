import { Title } from "@solidjs/meta";
import { useNavigate, useSearchParams } from "@solidjs/router";
import {
  For,
  Show,
  batch,
  createEffect,
  createMemo,
  createSignal,
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
  matchLabel,
  methodologyOf,
  subtypeOf,
  valuationOf,
} from "@/lib/industry-valuation";
import type { MemberMatch } from "@/lib/industry-valuation";
import { cachedPublicUser, setCachedPublicUser } from "@/lib/public-session-cache";
import type {
  Industry,
  IndustryEditField,
  IndustryMapSnapshot,
  IndustryMember,
  PublicAuthUserInfo,
} from "@/lib/types";

import {
  AddMemberRow,
  IndustryForm,
  MemberSubtypeSelect,
  SignalForm,
  SourceForm,
  WatchForm,
} from "./public-industry-map/admin-editors";
import {
  MethodologyStrip,
  ValuationCard,
  ValuationLogicBlock,
  VariablesTable,
} from "./public-industry-map/dossier";
import {
  FieldEditor,
  LatestEditor,
  changePercent,
  editedAt,
  marketCap,
  relationLabel,
  upstreamSignals,
  type Editor,
  type Flash,
} from "./public-industry-map/shared";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-industry-map.css";

type ViewState = "loading" | "ready" | "login" | "forbidden" | "error";

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
