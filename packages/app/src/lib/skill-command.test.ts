import { describe, expect, it } from "bun:test"
import { parseSkillSlashCommand, resolveSkillSlashCommand, searchSkillMatches } from "./skill-command"
import type { SkillInfo } from "./types"

const skills: SkillInfo[] = [
  {
    id: "stock_research",
    display_name: "个股研究",
    description: "研究单个股票的基本面与走势",
    aliases: ["stock", "equity research"],
    allowed_tools: ["data_fetch"],
    user_invocable: true,
    context: "inline",
    loaded_from: "system",
    enabled: true,
    has_script: false,
    has_path_gate: false,
    paths: [],
  },
  {
    id: "macro_watch",
    display_name: "宏观观察",
    description: "跟踪宏观事件",
    aliases: ["macro"],
    allowed_tools: ["web_search"],
    user_invocable: true,
    context: "inline",
    loaded_from: "system",
    enabled: true,
    has_script: false,
    has_path_gate: false,
    paths: [],
  },
  {
    id: "disabled_skill",
    display_name: "禁用技能",
    description: "已被禁用",
    aliases: ["disabled"],
    allowed_tools: ["skill_tool"],
    user_invocable: true,
    context: "inline",
    loaded_from: "system",
    enabled: false,
    has_script: false,
    has_path_gate: false,
    paths: [],
  },
]

describe("skill slash command", () => {
  it("opens command mode on slash prefix", () => {
    const result = resolveSkillSlashCommand(skills, "/")
    expect(result?.command.stage).toBe("command")
    expect(result?.matches[0]?.id).toBe("stock_research")
  })

  it("keeps partial /skill prefixes in command mode", () => {
    expect(parseSkillSlashCommand("/sk")).toEqual({
      commandInput: "/sk",
      query: "",
      stage: "command",
    })
  })

  it("resolves exact id matches", () => {
    const result = resolveSkillSlashCommand(skills, "/skill stock_research")
    expect(result?.exactMatch?.id).toBe("stock_research")
  })

  it("normalizes surrounding whitespace for exact display-name matches", () => {
    const result = resolveSkillSlashCommand(skills, "   /skill   个股研究   ")
    expect(result?.exactMatch?.id).toBe("stock_research")
  })

  it("matches aliases", () => {
    const matches = searchSkillMatches(skills, "macro")
    expect(matches[0]?.id).toBe("macro_watch")
  })

  it("hides disabled skills from slash search", () => {
    const matches = searchSkillMatches(skills, "disabled")
    expect(matches).toEqual([])
  })

  it("returns null for unrelated slash commands", () => {
    expect(resolveSkillSlashCommand(skills, "/help")).toBeNull()
  })
})
