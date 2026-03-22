import { createEffect } from "solid-js"
import { useParams } from "@solidjs/router"
import { useKb } from "@/context/kb"
import { KbDetail } from "@/components/kb-detail"

export default function KbPage() {
  const kb = useKb()
  const params = useParams()

  // 路由参数变化时加载详情
  createEffect(() => {
    const id = params.entryId ? decodeURIComponent(params.entryId) : null
    void kb.selectEntry(id)
  })

  return (
    <div class="h-full overflow-hidden">
      <KbDetail />
    </div>
  )
}
