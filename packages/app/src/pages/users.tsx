import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { useNavigate, useParams } from "@solidjs/router"
import { For, Show, createEffect, createMemo } from "solid-js"
import { CompanyProfileDetail } from "@/components/company-profile-detail"
import { PortfolioDetail } from "@/components/portfolio-detail"
import { UserMainlineView } from "@/components/user-mainline-view"
import { useBackend } from "@/context/backend"
import { useCompanyProfiles } from "@/context/company-profiles"
import { usePortfolio } from "@/context/portfolio"
import { useResearch } from "@/context/research"
import { useSessions } from "@/context/sessions"
import { actorFromUser, actorKey, parseActorKey, type ActorRef } from "@/lib/actors"
import { USERS } from "@/lib/admin-content/users"
import { tpl } from "@/lib/i18n"

type UsersTab = "portfolio" | "profiles" | "mainline" | "sessions" | "research"

const TAB_LIST: { id: UsersTab; labelKey: keyof typeof USERS.page; capability?: string }[] = [
  { id: "portfolio", labelKey: "tab_portfolio" },
  { id: "profiles", labelKey: "tab_profiles", capability: "company_profiles" },
  { id: "mainline", labelKey: "tab_mainline" },
  { id: "sessions", labelKey: "tab_sessions" },
  { id: "research", labelKey: "tab_research", capability: "research" },
]

function TabBtn(props: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      class={[
        "px-5 py-2.5 text-sm font-medium transition border-b-2 -mb-px",
        props.active
          ? "border-[color:var(--accent)] text-[color:var(--text-primary)]"
          : "border-transparent text-[color:var(--text-muted)] hover:text-[color:var(--text-primary)]",
      ].join(" ")}
    >
      {props.label}
    </button>
  )
}

function formatDate(iso?: string) {
  if (!iso) return "—"
  try {
    return new Date(iso).toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    })
  } catch {
    return iso
  }
}

