import { Navigate } from "@solidjs/router"
import { useConsole } from "@/context/console"

export default function HomePage() {
  const consoleState = useConsole()
  if (consoleState.state.module === "skills") {
    const target = consoleState.state.lastSkillId
    return <Navigate href={target ? `/skills/${encodeURIComponent(target)}` : "/skills"} />
  }
  if (consoleState.state.module === "settings") {
    return <Navigate href="/settings" />
  }
  // sessions 模块保留原有行为
  if (consoleState.state.module === "sessions") {
    const target = consoleState.state.lastUserId
    return <Navigate href={target ? `/sessions/${encodeURIComponent(target)}` : "/dashboard"} />
  }
  // 默认进入 dashboard 概览页
  return <Navigate href="/dashboard" />
}
