import { useLocation, useNavigate, useParams } from "@solidjs/router"
import { createEffect, type ParentProps } from "solid-js"
import { SidebarNav } from "@/components/sidebar-nav"
import { ChannelStatusBadge } from "@/components/channel-status-badge"
import { SessionList } from "@/components/session-list"
import { SkillList } from "@/components/skill-list"
import { ActorList } from "@/components/actor-list"
import { SymbolDrawer } from "@/components/symbol-drawer"
import { SymbolDrawerProvider } from "@/context/symbol-drawer"
import { useConsole } from "@/context/console"
import { useSessions } from "@/context/sessions"
import { useSkills } from "@/context/skills"
import { useTasks } from "@/context/tasks"
import { useResearch } from "@/context/research"
import { useBackend } from "@/context/backend"
import { TaskList } from "@/components/task-list"
import { ResearchList } from "@/components/research-list"
import { actorKey } from "@/lib/actors"

export default function ConsoleLayout(props: ParentProps) {
  const location = useLocation()
  const navigate = useNavigate()
  const params = useParams()
  const consoleState = useConsole()
  const backend = useBackend()
  const sessions = useSessions()
  const skills = useSkills()

  const tasks = useTasks()
  const research = useResearch()

  createEffect(() => {
    const p = location.pathname
    if (p.startsWith("/dashboard") || p.startsWith("/start")) {
      consoleState.setModule("dashboard")
    } else if (p.startsWith("/skills")) {
      consoleState.setModule("skills")
    } else if (p.startsWith("/tasks")) {
      consoleState.setModule("tasks")
    } else if (p.startsWith("/users")) {
      consoleState.setModule("users")
    } else if (p.startsWith("/research")) {
      consoleState.setModule("research")
    } else if (p.startsWith("/llm-audit")) {
      consoleState.setModule("llm-audit")
    } else if (p.startsWith("/logs")) {
      consoleState.setModule("logs")
    } else if (p.startsWith("/settings")) {
      consoleState.setModule("settings")
    } else {
      consoleState.setModule("sessions")
    }
  })

  createEffect(() => {
    const userId = params.userId ? decodeURIComponent(params.userId) : undefined
    if (location.pathname.startsWith("/sessions")) {
      void sessions.selectUser(userId)
    }
  })

  createEffect(() => {
    const skillId = params.skillId ? decodeURIComponent(params.skillId) : undefined
    if (location.pathname.startsWith("/skills")) {
      skills.selectSkill(skillId)
    }
  })

  createEffect(() => {
    const taskId = params.taskId ? decodeURIComponent(params.taskId) : undefined
    if (location.pathname.startsWith("/tasks")) {
      tasks.selectTask(taskId)
    }
  })

  createEffect(() => {
    const taskId = params.taskId ? decodeURIComponent(params.taskId) : undefined
    if (location.pathname.startsWith("/research")) {
      research.selectTask(taskId ?? null)
    }
  })

  const usersCurrentKey = () =>
    params.actorKey ? decodeURIComponent(params.actorKey) : ""

  const onSelectUserActor = (actor: { channel: string; user_id: string; channel_scope?: string }) => {
    const k = encodeURIComponent(actorKey(actor))
    // 保留当前 tab,默认 portfolio
    const tab = (params.tab as string) || "portfolio"
    navigate(`/users/${k}/${tab}`)
  }

  return (
    <SymbolDrawerProvider>
    <div class="flex h-screen min-h-0 overflow-hidden">
      <SidebarNav />

      {/* 侧边栏右侧：header + 内容区纵向排列 */}
      <div class="flex min-h-0 min-w-0 flex-1 flex-col">
        {/* 全局顶部 header 行：仅放渠道状态 Badge，靠右对齐 */}
        <div class="flex h-10 shrink-0 items-center justify-end border-b border-[color:var(--border)] bg-[color:var(--panel)] px-4">
          <ChannelStatusBadge />
        </div>

        {/* 内容行：中间列表面板 + 主区域 */}
        <div class="flex min-h-0 flex-1 overflow-hidden">
          {consoleState.state.module === "sessions" ? <SessionList /> : null}
          {consoleState.state.module === "skills" ? <SkillList /> : null}
          {consoleState.state.module === "tasks" ? <TaskList /> : null}
          {consoleState.state.module === "users" ? (
            <ActorList currentKey={usersCurrentKey()} onSelect={onSelectUserActor} />
          ) : null}
          {consoleState.state.module === "research" ? <ResearchList /> : null}
          {/* logs 模块不需要侧边列表，main 区域全宽展示 */}
          <main class="min-h-0 min-w-0 flex-1 overflow-hidden p-3 md:p-4">
            {!backend.state.connected && !backend.state.initializing ? (
              <div class="mb-3 rounded-lg border border-rose-300/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-300">
                后端未连接：{backend.state.error || "请在设置页检查连接。"}
              </div>
            ) : null}
            {props.children}
          </main>
        </div>
      </div>
      <SymbolDrawer />
    </div>
    </SymbolDrawerProvider>
  )
}
