import { useRef, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  ArrowRight,
  BookmarkSimple,
  ChatCircle,
  DotsThree,
  ImageSquare,
  MagnifyingGlass,
  PaperPlaneTilt,
  Plus,
  ShareNetwork,
  Sparkle,
  TrendUp,
  X,
} from "@phosphor-icons/react";
import { insights } from "@/data";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";

const initialFeed = [
  {
    id: "hbm",
    time: "今天 08:30",
    text: "最近不少人问，HBM4 量产之后，GPU 公司的护城河会不会被削弱。我的判断相反：下一个阶段的竞争会从单颗芯片设计，转向计算、存储、先进封装和软件生态的系统协同。",
    tags: ["NVIDIA", "AMD", "HBM"],
    image: "/content/hbm-packaging.webp",
    imageAlt: "先进封装基板与 HBM 存储堆栈",
    article: insights[0],
    comments: 18,
  },
  {
    id: "cowos",
    time: "昨天 21:15",
    text: "先进封装的扩产速度，正在成为 AI 芯片交付的真实上限。\n\n只看 GPU 设计会低估供给约束。CoWoS 与 HBM 的协同扩产，决定了未来四个季度的兑现节奏。这也是为什么我们跟踪台积电时，要把封装产能和晶圆产能分开看。",
    tags: ["台积电", "先进封装"],
    comments: 11,
  },
  {
    id: "datacenter",
    time: "07月11日 14:20",
    text: "云厂商资本开支没有减速，但投资者接下来要区分两件事：新增 GPU 数量，和真正能投入生产的推理集群。网络、供电与液冷正在成为交付节奏的一部分。",
    tags: ["云计算", "AI 基础设施"],
    image: "/content/ai-datacenter.webp",
    imageAlt: "现代 AI 数据中心服务器机房",
    comments: 23,
  },
  {
    id: "amd",
    time: "07月10日 18:40",
    text: "AMD 的机会不在复制 NVIDIA，而在推理侧建立第二选择。\n\nMI350 的意义，是让客户在成本敏感的推理集群中拥有更强的议价空间。硬件的差距已经不是最大障碍，软件成熟度和客户迁移成本才是核心观察点。",
    tags: ["AMD", "AI 推理"],
    comments: 9,
  },
];

