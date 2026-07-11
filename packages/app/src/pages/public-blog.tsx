import { For } from "solid-js"
import { Title } from "@solidjs/meta"
import { useNavigate } from "@solidjs/router"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import { CONTENT } from "@/lib/public-content"
import { publicBlogPosts } from "@/lib/public-blog"
import { formatDate, useLocale } from "@/lib/i18n"
import "./public-site.css"

export default function PublicBlogPage() {
  const navigate = useNavigate()
  const posts = () => publicBlogPosts()

  const openPost = (slug: string) => {
    navigate(`/blog/${slug}`)
    window.scrollTo({ top: 0, left: 0, behavior: "auto" })
  }

  return (
    <div class="public-blog-page">
      <Title>HONE Blog</Title>
      <PublicNav />

      <main class="public-blog-main">
        <section class="public-blog-hero">
          <div class="public-blog-kicker">HONE BLOG</div>
          <h1>{useLocale() === "zh" ? "工程、产品与投研实践" : "Engineering, product, and research practice"}</h1>
          <p>
            {useLocale() === "zh"
              ? "记录 HONE 在开源 AI Agent、投研工作流和 Rust 工程实践中的真实选择。"
              : "Notes from building HONE across open-source AI agents, investment research workflows, and Rust engineering."}
          </p>
        </section>

        <section class="public-blog-list" aria-label="Blog posts">
          <For each={posts()}>
            {(post) => (
              <article class="public-blog-card" onClick={() => openPost(post.slug)}>
                <div class="public-blog-card-media">
                  <img src={post.heroImage} alt={post.title} loading="lazy" />
                </div>
                <div class="public-blog-card-body">
                  <div class="public-blog-meta">
                    <span>{post.category}</span>
                    <span>{formatDate(post.date, { month: "short", day: "numeric", year: "numeric" })}</span>
                    <span>{post.readTime}</span>
                  </div>
                  <h2>{post.title}</h2>
                  <p>{post.excerpt}</p>
                  <button type="button">
                    {CONTENT.home_page.blog_cta}
                    <span aria-hidden="true">→</span>
                  </button>
                </div>
              </article>
            )}
          </For>
        </section>
      </main>

      <PublicFooter />
    </div>
  )
}
