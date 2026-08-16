import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import {
  commentCommunityForumPost,
  communityForumAttachmentUrl,
  createCommunityForumPost,
  deleteCommunityForumComment,
  deleteCommunityForumPost,
  getCommunityForum,
  moderateCommunityForumPost,
  reportCommunityForumPost,
  toggleCommunityForumLike,
} from "@/lib/api";
import { ResearchFeed, ResearchFeedItem } from "@/components/research/research-feed";
import { shortLocalTimestamp } from "@/components/research/research-panel";
import type { CommunityForumAttachment, CommunityForumPost } from "@/lib/types";

import "./community-forum.css";

const REPORT_REASONS = ["疑似虚假信息", "广告或骚扰", "人身攻击", "侵权内容"];

function formatLocal(value: string) {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(value));
}

/** `created_at` is a UTC instant, so it is localised before being shortened. */
function postTime(value: string) {
  return shortLocalTimestamp(formatLocal(value));
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

/** Moderation state, as one label plus the item's single left accent. */
function moderationState(status: string) {
  if (status === "pending_review") return { label: "待管理员复核", accent: "yellow" };
  if (status === "hidden") return { label: "已隐藏", accent: "orange" };
  if (status === "deleted") return { label: "已删除", accent: "red" };
  return undefined;
}

/**
 * The composer asks for a title and for the post; most members answer both with
 * the same sentence. Printing it twice — once as a heading, once as the opening
 * line — is the restatement this feed exists to drop, so the title survives only
 * when it actually says something the first line does not.
 */
function leadTitle(post: CommunityForumPost) {
  const title = post.title.trim();
  if (!title) return "";
  const firstLine = post.body.trim().split("\n")[0]?.trim() ?? "";
  const strip = (value: string) => value.replace(/[\s，。、,.:：;；!！?？~—－-]/g, "");
  const a = strip(title);
  const b = strip(firstLine);
  if (!a || !b) return title;
  return a.includes(b) || b.includes(a) ? "" : title;
}

/** Author-typed index terms. One quiet line, not a dozen equal-weight chips. */
function indexTerms(post: CommunityForumPost) {
  return [...post.tickers.map((ticker) => `$${ticker}`), ...post.topics.map((topic) => `#${topic}`)]
    .join(" · ");
}

const isImage = (attachment: CommunityForumAttachment) =>
  attachment.content_type.startsWith("image/");

export function CommunityForum(props: { query: string }) {
  const [posts, setPosts] = createSignal<CommunityForumPost[]>([]);
  const [isAdmin, setIsAdmin] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [composerOpen, setComposerOpen] = createSignal(false);
  const [title, setTitle] = createSignal("");
  const [body, setBody] = createSignal("");
  const [tickers, setTickers] = createSignal("");
  const [topics, setTopics] = createSignal("");
  const [sourceUrl, setSourceUrl] = createSignal("");
  const [attachment, setAttachment] = createSignal<File | null>(null);
  const [working, setWorking] = createSignal("");
  const [commentDrafts, setCommentDrafts] = createSignal<Record<string, string>>({});
  const [commentOpen, setCommentOpen] = createSignal<string | null>(null);
  const [reportOpen, setReportOpen] = createSignal<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = createSignal<string | null>(null);
  let controller: AbortController | undefined;
  let fileInput: HTMLInputElement | undefined;

  const visiblePosts = createMemo(() => {
    const query = props.query.trim().toLowerCase();
    if (!query) return posts();
    return posts().filter((post) =>
      `${post.title} ${post.body} ${post.tickers.join(" ")} ${post.topics.join(" ")} ${post.author_label}`
        .toLowerCase()
        .includes(query),
    );
  });

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      const page = await getCommunityForum(controller.signal);
      setPosts(page.items);
      setIsAdmin(page.is_admin);
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "讨论区暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  const mergePost = (post: CommunityForumPost) => {
    setPosts((current) => {
      if (post.moderation_status !== "visible" && !post.can_delete && !isAdmin()) {
        return current.filter((item) => item.id !== post.id);
      }
      const found = current.some((item) => item.id === post.id);
      return found
        ? current.map((item) => (item.id === post.id ? post : item))
        : [post, ...current];
    });
  };

  const runPostAction = async (
    id: string,
    label: string,
    action: () => Promise<CommunityForumPost>,
  ) => {
    if (working()) return;
    setWorking(`${id}:${label}`);
    setError("");
    try {
      mergePost(await action());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "操作失败，请稍后重试");
    } finally {
      setWorking("");
    }
  };

  const publish = async (event: SubmitEvent) => {
    event.preventDefault();
    if (working()) return;
    setWorking("publish");
    setError("");
    try {
      const post = await createCommunityForumPost({
        title: title(),
        body: body(),
        tickers: tickers(),
        topics: topics(),
        sourceUrl: sourceUrl(),
        attachment: attachment(),
      });
      mergePost(post);
      setTitle("");
      setBody("");
      setTickers("");
      setTopics("");
      setSourceUrl("");
      setAttachment(null);
      if (fileInput) fileInput.value = "";
      setComposerOpen(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "发布失败，请稍后重试");
    } finally {
      setWorking("");
    }
  };

  const submitComment = (post: CommunityForumPost) => {
    const value = commentDrafts()[post.id]?.trim() ?? "";
    if (!value) return;
    void runPostAction(post.id, "comment", () => commentCommunityForumPost(post.id, value)).then(
      () => setCommentDrafts((current) => ({ ...current, [post.id]: "" })),
    );
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  return (
    <section class="community-forum" aria-label="HONE 社区讨论区">
      <div class="community-forum-intro">
        <div>
          <span>成员讨论区</span>
          <h2>投资讨论区</h2>
          <p>分享观点和证据，但帖子不是 HONE 结论，也不会自动进入投资助手、评级或每日产品。</p>
        </div>
        <button type="button" class="community-forum-compose-button" onClick={() => setComposerOpen(!composerOpen())}>
          {composerOpen() ? "收起" : "＋ 发起讨论"}
        </button>
      </div>

      <Show when={composerOpen()}>
        <form class="community-forum-composer" onSubmit={publish}>
          <label>标题<input required minLength={4} maxLength={80} value={title()} onInput={(event) => setTitle(event.currentTarget.value)} placeholder="一句话说清你想讨论的问题" /></label>
          <label>观点与依据<textarea required minLength={10} maxLength={5000} rows={6} value={body()} onInput={(event) => setBody(event.currentTarget.value)} placeholder="写出事实、判断、反方可能性和下一验证点。请勿发布个人隐私或交易指令。" /></label>
          <div class="community-forum-composer-grid">
            <label>股票代码<input value={tickers()} onInput={(event) => setTickers(event.currentTarget.value)} placeholder="NVDA, MU, SNDK" /></label>
            <label>主题<input value={topics()} onInput={(event) => setTopics(event.currentTarget.value)} placeholder="HBM, 数据中心" /></label>
          </div>
          <label>原始来源（可选）<input type="url" value={sourceUrl()} onInput={(event) => setSourceUrl(event.currentTarget.value)} placeholder="https://公司官网或原文" /></label>
          <label class="community-forum-file">附件（可选，最多 10 MB）<input ref={fileInput} type="file" accept=".pdf,.md,.markdown,.txt,.png,.jpg,.jpeg,.webp,application/pdf,text/plain,text/markdown,image/png,image/jpeg,image/webp" onChange={(event) => setAttachment(event.currentTarget.files?.[0] ?? null)} /><small>PDF、Markdown、纯文本或安全图片；论坛附件仍是不可信用户内容。</small></label>
          <div class="community-forum-composer-actions"><button type="button" onClick={() => setComposerOpen(false)}>取消</button><button type="submit" disabled={working() === "publish"}>{working() === "publish" ? "发布中…" : "发布讨论"}</button></div>
        </form>
      </Show>

      <Show when={error()}><p class="community-forum-error" role="alert">{error()}</p></Show>
      <Show when={loading()}><div class="public-workspace-state" role="status">正在读取讨论…</div></Show>
      <Show when={!loading() && visiblePosts().length === 0}><div class="public-workspace-state">还没有匹配的讨论。可以发布第一篇，但请尽量附原始来源。</div></Show>

      <ResearchFeed>
        <For each={visiblePosts()}>{(post) => (
          <ResearchFeedItem
            author={post.author_label}
            time={postTime(post.created_at)}
            // Reach, moderation state and — for admins — the report tally are
            // facts about the post, so they read once here instead of being
            // stamped onto the action buttons a second time.
            meta={[
              moderationState(post.moderation_status)?.label,
              post.like_count ? `${post.like_count} 赞` : undefined,
              post.comments.length ? `${post.comments.length} 评论` : undefined,
              isAdmin() && post.report_count ? `${post.report_count} 举报` : undefined,
            ].filter(Boolean) as string[]}
            accent={moderationState(post.moderation_status)?.accent}
            // Only picture attachments are pictures; a PDF is a link.
            media={
              post.attachment && isImage(post.attachment)
                ? [communityForumAttachmentUrl(post.id, post.attachment.id)]
                : undefined
            }
            mediaAlt={`${post.author_label} 上传的附件图片`}
            links={[
              ...(post.source_url ? [{ href: post.source_url, label: "查看原始来源" }] : []),
              ...(post.attachment
                ? [
                    {
                      href: communityForumAttachmentUrl(post.id, post.attachment.id),
                      label: `${post.attachment.filename} · ${formatBytes(post.attachment.byte_size)} · 用户上传，未经 HONE 核验`,
                    },
                  ]
                : []),
            ]}
            footer={
              <>
                <div class="community-forum-actions">
                  <button type="button" classList={{ active: post.liked_by_me }} disabled={!!working()} onClick={() => void runPostAction(post.id, "like", () => toggleCommunityForumLike(post.id))}>{post.liked_by_me ? "♥ 已赞" : "♡ 赞"}</button>
                  <button type="button" onClick={() => setCommentOpen(commentOpen() === post.id ? null : post.id)}>{commentOpen() === post.id ? "收起评论" : "评论"}</button>
                  <Show when={!post.can_delete && post.moderation_status === "visible"}><button type="button" onClick={() => setReportOpen(reportOpen() === post.id ? null : post.id)}>举报</button></Show>
                  <Show when={post.can_delete}>
                    <Show
                      when={deleteConfirm() === post.id}
                      fallback={<button type="button" onClick={() => setDeleteConfirm(post.id)}>删除</button>}
                    >
                      <span class="community-forum-delete-confirm" role="group" aria-label="确认删除这篇帖子">
                        <button
                          type="button"
                          class="is-danger"
                          disabled={!!working()}
                          onClick={() => {
                            setDeleteConfirm(null);
                            void runPostAction(post.id, "delete", () => deleteCommunityForumPost(post.id));
                          }}
                        >
                          确认删除
                        </button>
                        <button type="button" onClick={() => setDeleteConfirm(null)}>取消</button>
                      </span>
                    </Show>
                  </Show>
                  <Show when={isAdmin()}><button type="button" onClick={() => void runPostAction(post.id, "moderate", () => moderateCommunityForumPost(post.id, post.moderation_status === "visible" ? "hide" : "restore"))}>{post.moderation_status === "visible" ? "管理员隐藏" : "管理员恢复"}</button></Show>
                </div>
                <Show when={reportOpen() === post.id}><div class="community-forum-report"><span>请选择举报原因</span><For each={REPORT_REASONS}>{(reason) => <button type="button" onClick={() => { setReportOpen(null); void runPostAction(post.id, "report", () => reportCommunityForumPost(post.id, reason)); }}>{reason}</button>}</For></div></Show>
                <Show when={commentOpen() === post.id}>
                  <div class="community-forum-comments">
                    <For each={post.comments}>{(comment) => <article data-status={comment.moderation_status}><header><strong>{comment.author_label}</strong><time>{postTime(comment.created_at)}</time><Show when={comment.can_delete}><button type="button" onClick={() => void runPostAction(post.id, "delete-comment", () => deleteCommunityForumComment(post.id, comment.id))}>删除</button></Show></header><p>{comment.moderation_status === "deleted" ? "该评论已删除" : comment.body}</p></article>}</For>
                    <div class="community-forum-comment-box"><textarea aria-label={`评论 ${post.title}`} rows={2} maxLength={1000} value={commentDrafts()[post.id] ?? ""} onInput={(event) => setCommentDrafts((current) => ({ ...current, [post.id]: event.currentTarget.value }))} placeholder="回应观点，尽量说明依据…" /><button type="button" disabled={!commentDrafts()[post.id]?.trim() || !!working()} onClick={() => submitComment(post)}>发送评论</button></div>
                  </div>
                </Show>
              </>
            }
          >
            {/* The post, expanded, with its own line breaks. It used to sit
                under a heading that repeated its first sentence. */}
            <Show when={leadTitle(post)}>
              <p class="community-forum-lead">{leadTitle(post)}</p>
            </Show>
            <p>{post.body}</p>
            <Show when={indexTerms(post)}>
              <p class="community-forum-terms">{indexTerms(post)}</p>
            </Show>
          </ResearchFeedItem>
        )}</For>
      </ResearchFeed>

      <p class="community-forum-boundary">社区讨论不构成投资建议。需要进入 HONE 官方研究资料库的内容，请在<a href="/research-library">“我的知识源”</a>提交，并等待管理员核验与采纳。</p>
    </section>
  );
}
