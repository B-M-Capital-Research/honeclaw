import { useLocale, type Locale } from "./i18n"
import whyHoneUsesRustEn from "@/content/blog/why-hone-uses-rust.en.md?raw"
import whyHoneUsesRustZh from "@/content/blog/why-hone-uses-rust.zh.md?raw"

export type PublicBlogPost = {
  slug: string
  title: string
  excerpt: string
  date: string
  category: string
  readTime: string
  heroImage: string
  markdown: string
}

const POSTS: Record<Locale, PublicBlogPost[]> = {
  zh: [
    {
      slug: "why-hone-uses-rust",
      title: "Hone 为什么采用 Rust，以及推荐大家都开始使用 Rust",
      excerpt:
        "从 Python + Node.js 到 Rust 的完整重构复盘：为什么 Rust 更适合 AI Coding 时代的上下文治理、稳定性和多端工程。",
      date: "2026-03-10",
      category: "工程实践",
      readTime: "约 12 分钟",
      heroImage: "/blog/why-hone-uses-rust-zh.png",
      markdown: whyHoneUsesRustZh,
    },
  ],
  en: [
    {
      slug: "why-hone-uses-rust",
      title: "Why Hone uses Rust, and why more teams should start using Rust",
      excerpt:
        "A field report from Hone's rewrite from Python + Node.js to Rust, and why Rust fits context management, stability, and multi-endpoint engineering in the AI Coding era.",
      date: "2026-03-10",
      category: "Engineering",
      readTime: "12 min read",
      heroImage: "/blog/why-hone-uses-rust-en.png",
      markdown: whyHoneUsesRustEn,
    },
  ],
}

export function publicBlogPosts(locale: Locale = useLocale()): PublicBlogPost[] {
  return POSTS[locale]
}

export function findPublicBlogPost(
  slug: string | undefined,
  locale: Locale = useLocale(),
): PublicBlogPost | undefined {
  if (!slug) return undefined
  return POSTS[locale].find((post) => post.slug === slug)
}

export function latestPublicBlogPost(locale: Locale = useLocale()): PublicBlogPost {
  return POSTS[locale][0]!
}

export const __PUBLIC_BLOG_POSTS__ = POSTS
