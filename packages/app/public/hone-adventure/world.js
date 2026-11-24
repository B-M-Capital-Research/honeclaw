// HONE Adventure V3 — Cleanroom Simulation, 10 Weapons, Active Skills, Interactive Sector Nodes, Synergies & Relics

import {
  AI_TYPE,
  BOOSTS,
  BOSS_IDS,
  BOSS_SEC,
  COMBO_STEPS,
  CURVES,
  DEFAULT_P_SLOTS,
  DEFAULT_W_SLOTS,
  DIFFICULTIES,
  DIFF_IDX,
  ELITE_AFFIXES,
  ENEMIES,
  ENEMY_IDX,
  HEROES,
  HERO_IDX,
  MAX_OVERCLOCK_LV,
  MAX_WEAPON_LV,
  PASSIVES,
  PASSIVE_IDX,
  RELICS,
  RULES,
  STAGES,
  STAGE_IDX,
  STAGE_SEC,
  SYNERGIES,
  TALENT_BRANCHES,
  WAVE_SEC,
  WEAPONS,
  WEAPON_IDX,
  branchPoints,
  getHeroMasteryBonuses,
  getPromotionBonuses,
  mulberry32,
} from "./data.js?v=3.0.0";

export const FX_EVT = {
  HIT: 1,
  KILL: 2,
  HURT: 3,
  SHOT: 4,
  GEM: 5,
  PICK: 6,
  LEVEL: 7,
  BOLT: 8,
  RING: 9,
  BOOM: 10,
  NUKE: 11,
  COMBO: 12,
  WAVE: 13,
  BOSSWARN: 14,
  BOSS: 15,
  BOSSDEAD: 16,
  ELITE: 17,
  DEAD: 18,
  REVIVE: 19,
  EVO: 20,
  CHEST: 21,
  ESHOT: 22,
  MAGNET: 23,
  DASH: 24,
  POP: 25,
  HEAL: 26,
  NOVA: 27,
  COMBOEND: 28,
  STAGE: 29,
  MATRIX: 30,
  SKILL: 31,
  NODE: 32,
  SHIELD: 33,
};

export const BULLET_KIND = {
  HBM: 0,
  PRISM_MAIN: 1,
  PRISM_SUB: 2,
  DRONE: 3,
  N2_CANISTER: 4,
};

export const ZONE_KIND = {
  TENSOR: 0,
  CRYO: 1,
  PULSE: 2,
  BLAST: 3,
  EBEAM: 4,
  NVLINK: 5,
  FROST_RAIL: 6,
  NODE: 7,
  SKYBEAM: 8,
};

export const PICK_KIND = { HEART: 0, MAGNET: 1, NUKE: 2, CORE: 3, CHEST: 4 };

const MAX_ENEMIES = 460;
const MAX_BULLETS = 420;
const MAX_EBULLETS = 260;
const MAX_GEMS = 440;
const MAX_ZONES = 148;
const MAX_PICKS = 44;
const CELL_SIZE = 40;
const GRID_W = 72;
const GRID_H = 72;
const TAU = Math.PI * 2;

const makePool = (n, fn) => {
  const a = new Array(n);
  for (let i = 0; i < n; i++) a[i] = fn();
  return a;
};

