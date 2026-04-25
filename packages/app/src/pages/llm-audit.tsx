import { createResource, createSignal, For, Show } from "solid-js"
import { getAuditRecordDetail, getAuditRecords } from "@/lib/api"
import type { AuditQueryFilter, AuditRecordSummary, LlmAuditRecord } from "@/lib/types"
import { useBackend } from "@/context/backend"
import { EntityRefLink } from "@/components/entity-ref-link"

function shortTime(ts: string): string {
    if (!ts) return ""
    return ts.replace("T", " ").replace("Z", "").split(".")[0]
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null && !Array.isArray(value)
}

function JsonValueView(props: {
    value: unknown
    path: string
    depth: number
    keyName?: string
    expandedState: () => Record<string, boolean>
    setExpandedState: (updater: (prev: Record<string, boolean>) => Record<string, boolean>) => void
}) {
    const isArray = () => Array.isArray(props.value)
    const isObject = () => isPlainObject(props.value)
    const isExpandable = () => isArray() || isObject()
    const isExpanded = () => props.expandedState()[props.path] ?? props.depth < 1

    const toggle = () => {
        if (!isExpandable()) return
        props.setExpandedState((prev) => ({
            ...prev,
            [props.path]: !isExpanded(),
        }))
    }

    const entries = () => {
        if (isArray()) {
            return (props.value as unknown[]).map((item, index) => [String(index), item] as const)
        }
        if (isObject()) {
            return Object.entries(props.value as Record<string, unknown>)
        }
        return []
    }

    const summary = () => {
        if (isArray()) return `[${(props.value as unknown[]).length}]`
        if (isObject()) return `{${Object.keys(props.value as Record<string, unknown>).length}}`
        return ""
    }

    const renderPrimitive = () => {
        if (props.value === null) return <span class="text-rose-600 dark:text-rose-400">null</span>
        if (typeof props.value === "string") return <span class="break-all text-emerald-600 dark:text-emerald-400">"{props.value}"</span>
        if (typeof props.value === "number") return <span class="text-sky-600 dark:text-sky-400">{String(props.value)}</span>
        if (typeof props.value === "boolean") return <span class="text-amber-600 dark:text-amber-400">{String(props.value)}</span>
        if (props.value === undefined) return <span class="text-[color:var(--text-muted)]">undefined</span>
        return <span class="text-[color:var(--text-secondary)]">{String(props.value)}</span>
    }

    return (
        <div class="font-mono text-[11px] leading-6">
            <div class="flex items-start gap-2">
                <Show when={isExpandable()} fallback={<span class="inline-block w-4 shrink-0" />}>
                    <button
                        class="mt-[2px] inline-flex h-4 w-4 shrink-0 items-center justify-center rounded text-[color:var(--text-muted)] transition hover:bg-white/5 hover:text-[color:var(--text-primary)]"
                        onClick={toggle}
                    >
                        <span class={["transition-transform", isExpanded() ? "rotate-90" : ""].join(" ")}>&gt;</span>
                    </button>
                </Show>
                <div class="min-w-0 flex-1">
                    <div class="break-all">
                        <Show when={props.keyName != null}>
                            <span class="text-violet-600 dark:text-violet-400">"{props.keyName}"</span>
                            <span class="text-[color:var(--text-muted)]">: </span>
                        </Show>
                        <Show when={isExpandable()} fallback={renderPrimitive()}>
                            <span class="text-[color:var(--text-secondary)]">
                                {isArray() ? "[" : "{"}
                                <Show when={!isExpanded()}>
                                    <span class="text-[color:var(--text-muted)]"> {summary()} </span>
                                </Show>
                                {isArray() ? "]" : "}"}
                            </span>
                        </Show>
                    </div>
                    <Show when={isExpandable() && isExpanded()}>
                        <div class="mt-1 border-l border-white/8 pl-3">
                            <Show
                                when={entries().length > 0}
                                fallback={<div class="text-[color:var(--text-muted)]">{isArray() ? "[empty]" : "{empty}"}</div>}
                            >
                                <For each={entries()}>
                                    {([childKey, childValue]) => (
                                        <JsonValueView
                                            value={childValue}
                                            keyName={childKey}
                                            path={`${props.path}.${childKey}`}
                                            depth={props.depth + 1}
                                            expandedState={props.expandedState}
                                            setExpandedState={props.setExpandedState}
                                        />
                                    )}
                                </For>
                            </Show>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    )
}

