import { For, Show, createSignal } from "solid-js"
import { ActorSelect } from "@/components/actor-select"
import {
  getSchedule,
  type ScheduleEntry,
  type ScheduleOverview,
  type ScheduleSource,
} from "@/lib/api"
import { actorKey as uiActorKey, type ActorRef } from "@/lib/actors"
import { SCHEDULE } from "@/lib/admin-content/schedule"
import { tpl } from "@/lib/i18n"

function sourceLabel(s: ScheduleSource): string {
  switch (s) {
    case "digest":
      return SCHEDULE.source.digest
    case "cron_job":
      return SCHEDULE.source.cron_job
  }
}

function sourceBadgeClass(s: ScheduleSource): string {
  switch (s) {
    case "digest":
      return "text-purple-300 bg-purple-500/15"
    case "cron_job":
      return "text-emerald-300 bg-emerald-500/15"
  }
}

function activeCellClass(held: boolean): string {
  return held
    ? "text-amber-300 bg-amber-500/15"
    : "text-emerald-300 bg-emerald-500/15"
}

function scheduleActorKey(actor: ActorRef): string {
  return `${actor.channel.trim()}::${(actor.channel_scope ?? "").trim()}::${actor.user_id.trim()}`
}

export default function SchedulePage() {
  const [selectedActor, setSelectedActor] = createSignal<ActorRef | null>(null)
  const [overview, setOverview] = createSignal<ScheduleOverview | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [err, setErr] = createSignal<string | null>(null)

  async function refresh(actor = selectedActor()) {
    if (!actor) {
      setErr(SCHEDULE.page.err_pick_user)
      setOverview(null)
      return
    }
    setLoading(true)
    setErr(null)
    try {
      const data = await getSchedule(scheduleActorKey(actor))
      setOverview(data)
    } catch (e) {
      setErr(String(e))
      setOverview(null)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div class="flex h-full min-h-0 flex-col gap-4 p-4 text-sm">
      <div class="flex flex-wrap items-center gap-3">
        <h1 class="text-lg font-semibold text-[color:var(--text-primary)]">
          {SCHEDULE.page.title}
        </h1>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{SCHEDULE.page.user_label}</span>
          <ActorSelect
            value={selectedActor() ? uiActorKey(selectedActor()!) : ""}
            onChange={(actor) => {
              setSelectedActor(actor)
              if (actor) void refresh(actor)
              else setOverview(null)
            }}
          />
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={loading()}
          class="rounded border border-[color:var(--border)] px-3 py-1 text-xs text-[color:var(--text-primary)] hover:bg-white/5 disabled:opacity-50"
        >
          {loading() ? SCHEDULE.page.loading_button : SCHEDULE.page.query_button}
        </button>
      </div>

      <Show when={err()}>
        <div class="rounded border border-rose-500/30 bg-rose-500/10 p-3 text-xs text-rose-300">
          {err()}
        </div>
      </Show>

      <Show when={overview()}>
        {(data) => (
          <div class="flex flex-col gap-4">
            <div class="flex flex-wrap gap-3 text-xs">
              <div class="rounded border border-[color:var(--border)] bg-white/5 px-3 py-2">
                <div class="text-[color:var(--text-muted)]">{SCHEDULE.card.actor}</div>
                <div class="font-mono text-[color:var(--text-primary)]">
                  {data().actor}
                </div>
              </div>
              <div class="rounded border border-[color:var(--border)] bg-white/5 px-3 py-2">
                <div class="text-[color:var(--text-muted)]">{SCHEDULE.card.timezone}</div>
                <div class="text-[color:var(--text-primary)]">
                  {data().timezone}
                </div>
              </div>
              <div class="rounded border border-[color:var(--border)] bg-white/5 px-3 py-2">
                <div class="text-[color:var(--text-muted)]">{SCHEDULE.card.quiet_hours}</div>
                <div class="text-[color:var(--text-primary)]">
                  <Show
                    when={data().quiet_hours}
                    fallback={
                      <span class="text-[color:var(--text-muted)]">{SCHEDULE.card.quiet_disabled}</span>
                    }
                  >
                    {(qh) => (
                      <>
                        🌙 {qh().from} – {qh().to}
                        <Show when={qh().exempt_kinds.length > 0}>
                          <span class="ml-2 text-[color:var(--text-muted)]">
                            {tpl(SCHEDULE.card.quiet_exempt_prefix, { kinds: qh().exempt_kinds.join(", ") })}
                          </span>
                        </Show>
                      </>
                    )}
                  </Show>
                </div>
              </div>
              <div class="rounded border border-[color:var(--border)] bg-white/5 px-3 py-2">
                <div class="text-[color:var(--text-muted)]">{SCHEDULE.card.immediate}</div>
                <div class="text-[color:var(--text-primary)]">
                  {data().immediate.enabled ? SCHEDULE.card.immediate_enabled : SCHEDULE.card.immediate_disabled}
                  {SCHEDULE.card.immediate_min_prefix}
                  {data().immediate.min_severity}
                  <Show when={data().immediate.portfolio_only}>
                    {SCHEDULE.card.immediate_only_portfolio}
                  </Show>
                  <Show when={data().immediate.price_high_pct != null}>
                    {tpl(SCHEDULE.card.immediate_price_threshold, { pct: data().immediate.price_high_pct ?? "" })}
                  </Show>
                </div>
              </div>
            </div>

            <div class="overflow-x-auto rounded border border-[color:var(--border)]">
              <table class="min-w-full text-xs">
                <thead class="bg-white/5 text-[color:var(--text-muted)]">
                  <tr>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_time}</th>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_type}</th>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_content}</th>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_freq}</th>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_active}</th>
                    <th class="px-3 py-2 text-left">{SCHEDULE.table.col_hint}</th>
                  </tr>
                </thead>
                <tbody>
                  <Show
                    when={data().schedule.length > 0}
                    fallback={
                      <tr>
                        <td
                          colspan="6"
                          class="px-3 py-6 text-center text-[color:var(--text-muted)]"
                        >
                          {SCHEDULE.table.empty}
                        </td>
                      </tr>
                    }
                  >
                    <For each={data().schedule}>
                      {(e: ScheduleEntry) => (
                        <tr class="border-t border-[color:var(--border)]">
                          <td class="px-3 py-2 font-mono text-[color:var(--text-primary)]">
                            {e.time_local}
                          </td>
                          <td class="px-3 py-2">
                            <span
                              class={`rounded px-2 py-0.5 text-xs ${sourceBadgeClass(e.source)}`}
                            >
                              {sourceLabel(e.source)}
                            </span>
                          </td>
                          <td class="px-3 py-2 text-[color:var(--text-primary)]">
                            {e.content_hint}
                          </td>
                          <td class="px-3 py-2 text-[color:var(--text-muted)]">
                            {e.frequency}
                          </td>
                          <td class="px-3 py-2">
                            <span
                              class={`rounded px-2 py-0.5 text-xs ${activeCellClass(e.will_be_held_by_quiet)}`}
                            >
                              {e.will_be_held_by_quiet
                                ? SCHEDULE.table.cell_quiet_held
                                : e.bypass_quiet_hours
                                  ? SCHEDULE.table.cell_bypass_quiet
                                  : SCHEDULE.table.cell_active}
                            </span>
                          </td>
                          <td class="px-3 py-2 font-mono text-[10px] text-[color:var(--text-muted)]">
                            {e.edit_hint}
                          </td>
                        </tr>
                      )}
                    </For>
                  </Show>
                </tbody>
              </table>
            </div>

            <Show
              when={
                data().immediate.blocked_kinds.length > 0 ||
                (data().immediate.allow_kinds &&
                  data().immediate.allow_kinds!.length > 0) ||
                data().immediate.exempt_in_quiet.length > 0
              }
            >
              <div class="rounded border border-[color:var(--border)] bg-white/5 p-3 text-xs text-[color:var(--text-muted)]">
                <Show when={data().immediate.blocked_kinds.length > 0}>
                  <div>
                    {SCHEDULE.filters.blocked_kinds}
                    <span class="text-[color:var(--text-primary)]">
                      {data().immediate.blocked_kinds.join(", ")}
                    </span>
                  </div>
                </Show>
                <Show
                  when={
                    data().immediate.allow_kinds &&
                    data().immediate.allow_kinds!.length > 0
                  }
                >
                  <div>
                    {SCHEDULE.filters.allow_kinds}
                    <span class="text-[color:var(--text-primary)]">
                      {data().immediate.allow_kinds!.join(", ")}
                    </span>
                  </div>
                </Show>
                <Show when={data().immediate.exempt_in_quiet.length > 0}>
                  <div>
                    {SCHEDULE.filters.exempt_in_quiet}
                    <span class="text-[color:var(--text-primary)]">
                      {data().immediate.exempt_in_quiet.join(", ")}
                    </span>
                  </div>
                </Show>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  )
}
