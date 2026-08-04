// public-content.ts — HONE Public Site Content (bilingual)
//
// Copy for the public surface (hone-claw.com) lives here in two parallel
// trees: CONTENT_ZH and CONTENT_EN. The exported `CONTENT` is a deep Proxy
// that reads the current locale via `useLocale()` on every property access,
// so JSX expressions like `{CONTENT.hero.headline_1}` or `<For each={CONTENT.cases.items}>`
// re-evaluate automatically when the locale signal changes.
//
// Adding a key: add it to BOTH trees with parallel shape.

import { makeContentProxy } from "./i18n";

// ── Legal copy structured nodes (terms & privacy) ────────────────────────────
// Rich prose is modeled as a typed block tree so ZH/EN stay parallel and the
// pages render via a tiny interpreter instead of embedding JSX in content.
export type LegalInline = string | { strong: string } | { code: string };
export type LegalBlock =
  | { kind: "p"; parts: LegalInline[] }
  | { kind: "ul"; items: LegalInline[][] };
export type LegalSection = { title: string; body: LegalBlock[] };

const CONTENT_ZH = {
  nav: {
    logo_tagline: "HONE",
    home: "首页",
    community: "社区",
    roadmap: "路线图与文档",
    blog: "Blog",
    me: "个人",
    chat: "对话",
    plan: "定价",
    buy: "购买完整服务",
    more: "更多",
    back_home: "返回首页",
    menu_aria: "菜单",
    locale_zh: "中文",
    locale_en: "EN",
    contact_label: "联系",
    contact_title: "联系我们",
    contact_wechat_label: "微信",
    contact_email_label: "邮箱",
    contact_wechat: "xiaobamang6677",
    contact_wechat_group: "微信社群",
    contact_wechat_hint_prefix: "联系",
    bilibili_label: "B站",
    youtube_channel_name: "巴芒投研美股频道",
    contact_email: "official@hone-claw.com",
    github_url: "https://github.com/B-M-Capital-Research/honeclaw",
  },

  hero: {
    eyebrow: "HONE · AI 投资纪律助手",
    headline_1: "不是迎合你的聊天玩具",
    headline_2: "是你的投研纪律守卫者",
    description:
      "冷静、克制、长期记忆、研究导向。HONE 是专为严肃投资者打造的开源 AI Agent，帮你建立并坚守投研纪律，而不是告诉你想听的答案。",
    cta_primary: "进入对话",
    cta_secondary: "查看路线图",
    scroll_hint: "滚动探索",
    stat_1: { value: "Rust", label: "核心引擎" },
    stat_2: { value: "7", label: "接入渠道" },
    stat_3: { value: "MIT", label: "开源协议" },
  },

  home_page: {
    roadmap_button: "产品路线图",
    roadmap_slide_tag: "路线图",
    hero_slogan: "并非迎合你的聊天玩具，而是你投资纪律的无情捍卫者。",
    start_trial: "开始试用",
    video_demo: "视频演示",
    view_full_roadmap: "完整路线图",
    zoom_hint: "查看详情",
    blog_eyebrow: "工程 Blog",
    blog_title: "为什么 HONE 选择 Rust",
    blog_desc:
      "从 Python + Node.js 到 Rust 的重构复盘：AI Coding 时代的上下文、稳定性和多端工程选择。",
    blog_cta: "阅读文章",
    plan_eyebrow: "PLAN 与定价",
    plan_title: "开源免费，完整服务一次订阅打通",
    plan_desc: "HONE 核心永远 MIT 开源、可自托管；完整服务提供每周深度直播、VIP 社群与完整研报分享。",
    plan_cta: "查看定价",
  },

  plan: {
    eyebrow: "HONE · 巴芒投研",
    share_title: "巴芒投研会员邀请",
    share_sub: "六张海报看懂完整服务：原创研报、财报前瞻、每周直播与高手社群。把这一页直接发给想加入的朋友就行。",
    title: "两个版本，简单直接",
    sub: "核心永远开源免费、可自托管；完整服务把直播、社群与研报一次打通。",
    host: {
      name: "Hari老王",
      role: "美股主理人 · HONE 出品人",
      bio: "深耕美股二十年。视频免费看，研报与社群按年订阅，HONE 投研助手开源可自托管——这一页汇总了所有入口。",
      stats: [
        { value: "20 年", label: "深耕美股" },
        { value: "100%+", label: "近几年年化收益*" },
        { value: "300+", label: "每年原创研报" },
      ],
      stats_note: "* 为主理人历史业绩，过往表现不代表未来收益。",
    },
    channel: {
      label: "免费内容",
      title: "Hari老王 · YouTube",
      handle: "@Hari老王 · 美股深度视频，每周更新",
      cta: "前往频道",
      videos_label: "近期视频",
      bilibili_title: "Bilibili 同步更新",
      bilibili_desc: "B 站同步发布，方便国内观看与弹幕互动",
      bilibili_cta: "前往 B 站",
    },
    product_card: {
      label: "研究工具",
      title: "HONE 投研助手",
      desc: "老王团队打造的开源 AI 投研工作台：个股深研、持仓跟踪、财报日历与主动推送，支持自托管。",
      cta_chat: "开始对话",
      cta_github: "GitHub 开源仓库",
    },
    member: {
      label: "付费会员",
      title: "巴芒投研会员",
      benefits: [
        { title: "知识星球", desc: "每年 300+ 份原创万字研报，每季 100+ 份财报前瞻" },
        { title: "每周直播", desc: "周四主理人深度讲解精选公司，直播随时提问" },
        { title: "VIP 群聊", desc: "与 500+ 高手实时探讨市场动态" },
        { title: "禁言精选群", desc: "主理人精选资料并标注重点，高信噪比" },
      ],
      trial_cta: "先加免费体验群",
      trial_hint: "加客服微信，先进免费群感受氛围，合适再付费。",
      activation_cta: "海外会员：开通或恢复 HONE 账号",
    },
    posters_label: "六张海报看懂完整服务",
    posters_hint: "点击任意海报可放大查看，长按或右键即可保存转发。",
    socials_label: "平台入口",
    socials: { zsxq: "知识星球", wechat: "客服微信" },
    trial_line: "想先体验？联系客服加入免费群。",
    free: {
      name: "开源免费版",
      price: "免费",
      period: "MIT 开源 · GitHub 获取",
      desc: "在自己的设备上运行完整的 HONE。",
      notes_label: "使用前需要了解",
      notes: [
        "模型供应商需要自行采买（API Key 自备）",
        "飞书 / Discord / iMessage 等渠道需要自行配置",
        "不含社区的深度投研资料分享",
        "不含社群的公司研究交流",
      ],
      cta: "去 GitHub 获取",
    },
    full: {
      name: "完整服务",
      badge: "推荐",
      price: "¥1299",
      period: "/ 年",
      promos: ["新加入立减 ¥100", "老带新再减 ¥100"],
      desc: "直播、社群、研报与 HONE 畅享，一次订阅全部打通。",
      features: [
        "每周四主理人深度公司讲解，在线直播可任意提问",
        "VIP 群与 500+ 高手畅聊，禁言群持续分享深度投研资料与实时动态",
        "知识星球 & 社区：完整的公司研报、估值和投资策略分享",
        "HONE 畅享：任何问题在社区都能得到及时反馈",
      ],
      cta: "立即购买",
      qr_title: "扫码加入知识星球",
      qr_hint: "长按或右键保存图片；扫码即可付费加入，新人立减 ¥100。",
    },
    support: {
      title: "有疑问？加客服微信",
      desc: "扫码添加企业微信客服，购买与使用问题都可以随时咨询。",
    },
    foot: "服务由 B&M Capital Research 提供；开源版能力不受订阅影响。",
    close_aria: "关闭",
  },

  trust: {
    section_label: "为什么是 HONE",
    items: [
      {
        symbol: "◈",
        title: "纪律先于观点",
        body: "HONE 不会迎合你的仓位偏见。每一次对话都以研究纪律为约束，主动识别并克制情绪驱动的决策冲动。",
      },
      {
        symbol: "∞",
        title: "长期研究记忆",
        body: "每家公司的深度画像在对话中持续积累，跨会话保留上下文，形成你独有的、不断生长的投研知识库。",
      },
      {
        symbol: "✦",
        title: "客观多维判断",
        body: "内置正反博弈推演与零幻觉协议，在噪音中找到信号——而不是把你的情绪包装成分析结论反馈给你。",
      },
    ],
  },

  cases: {
    section_label: "真实工作流",
    section_sub: "HONE 如何融入你的投研日常",
    placeholder_suffix: "场景演示截图",
    items: [
      {
        tag: "个股分析",
        title: "系统性深度研究一家公司",
        body: "从财务数据到行业竞争格局，HONE 帮你构建完整研究框架，记录每一个关键假设和风险因子。",
        image: "/hone_introduction_zh.jpg" as string | null,
      },
      {
        tag: "持仓追踪",
        title: "追踪持仓，主动提醒关键节点",
        body: "设置止盈止损逻辑，HONE 定时检查持仓状态，在你设定的条件触发时主动推送提醒。",
        image: "/hone_work_zh.jpg" as string | null,
      },
      {
        tag: "定时任务",
        title: "每周五自动触发投资复盘",
        body: "把固定工作流交给 HONE：每周复盘、月度总结、关键节点检查——按你设定的时间自动跑，不用手动催。",
        image: "/hone_page.jpg" as string | null,
      },
      {
        tag: "长期画像",
        title: "建立公司专属研究档案",
        body: "每次研究结果自动归档到公司画像，下次提问直接调用历史上下文，越用越聪明。",
        image: "/hone_solution_zh.jpg" as string | null,
      },
      {
        tag: "跨平台通知",
        title: "在 iMessage / Lark 收到 HONE",
        body: "不只是网页。HONE 通过 iMessage、Lark、Discord 等渠道主动联系你，在你最顺手的地方工作。",
        image: "/hone_channels_zh.jpg" as string | null,
      },
    ],
  },

  video: {
    section_label: "看 HONE 如何工作",
    title: "老王讲 HONE：投研 AI Agent 的实际用法",
    description:
      "从开户到深度研究，10 分钟了解 HONE 如何改变你的投研工作流。完整演示个股分析、持仓追踪、定时任务等核心场景。",
    video_url: "https://www.bilibili.com/video/BV1ByXNBGET5/",
    thumbnail: "/hone_introduction_zh.jpg",
    duration: "约 10 分钟",
    coverage: "视频涵盖：个股深度研究、持仓追踪、定时任务、多端接入演示",
    url_placeholder: "视频链接待配置",
  },

  capabilities: {
    section_label: "核心能力",
    items: [
      {
        symbol: "⚡",
        title: "投研纪律约束",
        body: "对话时主动约束情绪决策，帮你坚守原则。不是复读你的想法，而是质疑它。",
      },
      {
        symbol: "◈",
        title: "公司画像 & 长期记忆",
        body: "对每家公司建立持久档案，跨会话积累研究成果，形成真正的知识资产。",
      },
      {
        symbol: "∞",
        title: "定时任务与自动提醒",
        body: "定时工作流自动运行：复盘、持仓检查、重要节点提醒，按你设定的时间触发。",
      },
      {
        symbol: "✦",
        title: "多端接入",
        body: "Web、iMessage、Lark / Feishu、Discord、Telegram、CLI——在你最顺手的地方使用 HONE。",
      },
      {
        symbol: "⌘",
        title: "Rust 驱动的稳定性",
        body: "核心引擎用 Rust 构建，低延迟、高可靠，长期运行不掉线、不崩溃。",
      },
      {
        symbol: "ℹ",
        title: "可编程投研操作系统",
        body: "自定义 Skill、动态任务链、跨会话记忆调用，构建完全属于你的投研工作流。",
      },
    ],
  },

  community: {
    section_label: "加入社群",
    section_sub: "找到认真对待投研的同行者",
    qr_label: "二维码",
    tier1: [
      {
        key: "wechat_group",
        tier_label: "免费",
        name: "微信交流群",
        desc: "扫码加入，交流投研方法、产品反馈、使用心得",
        qr: null as string | null,
        cta: "扫码加群",
      },
      {
        key: "author_wechat",
        tier_label: "作者",
        name: "老王个人微信",
        desc: "产品问题直接反馈，重要更新优先通知",
        qr: null as string | null,
        cta: "添加微信",
      },
    ],
    tier2: [
      {
        key: "discord",
        name: "Discord",
        desc: "英文社区讨论",
        url: "#",
        label: "开放",
        symbol: "⚡",
      },
      {
        key: "zsxq",
        name: "知识星球",
        desc: "付费深度内容",
        url: "#",
        label: "付费",
        symbol: "◈",
      },
      {
        key: "vip",
        name: "VIP 群",
        desc: "私域高级功能体验",
        url: "#",
        label: "邀请制",
        symbol: "✦",
      },
      {
        key: "content",
        name: "内容号",
        desc: "投研方法论 & 产品更新",
        url: "#",
        label: "关注",
        symbol: "∞",
      },
    ],
  },

  repo: {
    section_label: "开源",
    section_sub: "B&M Capital Research 出品，MIT 协议开放",
    items: [
      {
        title: "GitHub 仓库",
        desc: "Star、Fork、提 Issue，参与开源建设",
        url: "https://github.com/B-M-Capital-Research/honeclaw",
        tag: "开源",
        icon: "⌘",
      },
      {
        title: "中文文档",
        desc: "README、使用说明、案例示范",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
        tag: "文档",
        icon: "◈",
      },
      {
        title: "安装方式",
        desc: "macOS 桌面端 + 服务端自部署指南",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动",
        tag: "安装",
        icon: "⚡",
      },
      {
        title: "代码库地图",
        desc: "模块结构、数据流与运行时边界说明",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
        tag: "技术",
        icon: "∞",
      },
      {
        title: "案例集",
        desc: "真实投研场景使用示例",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md",
        tag: "案例",
        icon: "✦",
      },
      {
        title: "贡献指南",
        desc: "参与开发、提交 PR、讨论功能方向",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md",
        tag: "贡献",
        icon: "ℹ",
      },
    ],
  },

  roadmap: {
    hero_title: "路线图与文档",
    hero_sub:
      "透明、务实、长期主义。下面是 HONE 目前能做什么、接下来做什么、以及如何接入你的投研工作流。",
    hero_meta: "ROADMAP · DOCS · API",
    sidebar_title: "ON THIS PAGE",
    version: "v0.12.4",

    toc: [
      { id: "quick-start", label: "快速开始", sub: "Quick Start" },
      { id: "capabilities", label: "能力矩阵", sub: "Capability Matrix" },
      { id: "channels", label: "渠道接入", sub: "Channels" },
      { id: "architecture", label: "架构", sub: "Architecture" },
      { id: "skills", label: "内置 Skill", sub: "Skills" },
      { id: "roadmap", label: "产品路线图", sub: "Roadmap" },
      { id: "boundary", label: "开源边界", sub: "Open Source" },
      { id: "docs", label: "文档入口", sub: "Docs" },
      { id: "contributing", label: "参与贡献", sub: "Contributing" },
      { id: "faq", label: "常见问题", sub: "FAQ" },
    ] as ReadonlyArray<{ id: string; label: string; sub: string }>,

    sections: {
      quick_start: {
        eyebrow: "§ 01 · QUICK START",
        title: "快速开始",
        intro:
          "三种方式接入 HONE：一键安装脚本、Homebrew、或源码开发。安装后可用 `hone-cli start` 跑完整运行时，也可用 `hone-cli web admin-ui` / `hone-cli web user-ui` 单独打开管理端或公开用户端界面。",
      },
      capabilities: {
        eyebrow: "§ 02 · CAPABILITY MATRIX",
        title: "能力矩阵",
        legend: { stable: "生产可用", beta: "预览", planned: "规划中" },
      },
      channels: {
        eyebrow: "§ 03 · CHANNELS",
        title: "渠道接入",
        intro:
          "HONE 是多端接入的投研助手。每个渠道都是独立进程，可独立启停、独立配置。",
      },
      architecture: {
        eyebrow: "§ 04 · ARCHITECTURE",
        title: "系统架构",
        intro:
          "Rust 核心引擎 · 多 Agent 引擎抽象 · SolidJS 前端。公开用户端、管理后台和渠道进程共用同一套后端能力，但按界面、端口和进程边界隔离；Cloud PG / OSS 正在分阶段接管运行时存储。",
        footnote_prefix: "完整模块说明见",
        footnote_link: "docs/repo-map.md ↗",
      },
      skills: {
        eyebrow: "§ 05 · BUILT-IN SKILLS",
        title: "内置 Skill",
        intro_prefix: "HONE 的 Skill 由模型根据上下文自动调用。下面是仓库",
        intro_suffix: "目录下的 17 个公开 Skill。",
      },
      roadmap: {
        eyebrow: "§ 06 · ROADMAP",
        title: "产品路线图",
        intro_lead: "我们按",
        intro_highlight: "Now / Next / Later",
        intro_trail: "三阶段推进，具体发布节奏见 GitHub Releases。",
      },
      boundary: {
        eyebrow: "§ 07 · OPEN SOURCE BOUNDARY",
        title: "开源边界",
        intro:
          "MIT 协议开源。开源仓库包含完整可运行的核心系统，私域增强能力不公开但不影响主流程可用性。",
        open_label: "开源公开",
        closed_label: "私域 / 付费",
      },
      docs: {
        eyebrow: "§ 08 · DOCUMENTATION",
        title: "文档入口",
      },
      contributing: {
        eyebrow: "§ 09 · CONTRIBUTING",
        title: "参与贡献",
        intro: "HONE 是开源项目，欢迎所有形式的参与——不只是代码。",
      },
      faq: {
        eyebrow: "§ 10 · FAQ",
        title: "常见问题",
      },
    },

    install: {
      tabs: [
        {
          key: "curl" as const,
          label: "curl | bash",
          badge: "推荐" as string | null,
        },
        {
          key: "brew" as const,
          label: "Homebrew",
          badge: null as string | null,
        },
        {
          key: "source" as const,
          label: "源码 / CLI",
          badge: null as string | null,
        },
      ],
      requirements_prefix: "系统要求：",
      curl: [
        "# macOS / Linux 一键安装（推荐）",
        "$ curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
        "$ hone-cli start",
      ],
      brew: [
        "# Homebrew tap (macOS / Linux)",
        "$ brew install B-M-Capital-Research/honeclaw/honeclaw",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
        "$ hone-cli start",
      ],
      source: [
        "# 源码开发模式（本地 CLI 构建启动）",
        "$ git clone https://github.com/B-M-Capital-Research/honeclaw",
        "$ cd honeclaw",
        "$ cargo run -p hone-cli -- start --build",
      ],
    },

    requirements:
      "macOS 13+ / Linux x86_64 / arm64 · 首次源码构建约 10 分钟（需本机已有 Rust / Bun）",

    architecture_points: [
      {
        title: "CLI 启动",
        desc: "`hone-cli doctor / onboard / start` 负责体检、首装向导、启动 hone-console-page 与已启用渠道；`hone-cli web admin-ui` / `hone-cli web user-ui` 可定位或启动管理端与公开用户端；源码模式使用 `cargo run -p hone-cli -- start --build`，并会把已定位的 `hone-mcp` 作为 `HONE_MCP_BIN` 透传给子进程。",
      },
      {
        title: "公开用户端",
        desc: "公开用户端路由包含 `/`、`/roadmap`、`/blog`、`/blog/:slug`、`/chat`、`/me`、`/activate`、`/portfolio`、`/terms`、`/privacy`，并保留开发用 `/__share-preview` 分享卡预览页；`/blog` 是双语静态长文内容面，Cloudflare Worker 为文章分享卡注入 crawler 友好的 metadata；国内 `/chat` 使用阿里云行为验证 + 手机短信验证码登录，管理端邀请名单是准入来源，海外用户则在 `/activate` 使用邮箱验证码完成 Stripe 购买或恢复会员权益，`/me` 展示服务端统一记录的会员与续费状态；桌面端为可收起左侧栏 + 右侧对话工作台，侧栏聚合导航、账号、最近对话历史、联系入口和 GitHub stars，支持助手回答复制、图片分享、非图片生成物附件下载、历史回看，以及图片 / 文件附件进入共享 ingest 后再交给 runner 读取；`/portfolio` 只读展示推送上下文与公司画像入口，后端公开面收敛在 `/api/public/*`，其中 `/api/public/digest-context` 与 `/api/public/company-profile` 暴露当前用户的投资主线和单票画像，`/api/public/file` 代理可下载生成物，`/api/public/v1/chat/completions` 提供 API key 鉴权的 OpenAI-compatible 对话接口。",
      },
      {
        title: "存储与云运行时",
        desc: "`cloud.postgres` / `cloud.oss` 是 v0.12.4 起的一等配置项，并通过 env 引用真实凭证；配置 OSS 后，公开 Web 上传会写入 `public-uploads/...` 并返回 `oss://bucket/key`，`/api/public/image` 与 `/api/public/file` 可代理托管对象；`/api/meta` 暴露 `cloud_runtime`、`cloud_postgres`、`cloud_oss`、`oss_file_proxy` 与本地 durable dependency 计数。当前 main 的 PG 热路径已覆盖 sessions、Web invite/auth sessions、conversation quota、cron jobs/runs、due-job claims、skill registry、notification prefs、portfolio、LLM audit 与 company profile files；`hone-cli cloud doctor / migrate / object-bench` 可做云端体检、本地 `data/` dry-run 或幂等导入、OSS/R2 小对象延迟对比。`cloud.strict_no_local_storage=true` 会依据当前配置阻止仍有 durable 本地依赖的启动；在 cloud 模式同时配置 PG 与 OSS 后，已知 durable 数据面不再被这些本地存储阻塞。",
      },
      {
        title: "管理后台",
        desc: "管理后台提供 dashboard、sessions、skills、tasks、users、research、llm-audit、task-health、notifications、schedule、settings、logs 等维护入口；users 页把持仓、公司画像、会话与研究任务按用户主体聚合，公司画像支持 actor 空间列表、详情查看、删除、zip 导出、导入预览与冲突处理后导入。",
      },
      {
        title: "Agent 引擎层",
        desc: "推荐 Agent 引擎是 Codex ACP、HONE Cloud 和 OpenCode ACP；同时保留 Gemini CLI 与 Codex CLI。Codex ACP 默认使用 GPT-5.6 Sol 和 xhigh reasoning effort。LLM 凭证以 `config.yaml` 为唯一真相源，OpenRouter 与通用 OpenAI-compatible provider 都支持 `llm.providers.*.api_key/api_keys` key pool，遇到上游 429 / 配额错误时可尝试下一个 key；`gemini_acp` 仅保留为迁移配置，不作为运行时入口。",
      },
      {
        title: "事件与任务",
        desc: "Cron 定时任务、事件引擎摘要、`/missed` 回查、通知偏好与渠道投递共享同一套 Rust 后端与执行历史模型。定时结果先写入会话历史再做在线提示，浏览器离线也不会丢失或误报；执行失败会以产品化的提示呈现并可追溯。Web、飞书、Discord 等渠道读取同一份 Cron、持仓与云端配置。",
      },
    ],

    capability_matrix: [
      {
        group: "投研核心",
        rows: [
          {
            name: "投研纪律约束 & 零幻觉协议",
            status: "stable",
            note: "system prompt 强约束",
          },
          {
            name: "公司画像 & 长期记忆",
            status: "stable",
            note: "公司画像 Skill + 管理端导入/导出",
          },
          {
            name: "个股研究 / 深度研究",
            status: "stable",
            note: "stock_research + deep_stock_research",
          },
          {
            name: "持仓追踪与提醒",
            status: "stable",
            note: "portfolio_management + cron",
          },
          {
            name: "估值 / 选股 / 仓位建议",
            status: "stable",
            note: "stock_research 覆盖估值与筛选，position_advice 覆盖仓位建议",
          },
          {
            name: "图表 & 图像生成",
            status: "stable",
            note: "chart_visualization / image_generation",
          },
          {
            name: "公开聊天工作台与分享",
            status: "stable",
            note: "侧栏历史 + html2canvas + qrcode + markdown 渲染 + CJK 代码块字体 + 附件下载卡片",
          },
          { name: "向量检索增强记忆", status: "planned", note: "规划中" },
        ],
      },
      {
        group: "运行时",
        rows: [
          {
            name: "Rust 核心引擎",
            status: "stable",
            note: "Tokio · axum · SSE",
          },
          {
            name: "SolidJS 前端",
            status: "stable",
            note: "Vite · Tailwind v4 · stale asset recovery",
          },
          {
            name: "公开 Blog 与文档内容面",
            status: "stable",
            note: "双语 Markdown 文章 + 文章路由 + Cloudflare 分享 metadata",
          },
          { name: "Tauri 桌面端", status: "stable", note: "macOS 已发布" },
          {
            name: "多 Agent 引擎抽象",
            status: "stable",
            note: "Gemini CLI · Codex CLI/ACP · OpenCode ACP · Hone Cloud",
          },
          {
            name: "LLM provider key pool 与上游错误保真",
            status: "stable",
            note: "config.yaml llm.providers.*.api_key/api_keys · OpenRouter / OpenAI-compatible fallback",
          },
          {
            name: "Cloud PG / OSS 运行时迁移",
            status: "beta",
            note: "sessions / web auth / quota / cron 已有 PG 热路径，OSS 公开上传代理与迁移工具仍在 beta",
          },
          {
            name: "渠道回复收口与副作用确认",
            status: "stable",
            note: "response_finalizer + 输出净化层可恢复成功副作用确认并隐藏内部路径 / skill 降级措辞",
          },
          {
            name: "Windows / Linux 桌面端",
            status: "planned",
            note: "Tauri 多平台打包",
          },
        ],
      },
      {
        group: "扩展",
        rows: [
          {
            name: "Cron 定时任务",
            status: "stable",
            note: "scheduled_task skill + /api/cron-jobs + 执行历史 / heartbeat / quiet_hours / guard / Web SSE / Discord 发送失败诊断 / ACP transport 断连边界回归",
          },
          {
            name: "自定义 Skill",
            status: "stable",
            note: "skill_manager · create_skill.sh",
          },
          {
            name: "MCP 协议",
            status: "stable",
            note: "hone-mcp server + HONE_MCP_BIN / HONE_CONFIG_PATH / HONE_DATA_DIR 绝对化与透传",
          },
          {
            name: "HTTP + SSE 内部 API",
            status: "stable",
            note: "hone-web-api 路由全开",
          },
          {
            name: "公开用户 SMS 登录与验证码守门",
            status: "stable",
            note: "Aliyun Captcha + Aliyun SMS + 管理端 Web 邀请名单",
          },
          {
            name: "公开 OpenAI-compatible Chat API",
            status: "beta",
            note: "用户 API key + /api/public/v1/chat/completions",
          },
          {
            name: "按用户细粒度推送偏好",
            status: "stable",
            note: "notification_preferences skill + 设置页 + config 全局节流",
          },
          {
            name: "漏推 / 截断事件回查",
            status: "stable",
            note: "missed skill + missed_events tool",
          },
          { name: "公开 Skill 市场", status: "planned", note: "社区共享" },
        ],
      },
    ],

    channels: [
      {
        name: "Web",
        icon: "⚡",
        status: "stable",
        desc: "手机号 + 短信验证码登录的邀请制聊天页，定时任务结果会落入历史并用 SSE 做在线提示",
      },
      {
        name: "iMessage",
        icon: "✦",
        status: "stable",
        desc: "macOS 原生短信集成",
      },
      {
        name: "Lark / Feishu",
        icon: "◈",
        status: "stable",
        desc: "飞书机器人双向通信、scheduler heartbeat 推送与 loop 监督恢复",
      },
      {
        name: "Discord",
        icon: "∞",
        status: "stable",
        desc: "Bot 应用集成；scheduler 发送失败会保留脱敏错误原因",
      },
      {
        name: "Telegram",
        icon: "⌘",
        status: "stable",
        desc: "Bot API 接入",
      },
      {
        name: "CLI",
        icon: "ℹ",
        status: "stable",
        desc: "命令行流式对话",
      },
      {
        name: "MCP",
        icon: "✧",
        status: "stable",
        desc: "作为 MCP server 嵌入 Claude / Cursor 等",
      },
    ],

    skills: [
      { name: "stock_research", desc: "单只个股研究、估值框架、按条件筛选" },
      {
        name: "deep_stock_research",
        desc: "约 1–2 小时的深度研究任务（管理员）",
      },
      { name: "company_portrait", desc: "维护公司画像、投资主线、事件时间线" },
      { name: "portfolio_management", desc: "持仓增减、再平衡、Ticker 校验" },
      { name: "position_advice", desc: "结合行情与持仓给出加减仓建议" },
      { name: "market_analysis", desc: "宏观、政策、行业动量与指数判断" },
      { name: "gold-analysis", desc: "黄金、金 ETF、金矿股的宏观与持仓分析" },
      { name: "scheduled_task", desc: "注册 / 修改 / 取消用户定时推送任务" },
      {
        name: "missed",
        desc: "查询 digest 被截断、冷却、过滤或折叠的漏推事件",
      },
      { name: "chart_visualization", desc: "趋势 / 对比 / 分布 / 散点研究图" },
      { name: "image_generation", desc: "持仓截图、研究图卡、说明图" },
      {
        name: "image_understanding",
        desc: "解析可读图片输入；Web direct 图片附件已复用共享 ingest，失败时只给产品化重试提示",
      },
      {
        name: "pdf_understanding",
        desc: "解析 PDF（财报、研报）输出要点与风险",
      },
      { name: "skill_manager", desc: "查看 / 新建 / 修改 HONE Skill" },
      {
        name: "notification_preferences",
        desc: "用自然语言调整自己的推送偏好（严重度、持仓过滤、事件类型允许/屏蔽范围）",
      },
      { name: "hone_admin", desc: "查看修改 HONE 源码与配置（管理员）" },
    ],

    now: {
      label: "当前已有",
      items: [
        "Web 对话工作台：桌面三栏布局 + 移动端抽屉导航，历史记录完整可回看",
        "研究分享：把回答一键导出成品牌长图，或复制图片 / 文字直接分享",
        "图片与文件：对话中可上传图片 / PDF 等材料，生成的 CSV / XLSX / PDF 可直接下载",
        "macOS 桌面端：内置后端，下载即用",
        "多渠道接入：Web / iMessage / 飞书 / Discord / Telegram / CLI / MCP，共 7 个",
        "17 个内置投研 Skill：个股分析、持仓、财报研究、估值筛选、图表、PDF、定时任务、推送偏好等",
        "投研纪律约束与零幻觉协议：不迎合仓位偏见，先核实数据再下结论",
        "公司画像与跨会话长期记忆：每家公司的研究上下文持续积累、随时调用",
        "定时任务与主动推送：盘前快报、收盘复盘、财报提醒可按需订阅",
        "财经日历：宏观事件与持仓财报放进同一条时间线",
        "投资页：持仓、投资主线与公司画像的研究上下文一页总览",
        "官方社区：研究判断、市场观察与关键资料的只读时间线",
        "OpenAI 兼容 API：用 API key 把 HONE 接进你自己的工具链",
        "部署自由：本地自托管与云端运行时（Postgres / OSS）都支持",
        "完整的工程改动与修复记录见 GitHub Releases 与仓库提交历史",
      ],
    },
    next: {
      label: "近期计划",
      items: [
        "Windows / Linux 桌面端打包",
        "用户自定义 Skill 编辑器",
        "更完整的数据导入 / 导出（公司画像包已支持，持仓与研究结果进行中）",
        "云端部署的运维与迁移工具持续完善",
        "公开 Skill 文档与示例集",
        "向量检索增强长期记忆",
      ],
    },
    later: {
      label: "长期愿景",
      items: [
        "多用户协作研究空间",
        "可视化持仓分析面板",
        "更完整的开发者 API、SDK 与示例",
        "社区 Skill 市场",
        "多 Agent 协同编排",
      ],
    },

    boundary: {
      label: "开源边界",
      open: [
        "Rust 核心引擎（hone-core / hone-channels / hone-llm / hone-tools）",
        "前端 UI（SolidJS + Tailwind v4）",
        "Tauri 桌面端壳",
        "全部 17 个公开 Skill",
        "全部渠道集成代码（Web / iMessage / Lark / Discord / Telegram / CLI / MCP）",
      ],
      closed: [
        "私域高级 Skill 库",
        "付费数据源 API Key",
        "VIP 专属功能 / 托管服务",
      ],
    },

    docs: [
      {
        title: "README（English）",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README.md",
        desc: "Project overview, install, quick start",
      },
      {
        title: "README（中文）",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
        desc: "项目总览、安装、快速上手",
      },
      {
        title: "Wiki",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/wiki.md",
        desc: "安装、启动、端口、配置、验证与排障入口",
      },
      {
        title: "Release Notes v0.12.4",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/releases/v0.12.4.md",
        desc: "最新 release 的用户影响、升级方式与已知注意事项",
      },
      {
        title: "HONE Blog",
        url: "https://hone-claw.com/blog",
        desc: "公开双语长文，记录架构选择、迁移复盘与产品说明",
      },
      {
        title: "Repo Map",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
        desc: "模块边界、运行时数据流与常见联动改动",
      },
      {
        title: "Cases (中文)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md",
        desc: "真实投研场景使用示例集",
      },
      {
        title: "Cases (English)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_EN.md",
        desc: "Real-world case studies",
      },
      {
        title: "Skills 目录",
        url: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills",
        desc: "全部公开 Skill 的源码与说明",
      },
      {
        title: "CONTRIBUTING.md",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md",
        desc: "贡献指南",
      },
      {
        title: "SECURITY.md",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/SECURITY.md",
        desc: "漏洞披露策略",
      },
    ],

    contributing: [
      {
        icon: "◈",
        title: "提交 Issue",
        desc: "报告 bug、提功能建议、讨论设计",
        href: "https://github.com/B-M-Capital-Research/honeclaw/issues/new/choose",
      },
      {
        icon: "⚡",
        title: "发 Pull Request",
        desc: "修 bug、加功能、优化文档",
        href: "https://github.com/B-M-Capital-Research/honeclaw/pulls",
      },
      {
        icon: "∞",
        title: "贡献 Skill",
        desc: "用 skills/skill_manager/create_skill.sh 起一个新 Skill",
        href: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills",
      },
    ],

    bottom_cta: {
      title: "准备好开始了吗？",
      desc: "进入对话，或直接 clone 仓库开始本地运行。",
      primary: "进入对话 →",
    },

    faqs: [
      {
        q: "HONE 和普通 AI 聊天工具有什么区别？",
        a: "HONE 不会迎合你的观点。它以投研纪律为约束，主动识别并反驳情绪化决策。每次对话都以长期研究记忆（公司画像）为基础，而不是每次重新开始。",
      },
      {
        q: "需要自己部署吗？",
        a: "三种方式任选：①「curl | bash」一键装 hone-cli；② Homebrew tap；③ clone 仓库后用本地 CLI 构建启动。前两种共享同一份 GitHub release bundle，不需要自己编译 Rust。公开 SMS 登录需要配置阿里云短信；如启用行为验证，还需要配置阿里云验证码环境变量。升级前端后，公开页面会通过资产恢复逻辑处理旧 chunk 缓存导致的加载失败。",
      },
      {
        q: "支持哪些 LLM？",
        a: "通过 Agent 引擎抽象层支持 HONE Cloud、Gemini CLI、Codex CLI / ACP 与 OpenCode ACP。默认本地路径为 Codex ACP，使用 GPT-5.6 Sol 和 xhigh reasoning effort。凭证统一写入 `config.yaml` 的 `llm.providers.*.api_key/api_keys`，通用 OpenAI-compatible provider 与 OpenRouter 都能在 key pool 内尝试下一个可用 key。",
      },
      {
        q: "开源协议？能商用吗？",
        a: "MIT 协议，可商用。开源仓库包含完整可运行的核心引擎、UI、桌面端、全部 17 个公开 Skill 和 7 个渠道集成。私域高级 Skill 与付费数据源接入不在仓库中，不影响主流程。",
      },
      {
        q: "数据存在哪里？",
        a: "默认仍在本地或自部署服务器存储（macOS 桌面端用户目录 ~/.honeclaw）。v0.12.4 已加入 Cloud PG / OSS 运行时配置；当前 main 的 cloud 模式可把 sessions、Web invite/auth sessions、conversation quota、cron jobs/runs、due-job claims、skill registry、notification prefs、portfolio、LLM audit 与 company profile files 放到 PG，把公开上传、生成图片 / 文件与迁移文档放到 OSS。`cloud.strict_no_local_storage=true` 会按配置检查是否仍有 durable 本地依赖；HONE 官方不默认托管你的数据。",
      },
      {
        q: "和 Codex / RooCode 等 coding agent 的关系？",
        a: "HONE 借鉴了这些产品的 Agent 引擎、Skill 与会话架构，但专注投研而非写代码。Codex CLI / ACP、Gemini CLI 和 OpenCode ACP 在 HONE 中作为可插拔引擎存在。",
      },
    ],
  },

  me: {
    logged_in_title: "账号中心",
    logged_in_eyebrow: "",
    logged_out_title: "请先登录",
    logged_out_desc: "登录后查看你的历史记录和账号信息。",
    logged_out_cta: "前往对话页登录",
    invite_note: "需要手机号加入邀请名单后才能进入对话",
    loading: "加载中…",
    account_info_title: "账号信息",
    usage_today_label: "账号状态",
    date_locale: "zh-CN",
    date_placeholder: "—",
    stats: {
      remaining_today_label: "账号状态",
      remaining_today_sub_template: "",
      total_label: "历史记录",
      total_sub: "",
      daily_limit_label: "访问权限",
      daily_limit_sub: "",
    },
    actions: {
      chat: "进入对话 →",
      roadmap: "查看路线图",
      community: "查看社区",
      logout: "退出登录",
    },
    membership: {
      title: "会员 / 高级功能",
      desc: "付费体系、VIP 群、专属能力——即将推出。加入社群获取第一手信息。",
    },
    fields: {
      user_id: "账号",
      created_at: "注册时间",
      last_login: "最近登录",
      daily_limit: "访问权限",
      used_today: "历史记录",
      remaining: "账号状态",
    },
  },

  chat_page: {
    header: {
      subtitle: "投资助手",
    },
    sidebar: {
      label: "聊天导航",
      collapse: "收起侧边栏",
      expand: "展开侧边栏",
      signed_in: "已登录",
      account_center: "账号中心",
      history_title: "对话记录",
      history_empty: "开始提问后，这里会显示最近的问题。",
      history_attachment: "带附件的问题",
      history_empty_item: "空消息",
    },
    pushes: {
      nav: "推送",
      open_aria: "打开推送中心",
      fallback_title: "定时推送",
      fallback_summary: "任务已完成",
    },
    prefs: {
      aria_label: "字号与主题",
      font_size: "字号",
      theme: "主题",
      theme_auto: "自动",
      theme_light: "浅",
      theme_dark: "深",
      language: "语言",
      language_zh: "中文",
      language_en: "English",
    },
    status: {
      error: "HONE 出错了",
      streaming: "HONE 输出中",
      running: "HONE 执行中",
      thinking: "HONE 思考中",
      done: "本轮已完成",
      fallback_error: "请求出错，请重试。",
      stop: "停止",
      stopped: "已停止本轮回答",
    },
    attachments: {
      image_title: "图片",
      image_subtitle: "照片与截图",
      file_title: "文件",
      file_subtitle: "PDF · 文档 · 其他",
    },
    composer: {
      quota_exhausted: "今日对话次数已用完",
      placeholder: "向 HONE 提问…",
      send_aria: "发送",
      proactive_tip: "持仓分析",
      proactive_title: "HONE 可以主动盯住你的持仓",
      proactive_intro:
        "把持仓或关注标的告诉 HONE 后，它会按你的偏好筛选重要变化，并在合适的时候提醒你。",
      proactive_items: [
        {
          title: "持仓相关提醒",
          body: "财报发布、电话会、SEC 文件、重大新闻、评级变化和价格异动。",
        },
        {
          title: "持仓分析",
          body: "结合你的仓位、关注理由和长期主线，整理可能影响判断的信号。",
        },
        {
          title: "自然语言管理",
          body: "直接说「只推持仓相关」「今晚勿扰」「每周五复盘」即可开关偏好或管理定时任务。",
        },
      ],
      proactive_examples_title: "你可以这样说",
      proactive_examples: [
        "介绍一下磷化铟产业链，推荐一些相关的光模块公司",
        "我持有 AAPL 和 NVDA，帮我开启关键事件提醒",
        "只给我推持仓相关的财报和重大新闻",
        "每周五收盘后做一次持仓复盘",
      ],
      proactive_close_aria: "关闭推送模式说明",
      proactive_got_it: "知道了",
      finance_calendar_tip: "财经日历",
      finance_calendar_title: "我的财经日历",
      finance_calendar_intro:
        "选择月份后，HONE 会整理当月宏观事项和你持仓/关注公司的财报日期，并作为图片发到聊天里。",
      finance_calendar_months_label: "日历月份",
      finance_calendar_current_month: "回到本月",
      finance_calendar_previous_aria: "上一个月",
      finance_calendar_next_aria: "下一个月",
      finance_calendar_preview_aria: "财经日历图片预览",
      finance_calendar_preview_open: "查看日历大图",
      finance_calendar_preview_hint: "点击查看大图，可缩放和拖动",
      finance_calendar_preview_close: "关闭日历大图",
      finance_calendar_image_loading: "高清日历正在加载",
      finance_calendar_image_loading_hint: "图片生成完成，正在从安全存储读取，请稍候",
      finance_calendar_image_failed: "日历图片没有加载成功",
      finance_calendar_image_failed_hint: "网络可能较慢，可以立即重新加载",
      finance_calendar_image_retry: "重新加载日历",
      finance_calendar_image_save: "保存图片",
      finance_calendar_image_saving: "保存中…",
      finance_calendar_image_share: "分享",
      finance_calendar_image_zoom: "日历缩放",
      finance_calendar_image_save_hint: "双指缩放、单指拖动画布；长按图片可存入照片",
      finance_calendar_image_action_failed: "当前浏览器未完成操作，请打开大图后长按保存",
      finance_calendar_share_text: "我的 HONE 财经日历",
      finance_calendar_zoom_in: "放大",
      finance_calendar_zoom_out: "缩小",
      finance_calendar_zoom_fit: "适合屏幕",
      finance_calendar_loading: "正在整理本月日历…",
      finance_calendar_macro_label: "宏观事件",
      finance_calendar_earnings_label: "持仓财报",
      finance_calendar_holdings_label: "关注标的",
      finance_calendar_sources: "BLS · BEA · Federal Reserve · Census · ISM · FMP",
      finance_calendar_send: "发送这张日历",
      finance_calendar_sending: "正在生成图片…",
      finance_calendar_error: "发送失败",
      finance_calendar_render_error: "财经日历模板还没准备好",
      finance_calendar_upload_error: "图片上传失败",
      finance_calendar_close_aria: "关闭财经日历",
    },
    history: {
      loading_older: "加载中…",
      load_older: "继续向上滚动加载更早消息",
    },
    restoring: {
      title: "正在恢复对话",
      desc: "正在校验当前会话并恢复聊天历史",
      retrying: "后端响应较慢，正在自动重试（第 {attempt} 次）…",
      failed_title: "恢复对话失败",
      failed_desc: "当前会话暂时没有恢复成功，可以立即重新尝试。",
      retry_button: "重新恢复",
      timeout_reason: "请求超时",
      generic_reason: "网络或服务暂时不可用",
      reason_prefix: "原因：{message}",
    },
    earnings: {
      preview_label: "财报前瞻",
      analysis_label: "财报分析",
      preview_hint:
        "输入公司后，HONE 会核验实体、预期和关键变量，并生成带品牌水印的分享 PDF。",
      analysis_hint:
        "输入公司并可上传财报、公告或电话会材料；HONE 会先读取材料，再完成分析和分享 PDF。",
      company_placeholder: "例如：NVIDIA / NVDA",
      company_required: "请输入公司名称或股票代码",
      pick_files: "选择财报文件",
      file_hint: "PDF、Word、Excel、图片或文本",
      starting: "正在启动…",
      close: "关闭",
      start_failed: "启动失败，请稍后重试",
      busy: "当前无法启动新的分析任务",
      loading_preview_skill: "正在加载财报前瞻技能",
      loading_analysis_skill: "正在加载财报分析技能",
      selected_files: "已选择 {count} 个文件",
      start_action: "启动{label}",
    },
    workspace: {
      brand_aria: "HONE 工作台",
      default_user: "HONE 用户",
      user_prefix: "用户 ",
      history_label: "对话记录",
      new_chat: "新对话",
      search_history: "搜索对话记录",
      recent: "最近",
      syncing_history: "正在同步对话记录…",
      no_match: "没有匹配的对话记录，换一个关键词试试。",
      history_empty: "还没有对话记录，开始提问后会出现在这里。",
      loading: "正在加载…",
      load_older: "加载更早记录",
      personal_space: "个人研究空间",
      logout: "退出",
      agent_tagline: "你的投资研究智能体",
      search_all: "搜索公司、主题或社区内容",
      open_pushes: "打开通知",
      reconnecting: "正在重新连接研究空间",
      restoring: "正在恢复研究空间",
      sync_attempt: "后端响应较慢，正在进行第 {attempt} 次同步。",
      sync_detail: "正在同步研究记录、推送与最近会话。",
      qa_moves_title: "解释组合波动",
      qa_moves_summary: "解释今天组合上涨或下跌的主要原因",
      qa_moves_meta: "组合 · 今日",
      qa_moves_prompt: "请结合我的持仓，解释今天组合波动的主要原因，并按影响大小排序。",
      qa_compare_title: "比较两家公司",
      qa_compare_summary: "比较两家公司当前的推理侧机会",
      qa_compare_meta: "公司 · 对比",
      qa_compare_prompt: "我想比较两家公司，请先问我公司名称，再从业务、竞争力、估值和风险展开。",
      qa_filing_title: "阅读财报材料",
      qa_filing_summary: "从财报中识别需要验证的主线",
      qa_filing_meta: "材料 · 深度",
      qa_filing_prompt: "我会上传一份财报材料，请提取关键数据、管理层表述、变化和待验证问题。",
      qa_track_title: "建立跟踪计划",
      qa_track_summary: "为持仓或关注标的建立持续跟踪",
      qa_track_meta: "任务 · 持续",
      qa_track_prompt: "请根据我的持仓和关注标的，帮我建立一套持续跟踪计划。",
      seed_portfolio_eyebrow: "持仓研究",
      seed_portfolio_title: "梳理今天的组合变化",
      seed_portfolio_summary: "从持仓、新闻与事件中找出值得关注的变量",
      seed_event_eyebrow: "即将发生",
      seed_event_title: "查看近期重要事件",
      seed_event_summary: "把宏观日程与持仓财报放进同一条时间线",
      seed_research_eyebrow: "新研究",
      seed_research_title: "建立一条研究主线",
      seed_research_summary: "从问题出发，保留来源、结论与后续跟踪",
      insight_prompt: "请基于这条研究线索继续分析：{title}。{summary}",
      context_prefix: "正在基于：",
      context_portfolio: "我的组合",
      context_events: "今日事件",
      insight_count: "今天有 {count} 条值得继续研究的线索。",
      quick_start: "快速开始",
      quick_start_hint: "会展开来源与推理过程",
      today_insights: "今日研究线索",
      browse_community: "逛逛社区",
      no_insight_match: "没有匹配的研究线索，换一个关键词试试。",
      key_events: "重要事件",
      finance_calendar: "财经日历",
      open_my_calendar: "查看我的财经日历",
      calendar_summary: "宏观日程与持仓财报",
      upcoming_events: "即将到来的事件",
      open_your_calendar: "查看你的财经日历",
      recent_research: "最近研究",
      research_empty: "发起研究后会自动保存在这里。",
      continue_research: "继续这项研究",
      open_menu: "打开菜单",
      history_title: "对话历史",
      pushes: "通知",
      open_account: "打开{name}的账户",
      drawer_aria: "工作区菜单与对话记录",
      close_menu: "关闭菜单",
      main_menu: "主要菜单",
      search_chats: "搜索聊天记录",
      chat_records: "聊天记录",
      drawer_history_empty: "开始提问后，对话记录会出现在这里。",
      main_nav: "主要导航",
      insights: "洞察",
      me: "我的",
    },
    me_page: {
      plan_live: "每周四主理人深度公司讲解，在线直播可任意提问",
      plan_group: "群与 500+ 高手畅聊，持续分享深度投研资料与实时动态",
      plan_planet: "知识星球与社区：完整的公司研报、估值和投资策略分享",
      plan_qa: "畅享：任何问题在社区都能得到及时反馈",
      cycle_until: "当前周期至",
      no_renew: "到期后不再自动续费。",
      opening: "正在打开…",
      manage_stripe: "在 Stripe 管理订阅",
      no_access_sub: "这条订阅当前不授予 HONE 访问权限。",
      intl_member: "国际会员 · 统一权益",
      cn_invite: "账号 · 国内邀请",
      entitled: "你的 HONE 权益已启用",
      not_entitled: "会员权益当前不可用",
      awaiting_payment: "正在等待付款平台确认。页面会自动刷新；成功跳转不会直接开通权益。",
      duplicate_subs: "检测到多条有效 Stripe 订阅。HONE 访问不会中断，但你可能被重复扣费，请在 Stripe 中取消不需要的一条。",
      paused_note: "账号资料仍会保留，付费功能已暂停。你可以恢复付款或重新订阅，服务端确认后会自动恢复访问。",
      any_stripe_grants: "任意一条有效 Stripe 权益都可授予访问：",
      cn_channel_grants: "该账号由国内邀请渠道授予访问：",
      wecom_qr_alt: "企业微信客服二维码",
      support_cta: "有疑问？加客服微信",
      support_hint: "扫码添加企业微信客服，会员与账单问题随时咨询。",
      support_title: "企业微信客服",
      close: "关闭",
      save_qr_hint: "长按或右键保存图片，扫码添加客服。",
      personal_space: "个人研究空间",
      me: "我的",
      me_subtitle: "自选与持仓、投资画像风格、账户信息和订阅都在这里管理。",
      account_info: "账户信息",
      account: "账户",
      verify_channel: "验证渠道",
      email_channel: "邮箱{value}",
      cn_phone_invite: "国内手机号邀请",
      registered_at: "注册时间",
      last_login: "最近登录",
      access: "访问权限",
      enabled_quota: "已启用 · 每日 {count}",
      times: "次",
      enabled: "已启用",
      paused: "已暂停",
      view_membership: "查看会员与续费",
      open_agent: "进入 Agent",
      open_community: "去社区看看",
      sign_out: "退出登录",
      billing_note: "负责海外账单处理；HONE 以服务端权益记录决定访问权限。",
      about_help: "关于与帮助",
      home: "官网首页",
      pricing: "会员与定价",
      roadmap: "路线图与文档",
      tos: "用户协议",
      privacy: "隐私政策",
      disclaimer: "内容仅供研究参考，不构成投资建议。市场有风险，决策需独立判断。",
      loading_title: "正在加载个人空间",
      loading_detail: "正在确认账户与研究权限。",
    },
    community_page: {
      just_now: "刚刚",
      official: "官方社区",
      resources: "社区资源",
      close_preview: "关闭预览",
      pdf_unsupported: "当前宿主无法显示这份 PDF",
      pdf_fallback: "文件本身仍可访问，请使用下方“下载资源”继续查看。",
      pdf_preparing: "正在准备 PDF 安全预览…",
      file_preview: "社区文件预览",
      pdf_slow: "内嵌预览响应较慢；若画面仍为空白，请直接下载资源。",
      pdf_unavailable: "内嵌预览不可用，请直接下载资源。",
      pdf_loaded: "已载入；若宿主未绘制页面，可直接下载资源。",
      pdf_verifying: "正在校验并载入 PDF…",
      image: "社区图片",
      zoom_hint: "双指或滚轮缩放，放大后可拖动",
      preview_na_hint: "内嵌预览不可用，下载后可完整查看",
      sandboxed: "已通过安全沙箱载入",
      image_zoom: "图片缩放",
      zoom_out: "缩小",
      zoom_in: "放大",
      fit_screen: "适应屏幕",
      downloading: "正在下载…",
      download: "下载资源",
      download_failed: "下载失败，请重试",
      older_failed: "更早动态加载失败",
      load_failed: "社区内容暂时无法加载",
      resource_failed: "资源下载失败",
      login_title: "登录后查看 HONE 社区",
      login_hint: "社区当前为只读，内容仅向已登录用户展示。",
      search: "搜索社区、公司或主题",
      eyebrow: "社区研究 · 只读",
      title: "社区",
      subtitle: "来自 HONE 社区的研究判断、市场观察与关键资料，按发生时间连续沉淀。",
      loading: "正在加载社区内容…",
      reload: "重新加载",
      feed_title: "官方社区动态",
      no_match: "没有匹配的社区内容。",
      read_only: "只读",
      preview_label: "预览{name}",
      image_protected: "图片受来源保护",
      community_file: "社区文件",
      meta_only: "受来源保护，仅保留元数据",
      click_preview: "点击安全预览",
      click_download: "点击下载",
      collapsed_note: "该长文在来源页展示为折叠摘要。",
      loading_short: "正在加载…",
      retry_older: "重试加载更早动态",
      load_older: "加载更早动态",
      disclaimer: "社区内容为研究分享，仅供参考，不构成投资建议。",
    },
    recovery: {
      interrupted: "上次请求已中断，请重新发送",
      reconnecting: "连接中断，正在恢复任务状态",
      reconnect_failed: "连接已中断，未能恢复任务状态，请刷新页面后重试",
      attach_aria: "添加图片或文件",
    },
    community: {
      open_aria: "查看社区动态",
      open_aria_unread: "查看社区动态，有新动态",
    },
    actions: {
      logout: "退出",
      copy_aria: "复制",
      copied: "已复制",
      scroll_to_bottom_aria: "回到最新消息",
      share_aria: "分享",
      dismiss_aria: "关闭",
    },
    share: {
      brand_name: "HONE",
      brand_tagline: "你的 AI 投资助手",
      qr_caption: "扫码体验 HONE — 给投资人的 AI 助手",
      strings: {
        title: "分享对话",
        subtitle: "从最近 4 条消息里选择要分享的内容",
        preview_subtitle: "预览图片后保存、复制或分享到其他应用",
        preview_scroll_hint: "长图可在预览框内上下滚动，键盘用户可先聚焦预览框",
        generate_image: "生成分享图片",
        back_to_select: "重新选择消息",
        download: "下载图片",
        save_image: "保存图片",
        copy_image: "复制图片",
        copy_text: "仅复制文字",
        share: "系统分享",
        share_other_app: "分享到其他应用",
        close_aria: "关闭",
        success_download: "图片已保存",
        success_copy_image: "图片已复制",
        success_copy_text: "文字已复制",
        success_share: "已分享",
        save_image_hint: "请在系统分享面板选择保存图片，或长按图片存入相册",
        error_download: "保存失败，请重试",
        error_copy_image: "复制失败，请改用保存图片",
        error_copy_text: "复制文字失败，请手动选择文本",
        error_render: "生成图片失败，请减少消息后重试",
        error_share: "分享已取消",
        error_system_share: "系统分享失败，请改用保存图片或复制",
        role_user: "我",
        role_assistant: "HONE",
        nothing_selected: "请选择至少一条消息",
        rendering: "生成中…",
      },
    },
  },

  auth: {
    login: {
      title: "登录 HONE",
      subtitle: "使用手机号和短信验证码登录。",
      hint_sms: "目前是邀请制，请联系 bm@hone-claw.com 加入邀请名单。",
      phone_label: "手机号",
      phone_placeholder: "例如 13800138000",
      phone_aria: "手机号",
      code_label: "验证码",
      code_placeholder: "短信验证码",
      code_aria: "短信验证码",
      send_code: "获取验证码",
      sending_code: "发送中",
      resend_in: "{seconds}秒后重发",
      code_sent: "验证码已发送，请查看短信。",
      remember_30d: "保持登录（30 天）",
      submit_sms: "登录",
      loading: "登录中…",
    },
    tos: {
      prefix: "我已阅读并同意",
      terms: "《用户协议》",
      and: "和",
      privacy: "《隐私政策》",
      version_template: "（v{version}）",
    },
  },

  legal: {
    version_banner_template: "v{version} · {date} 生效",
    terms: {
      page_title: "用户协议",
      intro: "请仔细阅读以下条款。继续使用 HONE 即表示您接受本协议。",
      sections: [
        {
          title: "1. 协议接受与生效",
          body: [
            {
              kind: "p",
              parts: [
                "欢迎使用 HONE（以下简称“本服务”）。本服务由 ",
                { strong: "Snowdrift Capital LLC" },
                "（一家依据美国怀俄明州法律设立的有限责任公司，以下简称“我们”）运营。本《用户协议》（以下简称“本协议”）是您与我们之间就您使用本服务所订立的有效合同。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您在勾选同意或继续使用本服务时，即视为您已充分阅读并同意本协议全部条款。若您不同意本协议任何条款，请立即停止使用本服务。",
              ],
            },
          ],
        },
        {
          title: "2. 服务说明",
          body: [
            {
              kind: "p",
              parts: [
                "HONE 是一款面向个人投资者的研究与决策辅助工具，提供资料检索、对话式研究、投资笔记、定时提醒等能力。",
              ],
            },
            {
              kind: "p",
              parts: [
                { strong: "本服务不构成任何形式的投资建议、要约或推荐。" },
                "本服务输出的全部内容仅供参考，任何投资决策均应由您本人独立作出并自行承担相应风险与后果。",
              ],
            },
          ],
        },
        {
          title: "3. 账号与验证",
          body: [
            {
              kind: "p",
              parts: [
                "中国大陆渠道用户使用经我们登记的中国大陆手机号作为账号，并通过短信验证码完成身份验证；海外 Stripe 用户可使用购买邮箱和邮箱验证码完成身份验证。账号必须对应有效的邀请资格或会员权益。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您应妥善保管手机号码或邮箱、验证码与登录设备，不得将账号借予他人使用。若发现账号被未经授权使用，您应立即通知我们。",
              ],
            },
          ],
        },
        {
          title: "4. 用户行为规范",
          body: [
            {
              kind: "p",
              parts: ["使用本服务时，您承诺不从事以下行为，包括但不限于："],
            },
            {
              kind: "ul",
              items: [
                [
                  "违反美国联邦、州或地方适用法律法规，包括但不限于出口管制、OFAC 制裁、反洗钱、证券、隐私、网络安全及其他相关规定；",
                ],
                [
                  "违反中国大陆法律法规、监管要求、公序良俗或社会公共利益，或生成、传播、诱导生成中国法律法规及主流平台治理规则明确禁止或不倡导的内容；",
                ],
                [
                  "侵犯他人合法权益，包括知识产权、隐私权、名誉权、商业秘密、肖像权或其他财产或人身权利；",
                ],
                ["发布或传播威胁、骚扰、仇恨、歧视性、欺诈性或诽谤性内容；"],
                [
                  "发布、传播或索取淫秽色情、儿童性剥削材料、赌博、毒品交易、诈骗、暴力恐怖主义、极端主义或其他非法、有害内容；",
                ],
                [
                  "发布、传播或诱导生成危害国家安全、煽动颠覆国家政权、分裂国家、破坏国家统一、煽动民族仇恨、反华、政治敏感违法违规、损害公共秩序或违背公序良俗的内容；",
                ],
                [
                  "通过提示词注入、越狱、角色扮演、伪造系统指令、上下文污染或其他方式诱导本服务输出、协助、掩饰或放大违反前述规定的内容；",
                ],
                [
                  "对本服务进行反向工程、爬取、批量自动化访问、漏洞利用、规避访问控制或其他形式的滥用；",
                ],
                [
                  "上传、传播或部署恶意代码、垃圾信息、钓鱼链接或其他有害技术；",
                ],
                ["冒用他人身份、伪造账号信息或从事任何形式的欺诈行为。"],
              ],
            },
            {
              kind: "p",
              parts: [
                "若您违反前述规定，我们有权立即暂停或终止您的账号、取消使用资格、保留相关证据，并依法配合执法、监管或司法机关的合法请求。由此产生的全部法律责任由您本人承担。",
              ],
            },
          ],
        },
        {
          title: "5. 内容与知识产权",
          body: [
            {
              kind: "p",
              parts: [
                "本服务及其相关界面、文案、代码、商标等所有相关知识产权归我们或合法权利人所有，受著作权法及相关法律法规保护。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您在本服务中输入的内容（包括对话、笔记、附件等）的著作权归您本人所有。您授予我们必要的、为提供和改进本服务所需的非排他性使用权。",
              ],
            },
          ],
        },
        {
          title: "6. 第三方服务与数据源",
          body: [
            {
              kind: "p",
              parts: [
                "本服务可能调用第三方大型语言模型（LLM）、行情数据、搜索引擎、短信或邮件发送服务，并使用 Stripe 处理海外付款与订阅状态同步。第三方服务由其运营方独立提供，其稳定性、准确性及合规性以其官方声明为准。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您理解并同意，在调用第三方服务的过程中，我们可能向第三方传递必要的请求内容。我们将依照第三方服务条款选择正规、可信的合作方。",
              ],
            },
          ],
        },
        {
          title: "7. 订阅、自动续费、取消与退款",
          body: [
            {
              kind: "p",
              parts: [
                "海外年度会员通过 Stripe 结算。结账页会显示币种、价格、计费周期和适用税费；除非您在当前周期结束前取消，订阅将按结账时披露的周期自动续费并由原付款方式扣款。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您可通过账户页进入相应付款平台管理或取消订阅。取消通常在当前已付款周期结束时生效，不会自动退还已开始周期的费用。退款申请依购买页面披露、适用法律、我们明确作出的书面承诺及相应付款平台规则处理；适用法律要求退款的情形不受本条限制。",
              ],
            },
          ],
        },
        {
          title: "8. 服务变更、中断与终止",
          body: [
            {
              kind: "p",
              parts: [
                "我们可能因升级维护、安全事件、不可抗力或经营调整等原因暂停、变更或终止部分或全部服务。我们将在合理范围内事先通过本服务内通知或其他方式告知。",
              ],
            },
            {
              kind: "p",
              parts: [
                "若您严重违反本协议，我们有权立即暂停或终止向您提供服务，并保留依法追究责任的权利。",
              ],
            },
          ],
        },
        {
          title: "9. 免责与责任限制",
          body: [
            {
              kind: "p",
              parts: [
                "在适用法律允许的最大范围内，本服务以“现状”和“现有”方式提供。我们不对服务的连续性、准确性、完整性、及时性作出任何明示或默示保证。",
              ],
            },
            {
              kind: "p",
              parts: [
                "部分能力仅向有效会员开放。在适用法律允许的最大范围内，我们不对您因使用或无法使用本服务而遭受的任何间接、附带或后果性损失（包括但不限于投资或交易损失、数据丢失、利润损失等）承担责任；适用法律不得排除或限制的责任不受本条限制。",
              ],
            },
          ],
        },
        {
          title: "10. 协议变更与通知",
          body: [
            {
              kind: "p",
              parts: [
                "我们可能根据法律法规或业务调整需要修改本协议。修改后的协议将在本服务内公布，并标明版本号与生效日期。",
              ],
            },
            {
              kind: "p",
              parts: [
                "重大修改将以站内提醒等方式提示您再次确认。若您在协议变更后继续使用本服务，即视为您接受修改后的协议。",
              ],
            },
          ],
        },
        {
          title: "11. 适用法律与争议解决",
          body: [
            {
              kind: "p",
              parts: [
                "本协议的订立、效力、解释、履行及争议解决，均适用 ",
                { strong: "美国怀俄明州（State of Wyoming, USA）法律" },
                "（不含其法律冲突规则）。《联合国国际货物销售合同公约》（CISG）不适用于本协议。",
              ],
            },
            {
              kind: "p",
              parts: [
                "因本协议引起的或与之相关的任何争议，双方应首先以诚信原则协商解决；协商不成的，任一方可在美国怀俄明州 Sheridan 县有管辖权的州法院或联邦法院提起诉讼，双方对该等法院具有专属管辖权并放弃任何管辖权异议。",
              ],
            },
            {
              kind: "p",
              parts: [
                "在适用法律允许的最大范围内，您同意以个人名义而非作为任何集体诉讼或代表诉讼成员的身份与我们解决争议。",
              ],
            },
          ],
        },
        {
          title: "12. 联系方式",
          body: [
            {
              kind: "p",
              parts: [
                "若您对本协议有任何疑问、意见或建议，请通过以下方式联系我们：",
              ],
            },
            {
              kind: "ul",
              items: [
                [{ strong: "电子邮件：" }, { code: "bm@hone-claw.com" }],
                [
                  { strong: "GitHub Issue：" },
                  {
                    code: "https://github.com/B-M-Capital-Research/honeclaw/issues",
                  },
                ],
                [
                  { strong: "邮寄地址：" },
                  "Snowdrift Capital LLC, 30 N Gould St, Ste N, Sheridan, WY 82801, United States",
                ],
              ],
            },
            { kind: "p", parts: ["我们将在合理时间内回复并处理。"] },
          ],
        },
      ] as LegalSection[],
    },
    privacy: {
      page_title: "隐私政策",
      intro: "我们在乎您的数据。本政策说明 HONE 如何处理您的个人信息。",
      sections: [
        {
          title: "1. 引言与适用范围",
          body: [
            {
              kind: "p",
              parts: [
                "本《隐私政策》说明 HONE（运营方为 ",
                { strong: "Snowdrift Capital LLC" },
                "，一家依据美国怀俄明州法律设立的有限责任公司，以下简称“我们”）在提供服务过程中如何收集、使用、存储、共享和保护您的个人信息。本政策适用于您通过 HONE 网站及客户端使用本服务的全部场景。",
              ],
            },
            {
              kind: "p",
              parts: [
                "请您在使用本服务前完整阅读本政策。继续使用本服务即视为您已充分了解并同意本政策。",
              ],
            },
          ],
        },
        {
          title: "2. 我们收集的信息",
          body: [
            {
              kind: "p",
              parts: ["为提供服务，我们会按最小必要原则收集下列类别的信息："],
            },
            {
              kind: "ul",
              items: [
                [
                  { strong: "账号信息：" },
                  "中国大陆渠道用户的手机号、短信验证码核验结果与历史邀请记录；海外渠道用户的购买邮箱、邮箱验证码核验结果，以及 Stripe 的 Customer、Subscription、Invoice、产品、价格、状态、续费周期和 webhook 事件标识；",
                ],
                [
                  { strong: "付款信息：" },
                  "海外付款由 Stripe 处理。我们接收用于确认权益的客户、订阅和账单状态标识，但不接收或存储完整银行卡号、安全码或磁条数据；",
                ],
                [
                  { strong: "使用数据：" },
                  "对话记录、提问与回复内容、上传的附件、笔记与定时任务；",
                ],
                [
                  { strong: "设备与日志：" },
                  "IP 地址、浏览器类型、访问时间戳、错误日志、Cookie 标识；",
                ],
                [
                  { strong: "授权事件：" },
                  "用户协议与隐私政策的接受版本与时间。",
                ],
              ],
            },
          ],
        },
        {
          title: "3. 使用目的",
          body: [
            { kind: "p", parts: ["我们使用上述信息用于以下目的："] },
            {
              kind: "ul",
              items: [
                ["身份认证、登录会话维持、账号风控与频率限制；"],
                ["确认购买渠道、会员资格、续费状态与产品访问权限；"],
                ["调用大型语言模型与外部数据源以完成您发起的查询；"],
                ["记录会话上下文以提供连续对话能力；"],
                ["系统故障排查、安全事件响应与服务优化。"],
              ],
            },
          ],
        },
        {
          title: "4. 存储、保留期与安全",
          body: [
            {
              kind: "p",
              parts: [
                "您的账号与对话数据默认存储于本服务的本地 SQLite 数据库中，并可按部署配置同步到服务端数据库。短信或邮箱验证码由相应发送服务交付；HONE 仅保存验证码摘要和有效期，不存储验证码明文。",
              ],
            },
            {
              kind: "p",
              parts: [
                "我们采用 HTTPS 加密传输、最小权限访问控制、服务端会话 Cookie 等技术与管理措施，保护您的信息安全。在法律允许范围内，我们将在为完成相应目的所必需的期间内保留您的信息。",
              ],
            },
          ],
        },
        {
          title: "5. 信息共享与第三方",
          body: [
            {
              kind: "p",
              parts: [
                "为完成身份验证、付款权益同步和您发起的查询，我们可能向以下类别的第三方服务方传递必要信息：",
              ],
            },
            {
              kind: "ul",
              items: [
                ["短信与邮件服务商（用于发送登录验证码）；"],
                ["Stripe（用于处理海外付款、创建订阅管理会话并同步产品、账单和续费状态）；"],
                ["大型语言模型提供方（用于生成回复）；"],
                ["行情数据与搜索数据源（用于补充查询所需的市场或公开信息）。"],
              ],
            },
            {
              kind: "p",
              parts: [
                "除上述必要场景以及法律法规另有规定外，我们不会向任何第三方出售或出租您的个人信息。",
              ],
            },
          ],
        },
        {
          title: "6. Cookie 与追踪",
          body: [
            {
              kind: "p",
              parts: [
                "我们使用名为 ",
                { code: "hone_web_session" },
                " 的 HTTP-only Cookie 维持登录态。该 Cookie 在您勾选“保持登录”时有效期为 30 天，否则为 1 天。",
              ],
            },
            { kind: "p", parts: ["我们不使用第三方广告追踪 Cookie。"] },
          ],
        },
        {
          title: "7. 未成年人保护",
          body: [
            {
              kind: "p",
              parts: [
                "本服务面向 18 周岁以上具有完全民事行为能力的成年人。若您是未成年人，请在监护人指导下使用本服务。我们不会主动收集未成年人的个人信息。",
              ],
            },
          ],
        },
        {
          title: "8. 数据处理地点与跨境传输",
          body: [
            {
              kind: "p",
              parts: [
                "我们的数据处理基础设施位于 ",
                { strong: "美国" },
                "（运营方所在地）。我们调用的语言模型与数据源服务商主要位于美国及其他司法管辖区。在您使用本服务时，您的相关个人信息和查询内容将被传输至并存储于美国。",
              ],
            },
            {
              kind: "p",
              parts: [
                "若您位于美国境外（包括欧洲经济区、英国、中华人民共和国大陆地区或其他任何司法管辖区），您理解并同意您的个人信息将跨境传输至美国进行处理。我们将选择具备合规资质的合作方，并采取必要的技术与组织措施保护信息安全。",
              ],
            },
          ],
        },
        {
          title: "9. 您的权利",
          body: [
            {
              kind: "p",
              parts: ["就您的个人信息，您依据适用法律享有下列权利："],
            },
            {
              kind: "ul",
              items: [
                ["访问、更正您的账号资料；"],
                ["修改您的登录密码；"],
                ["请求删除您的账号及关联数据；"],
                ["撤回您此前给出的同意；"],
                ["请求获取您提供给我们的个人信息副本（数据可携带权）；"],
                ["反对或限制特定的个人信息处理活动。"],
              ],
            },
            {
              kind: "p",
              parts: [
                "如您是 ",
                { strong: "美国加州居民" },
                "，根据《加州消费者隐私法》（CCPA / CPRA），您还享有了解我们收集与共享个人信息类别的权利、请求删除已收集信息的权利，以及不因行使权利而受到歧视的权利。我们 ",
                { strong: "不向第三方“出售”" },
                " 您的个人信息。",
              ],
            },
            {
              kind: "p",
              parts: [
                "如您位于 ",
                { strong: "欧洲经济区或英国" },
                "，根据《通用数据保护条例》（GDPR / UK GDPR），您还享有向所在地数据保护监管机构投诉的权利。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您可在“个人页面”中行使前三项权利，或通过下文联系方式与我们联系。撤回同意可能导致您无法继续使用部分功能。我们将在合理时间内（通常 30 日内）回应您的请求。",
              ],
            },
          ],
        },
        {
          title: "10. 政策更新",
          body: [
            {
              kind: "p",
              parts: [
                "我们可能根据法律法规变化或业务调整需要更新本政策。更新后的政策将在本服务内公布，并标明版本号与生效日期；重大变更将以站内提醒等方式向您提示。",
              ],
            },
          ],
        },
        {
          title: "11. 联系方式",
          body: [
            {
              kind: "p",
              parts: [
                "若您对本政策或您的个人信息处理有任何疑问、意见或投诉，请通过以下方式联系我们：",
              ],
            },
            {
              kind: "ul",
              items: [
                [{ strong: "电子邮件：" }, { code: "bm@hone-claw.com" }],
                [
                  { strong: "GitHub Issue：" },
                  {
                    code: "https://github.com/B-M-Capital-Research/honeclaw/issues",
                  },
                ],
                [
                  { strong: "邮寄地址：" },
                  "Snowdrift Capital LLC, Attn: Privacy, 30 N Gould St, Ste N, Sheridan, WY 82801, United States",
                ],
              ],
            },
            { kind: "p", parts: ["我们将在合理时间内回复并妥善处理。"] },
          ],
        },
      ] as LegalSection[],
    },
  },

  footer: {
    tagline: "磨砺认知，剔除噪音",
    mantra: "HONE · 磨砺认知 · 剔除噪音",
    copyright:
      "© 2026 Snowdrift Capital LLC · Sheridan, WY, USA · 开源代码遵循 MIT License。",
    columns: {
      product: {
        title: "产品",
        items: [
          { label: "首页", href: "/" },
          { label: "路线图", href: "/roadmap" },
          { label: "Blog", href: "/blog" },
          { label: "对话", href: "/chat" },
          { label: "个人", href: "/me" },
        ],
      },
      resources: {
        title: "资源",
        items: [
          {
            label: "GitHub",
            href: "https://github.com/B-M-Capital-Research/honeclaw",
          },
          {
            label: "中文文档",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
          },
          {
            label: "安装方式",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动",
          },
          {
            label: "代码库地图",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
          },
        ],
      },
      community: {
        title: "社群",
        items: [
          { label: "Discord", href: "#" },
          { label: "知识星球", href: "#" },
          { label: "微信群", href: "#" },
          { label: "内容号", href: "#" },
        ],
      },
      legal: {
        title: "条款",
        items: [
          { label: "用户协议", href: "/terms" },
          { label: "隐私政策", href: "/privacy" },
        ],
      },
    },
  },
};

