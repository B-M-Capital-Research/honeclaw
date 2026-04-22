import {
  For,
  Show,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import {
  getNotificationPrefs,
  getUsers,
  listPortfolioActors,
  putNotificationPrefs,
  type NotificationPrefs,
} from "@/lib/api";
import {
  actorKey,
  actorLabel,
  parseActorKey,
  type ActorRef,
} from "@/lib/actors";
import type { UserInfo } from "@/lib/types";

const DEFAULT_PREFS: NotificationPrefs = {
  enabled: true,
  portfolio_only: false,
  min_severity: "low",
  allow_kinds: null,
  blocked_kinds: [],
  timezone: null,
  digest_windows: null,
  price_high_pct_override: null,
  immediate_kinds: null,
};

type RosterEntry = {
  actor: ActorRef;
  prefs: NotificationPrefs;
  kindTags: string[];
};

function sameActor(a?: ActorRef, b?: ActorRef) {
  if (!a || !b) return false;
  return (
    a.channel === b.channel &&
    a.user_id === b.user_id &&
    (a.channel_scope ?? "") === (b.channel_scope ?? "")
  );
}

async function loadActorsList(): Promise<ActorRef[]> {
  const [portfolioList, userList] = await Promise.all([
    listPortfolioActors().catch(() => []),
    getUsers().catch(() => [] as UserInfo[]),
  ]);
  const map = new Map<string, ActorRef>();
  for (const s of portfolioList) {
    const a: ActorRef = {
      channel: s.channel,
      user_id: s.user_id,
      channel_scope: s.channel_scope,
    };
    map.set(actorKey(a), a);
  }
  for (const u of userList) {
    const a: ActorRef = {
      channel: u.channel,
      user_id: u.user_id,
      channel_scope: u.channel_scope,
    };
    if (!map.has(actorKey(a))) map.set(actorKey(a), a);
  }
  return Array.from(map.values());
}

function summarize(p: NotificationPrefs): string {
  if (!p.enabled) return "已关闭";
  const parts: string[] = [p.min_severity];
  if (p.portfolio_only) parts.push("仅持仓");
  if (p.allow_kinds && p.allow_kinds.length)
    parts.push(`白名单 ${p.allow_kinds.length}`);
  if (p.blocked_kinds && p.blocked_kinds.length)
    parts.push(`黑名单 ${p.blocked_kinds.length}`);
  if (p.timezone) parts.push(`TZ=${p.timezone}`);
  if (p.digest_windows) {
    parts.push(
      p.digest_windows.length === 0
        ? "关 digest"
        : `digest×${p.digest_windows.length}`,
    );
  }
  if (p.price_high_pct_override != null)
    parts.push(`⚡${p.price_high_pct_override}%`);
  if (p.immediate_kinds && p.immediate_kinds.length)
    parts.push(`强升 ${p.immediate_kinds.length}`);
  return `启用 · ${parts.join(" · ")}`;
}

export function NotificationPreferencesCard() {
  const [roster, setRoster] = createSignal<RosterEntry[]>([]);
  const [rosterLoading, setRosterLoading] = createSignal(false);
  const [rosterError, setRosterError] = createSignal("");
  const [selectedKey, setSelectedKey] = createSignal("");
  const [savingKey, setSavingKey] = createSignal("");
  const [detailDirty, setDetailDirty] = createSignal(false);
  const [message, setMessage] = createSignal("");
  const [error, setError] = createSignal("");
  const [manual, setManual] = createSignal<ActorRef>({
    channel: "",
    user_id: "",
    channel_scope: "",
  });

  const currentActor = createMemo(() => parseActorKey(selectedKey()));
  const currentEntry = createMemo(() => {
    const a = currentActor();
    if (!a) return undefined;
    return roster().find((e) => sameActor(e.actor, a));
  });
  const currentPrefs = createMemo(
    () => currentEntry()?.prefs ?? DEFAULT_PREFS,
  );
  const currentKindTags = createMemo(() => currentEntry()?.kindTags ?? []);

  const patchEntry = (
    actor: ActorRef,
    patch: Partial<RosterEntry> | ((e: RosterEntry) => RosterEntry),
  ) => {
    setRoster(
      roster().map((e) =>
        sameActor(e.actor, actor)
          ? typeof patch === "function"
            ? patch(e)
            : { ...e, ...patch }
          : e,
      ),
    );
  };

  const upsertEntry = (entry: RosterEntry) => {
    const list = roster();
    if (list.some((e) => sameActor(e.actor, entry.actor))) {
      patchEntry(entry.actor, entry);
    } else {
      setRoster([...list, entry]);
    }
  };

  const fetchEntry = async (actor: ActorRef): Promise<RosterEntry> => {
    const b = await getNotificationPrefs(actor);
    return { actor, prefs: b.prefs, kindTags: b.kind_tags };
  };

  const refreshRoster = async () => {
    setRosterLoading(true);
    setRosterError("");
    try {
      const actors = await loadActorsList();
      const bundles = await Promise.all(
        actors.map(async (actor) => {
          try {
            return await fetchEntry(actor);
          } catch {
            return {
              actor,
              prefs: { ...DEFAULT_PREFS },
              kindTags: [],
            } satisfies RosterEntry;
          }
        }),
      );
      setRoster(bundles);
    } catch (e) {
      setRosterError(e instanceof Error ? e.message : String(e));
    } finally {
      setRosterLoading(false);
    }
  };

  onMount(() => {
    void refreshRoster();
  });

  const savePrefs = async (actor: ActorRef, prefs: NotificationPrefs) => {
    const k = actorKey(actor);
    setSavingKey(k);
    setMessage("");
    setError("");
    try {
      const saved = await putNotificationPrefs(actor, prefs);
      patchEntry(actor, (e) => ({ ...e, prefs: saved }));
      if (sameActor(actor, currentActor())) setDetailDirty(false);
      setMessage(
        `已保存 ${actor.channel} · ${actorLabel(actor)} 的推送偏好,下一条事件即刻生效`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      throw e;
    } finally {
      setSavingKey("");
    }
  };

  const toggleRosterEnabled = async (actor: ActorRef, enabled: boolean) => {
    const entry = roster().find((e) => sameActor(e.actor, actor));
    if (!entry) return;
    const next = { ...entry.prefs, enabled };
    patchEntry(actor, (e) => ({ ...e, prefs: next }));
    try {
      await savePrefs(actor, next);
    } catch {
      patchEntry(actor, (e) => ({ ...e, prefs: entry.prefs }));
    }
  };

  const chooseActor = async (actor: ActorRef) => {
    setMessage("");
    setError("");
    setSelectedKey(actorKey(actor));
    setDetailDirty(false);
    if (!roster().some((e) => sameActor(e.actor, actor))) {
      try {
        upsertEntry(await fetchEntry(actor));
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        upsertEntry({ actor, prefs: { ...DEFAULT_PREFS }, kindTags: [] });
      }
    }
  };

  const applyManual = () => {
    const m = manual();
    if (!m.channel.trim() || !m.user_id.trim()) return;
    void chooseActor({
      channel: m.channel.trim(),
      user_id: m.user_id.trim(),
      channel_scope: m.channel_scope?.trim() || undefined,
    });
  };

  const editCurrent = (
    updater: (p: NotificationPrefs) => NotificationPrefs,
  ) => {
    const a = currentActor();
    if (!a) return;
    patchEntry(a, (e) => ({ ...e, prefs: updater(e.prefs) }));
    setDetailDirty(true);
  };

  const toggleTag = (list: string[], tag: string) =>
    list.includes(tag) ? list.filter((t) => t !== tag) : [...list, tag];

  const handleAllowToggle = (tag: string) => {
    editCurrent((p) => {
      const next = toggleTag(p.allow_kinds ?? [], tag);
      return { ...p, allow_kinds: next.length === 0 ? null : next };
    });
  };

  const handleBlockToggle = (tag: string) => {
    editCurrent((p) => ({
      ...p,
      blocked_kinds: toggleTag(p.blocked_kinds ?? [], tag),
    }));
  };

  const handleImmediateToggle = (tag: string) => {
    editCurrent((p) => {
      const next = toggleTag(p.immediate_kinds ?? [], tag);
      return { ...p, immediate_kinds: next.length === 0 ? null : next };
    });
  };

  // digest_windows 操作:null = 沿用全局,[] = 关 digest,[..] = 自定义。
  const [windowDraft, setWindowDraft] = createSignal("");
  const HHMM_RE = /^([01]\d|2[0-3]):[0-5]\d$/;
  const sortedUniqueWindows = (list: string[]): string[] =>
    Array.from(new Set(list)).sort();
  const addWindow = () => {
    const v = windowDraft().trim();
    if (!HHMM_RE.test(v)) return;
    editCurrent((p) => ({
      ...p,
      digest_windows: sortedUniqueWindows([...(p.digest_windows ?? []), v]),
    }));
    setWindowDraft("");
  };
  const removeWindow = (hhmm: string) => {
    editCurrent((p) => ({
      ...p,
      digest_windows: (p.digest_windows ?? []).filter((w) => w !== hhmm),
    }));
  };
  const resetWindowsToGlobal = () => {
    editCurrent((p) => ({ ...p, digest_windows: null }));
  };
  const muteAllDigest = () => {
    editCurrent((p) => ({ ...p, digest_windows: [] }));
  };

  const handleTimezoneInput = (raw: string) => {
    const v = raw.trim();
    editCurrent((p) => ({ ...p, timezone: v === "" ? null : v }));
  };

  const handlePriceHighInput = (raw: string) => {
    const v = raw.trim();
    if (v === "") {
      editCurrent((p) => ({ ...p, price_high_pct_override: null }));
      return;
    }
    const n = Number(v);
    editCurrent((p) => ({
      ...p,
      price_high_pct_override: Number.isFinite(n) ? n : p.price_high_pct_override,
    }));
  };

  const submitDetail = async () => {
    const a = currentActor();
    const e = currentEntry();
    if (!a || !e) return;
    try {
      await savePrefs(a, e.prefs);
    } catch {
      /* savePrefs 已把 error 落到 banner */
    }
  };

  return (
    <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
      <div class="flex items-center justify-between">
        <div>
          <div class="text-sm font-bold text-[color:var(--text-primary)]">
            通知偏好(per-actor)
          </div>
          <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">
            上区:一键启停,切换即保存;下区:点一个 actor 做严重度 / 白黑名单细调
          </div>
        </div>
        <button
          type="button"
          class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] transition hover:text-[color:var(--text-primary)]"
          onClick={() => void refreshRoster()}
        >
          刷新
        </button>
      </div>

      <div class="mt-4">
        <div class="text-[11px] font-semibold text-[color:var(--text-secondary)]">
          actor 列表
        </div>
        <div class="mt-2 divide-y divide-[color:var(--border)] rounded-md border border-[color:var(--border)] bg-[color:var(--surface)]">
          <Show when={rosterLoading()}>
            <div class="px-3 py-2 text-[11px] text-[color:var(--text-secondary)]">
              加载中...
            </div>
          </Show>
          <Show when={!rosterLoading() && roster().length === 0}>
            <div class="px-3 py-2 text-[11px] text-[color:var(--text-secondary)]">
              还没有可选 actor(手动输入下方字段或先让渠道产生一条消息)
            </div>
          </Show>
          <For each={roster()}>
            {(entry) => {
              const k = actorKey(entry.actor);
              const isSelected = () => selectedKey() === k;
              const isSaving = () => savingKey() === k;
              return (
                <div
                  class="flex items-center justify-between gap-3 px-3 py-2 cursor-pointer transition"
                  classList={{
                    "bg-[color:var(--accent)]/10": isSelected(),
                    "hover:bg-black/5": !isSelected(),
                  }}
                  onClick={() => void chooseActor(entry.actor)}
                >
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-xs font-semibold text-[color:var(--text-primary)]">
                      {entry.actor.channel} · {actorLabel(entry.actor)}
                    </div>
                    <div
                      class="truncate text-[10px]"
                      classList={{
                        "text-[color:var(--text-secondary)]":
                          entry.prefs.enabled,
                        "text-rose-500": !entry.prefs.enabled,
                      }}
                    >
                      {summarize(entry.prefs)}
                    </div>
                  </div>
                  <label
                    class="flex items-center gap-1.5 text-[10px] text-[color:var(--text-secondary)]"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <span>{isSaving() ? "保存中..." : entry.prefs.enabled ? "推送中" : "已关"}</span>
                    <input
                      type="checkbox"
                      checked={entry.prefs.enabled}
                      disabled={isSaving()}
                      onChange={(e) =>
                        void toggleRosterEnabled(
                          entry.actor,
                          e.currentTarget.checked,
                        )
                      }
                    />
                  </label>
                </div>
              );
            }}
          </For>
        </div>
      </div>

      <div class="mt-3 grid grid-cols-3 gap-2">
        <input
          class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
          placeholder="channel"
          value={manual().channel}
          onInput={(e) =>
            setManual({ ...manual(), channel: e.currentTarget.value })
          }
        />
        <input
          class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
          placeholder="user_id"
          value={manual().user_id}
          onInput={(e) =>
            setManual({ ...manual(), user_id: e.currentTarget.value })
          }
        />
        <div class="flex gap-1">
          <input
            class="flex-1 rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
            placeholder="channel_scope(可选)"
            value={manual().channel_scope ?? ""}
            onInput={(e) =>
              setManual({ ...manual(), channel_scope: e.currentTarget.value })
            }
          />
          <button
            type="button"
            class="rounded-md border border-[color:var(--border)] px-2 text-[11px]"
            onClick={applyManual}
          >
            载入
          </button>
        </div>
      </div>

      <Show when={currentActor() && currentEntry()}>
        <div class="mt-5 space-y-4 rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
          <div class="flex items-center justify-between">
            <div class="text-[11px] font-semibold text-[color:var(--text-primary)]">
              细调 {currentActor()!.channel} · {actorLabel(currentActor()!)}
            </div>
            <div class="text-[10px] text-[color:var(--text-secondary)]">
              启用/关闭回上方列表切换
            </div>
          </div>
          <div class="flex items-center justify-between">
            <label class="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={currentPrefs().portfolio_only}
                onChange={(e) =>
                  editCurrent((p) => ({
                    ...p,
                    portfolio_only: e.currentTarget.checked,
                  }))
                }
              />
              <span>仅持仓相关</span>
            </label>
            <label class="flex items-center gap-2 text-sm">
              <span>最低严重度</span>
              <select
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                value={currentPrefs().min_severity}
                onChange={(e) =>
                  editCurrent((p) => ({
                    ...p,
                    min_severity: e.currentTarget
                      .value as NotificationPrefs["min_severity"],
                  }))
                }
              >
                <option value="low">low</option>
                <option value="medium">medium</option>
                <option value="high">high</option>
              </select>
            </label>
          </div>

          <div>
            <div class="text-[11px] font-semibold text-[color:var(--text-secondary)]">
              白名单 allow_kinds(空 = 不启用白名单)
            </div>
            <div class="mt-1 flex flex-wrap gap-1">
              <For each={currentKindTags()}>
                {(tag) => {
                  const selected = () =>
                    (currentPrefs().allow_kinds ?? []).includes(tag);
                  return (
                    <button
                      type="button"
                      class="rounded-md border px-2 py-0.5 text-[11px]"
                      classList={{
                        "border-emerald-500 bg-emerald-500/10 text-emerald-500":
                          selected(),
                        "border-[color:var(--border)] text-[color:var(--text-secondary)]":
                          !selected(),
                      }}
                      onClick={() => handleAllowToggle(tag)}
                    >
                      {tag}
                    </button>
                  );
                }}
              </For>
            </div>
          </div>

          <div>
            <div class="text-[11px] font-semibold text-[color:var(--text-secondary)]">
              黑名单 blocked_kinds(优先级高于白名单)
            </div>
            <div class="mt-1 flex flex-wrap gap-1">
              <For each={currentKindTags()}>
                {(tag) => {
                  const selected = () =>
                    (currentPrefs().blocked_kinds ?? []).includes(tag);
                  return (
                    <button
                      type="button"
                      class="rounded-md border px-2 py-0.5 text-[11px]"
                      classList={{
                        "border-rose-500 bg-rose-500/10 text-rose-500":
                          selected(),
                        "border-[color:var(--border)] text-[color:var(--text-secondary)]":
                          !selected(),
                      }}
                      onClick={() => handleBlockToggle(tag)}
                    >
                      {tag}
                    </button>
                  );
                }}
              </For>
            </div>
          </div>

          <div class="space-y-3 rounded-md border border-dashed border-[color:var(--border)] p-3">
            <div class="text-[11px] font-semibold text-[color:var(--text-secondary)]">
              推送节奏(per-actor;留空 = 沿用全局)
            </div>

            <label class="flex flex-col gap-1 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                时区 (IANA, 例 Asia/Shanghai、America/New_York)
              </span>
              <input
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                placeholder="留空 → 沿用全局 digest.timezone"
                value={currentPrefs().timezone ?? ""}
                onInput={(e) => handleTimezoneInput(e.currentTarget.value)}
              />
            </label>

            <div class="flex flex-col gap-1.5 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                Digest 时刻 (本地 HH:MM;不设 = 沿用全局,清空 = 关 digest)
              </span>
              <div class="flex flex-wrap items-center gap-1">
                <Show when={currentPrefs().digest_windows === null}>
                  <span class="text-[10px] italic text-[color:var(--text-secondary)]">
                    当前:沿用全局 pre/post-market
                  </span>
                </Show>
                <Show when={currentPrefs().digest_windows?.length === 0}>
                  <span class="rounded-md border border-amber-500 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-500">
                    关 digest(只接收 immediate sink)
                  </span>
                </Show>
                <For each={currentPrefs().digest_windows ?? []}>
                  {(hhmm) => (
                    <span class="inline-flex items-center gap-1 rounded-md border border-emerald-500 bg-emerald-500/10 px-2 py-0.5 font-mono text-[11px] text-emerald-600">
                      {hhmm}
                      <button
                        type="button"
                        class="-mr-0.5 rounded text-emerald-700 hover:text-rose-500"
                        title="移除"
                        onClick={() => removeWindow(hhmm)}
                      >
                        ×
                      </button>
                    </span>
                  )}
                </For>
              </div>
              <div class="flex flex-wrap items-center gap-1">
                <input
                  type="time"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 font-mono text-xs"
                  value={windowDraft()}
                  onInput={(e) => setWindowDraft(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      addWindow();
                    }
                  }}
                />
                <button
                  type="button"
                  class="rounded-md border border-emerald-500 px-2 py-1 text-[11px] text-emerald-600 hover:bg-emerald-500/10 disabled:opacity-40"
                  disabled={!HHMM_RE.test(windowDraft().trim())}
                  onClick={addWindow}
                >
                  + 添加
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                  onClick={resetWindowsToGlobal}
                  title="清掉自定义 → 沿用全局 pre/post-market"
                >
                  恢复全局
                </button>
                <button
                  type="button"
                  class="rounded-md border border-amber-500 px-2 py-1 text-[11px] text-amber-500 hover:bg-amber-500/10"
                  onClick={muteAllDigest}
                  title="设为空数组,即完全不发 digest"
                >
                  关 digest
                </button>
              </div>
            </div>

            <label class="flex flex-col gap-1 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                价格异动即时推阈值 (% 绝对值, 0&lt;x≤50;留空 = 沿用全局,
                通常调低如 3.5 = 更敏感)
              </span>
              <input
                type="number"
                step="0.1"
                min="0"
                max="50"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                placeholder="留空 → 沿用全局 thresholds.price_alert_high_pct"
                value={currentPrefs().price_high_pct_override ?? ""}
                onInput={(e) => handlePriceHighInput(e.currentTarget.value)}
              />
            </label>

            <div>
              <div class="text-[11px] text-[color:var(--text-secondary)]">
                强制升 High 即时推 immediate_kinds(命中元素无视 poller 给的
                severity,直接 High 走 sink)
              </div>
              <div class="mt-1 flex flex-wrap gap-1">
                <For each={currentKindTags()}>
                  {(tag) => {
                    const selected = () =>
                      (currentPrefs().immediate_kinds ?? []).includes(tag);
                    return (
                      <button
                        type="button"
                        class="rounded-md border px-2 py-0.5 text-[11px]"
                        classList={{
                          "border-amber-500 bg-amber-500/10 text-amber-500":
                            selected(),
                          "border-[color:var(--border)] text-[color:var(--text-secondary)]":
                            !selected(),
                        }}
                        onClick={() => handleImmediateToggle(tag)}
                      >
                        {tag}
                      </button>
                    );
                  }}
                </For>
              </div>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2">
            <Show when={detailDirty()}>
              <span class="text-[10px] text-amber-500">有未保存改动</span>
            </Show>
            <button
              type="button"
              class="rounded-md bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white disabled:opacity-50"
              disabled={
                savingKey() === actorKey(currentActor()!) || !detailDirty()
              }
              onClick={() => void submitDetail()}
            >
              {savingKey() === actorKey(currentActor()!)
                ? "保存中..."
                : "保存细调"}
            </button>
          </div>
        </div>
      </Show>

      <Show when={error()}>
        <div class="mt-3 rounded-md border border-rose-500/50 bg-rose-500/10 px-3 py-2 text-xs text-rose-500">
          {error()}
        </div>
      </Show>
      <Show when={message()}>
        <div class="mt-3 rounded-md border border-emerald-500/50 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-600">
          {message()}
        </div>
      </Show>
      <Show when={rosterError()}>
        <div class="mt-3 rounded-md border border-rose-500/50 bg-rose-500/10 px-3 py-2 text-xs text-rose-500">
          {rosterError()}
        </div>
      </Show>
    </div>
  );
}
