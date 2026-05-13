import { describe, expect, it } from "bun:test"
import { parseSseChunks } from "./stream"

describe("parseSseChunks", () => {
  it("parses complete sse events and keeps pending", () => {
    const input =
      'event: run_started\ndata: {"text":"ok"}\n\nevent: run_finished\ndata: {"success":true}\n\nevent: run_started'
    const parsed = parseSseChunks(input)
    expect(parsed.events).toEqual([
      { event: "run_started", data: { text: "ok" } },
      { event: "run_finished", data: { success: true } },
    ])
    expect(parsed.pending).toBe("event: run_started")
  })

  it("parses run_error and run_finished in one buffer (same read chunk)", () => {
    const input =
      'event: run_error\ndata: {"message":"bad"}\n\nevent: run_finished\ndata: {"success":false}\n\n'
    const parsed = parseSseChunks(input)
    expect(parsed.events).toEqual([
      { event: "run_error", data: { message: "bad" } },
      { event: "run_finished", data: { success: false } },
    ])
  })

  it("parses error and done from early chat exit", () => {
    const input = 'event: error\ndata: {"text":"no actor"}\n\nevent: done\ndata: {}\n\n'
    const parsed = parseSseChunks(input)
    expect(parsed.events).toEqual([
      { event: "error", data: { text: "no actor" } },
      { event: "done", data: {} },
    ])
  })

  it("drops malformed json events while preserving later valid events", () => {
    const input =
      'event: bad\ndata: {"unterminated"\n\nevent: done\ndata: {}\n\n'
    const parsed = parseSseChunks(input)

    expect(parsed.events).toEqual([{ event: "done", data: {} }])
    expect(parsed.pending).toBe("")
  })
})