function PublishDialog({ open, onOpenChange, onPublish }) {
  const inputRef = useRef(null);
  const [text, setText] = useState("");
  const [preview, setPreview] = useState(null);
  const chooseImage = (event) => {
    const file = event.target.files?.[0];
    if (file) setPreview(URL.createObjectURL(file));
  };
  const publish = () => {
    if (!text.trim() && !preview) return;
    onPublish({
      id: `local-${Date.now()}`,
      time: "刚刚",
      text: text.trim() || "分享了一张研究图片。",
      tags: ["新发布"],
      image: preview,
      imageAlt: "老王刚刚上传的研究图片",
      comments: 0,
    });
    setText("");
    setPreview(null);
    onOpenChange(false);
    toast.success("内容已发布到洞察流");
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>发布洞察</DialogTitle>
          <DialogDescription>
            当前只有老王和授权编辑可以发布；未来可扩展为用户讨论社区。
          </DialogDescription>
        </DialogHeader>
        <div className="flex gap-3 py-2">
          <Avatar className="size-10 shrink-0">
            <AvatarFallback>王</AvatarFallback>
          </Avatar>
          <div className="min-w-0 flex-1">
            <Textarea
              value={text}
              onChange={(event) => setText(event.target.value)}
              placeholder="分享一条判断、研究进展或值得讨论的问题……"
              className="min-h-32 resize-none border-0 px-0 text-base shadow-none focus-visible:ring-0"
            />
            {preview && (
              <div className="relative mt-3 overflow-hidden rounded-xl border">
                <img
                  src={preview}
                  alt="待发布图片预览"
                  className="max-h-72 w-full object-cover"
                />
                <Button
                  variant="secondary"
                  size="icon"
                  className="absolute right-2 top-2"
                  onClick={() => setPreview(null)}
                  aria-label="移除图片"
                >
                  <X />
                </Button>
              </div>
            )}
            <input
              ref={inputRef}
              type="file"
              accept="image/*"
              className="hidden"
              onChange={chooseImage}
            />
            <div className="mt-3 flex items-center justify-between border-t pt-3">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => inputRef.current?.click()}
              >
                <ImageSquare />
                添加图片
              </Button>
              <span className="text-xs text-muted-foreground">
                最多 4 张 · JPG/PNG/WebP
              </span>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button disabled={!text.trim() && !preview} onClick={publish}>
            <PaperPlaneTilt />
            发布
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function DiscussionDialog({ open, onOpenChange }) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>讨论功能准备中</DialogTitle>
          <DialogDescription>
            当前洞察是老王单向发布。社区阶段将开放回复、引用讨论和用户发帖，并保留内容审核与投资风险提示。
          </DialogDescription>
        </DialogHeader>
        <div className="rounded-xl bg-muted p-4 text-sm leading-6">
          <b>本期已经预留：</b>
          讨论数量、帖子稳定链接、作者身份、图片与文章附件。后续开放社区时不需要推翻信息流结构。
        </div>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>知道了</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function PostActions({ post, bookmarked, onBookmark, onAsk, onDiscuss }) {
  return (
    <div className="mt-3 flex items-center justify-between text-muted-foreground">
      <Button
        variant="ghost"
        size="sm"
        className="gap-2 px-1.5"
        onClick={onDiscuss}
      >
        <ChatCircle /> {post.comments}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="gap-2 px-1.5"
        onClick={onAsk}
      >
        <Sparkle />问 Agent
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="px-1.5"
        aria-label="分享"
        onClick={() => toast.success("洞察链接已复制")}
      >
        <ShareNetwork />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="px-1.5"
        aria-label="收藏"
        onClick={onBookmark}
      >
        <BookmarkSimple weight={bookmarked ? "fill" : "regular"} />
      </Button>
    </div>
  );
}

function FeedPost({ post, bookmarked, onBookmark, onAsk, onDiscuss }) {
  return (
    <article className="bg-background px-4 py-4 transition-colors hover:bg-muted/20 sm:px-5 sm:py-5">
      <div className="flex gap-3">
        <Avatar className="size-10 shrink-0">
          <AvatarFallback>王</AvatarFallback>
        </Avatar>
        <div className="min-w-0 flex-1">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <div className="flex min-w-0 items-center gap-1.5 text-sm">
                <b>老王</b>
                <span className="truncate text-muted-foreground">
                  @HariWang
                </span>
                <span className="text-muted-foreground">· {post.time}</span>
              </div>
              <div className="mt-0.5 text-[11px] text-muted-foreground">
                作者原文 · 已编辑审核
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="-mr-2 -mt-2 shrink-0"
              aria-label="更多"
            >
              <DotsThree />
            </Button>
          </div>
          <p className="mt-3 whitespace-pre-line text-[15px] leading-7">
            {post.text}
          </p>
          {post.image && (
            <img
              src={post.image}
              alt={post.imageAlt}
              className="mt-3 max-h-[420px] w-full rounded-xl border object-cover"
            />
          )}
          <div className="mt-3 flex flex-wrap gap-1.5">
            {post.tags.map((tag) => (
              <Badge key={tag} variant="secondary" className="font-normal">
                {tag}
              </Badge>
            ))}
          </div>
          {post.article && (
            <Link
              to={`/app/insights/${post.article.slug}`}
              className="mt-3 block overflow-hidden rounded-xl border transition-colors hover:bg-muted/40"
            >
              <div className="p-4">
                <div className="text-[11px] text-muted-foreground">
                  附带深度文章 · {post.article.reading}
                </div>
                <h2 className="mt-2 text-lg font-semibold leading-7">
                  {post.article.title}
                </h2>
                <p className="mt-1 line-clamp-2 text-sm leading-6 text-muted-foreground">
                  {post.article.excerpt}
                </p>
                <div className="mt-3 flex items-center gap-1 text-sm font-medium">
                  阅读全文 <ArrowRight />
                </div>
              </div>
            </Link>
          )}
          <PostActions
            post={post}
            bookmarked={bookmarked}
            onBookmark={onBookmark}
            onAsk={onAsk}
            onDiscuss={onDiscuss}
          />
        </div>
      </div>
    </article>
  );
}

export function InsightsPage() {
  const navigate = useNavigate();
  const [feed, setFeed] = useState(initialFeed);
  const [bookmarks, setBookmarks] = useState(["hbm"]);
  const [publishOpen, setPublishOpen] = useState(false);
  const [discussionOpen, setDiscussionOpen] = useState(false);
  const toggleBookmark = (id) =>
    setBookmarks((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
    );
  return (
    <div className="mx-auto max-w-[1160px] px-0 py-0 sm:px-6 sm:py-6 lg:py-7">
      <div className="grid gap-6 lg:grid-cols-[minmax(0,680px)_320px] lg:justify-center">
        <section className="min-w-0">
          <div className="border-b px-4 py-4 sm:px-0 sm:pb-5 sm:pt-0">
            <p className="text-sm text-muted-foreground">老王洞察</p>
            <div className="mt-1 flex items-center justify-between gap-4">
              <h1 className="text-3xl font-semibold tracking-tight">动态</h1>
              <Button size="sm" onClick={() => setPublishOpen(true)}>
                <Plus />
                发布
              </Button>
            </div>
            <p className="mt-2 text-sm text-muted-foreground">
              观点、图片与深度研究，都在同一条消息流。
            </p>
            <div className="relative mt-4 lg:hidden">
              <MagnifyingGlass className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
              <Input className="pl-9" placeholder="搜索公司、主题或观点" />
            </div>
          </div>
          <div className="divide-y sm:border-x sm:border-b">
            {feed.map((post) => (
              <FeedPost
                key={post.id}
                post={post}
                bookmarked={bookmarks.includes(post.id)}
                onBookmark={() => toggleBookmark(post.id)}
                onAsk={() => navigate(`/app/agent?insight=${post.id}`)}
                onDiscuss={() => setDiscussionOpen(true)}
              />
            ))}
          </div>
        </section>
        <aside className="hidden space-y-4 lg:block">
          <div className="sticky top-20 space-y-4">
            <div className="relative">
              <MagnifyingGlass className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
              <Input className="pl-9" placeholder="搜索公司、主题或观点" />
            </div>
            <Card className="shadow-none">
              <CardHeader>
                <CardTitle className="text-sm">正在跟踪的主题</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {[
                  ["AI 推理基础设施", "26 条"],
                  ["HBM 与先进封装", "18 条"],
                  ["云厂商资本开支", "14 条"],
                  ["AI 应用商业化", "9 条"],
                ].map(([name, count], index) => (
                  <button
                    key={name}
                    className="flex w-full items-start gap-3 text-left"
                  >
                    <span className="mt-0.5 grid size-7 place-items-center rounded-full bg-muted text-xs font-semibold">
                      {index + 1}
                    </span>
                    <span className="flex-1">
                      <span className="block text-sm font-medium">{name}</span>
                      <span className="text-xs text-muted-foreground">
                        {count}
                      </span>
                    </span>
                    <TrendUp size={16} />
                  </button>
                ))}
              </CardContent>
            </Card>
            <Card className="shadow-none">
              <CardHeader className="flex-row items-center justify-between space-y-0">
                <CardTitle className="text-sm">按公司回看</CardTitle>
                <Button variant="ghost" size="sm">
                  全部
                </Button>
              </CardHeader>
              <CardContent className="divide-y p-0">
                {[
                  ["NVIDIA", "18 条", "NVDA"],
                  ["AMD", "12 条", "AMD"],
                  ["台积电", "15 条", "TSM"],
                  ["Microsoft", "9 条", "MSFT"],
                ].map(([name, count, ticker]) => (
                  <button
                    key={ticker}
                    onClick={() => navigate(`/app/invest/company/${ticker}`)}
                    className="flex w-full items-center justify-between px-6 py-3 text-sm"
                  >
                    <span>{name}</span>
                    <span className="text-muted-foreground">{count}</span>
                  </button>
                ))}
              </CardContent>
            </Card>
            <Card className="border-foreground shadow-none">
              <CardContent className="p-5">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <Sparkle />让 Agent 帮你回顾
                </div>
                <p className="mt-2 text-xs leading-5 text-muted-foreground">
                  把过去 30 天的观点按公司、主线和证伪信号重新组织。
                </p>
                <Button
                  className="mt-4 w-full"
                  onClick={() => navigate("/app/agent?insight=monthly-review")}
                >
                  生成洞察回顾
                </Button>
              </CardContent>
            </Card>
          </div>
        </aside>
      </div>
      <PublishDialog
        open={publishOpen}
        onOpenChange={setPublishOpen}
        onPublish={(post) => setFeed((prev) => [post, ...prev])}
      />
      <DiscussionDialog
        open={discussionOpen}
        onOpenChange={setDiscussionOpen}
      />
    </div>
  );
}
