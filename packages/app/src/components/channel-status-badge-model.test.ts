import { describe, expect, it } from "bun:test"

import {
  channelBadgeDotClass,
  statusDotClass,
  statusLabel,
  statusTextClass,
  summarizeChannelStatuses,
} from "./channel-status-badge-model"
import type { ChannelStatusInfo } from "@/lib/types"

function channel(
  patch: Partial<ChannelStatusInfo> = {},
): ChannelStatusInfo {
  return {
    id: "discord",
    label: "Discord",
    enabled: true,
    running: true,
    status: "running",
    detail: "ok",
    processes: [],
    ...patch,
  }
}

describe("channel-status-badge-model", () => {
  it("maps channel status tokens to labels and classes", () => {
    expect(statusLabel("running")).toBe("运行中")
    expect(statusLabel("unsupported")).toBe("不支持")
    expect(statusLabel("custom")).toBe("custom")

    expect(statusDotClass("running")).toBe("bg-[color:var(--success)]")
    expect(statusDotClass("disabled")).toBe(
      "bg-[color:var(--text-muted)] opacity-40",
    )
    expect(statusTextClass("stopped")).toBe("text-rose-400")
  })

  it("summarizes channel state without repeating component filters", () => {
    expect(summarizeChannelStatuses([])).toEqual({
      hasData: false,
      successCount: 0,
      failCount: 0,
      duplicateProcessCount: 0,
    })

    expect(
      summarizeChannelStatuses([
        channel({ processes: [{ pid: 1, running: true }] }),
        channel({
          id: "telegram",
          enabled: true,
          running: false,
          processes: [
            { pid: 2, running: true },
            { pid: 3, running: false },
          ],
        }),
        channel({ id: "feishu", enabled: false, running: false }),
      ]),
    ).toEqual({
      hasData: true,
      successCount: 1,
      failCount: 1,
      duplicateProcessCount: 1,
    })
  })

  it("derives badge dot priority from backend and channel state", () => {
    const empty = summarizeChannelStatuses([])
    const healthy = summarizeChannelStatuses([channel()])
    const failing = summarizeChannelStatuses([
      channel({ running: false, enabled: true }),
    ])

    expect(
      channelBadgeDotClass({
        backendConnected: false,
        backendInitializing: false,
        channelError: "",
        counts: healthy,
      }),
    ).toBe("bg-rose-500")
    expect(
      channelBadgeDotClass({
        backendConnected: true,
        backendInitializing: false,
        channelError: "poll failed",
        counts: healthy,
      }),
    ).toBe("bg-amber-400")
    expect(
      channelBadgeDotClass({
        backendConnected: true,
        backendInitializing: false,
        channelError: "",
        counts: empty,
      }),
    ).toBe("bg-[color:var(--text-muted)]")
    expect(
      channelBadgeDotClass({
        backendConnected: true,
        backendInitializing: false,
        channelError: "",
        counts: failing,
      }),
    ).toBe("bg-rose-500")
    expect(
      channelBadgeDotClass({
        backendConnected: true,
        backendInitializing: false,
        channelError: "",
        counts: healthy,
      }),
    ).toBe("bg-[color:var(--success)]")
  })
})
