import { createSignal } from "solid-js"
import { Show } from "solid-js"
import { PortfolioList } from "@/components/portfolio-list"
import { PortfolioDetail } from "@/components/portfolio-detail"
import { KbStockTable } from "@/components/kb-stock-table"

type MemoryTab = "portfolio" | "knowledge"

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

export default function MemoryPage() {
  const [tab, setTab] = createSignal<MemoryTab>("portfolio")

  return (
    <div class="flex h-full flex-col overflow-hidden">
      {/* Tab 栏 */}
      <div class="flex shrink-0 border-b border-[color:var(--border)] px-2">
        <TabBtn
          label="持仓记忆"
          active={tab() === "portfolio"}
          onClick={() => setTab("portfolio")}
        />
        <TabBtn
          label="知识记忆"
          active={tab() === "knowledge"}
          onClick={() => setTab("knowledge")}
        />
      </div>

      {/* 持仓记忆 — 两栏布局：左侧选人 + 右侧详情 */}
      <Show when={tab() === "portfolio"}>
        <div class="flex min-h-0 flex-1 overflow-hidden">
          <PortfolioList />
          <div class="min-h-0 flex-1 overflow-hidden">
            <PortfolioDetail />
          </div>
        </div>
      </Show>

      {/* 知识记忆 — 股票信息表（含跳转至知识库的链接） */}
      <Show when={tab() === "knowledge"}>
        <div class="min-h-0 flex-1 overflow-y-auto p-5">
          <KbStockTable />
        </div>
      </Show>
    </div>
  )
}
