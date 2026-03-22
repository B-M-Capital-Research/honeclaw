import "@/index.css"
import { DialogProvider } from "@hone-financial/ui/context/dialog"
import { MarkedProvider } from "@hone-financial/ui/context/marked"
import { ToastProvider } from "@hone-financial/ui/context/toast"
import { ThemeProvider } from "@hone-financial/ui/theme"
import { MetaProvider, Title } from "@solidjs/meta"
import { Route, Router } from "@solidjs/router"
import { ErrorBoundary, Suspense, lazy, type ParentProps } from "solid-js"
import { BackendProvider } from "@/context/backend"
import { ConsoleProvider } from "@/context/console"
import { KbProvider } from "@/context/kb"
import { SessionsProvider } from "@/context/sessions"
import { SkillsProvider } from "@/context/skills"
import { TasksProvider } from "@/context/tasks"
import { PortfolioProvider } from "@/context/portfolio"
import { ResearchProvider } from "@/context/research"
import ConsoleLayout from "@/pages/layout"

const HomePage = lazy(() => import("@/pages/home"))
const StartPage = lazy(() => import("@/pages/start"))
const SessionsPage = lazy(() => import("@/pages/sessions"))
const SkillsPage = lazy(() => import("@/pages/skills"))
const TasksPage = lazy(() => import("@/pages/tasks"))
const PortfolioPage = lazy(() => import("@/pages/portfolio"))
const MemoryPage = lazy(() => import("@/pages/memory"))
const ResearchPage = lazy(() => import("@/pages/research"))
const KbPage = lazy(() => import("@/pages/kb"))
const LlmAuditPage = lazy(() => import("@/pages/llm-audit"))
const LogsPage = lazy(() => import("@/pages/logs"))
const SettingsPage = lazy(() => import("@/pages/settings"))

function Loading() {
  return <div class="flex min-h-screen items-center justify-center text-sm text-[color:var(--text-secondary)]">Loading…</div>
}

function Providers(props: ParentProps) {
  return (
    <MetaProvider>
      <Title>Hone Console</Title>
      <ThemeProvider>
        <DialogProvider>
          <MarkedProvider>
            <ToastProvider>
              <BackendProvider>
                <ConsoleProvider>
                  <SessionsProvider>
                    <SkillsProvider>
                      <TasksProvider>
                        <PortfolioProvider>
                          <ResearchProvider>
                            <KbProvider>{props.children}</KbProvider>
                          </ResearchProvider>
                        </PortfolioProvider>
                      </TasksProvider>
                    </SkillsProvider>
                  </SessionsProvider>
                </ConsoleProvider>
              </BackendProvider>
            </ToastProvider>
          </MarkedProvider>
        </DialogProvider>
      </ThemeProvider>
    </MetaProvider>
  )
}

export function App() {
  return (
    <Providers>
      <ErrorBoundary fallback={(error) => <div class="p-8 text-red-300">{String(error)}</div>}>
        <Suspense fallback={<Loading />}>
          <Router>
            <Route path="/" component={HomePage} />
            <Route path="/" component={ConsoleLayout}>
              <Route path="/start" component={StartPage} />
              <Route path="/sessions/:userId?" component={SessionsPage} />
              <Route path="/skills/:skillId?" component={SkillsPage} />
              <Route path="/tasks/:taskId?" component={TasksPage} />
              <Route path="/memory" component={MemoryPage} />
              <Route path="/portfolio/:userId?" component={PortfolioPage} />
              <Route path="/research/:taskId?" component={ResearchPage} />
              <Route path="/kb/:entryId?" component={KbPage} />
              <Route path="/llm-audit" component={LlmAuditPage} />
              <Route path="/logs" component={LogsPage} />
              <Route path="/settings" component={SettingsPage} />
            </Route>
          </Router>
        </Suspense>
      </ErrorBoundary>
    </Providers>
  )
}
