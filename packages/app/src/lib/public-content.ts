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
    placeholder_suffix: "场景演示截图（占位）",
    items: [
      {
        tag: "个股分析",
        title: "系统性深度研究一家公司",
        body: "从财务数据到行业竞争格局，Hone 帮你构建完整研究框架，记录每一个关键假设和风险因子。",
        image: "/company_profile.png",
      },
      {
        tag: "持仓追踪",
        title: "追踪持仓，主动提醒关键节点",
        body: "设置止盈止损逻辑，Hone 定时检查持仓状态，在你设定的条件触发时主动推送提醒。",
        image: null as string | null,
      },
      {
        tag: "定时任务",
        title: "每周五自动触发复盘 Skill",
        body: "用 Cron 任务把固定工作流交给 Hone 自动完成。每周复盘、月度总结——无需手动触发。",
        image: null as string | null,
      },
      {
        tag: "长期画像",
        title: "建立公司专属研究档案",
        body: "每次研究结果自动归档到公司画像，下次提问直接调用历史上下文，越用越聪明。",
        image: "/hone_solution.jpg" as string | null,
      },
      {
        tag: "跨平台通知",
        title: "在 iMessage / Lark 收到 Hone",
        body: "不只是网页。Hone 通过 iMessage、Lark、Discord 等渠道主动联系你，在你最顺手的地方工作。",
        image: "/hone_channels.jpg" as string | null,
      },
    ],
  },

  video: {
    section_label: "看 HONE 如何工作",
    title: "老王讲 Hone：投研 AI Agent 的实际用法",
    description:
      "从开户到深度研究，10 分钟了解 Hone 如何改变你的投研工作流。完整演示个股分析、持仓追踪、定时任务等核心场景。",
    video_url: "",
    thumbnail: "/hone_introduction.jpg",
    duration: "约 10 分钟",
    coverage: "视频涵盖：个股深度研究、持仓追踪、定时任务、多端接入演示",
    url_placeholder: "视频链接待配置（替换 video_url）",
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
    version: "v0.1.41",

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
        intro_suffix: "目录下的 18 个公开 Skill。",
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
          { name: "多 Runner 抽象", status: "stable", note: "OpenAI · Gemini CLI/ACP · Codex CLI/ACP" },
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
      { name: "hone_admin", desc: "查看修改 Hone 源码与配置（管理员）" },
    ],

    now: {
      label: "当前已有",
      items: [
        "Web 聊天界面（邀请制）+ 公开门面站",
        "Tauri macOS 桌面端 + 内置后端",
        "7 个渠道：Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "18 个内置 Skill（个股、持仓、估值、图表、PDF、Cron…）",
        "投研纪律约束 & 零幻觉协议",
        "公司画像与跨会话长期记忆",
        "Cron 定时任务系统",
        "多 Runner 抽象：OpenAI / Gemini CLI/ACP / Codex CLI/ACP / OpenCode ACP",
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
        "全部 18 个公开 Skill",
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
        a: "通过 Runner 抽象层支持：OpenAI 兼容协议（含 OpenRouter）、Gemini CLI / ACP、Codex CLI / ACP、OpenCode ACP。可以在桌面端设置里随时切换。",
      },
      {
        q: "开源协议？能商用吗？",
        a: "MIT 协议，可商用。开源仓库包含完整可运行的核心引擎、UI、桌面端、全部 18 个公开 Skill 和 7 个渠道集成。私域高级 Skill 与付费数据源接入不在仓库中，不影响主流程。",
      },
      {
        q: "数据存在哪里？",
        a: "所有会话、公司画像、研究结果默认存储在本地（macOS 桌面端用户目录 ~/.honeclaw 或自部署服务器）。Hone 官方不托管用户数据。",
      },
      {
        q: "和 Codex / RooCode 等 coding agent 的关系？",
        a: "Hone 借鉴了这些产品的 runner / skill / session 架构，但专注投研而非写代码。Codex CLI / ACP、Gemini CLI / ACP、OpenCode ACP 在 Hone 中作为可插拔 Runner 存在。",
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

  footer: {
    tagline: "磨砺认知，剔除噪音",
    mantra: "磨砺认知 · 剔除噪音 · OPEN FINANCIAL CONSOLE",
    copyright: "© 2025 B&M Capital Research. Open source, MIT License.",
    columns: {
      product: {
        title: "产品",
        items: [
          { label: "首页", href: "/" },
          { label: "路线图", href: "/roadmap" },
          { label: "对话", href: "/chat" },
          { label: "个人", href: "/me" },
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
        image: null,
      },
      {
        tag: "Scheduled tasks",
        title: "Trigger a weekly review skill every Friday",
        body: "Hand fixed workflows to Hone via cron. Weekly reviews, monthly summaries — all run themselves.",
        image: null,
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
    video_url: "",
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
    version: "v0.1.41",

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
        intro_prefix: "Hone's skills are invoked by the model from context. Below are the 18 public skills in the",
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
          { name: "Multi-runner abstraction", status: "stable", note: "OpenAI · Gemini CLI/ACP · Codex CLI/ACP" },
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
      { name: "hone_admin", desc: "Inspect and modify Hone source & config (admin)" },
    ],

    now: {
      label: "Shipping today",
      items: [
        "Web chat (invite-only) + public landing site",
        "Tauri macOS desktop with bundled backend",
        "7 channels: Web / iMessage / Lark / Discord / Telegram / CLI / MCP",
        "18 built-in skills (stocks, portfolio, valuation, charts, PDF, cron…)",
        "Research discipline & zero-hallucination protocol",
        "Company profiles + cross-session long memory",
        "Cron-driven scheduled tasks",
        "Multi-runner: OpenAI / Gemini CLI/ACP / Codex CLI/ACP / OpenCode ACP",
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
        "All 18 public skills",
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
        a: "Through the runner abstraction: OpenAI-compatible protocols (including OpenRouter), Gemini CLI / ACP, Codex CLI / ACP, and OpenCode ACP. Switch at any time from the desktop settings.",
      },
      {
        q: "What license? Commercial use?",
        a: "MIT, commercial use allowed. The repo ships a fully-working core engine, UI, desktop, all 18 public skills, and 7 channel integrations. Private premium skills and paid data sources live outside the repo and don't block the main flow.",
      },
      {
        q: "Where is data stored?",
        a: "Sessions, company profiles, and research results default to local storage (macOS desktop's `~/.honeclaw` or your self-hosted server). Hone does not host user data.",
      },
      {
        q: "How does Hone relate to Codex / RooCode and other coding agents?",
        a: "Hone borrows their runner / skill / session architecture but targets investment research, not coding. Codex CLI / ACP, Gemini CLI / ACP, and OpenCode ACP show up inside Hone as pluggable runners.",
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

  footer: {
    tagline: "Sharpen cognition, strip the noise.",
    mantra: "SHARPEN COGNITION · STRIP THE NOISE · OPEN FINANCIAL CONSOLE",
    copyright: "© 2025 B&M Capital Research. Open source, MIT License.",
    columns: {
      product: {
        title: "Product",
        items: [
          { label: "Home", href: "/" },
          { label: "Roadmap", href: "/roadmap" },
          { label: "Chat", href: "/chat" },
          { label: "Account", href: "/me" },
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
