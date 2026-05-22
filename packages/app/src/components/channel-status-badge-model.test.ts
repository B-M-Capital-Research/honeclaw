import { describe, expect, it } from "bun:test"

import {
  backendConnectionLabel,
  backendConnectionStatus,
  channelBadgeDotClass,
  channelSummaryText,
  frontendConnectionStatus,
  statusDotClass,
  statusLabel,
  statusTextClass,
  summarizeChannelStatuses,
} from "./channel-status-badge-model"
import type { ChannelStatusInfo } from "@/lib/types"

function channelStatusFixture(
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
        channelStatusFixture({ processes: [{ pid: 1, running: true }] }),
        channelStatusFixture({
          id: "telegram",
          enabled: true,
          running: false,
          processes: [
            { pid: 2, running: true },
            { pid: 3, running: false },
          ],
        }),
        channelStatusFixture({ id: "feishu", enabled: false, running: false }),
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
    const healthy = summarizeChannelStatuses([channelStatusFixture()])
    const failing = summarizeChannelStatuses([
      channelStatusFixture({ running: false, enabled: true }),
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

  it("derives summary text and backend connection labels", () => {
    const empty = summarizeChannelStatuses([])
    const healthy = summarizeChannelStatuses([
      channelStatusFixture(),
      channelStatusFixture({ id: "telegram" }),
    ])

    expect(
      backendConnectionLabel({ initializing: true, connected: false }),
    ).toBe("后端连接中")
    expect(
      backendConnectionLabel({ initializing: false, connected: true }),
    ).toBe("管理端后端正常连接中")
    expect(channelSummaryText(empty, "后端连接中")).toBe(
      "渠道加载中，后端连接中，管理端前端正常连接中",
    )
    expect(channelSummaryText(healthy, "管理端后端正常连接中")).toBe(
      "2 个渠道监听中，管理端后端正常连接中，管理端前端正常连接中",
    )
  })

  it("derives backend and frontend connection rows outside the component", () => {
    expect(
      backendConnectionStatus({
        initializing: true,
        connected: false,
        isRemote: false,
        baseUrl: "",
      }),
    ).toEqual({
      label: "管理端后端",
      detail: "正在建立连接…",
      status: "degraded",
    })
    expect(
      backendConnectionStatus({
        initializing: false,
        connected: true,
        isRemote: true,
        baseUrl: "http://localhost:8077",
        resolvedBaseUrl: "http://127.0.0.1:8077",
      }),
    ).toEqual({
      label: "管理端后端",
      detail: "remote · http://127.0.0.1:8077（管理端端口 8077）",
      status: "running",
    })
    expect(
      backendConnectionStatus({
        initializing: false,
        connected: false,
        error: "connection refused",
        isRemote: false,
        baseUrl: "",
      }),
    ).toEqual({
      label: "管理端后端",
      detail: "connection refused",
      status: "stopped",
    })
    expect(
      frontendConnectionStatus({
        isDesktop: false,
        origin: "http://localhost:8077",
      }),
    ).toEqual({
      label: "管理端前端",
      detail: "browser · http://localhost:8077（管理端页面）",
      status: "running",
    })
  })
})
