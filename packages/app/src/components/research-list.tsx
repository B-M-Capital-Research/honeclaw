import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { For, Show, createEffect, createSignal } from "solid-js"
import { Portal } from "solid-js/web"
import { useNavigate, useSearchParams } from "@solidjs/router"
import { useResearch } from "@/context/research"
import type { ResearchTask } from "@/lib/types"
import { RESEARCH } from "@/lib/admin-content/research"
import { useLocale } from "@/lib/i18n"
import {
  confirmableResearchName,
  formatResearchTaskTime,
  researchStatusBadgeConfig,
  researchSymbolFromSearchParam,
} from "./research-list-model"

// ── 状态徽章 ──────────────────────────────────────────────────────────────────

function StatusBadge(props: { task: ResearchTask }) {
  const statusConfig = () => researchStatusBadgeConfig(props.task)

  return (
    <div class="flex items-center gap-1.5">
      <span class={["h-1.5 w-1.5 shrink-0 rounded-full", statusConfig().dot].join(" ")} />
      <span class={["text-[11px] font-medium", statusConfig().text].join(" ")}>{statusConfig().label}</span>
    </div>
  )
}

// ── 主组件 ───────────────────────────────────────────────────────────────────

export function ResearchList() {
  const navigate = useNavigate()
  const research = useResearch()
  const [searchParams, setSearchParams] = useSearchParams()
  const [companyInput, setCompanyInput] = createSignal("")
  const [starting, setStarting] = createSignal(false)
  const [confirmName, setConfirmName] = createSignal<string | null>(null)

  // 接受 ?symbol=AAPL 自动预填(供 SymbolDrawer "启动研究" 跳转使用)
  createEffect(() => {
    const sym = researchSymbolFromSearchParam(searchParams.symbol)
    if (sym && !companyInput()) {
      setCompanyInput(sym)
      // 用过即清,避免反复回填
      setSearchParams({ symbol: undefined }, { replace: true })
    }
  })

  const handleConfirmOpen = () => {
    const name = confirmableResearchName(companyInput(), starting())
    if (!name) return
    setConfirmName(name)
  }

  const handleConfirm = async () => {
    const name = confirmName()
    setConfirmName(null)
    if (!name) return
    setStarting(true)
    await research.startTask(name)
    setCompanyInput("")
    setStarting(false)
    // 导航到新建任务
    const first = research.state.tasks[0]
    if (first) {
      navigate(`/research/${encodeURIComponent(first.id)}`)
    }
  }

  const handleCancel = () => {
    setConfirmName(null)
  }

  const openTask = (task: ResearchTask) => {
    research.selectTask(task.id)
    navigate(`/research/${encodeURIComponent(task.id)}`)
  }

  return (
    <div class="flex h-full min-h-0 w-[260px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      {/* 头部：标题 + 输入框 */}
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div class="text-sm font-semibold tracking-tight">{RESEARCH.list.title}</div>
        <div class="text-xs text-[color:var(--text-muted)] mt-0.5">{RESEARCH.list.subtitle}</div>
        <div class="mt-3 flex gap-2">
          <Input
            class="h-8 text-xs flex-1"
            value={companyInput()}
            onInput={(e) => setCompanyInput(e.currentTarget.value)}
            placeholder={RESEARCH.list.input_placeholder}
            disabled={starting()}
          />
          <Button
            class="shrink-0 h-8 px-3 text-xs"
            onClick={handleConfirmOpen}
            disabled={!companyInput().trim() || starting()}
          >
            {starting() ? RESEARCH.list.starting_button : RESEARCH.list.start_button}
          </Button>
        </div>
        <Show when={research.state.submitError}>
          <div class="mt-2 text-[11px] text-rose-500">{research.state.submitError}</div>
        </Show>
      </div>

      {/* 二次确认弹窗 */}
      <Show when={confirmName() !== null}>
        <Portal>
          <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
            onClick={handleCancel}
          >
            <div
              class="w-[320px] rounded-xl border border-[color:var(--border)] bg-[color:var(--surface)] p-5 shadow-xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div class="text-sm font-semibold text-[color:var(--text-primary)] mb-1">{RESEARCH.list.confirm_title}</div>
              <div class="text-xs text-[color:var(--text-muted)] mb-4">
                {RESEARCH.list.confirm_description}
              </div>
              <div class="rounded-lg bg-[color:var(--accent-soft)] border border-[color:var(--accent)] px-4 py-2.5 text-sm font-semibold text-[color:var(--accent)] text-center mb-5">
                {confirmName()}
              </div>
              <div class="flex gap-2 justify-end">
                <Button
                  class="h-8 px-4 text-xs"
                  variant="outline"
                  onClick={handleCancel}
                >
                  {RESEARCH.list.confirm_cancel}
                </Button>
                <Button
                  class="h-8 px-4 text-xs"
                  onClick={() => void handleConfirm()}
                >
                  {RESEARCH.list.confirm_submit}
                </Button>
              </div>
            </div>
          </div>
        </Portal>
      </Show>

      {/* 任务列表 */}
      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show
          when={research.state.tasks.length > 0}
          fallback={
            <EmptyState
              title={RESEARCH.list.empty_title}
              description={RESEARCH.list.empty_description}
            />
          }
        >
          <div class="space-y-2">
            <For each={research.state.tasks}>
              {(task) => {
                const isSelected = () => research.state.selectedTaskId === task.id
                return (
                  <button
                    type="button"
                    onClick={() => openTask(task)}
                    class={[
                      "w-full rounded-md border p-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                      isSelected()
                        ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                        : "border-[color:var(--border)] bg-[color:var(--panel)] hover:bg-black/5",
                    ].join(" ")}
                  >
                    <div class="flex items-start justify-between gap-2">
                      <div class="min-w-0 flex-1">
                        <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                          {task.company_name}
                        </div>
                        <div class="mt-1">
                          <StatusBadge task={task} />
                        </div>
                      </div>
                      <div class="shrink-0 text-[10px] text-[color:var(--text-muted)] mt-0.5">
                        {formatResearchTaskTime(task.created_at, useLocale())}
                      </div>
                    </div>
                  </button>
                )
              }}
            </For>
          </div>
        </Show>
      </div>
    </div>
  )
}
