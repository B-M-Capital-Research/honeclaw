// HONE Adventure V2 — Cleanroom AI Data Center Stages, Hardware Weapons, Bosses & Promotion System

export const VERSION = "2.0.0";
export const HONECLAW_GITHUB_URL = "https://github.com/B-M-Capital-Research/honeclaw";
export const HONECLAW_REPO_LABEL = "github.com/B-M-Capital-Research/honeclaw";

export const MAX_WEAPON_LV = 5;
export const DEFAULT_W_SLOTS = 4;
export const DEFAULT_P_SLOTS = 4;
export const WAVE_SEC = 20;
export const STAGE_SEC = 75;
export const BOSS_SEC = 90;

// 6 Selectable & Progressive AI Data Center Sector Stages
export const STAGES = [
  {
    id: "storage",
    code: "SECTOR-01",
    short: "存储阵列",
    name: "NAND & HBM 存储阵列",
    sub: "高并发闪存走廊 · 晶核掉落率 +15%",
    accent: "#2563eb",
    badgeBg: "#dbeafe",
    floorTint: [0.95, 0.97, 1.0],
    gridCol: 0x3b82f6,
    bossId: "boss1",
    mod: { coreBonus: 0.15 },
    bias: [0, 5, 2],
    propMode: 0,
  },
  {
    id: "optics",
    code: "SECTOR-02",
    short: "光模块",
    name: "800G / 1.6T 光模块走廊",
    sub: "硅光低时延互连 · 弹道飞行速度 +18%",
    accent: "#0891b2",
    badgeBg: "#cffafe",
    floorTint: [0.93, 0.98, 1.0],
    gridCol: 0x06b6d4,
    bossId: "boss2",
    mod: { projSpeed: 0.18 },
    bias: [3, 1, 7],
    propMode: 1,
  },
  {
    id: "cpu",
    code: "SECTOR-03",
    short: "CPU 中枢",
    name: "乱序指令集 CPU 中枢",
    sub: "高频时钟流水线 · 武器冷却缩减 +8%",
    accent: "#4f46e5",
    badgeBg: "#e0e7ff",
    floorTint: [0.95, 0.96, 1.0],
    gridCol: 0x6366f1,
    bossId: "boss3",
    mod: { cdMul: -0.08 },
    bias: [1, 4, 0],
    propMode: 0,
  },
  {
    id: "gpu",
    code: "SECTOR-04",
    short: "GPU 矩阵",
    name: "张量加速 GPU 算力矩阵",
    sub: "脉动阵列满载超频 · 暴击伤害 +22%",
    accent: "#059669",
    badgeBg: "#d1fae5",
    floorTint: [0.93, 0.99, 0.96],
    gridCol: 0x10b981,
    bossId: "boss4",
    mod: { critDmg: 0.22 },
    bias: [2, 5, 6],
    propMode: 2,
  },
  {
    id: "packaging",
    code: "SECTOR-05",
    short: "先进封装",
    name: "CoWoS & EUV 先进封装工坊",
    sub: "黄光无尘洁净室 · 算力经验获取 +16%",
    accent: "#d97706",
    badgeBg: "#fef3c7",
    floorTint: [1.0, 0.98, 0.94],
    gridCol: 0xf59e0b,
    bossId: "boss5",
    mod: { xpMul: 0.16 },
    bias: [6, 7, 3],
    propMode: 3,
  },
  {
    id: "hyperscale",
    code: "SECTOR-06",
    short: "超算总机房",
    name: "GW 级液冷超算总机房",
    sub: "全栈智算终极试炼 · 全局伤害 +12% · 五大巨兽轮战",
    accent: "#7c3aed",
    badgeBg: "#ede9fe",
    floorTint: [0.95, 0.96, 1.0],
    gridCol: 0x8b5cf6,
    bossId: "boss4",
    mod: { might: 0.12 },
    bias: [4, 5, 6, 3],
    propMode: 2,
  },
];

export const STAGE_IDX = {};
STAGES.forEach((s, i) => {
  STAGE_IDX[s.id] = i;
  s.k = i;
});

