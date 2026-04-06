import { useLocation, useParams } from "@solidjs/router"
import { createEffect, type ParentProps } from "solid-js"
import { SidebarNav } from "@/components/sidebar-nav"
import { ChannelStatusBadge } from "@/components/channel-status-badge"
import { SessionList } from "@/components/session-list"
import { SkillList } from "@/components/skill-list"
import { useConsole } from "@/context/console"
import { useSessions } from "@/context/sessions"
import { useSkills } from "@/context/skills"
import { useTasks } from "@/context/tasks"
import { usePortfolio } from "@/context/portfolio"
import { useResearch } from "@/context/research"
import { useKb } from "@/context/kb"
import { useBackend } from "@/context/backend"
import { TaskList } from "@/components/task-list"
import { PortfolioList } from "@/components/portfolio-list"
import { ResearchList } from "@/components/research-list"
import { KbList } from "@/components/kb-list"
import { parseActorKey } from "@/lib/actors"

export default function ConsoleLayout(props: ParentProps) {
  const location = useLocation()
  const params = useParams()
  const consoleState = useConsole()
  const backend = useBackend()
  const sessions = useSessions()
  const skills = useSkills()

  const tasks = useTasks()
  const portfolio = usePortfolio()
  const research = useResearch()
  useKb() // 确保 KbProvider 上下文可访问（供子路由使用）

  createEffect(() => {
    const p = location.pathname
    if (p.startsWith("/start")) {
      consoleState.setModule("start")
    } else if (p.startsWith("/skills")) {
      consoleState.setModule("skills")
    } else if (p.startsWith("/tasks")) {
      consoleState.setModule("tasks")
    } else if (p.startsWith("/memory")) {
      consoleState.setModule("memory")
    } else if (p.startsWith("/portfolio")) {
      consoleState.setModule("portfolio")
    } else if (p.startsWith("/research")) {
      consoleState.setModule("research")
    } else if (p.startsWith("/llm-audit")) {
      consoleState.setModule("llm-audit")
    } else if (p.startsWith("/logs")) {
      consoleState.setModule("logs")
    } else if (p.startsWith("/kb")) {
      consoleState.setModule("kb")
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
    const actor = parseActorKey(params.userId ? decodeURIComponent(params.userId) : undefined)
    if (location.pathname.startsWith("/portfolio")) {
      portfolio.selectActor(actor)
    }
  })

  createEffect(() => {
    const taskId = params.taskId ? decodeURIComponent(params.taskId) : undefined
    if (location.pathname.startsWith("/research")) {
      research.selectTask(taskId ?? null)
    }
  })

  return (
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
          {consoleState.state.module === "portfolio" ? <PortfolioList /> : null}
          {consoleState.state.module === "research" ? <ResearchList /> : null}
          {consoleState.state.module === "kb" ? <KbList /> : null}
          {/* logs / kb-detail 模块不需要侧边列表，main 区域全宽展示 */}
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
    </div>
  )
}
