// structure.test.ts — guard against ZH/EN content drift in admin-content/.
//
// Every per-page module exports a `__<NAME>_TREES__` object containing both
// the ZH and EN trees. This test walks both trees in parallel and asserts that
// every nested key on one side has a matching key on the other. The proxy in
// makeContentProxy() silently returns `undefined` for missing keys, so without
// this guard a typo or a forgotten translation would surface only at runtime
// as blank UI — usually noticed weeks later.

import { describe, expect, it } from "bun:test"

import { __SHARED_TREES__ } from "./shared"
import { __DASH_TREES__ } from "./dashboard"
import { __SESSIONS_TREES__ } from "./sessions"
import { __TASKS_TREES__ } from "./tasks"
import { __USERS_TREES__ } from "./users"
import { __RESEARCH_TREES__ } from "./research"
import { __PORTFOLIO_TREES__ } from "./portfolio"
import { __COMPANY_PROFILES_TREES__ } from "./company-profiles"
import { __NOTIFICATIONS_TREES__ } from "./notifications"
import { __SCHEDULE_TREES__ } from "./schedule"
import { __TASK_HEALTH_TREES__ } from "./task-health"
import { __LLM_AUDIT_TREES__ } from "./llm-audit"
import { __LOGS_TREES__ } from "./logs"
import { __SKILLS_TREES__ } from "./skills"
import { __SETTINGS_TREES__ } from "./settings"

type Tree = Record<string, unknown>
type TreesPair = { zh: Tree; en: Tree }

const MODULES: Array<[string, TreesPair]> = [
  ["shared", __SHARED_TREES__],
  ["dashboard", __DASH_TREES__],
  ["sessions", __SESSIONS_TREES__],
  ["tasks", __TASKS_TREES__],
  ["users", __USERS_TREES__],
  ["research", __RESEARCH_TREES__],
  ["portfolio", __PORTFOLIO_TREES__],
  ["company-profiles", __COMPANY_PROFILES_TREES__],
  ["notifications", __NOTIFICATIONS_TREES__],
  ["schedule", __SCHEDULE_TREES__],
  ["task-health", __TASK_HEALTH_TREES__],
  ["llm-audit", __LLM_AUDIT_TREES__],
  ["logs", __LOGS_TREES__],
  ["skills", __SKILLS_TREES__],
  ["settings", __SETTINGS_TREES__],
]

function isPlainObject(value: unknown): value is Tree {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.getPrototypeOf(value) === Object.prototype
  )
}

function diffShape(
  a: Tree,
  b: Tree,
  path: string[] = [],
): { onlyInA: string[]; onlyInB: string[]; typeMismatch: string[] } {
  const onlyInA: string[] = []
  const onlyInB: string[] = []
  const typeMismatch: string[] = []
  const seen = new Set<string>()
  for (const key of Object.keys(a)) {
    seen.add(key)
    const dotted = [...path, key].join(".")
    if (!(key in b)) {
      onlyInA.push(dotted)
      continue
    }
    const av = a[key]
    const bv = b[key]
    if (isPlainObject(av) && isPlainObject(bv)) {
      const sub = diffShape(av, bv, [...path, key])
      onlyInA.push(...sub.onlyInA)
      onlyInB.push(...sub.onlyInB)
      typeMismatch.push(...sub.typeMismatch)
    } else if (isPlainObject(av) !== isPlainObject(bv)) {
      typeMismatch.push(dotted)
    } else if (Array.isArray(av) !== Array.isArray(bv)) {
      typeMismatch.push(dotted)
    } else if (Array.isArray(av) && Array.isArray(bv) && av.length !== bv.length) {
      typeMismatch.push(`${dotted} (zh.len=${av.length} != en.len=${bv.length})`)
    }
  }
  for (const key of Object.keys(b)) {
    if (seen.has(key)) continue
    onlyInB.push([...path, key].join("."))
  }
  return { onlyInA, onlyInB, typeMismatch }
}

describe("admin-content key parity", () => {
  for (const [name, trees] of MODULES) {
    it(`${name}: zh and en trees share the same shape`, () => {
      const diff = diffShape(trees.zh, trees.en)
      expect({
        module: name,
        onlyInZh: diff.onlyInA,
        onlyInEn: diff.onlyInB,
        typeMismatch: diff.typeMismatch,
      }).toEqual({
        module: name,
        onlyInZh: [],
        onlyInEn: [],
        typeMismatch: [],
      })
    })
  }
})