// 8 Redesigned AI Hardware & Photonics Weapons + 8 Super Evolutions
export const WEAPONS = [
  {
    id: "hbm",
    name: "HBM 带宽飞梭",
    evoName: "3D 堆叠数据洪流",
    partner: "clock",
    color: "#2563eb",
    desc: "并列齐射多通道矩形 HBM 数据帧，高速贯穿前方故障体",
    evoDesc: "四通道超宽总线连发，命中终点迸发十字总线脉冲二次清场",
    up: ["", "并发通道 +1", "伤害 +7，穿透 +1", "并发通道 +1", "伤害 +9，发射周期缩短"],
    lv: [
      { dmg: 12, cd: 0.76, n: 2, pierce: 0 },
      { dmg: 12, cd: 0.74, n: 3, pierce: 0 },
      { dmg: 19, cd: 0.72, n: 3, pierce: 1 },
      { dmg: 19, cd: 0.68, n: 4, pierce: 1 },
      { dmg: 28, cd: 0.54, n: 4, pierce: 2 },
    ],
    evo: { dmg: 24, cd: 0.14, n: 4, pierce: 2, pop: 42 },
  },
  {
    id: "prism",
    name: "硅光分光束",
    evoName: "1.6T 波分相干阵列",
    partner: "waveguide",
    color: "#0891b2",
    desc: "发射相干硅光棱枪，命中首个目标后折射分裂为三波长子光束",
    evoDesc: "双路主光枪高速齐射，每次命中折射出 5 道全彩波分激光扇面",
    up: ["", "折射子光束伤害提升", "主光束 +1", "伤害 +9，射程更远", "折射分裂增至 4 道"],
    lv: [
      { dmg: 15, cd: 1.18, n: 1, split: 3 },
      { dmg: 20, cd: 1.14, n: 1, split: 3 },
      { dmg: 20, cd: 1.12, n: 2, split: 3 },
      { dmg: 29, cd: 1.05, n: 2, split: 3 },
      { dmg: 34, cd: 0.92, n: 2, split: 4 },
    ],
    evo: { dmg: 38, cd: 0.56, n: 2, split: 5 },
  },
  {
    id: "tensor",
    name: "张量矩阵炮",
    evoName: "Rubin 脉动方阵",
    partner: "fp8",
    color: "#059669",
    desc: "锁定密集敌群投射 3×3 方形脉动矩阵印章，对齐后瞬时网格爆破",
    evoDesc: "高频轰击巨型双层张量方阵，主方阵引爆后向四角连锁展开子矩阵",
    up: ["", "伤害 +12", "矩阵数量 +1，格网扩大", "伤害 +15", "矩阵数量 +1，装填更快"],
    lv: [
      { dmg: 30, cd: 2.05, n: 1, rad: 62 },
      { dmg: 42, cd: 2.0, n: 1, rad: 62 },
      { dmg: 42, cd: 1.95, n: 2, rad: 72 },
      { dmg: 57, cd: 1.9, n: 2, rad: 72 },
      { dmg: 57, cd: 1.55, n: 3, rad: 80 },
    ],
    evo: { dmg: 76, cd: 1.32, n: 3, rad: 96, mini: 4, miniR: 52 },
  },
  {
    id: "cache",
    name: "L3 缓存环刃",
    evoName: "零延迟一致性圣环",
    partner: "ecc",
    color: "#4f46e5",
    desc: "六角硅晶裸片沿高速缓存总线环绕周身，切割并击退近身异常",
    evoDesc: "内外双层晶圆裸片反向高速运转，周期性释放全环缓存刷新震波",
    up: ["", "晶圆片 +1", "伤害 +7，总线半径扩大", "晶圆片 +1", "伤害 +7，常驻无间断运转"],
    lv: [
      { dmg: 11, n: 2, rad: 60, on: 4.4, off: 2.0 },
      { dmg: 11, n: 3, rad: 60, on: 4.4, off: 1.8 },
      { dmg: 18, n: 3, rad: 70, on: 4.8, off: 1.6 },
      { dmg: 18, n: 4, rad: 70, on: 5.4, off: 1.2 },
      { dmg: 25, n: 4, rad: 76, on: 1, off: 0 },
    ],
    evo: { dmg: 31, n: 5, n2: 7, rad: 58, rad2: 114, wave: 2.6, waveDmg: 34, waveR: 168 },
  },
  {
    id: "bus",
    name: "PCB 母线电弧",
    evoName: "吉瓦级超导馈线",
    partner: "overvolt",
    color: "#d97706",
    desc: "沿 45°/90° 电路板布线规则在故障体间高速跳接高压母线电弧",
    evoDesc: "双路超导金铜母线齐发，连续跳接 7 次并在首焊盘触发电容过载震爆",
    up: ["", "跳接节点 +1", "伤害 +10", "母线 +1，跳接 +2", "伤害 +12，放电更快"],
    lv: [
      { dmg: 18, cd: 1.32, n: 1, chain: 2 },
      { dmg: 18, cd: 1.32, n: 1, chain: 3 },
      { dmg: 28, cd: 1.28, n: 1, chain: 3 },
      { dmg: 28, cd: 1.22, n: 2, chain: 5 },
      { dmg: 40, cd: 0.98, n: 2, chain: 5 },
    ],
    evo: { dmg: 48, cd: 0.38, n: 2, chain: 7, boom: 48 },
  },
  {
    id: "litho",
    name: "EUV 光刻扫描",
    evoName: "2nm 极紫外全域曝光",
    partner: "reticle",
    color: "#7c3aed",
    desc: "发射极紫外横幅光刻扫描梁横掠洁净室，对整排目标执行晶圆曝光打击",
    evoDesc: "正交十字极紫外光刻梁绕场持续旋转扫描，全屏无死角蚀刻净化",
    up: ["", "伤害 +4", "曝光梁更宽更远", "扫描驻留更久", "伤害 +5，双向对穿扫描"],
    lv: [
      { dmg: 9, cd: 2.25, len: 250, wid: 20, dur: 0.56, n: 1 },
      { dmg: 13, cd: 2.25, len: 250, wid: 20, dur: 0.56, n: 1 },
      { dmg: 13, cd: 2.2, len: 300, wid: 28, dur: 0.56, n: 1 },
      { dmg: 13, cd: 2.15, len: 300, wid: 28, dur: 0.92, n: 1 },
      { dmg: 18, cd: 2.05, len: 315, wid: 28, dur: 0.92, n: 2 },
    ],
    evo: { dmg: 20, len: 295, wid: 30, n: 4, spin: 1.65 },
  },
  {
    id: "drone",
    name: "ASIC 推理蜂群",
    evoName: "自主智能体编队",
    partner: "fabric",
    color: "#0284c7",
    desc: "放飞三角翼专用推理无人机，自动巡航穿插敌群并折返切割",
    evoDesc: "5 架常驻金蓝智能体僚机环绕巡航高速掠袭，永不返航停歇",
    up: ["", "无人机 +1", "伤害 +8，巡航半径扩大", "无人机 +1", "伤害 +10，机体扩容"],
    lv: [
      { dmg: 16, cd: 1.48, n: 1, range: 185, size: 1 },
      { dmg: 16, cd: 1.48, n: 2, range: 185, size: 1 },
      { dmg: 24, cd: 1.45, n: 2, range: 225, size: 1.08 },
      { dmg: 24, cd: 1.38, n: 3, range: 225, size: 1.14 },
      { dmg: 34, cd: 1.28, n: 3, range: 245, size: 1.35 },
    ],
    evo: { dmg: 40, n: 5, size: 1.55, rMin: 54, rMax: 198 },
  },
  {
    id: "cryo",
    name: "浸没液冷喷射",
    evoName: "绝对零度超导池",
    partner: "manifold",
    color: "#0d9488",
    desc: "在怪群中心铺设六角氟化液冷板，持续冻结减速并剥离过热外壳",
    evoDesc: "展开巨型超导零度力场强效聚拢减速，坍缩瞬间触发晶格脆裂重创",
    up: ["", "伤害 +3", "液冷池更大更久", "液冷池 +1", "伤害 +5，铺设更密"],
    lv: [
      { dmg: 7, cd: 3.7, n: 1, rad: 56, dur: 3.1 },
      { dmg: 10, cd: 3.7, n: 1, rad: 56, dur: 3.1 },
      { dmg: 10, cd: 3.6, n: 1, rad: 68, dur: 4.0 },
      { dmg: 10, cd: 3.5, n: 2, rad: 68, dur: 4.0 },
      { dmg: 15, cd: 2.85, n: 2, rad: 72, dur: 4.0 },
    ],
    evo: { dmg: 22, cd: 5.4, n: 1, rad: 142, dur: 5.0, burst: 112 },
  },
];

