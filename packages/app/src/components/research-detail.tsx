import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Show, lazy, Suspense } from "solid-js"
import { useResearch } from "@/context/research"

// 懒加载 Markdown 预览组件（含 marked/hljs/mermaid/html2canvas/jspdf，体积较大）
const ResearchPreview = lazy(() => import("@/components/research-preview"))

// ── 进度条 ───────────────────────────────────────────────────────────────────

function ProgressBar(props: { progress: string }) {
  const pct = () => {
    const n = parseInt(props.progress ?? "0", 10)
    return isNaN(n) ? 0 : Math.min(100, Math.max(0, n))
  }
  return (
    <div class="w-full rounded-full bg-black/10 h-2 overflow-hidden">
      <div
        class="h-full rounded-full bg-[color:var(--accent)] transition-all duration-500"
        style={{ width: `${pct()}%` }}
      />
    </div>
  )
}

// ── 步骤指示器（2步）──────────────────────────────────────────────────────────

function StepIndicator(props: { step: 1 | 2; current: number; label: string; sublabel?: string }) {
  const done = () => props.current > props.step
  const active = () => props.current === props.step

  return (
    <div class="flex items-start gap-3">
      <div
        class={[
          "mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-xs font-bold",
          done()
            ? "bg-[color:var(--success)] text-white"
            : active()
              ? "bg-[color:var(--accent)] text-white"
              : "bg-black/10 text-[color:var(--text-muted)]",
        ].join(" ")}
      >
        {done() ? "✓" : props.step}
      </div>
      <div>
        <div
          class={[
            "text-sm font-medium",
            active()
              ? "text-[color:var(--text-primary)]"
              : done()
                ? "text-[color:var(--text-secondary)]"
                : "text-[color:var(--text-muted)]",
          ].join(" ")}
        >
          {props.label}
        </div>
        <Show when={props.sublabel}>
          <div class="text-xs text-[color:var(--text-muted)] mt-0.5">{props.sublabel}</div>
        </Show>
      </div>
    </div>
  )
}

// ── 主组件 ───────────────────────────────────────────────────────────────────

export function ResearchDetail() {
  const research = useResearch()
  const task = () => research.selectedTask()

  // 1 = 研究中，2 = 完成（有 answer_markdown）
  const currentStep = () => {
    const t = task()
    if (!t) return 0
    if (t.answer_markdown) return 2
    return 1
  }

  const formatTime = (iso?: string | null) => {
    if (!iso) return "—"
    try {
      return new Date(iso).toLocaleString("zh-CN")
    } catch {
      return iso
    }
  }

  return (
    <Show
      when={task()}
      fallback={
        <EmptyState
          title="从左侧选择或发起深度研究"
          description="输入公司名称，AI 将自动完成深度研究，完成后直接在此展示报告。"
        />
      }
    >
      {(t) => (
        <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm overflow-hidden">
          {/* 顶部标题栏 */}
          <div class="flex items-center justify-between border-b border-[color:var(--border)] px-6 py-4 shrink-0">
            <div>
              <div class="text-xl font-semibold">{t().company_name} 深度研究</div>
              <div class="mt-1 text-xs text-[color:var(--text-muted)] font-mono">
                task_id: {t().task_id}
              </div>
            </div>
            <Show when={t().status === "running" || t().status === "pending"}>
              <Button
                class="h-8 px-3 text-xs"
                onClick={() => research.refreshTask(t().task_id)}
              >
                刷新进度
              </Button>
            </Show>
          </div>

          {/* 内容区：有 Markdown 时直接渲染，否则显示进度 */}
          <Show
            when={t().answer_markdown}
            fallback={
              /* 进度状态面板 */
              <div class="flex-1 overflow-y-auto hf-scrollbar p-6 flex flex-col gap-6">
                {/* 步骤指示器 */}
                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                  <div class="text-sm font-semibold mb-4">任务进度</div>
                  <div class="space-y-4">
                    <StepIndicator
                      step={1}
                      current={currentStep()}
                      label="AI 深度研究"
                      sublabel={
                        t().status === "running" || t().status === "pending"
                          ? `进行中：${t().progress || "0%"}，预计需要约 2 小时`
                          : t().status === "completed"
                            ? `已完成 · ${formatTime(t().completed_at)}`
                            : undefined
                      }
                    />
                    <StepIndicator
                      step={2}
                      current={currentStep()}
                      label="报告渲染完成，可在线查看"
                    />
                  </div>
                </div>

                {/* 进度条（研究中时显示） */}
                <Show when={t().status === "running" || t().status === "pending"}>
                  <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                    <div class="flex items-center justify-between mb-3">
                      <div class="text-sm font-medium">研究进度</div>
                      <div class="text-sm font-bold text-[color:var(--accent)]">{t().progress || "0%"}</div>
                    </div>
                    <ProgressBar progress={t().progress || "0%"} />
                    <div class="mt-3 text-xs text-[color:var(--text-muted)]">
                      每 10 秒自动刷新进度 · 启动时间：{formatTime(t().created_at)}
                    </div>
                  </div>
                </Show>

                {/* 错误状态 */}
                <Show when={t().status === "error"}>
                  <div class="rounded-lg border border-rose-200 bg-rose-50 p-5">
                    <div class="text-sm font-semibold text-rose-600 mb-1">研究任务异常</div>
                    <div class="text-xs text-rose-500">
                      {t().error_message || "请联系管理员排查原因"}
                    </div>
                  </div>
                </Show>

                {/* 任务信息 */}
                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                  <div class="text-sm font-semibold mb-3">任务信息</div>
                  <dl class="space-y-2 text-xs">
                    <div class="flex justify-between">
                      <dt class="text-[color:var(--text-muted)]">公司名称</dt>
                      <dd class="font-medium">{t().company_name}</dd>
                    </div>
                    <div class="flex justify-between">
                      <dt class="text-[color:var(--text-muted)]">任务名称</dt>
                      <dd class="font-mono text-[10px] truncate max-w-[220px]">{t().task_name}</dd>
                    </div>
                    <div class="flex justify-between">
                      <dt class="text-[color:var(--text-muted)]">启动时间</dt>
                      <dd>{formatTime(t().created_at)}</dd>
                    </div>
                    <div class="flex justify-between">
                      <dt class="text-[color:var(--text-muted)]">最近更新</dt>
                      <dd>{formatTime(t().updated_at)}</dd>
                    </div>
                    <Show when={t().completed_at}>
                      <div class="flex justify-between">
                        <dt class="text-[color:var(--text-muted)]">完成时间</dt>
                        <dd>{formatTime(t().completed_at)}</dd>
                      </div>
                    </Show>
                  </dl>
                </div>
              </div>
            }
          >
            {(markdown) => (
              /* Markdown 报告渲染区 */
              <div class="flex-1 min-h-0 overflow-hidden">
                <Suspense
                  fallback={
                    <div class="flex flex-1 h-full items-center justify-center text-sm text-[color:var(--text-muted)]">
                      加载渲染器…
                    </div>
                  }
                >
                  <ResearchPreview
                    markdown={markdown()}
                    companyName={t().company_name}
                  />
                </Suspense>
              </div>
            )}
          </Show>
        </div>
      )}
    </Show>
  )
}
