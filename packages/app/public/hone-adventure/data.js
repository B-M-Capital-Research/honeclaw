// HONE Adventure V3 — AI Data Center Sectors, 10 Hardware Weapons, Synergies, Relics & Deep Agent System

export const VERSION = "3.0.0";
export const HONECLAW_GITHUB_URL = "https://github.com/B-M-Capital-Research/honeclaw";
export const HONECLAW_REPO_LABEL = "github.com/B-M-Capital-Research/honeclaw";

export const MAX_WEAPON_LV = 5;
export const MAX_OVERCLOCK_LV = 5;
export const DEFAULT_W_SLOTS = 5;
export const DEFAULT_P_SLOTS = 5;
export const WAVE_SEC = 22;
export const STAGE_SEC = 70;
export const BOSS_SEC = 85;

// 3 Cleanroom Stress-Test Difficulty Tiers
export const DIFFICULTIES = [
  {
    id: "std",
    name: "标准巡检",
    short: "标准",
    desc: "标准机房负载 · 适合熟悉武器联动与产线机制",
    hpMul: 1,
    dmgMul: 1,
    spMul: 1,
    coreMul: 1,
    xpMul: 1,
    maxStar: 1,
    color: "#2563eb",
  },
  {
    id: "oc",
    name: "满载压测",
    short: "满载 ×1.45",
    desc: "故障血量 +45% · 词条精英增多 · 算力晶核产出 +45% · 可拿二星徽章",
    hpMul: 1.45,
    dmgMul: 1.18,
    spMul: 1.07,
    coreMul: 1.45,
    xpMul: 1.15,
    maxStar: 2,
    color: "#d97706",
  },
  {
    id: "apex",
    name: "极限流片",
    short: "极限 ×2.0",
    desc: "故障血量翻倍 · 终局双巨兽同屏压测 · 算力晶核产出 ×2.0 · 可拿三星金章",
    hpMul: 2.0,
    dmgMul: 1.32,
    spMul: 1.13,
    coreMul: 2.0,
    xpMul: 1.28,
    maxStar: 3,
    color: "#7c3aed",
  },
];

export const DIFF_IDX = {};
DIFFICULTIES.forEach((d, i) => {
  DIFF_IDX[d.id] = i;
});

// 6 Switchable & Interactive AI Data Center Sector Stages
export const STAGES = [
  {
    id: "storage",
    code: "SECTOR-01",
    short: "存储阵列",
    name: "NAND & HBM 存储阵列",
    sub: "晶核掉落 +18% · 踩「闪存补给桩」爆大量经验球与晶核",
    nodeName: "闪存补给桩",
    nodeKind: "cache_drop",
    nodeDesc: "站进圆圈充能 2 秒，喷出满地经验球与额外晶核",
    homeHeroes: ["hone", "astra"],
    accent: "#2563eb",
    badgeBg: "#dbeafe",
    floorTint: [0.95, 0.97, 1.0],
    gridCol: 0x3b82f6,
    bossId: "boss1",
    mod: { coreBonus: 0.18 },
    bias: [0, 5, 2],
    propMode: 0,
  },
  {
    id: "optics",
    code: "SECTOR-02",
    short: "光模块",
    name: "800G / 1.6T 光模块走廊",
    sub: "子弹飞速 +20% · 踩「硅光增幅塔」获 8 秒高暴击高射速",
    nodeName: "硅光增幅塔",
    nodeKind: "optic_haste",
    nodeDesc: "站进圆圈充能 2 秒，8 秒内暴击率 +25%、攻速 +25%",
    homeHeroes: ["photon", "quant"],
    accent: "#0891b2",
    badgeBg: "#cffafe",
    floorTint: [0.93, 0.98, 1.0],
    gridCol: 0x06b6d4,
    bossId: "boss2",
    mod: { projSpeed: 0.2 },
    bias: [3, 1, 7],
    propMode: 1,
  },
  {
    id: "cpu",
    code: "SECTOR-03",
    short: "CPU 中枢",
    name: "超标量指令 CPU 中枢",
    sub: "技能冷却 −10% · 踩「超频时钟站」立刷新主动大招",
    nodeName: "超频时钟站",
    nodeKind: "cpu_overclock",
    nodeDesc: "站进圆圈充能 2 秒，立即刷新主动大招并加速全武器 6 秒",
    homeHeroes: ["quant", "hone"],
    accent: "#4f46e5",
    badgeBg: "#e0e7ff",
    floorTint: [0.95, 0.96, 1.0],
    gridCol: 0x6366f1,
    bossId: "boss3",
    mod: { cdMul: -0.1 },
    bias: [1, 4, 0],
    propMode: 0,
  },
  {
    id: "gpu",
    code: "SECTOR-04",
    short: "GPU 矩阵",
    name: "张量加速 GPU 算力矩阵",
    sub: "暴击伤害 +25% · 踩「张量轰炸台」对全屏怪群降下连环轰炸",
    nodeName: "张量轰炸台",
    nodeKind: "gpu_barrage",
    nodeDesc: "站进圆圈充能 2 秒，自动锁定周围怪堆连续引爆 6 轮矩阵轰炸",
    homeHeroes: ["rubin", "photon"],
    accent: "#059669",
    badgeBg: "#d1fae5",
    floorTint: [0.93, 0.99, 0.96],
    gridCol: 0x10b981,
    bossId: "boss4",
    mod: { critDmg: 0.25 },
    bias: [2, 5, 6],
    propMode: 2,
  },
  {
    id: "packaging",
    code: "SECTOR-05",
    short: "先进封装",
    name: "CoWoS & EUV 先进封装工坊",
    sub: "升级经验 +18% · 踩「无尘修复舱」回血 30% 并套一层护盾",
    nodeName: "无尘修复舱",
    nodeKind: "wafer_heal",
    nodeDesc: "站进圆圈充能 2 秒，恢复 30% 最大生命并获得 1 层免伤护盾",
    homeHeroes: ["rubin", "astra"],
    accent: "#d97706",
    badgeBg: "#fef3c7",
    floorTint: [1.0, 0.98, 0.94],
    gridCol: 0xf59e0b,
    bossId: "boss5",
    mod: { xpMul: 0.18 },
    bias: [6, 7, 3],
    propMode: 3,
  },
  {
    id: "hyperscale",
    code: "SECTOR-06",
    short: "超算总机房",
    name: "GW 级液冷超算总机房",
    sub: "全局伤害 +15% · 踩「吉瓦母线塔」全屏落雷清怪并吸回全图经验",
    nodeName: "吉瓦母线塔",
    nodeKind: "gw_surge",
    nodeDesc: "站进圆圈充能 2 秒，释放全屏高压落雷并瞬间吸取全图经验球",
    homeHeroes: ["astra", "hone"],
    accent: "#7c3aed",
    badgeBg: "#ede9fe",
    floorTint: [0.95, 0.96, 1.0],
    gridCol: 0x8b5cf6,
    bossId: "boss4",
    mod: { might: 0.15 },
    bias: [4, 5, 6, 3],
    propMode: 2,
  },
];