export const WEAPON_IDX = {};
WEAPONS.forEach((w, i) => {
  WEAPON_IDX[w.id] = i;
  w.k = i;
});

export const PASSIVES = [
  { id: "clock", name: "超频时钟", max: 3, desc: "武器冷却 −10%", color: "#2563eb" },
  { id: "waveguide", name: "波导扩束", max: 3, desc: "作用范围 +15%", color: "#0891b2" },
  { id: "ecc", name: "ECC 纠错甲", max: 3, desc: "生命 +30，护甲 +1", color: "#4f46e5" },
  { id: "fp8", name: "FP8 锐化", max: 3, desc: "暴击率 +10%，暴伤 +20%", color: "#059669" },
  { id: "reticle", name: "掩膜校准", max: 3, desc: "拾取带宽 +42%，经验 +7%", color: "#7c3aed" },
  { id: "overvolt", name: "高压偏置", max: 3, desc: "全局伤害 +14%", color: "#d97706" },
  { id: "manifold", name: "液冷歧管", max: 3, desc: "持续 +18%，弹速 +14%", color: "#0d9488" },
  { id: "fabric", name: "拓扑路由", max: 3, desc: "移动速度 +10%", color: "#0284c7" },
  { id: "ups", name: "固态储能", max: 3, desc: "每秒修复 +0.8 生命", color: "#e11d48" },
];

