import { beforeEach, describe, expect, it } from "bun:test"
import {
  readStoredModule,
  readStoredReadAt,
  readStoredSelection,
  writeStoredModule,
  writeStoredReadAt,
  writeStoredSelection,
} from "./persist"

describe("persist", () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it("stores module and selection", () => {
    writeStoredModule("skills")
    writeStoredSelection({ userId: "alice", skillId: "market_analysis" })
    writeStoredReadAt({ alice: "2026-03-07T00:00:00Z" })

    expect(readStoredModule()).toBe("skills")
    expect(readStoredSelection()).toEqual({ userId: "alice", skillId: "market_analysis" })
    expect(readStoredReadAt()).toEqual({ alice: "2026-03-07T00:00:00Z" })
  })
})
