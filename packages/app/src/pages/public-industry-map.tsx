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
import { contentAsOf, deriveIndustryBrief } from "@/lib/industry-brief";
import {
  resolveIndustryMapLens,
  resolveIndustryMapSelection,
} from "@/lib/industry-map-navigation";
import {
  askHoneHref,
  askHonePrompt,
  findMember,
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

import { IndustryForm } from "./public-industry-map/admin-editors";
import { IndustryBriefSection } from "./public-industry-map/brief";
import { RecentChanges } from "./public-industry-map/changes";
import { CompanyLensCard, MembersTable } from "./public-industry-map/company-lens";
import { DOSSIER_IDS, ResearchDossier } from "./public-industry-map/dossier";
import {
  FieldEditor,
  editedAt,
  type DetailJump,
  type Editor,
  type Flash,
  type Lens,
} from "./public-industry-map/shared";
import { SourcesList, WatchList } from "./public-industry-map/watch-sources";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-industry-map.css";

type ViewState = "loading" | "ready" | "login" | "forbidden" | "error";

/**
 * 行业分析：每个行业按「当前重点 → 最近变化 → 相关公司与影响 → 接下来重点看什么 →
 * 完整研究底稿（折叠）→ 研报与数据来源」的顺序读。URL 拥有两个选择：`?industry=` 是哪一行，
 * `?symbol=` 是公司视角；刷新、分享与浏览器历史都跟着 URL 走。
 */
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
  // 切行业时把公司视角一起清掉：setSearchParams 是合并语义，不显式清 symbol 会残留。
  const selectIndustry = (id: string | undefined, replace = false) => {
    const next = resolveIndustryMapSelection(snapshot()?.industries ?? [], id);
    if (searchParams.industry === next) return;
    setSearchParams({ industry: next, symbol: undefined }, { replace, scroll: false });
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
  // 找公司：输入即匹配；命中后 URL 跟着切到那一行并进入公司视角（replace，不堆历史），
  // 底稿里的估值执行卡切到它的子类型，公司表里那一行亮几秒。
  const [query, setQuery] = createSignal("");
  const [match, setMatch] = createSignal<MemberMatch>();
  const [highlight, setHighlight] = createSignal<string>();
  // 估值执行卡里选中的子类型，按行业记：切到别的行业时自然失效，回到第一个。
  const [subtypePick, setSubtypePick] = createSignal<{ industry: string; id: string }>();
  // 完整研究底稿默认折起；编辑态默认展开；点目录或子类型标签时先展开再滚。
  const [dossierOpen, setDossierOpen] = createSignal(false);
  let highlightTimer: ReturnType<typeof setTimeout> | undefined;
  let scrollTimer: ReturnType<typeof setTimeout> | undefined;
  const pickSubtype = (industry: string, id: string) => setSubtypePick({ industry, id });
  const flashRow = (symbol: string) => {
    setHighlight(symbol);
    clearTimeout(highlightTimer);
    highlightTimer = setTimeout(() => setHighlight(undefined), 4000);
  };
  const scrollTo = (selector: string) => {
    clearTimeout(scrollTimer);
    // 行业切换后详情才重画，等一拍再把目标滚进视野。
    scrollTimer = setTimeout(() => {
      document
        .querySelector<HTMLElement>(selector)
        ?.scrollIntoView({ block: "start", behavior: "smooth" });
    }, 60);
  };
  const jump: DetailJump = (id) => {
    if ((DOSSIER_IDS as readonly string[]).includes(id)) setDossierOpen(true);
    scrollTo(`#${id}`);
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

  onMount(() => {
    if ((DOSSIER_IDS as readonly string[]).includes(window.location.hash.slice(1))) {
      setDossierOpen(true);
    }
    void bootstrap();
  });
  onCleanup(() => {
    controller?.abort();
    clearTimeout(highlightTimer);
    clearTimeout(scrollTimer);
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
  const briefView = createMemo(() => {
    const industry = current();
    return industry ? deriveIndustryBrief(industry) : undefined;
  });

  // 公司视角：`?symbol=` 只在它是当前行成员时生效；不是就从 URL 里规范化掉。
  const lensSymbol = createMemo(() => resolveIndustryMapLens(current(), searchParams.symbol));
  const lensMember = createMemo<IndustryMember | undefined>(() => {
    const symbol = lensSymbol();
    return symbol ? current()?.members.find((member) => member.symbol === symbol) : undefined;
  });
  createEffect(() => {
    if (!snapshot() || searchParams.symbol === undefined) return;
    if (lensSymbol() === undefined) {
      setSearchParams({ symbol: undefined }, { replace: true, scroll: false });
    }
  });
  const lens: Lens = {
    member: lensMember,
    enter: (symbol, options) => {
      if (searchParams.symbol === symbol) return;
      setSearchParams({ symbol }, { replace: options?.replace ?? false, scroll: false });
    },
    exit: () => {
      if (searchParams.symbol === undefined) return;
      setSearchParams({ symbol: undefined }, { scroll: false });
    },
  };
  // 行业与公司一次写进 URL：分两次 setSearchParams 时第二次会与还没更新的旧参数合并，
  // 得到「旧行业 + 新公司」，随后被规范化成没有公司视角。
  const viewMember = (industryId: string, symbol: string, replace = false) => {
    const industries = snapshot()?.industries ?? [];
    const industry = industries.find((item) => item.id === industryId);
    const subtype = industry ? subtypeOf(industry, symbol) : undefined;
    batch(() => {
      setSearchParams(
        { industry: resolveIndustryMapSelection(industries, industryId), symbol },
        { replace, scroll: false },
      );
      if (subtype) pickSubtype(industryId, subtype.id);
      flashRow(symbol);
    });
    scrollTo("#company-lens");
  };

  const runSearch = (text: string) => {
    setQuery(text);
    const hit = findMember(snapshot()?.industries ?? [], text);
    setMatch(hit);
    if (!hit) return;
    viewMember(hit.industry.id, hit.member.symbol, true);
  };
  /** 「问 HONE」：带着这一行的本体去开一轮前瞻估值；没归入子类型的公司明说，让模型按行级锚走。 */
  const askHone = (industry: Industry, member: IndustryMember) => {
    const subtype = subtypeOf(industry, member.symbol);
    navigate(
      askHoneHref(askHonePrompt(member.symbol, member.name, subtype?.name ?? "未分子类型")),
    );
  };
  const onAsk = (href: string) => navigate(href);

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
                          研究底稿基线：{data().generated_at}
                          <span class="industry-map-meta-note">
                            各行内容的截至日看该行详情；管理员的小改动不会推进这个基线日期
                          </span>
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
                              : "打开后可就地改这一页的每一块：简报、一句话、公司、上游信号、关注点、估值逻辑与来源。"}
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
                          <Show
                            when={current()}
                            fallback={<p class="industry-map-empty">选择左侧的一个行业。</p>}
                          >
                            {(industry) => (
                              <section class="industry-detail" id="industry-detail">
                                <h2>
                                  {industry().name}
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
                                <p class="industry-detail-dates">
                                  <span>
                                    内容截至 {industry().content_as_of ?? contentAsOf(industry()) ?? "—"}
                                  </span>
                                  <span>
                                    {industry().last_edited_at
                                      ? `本行最近改动 ${editedAt(industry().last_edited_at)}`
                                      : "本行自基线后未改动"}
                                  </span>
                                </p>

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
                                        <span class="industry-state-label">什么情况下适用</span>
                                        {valuation().logic.state_note}
                                      </p>
                                    </Show>
                                  }
                                >
                                  <FieldEditor
                                    label="什么情况下适用（典型 State）"
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

                                <Show when={lensMember()}>
                                  {(member) => (
                                    <CompanyLensCard
                                      industry={industry()}
                                      member={member()}
                                      onExit={lens.exit}
                                      onAsk={() => askHone(industry(), member())}
                                      onLocate={() => {
                                        flashRow(member().symbol);
                                        scrollTo(`.industry-members tr[data-symbol="${member().symbol}"]`);
                                      }}
                                      jump={jump}
                                    />
                                  )}
                                </Show>

                                <IndustryBriefSection
                                  industry={industry()}
                                  view={briefView()}
                                  editMode={editMode()}
                                  editor={editor}
                                  onAsk={onAsk}
                                  jump={jump}
                                />

                                <RecentChanges
                                  industry={industry()}
                                  editMode={editMode()}
                                  editor={editor}
                                  lens={lens}
                                  onAsk={onAsk}
                                />

                                <section
                                  class="industry-members-section"
                                  id="members"
                                  aria-labelledby="industry-members-title"
                                >
                                  <div class="industry-section-head">
                                    <h3 id="industry-members-title">相关公司与影响</h3>
                                    <span class="industry-section-sub">点「查看」让整页围绕它重排</span>
                                  </div>
                                  <p class="industry-detail-note">按市值降序；本轮未取到行情的排在最后。树里只收美股与 ADR。标着「官方股本口径」的行，市值是现价 × 最近一期定期报告封面上的官方股本；提供方的股本会整整落后一份申报，所以并列给出提供方市值供对照。「问 HONE」会带着这一行的本体去开一轮前瞻估值。</p>
                                  <MembersTable
                                    industry={industry()}
                                    editMode={editMode()}
                                    editor={editor}
                                    highlight={highlight()}
                                    lensSymbol={lensSymbol()}
                                    onPickSubtype={(id) => {
                                      pickSubtype(industry().id, id);
                                      jump("valuation-card");
                                    }}
                                    onAsk={(member) => askHone(industry(), member)}
                                    onView={(symbol) => viewMember(industry().id, symbol)}
                                  />
                                </section>

                                <WatchList
                                  industry={industry()}
                                  editMode={editMode()}
                                  editor={editor}
                                  lens={lens}
                                  onAsk={onAsk}
                                />

                                <ResearchDossier
                                  industry={industry()}
                                  valuation={valuation()}
                                  methodology={methodologyOf(data())}
                                  activeSubtype={activeSubtype()}
                                  onSelectSubtype={(id) => pickSubtype(industry().id, id)}
                                  editMode={editMode()}
                                  editor={editor}
                                  setField={(field, value) => setField(industry().id, field, value)}
                                  open={dossierOpen() || editMode()}
                                  onToggle={setDossierOpen}
                                  jump={jump}
                                  member={lensMember()}
                                />

                                <SourcesList
                                  industry={industry()}
                                  editMode={editMode()}
                                  editor={editor}
                                  lens={lens}
                                />
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
