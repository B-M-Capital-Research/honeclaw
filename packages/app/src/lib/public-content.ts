// public-content.ts — Hone Public Site Content (bilingual)
//
// Copy for the public surface (hone-claw.com) lives here in two parallel
// trees: CONTENT_ZH and CONTENT_EN. The exported `CONTENT` is a deep Proxy
// that reads the current locale via `useLocale()` on every property access,
// so JSX expressions like `{CONTENT.hero.headline_1}` or `<For each={CONTENT.cases.items}>`
// re-evaluate automatically when the locale signal changes.
//
// Adding a key: add it to BOTH trees with parallel shape.

import { useLocale } from "./i18n"

// ── Legal copy structured nodes (terms & privacy) ────────────────────────────
// Rich prose is modeled as a typed block tree so ZH/EN stay parallel and the
// pages render via a tiny interpreter instead of embedding JSX in content.
export type LegalInline = string | { strong: string } | { code: string }
export type LegalBlock =
  | { kind: "p"; parts: LegalInline[] }
  | { kind: "ul"; items: LegalInline[][] }
export type LegalSection = { title: string; body: LegalBlock[] }

const CONTENT_ZH = {
  nav: {
    logo_tagline: "OPEN FINANCIAL CONSOLE",
    home: "首页",
    roadmap: "路线图与文档",
    me: "个人",
    chat: "对话",
    menu_aria: "菜单",
    locale_zh: "中文",
    locale_en: "EN",
    contact_label: "联系",
    contact_wechat_label: "微信",
    contact_email_label: "邮箱",
    contact_wechat: "xiaobamang6677",
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
        title: "每周五自动触发复盘 Skill",
        body: "用 Cron 任务把固定工作流交给 Hone 自动完成。每周复盘、月度总结——无需手动触发。",
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
      { symbol: "⚡", title: "投研纪律约束", body: "对话时主动约束情绪决策，帮你坚守原则。不是复读你的想法，而是质疑它。" },
      { symbol: "◈", title: "公司画像 & 长期记忆", body: "对每家公司建立持久档案，跨会话积累研究成果，形成真正的知识资产。" },
      { symbol: "∞", title: "定时任务与自动提醒", body: "Cron 驱动的定时工作流，让复盘、持仓检查、重要节点提醒全自动运行。" },
      { symbol: "✦", title: "多端接入", body: "Web、iMessage、Lark / Feishu、Discord、Telegram、CLI——在你最顺手的地方使用 Hone。" },
      { symbol: "⌘", title: "Rust 驱动的稳定性", body: "核心引擎用 Rust 构建，低延迟、高可靠，长期运行不掉线、不崩溃。" },
      { symbol: "ℹ", title: "可编程投研操作系统", body: "自定义 Skill、动态任务链、跨会话记忆调用，构建完全属于你的投研工作流。" },
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
      { key: "discord", name: "Discord", desc: "英文社区讨论", url: "#", label: "开放", symbol: "⚡" },
      { key: "zsxq", name: "知识星球", desc: "付费深度内容", url: "#", label: "付费", symbol: "◈" },
      { key: "vip", name: "VIP 群", desc: "私域高级功能体验", url: "#", label: "邀请制", symbol: "✦" },
      { key: "content", name: "内容号", desc: "投研方法论 & 产品更新", url: "#", label: "关注", symbol: "∞" },
    ],
  },

  repo: {
    section_label: "开源",
    section_sub: "B&M Capital Research 出品，MIT 协议开放",
    items: [
      { title: "GitHub 仓库", desc: "Star、Fork、提 Issue，参与开源建设", url: "https://github.com/B-M-Capital-Research/honeclaw", tag: "开源", icon: "⌘" },
      { title: "中文文档", desc: "README、使用说明、案例示范", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md", tag: "文档", icon: "◈" },
      { title: "安装方式", desc: "macOS 桌面端 + 服务端自部署指南", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动", tag: "安装", icon: "⚡" },
      { title: "架构图", desc: "系统模块结构与技术架构说明", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md", tag: "技术", icon: "∞" },
      { title: "案例集", desc: "真实投研场景使用示例", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md", tag: "案例", icon: "✦" },
      { title: "贡献指南", desc: "参与开发、提交 PR、讨论功能方向", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md", tag: "贡献", icon: "ℹ" },
    ],
  },

  roadmap: {
    hero_title: "路线图与文档",
    hero_sub: "透明、务实、长期主义。下面是 Hone 目前能做什么、接下来做什么、以及如何接入你的投研工作流。",
    hero_meta: "ROADMAP · DOCS · API",
    sidebar_title: "ON THIS PAGE",
    version: "v0.2.6",

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
        intro: "三种方式接入 Hone：一键安装脚本、Homebrew、或源码开发。任选其一即可开始。",
      },
      capabilities: {
        eyebrow: "§ 02 · CAPABILITY MATRIX",
        title: "能力矩阵",
        legend: { stable: "生产可用", beta: "预览", planned: "规划中" },
      },
      channels: {
        eyebrow: "§ 03 · CHANNELS",
        title: "渠道接入",
        intro: "Hone 是多端接入的投研 agent。每个渠道都是独立进程，可独立启停、独立配置。",
      },
      architecture: {
        eyebrow: "§ 04 · ARCHITECTURE",
        title: "系统架构",
        intro: "Rust 核心引擎 · 多 Runner 抽象 · SolidJS 前端。设计目标：长时间运行不掉线、多渠道状态隔离、Skill 可热插拔。",
        footnote_prefix: "完整模块说明见",
        footnote_link: "AGENTS.md ↗",
      },
      skills: {
        eyebrow: "§ 05 · BUILT-IN SKILLS",
        title: "内置 Skill",
        intro_prefix: "Hone 的 Skill 由模型根据上下文自动调用。下面是仓库",
        intro_suffix: "目录下的 19 个公开 Skill。",
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
        intro: "MIT 协议开源。开源仓库包含完整可运行的核心系统，私域增强能力不公开但不影响主流程可用性。",
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
        { key: "curl" as const, label: "curl | bash", badge: "推荐" as string | null },
        { key: "brew" as const, label: "Homebrew", badge: null as string | null },
        { key: "source" as const, label: "源码 / launch.sh", badge: null as string | null },
      ],
      requirements_prefix: "系统要求：",
      curl: [
        "# macOS / Linux 一键安装（推荐）",
        "$ curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
      ],
      brew: [
        "# Homebrew tap (macOS / Linux)",
        "$ brew install B-M-Capital-Research/honeclaw/honeclaw",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
      ],
      source: [
        "# 源码开发模式（含桌面端 hot reload）",
        "$ git clone https://github.com/B-M-Capital-Research/honeclaw",
        "$ cd honeclaw",
        "$ ./launch.sh --desktop",
      ],
    },

    requirements: "macOS 13+ / Linux x86_64 / arm64 · 首次源码启动约 10 分钟（自动装 bun + rustup）",

    capability_matrix: [
      {
        group: "投研核心",
        rows: [
          { name: "投研纪律约束 & 零幻觉协议", status: "stable", note: "system prompt 强约束" },
          { name: "公司画像 & 长期记忆", status: "stable", note: "company_portrait skill" },
          { name: "个股研究 / 深度研究", status: "stable", note: "stock_research + deep_stock_research" },
          { name: "持仓追踪与提醒", status: "stable", note: "portfolio_management + cron" },
          { name: "估值 / 选股 / 仓位建议", status: "stable", note: "valuation / stock_selection / position_advice" },
          { name: "图表 & 图像生成", status: "stable", note: "chart_visualization / image_generation" },
          { name: "向量检索增强记忆", status: "planned", note: "规划中" },
        ],
      },
      {
        group: "运行时",
        rows: [
          { name: "Rust 核心引擎", status: "stable", note: "Tokio · axum · SSE" },
          { name: "SolidJS 前端", status: "stable", note: "Vite · Tailwind v4" },
          { name: "Tauri 桌面端", status: "stable", note: "macOS 已发布" },
          { name: "多 Runner 抽象", status: "stable", note: "OpenAI · Gemini CLI · Codex CLI/ACP · OpenCode ACP · multi-agent" },
          { name: "Windows / Linux 桌面端", status: "planned", note: "Tauri 多平台打包" },
        ],
      },
      {
        group: "扩展",
        rows: [
          { name: "Cron 定时任务", status: "stable", note: "scheduled_task skill + /api/cron-jobs" },
          { name: "自定义 Skill", status: "stable", note: "skill_manager · create_skill.sh" },
          { name: "MCP 协议", status: "stable", note: "hone-mcp 二进制可作为 MCP server" },
          { name: "HTTP + SSE 内部 API", status: "stable", note: "hone-web-api 路由全开" },
          { name: "按用户细粒度推送偏好", status: "stable", note: "notification_preferences skill + 设置页 + config 全局节流" },
          { name: "公开 Skill 市场", status: "planned", note: "社区共享" },
        ],
      },
    ],

    channels: [
      { name: "Web", icon: "⚡", status: "stable", desc: "邀请制聊天页，浏览器直开（hone-web-api）" },
      { name: "iMessage", icon: "✦", status: "stable", desc: "macOS 原生短信集成（hone-imessage）" },
      { name: "Lark / Feishu", icon: "◈", status: "stable", desc: "飞书机器人双向通信（hone-feishu）" },
      { name: "Discord", icon: "∞", status: "stable", desc: "Bot 应用集成（hone-discord）" },
      { name: "Telegram", icon: "⌘", status: "stable", desc: "Bot API 接入（hone-telegram）" },
      { name: "CLI", icon: "ℹ", status: "stable", desc: "命令行流式对话（hone-cli）" },
      { name: "MCP", icon: "✧", status: "stable", desc: "作为 MCP server 嵌入 Claude / Cursor 等（hone-mcp）" },
    ],

    skills: [
      { name: "stock_research", desc: "单只个股研究、估值框架、按条件筛选" },
      { name: "deep_stock_research", desc: "约 1–2 小时的深度研究任务（管理员）" },
      { name: "company_portrait", desc: "维护公司画像、thesis、事件时间线" },
      { name: "portfolio_management", desc: "持仓增减、再平衡、Ticker 校验" },
      { name: "position_advice", desc: "结合行情与持仓给出加减仓建议" },
      { name: "valuation", desc: "估值方法选择与区间推断" },
      { name: "stock_selection", desc: "按条件筛选潜在标的" },
      { name: "market_analysis", desc: "宏观、政策、行业动量与指数判断" },
      { name: "gold_analysis", desc: "黄金、金 ETF、金矿股的宏观与持仓分析" },
      { name: "scheduled_task", desc: "注册 / 修改 / 取消用户定时推送任务" },
      { name: "major_alert", desc: "重大事件 / 新闻预警推送" },
      { name: "one_sentence_memory", desc: "把对话沉淀成一句长期记忆" },
      { name: "chart_visualization", desc: "趋势 / 对比 / 分布 / 散点研究图" },
      { name: "image_generation", desc: "持仓截图、研究图卡、说明图" },
      { name: "image_understanding", desc: "解析用户上传的 K 线 / 持仓截图" },
      { name: "pdf_understanding", desc: "解析 PDF（财报、研报）输出要点与风险" },
      { name: "skill_manager", desc: "查看 / 新建 / 修改 Hone Skill" },
      { name: "notification_preferences", desc: "用自然语言调整自己的推送偏好（严重度、持仓过滤、kind 白/黑名单）" },
      { name: "hone_admin", desc: "查看修改 Hone 源码与配置（管理员）" },
    ],

    now: {
      label: "当前已有",
      items: [
        "Web 聊天界面（邀请制）+ 公开门面站",
        "Tauri macOS 桌面端 + 内置后端",
        "7 个渠道：Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "19 个内置 Skill（个股、持仓、估值、图表、PDF、Cron、推送偏好…）",
        "投研纪律约束 & 零幻觉协议",
        "公司画像与跨会话长期记忆",
        "Cron 定时任务系统",
        "事件引擎推送质量收口：digest 去重 / min-gap / topic memory / 分类预算 / 方向性价格阈值",
        "ACP 自管上下文与 compact 防泄漏，支持 codex_acp / opencode_acp 长会话恢复",
        "多 Runner 抽象：OpenAI / Gemini CLI / Codex CLI/ACP / OpenCode ACP / multi-agent",
      ],
    },
    next: {
      label: "近期计划",
      items: [
        "Windows / Linux 桌面端打包",
        "用户自定义 Skill 编辑器（前端化的 skill_manager）",
        "数据导入 / 导出工具",
        "公开 Skill 文档与示例集",
        "向量检索增强长期记忆",
      ],
    },
    later: {
      label: "长期愿景",
      items: [
        "多用户协作研究空间",
        "可视化持仓分析面板",
        "面向开发者的开放 API",
        "社区 Skill 市场",
        "多 Agent 协同编排",
      ],
    },

    boundary: {
      label: "开源边界",
      open: [
        "Rust 核心引擎 (hone-core / hone-channels / hone-llm / hone-tools)",
        "前端 UI (SolidJS + Tailwind v4)",
        "Tauri 桌面端壳",
        "全部 19 个公开 Skill",
        "全部渠道集成代码 (Web / iMessage / Lark / Discord / Telegram / CLI / MCP)",
      ],
      closed: [
        "私域高级 Skill 库",
        "付费数据源 API Key",
        "VIP 专属功能 / 托管服务",
      ],
    },

    docs: [
      { title: "README（English）", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README.md", desc: "Project overview, install, quick start" },
      { title: "README（中文）", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md", desc: "项目总览、安装、快速上手" },
      { title: "AGENTS.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md", desc: "Agent / Runner 架构与运行时约束" },
      { title: "Cases (中文)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md", desc: "真实投研场景使用示例集" },
      { title: "Cases (English)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_EN.md", desc: "Real-world case studies" },
      { title: "Skills 目录", url: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills", desc: "全部公开 Skill 的源码与说明" },
      { title: "CONTRIBUTING.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md", desc: "贡献指南" },
      { title: "SECURITY.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/SECURITY.md", desc: "漏洞披露策略" },
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
        a: "三种方式任选：①「curl | bash」一键装 hone-cli;②Homebrew tap;③clone 仓库 ./launch.sh --desktop 启动桌面端。前两种共享同一份 GitHub release bundle，不需要自己编译 Rust。",
      },
      {
        q: "支持哪些 LLM？",
        a: "通过 Runner 抽象层支持：OpenAI 兼容协议（含 OpenRouter）、Gemini CLI、Codex CLI / ACP、OpenCode ACP，以及 multi-agent 搜索+回答链路。可以在桌面端设置里随时切换。",
      },
      {
        q: "开源协议？能商用吗？",
        a: "MIT 协议，可商用。开源仓库包含完整可运行的核心引擎、UI、桌面端、全部 19 个公开 Skill 和 7 个渠道集成。私域高级 Skill 与付费数据源接入不在仓库中，不影响主流程。",
      },
      {
        q: "数据存在哪里？",
        a: "所有会话、公司画像、研究结果默认存储在本地（macOS 桌面端用户目录 ~/.honeclaw 或自部署服务器）。Hone 官方不托管用户数据。",
      },
      {
        q: "和 Codex / RooCode 等 coding agent 的关系？",
        a: "Hone 借鉴了这些产品的 runner / skill / session 架构，但专注投研而非写代码。Codex CLI / ACP、Gemini CLI、OpenCode ACP 和 multi-agent 在 Hone 中作为可插拔 Runner 存在。",
      },
    ],
  },

  me: {
    logged_in_title: "账号中心",
    logged_in_eyebrow: "账号中心",
    logged_out_title: "请先登录",
    logged_out_desc: "登录后查看你的使用额度、历史记录和账号信息。",
    logged_out_cta: "前往对话页登录",
    invite_note: "需要邀请码才能进入对话",
    loading: "加载中…",
    account_info_title: "账号信息",
    usage_today_label: "今日用量",
    date_locale: "zh-CN",
    date_placeholder: "—",
    stats: {
      remaining_today_label: "今日剩余",
      remaining_today_sub_template: "/ {daily} 次每日",
      total_label: "累计使用",
      total_sub: "次成功对话",
      daily_limit_label: "每日额度",
      daily_limit_sub: "次 / 天",
    },
    actions: {
      chat: "进入对话 →",
      roadmap: "查看路线图",
      community: "加入社群",
      logout: "退出登录",
    },
    membership: {
      title: "会员 / 高级功能（结构预留）",
      desc: "付费体系、VIP 群、私域高级 Skill——即将推出。加入社群获取第一手信息。",
    },
    fields: {
      user_id: "账号 ID",
      created_at: "注册时间",
      last_login: "最近登录",
      daily_limit: "每日额度",
      used_today: "今日已用",
      remaining: "剩余次数",
    },
  },

  auth: {
    login: {
      title: "登录 Hone",
      subtitle: "老用户请使用密码登录；新用户请用邀请码激活账号。",
      tab_password: "密码登录",
      tab_invite: "邀请码激活",
      hint_password: "已有账号：使用手机号 + 个人密码登录。",
      hint_invite:
        "新用户首次进入：凭邀请码激活账号。没有邀请码？可加微信 xiaobamang6677 或发邮件到 bm@hone-claw.com 获取；也务必帮忙点一个 Star。",
      phone_label: "手机号",
      phone_placeholder: "例如 13800138000",
      phone_aria: "手机号",
      password_label: "密码",
      password_placeholder: "您的密码",
      password_aria: "密码",
      invite_label: "邀请码",
      invite_placeholder: "HONE-XXXXXX-XXXXXX",
      invite_aria: "邀请码",
      remember_30d: "保持登录（30 天）",
      submit_password: "登录",
      submit_invite: "激活并登录",
      loading: "登录中…",
    },
    guard: {
      title: "首次登录：请设置密码",
      hint: "为保护账号安全，请设置一个个人密码。设置后，你将通过手机号 + 密码登录，无需再使用邀请码。",
      new_label: "新密码",
      new_placeholder: "至少 8 位，含字母与数字",
      confirm_label: "确认密码",
      confirm_placeholder: "再输入一次",
      error_mismatch: "两次输入的密码不一致",
      button_skip: "暂不设置 · 退出登录",
      button_submit: "保存并继续",
      loading: "保存中…",
    },
    password_field: {
      default_placeholder: "密码",
      show_aria: "显示密码",
      hide_aria: "隐藏密码",
      rule_length: "8–128 位",
      rule_letter: "至少一个字母",
      rule_digit: "至少一个数字",
    },
    change_password: {
      open_button: "修改密码",
      title: "修改密码",
      current_label: "当前密码",
      current_placeholder: "当前密码",
      current_aria: "当前密码",
      new_label: "新密码",
      new_placeholder: "至少 8 位，含字母与数字",
      new_aria: "新密码",
      confirm_label: "确认新密码",
      confirm_placeholder: "再输入一次",
      confirm_aria: "确认新密码",
      error_mismatch: "两次输入的密码不一致",
      success: "✓ 密码已更新。下次登录请使用新密码。",
      button_ok: "好的",
      button_submit: "保存",
      loading: "保存中…",
    },
    password_error: {
      length_template: "密码长度需 {min}–{max} 位",
      no_digit: "密码至少包含一位数字",
      no_letter: "密码至少包含一位字母",
    },
    tos: {
      prefix: "我已阅读并同意",
      terms: "《用户协议》",
      and: "和",
      privacy: "《隐私政策》",
      version_template: "（v{version}）",
    },
  },

  common: {
    cancel: "取消",
    close_aria: "关闭",
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
            { kind: "p", parts: ["欢迎使用 Hone（以下简称“本服务”）。本服务由 ", { strong: "杭州巴芒科技有限责任公司" }, "（以下简称“我们”）运营。本《用户协议》（以下简称“本协议”）是您与我们之间就您使用本服务所订立的有效合同。"] },
            { kind: "p", parts: ["您在勾选同意或继续使用本服务时，即视为您已充分阅读并同意本协议全部条款。若您不同意本协议任何条款，请立即停止使用本服务。"] },
          ],
        },
        {
          title: "2. 服务说明",
          body: [
            { kind: "p", parts: ["Hone 是一款面向个人投资者的研究与决策辅助工具，提供资料检索、对话式研究、投资笔记、定时提醒等能力。"] },
            { kind: "p", parts: [{ strong: "本服务不构成任何形式的投资建议、要约或推荐。" }, "本服务输出的全部内容仅供参考，任何投资决策均应由您本人独立作出并自行承担相应风险与后果。"] },
          ],
        },
        {
          title: "3. 账号与密码",
          body: [
            { kind: "p", parts: ["您需要使用经我们登记的手机号作为账号，并设置个人密码用于身份验证。您应妥善保管账号密码，不得将账号借予他人使用。"] },
            { kind: "p", parts: ["您应对在您账号下发生的所有行为负责。若发现账号被未经授权使用，您应立即通知我们并修改密码。"] },
          ],
        },
        {
          title: "4. 用户行为规范",
          body: [
            { kind: "p", parts: ["使用本服务时，您承诺不从事下列行为，包括但不限于："] },
            {
              kind: "ul",
              items: [
                ["违反中华人民共和国法律法规、社会主义核心价值观或公序良俗；"],
                ["危害国家安全、国家荣誉与利益，煽动颠覆国家政权、推翻社会主义制度，或破坏国家统一与领土完整；"],
                ["煽动民族仇恨、民族歧视，破坏民族团结，或宣扬恐怖主义、极端主义、暴力；"],
                ["制作、复制、传播淫秽色情、赌博、毒品或其他违法信息；"],
                ["通过提示词或其他方式诱导本服务输出违反前述规定的内容，包括但不限于政治敏感、反华、虚假信息、歧视性或仇恨性言论；"],
                ["侵犯他人合法权益，包括知识产权、隐私权、名誉权、商业秘密等；"],
                ["对本服务进行反向工程、爬取、批量自动化访问、漏洞利用或其他形式的滥用；"],
                ["上传或传播恶意代码、垃圾信息、违法或不良信息；"],
                ["冒用他人身份或伪造账号信息。"],
              ],
            },
            { kind: "p", parts: ["若您违反前述规定，我们有权立即暂停或终止您的账号、保留相关证据，必要时向有权机关报告。由此产生的全部法律责任由您本人承担。"] },
          ],
        },
        {
          title: "5. 内容与知识产权",
          body: [
            { kind: "p", parts: ["本服务及其相关界面、文案、代码、商标等所有相关知识产权归我们或合法权利人所有，受著作权法及相关法律法规保护。"] },
            { kind: "p", parts: ["您在本服务中输入的内容（包括对话、笔记、附件等）的著作权归您本人所有。您授予我们必要的、为提供和改进本服务所需的非排他性使用权。"] },
          ],
        },
        {
          title: "6. 第三方服务与数据源",
          body: [
            { kind: "p", parts: ["本服务可能调用第三方大型语言模型（LLM）、行情数据、搜索引擎等第三方服务以完成功能交付。第三方服务由其运营方独立提供，其稳定性、准确性及合规性以其官方声明为准。"] },
            { kind: "p", parts: ["您理解并同意，在调用第三方服务的过程中，我们可能向第三方传递必要的请求内容。我们将依照第三方服务条款选择正规、可信的合作方。"] },
          ],
        },
        {
          title: "7. 服务变更、中断与终止",
          body: [
            { kind: "p", parts: ["我们可能因升级维护、安全事件、不可抗力或经营调整等原因暂停、变更或终止部分或全部服务。我们将在合理范围内事先通过本服务内通知或其他方式告知。"] },
            { kind: "p", parts: ["若您严重违反本协议，我们有权立即暂停或终止向您提供服务，并保留依法追究责任的权利。"] },
          ],
        },
        {
          title: "8. 免责与责任限制",
          body: [
            { kind: "p", parts: ["在适用法律允许的最大范围内，本服务以“现状”和“现有”方式提供。我们不对服务的连续性、准确性、完整性、及时性作出任何明示或默示保证。"] },
            { kind: "p", parts: ["本服务目前以免费形式提供。在适用法律允许的最大范围内，我们不对您因使用或无法使用本服务而遭受的任何直接或间接损失（包括但不限于投资或交易损失、数据丢失、利润损失等）承担金钱赔偿责任。"] },
          ],
        },
        {
          title: "9. 协议变更与通知",
          body: [
            { kind: "p", parts: ["我们可能根据法律法规或业务调整需要修改本协议。修改后的协议将在本服务内公布，并标明版本号与生效日期。"] },
            { kind: "p", parts: ["重大修改将以站内提醒等方式提示您再次确认。若您在协议变更后继续使用本服务，即视为您接受修改后的协议。"] },
          ],
        },
        {
          title: "10. 争议解决与法律适用",
          body: [
            { kind: "p", parts: ["本协议的订立、效力、解释、履行及争议解决，均适用中华人民共和国大陆地区法律（不含港澳台地区法律）。"] },
            { kind: "p", parts: ["因本协议引起的或与之相关的任何争议，双方应首先协商解决；协商不成的，任何一方均可向运营方所在地（浙江省杭州市）有管辖权的人民法院提起诉讼。"] },
          ],
        },
        {
          title: "11. 联系方式",
          body: [
            { kind: "p", parts: ["若您对本协议有任何疑问、意见或建议，请在本项目 GitHub 仓库提交 issue 联系我们："] },
            { kind: "p", parts: [{ code: "https://github.com/B-M-Capital-Research/honeclaw/issues" }] },
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
            { kind: "p", parts: ["本《隐私政策》说明 Hone（运营方为 ", { strong: "杭州巴芒科技有限责任公司" }, "，以下简称“我们”）在提供服务过程中如何收集、使用、存储、共享和保护您的个人信息。本政策适用于您通过 Hone 网站及客户端使用本服务的全部场景。"] },
            { kind: "p", parts: ["请您在使用本服务前完整阅读本政策。继续使用本服务即视为您已充分了解并同意本政策。"] },
          ],
        },
        {
          title: "2. 我们收集的信息",
          body: [
            { kind: "p", parts: ["为提供服务，我们会按最小必要原则收集下列类别的信息："] },
            {
              kind: "ul",
              items: [
                [{ strong: "账号信息：" }, "手机号（作为账号识别）、邀请码（用于初次注册）、密码哈希（我们仅存储 Argon2id 哈希，绝不存储明文密码）；"],
                [{ strong: "使用数据：" }, "对话记录、提问与回复内容、上传的附件、笔记与定时任务；"],
                [{ strong: "设备与日志：" }, "IP 地址、浏览器类型、访问时间戳、错误日志、Cookie 标识；"],
                [{ strong: "授权事件：" }, "用户协议与隐私政策的接受版本与时间。"],
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
            { kind: "p", parts: ["您的账号与对话数据默认存储于本服务的本地 SQLite 数据库中。密码采用 ", { strong: "Argon2id" }, " 算法配合随机盐进行哈希存储，我们无法恢复您的明文密码。"] },
            { kind: "p", parts: ["我们采用 HTTPS 加密传输、最小权限访问控制、密码哈希等技术与管理措施，保护您的信息安全。在法律允许范围内，我们将在为完成相应目的所必需的期间内保留您的信息。"] },
          ],
        },
        {
          title: "5. 信息共享与第三方",
          body: [
            { kind: "p", parts: ["为完成您发起的查询，我们可能将您输入的相关内容传递给以下类别的第三方服务方："] },
            {
              kind: "ul",
              items: [
                ["大型语言模型提供方（用于生成回复）；"],
                ["行情数据与搜索数据源（用于补充查询所需的市场或公开信息）。"],
              ],
            },
            { kind: "p", parts: ["除上述必要场景以及法律法规另有规定外，我们不会向任何第三方出售或出租您的个人信息。"] },
          ],
        },
        {
          title: "6. Cookie 与追踪",
          body: [
            { kind: "p", parts: ["我们使用名为 ", { code: "hone_web_session" }, " 的 HTTP-only Cookie 维持登录态。该 Cookie 在您勾选“保持登录”时有效期为 30 天，否则为 1 天。"] },
            { kind: "p", parts: ["我们不使用第三方广告追踪 Cookie。"] },
          ],
        },
        {
          title: "7. 未成年人保护",
          body: [
            { kind: "p", parts: ["本服务面向 18 周岁以上具有完全民事行为能力的成年人。若您是未成年人，请在监护人指导下使用本服务。我们不会主动收集未成年人的个人信息。"] },
          ],
        },
        {
          title: "8. 跨境传输",
          body: [
            { kind: "p", parts: ["若我们调用的语言模型或数据源服务器位于中华人民共和国大陆地区以外，您的相关查询内容可能被传输至境外。我们会选择具备合规资质的合作方，并采取必要的安全措施。"] },
          ],
        },
        {
          title: "9. 用户权利",
          body: [
            { kind: "p", parts: ["就您的个人信息，您依法享有下列权利："] },
            {
              kind: "ul",
              items: [
                ["访问、更正您的账号资料；"],
                ["修改您的登录密码；"],
                ["请求删除您的账号及关联数据；"],
                ["撤回您此前给出的同意。"],
              ],
            },
            { kind: "p", parts: ["您可在“个人页面”中行使前三项权利，或通过下文联系方式与我们联系。撤回同意可能导致您无法继续使用部分功能。"] },
          ],
        },
        {
          title: "10. 政策更新",
          body: [
            { kind: "p", parts: ["我们可能根据法律法规变化或业务调整需要更新本政策。更新后的政策将在本服务内公布，并标明版本号与生效日期；重大变更将以站内提醒等方式向您提示。"] },
          ],
        },
        {
          title: "11. 联系方式",
          body: [
            { kind: "p", parts: ["若您对本政策或您的个人信息处理有任何疑问、意见或投诉，请在本项目 GitHub 仓库提交 issue 联系我们："] },
            { kind: "p", parts: [{ code: "https://github.com/B-M-Capital-Research/honeclaw/issues" }] },
            { kind: "p", parts: ["我们将在合理时间内回复并妥善处理。"] },
          ],
        },
      ] as LegalSection[],
    },
  },

  footer: {
    tagline: "磨砺认知，剔除噪音",
    mantra: "磨砺认知 · 剔除噪音 · OPEN FINANCIAL CONSOLE",
    copyright: "© 2026 杭州巴芒科技有限责任公司. Open source, MIT License.",
    columns: {
      product: {
        title: "产品",
        items: [
          { label: "首页", href: "/" },
          { label: "路线图", href: "/roadmap" },
          { label: "对话", href: "/chat" },
          { label: "个人", href: "/me" },
          { label: "用户协议", href: "/terms" },
          { label: "隐私政策", href: "/privacy" },
        ],
      },
      resources: {
        title: "资源",
        items: [
          { label: "GitHub", href: "https://github.com/B-M-Capital-Research/honeclaw" },
          { label: "中文文档", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md" },
          { label: "安装方式", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动" },
          { label: "架构图", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md" },
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
    },
  },
}

const CONTENT_EN: typeof CONTENT_ZH = {
  nav: {
    logo_tagline: "OPEN FINANCIAL CONSOLE",
    home: "Home",
    roadmap: "Roadmap & Docs",
    me: "Account",
    chat: "Chat",
    menu_aria: "Menu",
    locale_zh: "中文",
    locale_en: "EN",
    contact_label: "Contact",
    contact_wechat_label: "WeChat",
    contact_email_label: "Email",
    contact_wechat: "xiaobamang6677",
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
        title: "Trigger a weekly review skill every Friday",
        body: "Hand fixed workflows to Hone via cron. Weekly reviews, monthly summaries — all run themselves.",
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
        body: "Not just the web. Hone reaches you through iMessage, Lark, Discord and more — in whatever surface you're already using.",
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
    coverage: "Covered: deep single-stock research, portfolio tracking, scheduled tasks, multi-channel demo",
    url_placeholder: "Video link not configured yet (set video_url)",
  },

  capabilities: {
    section_label: "CORE CAPABILITIES",
    items: [
      { symbol: "⚡", title: "Research discipline", body: "Constrains emotional decisions in-conversation. It doesn't echo your thinking — it questions it." },
      { symbol: "◈", title: "Company profiles & long memory", body: "A persistent dossier per company; research compounds across sessions into a real knowledge asset." },
      { symbol: "∞", title: "Scheduled tasks & alerts", body: "Cron-driven workflows: reviews, portfolio checks, key-moment alerts — all running on their own." },
      { symbol: "✦", title: "Multi-surface access", body: "Web, iMessage, Lark / Feishu, Discord, Telegram, CLI — Hone on whichever surface you already live in." },
      { symbol: "⌘", title: "Rust-powered stability", body: "Core engine built in Rust — low latency, high reliability, no drift or crash on long runs." },
      { symbol: "ℹ", title: "Programmable research OS", body: "Custom skills, dynamic task chains, cross-session memory — compose a workflow that's fully yours." },
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
      { key: "discord", name: "Discord", desc: "English community discussion", url: "#", label: "Open", symbol: "⚡" },
      { key: "zsxq", name: "Zhishixingqiu", desc: "Paid deep-dive content", url: "#", label: "Paid", symbol: "◈" },
      { key: "vip", name: "VIP group", desc: "Premium / private feature preview", url: "#", label: "Invite", symbol: "✦" },
      { key: "content", name: "Content channel", desc: "Research methodology & product updates", url: "#", label: "Follow", symbol: "∞" },
    ],
  },

  repo: {
    section_label: "OPEN SOURCE",
    section_sub: "Made by B&M Capital Research. MIT licensed.",
    items: [
      { title: "GitHub repo", desc: "Star, fork, open issues, help build in the open", url: "https://github.com/B-M-Capital-Research/honeclaw", tag: "Source", icon: "⌘" },
      { title: "Chinese docs", desc: "README, usage guide, case studies", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md", tag: "Docs", icon: "◈" },
      { title: "Install guide", desc: "macOS desktop + self-hosted server setup", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动", tag: "Install", icon: "⚡" },
      { title: "Architecture", desc: "Module structure and runtime constraints", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md", tag: "Tech", icon: "∞" },
      { title: "Case studies", desc: "Real-world research scenarios", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md", tag: "Cases", icon: "✦" },
      { title: "Contributing", desc: "How to contribute code, ideas, and skills", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md", tag: "Contribute", icon: "ℹ" },
    ],
  },

  roadmap: {
    hero_title: "Roadmap & Docs",
    hero_sub: "Transparent, pragmatic, long-term. Here's what Hone does today, what's next, and how to bring it into your research workflow.",
    hero_meta: "ROADMAP · DOCS · API",
    sidebar_title: "ON THIS PAGE",
    version: "v0.2.6",

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
        intro: "Three paths to run Hone: the one-line installer, Homebrew, or source. Pick whichever fits.",
      },
      capabilities: {
        eyebrow: "§ 02 · CAPABILITY MATRIX",
        title: "Capability Matrix",
        legend: { stable: "Production", beta: "Preview", planned: "Planned" },
      },
      channels: {
        eyebrow: "§ 03 · CHANNELS",
        title: "Channels",
        intro: "Hone is a multi-surface research agent. Each channel is an independent process — start, stop, and configure them on their own.",
      },
      architecture: {
        eyebrow: "§ 04 · ARCHITECTURE",
        title: "Architecture",
        intro: "Rust core · multi-runner abstraction · SolidJS frontend. Designed for long uptime, per-channel isolation, and hot-pluggable skills.",
        footnote_prefix: "Full module walkthrough in",
        footnote_link: "AGENTS.md ↗",
      },
      skills: {
        eyebrow: "§ 05 · BUILT-IN SKILLS",
        title: "Built-in Skills",
        intro_prefix: "Hone's skills are invoked by the model from context. Below are the 19 public skills in the",
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
        intro: "MIT licensed. The repo contains a fully-working core; premium additions stay closed but don't block the main flow.",
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
        intro: "Hone is open source. Every kind of contribution is welcome — not just code.",
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
        { key: "source" as const, label: "Source / launch.sh", badge: null },
      ],
      requirements_prefix: "Requirements:",
      curl: [
        "# macOS / Linux one-line install (recommended)",
        "$ curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
      ],
      brew: [
        "# Homebrew tap (macOS / Linux)",
        "$ brew install B-M-Capital-Research/honeclaw/honeclaw",
        "$ hone-cli doctor",
        "$ hone-cli onboard",
      ],
      source: [
        "# Source dev mode (desktop hot reload included)",
        "$ git clone https://github.com/B-M-Capital-Research/honeclaw",
        "$ cd honeclaw",
        "$ ./launch.sh --desktop",
      ],
    },

    requirements: "macOS 13+ / Linux x86_64 / arm64 · first source build ~10 min (bun + rustup auto-installed)",

    capability_matrix: [
      {
        group: "Research core",
        rows: [
          { name: "Research discipline & zero-hallucination protocol", status: "stable", note: "hardened system prompt" },
          { name: "Company profiles & long memory", status: "stable", note: "company_portrait skill" },
          { name: "Stock research / deep research", status: "stable", note: "stock_research + deep_stock_research" },
          { name: "Portfolio tracking & alerts", status: "stable", note: "portfolio_management + cron" },
          { name: "Valuation / selection / position advice", status: "stable", note: "valuation / stock_selection / position_advice" },
          { name: "Chart & image generation", status: "stable", note: "chart_visualization / image_generation" },
          { name: "Vector-augmented memory", status: "planned", note: "planned" },
        ],
      },
      {
        group: "Runtime",
        rows: [
          { name: "Rust core engine", status: "stable", note: "Tokio · axum · SSE" },
          { name: "SolidJS frontend", status: "stable", note: "Vite · Tailwind v4" },
          { name: "Tauri desktop", status: "stable", note: "macOS released" },
          { name: "Multi-runner abstraction", status: "stable", note: "OpenAI · Gemini CLI · Codex CLI/ACP · OpenCode ACP · multi-agent" },
          { name: "Windows / Linux desktop", status: "planned", note: "Tauri multi-platform packaging" },
        ],
      },
      {
        group: "Extensions",
        rows: [
          { name: "Cron scheduled tasks", status: "stable", note: "scheduled_task skill + /api/cron-jobs" },
          { name: "Custom skills", status: "stable", note: "skill_manager · create_skill.sh" },
          { name: "MCP protocol", status: "stable", note: "hone-mcp binary can act as an MCP server" },
          { name: "HTTP + SSE internal API", status: "stable", note: "hone-web-api fully exposed" },
          { name: "Per-actor notification prefs", status: "stable", note: "notification_preferences skill + settings page + config-level mute" },
          { name: "Public skill marketplace", status: "planned", note: "community sharing" },
        ],
      },
    ],

    channels: [
      { name: "Web", icon: "⚡", status: "stable", desc: "Invite-only chat, opens in browser (hone-web-api)" },
      { name: "iMessage", icon: "✦", status: "stable", desc: "Native macOS SMS integration (hone-imessage)" },
      { name: "Lark / Feishu", icon: "◈", status: "stable", desc: "Two-way Feishu bot (hone-feishu)" },
      { name: "Discord", icon: "∞", status: "stable", desc: "Bot application integration (hone-discord)" },
      { name: "Telegram", icon: "⌘", status: "stable", desc: "Bot API integration (hone-telegram)" },
      { name: "CLI", icon: "ℹ", status: "stable", desc: "Streaming CLI chat (hone-cli)" },
      { name: "MCP", icon: "✧", status: "stable", desc: "Run as MCP server inside Claude / Cursor, etc. (hone-mcp)" },
    ],

    skills: [
      { name: "stock_research", desc: "Single-stock research, valuation, conditional screening" },
      { name: "deep_stock_research", desc: "1–2 hour deep research tasks (admin only)" },
      { name: "company_portrait", desc: "Maintain company profiles, theses, and event timelines" },
      { name: "portfolio_management", desc: "Add, trim, rebalance, validate tickers" },
      { name: "position_advice", desc: "Suggest adds / trims from market + position context" },
      { name: "valuation", desc: "Pick valuation methods and derive price ranges" },
      { name: "stock_selection", desc: "Screen candidates by your criteria" },
      { name: "market_analysis", desc: "Macro, policy, sector momentum, index calls" },
      { name: "gold_analysis", desc: "Gold, gold ETFs, and miners — macro and positioning" },
      { name: "scheduled_task", desc: "Register / modify / cancel scheduled pushes" },
      { name: "major_alert", desc: "Send major-event / news alerts" },
      { name: "one_sentence_memory", desc: "Distill a conversation into one durable sentence" },
      { name: "chart_visualization", desc: "Trend, comparison, distribution, scatter charts" },
      { name: "image_generation", desc: "Portfolio screenshots, research visuals, explainers" },
      { name: "image_understanding", desc: "Parse K-line / portfolio screenshots from users" },
      { name: "pdf_understanding", desc: "Parse PDFs (filings, reports) into key points and risks" },
      { name: "skill_manager", desc: "View / create / edit Hone skills" },
      { name: "notification_preferences", desc: "Tune your own push prefs in natural language (severity, portfolio-only, kind allow/block)" },
      { name: "hone_admin", desc: "Inspect and modify Hone source & config (admin)" },
    ],

    now: {
      label: "Shipping today",
      items: [
        "Web chat (invite-only) + public landing site",
        "Tauri macOS desktop with bundled backend",
        "7 channels: Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "19 built-in skills (stocks, portfolio, valuation, charts, PDF, cron, notification prefs…)",
        "Research discipline & zero-hallucination protocol",
        "Company profiles + cross-session long memory",
        "Cron-driven scheduled tasks",
        "Event-engine push-quality pass: digest dedupe / min-gap / topic memory / category budgets / directional price thresholds",
        "ACP self-managed context with compact-leak suppression for long codex_acp / opencode_acp sessions",
        "Multi-runner: OpenAI / Gemini CLI / Codex CLI/ACP / OpenCode ACP / multi-agent",
      ],
    },
    next: {
      label: "Near term",
      items: [
        "Windows / Linux desktop builds",
        "User-facing skill editor (frontend for skill_manager)",
        "Data import / export tools",
        "Public skill documentation and example pack",
        "Vector-augmented long memory",
      ],
    },
    later: {
      label: "Long horizon",
      items: [
        "Multi-user collaborative research space",
        "Visual portfolio analytics dashboard",
        "Open API for developers",
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
        "All 19 public skills",
        "All channel integrations (Web / iMessage / Lark / Discord / Telegram / CLI / MCP)",
      ],
      closed: [
        "Private premium skill library",
        "Paid data-source API keys",
        "VIP-only features / hosted services",
      ],
    },

    docs: [
      { title: "README (English)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README.md", desc: "Project overview, install, quick start" },
      { title: "README (中文)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md", desc: "Overview, install, quick start in Chinese" },
      { title: "AGENTS.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md", desc: "Agent / runner architecture and runtime rules" },
      { title: "Cases (中文)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_ZH.md", desc: "Real-world research scenario examples" },
      { title: "Cases (English)", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CASES_EN.md", desc: "Real-world case studies" },
      { title: "Skills directory", url: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills", desc: "Source and notes for every public skill" },
      { title: "CONTRIBUTING.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/CONTRIBUTING.md", desc: "Contribution guide" },
      { title: "SECURITY.md", url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/SECURITY.md", desc: "Vulnerability disclosure policy" },
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
        a: "Three options: (1) the `curl | bash` installer for hone-cli; (2) a Homebrew tap; (3) clone the repo and run `./launch.sh --desktop`. The first two share the same GitHub release bundle — no Rust compile needed.",
      },
      {
        q: "Which LLMs are supported?",
        a: "Through the runner abstraction: OpenAI-compatible protocols (including OpenRouter), Gemini CLI, Codex CLI / ACP, OpenCode ACP, and the multi-agent search-plus-answer flow. Switch at any time from the desktop settings.",
      },
      {
        q: "What license? Commercial use?",
        a: "MIT, commercial use allowed. The repo ships a fully-working core engine, UI, desktop, all 19 public skills, and 7 channel integrations. Private premium skills and paid data sources live outside the repo and don't block the main flow.",
      },
      {
        q: "Where is data stored?",
        a: "Sessions, company profiles, and research results default to local storage (macOS desktop's `~/.honeclaw` or your self-hosted server). Hone does not host user data.",
      },
      {
        q: "How does Hone relate to Codex / RooCode and other coding agents?",
        a: "Hone borrows their runner / skill / session architecture but targets investment research, not coding. Codex CLI / ACP, Gemini CLI, OpenCode ACP, and multi-agent show up inside Hone as pluggable runners.",
      },
    ],
  },

  me: {
    logged_in_title: "Account",
    logged_in_eyebrow: "Account Center",
    logged_out_title: "Sign in first",
    logged_out_desc: "Sign in to see your usage, history, and account info.",
    logged_out_cta: "Go to chat to sign in",
    invite_note: "An invite code is required to enter chat",
    loading: "Loading…",
    account_info_title: "Account info",
    usage_today_label: "Today's usage",
    date_locale: "en-US",
    date_placeholder: "—",
    stats: {
      remaining_today_label: "Left today",
      remaining_today_sub_template: "/ {daily} per day",
      total_label: "Total",
      total_sub: "successful chats",
      daily_limit_label: "Daily quota",
      daily_limit_sub: "per day",
    },
    actions: {
      chat: "Enter chat →",
      roadmap: "View roadmap",
      community: "Join community",
      logout: "Sign out",
    },
    membership: {
      title: "Membership / premium (placeholder)",
      desc: "Billing, VIP group, private premium skills — coming soon. Join the community to hear first.",
    },
    fields: {
      user_id: "Account ID",
      created_at: "Joined",
      last_login: "Last login",
      daily_limit: "Daily quota",
      used_today: "Used today",
      remaining: "Remaining",
    },
  },

  auth: {
    login: {
      title: "Sign in to Hone",
      subtitle: "Existing users sign in with a password; new users activate with an invite code.",
      tab_password: "Password",
      tab_invite: "Invite code",
      hint_password: "Already have an account: sign in with phone + password.",
      hint_invite:
        "First time in: activate your account with an invite code. Need one? Add WeChat xiaobamang6677 or email bm@hone-claw.com; please also star us on GitHub.",
      phone_label: "Phone",
      phone_placeholder: "e.g. +1 555 0134",
      phone_aria: "Phone",
      password_label: "Password",
      password_placeholder: "Your password",
      password_aria: "Password",
      invite_label: "Invite code",
      invite_placeholder: "HONE-XXXXXX-XXXXXX",
      invite_aria: "Invite code",
      remember_30d: "Keep me signed in (30 days)",
      submit_password: "Sign in",
      submit_invite: "Activate & sign in",
      loading: "Signing in…",
    },
    guard: {
      title: "First sign-in: set a password",
      hint: "Set a personal password to protect your account. From now on you'll sign in with phone + password — no invite code needed.",
      new_label: "New password",
      new_placeholder: "At least 8 chars, with letters and numbers",
      confirm_label: "Confirm password",
      confirm_placeholder: "Type it again",
      error_mismatch: "Passwords do not match",
      button_skip: "Skip for now · Sign out",
      button_submit: "Save & continue",
      loading: "Saving…",
    },
    password_field: {
      default_placeholder: "Password",
      show_aria: "Show password",
      hide_aria: "Hide password",
      rule_length: "8–128 characters",
      rule_letter: "At least one letter",
      rule_digit: "At least one digit",
    },
    change_password: {
      open_button: "Change password",
      title: "Change password",
      current_label: "Current password",
      current_placeholder: "Current password",
      current_aria: "Current password",
      new_label: "New password",
      new_placeholder: "At least 8 chars, with letters and numbers",
      new_aria: "New password",
      confirm_label: "Confirm new password",
      confirm_placeholder: "Type it again",
      confirm_aria: "Confirm new password",
      error_mismatch: "Passwords do not match",
      success: "✓ Password updated. Use the new password next time you sign in.",
      button_ok: "Got it",
      button_submit: "Save",
      loading: "Saving…",
    },
    password_error: {
      length_template: "Password must be {min}–{max} characters",
      no_digit: "Password must contain at least one digit",
      no_letter: "Password must contain at least one letter",
    },
    tos: {
      prefix: "I have read and agree to the ",
      terms: "Terms of Service",
      and: " and ",
      privacy: "Privacy Policy",
      version_template: " (v{version})",
    },
  },

  common: {
    cancel: "Cancel",
    close_aria: "Close",
  },

  legal: {
    version_banner_template: "v{version} · effective {date}",
    terms: {
      page_title: "Terms of Service",
      intro: "Please read the following carefully. Continuing to use Hone means you accept these terms.",
      sections: [
        {
          title: "1. Acceptance & effective date",
          body: [
            { kind: "p", parts: ["Welcome to Hone (\"the service\"). The service is operated by ", { strong: "Hangzhou Bamang Technology Co., Ltd." }, " (\"we\"). These Terms of Service (\"Terms\") form a binding agreement between you and us regarding your use of the service."] },
            { kind: "p", parts: ["By checking the agreement box or continuing to use the service, you confirm that you have read and accept these Terms in full. If you disagree with any clause, stop using the service immediately."] },
          ],
        },
        {
          title: "2. Service description",
          body: [
            { kind: "p", parts: ["Hone is a research and decision-assistant tool for individual investors, offering information retrieval, conversational research, investment notes, and scheduled reminders."] },
            { kind: "p", parts: [{ strong: "The service does not constitute investment advice, an offer, or a recommendation of any kind." }, " All output from the service is for reference only; every investment decision is yours to make and yours to bear."] },
          ],
        },
        {
          title: "3. Account & password",
          body: [
            { kind: "p", parts: ["You sign in with a phone number we have registered and set a personal password for authentication. Keep your credentials safe and do not share your account with others."] },
            { kind: "p", parts: ["You are responsible for everything that happens under your account. If you notice unauthorized access, notify us and change your password immediately."] },
          ],
        },
        {
          title: "4. Acceptable use",
          body: [
            { kind: "p", parts: ["When using the service, you agree not to (including but not limited to):"] },
            {
              kind: "ul",
              items: [
                ["violate the laws and regulations of the People's Republic of China, the core socialist values, or public order and morals;"],
                ["endanger national security, honor, or interests; incite subversion of state power or the socialist system; or undermine national unity or territorial integrity;"],
                ["incite ethnic hatred or discrimination, undermine ethnic unity, or promote terrorism, extremism, or violence;"],
                ["produce, reproduce, or distribute pornographic, gambling, drug-related, or other unlawful content;"],
                ["use prompts or any other means to induce the service to produce content that violates the above (including but not limited to politically sensitive material, anti-China content, disinformation, or discriminatory or hateful speech);"],
                ["infringe on others' rights, including intellectual property, privacy, reputation, or trade secrets;"],
                ["reverse-engineer, scrape, bulk-automate, exploit vulnerabilities, or otherwise abuse the service;"],
                ["upload or distribute malware, spam, or unlawful or harmful content;"],
                ["impersonate others or falsify account information."],
              ],
            },
            { kind: "p", parts: ["If you violate the above, we may immediately suspend or terminate your account, preserve relevant evidence, and report to competent authorities where necessary. You bear sole legal responsibility for any consequences."] },
          ],
        },
        {
          title: "5. Content & intellectual property",
          body: [
            { kind: "p", parts: ["All intellectual property rights in the service — interface, copy, code, marks, and related materials — belong to us or our lawful rights holders, protected by copyright and related laws."] },
            { kind: "p", parts: ["Content you input (conversations, notes, attachments, etc.) remains yours. You grant us a non-exclusive license, limited to what is necessary to operate and improve the service."] },
          ],
        },
        {
          title: "6. Third-party services & data sources",
          body: [
            { kind: "p", parts: ["The service may call third-party large language models (LLMs), market data, search engines, and similar providers to deliver features. Third-party services are operated independently; their stability, accuracy, and compliance are governed by their own official statements."] },
            { kind: "p", parts: ["You understand and agree that, when calling a third-party service, we may transmit the necessary request content. We will choose reputable and trustworthy partners in line with their terms."] },
          ],
        },
        {
          title: "7. Service changes, suspension & termination",
          body: [
            { kind: "p", parts: ["We may suspend, change, or terminate part or all of the service for upgrades, maintenance, security incidents, force majeure, or business adjustments. We will give reasonable prior notice through in-service messages or other channels."] },
            { kind: "p", parts: ["If you materially breach these Terms, we may suspend or terminate your access immediately and reserve the right to pursue remedies under the law."] },
          ],
        },
        {
          title: "8. Disclaimers & limitation of liability",
          body: [
            { kind: "p", parts: ["To the maximum extent permitted by applicable law, the service is provided \"as is\" and \"as available.\" We make no express or implied warranty of continuity, accuracy, completeness, or timeliness."] },
            { kind: "p", parts: ["The service is currently provided free of charge. To the maximum extent permitted by applicable law, we are not liable for any direct or indirect monetary loss you suffer from using or being unable to use the service (including but not limited to investment or trading losses, data loss, or lost profits)."] },
          ],
        },
        {
          title: "9. Changes to these Terms",
          body: [
            { kind: "p", parts: ["We may revise these Terms to reflect changes in law or our business. Updated Terms will be published in-service with a version number and effective date."] },
            { kind: "p", parts: ["Material changes will be surfaced via in-service notice for reconfirmation. Continuing to use the service after an update means you accept the revised Terms."] },
          ],
        },
        {
          title: "10. Governing law & dispute resolution",
          body: [
            { kind: "p", parts: ["The formation, validity, interpretation, performance, and dispute resolution of these Terms are governed by the laws of mainland China (excluding Hong Kong SAR, Macao SAR, and Taiwan)."] },
            { kind: "p", parts: ["Any dispute arising from or related to these Terms should first be addressed by good-faith negotiation. If negotiation fails, either party may bring a claim before the competent people's court at the operator's place of registration (Hangzhou, Zhejiang Province)."] },
          ],
        },
        {
          title: "11. Contact",
          body: [
            { kind: "p", parts: ["If you have any questions, comments, or suggestions about these Terms, please open an issue on the project's GitHub repository:"] },
            { kind: "p", parts: [{ code: "https://github.com/B-M-Capital-Research/honeclaw/issues" }] },
            { kind: "p", parts: ["We will respond within a reasonable period."] },
          ],
        },
      ] as LegalSection[],
    },
    privacy: {
      page_title: "Privacy Policy",
      intro: "We care about your data. This policy explains how Hone handles your personal information.",
      sections: [
        {
          title: "1. Introduction & scope",
          body: [
            { kind: "p", parts: ["This Privacy Policy describes how Hone (operated by ", { strong: "Hangzhou Bamang Technology Co., Ltd." }, ", \"we\") collects, uses, stores, shares, and protects your personal information while providing the service. It applies to every scenario in which you use the service through the Hone website or client."] },
            { kind: "p", parts: ["Please read this policy in full before using the service. Continuing to use it means you have understood and accepted the policy."] },
          ],
        },
        {
          title: "2. Information we collect",
          body: [
            { kind: "p", parts: ["To provide the service, we collect the following categories of information under the principle of data minimization:"] },
            {
              kind: "ul",
              items: [
                [{ strong: "Account info:" }, " phone number (as account identifier), invite code (for initial registration), and a password hash (we only store the Argon2id hash; we never store plaintext passwords);"],
                [{ strong: "Usage data:" }, " conversation history, prompts and responses, uploaded attachments, notes, and scheduled tasks;"],
                [{ strong: "Device & logs:" }, " IP address, browser type, access timestamps, error logs, cookie identifiers;"],
                [{ strong: "Consent events:" }, " the version and time at which you accepted the Terms and this policy."],
              ],
            },
          ],
        },
        {
          title: "3. How we use it",
          body: [
            { kind: "p", parts: ["We use the above information for the following purposes:"] },
            {
              kind: "ul",
              items: [
                ["authentication, session maintenance, account risk control, and rate limiting;"],
                ["calling large language models and external data sources to fulfill your queries;"],
                ["recording session context to enable continuous conversation;"],
                ["troubleshooting, security incident response, and service optimization."],
              ],
            },
          ],
        },
        {
          title: "4. Storage, retention & security",
          body: [
            { kind: "p", parts: ["Your account and conversation data are stored in the service's local SQLite database by default. Passwords are hashed with ", { strong: "Argon2id" }, " and a random salt; we cannot recover plaintext passwords."] },
            { kind: "p", parts: ["We protect your information with HTTPS in transit, least-privilege access controls, password hashing, and other technical and organizational measures. Within the limits of applicable law, we retain information only for as long as necessary to meet the stated purposes."] },
          ],
        },
        {
          title: "5. Sharing & third parties",
          body: [
            { kind: "p", parts: ["To fulfill your queries we may transmit relevant input to the following categories of third-party service providers:"] },
            {
              kind: "ul",
              items: [
                ["large language model providers (to generate responses);"],
                ["market data and search data sources (to supplement queries with market or public information)."],
              ],
            },
            { kind: "p", parts: ["Except for the necessary scenarios above or as otherwise required by law, we do not sell or lease your personal information to any third party."] },
          ],
        },
        {
          title: "6. Cookies & tracking",
          body: [
            { kind: "p", parts: ["We use an HTTP-only cookie named ", { code: "hone_web_session" }, " to maintain your sign-in state. Its lifetime is 30 days when you check \"Keep me signed in,\" otherwise 1 day."] },
            { kind: "p", parts: ["We do not use third-party advertising tracking cookies."] },
          ],
        },
        {
          title: "7. Minors",
          body: [
            { kind: "p", parts: ["The service is intended for adults aged 18 or older with full legal capacity. If you are a minor, please use the service under a guardian's supervision. We do not actively collect personal information from minors."] },
          ],
        },
        {
          title: "8. Cross-border transfers",
          body: [
            { kind: "p", parts: ["If the language model or data source servers we call are located outside mainland China, the related query content may be transmitted overseas. We will choose partners with appropriate compliance credentials and apply necessary security measures."] },
          ],
        },
        {
          title: "9. Your rights",
          body: [
            { kind: "p", parts: ["You have the following rights regarding your personal information:"] },
            {
              kind: "ul",
              items: [
                ["access and correct your account details;"],
                ["change your sign-in password;"],
                ["request deletion of your account and associated data;"],
                ["withdraw a consent you previously granted."],
              ],
            },
            { kind: "p", parts: ["You can exercise the first three rights on the \"Account\" page, or contact us via the channel below. Withdrawing consent may render parts of the service unavailable."] },
          ],
        },
        {
          title: "10. Policy updates",
          body: [
            { kind: "p", parts: ["We may update this policy to reflect legal or business changes. Updated policies will be published in-service with a version number and effective date; material changes will be surfaced via in-service notice."] },
          ],
        },
        {
          title: "11. Contact",
          body: [
            { kind: "p", parts: ["If you have questions, comments, or complaints about this policy or how your data is handled, please open an issue on the project's GitHub repository:"] },
            { kind: "p", parts: [{ code: "https://github.com/B-M-Capital-Research/honeclaw/issues" }] },
            { kind: "p", parts: ["We will respond and address them within a reasonable period."] },
          ],
        },
      ] as LegalSection[],
    },
  },

  footer: {
    tagline: "Sharpen cognition, strip the noise.",
    mantra: "SHARPEN COGNITION · STRIP THE NOISE · OPEN FINANCIAL CONSOLE",
    copyright: "© 2026 Hangzhou Bamang Technology Co., Ltd. Open source, MIT License.",
    columns: {
      product: {
        title: "Product",
        items: [
          { label: "Home", href: "/" },
          { label: "Roadmap", href: "/roadmap" },
          { label: "Chat", href: "/chat" },
          { label: "Account", href: "/me" },
          { label: "Terms of Service", href: "/terms" },
          { label: "Privacy Policy", href: "/privacy" },
        ],
      },
      resources: {
        title: "Resources",
        items: [
          { label: "GitHub", href: "https://github.com/B-M-Capital-Research/honeclaw" },
          { label: "Chinese docs", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md" },
          { label: "Install", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/README_ZH.md#安装与启动" },
          { label: "Architecture", href: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md" },
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
    },
  },
}

const SOURCES = { zh: CONTENT_ZH, en: CONTENT_EN } as const

function resolveAt(path: readonly (string | symbol)[]): any {
  let v: any = SOURCES[useLocale()]
  for (const seg of path) {
    if (v == null) return undefined
    v = v[seg as any]
  }
  return v
}

function makeProxy(path: readonly (string | symbol)[]): any {
  return new Proxy(Object.create(null), {
    get(_target, key) {
      if (typeof key === "symbol") {
        // Let Solid / JS introspection (toPrimitive, iterator, etc.) pass through
        // to the resolved value if it is an object.
        const resolved = resolveAt(path)
        return resolved == null ? undefined : (resolved as any)[key]
      }
      const next = resolveAt([...path, key])
      if (next === null || next === undefined) return next
      if (typeof next !== "object") return next
      if (Array.isArray(next)) return next
      return makeProxy([...path, key])
    },
    has(_target, key) {
      const v = resolveAt(path)
      return v != null && typeof v === "object" && key in (v as object)
    },
    ownKeys() {
      const v = resolveAt(path)
      if (v == null || typeof v !== "object") return []
      return Reflect.ownKeys(v as object)
    },
    getOwnPropertyDescriptor(_target, key) {
      const v = resolveAt(path)
      if (v == null || typeof v !== "object") return undefined
      if (!(key in (v as object))) return undefined
      return { enumerable: true, configurable: true, writable: false, value: (v as any)[key] }
    },
  })
}

export const CONTENT = makeProxy([]) as typeof CONTENT_ZH