export const STAGE_IDX = {};
STAGES.forEach((s, i) => {
  STAGE_IDX[s.id] = i;
  s.k = i;
});

// 10 AI Hardware Weapons + 10 Super Evolutions (Titles Kept, Plain-Language Intuitive Descriptions!)
export const WEAPONS = [
  {
    id: "hbm",
    name: "HBM 带宽飞梭",
    evoName: "3D 堆叠数据洪流",
    partner: "clock",
    color: "#2563eb",
    desc: "向面朝方向并排齐射多发高速子弹，可一路穿透整排怪物",
    evoDesc: "进化成四管机枪极速连发不停，子弹命中终点还会小范围爆炸",
    up: ["", "并排子弹 +1 发", "伤害 +8，穿透数量 +1", "并排子弹 +1 发", "伤害 +10，射击间隔大幅缩短"],
    lv: [
      { dmg: 13, cd: 0.72, n: 2, pierce: 0 },
      { dmg: 13, cd: 0.7, n: 3, pierce: 0 },
      { dmg: 21, cd: 0.68, n: 3, pierce: 1 },
      { dmg: 21, cd: 0.64, n: 4, pierce: 1 },
      { dmg: 31, cd: 0.5, n: 4, pierce: 2 },
    ],
    evo: { dmg: 26, cd: 0.14, n: 4, pierce: 2, pop: 44 },
  },
  {
    id: "prism",
    name: "硅光分光束",
    evoName: "1.6T 波分相干阵列",
    partner: "waveguide",
    color: "#0891b2",
    desc: "自动瞄准最近敌人发射激光枪，命中后向后散射出 3 道子激光",
    evoDesc: "同时射出 2 道主激光，每次命中散射出 5 道扇形彩色激光横扫后排",
    up: ["", "主激光与子激光伤害 +6", "同时发射主激光 +1 道", "伤害 +10，飞行距离更远", "命中分裂子激光增至 4 道"],
    lv: [
      { dmg: 16, cd: 1.12, n: 1, split: 3 },
      { dmg: 22, cd: 1.08, n: 1, split: 3 },
      { dmg: 22, cd: 1.05, n: 2, split: 3 },
      { dmg: 32, cd: 0.98, n: 2, split: 3 },
      { dmg: 38, cd: 0.86, n: 2, split: 4 },
    ],
    evo: { dmg: 42, cd: 0.52, n: 2, split: 5 },
  },
  {
    id: "tensor",
    name: "张量矩阵炮",
    evoName: "Rubin 脉动方阵",
    partner: "fp8",
    color: "#059669",
    desc: "自动锁定怪最多的位置落下方形轰炸区，短暂瞄准后大范围爆炸",
    evoDesc: "连续轰炸超大范围方阵，主爆炸后还会向四角分裂出 4 个小爆炸区",
    up: ["", "爆炸伤害 +14", "轰炸区数量 +1，范围变大", "爆炸伤害 +16", "轰炸区数量 +1，冷却更快"],
    lv: [
      { dmg: 32, cd: 1.95, n: 1, rad: 64 },
      { dmg: 46, cd: 1.9, n: 1, rad: 64 },
      { dmg: 46, cd: 1.85, n: 2, rad: 74 },
      { dmg: 62, cd: 1.78, n: 2, rad: 74 },
      { dmg: 62, cd: 1.48, n: 3, rad: 82 },
    ],
    evo: { dmg: 82, cd: 1.25, n: 3, rad: 98, mini: 4, miniR: 54 },
  },
  {
    id: "cache",
    name: "L3 缓存环刃",
    evoName: "零延迟一致性圣环",
    partner: "ecc",
    color: "#4f46e5",
    desc: "召唤多枚芯片全天候绕着身体旋转，切碎并弹开靠近的怪物",
    evoDesc: "内外双圈共 12 枚芯片反向高速护体，每 2.4 秒向外震退全圈敌人",
    up: ["", "环绕芯片 +1 枚", "伤害 +8，旋转圈变大", "环绕芯片 +1 枚，转速加快", "芯片 +1 枚，附带周期震退波"],
    lv: [
      { dmg: 12, n: 2, rad: 62, spin: 3.3, wave: 0 },
      { dmg: 12, n: 3, rad: 64, spin: 3.5, wave: 0 },
      { dmg: 20, n: 3, rad: 72, spin: 3.7, wave: 0 },
      { dmg: 22, n: 4, rad: 76, spin: 4.0, wave: 0 },
      { dmg: 28, n: 5, rad: 82, spin: 4.3, wave: 3.6, waveDmg: 22, waveR: 132 },
    ],
    evo: { dmg: 34, n: 5, n2: 7, rad: 58, rad2: 116, spin: 4.6, wave: 2.4, waveDmg: 38, waveR: 174 },
  },
  {
    id: "bus",
    name: "PCB 母线电弧",
    evoName: "吉瓦级超导馈线",
    partner: "overvolt",
    color: "#d97706",
    desc: "放出会在怪物之间自动来回弹射的连锁闪电，怪越密集清得越快",
    evoDesc: "双路高压闪电超高频连劈，连续弹射 7 次并在首个目标脚下引发电爆",
    up: ["", "弹射次数 +1", "闪电伤害 +11", "闪电条数 +1，弹射次数 +2", "伤害 +14，劈击频率加快"],
    lv: [
      { dmg: 19, cd: 1.26, n: 1, chain: 2 },
      { dmg: 19, cd: 1.24, n: 1, chain: 3 },
      { dmg: 30, cd: 1.2, n: 1, chain: 3 },
      { dmg: 30, cd: 1.14, n: 2, chain: 5 },
      { dmg: 44, cd: 0.92, n: 2, chain: 5 },
    ],
    evo: { dmg: 52, cd: 0.36, n: 2, chain: 7, boom: 50 },
  },
  {
    id: "litho",
    name: "EUV 光刻扫描",
    evoName: "2nm 极紫外全域曝光",
    partner: "reticle",
    color: "#7c3aed",
    desc: "朝前方扫出一条宽幅穿透激光束，直接贯穿整条直线上的所有敌人",
    evoDesc: "在身体四周展开十字形 4 道巨型激光束，全天候旋转横扫全屏",
    up: ["", "激光每次伤害 +5", "激光束变宽变长", "激光持续时间延长", "前后双向同时扫射，伤害 +6"],
    lv: [
      { dmg: 10, cd: 2.15, len: 255, wid: 22, dur: 0.58, n: 1 },
      { dmg: 15, cd: 2.1, len: 255, wid: 22, dur: 0.58, n: 1 },
      { dmg: 15, cd: 2.05, len: 305, wid: 30, dur: 0.62, n: 1 },
      { dmg: 16, cd: 2.0, len: 305, wid: 30, dur: 0.95, n: 1 },
      { dmg: 22, cd: 1.9, len: 320, wid: 30, dur: 0.95, n: 2 },
    ],
    evo: { dmg: 24, len: 305, wid: 32, n: 4, spin: 1.72 },
  },
  {
    id: "drone",
    name: "ASIC 推理蜂群",
    evoName: "自主智能体编队",
    partner: "fabric",
    color: "#0284c7",
    desc: "放出回旋镖式的智能无人机，穿进怪堆弧线切割一圈后再飞回身边",
    evoDesc: "召唤 5 架永久伴飞的护卫战机绕身高速盘旋，全自动切碎沿途敌人",
    up: ["", "同时放飞无人机 +1 架", "伤害 +9，飞行圈扩大", "无人机 +1 架", "伤害 +11，机身变大更好命中"],
    lv: [
      { dmg: 17, cd: 1.42, n: 1, range: 190, size: 1 },
      { dmg: 17, cd: 1.4, n: 2, range: 190, size: 1 },
      { dmg: 26, cd: 1.36, n: 2, range: 230, size: 1.08 },
      { dmg: 26, cd: 1.3, n: 3, range: 230, size: 1.15 },
      { dmg: 37, cd: 1.2, n: 3, range: 250, size: 1.35 },
    ],
    evo: { dmg: 44, n: 5, size: 1.55, rMin: 56, rMax: 202 },
  },
  {
    id: "cryo",
    name: "浸没液冷喷射",
    evoName: "绝对零度超导池",
    partner: "manifold",
    color: "#0d9488",
    desc: "在怪群脚下铺开冰冻法阵，让里面的怪物持续掉血并大幅减速",
    evoDesc: "生成超大冰冻漩涡把周围怪物强行吸向中心，结束时引爆高额冰霜新星",
    up: ["", "持续伤害 +4", "冰冻阵范围更大、持续更久", "同时铺设冰冻阵 +1 个", "伤害 +6，铺设间隔缩短"],
    lv: [
      { dmg: 8, cd: 3.4, n: 1, rad: 58, dur: 3.1 },
      { dmg: 12, cd: 3.35, n: 1, rad: 58, dur: 3.1 },
      { dmg: 12, cd: 3.25, n: 1, rad: 70, dur: 4.0 },
      { dmg: 12, cd: 3.15, n: 2, rad: 70, dur: 4.0 },
      { dmg: 18, cd: 2.65, n: 2, rad: 76, dur: 4.0 },
    ],
    evo: { dmg: 25, cd: 4.9, n: 1, rad: 146, dur: 4.8, burst: 128 },
  },
  {
    id: "nvlink",
    name: "NVLink 光网矩阵",
    evoName: "全互联超算穹顶",
    partner: "ups",
    color: "#db2777",
    desc: "用高能激光线把周围多个怪物连成电网，碰到连线的怪物持续灼烧掉血",
    evoDesc: "张开覆盖周身的多节点星型激光大网，网内所有敌人高频灼烧并减速",
    up: ["", "同时连接节点 +1 个", "灼烧伤害 +6，连接半径扩大", "同时连接节点 +1 个", "伤害 +8，激光网持续更久"],
    lv: [
      { dmg: 14, cd: 1.8, nodes: 3, rad: 155, dur: 1.35 },
      { dmg: 14, cd: 1.75, nodes: 4, rad: 165, dur: 1.4 },
      { dmg: 20, cd: 1.7, nodes: 4, rad: 190, dur: 1.45 },
      { dmg: 20, cd: 1.62, nodes: 5, rad: 200, dur: 1.55 },
      { dmg: 28, cd: 1.48, nodes: 6, rad: 215, dur: 1.7 },
    ],
    evo: { dmg: 36, cd: 0.95, nodes: 8, rad: 245, dur: 1.15 },
  },
  {
    id: "liquid_n2",
    name: "液氮超频迫击",
    evoName: "极寒坍缩奇点",
    partner: "power_rail",
    color: "#0369a1",
    desc: "朝远处怪群抛射液氮罐，落地炸出十字冰霜射线，冻结并贯穿四方敌人",
    evoDesc: "双发极寒奇点高频轰击，落地向八个方向迸发超长冰矛并冻结大片区域",
    up: ["", "十字冰轨伤害 +10", "冰轨长度增加，装填更快", "同时抛射液氮罐 +1 枚", "伤害 +14，冻结时间延长"],
    lv: [
      { dmg: 22, cd: 1.85, n: 1, rays: 4, rayLen: 105 },
      { dmg: 32, cd: 1.8, n: 1, rays: 4, rayLen: 105 },
      { dmg: 32, cd: 1.68, n: 1, rays: 4, rayLen: 132 },
      { dmg: 34, cd: 1.6, n: 2, rays: 4, rayLen: 135 },
      { dmg: 48, cd: 1.42, n: 2, rays: 4, rayLen: 150 },
    ],
    evo: { dmg: 60, cd: 1.08, n: 2, rays: 8, rayLen: 175 },
  },
];

