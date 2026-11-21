// HONE Adventure — Universe Data, Progression Curves, Weapons, Talents & Bestiary

export const VERSION = "1.0.0";
export const MAX_WEAPON_LV = 5;
export const DEFAULT_W_SLOTS = 4;
export const DEFAULT_P_SLOTS = 4;
export const WAVE_SEC = 20;
export const BOSS_SEC = 120;

export const WEAPONS = [
  {
    id: "blade",
    name: "研报飞刃",
    evoName: "万刃归宗",
    partner: "haste",
    color: "#ffad36",
    desc: "朝最近的市场噪音高速投掷锋利飞刃",
    evoDesc: "极速双刃风暴，命中终点绽放十字剑气爆破",
    up: ["", "飞刃 +1", "伤害 +6，穿透 +1", "飞刃 +1", "伤害 +7，出手更快"],
    lv: [
      { dmg: 11, cd: 0.78, n: 1, pierce: 0 },
      { dmg: 11, cd: 0.78, n: 2, pierce: 0 },
      { dmg: 17, cd: 0.78, n: 2, pierce: 1 },
      { dmg: 17, cd: 0.76, n: 3, pierce: 1 },
      { dmg: 24, cd: 0.58, n: 3, pierce: 2 },
    ],
    evo: { dmg: 20, cd: 0.11, n: 2, pierce: 2, pop: 36 },
  },
  {
    id: "wafer",
    name: "回旋晶圆",
    evoName: "超算星盘",
    partner: "area",
    color: "#5ce1e6",
    desc: "掷出旋转硅晶圆切割沿途目标并折返",
    evoDesc: "5 枚巨型晶圆呈呼吸轨道环绕周身，永不落地",
    up: ["", "晶圆 +1", "伤害 +7，射程更远", "晶圆 +1", "伤害 +9，晶圆放大"],
    lv: [
      { dmg: 15, cd: 1.55, n: 1, range: 175, size: 1 },
      { dmg: 15, cd: 1.55, n: 2, range: 175, size: 1 },
      { dmg: 22, cd: 1.55, n: 2, range: 215, size: 1.05 },
      { dmg: 22, cd: 1.45, n: 3, range: 215, size: 1.1 },
      { dmg: 31, cd: 1.35, n: 3, range: 230, size: 1.38 },
    ],
    evo: { dmg: 37, n: 5, size: 1.62, rMin: 48, rMax: 190 },
  },
  {
    id: "moat",
    name: "护城河盾",
    evoName: "固若金汤",
    partner: "margin",
    color: "#ffd97d",
    desc: "黄金护城河力场盾环绕周身，撞退近身敌人",
    evoDesc: "内外双层金盾反向运转，并定时释放护城河震波",
    up: ["", "金盾 +1", "伤害 +6，轨道扩大", "金盾 +1", "伤害 +6，常驻不停转"],
    lv: [
      { dmg: 10, n: 2, rad: 58, on: 4.2, off: 2.2 },
      { dmg: 10, n: 3, rad: 58, on: 4.2, off: 2.0 },
      { dmg: 16, n: 3, rad: 68, on: 4.6, off: 1.8 },
      { dmg: 16, n: 4, rad: 68, on: 5.2, off: 1.4 },
      { dmg: 22, n: 4, rad: 74, on: 1, off: 0 },
    ],
    evo: { dmg: 28, n: 5, n2: 7, rad: 56, rad2: 110, wave: 2.8, waveDmg: 28, waveR: 156 },
  },
  {
    id: "arc",
    name: "高频电弧",
    evoName: "闪崩雷劫",
    partner: "sharpe",
    color: "#6fffe9",
    desc: "降下量化闪电劈向噪音，并在怪群间弹射连锁",
    evoDesc: "高频双雷齐落，连锁 6 次并在首落点引发电磁轰炸",
    up: ["", "连锁 +1", "伤害 +9", "电弧 +1，连锁 +2", "伤害 +11，落雷更快"],
    lv: [
      { dmg: 17, cd: 1.38, n: 1, chain: 2 },
      { dmg: 17, cd: 1.38, n: 1, chain: 3 },
      { dmg: 26, cd: 1.38, n: 1, chain: 3 },
      { dmg: 26, cd: 1.32, n: 2, chain: 5 },
      { dmg: 37, cd: 1.06, n: 2, chain: 5 },
    ],
    evo: { dmg: 44, cd: 0.4, n: 2, chain: 6, boom: 42 },
  },
  {
    id: "pulse",
    name: "宏观脉冲",
    evoName: "周期共振",
    partner: "siphon",
    color: "#2ee59d",
    desc: "向外扩散翡翠脉冲环，震退身边一切噪音",
    evoDesc: "巨型周期共振波，强减速敌群并虹吸大范围算力晶石",
    up: ["", "范围 +20%", "伤害 +9", "范围 +20%，脉冲更密", "伤害 +11，二连共振"],
    lv: [
      { dmg: 13, cd: 2.55, rad: 108, n: 1 },
      { dmg: 13, cd: 2.55, rad: 130, n: 1 },
      { dmg: 22, cd: 2.55, rad: 130, n: 1 },
      { dmg: 22, cd: 2.05, rad: 155, n: 1 },
      { dmg: 33, cd: 2.05, rad: 155, n: 2 },
    ],
    evo: { dmg: 50, cd: 1.45, rad: 255, n: 1, slow: 1.7 },
  },
  {
    id: "mine",
    name: "止损爆雷",
    evoName: "连环清算",
    partner: "leverage",
    color: "#ff6b6b",
    desc: "布设止损触发雷，噪音触碰即刻引爆",
    evoDesc: "主雷引爆后向四周分裂弹射 4 枚子清算雷二次轰炸",
    up: ["", "伤害 +11", "爆雷 +1，范围扩大", "伤害 +13", "爆雷 +1，布雷更快"],
    lv: [
      { dmg: 28, cd: 2.15, n: 1, rad: 60 },
      { dmg: 39, cd: 2.15, n: 1, rad: 60 },
      { dmg: 39, cd: 2.15, n: 2, rad: 68 },
      { dmg: 52, cd: 2.15, n: 2, rad: 68 },
      { dmg: 52, cd: 1.7, n: 3, rad: 74 },
    ],
    evo: { dmg: 68, cd: 1.45, n: 3, rad: 90, mini: 4, miniR: 48 },
  },
  {
    id: "laser",
    name: "硅光射线",
    evoName: "太比特极光",
    partner: "compound",
    color: "#ffd166",
    desc: "发射高能穿透硅光束，横扫路径上所有目标",
    evoDesc: "3 道太比特极光绕身持续旋转横扫，永不熄灭",
    up: ["", "伤害 +3", "光束更粗更远", "照射更久", "伤害 +4，前后双光束"],
    lv: [
      { dmg: 8, cd: 2.35, len: 235, wid: 16, dur: 0.52, n: 1 },
      { dmg: 11, cd: 2.35, len: 235, wid: 16, dur: 0.52, n: 1 },
      { dmg: 11, cd: 2.35, len: 285, wid: 24, dur: 0.52, n: 1 },
      { dmg: 11, cd: 2.35, len: 285, wid: 24, dur: 0.88, n: 1 },
      { dmg: 15, cd: 2.25, len: 295, wid: 24, dur: 0.88, n: 2 },
    ],
    evo: { dmg: 17, len: 278, wid: 27, n: 3, spin: 1.75 },
  },
  {
    id: "vortex",
    name: "暗池引力",
    evoName: "奇点黑洞",
    partner: "momentum",
    color: "#a78bfa",
    desc: "在怪群中心展开暗池引力涡流，聚拢并持续研磨",
    evoDesc: "锁定最密集敌群展开巨型奇点，坍缩结束时引发超新星内爆",
    up: ["", "伤害 +2", "引力场更大更久", "引力场 +1", "伤害 +4，生成更勤"],
    lv: [
      { dmg: 6, cd: 3.9, n: 1, rad: 52, dur: 3.0 },
      { dmg: 8, cd: 3.9, n: 1, rad: 52, dur: 3.0 },
      { dmg: 8, cd: 3.9, n: 1, rad: 64, dur: 4.0 },
      { dmg: 8, cd: 3.9, n: 2, rad: 64, dur: 4.0 },
      { dmg: 12, cd: 2.95, n: 2, rad: 66, dur: 4.0 },
    ],
    evo: { dmg: 19, cd: 5.8, n: 1, rad: 134, dur: 5.0, burst: 98 },
  },
];

