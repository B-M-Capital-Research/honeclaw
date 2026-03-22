import { describe, expect, it } from "bun:test"
import { resolveSkillSlashCommand, searchSkillMatches } from "./skill-command"
import type { SkillInfo } from "./types"

const skills: SkillInfo[] = [
  {
    id: "stock_research",
    display_name: "个股研究",
    description: "研究单个股票的基本面与走势",
    aliases: ["stock", "equity research"],
    tools: ["data_fetch"],
    guide: "",
  },
  {
    id: "macro_watch",
    display_name: "宏观观察",
    description: "跟踪宏观事件",
    aliases: ["macro"],
    tools: ["web_search"],
    guide: "",
  },
]

describe("skill slash command", () => {
  it("opens command mode on slash prefix", () => {
    const result = resolveSkillSlashCommand(skills, "/")
    expect(result?.command.stage).toBe("command")
    expect(result?.matches[0]?.id).toBe("stock_research")
  })

  it("resolves exact id matches", () => {
    const result = resolveSkillSlashCommand(skills, "/skill stock_research")
    expect(result?.exactMatch?.id).toBe("stock_research")
  })

  it("matches aliases", () => {
    const matches = searchSkillMatches(skills, "macro")
    expect(matches[0]?.id).toBe("macro_watch")
  })
})
