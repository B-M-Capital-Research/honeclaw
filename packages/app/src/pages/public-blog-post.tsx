import { For, Show } from "solid-js"
import { Title } from "@solidjs/meta"
import { Navigate, useNavigate, useParams } from "@solidjs/router"
import { Markdown } from "@hone-financial/ui/markdown"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import {
  findPublicBlogPost,
  publicBlogPosts,
  type PublicBlogPost,
} from "@/lib/public-blog"
import { CONTENT } from "@/lib/public-content"
import { formatDate, useLocale } from "@/lib/i18n"
import "./public-site.css"

function ArticleBody(props: { post: PublicBlogPost }) {
  const navigate = useNavigate()
  const related = () =>
    publicBlogPosts().filter((post) => post.slug !== props.post.slug).slice(0, 2)

  return (
    <>
      <Title>{props.post.title} | Hone Blog</Title>
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