export const PASSIVE_IDX = {};
PASSIVES.forEach((p, i) => {
  PASSIVE_IDX[p.id] = i;
});

export const BOOSTS = [
  { id: "b_might", name: "算力扩容包", desc: "全局伤害 +8%（可无限叠加）" },
  { id: "b_heal", name: "液冷应急换液", desc: "立即恢复 40% 最大生命" },
  { id: "b_core", name: "晶圆良率红利", desc: "算力晶核 +18" },
];

export const AI_TYPE = {
  CHASE: 0,
  DART: 1,
  SPIT: 2,
  DASH: 3,
  GHOST: 4,
  BOSS: 5,
};

export const ENEMIES = [
  { id: "packet", name: "丢包爬虫", hp: 8, sp: 50, dmg: 8, r: 10, xp: 1, wt: 1, ai: 0, from: 0, w: 10 },
  { id: "jitter", name: "时延飞蚊", hp: 5, sp: 95, dmg: 6, r: 8, xp: 1, wt: 0.8, ai: 1, from: 25, w: 5.2 },
  { id: "thermal", name: "积热熔岩体", hp: 42, sp: 35, dmg: 14, r: 16, xp: 5, wt: 3.5, ai: 0, from: 55, w: 2.4 },
  { id: "crosstalk", name: "串扰光炮", hp: 19, sp: 41, dmg: 8, r: 12, xp: 3, wt: 1.4, ai: 2, from: 95, w: 2.2 },
  { id: "surge", name: "浪涌电牛", hp: 24, sp: 52, dmg: 14, r: 12, xp: 3, wt: 1.5, ai: 3, from: 130, w: 2.5 },
  { id: "leak", name: "内存泄漏块", hp: 28, sp: 52, dmg: 10, r: 14, xp: 2, wt: 1.6, ai: 0, from: 165, w: 2.0 },
  { id: "short", name: "短路电容弹", hp: 22, sp: 60, dmg: 18, r: 12, xp: 3, wt: 1.2, ai: 0, from: 210, w: 1.7 },
  { id: "halluc", name: "位翻转幽影", hp: 13, sp: 74, dmg: 9, r: 11, xp: 2, wt: 1, ai: 4, from: 1e9, w: 0 },
  { id: "frag", name: "坏道碎晶", hp: 7, sp: 85, dmg: 7, r: 8, xp: 1, wt: 0.7, ai: 0, from: 1e9, w: 0, hide: 1 },
  { id: "boss1", name: "坏块吞噬者·闪存巨像", hp: 980, sp: 40, dmg: 18, r: 42, xp: 70, wt: 40, ai: 5, boss: 1, from: 1e9, w: 0 },
  { id: "boss2", name: "色散棱镜·硅光主宰", hp: 2900, sp: 45, dmg: 21, r: 44, xp: 100, wt: 40, ai: 5, boss: 2, from: 1e9, w: 0 },
  { id: "boss3", name: "死锁仲裁官·乱序巨核", hp: 6200, sp: 39, dmg: 24, r: 45, xp: 135, wt: 40, ai: 5, boss: 3, from: 1e9, w: 0 },
  { id: "boss4", name: "显存墙霸主·张量熔炉", hp: 10500, sp: 43, dmg: 27, r: 48, xp: 175, wt: 40, ai: 5, boss: 4, from: 1e9, w: 0 },
  { id: "boss5", name: "良率黑洞·光刻母神", hp: 15200, sp: 41, dmg: 30, r: 50, xp: 220, wt: 45, ai: 5, boss: 5, from: 1e9, w: 0 },
];