export const WEAPON_IDX = {};
WEAPONS.forEach((w, i) => {
  WEAPON_IDX[w.id] = i;
  w.k = i;
});

// 10 Passive Modules (1-to-1 Evolution Partners for all 10 Weapons, Clear Descriptions!)
export const PASSIVES = [
  { id: "clock", name: "超频时钟", max: 3, desc: "所有武器攻击间隔缩短 10%（攻速更快）", color: "#2563eb" },
  { id: "waveguide", name: "波导扩束", max: 3, desc: "爆炸、光环、激光等攻击范围扩大 15%", color: "#0891b2" },
  { id: "ecc", name: "ECC 纠错甲", max: 3, desc: "最大生命 +32，每次受击少扣 1 点血", color: "#4f46e5" },
  { id: "fp8", name: "FP8 锐化", max: 3, desc: "暴击率 +10%，暴击伤害 +22%", color: "#059669" },
  { id: "reticle", name: "掩膜校准", max: 3, desc: "自动吸经验范围 +42%，获得经验 +8%", color: "#7c3aed" },
  { id: "overvolt", name: "高压偏置", max: 3, desc: "所有武器与技能伤害直接 +14%", color: "#d97706" },
  { id: "manifold", name: "液冷歧管", max: 3, desc: "技能持续时间 +18%，子弹飞行速度 +15%", color: "#0d9488" },
  { id: "fabric", name: "拓扑路由", max: 3, desc: "角色移动速度 +11%", color: "#0284c7" },
  { id: "ups", name: "固态储能", max: 3, desc: "每秒自动回复 1.0 点生命值", color: "#db2777" },
  { id: "power_rail", name: "超导供电轨", max: 3, desc: "主动大招冷却缩短 12%，精英与Boss伤害 +12%", color: "#0369a1" },
];

