import type { SkillInfo } from "./types"

export type SkillSlashCommand = {
  commandInput: string
  query: string
  stage: "command" | "search"
}

export function parseSkillSlashCommand(input: string): SkillSlashCommand | null {
  const trimmed = input.trim()
  if (!trimmed.startsWith("/")) {
    return null
  }

  if (trimmed === "/" || "/skill".startsWith(trimmed)) {
    return {
      commandInput: trimmed,
      query: "",
      stage: trimmed === "/skill" ? "search" : "command",
    }
  }

  if (!trimmed.startsWith("/skill")) {
    return null
  }

  return {
    commandInput: "/skill",
    query: trimmed.slice("/skill".length).trim(),
    stage: "search",
  }
}

export function searchSkillMatches(skills: SkillInfo[], query: string, limit = 6) {
  const normalizedQuery = normalizeSkillText(query)
  if (!normalizedQuery) {
    return skills.slice(0, limit)
  }

  return [...skills]
    .map((skill) => ({ skill, score: scoreSkill(skill, normalizedQuery) }))
    .filter((item) => item.score > 0)
    .sort((left, right) => {
      if (right.score !== left.score) {
        return right.score - left.score
      }
      return left.skill.display_name.localeCompare(right.skill.display_name)
    })
    .slice(0, limit)
    .map((item) => item.skill)
}

export function resolveSkillSlashCommand(skills: SkillInfo[], input: string) {
  const command = parseSkillSlashCommand(input)
  if (!command) {
    return null
  }

  const matches = searchSkillMatches(skills, command.query)
  const exactMatch =
    matches.find((skill) => matchesSkillExactly(skill, command.query)) ??
    (matches.length === 1 ? matches[0] : undefined)

  return {
    command,
    matches,
    exactMatch,
  }
}

function scoreSkill(skill: SkillInfo, query: string) {
  const fields = [
    { value: skill.id, base: 130 },
    { value: skill.display_name, base: 120 },
    { value: skill.description, base: 40 },
    ...skill.aliases.map((value) => ({ value, base: 110 })),
    ...skill.allowed_tools.map((value) => ({ value, base: 20 })),
  ]

  return fields.reduce((best, field) => Math.max(best, scoreField(field.value, query, field.base)), 0)
}

function scoreField(value: string, query: string, base: number) {
  const normalized = normalizeSkillText(value)
  if (!normalized) {
    return 0
  }
  if (normalized === query) {
    return base + 1000
  }
  if (normalized.startsWith(query)) {
    return base + 800
  }
  if (normalized.includes(query)) {
    return base + 600
  }

  const tokens = query.split(/\s+/).filter(Boolean)
  if (tokens.length > 0 && tokens.every((token) => normalized.includes(token))) {
    return base + 400 - tokens.length
  }

  return 0
}

function matchesSkillExactly(skill: SkillInfo, query: string) {
  const normalizedQuery = normalizeSkillText(query)
  if (!normalizedQuery) {
    return false
  }

  return [skill.id, skill.display_name, skill.description, ...skill.aliases].some(
    (field) => normalizeSkillText(field) === normalizedQuery,
  )
}

function normalizeSkillText(value: string) {
  return value.trim().toLowerCase()
}
