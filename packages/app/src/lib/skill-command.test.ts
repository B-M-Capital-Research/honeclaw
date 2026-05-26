import { describe, expect, it } from "bun:test"
import { parseSkillSlashCommand, resolveSkillSlashCommand, searchSkillMatches } from "./skill-command"
import type { SkillInfo } from "./types"

const skillFixtures: SkillInfo[] = [
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

function requireValue<T>(value: T | null | undefined, label: string): T {
  if (value == null) {
    throw new Error(`${label} was not found`)
  }
  return value
}

describe("skill slash command", () => {
  it("opens command mode on slash prefix", () => {
    const slashCommandResult = requireValue(
      resolveSkillSlashCommand(skillFixtures, "/"),
      "slash command result",
    )
    expect(slashCommandResult.command.stage).toBe("command")
    expect(
      requireValue(slashCommandResult.matches[0], "first slash match").id,
    ).toBe("stock_research")
  })

  it("keeps partial /skill prefixes in command mode", () => {
    expect(parseSkillSlashCommand("/sk")).toEqual({
      commandInput: "/sk",
      query: "",
      stage: "command",
    })
  })

  it("resolves exact id matches", () => {
    const exactIdResult = requireValue(
      resolveSkillSlashCommand(skillFixtures, "/skill stock_research"),
      "exact id result",
    )
    expect(requireValue(exactIdResult.exactMatch, "exact id match").id).toBe(
      "stock_research",
    )
  })

  it("normalizes surrounding whitespace for exact display-name matches", () => {
    const exactDisplayNameResult = requireValue(
      resolveSkillSlashCommand(skillFixtures, "   /skill   个股研究   "),
      "exact display-name result",
    )
    expect(
      requireValue(exactDisplayNameResult.exactMatch, "exact display-name match")
        .id,
    ).toBe("stock_research")
  })

  it("matches aliases", () => {
    const matches = searchSkillMatches(skillFixtures, "macro")
    expect(requireValue(matches[0], "first alias match").id).toBe("macro_watch")
  })

  it("hides disabled skills from slash search", () => {
    const matches = searchSkillMatches(skillFixtures, "disabled")
    expect(matches).toEqual([])
  })

  it("returns null for unrelated slash commands", () => {
    expect(resolveSkillSlashCommand(skillFixtures, "/help")).toBeNull()
  })
})