export const PASSIVE_IDX = {};
PASSIVES.forEach((p, i) => {
  PASSIVE_IDX[p.id] = i;
});

// 6 Cross-Weapon Synergy Protocols (Mid/Late-Game Build Depth!)
export const SYNERGIES = [
  {
    id: "syn_bus_hbm",
    name: "存算电弧总线",
    req: ["hbm", "bus"],
    icon: "bus",
    desc: "【双武联动】HBM 飞梭每命中 4 次自动向周围劈出一道连锁闪电，两把武器伤害 +18%",
  },
  {
    id: "syn_prism_litho",
    name: "全光谱相干蚀刻",
    req: ["prism", "litho"],
    icon: "prism",
    desc: "【双武联动】硅光分光束与光刻激光暴击率 +15%，被激光命中的敌人额外承受 20% 易伤",
  },
  {
    id: "syn_tensor_cryo",
    name: "超导低温爆破",
    req: ["tensor", "cryo"],
    icon: "tensor",
    desc: "【双武联动】张量矩阵炮轰炸范围 +18%，对减速状态敌人的爆炸伤害提高 35%",
  },
  {
    id: "syn_cache_drone",
    name: "近防蜂群卫队",
    req: ["cache", "drone"],
    icon: "drone",
    desc: "【双武联动】环刃与无人机伤害 +24%，无人机每次切割额外击退周围贴身怪",
  },
  {
    id: "syn_nvlink_bus",
    name: "全网高压过载",
    req: ["nvlink", "bus"],
    icon: "nvlink",
    desc: "【双武联动】光网额外多连 2 个敌人，母线电弧多弹射 2 次，两者攻速提升 16%",
  },
  {
    id: "syn_n2_prism",
    name: "极寒碎晶折射",
    req: ["liquid_n2", "prism"],
    icon: "liquid_n2",
    desc: "【双武联动】液氮迫击命中中心额外迸发 4 发追踪硅光针，冰系与光系伤害 +20%",
  },
];

// 8 Late-Game Mythic Compute Relics (Appears at Lv.10+ or after 1st Super Evolution!)
export const RELICS = [
  {
    id: "r_multishot",
    name: "异构多核编译器",
    icon: "hbm",
    desc: "【终局圣物】所有发射与投掷类主武器每次攻击额外多发射 +1 发弹道",
  },
  {
    id: "r_execute",
    name: "良率斩杀熔断器",
    icon: "fp8",
    desc: "【终局圣物】普通怪与精英怪血量低于 16% 时，任意命中直接秒杀",
  },
  {
    id: "r_shield",
    name: "容错隔离力场",
    icon: "ecc",
    desc: "【终局圣物】每 15 秒自动套上 1 层免伤护盾，护盾抵挡伤害时震退周围敌人",
  },
  {
    id: "r_siphon",
    name: "余热回收循环",
    icon: "ups",
    desc: "【终局圣物】每击败 28 个敌人自动回复 6 点生命；满血时改为释放清场冲击波",
  },
  {
    id: "r_chain_pop",
    name: "链式聚变电容",
    icon: "overvolt",
    desc: "【终局圣物】击败敌人时有 25% 概率原地引发电浆连环爆破，炸伤周围怪群",
  },
  {
    id: "r_slots",
    name: "超宽机架扩容",
    icon: "waveguide",
    desc: "【终局圣物】本局主武器槽位上限 +1、被动槽位上限 +1（最高可带 6 武器 + 6 被动）",
  },
  {
    id: "r_skill_cd",
    name: "纳秒中断控制器",
    icon: "clock",
    desc: "【终局圣物】角色主动大招冷却缩短 32%，放大招后 4 秒内全局伤害 +28%",
  },
  {
    id: "r_compound",
    name: "算力复利引擎",
    icon: "core",
    desc: "【终局圣物】当前及后续每升 1 级，额外提升 2.2% 全局伤害与晶核结算收益",
  },
];

