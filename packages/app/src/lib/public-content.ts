// public-content.ts — Hone Public Site Content (bilingual)
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
    logo_tagline: "OPEN FINANCIAL CONSOLE",
    home: "首页",
    roadmap: "路线图与文档",
    blog: "Blog",
    me: "个人",
    chat: "对话",
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
    contact_email: "bm@hone-claw.com",
    github_url: "https://github.com/B-M-Capital-Research/honeclaw",
  },

  hero: {
    eyebrow: "OPEN FINANCIAL CONSOLE · B&M CAPITAL RESEARCH",
    headline_1: "不是迎合你的聊天玩具",
    headline_2: "是你的投研纪律守卫者",
    description:
      "冷静、克制、长期记忆、研究导向。Hone 是专为严肃投资者打造的开源 AI Agent，帮你建立并坚守投研纪律，而不是告诉你想听的答案。",
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
    blog_title: "为什么 Hone 选择 Rust",
    blog_desc:
      "从 Python + Node.js 到 Rust 的重构复盘：AI Coding 时代的上下文、稳定性和多端工程选择。",
    blog_cta: "阅读文章",
  },

  trust: {
    section_label: "为什么是 HONE",
    items: [
      {
        symbol: "◈",
        title: "纪律先于观点",
        body: "Hone 不会迎合你的仓位偏见。每一次对话都以研究纪律为约束，主动识别并克制情绪驱动的决策冲动。",
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
    section_sub: "Hone 如何融入你的投研日常",
    placeholder_suffix: "场景演示截图",
    items: [
      {
        tag: "个股分析",
        title: "系统性深度研究一家公司",
        body: "从财务数据到行业竞争格局，Hone 帮你构建完整研究框架，记录每一个关键假设和风险因子。",
        image: "/hone_introduction_zh.jpg" as string | null,
      },
      {
        tag: "持仓追踪",
        title: "追踪持仓，主动提醒关键节点",
        body: "设置止盈止损逻辑，Hone 定时检查持仓状态，在你设定的条件触发时主动推送提醒。",
        image: "/hone_work_zh.jpg" as string | null,
      },
      {
        tag: "定时任务",
        title: "每周五自动触发投资复盘",
        body: "把固定工作流交给 Hone：每周复盘、月度总结、关键节点检查——按你设定的时间自动跑，不用手动催。",
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
        title: "在 iMessage / Lark 收到 Hone",
        body: "不只是网页。Hone 通过 iMessage、Lark、Discord 等渠道主动联系你，在你最顺手的地方工作。",
        image: "/hone_channels_zh.jpg" as string | null,
      },
    ],
  },

  video: {
    section_label: "看 HONE 如何工作",
    title: "老王讲 Hone：投研 AI Agent 的实际用法",
    description:
      "从开户到深度研究，10 分钟了解 Hone 如何改变你的投研工作流。完整演示个股分析、持仓追踪、定时任务等核心场景。",
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
        body: "Web、iMessage、Lark / Feishu、Discord、Telegram、CLI——在你最顺手的地方使用 Hone。",
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
      "透明、务实、长期主义。下面是 Hone 目前能做什么、接下来做什么、以及如何接入你的投研工作流。",
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
          "三种方式接入 Hone：一键安装脚本、Homebrew、或源码开发。安装后可用 `hone-cli start` 跑完整运行时，也可用 `hone-cli web admin-ui` / `hone-cli web user-ui` 单独打开管理端或公开用户端界面。",
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
          "Hone 是多端接入的投研助手。每个渠道都是独立进程，可独立启停、独立配置。",
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
        intro_prefix: "Hone 的 Skill 由模型根据上下文自动调用。下面是仓库",
        intro_suffix: "目录下的 16 个公开 Skill。",
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
        intro: "Hone 是开源项目，欢迎所有形式的参与——不只是代码。",
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
        desc: "公开用户端路由包含 `/`、`/roadmap`、`/blog`、`/blog/:slug`、`/chat`、`/me`、`/portfolio`、`/terms`、`/privacy`，并保留开发用 `/__share-preview` 分享卡预览页；`/blog` 是双语静态长文内容面，Cloudflare Worker 为文章分享卡注入 crawler 友好的 metadata；`/chat` 使用阿里云行为验证 + 手机短信验证码登录，管理端邀请名单是准入来源，桌面端为可收起左侧栏 + 右侧对话工作台，侧栏聚合导航、账号、最近对话历史、联系入口和 GitHub stars，支持助手回答复制、图片分享、非图片生成物附件下载与历史回看；`/portfolio` 只读展示推送上下文与公司画像入口，后端公开面收敛在 `/api/public/*`，其中 `/api/public/digest-context` 与 `/api/public/company-profile` 暴露当前用户的投资主线和单票画像，`/api/public/file` 代理可下载生成物，`/api/public/v1/chat/completions` 提供 API key 鉴权的 OpenAI-compatible 对话接口。",
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
        desc: "推荐 Agent 引擎是 Hone Cloud、Codex ACP 和 OpenCode ACP；同时保留 OpenAI 兼容函数调用、Gemini CLI、Codex CLI 与 multi-agent。LLM 凭证以 `config.yaml` 为唯一真相源，OpenRouter 与通用 OpenAI-compatible provider 都支持 `llm.providers.*.api_key/api_keys` key pool，遇到上游 429 / 配额错误时可尝试下一个 key；`gemini_acp` 仅保留为迁移配置，不作为运行时入口。",
      },
      {
        title: "事件与任务",
        desc: "Cron 任务、事件引擎摘要、`/missed` 回查、通知偏好与渠道投递共享 Rust 后端、SQLite/JSON 或 PG 执行历史和用户归属模型；Web scheduler 结果先落入会话历史，SSE 只负责在线实时提示，因此浏览器离线不会把已落库结果误记为送达失败，执行错误会写入产品化失败提示并通过同一 `scheduled_message` 事件推送给在线控制台；MCP/ACP runner 现在使用绝对 `HONE_CONFIG_PATH`，并会绝对化父进程传入的 `HONE_DATA_DIR`、忽略空串后从 runtime dir 反推数据根，目标是让 Feishu / Web / scheduler 工具读取同一份 Cron 与持仓数据；Feishu direct Cron 与 portfolio 作用域读空当前是代码级 Fixed 但尚未 Closed，仍需 live 重启 / 复核后才能表述为完全闭环；Feishu 等渠道的 scheduler heartbeat 已补齐 revision-aware 重复抑制、stale running 行终结、cloud cron 操作超时保护与 listener 内 scheduler loop 监督重启；Discord scheduler 发送失败台账现在会优先保留 runner 错误或 Discord 发送错误，没有上游文案时也会写入脱敏的通用失败原因；event-engine 默认 LLM 配置已切到当前可用的 `x-ai/grok-4.3`，避免继续依赖已下线的 Grok 4.1 Fast。",
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
            note: "portfolio_management + cron；Feishu direct 作用域读空已代码级 Fixed，待 live 复核关闭",
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
            note: "OpenAI-compatible · Gemini CLI · Codex CLI/ACP · OpenCode ACP · multi-agent",
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
            note: "scheduled_task skill + /api/cron-jobs + 执行历史 / heartbeat / quiet_hours / guard / Web SSE 与 Discord 发送失败诊断回归",
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
        desc: "解析可读图片输入；Web direct 图片附件进入 OCR / 可读链路仍按 active P2 bug ledger 复核",
      },
      {
        name: "pdf_understanding",
        desc: "解析 PDF（财报、研报）输出要点与风险",
      },
      { name: "skill_manager", desc: "查看 / 新建 / 修改 Hone Skill" },
      {
        name: "notification_preferences",
        desc: "用自然语言调整自己的推送偏好（严重度、持仓过滤、事件类型允许/屏蔽范围）",
      },
      { name: "hone_admin", desc: "查看修改 Hone 源码与配置（管理员）" },
    ],

    now: {
      label: "当前已有",
      items: [
        "Web 聊天界面（阿里云行为验证 + 手机短信验证码，管理端邀请名单准入）+ 公开门面站",
        "公开 `/chat` 桌面工作台布局：可收起左侧栏、账号入口、最近对话历史、联系入口、GitHub stars 与右侧完整高度对话区",
        "公开 `/chat` 助手回答复制与分享：可选择消息，导出品牌长图、复制图片/文字或调用系统分享；分享卡支持 CJK 代码块字体并有开发预览页",
        "公开 `/chat` 非图片生成物附件卡片：runner 新生成且正文提到的 CSV / XLSX / PDF 等文件会追加为可下载附件，经 `/api/public/file` 打开",
        "公开 `/chat` markdown 渲染、移动输入框、键盘聚焦、滚动锚定与回到底部按钮已完成稳定性打磨",
        "公开 `/blog` 与 `/blog/:slug` 双语长文页面，首篇 Rust 迁移复盘已随仓库发布，并由 Cloudflare Worker 为分享卡补齐 metadata",
        "Tauri macOS 桌面端 + 内置后端",
        "7 个渠道：Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "16 个公开 Skill（个股、持仓、估值/筛选入口、图表、PDF、Cron、漏推回查、推送偏好…）",
        "投研纪律约束 & 零幻觉协议",
        "公司画像与跨会话长期记忆",
        "管理端用户视图聚合持仓、画像、会话与研究任务；公司画像可按 actor 空间查看详情、删除、导出 zip、导入预览并处理冲突",
        "Cron 定时任务系统",
        "定时任务投递安全 guard：原油 / 大宗商品归因防护仍覆盖商品播报，但不会因市场复盘中的局部油价从句整篇替换 A/H 或美股大盘复盘",
        "Web 定时任务可靠性收口：结果先写入会话历史，在线浏览器通过 `scheduled_message` SSE 看到成功或失败提示，离线浏览器不会把已落库结果误标为 send_failed，handler 超时会记录为可排查的失败 trace",
        "事件引擎与 scheduler 质量收口：digest 去重 / min-gap / topic memory / 分类预算 / 方向性价格阈值 / Feishu heartbeat revision 去重 / stale running 记录恢复 / scheduler loop 监督重启",
        "Event-engine 默认模型与示例配置已替换为 `x-ai/grok-4.3`，避免 Grok 4.1 Fast 下线导致新闻分类、global digest、mainline distill 等 LLM 增强链路失效",
        "Event-engine FMP 行情 / 新闻 poller 持续请求失败仍在 active P2 跟踪；这会影响实时行情、新闻增量和 digest 候选新鲜度，公开页不把这条摄取链路表述为完全健康",
        "截至 2026-06-08 07:03 CST，活跃 bug ledger 没有 P0 / P1；仍有 Web 图片附件 OCR / 内部排障口径、FMP price/news poller、Codex ACP transport 断连三条 P2，以及公司画像路径、行情未校验、daily-limit 重复落库、all-in 高风险仓位建议等 P3，公开文案按这些边界保守描述能力",
        "LLM provider 配置收口到 `config.yaml`，OpenRouter 与通用 OpenAI-compatible provider 支持 `api_key/api_keys` 轮换，并保留上游错误详情便于诊断",
        "Cloud PG / OSS 运行时：`cloud.postgres` / `cloud.oss` 可通过 env 配置，公开上传、生成图片 / 文件与迁移文档可写入 OSS，公开图片 / 文件代理可读取 `oss://bucket/key` 托管对象，`/api/meta` 会暴露云能力状态和本地 durable dependency 计数",
        "云迁移边界清晰：sessions、Web invite/auth sessions、conversation quota、cron jobs/runs、due-job claims、skill registry、notification prefs、portfolio、LLM audit 与 company profile files 已有 PG 热路径；`cloud.strict_no_local_storage=true` 会在当前配置仍有 durable 本地依赖时阻止启动",
        "`hone-cli cloud doctor / migrate / object-bench` 已可做云端体检、本地 data dry-run / 幂等导入、以及 OSS/R2 小对象延迟对比；迁移器支持 session、Web auth、quota、cron、skill registry、notification prefs、portfolio、LLM audit 与 company profiles 的单项导入开关",
        "渠道回复收口层可在 runner 只产出过渡性规划句时，从成功的定时任务或持仓工具结果恢复用户可见确认；共享输出净化层会隐藏内部绝对路径、hone-mcp 依赖启动错误和 skill/tool 降级前言。公司画像相对路径措辞仍有活跃 P3 质量缺陷在 bug ledger 跟踪，公开页不把它表述为完全解决",
        "MCP / ACP 子进程运行时边界已收口：`hone-cli start` 会显式传递 `HONE_MCP_BIN`，runner 请求使用绝对配置路径，MCP server 会继承、绝对化或反推出同一份 `HONE_DATA_DIR`。该改动用于收敛 sandbox cwd 下误读空数据树的问题；Feishu direct Cron 与 portfolio 作用域读空目前是代码级 Fixed 但未 Closed，继续以 live 复核和 bug ledger 为准",
        "前端部署资产恢复：service worker 与全局错误处理可识别 stale chunk，并在安全间隔内自动刷新到新版本",
        "公开 API key 对话入口：管理端可为 Web 用户生成 API key，客户端可按 OpenAI-compatible `/api/public/v1/chat/completions` 形状调用 Hone",
        "ACP 自管上下文与 compact 防泄漏，支持 codex_acp / opencode_acp 长会话恢复",
        "多 Agent 引擎：OpenAI-compatible / Gemini CLI / Codex CLI/ACP / OpenCode ACP / multi-agent",
        "`scripts/diagnose_llm.sh` 已按当前 LLM provider 配置路径读取 OpenRouter key，保留 legacy 路径兼容",
      ],
    },
    next: {
      label: "近期计划",
      items: [
        "Windows / Linux 桌面端打包",
        "用户自定义 Skill 编辑器（前端化的 skill_manager）",
        "更广泛的数据导入 / 导出工具（公司画像包转移已上线，继续补持仓、研究结果等用户可见迁移面）",
        "继续加固 cloud migration 的观测、回滚和后台运维入口，确保 PG / OSS 模式的严格无本地 durable 依赖检查可被部署者稳定验证",
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
        "全部 16 个公开 Skill",
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
        title: "Hone Blog",
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
        q: "Hone 和普通 AI 聊天工具有什么区别？",
        a: "Hone 不会迎合你的观点。它以投研纪律为约束，主动识别并反驳情绪化决策。每次对话都以长期研究记忆（公司画像）为基础，而不是每次重新开始。",
      },
      {
        q: "需要自己部署吗？",
        a: "三种方式任选：①「curl | bash」一键装 hone-cli；② Homebrew tap；③ clone 仓库后用本地 CLI 构建启动。前两种共享同一份 GitHub release bundle，不需要自己编译 Rust。公开 SMS 登录需要配置阿里云短信；如启用行为验证，还需要配置阿里云验证码环境变量。升级前端后，公开页面会通过资产恢复逻辑处理旧 chunk 缓存导致的加载失败。",
      },
      {
        q: "支持哪些 LLM？",
        a: "通过 Agent 引擎抽象层支持：Hone Cloud、OpenAI 兼容协议（含 OpenRouter）、Gemini CLI、Codex CLI / ACP、OpenCode ACP，以及 multi-agent 搜索+回答链路。凭证统一写入 `config.yaml` 的 `llm.providers.*.api_key/api_keys`，通用 OpenAI-compatible provider 与 OpenRouter 都能在 key pool 内尝试下一个可用 key。",
      },
      {
        q: "开源协议？能商用吗？",
        a: "MIT 协议，可商用。开源仓库包含完整可运行的核心引擎、UI、桌面端、全部 16 个公开 Skill 和 7 个渠道集成。私域高级 Skill 与付费数据源接入不在仓库中，不影响主流程。",
      },
      {
        q: "数据存在哪里？",
        a: "默认仍在本地或自部署服务器存储（macOS 桌面端用户目录 ~/.honeclaw）。v0.12.4 已加入 Cloud PG / OSS 运行时配置；当前 main 的 cloud 模式可把 sessions、Web invite/auth sessions、conversation quota、cron jobs/runs、due-job claims、skill registry、notification prefs、portfolio、LLM audit 与 company profile files 放到 PG，把公开上传、生成图片 / 文件与迁移文档放到 OSS。`cloud.strict_no_local_storage=true` 会按配置检查是否仍有 durable 本地依赖；Hone 官方不默认托管你的数据。",
      },
      {
        q: "和 Codex / RooCode 等 coding agent 的关系？",
        a: "Hone 借鉴了这些产品的 Agent 引擎、Skill 与会话架构，但专注投研而非写代码。Codex CLI / ACP、Gemini CLI、OpenCode ACP 和 multi-agent 在 Hone 中作为可插拔引擎存在。",
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
      community: "加入社群",
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
    prefs: {
      aria_label: "字号与主题",
      font_size: "字号",
      theme: "主题",
      theme_auto: "自动",
      theme_light: "浅",
      theme_dark: "深",
    },
    status: {
      error: "Hone 出错了",
      streaming: "Hone 输出中",
      running: "Hone 执行中",
      thinking: "Hone 思考中",
      done: "本轮已完成",
      fallback_error: "请求出错，请重试。",
      stop: "停止",
    },
    attachments: {
      image_title: "图片",
      image_subtitle: "照片与截图",
      file_title: "文件",
      file_subtitle: "PDF · 文档 · 其他",
    },
    composer: {
      quota_exhausted: "今日对话次数已用完",
      placeholder: "向 Hone 提问…",
      send_aria: "发送",
      proactive_tip: "录入持仓，开启推送模式",
      proactive_title: "Hone 可以主动盯住你的持仓",
      proactive_intro:
        "把持仓或关注标的告诉 Hone 后，它会按你的偏好筛选重要变化，并在合适的时候提醒你。",
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
    actions: {
      logout: "退出",
      copy_aria: "复制",
      copied: "已复制",
      scroll_to_bottom_aria: "回到最新消息",
      share_aria: "分享",
    },
    share: {
      brand_name: "Hone",
      brand_tagline: "你的 AI 投资助手",
      qr_caption: "扫码体验 Hone — 给投资人的 AI 助手",
      strings: {
        title: "分享对话",
        subtitle: "从最近 4 条消息里选择要分享的内容",
        preview_subtitle: "预览图片后保存、复制或分享到其他应用",
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
        role_assistant: "Hone",
        nothing_selected: "请选择至少一条消息",
        rendering: "生成中…",
      },
    },
  },

  auth: {
    login: {
      title: "登录 Hone",
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
      intro: "请仔细阅读以下条款。继续使用 Hone 即表示您接受本协议。",
      sections: [
        {
          title: "1. 协议接受与生效",
          body: [
            {
              kind: "p",
              parts: [
                "欢迎使用 Hone（以下简称“本服务”）。本服务由 ",
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
                "Hone 是一款面向个人投资者的研究与决策辅助工具，提供资料检索、对话式研究、投资笔记、定时提醒等能力。",
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
                "您需要使用经我们登记的手机号作为账号，并通过短信验证码完成身份验证。本服务目前为邀请制，未进入邀请名单的手机号无法登录。",
              ],
            },
            {
              kind: "p",
              parts: [
                "您应妥善保管手机号码、短信验证码与登录设备，不得将账号借予他人使用。若发现账号被未经授权使用，您应立即通知我们。",
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
                "本服务可能调用第三方大型语言模型（LLM）、行情数据、搜索引擎等第三方服务以完成功能交付。第三方服务由其运营方独立提供，其稳定性、准确性及合规性以其官方声明为准。",
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
          title: "7. 服务变更、中断与终止",
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
          title: "8. 免责与责任限制",
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
                "本服务目前以免费形式提供。在适用法律允许的最大范围内，我们不对您因使用或无法使用本服务而遭受的任何直接或间接损失（包括但不限于投资或交易损失、数据丢失、利润损失等）承担金钱赔偿责任。",
              ],
            },
          ],
        },
        {
          title: "9. 协议变更与通知",
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
          title: "10. 适用法律与争议解决",
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
          title: "11. 联系方式",
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
      intro: "我们在乎您的数据。本政策说明 Hone 如何处理您的个人信息。",
      sections: [
        {
          title: "1. 引言与适用范围",
          body: [
            {
              kind: "p",
              parts: [
                "本《隐私政策》说明 Hone（运营方为 ",
                { strong: "Snowdrift Capital LLC" },
                "，一家依据美国怀俄明州法律设立的有限责任公司，以下简称“我们”）在提供服务过程中如何收集、使用、存储、共享和保护您的个人信息。本政策适用于您通过 Hone 网站及客户端使用本服务的全部场景。",
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
                  "手机号（作为账号识别与邀请资格判断）、短信验证码核验结果、历史邀请记录（作为邀请名单来源）；",
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
                "您的账号与对话数据默认存储于本服务的本地 SQLite 数据库中。短信验证码由第三方短信认证服务发送与核验，我们不会存储验证码明文。",
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
                "为完成您发起的查询，我们可能将您输入的相关内容传递给以下类别的第三方服务方：",
              ],
            },
            {
              kind: "ul",
              items: [
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
    mantra: "磨砺认知 · 剔除噪音 · OPEN FINANCIAL CONSOLE",
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
    logo_tagline: "OPEN FINANCIAL CONSOLE",
    home: "Home",
    roadmap: "Roadmap & Docs",
    blog: "Blog",
    me: "Account",
    chat: "Chat",
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
    contact_email: "bm@hone-claw.com",
    github_url: "https://github.com/B-M-Capital-Research/honeclaw",
  },

  hero: {
    eyebrow: "OPEN FINANCIAL CONSOLE · B&M CAPITAL RESEARCH",
    headline_1: "Not a chatbot that flatters you.",
    headline_2: "A research-discipline guardian.",
    description:
      "Calm, restrained, long-memory, research-first. Hone is an open-source AI agent built for serious investors — it helps you set and keep your research discipline, not tell you what you want to hear.",
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
    blog_title: "Why Hone chose Rust",
    blog_desc:
      "A field report on moving from Python + Node.js to Rust, and what it means for context, stability, and multi-endpoint engineering in the AI Coding era.",
    blog_cta: "Read article",
  },

  trust: {
    section_label: "WHY HONE",
    items: [
      {
        symbol: "◈",
        title: "Discipline over opinion",
        body: "Hone will not flatter your position. Every conversation is constrained by research discipline — it actively surfaces and pushes back on emotion-driven decisions.",
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
    section_sub: "How Hone fits into your research routine",
    placeholder_suffix: "scenario screenshot (placeholder)",
    items: [
      {
        tag: "Stock analysis",
        title: "Systematically research a company in depth",
        body: "From financials to competitive landscape, Hone helps you assemble a complete research framework, logging every key assumption and risk factor.",
        image: "/company_profile.png",
      },
      {
        tag: "Portfolio tracking",
        title: "Track holdings, nudge on key moments",
        body: "Set stop-loss / take-profit logic; Hone checks your portfolio on a schedule and pushes an alert the moment your conditions trigger.",
        image: null as string | null,
      },
      {
        tag: "Scheduled tasks",
        title: "Trigger a weekly review every Friday",
        body: "Hand fixed workflows to Hone. Weekly reviews, monthly summaries, key-moment checks — all run themselves at the time you set.",
        image: null as string | null,
      },
      {
        tag: "Long-term profile",
        title: "Build a company's personal dossier",
        body: "Each research result is archived into the company profile. Next time you ask, Hone calls back the full history — smarter with every use.",
        image: "/hone_solution.jpg",
      },
      {
        tag: "Cross-platform notifications",
        title: "Get Hone in iMessage / Lark",
        body: "Not just the web. Hone reaches you through iMessage, Lark, Discord and more — in whatever channel you're already using.",
        image: "/hone_channels.jpg",
      },
    ],
  },

  video: {
    section_label: "SEE HONE IN ACTION",
    title: "Lao Wang on Hone: the research AI agent in practice",
    description:
      "From onboarding to deep research, learn in ten minutes how Hone changes the way you work. Full walkthrough of stock analysis, portfolio tracking, scheduled tasks, and more.",
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
        body: "Web, iMessage, Lark / Feishu, Discord, Telegram, CLI — Hone on whichever channel you already live in.",
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
      "Transparent, pragmatic, long-term. Here's what Hone does today, what's next, and how to bring it into your research workflow.",
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
          "Three paths to run Hone: the one-line installer, Homebrew, or source. After install, use `hone-cli start` for the full runtime or `hone-cli web admin-ui` / `hone-cli web user-ui` to open the admin console or public user app.",
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
          "Hone is a multi-channel research agent. Each channel is an independent process — start, stop, and configure them on their own.",
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
          "Hone's skills are invoked by the model from context. Below are the 16 public skills in the",
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
          "Hone is open source. Every kind of contribution is welcome — not just code.",
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
        desc: "The public user app routes `/`, `/roadmap`, `/blog`, `/blog/:slug`, `/chat`, `/me`, `/portfolio`, `/terms`, and `/privacy`, with a dev-only `/__share-preview` page for share-card QA; `/blog` is a bilingual static long-form content surface, with Cloudflare Worker metadata for crawler-friendly article cards; `/chat` signs users in with Aliyun behavior captcha plus phone/SMS verification from the admin invite list, uses a collapsible desktop left rail plus full-height conversation workspace, and gathers navigation, account access, recent conversation history, contact links, and GitHub stars in that rail while supporting assistant-reply copy, image sharing, non-image generated-file downloads, and history review; `/portfolio` is a read-only investment context surface for push context and company-profile entry points, and the public backend is scoped to `/api/public/*`, including `/api/public/digest-context` and `/api/public/company-profile` for the signed-in user's investment mainline and single-name profiles, `/api/public/file` for downloadable generated artifacts, and `/api/public/v1/chat/completions` for API-key-authenticated OpenAI-compatible chat.",
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
        desc: "Recommended agent engines are Hone Cloud, Codex ACP, and OpenCode ACP; OpenAI-compatible function calling, Gemini CLI, Codex CLI, and multi-agent remain supported. LLM credentials use `config.yaml` as the only source of truth, and both OpenRouter and generic OpenAI-compatible providers support `llm.providers.*.api_key/api_keys` key pools so the runtime can try the next key after upstream 429 / quota failures; `gemini_acp` is kept only as migration config, not a runtime entry point.",
      },
      {
        title: "Events and tasks",
        desc: "Cron jobs, event-engine digests, `/missed` recovery, notification preferences, and channel delivery share the Rust backend, SQLite/JSON or PG execution history, and user ownership model; Web scheduler results are persisted to conversation history first and use SSE only for live hints, so an offline browser no longer turns persisted results into false delivery failures, while execution errors are stored as productized failure messages and broadcast through the same `scheduled_message` event for online consoles; MCP/ACP runners now receive an absolute `HONE_CONFIG_PATH`, and the MCP bridge absolutizes inherited `HONE_DATA_DIR` values or ignores empty ones before deriving the data root from the runtime dir, so Feishu / Web / scheduler tools are intended to read the same Cron and portfolio stores; the Feishu direct empty Cron / portfolio-scope bug is code-level Fixed but not yet Closed, so the public page still waits for live restart / verification before describing that path as fully closed; Feishu and other channel scheduler heartbeats now include revision-aware duplicate suppression, stale running-row finalization, cloud cron operation timeouts, and supervised scheduler-loop restarts inside the listener; Discord scheduler failure records now preserve the runner error or Discord send error first, and fall back to a redacted generic send-failure reason when the upstream API gives no detail; event-engine default LLM config now uses the currently available `x-ai/grok-4.3` instead of the retired Grok 4.1 Fast.",
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
            note: "portfolio_management + cron; Feishu direct empty-scope bug is code-level Fixed pending live verification",
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
            note: "OpenAI-compatible · Gemini CLI · Codex CLI/ACP · OpenCode ACP · multi-agent",
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
            note: "scheduled_task skill + /api/cron-jobs + execution history / heartbeat / quiet_hours / guard / Web SSE and Discord send-failure diagnostics regressions",
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
        desc: "Parse readable image inputs; Web direct image attachments entering OCR / readable context remain under active P2 bug-ledger verification",
      },
      {
        name: "pdf_understanding",
        desc: "Parse PDFs (filings, reports) into key points and risks",
      },
      { name: "skill_manager", desc: "View / create / edit Hone skills" },
      {
        name: "notification_preferences",
        desc: "Tune your own push prefs in natural language (severity, portfolio-only, kind allow/block)",
      },
      {
        name: "hone_admin",
        desc: "Inspect and modify Hone source & config (admin)",
      },
    ],

    now: {
      label: "Shipping today",
      items: [
        "Web chat (Aliyun behavior captcha + phone/SMS verification, admitted by the admin invite list) + public landing site",
        "Public `/chat` desktop workbench layout: collapsible left rail, account entry, recent conversation history, contact links, GitHub stars, and a full-height conversation area",
        "Public `/chat` assistant-reply copy and sharing: select messages, export a branded long image, copy image/text, or invoke native share; share cards support CJK code-block fonts and have a dev preview route",
        "Public `/chat` non-image generated-file cards: new runner-created CSV / XLSX / PDF-style files mentioned in the final answer are attached as downloads through `/api/public/file`",
        "Public `/chat` markdown rendering, mobile composer, keyboard focus, scroll anchoring, and jump-to-latest behavior have been stabilized",
        "Public `/blog` and `/blog/:slug` bilingual long-form pages, with the first Rust migration retrospective checked into the repo and Cloudflare Worker metadata for share cards",
        "Tauri macOS desktop with bundled backend",
        "7 channels: Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "16 public skills (stocks, portfolio, valuation/screening entry points, charts, PDF, cron, missed-event recovery, notification prefs…)",
        "Research discipline & zero-hallucination protocol",
        "Company profiles + cross-session long memory",
        "Admin user views group holdings, profiles, sessions, and research tasks; company profiles can be inspected by actor space, deleted, exported as zip bundles, preview-imported, and imported with conflict decisions",
        "Cron-driven scheduled tasks",
        "Scheduled-delivery safety guard: crude oil / commodity causality protection still covers commodity briefings, while broad A/H or U.S. market reviews are not fully replaced just because they contain a secondary oil-price clause",
        "Web scheduled-task reliability pass: results are written to conversation history first, online browsers receive success or failure hints through `scheduled_message` SSE, offline browsers no longer mark persisted results as send_failed, and handler timeouts become diagnosable failure traces",
        "Event-engine and scheduler quality pass: digest dedupe / min-gap / topic memory / category budgets / directional price thresholds / Feishu heartbeat revision dedupe / stale running-row recovery / scheduler-loop supervision",
        "Event-engine default models and sample config now use `x-ai/grok-4.3`, avoiding failures from the retired Grok 4.1 Fast in news classification, global digest, and mainline distillation paths",
        "Event-engine FMP price/news poller persistent request failures remain tracked as an active P2; they can degrade real-time quote, news-ingest, and digest-candidate freshness, so the public page does not describe that ingest path as fully healthy",
        "As of 2026-06-08 07:03 CST, the active bug ledger has no P0 / P1 items; Web image-attachment OCR / internal-debug wording, the FMP price/news poller, and Codex ACP transport disconnects remain P2, while company-profile path wording, unverified quote numbers, daily-limit duplicate transcript rows, and all-in high-risk sizing advice remain P3, so public copy keeps those capability boundaries conservative",
        "LLM provider config is consolidated into `config.yaml`; OpenRouter and generic OpenAI-compatible providers support `api_key/api_keys` rotation and preserve upstream error details for diagnosis",
        "Cloud PG / OSS runtime: `cloud.postgres` / `cloud.oss` can be configured through env references; public uploads, generated images / files, and migrated documents can write to OSS; public image / file proxies can read `oss://bucket/key` managed objects; `/api/meta` exposes cloud capability state and the local durable dependency count",
        "Cloud migration boundaries are explicit: sessions, Web invites/auth sessions, conversation quota, cron jobs/runs, due-job claims, the skill registry, notification prefs, portfolio, LLM audit, and company profile files have PG hot paths; `cloud.strict_no_local_storage=true` blocks startup while the current config still has durable local dependencies",
        "`hone-cli cloud doctor / migrate / object-bench` now covers cloud health checks, local data dry-runs / idempotent imports, and OSS/R2 small-object latency checks; the migrator has per-store import switches for sessions, Web auth, quota, cron, skill registry, notification prefs, portfolio, LLM audit, and company profiles",
        "The channel response finalizer can recover user-visible confirmations from successful scheduled-task or portfolio tool results when a runner only emits a transitional planning sentence; the shared output sanitizer hides internal absolute paths, hone-mcp dependency startup errors, and skill/tool degradation preludes. Relative company-profile path wording is still tracked as an active P3 quality bug in the bug ledger, so the public page no longer describes it as fully solved",
        "MCP / ACP child-process runtime boundaries are now explicit: `hone-cli start` passes `HONE_MCP_BIN`, runner requests use an absolute config path, and the MCP server inherits, absolutizes, or derives the same `HONE_DATA_DIR`. This is the code-side fix for empty data trees under sandbox cwd; Feishu direct Cron and portfolio scope reads are currently code-level Fixed but not Closed, so live verification and the bug ledger remain the source of truth",
        "Frontend deploy asset recovery: the service worker and global error handlers detect stale chunks and safely reload onto the new version",
        "Public API-key chat entry point: admins can issue API keys for Web users, and clients can call Hone through the OpenAI-compatible `/api/public/v1/chat/completions` shape",
        "ACP self-managed context with compact-leak suppression for long codex_acp / opencode_acp sessions",
        "Multi-engine setup: OpenAI-compatible / Gemini CLI / Codex CLI/ACP / OpenCode ACP / multi-agent",
        "`scripts/diagnose_llm.sh` reads OpenRouter keys from the current LLM provider config paths while keeping legacy path compatibility",
      ],
    },
    next: {
      label: "Near term",
      items: [
        "Windows / Linux desktop builds",
        "User-facing skill editor (frontend for skill_manager)",
        "Broader data import / export tools (company-profile bundle transfer is live; portfolio and research-result user-facing migration surfaces still need coverage)",
        "Continue hardening cloud migration observability, rollback, and admin operations so deployers can reliably verify strict no-local-durable-dependency mode for PG / OSS deployments",
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
        title: "Hone Blog",
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
        q: "How is Hone different from a general AI chat tool?",
        a: "Hone won't flatter you. It treats research discipline as a hard constraint and actively pushes back on emotional decisions. Every conversation builds on long-term memory (company profiles), not a blank slate.",
      },
      {
        q: "Do I have to self-host?",
        a: "Three options: (1) the `curl | bash` installer for hone-cli; (2) a Homebrew tap; (3) clone the repo and start through the local CLI build path. The first two share the same GitHub release bundle — no Rust compile needed. Public SMS login requires Aliyun SMS configuration; if the behavior captcha gate is enabled, configure Aliyun Captcha environment variables too. After frontend upgrades, the public app uses asset recovery to handle load failures caused by stale cached chunks.",
      },
      {
        q: "Which LLMs are supported?",
        a: "Hone supports Hone Cloud, OpenAI-compatible protocols (including OpenRouter), Gemini CLI, Codex CLI / ACP, OpenCode ACP, and the multi-agent search-plus-answer flow through the agent-engine abstraction. Credentials live in `config.yaml` under `llm.providers.*.api_key/api_keys`, and generic OpenAI-compatible providers plus OpenRouter can try the next key in the pool.",
      },
      {
        q: "What license? Commercial use?",
        a: "MIT, commercial use allowed. The repo ships a fully working core engine, UI, desktop, all 16 public skills, and 7 channel integrations. Private premium skills and paid data sources live outside the repo and don't block the main flow.",
      },
      {
        q: "Where is data stored?",
        a: "Data still defaults to local storage or your self-hosted server (macOS desktop's `~/.honeclaw`). v0.12.4 adds Cloud PG / OSS runtime config; current main can place sessions, Web invites/auth sessions, conversation quota, cron jobs/runs, due-job claims, the skill registry, notification prefs, portfolio, LLM audit, and company profile files in PG, plus public uploads, generated images / files, and migrated documents in OSS. `cloud.strict_no_local_storage=true` checks the current config for remaining durable local dependencies. Hone does not host your data by default.",
      },
      {
        q: "How does Hone relate to Codex / RooCode and other coding agents?",
        a: "Hone borrows their agent-engine, skill, and session architecture but targets investment research, not coding. Codex CLI / ACP, Gemini CLI, OpenCode ACP, and multi-agent show up inside Hone as pluggable engines.",
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
      community: "Join community",
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
    prefs: {
      aria_label: "Font size and theme",
      font_size: "Size",
      theme: "Theme",
      theme_auto: "Auto",
      theme_light: "Light",
      theme_dark: "Dark",
    },
    status: {
      error: "Hone hit an error",
      streaming: "Hone is responding",
      running: "Hone is working",
      thinking: "Hone is thinking",
      done: "Done",
      fallback_error: "Request failed. Please try again.",
      stop: "Stop",
    },
    attachments: {
      image_title: "Image",
      image_subtitle: "Photos and screenshots",
      file_title: "File",
      file_subtitle: "PDF · documents · other",
    },
    composer: {
      quota_exhausted: "You've used today's chat quota",
      placeholder: "Ask Hone…",
      send_aria: "Send",
      proactive_tip: "Add holdings to enable push mode",
      proactive_title: "Hone can watch your holdings for you",
      proactive_intro:
        "Tell Hone what you hold or follow, and it will filter important changes by your preferences and reach out at the right time.",
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
    actions: {
      logout: "Log out",
      copy_aria: "Copy",
      copied: "Copied",
      scroll_to_bottom_aria: "Jump to latest",
      share_aria: "Share",
    },
    share: {
      brand_name: "Hone",
      brand_tagline: "Your AI investment co-pilot",
      qr_caption: "Scan to try Hone — an AI co-pilot for investors",
      strings: {
        title: "Share conversation",
        subtitle: "Pick from the latest 4 messages",
        preview_subtitle: "Preview the image, then save, copy, or share it",
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
        role_assistant: "Hone",
        nothing_selected: "Select at least one message",
        rendering: "Rendering…",
      },
    },
  },

  auth: {
    login: {
      title: "Sign in to Hone",
      subtitle: "Sign in with your phone number and SMS code.",
      hint_sms:
        "Hone is currently invite-only. Contact bm@hone-claw.com to join the invite list.",
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
        "Please read the following carefully. Continuing to use Hone means you accept these terms.",
      sections: [
        {
          title: "1. Acceptance & effective date",
          body: [
            {
              kind: "p",
              parts: [
                'Welcome to Hone ("the service"). The service is operated by ',
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
                "Hone is a research and decision-assistant tool for individual investors, offering information retrieval, conversational research, investment notes, and scheduled reminders.",
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
                "You sign in with a phone number we have registered and verify your identity with an SMS code. Hone is currently invite-only, and phone numbers outside the invite list cannot sign in.",
              ],
            },
            {
              kind: "p",
              parts: [
                "Keep your phone number, SMS codes, and signed-in devices secure. Do not share your account with others. If you notice unauthorized access, notify us immediately.",
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
                "The service may call third-party large language models (LLMs), market data, search engines, and similar providers to deliver features. Third-party services are operated independently; their stability, accuracy, and compliance are governed by their own official statements.",
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
          title: "7. Service changes, suspension & termination",
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
          title: "8. Disclaimers & limitation of liability",
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
                "The service is currently provided free of charge. To the maximum extent permitted by applicable law, we are not liable for any direct or indirect monetary loss you suffer from using or being unable to use the service (including but not limited to investment or trading losses, data loss, or lost profits).",
              ],
            },
          ],
        },
        {
          title: "9. Changes to these Terms",
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
          title: "10. Governing law & dispute resolution",
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
          title: "11. Contact",
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
        "We care about your data. This policy explains how Hone handles your personal information.",
      sections: [
        {
          title: "1. Introduction & scope",
          body: [
            {
              kind: "p",
              parts: [
                "This Privacy Policy describes how Hone (operated by ",
                { strong: "Snowdrift Capital LLC" },
                ', a Wyoming limited liability company, "we," "us," or "our") collects, uses, stores, shares, and protects your personal information while providing the service. It applies to every scenario in which you use the service through the Hone website or client.',
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
                  " phone number (as account identifier and invite-list key), SMS verification result, and historical invite records used as the invite-list source;",
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
                "Your account and conversation data are stored in the service's local SQLite database by default. SMS codes are sent and checked by a third-party SMS verification provider; we do not store plaintext verification codes.",
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
                "To fulfill your queries we may transmit relevant input to the following categories of third-party service providers:",
              ],
            },
            {
              kind: "ul",
              items: [
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
    mantra: "SHARPEN COGNITION · STRIP THE NOISE · OPEN FINANCIAL CONSOLE",
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
