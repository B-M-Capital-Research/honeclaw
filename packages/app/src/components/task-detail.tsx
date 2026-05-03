import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { Textarea } from "@hone-financial/ui/textarea"
import { Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { useTasks } from "@/context/tasks"
import { formatShanghaiDateTime } from "@/lib/time"
import { TASKS } from "@/lib/admin-content/tasks"
import { tpl } from "@/lib/i18n"

export function TaskDetail() {
    const navigate = useNavigate()
    const tasks = useTasks()
    const heartbeatParseKind = (detail?: unknown) => {
        const root = (detail && typeof detail === "object" ? detail : {}) as Record<string, unknown>
        const scheduler =
            root["scheduler"] && typeof root["scheduler"] === "object"
                ? (root["scheduler"] as Record<string, unknown>)
                : {}
        return String(root["parse_kind"] ?? scheduler["parse_kind"] ?? "-")
    }

    const isNew = () => tasks.state.currentTaskId === "new"
    const currentJob = () => tasks.currentTask()

    const isHeartbeatDraft = () =>
        tasks.state.draft.repeat === "heartbeat" || (tasks.state.draft.tags || []).includes("heartbeat")

    const executionStatusLabel = (status: string) => {
        switch (status) {
            case "completed":
                return TASKS.detail.exec.completed
            case "noop":
                return TASKS.detail.exec.noop
            case "execution_failed":
                return TASKS.detail.exec.execution_failed
            default:
                return status || TASKS.detail.exec.unknown
        }
    }

    const sendStatusLabel = (status: string) => {
        switch (status) {
            case "sent":
                return TASKS.detail.send.sent
            case "skipped_noop":
                return TASKS.detail.send.skipped_noop
            case "skipped_error":
                return TASKS.detail.send.skipped_error
            case "send_failed":
                return TASKS.detail.send.send_failed
            case "target_resolution_failed":
                return TASKS.detail.send.target_resolution_failed
            case "duplicate_suppressed":
                return TASKS.detail.send.duplicate_suppressed
            default:
                return status || TASKS.detail.send.unknown
        }
    }

    const handleSubmit = async (e: Event) => {
        e.preventDefault()
        // Validation is mostly handled by HTML5, but we need numeric conversions
        const draft = tasks.state.draft
        await tasks.saveTask({
            channel: draft.channel,
            name: draft.name,
            task_prompt: draft.task_prompt,
            user_id: draft.user_id,
            channel_scope: draft.channel_scope,
            hour: isHeartbeatDraft() ? undefined : Number(draft.hour),
            minute: isHeartbeatDraft() ? undefined : Number(draft.minute),
            repeat: draft.repeat,
            weekday: draft.weekday !== undefined && !isNaN(Number(draft.weekday)) ? Number(draft.weekday) : undefined,
            enabled: draft.enabled,
            channel_target: draft.channel_target,
            tags: draft.tags,
        })
    }

    return (
        <Show
            when={currentJob() || isNew()}
            fallback={<EmptyState title={TASKS.detail.empty_title} description={TASKS.detail.empty_description} />}
        >
            <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm">
                <div class="flex items-center justify-between border-b border-[color:var(--border)] px-6 py-4">
                    <div>
                        <div class="text-xl font-semibold">{isNew() ? TASKS.detail.new_title : tasks.state.draft.name || TASKS.detail.fallback_title}</div>
                        <div class="mt-1 text-sm text-[color:var(--text-muted)]">
                            {isNew() ? TASKS.detail.new_subtitle : tpl(TASKS.detail.id_prefix, { id: currentJob()?.id ?? "" })}
                        </div>
                    </div>
                    <Show when={!isNew()}>
                        <div class="flex items-center gap-3">
                            <Button
                                variant="outline"
                                onClick={async () => {
                                    if (confirm(TASKS.detail.delete_confirm)) {
                                        await tasks.removeTask(currentJob()!.id)
                                        navigate("/tasks")
                                    }
                                }}
                            >
                                {TASKS.detail.delete_button}
                            </Button>
                            <Button
                                variant={tasks.state.draft.enabled ? "outline" : "primary"}
                                onClick={async () => {
                                    await tasks.toggleTask(currentJob()!.id)
                                }}
                            >
                                {tasks.state.draft.enabled ? TASKS.detail.disable_button : TASKS.detail.enable_button}
                            </Button>
                        </div>
                    </Show>
                </div>

                <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-6">
                    <form class="mx-auto max-w-2xl space-y-6" onSubmit={handleSubmit}>
                        <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_name}</label>
                                <Input
                                    required
                                    value={tasks.state.draft.name || ""}
                                    onInput={(e) => tasks.setDraft("name", e.currentTarget.value)}
                                    placeholder={TASKS.detail.field_name_placeholder}
                                />
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_user_id}</label>
                                <Input
                                    required
                                    value={tasks.state.draft.user_id || ""}
                                    onInput={(e) => tasks.setDraft("user_id", e.currentTarget.value)}
                                    placeholder={TASKS.detail.field_user_id_placeholder}
                                    disabled={!isNew()}
                                />
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_channel}</label>
                                <select
                                    class="flex h-10 w-full rounded-md border border-[color:var(--border)] bg-transparent px-3 py-2 text-sm placeholder:text-[color:var(--text-muted)] focus:outline-none focus:ring-2 focus:ring-[color:var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
                                    value={tasks.state.draft.channel || "telegram"}
                                    onChange={(e) => tasks.setDraft("channel", e.currentTarget.value)}
                                    disabled={!isNew()}
                                >
                                    <option value="telegram">Telegram</option>
                                    <option value="discord">Discord</option>
                                    <Show when={tasks.state.draft.channel === "imessage"}>
                                        <option value="imessage" disabled>
                                            {TASKS.detail.channel_imessage_disabled}
                                        </option>
                                    </Show>
                                </select>
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_channel_scope}</label>
                                <Input
                                    value={tasks.state.draft.channel_scope || ""}
                                    onInput={(e) => tasks.setDraft("channel_scope", e.currentTarget.value)}
                                    placeholder={TASKS.detail.field_channel_scope_placeholder}
                                    disabled={!isNew()}
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_hour}</label>
                                <Input
                                    type="number"
                                    min="0"
                                    max="23"
                                    required={!isHeartbeatDraft()}
                                    value={tasks.state.draft.hour ?? ""}
                                    onInput={(e) => tasks.setDraft("hour", parseInt(e.currentTarget.value, 10))}
                                    placeholder={TASKS.detail.field_hour_placeholder}
                                    disabled={isHeartbeatDraft()}
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_minute}</label>
                                <Input
                                    type="number"
                                    min="0"
                                    max="59"
                                    required={!isHeartbeatDraft()}
                                    value={tasks.state.draft.minute ?? ""}
                                    onInput={(e) => tasks.setDraft("minute", parseInt(e.currentTarget.value, 10))}
                                    placeholder={TASKS.detail.field_minute_placeholder}
                                    disabled={isHeartbeatDraft()}
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">{TASKS.detail.field_repeat}</label>
                                <select
                                    class="flex h-10 w-full rounded-md border border-[color:var(--border)] bg-transparent px-3 py-2 text-sm placeholder:text-[color:var(--text-muted)] focus:outline-none focus:ring-2 focus:ring-[color:var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
                                    value={tasks.state.draft.repeat || "daily"}
                                    onChange={(e) => {
                                        const repeat = e.currentTarget.value
                                        tasks.setDraft("repeat", repeat)
                                        if (repeat !== "weekly") {
                                            tasks.setDraft("weekday", undefined)
                                        }
                                        if (repeat === "heartbeat") {
                                            tasks.setDraft("tags", ["heartbeat"])
                                        } else {
                                            tasks.setDraft("tags", (tasks.state.draft.tags || []).filter((tag) => tag !== "heartbeat"))
                                        }
                                    }}
                                >
                                    <option value="once">{TASKS.detail.repeat_once}</option>
                                    <option value="daily">{TASKS.detail.repeat_daily}</option>
                                    <option value="workday">{TASKS.detail.repeat_workday}</option>
                                    <option value="trading_day">{TASKS.detail.repeat_trading_day}</option>
                                    <option value="holiday">{TASKS.detail.repeat_holiday}</option>
                                    <option value="weekly">{TASKS.detail.repeat_weekly}</option>
                                    <option value="heartbeat">{TASKS.detail.repeat_heartbeat}</option>
                                </select>
                                <Show when={isHeartbeatDraft()}>
                                    <p class="text-[11px] text-[color:var(--text-muted)] mt-1">
                                        {TASKS.detail.heartbeat_help}
                                    </p>
                                </Show>
                            </div>

                            <Show when={tasks.state.draft.repeat === "weekly"}>
                                <div class="space-y-2">
                                    <label class="text-sm font-medium">{TASKS.detail.field_weekday}</label>
                                    <Input
                                        type="number"
                                        min="0"
                                        max="6"
                                        value={tasks.state.draft.weekday !== undefined ? String(tasks.state.draft.weekday) : ""}
                                        onInput={(e) => tasks.setDraft("weekday", parseInt(e.currentTarget.value, 10))}
                                        placeholder={TASKS.detail.field_weekday_placeholder}
                                    />
                                    <p class="text-[11px] text-[color:var(--text-muted)] mt-1">{TASKS.detail.weekday_help}</p>
                                </div>
                            </Show>
                        </div>

                        <div class="space-y-2">
                            <label class="text-sm font-medium">{TASKS.detail.field_target}</label>
                            <Input
                                value={tasks.state.draft.channel_target || ""}
                                onInput={(e) => tasks.setDraft("channel_target", e.currentTarget.value)}
                                placeholder={TASKS.detail.field_target_placeholder}
                            />
                            <p class="text-[11px] text-[color:var(--text-muted)]">{TASKS.detail.field_target_help}</p>
                        </div>

                        <div class="space-y-2">
                            <label class="text-sm font-medium">{TASKS.detail.field_prompt}</label>
                            <Textarea
                                required
                                rows={5}
                                value={tasks.state.draft.task_prompt || ""}
                                onInput={(e) => tasks.setDraft("task_prompt", e.currentTarget.value)}
                                placeholder={TASKS.detail.field_prompt_placeholder}
                            />
                        </div>

                        <div class="pt-4 flex items-center justify-end border-t border-[color:var(--border)]">
                            <Button type="submit" disabled={tasks.state.submitting}>
                                {tasks.state.submitting ? TASKS.detail.saving_button : TASKS.detail.save_button}
                            </Button>
                        </div>

                        <Show when={!isNew()}>
                            <div class="space-y-3 border-t border-[color:var(--border)] pt-6">
                                <div class="flex items-center justify-between gap-3">
                                    <div>
                                        <div class="text-base font-semibold">{TASKS.detail.history_title}</div>
                                        <div class="text-xs text-[color:var(--text-muted)]">
                                            {TASKS.detail.history_subtitle}
                                        </div>
                                    </div>
                                    <div class="text-xs text-[color:var(--text-muted)]">
                                        {tpl(TASKS.detail.history_count, { count: tasks.executionRecords().length })}
                                    </div>
                                </div>

                                <div class="overflow-x-auto rounded-lg border border-[color:var(--border)]">
                                    <Show
                                        when={tasks.executionRecords().length > 0}
                                        fallback={
                                            <div class="px-4 py-8 text-center text-sm text-[color:var(--text-muted)]">
                                                {TASKS.detail.history_empty}
                                            </div>
                                        }
                                    >
                                        <table class="min-w-full divide-y divide-[color:var(--border)] text-sm">
                                            <thead class="bg-black/5 text-left text-xs uppercase tracking-wide text-[color:var(--text-muted)]">
                                                <tr>
                                                    <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_time}</th>
                                                    <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_exec}</th>
                                                    <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_send}</th>
                                                    <Show when={isHeartbeatDraft()}>
                                                        <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_hit}</th>
                                                        <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_delivered}</th>
                                                    </Show>
                                                    <th class="px-4 py-3 font-medium">{TASKS.detail.history_col_summary}</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-[color:var(--border)]">
                                                {tasks.executionRecords().map((record) => (
                                                    <tr class="align-top">
                                                        <td class="whitespace-nowrap px-4 py-3 text-[color:var(--text-secondary)]">
                                                            {formatShanghaiDateTime(record.executed_at)}
                                                        </td>
                                                        <td class="whitespace-nowrap px-4 py-3 text-[color:var(--text-primary)]">
                                                            {executionStatusLabel(record.execution_status)}
                                                        </td>
                                                        <td class="whitespace-nowrap px-4 py-3 text-[color:var(--text-primary)]">
                                                            {sendStatusLabel(record.message_send_status)}
                                                        </td>
                                                        <Show when={isHeartbeatDraft()}>
                                                            <td class="whitespace-nowrap px-4 py-3 text-[color:var(--text-primary)]">
                                                                {record.should_deliver ? TASKS.detail.yes : TASKS.detail.no}
                                                            </td>
                                                            <td class="whitespace-nowrap px-4 py-3 text-[color:var(--text-primary)]">
                                                                {record.delivered ? TASKS.detail.yes : TASKS.detail.no}
                                                            </td>
                                                        </Show>
                                                        <td class="max-w-[460px] px-4 py-3 text-[color:var(--text-secondary)]">
                                                            <div class="space-y-1">
                                                                <Show when={record.response_preview}>
                                                                    <div class="line-clamp-3 break-words">{record.response_preview}</div>
                                                                </Show>
                                                                <Show when={record.error_message}>
                                                                    <div class="break-words text-rose-500">{record.error_message}</div>
                                                                </Show>
                                                                <Show when={isHeartbeatDraft() && record.detail}>
                                                                    <div class="text-[11px] text-[color:var(--text-muted)]">
                                                                        {TASKS.detail.parse_kind_prefix} {heartbeatParseKind(record.detail)}
                                                                    </div>
                                                                </Show>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </Show>
                                </div>
                            </div>
                        </Show>
                    </form>
                </div>
            </div>
        </Show>
    )
}
