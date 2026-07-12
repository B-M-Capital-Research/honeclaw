export const holdings = [
  { name: "NVIDIA", ticker: "NVDA", weight: "28.1%", value: "¥360,982", today: "+2.6%", todayAmount: "+¥9,832", ytd: "+18.7%", ytdAmount: "+¥56,882", cost: "$102.43", price: "$126.78", quantity: "402.6", thesis: "AI 计算平台与 CUDA 生态", spark: [18,20,19,23,22,26,25,29,31,35] },
  { name: "AMD", ticker: "AMD", weight: "14.8%", value: "¥190,122", today: "+2.1%", todayAmount: "+¥3,882", ytd: "+10.3%", ytdAmount: "+¥17,712", cost: "$154.32", price: "$176.54", quantity: "153.8", thesis: "推理侧份额与开放生态", spark: [22,25,21,19,23,22,26,25,28,31] },
  { name: "台积电", ticker: "TSM", weight: "13.2%", value: "¥169,498", today: "+1.9%", todayAmount: "+¥3,192", ytd: "+15.4%", ytdAmount: "+¥22,662", cost: "NT$680.50", price: "NT$812.00", quantity: "922", thesis: "先进制程与封装稀缺性", spark: [16,18,19,22,21,24,25,26,29,30] },
  { name: "Microsoft", ticker: "MSFT", weight: "11.6%", value: "¥148,948", today: "+1.2%", todayAmount: "+¥1,802", ytd: "+7.8%", ytdAmount: "+¥10,764", cost: "$378.64", price: "$415.21", quantity: "51.2", thesis: "云与 AI 应用渗透", spark: [20,19,21,22,21,23,25,24,26,27] },
  { name: "ASML Holding", ticker: "ASML", weight: "6.7%", value: "¥86,019", today: "+0.8%", todayAmount: "+¥688", ytd: "+12.1%", ytdAmount: "+¥9,251", cost: "€627.50", price: "€692.10", quantity: "17.8", thesis: "先进光刻不可替代性", spark: [17,18,16,20,22,21,24,22,25,24] },
];

export const performanceData = [
  { month: "01", portfolio: 0, benchmark: 0 },
  { month: "01", portfolio: 2, benchmark: 1 },
  { month: "02", portfolio: 5, benchmark: 3 },
  { month: "02", portfolio: 8, benchmark: 5 },
  { month: "03", portfolio: 7, benchmark: 2 },
  { month: "03", portfolio: 9, benchmark: 4 },
  { month: "04", portfolio: 5, benchmark: -8 },
  { month: "04", portfolio: 3, benchmark: -3 },
  { month: "05", portfolio: 8, benchmark: 2 },
  { month: "05", portfolio: 11, benchmark: 4 },
  { month: "06", portfolio: 14, benchmark: 7 },
  { month: "06", portfolio: 16, benchmark: 8 },
  { month: "07", portfolio: 19, benchmark: 10 },
  { month: "07", portfolio: 21, benchmark: 9 },
];

export const companyChart = [
  { day: "06-16", price: 118 }, { day: "06-19", price: 120 }, { day: "06-22", price: 117 },
  { day: "06-25", price: 121 }, { day: "06-28", price: 122 }, { day: "07-01", price: 119 },
  { day: "07-04", price: 123 }, { day: "07-07", price: 125 }, { day: "07-10", price: 124 },
  { day: "07-12", price: 126.78 },
];