export const WEAPON_IDX = {};
WEAPONS.forEach((w, i) => {
  WEAPON_IDX[w.id] = i;
  w.k = i;
});

export const PASSIVES = [
  { id: "haste", name: "低延迟", max: 3, desc: "武器冷却 −10%", color: "#ffad36" },
  { id: "area", name: "广域覆盖", max: 3, desc: "攻击范围 +15%", color: "#5ce1e6" },
  { id: "margin", name: "安全边际", max: 3, desc: "生命 +28，护甲 +1", color: "#ffd97d" },
  { id: "sharpe", name: "夏普锐度", max: 3, desc: "暴击率 +10%，暴伤 +18%", color: "#6fffe9" },
  { id: "siphon", name: "数据虹吸", max: 3, desc: "拾取范围 +40%，经验 +6%", color: "#2ee59d" },
  { id: "leverage", name: "杠杆倍增", max: 3, desc: "全局伤害 +13%", color: "#ff6b6b" },
  { id: "compound", name: "长效复利", max: 3, desc: "持续 +16%，弹速 +12%", color: "#ffd166" },
  { id: "momentum", name: "动量推进", max: 3, desc: "移动速度 +9%", color: "#a78bfa" },
  { id: "dividend", name: "现金分红", max: 3, desc: "每秒回血 +0.7", color: "#fb7185" },
];

