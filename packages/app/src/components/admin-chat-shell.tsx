import { useNavigate } from "@solidjs/router"
import { Show, createMemo } from "solid-js"
import { ChatView } from "@/components/chat-view"
import { useSessions } from "@/context/sessions"
import { actorFromSessionId, actorFromUser, actorKey, type ActorRef } from "@/lib/actors"
import { SESSIONS } from "@/lib/admin-content/sessions"

/**
 * Admin 端会话页的外壳 — 在 ChatView 上方加一行 actor 工具条,
 * 用来从会话快速跳到该用户的"用户档案 / 公司画像 / 推送任务"。
 *
 * **不动 chat-view 核心**(它两端复用)。
 */
export function AdminChatShell(props: { userId?: string }) {
  const navigate = useNavigate()
  const sessions = useSessions()

  // 解析当前会话对应的 actor:优先从 users 列表反查,fallback 到 session_id 编码
  const currentActor = createMemo<ActorRef | undefined>(() => {
    const key = props.userId
    if (!key) return undefined
    const user = sessions.state.users.find((u) => u.session_id === key)
    if (user) return actorFromUser(user)
    return actorFromSessionId(key)
  })

  const goUsers = (tab: "portfolio" | "profiles" | "sessions" | "research") => {
    const a = currentActor()
    if (!a) return
    navigate(`/users/${encodeURIComponent(actorKey(a))}/${tab}`)
  }

  const goNewTask = () => {
    const a = currentActor()
    if (!a) {
      navigate(`/tasks`)
      return
    }
    navigate(`/tasks?actor=${encodeURIComponent(actorKey(a))}`)
  }

  return (
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <Show when={currentActor()}>
        {(actor) => (
          <div class="flex h-9 shrink-0 items-center gap-2 border-b border-[color:var(--border)] bg-[color:var(--panel)] px-3 text-xs">
            <span class="text-[color:var(--text-muted)]">{SESSIONS.shell.current_user}</span>
            <span class="font-mono font-medium text-[color:var(--text-primary)]">
              {actor().user_id}
            </span>
            <span class="text-[10px] text-[color:var(--text-muted)]">
              {actor().channel}
              <Show when={actor().channel_scope}>
                <> · {actor().channel_scope}</>
              </Show>
            </span>
            <div class="ml-auto flex items-center gap-1">
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)]"
                onClick={() => goUsers("portfolio")}
                title={SESSIONS.shell.portfolio_title}
              >
                {SESSIONS.shell.portfolio_button}
              </button>
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)]"
                onClick={() => goUsers("profiles")}
                title={SESSIONS.shell.profiles_title}
              >
                {SESSIONS.shell.profiles_button}
              </button>
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)]"
                onClick={goNewTask}
                title={SESSIONS.shell.new_task_title}
              >
                {SESSIONS.shell.new_task_button}
              </button>
            </div>
          </div>
        )}
      </Show>
      <div class="min-h-0 flex-1 overflow-hidden">
        <ChatView userId={props.userId} />
      </div>
    </div>
  )
}