function JsonInspector(props: { title: string; value: unknown }) {
    const [expandedState, setExpandedState] = createSignal<Record<string, boolean>>({})

    return (
        <div>
            <h4 class="mb-2 font-bold text-[color:var(--text-muted)] text-[10px] uppercase tracking-wider">{props.title}</h4>
            <div class="overflow-x-auto rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] p-3">
                <JsonValueView
                    value={props.value}
                    path="$"
                    depth={-1}
                    expandedState={expandedState}
                    setExpandedState={setExpandedState}
                />
            </div>
        </div>
    )
}

export default function LlmAuditPage() {
    const backend = useBackend()
    const [filter, setFilter] = createSignal<AuditQueryFilter>({ page: 1, page_size: 50 })
    const [data, { refetch }] = createResource(
        () => (backend.hasCapability("llm_audit") ? filter() : undefined),
        getAuditRecords,
    )
    const [selectedId, setSelectedId] = createSignal<string | null>(null)
    const [detailData] = createResource(
        () => (backend.hasCapability("llm_audit") ? selectedId() : null),
        getAuditRecordDetail,
    )

    const records = () => data()?.records ?? []
    const total = () => data()?.total ?? 0

    function handleFilterChange(e: Event) {
        const target = e.target as HTMLInputElement | HTMLSelectElement
        const { name, value } = target
        setFilter(prev => ({ ...prev, [name]: value === "" ? undefined : value, page: 1 }))
    }

    return (
        <Show
            when={backend.hasCapability("llm_audit")}
            fallback={
                <div class="flex h-full items-center justify-center rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] text-sm text-[color:var(--text-secondary)]">
                    当前 backend 未开放 LLM 审计能力。
                </div>
            }
        >
        <div class="flex h-full flex-col overflow-hidden">
            {/* 工具栏 */}
            <div class="flex flex-shrink-0 flex-wrap items-center gap-2 border-b border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-2.5">
                <span class="text-sm font-semibold text-[color:var(--text-primary)] mr-1">LLM 审计</span>

                <input
                    name="actor_user_id"
                    type="text"
                    placeholder="过滤用户 ID"
                    class="w-32 rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] placeholder:text-[color:var(--text-muted)] outline-none focus:border-[color:var(--accent)] transition"
                    onChange={handleFilterChange}
                />
                <input
                    name="session_id"
                    type="text"
                    placeholder="过滤 Session"
                    class="w-32 rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] placeholder:text-[color:var(--text-muted)] outline-none focus:border-[color:var(--accent)] transition"
                    onChange={handleFilterChange}
                />
                <select
                    name="success"
                    class="rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)] transition"
                    onChange={handleFilterChange}
                >
                    <option value="">全部状态</option>
                    <option value="true">成功</option>
                    <option value="false">失败</option>
                </select>

                <div class="ml-auto flex items-center gap-2">
                    <button
                        class="flex items-center gap-1.5 rounded border border-transparent bg-[color:var(--surface)] px-2.5 py-1 text-xs font-medium text-[color:var(--text-secondary)] transition hover:text-[color:var(--text-primary)]"
                        onClick={() => refetch()}
                    >
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M21 2v6h-6" /><path d="M3 12a9 9 0 0 1 15-6.7L21 8" /><path d="M3 22v-6h6" /><path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                        </svg>
                        刷新
                    </button>
                    <span class="text-[11px] text-[color:var(--text-muted)]">
                        共 {total()} 条
                    </span>
                </div>
            </div>

            {/* 主体区 */}
            <div class="flex min-h-0 flex-1 overflow-hidden">
                {/* 左侧：列表 + 分页 */}
                <div class="flex flex-1 flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
                    <div class="flex-1 overflow-y-auto">
                        <table class="w-full text-left text-sm">
                            <thead class="sticky top-0 bg-[color:var(--panel)]">
                                <tr class="border-b border-[color:var(--border)] text-[11px] font-semibold text-[color:var(--text-muted)] uppercase tracking-wider">
                                    <th class="px-4 py-2 font-medium">时间</th>
                                    <th class="px-3 py-2 font-medium">Actor / Session</th>
                                    <th class="px-3 py-2 font-medium">服务 / 模型</th>
                                    <th class="px-3 py-2 font-medium">操作</th>
                                    <th class="px-3 py-2 font-medium text-center">状态</th>
                                    <th class="px-3 py-2 font-medium text-right">Tokens</th>
                                    <th class="px-3 py-2 font-medium text-right">耗时</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-white/5 font-mono text-xs">
                                <Show
                                    when={records().length > 0}
                                    fallback={
                                        <tr>
                                            <td colspan="7" class="p-8 text-center text-sm text-[color:var(--text-muted)]">
                                                暂无审计记录
                                            </td>
                                        </tr>
                                    }
                                >
                                    <For each={records()}>
                                        {(row) => (
                                            <tr
                                                class={[
                                                    "group cursor-pointer transition-colors hover:bg-white/[0.02]",
                                                    selectedId() === row.id ? "bg-[color:var(--accent-soft)]" : "",
                                                ].join(" ")}
                                                onClick={() => setSelectedId(row.id)}
                                            >
                                                <td class="whitespace-nowrap px-4 py-2 text-[color:var(--text-secondary)]">{shortTime(row.created_at)}</td>
                                                <td class="px-3 py-2 max-w-[180px]" title={`${row.actor_user_id || "无"} / ${row.session_id}`}>
                                                    <div class="flex flex-col gap-1">
                                                        <Show
                                                            when={row.actor_user_id && row.actor_channel}
                                                            fallback={
                                                                <span class="text-[10px] text-[color:var(--text-muted)]">-</span>
                                                            }
                                                        >
                                                            <EntityRefLink
                                                                kind="actor"
                                                                id={row.actor_user_id!}
                                                                channel={row.actor_channel}
                                                                scope={row.actor_scope}
                                                                compact
                                                            />
                                                        </Show>
                                                        <Show when={row.session_id}>
                                                            <EntityRefLink
                                                                kind="session"
                                                                id={row.session_id}
                                                                label={row.session_id.replace(/^Actor_/, "").slice(0, 14) + "…"}
                                                                compact
                                                            />
                                                        </Show>
                                                    </div>
                                                </td>
                                                <td class="px-3 py-2">
                                                    <div class="text-[color:var(--text-primary)]">{row.provider}</div>
                                                    <div class="text-[10px] text-[color:var(--text-muted)] truncate max-w-[100px]" title={row.model}>{row.model || "-"}</div>
                                                </td>
                                                <td class="px-3 py-2 text-[color:var(--text-secondary)]">{row.operation}</td>
                                                <td class="px-3 py-2 text-center">
                                                    {row.success ? (
                                                        <span class="inline-block rounded px-2 py-0.5 text-[10px] font-semibold text-emerald-400 bg-emerald-400/10">成功</span>
                                                    ) : (
                                                        <span class="inline-block rounded px-2 py-0.5 text-[10px] font-semibold text-rose-400 bg-rose-400/10">失败</span>
                                                    )}
                                                </td>
                                                <td class="px-3 py-2 text-right">
                                                    <Show when={row.prompt_tokens != null || row.completion_tokens != null} fallback={<span class="text-[color:var(--text-muted)]">-</span>}>
                                                        <div class="text-[10px] leading-tight flex flex-col items-end">
                                                            <div class="text-[color:var(--text-secondary)]"><span class="text-[color:var(--text-muted)]">P:</span>{row.prompt_tokens ?? 0}</div>
                                                            <div class="text-[color:var(--text-secondary)]"><span class="text-[color:var(--text-muted)]">C:</span>{row.completion_tokens ?? 0}</div>
                                                        </div>
                                                    </Show>
                                                </td>
                                                <td class="px-3 py-2 text-right text-[color:var(--text-secondary)]">
                                                    {row.latency_ms != null ? `${row.latency_ms}ms` : "-"}
                                                </td>
                                            </tr>
                                        )}
                                    </For>
                                </Show>
                            </tbody>
                        </table>
                    </div>

                    {/* 分页控制 */}
                    <div class="flex items-center justify-between border-t border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-2 flex-shrink-0">
                        <button
                            class="rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={filter().page === 1}
                            onClick={() => setFilter(p => ({ ...p, page: (p.page || 1) - 1 }))}
                        >
                            上一页
                        </button>
                        <span class="text-[11px] text-[color:var(--text-muted)]">
                            第 {filter().page || 1} 页
                        </span>
                        <button
                            class="rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={(filter().page || 1) * (filter().page_size || 50) >= total()}
                            onClick={() => setFilter(p => ({ ...p, page: (p.page || 1) + 1 }))}
                        >
                            下一页
                        </button>
                    </div>
                </div>

                {/* 详情侧栏 */}
                <Show when={selectedId()}>
                    <div class="w-[40%] min-w-[300px] flex-shrink-0 flex flex-col border-l border-[color:var(--border)] bg-[color:var(--surface)]">
                        <div class="flex items-center justify-between border-b border-[color:var(--border)] px-4 py-3">
                            <h3 class="font-medium text-[color:var(--text-primary)] text-sm">记录详情</h3>
                            <button
                                class="rounded-md p-1 hover:bg-black/5 hover:text-[color:var(--text-primary)] text-[color:var(--text-muted)] transition"
                                onClick={() => setSelectedId(null)}
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M18 6L6 18M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-4 space-y-4 text-xs font-mono">
                            <Show when={!detailData.error} fallback={
                                <div class="rounded-md border border-rose-500/30 bg-rose-500/10 p-4 text-center">
                                    <div class="text-rose-400 text-sm font-medium mb-1">加载失败</div>
                                    <div class="text-rose-300 text-xs">{String(detailData.error)}</div>
                                </div>
                            }>
                                <Show when={!detailData.loading ? detailData() : null} fallback={<div class="text-[color:var(--text-muted)]">加载中...</div>}>
                                    {(detail) => (
                                        <>
                                            <Show when={detail().actor_user_id || detail().session_id}>
                                                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3 font-sans">
                                                    <h4 class="mb-2 font-bold text-[color:var(--text-muted)] text-[10px] uppercase tracking-wider">关联实体</h4>
                                                    <div class="flex flex-wrap gap-1.5">
                                                        <Show when={detail().actor_user_id && detail().actor_channel}>
                                                            <EntityRefLink
                                                                kind="actor"
                                                                id={detail().actor_user_id!}
                                                                channel={detail().actor_channel}
                                                                scope={detail().actor_scope}
                                                            />
                                                        </Show>
                                                        <Show when={detail().session_id}>
                                                            <EntityRefLink
                                                                kind="session"
                                                                id={detail().session_id}
                                                                label={detail().session_id.replace(/^Actor_/, "").slice(0, 18) + "…"}
                                                            />
                                                        </Show>
                                                    </div>
                                                </div>
                                            </Show>
                                            <Show when={detail().error}>
                                                <div class="rounded-md border border-rose-500/30 bg-rose-500/10 p-3">
                                                    <h4 class="mb-2 font-bold text-rose-400 text-[10px] uppercase tracking-wider">错误信息</h4>
                                                    <div class="text-rose-300 break-words whitespace-pre-wrap">{detail().error}</div>
                                                </div>
                                            </Show>
                                            <Show when={detail().prompt_tokens != null || detail().completion_tokens != null}>
                                                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3">
                                                    <h4 class="mb-3 font-bold text-[color:var(--text-muted)] text-[10px] uppercase tracking-wider">Token 使用</h4>
                                                    <div class="flex items-center justify-between text-[11px] mb-2 last:mb-0">
                                                        <span class="text-[color:var(--text-secondary)]">提示 (Prompt)</span>
                                                        <span class="font-bold text-sky-400">{detail().prompt_tokens ?? 0}</span>
                                                    </div>
                                                    <div class="flex items-center justify-between text-[11px] mb-2 last:mb-0">
                                                        <span class="text-[color:var(--text-secondary)]">补全 (Completion)</span>
                                                        <span class="font-bold text-emerald-400">{detail().completion_tokens ?? 0}</span>
                                                    </div>
                                                    <div class="mt-2 border-t border-white/5 pt-2 flex items-center justify-between text-[11px]">
                                                        <span class="text-[color:var(--text-primary)] font-medium">总计 (Total)</span>
                                                        <span class="font-bold text-amber-400">{detail().total_tokens ?? (detail().prompt_tokens ?? 0) + (detail().completion_tokens ?? 0)}</span>
                                                    </div>
                                                </div>
                                            </Show>
                                            <div>
                                                <JsonInspector title="请求 JSON" value={detail().request} />
                                            </div>
                                            <Show when={detail().response}>
                                                <div>
                                                    <JsonInspector title="响应 JSON" value={detail().response} />
                                                </div>
                                            </Show>
                                            <Show when={detail().metadata && (Array.isArray(detail().metadata) || (isPlainObject(detail().metadata) && Object.keys(detail().metadata as object).length > 0))}>
                                                <div>
                                                    <JsonInspector title="元数据" value={detail().metadata} />
                                                </div>
                                            </Show>
                                        </>
                                    )}
                                </Show>
                            </Show>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
        </Show>
    )
}