export const PASSIVE_IDX = {};
PASSIVES.forEach((p, i) => {
  PASSIVE_IDX[p.id] = i;
});

export const BOOSTS = [
  { id: "b_might", name: "复利滚雪球", desc: "全局伤害 +7%（可无限叠加）" },
  { id: "b_heal", name: "紧急注资", desc: "立即恢复 38% 最大生命" },
  { id: "b_core", name: "顺手套利", desc: "阿尔法晶核 +15" },
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
  { id: "glitch", name: "杂讯爬虫", hp: 8, sp: 49, dmg: 8, r: 10, xp: 1, wt: 1, ai: 0, from: 0, w: 10 },
  { id: "fomo", name: "追高飞蛾", hp: 5, sp: 94, dmg: 6, r: 8, xp: 1, wt: 0.8, ai: 1, from: 40, w: 5 },
  { id: "spam", name: "水文巨怪", hp: 42, sp: 34, dmg: 14, r: 16, xp: 5, wt: 3.5, ai: 0, from: 80, w: 2.2 },
  { id: "rumor", name: "谣言喷子", hp: 19, sp: 40, dmg: 8, r: 12, xp: 3, wt: 1.4, ai: 2, from: 140, w: 2.0 },
  { id: "crash", name: "闪崩蛮牛", hp: 23, sp: 51, dmg: 14, r: 12, xp: 3, wt: 1.5, ai: 3, from: 180, w: 2.4 },
  { id: "bubble_mob", name: "泡沫母体", hp: 27, sp: 52, dmg: 10, r: 14, xp: 2, wt: 1.6, ai: 0, from: 220, w: 1.8 },
  { id: "margin_bomb", name: "暴雷炸弹", hp: 21, sp: 60, dmg: 18, r: 12, xp: 3, wt: 1.2, ai: 0, from: 280, w: 1.5 },
  { id: "hallucination", name: "幻觉幽影", hp: 12, sp: 72, dmg: 9, r: 11, xp: 2, wt: 1, ai: 4, from: 1e9, w: 0 },
  { id: "mini", name: "碎裂子泡", hp: 7, sp: 84, dmg: 7, r: 8, xp: 1, wt: 0.7, ai: 0, from: 1e9, w: 0, hide: 1 },
  { id: "boss1", name: "伪研报之王", hp: 950, sp: 40, dmg: 18, r: 42, xp: 65, wt: 40, ai: 5, boss: 1, from: 1e9, w: 0 },
  { id: "boss2", name: "高频收割机", hp: 3400, sp: 46, dmg: 22, r: 44, xp: 95, wt: 40, ai: 5, boss: 2, from: 1e9, w: 0 },
  { id: "boss3", name: "幻觉魔眼主脑", hp: 8400, sp: 37, dmg: 24, r: 46, xp: 130, wt: 40, ai: 5, boss: 3, from: 1e9, w: 0 },
  { id: "boss4", name: "流动性黑天鹅", hp: 13200, sp: 43, dmg: 28, r: 50, xp: 175, wt: 40, ai: 5, boss: 4, from: 1e9, w: 0 },
];

