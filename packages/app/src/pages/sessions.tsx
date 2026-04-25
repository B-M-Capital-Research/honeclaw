import { useParams } from "@solidjs/router"
import { AdminChatShell } from "@/components/admin-chat-shell"

export default function SessionsPage() {
  const params = useParams()
  const userId = () => (params.userId ? decodeURIComponent(params.userId) : undefined)
  return <AdminChatShell userId={userId()} />
}