export const BOOSTS = [
  { id: "b_might", name: "超频增幅包", desc: "所有武器伤害永久 +10%，并小幅加速大招充能（可无限叠加）" },
  { id: "b_heal", name: "液冷应急维护", desc: "立即回复 45% 最大生命，并获得 2 秒短暂无敌" },
  { id: "b_core", name: "晶圆良率红利", desc: "直接获得 +24 算力晶核，并提升本局拾取范围 10%" },
];

export const AI_TYPE = {
  CHASE: 0,
  DART: 1,
  SPIT: 2,
  DASH: 3,
  GHOST: 4,
  BOSS: 5,
};

// Elite Affixes for Late-Game Variety
export const ELITE_AFFIXES = [
  { id: "shield", name: "护盾型", hpMul: 1.35, spMul: 1.0, tint: 0x2563eb },
  { id: "haste", name: "极速型", hpMul: 1.0, spMul: 1.38, tint: 0x0891b2 },
  { id: "berserk", name: "狂暴型", hpMul: 1.15, spMul: 1.16, dmgMul: 1.3, tint: 0xe11d48 },
  { id: "split", name: "裂变型", hpMul: 1.1, spMul: 1.05, split: 4, tint: 0x059669 },
];

export const ENEMIES = [
  { id: "packet", name: "丢包爬虫", hp: 9, sp: 52, dmg: 8, r: 10, xp: 1, wt: 1, ai: 0, from: 0, w: 10 },
  { id: "jitter", name: "时延飞蚊", hp: 6, sp: 98, dmg: 6, r: 8, xp: 1, wt: 0.8, ai: 1, from: 22, w: 5.4 },
  { id: "thermal", name: "积热熔岩体", hp: 46, sp: 36, dmg: 14, r: 16, xp: 5, wt: 3.5, ai: 0, from: 50, w: 2.5 },
  { id: "crosstalk", name: "串扰光炮", hp: 21, sp: 43, dmg: 9, r: 12, xp: 3, wt: 1.4, ai: 2, from: 88, w: 2.3 },
  { id: "surge", name: "浪涌电牛", hp: 27, sp: 54, dmg: 14, r: 12, xp: 3, wt: 1.5, ai: 3, from: 120, w: 2.6 },
  { id: "leak", name: "内存泄漏块", hp: 31, sp: 53, dmg: 10, r: 14, xp: 3, wt: 1.6, ai: 0, from: 155, w: 2.1 },
  { id: "short", name: "短路电容弹", hp: 25, sp: 62, dmg: 18, r: 12, xp: 3, wt: 1.2, ai: 0, from: 195, w: 1.8 },
  { id: "halluc", name: "位翻转幽影", hp: 15, sp: 76, dmg: 9, r: 11, xp: 2, wt: 1, ai: 4, from: 1e9, w: 0 },
  { id: "frag", name: "坏道碎晶", hp: 8, sp: 88, dmg: 7, r: 8, xp: 1, wt: 0.7, ai: 0, from: 1e9, w: 0, hide: 1 },
  { id: "boss1", name: "坏块吞噬者·闪存巨像", hp: 1150, sp: 41, dmg: 18, r: 42, xp: 80, wt: 40, ai: 5, boss: 1, from: 1e9, w: 0 },
  { id: "boss2", name: "色散棱镜·硅光主宰", hp: 3300, sp: 46, dmg: 21, r: 44, xp: 115, wt: 40, ai: 5, boss: 2, from: 1e9, w: 0 },
  { id: "boss3", name: "死锁仲裁官·乱序巨核", hp: 7000, sp: 40, dmg: 24, r: 45, xp: 155, wt: 40, ai: 5, boss: 3, from: 1e9, w: 0 },
  { id: "boss4", name: "显存墙霸主·张量熔炉", hp: 11800, sp: 44, dmg: 27, r: 48, xp: 195, wt: 40, ai: 5, boss: 4, from: 1e9, w: 0 },
  { id: "boss5", name: "良率黑洞·光刻母神", hp: 17200, sp: 42, dmg: 30, r: 50, xp: 250, wt: 45, ai: 5, boss: 5, from: 1e9, w: 0 },
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
    return (1 + 0.36 * m + 0.11 * m * m) * (m > 4.5 ? Math.pow(1.25, m - 4.5) : 1);
  },
  dmgMul: (t) => {
    const m = t / 60;
    return (1 + 0.085 * m) * (m > 5 ? Math.pow(1.08, m - 5) : 1);
  },
  spMul: (t) => Math.min(1.46, 1 + 0.032 * (t / 60)),
  target: (t) => Math.min(360, 12 + 0.96 * t),
  rate: (t) => 1.4 + t / 19.5,
  xpNeed: (lv) => Math.round(4 + 3.9 * (lv - 1) + 0.86 * Math.pow(lv - 1, 1.86)),
};

