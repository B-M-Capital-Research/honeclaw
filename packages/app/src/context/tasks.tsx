import { createContext, createEffect, createResource, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getCronJobs, getCronJob, createCronJob, updateCronJob, toggleCronJob, deleteCronJob } from "@/lib/api"
import { actorFromJob } from "@/lib/actors"
import type { CronJobDetailInfo, CronJobInfo, CronJobUpsertInput } from "@/lib/types"
import { useBackend } from "./backend"

type TasksContextValue = ReturnType<typeof createTasksState>

const TasksContext = createContext<TasksContextValue>()

function createTasksState() {
    const backend = useBackend()
    const [state, setState] = createStore({
        currentTaskId: undefined as string | undefined,
        loading: false,
        draft: {} as Partial<CronJobUpsertInput>,
        submitting: false,
    })

    // We load jobs for all users since there's no selected user for tasks in the UI generally
    // However, it can be passed if needed at the component level. Here we fetch all.
    const [jobs, { refetch }] = createResource(
        () => {
            if (!backend.state.connected || !backend.hasCapability("cron_jobs")) {
                return undefined
            }
            if (backend.state.isDesktop && !backend.state.resolvedBaseUrl) {
                return undefined
            }
            return "cron_jobs"
        },
        async () => {
            try {
                return await getCronJobs()
            } catch (error) {
                console.warn("getCronJobs failed", error)
                return []
            }
        },
    )

    const [taskDetail, { refetch: refetchTaskDetail }] = createResource(
        () => {
            if (!backend.state.connected || !backend.hasCapability("cron_jobs")) return undefined
            const taskId = state.currentTaskId
            if (!taskId || taskId === "new") return undefined
            return taskId
        },
        async (taskId): Promise<CronJobDetailInfo | undefined> => {
            try {
                return await getCronJob(taskId)
            } catch (error) {
                console.warn("getCronJob failed", error)
                return undefined
            }
        },
    )

    const currentTask = () => {
        if (!state.currentTaskId || state.currentTaskId === "new") return undefined
        return taskDetail()?.job || jobs()?.find((j) => j.id === state.currentTaskId)
    }

    const executionRecords = () => taskDetail()?.executions || []

    const selectTask = (taskId?: string) => {
        setState("currentTaskId", taskId)
        if (taskId === "new") {
                setState("draft", {
                    user_id: "", // 明确表示置空以待用户输入或是预设值
                    name: "",
                    task_prompt: "",
                    hour: 0,
                    minute: 0,
                    repeat: "daily",
                    enabled: true,
                    channel: "telegram",
                    tags: [],
                })
        } else {
            const existing = taskDetail()?.job || currentTask()
            if (existing) {
                setState("draft", {
                    user_id: existing.user_id,
                    channel_scope: existing.channel_scope,
                    name: existing.name,
                    task_prompt: existing.task_prompt,
                    hour: existing.schedule.hour,
                    minute: existing.schedule.minute,
                    repeat: existing.schedule.repeat,
                    weekday: existing.schedule.weekday,
                    enabled: existing.enabled,
                    channel: existing.channel,
                    channel_target: existing.channel_target,
                    tags: existing.tags || [],
                })
            }
        }
    }

    createEffect(() => {
        // 如果我们尚未成功选中或者没有把数据赋予 draft，但实际已经 load 完毕了
        const cid = state.currentTaskId
        if (cid && cid !== "new" && !state.draft.name && (taskDetail() || jobs())) {
            // 这个操作会在资源拉取完毕后把数据注入 draft 中解决刷新空白的问题
            selectTask(cid)
        }
    })

    const saveTask = async (input: CronJobUpsertInput) => {
        setState("submitting", true)
        try {
            if (state.currentTaskId === "new") {
                if (!input.user_id) {
                    alert("归属用户 ID 不能为空")
                    return
                }
                const newJob = await createCronJob(input)
                await refetch()
                await refetchTaskDetail()
                selectTask(newJob.id)
            } else if (state.currentTaskId) {
                const existing = currentTask()
                await updateCronJob(state.currentTaskId, input, existing ? actorFromJob(existing) : undefined)
                await refetch()
                await refetchTaskDetail()
            }
        } finally {
            setState("submitting", false)
        }
    }

    const toggleTask = async (taskId: string) => {
        const existing = jobs()?.find((job) => job.id === taskId)
        await toggleCronJob(taskId, existing ? actorFromJob(existing) : undefined)
        await refetch()
        if (state.currentTaskId === taskId) {
            await refetchTaskDetail()
        }
    }

    const removeTask = async (taskId: string) => {
        const existing = jobs()?.find((job) => job.id === taskId)
        await deleteCronJob(taskId, existing ? actorFromJob(existing) : undefined)
        await refetch()
        if (state.currentTaskId === taskId) {
            selectTask(undefined)
        }
    }

    return {
        state,
        jobs,
        taskDetail,
        refetch,
        currentTask,
        executionRecords,
        selectTask,
        saveTask,
        toggleTask,
        removeTask,
        setDraft: (key: keyof CronJobUpsertInput, value: any) => {
            setState("draft", key as any, value)
        },
    }
}

export function TasksProvider(props: ParentProps) {
    const value = createTasksState()
    return <TasksContext.Provider value={value}>{props.children}</TasksContext.Provider>
}

export function useTasks() {
    const value = useContext(TasksContext)
    if (!value) throw new Error("TasksProvider missing")
    return value
}