function UserSessionsView(props: { actor: ActorRef }) {
  const navigate = useNavigate()
  const sessions = useSessions()

  const userSessions = createMemo(() =>
    sessions.state.users
      .filter((u) => {
        const a = actorFromUser(u)
        return (
          a.channel === props.actor.channel &&
          a.user_id === props.actor.user_id &&
          (a.channel_scope ?? "") === (props.actor.channel_scope ?? "")
        )
      })
      .sort((a, b) => (b.last_time ?? "").localeCompare(a.last_time ?? "")),
  )

  return (
    <div class="hf-scrollbar h-full overflow-y-auto bg-[color:var(--surface)] p-6">
      <Show
        when={userSessions().length > 0}
        fallback={
          <EmptyState
            title={USERS.sessions.empty_title}
            description={USERS.sessions.empty_description}
          />
        }
      >
        <div class="space-y-2">
          <For each={userSessions()}>
            {(u) => (
              <button
                type="button"
                class="block w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4 text-left transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)]"
                onClick={() =>
                  navigate(`/sessions/${encodeURIComponent(u.session_id)}`)
                }
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                    {u.session_label || u.user_id}
                  </div>
                  <div class="text-[11px] text-[color:var(--text-muted)]">
                    {formatDate(u.last_time)} · {tpl(USERS.sessions.message_count, { count: u.message_count })}
                  </div>
                </div>
                <div class="mt-2 line-clamp-2 text-xs text-[color:var(--text-secondary)]">
                  <span class="text-[color:var(--text-muted)]">{u.last_role}:</span>{" "}
                  {u.last_message || USERS.sessions.empty_message}
                </div>
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  )
}

function UserResearchView(props: { actor: ActorRef }) {
  const navigate = useNavigate()
  const portfolio = usePortfolio()
  const research = useResearch()

  const symbols = createMemo(() => {
    const set = new Set<string>()
    for (const h of portfolio.holdingsList()) set.add(h.symbol.toUpperCase())
    for (const w of portfolio.watchlist()) set.add(w.symbol.toUpperCase())
    return Array.from(set).sort()
  })

  const relatedTasks = createMemo(() => {
    const syms = symbols()
    if (syms.length === 0) return []
    return research.state.tasks.filter((t) => {
      const name = t.company_name.toUpperCase()
      return syms.some((s) => name.includes(s))
    })
  })

  const startFor = (symbol: string) => {
    navigate(`/research?symbol=${encodeURIComponent(symbol)}`)
  }

  return (
    <div class="hf-scrollbar h-full overflow-y-auto bg-[color:var(--surface)] p-6">
      <Show
        when={symbols().length > 0}
        fallback={
          <EmptyState
            title={USERS.research.empty_title}
            description={USERS.research.empty_description}
          />
        }
      >
        <div class="mb-6 rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
          <div class="mb-2 text-sm font-semibold">{USERS.research.symbols_title}</div>
          <div class="flex flex-wrap gap-2">
            <For each={symbols()}>
              {(s) => (
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1 text-xs font-mono text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)] hover:text-[color:var(--text-primary)]"
                  onClick={() => startFor(s)}
                  title={tpl(USERS.research.start_for_title, { symbol: s })}
                >
                  {s}
                </button>
              )}
            </For>
          </div>
          <div class="mt-2 text-[11px] text-[color:var(--text-muted)]">
            {USERS.research.symbols_hint}
          </div>
        </div>

        <div class="text-sm font-semibold mb-3">{USERS.research.related_title}</div>
        <Show
          when={relatedTasks().length > 0}
          fallback={
            <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
              {USERS.research.related_empty}
            </div>
          }
        >
          <div class="space-y-2">
            <For each={relatedTasks()}>
              {(task) => (
                <button
                  type="button"
                  class="block w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4 text-left transition hover:border-[color:var(--accent)]"
                  onClick={() =>
                    navigate(`/research/${encodeURIComponent(task.task_id)}`)
                  }
                >
                  <div class="flex items-center justify-between gap-3">
                    <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                      {task.company_name}
                    </div>
                    <div class="text-[11px] text-[color:var(--text-muted)]">
                      {task.status} · {task.progress}
                    </div>
                  </div>
                  <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                    {tpl(USERS.research.created_at, { date: formatDate(task.created_at) })}
                  </div>
                </button>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  )
}

export default function UsersPage() {
  const params = useParams()
  const navigate = useNavigate()
  const backend = useBackend()
  const portfolio = usePortfolio()
  const companyProfiles = useCompanyProfiles()

  const currentActor = createMemo<ActorRef | undefined>(() =>
    parseActorKey(params.actorKey ? decodeURIComponent(params.actorKey) : undefined),
  )

  const tab = createMemo<UsersTab>(() => {
    const t = params.tab as UsersTab | undefined
    if (t === "profiles" || t === "sessions" || t === "research" || t === "mainline") return t
    return "portfolio"
  })

  const tabsAvailable = createMemo(() =>
    TAB_LIST.filter((t) => !t.capability || backend.hasCapability(t.capability)),
  )

  // URL → context 单向同步:把当前 actor 推到 portfolio 和 companyProfiles 两个 store
  createEffect(() => {
    const actor = currentActor()
    portfolio.selectActor(actor ?? undefined)
    companyProfiles.selectActor(actor ?? null)
  })

  const switchTab = (next: UsersTab) => {
    const actor = currentActor()
    const keyPart = actor ? encodeURIComponent(actorKey(actor)) : ""
    navigate(`/users/${keyPart}/${next}`)
  }

  return (
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <Show
        when={currentActor()}
        fallback={
          <div class="flex h-full items-center justify-center bg-[color:var(--surface)] p-8">
            <EmptyState
              title={USERS.page.empty_title}
              description={USERS.page.empty_description}
            />
          </div>
        }
      >
        {(actor) => (
          <>
            {/* Actor 信息 + tab */}
            <div class="flex shrink-0 items-center gap-4 border-b border-[color:var(--border)] bg-[color:var(--panel)] px-4">
              <div class="flex items-baseline gap-2 py-2">
                <span class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {actor().user_id}
                </span>
                <span class="text-[11px] text-[color:var(--text-muted)]">
                  {actor().channel}
                  {actor().channel_scope ? ` · ${actor().channel_scope}` : ""}
                </span>
              </div>
              <div class="ml-auto flex items-center gap-2">
                <Button
                  variant="ghost"
                  class="h-7 px-2 text-[11px]"
                  onClick={() => {
                    void portfolio.refetch()
                    void companyProfiles.refetchProfiles()
                  }}
                >
                  {USERS.page.refresh_button}
                </Button>
              </div>
            </div>
            <div class="flex shrink-0 border-b border-[color:var(--border)] px-2">
              <For each={tabsAvailable()}>
                {(t) => (
                  <TabBtn
                    label={USERS.page[t.labelKey]}
                    active={tab() === t.id}
                    onClick={() => switchTab(t.id)}
                  />
                )}
              </For>
            </div>

            <div class="min-h-0 flex-1 overflow-hidden">
              <Show when={tab() === "portfolio"}>
                <PortfolioDetail />
              </Show>
              <Show when={tab() === "profiles"}>
                <CompanyProfileDetail />
              </Show>
              <Show when={tab() === "mainline"}>
                <UserMainlineView actor={actor()} />
              </Show>
              <Show when={tab() === "sessions"}>
                <UserSessionsView actor={actor()} />
              </Show>
              <Show when={tab() === "research"}>
                <UserResearchView actor={actor()} />
              </Show>
            </div>
          </>
        )}
      </Show>
    </div>
  )
}
