import { beforeEach, describe, expect, it } from "bun:test"
import { setBackendRuntime } from "./backend"
import { parseMessageContent } from "./messages"
import localImageMarkerFixtures from "../../../../tests/fixtures/local_image_markers.json"

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

  it("preserves interleaved text and multiple local images", () => {
    const parts = parseMessageContent(
      "alpha\nfile:///tmp/a.png\nbeta file:///tmp/b.webp gamma",
    )
    expect(parts.map((item) => item.type)).toEqual([
      "text",
      "image",
      "text",
      "image",
      "text",
    ])
  })

  it("extracts local image links wrapped in html anchors", () => {
    const parts = parseMessageContent(
      'before <a href="file:///tmp/a.png">file:///tmp/a.png</a> after',
    )
    expect(parts.map((item) => item.type)).toEqual(["text", "image", "text"])
  })

  it("extracts local image links wrapped in markdown links", () => {
    const parts = parseMessageContent("before [图表](file:///tmp/a.png) after")
    expect(parts.map((item) => item.type)).toEqual(["text", "image", "text"])
  })

  it("matches the shared local image marker fixture", () => {
    for (const fixture of localImageMarkerFixtures) {
      const parts = parseMessageContent(fixture.input)
      expect(parts.map((item) => item.type)).toEqual(
        fixture.part_types as Array<"text" | "image">,
      )
    }
  })
})
