import { useParams } from "@solidjs/router"
import { ChatView } from "@/components/chat-view"

export default function SessionsPage() {
  const params = useParams()
  const userId = () => (params.userId ? decodeURIComponent(params.userId) : undefined)
  return <ChatView userId={userId()} />
}
