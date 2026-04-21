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