export const ENEMY_IDX = {};
ENEMIES.forEach((e, i) => {
  ENEMY_IDX[e.id] = i;
  e.k = i;
});

export const BOSS_IDS = ["boss1", "boss2", "boss3", "boss4", "boss5"];
export const CODEX_ENEMY_IDS = ["packet", "jitter", "thermal", "crosstalk", "surge", "leak", "short", "halluc"];

export const CURVES = {
  hpMul: (t) => {
    const m = t / 60;
    return (1 + 0.34 * m + 0.1 * m * m) * (m > 4 ? Math.pow(1.24, m - 4) : 1);
  },
  dmgMul: (t) => {
    const m = t / 60;
    return (1 + 0.09 * m) * (m > 4 ? Math.pow(1.09, m - 4) : 1);
  },
  spMul: (t) => Math.min(1.42, 1 + 0.03 * (t / 60)),
  target: (t) => Math.min(340, 11 + 0.92 * t),
  rate: (t) => 1.3 + t / 21,
  xpNeed: (lv) => Math.round(4 + 4.1 * (lv - 1) + 0.92 * Math.pow(lv - 1, 1.9)),
};

export const HEROES = [
  {
    id: "hone",
    name: "小砺",
    kind: "首席算力巡检官",
    weapon: "hbm",
    trait: "基准总线 · 算力经验 +14% · 均衡全能",
    mod: { xpMul: 0.14 },
    unlock: null,
    price: 0,
  },
  {
    id: "rubin",
    name: "芯核",
    kind: "液冷机柜重卫",
    weapon: "tensor",
    trait: "浸没重装 · 生命 +50 · 受击释放缓存清场波",
    mod: { maxHp: 50, speed: -0.03, thorns: 1 },
    unlock: { stat: "kills", n: 800, text: "累计清除 800 个硬件异常" },
    price: 280,
  },
  {
    id: "photon",
    name: "光瞳",
    kind: "硅光互连斥候",
    weapon: "prism",
    trait: "波分洞察 · 拾取带宽 +50% · 暴击率 +12%",
    mod: { pickup: 0.5, crit: 0.12, maxHp: -12 },
    unlock: { stat: "bosses", n: 1, text: "累计击破 1 台产线巨兽 Boss" },
    price: 380,
  },
  {
    id: "quant",
    name: "量闪",
    kind: "超频调度灵猫",
    weapon: "bus",
    trait: "纳秒抢占 · 移速 +15% · 武器冷却 −10%",
    mod: { speed: 0.15, cd: -0.1 },
    unlock: { stat: "bestTime", n: 240, text: "单局巡检坚守超过 4 分钟" },
    price: 520,
  },
];