export const ENEMY_IDX = {};
ENEMIES.forEach((e, i) => {
  ENEMY_IDX[e.id] = i;
  e.k = i;
});

export const BOSS_IDS = ["boss1", "boss2", "boss3", "boss4"];
export const CODEX_ENEMY_IDS = ["glitch", "fomo", "spam", "rumor", "crash", "bubble_mob", "margin_bomb", "hallucination"];

export const CURVES = {
  hpMul: (t) => {
    const m = t / 60;
    return (1 + 0.35 * m + 0.11 * m * m) * (m > 4 ? Math.pow(1.25, m - 4) : 1);
  },
  dmgMul: (t) => {
    const m = t / 60;
    return (1 + 0.09 * m) * (m > 4 ? Math.pow(1.09, m - 4) : 1);
  },
  spMul: (t) => Math.min(1.42, 1 + 0.03 * (t / 60)),
  target: (t) => Math.min(340, 10 + 0.92 * t),
  rate: (t) => 1.25 + t / 21.5,
  xpNeed: (lv) => Math.round(4 + 4.15 * (lv - 1) + 0.94 * Math.pow(lv - 1, 1.9)),
};

export const HEROES = [
  {
    id: "hone",
    name: "小砺",
    kind: "磨刃特工",
    weapon: "blade",
    trait: "基准阿尔法 · 算力经验 +12%",
    mod: { xpMul: 0.12 },
    unlock: null,
    price: 0,
  },
  {
    id: "rubin",
    name: "芯核",
    kind: "算力铁卫",
    weapon: "mine",
    trait: "液冷重装 · 生命 +45 · 受击释放超频震波",
    mod: { maxHp: 45, speed: -0.04, thorns: 1 },
    unlock: { stat: "kills", n: 1000, text: "累计净除 1000 个市场噪音" },
    price: 300,
  },
  {
    id: "photon",
    name: "光瞳",
    kind: "硅光斥候",
    weapon: "laser",
    trait: "光速洞察 · 拾取 +50% · 暴击 +12%",
    mod: { pickup: 0.5, crit: 0.12, maxHp: -15 },
    unlock: { stat: "bosses", n: 1, text: "累计击破 1 只深渊巨兽" },
    price: 400,
  },
  {
    id: "quant",
    name: "量闪",
    kind: "高频灵猫",
    weapon: "arc",
    trait: "极速因子 · 移速 +14% · 冷却 −10%",
    mod: { speed: 0.14, cd: -0.1 },
    unlock: { stat: "bestTime", n: 300, text: "单局坚守超过 5 分钟" },
    price: 600,
  },
];

