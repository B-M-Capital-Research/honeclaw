import { describe, expect, it } from "bun:test"
import { parseMessageContent } from "./messages"

describe("parseMessageContent", () => {
  it("extracts local image links", () => {
    const parts = parseMessageContent("before file:///tmp/a.png after")
    expect(parts.map((item) => item.type)).toEqual(["text", "image", "text"])
  })
})
