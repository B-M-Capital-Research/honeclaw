import "@/index.css"
import { MarkedProvider } from "@hone-financial/ui/context/marked"
import { ToastProvider } from "@hone-financial/ui/context/toast"
import { MetaProvider, Title } from "@solidjs/meta"
import { Navigate, Route, Router } from "@solidjs/router"
import { ErrorBoundary, Suspense, createEffect, createSignal, lazy, type ParentProps } from "solid-js"
import { BackendProvider } from "@/context/backend"
import { ConsoleProvider } from "@/context/console"
import { SessionsProvider } from "@/context/sessions"
import { SkillsProvider } from "@/context/skills"
import { TasksProvider } from "@/context/tasks"
import { PortfolioProvider } from "@/context/portfolio"
import { ResearchProvider } from "@/context/research"
import { CompanyProfilesProvider } from "@/context/company-profiles"
import { isRecoverableAssetLoadError, recoverFromAssetLoadError } from "@/lib/asset-recovery"
import ConsoleLayout from "@/pages/layout"

const HomePage = lazy(() => import("@/pages/home"))
const PublicChatPage = lazy(() => import("@/pages/chat"))
const PublicSiteHomePage = lazy(() => import("@/pages/public-home"))
const PublicSiteRoadmapPage = lazy(() => import("@/pages/public-roadmap"))
const PublicBlogPage = lazy(() => import("@/pages/public-blog"))
const PublicBlogPostPage = lazy(() => import("@/pages/public-blog-post"))
const PublicSiteMePage = lazy(() => import("@/pages/public-me"))
const PublicSiteTermsPage = lazy(() => import("@/pages/public-terms"))
const PublicSitePrivacyPage = lazy(() => import("@/pages/public-privacy"))
const PublicSitePortfolioPage = lazy(() => import("@/pages/public-portfolio"))
const SharePreviewPage = lazy(() => import("@/pages/__share-preview"))
const DashboardPage = lazy(() => import("@/pages/dashboard"))
const SessionsPage = lazy(() => import("@/pages/sessions"))
const SkillsPage = lazy(() => import("@/pages/skills"))
const TasksPage = lazy(() => import("@/pages/tasks"))
const UsersPage = lazy(() => import("@/pages/users"))
const ResearchPage = lazy(() => import("@/pages/research"))
const LlmAuditPage = lazy(() => import("@/pages/llm-audit"))
const LogsPage = lazy(() => import("@/pages/logs"))
const TaskHealthPage = lazy(() => import("@/pages/task-health"))
const NotificationsPage = lazy(() => import("@/pages/notifications"))
const SchedulePage = lazy(() => import("@/pages/schedule"))
const SettingsPage = lazy(() => import("@/pages/settings"))
const APP_SURFACE = import.meta.env.VITE_HONE_APP_SURFACE === "public" ? "public" : "admin"

function Loading() {
  return <div class="flex min-h-screen items-center justify-center text-sm text-[color:var(--text-secondary)]">Loading…</div>
}

function AppErrorFallback(props: { error: unknown }) {
  const [recovering, setRecovering] = createSignal(false)
  createEffect(() => {
    setRecovering(recoverFromAssetLoadError(props.error))
  })
  return (
    <div class="flex min-h-screen items-center justify-center p-8 text-center text-sm text-[color:var(--text-secondary)]">
      {recovering()
        ? "正在加载最新版本…"
        : isRecoverableAssetLoadError(props.error)
          ? "页面资源已更新，请刷新页面。"
          : String(props.error)}
    </div>
  )
}

function AdminProviders(props: ParentProps) {
  return (
    <MetaProvider>
      <Title>Hone Console</Title>
      <MarkedProvider>
        <ToastProvider>
          <BackendProvider>
            <ConsoleProvider>
              <SessionsProvider>
                <SkillsProvider>
                  <TasksProvider>
                    <PortfolioProvider>
                      <ResearchProvider>
                        <CompanyProfilesProvider>{props.children}</CompanyProfilesProvider>
                      </ResearchProvider>
                    </PortfolioProvider>
                  </TasksProvider>
                </SkillsProvider>
              </SessionsProvider>
            </ConsoleProvider>
          </BackendProvider>
        </ToastProvider>
      </MarkedProvider>
    </MetaProvider>
  )
}

function PublicProviders(props: ParentProps) {
  return (
    <MetaProvider>
      <Title>Hone Chat</Title>
      <MarkedProvider>
        <ToastProvider>{props.children}</ToastProvider>
      </MarkedProvider>
    </MetaProvider>
  )
}

function PublicSurface() {
  return (
    <PublicProviders>
      <ErrorBoundary fallback={(error) => <AppErrorFallback error={error} />}>
        <Suspense fallback={<Loading />}>
          <Router>
            <Route path="/" component={PublicSiteHomePage} />
            <Route path="/roadmap" component={PublicSiteRoadmapPage} />
            <Route path="/blog" component={PublicBlogPage} />
            <Route path="/blog/:slug" component={PublicBlogPostPage} />
            <Route path="/me" component={PublicSiteMePage} />
            <Route path="/portfolio" component={PublicSitePortfolioPage} />
            <Route path="/terms" component={PublicSiteTermsPage} />
            <Route path="/privacy" component={PublicSitePrivacyPage} />
            <Route path="/chat" component={PublicChatPage} />
            <Route path="/__share-preview" component={SharePreviewPage} />
            <Route path="*" component={() => <Navigate href="/" />} />
          </Router>
        </Suspense>
      </ErrorBoundary>
    </PublicProviders>
  )
}

function AdminSurface() {
  return (
    <AdminProviders>
      <ErrorBoundary fallback={(error) => <AppErrorFallback error={error} />}>
        <Suspense fallback={<Loading />}>
          <Router>
            <Route path="/" component={HomePage} />
            <Route path="/" component={ConsoleLayout}>
              <Route path="/dashboard" component={DashboardPage} />
              <Route path="/start" component={() => <Navigate href="/dashboard" />} />
              <Route path="/sessions/:userId?" component={SessionsPage} />
              <Route path="/skills/:skillId?" component={SkillsPage} />
              <Route path="/tasks/:taskId?" component={TasksPage} />
              <Route path="/users/:actorKey?/:tab?" component={UsersPage} />
              {/* 旧路径兼容:/memory 已并入用户视图,保留跳转给旧书签。 */}
              <Route path="/memory" component={() => <Navigate href="/users" />} />
              <Route
                path="/portfolio/:userId?"
                component={(props: any) => {
                  // SolidJS Router 给的 params 已经 URL-decoded,Navigate 会再次编码,
                  // 因此这里直接传原始字符串,不要再 encodeURIComponent。
                  const id: string | undefined = props.params?.userId
                  return (
                    <Navigate href={id ? `/users/${id}/portfolio` : "/users"} />
                  )
                }}
              />
              <Route path="/research/:taskId?" component={ResearchPage} />
              <Route path="/llm-audit" component={LlmAuditPage} />
              <Route path="/logs" component={LogsPage} />
              <Route path="/task-health" component={TaskHealthPage} />
              <Route path="/notifications" component={NotificationsPage} />
              <Route path="/schedule" component={SchedulePage} />
              <Route path="/settings" component={SettingsPage} />
            </Route>
          </Router>
        </Suspense>
      </ErrorBoundary>
    </AdminProviders>
  )
}

export function App() {
  return APP_SURFACE === "public" ? <PublicSurface /> : <AdminSurface />
}
