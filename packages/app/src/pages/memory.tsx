import { createEffect, createSignal } from "solid-js"
import { Show } from "solid-js"
import { useSearchParams } from "@solidjs/router"
import { PortfolioList } from "@/components/portfolio-list"
import { PortfolioDetail } from "@/components/portfolio-detail"
import { CompanyProfileList } from "@/components/company-profile-list"
import { CompanyProfileDetail } from "@/components/company-profile-detail"
import { useCompanyProfiles } from "@/context/company-profiles"
import { useBackend } from "@/context/backend"
import { parseActorKey } from "@/lib/actors"

type MemoryTab = "portfolio" | "profiles"

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
  const backend = useBackend()
  const companyProfiles = useCompanyProfiles()
  const [tab, setTab] = createSignal<MemoryTab>("portfolio")
  const [searchParams] = useSearchParams()

  createEffect(() => {
    const requested = typeof searchParams.tab === "string" ? searchParams.tab : undefined
    if (requested === "profiles") {
      setTab("profiles")
    } else if (requested === "portfolio") {
      setTab("portfolio")
    }

    const profileId =
      typeof searchParams.profile === "string" ? searchParams.profile : undefined
    const profileActorKey =
      typeof searchParams.profile_actor === "string" ? searchParams.profile_actor : undefined
    if (profileActorKey) {
      companyProfiles.selectActor(parseActorKey(profileActorKey))
    }
    if (profileId) {
      companyProfiles.selectProfile(profileId)
    }
  })

  return (
    <div class="flex h-full flex-col overflow-hidden">
      {/* Tab 栏 */}
      <div class="flex shrink-0 border-b border-[color:var(--border)] px-2">
        <TabBtn
          label="持仓记忆"
          active={tab() === "portfolio"}
          onClick={() => setTab("portfolio")}
        />
        <Show when={backend.hasCapability("company_profiles")}>
          <TabBtn
            label="公司画像"
            active={tab() === "profiles"}
            onClick={() => setTab("profiles")}
          />
        </Show>
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

      <Show when={tab() === "profiles"}>
        <div class="flex min-h-0 flex-1 overflow-hidden">
          <CompanyProfileList />
          <div class="min-h-0 flex-1 overflow-hidden">
            <CompanyProfileDetail />
          </div>
        </div>
      </Show>
    </div>
  )
}