// 5 Distinct Playable Agents with Active Tactical Skills, Home-Sector Synergies & Mastery Progression!
export const HEROES = [
  {
    id: "hone",
    sprite: "hone",
    name: "小砺",
    kind: "全栈算力架构师",
    weapon: "hbm",
    skillName: "全频调度脉冲",
    skillCd: 14,
    skillDesc: "清空身边敌方弹幕、震退周围怪群，并瞬间吸回大范围经验球",
    trait: "升级经验 +16% · 当前产线加成效果翻倍 · 均衡全能",
    homeStages: ["storage", "cpu"],
    homeBonusDesc: "主场（存储/CPU）：攻速额外 +10%",
    mod: { xpMul: 0.16, stageBoost: 1 },
    unlock: null,
    price: 0,
  },
  {
    id: "rubin",
    sprite: "rubin",
    name: "芯核",
    kind: "浸没液冷重卫",
    weapon: "tensor",
    skillName: "超导冰甲壁垒",
    skillCd: 16,
    skillDesc: "开启 3.5 秒免伤冰盾并冻结近身怪，结束时向外炸开高伤冰霜新星",
    trait: "生命 +65 · 减伤护甲 +2 · 最大生命越高爆炸范围伤害越强",
    homeStages: ["gpu", "packaging"],
    homeBonusDesc: "主场（GPU/封装）：每秒额外回血 +1.2",
    mod: { maxHp: 65, armor: 2, hpToMight: 1, thorns: 1 },
    unlock: { stat: "kills", n: 500, text: "累计清除 500 个硬件异常" },
    price: 260,
  },
  {
    id: "photon",
    sprite: "photon",
    name: "光瞳",
    kind: "硅光狙击专家",
    weapon: "prism",
    skillName: "相干超频齐射",
    skillCd: 13,
    skillDesc: "进入 4 秒超频聚焦（暴击 +35% / 攻速 +30%），并连射 8 发高伤穿透光枪",
    trait: "暴击率 +14% · 子弹飞速 +22% · 暴击击败敌人加速大招充能",
    homeStages: ["optics", "gpu"],
    homeBonusDesc: "主场（光模块/GPU）：暴击伤害额外 +25%",
    mod: { crit: 0.14, pspd: 0.22, pickup: 0.3, critCdRefund: 1 },
    unlock: { stat: "bosses", n: 1, text: "首次击败任意 1 台产线巨兽 Boss" },
    price: 360,
  },
  {
    id: "quant",
    sprite: "quant",
    name: "量闪",
    kind: "高频极速猎手",
    weapon: "bus",
    skillName: "纳秒相位闪现",
    skillCd: 10,
    skillDesc: "朝移动方向无敌瞬移穿过怪群，在起点与终点引发双重电爆并加速 3 秒",
    trait: "移速 +18% · 武器冷却 −12% · 移动蓄能自动向周围劈出电弧",
    homeStages: ["cpu", "optics"],
    homeBonusDesc: "主场（CPU/光模块）：技能冷却再缩减 6%",
    mod: { speed: 0.18, cd: -0.12, strideZap: 1 },
    unlock: { stat: "bestTime", n: 180, text: "单局巡检坚持满 3 分钟" },
    price: 460,
  },
  {
    id: "astra",
    sprite: "gold",
    name: "星枢",
    kind: "曜金主权统帅",
    weapon: "nvlink",
    skillName: "天基超算降临",
    skillCd: 17,
    skillDesc: "展开 4.5 秒曜金超算领域减速全场，连续降下 6 道高能光柱精准轰杀强敌",
    trait: "初始被动槽 +1 · 重抽次数 +1 · 每进化 1 把超武全局伤害 +12%",
    homeStages: ["hyperscale", "packaging"],
    homeBonusDesc: "主场（超算/封装）：开局直接直升 Lv.2",
    mod: { extraPSlot: 1, bonusReroll: 1, evoMight: 0.12, xpMul: 0.1 },
    unlock: { stat: "evos", n: 3, text: "累计完成 3 次超武进化（或点亮 GitHub Star 直接解锁）" },
    price: 640,
  },
];

export const HERO_IDX = {};
HEROES.forEach((h, i) => {
  HERO_IDX[h.id] = i;
});

// Agent Mastery Ranks (Lv.1 ~ Lv.5 per Hero — Unlocks Starter Weapon Swapping at Lv.2!)
export const MASTERY_COSTS = [0, 140, 260, 420, 620];
export const MASTERY_PERKS = [
  "Lv.1 基础执勤权限",
  "Lv.2 解锁「自由更换起手主武器」+ 角色生命 +15",
  "Lv.3 角色主动大招冷却缩短 12% + 全局伤害 +6%",
  "Lv.4 开局自带 1 次免费锁卡重抽 + 拾取范围 +20%",
  "Lv.5 满阶专精：暴击率 +6% · 终伤 +10% · 开局额外获 15 晶核",
];

export function getHeroMasteryBonuses(lv = 1) {
  return {
    freeWeapon: lv >= 2,
    maxHp: lv >= 2 ? 15 : 0,
    skillCdMul: lv >= 3 ? 0.88 : 1,
    might: (lv >= 3 ? 0.06 : 0) + (lv >= 5 ? 0.1 : 0),
    reroll: lv >= 4 ? 1 : 0,
    pickup: lv >= 4 ? 0.2 : 0,
    crit: lv >= 5 ? 0.06 : 0,
    startCores: lv >= 5 ? 15 : 0,
  };
}

// 4-Pillar × 4-Node = 16-Node AI Infrastructure Upgrade Matrix (With 8-pt Apex Core Synergies!)
export const TALENT_BRANCHES = [
  {
    id: "interconnect",
    name: "存算互联",
    sub: "满 6 点激活「终伤 +10%」· 满 10 点再激活「所有发射武器穿透 +1」",
    milestone: 6,
    apexMilestone: 10,
    bonusDesc: "终伤 +10%",
    apexDesc: "弹道穿透 +1",
  },
  {
    id: "litho",
    name: "光刻制程",
    sub: "满 6 点激活「暴击率 +7%」· 满 10 点再激活「超武突破伤害加成提高 25%」",
    milestone: 6,
    apexMilestone: 10,
    bonusDesc: "暴击率 +7%",
    apexDesc: "超武突破增效",
  },
  {
    id: "thermal",
    name: "液冷供电",
    sub: "满 6 点激活「减伤护甲 +2」· 满 10 点再激活「开局自带 1 层免伤护盾」",
    milestone: 6,
    apexMilestone: 10,
    bonusDesc: "护甲 +2",
    apexDesc: "开局免伤护盾",
  },
  {
    id: "cluster",
    name: "智能调度",
    sub: "满 6 点激活「开局直升 Lv.2」· 满 10 点再激活「地图基站充能快 35%」",
    milestone: 6,
    apexMilestone: 10,
    bonusDesc: "开局升 Lv.2",
    apexDesc: "基站充能加速",
  },
];

