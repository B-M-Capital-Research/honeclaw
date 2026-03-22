import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { Textarea } from "@hone-financial/ui/textarea"
import { Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { useTasks } from "@/context/tasks"

export function TaskDetail() {
    const navigate = useNavigate()
    const tasks = useTasks()

    const isNew = () => tasks.state.currentTaskId === "new"
    const currentJob = () => tasks.currentTask()

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
            hour: Number(draft.hour),
            minute: Number(draft.minute),
            repeat: draft.repeat,
            weekday: draft.weekday !== undefined && !isNaN(Number(draft.weekday)) ? Number(draft.weekday) : undefined,
            enabled: draft.enabled,
            channel_target: draft.channel_target,
        })
    }

    return (
        <Show
            when={currentJob() || isNew()}
            fallback={<EmptyState title="从左侧选择一个任务" description="你可以查看、新建或管理你的定时触发工作流。" />}
        >
            <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm">
                <div class="flex items-center justify-between border-b border-[color:var(--border)] px-6 py-4">
                    <div>
                        <div class="text-xl font-semibold">{isNew() ? "新建定时任务" : tasks.state.draft.name || "任务详情"}</div>
                        <div class="mt-1 text-sm text-[color:var(--text-muted)]">
                            {isNew() ? "设定触发条件和 Agent 执行动作" : `ID: ${currentJob()?.id}`}
                        </div>
                    </div>
                    <Show when={!isNew()}>
                        <div class="flex items-center gap-3">
                            <Button
                                variant="outline"
                                onClick={async () => {
                                    if (confirm("确定要删除此任务吗？")) {
                                        await tasks.removeTask(currentJob()!.id)
                                        navigate("/tasks")
                                    }
                                }}
                            >
                                删除
                            </Button>
                            <Button
                                variant={tasks.state.draft.enabled ? "outline" : "primary"}
                                onClick={async () => {
                                    await tasks.toggleTask(currentJob()!.id)
                                }}
                            >
                                {tasks.state.draft.enabled ? "停用任务" : "启用任务"}
                            </Button>
                        </div>
                    </Show>
                </div>

                <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-6">
                    <form class="mx-auto max-w-2xl space-y-6" onSubmit={handleSubmit}>
                        <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                            <div class="space-y-2">
                                <label class="text-sm font-medium">任务名称</label>
                                <Input
                                    required
                                    value={tasks.state.draft.name || ""}
                                    onInput={(e) => tasks.setDraft("name", e.currentTarget.value)}
                                    placeholder="例如：每日大盘早报"
                                />
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">归属用户 (User ID)</label>
                                <Input
                                    required
                                    value={tasks.state.draft.user_id || ""}
                                    onInput={(e) => tasks.setDraft("user_id", e.currentTarget.value)}
                                    placeholder="输入当前任务的所有者 ID"
                                    disabled={!isNew()}
                                />
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">触发渠道</label>
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
                                            iMessage (disabled)
                                        </option>
                                    </Show>
                                </select>
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">群范围 (Channel Scope)</label>
                                <Input
                                    value={tasks.state.draft.channel_scope || ""}
                                    onInput={(e) => tasks.setDraft("channel_scope", e.currentTarget.value)}
                                    placeholder="私聊留空，群聊可填 g:123:c:456"
                                    disabled={!isNew()}
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">执行时间 (小时)</label>
                                <Input
                                    type="number"
                                    min="0"
                                    max="23"
                                    required
                                    value={tasks.state.draft.hour ?? ""}
                                    onInput={(e) => tasks.setDraft("hour", parseInt(e.currentTarget.value, 10))}
                                    placeholder="0 - 23"
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">执行时间 (分钟)</label>
                                <Input
                                    type="number"
                                    min="0"
                                    max="59"
                                    required
                                    value={tasks.state.draft.minute ?? ""}
                                    onInput={(e) => tasks.setDraft("minute", parseInt(e.currentTarget.value, 10))}
                                    placeholder="0 - 59"
                                />
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium">重复频率</label>
                                <select
                                    class="flex h-10 w-full rounded-md border border-[color:var(--border)] bg-transparent px-3 py-2 text-sm placeholder:text-[color:var(--text-muted)] focus:outline-none focus:ring-2 focus:ring-[color:var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
                                    value={tasks.state.draft.repeat || "daily"}
                                    onChange={(e) => {
                                        tasks.setDraft("repeat", e.currentTarget.value)
                                        if (e.currentTarget.value !== "weekly") {
                                            tasks.setDraft("weekday", undefined)
                                        }
                                    }}
                                >
                                    <option value="once">单次 (Once)</option>
                                    <option value="daily">每天 (Daily)</option>
                                    <option value="workday">工作日 (Workday)</option>
                                    <option value="trading_day">交易日 (Trading Day)</option>
                                    <option value="holiday">节假日 (Holiday)</option>
                                    <option value="weekly">每周 (Weekly)</option>
                                </select>
                            </div>

                            <Show when={tasks.state.draft.repeat === "weekly"}>
                                <div class="space-y-2">
                                    <label class="text-sm font-medium">周几执行 (针对 Weekly)</label>
                                    <Input
                                        type="number"
                                        min="0"
                                        max="6"
                                        value={tasks.state.draft.weekday !== undefined ? String(tasks.state.draft.weekday) : ""}
                                        onInput={(e) => tasks.setDraft("weekday", parseInt(e.currentTarget.value, 10))}
                                        placeholder="0(周一) - 6(周日)"
                                    />
                                    <p class="text-[11px] text-[color:var(--text-muted)] mt-1">取值 0 - 6，其中 0 代表星期一</p>
                                </div>
                            </Show>
                        </div>

                        <div class="space-y-2">
                            <label class="text-sm font-medium">Target ID / UID (渠道目标)</label>
                            <Input
                                value={tasks.state.draft.channel_target || ""}
                                onInput={(e) => tasks.setDraft("channel_target", e.currentTarget.value)}
                                placeholder="缺省为当前用户"
                            />
                            <p class="text-[11px] text-[color:var(--text-muted)]">如果是给特定群发，可以填群组 ID 或手机号。</p>
                        </div>

                        <div class="space-y-2">
                            <label class="text-sm font-medium">任务指令 (Task Prompt)</label>
                            <Textarea
                                required
                                rows={5}
                                value={tasks.state.draft.task_prompt || ""}
                                onInput={(e) => tasks.setDraft("task_prompt", e.currentTarget.value)}
                                placeholder="给 Agent 发送的确切指令，例如：总结昨天纳斯达克市场的核心科技股走势并提取关注点。"
                            />
                        </div>

                        <div class="pt-4 flex items-center justify-end border-t border-[color:var(--border)]">
                            <Button type="submit" disabled={tasks.state.submitting}>
                                {tasks.state.submitting ? "保存中..." : "保存设置"}
                            </Button>
                        </div>
                    </form>
                </div>
            </div>
        </Show>
    )
}
