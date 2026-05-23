import { describe, expect, it } from "bun:test"

import {
  cronJobUpsertInputFromDraft,
  isHeartbeatCronDraft,
  tagsForRepeatDraft,
} from "./task-detail-model"

describe("task-detail-model", () => {
  it("detects heartbeat drafts from repeat or tags", () => {
    expect(isHeartbeatCronDraft({ repeat: "heartbeat" })).toBe(true)
    expect(isHeartbeatCronDraft({ repeat: "daily", tags: ["heartbeat"] })).toBe(
      true,
    )
    expect(isHeartbeatCronDraft({ repeat: "daily", tags: ["market"] })).toBe(
      false,
    )
  })

  it("keeps heartbeat tags synchronized with repeat changes", () => {
    expect(tagsForRepeatDraft("heartbeat", ["market"])).toEqual(["heartbeat"])
    expect(tagsForRepeatDraft("daily", ["market", "heartbeat"])).toEqual([
      "market",
    ])
    expect(tagsForRepeatDraft("weekly", undefined)).toEqual([])
  })

  it("builds cron upsert payloads from draft state", () => {
    expect(
      cronJobUpsertInputFromDraft({
        channel: "discord",
        user_id: "u1",
        channel_scope: "dm",
        name: "daily",
        task_prompt: "run",
        hour: 9,
        minute: 30,
        repeat: "weekly",
        weekday: 2,
        enabled: true,
        channel_target: "alerts",
        tags: ["market"],
      }),
    ).toEqual({
      channel: "discord",
      user_id: "u1",
      channel_scope: "dm",
      name: "daily",
      task_prompt: "run",
      hour: 9,
      minute: 30,
      repeat: "weekly",
      weekday: 2,
      enabled: true,
      channel_target: "alerts",
      tags: ["market"],
    })

    expect(
      cronJobUpsertInputFromDraft({
        repeat: "heartbeat",
        hour: 9,
        minute: 30,
        tags: ["heartbeat"],
      }),
    ).toMatchObject({
      hour: undefined,
      minute: undefined,
      repeat: "heartbeat",
      tags: ["heartbeat"],
    })
  })
})
