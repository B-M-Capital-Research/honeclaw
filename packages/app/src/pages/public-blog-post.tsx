import { For, Show } from "solid-js"
import { Meta, Title } from "@solidjs/meta"
import { Navigate, useNavigate, useParams } from "@solidjs/router"
import { Markdown } from "@hone-financial/ui/markdown"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import {
  alternatePublicBlogPost,
  findPublicBlogPost,
  publicBlogPosts,
  type PublicBlogPost,
} from "@/lib/public-blog"
import { CONTENT } from "@/lib/public-content"
import { formatDate, setLocale, useLocale } from "@/lib/i18n"
import "./public-site.css"

const SITE_ORIGIN = "https://hone-claw.com"

function absoluteUrl(path: string): string {
  if (path.startsWith("http://") || path.startsWith("https://")) return path
  return `${SITE_ORIGIN}${path.startsWith("/") ? path : `/${path}`}`
}

function ArticleBody(props: { post: PublicBlogPost }) {
  const navigate = useNavigate()
  const alternatePost = () => alternatePublicBlogPost(props.post)
  const alternateLocale = () => (useLocale() === "zh" ? "en" : "zh")
  const related = () =>
    publicBlogPosts().filter((post) => post.slug !== props.post.slug).slice(0, 2)
  const canonicalUrl = () => `${SITE_ORIGIN}/blog/${props.post.slug}`
  const imageUrl = () => absoluteUrl(props.post.heroImage)

  const switchLanguage = () => {
    setLocale(alternateLocale())
    window.scrollTo({ top: 0, left: 0, behavior: "auto" })
  }

  return (
    <>
      <Title>{props.post.title} | Hone Blog</Title>
      <Meta name="description" content={props.post.excerpt} />
      <Meta property="og:type" content="article" />
      <Meta property="og:site_name" content="Hone" />
      <Meta property="og:title" content={props.post.title} />
      <Meta property="og:description" content={props.post.excerpt} />
      <Meta property="og:url" content={canonicalUrl()} />
      <Meta property="og:image" content={imageUrl()} />
      <Meta property="og:image:width" content="1491" />
      <Meta property="og:image:height" content="1055" />
      <Meta property="og:image:type" content="image/png" />
      <Meta name="twitter:card" content="summary_large_image" />
      <Meta name="twitter:title" content={props.post.title} />
      <Meta name="twitter:description" content={props.post.excerpt} />
      <Meta name="twitter:image" content={imageUrl()} />
      <PublicNav />
      <main class="public-blog-post-main">
        <button class="public-blog-back" onClick={() => navigate("/blog")}>
          ← {useLocale() === "zh" ? "返回 Blog" : "Back to Blog"}
        </button>

        <article class="public-blog-post">
          <header class="public-blog-post-header">
            <div class="public-blog-meta">
              <span>{props.post.category}</span>
              <span>{formatDate(props.post.date, { month: "long", day: "numeric", year: "numeric" })}</span>
              <span>{props.post.readTime}</span>
            </div>
            <h1>{props.post.title}</h1>
            <p>{props.post.excerpt}</p>
          </header>

          <button class="public-blog-language-card" onClick={switchLanguage}>
            <span>
              {useLocale() === "zh" ? "English version" : "中文版"}
            </span>
            <strong>{alternatePost().title}</strong>
            <em>{alternatePost().excerpt}</em>
          </button>

          <figure class="public-blog-post-hero">
            <img src={props.post.heroImage} alt={props.post.title} />
          </figure>

          <Markdown text={props.post.markdown} class="public-blog-markdown" />
        </article>

        <Show when={related().length > 0}>
          <section class="public-blog-keep-reading">
            <div class="public-blog-kicker">
              {useLocale() === "zh" ? "继续阅读" : "Keep reading"}
            </div>
            <For each={related()}>
              {(post) => (
                <button onClick={() => navigate(`/blog/${post.slug}`)}>
                  <span>{post.category}</span>
                  <strong>{post.title}</strong>
                </button>
              )}
            </For>
          </section>
        </Show>
      </main>
      <PublicFooter />
    </>
  )
}

export default function PublicBlogPostPage() {
  const params = useParams()
  const post = () => findPublicBlogPost(params.slug)

  return (
    <Show when={post()} fallback={<Navigate href="/blog" />}>
      {(value) => <ArticleBody post={value()} />}
    </Show>
  )
}
