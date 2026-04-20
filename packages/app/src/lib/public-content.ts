// public-content.ts — Hone Public Site Content Configuration
// All copy / data lives here for easy i18n in future.

export const CONTENT = {
  nav: {
    logo_tagline: "OPEN FINANCIAL CONSOLE",
    home: "首页",
    roadmap: "路线图与文档",
    me: "个人",
    chat: "对话",
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
        image: null,
      },
      {
        tag: "定时任务",
        title: "每周五自动触发复盘 Skill",
        body: "用 Cron 任务把固定工作流交给 Hone 自动完成。每周复盘、月度总结——无需手动触发。",
        image: null,
      },
      {
        tag: "长期画像",
        title: "建立公司专属研究档案",
        body: "每次研究结果自动归档到公司画像，下次提问直接调用历史上下文，越用越聪明。",
        image: "/hone_solution.jpg",
      },
      {
        tag: "跨平台通知",
        title: "在 iMessage / Lark 收到 Hone",
        body: "不只是网页。Hone 通过 iMessage、Lark、Discord 等渠道主动联系你，在你最顺手的地方工作。",
        image: "/hone_channels.jpg",
      },
    ],
  },

  video: {
    section_label: "看 HONE 如何工作",
    title: "老王讲 Hone：投研 AI Agent 的实际用法",
    description:
      "从开户到深度研究，10 分钟了解 Hone 如何改变你的投研工作流。完整演示个股分析、持仓追踪、定时任务等核心场景。",
    // Replace with actual video embed URL (YouTube / Bilibili)
    video_url: "",
    thumbnail: "/hone_introduction.jpg",
    duration: "约 10 分钟",
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
      { title: "中文文档", desc: "README、使用说明、案例示范", url: "#", tag: "文档", icon: "◈" },
      { title: "安装方式", desc: "macOS 桌面端 + 服务端自部署指南", url: "#", tag: "安装", icon: "⚡" },
      { title: "架构图", desc: "系统模块结构与技术架构说明", url: "#", tag: "技术", icon: "∞" },
      { title: "案例集", desc: "真实投研场景使用示例", url: "#", tag: "案例", icon: "✦" },
      { title: "贡献指南", desc: "参与开发、提交 PR、讨论功能方向", url: "#", tag: "贡献", icon: "ℹ" },
    ],
  },

  roadmap: {
    hero_title: "路线图与文档",
    hero_sub: "透明、务实、长期主义。下面是 Hone 目前能做什么、接下来做什么、以及如何接入你的投研工作流。",
    version: "v0.1.32",

    install: {
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
        title: "AGENTS.md",
        url: "https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md",
        desc: "Agent / Runner 架构与运行时约束",
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
    logged_out_title: "请先登录",
    logged_out_desc: "登录后查看你的使用额度、历史记录和账号信息。",
    logged_out_cta: "前往对话页登录",
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
          { label: "中文文档", href: "#" },
          { label: "安装方式", href: "#" },
          { label: "架构图", href: "#" },
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
} as const