export const HERO_IDX = {};
HEROES.forEach((h, i) => {
  HERO_IDX[h.id] = i;
});

// 4-Pillar × 3-Node = 12-Node AI Infrastructure Upgrade Matrix
export const TALENT_BRANCHES = [
  {
    id: "interconnect",
    name: "存算互联",
    sub: "投入 6 点激活「高带宽共鸣：终伤 +9%」",
    milestone: 6,
    bonusDesc: "终伤 +9%",
  },
  {
    id: "litho",
    name: "光刻制程",
    sub: "投入 6 点激活「极紫外共鸣：暴击率 +6%」",
    milestone: 6,
    bonusDesc: "暴击率 +6%",
  },
  {
    id: "thermal",
    name: "液冷供电",
    sub: "投入 6 点激活「超导散热共鸣：护甲 +2」",
    milestone: 6,
    bonusDesc: "护甲 +2",
  },
  {
    id: "cluster",
    name: "智能调度",
    sub: "投入 6 点激活「先发调度共鸣：开局秒升 Lv.2」",
    milestone: 6,
    bonusDesc: "开局秒升 Lv.2",
  },
];

export const TALENTS = [
  // Pillar 1: Interconnect
  { id: "might", branch: "interconnect", icon: "overvolt", name: "HBM3E 预取", max: 5, per: "全局伤害 +7%", cost: [55, 110, 200, 340, 520] },
  { id: "haste", branch: "interconnect", icon: "clock", name: "PCIe 6.0 通道", max: 4, per: "武器冷却 −3.5%", cost: [80, 160, 280, 450] },
  { id: "area", branch: "interconnect", icon: "waveguide", name: "NVLink 交换网", max: 4, per: "作用范围 +6%", cost: [70, 140, 250, 400] },
  // Pillar 2: Lithography
  { id: "crit", branch: "litho", icon: "fp8", name: "EUV 对准精度", max: 5, per: "暴击 +3% · 暴伤 +7%", cost: [65, 130, 230, 380, 560] },
  { id: "proj", branch: "litho", icon: "manifold", name: "GAA 环绕栅极", max: 4, per: "弹速与持续 +7%", cost: [70, 140, 250, 400] },
  { id: "reroll", branch: "litho", icon: "ui_retry", name: "晶圆良率重筛", max: 3, per: "每局重抽次数 +1（支持锁卡）", cost: [120, 240, 400] },
  // Pillar 3: Thermal & Power
  { id: "hp", branch: "thermal", icon: "ecc", name: "双相浸没液冷", max: 5, per: "最大生命 +16", cost: [55, 110, 200, 340, 520] },
  { id: "regen", branch: "thermal", icon: "ups", name: "SOFC 固态供电", max: 4, per: "每秒生命修复 +0.35", cost: [70, 140, 250, 400] },
  { id: "revive", branch: "thermal", icon: "ecc", name: "UPS 灾备冗余", max: 1, per: "致命故障时半血重启并清场 1 次", cost: [420] },
  // Pillar 4: Cluster Orchestration
  { id: "pickup", branch: "cluster", icon: "reticle", name: "光模块遥测", max: 5, per: "拾取 +15% · 经验 +5%", cost: [55, 110, 200, 340, 520] },
  { id: "speed", branch: "cluster", icon: "fabric", name: "RDMA 零拷贝", max: 5, per: "移动速度 +3.8%", cost: [55, 110, 200, 340, 520] },
  { id: "core", branch: "cluster", icon: "core", name: "强化学习挖矿", max: 5, per: "算力晶核产出 +13%", cost: [60, 120, 210, 360, 540] },
];

export function branchPoints(talents, branchId) {
  let sum = 0;
  for (const t of TALENTS) {
    if (t.branch === branchId) sum += talents[t.id] || 0;
  }
  return sum;
}

