// admin-content/index.ts — barrel re-exports for the bilingual admin string trees.
// Each per-page module owns its own ZH/EN parallel object trees plus a
// makeContentProxy()-wrapped accessor. Pages import only the symbols they need.

export { SHARED } from "./shared"
export { DASH } from "./dashboard"
export { SESSIONS } from "./sessions"
export { TASKS } from "./tasks"
export { USERS } from "./users"
export { RESEARCH } from "./research"
export { PORTFOLIO } from "./portfolio"
export { COMPANY_PROFILES } from "./company-profiles"
export { NOTIFICATIONS } from "./notifications"
export { SCHEDULE } from "./schedule"
export { TASK_HEALTH } from "./task-health"
export { LLM_AUDIT } from "./llm-audit"
export { LOGS } from "./logs"
export { SKILLS } from "./skills"
export { SETTINGS } from "./settings"