export class GameWorld {
  constructor(opts = {}) {
    this.rnd = mulberry32(opts.seed == null ? (Math.random() * 1e9) | 0 : opts.seed);
    this.fx = opts.fx || null;
    this.vw = opts.vw || 420;
    this.vh = opts.vh || 900;
    this.char = HEROES[HERO_IDX[opts.char || "hone"]] || HEROES[0];
    this.masteryLv = Math.max(1, Math.min(5, opts.masteryLv || 1));
    this.mastery = getHeroMasteryBonuses(this.masteryLv);
    this.diff = DIFFICULTIES[DIFF_IDX[opts.diff || "std"]] || DIFFICULTIES[0];
    this.routeMode = opts.routeMode === "lock" ? "lock" : "auto";
    this.talents = opts.talents || {};
    this.promo = getPromotionBonuses(opts.bestTime || 0);
    this.rule = (opts.rule && RULES.find((r) => r.id === opts.rule)) || {};
    this.daily = !!opts.daily;
    this.autoLevel = !opts.manualLevel;
    this.t = 0;
    this.over = false;
    this.pending = null;
    this.lockedChoice = null;
    this.uid = 0;

    this.startStageIdx = STAGE_IDX[opts.stage || "storage"] ?? 0;
    this.stageIdx = this.startStageIdx;
    this.stageStep = 0;
    this.jumpReady = false;
    this.jumpCd = STAGE_SEC;
    this.jumpBuffT = 0;
    this.stagesVisited = [STAGES[this.stageIdx].id];

    this.en = makePool(MAX_ENEMIES, () => ({
      x: 0, y: 0, kx: 0, ky: 0, hp: 0, maxHp: 0, type: 0, ai: 0, r: 10, sp: 0, dmg: 0,
      xpv: 1, wt: 1, flash: 0, slow: 0, vuln: 0, affix: -1, elite: 0, boss: 0, uid: 0, st: 0, stT: 0,
      a1: 0, a2: 0, ax: 0, ay: 0, ph: 0, pulse: 0, mark: 0, dead: false, gone: false,
      tele: 0, born: 0, swell: 0, face: 1, mvx: 0, mvy: 0, cd: new Float32Array(12),
    }));
    this.nE = 0;

    this.bl = makePool(MAX_BULLETS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, r: 6, dmg: 0, kind: 0, k: 0, life: 0,
      pierce: 0, st: 0, rot: 0, size: 1, pop: 0, split: 0, wl: 0, lane: 0, t: 0,
      tx: 0, ty: 0, rays: 4, rayLen: 100, hits: [],
    }));
    this.nB = 0;

    this.eb = makePool(MAX_EBULLETS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, r: 6, dmg: 0, life: 0, src: 0, sq: 0,
    }));
    this.nEB = 0;

    this.gm = makePool(MAX_GEMS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, v: 1, tier: 0, st: 0, sp: 0, t: 0,
    }));
    this.nG = 0;

    this.zn = makePool(MAX_ZONES, () => ({
      kind: 0, x: 0, y: 0, ox: 0, oy: 0, r: 0, rMax: 0, t: 0, life: 0, arm: 0,
      dmg: 0, tickT: 0, evo: 0, mini: 0, id: 0, a: 0, av: 0, len: 0, st: 0,
      own: null, src: 0, slow: 0, burst: 0, k: 0, prog: 0, stageIdx: 0,
      pts: new Float32Array(20), nPts: 0,
    }));
    this.nZ = 0;

    this.pk = makePool(MAX_PICKS, () => ({ x: 0, y: 0, kind: 0, t: 0, n: 0 }));
    this.nPK = 0;

    this.gStart = new Int32Array(GRID_W * GRID_H + 1);
    this.gCur = new Int32Array(GRID_W * GRID_H);
    this.gItems = new Int32Array(MAX_ENEMIES);
    this.gCell = new Int32Array(MAX_ENEMIES);
    this.gox = 0;
    this.goy = 0;
    this.q = new Int32Array(MAX_ENEMIES);

    const p = (this.p = {
      x: 0, y: 0, r: 13, hp: 100, maxHp: 0, face: 1, aimX: 1, aimY: 0, mx: 0, my: 0,
      inv: 0, shields: 0, shieldCd: 15, lvl: 1, xp: 0, need: CURVES.xpNeed(1),
      weapons: [], passives: [], boosts: 0, corePickBoost: 0,
      st: {}, moving: false, hurtT: 0, strideAcc: 0, hbmCounter: 0,
    });

    this.synergies = {};
    this.synergyList = [];
    this.relics = {};
    this.relicList = [];

    this.skillCd = 0;
    this.skillMaxCd = this.char.skillCd || 14;
    this.skillBuffT = 0;
    this.skillPostT = 0;
    this.skyTickT = 0;

    this.nodeTimer = 15;
    this.nodesCaptured = 0;
    this.nodeHasteT = 0;
    this.nodeCdT = 0;

    this.kills = 0;
    this.siphonKills = 0;
    this.killsBy = new Int32Array(ENEMIES.length);
    this.dmgBy = new Float64Array(WEAPONS.length);
    this.combo = 0;
    this.comboT = 0;
    this.maxCombo = 0;
    this.comboIdx = 0;
    this.coresPicked = this.mastery.startCores || 0;
    this.elites = 0;
    this.bosses = 0;
    this.bossList = [];
    this.wave = 0;
    this.spawnAcc = 0;
    this.nextElite = 46;
    this.nextBoss = BOSS_SEC * (this.rule.bossFast || 1);
    this.bossIdx = 0;
    this.bossWarned = false;
    this.bossWarnType = 0;
    this.boss = null;
    this.pendLv = 0;
    this.pulseId = 0;
    this.stamp = 0;
    this.magnetT = 0;
    this.lastMagnet = -20;
    this.lastNuke = -30;
    this.overXp = 0;
    this.revives = this.talents.revive ? 1 : 0;
    this.rerolls =
      1 +
      (this.talents.reroll || 0) +
      (this.promo.reroll || 0) +
      (this.char.mod.bonusReroll || 0) +
      (this.mastery.reroll || 0);
    this.wipeEB = false;
    this.lastSrc = -1;
    this.hurts = 0;
    this.evos = [];
    this.maxOverclock = 0;
    this.near = null;
    this.nearD = 1e9;
    this.sx = 0;
    this.sy = 0;
    this.hpMulNow = 1;
    this.dmgMulNow = 1;
    this.spMulNow = 1;
    this.ghosts = 0;

    this.recalc();
    p.hp = p.maxHp;

    // Initial Energy Shields (Thermal 10pt Apex / Promotion L12 / Glass Rule)
    const thermalApex = branchPoints(this.talents, "thermal") >= TALENT_BRANCHES[2].apexMilestone ? 1 : 0;
    p.shields = Math.min(3, thermalApex + (this.promo.startShield || 0) + (this.rule.startShield || 0));

    // Starter Weapon (Supports Custom Loadout when Agent Mastery >= Lv.2!)
    const startW = opts.weapon || this.char.weapon;
    this.addWeapon(startW);
    if (this.rule.dualStart) {
      const altIdx = (WEAPON_IDX[startW] + 1 + ((this.rnd() * (WEAPONS.length - 1)) | 0)) % WEAPONS.length;
      this.addWeapon(WEAPONS[altIdx].id);
    }

    // Cluster Orchestration milestone (>=6 pts) OR Astra in Home Sector: instant Lv.2 headstart
    const homeAstra = this.char.id === "astra" && this.isHomeStage();
    if ((branchPoints(this.talents, "cluster") >= TALENT_BRANCHES[3].milestone || homeAstra) && !this.daily) {
      p.lvl++;
      p.need = CURVES.xpNeed(p.lvl);
      this.pendLv++;
    }

    const initType = STAGES[this.stageIdx].bias[0] ?? ENEMY_IDX.packet;
    for (let i = 0; i < 7; i++) {
      const ang = ((i + this.rnd() * 0.6) / 7) * TAU;
      const dist = 155 + this.rnd() * 80;
      this.spawnEnemy(i % 2 === 0 ? ENEMY_IDX.packet : initType, Math.cos(ang) * dist, Math.sin(ang) * dist, 0);
    }
  }

  stage() {
    return STAGES[this.stageIdx];
  }

  isHomeStage(idx = this.stageIdx) {
    const stId = STAGES[idx]?.id;
    return !!(this.char.homeStages && this.char.homeStages.indexOf(stId) >= 0);
  }

  setStage(stageIdOrIdx) {
    const idx = typeof stageIdOrIdx === "number" ? stageIdOrIdx : STAGE_IDX[stageIdOrIdx];
    if (idx == null || idx < 0 || idx >= STAGES.length) return false;
    this.stageIdx = idx;
    const id = STAGES[idx].id;
    if (this.stagesVisited.indexOf(id) < 0) this.stagesVisited.push(id);
    this.recalc();
    return true;
  }

  // Player-Controlled Tactical Sector Jump!
  jumpStage(stageIdOrIdx) {
    const idx = typeof stageIdOrIdx === "number" ? stageIdOrIdx : STAGE_IDX[stageIdOrIdx];
    if (idx == null || idx < 0 || idx >= STAGES.length) return false;
    this.setStage(idx);
    this.jumpReady = false;
    this.jumpCd = STAGE_SEC;
    this.jumpBuffT = 10;
    this.magnetT = Math.max(this.magnetT, 0.75);
    this.recalc();
    this.fx && this.fx(FX_EVT.STAGE, idx);
    return true;
  }

  plv(id) {
    for (let i = 0; i < this.p.passives.length; i++) {
      if (this.p.passives[i].id === id) return this.p.passives[i].lv;
    }
    return 0;
  }

  hasSynergy(id) {
    return !!this.synergies[id];
  }

  hasRelic(id) {
    return !!this.relics[id];
  }

  recalc() {
    const p = this.p;
    const st = p.st;
    const mod = this.char.mod || {};
    const mas = this.mastery;
    const tal = this.talents;
    const prm = this.promo;
    const rule = this.rule;
    const smod = STAGES[this.stageIdx].mod || {};
    const stageAmp = mod.stageBoost ? 2.0 : 1.0;
    const inHome = this.isHomeStage();

    const interconnectSyn = branchPoints(tal, "interconnect") >= TALENT_BRANCHES[0].milestone ? 1.1 : 1;
    const interconnectApex = branchPoints(tal, "interconnect") >= TALENT_BRANCHES[0].apexMilestone ? 1 : 0;
    const lithoSyn = branchPoints(tal, "litho") >= TALENT_BRANCHES[1].milestone ? 0.07 : 0;
    const lithoApex = branchPoints(tal, "litho") >= TALENT_BRANCHES[1].apexMilestone ? 1.25 : 1;
    const thermalSyn = branchPoints(tal, "thermal") >= TALENT_BRANCHES[2].milestone ? 2 : 0;

    st.maxHp = Math.round(
      (100 +
        18 * (tal.hp || 0) +
        32 * this.plv("ecc") +
        (mod.maxHp || 0) +
        (mas.maxHp || 0) +
        (prm.maxHp || 0)) *
        (rule.hpMul || 1),
    );
    if (st.maxHp > p.maxHp && p.maxHp > 0) p.hp += st.maxHp - p.maxHp;
    p.maxHp = st.maxHp;
    if (p.hp > p.maxHp) p.hp = p.maxHp;

    const hpScaleMight = mod.hpToMight ? Math.max(0, (st.maxHp - 100) / 40) * 0.04 : 0;
    const evoScaleMight = mod.evoMight ? this.evos.length * mod.evoMight : 0;
    const compoundBonus = this.hasRelic("r_compound") ? p.lvl * 0.022 : 0;
    const jumpBonus = this.jumpBuffT > 0 ? 0.22 : 0;
    const postSkillBonus = this.skillPostT > 0 ? 0.28 : 0;

    st.might =
      (1 +
        0.07 * (tal.might || 0) +
        0.14 * this.plv("overvolt") +
        0.1 * p.boosts +
        (prm.might || 0) +
        (mas.might || 0) +
        (smod.might || 0) * stageAmp +
        hpScaleMight +
        evoScaleMight +
        compoundBonus +
        jumpBonus +
        postSkillBonus) *
      interconnectSyn *
      (rule.might || 1);

    st.bossDmgMul = 1 + 0.1 * (tal.boss_dmg || 0) + 0.12 * this.plv("power_rail");
    st.armor = this.plv("ecc") + thermalSyn + (mod.armor || 0) + (prm.armor || 0);
    st.speed =
      158 *
      (1 +
        0.04 * (tal.speed || 0) +
        0.11 * this.plv("fabric") +
        (mod.speed || 0) +
        (this.jumpBuffT > 0 ? 0.18 : 0) +
        (this.char.id === "quant" && this.skillBuffT > 0 ? 0.32 : 0)) *
      (rule.speedAll || 1);
    st.pickup =
      68 *
      (1 +
        0.16 * (tal.pickup || 0) +
        0.42 * this.plv("reticle") +
        (mod.pickup || 0) +
        (mas.pickup || 0) +
        (prm.pickup || 0) +
        0.1 * p.corePickBoost) *
      (rule.pickupMul || 1);
    st.xpMul =
      (1 +
        0.05 * (tal.pickup || 0) +
        0.08 * this.plv("reticle") +
        (mod.xpMul || 0) +
        (prm.xpMul || 0) +
        (smod.xpMul || 0) * stageAmp) *
      (this.diff.xpMul || 1) *
      (rule.xpMul || 1);

    const homeHaste =
      (this.char.id === "hone" && inHome ? -0.1 : 0) +
      (this.char.id === "quant" && inHome ? -0.06 : 0);
    const activeHaste =
      (this.nodeHasteT > 0 ? -0.25 : 0) +
      (this.nodeCdT > 0 ? -0.28 : 0) +
      (this.char.id === "photon" && this.skillBuffT > 0 ? -0.3 : 0) +
      (this.char.id === "hone" && this.skillBuffT > 0 ? -0.18 : 0);

    st.cd = Math.max(
      0.28,
      1 -
        0.035 * (tal.haste || 0) -
        0.1 * this.plv("clock") +
        (mod.cd || 0) +
        (prm.cd || 0) +
        (smod.cdMul || 0) * stageAmp +
        homeHaste +
        activeHaste,
    );
    st.area = 1 + 0.06 * (tal.area || 0) + 0.15 * this.plv("waveguide") + (mod.hpToMight ? 0.08 : 0);
    st.crit =
      0.06 +
      0.03 * (tal.crit || 0) +
      0.1 * this.plv("fp8") +
      lithoSyn +
      (mod.crit || 0) +
      (mas.crit || 0) +
      (prm.crit || 0) +
      (this.nodeHasteT > 0 ? 0.25 : 0) +
      (this.char.id === "photon" && this.skillBuffT > 0 ? 0.35 : 0);
    st.critMul =
      2.0 +
      0.08 * (tal.crit || 0) +
      0.22 * this.plv("fp8") +
      (prm.critDmg || 0) +
      (smod.critDmg || 0) * stageAmp +
      (this.char.id === "photon" && inHome ? 0.25 : 0);
    st.dur = 1 + 0.07 * (tal.proj || 0) + 0.18 * this.plv("manifold");
    st.pspd =
      1 +
      0.07 * (tal.proj || 0) +
      0.15 * this.plv("manifold") +
      (mod.pspd || 0) +
      (smod.projSpeed || 0) * stageAmp;
    st.regen =
      0.4 * (tal.regen || 0) +
      1.0 * this.plv("ups") +
      (this.char.id === "rubin" && inHome ? 1.2 : 0);
    st.coreMul =
      (1 +
        0.14 * (tal.core || 0) +
        (prm.coreBonus || 0) +
        (smod.coreBonus || 0) * stageAmp +
        compoundBonus) *
      (this.diff.coreMul || 1);
    st.thorns = mod.thorns || 0;
    st.pierceBonus = interconnectApex;
    st.ocScale = lithoApex;
    st.multishot = this.hasRelic("r_multishot") ? 1 : 0;

    const skillCdr =
      (1 - 0.08 * (tal.skill_cdr || 0) - 0.12 * this.plv("power_rail") + (prm.skillCd || 0)) *
      (mas.skillCdMul || 1) *
      (this.hasRelic("r_skill_cd") ? 0.68 : 1) *
      (rule.skillCdMul || 1);
    this.skillMaxCd = Math.max(3.5, (this.char.skillCd || 14) * Math.max(0.35, skillCdr));
  }

  // Hero Active Tactical Skill!
  castSkill() {
    if (this.over || this.pending || this.skillCd > 0) return false;
    const p = this.p;
    const st = p.st;
    this.skillCd = this.skillMaxCd;
    if (this.hasRelic("r_skill_cd")) this.skillPostT = 4.0;
    if (this.rule.id === "skill_frenzy") this.coresPicked += 4;
    this.fx && this.fx(FX_EVT.SKILL, this.char.id, p.x, p.y);

    switch (this.char.id) {
      case "hone": {
        // Full-band dispatch pulse: clear enemy bullets, stun/knockback foes, pull all nearby XP!
        this.wipeEB = true;
        this.magnetT = Math.max(this.magnetT, 1.1);
        this.skillBuffT = 4.0;
        this.pulse(0, 36, 225 * st.area, 1.4, 480);
        break;
      }
      case "rubin": {
        // Superconducting cryo-fortress: +2 shields, freeze aura, HP-scaled frost nova!
        p.shields = Math.min(3, p.shields + 2);
        p.inv = Math.max(p.inv, 3.5);
        this.skillBuffT = 3.5;
        const burstDmg = 34 + p.maxHp * 0.35;
        this.pulse(2, burstDmg, 195 * st.area, 2.2, 420);
        break;
      }
      case "photon": {
        // Coherent laser barrage: 4s high-crit haste + 8 piercing photon lances!
        this.skillBuffT = 4.0;
        const sp = 520 * st.pspd;
        for (let n = 0; n < 8; n++) {
          const tgt = n === 0 && this.near ? this.near : this.randNear(380);
          const a = tgt ? Math.atan2(tgt.y - p.y, tgt.x - p.x) + (n - 3.5) * 0.06 : (n / 8) * TAU;
          const b = this.addBullet(
            BULLET_KIND.PRISM_MAIN,
            WEAPON_IDX.prism,
            p.x,
            p.y,
            Math.cos(a) * sp,
            Math.sin(a) * sp,
            10,
            42,
            2,
            1.2,
          );
          if (b) {
            b.split = 3;
            b.size = 1.25;
          }
        }
        break;
      }
      case "quant": {
        // Nanosecond phase blink: dash forward invulnerably, dual lightning explosions!
        const ox = p.x;
        const oy = p.y;
        const dx = p.moving ? p.mx : p.aimX;
        const dy = p.moving ? p.my : p.aimY;
        const len = Math.sqrt(dx * dx + dy * dy) || 1;
        const ux = dx / len;
        const uy = dy / len;
        this.explode(ox, oy, 88 * st.area, 38, WEAPON_IDX.bus, 220, 0);
        p.x += ux * 148;
        p.y += uy * 148;
        p.inv = Math.max(p.inv, 1.1);
        this.skillBuffT = 3.5;
        this.fx && this.fx(FX_EVT.BOLT, ox, oy, p.x, p.y, 1);
        this.explode(p.x, p.y, 108 * st.area, 52, WEAPON_IDX.bus, 280, 0);
        break;
      }
      case "astra": {
        // Sovereign sky-compute descent: 4.5s slow field + immediate & periodic orbital beams!
        this.skillBuffT = 4.5;
        this.skyTickT = 0.05;
        for (let i = 0; i < this.nE; i++) {
          if (!this.en[i].dead) this.en[i].slow = Math.max(this.en[i].slow, 2.5);
        }
        break;
      }
      default:
        break;
    }
    this.recalc();
    return true;
  }

  spawnSkyBeam() {
    const p = this.p;
    let best = null;
    let bestHp = -1;
    for (let i = 0; i < this.nE; i++) {
      const e = this.en[i];
      if (e.dead) continue;
      const dx = e.x - p.x;
      const dy = e.y - p.y;
      if (dx * dx + dy * dy > 360 * 360) continue;
      if (e.hp > bestHp) {
        bestHp = e.hp;
        best = e;
      }
    }
    const tx = best ? best.x : p.x + (this.rnd() - 0.5) * 260;
    const ty = best ? best.y : p.y + (this.rnd() - 0.5) * 260;
    const z = this.addZone();
    if (z) {
      z.kind = ZONE_KIND.SKYBEAM;
      z.x = tx;
      z.y = ty;
      z.r = 68 * p.st.area;
      z.life = 0.36;
    }
    this.explode(tx, ty, 68 * p.st.area, 56, WEAPON_IDX.nvlink, 190, 0);
  }

  // Interactive Cleanroom Sector Node!
  spawnSectorNode() {
    const p = this.p;
    // Keep at most 2 active nodes on screen
    let activeNodes = 0;
    for (let i = 0; i < this.nZ; i++) if (this.zn[i].kind === ZONE_KIND.NODE) activeNodes++;
    if (activeNodes >= 2) return null;

    const z = this.addZone();
    if (!z) return null;
    const ang = this.rnd() * TAU;
    const dist = 145 + this.rnd() * 95;
    z.kind = ZONE_KIND.NODE;
    z.x = p.x + Math.cos(ang) * dist;
    z.y = p.y + Math.sin(ang) * dist;
    z.r = 54;
    z.prog = 0;
    z.life = 26;
    z.stageIdx = this.stageIdx;
    return z;
  }

  triggerNodeReward(z) {
    const p = this.p;
    const stg = STAGES[z.stageIdx] || STAGES[this.stageIdx];
    const amp = 1 + 0.2 * (this.talents.node_boost || 0) + (this.rule.nodeFast ? 0.25 : 0);
    this.nodesCaptured++;
    this.fx && this.fx(FX_EVT.NODE, z.x, z.y, z.stageIdx);

    switch (stg.nodeKind) {
      case "cache_drop": {
        const cnt = Math.round(12 * amp);
        for (let i = 0; i < cnt; i++) this.addGem(z.x + (this.rnd() - 0.5) * 36, z.y + (this.rnd() - 0.5) * 36, 6);
        this.coresPicked += Math.round(12 * amp);
        this.magnetT = Math.max(this.magnetT, 0.85);
        break;
      }
      case "optic_haste":
        this.nodeHasteT = 8 * amp;
        this.recalc();
        break;
      case "cpu_overclock":
        this.skillCd = 0;
        this.nodeCdT = 6 * amp;
        this.recalc();
        break;
      case "gpu_barrage":
        for (let i = 0; i < 6; i++) {
          const tgt = this.randNear(320);
          const bz = this.addZone();
          if (!bz) break;
          bz.kind = ZONE_KIND.TENSOR;
          bz.k = WEAPON_IDX.tensor;
          bz.x = tgt ? tgt.x : z.x + Math.cos((i / 6) * TAU) * 120;
          bz.y = tgt ? tgt.y : z.y + Math.sin((i / 6) * TAU) * 120;
          bz.arm = 0.25 + i * 0.08;
          bz.life = bz.arm + 0.08;
          bz.r = 82 * p.st.area;
          bz.dmg = 64 * amp;
        }
        break;
      case "wafer_heal":
        p.hp = Math.min(p.maxHp, p.hp + Math.round(p.maxHp * 0.3 * amp));
        p.shields = Math.min(3, p.shields + 1);
        this.fx && this.fx(FX_EVT.HEAL, p.x, p.y);
        break;
      case "gw_surge":
      default: {
        let hits = 0;
        for (let i = 0; i < this.nE && hits < 12; i++) {
          const e = this.en[i];
          if (e.dead) continue;
          this.fx && this.fx(FX_EVT.BOLT, z.x, z.y, e.x, e.y, 1);
          this.hit(e, 58 * amp, WEAPON_IDX.bus, 150, 0, 0);
          hits++;
        }
        this.magnetT = Math.max(this.magnetT, 1.2);
        break;
      }
    }
  }

  hasWeapon(id) {
    for (let i = 0; i < this.p.weapons.length; i++) {
      if (this.p.weapons[i].id === id) return this.p.weapons[i];
    }
    return null;
  }

  addWeapon(id) {
    const k = WEAPON_IDX[id] ?? 0;
    const realId = WEAPONS[k].id;
    const existing = this.hasWeapon(realId);
    if (existing) {
      existing.lv = Math.min(MAX_WEAPON_LV, existing.lv + 1);
      return existing;
    }
    const w = {
      id: realId, k, lv: 1, evo: false, oc: 0, t: 0.2, ang: this.rnd() * TAU, ph: 0, on: 1, onT: 0,
      vis: 1, bt: 0, tick: 0, extra: 0, extraT: 0, waveT: 2.2, pos: new Float32Array(32), np: 0,
    };
    this.p.weapons.push(w);
    return w;
  }

  wslots() {
    return (this.rule.slots || DEFAULT_W_SLOTS) + (this.hasRelic("r_slots") ? 1 : 0);
  }

  pslots() {
    return DEFAULT_P_SLOTS + (this.char.mod.extraPSlot || 0) + (this.hasRelic("r_slots") ? 1 : 0);
  }

  evoLv() {
    return this.rule.evoLv || MAX_WEAPON_LV;
  }

  canEvolve(w) {
    return !w.evo && w.lv >= this.evoLv() && this.plv(WEAPONS[w.k].partner) > 0;
  }

  wDmgMul(w) {
    let m = 1;
    if (w.oc > 0) m *= 1 + w.oc * 0.2 * (this.p.st.ocScale || 1);
    if ((w.id === "hbm" || w.id === "bus") && this.hasSynergy("syn_bus_hbm")) m *= 1.18;
    if ((w.id === "cache" || w.id === "drone") && this.hasSynergy("syn_cache_drone")) m *= 1.24;
    if ((w.id === "liquid_n2" || w.id === "prism") && this.hasSynergy("syn_n2_prism")) m *= 1.2;
    return m;
  }

  buildPool() {
    const p = this.p;
    const pool = [];
    const canAddW = p.weapons.length < this.wslots();
    const canAddP = p.passives.length < this.pslots();
    const synWeightAmp = 1 + 0.35 * (this.talents.syn_rate || 0);

    // 1. Weapons & Post-Evolution Super Overclocks (★+1 ~ ★+5)
    for (let i = 0; i < WEAPONS.length; i++) {
      const def = WEAPONS[i];
      const cur = this.hasWeapon(def.id);
      if (cur) {
        if (!cur.evo && cur.lv < MAX_WEAPON_LV) {
          pool.push({ kind: "w", id: def.id, lv: cur.lv + 1, isNew: false, wt: 3.2 });
        } else if (cur.evo && cur.oc < MAX_OVERCLOCK_LV) {
          pool.push({ kind: "oc", id: def.id, lv: cur.oc + 1, isNew: false, wt: 2.3 });
        }
      } else if (canAddW) {
        pool.push({ kind: "w", id: def.id, lv: 1, isNew: true, wt: this.plv(def.partner) > 0 ? 2.7 : 1.75 });
      }
    }

    // 2. Passive Modules
    for (let i = 0; i < PASSIVES.length; i++) {
      const def = PASSIVES[i];
      const lv = this.plv(def.id);
      let isPartner = false;
      for (let j = 0; j < p.weapons.length; j++) {
        if (WEAPONS[p.weapons[j].k].partner === def.id && !p.weapons[j].evo) isPartner = true;
      }
      if (lv > 0) {
        if (lv < def.max) pool.push({ kind: "p", id: def.id, lv: lv + 1, isNew: false, wt: 2.1 });
      } else if (canAddP) {
        pool.push({ kind: "p", id: def.id, lv: 1, isNew: true, wt: isPartner ? 3.5 : 1.55 });
      }
    }

    // 3. Cross-Weapon Synergy Protocols (Available as soon as both weapons are equipped!)
    for (let i = 0; i < SYNERGIES.length; i++) {
      const syn = SYNERGIES[i];
      if (this.hasSynergy(syn.id)) continue;
      if (this.hasWeapon(syn.req[0]) && this.hasWeapon(syn.req[1])) {
        pool.push({ kind: "syn", id: syn.id, lv: 1, isNew: true, wt: 4.2 * synWeightAmp });
      }
    }

    // 4. Late-Game Mythic Relics (Lv.10+, or after 1st Super Evolution, or Relic Daily!)
    if (p.lvl >= 10 || this.evos.length > 0 || (this.rule.earlyRelic && p.lvl >= 3)) {
      for (let i = 0; i < RELICS.length; i++) {
        const rel = RELICS[i];
        if (!this.hasRelic(rel.id)) {
          pool.push({ kind: "relic", id: rel.id, lv: 1, isNew: true, wt: 2.2 * synWeightAmp });
        }
      }
    }
    return pool;
  }

  isChoiceStillValid(c) {
    if (!c) return false;
    if (c.kind === "evo") {
      const w = this.hasWeapon(c.id);
      return !!(w && this.canEvolve(w));
    }
    if (c.kind === "oc") {
      const w = this.hasWeapon(c.id);
      return !!(w && w.evo && w.oc < MAX_OVERCLOCK_LV);
    }
    if (c.kind === "syn") return !this.hasSynergy(c.id);
    if (c.kind === "relic") return !this.hasRelic(c.id);
    if (c.kind === "w") {
      const w = this.hasWeapon(c.id);
      if (w) return !w.evo && w.lv < MAX_WEAPON_LV;
      return this.p.weapons.length < this.wslots();
    }
    if (c.kind === "p") {
      const lv = this.plv(c.id);
      if (lv > 0) return lv < PASSIVES[PASSIVE_IDX[c.id]].max;
      return this.p.passives.length < this.pslots();
    }
    return true;
  }

  refreshChoice(c) {
    if (c.kind === "w") {
      const w = this.hasWeapon(c.id);
      return { ...c, lv: w ? w.lv + 1 : 1, isNew: !w };
    }
    if (c.kind === "oc") {
      const w = this.hasWeapon(c.id);
      return { ...c, lv: w ? w.oc + 1 : 1, isNew: false };
    }
    if (c.kind === "p") {
      const lv = this.plv(c.id);
      return { ...c, lv: lv + 1, isNew: lv === 0 };
    }
    return { ...c };
  }

  genChoices(count = 3, preserveLocked = true) {
    const out = [];
    const p = this.p;

    if (preserveLocked && this.lockedChoice && this.isChoiceStillValid(this.lockedChoice)) {
      out.push({ ...this.refreshChoice(this.lockedChoice), locked: true });
    } else if (preserveLocked) {
      this.lockedChoice = null;
    }

    for (let i = 0; i < p.weapons.length && out.length < 2; i++) {
      if (this.canEvolve(p.weapons[i]) && !out.some((x) => x.kind === "evo" && x.id === p.weapons[i].id)) {
        out.push({ kind: "evo", id: p.weapons[i].id, lv: 0, isNew: true });
      }
    }

    const pool = this.buildPool().filter(
      (item) => !out.some((o) => o.kind === item.kind && o.id === item.id),
    );

    while (out.length < count && pool.length) {
      let sum = 0;
      for (let i = 0; i < pool.length; i++) sum += pool[i].wt;
      let pick = this.rnd() * sum;
      let idx = 0;
      for (let i = 0; i < pool.length; i++) {
        pick -= pool[i].wt;
        if (pick <= 0) {
          idx = i;
          break;
        }
      }
      out.push(pool[idx]);
      pool.splice(idx, 1);
    }

    for (let i = 0; out.length < count; i++) {
      out.push({ kind: "boost", id: BOOSTS[i % BOOSTS.length].id, lv: 0, isNew: false });
    }
    return out;
  }

  toggleLockChoice(idx) {
    const pend = this.pending;
    if (!pend || pend.type !== "level" || !pend.choices[idx]) return false;
    const target = pend.choices[idx];
    if (target.locked) {
      target.locked = false;
      this.lockedChoice = null;
    } else {
      for (const c of pend.choices) c.locked = false;
      target.locked = true;
      this.lockedChoice = { ...target };
    }
    return true;
  }

  apply(c) {
    const p = this.p;
    if (c.kind === "w") {
      const w = this.hasWeapon(c.id);
      if (w) w.lv = Math.min(MAX_WEAPON_LV, w.lv + 1);
      else this.addWeapon(c.id);
    } else if (c.kind === "oc") {
      const w = this.hasWeapon(c.id);
      if (w && w.evo) {
        w.oc = Math.min(MAX_OVERCLOCK_LV, w.oc + 1);
        if (w.oc > this.maxOverclock) this.maxOverclock = w.oc;
      }
    } else if (c.kind === "p") {
      let cur = null;
      for (let i = 0; i < p.passives.length; i++) {
        if (p.passives[i].id === c.id) cur = p.passives[i];
      }
      if (cur) cur.lv = Math.min(PASSIVES[PASSIVE_IDX[c.id]].max, cur.lv + 1);
      else p.passives.push({ id: c.id, lv: 1 });
      this.recalc();
    } else if (c.kind === "evo") {
      const w = this.hasWeapon(c.id);
      if (w) {
        w.evo = true;
        w.t = 0;
        w.on = 1;
        this.evos.push(c.id);
        this.recalc();
        this.fx && this.fx(FX_EVT.EVO, w.k, p.x, p.y);
      }
    } else if (c.kind === "syn") {
      if (!this.synergies[c.id]) {
        this.synergies[c.id] = 1;
        this.synergyList.push(c.id);
        this.recalc();
      }
    } else if (c.kind === "relic") {
      if (!this.relics[c.id]) {
        this.relics[c.id] = 1;
        this.relicList.push(c.id);
        if (c.id === "r_shield") p.shields = Math.min(3, p.shields + 1);
        this.recalc();
      }
    } else if (c.id === "b_might") {
      p.boosts++;
      this.skillCd = Math.max(0, this.skillCd - 2.5);
      this.recalc();
    } else if (c.id === "b_heal") {
      p.hp = Math.min(p.maxHp, p.hp + p.maxHp * 0.45);
      p.inv = Math.max(p.inv, 2.0);
      this.fx && this.fx(FX_EVT.HEAL, p.x, p.y);
    } else if (c.id === "b_core") {
      this.coresPicked += 24;
      p.corePickBoost++;
      this.recalc();
    }
  }

  openLevel() {
    this.pending = {
      type: "level",
      choices: this.genChoices(3, true),
      lvl: this.p.lvl - this.pendLv + 1,
    };
    this.fx && this.fx(FX_EVT.LEVEL, this.pending.lvl);
  }

  choose(idx) {
    const pend = this.pending;
    if (!pend || pend.type !== "level") return;
    const picked = pend.choices[idx] || pend.choices[0];
    if (picked.locked) this.lockedChoice = null;
    this.apply(picked);
    this.pendLv--;
    this.pending = null;
    this.nova(155, 420, 0);
    this.p.inv = Math.max(this.p.inv, 0.65);
    if (this.pendLv > 0) this.openLevel();
  }

  reroll() {
    const pend = this.pending;
    if (!pend || pend.type !== "level" || this.rerolls <= 0) return false;
    this.rerolls--;
    pend.choices = this.genChoices(3, true);
    return true;
  }

  openChest(count) {
    const items = [];
    for (let n = 0; n < count; n++) {
      let pick = null;
      for (let i = 0; i < this.p.weapons.length; i++) {
        if (this.canEvolve(this.p.weapons[i])) {
          pick = { kind: "evo", id: this.p.weapons[i].id, lv: 0, isNew: true };
          break;
        }
      }
      if (!pick) {
        const pool = this.buildPool();
        const upgrades = pool.filter((x) => !x.isNew || x.kind === "syn" || x.kind === "relic");
        const candidates = upgrades.length ? upgrades : pool;
        pick = candidates.length
          ? candidates[(this.rnd() * candidates.length) | 0]
          : { kind: "boost", id: "b_might", lv: 0, isNew: false };
      }
      this.apply(pick);
      items.push(pick);
    }
    this.pending = { type: "chest", items };
    this.fx && this.fx(FX_EVT.CHEST, count);
  }

  closeChest() {
    if (this.pending && this.pending.type === "chest") {
      this.pending = null;
      this.p.inv = Math.max(this.p.inv, 0.65);
      this.nova(155, 420, 0);
      if (this.pendLv > 0) this.openLevel();
    }
  }

  spawnEnemy(type, x, y, elite) {
    if (this.nE >= MAX_ENEMIES) return null;
    const e = this.en[this.nE++];
    const def = ENEMIES[type];
    e.x = x;
    e.y = y;
    e.kx = 0;
    e.ky = 0;
    e.type = type;
    e.ai = def.ai;

    let affixIdx = -1;
    let affix = null;
    if (elite && !def.boss && (this.t >= 80 || this.diff.id !== "std")) {
      affixIdx = (this.rnd() * ELITE_AFFIXES.length) | 0;
      affix = ELITE_AFFIXES[affixIdx];
    }
    e.affix = affixIdx;

    const scale = def.boss ? Math.pow(1.68, Math.floor(this.bossIdx / 5)) : this.hpMulNow;
    e.maxHp = e.hp = Math.round(
      def.hp * scale * (this.diff.hpMul || 1) * (elite ? 12 : 1) * (affix ? affix.hpMul : 1),
    );
    e.r = def.r * (elite ? 1.58 : 1);
    e.sp =
      def.sp *
      (def.boss ? 1 : this.spMulNow) *
      (this.diff.spMul || 1) *
      (this.rule.speedAll || 1) *
      (affix ? affix.spMul : 1);
    e.dmg = def.dmg * (elite ? 1.45 : 1) * (this.diff.dmgMul || 1) * (affix?.dmgMul || 1);
    e.xpv = def.xp * (elite ? 11 : 1);
    e.wt = def.wt * (elite ? 4 : 1);
    e.flash = 0;
    e.slow = 0;
    e.vuln = 0;
    e.elite = elite ? 1 : 0;
    e.boss = def.boss || 0;
    e.uid = ++this.uid;
    e.st = 0;
    e.stT = 0;
    e.a1 = 0;
    e.a2 = 0;
    e.ax = 0;
    e.ay = 0;
    e.ph = this.rnd() * TAU;
    e.pulse = 0;
    e.mark = 0;
    e.dead = false;
    e.gone = false;
    e.tele = 0;
    e.born = this.t;
    e.swell = 0;
    e.face = 1;
    for (let i = 0; i < 12; i++) e.cd[i] = -9;
    if (def.ai === AI_TYPE.SPIT) e.stT = 1 + this.rnd() * 2;
    return e;
  }

  spawnPos() {
    const hw = this.vw / 2 + 36;
    const hh = this.vh / 2 + 36;
    const perim = hw * 4 + hh * 4;
    const s = this.rnd() * perim;
    let dx, dy;
    if (s < hw * 2) {
      dx = -hw + s;
      dy = -hh;
    } else if (s < hw * 4) {
      dx = -hw + (s - hw * 2);
      dy = hh;
    } else if (s < hw * 4 + hh * 2) {
      dx = -hw;
      dy = -hh + (s - hw * 4);
    } else {
      dx = hw;
      dy = -hh + (s - hw * 4 - hh * 2);
    }
    this.sx = this.p.x + dx;
    this.sy = this.p.y + dy;
  }

  typeW(def, t) {
    let w = def.id === "packet" && t > 180 ? Math.max(2, 10 - (t - 180) * 0.025) : def.w;
    const bias = STAGES[this.stageIdx].bias;
    if (bias && bias.indexOf(def.k) >= 0) w *= 1.9;
    return w;
  }

  pickType() {
    const t = this.t;
    const bias = STAGES[this.stageIdx].bias;
    let sum = 0;
    for (let i = 0; i < 7; i++) {
      const def = ENEMIES[i];
      const unlocked = t >= def.from || (bias && bias.indexOf(i) >= 0 && t >= 10);
      if (unlocked) sum += this.typeW(def, t);
    }
    let roll = this.rnd() * sum;
    for (let i = 0; i < 7; i++) {
      const def = ENEMIES[i];
      const unlocked = t >= def.from || (bias && bias.indexOf(i) >= 0 && t >= 10);
      if (unlocked) {
        roll -= this.typeW(def, t);
        if (roll <= 0) return i;
      }
    }
    return 0;
  }

  nextBossTypeIndex() {
    const stage = STAGES[this.stageIdx];
    if (stage.id === "hyperscale") {
      return ENEMY_IDX[BOSS_IDS[this.bossIdx % BOSS_IDS.length]];
    }
    return ENEMY_IDX[stage.bossId];
  }

  addGem(x, y, val) {
    if (this.nG >= MAX_GEMS) {
      this.overXp += val;
      return;
    }
    if (this.overXp > 0 && this.nG < MAX_GEMS - 30) {
      val += this.overXp;
      this.overXp = 0;
    }
    const g = this.gm[this.nG++];
    const a = this.rnd() * TAU;
    const sp = 40 + this.rnd() * 70;
    g.x = x;
    g.y = y;
    g.vx = Math.cos(a) * sp;
    g.vy = Math.sin(a) * sp;
    g.v = val;
    g.tier = val >= 20 ? 2 : val >= 5 ? 1 : 0;
    g.st = 0;
    g.sp = 0;
    g.t = 0;
  }

  addPickup(kind, x, y, n = 1) {
    if (this.nPK >= MAX_PICKS) return;
    const pk = this.pk[this.nPK++];
    pk.x = x;
    pk.y = y;
    pk.kind = kind;
    pk.t = 0;
    pk.n = n;
  }

  addZone() {
    if (this.nZ >= MAX_ZONES) return null;
    const z = this.zn[this.nZ++];
    z.t = 0;
    z.tickT = 0;
    z.evo = 0;
    z.mini = 0;
    z.own = null;
    z.st = 0;
    z.slow = 0;
    z.burst = 0;
    z.arm = 0;
    z.prog = 0;
    z.nPts = 0;
    return z;
  }

  shootE(x, y, dx, dy, sp, r, dmg, src, sq = 0) {
    if (this.nEB >= MAX_EBULLETS) return;
    const b = this.eb[this.nEB++];
    b.x = x;
    b.y = y;
    b.vx = dx * sp;
    b.vy = dy * sp;
    b.r = r;
    b.dmg = dmg;
    b.life = 6;
    b.src = src;
    b.sq = sq;
  }

  addBullet(kind, k, x, y, vx, vy, r, dmg, pierce, life) {
    if (this.nB >= MAX_BULLETS) return null;
    const b = this.bl[this.nB++];
    b.kind = kind;
    b.k = k;
    b.x = x;
    b.y = y;
    b.vx = vx;
    b.vy = vy;
    b.r = r;
    b.dmg = dmg;
    b.pierce = pierce;
    b.life = life;
    b.st = 0;
    b.rot = 0;
    b.size = 1;
    b.pop = 0;
    b.split = 0;
    b.wl = 0;
    b.lane = 0;
    b.t = 0;
    b.rays = 4;
    b.rayLen = 100;
    b.hits.length = 0;
    return b;
  }

  buildGrid() {
    const nE = this.nE;
    const en = this.en;
    const gStart = this.gStart;
    const gCell = this.gCell;
    this.gox = this.p.x - (GRID_W * CELL_SIZE) / 2;
    this.goy = this.p.y - (GRID_H * CELL_SIZE) / 2;
    gStart.fill(0);
    for (let i = 0; i < nE; i++) {
      const e = en[i];
      let cx = ((e.x - this.gox) / CELL_SIZE) | 0;
      let cy = ((e.y - this.goy) / CELL_SIZE) | 0;
      if (cx < 0) cx = 0;
      else if (cx >= GRID_W) cx = GRID_W - 1;
      if (cy < 0) cy = 0;
      else if (cy >= GRID_H) cy = GRID_H - 1;
      const cell = cy * GRID_W + cx;
      gCell[i] = cell;
      gStart[cell + 1]++;
    }
    for (let i = 1; i <= GRID_W * GRID_H; i++) gStart[i] += gStart[i - 1];
    this.gCur.set(gStart.subarray(0, GRID_W * GRID_H));
    for (let i = 0; i < nE; i++) {
      this.gItems[this.gCur[gCell[i]]++] = i;
    }
  }

  query(x, y, rad) {
    const pad = rad + 52;
    let x0 = ((x - pad - this.gox) / CELL_SIZE) | 0;
    let x1 = ((x + pad - this.gox) / CELL_SIZE) | 0;
    let y0 = ((y - pad - this.goy) / CELL_SIZE) | 0;
    let y1 = ((y + pad - this.goy) / CELL_SIZE) | 0;
    if (x0 < 0) x0 = 0;
    if (y0 < 0) y0 = 0;
    if (x1 >= GRID_W) x1 = GRID_W - 1;
    if (y1 >= GRID_H) y1 = GRID_H - 1;

    const en = this.en;
    const gStart = this.gStart;
    const gItems = this.gItems;
    const q = this.q;
    let count = 0;

    for (let cy = y0; cy <= y1; cy++) {
      const row = cy * GRID_W;
      for (let cx = x0; cx <= x1; cx++) {
        const s = gStart[row + cx];
        const eIdx = gStart[row + cx + 1];
        for (let k = s; k < eIdx; k++) {
          const idx = gItems[k];
          const m = en[idx];
          if (m.dead) continue;
          const dx = m.x - x;
          const dy = m.y - y;
          const rr = rad + m.r;
          if (dx * dx + dy * dy <= rr * rr) q[count++] = idx;
        }
      }
    }
    return count;
  }

  hit(e, baseDmg, weaponK, kb, dirX, dirY) {
    if (e.dead) return 0;
    const st = this.p.st;
    const bonusCrit =
      (weaponK === WEAPON_IDX.prism || weaponK === WEAPON_IDX.litho) && this.hasSynergy("syn_prism_litho") ? 0.15 : 0;
    const crit = this.rnd() < st.crit + bonusCrit ? 1 : 0;
    let mul = st.might * (crit ? st.critMul : 1);
    if (e.elite || e.boss) mul *= st.bossDmgMul || 1;
    if (e.vuln > 0) mul *= 1.2;
    if (weaponK === WEAPON_IDX.tensor && e.slow > 0 && this.hasSynergy("syn_tensor_cryo")) mul *= 1.35;

    const dmg = Math.max(1, Math.round(baseDmg * mul));
    e.hp -= dmg;
    e.flash = 0.11;

    if ((weaponK === WEAPON_IDX.litho || weaponK === WEAPON_IDX.prism) && this.hasSynergy("syn_prism_litho")) {
      e.vuln = 2.5;
    }
    if (kb > 0) {
      const f = kb / e.wt;
      e.kx += dirX * f;
      e.ky += dirY * f;
    }
    if (weaponK >= 0 && weaponK < this.dmgBy.length) this.dmgBy[weaponK] += dmg;
    this.fx && this.fx(FX_EVT.HIT, e.x, e.y, dmg, crit, weaponK);

    // Late-game Execution Threshold Relic (16% HP instant execute on non-bosses!)
    if (!e.boss && e.hp > 0 && this.hasRelic("r_execute") && e.hp <= e.maxHp * 0.16) {
      e.hp = 0;
    }
    if (e.hp <= 0) this.kill(e, crit);
    return dmg;
  }

  explode(x, y, rad, dmg, weaponK, kb, matrix = 0) {
    const n = this.query(x, y, rad);
    for (let i = 0; i < n; i++) {
      const e = this.en[this.q[i]];
      const dx = e.x - x;
      const dy = e.y - y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      this.hit(e, dmg, weaponK, kb, dx / d, dy / d);
    }
    this.fx && this.fx(matrix ? FX_EVT.MATRIX : FX_EVT.BOOM, x, y, rad, weaponK);
  }

  nova(rad, kb, dmg) {
    const p = this.p;
    const n = this.query(p.x, p.y, rad);
    for (let i = 0; i < n; i++) {
      const e = this.en[this.q[i]];
      const dx = e.x - p.x;
      const dy = e.y - p.y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      if (dmg) this.hit(e, dmg, 0, kb, dx / d, dy / d);
      else {
        e.kx += ((dx / d) * kb) / e.wt;
        e.ky += ((dy / d) * kb) / e.wt;
      }
    }
    this.fx && this.fx(FX_EVT.NOVA, p.x, p.y, rad);
  }

  kill(e, crit) {
    if (e.dead) return;
    e.dead = true;
    const p = this.p;
    this.kills++;
    this.killsBy[e.type]++;
    this.combo++;
    this.comboT = 2.4;
    if (this.combo > this.maxCombo) this.maxCombo = this.combo;
    while (this.comboIdx < COMBO_STEPS.length && this.combo >= COMBO_STEPS[this.comboIdx]) {
      this.fx && this.fx(FX_EVT.COMBO, COMBO_STEPS[this.comboIdx], this.comboIdx);
      this.comboIdx++;
    }

    // Photon Crit Refund Passive
    if (crit && this.char.mod.critCdRefund && this.skillCd > 0) {
      this.skillCd = Math.max(0, this.skillCd - 0.35);
    }

    // Siphon Relic: every 28 kills heals 6 HP (or pulses if full)
    if (this.hasRelic("r_siphon")) {
      this.siphonKills++;
      if (this.siphonKills >= 28) {
        this.siphonKills = 0;
        if (p.hp < p.maxHp) {
          p.hp = Math.min(p.maxHp, p.hp + 6);
        } else {
          this.pulse(0, 28, 135 * p.st.area, 0.6, 220);
        }
      }
    }

    // Chain Fusion Capacitor Relic: 25% chance to plasma-pop on kill
    if (!e.boss && this.hasRelic("r_chain_pop") && this.rnd() < 0.25) {
      this.explode(e.x, e.y, 48 * p.st.area, 22, 0, 90, 0);
    }

    this.addGem(e.x, e.y, e.xpv);
    if (e.elite) {
      this.elites++;
      this.addPickup(PICK_KIND.CHEST, e.x, e.y, 1);
      if (e.affix >= 0 && ELITE_AFFIXES[e.affix]?.split) {
        for (let s = 0; s < 4; s++) {
          const a = (s / 4) * TAU;
          this.spawnEnemy(ENEMY_IDX.frag, e.x + Math.cos(a) * 14, e.y + Math.sin(a) * 14, 0);
        }
      }
    }
    if (e.type === ENEMY_IDX.leak) {
      for (let i = 0; i < 3; i++) {
        const a = this.rnd() * TAU;
        const sub = this.spawnEnemy(ENEMY_IDX.frag, e.x + Math.cos(a) * 10, e.y + Math.sin(a) * 10, 0);
        if (sub) {
          sub.kx = Math.cos(a) * 165;
          sub.ky = Math.sin(a) * 165;
        }
      }
    } else if (e.type === ENEMY_IDX.short) {
      const z = this.addZone();
      if (z) {
        z.kind = ZONE_KIND.BLAST;
        z.x = e.x;
        z.y = e.y;
        z.r = 54;
        z.life = 0.75;
        z.dmg = 18;
        z.src = e.type;
      }
    }

    const roll = this.rnd();
    if (roll < 0.014) {
      if (p.hp < p.maxHp * 0.92) this.addPickup(PICK_KIND.HEART, e.x, e.y, 1);
    } else if (roll < 0.048) {
      this.addPickup(PICK_KIND.CORE, e.x, e.y, 1 + ((this.rnd() * 3) | 0));
    } else if (roll < 0.053 && this.t - this.lastMagnet > 35) {
      this.lastMagnet = this.t;
      this.addPickup(PICK_KIND.MAGNET, e.x, e.y, 1);
    } else if (roll < 0.056 && this.t - this.lastNuke > 45) {
      this.lastNuke = this.t;
      this.addPickup(PICK_KIND.NUKE, e.x, e.y, 1);
    }
    this.fx && this.fx(FX_EVT.KILL, e.x, e.y, e.type, e.boss ? 2 : e.elite ? 1 : 0, crit);
    if (e.boss) this.bossDead(e);
  }

  bossDead(e) {
    this.bosses++;
    this.bossList.push(ENEMIES[e.type].id);
    if (this.boss === e) {
      this.boss = null;
      for (let i = 0; i < this.nE; i++) {
        if (!this.en[i].dead && this.en[i].boss) {
          this.boss = this.en[i];
          break;
        }
      }
    }
    this.jumpReady = true;
    this.fx && this.fx(FX_EVT.BOSSDEAD, e.x, e.y, e.boss);
    for (let i = 0; i < this.nE; i++) {
      const m = this.en[i];
      if (m.dead || m.boss) continue;
      const dx = m.x - e.x;
      const dy = m.y - e.y;
      if (dx * dx + dy * dy < 700 * 700) {
        m.hp = 0;
        this.kill(m, 0);
      }
    }
    this.wipeEB = true;
    for (let i = 0; i < this.nZ; i++) {
      const z = this.zn[i];
      if (z.kind === ZONE_KIND.BLAST || z.kind === ZONE_KIND.EBEAM) z.st = 9;
    }
    this.addPickup(PICK_KIND.CHEST, e.x, e.y, 3);
    this.magnetT = 0.75;
  }

  hurt(rawDmg, src) {
    const p = this.p;
    if (p.inv > 0 || this.over) return;

    // Energy Shield Absorption!
    if (p.shields > 0) {
      p.shields--;
      p.inv = 0.85;
      this.nova(125, 420, 20);
      this.fx && this.fx(FX_EVT.SHIELD, p.x, p.y);
      return;
    }

    const dmg = Math.max(1, Math.round(rawDmg * this.dmgMulNow) - p.st.armor);
    p.hp -= dmg;
    p.inv = 0.62;
    this.lastSrc = src;
    this.hurts++;
    if (this.combo >= 10) this.fx && this.fx(FX_EVT.COMBOEND, this.combo);
    this.combo = 0;
    this.comboIdx = 0;
    this.comboT = 0;
    this.fx && this.fx(FX_EVT.HURT, dmg, p.x, p.y);
    if (p.st.thorns) this.nova(115, 390, 20);
    if (p.hp <= 0) {
      if (this.revives > 0) {
        this.revives--;
        p.hp = Math.round(p.maxHp * 0.5);
        p.shields = Math.min(3, p.shields + 1);
        p.inv = 2.5;
        this.nova(225, 620, 68);
        this.wipeEB = true;
        this.fx && this.fx(FX_EVT.REVIVE, p.x, p.y);
      } else {
        p.hp = 0;
        this.over = true;
        this.fx && this.fx(FX_EVT.DEAD, p.x, p.y);
      }
    }
  }

  rmZone(idx) {
    const z = this.zn[idx];
    z.own = null;
    const last = --this.nZ;
    this.zn[idx] = this.zn[last];
    this.zn[last] = z;
  }

  nuke() {
    const p = this.p;
    const hw = this.vw / 2 + 50;
    const hh = this.vh / 2 + 50;
    for (let i = 0; i < this.nE; i++) {
      const e = this.en[i];
      if (e.dead) continue;
      const dx = e.x - p.x;
      const dy = e.y - p.y;
      if (dx > hw || dx < -hw || dy > hh || dy < -hh) continue;
      if (e.boss) {
        const d = Math.sqrt(dx * dx + dy * dy) || 1;
        e.hp -= Math.round(e.maxHp * 0.12);
        e.flash = 0.2;
        e.kx += (dx / d) * 30;
        e.ky += (dy / d) * 30;
        if (e.hp <= 0) this.kill(e, 0);
      } else {
        e.hp = 0;
        this.kill(e, 0);
      }
    }
    this.wipeEB = true;
    this.magnetT = 0.65;
    this.fx && this.fx(FX_EVT.NUKE, p.x, p.y);
  }

  input(mx, my) {
    const len = Math.sqrt(mx * mx + my * my);
    if (len > 1) {
      mx /= len;
      my /= len;
    }
    this.p.mx = mx;
    this.p.my = my;
  }

  update(dt) {
    if (this.over || this.pending) return;
    const p = this.p;
    this.t += dt;
    const t = this.t;
    this.hpMulNow = CURVES.hpMul(t);
    this.dmgMulNow = CURVES.dmgMul(t);
    this.spMulNow = CURVES.spMul(t);

    // Cooldowns & Buff Timers
    if (this.skillCd > 0) this.skillCd = Math.max(0, this.skillCd - dt);
    let needRecalc = false;
    if (this.skillBuffT > 0) {
      this.skillBuffT -= dt;
      if (this.char.id === "astra") {
        this.skyTickT -= dt;
        if (this.skyTickT <= 0) {
          this.skyTickT = 0.72;
          this.spawnSkyBeam();
        }
      }
      if (this.skillBuffT <= 0) {
        this.skillBuffT = 0;
        if (this.char.id === "rubin") {
          this.explode(p.x, p.y, 180 * p.st.area, 40 + p.maxHp * 0.4, WEAPON_IDX.tensor, 340, 0);
        }
        needRecalc = true;
      }
    }
    if (this.skillPostT > 0) {
      this.skillPostT -= dt;
      if (this.skillPostT <= 0) needRecalc = true;
    }
    if (this.jumpBuffT > 0) {
      this.jumpBuffT -= dt;
      if (this.jumpBuffT <= 0) needRecalc = true;
    }
    if (this.nodeHasteT > 0) {
      this.nodeHasteT -= dt;
      if (this.nodeHasteT <= 0) needRecalc = true;
    }
    if (this.nodeCdT > 0) {
      this.nodeCdT -= dt;
      if (this.nodeCdT <= 0) needRecalc = true;
    }
    if (needRecalc) this.recalc();

    // Auto Shield Relic (every 15s)
    if (this.hasRelic("r_shield") && p.shields < 2) {
      p.shieldCd -= dt;
      if (p.shieldCd <= 0) {
        p.shieldCd = 15;
        p.shields++;
      }
    }

    // Sector Pipeline Jump Capacitor & Auto-Advance
    if (!this.jumpReady) {
      this.jumpCd -= dt;
      if (this.jumpCd <= 0) {
        this.jumpReady = true;
        this.jumpCd = STAGE_SEC;
        if (this.routeMode === "auto") {
          const nextIdx = (this.stageIdx + 1) % STAGES.length;
          this.jumpStage(nextIdx);
        }
      }
    }

    const sp = p.st.speed;
    const dxMove = p.mx * sp * dt;
    const dyMove = p.my * sp * dt;
    p.x += dxMove;
    p.y += dyMove;
    p.moving = p.mx * p.mx + p.my * p.my > 0.01;
    if (p.mx > 0.15) p.face = 1;
    else if (p.mx < -0.15) p.face = -1;
    if (p.inv > 0) p.inv -= dt;
    if (p.st.regen > 0 && p.hp < p.maxHp) p.hp = Math.min(p.maxHp, p.hp + p.st.regen * dt);
    if (this.magnetT > 0) this.magnetT -= dt;

    // Quant Stride-Zap Passive: moving 220px discharges an auto-arc!
    if (p.moving && this.char.mod.strideZap) {
      p.strideAcc += Math.sqrt(dxMove * dxMove + dyMove * dyMove);
      if (p.strideAcc >= 215 && this.near && this.nearD < 280) {
        p.strideAcc = 0;
        this.fx && this.fx(FX_EVT.BOLT, p.x, p.y, this.near.x, this.near.y, 0);
        this.hit(this.near, 24, WEAPON_IDX.bus, 90, 0, 0);
      }
    }

    this.director(dt);
    this.compact();
    this.buildGrid();
    this.findNear();
    this.updEnemies(dt);
    this.separate();
    this.updWeapons(dt);
    this.updBullets(dt);
    this.updZones(dt);
    this.updEB(dt);
    this.updGems(dt);
    this.updPicks(dt);

    if (this.comboT > 0) {
      this.comboT -= dt;
      if (this.comboT <= 0) {
        if (this.fx && this.combo >= 10) this.fx(FX_EVT.COMBOEND, this.combo);
        this.combo = 0;
        this.comboIdx = 0;
      }
    }
    if (this.pendLv > 0 && !this.pending && !this.over && this.autoLevel) this.openLevel();
  }

  compact() {
    let n = this.nE;
    let ghosts = 0;
    for (let i = 0; i < n; i++) {
      if (this.en[i].dead) {
        const tmp = this.en[i];
        n--;
        this.en[i] = this.en[n];
        this.en[n] = tmp;
        i--;
      } else if (this.en[i].ai === AI_TYPE.GHOST) {
        ghosts++;
      }
    }
    this.nE = n;
    this.ghosts = ghosts;
  }

  findNear() {
    const p = this.p;
    let best = null;
    let bestD2 = 405 * 405;
    for (let i = 0; i < this.nE; i++) {
      const e = this.en[i];
      const dx = e.x - p.x;
      const dy = e.y - p.y;
      const d2 = dx * dx + dy * dy;
      if (d2 < bestD2) {
        bestD2 = d2;
        best = e;
      }
    }
    this.near = best;
    this.nearD = best ? Math.sqrt(bestD2) : 1e9;
    if (best) {
      const d = this.nearD || 1;
      p.aimX = (best.x - p.x) / d;
      p.aimY = (best.y - p.y) / d;
    } else if (p.moving) {
      const d = Math.sqrt(p.mx * p.mx + p.my * p.my) || 1;
      p.aimX = p.mx / d;
      p.aimY = p.my / d;
    }
  }

  randNear(maxR) {
    if (!this.nE) return null;
    for (let i = 0; i < 10; i++) {
      const e = this.en[(this.rnd() * this.nE) | 0];
      if (e.dead) continue;
      const dx = e.x - this.p.x;
      const dy = e.y - this.p.y;
      if (dx * dx + dy * dy < maxR * maxR) return e;
    }
    return this.near && this.nearD < maxR ? this.near : null;
  }

  director(dt) {
    const t = this.t;
    const p = this.p;
    const countMul = this.rule.countMul || 1;
    const wv = Math.floor(t / WAVE_SEC);
    if (wv !== this.wave) {
      this.wave = wv;
      this.onWave(wv);
    }

    // Periodic Interactive Cleanroom Sector Nodes (replaces old scripted 12s ring!)
    this.nodeTimer -= dt * (this.rule.nodeFast || 1);
    if (this.nodeTimer <= 0) {
      this.nodeTimer = 32;
      this.spawnSectorNode();
    }

    let target = CURVES.target(t) * countMul;
    if (this.boss) target *= 0.65;
    this.spawnAcc = Math.min(8, this.spawnAcc + CURVES.rate(t) * countMul * dt);
    let loops = 0;
    while (this.spawnAcc >= 1 && loops++ < 10) {
      this.spawnAcc -= 1;
      if (this.nE - this.ghosts < target) {
        this.spawnPos();
        this.spawnEnemy(this.pickType(), this.sx, this.sy, 0);
      }
    }

    if (t >= this.nextElite) {
      const eliteMul = this.rule.eliteMul || 1;
      this.nextElite += (t < 220 ? 82 : 34) / eliteMul;
      const pool = [ENEMY_IDX.thermal];
      if (t >= 80) pool.push(ENEMY_IDX.crosstalk);
      if (t >= 120) pool.push(ENEMY_IDX.surge);
      if (t >= 155) pool.push(ENEMY_IDX.leak);
      this.spawnPos();
      const el = this.spawnEnemy(pool[(this.rnd() * pool.length) | 0], this.sx, this.sy, 1);
      if (el && this.fx) this.fx(FX_EVT.ELITE, el.x, el.y);
    }

    if (!this.bossWarned && t >= this.nextBoss - 3) {
      this.bossWarned = true;
      this.bossWarnType = this.nextBossTypeIndex();
      this.fx && this.fx(FX_EVT.BOSSWARN, this.bossWarnType - 9);
    }
    if (t >= this.nextBoss) {
      this.nextBoss += BOSS_SEC * (this.rule.bossFast || 1);
      this.bossWarned = false;
      const bType = this.bossWarnType || this.nextBossTypeIndex();
      const b = this.spawnEnemy(bType, p.x, p.y - this.vh / 2 - 50, 0);
      this.bossIdx++;
      if (b) {
        this.boss = b;
        b.stT = 1.8;
        b.a1 = 4.5;
        this.fx && this.fx(FX_EVT.BOSS, b.x, b.y, b.boss);
      }
      // Late-game Twin-Boss Endgame Escalation (8:00+ or Apex Tier Wave 2+)!
      if (t >= 480 || (this.diff.id === "apex" && this.bossIdx >= 2)) {
        const twinType = ENEMY_IDX[BOSS_IDS[(this.bossIdx + 2) % BOSS_IDS.length]];
        const twin = this.spawnEnemy(twinType, p.x + 140, p.y + this.vh / 2 + 50, 0);
        if (twin) {
          twin.stT = 2.4;
          twin.a1 = 5.0;
        }
      }
    }
  }

  onWave(w) {
    if (w <= 0) return;
    this.fx && this.fx(FX_EVT.WAVE, w + 1);
    const p = this.p;
    const mode = w % 3;
    if (mode === 1) {
      const cnt = Math.min(42, 11 + w * 2);
      const tp = this.t > 130 && this.rnd() < 0.5 ? ENEMY_IDX.jitter : ENEMY_IDX.packet;
      for (let i = 0; i < cnt; i++) {
        const a = (i / cnt) * TAU;
        this.spawnEnemy(tp, p.x + Math.cos(a) * (this.vw / 2 + 50), p.y + Math.sin(a) * (this.vw / 2 + 50) * 1.45, 0);
      }
    } else if (mode === 2) {
      const cnt = Math.min(28, 8 + w);
      const dir = this.rnd() < 0.5 ? 1 : -1;
      const span = this.vh * 0.36;
      for (let i = 0; i < cnt; i++) {
        const g = this.spawnEnemy(
          ENEMY_IDX.halluc,
          p.x - dir * (this.vw / 2 + 40 + (i % 4) * 26),
          p.y - span + ((i + 0.5) / cnt) * span * 2,
          0,
        );
        if (g) {
          g.ax = dir;
          g.ay = 0;
          g.stT = 13;
          g.ph = i * 0.7;
        }
      }
    } else {
      const cnt = Math.min(34, 10 + Math.floor(w * 1.5));
      const ang = this.rnd() * TAU;
      const dist = this.vh / 2 + 60;
      for (let i = 0; i < cnt; i++) {
        const rr = 70 * Math.sqrt(this.rnd());
        const ra = this.rnd() * TAU;
        this.spawnEnemy(
          ENEMY_IDX.jitter,
          p.x + Math.cos(ang) * dist * 0.7 + Math.cos(ra) * rr,
          p.y + Math.sin(ang) * dist + Math.sin(ra) * rr,
          0,
        );
      }
    }
  }

  updEnemies(dt) {
    const p = this.p;
    const damp = Math.exp(-8 * dt);
    const maxDist = Math.max(this.vw, this.vh) * 0.5 + 360;
    const t = this.t;
    for (let i = 0; i < this.nE; i++) {
      const e = this.en[i];
      if (e.dead) continue;
      if (e.flash > 0) e.flash -= dt;
      if (e.slow > 0) e.slow -= dt;
      if (e.vuln > 0) e.vuln -= dt;
      let dx = p.x - e.x;
      let dy = p.y - e.y;
      let dist = Math.sqrt(dx * dx + dy * dy) || 1;
      if (dist > maxDist && e.ai !== AI_TYPE.GHOST) {
        this.spawnPos();
        e.x = this.sx;
        e.y = this.sy;
        e.kx = e.ky = 0;
        dx = p.x - e.x;
        dy = p.y - e.y;
        dist = Math.sqrt(dx * dx + dy * dy) || 1;
      }
      const nx = dx / dist;
      const ny = dy / dist;
      let sp = e.sp * (e.slow > 0 ? 0.5 : 1);
      let vx = nx;
      let vy = ny;
      switch (e.ai) {
        case AI_TYPE.DART: {
          const w = Math.sin(t * 5 + e.ph) * 0.6;
          vx = nx - ny * w;
          vy = ny + nx * w;
          break;
        }
        case AI_TYPE.SPIT:
          if (dist < 120) {
            vx = -nx * 0.7;
            vy = -ny * 0.7;
          } else if (dist < 170) {
            vx = -ny * 0.35;
            vy = nx * 0.35;
          }
          e.stT -= dt;
          if (e.stT <= 0 && dist < 305) {
            e.stT = e.elite ? 1.5 : 2.75;
            e.swell = 0.25;
            this.shootE(e.x, e.y, nx, ny, 118, 6, 10, e.type, 1);
            this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 0);
          }
          break;
        case AI_TYPE.DASH:
          if (e.st === 0) {
            if (dist < 155) {
              e.st = 1;
              e.stT = 0.65;
              e.ax = nx;
              e.ay = ny;
              e.tele = 240;
            }
          } else if (e.st === 1) {
            sp = 0;
            e.stT -= dt;
            if (e.stT <= 0) {
              e.st = 2;
              e.stT = 0.5;
              e.tele = 0;
              this.fx && this.fx(FX_EVT.DASH, e.x, e.y);
            }
          } else if (e.st === 2) {
            vx = e.ax;
            vy = e.ay;
            sp = 345;
            e.stT -= dt;
            if (e.stT <= 0) {
              e.st = 3;
              e.stT = 0.9;
            }
          } else {
            sp *= 0.3;
            e.stT -= dt;
            if (e.stT <= 0) e.st = 0;
          }
          break;
        case AI_TYPE.GHOST: {
          const w = Math.cos(t * 2.4 + e.ph) * 0.55;
          vx = e.ax - e.ay * w;
          vy = e.ay + e.ax * w;
          e.stT -= dt;
          if (e.stT <= 0) {
            e.gone = true;
            e.dead = true;
            continue;
          }
          break;
        }
        case AI_TYPE.BOSS:
          sp = this.bossAI(e, dt, dist, nx, ny);
          vx = e.mvx;
          vy = e.mvy;
          break;
        default:
          break;
      }
      if (e.swell > 0) e.swell -= dt;
      e.x += (vx * sp + e.kx) * dt;
      e.y += (vy * sp + e.ky) * dt;
      e.kx *= damp;
      e.ky *= damp;
      if (vx > 0.2) e.face = 1;
      else if (vx < -0.2) e.face = -1;
      if (dist < e.r + p.r - 5 && p.inv <= 0) this.hurt(e.dmg, e.type);
    }
  }

  // 5 Distinct Sector Boss AIs
  bossAI(e, dt, dist, nx, ny) {
    const baseSp = e.sp * (e.slow > 0 ? 0.72 : 1);
    e.mvx = nx;
    e.mvy = ny;
    const catchup = dist > 520 ? 2.2 : 1;
    const rage = e.hp < e.maxHp * 0.42;

    if (e.boss === 1) {
      e.stT -= dt;
      e.a1 -= dt;
      if (e.stT < 0.45) e.swell = 0.3;
      if (e.stT <= 0) {
        const baseA = Math.atan2(ny, nx);
        const px = -Math.sin(baseA);
        const py = Math.cos(baseA);
        const gap = 2 + ((this.rnd() * 3) | 0);
        for (let i = 0; i < 7; i++) {
          if (i === gap) continue;
          const off = (i - 3) * 28;
          this.shootE(e.x + px * off, e.y + py * off, Math.cos(baseA), Math.sin(baseA), rage ? 132 : 112, 7, 13, e.type, 1);
        }
        e.stT = rage ? 2.0 : 3.1;
        this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 1);
      }
      if (e.a1 <= 0) {
        e.a1 = 8.5;
        for (let i = 0; i < 6; i++) {
          const a = (i / 6) * TAU;
          this.spawnEnemy(ENEMY_IDX.packet, e.x + Math.cos(a) * 64, e.y + Math.sin(a) * 64, 0);
        }
      }
      return baseSp * (rage ? 1.32 : 1) * catchup;
    }

    if (e.boss === 2) {
      e.stT -= dt;
      if (e.st === 0) {
        if (e.stT <= 0) {
          const arms = rage ? 4 : 3;
          const base = Math.atan2(ny, nx);
          for (let i = 0; i < arms; i++) {
            const z = this.addZone();
            if (!z) break;
            z.kind = ZONE_KIND.EBEAM;
            z.own = e;
            z.x = e.x;
            z.y = e.y;
            z.a = base + (i / arms) * TAU;
            z.av = rage ? 0.85 : 0.65;
            z.len = 520;
            z.r = 10;
            z.life = 2.6;
            z.arm = 0.85;
            z.dmg = 21;
            z.src = e.type;
          }
          e.st = 1;
          e.stT = 2.7;
        }
        return baseSp * catchup;
      }
      if (e.st === 1) {
        if (e.stT <= 0) {
          e.st = 2;
          e.stT = 0.6;
          e.ax = nx;
          e.ay = ny;
          e.tele = 320;
        }
        return baseSp * 0.25;
      }
      if (e.st === 2) {
        if (e.stT <= 0) {
          e.st = 3;
          e.stT = 0.52;
          e.tele = 0;
          this.fx && this.fx(FX_EVT.DASH, e.x, e.y);
        }
        return 0;
      }
      e.mvx = e.ax;
      e.mvy = e.ay;
      if (e.stT <= 0) {
        e.st = 0;
        e.stT = 1.6;
      }
      return 430;
    }

    if (e.boss === 3) {
      e.stT -= dt;
      if (e.st === 0) {
        e.a2 -= dt;
        if (e.a2 <= 0) {
          e.a2 = rage ? 0.13 : 0.18;
          e.a1 += Math.PI * 0.25;
          for (let i = 0; i < 4; i++) {
            const a = e.a1 + i * (Math.PI / 2);
            this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 108, 6, 13, e.type, 1);
          }
        }
        if (e.stT <= 0) {
          e.st = 1;
          const rot0 = (this.bossIdx % 2) * (Math.PI / 4);
          for (let i = 0; i < 4; i++) {
            const z = this.addZone();
            if (!z) break;
            z.kind = ZONE_KIND.EBEAM;
            z.own = e;
            z.x = e.x;
            z.y = e.y;
            z.a = rot0 + i * (Math.PI / 2);
            z.av = 0.35;
            z.len = 540;
            z.r = 10;
            z.life = 2.2;
            z.arm = 0.8;
            z.dmg = 23;
            z.src = e.type;
          }
          e.stT = 2.3;
        }
        return baseSp * 0.55 * catchup;
      }
      if (e.stT <= 0) {
        e.st = 0;
        e.stT = 3.0;
      }
      return baseSp * 0.2;
    }

    if (e.boss === 4) {
      e.stT -= dt;
      if (e.st === 0) {
        if (e.stT <= 0) {
          const p = this.p;
          const step = 108;
          for (let gx = -0.5; gx <= 0.5; gx += 1) {
            for (let gy = -0.5; gy <= 0.5; gy += 1) {
              const z = this.addZone();
              if (!z) break;
              z.kind = ZONE_KIND.BLAST;
              z.x = p.x + gx * step + (this.rnd() - 0.5) * 24;
              z.y = p.y + gy * step + (this.rnd() - 0.5) * 24;
              z.r = 56;
              z.life = 1.05;
              z.dmg = 24;
              z.src = e.type;
            }
          }
          e.st = 1;
          e.stT = rage ? 1.5 : 2.3;
        }
        return baseSp * catchup;
      }
      if (e.stT <= 0) {
        for (let i = 0; i < 10; i++) {
          const a = (i / 10) * TAU + e.ph;
          this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 116, 7, 15, e.type, 0);
        }
        this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 1);
        e.st = 0;
        e.stT = rage ? 1.9 : 2.8;
      }
      return baseSp * 0.7;
    }

    // Boss 5: Lithography Mother-Core
    e.stT -= dt;
    if (e.st === 0) {
      if (e.stT <= 0) {
        const p = this.p;
        for (let i = 0; i < 2; i++) {
          const z = this.addZone();
          if (!z) break;
          z.kind = ZONE_KIND.EBEAM;
          z.own = e;
          z.x = e.x;
          z.y = e.y;
          z.a = Math.atan2(p.y - e.y, p.x - e.x) + (i - 0.5) * 0.42;
          z.av = i === 0 ? 0.25 : -0.25;
          z.len = 560;
          z.r = 11;
          z.life = 2.0;
          z.arm = 0.78;
          z.dmg = 26;
          z.src = e.type;
        }
        for (let m = 0; m < 3; m++) {
          const z = this.addZone();
          if (!z) break;
          const a = (m / 3) * TAU + this.t;
          z.kind = ZONE_KIND.BLAST;
          z.x = p.x + Math.cos(a) * 86;
          z.y = p.y + Math.sin(a) * 86;
          z.r = 58;
          z.life = 1.0;
          z.dmg = 25;
          z.src = e.type;
        }
        e.st = 1;
        e.stT = 2.2;
      }
      return baseSp * catchup;
    }
    if (e.stT <= 0) {
      e.st = 0;
      e.stT = rage ? 1.8 : 2.7;
    }
    return baseSp * 0.4;
  }

  separate() {
    const en = this.en;
    const gStart = this.gStart;
    const gItems = this.gItems;
    for (let cy = 0; cy < GRID_H; cy++) {
      const row = cy * GRID_W;
      for (let cx = 0; cx < GRID_W; cx++) {
        const idx = row + cx;
        const s = gStart[idx];
        const e = gStart[idx + 1];
        if (s === e) continue;
        for (let i = s; i < e; i++) {
          const ia = gItems[i];
          const a = en[ia];
          if (a.dead || a.ai === AI_TYPE.GHOST) continue;
          for (let j = i + 1; j < e; j++) this.push(a, en[gItems[j]]);
          if (cx + 1 < GRID_W) {
            for (let j = gStart[idx + 1], e2 = gStart[idx + 2]; j < e2; j++) this.push(a, en[gItems[j]]);
          }
          if (cy + 1 < GRID_H) {
            const below = idx + GRID_W;
            for (let j = gStart[below], e2 = gStart[below + 1]; j < e2; j++) this.push(a, en[gItems[j]]);
            if (cx + 1 < GRID_W) {
              for (let j = gStart[below + 1], e2 = gStart[below + 2]; j < e2; j++) this.push(a, en[gItems[j]]);
            }
            if (cx > 0) {
              for (let j = gStart[below - 1], e2 = gStart[below]; j < e2; j++) this.push(a, en[gItems[j]]);
            }
          }
        }
      }
    }
  }

  push(a, b) {
    if (b.dead || b.ai === AI_TYPE.GHOST) return;
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const minD = (a.r + b.r) * 0.92;
    const d2 = dx * dx + dy * dy;
    if (d2 >= minD * minD || d2 < 1e-4) return;
    const d = Math.sqrt(d2);
    const pen = (minD - d) * 0.3;
    const nx = dx / d;
    const ny = dy / d;
    const wa = b.wt / (a.wt + b.wt);
    const wb = 1 - wa;
    a.x -= nx * pen * wa * 2;
    a.y -= ny * pen * wa * 2;
    b.x += nx * pen * wb * 2;
    b.y += ny * pen * wb * 2;
  }

  // 10 Hardware Weapons Simulation + Overclock Breakthroughs & Synergies!
  updWeapons(dt) {
    const p = this.p;
    const st = p.st;
    const weapons = p.weapons;
    const en = this.en;

    for (let i = 0; i < weapons.length; i++) {
      const w = weapons[i];
      const def = WEAPONS[w.k];
      const cfg = w.evo ? def.evo : def.lv[w.lv - 1];
      const k = w.k;
      const wMul = this.wDmgMul(w);
      const ocCdMul = w.oc > 0 ? Math.pow(0.94, w.oc) : 1;
      const ocAreaMul = w.oc > 0 ? 1 + w.oc * 0.06 : 1;
      const extraN = st.multishot || 0;
      w.t -= dt;

      switch (def.id) {
        // 1. HBM Parallel Bus Packets
        case "hbm":
          if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.1;
              break;
            }
            w.t = cfg.cd * st.cd * ocCdMul;
            const aim = Math.atan2(p.aimY, p.aimX) + (w.evo ? (this.rnd() - 0.5) * 0.16 : 0);
            const vx = Math.cos(aim);
            const vy = Math.sin(aim);
            const px = -vy;
            const py = vx;
            const sp = 385 * st.pspd;
            const lanes = cfg.n + extraN;
            const laneGap = w.evo ? 10.5 : 12;
            for (let n = 0; n < lanes; n++) {
              const off = (n - (lanes - 1) / 2) * laneGap;
              const b = this.addBullet(
                BULLET_KIND.HBM, k,
                p.x + vx * 14 + px * off,
                p.y + vy * 14 + py * off,
                vx * sp, vy * sp,
                w.evo ? 9 : 7.5,
                cfg.dmg * wMul,
                cfg.pierce + (st.pierceBonus || 0),
                1.15 * st.dur,
              );
              if (b) {
                b.lane = n;
                if (w.evo) {
                  b.pop = cfg.pop * st.area * ocAreaMul;
                  b.size = 1.24 * ocAreaMul;
                }
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        // 2. Silicon Photonics WDM Prism Lance
        case "prism":
          if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.12;
              break;
            }
            w.t = cfg.cd * st.cd * ocCdMul;
            const sp = 435 * st.pspd;
            const shots = cfg.n + extraN;
            for (let n = 0; n < shots; n++) {
              const tgt = n === 0 ? this.near : this.randNear(320) || this.near;
              const a = Math.atan2(tgt.y - p.y, tgt.x - p.x) + (n > 0 ? (this.rnd() - 0.5) * 0.24 : 0);
              const vx = Math.cos(a);
              const vy = Math.sin(a);
              const b = this.addBullet(
                BULLET_KIND.PRISM_MAIN, k,
                p.x + vx * 14, p.y + vy * 14,
                vx * sp, vy * sp,
                9, cfg.dmg * wMul, st.pierceBonus || 0, 1.08 * st.dur,
              );
              if (b) {
                b.split = cfg.split;
                b.size = (w.evo ? 1.3 : 1.05) * ocAreaMul;
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        // 3. Tensor 3x3 Matrix Stamp
        case "tensor":
          if (w.t <= 0) {
            if (this.nearD > 360) {
              w.t = 0.18;
              break;
            }
            w.t = cfg.cd * st.cd * ocCdMul;
            const synArea = this.hasSynergy("syn_tensor_cryo") ? 1.18 : 1;
            const count = cfg.n + extraN;
            for (let n = 0; n < count; n++) {
              const tgt = n === 0 ? this.near : this.randNear(300) || this.near;
              const z = this.addZone();
              if (!z) break;
              z.kind = ZONE_KIND.TENSOR;
              z.k = k;
              z.ox = p.x;
              z.oy = p.y;
              z.x = tgt ? tgt.x : p.x + p.aimX * 90;
              z.y = tgt ? tgt.y : p.y + p.aimY * 90;
              z.arm = 0.4;
              z.life = 0.52;
              z.r = cfg.rad * st.area * ocAreaMul * synArea;
              z.dmg = cfg.dmg * wMul;
              z.evo = w.evo ? cfg.mini : 0;
              z.rMax = w.evo ? cfg.miniR * st.area * ocAreaMul : 0;
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        // 4. L3 Cache Silicon Die Ring (100% Always Active — Zero Downtime!)
        case "cache": {
          w.on = 1;
          w.vis = 1;
          w.ang += (cfg.spin || 3.6) * dt;
          const n1 = cfg.n;
          const n2 = w.evo ? cfg.n2 : 0;
          w.np = n1 + n2;
          const synScale = this.hasSynergy("syn_cache_drone") ? 1.18 : 1;
          for (let n = 0; n < n1 + n2; n++) {
            const inner = n < n1;
            const a = inner ? w.ang + (n / n1) * TAU : -w.ang * 0.78 + ((n - n1) / n2) * TAU;
            const rad = (inner ? cfg.rad : cfg.rad2) * st.area * ocAreaMul;
            const sx = p.x + Math.cos(a) * rad;
            const sy = p.y + Math.sin(a) * rad;
            w.pos[n * 2] = sx;
            w.pos[n * 2 + 1] = sy;
            const cnt = this.query(sx, sy, 11 * synScale);
            for (let j = 0; j < cnt; j++) {
              const e = en[this.q[j]];
              if (this.t - e.cd[k] < 0.34) continue;
              e.cd[k] = this.t;
              const dx = e.x - p.x;
              const dy = e.y - p.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              this.hit(e, cfg.dmg * wMul, k, 265, dx / d, dy / d);
            }
          }
          if (cfg.wave > 0) {
            w.waveT -= dt;
            if (w.waveT <= 0) {
              w.waveT = cfg.wave * st.cd * ocCdMul;
              this.pulse(k, cfg.waveDmg * wMul, cfg.waveR * st.area * ocAreaMul, 0, 300);
            }
          }
          break;
        }

        // 5. PCB High-Voltage Bus Arc
        case "bus":
          if (w.t <= 0) {
            let fired = 0;
            const synBoost = this.hasSynergy("syn_nvlink_bus");
            const chains = cfg.chain + (synBoost ? 2 : 0);
            const arcs = cfg.n + extraN;
            for (let n = 0; n < arcs; n++) {
              let cur = n === 0 && !w.evo ? this.near : this.randNear(w.evo ? 355 : 305);
              if (!cur || (n === 0 && this.nearD > 325 && !w.evo)) break;
              fired++;
              const mark = ++this.stamp;
              let sx = p.x;
              let sy = p.y;
              for (let c = 0; c <= chains && cur; c++) {
                cur.mark = mark;
                const dx = cur.x - sx;
                const dy = cur.y - sy;
                const d = Math.sqrt(dx * dx + dy * dy) || 1;
                this.fx && this.fx(FX_EVT.BOLT, sx, sy, cur.x, cur.y, w.evo ? 1 : 0);
                const tx = cur.x;
                const ty = cur.y;
                this.hit(cur, cfg.dmg * wMul, k, 75, dx / d, dy / d);
                if (w.evo && c === 0) this.explode(tx, ty, cfg.boom * st.area * ocAreaMul, cfg.dmg * 0.55 * wMul, k, 125, 0);
                sx = tx;
                sy = ty;
                const cnt = this.query(tx, ty, 138 * st.area);
                let next = null;
                let bestD2 = 1e9;
                for (let j = 0; j < cnt; j++) {
                  const cand = en[this.q[j]];
                  if (cand.mark === mark || cand.dead) continue;
                  const cdx = cand.x - tx;
                  const cdy = cand.y - ty;
                  const cd2 = cdx * cdx + cdy * cdy;
                  if (cd2 < bestD2) {
                    bestD2 = cd2;
                    next = cand;
                  }
                }
                cur = next;
              }
            }
            if (fired) {
              w.t = cfg.cd * st.cd * ocCdMul * (synBoost ? 0.86 : 1);
              this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
            } else {
              w.t = 0.15;
            }
          }
          break;

        // 6. EUV Lithography Scanner Bar
        case "litho":
          if (w.evo) {
            w.ang += cfg.spin * dt;
            w.bt = 1;
            w.np = cfg.n;
          } else if (w.bt > 0) {
            w.bt -= dt;
            let diff = Math.atan2(p.aimY, p.aimX) - w.ang;
            while (diff > Math.PI) diff -= TAU;
            while (diff < -Math.PI) diff += TAU;
            w.ang += Math.max(-1.4 * dt, Math.min(1.4 * dt, diff));
          } else if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.15;
              break;
            }
            w.bt = cfg.dur * st.dur;
            w.t = cfg.cd * st.cd * ocCdMul + w.bt;
            w.ang = Math.atan2(p.aimY, p.aimX);
            w.tick = 0;
            w.np = cfg.n;
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
          }
          if (w.bt > 0) {
            w.tick -= dt;
            if (w.tick <= 0) {
              w.tick = 0.1;
              const len = cfg.len * st.area * ocAreaMul;
              const hw = cfg.wid * 0.5 * Math.sqrt(st.area * ocAreaMul);
              for (let n = 0; n < cfg.n; n++) {
                const a = w.ang + (n / cfg.n) * TAU;
                const ux = Math.cos(a);
                const uy = Math.sin(a);
                for (let j = 0; j < this.nE; j++) {
                  const e = en[j];
                  if (e.dead) continue;
                  const rx = e.x - p.x;
                  const ry = e.y - p.y;
                  const proj = rx * ux + ry * uy;
                  if (proj < -e.r || proj > len + e.r) continue;
                  const perp = ry * ux - rx * uy;
                  if (perp <= hw + e.r && perp >= -hw - e.r) {
                    this.hit(e, cfg.dmg * wMul, k, 45, ux, uy);
                  }
                }
              }
            }
          }
          break;

        // 7. Autonomous ASIC Pursuit Drones
        case "drone": {
          const synDrone = this.hasSynergy("syn_cache_drone");
          if (w.evo) {
            w.ang += 2.9 * dt;
            w.ph += 1.3 * dt;
            w.np = cfg.n;
            const hr = 13 * cfg.size * Math.sqrt(st.area * ocAreaMul) * (synDrone ? 1.18 : 1);
            for (let n = 0; n < cfg.n; n++) {
              const a = w.ang + (n / cfg.n) * TAU;
              const rad = (cfg.rMin + (cfg.rMax - cfg.rMin) * (0.5 + 0.5 * Math.sin(w.ph + n * 1.26))) * st.area;
              const wx = p.x + Math.cos(a) * rad;
              const wy = p.y + Math.sin(a) * rad;
              w.pos[n * 2] = wx;
              w.pos[n * 2 + 1] = wy;
              const cnt = this.query(wx, wy, hr);
              for (let j = 0; j < cnt; j++) {
                const e = en[this.q[j]];
                if (this.t - e.cd[k] >= 0.27) {
                  e.cd[k] = this.t;
                  this.hit(e, cfg.dmg * wMul, k, synDrone ? 240 : 155, -Math.sin(a), Math.cos(a));
                }
              }
            }
          } else if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.15;
              break;
            }
            w.t = cfg.cd * st.cd * ocCdMul;
            const count = cfg.n + extraN;
            for (let n = 0; n < count; n++) {
              const tgt = n === 0 ? this.near : this.randNear(285) || this.near;
              let dx = tgt.x - p.x;
              let dy = tgt.y - p.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              dx /= d;
              dy /= d;
              if (n > 0) {
                const a = Math.atan2(dy, dx) + (this.rnd() - 0.5) * 0.55;
                dx = Math.cos(a);
                dy = Math.sin(a);
              }
              const sp = 345 * st.pspd;
              const b = this.addBullet(
                BULLET_KIND.DRONE, k, p.x, p.y, dx * sp, dy * sp,
                11 * cfg.size * Math.sqrt(st.area * ocAreaMul),
                cfg.dmg * wMul, 999, 6,
              );
              if (b) {
                b.t = (cfg.range * st.area) / sp;
                b.size = cfg.size * Math.sqrt(st.area * ocAreaMul) * (synDrone ? 1.16 : 1);
                b.lane = n % 2 ? 1 : -1;
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
          }
          break;
        }

        // 8. Immersion Cryo-Coolant Hexagonal Plates
        case "cryo":
          if (w.t <= 0) {
            let spawned = 0;
            const count = cfg.n + (w.evo ? 0 : extraN);
            for (let n = 0; n < count; n++) {
              let tgt = this.randNear(275);
              if (!tgt) break;
              if (w.evo) {
                let bestDensity = -1;
                for (let s = 0; s < 6; s++) {
                  const cand = this.randNear(295);
                  if (!cand) break;
                  const dens = this.query(cand.x, cand.y, 85);
                  if (dens > bestDensity) {
                    bestDensity = dens;
                    tgt = cand;
                  }
                }
              }
              const z = this.addZone();
              if (!z) break;
              z.kind = ZONE_KIND.CRYO;
              z.k = k;
              z.x = tgt.x;
              z.y = tgt.y;
              z.r = cfg.rad * st.area * ocAreaMul;
              z.life = cfg.dur * st.dur;
              z.dmg = cfg.dmg * wMul;
              z.evo = w.evo ? 1 : 0;
              z.burst = w.evo ? cfg.burst * wMul : 0;
              z.a = this.rnd() * TAU;
              spawned++;
            }
            if (spawned) {
              w.t = cfg.cd * st.cd * ocCdMul;
              this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
            } else {
              w.t = 0.25;
            }
          }
          break;

        // 9. NVLink Laser Mesh Matrix (Connects nearby enemies into a burning star-web!)
        case "nvlink":
          if (w.t <= 0) {
            const synBoost = this.hasSynergy("syn_nvlink_bus");
            const maxNodes = Math.min(10, cfg.nodes + (synBoost ? 2 : 0) + extraN);
            const rad = cfg.rad * st.area * ocAreaMul;
            const cnt = this.query(p.x, p.y, rad);
            if (cnt < 1) {
              w.t = 0.16;
              break;
            }
            const z = this.addZone();
            if (z) {
              z.kind = ZONE_KIND.NVLINK;
              z.k = k;
              z.x = p.x;
              z.y = p.y;
              z.r = rad;
              z.life = cfg.dur * st.dur;
              z.dmg = cfg.dmg * wMul;
              z.evo = w.evo ? 1 : 0;
              const picked = Math.min(maxNodes, cnt);
              z.nPts = picked;
              const stride = Math.max(1, Math.floor(cnt / picked));
              for (let m = 0; m < picked; m++) {
                const e = en[this.q[(m * stride) % cnt]];
                z.pts[m * 2] = e.x;
                z.pts[m * 2 + 1] = e.y;
              }
              w.t = cfg.cd * st.cd * ocCdMul * (synBoost ? 0.86 : 1);
              this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
            }
          }
          break;

        // 10. Liquid-N2 Cryo Mortar (Fires canisters that shatter into 4/8 frost cross-rails!)
        case "liquid_n2":
          if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.16;
              break;
            }
            w.t = cfg.cd * st.cd * ocCdMul;
            const shots = cfg.n + extraN;
            const sp = 330 * st.pspd;
            for (let n = 0; n < shots; n++) {
              const tgt = n === 0 ? this.near : this.randNear(310) || this.near;
              const dx = tgt.x - p.x;
              const dy = tgt.y - p.y;
              const dist = Math.max(40, Math.sqrt(dx * dx + dy * dy));
              const b = this.addBullet(
                BULLET_KIND.N2_CANISTER, k,
                p.x, p.y,
                (dx / dist) * sp, (dy / dist) * sp,
                10, cfg.dmg * wMul, 0, dist / sp,
              );
              if (b) {
                b.rays = cfg.rays;
                b.rayLen = cfg.rayLen * st.area * ocAreaMul;
                b.size = (w.evo ? 1.25 : 1.0) * ocAreaMul;
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        default:
          break;
      }
    }
  }

  shatterN2Canister(b) {
    const z = this.addZone();
    if (z) {
      z.kind = ZONE_KIND.FROST_RAIL;
      z.k = b.k;
      z.x = b.x;
      z.y = b.y;
      z.evo = b.rays >= 8 ? 1 : 0;
      z.nPts = b.rays;
      z.len = b.rayLen;
      z.life = 0.55 * this.p.st.dur;
      z.dmg = b.dmg;
      z.a = b.rays >= 8 ? 0 : Math.PI / 4;
    }
    this.explode(b.x, b.y, 44 * b.size, b.dmg * 0.7, b.k, 140, 0);

    // Synergy: Liquid-N2 + Prism shoots 4 refracted needles from frost center!
    if (this.hasSynergy("syn_n2_prism")) {
      const sp = 420 * this.p.st.pspd;
      for (let s = 0; s < 4; s++) {
        const a = (s / 4) * TAU;
        const sub = this.addBullet(
          BULLET_KIND.PRISM_SUB, WEAPON_IDX.prism,
          b.x, b.y, Math.cos(a) * sp, Math.sin(a) * sp,
          6.5, b.dmg * 0.55, 2, 0.7,
        );
        if (sub) sub.wl = s % 3;
      }
    }
  }

  pulse(k, dmg, rMax, slow, kb) {
    const z = this.addZone();
    if (z) {
      z.kind = ZONE_KIND.PULSE;
      z.k = k;
      z.x = this.p.x;
      z.y = this.p.y;
      z.r = 14;
      z.rMax = rMax;
      z.dmg = dmg;
      z.id = ++this.pulseId;
      z.slow = slow;
      z.life = 1;
      z.len = kb;
      this.fx && this.fx(FX_EVT.RING, z.x, z.y, rMax, k);
    }
  }

  updBullets(dt) {
    const bl = this.bl;
    const en = this.en;
    const p = this.p;
    for (let i = this.nB - 1; i >= 0; i--) {
      const b = bl[i];
      let dead = false;
      b.life -= dt;

      if (b.kind === BULLET_KIND.N2_CANISTER) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        b.rot += 7 * dt;
        if (b.life <= 0) {
          this.shatterN2Canister(b);
          dead = true;
        }
      } else if (b.kind === BULLET_KIND.HBM || b.kind === BULLET_KIND.PRISM_SUB) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        const cnt = this.query(b.x, b.y, b.r);
        for (let j = 0; j < cnt && !dead; j++) {
          const e = en[this.q[j]];
          if (b.hits.indexOf(e.uid) >= 0) continue;
          b.hits.push(e.uid);
          const sp = Math.sqrt(b.vx * b.vx + b.vy * b.vy) || 1;
          this.hit(e, b.dmg, b.k, 110, b.vx / sp, b.vy / sp);

          // Synergy: HBM + PCB Bus triggers chain spark every 4th HBM hit!
          if (b.kind === BULLET_KIND.HBM && this.hasSynergy("syn_bus_hbm")) {
            p.hbmCounter++;
            if (p.hbmCounter >= 4) {
              p.hbmCounter = 0;
              const nearCnt = this.query(e.x, e.y, 120);
              for (let m = 0; m < nearCnt; m++) {
                const other = en[this.q[m]];
                if (other !== e && !other.dead) {
                  this.fx && this.fx(FX_EVT.BOLT, e.x, e.y, other.x, other.y, 0);
                  this.hit(other, b.dmg * 0.85, WEAPON_IDX.bus, 80, 0, 0);
                  break;
                }
              }
            }
          }

          if (b.hits.length > b.pierce) dead = true;
        }
        if (b.life <= 0) dead = true;
        if (dead) {
          if (b.pop > 0) this.explode(b.x, b.y, b.pop, b.dmg * 0.65, b.k, 95, 0);
          else this.fx && this.fx(FX_EVT.POP, b.x, b.y, b.k);
        }
      } else if (b.kind === BULLET_KIND.PRISM_MAIN) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        const cnt = this.query(b.x, b.y, b.r);
        for (let j = 0; j < cnt && !dead; j++) {
          const e = en[this.q[j]];
          if (b.hits.indexOf(e.uid) >= 0) continue;
          b.hits.push(e.uid);
          const sp = Math.sqrt(b.vx * b.vx + b.vy * b.vy) || 1;
          const baseAng = Math.atan2(b.vy, b.vx);
          this.hit(e, b.dmg, b.k, 115, b.vx / sp, b.vy / sp);
          const splits = b.split || 3;
          for (let s = 0; s < splits; s++) {
            const sa = baseAng + (s - (splits - 1) / 2) * 0.28;
            const svx = Math.cos(sa);
            const svy = Math.sin(sa);
            const sub = this.addBullet(
              BULLET_KIND.PRISM_SUB, b.k,
              e.x + svx * 10, e.y + svy * 10,
              svx * sp * 0.96, svy * sp * 0.96,
              6.5, b.dmg * 0.68, 2, 0.72 * p.st.dur,
            );
            if (sub) {
              sub.hits.push(e.uid);
              sub.wl = s % 3;
              sub.size = b.size * 0.9;
            }
          }
          if (b.hits.length > b.pierce) dead = true;
        }
        if (b.life <= 0) dead = true;
      } else {
        // BULLET_KIND.DRONE
        if (b.st === 0) {
          b.t -= dt;
          if (b.t <= 0) b.st = 1;
          const ease = Math.max(0.3, Math.min(1, b.t * 3.8));
          const curve = b.lane * 0.95 * dt;
          const cvx = b.vx * Math.cos(curve) - b.vy * Math.sin(curve);
          const cvy = b.vx * Math.sin(curve) + b.vy * Math.cos(curve);
          b.vx = cvx;
          b.vy = cvy;
          b.x += b.vx * ease * dt;
          b.y += b.vy * ease * dt;
          b.rot = Math.atan2(b.vy, b.vx);
        } else {
          const dx = p.x - b.x;
          const dy = p.y - b.y;
          const d = Math.sqrt(dx * dx + dy * dy) || 1;
          b.t += dt;
          const sp = Math.min(560, 230 + b.t * 720);
          b.vx = (dx / d) * sp;
          b.vy = (dy / d) * sp;
          b.x += b.vx * dt;
          b.y += b.vy * dt;
          b.rot = Math.atan2(b.vy, b.vx);
          if (d < 18) dead = true;
        }
        const cnt = this.query(b.x, b.y, b.r);
        for (let j = 0; j < cnt; j++) {
          const e = en[this.q[j]];
          if (this.t - e.cd[b.k] < 0.3) continue;
          e.cd[b.k] = this.t;
          const sp = Math.sqrt(b.vx * b.vx + b.vy * b.vy) || 1;
          this.hit(e, b.dmg, b.k, 135, b.vx / sp, b.vy / sp);
        }
        if (b.life <= 0) dead = true;
      }

      if (dead) {
        const last = --this.nB;
        bl[i] = bl[last];
        bl[last] = b;
      }
    }
  }

  updZones(dt) {
    const zn = this.zn;
    const en = this.en;
    const p = this.p;
    const fastNode = branchPoints(this.talents, "cluster") >= TALENT_BRANCHES[3].apexMilestone ? 1.38 : 1;

    for (let i = this.nZ - 1; i >= 0; i--) {
      const z = zn[i];
      let dead = false;
      z.t += dt;
      if (z.st === 9) {
        this.rmZone(i);
        continue;
      }
      switch (z.kind) {
        case ZONE_KIND.NODE: {
          const dx = p.x - z.x;
          const dy = p.y - z.y;
          if (dx * dx + dy * dy <= (z.r + p.r) * (z.r + p.r)) {
            z.prog += (dt / 1.85) * fastNode;
            if (z.prog >= 1) {
              this.triggerNodeReward(z);
              dead = true;
              break;
            }
          } else if (z.prog > 0) {
            z.prog = Math.max(0, z.prog - dt * 0.2);
          }
          if (z.t >= z.life) dead = true;
          break;
        }
        case ZONE_KIND.SKYBEAM:
          if (z.t >= z.life) dead = true;
          break;
        case ZONE_KIND.NVLINK: {
          z.tickT -= dt;
          if (z.tickT <= 0) {
            z.tickT = 0.26;
            const cnt = this.query(z.x, z.y, z.r);
            for (let j = 0; j < cnt; j++) {
              const e = en[this.q[j]];
              if (z.evo) e.slow = Math.max(e.slow, 0.45);
              this.hit(e, z.dmg, z.k, 25, 0, 0);
            }
          }
          if (z.t >= z.life) dead = true;
          break;
        }
        case ZONE_KIND.FROST_RAIL: {
          if (z.st === 0) {
            z.st = 1;
            const rays = z.nPts || 4;
            for (let r = 0; r < rays; r++) {
              const a = z.a + (r / rays) * TAU;
              const ux = Math.cos(a);
              const uy = Math.sin(a);
              const cnt = this.query(z.x, z.y, z.len + 20);
              for (let j = 0; j < cnt; j++) {
                const e = en[this.q[j]];
                const rx = e.x - z.x;
                const ry = e.y - z.y;
                const proj = rx * ux + ry * uy;
                if (proj < -10 || proj > z.len + e.r) continue;
                const perp = Math.abs(ry * ux - rx * uy);
                if (perp <= 16 + e.r) {
                  e.slow = Math.max(e.slow, 1.4);
                  this.hit(e, z.dmg, z.k, 110, ux, uy);
                }
              }
            }
          }
          if (z.t >= z.life) dead = true;
          break;
        }
        case ZONE_KIND.PULSE: {
          z.x = p.x;
          z.y = p.y;
          z.r += (z.rMax / 0.3) * dt;
          if (z.r >= z.rMax) {
            z.r = z.rMax;
            dead = true;
          }
          const cnt = this.query(z.x, z.y, z.r);
          for (let j = 0; j < cnt; j++) {
            const e = en[this.q[j]];
            if (e.pulse === z.id) continue;
            e.pulse = z.id;
            const dx = e.x - z.x;
            const dy = e.y - z.y;
            const d = Math.sqrt(dx * dx + dy * dy) || 1;
            if (z.slow) e.slow = Math.max(e.slow, z.slow);
            this.hit(e, z.dmg, z.k, z.len, dx / d, dy / d);
          }
          break;
        }
        case ZONE_KIND.TENSOR: {
          if (z.t >= z.life) {
            this.explode(z.x, z.y, z.r, z.dmg, z.k, 280, 1);
            if (z.evo > 0 && !z.mini) {
              for (let m = 0; m < z.evo; m++) {
                const sub = this.addZone();
                if (!sub) break;
                const a = Math.PI / 4 + (m / z.evo) * TAU;
                sub.kind = ZONE_KIND.TENSOR;
                sub.k = z.k;
                sub.ox = z.x;
                sub.oy = z.y;
                sub.x = z.x + Math.cos(a) * 68;
                sub.y = z.y + Math.sin(a) * 68;
                sub.arm = 0.24;
                sub.life = 0.32;
                sub.r = z.rMax;
                sub.dmg = z.dmg * 0.54;
                sub.mini = 1;
                sub.evo = 0;
              }
            }
            dead = true;
          }
          break;
        }
        case ZONE_KIND.CRYO: {
          z.a += dt * 1.8;
          z.tickT -= dt;
          if (z.tickT <= 0) {
            z.tickT = 0.25;
            const cnt = this.query(z.x, z.y, z.r);
            const pull = z.evo ? 160 : 75;
            for (let j = 0; j < cnt; j++) {
              const e = en[this.q[j]];
              e.slow = Math.max(e.slow, 0.65);
              const dx = z.x - e.x;
              const dy = z.y - e.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              e.kx += ((dx / d) * pull) / e.wt;
              e.ky += ((dy / d) * pull) / e.wt;
              this.hit(e, z.dmg, z.k, 0, 0, 0);
            }
          }
          if (z.t >= z.life) {
            if (z.burst) this.explode(z.x, z.y, z.r * 1.08, z.burst, z.k, 360, 0);
            dead = true;
          }
          break;
        }
        case ZONE_KIND.BLAST:
          if (z.t >= z.life) {
            const dx = p.x - z.x;
            const dy = p.y - z.y;
            if (dx * dx + dy * dy < (z.r - 4) * (z.r - 4)) this.hurt(z.dmg, z.src);
            this.fx && this.fx(FX_EVT.BOOM, z.x, z.y, z.r, 99);
            dead = true;
          }
          break;
        case ZONE_KIND.EBEAM: {
          const own = z.own;
          if (!own || own.dead || z.t >= z.life) {
            dead = true;
            break;
          }
          z.x = own.x;
          z.y = own.y;
          if (z.t >= z.arm) {
            z.a += z.av * dt;
            const ux = Math.cos(z.a);
            const uy = Math.sin(z.a);
            const rx = p.x - z.x;
            const ry = p.y - z.y;
            const proj = rx * ux + ry * uy;
            if (proj > 0 && proj < z.len) {
              const perp = Math.abs(ry * ux - rx * uy);
              if (perp < z.r + p.r - 3) this.hurt(z.dmg, z.src);
            }
          }
          break;
        }
        default:
          break;
      }
      if (dead) this.rmZone(i);
    }
  }

  updEB(dt) {
    const eb = this.eb;
    const p = this.p;
    if (this.wipeEB) {
      this.wipeEB = false;
      this.nEB = 0;
      return;
    }
    for (let i = this.nEB - 1; i >= 0; i--) {
      const b = eb[i];
      b.x += b.vx * dt;
      b.y += b.vy * dt;
      b.life -= dt;
      const dx = p.x - b.x;
      const dy = p.y - b.y;
      const rr = p.r + b.r - 3;
      if (dx * dx + dy * dy <= rr * rr) {
        this.hurt(b.dmg, b.src);
        b.life = 0;
      }
      if (b.life <= 0) {
        const last = --this.nEB;
        eb[i] = eb[last];
        eb[last] = b;
      }
    }
  }

  updGems(dt) {
    const gm = this.gm;
    const p = this.p;
    const pickR2 = p.st.pickup * p.st.pickup;
    const mag = this.magnetT > 0;
    for (let i = this.nG - 1; i >= 0; i--) {
      const g = gm[i];
      g.t += dt;
      const dx = p.x - g.x;
      const dy = p.y - g.y;
      const d2 = dx * dx + dy * dy;
      if (g.st === 0) {
        if (g.t < 0.25) {
          g.x += g.vx * dt;
          g.y += g.vy * dt;
          g.vx *= 0.86;
          g.vy *= 0.86;
        }
        if ((d2 < pickR2 && g.t > 0.12) || mag) {
          g.st = 1;
          g.sp = 45;
        }
      } else {
        const d = Math.sqrt(d2) || 1;
        g.sp += 1180 * dt;
        const step = g.sp * dt;
        if (step >= d - 10) {
          p.xp += g.v * p.st.xpMul;
          this.fx && this.fx(FX_EVT.GEM, g.tier, g.x, g.y);
          while (p.xp >= p.need) {
            p.xp -= p.need;
            p.lvl++;
            p.need = CURVES.xpNeed(p.lvl);
            this.pendLv++;
            if (this.hasRelic("r_compound")) this.recalc();
          }
          const last = --this.nG;
          gm[i] = gm[last];
          gm[last] = g;
          continue;
        }
        g.x += (dx / d) * step;
        g.y += (dy / d) * step;
      }
    }
  }

  updPicks(dt) {
    const pk = this.pk;
    const p = this.p;
    for (let i = this.nPK - 1; i >= 0; i--) {
      const item = pk[i];
      item.t += dt;
      const dx = p.x - item.x;
      const dy = p.y - item.y;
      const d2 = dx * dx + dy * dy;
      const pullR = p.st.pickup * 0.75;
      if (item.kind !== PICK_KIND.CHEST && d2 < pullR * pullR && item.t > 0.2) {
        const d = Math.sqrt(d2) || 1;
        item.x += (dx / d) * 265 * dt;
        item.y += (dy / d) * 265 * dt;
      }
      if (d2 < 24 * 24 && item.t > 0.15) {
        const kind = item.kind;
        const n = item.n;
        const ix = item.x;
        const iy = item.y;
        const last = --this.nPK;
        pk[i] = pk[last];
        pk[last] = item;
        this.fx && this.fx(FX_EVT.PICK, kind, ix, iy, n);
        if (kind === PICK_KIND.HEART) p.hp = Math.min(p.maxHp, p.hp + 32);
        else if (kind === PICK_KIND.MAGNET) {
          this.magnetT = 0.85;
          this.fx && this.fx(FX_EVT.MAGNET, p.x, p.y);
        } else if (kind === PICK_KIND.NUKE) this.nuke();
        else if (kind === PICK_KIND.CORE) this.coresPicked += n;
        else if (kind === PICK_KIND.CHEST) {
          this.openChest(n);
          return;
        }
      }
    }
  }

  cores() {
    return Math.floor(
      (this.t * 0.34 + this.kills * 0.075 + this.elites * 7 + this.nodesCaptured * 5 + this.coresPicked) *
        this.p.st.coreMul,
    );
  }

  summary() {
    const p = this.p;
    let topIdx = 0;
    for (let i = 1; i < this.dmgBy.length; i++) {
      if (this.dmgBy[i] > this.dmgBy[topIdx]) topIdx = i;
    }
    return {
      time: this.t,
      kills: this.kills,
      level: p.lvl,
      maxCombo: this.maxCombo,
      cores: this.cores(),
      bosses: this.bosses,
      bossList: this.bossList.slice(),
      elites: this.elites,
      nodesCaptured: this.nodesCaptured,
      synergies: this.synergyList.length,
      synergyList: this.synergyList.slice(),
      relicList: this.relicList.slice(),
      maxOverclock: this.maxOverclock,
      diff: this.diff.id,
      weapons: p.weapons.map((w) => ({
        id: w.id,
        lv: w.lv,
        evo: w.evo,
        oc: w.oc || 0,
        dmg: Math.round(this.dmgBy[w.k]),
      })),
      passives: p.passives.map((ps) => ({ id: ps.id, lv: ps.lv })),
      evos: this.evos.slice(),
      top: WEAPONS[topIdx].id,
      cause: this.lastSrc >= 0 ? ENEMIES[this.lastSrc].name : "",
      causeId: this.lastSrc >= 0 ? ENEMIES[this.lastSrc].id : "",
      char: this.char.id,
      stage: STAGES[this.startStageIdx].id,
      endStage: STAGES[this.stageIdx].id,
      stagesVisited: this.stagesVisited.slice(),
      daily: this.daily,
      rule: this.rule.id || "",
      wave: this.wave + 1,
      hurts: this.hurts,
      killsBy: Array.prototype.slice.call(this.killsBy),
    };
  }
}