export const RULES = [
  { id: "surge", name: "峰值推理洪峰", desc: "故障密度 +40%，算力经验同步 +40%", countMul: 1.4, xpMul: 1.4 },
  { id: "overclock", name: "极限液氮超频", desc: "全局伤害翻倍，最大生命减半", might: 2, hpMul: 0.5 },
  { id: "low_lat", name: "纳秒级竞速", desc: "特工与故障体移速同时 +25%", speedAll: 1.25 },
  { id: "rack_alert", name: "机柜级告警", desc: "高危精英刷新快 3 倍，量子金库满地掉", eliteMul: 3 },
  { id: "telemetry", name: "全机房遥测", desc: "算力晶石拾取带宽直接 ×3", pickupMul: 3 },
  { id: "tapeout", name: "敏捷流片迭代", desc: "武器升至 Lv.3 即可触发超维进化", evoLv: 3 },
  { id: "asic_only", name: "单核专用 ASIC", desc: "限带 1 套主武器，但算力经验 +65%", slots: 1, xpMul: 1.65 },
];

// 10-Tier AI Compute Architect Promotion Route (with active permanent perks!)
export const TITLES = [
  { rank: 1, t: 0, name: "L1 · 见习机房巡检员", perk: "解锁基础 6 大算力产线权限", bonus: {} },
  { rank: 2, t: 45, name: "L2 · 存储阵列工程师", perk: "永久加成：最大生命 +10", bonus: { maxHp: 10 }, reward: 40 },
  { rank: 3, t: 90, name: "L3 · 光互连架构师", perk: "永久加成：拾取带宽 +12%", bonus: { pickup: 0.12 }, reward: 60 },
  { rank: 4, t: 150, name: "L4 · 微架构指令总师", perk: "永久加成：武器冷却 −4%", bonus: { cd: -0.04 }, reward: 80 },
  { rank: 5, t: 240, name: "L5 · 张量集群首席", perk: "永久加成：暴击率 +4% · 暴伤 +10%", bonus: { crit: 0.04, critDmg: 0.1 }, reward: 110 },
  { rank: 6, t: 330, name: "L6 · 先进封装巨匠", perk: "永久加成：全局伤害 +6%", bonus: { might: 0.06 }, reward: 140 },
  { rank: 7, t: 420, name: "L7 · 超算中心厂长", perk: "永久加成：每局重抽次数 +1", bonus: { reroll: 1 }, reward: 180 },
  { rank: 8, t: 540, name: "L8 · 吉瓦级算力统帅", perk: "永久加成：护甲 +1 · 生命 +18", bonus: { armor: 1, maxHp: 18 }, reward: 220 },
  { rank: 9, t: 660, name: "L9 · 硅基智能先驱", perk: "永久加成：算力经验 +10%", bonus: { xpMul: 0.1 }, reward: 280 },
  { rank: 10, t: 780, name: "L10 · HONE 首席算力官", perk: "永久加成：全局伤害 +10%", bonus: { might: 0.1 }, reward: 360 },
];

export function getTitle(sec) {
  let cur = TITLES[0];
  for (const item of TITLES) if (sec >= item.t) cur = item;
  return cur;
}

export function getNextTitle(sec) {
  for (const item of TITLES) if (sec < item.t) return item;
  return null;
}

export function getPromotionBonuses(bestSec) {
  const out = { maxHp: 0, pickup: 0, cd: 0, crit: 0, critDmg: 0, might: 0, reroll: 0, armor: 0, xpMul: 0 };
  for (const item of TITLES) {
    if (bestSec >= item.t && item.bonus) {
      for (const [k, v] of Object.entries(item.bonus)) {
        out[k] = (out[k] || 0) + v;
      }
    }
  }
  return out;
}

