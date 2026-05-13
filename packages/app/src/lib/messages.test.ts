import { beforeEach, describe, expect, it } from "bun:test"
import { setBackendRuntime } from "./backend"
import { parseMessageContent } from "./messages"
import localImageMarkerFixtures from "../../../../tests/fixtures/local_image_markers.json"

function parsedTypes(input: string): Array<"text" | "image"> {
  return parseMessageContent(input).map((item) => item.type)
}

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
    expect(parsedTypes("before file:///tmp/a.png after")).toEqual([
      "text",
      "image",
      "text",
    ])
  })

  it("preserves interleaved text and multiple local images", () => {
    expect(
      parsedTypes("alpha\nfile:///tmp/a.png\nbeta file:///tmp/b.webp gamma"),
    ).toEqual([
      "text",
      "image",
      "text",
      "image",
      "text",
    ])
  })

  it("extracts local image links wrapped in html anchors", () => {
    expect(
      parsedTypes(
        'before <a href="file:///tmp/a.png">file:///tmp/a.png</a> after',
      ),
    ).toEqual(["text", "image", "text"])
  })

  it("extracts local image links wrapped in markdown links", () => {
    expect(parsedTypes("before [图表](file:///tmp/a.png) after")).toEqual([
      "text",
      "image",
      "text",
    ])
  })

  it("matches the shared local image marker fixture", () => {
    for (const fixture of localImageMarkerFixtures) {
      expect(parsedTypes(fixture.input)).toEqual(
        fixture.part_types as ReturnType<typeof parsedTypes>,
      )
    }
  })
})
