import { beforeEach, describe, expect, it } from "bun:test"
import { setBackendRuntime } from "./backend"
import { parseMessageContent } from "./messages"

describe("parseMessageContent", () => {
  beforeEach(() => {
    setBackendRuntime({
      mode: "browser",
      baseUrl: "",
      bearerToken: "",
      meta: undefined,
      isDesktop: false,
    })
  })

  it("extracts local image links", () => {
    const parts = parseMessageContent("before file:///tmp/a.png after")
    expect(parts.map((item) => item.type)).toEqual(["text", "image", "text"])
  })
})