export const ACHIEVEMENTS = [
  { id: "first", name: "首次上电", text: "完成第 1 次数据中心巡检", stat: "runs", n: 1, cores: 60 },
  { id: "k200", name: "链路排障", text: "单局清除 200 个硬件异常", run: "kills", n: 200, cores: 45 },
  { id: "k800", name: "机柜清道夫", text: "单局清除 800 个硬件异常", run: "kills", n: 800, cores: 85 },
  { id: "k2000", name: "零故障集群", text: "单局清除 2000 个硬件异常", run: "kills", n: 2000, cores: 160 },
  { id: "t120", name: "满载烤机", text: "单局稳定运行超过 2 分钟", run: "time", n: 120, cores: 45 },
  { id: "t300", name: "五个九可用性", text: "单局稳定运行超过 5 分钟", run: "time", n: 300, cores: 90 },
  { id: "t480", name: "跨产线贯通", text: "单局稳定运行超过 8 分钟", run: "time", n: 480, cores: 160 },
  { id: "t720", name: "吉瓦级神话", text: "单局稳定运行超过 12 分钟", run: "time", n: 720, cores: 320 },
  { id: "b1", name: "坏块修复", text: "击破「坏块吞噬者·闪存巨像」", codex: "boss1", n: 1, cores: 65 },
  { id: "b2", name: "相干校准", text: "击破「色散棱镜·硅光主宰」", codex: "boss2", n: 1, cores: 100 },
  { id: "b3", name: "死锁解除", text: "击破「死锁仲裁官·乱序巨核」", codex: "boss3", n: 1, cores: 150 },
  { id: "b4", name: "突破显存墙", text: "击破「显存墙霸主·张量熔炉」", codex: "boss4", n: 1, cores: 220 },
  { id: "b5", name: "满分良率", text: "击破「良率黑洞·光刻母神」", codex: "boss5", n: 1, cores: 280 },
  { id: "evo1", name: "流片进化", text: "完成任意 1 套硬件超维进化", stat: "evos", n: 1, cores: 55 },
  { id: "evo8", name: "全栈超算库", text: "点亮全部 8 套超维进化图鉴", evoAll: 1, n: 8, cores: 320 },
  { id: "c100", name: "满带宽吞吐", text: "单局最高连击达到 100", run: "maxCombo", n: 100, cores: 45 },
  { id: "c400", name: "太比特洪流", text: "单局最高连击达到 400", run: "maxCombo", n: 400, cores: 120 },
  { id: "lv20", name: "算力集群扩容", text: "单局升至 Lv.20", run: "level", n: 20, cores: 60 },
  { id: "d1", name: "每日值班", text: "完成 1 次每日集群挑战", stat: "dailyDone", n: 1, cores: 65 },
  { id: "k10k", name: "万障清零", text: "生涯累计清除 10000 个硬件异常", stat: "kills", n: 10000, cores: 320 },
];

export const COMBO_STEPS = [25, 50, 100, 200, 300, 500, 800, 1000, 1500, 2000];
export const COMBO_CALLOUTS = [
  "链路畅通！",
  "带宽拉满！",
  "张量共振！",
  "光速吞吐！",
  "超频全开！",
  "零延迟清场！",
  "千卡同频！",
  "万卡集群觉醒！",
  "良率 100%！",
  "HONE 算力巅峰！",
];

export function fmtTime(sec) {
  sec = Math.max(0, Math.floor(sec));
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return (m < 10 ? "0" : "") + m + ":" + (s < 10 ? "0" : "") + s;
}

export function dateKey(d = new Date()) {
  const m = d.getMonth() + 1;
  const day = d.getDate();
  return `${d.getFullYear()}-${m < 10 ? "0" : ""}${m}-${day < 10 ? "0" : ""}${day}`;
}

export function fnv1a(str) {
  let h = 2166136261;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

export function getDailyChallenge(key = dateKey()) {
  const seed = fnv1a("hone-adventure-v2:" + key);
  const hero = HEROES[seed % HEROES.length];
  const rule = RULES[(seed >>> 5) % RULES.length];
  const stage = STAGES[(seed >>> 8) % STAGES.length];
  let weapon = WEAPONS[(seed >>> 11) % WEAPONS.length].id;
  if (rule.id === "asic_only") {
    weapon = ["hbm", "prism", "bus", "drone"][(seed >>> 17) % 4];
  }
  return { key, seed, char: hero.id, rule: rule.id, stage: stage.id, weapon };
}

export function mulberry32(seed) {
  let t = seed | 0;
  return function () {
    t = (t + 0x6d2b79f5) | 0;
    let r = Math.imul(t ^ (t >>> 15), 1 | t);
    r = (r + Math.imul(r ^ (r >>> 7), 61 | r)) ^ r;
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
}
