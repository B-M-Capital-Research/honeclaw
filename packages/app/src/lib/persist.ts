const moduleKey = "hone.console.module"
const selectionKey = "hone.console.selection"
const readAtKey = "hone.console.readAt"
const researchTasksKey = "hone.console.researchTasks"

export type StoredModule =
  | "dashboard"
  | "start"
  | "sessions"
  | "skills"
  | "tasks"
  | "users"
  | "portfolio"
  | "memory"
  | "research"
  | "llm-audit"
  | "logs"
  | "task-health"
  | "notifications"
  | "settings"

export type StoredSelection = {
  userId?: string
  skillId?: string
  taskId?: string
  portfolioActorKey?: string
  portfolioUserId?: string
  companyProfileActorKey?: string
  companyProfileId?: string
}

function readValue<T>(key: string, fallback: T) {
  if (typeof localStorage === "undefined") return fallback
  try {
    const raw = localStorage.getItem(key)
    return raw ? (JSON.parse(raw) as T) : fallback
  } catch {
    return fallback
  }
}

function writeValue<T>(key: string, value: T) {
  if (typeof localStorage === "undefined") return
  localStorage.setItem(key, JSON.stringify(value))
}

export function readStoredModule(): StoredModule {
  const v = readValue<string>(moduleKey, "start")
  if (v === "help") return "start"
  return v as StoredModule
}

export function writeStoredModule(value: StoredModule) {
  writeValue(moduleKey, value)
}

export function readStoredSelection() {
  return readValue<StoredSelection>(selectionKey, {})
}

export function writeStoredSelection(value: StoredSelection) {
  writeValue(selectionKey, value)
}

export function readStoredReadAt() {
  return readValue<Record<string, string>>(readAtKey, {})
}

export function writeStoredReadAt(value: Record<string, string>) {
  writeValue(readAtKey, value)
}

import type { ResearchTask } from "./types"

export function readStoredResearchTasks(): ResearchTask[] {
  return readValue<ResearchTask[]>(researchTasksKey, [])
}

export function writeStoredResearchTasks(tasks: ResearchTask[]) {
  writeValue(researchTasksKey, tasks)
}
