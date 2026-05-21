import { describe, expect, it } from "bun:test"

import {
  __PUBLIC_BLOG_POSTS__,
  findPublicBlogPost,
  latestPublicBlogPost,
  publicBlogPosts,
} from "./public-blog"

describe("public blog content", () => {
  it("keeps the same slugs in zh and en", () => {
    expect(__PUBLIC_BLOG_POSTS__.en.map((post) => post.slug)).toEqual(
      __PUBLIC_BLOG_POSTS__.zh.map((post) => post.slug),
    )
  })

  it("uses locale-specific hero images for the Rust article", () => {
    expect(findPublicBlogPost("why-hone-uses-rust", "zh")?.heroImage).toBe(
      "/blog/why-hone-uses-rust-zh.png",
    )
    expect(findPublicBlogPost("why-hone-uses-rust", "en")?.heroImage).toBe(
      "/blog/why-hone-uses-rust-en.png",
    )
  })

  it("looks up posts by slug and returns undefined for missing slugs", () => {
    expect(findPublicBlogPost("why-hone-uses-rust", "en")?.title).toContain(
      "Rust",
    )
    expect(findPublicBlogPost("missing", "zh")).toBeUndefined()
  })

  it("returns the latest post from the active list", () => {
    expect(latestPublicBlogPost("zh")).toBe(publicBlogPosts("zh")[0])
  })
})