export const TALENTS = [
  // Pillar 1: Interconnect (4 nodes)
  { id: "might", branch: "interconnect", icon: "overvolt", name: "HBM3E 预取", max: 5, per: "所有武器伤害 +7%", cost: [50, 105, 190, 320, 480] },
  { id: "haste", branch: "interconnect", icon: "clock", name: "PCIe 6.0 通道", max: 4, per: "武器攻击间隔 −3.5%", cost: [75, 150, 260, 420] },
  { id: "area", branch: "interconnect", icon: "waveguide", name: "NVLink 交换网", max: 4, per: "攻击与爆炸范围 +6%", cost: [65, 135, 240, 380] },
  { id: "boss_dmg", branch: "interconnect", icon: "bus", name: "矩阵定点算子", max: 3, per: "对精英与 Boss 伤害 +10%", cost: [110, 220, 360] },
  // Pillar 2: Lithography (4 nodes)
  { id: "crit", branch: "litho", icon: "fp8", name: "EUV 对准精度", max: 5, per: "暴击率 +3% · 暴击伤害 +8%", cost: [60, 120, 220, 360, 520] },
  { id: "proj", branch: "litho", icon: "manifold", name: "GAA 环绕栅极", max: 4, per: "子弹飞速与持续时间 +7%", cost: [65, 135, 240, 380] },
  { id: "reroll", branch: "litho", icon: "ui_retry", name: "晶圆良率重筛", max: 3, per: "每局重抽卡牌次数 +1（可锁卡）", cost: [110, 220, 360] },
  { id: "syn_rate", branch: "litho", icon: "prism", name: "异构封装桥接", max: 3, per: "双武联动与终局圣物出现率大幅提升", cost: [120, 240, 380] },
  // Pillar 3: Thermal & Power (4 nodes)
  { id: "hp", branch: "thermal", icon: "ecc", name: "双相浸没液冷", max: 5, per: "最大生命值 +18", cost: [50, 105, 190, 320, 480] },
  { id: "regen", branch: "thermal", icon: "ups", name: "SOFC 固态供电", max: 4, per: "每秒自动回血 +0.4", cost: [65, 135, 240, 380] },
  { id: "skill_cdr", branch: "thermal", icon: "power_rail", name: "超导储能电容", max: 3, per: "角色主动大招冷却缩短 8%", cost: [95, 190, 320] },
  { id: "revive", branch: "thermal", icon: "ecc", name: "UPS 灾备冗余", max: 1, per: "受到致命伤时半血复活并震开全屏怪物 1 次", cost: [400] },
  // Pillar 4: Cluster Orchestration (4 nodes)
  { id: "pickup", branch: "cluster", icon: "reticle", name: "光模块遥测", max: 5, per: "吸经验范围 +16% · 经验值 +5%", cost: [50, 105, 190, 320, 480] },
  { id: "speed", branch: "cluster", icon: "fabric", name: "RDMA 零拷贝", max: 5, per: "角色移动速度 +4%", cost: [50, 105, 190, 320, 480] },
  { id: "core", branch: "cluster", icon: "core", name: "强化学习挖矿", max: 5, per: "每局晶核产出 +14%", cost: [55, 115, 200, 340, 500] },
  { id: "node_boost", branch: "cluster", icon: "chest", name: "产线基站增幅", max: 3, per: "地图交互基站奖励与持续时间 +20%", cost: [100, 200, 340] },
];

export function branchPoints(talents, branchId) {
  let sum = 0;
  for (const t of TALENTS) {
    if (t.branch === branchId) sum += talents[t.id] || 0;
  }
  return sum;
}

// 8 Redesigned AI Data Center Daily Directives (Zero overlap with base template!)
export const RULES = [
  { id: "dual_boot", name: "双核并联启动", desc: "开局直接装配 2 把随机主武器，产线巨兽提前登场", dualStart: 1, bossFast: 0.8 },
  { id: "skill_frenzy", name: "主动技狂飙", desc: "角色主动大招冷却缩短 50%，放大招额外掉落晶核", skillCdMul: 0.5 },
  { id: "node_storm", name: "基站超频潮汐", desc: "地图产线基站刷新快 2 倍，踩圈奖励大幅强化", nodeFast: 2.0 },
  { id: "tapeout", name: "敏捷流片直通", desc: "主武器升到 Lv.3 配齐搭档即可直接进化超武", evoLv: 3 },
  { id: "overclock_glass", name: "液氮超限压测", desc: "所有伤害直接 ×2.1，最大生命减为 60% 但附带护盾", might: 2.1, hpMul: 0.6, startShield: 1 },
  { id: "boss_gauntlet", name: "巨兽压力测试", desc: "产线 Boss 每 60 秒轮番压境，击败必掉豪华三连金库", bossFast: 0.68, eliteMul: 2.2 },
  { id: "hyper_swarm", name: "峰值并发洪流", desc: "怪物潮密度 +45%，吸经验范围翻倍且升级更快", countMul: 1.45, xpMul: 1.4, pickupMul: 2.0 },
  { id: "relic_hunter", name: "终局圣物试炼", desc: "Lv.3 起即可刷出终局架构圣物卡，精英刷新翻倍", earlyRelic: 1, eliteMul: 2.0 },
];

