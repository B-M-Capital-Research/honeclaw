import { useNavigate } from "@solidjs/router"
import { Show } from "solid-js"
import { actorKey, type ActorRef } from "@/lib/actors"
import { useSymbolDrawer } from "@/context/symbol-drawer"
import { SHARED } from "@/lib/admin-content/shared"
import { tpl } from "@/lib/i18n"

/**
 * 跨模块的实体引用链接 — 在日志/审计/任务等纯文本视图里把实体 ID 渲染成可点击 chip,
 * 提供"反向查"动线(从一条错误 → 跳到引发它的用户 / 会话 / 任务)。
 *
 * 支持 kind: actor / session / task / research / skill,以及通过 SymbolDrawer
 * 打开的 symbol 引用。
 */
type EntityKind =
  | "actor"
  | "session"
  | "task"
  | "symbol"
  | "research"
  | "skill"

export type EntityRefLinkProps = {
  kind: EntityKind
  /** 主标识。actor 用 user_id;其它用对应 ID。 */
  id: string
  /** actor 专用 */
  channel?: string
  /** actor 专用 */
  scope?: string
  /** 自定义显示文本(默认按 kind 自动生成) */
  label?: string
  /** 紧凑模式(去掉 kind 前缀,只显示 id/label) */
  compact?: boolean
}

function kindLabel(kind: EntityKind): string {
  switch (kind) {
    case "actor":
      return SHARED.entity_ref.actor
    case "session":
      return SHARED.entity_ref.session
    case "task":
      return SHARED.entity_ref.task
    case "symbol":
      return SHARED.entity_ref.symbol
    case "research":
      return SHARED.entity_ref.research
    case "skill":
      return SHARED.entity_ref.skill
  }
}

function targetHref(props: EntityRefLinkProps): string | undefined {
  switch (props.kind) {
    case "actor": {
      if (!props.channel) return undefined
      const actor: ActorRef = {
        channel: props.channel,
        user_id: props.id,
        channel_scope: props.scope || undefined,
      }
      return `/users/${encodeURIComponent(actorKey(actor))}/portfolio`
    }
    case "session":
      return `/sessions/${encodeURIComponent(props.id)}`
    case "task":
      return `/tasks/${encodeURIComponent(props.id)}`
    case "research":
      return `/research/${encodeURIComponent(props.id)}`
    case "skill":
      return `/skills/${encodeURIComponent(props.id)}`
    case "symbol":
      // symbol 不走 navigate,而是打开 SymbolDrawer。返回 sentinel 让按钮可点。
      return "__symbol__"
  }
}

function defaultLabel(props: EntityRefLinkProps): string {
  if (props.kind === "actor" && props.channel) {
    const scopeSuffix = props.scope ? ` · ${props.scope}` : ""
    return `${props.id}${scopeSuffix}`
  }
  return props.id
}

export function EntityRefLink(props: EntityRefLinkProps) {
  const navigate = useNavigate()
  const drawer = useSymbolDrawer()
  const href = () => targetHref(props)
  const text = () => props.label ?? defaultLabel(props)
  const kindText = () => kindLabel(props.kind)
  const titleText = () =>
    tpl(SHARED.entity_ref.goto_title, { kind: kindText(), label: text() })

  const onClick = (e: MouseEvent) => {
    const dest = href()
    if (!dest) return
    e.preventDefault()
    e.stopPropagation()
    if (props.kind === "symbol") {
      drawer.openSymbol(props.id)
      return
    }
    navigate(dest)
  }

  return (
    <Show
      when={href()}
      fallback={
        <span
          class="inline-flex items-center gap-1 rounded-md border border-[color:var(--border)] bg-[color:var(--panel-strong)] px-1.5 py-0.5 text-[11px] text-[color:var(--text-muted)]"
          title={`${kindText()}:${text()}`}
        >
          <Show when={!props.compact}>
            <span class="opacity-60">{kindText()}</span>
          </Show>
          <span class="font-mono">{text()}</span>
        </span>
      }
    >
      <button
        type="button"
        onClick={onClick}
        class="inline-flex items-center gap-1 rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-1.5 py-0.5 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)] hover:text-[color:var(--text-primary)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]"
        title={titleText()}
      >
        <Show when={!props.compact}>
          <span class="opacity-60">{kindText()}</span>
        </Show>
        <span class="font-mono">{text()}</span>
      </button>
    </Show>
  )
}
