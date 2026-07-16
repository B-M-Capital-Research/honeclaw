import { describe, expect, it } from "bun:test"
import { parseSseChunks } from "./stream"

describe("parseSseChunks", () => {
  it("parses an assistant reset between streamed answer phases", () => {
    const parsed = parseSseChunks(
      'event: assistant_delta\ndata: {"content":"checking"}\n\nevent: assistant_reset\ndata: {}\n\n',
    );
    expect(parsed.events).toEqual([
      { event: "assistant_delta", data: { content: "checking" } },
      { event: "assistant_reset", data: {} },
    ]);
  });

  it("parses complete sse events and keeps pending", () => {
    const sseChunk =
      'event: run_started\ndata: {"text":"ok"}\n\nevent: run_finished\ndata: {"success":true}\n\nevent: run_started'
    const parsed = parseSseChunks(sseChunk)
    expect(parsed.events).toEqual([
      { event: "run_started", data: { text: "ok" } },
      { event: "run_finished", data: { success: true } },
    ])
    expect(parsed.pending).toBe("event: run_started")
  })

  it("preserves server-authoritative run timing and progress fields", () => {
    const parsed = parseSseChunks(
      'event: run_started\ndata: {"run_id":"r1","started_at_ms":1000,"phase":"thinking","status_text":"正在识别证券实体","updated_at_ms":1000}\n\nevent: run_progress\ndata: {"run_id":"r1","started_at_ms":1000,"phase":"running","status_text":"正在核验行情","updated_at_ms":2000}\n\n',
    )

    expect(parsed.events).toEqual([
      {
        event: "run_started",
        data: {
          run_id: "r1",
          started_at_ms: 1000,
          phase: "thinking",
          status_text: "正在识别证券实体",
          updated_at_ms: 1000,
        },
      },
      {
        event: "run_progress",
        data: {
          run_id: "r1",
          started_at_ms: 1000,
          phase: "running",
          status_text: "正在核验行情",
          updated_at_ms: 2000,
        },
      },
    ])
  })

  it("parses the dedicated public tool status field", () => {
    const parsed = parseSseChunks(
      'event: tool_call\ndata: {"public_status_text":"正在核验实时行情"}\n\n',
    )

    expect(parsed.events).toEqual([
      {
        event: "tool_call",
        data: { public_status_text: "正在核验实时行情" },
      },
    ])
  })

  it("parses run_error and run_finished in one buffer (same read chunk)", () => {
    const sseChunk =
      'event: run_error\ndata: {"message":"bad"}\n\nevent: run_finished\ndata: {"success":false}\n\n'
    const parsed = parseSseChunks(sseChunk)
    expect(parsed.events).toEqual([
      { event: "run_error", data: { message: "bad" } },
      { event: "run_finished", data: { success: false } },
    ])
  })

  it("parses error and done from early chat exit", () => {
    const sseChunk =
      'event: error\ndata: {"text":"no actor"}\n\nevent: done\ndata: {}\n\n'
    const parsed = parseSseChunks(sseChunk)
    expect(parsed.events).toEqual([
      { event: "error", data: { text: "no actor" } },
      { event: "done", data: {} },
    ])
  })

  it("drops malformed json events while preserving later valid events", () => {
    const sseChunk =
      'event: bad\ndata: {"unterminated"\n\nevent: done\ndata: {}\n\n'
    const parsed = parseSseChunks(sseChunk)

    expect(parsed.events).toEqual([{ event: "done", data: {} }])
    expect(parsed.pending).toBe("")
  })
})