export function createAutoBot(smart = 1, rnd = Math.random) {
  const state = { mx: 0, my: 0, think: 0, skillT: 3 };
  const dangerR = smart ? 200 : 105;
  const stepDist = smart ? 44 : 28;
  const DIRS = 16;

  function dir(world) {
    const p = world.p;
    let gx = 0, gy = 0, gWeight = 0, bestGoal = 1e18;

    // Prioritize Interactive Cleanroom Sector Nodes too!
    for (let i = 0; i < world.nZ; i++) {
      const z = world.zn[i];
      if (z.kind !== ZONE_KIND.NODE) continue;
      const dx = z.x - p.x;
      const dy = z.y - p.y;
      const d2 = dx * dx + dy * dy;
      if (d2 < 360 * 360 && d2 < bestGoal) {
        bestGoal = d2 * 0.25;
        const d = Math.sqrt(d2) || 1;
        gx = dx / d;
        gy = dy / d;
        gWeight = 2.8;
      }
    }

    for (let i = 0; i < world.nPK; i++) {
      const pk = world.pk[i];
      const dx = pk.x - p.x;
      const dy = pk.y - p.y;
      const d2 = dx * dx + dy * dy;
      let w =
        pk.kind === PICK_KIND.CHEST ? 3.2 :
        pk.kind === PICK_KIND.HEART ? (p.hp < p.maxHp * 0.6 ? 4 : 0.8) :
        pk.kind === PICK_KIND.NUKE ? 3 :
        pk.kind === PICK_KIND.MAGNET ? 2.4 : 1.2;
      if (!smart) w *= 0.6;
      const score = d2 / (w * w);
      if (d2 < 420 * 420 && score < bestGoal) {
        bestGoal = score;
        const d = Math.sqrt(d2) || 1;
        gx = dx / d;
        gy = dy / d;
        gWeight = w;
      }
    }
    if (gWeight < 2.5) {
      let nearG = 1e18, cx = 0, cy = 0;
      const lim = smart ? 340 : 200;
      for (let i = 0; i < world.nG; i += smart ? 1 : 2) {
        const gm = world.gm[i];
        if (gm.st) continue;
        const dx = gm.x - p.x;
        const dy = gm.y - p.y;
        const d2 = dx * dx + dy * dy;
        if (d2 < nearG) {
          nearG = d2;
          cx = dx;
          cy = dy;
        }
      }
      if (nearG < lim * lim && (gWeight === 0 || nearG < bestGoal)) {
        const d = Math.sqrt(nearG) || 1;
        gx = cx / d;
        gy = cy / d;
        gWeight = smart ? 1.3 : 2;
      }
    }

    let bestScore = -1e18, bx = 0, by = 0;
    for (let d = -1; d < DIRS; d++) {
      const a = (d / DIRS) * 6.2832;
      const vx = d < 0 ? 0 : Math.cos(a);
      const vy = d < 0 ? 0 : Math.sin(a);
      const tx = p.x + vx * stepDist;
      const ty = p.y + vy * stepDist;
      let sc = d < 0 ? -0.15 : 0;

      for (let i = 0; i < world.nE; i++) {
        const e = world.en[i];
        const dx = e.x - tx;
        const dy = e.y - ty;
        if ((dx > dangerR || dx < -dangerR || dy > dangerR || dy < -dangerR) && !e.tele) continue;
        let dist = Math.sqrt(dx * dx + dy * dy) - e.r - 12;
        if (dist < 4) dist = 4;
        if (dist < dangerR) sc -= ((e.boss ? 6 : e.elite ? 2.5 : 1) * 700) / (dist * dist);
      }
      for (let i = 0; i < world.nEB; i++) {
        const eb = world.eb[i];
        const lead = smart ? 0.3 : 0.1;
        const dx = eb.x + eb.vx * lead - tx;
        const dy = eb.y + eb.vy * lead - ty;
        if (dx > 90 || dx < -90 || dy > 90 || dy < -90) continue;
        let dist = Math.sqrt(dx * dx + dy * dy) - eb.r - 10;
        if (dist < 4) dist = 4;
        sc -= (smart ? 1400 : 500) / (dist * dist);
      }
      for (let i = 0; i < world.nZ; i++) {
        const z = world.zn[i];
        if (z.kind === ZONE_KIND.BLAST) {
          const dx = tx - z.x;
          const dy = ty - z.y;
          if (dx * dx + dy * dy < (z.r + 14) * (z.r + 14)) sc -= smart ? 5 : 1.5;
        } else if (z.kind === ZONE_KIND.EBEAM && smart) {
          const ux = Math.cos(z.a + z.av * 0.35);
          const uy = Math.sin(z.a + z.av * 0.35);
          const rx = tx - z.x;
          const ry = ty - z.y;
          const proj = rx * ux + ry * uy;
          if (proj > 0 && proj < z.len && Math.abs(ry * ux - rx * uy) < 46) sc -= 6;
        }
      }
      if (gWeight) sc += gWeight * (vx * gx + vy * gy) * 0.55;
      if (!smart) sc += (rnd() - 0.5) * 0.5;
      if (sc > bestScore) {
        bestScore = sc;
        bx = vx;
        by = vy;
      }
    }
    return [bx, by];
  }

  return {
    move(world, dt) {
      state.think -= dt;
      state.skillT -= dt;
      if (state.skillT <= 0 && world.nE >= 8 && world.skillCd <= 0) {
        state.skillT = 5;
        world.castSkill();
      }
      if (state.think <= 0) {
        state.think = smart ? 0.05 : 0.12;
        const [tx, ty] = dir(world);
        state.mx += (tx - state.mx) * 0.55;
        state.my += (ty - state.my) * 0.55;
      }
      world.input(state.mx, state.my);
    },
    pick(world, choices) {
      let best = 0;
      let bestVal = -9;
      for (let i = 0; i < choices.length; i++) {
        const c = choices[i];
        let v = 0;
        if (c.kind === "evo") v = 12;
        else if (c.kind === "syn") v = 9;
        else if (c.kind === "relic") v = 8;
        else if (c.kind === "w" && !c.isNew) v = 5 + c.lv * 0.5;
        else if (c.kind === "oc") v = 5.5;
        else if (c.kind === "w") v = 4.5;
        else if (c.kind === "p") {
          v = 2.5;
          for (let j = 0; j < world.p.weapons.length; j++) {
            if (WEAPONS[world.p.weapons[j].k].partner === c.id) v = 6.5;
          }
        } else {
          v = 1;
        }
        if (v > bestVal) {
          bestVal = v;
          best = i;
        }
      }
      return best;
    },
  };
}
