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
import { NOTIFICATIONS } from "@/lib/admin-content/notifications";
import { tpl } from "@/lib/i18n";
import {
  DEFAULT_NOTIFICATION_PREFS,
  isValidDigestSlotTime,
  sameActor,
  sortDigestSlots,
  timeFallsInQuiet,
  toggleTag,
} from "./notification-preferences-model";

type RosterEntry = {
  actor: ActorRef;
  prefs: NotificationPrefs;
  kindTags: string[];
};

async function loadActorsList(): Promise<ActorRef[]> {
  const [portfolioList, userList] = await Promise.all([
    listPortfolioActors().catch(() => []),
    getUsers().catch(() => [] as UserInfo[]),
  ]);
  const map = new Map<string, ActorRef>();
  for (const portfolioActor of portfolioList) {
    const actor: ActorRef = {
      channel: portfolioActor.channel,
      user_id: portfolioActor.user_id,
      channel_scope: portfolioActor.channel_scope,
    };
    map.set(actorKey(actor), actor);
  }
  for (const user of userList) {
    const actor: ActorRef = {
      channel: user.channel,
      user_id: user.user_id,
      channel_scope: user.channel_scope,
    };
    if (!map.has(actorKey(actor))) map.set(actorKey(actor), actor);
  }
  return Array.from(map.values());
}