export const HERO_IDX = {};
HEROES.forEach((h, i) => {
  HERO_IDX[h.id] = i;
});

// 3-Branch Talent Matrix (Offense / Defense / Economy) + Milestone Synergies
export const TALENT_BRANCHES = [
  {
    id: "offense",
    name: "火力因子",
    sub: "投入 6 点激活「破甲共鸣：终伤 +8%」",
    milestone: 6,
    bonusDesc: "终伤 +8%",
  },
  {
    id: "defense",
    name: "风控装甲",
    sub: "投入 6 点激活「冗余共鸣：护甲 +2」",
    milestone: 6,
    bonusDesc: "护甲 +2",
  },
  {
    id: "economy",
    name: "投研效率",
    sub: "投入 6 点激活「先发共鸣：开局秒升 Lv.2」",
    milestone: 6,
    bonusDesc: "开局秒升 Lv.2",
  },
];

export const TALENTS = [
  { id: "might", branch: "offense", icon: "leverage", name: "超频算力", max: 5, per: "全局伤害 +7%", cost: [60, 120, 220, 380, 600] },
  { id: "crit", branch: "offense", icon: "sharpe", name: "致命锐度", max: 5, per: "暴击 +3% · 暴伤 +6%", cost: [70, 140, 250, 400, 620] },
  { id: "haste", branch: "offense", icon: "haste", name: "极速时钟", max: 4, per: "武器冷却 −3%", cost: [90, 180, 320, 520] },
  { id: "hp", branch: "defense", icon: "margin", name: "冗余机柜", max: 5, per: "最大生命 +15", cost: [60, 120, 220, 380, 600] },
  { id: "speed", branch: "defense", icon: "momentum", name: "风控推进", max: 5, per: "移动速度 +3.5%", cost: [60, 120, 220, 380, 600] },
  { id: "revive", branch: "defense", icon: "dividend", name: "熔断保护", max: 1, per: "致命伤触发熔断半血重启 1 次", cost: [480] },
  { id: "pickup", branch: "economy", icon: "siphon", name: "全域雷达", max: 5, per: "拾取 +14% · 经验 +5%", cost: [60, 120, 220, 380, 600] },
  { id: "core", branch: "economy", icon: "core", name: "复利晶核", max: 5, per: "晶核收益 +12%", cost: [60, 120, 220, 380, 600] },
  { id: "reroll", branch: "economy", icon: "ui_retry", name: "因子重抽", max: 3, per: "每局重抽次数 +1（支持锁牌）", cost: [140, 280, 460] },
];

export function branchPoints(talents, branchId) {
  let sum = 0;
  for (const t of TALENTS) {
    if (t.branch === branchId) sum += talents[t.id] || 0;
  }
  return sum;
}

export const RULES = [
  { id: "surge", name: "非农狂潮", desc: "噪音密度 +40%，算力经验也 +40%", countMul: 1.4, xpMul: 1.4 },
  { id: "leverage_max", name: "满仓杠杆", desc: "全局伤害翻倍，最大生命减半", might: 2, hpMul: 0.5 },
  { id: "hft", name: "高频竞速", desc: "特工与噪音移速同时 +25%", speedAll: 1.25 },
  { id: "whale", name: "巨鲸异动", desc: "精英来袭快 3 倍，金库宝箱满地掉", eliteMul: 3 },
  { id: "radar", name: "全域虹吸", desc: "晶石拾取范围直接 ×3", pickupMul: 3 },
  { id: "fast_evo", name: "敏捷迭代", desc: "武器升至 Lv.3 即可超维进化", evoLv: 3 },
  { id: "conviction", name: "单吊重仓", desc: "限带 1 把主武器，但算力经验 +65%", slots: 1, xpMul: 1.65 },
];

