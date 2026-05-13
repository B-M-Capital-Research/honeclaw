import { beforeEach, describe, expect, it } from "bun:test"
import {
  readStoredModule,
  readStoredReadAt,
  readStoredResearchTasks,
  readStoredSelection,
  writeStoredModule,
  writeStoredReadAt,
  writeStoredResearchTasks,
  writeStoredSelection,
} from "./persist"
import type { ResearchTask } from "./types"

function researchTaskFixture(patch: Partial<ResearchTask> = {}): ResearchTask {
  return {
    id: "local-task-1",
    task_id: "remote-task-1",
    task_name: "AI earnings scan",
    company_name: "Apple",
    status: "running",
    progress: "40%",
    created_at: "2026-05-13T00:00:00Z",
    ...patch,
  }
}

function installLocalStorageMock(): void {
  const storage = new Map<string, string>()
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      clear: () => storage.clear(),
      getItem: (key: string) => storage.get(key) ?? null,
      removeItem: (key: string) => storage.delete(key),
      setItem: (key: string, value: string) => storage.set(key, value),
    },
  })
}

describe("persist", () => {
  beforeEach(() => {
    installLocalStorageMock()
    localStorage.clear()
  })

  it("round-trips module, selection, and read timestamps", () => {
    writeStoredModule("skills")
    writeStoredSelection({ userId: "alice", skillId: "market_analysis" })
    writeStoredReadAt({ alice: "2026-03-07T00:00:00Z" })

    expect(readStoredModule()).toBe("skills")
    expect(readStoredSelection()).toEqual({ userId: "alice", skillId: "market_analysis" })
    expect(readStoredReadAt()).toEqual({ alice: "2026-03-07T00:00:00Z" })
  })

  it("falls back for legacy modules, missing keys, and invalid JSON", () => {
    localStorage.setItem("hone.console.module", JSON.stringify("help"))
    localStorage.setItem("hone.console.selection", "{bad json")
    localStorage.setItem("hone.console.readAt", "{bad json")
    localStorage.setItem("hone.console.researchTasks", "{bad json")

    expect(readStoredModule()).toBe("start")
    expect(readStoredSelection()).toEqual({})
    expect(readStoredReadAt()).toEqual({})
    expect(readStoredResearchTasks()).toEqual([])
  })

  it("round-trips research tasks for local task resume", () => {
    const runningTask = researchTaskFixture()
    const completedTask = researchTaskFixture({
      id: "local-task-2",
      task_id: "remote-task-2",
      status: "completed",
      progress: "100%",
      completed_at: "2026-05-13T00:05:00Z",
      answer_markdown: "# Report",
    })

    writeStoredResearchTasks([runningTask, completedTask])

    expect(readStoredResearchTasks()).toEqual([runningTask, completedTask])
  })
})