// 12-Tier AI Compute Architect Promotion Route (Expanded Late-Game Career Ladder!)
export const TITLES = [
  { rank: 1, t: 0, name: "L1 · 见习机房巡检员", perk: "解锁全部 6 大算力产线自由切换权限", bonus: {} },
  { rank: 2, t: 45, name: "L2 · 存储阵列工程师", perk: "永久加成：最大生命 +12", bonus: { maxHp: 12 }, reward: 45 },
  { rank: 3, t: 90, name: "L3 · 光互连架构师", perk: "永久加成：吸经验范围 +15%", bonus: { pickup: 0.15 }, reward: 65 },
  { rank: 4, t: 150, name: "L4 · 微架构指令总师", perk: "永久加成：所有武器冷却 −4%", bonus: { cd: -0.04 }, reward: 85 },
  { rank: 5, t: 220, name: "L5 · 张量集群首席", perk: "永久加成：暴击率 +4% · 暴伤 +12%", bonus: { crit: 0.04, critDmg: 0.12 }, reward: 120 },
  { rank: 6, t: 300, name: "L6 · 先进封装巨匠", perk: "永久加成：全局伤害 +7%", bonus: { might: 0.07 }, reward: 150 },
  { rank: 7, t: 390, name: "L7 · 超算中心厂长", perk: "永久加成：每局重抽卡牌次数 +1", bonus: { reroll: 1 }, reward: 190 },
  { rank: 8, t: 480, name: "L8 · 吉瓦级算力统帅", perk: "永久加成：护甲 +1 · 生命 +20", bonus: { armor: 1, maxHp: 20 }, reward: 240 },
  { rank: 9, t: 600, name: "L9 · 异构超算总师", perk: "永久加成：主动大招冷却 −10%", bonus: { skillCd: -0.1 }, reward: 300 },
  { rank: 10, t: 720, name: "L10 · 硅基智能先驱", perk: "永久加成：经验获取 +12% · 晶核 +10%", bonus: { xpMul: 0.12, coreBonus: 0.1 }, reward: 360 },
  { rank: 11, t: 840, name: "L11 · 行星级算力缔造者", perk: "永久加成：暴击率 +5% · 全局伤害 +8%", bonus: { crit: 0.05, might: 0.08 }, reward: 440 },
  { rank: 12, t: 960, name: "L12 · HONE 首席算力官", perk: "终极特权：全局伤害 +12% · 开局自带免伤护盾", bonus: { might: 0.12, startShield: 1 }, reward: 600 },
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
  const out = { maxHp: 0, pickup: 0, cd: 0, crit: 0, critDmg: 0, might: 0, reroll: 0, armor: 0, xpMul: 0, skillCd: 0, coreBonus: 0, startShield: 0 };
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
  { id: "k200", name: "链路排障", text: "单局清除 200 个硬件异常", run: "kills", n: 200, cores: 50 },
  { id: "k800", name: "机柜清道夫", text: "单局清除 800 个硬件异常", run: "kills", n: 800, cores: 95 },
  { id: "k2500", name: "零故障集群", text: "单局清除 2500 个硬件异常", run: "kills", n: 2500, cores: 180 },
  { id: "t180", name: "满载烤机", text: "单局稳定运行满 3 分钟", run: "time", n: 180, cores: 55 },
  { id: "t360", name: "五个九可用性", text: "单局稳定运行满 6 分钟", run: "time", n: 360, cores: 110 },
  { id: "t600", name: "跨产线贯通", text: "单局稳定运行满 10 分钟", run: "time", n: 600, cores: 200 },
  { id: "t900", name: "吉瓦级神话", text: "单局稳定运行满 15 分钟", run: "time", n: 900, cores: 380 },
  { id: "b1", name: "坏块修复", text: "击败「坏块吞噬者·闪存巨像」", codex: "boss1", n: 1, cores: 70 },
  { id: "b2", name: "相干校准", text: "击败「色散棱镜·硅光主宰」", codex: "boss2", n: 1, cores: 100 },
  { id: "b3", name: "死锁解除", text: "击败「死锁仲裁官·乱序巨核」", codex: "boss3", n: 1, cores: 150 },
  { id: "b4", name: "突破显存墙", text: "击败「显存墙霸主·张量熔炉」", codex: "boss4", n: 1, cores: 220 },
  { id: "b5", name: "满分良率", text: "击败「良率黑洞·光刻母神」", codex: "boss5", n: 1, cores: 280 },
  { id: "evo1", name: "流片进化", text: "完成任意 1 把主武器的超武进化", stat: "evos", n: 1, cores: 60 },
  { id: "evo10", name: "全栈超算库", text: "点亮全部 10 把超武进化图鉴", evoAll: 1, n: 10, cores: 380 },
  { id: "syn1", name: "异构协同", text: "在局内激活任意 1 项双武联动协议", run: "synergies", n: 1, cores: 75 },
  { id: "oc3", name: "极限超频师", text: "将任意 1 把超武继续突破至 ★+3", run: "maxOverclock", n: 3, cores: 140 },
  { id: "node5", name: "基站巡检长", text: "单局充能激活 5 座产线交互基站", run: "nodesCaptured", n: 5, cores: 80 },
  { id: "d1", name: "每日值班", text: "完成 1 次每日集群挑战", stat: "dailyDone", n: 1, cores: 75 },
  { id: "k10k", name: "万障清零", text: "生涯累计清除 10000 个硬件异常", stat: "kills", n: 10000, cores: 350 },
];

export const COMBO_STEPS = [30, 60, 120, 240, 400, 600, 900, 1200, 1600, 2200];
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
  const seed = fnv1a("hone-adventure-v3:" + key);
  const hero = HEROES[seed % HEROES.length];
  const rule = RULES[(seed >>> 5) % RULES.length];
  const stage = STAGES[(seed >>> 8) % STAGES.length];
  const weapon = WEAPONS[(seed >>> 11) % WEAPONS.length].id;
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
