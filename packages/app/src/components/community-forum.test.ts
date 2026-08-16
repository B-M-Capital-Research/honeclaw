import { describe, expect, it } from "bun:test";

const component = await Bun.file(new URL("./community-forum.tsx", import.meta.url)).text();
const css = await Bun.file(new URL("./community-forum.css", import.meta.url)).text();
const feedStyles = await Bun.file(
  new URL("./research/research-feed.css", import.meta.url),
).text();

describe("community forum contract", () => {
  it("supports the member discussion loop", () => {
    for (const label of ["发起讨论", "发布讨论", "评论", "赞", "举报", "删除"]) {
      expect(component).toContain(label);
    }
  });

  it("keeps forum material outside official research", () => {
    expect(component).toContain("不会自动进入投资助手、评级或每日产品");
    expect(component).toContain("等待管理员核验与采纳");
    expect(component).toContain("未经 HONE 核验");
  });

  it("limits attachments and exposes moderation", () => {
    expect(component).toContain("最多 10 MB");
    expect(component).toContain("管理员隐藏");
    expect(component).toContain("pending_review");
  });

  it("reads as a feed: the post is the content, never a heading plus a card", () => {
    expect(component).toContain("<ResearchFeed>");
    expect(component).toContain("<ResearchFeedItem");
    // Author, time and reach carry the item; the post is the body.
    expect(component).toContain("author={post.author_label}");
    expect(component).toContain("time={postTime(post.created_at)}");
    expect(component).toContain("<p>{post.body}</p>");
    // No card chrome: no avatar block, no <h3> heading, no bespoke post class.
    expect(component).not.toContain("community-forum-avatar");
    expect(component).not.toContain("<h3>");
    expect(component).not.toContain("community-forum-list");
    expect(component).not.toContain('class="community-forum-post"');
    expect(css).not.toContain(".community-forum-post");
    expect(css).not.toContain(".community-forum-avatar");
    expect(css).not.toContain(".community-forum-body");
    expect(css).not.toContain(".community-forum-list");
    // The body is a paragraph in the feed item, not a disclosure.
    const bodyIndex = component.indexOf("<p>{post.body}</p>");
    expect(bodyIndex).toBeGreaterThan(-1);
    expect(component.slice(0, bodyIndex)).not.toContain("<details");
    // Line breaks the member typed survive.
    expect(feedStyles).toContain("white-space: pre-wrap");
  });

  it("drops the title when it only restates the post's first line", () => {
    expect(component).toContain("function leadTitle");
    expect(component).toContain("post.body.trim().split(\"\\n\")[0]");
    expect(component).toContain("a.includes(b) || b.includes(a)");
  });

  it("keeps counts and moderation state as facts, not as button decoration", () => {
    // Likes / comments / report tally read once in the item's meta line.
    expect(component).toContain("`${post.like_count} 赞`");
    expect(component).toContain("`${post.comments.length} 评论`");
    expect(component).toContain("`${post.report_count} 举报`");
    // …so the buttons stay verbs, not counters.
    expect(component).not.toContain("♡ {post.like_count");
    expect(component).not.toContain("评论 {post.comments.length");
    // One left accent per item, driven by moderation state.
    expect(component).toContain("function moderationState");
    expect(component).toContain("accent={moderationState(post.moderation_status)?.accent}");
    expect(component).not.toContain("data-status={post.moderation_status}");
  });

  it("shows only picture attachments as pictures and links the rest", () => {
    expect(component).toContain("attachment.content_type.startsWith(\"image/\")");
    expect(component).toContain("isImage(post.attachment)");
    expect(component).toContain("communityForumAttachmentUrl(post.id, post.attachment.id)");
    // Author-typed index terms are one quiet line, not a chip cluster.
    expect(component).toContain("function indexTerms");
    expect(component).not.toContain("community-forum-tags");
    expect(css).not.toContain(".community-forum-tags");
  });

  it("keeps interaction under the post instead of folding it into analysis", () => {
    // Actions are interaction, not commentary: they use the feed item's footer
    // slot, so they land under the pictures and the source links.
    expect(component).toContain("footer={");
    expect(component).toContain("community-forum-actions");
    expect(component).toContain("community-forum-comments");
    expect(component).not.toContain("analysis={");
  });

  it("confirms post deletion inline instead of a native dialog", () => {
    expect(component).not.toContain("window.confirm");
    expect(component).toContain("确认删除");
    expect(component).toContain("取消");
    expect(component).toContain("community-forum-delete-confirm");
    expect(css).toContain("var(--hone-error-600)");
  });

  it("uses hone tokens for warning and error accents", () => {
    expect(css).toContain("var(--hone-signal-yellow)");
    expect(css).toContain("var(--hone-coral-600)");
    expect(css).not.toContain("#d19a24");
    expect(css).not.toContain("#b67d09");
  });

  it("rides the theme tokens on mobile and in dark mode", () => {
    expect(css).toContain("@media (max-width: 768px)");
    expect(css).not.toContain("@media(max-width:768px)");
    expect(css).not.toContain("!important");
    // `--public-*` was never defined anywhere, so those rules silently dropped
    // their borders and surfaces. Every colour now comes from a real token…
    expect(css).not.toContain("--public-");
    // …which means dark mode needs no bespoke override block.
    expect(css).not.toContain('[data-theme="dark"]');
  });
});
