import { For, Show, createMemo } from "solid-js"
import { useCompanyProfiles } from "@/context/company-profiles"
import { usePortfolio } from "@/context/portfolio"
import { useSessions } from "@/context/sessions"
import {
  actorKey,
  mergeActorSummaries,
  type ActorListItem,
  type ActorRef,
} from "@/lib/actors"
import { USERS } from "@/lib/admin-content/users"
import { tpl } from "@/lib/i18n"

type ActorSelectProps = {
  value: string
  onChange: (actor: ActorRef | null) => void
  allowAll?: boolean
  allLabel?: string
  disabled?: boolean
  class?: string
}

function optionLabel(item: ActorListItem): string {
  const parts = [item.actor.channel, item.actor.user_id]
  if (item.actor.channel_scope) parts.push(item.actor.channel_scope)
  const tags: string[] = []
  if (item.holdingsCount) tags.push(tpl(USERS.select.tag_holdings, { count: item.holdingsCount }))
  if (item.profileCount) tags.push(tpl(USERS.select.tag_profiles, { count: item.profileCount }))
  if (item.lastSessionTime) tags.push(USERS.select.tag_sessions)
  return tags.length > 0 ? `${parts.join(" / ")} (${tags.join(", ")})` : parts.join(" / ")
}

export function ActorSelect(props: ActorSelectProps) {
  const sessions = useSessions()
  const portfolio = usePortfolio()
  const companyProfiles = useCompanyProfiles()

  const options = createMemo(() =>
    mergeActorSummaries({
      sessions: sessions.state.users.filter((user) => user.user_id !== "ME"),
      portfolios: portfolio.actorsList() ?? [],
      profiles: companyProfiles.actorsList() ?? [],
    }),
  )

  const selected = createMemo(() =>
    options().find((item) => item.key === props.value)?.actor ?? null,
  )

  return (
    <select
      value={props.value}
      disabled={props.disabled || options().length === 0}
      onChange={(e) => {
        const value = e.currentTarget.value
        if (!value) {
          props.onChange(null)
          return
        }
        const actor = options().find((item) => item.key === value)?.actor ?? selected()
        props.onChange(actor)
      }}
      class={[
        "min-w-[16rem] rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)] disabled:opacity-50",
        props.class ?? "",
      ].join(" ")}
    >
      <Show
        when={options().length > 0}
        fallback={<option value="">{USERS.select.no_options}</option>}
      >
        <Show when={props.allowAll}>
          <option value="">{props.allLabel ?? USERS.select.all_label}</option>
        </Show>
        <Show when={!props.allowAll && !props.value}>
          <option value="">{USERS.select.placeholder}</option>
        </Show>
        <For each={options()}>
          {(item) => <option value={actorKey(item.actor)}>{optionLabel(item)}</option>}
        </For>
      </Show>
    </select>
  )
}