export const companyCandles = [
  { time: "2026-06-01", open: 112.8, high: 115.2, low: 111.9, close: 114.4, volume: 32 },
  { time: "2026-06-02", open: 114.4, high: 116.1, low: 113.7, close: 115.5, volume: 28 },
  { time: "2026-06-03", open: 115.6, high: 116.0, low: 113.4, close: 114.1, volume: 31 },
  { time: "2026-06-04", open: 114.2, high: 117.0, low: 113.8, close: 116.4, volume: 37 },
  { time: "2026-06-05", open: 116.3, high: 117.4, low: 114.9, close: 115.2, volume: 34 },
  { time: "2026-06-08", open: 115.0, high: 116.8, low: 114.2, close: 116.0, volume: 27 },
  { time: "2026-06-09", open: 116.1, high: 118.3, low: 115.8, close: 117.7, volume: 41 },
  { time: "2026-06-10", open: 117.8, high: 119.2, low: 116.9, close: 118.4, volume: 39 },
  { time: "2026-06-11", open: 118.2, high: 119.0, low: 116.7, close: 117.1, volume: 36 },
  { time: "2026-06-12", open: 117.2, high: 120.1, low: 116.9, close: 119.5, volume: 44 },
  { time: "2026-06-15", open: 119.4, high: 121.0, low: 118.2, close: 120.2, volume: 48 },
  { time: "2026-06-16", open: 120.1, high: 121.4, low: 118.9, close: 119.3, volume: 33 },
  { time: "2026-06-17", open: 119.1, high: 120.4, low: 117.8, close: 118.5, volume: 35 },
  { time: "2026-06-18", open: 118.6, high: 121.2, low: 118.1, close: 120.8, volume: 46 },
  { time: "2026-06-19", open: 120.7, high: 122.5, low: 120.0, close: 121.9, volume: 52 },
  { time: "2026-06-22", open: 122.0, high: 123.1, low: 120.8, close: 121.2, volume: 38 },
  { time: "2026-06-23", open: 121.3, high: 123.8, low: 120.9, close: 123.0, volume: 47 },
  { time: "2026-06-24", open: 123.1, high: 124.4, low: 122.0, close: 123.8, volume: 43 },
  { time: "2026-06-25", open: 123.7, high: 124.0, low: 121.7, close: 122.4, volume: 36 },
  { time: "2026-06-26", open: 122.5, high: 125.1, low: 122.1, close: 124.7, volume: 49 },
  { time: "2026-06-29", open: 124.8, high: 126.0, low: 123.9, close: 125.4, volume: 45 },
  { time: "2026-06-30", open: 125.3, high: 126.7, low: 124.2, close: 124.9, volume: 42 },
  { time: "2026-07-01", open: 124.8, high: 127.2, low: 124.4, close: 126.6, volume: 51 },
  { time: "2026-07-02", open: 126.7, high: 127.5, low: 125.1, close: 125.8, volume: 40 },
  { time: "2026-07-06", open: 125.9, high: 128.0, low: 125.5, close: 127.4, volume: 54 },
  { time: "2026-07-07", open: 127.3, high: 128.4, low: 126.0, close: 126.5, volume: 37 },
  { time: "2026-07-08", open: 126.6, high: 127.8, low: 125.4, close: 127.1, volume: 39 },
  { time: "2026-07-09", open: 127.2, high: 128.2, low: 126.1, close: 126.9, volume: 35 },
  { time: "2026-07-10", open: 126.8, high: 128.0, low: 125.9, close: 126.3, volume: 41 },
  { time: "2026-07-12", open: 126.4, high: 127.6, low: 125.8, close: 126.78, volume: 44 },
];

export const insights = [
  {
    slug: "hbm-moat",
    type: "深度解读",
    title: "从 HBM 竞赛看 GPU 代际后的护城河",
    excerpt: "HBM4 进入量产窗口，显存带宽与堆叠能力成为下一代 GPU 性能跃迁的关键变量。真正的优势不再只体现为芯片设计，而在于与存储生态的深度绑定。",
    author: "老王",
    time: "2026-07-12 08:30",
    tags: ["NVIDIA", "AMD", "产业链"],
    reading: "8 分钟",
  },
  {
    slug: "tsmc-packaging",
    type: "短观点",
    title: "先进封装的扩产速度，正在成为 AI 芯片交付的真实上限",
    excerpt: "只看 GPU 设计会低估供给约束。CoWoS 与 HBM 的协同扩产，决定了未来四个季度的兑现节奏。",
    author: "老王",
    time: "2026-07-11 21:15",
    tags: ["台积电", "先进封装"],
    reading: "2 分钟",
  },
  {
    slug: "amd-inference",
    type: "短观点",
    title: "AMD 的机会不在复制 NVIDIA，而在推理侧建立第二选择",
    excerpt: "MI350 的意义，是让客户在成本敏感的推理集群中拥有更强的议价空间。软件成熟度仍是核心观察点。",
    author: "老王",
    time: "2026-07-10 18:40",
    tags: ["AMD", "AI 推理"],
    reading: "3 分钟",
  },
];

export const trackingEvents = [
  { id: 1, date: "07月12日", time: "16:00", title: "本周持仓复盘", meta: "Agent 周期任务", state: "已完成", kind: "任务" },
  { id: 2, date: "07月17日", time: "08:00", title: "TSMC 2026 Q2 财报", meta: "台北时间 · 持仓相关", state: "即将发生", kind: "财报" },
  { id: 3, date: "07月23日", time: "全天", title: "Microsoft Build 大会", meta: "07-23 至 07-25", state: "即将发生", kind: "活动" },
  { id: 4, date: "07月24日", time: "09:00", title: "NVIDIA GTC Tokyo", meta: "关注 Blackwell 与网络路线", state: "即将发生", kind: "活动" },
  { id: 5, date: "08月05日", time: "08:00", title: "AMD 2026 Q2 财报", meta: "美东时间 · 自选相关", state: "即将发生", kind: "财报" },
];

export const agentSources = [
  { title: "老王洞察：AI 基础设施下半年展望", time: "2026-07-11 22:15" },
  { title: "NVIDIA Investor Relations · Q2 FY2026 Update", time: "2026-07-12 08:30" },
  { title: "AMD Press Release · MI350 Series Update", time: "2026-07-12 09:05" },
  { title: "TSMC · June 2026 Revenue Report", time: "2026-07-12 10:00" },
];