export const TITLES = [
  { t: 0, name: "实习研究员" },
  { t: 60, name: "买方助理" },
  { t: 120, name: "算力猎手" },
  { t: 180, name: "首席分析师" },
  { t: 300, name: "多策略基金经理" },
  { t: 480, name: "全天候宏观掌门" },
  { t: 600, name: "硅谷阿尔法捕手" },
  { t: 720, name: "HONE 磨刃宗师" },
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

export const ACHIEVEMENTS = [
  { id: "first", name: "首次建仓", text: "完成第 1 次历险行动", stat: "runs", n: 1, cores: 60 },
  { id: "k200", name: "去伪存真", text: "单局净除 200 个市场噪音", run: "kills", n: 200, cores: 40 },
  { id: "k800", name: "降噪专家", text: "单局净除 800 个市场噪音", run: "kills", n: 800, cores: 80 },
  { id: "k2000", name: "算力清道夫", text: "单局净除 2000 个市场噪音", run: "kills", n: 2000, cores: 160 },
  { id: "t120", name: "穿越震荡", text: "单局坚守超过 2 分钟", run: "time", n: 120, cores: 40 },
  { id: "t300", name: "牛熊不惧", text: "单局坚守超过 5 分钟", run: "time", n: 300, cores: 80 },
  { id: "t480", name: "暗池常客", text: "单局坚守超过 8 分钟", run: "time", n: 480, cores: 160 },
  { id: "t720", name: "磨刃传说", text: "单局坚守超过 12 分钟", run: "time", n: 720, cores: 320 },
  { id: "b1", name: "戳穿伪研报", text: "击破「伪研报之王」", codex: "boss1", n: 1, cores: 60 },
  { id: "b2", name: "反收割大师", text: "击破「高频收割机」", codex: "boss2", n: 1, cores: 100 },
  { id: "b3", name: "零幻觉校准", text: "击破「幻觉魔眼主脑」", codex: "boss3", n: 1, cores: 160 },
  { id: "b4", name: "驯服黑天鹅", text: "击破「流动性黑天鹅」", codex: "boss4", n: 1, cores: 260 },
  { id: "evo1", name: "超维突破", text: "完成任意 1 把武器超武进化", stat: "evos", n: 1, cores: 50 },
  { id: "evo8", name: "全栈超武库", text: "解锁全部 8 种超武图鉴", evoAll: 1, n: 8, cores: 320 },
  { id: "c100", name: "丝滑连击", text: "单局最高连击达到 100", run: "maxCombo", n: 100, cores: 40 },
  { id: "c400", name: "高频印钞机", text: "单局最高连击达到 400", run: "maxCombo", n: 400, cores: 120 },
  { id: "lv20", name: "算力扩容", text: "单局升至 Lv.20", run: "level", n: 20, cores: 60 },
  { id: "lv35", name: "满载超算", text: "单局升至 Lv.35", run: "level", n: 35, cores: 160 },
  { id: "d1", name: "每日复盘", text: "完成 1 次每日行情挑战", stat: "dailyDone", n: 1, cores: 60 },
  { id: "k10k", name: "万噪归零", text: "生涯累计净除 10000 个噪音", stat: "kills", n: 10000, cores: 320 },
];

export const COMBO_STEPS = [25, 50, 100, 200, 300, 500, 800, 1000, 1500, 2000];
export const COMBO_CALLOUTS = [
  "信号确认！",
  "阿尔法涌现！",
  "一口气清场！",
  "高频收割！",
  "算力全开！",
  "磨刃锋芒！",
  "暗池震荡！",
  "史诗级净除！",
  "噪音闻风丧胆！",
  "HONE 无人能挡！",
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
  const seed = fnv1a("hone-adventure:" + key);
  const hero = HEROES[seed % HEROES.length];
  const rule = RULES[(seed >>> 5) % RULES.length];
  let weapon = WEAPONS[(seed >>> 11) % WEAPONS.length].id;
  if (rule.id === "conviction") {
    weapon = ["blade", "arc", "wafer", "pulse"][(seed >>> 17) % 4];
  }
  return { key, seed, char: hero.id, rule: rule.id, weapon };
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