const CONTENT_EN: typeof CONTENT_ZH = {
  nav: {
    logo_tagline: "HONE",
    home: "Home",
    community: "Community",
    roadmap: "Roadmap & Docs",
    blog: "Blog",
    me: "Account",
    chat: "Chat",
    plan: "Pricing",
    buy: "Get Full Access",
    more: "More",
    back_home: "Home",
    menu_aria: "Menu",
    locale_zh: "中文",
    locale_en: "EN",
    contact_label: "Contact",
    contact_title: "Contact us",
    contact_wechat_label: "WeChat",
    contact_email_label: "Email",
    contact_wechat: "xiaobamang6677",
    contact_wechat_group: "WeChat community",
    contact_wechat_hint_prefix: "Contact",
    bilibili_label: "Bilibili",
    youtube_channel_name: "B&M Capital Research",
    contact_email: "official@hone-claw.com",
    github_url: "https://github.com/B-M-Capital-Research/honeclaw",
  },

  hero: {
    eyebrow: "HONE · AI INVESTMENT DISCIPLINE",
    headline_1: "Not a chatbot that flatters you.",
    headline_2: "A research-discipline guardian.",
    description:
      "Calm, restrained, long-memory, research-first. HONE is an open-source AI agent built for serious investors — it helps you set and keep your research discipline, not tell you what you want to hear.",
    cta_primary: "Enter Chat",
    cta_secondary: "View Roadmap",
    scroll_hint: "Scroll",
    stat_1: { value: "Rust", label: "Core Engine" },
    stat_2: { value: "7", label: "Channels" },
    stat_3: { value: "MIT", label: "License" },
  },

  home_page: {
    roadmap_button: "Roadmap",
    roadmap_slide_tag: "ROADMAP",
    hero_slogan:
      "Not a chatbot that flatters you, but a ruthless defender of your investment discipline.",
    start_trial: "Start Now",
    video_demo: "VIDEO DEMO",
    view_full_roadmap: "View Full Roadmap",
    zoom_hint: "Zoom In",
    blog_eyebrow: "Engineering Blog",
    blog_title: "Why HONE chose Rust",
    blog_desc:
      "A field report on moving from Python + Node.js to Rust, and what it means for context, stability, and multi-endpoint engineering in the AI Coding era.",
    blog_cta: "Read article",
    plan_eyebrow: "PLAN & PRICING",
    plan_title: "Open source forever, full access in one subscription",
    plan_desc: "The HONE core stays MIT-licensed and self-hostable. Full access adds weekly live deep dives, the VIP community, and complete research notes.",
    plan_cta: "View pricing",
  },

  plan: {
    eyebrow: "HONE · B&M RESEARCH",
    share_title: "B&M Research Membership",
    share_sub: "Six posters that explain the full service: original research notes, earnings previews, weekly live sessions, and the investor community. Share this page with anyone who wants in.",
    title: "Two editions, plain and simple",
    sub: "The core stays open source and self-hostable; Full Access bundles the live sessions, community, and research notes.",
    host: {
      name: "Hari",
      role: "U.S. equity lead · creator of HONE",
      bio: "Twenty years in U.S. equities. Videos are free, research notes and the community are a yearly subscription, and the HONE research assistant is open source — this page gathers every entry point.",
      stats: [
        { value: "20 yrs", label: "in U.S. equities" },
        { value: "100%+", label: "recent annual returns*" },
        { value: "300+", label: "original notes / year" },
      ],
      stats_note: "* The host's historical performance. Past performance does not guarantee future results.",
    },
    channel: {
      label: "Free content",
      title: "Hari on YouTube",
      handle: "@Hari老王 · U.S. equity deep dives, weekly",
      cta: "Open channel",
      videos_label: "Recent videos",
      bilibili_title: "Synced on Bilibili",
      bilibili_desc: "Mirrored uploads for viewers in China",
      bilibili_cta: "Open Bilibili",
    },
    product_card: {
      label: "Research tool",
      title: "HONE research assistant",
      desc: "The open-source AI research workbench built by Hari's team: deep single-name research, portfolio tracking, earnings calendar, and proactive pushes. Self-hostable.",
      cta_chat: "Start chatting",
      cta_github: "GitHub repository",
    },
    member: {
      label: "Membership",
      title: "B&M Research Membership",
      benefits: [
        { title: "Zhishixingqiu", desc: "300+ original long-form notes a year, 100+ earnings previews a quarter" },
        { title: "Weekly live sessions", desc: "Thursday deep dives — ask anything live" },
        { title: "VIP group", desc: "Real-time discussion with 500+ experienced investors" },
        { title: "Curated feed", desc: "Hand-picked material with the key points highlighted" },
      ],
      trial_cta: "Join the free group first",
      trial_hint: "Add support on WeChat to try the free group before you pay.",
      activation_cta: "Already subscribed? Activate or restore HONE",
    },
    posters_label: "Six posters that explain the full service",
    posters_hint: "Tap any poster to enlarge; long-press or right-click to save and share.",
    socials_label: "Channels",
    socials: { zsxq: "Zhishixingqiu", wechat: "WeChat" },
    trial_line: "Want a taste first? Contact support to join the free group.",
    free: {
      name: "Open Source",
      price: "Free",
      period: "MIT licensed · on GitHub",
      desc: "Run the full HONE on your own machine.",
      notes_label: "Before you start",
      notes: [
        "Bring your own model provider (API keys not included)",
        "Channels like Feishu / Discord / iMessage need your own setup",
        "No in-depth community research material",
        "No community company-research discussions",
      ],
      cta: "Get it on GitHub",
    },
    full: {
      name: "Full Access",
      badge: "Recommended",
      price: "$199.99",
      period: "/ year",
      promos: ["USD · Stripe"],
      desc: "Live deep dives, the VIP community, research notes, and unlimited HONE — one subscription.",
      features: [
        "Thursday live deep dives — the host walks through companies in depth, ask anything live",
        "VIP group with 500+ investors — in-depth research material and real-time updates",
        "Zhishixingqiu & community — full company reports, valuations, and strategy write-ups",
        "Unlimited HONE — every question gets a timely answer in the community",
      ],
      cta: "Subscribe",
      qr_title: "Scan to join",
      qr_hint: "Long-press or right-click to save; scan to subscribe.",
    },
    support: {
      title: "Questions? Add our support WeChat",
      desc: "Scan to add our WeCom support — purchase or usage questions welcome anytime.",
    },
    foot: "Service provided by B&M Capital Research. The open-source edition is never limited by subscriptions.",
    close_aria: "Close",
  },

  trust: {
    section_label: "WHY HONE",
    items: [
      {
        symbol: "◈",
        title: "Discipline over opinion",
        body: "HONE will not flatter your position. Every conversation is constrained by research discipline — it actively surfaces and pushes back on emotion-driven decisions.",
      },
      {
        symbol: "∞",
        title: "Long-term research memory",
        body: "Deep profiles of each company grow across conversations. Context persists across sessions, building a personal, ever-growing research knowledge base.",
      },
      {
        symbol: "✦",
        title: "Multi-angle judgment",
        body: "Built-in pro/con dialectics and a zero-hallucination protocol find signal in the noise — instead of repackaging your feelings as analysis.",
      },
    ],
  },

  cases: {
    section_label: "REAL WORKFLOWS",
    section_sub: "How HONE fits into your research routine",
    placeholder_suffix: "scenario screenshot (placeholder)",
    items: [
      {
        tag: "Stock analysis",
        title: "Systematically research a company in depth",
        body: "From financials to competitive landscape, HONE helps you assemble a complete research framework, logging every key assumption and risk factor.",
        image: "/company_profile.png",
      },
      {
        tag: "Portfolio tracking",
        title: "Track holdings, nudge on key moments",
        body: "Set stop-loss / take-profit logic; HONE checks your portfolio on a schedule and pushes an alert the moment your conditions trigger.",
        image: null as string | null,
      },
      {
        tag: "Scheduled tasks",
        title: "Trigger a weekly review every Friday",
        body: "Hand fixed workflows to HONE. Weekly reviews, monthly summaries, key-moment checks — all run themselves at the time you set.",
        image: null as string | null,
      },
      {
        tag: "Long-term profile",
        title: "Build a company's personal dossier",
        body: "Each research result is archived into the company profile. Next time you ask, HONE calls back the full history — smarter with every use.",
        image: "/hone_solution.jpg",
      },
      {
        tag: "Cross-platform notifications",
        title: "Get HONE in iMessage / Lark",
        body: "Not just the web. HONE reaches you through iMessage, Lark, Discord and more — in whatever channel you're already using.",
        image: "/hone_channels.jpg",
      },
    ],
  },

  video: {
    section_label: "SEE HONE IN ACTION",
    title: "Lao Wang on HONE: the research AI agent in practice",
    description:
      "From onboarding to deep research, learn in ten minutes how HONE changes the way you work. Full walkthrough of stock analysis, portfolio tracking, scheduled tasks, and more.",
    video_url: "https://www.youtube.com/watch?v=hJr-81OdYcQ",
    thumbnail: "/hone_introduction.jpg",
    duration: "~10 min",
    coverage:
      "Covered: deep single-stock research, portfolio tracking, scheduled tasks, multi-channel demo",
    url_placeholder: "Video link not configured yet (set video_url)",
  },

  capabilities: {
    section_label: "CORE CAPABILITIES",
    items: [
      {
        symbol: "⚡",
        title: "Research discipline",
        body: "Constrains emotional decisions in-conversation. It doesn't echo your thinking — it questions it.",
      },
      {
        symbol: "◈",
        title: "Company profiles & long memory",
        body: "A persistent dossier per company; research compounds across sessions into a real knowledge asset.",
      },
      {
        symbol: "∞",
        title: "Scheduled tasks & alerts",
        body: "Scheduled workflows that run themselves: reviews, portfolio checks, key-moment alerts — all on the timing you set.",
      },
      {
        symbol: "✦",
        title: "Multi-channel access",
        body: "Web, iMessage, Lark / Feishu, Discord, Telegram, CLI — HONE on whichever channel you already live in.",
      },
      {
        symbol: "⌘",
        title: "Rust-powered stability",
        body: "Core engine built in Rust — low latency, high reliability, no drift or crash on long runs.",
      },
      {
        symbol: "ℹ",
        title: "Programmable research OS",
        body: "Custom skills, dynamic task chains, cross-session memory — compose a workflow that's fully yours.",
      },
    ],
  },

  community: {
    section_label: "JOIN THE COMMUNITY",
    section_sub: "Find people who take research seriously",
    qr_label: "QR code",
    tier1: [
      {
        key: "wechat_group",
        tier_label: "Free",
        name: "WeChat group",
        desc: "Scan to join — discuss methodology, give feedback, share notes",
        qr: null as string | null,
        cta: "Scan to join",
      },
      {
        key: "author_wechat",
        tier_label: "Author",
        name: "Lao Wang's WeChat",
        desc: "Direct product feedback; priority notice on important updates",
        qr: null as string | null,
        cta: "Add contact",
      },
    ],
    tier2: [
      {
        key: "discord",
        name: "Discord",
        desc: "English community discussion",
        url: "#",
        label: "Open",
        symbol: "⚡",
      },
      {
        key: "zsxq",
        name: "Zhishixingqiu",
        desc: "Paid deep-dive content",
        url: "#",
        label: "Paid",
        symbol: "◈",
      },
      {
        key: "vip",
        name: "VIP group",
        desc: "Premium / private feature preview",
        url: "#",
        label: "Invite",
        symbol: "✦",
      },
      {
        key: "content",
        name: "Content channel",
        desc: "Research methodology & product updates",
        url: "#",
        label: "Follow",
        symbol: "∞",
      },
    ],
  },

  repo: {
    section_label: "OPEN SOURCE",
    section_sub: "Made by B&M Capital Research. MIT licensed.",
    items: [
      {
        title: "GitHub repo",
        desc: "Star, fork, open issues, help build in the open",
        url: "https://github.com/B-M-Capital-Research/honeclaw",
        tag: "Source",
        icon: "⌘",
      },
      {
        title: "Chinese docs",
        desc: "README, usage guide, case studies",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
        tag: "Docs",
        icon: "◈",
      },
      {
        title: "Install guide",
        desc: "macOS desktop + self-hosted server setup",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动",
        tag: "Install",
        icon: "⚡",
      },
      {
        title: "Repository map",
        desc: "Module structure, data flow, and runtime boundaries",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
        tag: "Tech",
        icon: "∞",
      },
      {
        title: "Case studies",
        desc: "Real-world research scenarios",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md",
        tag: "Cases",
        icon: "✦",
      },
      {
        title: "Contributing",
        desc: "How to contribute code, ideas, and skills",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md",
        tag: "Contribute",
        icon: "ℹ",
      },
    ],
  },

  roadmap: {
    hero_title: "Roadmap & Docs",
    hero_sub:
      "Transparent, pragmatic, long-term. Here's what HONE does today, what's next, and how to bring it into your research workflow.",
    hero_meta: "ROADMAP · DOCS · API",
    sidebar_title: "ON THIS PAGE",
    version: "v0.12.4",

    toc: [
      { id: "quick-start", label: "Quick Start", sub: "Quick Start" },
      { id: "capabilities", label: "Capabilities", sub: "Capability Matrix" },
      { id: "channels", label: "Channels", sub: "Channels" },
      { id: "architecture", label: "Architecture", sub: "Architecture" },
      { id: "skills", label: "Built-in Skills", sub: "Skills" },
      { id: "roadmap", label: "Roadmap", sub: "Roadmap" },
      { id: "boundary", label: "Open Source", sub: "Open Source" },
      { id: "docs", label: "Documentation", sub: "Docs" },
      { id: "contributing", label: "Contributing", sub: "Contributing" },
      { id: "faq", label: "FAQ", sub: "FAQ" },
    ],

    sections: {
      quick_start: {
        eyebrow: "§ 01 · QUICK START",
        title: "Quick Start",
        intro:
          "Three paths to run HONE: the one-line installer, Homebrew, or source. After install, use `hone-cli start` for the full runtime or `hone-cli web admin-ui` / `hone-cli web user-ui` to open the admin console or public user app.",
      },
      capabilities: {
        eyebrow: "§ 02 · CAPABILITY MATRIX",
        title: "Capability Matrix",
        legend: { stable: "Production", beta: "Preview", planned: "Planned" },
      },
      channels: {
        eyebrow: "§ 03 · CHANNELS",
        title: "Channels",
        intro:
          "HONE is a multi-channel research agent. Each channel is an independent process — start, stop, and configure them on their own.",
      },
      architecture: {
        eyebrow: "§ 04 · ARCHITECTURE",
        title: "Architecture",
        intro:
          "Rust core · multi-engine abstraction · SolidJS frontend. The public user app, admin console, and channel processes share backend capabilities while staying separated by interface, port, and process boundary; Cloud PG / OSS is taking over runtime storage in stages.",
        footnote_prefix: "Full module walkthrough in",
        footnote_link: "docs/repo-map.md ↗",
      },
      skills: {
        eyebrow: "§ 05 · BUILT-IN SKILLS",
        title: "Built-in Skills",
        intro_prefix:
          "HONE's skills are invoked by the model from context. Below are the 16 public skills in the",
        intro_suffix: "directory.",
      },
      roadmap: {
        eyebrow: "§ 06 · ROADMAP",
        title: "Product Roadmap",
        intro_lead: "We ship in",
        intro_highlight: "Now / Next / Later",
        intro_trail: "phases. Exact releases live on GitHub Releases.",
      },
      boundary: {
        eyebrow: "§ 07 · OPEN SOURCE BOUNDARY",
        title: "Open Source Boundary",
        intro:
          "MIT licensed. The repo contains a fully working core; premium additions stay closed but don't block the main flow.",
        open_label: "Open source",
        closed_label: "Private / paid",
      },
      docs: {
        eyebrow: "§ 08 · DOCUMENTATION",
        title: "Documentation",
      },
      contributing: {
        eyebrow: "§ 09 · CONTRIBUTING",
        title: "Contributing",
        intro:
          "HONE is open source. Every kind of contribution is welcome — not just code.",
      },
      faq: {
        eyebrow: "§ 10 · FAQ",
        title: "FAQ",
      },
    },

    install: {
      tabs: [
        { key: "curl" as const, label: "curl | bash", badge: "Recommended" },
        { key: "brew" as const, label: "Homebrew", badge: null },
        { key: "source" as const, label: "Source / CLI", badge: null },
      ],
      requirements_prefix: "Requirements:",
      curl: [
        "# macOS / Linux one-line install (recommended)",
        "$ curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
        "$ hone-cli start",
      ],
      brew: [
        "# Homebrew tap (macOS / Linux)",
        "$ brew install B-M-Capital-Research/honeclaw/honeclaw",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
        "$ hone-cli start",
      ],
      source: [
        "# Source dev mode (local CLI build-and-start)",
        "$ git clone https://github.com/B-M-Capital-Research/honeclaw",
        "$ cd honeclaw",
        "$ cargo run -p hone-cli -- start --build",
      ],
    },

    requirements:
      "macOS 13+ / Linux x86_64 / arm64 · first source build ~10 min (Rust / Bun required locally)",

    architecture_points: [
      {
        title: "CLI startup",
        desc: "`hone-cli doctor / onboard / start` handles health checks, guided setup, and starting hone-console-page plus enabled channels; `hone-cli web admin-ui` / `hone-cli web user-ui` can locate or start the admin console and public user app; source mode uses `cargo run -p hone-cli -- start --build` and passes the located `hone-mcp` binary to child processes as `HONE_MCP_BIN`.",
      },
      {
        title: "Public user app",
        desc: "The public user app routes `/`, `/roadmap`, `/blog`, `/blog/:slug`, `/chat`, `/me`, `/activate`, `/portfolio`, `/terms`, and `/privacy`, with a dev-only `/__share-preview` page for share-card QA; `/blog` is a bilingual static long-form content surface, with Cloudflare Worker metadata for crawler-friendly article cards; mainland-China `/chat` admission uses Aliyun behavior captcha plus phone/SMS verification from the admin invite list, while overseas users verify email through `/activate` to purchase with Stripe or restore membership access, and `/me` renders the server-owned unified billing state; the client uses a collapsible desktop left rail plus full-height conversation workspace and gathers navigation, account access, recent conversation history, contact links, and GitHub stars in that rail while supporting assistant-reply copy, image sharing, non-image generated-file downloads, history review, and image / file attachments that pass through shared ingest before the runner reads them; `/portfolio` is a read-only investment context surface for push context and company-profile entry points, and the public backend is scoped to `/api/public/*`, including `/api/public/digest-context` and `/api/public/company-profile` for the signed-in user's investment mainline and single-name profiles, `/api/public/file` for downloadable generated artifacts, and `/api/public/v1/chat/completions` for API-key-authenticated OpenAI-compatible chat.",
      },
      {
        title: "Storage and cloud runtime",
        desc: "`cloud.postgres` / `cloud.oss` are first-class config sections as of v0.12.4 and reference real credentials through env vars; once OSS is configured, public Web uploads write under `public-uploads/...` and return `oss://bucket/key`, while `/api/public/image` and `/api/public/file` can proxy managed objects; `/api/meta` reports capabilities such as `cloud_runtime`, `cloud_postgres`, `cloud_oss`, `oss_file_proxy`, and the local durable dependency count. Current main already has PG hot paths for sessions, Web invites/auth sessions, conversation quota, cron jobs/runs, due-job claims, the skill registry, notification prefs, portfolio, LLM audit, and company profile files; `hone-cli cloud doctor / migrate / object-bench` covers cloud health checks, local `data/` dry-runs or idempotent imports, and OSS/R2 small-object latency checks. `cloud.strict_no_local_storage=true` blocks startup when the current config still has durable local dependencies; with cloud mode plus both PG and OSS configured, the known durable data plane is no longer blocked by those local stores.",
      },
      {
        title: "Admin console",
        desc: "The admin console includes dashboard, sessions, skills, tasks, users, research, llm-audit, task-health, notifications, schedule, settings, and logs for operators; the users page groups holdings, company profiles, sessions, and research tasks by actor, and company profiles support actor-space listing, detail review, deletion, zip export, import preview, and conflict-aware import.",
      },
      {
        title: "Agent engine layer",
        desc: "Recommended agent engines are Codex ACP, HONE Cloud, and OpenCode ACP; Gemini CLI and Codex CLI remain supported. Codex ACP defaults to GPT-5.6 Sol with xhigh reasoning effort. LLM credentials use `config.yaml` as the only source of truth, and both OpenRouter and generic OpenAI-compatible providers support `llm.providers.*.api_key/api_keys` key pools so the runtime can try the next key after upstream 429 / quota failures; `gemini_acp` is kept only as migration config, not a runtime entry point.",
      },
      {
        title: "Events and tasks",
        desc: "Cron scheduled tasks, event-engine digests, `/missed` recovery, notification preferences, and channel delivery share one Rust backend and execution-history model. Scheduled results are persisted to conversation history before any live hinting, so nothing is lost or falsely reported when the browser is offline; failures surface as productized, traceable messages. Web, Lark, Discord, and the scheduler all read the same Cron, portfolio, and cloud configuration.",
      },
    ],

    capability_matrix: [
      {
        group: "Research core",
        rows: [
          {
            name: "Research discipline & zero-hallucination protocol",
            status: "stable",
            note: "hardened system prompt",
          },
          {
            name: "Company profiles & long memory",
            status: "stable",
            note: "company profile skill + admin import/export",
          },
          {
            name: "Stock research / deep research",
            status: "stable",
            note: "stock_research + deep_stock_research",
          },
          {
            name: "Portfolio tracking & alerts",
            status: "stable",
            note: "portfolio_management + cron",
          },
          {
            name: "Valuation / selection / position advice",
            status: "stable",
            note: "stock_research covers valuation and screening; position_advice covers sizing changes",
          },
          {
            name: "Chart & image generation",
            status: "stable",
            note: "chart_visualization / image_generation",
          },
          {
            name: "Public chat workbench and sharing",
            status: "stable",
            note: "sidebar history + html2canvas + qrcode + markdown rendering + CJK code font + attachment download cards",
          },
          {
            name: "Vector-augmented memory",
            status: "planned",
            note: "planned",
          },
        ],
      },
      {
        group: "Runtime",
        rows: [
          {
            name: "Rust core engine",
            status: "stable",
            note: "Tokio · axum · SSE",
          },
          {
            name: "SolidJS frontend",
            status: "stable",
            note: "Vite · Tailwind v4 · stale asset recovery",
          },
          {
            name: "Public blog and docs surface",
            status: "stable",
            note: "bilingual Markdown posts + article routes + Cloudflare share metadata",
          },
          { name: "Tauri desktop", status: "stable", note: "macOS released" },
          {
            name: "Multi-engine abstraction",
            status: "stable",
            note: "Gemini CLI · Codex CLI/ACP · OpenCode ACP · Hone Cloud",
          },
          {
            name: "LLM provider key pools and upstream error fidelity",
            status: "stable",
            note: "config.yaml llm.providers.*.api_key/api_keys · OpenRouter / OpenAI-compatible fallback",
          },
          {
            name: "Cloud PG / OSS runtime migration",
            status: "beta",
            note: "PG hot paths for sessions / web auth / quota / cron; OSS public-upload proxy and migration tooling remain beta",
          },
          {
            name: "Channel finalization and side-effect confirmations",
            status: "stable",
            note: "response_finalizer + output sanitizer recover successful side-effect confirmations and hide internal paths / skill degradation text",
          },
          {
            name: "Windows / Linux desktop",
            status: "planned",
            note: "Tauri multi-platform packaging",
          },
        ],
      },
      {
        group: "Extensions",
        rows: [
          {
            name: "Cron scheduled tasks",
            status: "stable",
            note: "scheduled_task skill + /api/cron-jobs + execution history / heartbeat / quiet_hours / guard / Web SSE / Discord send-failure diagnostics / ACP transport disconnect regressions",
          },
          {
            name: "Custom skills",
            status: "stable",
            note: "skill_manager · create_skill.sh",
          },
          {
            name: "MCP protocol",
            status: "stable",
            note: "hone-mcp server + HONE_MCP_BIN / HONE_CONFIG_PATH / HONE_DATA_DIR absolutization and propagation",
          },
          {
            name: "Admin HTTP + SSE API",
            status: "stable",
            note: "hone-web-api admin surface",
          },
          {
            name: "Public SMS login with captcha gate",
            status: "stable",
            note: "Aliyun Captcha + Aliyun SMS + admin Web invite list",
          },
          {
            name: "Public OpenAI-compatible Chat API",
            status: "beta",
            note: "user API keys + /api/public/v1/chat/completions",
          },
          {
            name: "Per-user notification prefs",
            status: "stable",
            note: "notification_preferences skill + settings page + config-level mute",
          },
          {
            name: "Missed / truncated event recovery",
            status: "stable",
            note: "missed skill + missed_events tool",
          },
          {
            name: "Public skill marketplace",
            status: "planned",
            note: "community sharing",
          },
        ],
      },
    ],

    channels: [
      {
        name: "Web",
        icon: "⚡",
        status: "stable",
        desc: "Invite-only chat with phone + SMS login; scheduled results persist to history and use SSE for live hints",
      },
      {
        name: "iMessage",
        icon: "✦",
        status: "stable",
        desc: "Native macOS SMS integration",
      },
      {
        name: "Lark / Feishu",
        icon: "◈",
        status: "stable",
        desc: "Two-way Feishu bot with scheduler heartbeat pushes and loop supervision recovery",
      },
      {
        name: "Discord",
        icon: "∞",
        status: "stable",
        desc: "Bot integration; scheduler send failures keep redacted error reasons",
      },
      {
        name: "Telegram",
        icon: "⌘",
        status: "stable",
        desc: "Bot API integration",
      },
      {
        name: "CLI",
        icon: "ℹ",
        status: "stable",
        desc: "Streaming CLI chat",
      },
      {
        name: "MCP",
        icon: "✧",
        status: "stable",
        desc: "Run as MCP server inside Claude / Cursor, etc.",
      },
    ],

    skills: [
      {
        name: "stock_research",
        desc: "Single-stock research, valuation, conditional screening",
      },
      {
        name: "deep_stock_research",
        desc: "1–2 hour deep research tasks (admin only)",
      },
      {
        name: "company_portrait",
        desc: "Maintain company profiles, theses, and event timelines",
      },
      {
        name: "portfolio_management",
        desc: "Add, trim, rebalance, validate tickers",
      },
      {
        name: "position_advice",
        desc: "Suggest adds / trims from market + position context",
      },
      {
        name: "market_analysis",
        desc: "Macro, policy, sector momentum, index calls",
      },
      {
        name: "gold-analysis",
        desc: "Gold, gold ETFs, and miners — macro and positioning",
      },
      {
        name: "scheduled_task",
        desc: "Register / modify / cancel scheduled pushes",
      },
      {
        name: "missed",
        desc: "Inspect digest items that were capped, cooled down, filtered, or folded",
      },
      {
        name: "chart_visualization",
        desc: "Trend, comparison, distribution, scatter charts",
      },
      {
        name: "image_generation",
        desc: "Portfolio screenshots, research visuals, explainers",
      },
      {
        name: "image_understanding",
        desc: "Parse readable image inputs; Web direct image attachments now reuse shared ingest and fall back to productized retry guidance",
      },
      {
        name: "pdf_understanding",
        desc: "Parse PDFs (filings, reports) into key points and risks",
      },
      { name: "skill_manager", desc: "View / create / edit HONE skills" },
      {
        name: "notification_preferences",
        desc: "Tune your own push prefs in natural language (severity, portfolio-only, kind allow/block)",
      },
      {
        name: "hone_admin",
        desc: "Inspect and modify HONE source & config (admin)",
      },
    ],

    now: {
      label: "Shipping today",
      items: [
        "Web research workbench: three-pane desktop layout plus a drawer-style mobile navigation, with full conversation history",
        "Research sharing: export any answer as a branded long image, or copy image / text to share",
        "Images & files: upload images / PDFs into the conversation; generated CSV / XLSX / PDF files are downloadable",
        "macOS desktop app with a bundled backend — download and run",
        "7 channels: Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "17 built-in research skills: stock analysis, portfolio, earnings research, valuation screening, charts, PDF, scheduled tasks, notification preferences, and more",
        "Research discipline & zero-hallucination protocol: no flattering your positions — verify first, conclude second",
        "Company profiles + cross-session long memory: research context accumulates per company over time",
        "Scheduled tasks & proactive pushes: pre-market briefs, close reviews, and earnings reminders on your schedule",
        "Finance calendar: macro events and your holdings' earnings on one timeline",
        "Invest page: holdings, investment mainlines, and company profiles in one research overview",
        "Official community: a read-only stream of research judgments and key materials",
        "OpenAI-compatible API: plug HONE into your own tooling with an API key",
        "Deploy anywhere: local self-hosting and cloud runtime (Postgres / OSS) are both supported",
        "Full engineering changelog lives in GitHub Releases and the commit history",
      ],
    },
    next: {
      label: "Near term",
      items: [
        "Windows / Linux desktop builds",
        "User-facing skill editor",
        "Broader data import / export (company-profile bundles shipped; portfolio and research results in progress)",
        "Continued polish of cloud deployment operations and migration tooling",
        "Public skill documentation and example pack",
        "Vector-augmented long memory",
      ],
    },
    later: {
      label: "Long horizon",
      items: [
        "Multi-user collaborative research space",
        "Visual portfolio analytics dashboard",
        "Broader developer APIs, SDKs, and examples",
        "Community skill marketplace",
        "Multi-agent orchestration",
      ],
    },

    boundary: {
      label: "Open source boundary",
      open: [
        "Rust core engine (hone-core / hone-channels / hone-llm / hone-tools)",
        "Frontend UI (SolidJS + Tailwind v4)",
        "Tauri desktop shell",
        "All 16 public skills",
        "All channel integrations (Web / iMessage / Lark / Discord / Telegram / CLI / MCP)",
      ],
      closed: [
        "Private premium skill library",
        "Paid data-source API keys",
        "VIP-only features / hosted services",
      ],
    },

    docs: [
      {
        title: "README (English)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README.md",
        desc: "Project overview, install, quick start",
      },
      {
        title: "README (中文)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
        desc: "Overview, install, quick start in Chinese",
      },
      {
        title: "Wiki",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/wiki.md",
        desc: "Install, startup, ports, configuration, verification, and troubleshooting",
      },
      {
        title: "Release Notes v0.12.4",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/releases/v0.12.4.md",
        desc: "Latest release user impact, upgrade path, and known notes",
      },
      {
        title: "HONE Blog",
        url: "https://hone-claw.com/blog",
        desc: "Public bilingual long-form posts on architecture choices, migrations, and product notes",
      },
      {
        title: "Repo Map",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
        desc: "Module boundaries, runtime data flow, and linked change areas",
      },
      {
        title: "Cases (中文)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md",
        desc: "Real-world research scenario examples",
      },
      {
        title: "Cases (English)",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_EN.md",
        desc: "Real-world case studies",
      },
      {
        title: "Skills directory",
        url: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills",
        desc: "Source and notes for every public skill",
      },
      {
        title: "CONTRIBUTING.md",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md",
        desc: "Contribution guide",
      },
      {
        title: "SECURITY.md",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/SECURITY.md",
        desc: "Vulnerability disclosure policy",
      },
    ],

    contributing: [
      {
        icon: "◈",
        title: "Open an issue",
        desc: "Report a bug, request a feature, start a design discussion",
        href: "https://github.com/B-M-Capital-Research/honeclaw/issues/new/choose",
      },
      {
        icon: "⚡",
        title: "Send a pull request",
        desc: "Fix bugs, add features, improve docs",
        href: "https://github.com/B-M-Capital-Research/honeclaw/pulls",
      },
      {
        icon: "∞",
        title: "Contribute a skill",
        desc: "Use skills/skill_manager/create_skill.sh to bootstrap a new skill",
        href: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills",
      },
    ],

    bottom_cta: {
      title: "Ready to start?",
      desc: "Open the chat, or clone the repo and run locally.",
      primary: "Enter Chat →",
    },

    faqs: [
      {
        q: "How is HONE different from a general AI chat tool?",
        a: "HONE won't flatter you. It treats research discipline as a hard constraint and actively pushes back on emotional decisions. Every conversation builds on long-term memory (company profiles), not a blank slate.",
      },
      {
        q: "Do I have to self-host?",
        a: "Three options: (1) the `curl | bash` installer for hone-cli; (2) a Homebrew tap; (3) clone the repo and start through the local CLI build path. The first two share the same GitHub release bundle — no Rust compile needed. Public SMS login requires Aliyun SMS configuration; if the behavior captcha gate is enabled, configure Aliyun Captcha environment variables too. After frontend upgrades, the public app uses asset recovery to handle load failures caused by stale cached chunks.",
      },
      {
        q: "Which LLMs are supported?",
        a: "HONE supports HONE Cloud, Gemini CLI, Codex CLI / ACP, and OpenCode ACP through the agent-engine abstraction. The default local route is Codex ACP with GPT-5.6 Sol and xhigh reasoning effort. Credentials live in `config.yaml` under `llm.providers.*.api_key/api_keys`, and generic OpenAI-compatible providers plus OpenRouter can try the next key in the pool.",
      },
      {
        q: "What license? Commercial use?",
        a: "MIT, commercial use allowed. The repo ships a fully working core engine, UI, desktop, all 16 public skills, and 7 channel integrations. Private premium skills and paid data sources live outside the repo and don't block the main flow.",
      },
      {
        q: "Where is data stored?",
        a: "Data still defaults to local storage or your self-hosted server (macOS desktop's `~/.honeclaw`). v0.12.4 adds Cloud PG / OSS runtime config; current main can place sessions, Web invites/auth sessions, conversation quota, cron jobs/runs, due-job claims, the skill registry, notification prefs, portfolio, LLM audit, and company profile files in PG, plus public uploads, generated images / files, and migrated documents in OSS. `cloud.strict_no_local_storage=true` checks the current config for remaining durable local dependencies. HONE does not host your data by default.",
      },
      {
        q: "How does HONE relate to Codex / RooCode and other coding agents?",
        a: "HONE borrows their agent-engine, skill, and session architecture but targets investment research, not coding. Codex CLI / ACP, Gemini CLI, and OpenCode ACP show up inside HONE as pluggable engines.",
      },
    ],
  },

  me: {
    logged_in_title: "Account",
    logged_in_eyebrow: "",
    logged_out_title: "Sign in first",
    logged_out_desc: "Sign in to see your history and account info.",
    logged_out_cta: "Go to chat to sign in",
    invite_note: "Your phone number must be on the invite list before you can enter chat",
    loading: "Loading…",
    account_info_title: "Account info",
    usage_today_label: "Account status",
    date_locale: "en-US",
    date_placeholder: "—",
    stats: {
      remaining_today_label: "Account status",
      remaining_today_sub_template: "",
      total_label: "History",
      total_sub: "",
      daily_limit_label: "Access",
      daily_limit_sub: "",
    },
    actions: {
      chat: "Enter chat →",
      roadmap: "View roadmap",
      community: "Community",
      logout: "Sign out",
    },
    membership: {
      title: "Membership / premium",
      desc: "Billing, VIP group, premium capabilities — coming soon. Join the community to hear first.",
    },
    fields: {
      user_id: "Account",
      created_at: "Joined",
      last_login: "Last login",
      daily_limit: "Access",
      used_today: "History",
      remaining: "Account status",
    },
  },

  chat_page: {
    header: {
      subtitle: "Investment assistant",
    },
    sidebar: {
      label: "Chat navigation",
      collapse: "Collapse sidebar",
      expand: "Expand sidebar",
      signed_in: "Signed in",
      account_center: "Account center",
      history_title: "Conversation history",
      history_empty: "Recent questions will appear here after you start chatting.",
      history_attachment: "Question with attachments",
      history_empty_item: "Empty message",
    },
    pushes: {
      nav: "Dispatches",
      open_aria: "Open push center",
      fallback_title: "Scheduled push",
      fallback_summary: "Task completed",
    },
    prefs: {
      aria_label: "Font size and theme",
      font_size: "Size",
      theme: "Theme",
      theme_auto: "Auto",
      theme_light: "Light",
      theme_dark: "Dark",
      language: "Language",
      language_zh: "中文",
      language_en: "English",
    },
    status: {
      error: "HONE hit an error",
      streaming: "HONE is responding",
      running: "HONE is working",
      thinking: "HONE is thinking",
      done: "Done",
      fallback_error: "Request failed. Please try again.",
      stop: "Stop",
      stopped: "This response was stopped",
    },
    attachments: {
      image_title: "Image",
      image_subtitle: "Photos and screenshots",
      file_title: "File",
      file_subtitle: "PDF · documents · other",
    },
    composer: {
      quota_exhausted: "You've used today's chat quota",
      placeholder: "Ask HONE…",
      send_aria: "Send",
      proactive_tip: "Portfolio analysis",
      proactive_title: "HONE can watch your holdings for you",
      proactive_intro:
        "Tell HONE what you hold or follow, and it will filter important changes by your preferences and reach out at the right time.",
      proactive_items: [
        {
          title: "Holding-aware alerts",
          body: "Earnings, calls, SEC filings, major news, rating changes, and price moves.",
        },
        {
          title: "Portfolio analysis",
          body: "Signals are framed around your positions, watch reasons, and long-term thesis.",
        },
        {
          title: "Natural-language control",
          body: "Say things like “only holdings”, “quiet tonight”, or “review every Friday” to manage alerts and schedules.",
        },
      ],
      proactive_examples_title: "Try saying",
      proactive_examples: [
        "Introduce the indium phosphide industry chain and recommend related optical module companies.",
        "I hold AAPL and NVDA. Turn on key event alerts.",
        "Only push earnings and major news for my holdings.",
        "Run a portfolio review after market close every Friday.",
      ],
      proactive_close_aria: "Close push mode tips",
      proactive_got_it: "Got it",
      finance_calendar_tip: "Finance calendar",
      finance_calendar_title: "My finance calendar",
      finance_calendar_intro:
        "Pick a month and HONE will combine macro events with earnings dates for your holdings and watchlist, then send it as an image in chat.",
      finance_calendar_months_label: "Calendar month",
      finance_calendar_current_month: "Back to this month",
      finance_calendar_previous_aria: "Previous month",
      finance_calendar_next_aria: "Next month",
      finance_calendar_preview_aria: "Finance calendar image preview",
      finance_calendar_preview_open: "Open large calendar",
      finance_calendar_preview_hint: "Tap for a large, zoomable view",
      finance_calendar_preview_close: "Close large calendar",
      finance_calendar_image_loading: "Loading the high-resolution calendar",
      finance_calendar_image_loading_hint: "The image is ready and is being retrieved securely",
      finance_calendar_image_failed: "The calendar image did not load",
      finance_calendar_image_failed_hint: "Your connection may be slow. Retry now.",
      finance_calendar_image_retry: "Reload calendar",
      finance_calendar_image_save: "Save image",
      finance_calendar_image_saving: "Saving…",
      finance_calendar_image_share: "Share",
      finance_calendar_image_zoom: "Calendar zoom",
      finance_calendar_image_save_hint: "Pinch to zoom, drag to pan, or long-press the image to save to Photos",
      finance_calendar_image_action_failed: "The browser could not finish that action. Open the image and long-press to save it.",
      finance_calendar_share_text: "My HONE finance calendar",
      finance_calendar_zoom_in: "Zoom in",
      finance_calendar_zoom_out: "Zoom out",
      finance_calendar_zoom_fit: "Fit screen",
      finance_calendar_loading: "Building this month's calendar…",
      finance_calendar_macro_label: "macro events",
      finance_calendar_earnings_label: "holding earnings",
      finance_calendar_holdings_label: "tracked symbols",
      finance_calendar_sources: "BLS · BEA · Federal Reserve · Census · ISM · FMP",
      finance_calendar_send: "Send this calendar",
      finance_calendar_sending: "Generating image…",
      finance_calendar_error: "Send failed",
      finance_calendar_render_error: "The finance calendar template is not ready yet",
      finance_calendar_upload_error: "Image upload failed",
      finance_calendar_close_aria: "Close finance calendar",
    },
    history: {
      loading_older: "Loading…",
      load_older: "Keep scrolling up for earlier messages",
    },
    restoring: {
      title: "Restoring chat",
      desc: "Checking the current session and restoring chat history",
      retrying: "The backend is taking longer than expected. Retrying automatically (attempt {attempt})…",
      failed_title: "Could not restore chat",
      failed_desc: "The current session could not be restored. You can try again now.",
      retry_button: "Retry restore",
      timeout_reason: "Request timed out",
      generic_reason: "Network or service is temporarily unavailable",
      reason_prefix: "Reason: {message}",
    },
    earnings: {
      preview_label: "Earnings preview",
      analysis_label: "Earnings analysis",
      preview_hint:
        "Enter a company and HONE verifies the entity, expectations and key variables, then renders a watermarked shareable PDF.",
      analysis_hint:
        "Enter a company and optionally upload filings, releases or call materials; HONE reads them first, then completes the analysis and the shareable PDF.",
      company_placeholder: "For example: NVIDIA / NVDA",
      company_required: "Enter a company name or ticker",
      pick_files: "Choose earnings files",
      file_hint: "PDF, Word, Excel, image or text",
      starting: "Starting…",
      close: "Close",
      start_failed: "Could not start. Please try again shortly.",
      busy: "Cannot start another analysis right now",
      loading_preview_skill: "Loading the earnings preview skill",
      loading_analysis_skill: "Loading the earnings analysis skill",
      selected_files: "{count} file(s) selected",
      start_action: "Start {label}",
    },
    workspace: {
      brand_aria: "HONE workspace",
      default_user: "HONE user",
      user_prefix: "User ",
      history_label: "Conversations",
      new_chat: "New chat",
      search_history: "Search conversations",
      recent: "Recent",
      syncing_history: "Syncing conversations…",
      no_match: "No conversation matches that. Try another keyword.",
      history_empty: "No conversations yet. Ask something and it will show up here.",
      loading: "Loading…",
      load_older: "Load earlier",
      personal_space: "Personal research space",
      logout: "Sign out",
      agent_tagline: "Your investment research agent",
      search_all: "Search companies, topics or community posts",
      open_pushes: "Open notifications",
      reconnecting: "Reconnecting to your research space",
      restoring: "Restoring your research space",
      sync_attempt: "The backend is slow to respond. Sync attempt {attempt}.",
      sync_detail: "Syncing research records, pushes and recent sessions.",
      qa_moves_title: "Explain portfolio moves",
      qa_moves_summary: "Why the portfolio rose or fell today",
      qa_moves_meta: "Portfolio · Today",
      qa_moves_prompt: "Using my holdings, explain the main drivers of today's portfolio move, ranked by impact.",
      qa_compare_title: "Compare two companies",
      qa_compare_summary: "Compare the current inference-side opportunity",
      qa_compare_meta: "Companies · Compare",
      qa_compare_prompt: "I want to compare two companies. Ask me which ones first, then cover business, competitive position, valuation and risk.",
      qa_filing_title: "Read earnings materials",
      qa_filing_summary: "Find the threads worth verifying in a filing",
      qa_filing_meta: "Materials · Deep",
      qa_filing_prompt: "I will upload an earnings document. Extract the key figures, management commentary, changes and open questions.",
      qa_track_title: "Set up tracking",
      qa_track_summary: "Keep holdings and watchlist names under continuous review",
      qa_track_meta: "Tasks · Ongoing",
      qa_track_prompt: "Build a continuous tracking plan from my holdings and watchlist.",
      seed_portfolio_eyebrow: "Portfolio research",
      seed_portfolio_title: "Review today's portfolio changes",
      seed_portfolio_summary: "Surface the variables worth watching across holdings, news and events",
      seed_event_eyebrow: "Coming up",
      seed_event_title: "Review the upcoming events",
      seed_event_summary: "Put the macro calendar and holdings earnings on one timeline",
      seed_research_eyebrow: "New research",
      seed_research_title: "Open a research thread",
      seed_research_summary: "Start from a question and keep the sources, conclusions and follow-ups",
      insight_prompt: "Continue the analysis from this lead: {title}. {summary}",
      context_prefix: "Working from:",
      context_portfolio: "My portfolio",
      context_events: "Today's events",
      insight_count: "{count} leads worth continuing today.",
      quick_start: "Quick start",
      quick_start_hint: "Sources and reasoning are shown",
      today_insights: "Today's research leads",
      browse_community: "Browse community",
      no_insight_match: "No research lead matches that. Try another keyword.",
      key_events: "Key events",
      finance_calendar: "Finance calendar",
      open_my_calendar: "Open my finance calendar",
      calendar_summary: "Macro calendar and holdings earnings",
      upcoming_events: "Upcoming events",
      open_your_calendar: "Open your finance calendar",
      recent_research: "Recent research",
      research_empty: "Research you start is saved here automatically.",
      continue_research: "Continue this research",
      open_menu: "Open menu",
      history_title: "Conversation history",
      pushes: "Notifications",
      open_account: "Open {name}'s account",
      drawer_aria: "Workspace menu and conversations",
      close_menu: "Close menu",
      main_menu: "Main menu",
      search_chats: "Search chats",
      chat_records: "Chats",
      drawer_history_empty: "Ask something and your conversations will appear here.",
      main_nav: "Main navigation",
      insights: "Insights",
      me: "Me",
    },
    me_page: {
      plan_live: "Thursday deep-dive company sessions, live with open Q&A",
      plan_group: "A 500+ member group sharing deep research and live updates",
      plan_planet: "Knowledge Planet and community: full company reports, valuation and strategy",
      plan_qa: "Ask anything and get a timely answer from the community",
      cycle_until: "Current period through",
      no_renew: "Will not auto-renew after it ends.",
      opening: "Opening…",
      manage_stripe: "Manage subscription in Stripe",
      no_access_sub: "This subscription does not currently grant HONE access.",
      intl_member: "International member · unified benefits",
      cn_invite: "Account · China invite",
      entitled: "Your HONE access is active",
      not_entitled: "Membership benefits are unavailable",
      awaiting_payment: "Waiting for the payment platform to confirm. This page refreshes automatically; a successful redirect alone does not grant access.",
      duplicate_subs: "Multiple active Stripe subscriptions detected. HONE access continues, but you may be billed twice — cancel the one you do not need in Stripe.",
      paused_note: "Your account data is kept and paid features are paused. Resume payment or resubscribe, and access returns once the server confirms.",
      any_stripe_grants: "Any active Stripe entitlement grants access:",
      cn_channel_grants: "This account is granted access through the China invite channel:",
      wecom_qr_alt: "WeCom support QR code",
      support_cta: "Questions? Add support on WeChat",
      support_hint: "Scan to add WeCom support for membership and billing questions.",
      support_title: "WeCom support",
      close: "Close",
      save_qr_hint: "Long-press or right-click to save the image, then scan to add support.",
      personal_space: "Personal research space",
      me: "Me",
      me_subtitle: "Watchlist and holdings, research style, account details and subscription are all managed here.",
      account_info: "Account details",
      account: "Account",
      verify_channel: "Verification channel",
      email_channel: "Email {value}",
      cn_phone_invite: "China mobile invite",
      registered_at: "Registered",
      last_login: "Last sign-in",
      access: "Access",
      enabled_quota: "Active · {count} per day",
      times: "",
      enabled: "Active",
      paused: "Paused",
      view_membership: "Membership and renewal",
      open_agent: "Open Agent",
      open_community: "Visit the community",
      sign_out: "Sign out",
      billing_note: "Handles international billing; HONE grants access from its own server-side entitlement record.",
      about_help: "About and help",
      home: "Home",
      pricing: "Membership and pricing",
      roadmap: "Roadmap and docs",
      tos: "Terms of service",
      privacy: "Privacy policy",
      disclaimer: "For research reference only, not investment advice. Markets carry risk; decide independently.",
      loading_title: "Loading your space",
      loading_detail: "Confirming your account and research access.",
    },
    community_page: {
      just_now: "Just now",
      official: "Official community",
      resources: "Community resources",
      close_preview: "Close preview",
      pdf_unsupported: "This host cannot display the PDF",
      pdf_fallback: "The file itself is still available — use Download below to view it.",
      pdf_preparing: "Preparing a sandboxed PDF preview…",
      file_preview: "Community file preview",
      pdf_slow: "The embedded preview is slow. If it stays blank, download the file instead.",
      pdf_unavailable: "Embedded preview is unavailable — download the file instead.",
      pdf_loaded: "Loaded. If the host does not render it, download the file instead.",
      pdf_verifying: "Verifying and loading the PDF…",
      image: "Community image",
      zoom_hint: "Pinch or scroll to zoom; drag once zoomed in",
      preview_na_hint: "Embedded preview unavailable; download for the full view",
      sandboxed: "Loaded through the security sandbox",
      image_zoom: "Image zoom",
      zoom_out: "Zoom out",
      zoom_in: "Zoom in",
      fit_screen: "Fit to screen",
      downloading: "Downloading…",
      download: "Download",
      download_failed: "Download failed. Please try again.",
      older_failed: "Could not load earlier posts",
      load_failed: "Community content is temporarily unavailable",
      resource_failed: "Resource download failed",
      login_title: "Sign in to view the HONE community",
      login_hint: "The community is read-only and visible to signed-in users.",
      search: "Search the community, companies or topics",
      eyebrow: "Community research · read-only",
      title: "Community",
      subtitle: "Research calls, market observations and key materials from the HONE community, kept in chronological order.",
      loading: "Loading community content…",
      reload: "Reload",
      feed_title: "Official community feed",
      no_match: "No community content matches that.",
      read_only: "Read-only",
      preview_label: "Preview {name}",
      image_protected: "Image protected by the source",
      community_file: "Community file",
      meta_only: "Protected by the source; metadata only",
      click_preview: "Tap for a sandboxed preview",
      click_download: "Tap to download",
      collapsed_note: "The source page shows this long post as a collapsed summary.",
      loading_short: "Loading…",
      retry_older: "Retry loading earlier posts",
      load_older: "Load earlier posts",
      disclaimer: "Community posts are shared research for reference only, not investment advice.",
    },
    recovery: {
      interrupted: "The previous request was interrupted. Please send it again.",
      reconnecting: "Connection dropped, restoring task state",
      reconnect_failed:
        "Connection dropped and the task state could not be restored. Please refresh and try again.",
      attach_aria: "Add an image or file",
    },
    community: {
      open_aria: "Open community updates",
      open_aria_unread: "Open community updates, new activity",
    },
    actions: {
      logout: "Log out",
      copy_aria: "Copy",
      copied: "Copied",
      scroll_to_bottom_aria: "Jump to latest",
      share_aria: "Share",
      dismiss_aria: "Dismiss",
    },
    share: {
      brand_name: "HONE",
      brand_tagline: "Your AI investment co-pilot",
      qr_caption: "Scan to try HONE — an AI co-pilot for investors",
      strings: {
        title: "Share conversation",
        subtitle: "Pick from the latest 4 messages",
        preview_subtitle: "Preview the image, then save, copy, or share it",
        preview_scroll_hint: "Scroll inside the preview for long images; keyboard users can focus the preview first",
        generate_image: "Generate share image",
        back_to_select: "Choose again",
        download: "Download",
        save_image: "Save image",
        copy_image: "Copy image",
        copy_text: "Copy text only",
        share: "Share…",
        share_other_app: "Share to another app",
        close_aria: "Close",
        success_download: "Image saved",
        success_copy_image: "Image copied",
        success_copy_text: "Text copied",
        success_share: "Shared",
        save_image_hint: "Use the system share sheet to save the image, or long-press it to save to Photos.",
        error_download: "Save failed. Please try again.",
        error_copy_image: "Copy failed — try Save image instead",
        error_copy_text: "Text copy failed. Select the text manually.",
        error_render: "Image rendering failed. Try fewer messages.",
        error_share: "Share canceled",
        error_system_share: "System share failed. Try Save image or Copy instead.",
        role_user: "You",
        role_assistant: "HONE",
        nothing_selected: "Select at least one message",
        rendering: "Rendering…",
      },
    },
  },

  auth: {
    login: {
      title: "Sign in to HONE",
      subtitle: "Sign in with your phone number and SMS code.",
      hint_sms:
        "HONE is currently invite-only. Contact bm@hone-claw.com to join the invite list.",
      phone_label: "Phone",
      phone_placeholder: "e.g. +1 555 0134",
      phone_aria: "Phone",
      code_label: "Code",
      code_placeholder: "SMS code",
      code_aria: "SMS code",
      send_code: "Send code",
      sending_code: "Sending",
      resend_in: "{seconds}s",
      code_sent: "Code sent. Please check your SMS.",
      remember_30d: "Keep me signed in (30 days)",
      submit_sms: "Sign in",
      loading: "Signing in…",
    },
    tos: {
      prefix: "I have read and agree to the ",
      terms: "Terms of Service",
      and: " and ",
      privacy: "Privacy Policy",
      version_template: " (v{version})",
    },
  },

  legal: {
    version_banner_template: "v{version} · effective {date}",
    terms: {
      page_title: "Terms of Service",
      intro:
        "Please read the following carefully. Continuing to use HONE means you accept these terms.",
      sections: [
        {
          title: "1. Acceptance & effective date",
          body: [
            {
              kind: "p",
              parts: [
                'Welcome to HONE ("the service"). The service is operated by ',
                { strong: "Snowdrift Capital LLC" },
                ', a limited liability company organized under the laws of the State of Wyoming, United States ("we," "us," or "our"). These Terms of Service ("Terms") form a binding agreement between you and us regarding your use of the service.',
              ],
            },
            {
              kind: "p",
              parts: [
                "By checking the agreement box or continuing to use the service, you confirm that you have read and accept these Terms in full. If you disagree with any clause, stop using the service immediately.",
              ],
            },
          ],
        },
        {
          title: "2. Service description",
          body: [
            {
              kind: "p",
              parts: [
                "HONE is a research and decision-assistant tool for individual investors, offering information retrieval, conversational research, investment notes, and scheduled reminders.",
              ],
            },
            {
              kind: "p",
              parts: [
                {
                  strong:
                    "The service does not constitute investment advice, an offer, or a recommendation of any kind.",
                },
                " All output from the service is for reference only; every investment decision is yours to make and yours to bear.",
              ],
            },
          ],
        },
        {
          title: "3. Account & verification",
          body: [
            {
              kind: "p",
              parts: [
                "Users entering through mainland China channels sign in with a registered mainland China phone number and an SMS code. Overseas Stripe users may verify with their purchase email and an email code. Every account must correspond to a valid invitation or membership entitlement.",
              ],
            },
            {
              kind: "p",
              parts: [
                "Keep your phone number or email, verification codes, and signed-in devices secure. Do not share your account with others. If you notice unauthorized access, notify us immediately.",
              ],
            },
          ],
        },
        {
          title: "4. Acceptable use",
          body: [
            {
              kind: "p",
              parts: [
                "When using the service, you agree not to (including but not limited to):",
              ],
            },
            {
              kind: "ul",
              items: [
                [
                  "violate any U.S. federal, state, or local law or regulation, including export-control, OFAC sanctions, anti-money-laundering, securities, privacy, cybersecurity, and other applicable rules;",
                ],
                [
                  "violate mainland China laws, regulatory requirements, public-order and good-morals standards, or public interests, or generate, transmit, or induce content that mainland China laws or mainstream platform governance rules expressly prohibit or discourage;",
                ],
                [
                  "infringe on others' rights, including intellectual property, privacy, publicity, reputation, trade secrets, or other proprietary or personal rights;",
                ],
                [
                  "post or transmit content that is threatening, harassing, hateful, discriminatory, fraudulent, or defamatory;",
                ],
                [
                  "produce, reproduce, distribute, or solicit pornographic content, child sexual abuse material, gambling, drug trafficking, scams, violent terrorism, extremism, or other unlawful or harmful content;",
                ],
                [
                  "post, transmit, or induce content that harms national security, incites subversion of state power, separatism, destruction of national unity, ethnic hatred, anti-China content, unlawful politically sensitive content, disruption of public order, or violations of public morals;",
                ],
                [
                  "use prompt injection, jailbreaks, role-play, forged system instructions, context pollution, or any other means to induce the service to produce, assist, conceal, or amplify content that violates the above;",
                ],
                [
                  "reverse-engineer, scrape, bulk-automate, exploit vulnerabilities, circumvent access controls, or otherwise abuse the service;",
                ],
                [
                  "upload, distribute, or deploy malware, spam, phishing links, or other harmful technologies;",
                ],
                [
                  "impersonate others, falsify account information, or engage in any form of fraud.",
                ],
              ],
            },
            {
              kind: "p",
              parts: [
                "If you violate the above, we may immediately suspend or terminate your account, revoke your eligibility to use the service, preserve relevant evidence, and cooperate with lawful requests from law-enforcement, regulatory, or judicial authorities. You bear sole legal responsibility for any consequences.",
              ],
            },
          ],
        },
        {
          title: "5. Content & intellectual property",
          body: [
            {
              kind: "p",
              parts: [
                "All intellectual property rights in the service — interface, copy, code, marks, and related materials — belong to us or our lawful rights holders, protected by copyright and related laws.",
              ],
            },
            {
              kind: "p",
              parts: [
                "Content you input (conversations, notes, attachments, etc.) remains yours. You grant us a non-exclusive license, limited to what is necessary to operate and improve the service.",
              ],
            },
          ],
        },
        {
          title: "6. Third-party services & data sources",
          body: [
            {
              kind: "p",
              parts: [
                "The service may call third-party large language models (LLMs), market data, search engines, SMS or email delivery services, and Stripe for overseas payment processing and subscription synchronization. Third-party services are operated independently; their stability, accuracy, and compliance are governed by their own official statements.",
              ],
            },
            {
              kind: "p",
              parts: [
                "You understand and agree that, when calling a third-party service, we may transmit the necessary request content. We will choose reputable and trustworthy partners in line with their terms.",
              ],
            },
          ],
        },
        {
          title: "7. Subscriptions, renewal, cancellation & refunds",
          body: [
            {
              kind: "p",
              parts: [
                "Overseas annual memberships are billed through Stripe. Checkout displays the currency, price, billing interval, and applicable taxes. Unless you cancel before the end of the current period, the subscription automatically renews on the disclosed interval and charges the original payment method.",
              ],
            },
            {
              kind: "p",
              parts: [
                "You can open the applicable provider from your account page to manage or cancel. Cancellation normally takes effect at the end of the paid period and does not automatically refund a period that has begun. Refund requests are handled under checkout disclosures, applicable law, our express written commitments, and the provider's rules; nothing here limits a refund required by law.",
              ],
            },
          ],
        },
        {
          title: "8. Service changes, suspension & termination",
          body: [
            {
              kind: "p",
              parts: [
                "We may suspend, change, or terminate part or all of the service for upgrades, maintenance, security incidents, force majeure, or business adjustments. We will give reasonable prior notice through in-service messages or other channels.",
              ],
            },
            {
              kind: "p",
              parts: [
                "If you materially breach these Terms, we may suspend or terminate your access immediately and reserve the right to pursue remedies under the law.",
              ],
            },
          ],
        },
        {
          title: "9. Disclaimers & limitation of liability",
          body: [
            {
              kind: "p",
              parts: [
                'To the maximum extent permitted by applicable law, the service is provided "as is" and "as available." We make no express or implied warranty of continuity, accuracy, completeness, or timeliness.',
              ],
            },
            {
              kind: "p",
              parts: [
                "Some capabilities require an active paid membership. To the maximum extent permitted by applicable law, we are not liable for indirect, incidental, or consequential loss from using or being unable to use the service (including investment or trading losses, data loss, or lost profits). This does not limit liability that applicable law does not permit us to exclude.",
              ],
            },
          ],
        },
        {
          title: "10. Changes to these Terms",
          body: [
            {
              kind: "p",
              parts: [
                "We may revise these Terms to reflect changes in law or our business. Updated Terms will be published in-service with a version number and effective date.",
              ],
            },
            {
              kind: "p",
              parts: [
                "Material changes will be surfaced via in-service notice for reconfirmation. Continuing to use the service after an update means you accept the revised Terms.",
              ],
            },
          ],
        },
        {
          title: "11. Governing law & dispute resolution",
          body: [
            {
              kind: "p",
              parts: [
                "The formation, validity, interpretation, performance, and dispute resolution of these Terms are governed by the ",
                { strong: "laws of the State of Wyoming, United States" },
                ", without regard to its conflict-of-laws principles. The United Nations Convention on Contracts for the International Sale of Goods (CISG) does not apply to these Terms.",
              ],
            },
            {
              kind: "p",
              parts: [
                "Any dispute arising from or related to these Terms shall first be addressed in good faith through negotiation. Failing that, either party may bring a claim in the state or federal courts located in Sheridan County, Wyoming, USA, and both parties consent to the exclusive jurisdiction of those courts and waive any objection to venue.",
              ],
            },
            {
              kind: "p",
              parts: [
                "To the maximum extent permitted by applicable law, you agree to resolve disputes with us individually, and not as part of any class or representative action.",
              ],
            },
          ],
        },
        {
          title: "12. Contact",
          body: [
            {
              kind: "p",
              parts: [
                "If you have any questions, comments, or suggestions about these Terms, please contact us:",
              ],
            },
            {
              kind: "ul",
              items: [
                [{ strong: "Email:" }, " ", { code: "bm@hone-claw.com" }],
                [
                  { strong: "GitHub Issues:" },
                  " ",
                  {
                    code: "https://github.com/B-M-Capital-Research/honeclaw/issues",
                  },
                ],
                [
                  { strong: "Mailing address:" },
                  " Snowdrift Capital LLC, 30 N Gould St, Ste N, Sheridan, WY 82801, United States",
                ],
              ],
            },
            {
              kind: "p",
              parts: ["We will respond within a reasonable period."],
            },
          ],
        },
      ] as LegalSection[],
    },
    privacy: {
      page_title: "Privacy Policy",
      intro:
        "We care about your data. This policy explains how HONE handles your personal information.",
      sections: [
        {
          title: "1. Introduction & scope",
          body: [
            {
              kind: "p",
              parts: [
                "This Privacy Policy describes how HONE (operated by ",
                { strong: "Snowdrift Capital LLC" },
                ', a Wyoming limited liability company, "we," "us," or "our") collects, uses, stores, shares, and protects your personal information while providing the service. It applies to every scenario in which you use the service through the HONE website or client.',
              ],
            },
            {
              kind: "p",
              parts: [
                "Please read this policy in full before using the service. Continuing to use it means you have understood and accepted the policy.",
              ],
            },
          ],
        },
        {
          title: "2. Information we collect",
          body: [
            {
              kind: "p",
              parts: [
                "To provide the service, we collect the following categories of information under the principle of data minimization:",
              ],
            },
            {
              kind: "ul",
              items: [
                [
                  { strong: "Account info:" },
                  " for mainland China channels, phone number, SMS verification result, and historical invite records; for overseas channels, purchase email, email verification result, and Stripe Customer, Subscription, Invoice, product, price, status, renewal-period, and webhook event identifiers;",
                ],
                [
                  { strong: "Payment info:" },
                  " overseas payments are processed by Stripe. We receive customer, subscription, and invoice identifiers needed to confirm entitlements, but we do not receive or store full card numbers, security codes, or magnetic-stripe data;",
                ],
                [
                  { strong: "Usage data:" },
                  " conversation history, prompts and responses, uploaded attachments, notes, and scheduled tasks;",
                ],
                [
                  { strong: "Device & logs:" },
                  " IP address, browser type, access timestamps, error logs, cookie identifiers;",
                ],
                [
                  { strong: "Consent events:" },
                  " the version and time at which you accepted the Terms and this policy.",
                ],
              ],
            },
          ],
        },
        {
          title: "3. How we use it",
          body: [
            {
              kind: "p",
              parts: [
                "We use the above information for the following purposes:",
              ],
            },
            {
              kind: "ul",
              items: [
                [
                  "authentication, session maintenance, account risk control, and rate limiting;",
                ],
                [
                  "confirming purchase channel, membership eligibility, renewal status, and product access;",
                ],
                [
                  "calling large language models and external data sources to fulfill your queries;",
                ],
                [
                  "recording session context to enable continuous conversation;",
                ],
                [
                  "troubleshooting, security incident response, and service optimization.",
                ],
              ],
            },
          ],
        },
        {
          title: "4. Storage, retention & security",
          body: [
            {
              kind: "p",
              parts: [
                "Your account and conversation data are stored in the service's local SQLite database by default and may be synchronized to a server database when configured for the deployment. SMS or email codes are delivered by the applicable provider; HONE stores only a code digest and expiration, not the plaintext code.",
              ],
            },
            {
              kind: "p",
              parts: [
                "We protect your information with HTTPS in transit, least-privilege access controls, server-side session cookies, and other technical and organizational measures. Within the limits of applicable law, we retain information only for as long as necessary to meet the stated purposes.",
              ],
            },
          ],
        },
        {
          title: "5. Sharing & third parties",
          body: [
            {
              kind: "p",
              parts: [
                "To authenticate you, synchronize payment entitlements, and fulfill your queries, we may transmit necessary information to the following categories of third-party service providers:",
              ],
            },
            {
              kind: "ul",
              items: [
                ["SMS and email providers (to deliver login codes);"],
                [
                  "Stripe (to process overseas payments, create subscription-management sessions, and synchronize product, invoice, and renewal status);",
                ],
                ["large language model providers (to generate responses);"],
                [
                  "market data and search data sources (to supplement queries with market or public information).",
                ],
              ],
            },
            {
              kind: "p",
              parts: [
                "Except for the necessary scenarios above or as otherwise required by law, we do not sell or lease your personal information to any third party.",
              ],
            },
          ],
        },
        {
          title: "6. Cookies & tracking",
          body: [
            {
              kind: "p",
              parts: [
                "We use an HTTP-only cookie named ",
                { code: "hone_web_session" },
                ' to maintain your sign-in state. Its lifetime is 30 days when you check "Keep me signed in," otherwise 1 day.',
              ],
            },
            {
              kind: "p",
              parts: [
                "We do not use third-party advertising tracking cookies.",
              ],
            },
          ],
        },
        {
          title: "7. Minors",
          body: [
            {
              kind: "p",
              parts: [
                "The service is intended for adults aged 18 or older with full legal capacity. If you are a minor, please use the service under a guardian's supervision. We do not actively collect personal information from minors.",
              ],
            },
          ],
        },
        {
          title: "8. Data processing location & cross-border transfers",
          body: [
            {
              kind: "p",
              parts: [
                "Our data processing infrastructure is located in the ",
                { strong: "United States" },
                " (where the operator is registered). The language models and data sources we call are primarily located in the United States and other jurisdictions. When you use the service, your personal information and query content will be transmitted to and stored in the United States.",
              ],
            },
            {
              kind: "p",
              parts: [
                "If you are located outside the United States (including the European Economic Area, the United Kingdom, mainland China, or any other jurisdiction), you understand and consent that your personal information will be transferred across borders to the United States for processing. We choose partners with appropriate compliance credentials and apply technical and organizational measures to protect the information.",
              ],
            },
          ],
        },
        {
          title: "9. Your rights",
          body: [
            {
              kind: "p",
              parts: [
                "Subject to applicable law, you have the following rights regarding your personal information:",
              ],
            },
            {
              kind: "ul",
              items: [
                ["access and correct your account details;"],
                ["manage your signed-in session;"],
                ["request deletion of your account and associated data;"],
                ["withdraw a consent you previously granted;"],
                [
                  "request a copy of the personal information you provided to us (data portability);",
                ],
                [
                  "object to or restrict certain processing of your personal information.",
                ],
              ],
            },
            {
              kind: "p",
              parts: [
                "If you are a ",
                { strong: "California resident" },
                ", under the California Consumer Privacy Act (CCPA / CPRA) you also have the right to know the categories of personal information we collect and share, the right to request deletion of collected information, and the right not to be discriminated against for exercising your rights. We do ",
                { strong: 'not "sell"' },
                " your personal information to third parties.",
              ],
            },
            {
              kind: "p",
              parts: [
                "If you are located in the ",
                { strong: "European Economic Area or the United Kingdom" },
                ", under the GDPR / UK GDPR you also have the right to lodge a complaint with your local data protection authority.",
              ],
            },
            {
              kind: "p",
              parts: [
                'You can exercise the first three rights on the "Account" page, or contact us via the channels below. Withdrawing consent may render parts of the service unavailable. We will respond to your request within a reasonable time, typically within 30 days.',
              ],
            },
          ],
        },
        {
          title: "10. Policy updates",
          body: [
            {
              kind: "p",
              parts: [
                "We may update this policy to reflect legal or business changes. Updated policies will be published in-service with a version number and effective date; material changes will be surfaced via in-service notice.",
              ],
            },
          ],
        },
        {
          title: "11. Contact",
          body: [
            {
              kind: "p",
              parts: [
                "If you have questions, comments, or complaints about this policy or how your data is handled, please contact us:",
              ],
            },
            {
              kind: "ul",
              items: [
                [{ strong: "Email:" }, " ", { code: "bm@hone-claw.com" }],
                [
                  { strong: "GitHub Issues:" },
                  " ",
                  {
                    code: "https://github.com/B-M-Capital-Research/honeclaw/issues",
                  },
                ],
                [
                  { strong: "Mailing address:" },
                  " Snowdrift Capital LLC, Attn: Privacy, 30 N Gould St, Ste N, Sheridan, WY 82801, United States",
                ],
              ],
            },
            {
              kind: "p",
              parts: [
                "We will respond and address them within a reasonable period.",
              ],
            },
          ],
        },
      ] as LegalSection[],
    },
  },

  footer: {
    tagline: "Sharpen cognition, strip the noise.",
    mantra: "HONE · SHARPEN COGNITION · STRIP THE NOISE",
    copyright:
      "© 2026 Snowdrift Capital LLC · Sheridan, WY, USA · Open source under MIT License.",
    columns: {
      product: {
        title: "Product",
        items: [
          { label: "Home", href: "/" },
          { label: "Roadmap", href: "/roadmap" },
          { label: "Blog", href: "/blog" },
          { label: "Chat", href: "/chat" },
          { label: "Account", href: "/me" },
        ],
      },
      resources: {
        title: "Resources",
        items: [
          {
            label: "GitHub",
            href: "https://github.com/B-M-Capital-Research/honeclaw",
          },
          {
            label: "Chinese docs",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md",
          },
          {
            label: "Install",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动",
          },
          {
            label: "Repository map",
            href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md",
          },
        ],
      },
      community: {
        title: "Community",
        items: [
          { label: "Discord", href: "#" },
          { label: "Zhishixingqiu", href: "#" },
          { label: "WeChat group", href: "#" },
          { label: "Content channel", href: "#" },
        ],
      },
      legal: {
        title: "Legal",
        items: [
          { label: "Terms of Service", href: "/terms" },
          { label: "Privacy Policy", href: "/privacy" },
        ],
      },
    },
  },
};

export const CONTENT = makeContentProxy(
  CONTENT_ZH,
  CONTENT_EN as typeof CONTENT_ZH,
);