function summarize(prefs: NotificationPrefs): string {
  if (!prefs.enabled) return NOTIFICATIONS.prefs.summarize_disabled;
  const parts: string[] = [prefs.min_severity];
  if (prefs.portfolio_only) parts.push(NOTIFICATIONS.prefs.summarize_only_portfolio);
  if (prefs.allow_kinds && prefs.allow_kinds.length)
    parts.push(
      tpl(NOTIFICATIONS.prefs.summarize_allow, {
        count: prefs.allow_kinds.length,
      }),
    );
  if (prefs.blocked_kinds && prefs.blocked_kinds.length)
    parts.push(
      tpl(NOTIFICATIONS.prefs.summarize_block, {
        count: prefs.blocked_kinds.length,
      }),
    );
  if (prefs.timezone) parts.push(tpl(NOTIFICATIONS.prefs.summarize_tz, { tz: prefs.timezone }));
  if (prefs.digest_slots) {
    parts.push(
      prefs.digest_slots.length === 0
        ? NOTIFICATIONS.prefs.summarize_digest_off
        : tpl(NOTIFICATIONS.prefs.summarize_digest_count, { count: prefs.digest_slots.length }),
    );
  }
  if (prefs.price_high_pct_override != null)
    parts.push(tpl(NOTIFICATIONS.prefs.summarize_price, { value: prefs.price_high_pct_override }));
  if (prefs.immediate_kinds && prefs.immediate_kinds.length)
    parts.push(
      tpl(NOTIFICATIONS.prefs.summarize_immediate, {
        count: prefs.immediate_kinds.length,
      }),
    );
  if (prefs.quiet_hours)
    parts.push(
      tpl(NOTIFICATIONS.prefs.summarize_quiet, {
        from: prefs.quiet_hours.from,
        to: prefs.quiet_hours.to,
      }),
    );
  return `${NOTIFICATIONS.prefs.summarize_enabled_prefix} · ${parts.join(" · ")}`;
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
  const clearFeedback = () => {
    setMessage("");
    setError("");
  };
  const [manual, setManual] = createSignal<ActorRef>({
    channel: "",
    user_id: "",
    channel_scope: "",
  });
  const updateManual = (patch: Partial<ActorRef>) => {
    setManual((current) => ({ ...current, ...patch }));
  };

  const currentActor = createMemo(() => parseActorKey(selectedKey()));
  const currentActorKey = createMemo(() => {
    const actor = currentActor();
    return actor ? actorKey(actor) : "";
  });
  const currentEntry = createMemo(() => {
    const actor = currentActor();
    if (!actor) return undefined;
    return roster().find((entry) => sameActor(entry.actor, actor));
  });
  const currentPrefs = createMemo(
    () => currentEntry()?.prefs ?? DEFAULT_NOTIFICATION_PREFS,
  );
  const currentKindTags = createMemo(() => currentEntry()?.kindTags ?? []);

  const patchEntry = (
    actor: ActorRef,
    patch: Partial<RosterEntry> | ((entry: RosterEntry) => RosterEntry),
  ) => {
    setRoster(
      roster().map((entry) =>
        sameActor(entry.actor, actor)
          ? typeof patch === "function"
            ? patch(entry)
            : { ...entry, ...patch }
          : entry,
      ),
    );
  };

  const upsertEntry = (entry: RosterEntry) => {
    const currentRoster = roster();
    if (
      currentRoster.some((rosterEntry) =>
        sameActor(rosterEntry.actor, entry.actor),
      )
    ) {
      patchEntry(entry.actor, entry);
    } else {
      setRoster([...currentRoster, entry]);
    }
  };

  const fetchEntry = async (actor: ActorRef): Promise<RosterEntry> => {
    const prefsBundle = await getNotificationPrefs(actor);
    return { actor, prefs: prefsBundle.prefs, kindTags: prefsBundle.kind_tags };
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
              prefs: { ...DEFAULT_NOTIFICATION_PREFS },
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
    const savingActorKey = actorKey(actor);
    setSavingKey(savingActorKey);
    clearFeedback();
    try {
      const saved = await putNotificationPrefs(actor, prefs);
      patchEntry(actor, (entry) => ({ ...entry, prefs: saved }));
      if (sameActor(actor, currentActor())) setDetailDirty(false);
      setMessage(
        tpl(NOTIFICATIONS.prefs.save_success, {
          channel: actor.channel,
          label: actorLabel(actor),
        }),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      throw e;
    } finally {
      setSavingKey("");
    }
  };

  const toggleRosterEnabled = async (actor: ActorRef, enabled: boolean) => {
    const entry = roster().find((rosterEntry) =>
      sameActor(rosterEntry.actor, actor),
    );
    if (!entry) return;
    const next = { ...entry.prefs, enabled };
    patchEntry(actor, (rosterEntry) => ({ ...rosterEntry, prefs: next }));
    try {
      await savePrefs(actor, next);
    } catch {
      patchEntry(actor, (rosterEntry) => ({ ...rosterEntry, prefs: entry.prefs }));
    }
  };

  const chooseActor = async (actor: ActorRef) => {
    clearFeedback();
    setSelectedKey(actorKey(actor));
    setDetailDirty(false);
    if (!roster().some((rosterEntry) => sameActor(rosterEntry.actor, actor))) {
      try {
        upsertEntry(await fetchEntry(actor));
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        upsertEntry({ actor, prefs: { ...DEFAULT_NOTIFICATION_PREFS }, kindTags: [] });
      }
    }
  };

  const applyManual = () => {
    const manualActor = manual();
    if (!manualActor.channel.trim() || !manualActor.user_id.trim()) return;
    void chooseActor({
      channel: manualActor.channel.trim(),
      user_id: manualActor.user_id.trim(),
      channel_scope: manualActor.channel_scope?.trim() || undefined,
    });
  };

  const editCurrent = (
    updater: (prefs: NotificationPrefs) => NotificationPrefs,
  ) => {
    const actor = currentActor();
    if (!actor) return;
    patchEntry(actor, (entry) => ({ ...entry, prefs: updater(entry.prefs) }));
    setDetailDirty(true);
  };

  const updateCurrentPrefs = (
    patch:
      | Partial<NotificationPrefs>
      | ((prefs: NotificationPrefs) => Partial<NotificationPrefs>),
  ) => {
    editCurrent((prefs) => ({
      ...prefs,
      ...(typeof patch === "function" ? patch(prefs) : patch),
    }));
  };

  const handleAllowToggle = (tag: string) => {
    editCurrent((prefs) => {
      const nextTags = toggleTag(prefs.allow_kinds ?? [], tag);
      return { ...prefs, allow_kinds: nextTags.length === 0 ? null : nextTags };
    });
  };

  const handleBlockToggle = (tag: string) => {
    editCurrent((prefs) => ({
      ...prefs,
      blocked_kinds: toggleTag(prefs.blocked_kinds ?? [], tag),
    }));
  };

  const handleImmediateToggle = (tag: string) => {
    editCurrent((prefs) => {
      const nextTags = toggleTag(prefs.immediate_kinds ?? [], tag);
      return { ...prefs, immediate_kinds: nextTags.length === 0 ? null : nextTags };
    });
  };

  // digest_slots 操作:null = 沿用全局 default_slots,[] = 关 digest,[..] = 自定义。
  // 每个 slot 是 {id, time, label?, floor_macro?},UI 只编辑 time;新增时给 id
  // `slot_<n>`,label/floor_macro 留空(后端默认即可),已存在 slot 的 label/floor_macro
  // 如果是后端蒸馏出来的会原样透传不破坏。
  const [slotDraft, setSlotDraft] = createSignal("");
  const addSlot = () => {
    const slotTime = slotDraft().trim();
    if (!isValidDigestSlotTime(slotTime)) return;
    editCurrent((prefs) => {
      const existingSlots = prefs.digest_slots ?? [];
      if (existingSlots.some((slot) => slot.time === slotTime)) return prefs; // 同时刻去重
      const slotId = `slot_${existingSlots.length}`;
      return {
        ...prefs,
        digest_slots: sortDigestSlots([
          ...existingSlots,
          { id: slotId, time: slotTime },
        ]),
      };
    });
    setSlotDraft("");
  };
  const removeSlot = (id: string) => {
    updateCurrentPrefs((prefs) => ({
      digest_slots: (prefs.digest_slots ?? []).filter((slot) => slot.id !== id),
    }));
  };
  const resetSlotsToGlobal = () => {
    updateCurrentPrefs({ digest_slots: null });
  };
  const muteAllDigest = () => {
    updateCurrentPrefs({ digest_slots: [] });
  };

  // quiet_hours 操作:null = 关勿扰;{from,to,exempt_kinds} = 启用。from==to 等价于
  // 全天静音的歧义形式,后端会拒绝(空区间永远 false),UI 提示用户避免。
  const setQuietFrom = (raw: string) => {
    const quietStart = raw.trim();
    if (!isValidDigestSlotTime(quietStart)) return;
    editCurrent((prefs) => ({
      ...prefs,
      quiet_hours: {
        from: quietStart,
        to: prefs.quiet_hours?.to ?? "08:00",
        exempt_kinds: prefs.quiet_hours?.exempt_kinds ?? [],
      },
    }));
  };
  const setQuietTo = (raw: string) => {
    const quietEnd = raw.trim();
    if (!isValidDigestSlotTime(quietEnd)) return;
    editCurrent((prefs) => ({
      ...prefs,
      quiet_hours: {
        from: prefs.quiet_hours?.from ?? "00:00",
        to: quietEnd,
        exempt_kinds: prefs.quiet_hours?.exempt_kinds ?? [],
      },
    }));
  };
  const enableQuiet = () => {
    editCurrent((prefs) =>
      prefs.quiet_hours
        ? prefs
        : {
            ...prefs,
            quiet_hours: { from: "00:00", to: "08:00", exempt_kinds: [] },
          },
    );
  };
  const clearQuiet = () => {
    updateCurrentPrefs({ quiet_hours: null });
  };
  const toggleQuietExempt = (tag: string) => {
    editCurrent((prefs) => {
      if (!prefs.quiet_hours) return prefs;
      const nextTags = toggleTag(prefs.quiet_hours.exempt_kinds, tag);
      return {
        ...prefs,
        quiet_hours: { ...prefs.quiet_hours, exempt_kinds: nextTags },
      };
    });
  };

  const handleTimezoneInput = (raw: string) => {
    const timezone = raw.trim();
    updateCurrentPrefs({ timezone: timezone === "" ? null : timezone });
  };

  const handlePriceHighInput = (raw: string) => {
    const priceThreshold = raw.trim();
    if (priceThreshold === "") {
      editCurrent((prefs) => ({ ...prefs, price_high_pct_override: null }));
      return;
    }
    const parsedThreshold = Number(priceThreshold);
    editCurrent((prefs) => ({
      ...prefs,
      price_high_pct_override: Number.isFinite(parsedThreshold)
        ? parsedThreshold
        : prefs.price_high_pct_override,
    }));
  };

  const submitDetail = async () => {
    const actor = currentActor();
    const entry = currentEntry();
    if (!actor || !entry) return;
    try {
      await savePrefs(actor, entry.prefs);
    } catch {
      /* savePrefs 已把 error 落到 banner */
    }
  };

  return (
    <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
      <div class="flex items-center justify-between">
        <div>
          <div class="text-sm font-bold text-[color:var(--text-primary)]">
            {NOTIFICATIONS.prefs.title}
          </div>
          <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">
            {NOTIFICATIONS.prefs.subtitle}
          </div>
        </div>
        <button
          type="button"
          class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] transition hover:text-[color:var(--text-primary)]"
          onClick={() => void refreshRoster()}
        >
          {NOTIFICATIONS.prefs.refresh_button}
        </button>
      </div>

      <div class="mt-4">
        <div class="text-[11px] font-semibold text-[color:var(--text-secondary)]">
          {NOTIFICATIONS.prefs.actor_list_label}
        </div>
        <div class="mt-2 divide-y divide-[color:var(--border)] rounded-md border border-[color:var(--border)] bg-[color:var(--surface)]">
          <Show when={rosterLoading()}>
            <div class="px-3 py-2 text-[11px] text-[color:var(--text-secondary)]">
              {NOTIFICATIONS.prefs.actor_loading}
            </div>
          </Show>
          <Show when={!rosterLoading() && roster().length === 0}>
            <div class="px-3 py-2 text-[11px] text-[color:var(--text-secondary)]">
              {NOTIFICATIONS.prefs.actor_empty}
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
                    <span>{isSaving() ? NOTIFICATIONS.prefs.saving_label : entry.prefs.enabled ? NOTIFICATIONS.prefs.pushing_label : NOTIFICATIONS.prefs.off_label}</span>
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
          placeholder={NOTIFICATIONS.prefs.manual_channel_placeholder}
          value={manual().channel}
          onInput={(e) =>
            updateManual({ channel: e.currentTarget.value })
          }
        />
        <input
          class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
          placeholder={NOTIFICATIONS.prefs.manual_user_placeholder}
          value={manual().user_id}
          onInput={(e) =>
            updateManual({ user_id: e.currentTarget.value })
          }
        />
        <div class="flex gap-1">
          <input
            class="flex-1 rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
            placeholder={NOTIFICATIONS.prefs.manual_scope_placeholder}
            value={manual().channel_scope ?? ""}
            onInput={(e) =>
              updateManual({ channel_scope: e.currentTarget.value })
            }
          />
          <button
            type="button"
            class="rounded-md border border-[color:var(--border)] px-2 text-[11px]"
            onClick={applyManual}
          >
            {NOTIFICATIONS.prefs.manual_load}
          </button>
        </div>
      </div>

      <Show when={currentActor() && currentEntry()}>
        <div class="mt-5 space-y-4 rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
          <div class="flex items-center justify-between">
            <div class="text-[11px] font-semibold text-[color:var(--text-primary)]">
              {tpl(NOTIFICATIONS.prefs.detail_title, {
                channel: currentActor()!.channel,
                label: actorLabel(currentActor()!),
              })}
            </div>
            <div class="text-[10px] text-[color:var(--text-secondary)]">
              {NOTIFICATIONS.prefs.detail_hint}
            </div>
          </div>
          <div class="flex items-center justify-between">
            <label class="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={currentPrefs().portfolio_only}
                onChange={(e) =>
                  editCurrent((prefs) => ({
                    ...prefs,
                    portfolio_only: e.currentTarget.checked,
                  }))
                }
              />
              <span>{NOTIFICATIONS.prefs.portfolio_only}</span>
            </label>
            <label class="flex items-center gap-2 text-sm">
              <span>{NOTIFICATIONS.prefs.min_severity}</span>
              <select
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                value={currentPrefs().min_severity}
                onChange={(e) =>
                  editCurrent((prefs) => ({
                    ...prefs,
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
              {NOTIFICATIONS.prefs.allow_kinds_label}
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
              {NOTIFICATIONS.prefs.block_kinds_label}
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
              {NOTIFICATIONS.prefs.cadence_title}
            </div>

            <label class="flex flex-col gap-1 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                {NOTIFICATIONS.prefs.timezone_label}
              </span>
              <input
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                placeholder={NOTIFICATIONS.prefs.timezone_placeholder}
                value={currentPrefs().timezone ?? ""}
                onInput={(e) => handleTimezoneInput(e.currentTarget.value)}
              />
            </label>

            <div class="flex flex-col gap-1.5 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                {NOTIFICATIONS.prefs.digest_label}
              </span>
              <div class="flex flex-wrap items-center gap-1">
                <Show when={currentPrefs().digest_slots === null}>
                  <span class="text-[10px] italic text-[color:var(--text-secondary)]">
                    {NOTIFICATIONS.prefs.digest_inherit_global}
                  </span>
                </Show>
                <Show when={currentPrefs().digest_slots?.length === 0}>
                  <span class="rounded-md border border-amber-500 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-500">
                    {NOTIFICATIONS.prefs.digest_off_badge}
                  </span>
                </Show>
                <For each={currentPrefs().digest_slots ?? []}>
                  {(slot) => (
                    <span
                      class="inline-flex items-center gap-1 rounded-md border border-emerald-500 bg-emerald-500/10 px-2 py-0.5 font-mono text-[11px] text-emerald-600"
                      title={slot.label ?? slot.id}
                    >
                      {slot.time}
                      <Show when={slot.label}>
                        <span class="font-sans not-italic opacity-70">
                          · {slot.label}
                        </span>
                      </Show>
                      <button
                        type="button"
                        class="-mr-0.5 rounded text-emerald-700 hover:text-rose-500"
                        title={NOTIFICATIONS.prefs.digest_remove_title}
                        onClick={() => removeSlot(slot.id)}
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
                  value={slotDraft()}
                  onInput={(e) => setSlotDraft(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      addSlot();
                    }
                  }}
                />
                <button
                  type="button"
                  class="rounded-md border border-emerald-500 px-2 py-1 text-[11px] text-emerald-600 hover:bg-emerald-500/10 disabled:opacity-40"
                  disabled={!isValidDigestSlotTime(slotDraft().trim())}
                  onClick={addSlot}
                >
                  {NOTIFICATIONS.prefs.digest_add_button}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                  onClick={resetSlotsToGlobal}
                  title={NOTIFICATIONS.prefs.digest_reset_global_title}
                >
                  {NOTIFICATIONS.prefs.digest_reset_global}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-amber-500 px-2 py-1 text-[11px] text-amber-500 hover:bg-amber-500/10"
                  onClick={muteAllDigest}
                  title={NOTIFICATIONS.prefs.digest_mute_title}
                >
                  {NOTIFICATIONS.prefs.digest_mute_button}
                </button>
              </div>
              <Show
                when={(() => {
                  const digestSlots = currentPrefs().digest_slots ?? [];
                  const quietHours = currentPrefs().quiet_hours;
                  if (!quietHours) return false;
                  return digestSlots.some((digestSlot) =>
                    timeFallsInQuiet(digestSlot.time, quietHours),
                  );
                })()}
              >
                <span class="rounded-md border border-rose-500/50 bg-rose-500/10 px-2 py-1 text-[10px] text-rose-500">
                  {tpl(NOTIFICATIONS.prefs.digest_quiet_warning, {
                    from: currentPrefs().quiet_hours!.from,
                    to: currentPrefs().quiet_hours!.to,
                  })}
                </span>
              </Show>
            </div>

            <label class="flex flex-col gap-1 text-[11px]">
              <span class="text-[color:var(--text-secondary)]">
                {NOTIFICATIONS.prefs.price_label}
              </span>
              <input
                type="number"
                step="0.1"
                min="0"
                max="50"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs"
                placeholder={NOTIFICATIONS.prefs.price_placeholder}
                value={currentPrefs().price_high_pct_override ?? ""}
                onInput={(e) => handlePriceHighInput(e.currentTarget.value)}
              />
            </label>

            <div>
              <div class="text-[11px] text-[color:var(--text-secondary)]">
                {NOTIFICATIONS.prefs.immediate_label}
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

            <div class="space-y-1.5 rounded-md border border-dashed border-[color:var(--border)] p-2.5">
              <div class="flex items-center justify-between text-[11px]">
                <span class="font-semibold text-[color:var(--text-secondary)]">
                  {NOTIFICATIONS.prefs.quiet_section}
                </span>
                <Show
                  when={currentPrefs().quiet_hours}
                  fallback={
                    <button
                      type="button"
                      class="rounded-md border border-[color:var(--accent)] px-2 py-0.5 text-[10px] text-[color:var(--accent)] hover:bg-[color:var(--accent)]/10"
                      onClick={enableQuiet}
                    >
                      {NOTIFICATIONS.prefs.quiet_enable_button}
                    </button>
                  }
                >
                  <button
                    type="button"
                    class="rounded-md border border-rose-500 px-2 py-0.5 text-[10px] text-rose-500 hover:bg-rose-500/10"
                    onClick={clearQuiet}
                  >
                    {NOTIFICATIONS.prefs.quiet_disable_button}
                  </button>
                </Show>
              </div>
              <Show when={currentPrefs().quiet_hours}>
                <div class="flex flex-wrap items-center gap-2 text-[11px]">
                  <label class="flex items-center gap-1">
                    <span class="text-[color:var(--text-secondary)]">{NOTIFICATIONS.prefs.quiet_from}</span>
                    <input
                      type="time"
                      class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 font-mono text-xs"
                      value={currentPrefs().quiet_hours!.from}
                      onInput={(e) => setQuietFrom(e.currentTarget.value)}
                    />
                  </label>
                  <label class="flex items-center gap-1">
                    <span class="text-[color:var(--text-secondary)]">{NOTIFICATIONS.prefs.quiet_to}</span>
                    <input
                      type="time"
                      class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 font-mono text-xs"
                      value={currentPrefs().quiet_hours!.to}
                      onInput={(e) => setQuietTo(e.currentTarget.value)}
                    />
                  </label>
                  <span class="text-[10px] italic text-[color:var(--text-secondary)]">
                    {NOTIFICATIONS.prefs.quiet_hint}
                  </span>
                </div>
                <Show
                  when={
                    currentPrefs().quiet_hours!.from ===
                    currentPrefs().quiet_hours!.to
                  }
                >
                  <span class="rounded-md border border-rose-500/50 bg-rose-500/10 px-2 py-0.5 text-[10px] text-rose-500">
                    {NOTIFICATIONS.prefs.quiet_invalid}
                  </span>
                </Show>
                <div class="text-[10px] text-[color:var(--text-secondary)]">
                  {NOTIFICATIONS.prefs.quiet_exempt_hint}
                </div>
                <div class="flex flex-wrap gap-1">
                  <For each={currentKindTags()}>
                    {(tag) => {
                      const selected = () =>
                        (currentPrefs().quiet_hours?.exempt_kinds ?? []).includes(
                          tag,
                        );
                      return (
                        <button
                          type="button"
                          class="rounded-md border px-2 py-0.5 text-[11px]"
                          classList={{
                            "border-sky-500 bg-sky-500/10 text-sky-600":
                              selected(),
                            "border-[color:var(--border)] text-[color:var(--text-secondary)]":
                              !selected(),
                          }}
                          onClick={() => toggleQuietExempt(tag)}
                        >
                          {tag}
                        </button>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2">
            <Show when={detailDirty()}>
              <span class="text-[10px] text-amber-500">{NOTIFICATIONS.prefs.dirty_label}</span>
            </Show>
            <button
              type="button"
              class="rounded-md bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white disabled:opacity-50"
              disabled={savingKey() === currentActorKey() || !detailDirty()}
              onClick={() => void submitDetail()}
            >
              {savingKey() === currentActorKey()
                ? NOTIFICATIONS.prefs.save_detail_saving
                : NOTIFICATIONS.prefs.save_detail_button}
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
