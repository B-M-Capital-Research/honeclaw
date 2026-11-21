// HONE Adventure — Zero-GC Simulation World, Spatial Hash Grid, Boss AI & Auto-Play Bot

import {
  AI_TYPE,
  BOOSTS,
  BOSS_IDS,
  BOSS_SEC,
  COMBO_STEPS,
  CURVES,
  DEFAULT_P_SLOTS,
  DEFAULT_W_SLOTS,
  ENEMIES,
  ENEMY_IDX,
  HEROES,
  HERO_IDX,
  MAX_WEAPON_LV,
  PASSIVES,
  PASSIVE_IDX,
  RULES,
  TALENT_BRANCHES,
  WAVE_SEC,
  WEAPONS,
  WEAPON_IDX,
  branchPoints,
  mulberry32,
} from "./data.js";

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
};

export const BULLET_KIND = { BLADE: 0, WAFER: 1 };
export const ZONE_KIND = { MINE: 0, VORTEX: 1, PULSE: 2, BLAST: 3, EBEAM: 4 };
export const PICK_KIND = { HEART: 0, MAGNET: 1, NUKE: 2, CORE: 3, CHEST: 4 };

const MAX_ENEMIES = 440;
const MAX_BULLETS = 320;
const MAX_EBULLETS = 240;
const MAX_GEMS = 420;
const MAX_ZONES = 96;
const MAX_PICKS = 40;
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
    this.talents = opts.talents || {};
    this.rule = (opts.rule && RULES.find((r) => r.id === opts.rule)) || {};
    this.daily = !!opts.daily;
    this.autoLevel = !opts.manualLevel;
    this.intro = !!opts.intro;
    this.t = 0;
    this.over = false;
    this.pending = null;
    this.lockedChoice = null;
    this.uid = 0;

    this.en = makePool(MAX_ENEMIES, () => ({
      x: 0, y: 0, kx: 0, ky: 0, hp: 0, maxHp: 0, type: 0, ai: 0, r: 10, sp: 0, dmg: 0,
      xpv: 1, wt: 1, flash: 0, slow: 0, elite: 0, boss: 0, uid: 0, st: 0, stT: 0,
      a1: 0, a2: 0, ax: 0, ay: 0, ph: 0, pulse: 0, mark: 0, dead: false, gone: false,
      tele: 0, born: 0, swell: 0, face: 1, mvx: 0, mvy: 0, cd: new Float32Array(8),
    }));
    this.nE = 0;

    this.bl = makePool(MAX_BULLETS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, r: 6, dmg: 0, kind: 0, k: 0, life: 0,
      pierce: 0, st: 0, rot: 0, size: 1, pop: 0, t: 0, hits: [],
    }));
    this.nB = 0;

    this.eb = makePool(MAX_EBULLETS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, r: 6, dmg: 0, life: 0, src: 0,
    }));
    this.nEB = 0;

    this.gm = makePool(MAX_GEMS, () => ({
      x: 0, y: 0, vx: 0, vy: 0, v: 1, tier: 0, st: 0, sp: 0, t: 0,
    }));
    this.nG = 0;

    this.zn = makePool(MAX_ZONES, () => ({
      kind: 0, x: 0, y: 0, ox: 0, oy: 0, r: 0, rMax: 0, t: 0, life: 0, arm: 0,
      dmg: 0, tickT: 0, evo: 0, mini: 0, id: 0, a: 0, av: 0, len: 0, st: 0,
      own: null, src: 0, slow: 0, burst: 0, k: 0,
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
      inv: 0, lvl: 1, xp: 0, need: CURVES.xpNeed(1), weapons: [], passives: [], boosts: 0,
      st: {}, moving: false, hurtT: 0,
    });

    this.kills = 0;
    this.killsBy = new Int32Array(ENEMIES.length);
    this.dmgBy = new Float64Array(WEAPONS.length);
    this.combo = 0;
    this.comboT = 0;
    this.maxCombo = 0;
    this.comboIdx = 0;
    this.coresPicked = 0;
    this.elites = 0;
    this.bosses = 0;
    this.bossList = [];
    this.wave = 0;
    this.spawnAcc = 0;
    this.nextElite = 60;
    this.nextBoss = BOSS_SEC;
    this.bossIdx = 0;
    this.bossWarned = false;
    this.boss = null;
    this.pendLv = 0;
    this.pulseId = 0;
    this.stamp = 0;
    this.magnetT = 0;
    this.lastMagnet = -20;
    this.lastNuke = -30;
    this.overXp = 0;
    this.revives = this.talents.revive ? 1 : 0;
    this.rerolls = 1 + (this.talents.reroll || 0);
    this.wipeEB = false;
    this.lastSrc = -1;
    this.hurts = 0;
    this.evos = [];
    this.near = null;
    this.nearD = 1e9;
    this.tutDone = false;
    this.sx = 0;
    this.sy = 0;
    this.hpMulNow = 1;
    this.dmgMulNow = 1;
    this.spMulNow = 1;
    this.ghosts = 0;

    this.recalc();
    p.hp = p.maxHp;
    this.addWeapon(opts.weapon || this.char.weapon);

    // Economy branch milestone (>=6 pts): instant early level-up headstart!
    if (branchPoints(this.talents, "economy") >= TALENT_BRANCHES[2].milestone && !this.daily) {
      p.xp = p.need;
      p.xp -= p.need;
      p.lvl++;
      p.need = CURVES.xpNeed(p.lvl);
      this.pendLv++;
    }

    for (let i = 0; i < 7; i++) {
      const ang = ((i + this.rnd() * 0.6) / 7) * TAU;
      const dist = 150 + this.rnd() * 80;
      this.spawnEnemy(ENEMY_IDX.glitch, Math.cos(ang) * dist, Math.sin(ang) * dist, 0);
    }
  }

  plv(id) {
    for (let i = 0; i < this.p.passives.length; i++) {
      if (this.p.passives[i].id === id) return this.p.passives[i].lv;
    }
    return 0;
  }

  recalc() {
    const p = this.p;
    const st = p.st;
    const mod = this.char.mod;
    const tal = this.talents;
    const rule = this.rule;

    const offenseMilestone = branchPoints(tal, "offense") >= TALENT_BRANCHES[0].milestone ? 1.08 : 1;
    const defenseMilestone = branchPoints(tal, "defense") >= TALENT_BRANCHES[1].milestone ? 2 : 0;

    st.might =
      (1 + 0.07 * (tal.might || 0) + 0.13 * this.plv("leverage") + 0.07 * p.boosts) *
      offenseMilestone *
      (rule.might || 1);
    st.maxHp = Math.round(
      (100 + 15 * (tal.hp || 0) + 28 * this.plv("margin") + (mod.maxHp || 0)) * (rule.hpMul || 1),
    );
    st.armor = this.plv("margin") + defenseMilestone;
    st.speed =
      152 * (1 + 0.035 * (tal.speed || 0) + 0.09 * this.plv("momentum") + (mod.speed || 0)) * (rule.speedAll || 1);
    st.pickup =
      64 * (1 + 0.14 * (tal.pickup || 0) + 0.4 * this.plv("siphon") + (mod.pickup || 0)) * (rule.pickupMul || 1);
    st.xpMul =
      (1 + 0.05 * (tal.pickup || 0) + 0.06 * this.plv("siphon") + (mod.xpMul || 0)) * (rule.xpMul || 1);
    st.cd = Math.max(0.38, 1 - 0.03 * (tal.haste || 0) - 0.1 * this.plv("haste") + (mod.cd || 0));
    st.area = 1 + 0.15 * this.plv("area");
    st.crit = 0.05 + 0.03 * (tal.crit || 0) + 0.1 * this.plv("sharpe") + (mod.crit || 0);
    st.critMul = 2.0 + 0.06 * (tal.crit || 0) + 0.18 * this.plv("sharpe");
    st.dur = 1 + 0.16 * this.plv("compound");
    st.pspd = 1 + 0.12 * this.plv("compound");
    st.regen = 0.7 * this.plv("dividend");
    st.coreMul = 1 + 0.12 * (tal.core || 0);
    st.thorns = mod.thorns || 0;

    if (st.maxHp > p.maxHp && p.maxHp > 0) p.hp += st.maxHp - p.maxHp;
    p.maxHp = st.maxHp;
    if (p.hp > p.maxHp) p.hp = p.maxHp;
  }

  hasWeapon(id) {
    for (let i = 0; i < this.p.weapons.length; i++) {
      if (this.p.weapons[i].id === id) return this.p.weapons[i];
    }
    return null;
  }

  addWeapon(id) {
    const k = WEAPON_IDX[id];
    const w = {
      id, k, lv: 1, evo: false, t: 0.28, ang: this.rnd() * TAU, ph: 0, on: 0, onT: 0,
      vis: 0, bt: 0, tick: 0, extra: 0, extraT: 0, waveT: 2, pos: new Float32Array(32), np: 0,
    };
    this.p.weapons.push(w);
    return w;
  }

  wslots() {
    return this.rule.slots || DEFAULT_W_SLOTS;
  }

  evoLv() {
    return this.rule.evoLv || MAX_WEAPON_LV;
  }

  canEvolve(w) {
    return !w.evo && w.lv >= this.evoLv() && this.plv(WEAPONS[w.k].partner) > 0;
  }

  buildPool() {
    const p = this.p;
    const pool = [];
    const canAddW = p.weapons.length < this.wslots();
    const canAddP = p.passives.length < DEFAULT_P_SLOTS;

    for (let i = 0; i < WEAPONS.length; i++) {
      const def = WEAPONS[i];
      const cur = this.hasWeapon(def.id);
      if (cur) {
        if (cur.lv < MAX_WEAPON_LV && !cur.evo) {
          pool.push({ kind: "w", id: def.id, lv: cur.lv + 1, isNew: false, wt: 3.1 });
        }
      } else if (canAddW) {
        pool.push({ kind: "w", id: def.id, lv: 1, isNew: true, wt: this.plv(def.partner) > 0 ? 2.7 : 1.8 });
      }
    }

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
        pool.push({ kind: "p", id: def.id, lv: 1, isNew: true, wt: isPartner ? 3.5 : 1.6 });
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
    if (c.kind === "w") {
      const w = this.hasWeapon(c.id);
      if (w) return !w.evo && w.lv < MAX_WEAPON_LV;
      return this.p.weapons.length < this.wslots();
    }
    if (c.kind === "p") {
      const lv = this.plv(c.id);
      if (lv > 0) return lv < PASSIVES[PASSIVE_IDX[c.id]].max;
      return this.p.passives.length < DEFAULT_P_SLOTS;
    }
    return true;
  }

  refreshChoice(c) {
    if (c.kind === "w") {
      const w = this.hasWeapon(c.id);
      return { ...c, lv: w ? w.lv + 1 : 1, isNew: !w };
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
        this.fx && this.fx(FX_EVT.EVO, w.k, p.x, p.y);
      }
    } else if (c.id === "b_might") {
      p.boosts++;
      this.recalc();
    } else if (c.id === "b_heal") {
      p.hp = Math.min(p.maxHp, p.hp + p.maxHp * 0.38);
      this.fx && this.fx(FX_EVT.HEAL, p.x, p.y);
    } else if (c.id === "b_core") {
      this.coresPicked += 15;
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
        const upgrades = this.buildPool().filter((x) => !x.isNew);
        const candidates = upgrades.length ? upgrades : this.buildPool();
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
    const scale = def.boss ? Math.pow(1.8, Math.floor(this.bossIdx / 4)) : this.hpMulNow;
    e.maxHp = e.hp = Math.round(def.hp * scale * (elite ? 12 : 1));
    e.r = def.r * (elite ? 1.6 : 1);
    e.sp = def.sp * (def.boss ? 1 : this.spMulNow) * (this.rule.speedAll || 1);
    e.dmg = def.dmg * (elite ? 1.5 : 1);
    e.xpv = def.xp * (elite ? 10 : 1);
    e.wt = def.wt * (elite ? 4 : 1);
    e.flash = 0;
    e.slow = 0;
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
    for (let i = 0; i < 8; i++) e.cd[i] = -9;
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
    const ramp = Math.min(1, (t - def.from) / 20 + 0.15);
    return def.k === 0 ? Math.max(3.5, 10 - t / 50) : def.w * ramp;
  }

  pickType() {
    const t = this.t;
    let total = 0;
    for (let i = 0; i < 7; i++) {
      const def = ENEMIES[i];
      if (t >= def.from) total += this.typeW(def, t);
    }
    let roll = this.rnd() * total;
    for (let i = 0; i < 7; i++) {
      const def = ENEMIES[i];
      if (t >= def.from) {
        roll -= this.typeW(def, t);
        if (roll <= 0) return i;
      }
    }
    return 0;
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
    return z;
  }

  shootE(x, y, dx, dy, sp, r, dmg, src) {
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
    b.t = 0;
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
    for (let i = 0; i < GRID_W * GRID_H; i++) gStart[i + 1] += gStart[i];
    const gCur = this.gCur;
    for (let i = 0; i < GRID_W * GRID_H; i++) gCur[i] = gStart[i];
    for (let i = 0; i < nE; i++) this.gItems[gCur[gCell[i]]++] = i;
  }

  query(x, y, rad) {
    let x0 = ((x - rad - 20 - this.gox) / CELL_SIZE) | 0;
    let x1 = ((x + rad + 20 - this.gox) / CELL_SIZE) | 0;
    let y0 = ((y - rad - 20 - this.goy) / CELL_SIZE) | 0;
    let y1 = ((y + rad + 20 - this.goy) / CELL_SIZE) | 0;
    if (x0 < 0) x0 = 0;
    if (y0 < 0) y0 = 0;
    if (x1 >= GRID_W) x1 = GRID_W - 1;
    if (y1 >= GRID_H) y1 = GRID_H - 1;
    const gStart = this.gStart;
    const gItems = this.gItems;
    const en = this.en;
    const q = this.q;
    let count = 0;
    for (let cy = y0; cy <= y1; cy++) {
      const row = cy * GRID_W;
      for (let cx = x0; cx <= x1; cx++) {
        const cell = row + cx;
        for (let idx = gStart[cell], end = gStart[cell + 1]; idx < end; idx++) {
          const ei = gItems[idx];
          const e = en[ei];
          if (e.dead) continue;
          const dx = e.x - x;
          const dy = e.y - y;
          const rr = rad + e.r;
          if (dx * dx + dy * dy <= rr * rr) q[count++] = ei;
        }
      }
    }
    return count;
  }

  hit(e, baseDmg, weaponK, kb, kx, ky) {
    if (e.dead) return 0;
    const st = this.p.st;
    let dmg = baseDmg * st.might;
    let crit = 0;
    if (this.rnd() < st.crit) {
      dmg *= st.critMul;
      crit = 1;
    }
    dmg = Math.max(1, Math.round(dmg * (0.9 + 0.2 * this.rnd())));
    e.hp -= dmg;
    e.flash = 0.1;
    this.dmgBy[weaponK] += dmg;
    if (kb) {
      const f = kb / e.wt;
      e.kx += kx * f;
      e.ky += ky * f;
    }
    this.fx && this.fx(FX_EVT.HIT, e.x, e.y, dmg, crit, weaponK);
    if (e.hp <= 0) this.kill(e, crit);
    return dmg;
  }

  explode(x, y, rad, dmg, weaponK, kb) {
    const n = this.query(x, y, rad);
    for (let i = 0; i < n; i++) {
      const e = this.en[this.q[i]];
      const dx = e.x - x;
      const dy = e.y - y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      this.hit(e, dmg, weaponK, kb, dx / d, dy / d);
    }
    this.fx && this.fx(FX_EVT.BOOM, x, y, rad, weaponK);
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
    e.dead = true;
    if (e.gone) return;
    const p = this.p;
    this.kills++;
    this.killsBy[e.type]++;
    this.combo++;
    this.comboT = 2.1;
    if (this.combo > this.maxCombo) this.maxCombo = this.combo;
    if (this.comboIdx < COMBO_STEPS.length && this.combo >= COMBO_STEPS[this.comboIdx]) {
      this.fx && this.fx(FX_EVT.COMBO, this.combo, this.comboIdx);
      this.comboIdx++;
    }
    this.addGem(e.x, e.y, e.xpv);
    if (e.elite) {
      this.elites++;
      this.addPickup(PICK_KIND.CHEST, e.x, e.y, 1);
    }
    if (e.type === ENEMY_IDX.bubble_mob) {
      for (let i = 0; i < 3; i++) {
        const a = this.rnd() * TAU;
        const sub = this.spawnEnemy(ENEMY_IDX.mini, e.x + Math.cos(a) * 10, e.y + Math.sin(a) * 10, 0);
        if (sub) {
          sub.kx = Math.cos(a) * 165;
          sub.ky = Math.sin(a) * 165;
        }
      }
    } else if (e.type === ENEMY_IDX.margin_bomb) {
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
    if (roll < 0.013) {
      if (p.hp < p.maxHp * 0.92) this.addPickup(PICK_KIND.HEART, e.x, e.y, 1);
    } else if (roll < 0.044) {
      this.addPickup(PICK_KIND.CORE, e.x, e.y, 1 + ((this.rnd() * 3) | 0));
    } else if (roll < 0.049 && this.t - this.lastMagnet > 35) {
      this.lastMagnet = this.t;
      this.addPickup(PICK_KIND.MAGNET, e.x, e.y, 1);
    } else if (roll < 0.052 && this.t - this.lastNuke > 45) {
      this.lastNuke = this.t;
      this.addPickup(PICK_KIND.NUKE, e.x, e.y, 1);
    }
    this.fx && this.fx(FX_EVT.KILL, e.x, e.y, e.type, e.boss ? 2 : e.elite ? 1 : 0, crit);
    if (e.boss) this.bossDead(e);
  }

  bossDead(e) {
    this.bosses++;
    this.bossList.push(ENEMIES[e.type].id);
    this.boss = null;
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
    this.magnetT = 0.65;
  }

  hurt(rawDmg, src) {
    const p = this.p;
    if (p.inv > 0 || this.over) return;
    const dmg = Math.max(1, Math.round(rawDmg * this.dmgMulNow) - p.st.armor);
    p.hp -= dmg;
    p.inv = 0.6;
    this.lastSrc = src;
    this.hurts++;
    if (this.combo >= 10) this.fx && this.fx(FX_EVT.COMBOEND, this.combo);
    this.combo = 0;
    this.comboIdx = 0;
    this.comboT = 0;
    this.fx && this.fx(FX_EVT.HURT, dmg, p.x, p.y);
    if (p.st.thorns) this.nova(100, 390, 14);
    if (p.hp <= 0) {
      if (this.revives > 0) {
        this.revives--;
        p.hp = Math.round(p.maxHp * 0.5);
        p.inv = 2.5;
        this.nova(225, 620, 65);
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

    const sp = p.st.speed;
    p.x += p.mx * sp * dt;
    p.y += p.my * sp * dt;
    p.moving = p.mx * p.mx + p.my * p.my > 0.01;
    if (p.mx > 0.15) p.face = 1;
    else if (p.mx < -0.15) p.face = -1;
    if (p.inv > 0) p.inv -= dt;
    if (p.st.regen > 0 && p.hp < p.maxHp) p.hp = Math.min(p.maxHp, p.hp + p.st.regen * dt);
    if (this.magnetT > 0) this.magnetT -= dt;

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
    let bestD2 = 385 * 385;
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
    let target = CURVES.target(t) * countMul;
    if (this.boss) target *= 0.6;
    this.spawnAcc = Math.min(8, this.spawnAcc + CURVES.rate(t) * countMul * dt);
    let loops = 0;
    while (this.spawnAcc >= 1 && loops++ < 10) {
      this.spawnAcc -= 1;
      if (this.nE - this.ghosts < target) {
        this.spawnPos();
        this.spawnEnemy(this.pickType(), this.sx, this.sy, 0);
      }
    }
    if (this.intro && !this.tutDone && t >= 12) {
      this.tutDone = true;
      for (let i = 0; i < 36; i++) {
        const a = (i / 36) * TAU;
        const r = 175 + (i % 3) * 22;
        this.spawnEnemy(ENEMY_IDX.glitch, p.x + Math.cos(a) * r, p.y + Math.sin(a) * r * 1.25, 0);
      }
      this.addPickup(PICK_KIND.NUKE, p.x + p.face * 34, p.y - 4, 1);
      this.lastNuke = t;
    }
    if (t >= this.nextElite) {
      const eliteMul = this.rule.eliteMul || 1;
      this.nextElite += (t < 300 ? 120 : 40) / eliteMul;
      const pool = [ENEMY_IDX.spam];
      if (t >= 180) pool.push(ENEMY_IDX.crash);
      if (t >= 140) pool.push(ENEMY_IDX.rumor);
      if (t >= 220) pool.push(ENEMY_IDX.bubble_mob);
      this.spawnPos();
      const el = this.spawnEnemy(pool[(this.rnd() * pool.length) | 0], this.sx, this.sy, 1);
      if (el && this.fx) this.fx(FX_EVT.ELITE, el.x, el.y);
    }
    if (!this.bossWarned && t >= this.nextBoss - 3) {
      this.bossWarned = true;
      this.fx && this.fx(FX_EVT.BOSSWARN, this.bossIdx % 4);
    }
    if (t >= this.nextBoss) {
      this.nextBoss += BOSS_SEC;
      this.bossWarned = false;
      const b = this.spawnEnemy(ENEMY_IDX[BOSS_IDS[this.bossIdx % 4]], p.x, p.y - this.vh / 2 - 50, 0);
      this.bossIdx++;
      if (b) {
        this.boss = b;
        b.stT = 2;
        b.a1 = 5;
        this.fx && this.fx(FX_EVT.BOSS, b.x, b.y, b.boss);
      }
    }
  }

  onWave(w) {
    if (w <= 0) return;
    this.fx && this.fx(FX_EVT.WAVE, w + 1);
    const p = this.p;
    const mode = w % 3;
    if (mode === 1) {
      const cnt = Math.min(44, 12 + w * 2);
      const tp = this.t > 200 && this.rnd() < 0.5 ? ENEMY_IDX.fomo : ENEMY_IDX.glitch;
      for (let i = 0; i < cnt; i++) {
        const a = (i / cnt) * TAU;
        this.spawnEnemy(tp, p.x + Math.cos(a) * (this.vw / 2 + 50), p.y + Math.sin(a) * (this.vw / 2 + 50) * 1.5, 0);
      }
    } else if (mode === 2) {
      const cnt = Math.min(30, 9 + w);
      const dir = this.rnd() < 0.5 ? 1 : -1;
      const span = this.vh * 0.36;
      for (let i = 0; i < cnt; i++) {
        const g = this.spawnEnemy(
          ENEMY_IDX.hallucination,
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
      const cnt = Math.min(36, 10 + Math.floor(w * 1.5));
      const ang = this.rnd() * TAU;
      const dist = this.vh / 2 + 60;
      for (let i = 0; i < cnt; i++) {
        const rr = 70 * Math.sqrt(this.rnd());
        const ra = this.rnd() * TAU;
        this.spawnEnemy(
          ENEMY_IDX.fomo,
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
          if (e.stT <= 0 && dist < 300) {
            e.stT = e.elite ? 1.5 : 2.8;
            e.swell = 0.25;
            this.shootE(e.x, e.y, nx, ny, 114, 6, 10, e.type);
            this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 0);
          }
          break;
        case AI_TYPE.DASH:
          if (e.st === 0) {
            if (dist < 150) {
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

  bossAI(e, dt, dist, nx, ny) {
    const baseSp = e.sp * (e.slow > 0 ? 0.7 : 1);
    e.mvx = nx;
    e.mvy = ny;
    const catchup = dist > 520 ? 2.2 : 1;
    const rage = e.hp < e.maxHp * 0.4;

    if (e.boss === 1) {
      e.stT -= dt;
      e.a1 -= dt;
      if (e.stT < 0.45) e.swell = 0.3;
      if (e.stT <= 0) {
        const cnt = rage ? 16 : 12;
        const off = this.rnd() * TAU;
        for (let i = 0; i < cnt; i++) {
          const a = off + (i / cnt) * TAU;
          this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 106, 7, 12, e.type);
        }
        e.stT = rage ? 2.3 : 3.6;
        this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 1);
      }
      if (e.a1 <= 0) {
        e.a1 = 9;
        for (let i = 0; i < 8; i++) {
          const a = (i / 8) * TAU;
          this.spawnEnemy(ENEMY_IDX.glitch, e.x + Math.cos(a) * 64, e.y + Math.sin(a) * 64, 0);
        }
      }
      return baseSp * (rage ? 1.35 : 1) * catchup;
    }

    if (e.boss === 2) {
      e.stT -= dt;
      if (e.st === 0) {
        if (e.stT <= 0) {
          e.st = 1;
          e.stT = 0.7;
          e.ax = nx;
          e.ay = ny;
          e.tele = 330;
          e.a1 = 0;
        }
        return baseSp * catchup;
      }
      if (e.st === 1) {
        if (e.stT <= 0) {
          e.st = 2;
          e.stT = 0.55;
          e.tele = 0;
          this.fx && this.fx(FX_EVT.DASH, e.x, e.y);
        }
        return 0;
      }
      if (e.st === 2) {
        e.mvx = e.ax;
        e.mvy = e.ay;
        if (e.stT <= 0) {
          e.a1++;
          if (e.a1 < 3) {
            e.st = 1;
            e.stT = rage ? 0.4 : 0.55;
            e.ax = nx;
            e.ay = ny;
            e.tele = 330;
          } else {
            e.st = 3;
            e.stT = 0.5;
            e.a2 = 0;
          }
        }
        return 435;
      }
      if (e.stT <= 0) {
        if (e.a2 < 2) {
          const baseA = Math.atan2(ny, nx);
          const cnt = rage ? 11 : 9;
          for (let i = 0; i < cnt; i++) {
            const a = baseA + (i - (cnt - 1) / 2) * 0.2;
            this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 136, 7, 13, e.type);
          }
          e.a2++;
          e.stT = 0.55;
          e.swell = 0.3;
          this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 1);
        } else {
          e.st = 0;
          e.stT = 1.5;
        }
      }
      return baseSp * 0.3;
    }

    if (e.boss === 3) {
      e.stT -= dt;
      if (e.st === 0) {
        e.a2 -= dt;
        if (e.a2 <= 0) {
          e.a2 = rage ? 0.11 : 0.15;
          e.a1 += 0.42;
          for (let i = 0; i < 2; i++) {
            const a = e.a1 + i * Math.PI;
            this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 102, 6, 12, e.type);
          }
        }
        if (e.stT <= 0) {
          e.st = 1;
          e.stT = 0.4;
          e.a2 = 0;
        }
        return baseSp * 0.5 * catchup;
      }
      if (e.st === 1) {
        if (e.stT <= 0) {
          if (e.a2 < 2) {
            const p = this.p;
            for (let i = 0; i < 3; i++) {
              const z = this.addZone();
              if (!z) break;
              z.kind = ZONE_KIND.BLAST;
              z.x = p.x + (i ? (this.rnd() - 0.5) * 190 : 0);
              z.y = p.y + (i ? (this.rnd() - 0.5) * 190 : 0);
              z.r = 58;
              z.life = 0.95;
              z.dmg = 22;
              z.src = e.type;
            }
            e.a2++;
            e.stT = 1.15;
            e.swell = 0.3;
          } else {
            e.st = 2;
            e.stT = 1.2;
            for (let i = 0; i < 6; i++) {
              const a = (i / 6) * TAU;
              this.spawnEnemy(ENEMY_IDX.fomo, e.x + Math.cos(a) * 60, e.y + Math.sin(a) * 60, 0);
            }
          }
        }
        return baseSp * catchup;
      }
      if (e.stT <= 0) {
        e.st = 0;
        e.stT = 3.2;
        e.a2 = 0;
      }
      return baseSp * catchup;
    }

    // Boss 4: Black Swan Leviathan
    e.stT -= dt;
    if (e.st === 0) {
      if (e.stT <= 0) {
        const z = this.addZone();
        if (z) {
          const dir = this.rnd() < 0.5 ? 1 : -1;
          z.kind = ZONE_KIND.EBEAM;
          z.own = e;
          z.x = e.x;
          z.y = e.y;
          z.a = Math.atan2(ny, nx) - dir * 1.0;
          z.av = dir * (rage ? 1.02 : 0.84);
          z.len = 560;
          z.r = 11;
          z.life = 3.3;
          z.arm = 1.0;
          z.dmg = 24;
          z.src = e.type;
        }
        e.st = 1;
        e.stT = 3.4;
      }
      return baseSp * catchup;
    }
    if (e.st === 1) {
      if (e.stT <= 0) {
        e.st = 2;
        e.stT = 0.5;
        e.a2 = 0;
      }
      return 0;
    }
    if (e.st === 2) {
      if (e.stT <= 0) {
        if (e.a2 < 3) {
          const off = e.a2 * 0.22;
          for (let i = 0; i < 14; i++) {
            const a = off + (i / 14) * TAU;
            this.shootE(e.x, e.y, Math.cos(a), Math.sin(a), 124, 7, 14, e.type);
          }
          e.a2++;
          e.stT = 0.7;
          e.swell = 0.3;
          this.fx && this.fx(FX_EVT.ESHOT, e.x, e.y, 1);
        } else {
          e.st = 3;
          e.stT = 0.65;
          e.ax = nx;
          e.ay = ny;
          e.tele = 345;
          e.a1 = 0;
        }
      }
      return baseSp * 0.6 * catchup;
    }
    if (e.st === 3) {
      if (e.stT <= 0) {
        e.st = 4;
        e.stT = 0.5;
        e.tele = 0;
        this.fx && this.fx(FX_EVT.DASH, e.x, e.y);
      }
      return 0;
    }
    e.mvx = e.ax;
    e.mvy = e.ay;
    if (e.stT <= 0) {
      e.a1++;
      if (e.a1 < 2) {
        e.st = 3;
        e.stT = 0.55;
        e.ax = nx;
        e.ay = ny;
        e.tele = 345;
      } else {
        e.st = 0;
        e.stT = 1.4;
      }
    }
    return 445;
  }

  separate() {
    const gStart = this.gStart;
    const gItems = this.gItems;
    const en = this.en;
    for (let cy = 0; cy < GRID_H; cy++) {
      for (let cx = 0; cx < GRID_W; cx++) {
        const cell = cy * GRID_W + cx;
        const s0 = gStart[cell];
        const s1 = gStart[cell + 1];
        if (s0 === s1) continue;
        for (let i = s0; i < s1; i++) {
          const a = en[gItems[i]];
          if (a.dead || a.ai === AI_TYPE.GHOST) continue;
          for (let j = i + 1; j < s1; j++) this.push(a, en[gItems[j]]);
          if (cx + 1 < GRID_W) {
            for (let j = gStart[cell + 1], e2 = gStart[cell + 2]; j < e2; j++) this.push(a, en[gItems[j]]);
          }
          if (cy + 1 < GRID_H) {
            const below = cell + GRID_W;
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
      w.t -= dt;

      switch (def.id) {
        case "blade":
          if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.1;
              break;
            }
            w.t = cfg.cd * st.cd;
            const aim = Math.atan2(p.aimY, p.aimX);
            const sp = 350 * st.pspd;
            for (let n = 0; n < cfg.n; n++) {
              const a = aim + (w.evo ? (this.rnd() - 0.5) * 0.46 : (n - (cfg.n - 1) / 2) * 0.17);
              const vx = Math.cos(a);
              const vy = Math.sin(a);
              const b = this.addBullet(
                BULLET_KIND.BLADE, k, p.x + vx * 12, p.y + vy * 12,
                vx * sp, vy * sp, w.evo ? 8.5 : 7, cfg.dmg, cfg.pierce, 1.15 * st.dur,
              );
              if (b && w.evo) {
                b.pop = cfg.pop * st.area;
                b.size = 1.28;
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        case "wafer":
          if (w.evo) {
            w.ang += 2.7 * dt;
            w.ph += 1.15 * dt;
            w.np = cfg.n;
            const hr = 13 * cfg.size * Math.sqrt(st.area);
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
                if (this.t - e.cd[k] >= 0.3) {
                  e.cd[k] = this.t;
                  this.hit(e, cfg.dmg, k, 150, -Math.sin(a), Math.cos(a));
                }
              }
            }
          } else if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.15;
              break;
            }
            w.t = cfg.cd * st.cd;
            for (let n = 0; n < cfg.n; n++) {
              const tgt = n === 0 ? this.near : this.randNear(260) || this.near;
              let dx = tgt.x - p.x;
              let dy = tgt.y - p.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              dx /= d;
              dy /= d;
              if (n > 0) {
                const a = Math.atan2(dy, dx) + (this.rnd() - 0.5) * 0.5;
                dx = Math.cos(a);
                dy = Math.sin(a);
              }
              const b = this.addBullet(
                BULLET_KIND.WAFER, k, p.x, p.y, dx * 320, dy * 320,
                11 * cfg.size * Math.sqrt(st.area), cfg.dmg, 999, 6,
              );
              if (b) {
                b.t = (cfg.range * st.area) / 320;
                b.size = cfg.size * Math.sqrt(st.area);
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
          }
          break;

        case "moat": {
          let active = true;
          if (!w.evo && cfg.off > 0) {
            w.onT -= dt;
            if (w.onT <= 0) {
              w.on = w.on ? 0 : 1;
              w.onT = w.on ? cfg.on * st.dur : cfg.off * st.cd;
              if (w.on && this.fx) this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
            }
            active = !!w.on;
          } else {
            w.on = 1;
          }
          w.vis += ((active ? 1 : 0) - w.vis) * Math.min(1, dt * 10);
          w.ang += 3.35 * dt;
          const n1 = cfg.n;
          const n2 = w.evo ? cfg.n2 : 0;
          w.np = n1 + n2;
          for (let n = 0; n < n1 + n2; n++) {
            const inner = n < n1;
            const a = inner ? w.ang + (n / n1) * TAU : -w.ang * 0.72 + ((n - n1) / n2) * TAU;
            const rad = (inner ? cfg.rad : cfg.rad2) * st.area * (0.5 + 0.5 * w.vis);
            const sx = p.x + Math.cos(a) * rad;
            const sy = p.y + Math.sin(a) * rad;
            w.pos[n * 2] = sx;
            w.pos[n * 2 + 1] = sy;
            if (!active) continue;
            const cnt = this.query(sx, sy, 9.5);
            for (let j = 0; j < cnt; j++) {
              const e = en[this.q[j]];
              if (this.t - e.cd[k] < 0.4) continue;
              e.cd[k] = this.t;
              const dx = e.x - p.x;
              const dy = e.y - p.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              this.hit(e, cfg.dmg, k, 275, dx / d, dy / d);
            }
          }
          if (w.evo) {
            w.waveT -= dt;
            if (w.waveT <= 0) {
              w.waveT = cfg.wave * st.cd;
              this.pulse(k, cfg.waveDmg, cfg.waveR * st.area, 0, 300);
            }
          }
          break;
        }

        case "arc":
          if (w.t <= 0) {
            let fired = 0;
            for (let n = 0; n < cfg.n; n++) {
              let cur = n === 0 && !w.evo ? this.near : this.randNear(w.evo ? 335 : 285);
              if (!cur || (n === 0 && this.nearD > 305 && !w.evo)) break;
              fired++;
              const mark = ++this.stamp;
              let sx = p.x;
              let sy = p.y;
              for (let c = 0; c <= cfg.chain && cur; c++) {
                cur.mark = mark;
                const dx = cur.x - sx;
                const dy = cur.y - sy;
                const d = Math.sqrt(dx * dx + dy * dy) || 1;
                this.fx && this.fx(FX_EVT.BOLT, sx, sy, cur.x, cur.y, c);
                const tx = cur.x;
                const ty = cur.y;
                this.hit(cur, cfg.dmg, k, 70, dx / d, dy / d);
                if (w.evo && c === 0) this.explode(tx, ty, cfg.boom * st.area, cfg.dmg * 0.5, k, 120);
                sx = tx;
                sy = ty;
                const cnt = this.query(tx, ty, 120 * st.area);
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
              w.t = cfg.cd * st.cd;
              this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
            } else {
              w.t = 0.15;
            }
          }
          break;

        case "pulse":
          if (w.extra > 0) {
            w.extraT -= dt;
            if (w.extraT <= 0) {
              w.extra--;
              w.extraT = 0.28;
              this.pulse(k, cfg.dmg, cfg.rad * st.area, 0, 320);
            }
          }
          if (w.t <= 0) {
            if (!w.evo && this.nearD > cfg.rad * st.area * 1.25) {
              w.t = 0.15;
              break;
            }
            w.t = cfg.cd * st.cd;
            this.pulse(k, cfg.dmg, cfg.rad * st.area, w.evo ? cfg.slow : 0, w.evo ? 385 : 320);
            if (cfg.n > 1) {
              w.extra = cfg.n - 1;
              w.extraT = 0.28;
            }
            if (w.evo) {
              const pullR = cfg.rad * st.area * 1.55;
              for (let g = 0; g < this.nG; g++) {
                const gm = this.gm[g];
                const dx = gm.x - p.x;
                const dy = gm.y - p.y;
                if (dx * dx + dy * dy < pullR * pullR) gm.st = 1;
              }
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
          }
          break;

        case "mine":
          if (w.t <= 0) {
            if (this.nearD > 335) {
              w.t = 0.2;
              break;
            }
            w.t = cfg.cd * st.cd;
            for (let n = 0; n < cfg.n; n++) {
              const z = this.addZone();
              if (!z) break;
              const a = this.rnd() * TAU;
              const dist = n === 0 ? 0 : 55 + this.rnd() * 45;
              z.kind = ZONE_KIND.MINE;
              z.k = k;
              z.ox = p.x;
              z.oy = p.y;
              z.x = p.x + Math.cos(a) * dist;
              z.y = p.y + Math.sin(a) * dist;
              z.arm = 0.35;
              z.life = 2.6 * st.dur;
              z.r = cfg.rad * st.area;
              z.dmg = cfg.dmg;
              z.evo = w.evo ? cfg.mini : 0;
              z.rMax = w.evo ? cfg.miniR * st.area : 0;
            }
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
          }
          break;

        case "laser":
          if (w.evo) {
            w.ang += cfg.spin * dt;
            w.bt = 1;
            w.np = cfg.n;
          } else if (w.bt > 0) {
            w.bt -= dt;
            let diff = Math.atan2(p.aimY, p.aimX) - w.ang;
            while (diff > Math.PI) diff -= TAU;
            while (diff < -Math.PI) diff += TAU;
            w.ang += Math.max(-1.35 * dt, Math.min(1.35 * dt, diff));
          } else if (w.t <= 0) {
            if (!this.near) {
              w.t = 0.15;
              break;
            }
            w.bt = cfg.dur * st.dur;
            w.t = cfg.cd * st.cd + w.bt;
            w.ang = Math.atan2(p.aimY, p.aimX);
            w.tick = 0;
            w.np = cfg.n;
            this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, 0);
          }
          if (w.bt > 0) {
            w.tick -= dt;
            if (w.tick <= 0) {
              w.tick = 0.1;
              const len = cfg.len * st.area;
              const hw = cfg.wid * 0.5 * Math.sqrt(st.area);
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
                    this.hit(e, cfg.dmg, k, 45, ux, uy);
                  }
                }
              }
            }
          }
          break;

        case "vortex":
          if (w.t <= 0) {
            let spawned = 0;
            for (let n = 0; n < cfg.n; n++) {
              let tgt = this.randNear(255);
              if (!tgt) break;
              if (w.evo) {
                let bestDensity = -1;
                for (let s = 0; s < 6; s++) {
                  const cand = this.randNear(275);
                  if (!cand) break;
                  const dens = this.query(cand.x, cand.y, 82);
                  if (dens > bestDensity) {
                    bestDensity = dens;
                    tgt = cand;
                  }
                }
              }
              const z = this.addZone();
              if (!z) break;
              z.kind = ZONE_KIND.VORTEX;
              z.k = k;
              z.x = tgt.x;
              z.y = tgt.y;
              z.r = cfg.rad * st.area;
              z.life = cfg.dur * st.dur;
              z.dmg = cfg.dmg;
              z.evo = w.evo ? 1 : 0;
              z.burst = w.evo ? cfg.burst : 0;
              z.a = this.rnd() * TAU;
              spawned++;
            }
            if (spawned) {
              w.t = cfg.cd * st.cd;
              this.fx && this.fx(FX_EVT.SHOT, k, p.x, p.y, w.evo ? 1 : 0);
            } else {
              w.t = 0.25;
            }
          }
          break;
        default:
          break;
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
      if (b.kind === BULLET_KIND.BLADE) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        const cnt = this.query(b.x, b.y, b.r);
        for (let j = 0; j < cnt && !dead; j++) {
          const e = en[this.q[j]];
          if (b.hits.indexOf(e.uid) >= 0) continue;
          b.hits.push(e.uid);
          const sp = Math.sqrt(b.vx * b.vx + b.vy * b.vy) || 1;
          this.hit(e, b.dmg, b.k, 115, b.vx / sp, b.vy / sp);
          if (b.hits.length > b.pierce) dead = true;
        }
        if (b.life <= 0) dead = true;
        if (dead) {
          if (b.pop > 0) this.explode(b.x, b.y, b.pop, b.dmg * 0.65, b.k, 95);
          else this.fx && this.fx(FX_EVT.POP, b.x, b.y, b.k);
        }
      } else {
        b.rot += 14 * dt;
        if (b.st === 0) {
          b.t -= dt;
          if (b.t <= 0) b.st = 1;
          const ease = Math.max(0.25, Math.min(1, b.t * 4));
          b.x += b.vx * ease * dt;
          b.y += b.vy * ease * dt;
        } else {
          const dx = p.x - b.x;
          const dy = p.y - b.y;
          const d = Math.sqrt(dx * dx + dy * dy) || 1;
          b.t += dt;
          const sp = Math.min(540, 210 + b.t * 720);
          b.x += (dx / d) * sp * dt;
          b.y += (dy / d) * sp * dt;
          if (d < 18) dead = true;
        }
        const cnt = this.query(b.x, b.y, b.r);
        for (let j = 0; j < cnt; j++) {
          const e = en[this.q[j]];
          if (this.t - e.cd[b.k] < 0.32) continue;
          e.cd[b.k] = this.t;
          const sp = Math.sqrt(b.vx * b.vx + b.vy * b.vy) || 1;
          this.hit(e, b.dmg, b.k, 130, b.vx / sp, b.vy / sp);
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
    for (let i = this.nZ - 1; i >= 0; i--) {
      const z = zn[i];
      let dead = false;
      z.t += dt;
      if (z.st === 9) {
        this.rmZone(i);
        continue;
      }
      switch (z.kind) {
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
            if (z.slow) e.slow = z.slow;
            this.hit(e, z.dmg, z.k, z.len, dx / d, dy / d);
          }
          break;
        }
        case ZONE_KIND.MINE: {
          if (z.t < z.arm) break;
          let trig = z.t >= z.life;
          if (!trig) trig = this.query(z.x, z.y, z.mini ? 4 : 10) > 0;
          if (trig) {
            this.explode(z.x, z.y, z.r, z.dmg, z.k, 300);
            if (z.evo > 0 && !z.mini) {
              const off = this.rnd() * TAU;
              for (let m = 0; m < z.evo; m++) {
                const sub = this.addZone();
                if (!sub) break;
                const a = off + (m / z.evo) * TAU;
                sub.kind = ZONE_KIND.MINE;
                sub.k = z.k;
                sub.ox = z.x;
                sub.oy = z.y;
                sub.x = z.x + Math.cos(a) * 62;
                sub.y = z.y + Math.sin(a) * 62;
                sub.arm = 0.3;
                sub.life = 0.32;
                sub.r = z.rMax;
                sub.dmg = z.dmg * 0.52;
                sub.mini = 1;
                sub.evo = 0;
              }
            }
            dead = true;
          }
          break;
        }
        case ZONE_KIND.VORTEX: {
          z.a += dt * 5;
          z.tickT -= dt;
          if (z.tickT <= 0) {
            z.tickT = 0.25;
            const cnt = this.query(z.x, z.y, z.r);
            const pull = z.evo ? 175 : 105;
            for (let j = 0; j < cnt; j++) {
              const e = en[this.q[j]];
              const dx = z.x - e.x;
              const dy = z.y - e.y;
              const d = Math.sqrt(dx * dx + dy * dy) || 1;
              e.kx += ((dx / d) * pull - (dy / d) * 40) / e.wt;
              e.ky += ((dy / d) * pull + (dx / d) * 40) / e.wt;
              this.hit(e, z.dmg, z.k, 0, 0, 0);
            }
          }
          if (z.t >= z.life) {
            if (z.burst) this.explode(z.x, z.y, z.r * 1.1, z.burst, z.k, 385);
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
            if (z.tickT > 0) z.tickT -= dt;
            else if (proj > 0 && proj < z.len) {
              const perp = ry * ux - rx * uy;
              if (perp < z.r + 7 && perp > -z.r - 7 && p.inv <= 0) {
                this.hurt(z.dmg, z.src);
                z.tickT = 1;
              }
            }
          }
          break;
        }
        default:
          dead = true;
      }
      if (dead) this.rmZone(i);
    }
  }

  updEB(dt) {
    const eb = this.eb;
    const p = this.p;
    if (this.wipeEB) {
      this.nEB = 0;
      this.wipeEB = false;
    }
    for (let i = this.nEB - 1; i >= 0 && !this.wipeEB; i--) {
      const b = eb[i];
      b.x += b.vx * dt;
      b.y += b.vy * dt;
      b.life -= dt;
      const dx = b.x - p.x;
      const dy = b.y - p.y;
      const rr = b.r + p.r * 0.62;
      let dead = b.life <= 0;
      if (dx * dx + dy * dy < rr * rr) {
        if (p.inv <= 0) this.hurt(b.dmg, b.src);
        dead = true;
      }
      if (dead) {
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
        if (g.t < 0.3) {
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
        g.sp += 1150 * dt;
        const step = g.sp * dt;
        if (step >= d - 10) {
          p.xp += g.v * p.st.xpMul;
          this.fx && this.fx(FX_EVT.GEM, g.tier, g.x, g.y);
          while (p.xp >= p.need) {
            p.xp -= p.need;
            p.lvl++;
            p.need = CURVES.xpNeed(p.lvl);
            this.pendLv++;
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
        if (kind === PICK_KIND.HEART) p.hp = Math.min(p.maxHp, p.hp + 28);
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
    return Math.floor((this.t * 0.28 + this.kills * 0.065 + this.elites * 6 + this.coresPicked) * this.p.st.coreMul);
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
      weapons: p.weapons.map((w) => ({
        id: w.id,
        lv: w.lv,
        evo: w.evo,
        dmg: Math.round(this.dmgBy[w.k]),
      })),
      passives: p.passives.map((ps) => ({ id: ps.id, lv: ps.lv })),
      evos: this.evos.slice(),
      top: WEAPONS[topIdx].id,
      cause: this.lastSrc >= 0 ? ENEMIES[this.lastSrc].name : "",
      causeId: this.lastSrc >= 0 ? ENEMIES[this.lastSrc].id : "",
      char: this.char.id,
      daily: this.daily,
      rule: this.rule.id || "",
      wave: this.wave + 1,
      hurts: this.hurts,
      killsBy: Array.prototype.slice.call(this.killsBy),
    };
  }
}

export function createAutoBot(smart = 1, rnd = Math.random) {
  const state = { mx: 0, my: 0, think: 0 };
  const dangerR = smart ? 200 : 105;
  const stepDist = smart ? 44 : 28;
  const DIRS = 16;

  function dir(world) {
    const p = world.p;
    let gx = 0, gy = 0, gWeight = 0, bestGoal = 1e18;
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
        if (e.tele > 0 && smart) {
          const rx = tx - e.x;
          const ry = ty - e.y;
          const proj = rx * e.ax + ry * e.ay;
          if (proj > -20 && proj < e.tele + 40 && Math.abs(ry * e.ax - rx * e.ay) < e.r + 26) sc -= 3;
        }
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

  function move(world, dt) {
    state.think -= dt;
    if (state.think <= 0) {
      if (!smart && rnd() < 0.07) {
        state.mx = 0;
        state.my = 0;
        state.think = 0.4 + rnd() * 0.5;
      } else {
        const v = dir(world);
        state.mx = v[0];
        state.my = v[1];
        state.think = smart ? 0.08 : 0.3 + rnd() * 0.3;
      }
    }
    world.input(state.mx, state.my);
  }

  function scoreChoice(world, c) {
    const p = world.p;
    if (c.kind === "evo") return 100;
    if (c.kind === "w") {
      const def = WEAPONS[WEAPON_IDX[c.id]];
      const hasP = world.plv(def.partner) > 0;
      return c.isNew ? 6 + (hasP ? 2 : 0) + (p.weapons.length < 2 ? 3 : 0) : 7 + c.lv * 0.5 + (hasP ? 1.5 : 0);
    }
    if (c.kind === "p") {
      let partner = false;
      for (let i = 0; i < p.weapons.length; i++) {
        if (WEAPONS[p.weapons[i].k].partner === c.id && !p.weapons[i].evo) partner = true;
      }
      return c.isNew
        ? partner ? 9.5 : c.id === "leverage" || c.id === "haste" ? 5.5 : 4.5
        : c.id === "leverage" || c.id === "haste" || c.id === "area" ? 6 : 5;
    }
    return c.id === "b_heal" ? (p.hp < p.maxHp * 0.5 ? 8 : 1) : c.id === "b_might" ? 3 : 2;
  }

  function pick(world, choices) {
    if (!smart) return (rnd() * choices.length) | 0;
    let best = 0;
    let bestVal = -1;
    for (let i = 0; i < choices.length; i++) {
      const v = scoreChoice(world, choices[i]);
      if (v > bestVal) {
        bestVal = v;
        best = i;
      }
    }
    return best;
  }

  return { dir, move, pick };
}
