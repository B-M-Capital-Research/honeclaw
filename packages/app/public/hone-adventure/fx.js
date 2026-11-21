// HONE Adventure — Particle System, Floating Crit Numbers & 7-Zone Cyber-Quant Arena Renderer

import { AI_TYPE, ENEMIES, WEAPONS } from "./data.js";
import { BULLET_KIND, FX_EVT, PICK_KIND, ZONE_KIND } from "./world.js";
import { SPR_IDX } from "./art.js";
import { playGemChime, playKillPop, playSfx } from "./audio.js";

const MAX_PARTS = 1100;
const MAX_NUMS = 120;
const MAX_RINGS = 80;
const MAX_BOLTS = 48;
const TAU = Math.PI * 2;

const hexInt = (h) => parseInt(h.slice(1), 16);
const WEAPON_COLS = WEAPONS.map((w) => hexInt(w.color));

const ENEMY_SPARK_COLS = [
  0xc084fc, 0xf43f5e, 0x818cf8, 0xfb923c, 0xef4444, 0xd8b4fe, 0xf97316,
  0x67e8f9, 0xa855f7, 0xe879f9, 0xfb7185, 0xc084fc, 0xf43f5e,
];
const ENEMY_INK_COLS = [
  0x4c1d95, 0x881337, 0x312e81, 0x7c2d12, 0x7f1d1d, 0x581c87, 0x7c2d12,
  0x164e63, 0x4c1d95, 0x581c87, 0x881337, 0x4c1d95, 0x7f1d1d,
];

const NUM_NORM = [];
const NUM_CRIT = [];
for (let i = 0; i < 10; i++) {
  NUM_NORM.push(SPR_IDX["n" + i]);
  NUM_CRIT.push(SPR_IDX["c" + i]);
}

export function createFxSystem(hooks) {
  const parts = Array.from({ length: MAX_PARTS }, () => ({
    x: 0, y: 0, vx: 0, vy: 0, life: 0, max: 1, s0: 1, s1: 1, rot: 0, vr: 0,
    col: 0xffffff, spr: 0, add: 0, drag: 0, g: 0, ar: 1, a: 1,
  }));
  const nums = Array.from({ length: MAX_NUMS }, () => ({ x: 0, y: 0, life: 0, val: 0, crit: 0 }));
  const rings = Array.from({ length: MAX_RINGS }, () => ({
    x: 0, y: 0, r0: 0, r1: 0, life: 0, max: 1, col: 0xffffff, spr: 0, a: 1,
  }));
  const bolts = Array.from({ length: MAX_BOLTS }, () => ({ x0: 0, y0: 0, x1: 0, y1: 0, life: 0, seed: 0 }));
  const timers = [];

  const S = SPR_IDX;
  const fx = {
    nP: 0, nN: 0, nR: 0, nBo: 0,
    trauma: 0, flash: 0, flashCol: 0xffffff, tone: 0, toneCol: 0xffffff,
    hurtV: 0, q: 1, hitN: 0, killN: 0, numN: 0, gemN: 0, boomN: 0,
    stopCd: 0, shX: 0, shY: 0, quiet: false, t: 0, gemGlow: 0, hitI: 0,
  };

  const panAt = (wx) => {
    const p = (wx - hooks.camX()) / (hooks.viewW() * 0.5);
    return p < -1 ? -0.8 : p > 1 ? 0.8 : p * 0.8;
  };
  const sfx = (name, wx, rate, vol) => {
    if (!fx.quiet) playSfx(name, wx == null ? 0 : panAt(wx), rate, vol);
  };
  const qCnt = (n) => Math.max(1, Math.ceil(n * fx.q));

  function addPart(spr, x, y, vx, vy, life, s0, s1, col, add, drag = 0, g = 0, rot = 0, vr = 0, ar = 1, a = 1) {
    if (fx.nP >= MAX_PARTS) return null;
    const p = parts[fx.nP++];
    p.spr = spr; p.x = x; p.y = y; p.vx = vx; p.vy = vy;
    p.life = life; p.max = life; p.s0 = s0; p.s1 = s1; p.col = col;
    p.add = add; p.drag = drag; p.g = g; p.rot = rot; p.vr = vr; p.ar = ar; p.a = a;
    return p;
  }

  function addRing(x, y, r0, r1, life, col, spr = S.ring, a = 1) {
    if (fx.nR >= MAX_RINGS) return;
    const r = rings[fx.nR++];
    r.x = x; r.y = y; r.r0 = r0; r.r1 = r1; r.life = life; r.max = life; r.col = col; r.spr = spr; r.a = a;
  }

  function addNum(x, y, val, crit) {
    let slot;
    if (fx.nN >= MAX_NUMS) {
      slot = nums[0];
      for (let i = 1; i < MAX_NUMS; i++) if (nums[i].life < slot.life) slot = nums[i];
    } else {
      slot = nums[fx.nN++];
    }
    slot.x = x + (Math.random() - 0.5) * 14;
    slot.y = y - 10;
    slot.life = crit ? 0.8 : 0.55;
    slot.val = val;
    slot.crit = crit;
  }

  function burst(x, y, cnt, col, spd, sz, life) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      const a = Math.random() * TAU;
      const v = spd * (0.35 + Math.random() * 0.65);
      addPart(
        i % 3 ? S.shard : S.dotp, x, y, Math.cos(a) * v, Math.sin(a) * v,
        life * (0.6 + Math.random() * 0.6), sz * (0.7 + Math.random() * 0.7), 0.1,
        i % 4 === 0 ? 0xffffff : col, i & 1, 3.2, 0, Math.random() * TAU, (Math.random() - 0.5) * 16,
      );
    }
  }

  function sparks(x, y, cnt, col, spd, sz, life) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      const a = Math.random() * TAU;
      const v = spd * (0.5 + Math.random() * 0.5);
      addPart(S.streak, x, y, Math.cos(a) * v, Math.sin(a) * v, life * (0.7 + Math.random() * 0.5), sz, sz * 0.15, col, 1, 4, 0, a, 0, 1.4);
    }
  }

  function hexBubbles(x, y, cnt, rad) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      addPart(
        S.bub, x + (Math.random() - 0.5) * rad, y + (Math.random() - 0.5) * rad,
        (Math.random() - 0.5) * 30, -20 - Math.random() * 50,
        0.5 + Math.random() * 0.6, 0.3 + Math.random() * 0.45, 0.5 + Math.random() * 0.45,
        0x5ce1e6, 0, 1.2, -45,
      );
    }
  }

  function inkSplash(x, y, cnt, col, rad) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      const a = Math.random() * TAU;
      const v = 14 + Math.random() * 30;
      addPart(
        S.ink, x + Math.cos(a) * rad * 0.3, y + Math.sin(a) * rad * 0.3,
        Math.cos(a) * v, Math.sin(a) * v - 6, 0.5 + Math.random() * 0.35,
        rad / 30, rad / 12, col, 0, 1.5, 0, Math.random() * TAU, (Math.random() - 0.5) * 1.5, 1, 0.6,
      );
    }
  }

  function rays(x, y, cnt, col, sc, life) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      const a = (i / cnt) * TAU + Math.random() * 0.2;
      addPart(
        S.ray, x + Math.cos(a) * sc * 30, y + Math.sin(a) * sc * 30,
        Math.cos(a) * 60, Math.sin(a) * 60, life * (0.8 + Math.random() * 0.4),
        sc * 0.5, sc * 1.3, col, 1, 1, 0, a + Math.PI / 2, 0, 1, 0.7,
      );
    }
  }

  function rise(x, y, cnt, col, spread, upSpd = 60) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      addPart(
        i % 3 ? S.spark : S.flare,
        x + (Math.random() - 0.5) * spread, y + (Math.random() - 0.5) * spread * 0.6,
        (Math.random() - 0.5) * 30, -upSpd * (0.5 + Math.random()),
        0.6 + Math.random() * 0.5, 0.35 + Math.random() * 0.4, 0.05,
        i & 1 ? col : 0xffffff, 1, 0.6, -30, Math.random() * TAU, (Math.random() - 0.5) * 4,
      );
    }
  }

  const later = (sec, fn) => timers.push({ t: sec, fn });

  fx.ring = addRing;
  fx.bubbles = hexBubbles;
  fx.shake = (v) => { fx.trauma = Math.min(1, fx.trauma + v); };
  fx.setTone = (v, col) => {
    if (v > fx.tone) {
      fx.tone = v;
      fx.toneCol = col;
    }
  };
  fx.levelFx = (x, y) => {
    addPart(S.pillar, x, y - 150, 0, 0, 0.75, 2.2, 1, 0xffd166, 1, 0, 0, 0, 0, 1.5, 0.38);
    addPart(S.pillar, x, y - 170, 0, 0, 0.6, 0.7, 0.25, 0xffffff, 1, 0, 0, 0, 0, 1.8, 0.6);
    addPart(S.rune, x, y + 4, 0, 0, 0.85, 0.5, 1.7, 0xffad36, 1, 0, 0, 0, 2.4, 1, 0.75);
    addRing(x, y, 10, 190, 0.55, 0xffad36, S.ring2, 0.44);
    addRing(x, y, 10, 120, 0.4, 0xffffff, S.ring, 0.8);
    addRing(x, y, 20, 270, 0.75, 0x2ee59d, S.ring, 0.6);
    const cnt = qCnt(18);
    for (let i = 0; i < cnt; i++) {
      const a = (i / cnt) * TAU;
      const r = 16 + (i % 3) * 8;
      addPart(
        i & 1 ? S.spark : S.flare, x + Math.cos(a) * r, y + Math.sin(a) * r * 0.5,
        Math.cos(a) * 70, -70 - Math.random() * 90, 0.7 + Math.random() * 0.3,
        0.6, 0.05, i & 1 ? 0xffd166 : 0xffffff, 1, 1.4, -60, a, 5,
      );
    }
    fx.setTone(0.14, 0xffd166);
  };

  fx.on = (evt, a, b, c, d, e) => {
    switch (evt) {
      case FX_EVT.HIT:
        fx.hitN++;
        if (d || fx.numN < 9) {
          addNum(a, b, c, d);
          fx.numN++;
        }
        if (fx.hitN < 12) {
          addPart(S.flare, a, b, 0, 0, 0.13, 0.28, 0.6, WEAPON_COLS[e] || 0xffffff, 1, 0, 0, Math.random() * 1.5);
          if (fx.hitN < 6) sparks(a, b, 2, 0xffffff, 190, 0.42, 0.16);
        }
        if (d) {
          if (fx.hitN < 20) {
            addPart(S.flare, a, b, 0, 0, 0.24, 0.7, 1.7, 0xffd166, 1, 0, 0, Math.random() * 3, 3);
            addRing(a, b, 4, 30, 0.2, 0xffad36);
            sparks(a, b, 4, 0xffad36, 260, 0.6, 0.2);
          }
          sfx("crit", a, 0.94 + Math.random() * 0.14);
        } else {
          sfx((fx.hitI = (fx.hitI | 0) + 1) & 1 ? "hit" : "hit2", a, 0.88 + Math.random() * 0.3);
        }
        break;

      case FX_EVT.KILL: {
        fx.killN++;
        const col = ENEMY_SPARK_COLS[c] || 0xffad36;
        const big = d > 0;
        const rad = ENEMIES[c].r * (d === 1 ? 1.6 : 1);
        if (fx.killN < 16) inkSplash(a, b, big ? 5 : fx.killN < 6 ? 3 : 2, ENEMY_INK_COLS[c] || 0x312e81, rad * (big ? 1.5 : 1.1));
        burst(a, b, big ? 14 : fx.killN < 5 ? 5 : 3, col, big ? 260 : 170, big ? 1.5 : 1, 0.42);
        if (fx.killN < 10) hexBubbles(a, b, big ? 5 : 2, rad);
        if (fx.killN < 20) {
          addPart(S.glow, a, b, 0, 0, 0.16, rad / 16, rad / 7, col, 1);
          addRing(a, b, rad * 0.5, rad * 2.3, 0.22, col);
        }
        if (!fx.quiet) playKillPop(panAt(a), big, hooks.combo());
        if (d === 1) {
          sfx("shatter", a);
          hooks.hitstop(0.07);
          fx.shake(0.35);
          hooks.vibrate("medium");
          rays(a, b, 8, 0xffd166, 0.9, 0.4);
        } else if (d !== 2 && e && fx.stopCd <= 0) {
          hooks.hitstop(0.03);
          fx.stopCd = 0.22;
        }
        fx.shake(0.022);
        break;
      }

      case FX_EVT.HURT:
        fx.shake(0.55);
        fx.hurtV = 1;
        fx.setTone(0.22, 0xf43f5e);
        hooks.hitstop(0.06);
        sfx("hurt", b);
        hooks.vibrate("medium");
        hooks.hurt();
        addRing(b, c, 8, 70, 0.3, 0xf43f5e, S.ring2, 0.9);
        burst(b, c, 8, 0xfb7185, 160, 1, 0.35);
        break;

      case FX_EVT.SHOT: {
        const wid = WEAPONS[a].id;
        if (wid === "blade") sfx("shoot", b, d ? 1.25 + Math.random() * 0.2 : 0.95 + Math.random() * 0.15, d ? 0.7 : 1);
        else if (wid === "wafer") sfx("star", b, 0.95 + Math.random() * 0.1);
        else if (wid === "arc") sfx("zap", b, 0.92 + Math.random() * 0.2);
        else if (wid === "pulse") sfx("sonar", b, d ? 0.75 : 1);
        else if (wid === "mine") sfx("drop", b);
        else if (wid === "laser") sfx("beam", b);
        else if (wid === "vortex") sfx("whirl", b, d ? 0.8 : 1);
        else if (wid === "moat") sfx("magnet", b, 1.5, 0.35);
        break;
      }

      case FX_EVT.GEM:
        if (!fx.quiet) playGemChime(panAt(b), a);
        fx.gemGlow = 1;
        if (fx.gemN++ < 4) {
          addPart(S.flare, b, c, 0, 0, 0.16, 0.2, 0.55, a === 2 ? 0xffad36 : a === 1 ? 0x5ce1e6 : 0x2ee59d, 1);
        }
        break;

      case FX_EVT.PICK: {
        const col = a === PICK_KIND.HEART ? 0xfb7185 : a === PICK_KIND.CORE ? 0xffad36 : a === PICK_KIND.CHEST ? 0xffd166 : 0x2ee59d;
        addRing(b, c, 6, 46, 0.3, col);
        addPart(S.flare, b, c, 0, 0, 0.3, 0.5, 1.4, col, 1, 0, 0, 0, 2);
        rise(b, c, 5, col, 24, 50);
        if (a === PICK_KIND.HEART) sfx("heart", b);
        else if (a === PICK_KIND.CORE) sfx("core", b);
        else if (a === PICK_KIND.CHEST) {
          sfx("chest", b);
          rays(b, c, 10, 0xffd166, 1, 0.5);
          fx.setTone(0.14, 0xffd166);
        }
        break;
      }

      case FX_EVT.BOLT:
        if (fx.nBo < MAX_BOLTS) {
          const bo = bolts[fx.nBo++];
          bo.x0 = a; bo.y0 = b; bo.x1 = c; bo.y1 = d;
          bo.life = 0.16; bo.seed = (Math.random() * 1000) | 0;
        }
        addPart(S.flare, c, d, 0, 0, 0.16, 0.5, 1.1, 0x6fffe9, 1);
        break;

      case FX_EVT.RING:
        addRing(a, b, 14, c, 0.32, WEAPON_COLS[d] || 0x2ee59d, S.ring2, 0.5);
        addRing(a, b, 10, c * 0.96, 0.3, 0xffffff, S.ring, 0.6);
        break;

      case FX_EVT.BOOM: {
        const isEnemy = d === 99;
        const col = isEnemy ? 0xf43f5e : WEAPON_COLS[d] || 0xffad36;
        const full = fx.boomN++ < 3;
        addRing(a, b, c * 0.25, c * 1.05, 0.28, col, S.ring2, full ? 0.55 : 0.3);
        addRing(a, b, c * 0.2, c * 0.8, 0.22, 0xffffff, S.ring, 0.6);
        if (full) {
          addPart(S.glow, a, b, 0, 0, 0.2, c / 30, c / 15, col, 1, 0, 0, 0, 0, 1, 0.4);
          sparks(a, b, 6, col, c * 4, 0.7, 0.24);
        }
        sfx("boom", a, isEnemy ? 0.8 : 0.95 + Math.random() * 0.25, c > 80 ? 1.15 : 0.8);
        fx.shake(c > 80 ? 0.22 : 0.1);
        break;
      }

      case FX_EVT.POP:
        addRing(a, b, 3, 13, 0.16, 0xffad36);
        break;

      case FX_EVT.NUKE:
        fx.flash = Math.max(fx.flash, 0.3);
        fx.flashCol = 0xfffbeb;
        fx.setTone(0.32, 0x5ce1e6);
        fx.shake(1);
        hooks.hitstop(0.13);
        hooks.slowmo(0.35, 0.5);
        hooks.vibrate("heavy");
        sfx("nuke", null);
        addRing(a, b, 20, 640, 0.75, 0xffffff, S.ring2, 0.6);
        addRing(a, b, 20, 500, 0.6, 0xffad36, S.ring, 1);
        addRing(a, b, 10, 760, 0.95, 0x2ee59d, S.ring2, 0.38);
        rays(a, b, 16, 0xffd166, 2, 0.7);
        addPart(S.rune, a, b, 0, 0, 0.9, 0.8, 3.2, 0xffad36, 1, 0, 0, 0, 2, 1, 0.55);
        hooks.announce("去伪存真 · 全屏净除！", "big");
        break;

      case FX_EVT.COMBO:
        hooks.announce("连击 ×" + a, "combo", b);
        sfx("combo", null, 1 + b * 0.06);
        fx.shake(0.15);
        fx.setTone(0.1, 0xffad36);
        break;

      case FX_EVT.WAVE:
        hooks.announce("第 " + a + " 波噪音潮", "wave");
        sfx("wave", null);
        break;

      case FX_EVT.BOSSWARN:
        hooks.announce("极端行情预警", "warn", a);
        sfx("warn", null);
        hooks.vibrate("heavy");
        fx.setTone(0.25, 0xf43f5e);
        later(0.5, () => fx.setTone(0.2, 0xf43f5e));
        break;

      case FX_EVT.BOSS:
        fx.shake(0.6);
        sfx("elite", a, 0.7, 1.3);
        addRing(a, b, 30, 220, 0.6, 0xf43f5e, S.ring2, 0.9);
        addRing(a, b, 30, 320, 0.8, 0xffad36, S.ring, 0.7);
        break;

      case FX_EVT.BOSSDEAD: {
        const col = ENEMY_SPARK_COLS[8 + c] || 0xffad36;
        fx.flash = Math.max(fx.flash, 0.34);
        fx.flashCol = 0xfffbeb;
        fx.setTone(0.32, col);
        fx.shake(1);
        hooks.hitstop(0.28);
        hooks.slowmo(0.25, 1.3);
        hooks.vibrate("heavy");
        sfx("bossdead", null);
        addRing(a, b, 30, 720, 0.9, 0xffffff, S.ring2, 0.55);
        addRing(a, b, 30, 540, 0.7, col, S.ring, 1);
        burst(a, b, 50, col, 520, 2, 0.9);
        rays(a, b, 18, 0xffffff, 2.4, 0.8);
        for (let i = 0; i < 6; i++) {
          later(0.12 + i * 0.13, () => {
            const rx = a + (Math.random() - 0.5) * 110;
            const ry = b + (Math.random() - 0.5) * 90;
            addRing(rx, ry, 10, 90, 0.3, col, S.ring2, 0.9);
            sparks(rx, ry, 8, col, 320, 0.8, 0.3);
            fx.shake(0.3);
            sfx("boom", rx, 0.8 + Math.random() * 0.4);
          });
        }
        hooks.announce("击破「" + ENEMIES[8 + c].name + "」！", "big");
        break;
      }

      case FX_EVT.ELITE:
        hooks.announce("高危精英出现", "wave");
        sfx("elite", a);
        break;

      case FX_EVT.DEAD:
        fx.flash = Math.max(fx.flash, 0.4);
        fx.flashCol = 0xfecdd3;
        fx.setTone(0.35, 0xf43f5e);
        fx.shake(1);
        hooks.vibrate("heavy");
        sfx("dead", null);
        addRing(a, b, 10, 240, 0.8, 0xffffff, S.ring2, 1);
        burst(a, b, 40, 0xffad36, 320, 1.6, 0.9);
        break;

      case FX_EVT.REVIVE:
        fx.flash = Math.max(fx.flash, 0.45);
        fx.flashCol = 0xfffbeb;
        fx.setTone(0.35, 0x2ee59d);
        fx.shake(0.8);
        sfx("revive", null);
        addRing(a, b, 20, 420, 0.7, 0x2ee59d, S.ring2, 1);
        rays(a, b, 16, 0xffd166, 2, 0.7);
        hooks.announce("熔断保护触发 · 满血反击！", "big");
        hooks.slowmo(0.3, 0.6);
        break;

      case FX_EVT.EVO: {
        const col = WEAPON_COLS[a] || 0xffad36;
        fx.flash = Math.max(fx.flash, 0.4);
        fx.flashCol = 0xfffbeb;
        fx.setTone(0.36, 0xffad36);
        fx.shake(0.7);
        sfx("evo", null);
        hooks.vibrate("heavy");
        hooks.slowmo(0.3, 0.7);
        addRing(b, c, 20, 440, 0.8, 0xffad36, S.ring2, 0.6);
        addRing(b, c, 20, 540, 1, col, S.ring2, 0.42);
        rays(b, c, 18, 0xffd166, 2, 0.8);
        addPart(S.pillar, b, c - 190, 0, 0, 1, 2.6, 1, 0xffad36, 1, 0, 0, 0, 0, 1.8, 0.42);
        addPart(S.rune, b, c, 0, 0, 1.1, 0.8, 2.8, 0xffd166, 1, 0, 0, 0, -2.4, 1, 0.85);
        rise(b, c, 28, col, 120, 130);
        hooks.announce("超维进化 · " + WEAPONS[a].evoName + "！", "evo");
        break;
      }

      case FX_EVT.ESHOT:
        sfx("eshot", a, d ? 0.7 : 1, d ? 1.6 : 1);
        addPart(S.flare, a, b, 0, 0, d ? 0.25 : 0.15, d ? 1 : 0.4, d ? 2.4 : 0.8, 0xfb7185, 1);
        break;

      case FX_EVT.MAGNET:
        sfx("magnet", null);
        addRing(a, b, 400, 20, 0.45, 0x2ee59d, S.ring2, 0.8);
        fx.setTone(0.14, 0x2ee59d);
        break;

      case FX_EVT.DASH:
        sfx("dash", a);
        break;

      case FX_EVT.HEAL:
        sfx("heart", a);
        rise(a, b, 8, 0x2ee59d, 30, 60);
        break;

      case FX_EVT.NOVA:
        addRing(a, b, 14, c, 0.34, 0xffffff, S.ring2, 0.7);
        break;

      default:
        break;
    }
  };

  fx.update = (simDt, rawDt) => {
    fx.t += rawDt;
    fx.hitN = fx.killN = fx.numN = fx.gemN = fx.boomN = 0;
    if (fx.stopCd > 0) fx.stopCd -= rawDt;
    fx.trauma = Math.max(0, fx.trauma - rawDt * 2.1);
    const sh = fx.trauma * fx.trauma * 9;
    fx.shX = (Math.sin(fx.t * 61.3) + Math.sin(fx.t * 97.1) * 0.6) * sh * 0.62;
    fx.shY = (Math.cos(fx.t * 71.7) + Math.sin(fx.t * 113.3) * 0.6) * sh * 0.62;
    if (fx.flash > 0) fx.flash = Math.max(0, fx.flash - rawDt * 2.8);
    if (fx.tone > 0) fx.tone = Math.max(0, fx.tone - rawDt * 0.9 * (0.4 + fx.tone * 3));
    if (fx.hurtV > 0) fx.hurtV = Math.max(0, fx.hurtV - rawDt * 2.2);
    if (fx.gemGlow > 0) fx.gemGlow = Math.max(0, fx.gemGlow - rawDt * 5);

    for (let i = timers.length - 1; i >= 0; i--) {
      timers[i].t -= simDt > 0 ? simDt : rawDt * 0.3;
      if (timers[i].t <= 0) {
        const fn = timers[i].fn;
        timers.splice(i, 1);
        fn();
      }
    }
    for (let i = fx.nP - 1; i >= 0; i--) {
      const p = parts[i];
      p.life -= simDt;
      if (p.life <= 0) {
        const last = --fx.nP;
        parts[i] = parts[last];
        parts[last] = p;
        continue;
      }
      if (p.drag) {
        const d = Math.exp(-p.drag * simDt);
        p.vx *= d;
        p.vy *= d;
      }
      p.vy += p.g * simDt;
      p.x += p.vx * simDt;
      p.y += p.vy * simDt;
      p.rot += p.vr * simDt;
    }
    for (let i = fx.nN - 1; i >= 0; i--) {
      nums[i].life -= simDt;
      if (nums[i].life <= 0) {
        const tmp = nums[i];
        const last = --fx.nN;
        nums[i] = nums[last];
        nums[last] = tmp;
      }
    }
    for (let i = fx.nR - 1; i >= 0; i--) {
      rings[i].life -= simDt;
      if (rings[i].life <= 0) {
        const tmp = rings[i];
        const last = --fx.nR;
        rings[i] = rings[last];
        rings[last] = tmp;
      }
    }
    for (let i = fx.nBo - 1; i >= 0; i--) {
      bolts[i].life -= simDt;
      if (bolts[i].life <= 0) {
        const tmp = bolts[i];
        const last = --fx.nBo;
        bolts[i] = bolts[last];
        bolts[last] = tmp;
      }
    }
  };

  fx.draw = (R) => {
    for (let i = 0; i < fx.nP; i++) {
      const p = parts[i];
      if (p.add) continue;
      const k = p.life / p.max;
      const sc = p.s1 + (p.s0 - p.s1) * k;
      R.spr(p.spr, p.x, p.y, sc, sc * p.ar, p.rot, (k < 0.5 ? k * 2 : 1) * p.a, p.col, 0, 0);
    }
    for (let i = 0; i < fx.nR; i++) {
      const r = rings[i];
      const u = 1 - r.life / r.max;
      const ease = 1 - (1 - u) * (1 - u);
      const rad = r.r0 + (r.r1 - r.r0) * ease;
      const sc = rad / (r.spr === S.ring2 ? 51 : 56);
      R.spr(r.spr, r.x, r.y, sc, sc, 0, (1 - u) * r.a, r.col, 1, 0);
    }
    for (let i = 0; i < fx.nP; i++) {
      const p = parts[i];
      if (!p.add) continue;
      const k = p.life / p.max;
      const sc = p.s1 + (p.s0 - p.s1) * k;
      R.spr(p.spr, p.x, p.y, sc, sc * p.ar, p.rot, (k < 0.5 ? k * 2 : 1) * p.a, p.col, 1, 0);
    }
    for (let i = 0; i < fx.nBo; i++) {
      const bo = bolts[i];
      const alpha = Math.min(1, bo.life / 0.09);
      const dx = bo.x1 - bo.x0;
      const dy = bo.y1 - bo.y0;
      const len = Math.sqrt(dx * dx + dy * dy) || 1;
      const segs = Math.max(2, Math.min(7, Math.round(len / 26)));
      const nx = -dy / len;
      const ny = dx / len;
      let px = bo.x0, py = bo.y0;
      let sd = bo.seed + ((bo.life * 40) | 0) * 7;
      for (let s = 1; s <= segs; s++) {
        sd = (sd * 9301 + 49297) % 233280;
        const jit = s === segs ? 0 : (sd / 233280 - 0.5) * Math.min(24, len * 0.32);
        const qx = bo.x0 + (dx * s) / segs + nx * jit;
        const qy = bo.y0 + (dy * s) / segs + ny * jit;
        const sx = qx - px, sy = qy - py;
        const sl = Math.sqrt(sx * sx + sy * sy);
        const ang = Math.atan2(sy, sx);
        R.spr(S.bolt, (px + qx) / 2, (py + qy) / 2, (sl + 6) / 64, 3.2, ang, alpha * 0.55, 0x2ee59d, 1, 0);
        R.spr(S.bolt, (px + qx) / 2, (py + qy) / 2, (sl + 4) / 64, 1.3, ang, alpha, 0xffffff, 1, 0);
        px = qx;
        py = qy;
      }
    }
  };

  fx.drawNums = (R) => {
    for (let i = 0; i < fx.nN; i++) {
      const n = nums[i];
      const dur = n.crit ? 0.8 : 0.55;
      const u = 1 - n.life / dur;
      const pop = u < 0.16 ? 1 + (1 - u / 0.16) * 0.9 : 1;
      const sc = (n.crit ? 1.25 : 0.75) * pop;
      const alpha = n.life < 0.18 ? n.life / 0.18 : 1;
      const y = n.y - (1 - (1 - u) * (1 - u)) * (n.crit ? 32 : 20);
      const glyphs = n.crit ? NUM_CRIT : NUM_NORM;
      const step = (n.crit ? 11.5 : 10) * sc;
      let val = n.val | 0;
      const digits = val >= 1e4 ? 5 : val >= 1e3 ? 4 : val >= 100 ? 3 : val >= 10 ? 2 : 1;
      let x = n.x + (digits - 1) * step * 0.5;
      if (n.crit && u < 0.3) R.spr(S.flare, n.x, y, 0.9 * pop, 0.5 * pop, 0, (1 - u / 0.3) * 0.7, 0xffad36, 1, 0);
      for (let d = 0; d < digits; d++) {
        R.spr(glyphs[val % 10], x, y, sc, sc, 0, alpha, 0xffffff, 0, 0);
        val = (val / 10) | 0;
        x -= step;
      }
    }
  };

  fx.clear = () => {
    fx.nP = fx.nN = fx.nR = fx.nBo = 0;
    fx.trauma = fx.flash = fx.tone = fx.hurtV = 0;
    timers.length = 0;
  };

  return fx;
}

// 7 Dynamic Cyber-Quant Arena Zones
export const ZONES = {
  sandbox: {
    name: "研报沙盘",
    gr: [0.85, 0.95, 0.92], g2: [0.22, 0.42, 0.4], caus: 0.44, shaft: 0.085,
    sc: [1, 0.88, 0.65], decor: [0.88, 0.96, 0.94], near: [0.05, 0.14, 0.15],
    dust: [1, 0.86, 0.55], vig: 0.72, wash: [0.2, 0.5, 0.45], wa: 0,
  },
  cluster: {
    name: "算力机房",
    gr: [0.55, 0.82, 0.96], g2: [0.12, 0.28, 0.44], caus: 0.34, shaft: 0.055,
    sc: [0.55, 0.92, 1], decor: [0.68, 0.88, 1], near: [0.03, 0.09, 0.16],
    dust: [0.45, 0.95, 0.88], vig: 0.92, wash: [0.08, 0.35, 0.5], wa: 0.035,
  },
  darkpool: {
    name: "暗池深渊",
    gr: [0.55, 0.45, 0.78], g2: [0.18, 0.1, 0.36], caus: 0.2, shaft: 0.03,
    sc: [0.95, 0.7, 1], decor: [0.7, 0.62, 0.92], near: [0.06, 0.03, 0.14],
    dust: [1, 0.72, 0.42], vig: 1.08, wash: [0.35, 0.12, 0.55], wa: 0.06,
  },
  boss1: {
    name: "",
    gr: [0.72, 0.42, 0.8], g2: [0.35, 0.08, 0.44], caus: 0.3, shaft: 0.04,
    sc: [1, 0.6, 0.95], decor: [0.85, 0.58, 0.95], near: [0.14, 0.03, 0.18],
    dust: [1, 0.6, 0.9], vig: 1.12, wash: [0.65, 0.1, 0.6], wa: 0.07,
  },
  boss2: {
    name: "",
    gr: [0.82, 0.48, 0.44], g2: [0.42, 0.14, 0.12], caus: 0.3, shaft: 0.04,
    sc: [1, 0.75, 0.55], decor: [0.95, 0.68, 0.62], near: [0.18, 0.05, 0.04],
    dust: [1, 0.75, 0.55], vig: 1.12, wash: [0.78, 0.24, 0.12], wa: 0.07,
  },
  boss3: {
    name: "",
    gr: [0.64, 0.4, 0.86], g2: [0.28, 0.08, 0.48], caus: 0.3, shaft: 0.04,
    sc: [0.95, 0.6, 1], decor: [0.8, 0.58, 0.98], near: [0.12, 0.03, 0.22],
    dust: [1, 0.6, 1], vig: 1.12, wash: [0.58, 0.1, 0.72], wa: 0.07,
  },
  boss4: {
    name: "",
    gr: [0.78, 0.35, 0.42], g2: [0.38, 0.05, 0.14], caus: 0.26, shaft: 0.035,
    sc: [1, 0.55, 0.6], decor: [0.9, 0.52, 0.58], near: [0.2, 0.02, 0.06],
    dust: [1, 0.62, 0.5], vig: 1.2, wash: [0.82, 0.08, 0.2], wa: 0.085,
  },
};

export function pickZoneKey(world) {
  if (world.boss && !world.boss.dead) return "boss" + world.boss.boss;
  if (world.t < 100) return "sandbox";
  if (world.t < 300) return "cluster";
  return "darkpool";
}

export function cloneZone(key) {
  const src = ZONES[key];
  const out = {};
  for (const k in src) out[k] = Array.isArray(src[k]) ? src[k].slice() : src[k];
  return out;
}

export function lerpZone(cur, key, alpha) {
  const tgt = ZONES[key];
  for (const k in tgt) {
    if (Array.isArray(tgt[k])) {
      cur[k][0] += (tgt[k][0] - cur[k][0]) * alpha;
      cur[k][1] += (tgt[k][1] - cur[k][1]) * alpha;
      cur[k][2] += (tgt[k][2] - cur[k][2]) * alpha;
    } else if (typeof tgt[k] === "number") {
      cur[k] += (tgt[k] - cur[k]) * alpha;
    }
  }
}

const rgbInt = (arr) =>
  ((Math.min(255, arr[0] * 255) | 0) << 16) |
  ((Math.min(255, arr[1] * 255) | 0) << 8) |
  (Math.min(255, arr[2] * 255) | 0);

const cellHash = (x, y) => {
  let h = Math.imul(x, 73856093) ^ Math.imul(y, 19349663);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return (h ^ (h >>> 16)) >>> 0;
};

export function renderWorld(R, world, fx, view) {
  const S = SPR_IDX;
  const ENEMY_SPRS = ENEMIES.map((e) => S["e_" + e.id]);
  const PICK_SPRS = [S.heart, S.magnet, S.orb, S.core, S.chest];
  const GEM_SPRS = [S.gem0, S.gem1, S.gem2];
  const GEM_GLOWS = [0x2ee59d, 0x5ce1e6, 0xffad36];
  const SMALL_PROPS = [S.d_crystal, S.d_node, S.d_crystal, S.d_node];
  const BIG_PROPS = [S.d_rack, S.d_monolith];

  const t = view.t;
  const sc = view.sc;
  const camX = view.camX;
  const camY = view.camY;
  const hw = view.cw / 2 / sc;
  const hh = view.ch / 2 / sc;
  const p = world.p;
  const zn = view.zone;
  const lite = view.lite;
  const x0 = camX - hw - 70, x1 = camX + hw + 70;
  const y0 = camY - hh - 70, y1 = camY + hh + 70;
  const decorTint = rgbInt(zn.decor);

  R.begin();
  R.world(camX, camY, sc);
  R.tile(0, 0, 0, 440, 1, 0, rgbInt(zn.gr));
  if (!lite) R.tile(0, camX * 0.42 + 130, camY * 0.42 + 70, 1400, 0.36, 0, rgbInt(zn.g2));
  if (!lite || R.kind === "webgl") R.tile(1, camX * 0.2 + t * 8, camY * 0.2 + t * 5, 580, zn.caus, 1);

  // Small crystal/node grid props
  const STEP1 = 230;
  for (let gy = Math.floor((y0 - 60) / STEP1); gy <= Math.floor((y1 + 60) / STEP1); gy++) {
    for (let gx = Math.floor((x0 - 60) / STEP1); gx <= Math.floor((x1 + 60) / STEP1); gx++) {
      const h = cellHash(gx, gy);
      if (h % 100 > 58) continue;
      const wx = gx * STEP1 + ((h >>> 8) % STEP1);
      const wy = gy * STEP1 + ((h >>> 16) % STEP1);
      const sz = 0.68 + ((h >>> 4) % 6) * 0.06;
      const spr = SMALL_PROPS[(h >>> 12) % SMALL_PROPS.length];
      R.spr(S.shadow, wx, wy + 18 * sz, 1.3 * sz, 0.95 * sz, 0, 0.5, 0xffffff, 0, 0);
      R.spr(spr, wx, wy, (h & 2 ? -1 : 1) * sz, sz, 0, 0.92, decorTint, 0, 0);
    }
  }

  // Server rack & circuit monolith landmarks
  const STEP2 = 760;
  for (let gy = Math.floor((y0 - 160) / STEP2); gy <= Math.floor((y1 + 160) / STEP2); gy++) {
    for (let gx = Math.floor((x0 - 160) / STEP2); gx <= Math.floor((x1 + 160) / STEP2); gx++) {
      const h = cellHash(gx + 991, gy - 577);
      if (h % 100 > 52) continue;
      const wx = gx * STEP2 + 120 + ((h >>> 8) % (STEP2 - 240));
      const wy = gy * STEP2 + 120 + ((h >>> 16) % (STEP2 - 240));
      const spr = BIG_PROPS[(h >>> 5) % 2];
      const sz = 1.15 + ((h >>> 3) % 4) * 0.1;
      R.spr(S.shadow, wx, wy + 44 * sz, 3.0 * sz, 2.0 * sz, 0, 0.58, 0xffffff, 0, 0);
      R.spr(spr, wx, wy, (h & 2 ? -1 : 1) * sz, sz, 0, 0.95, decorTint, 0, 0);
    }
  }

  // Ground Zones (Vortex, Blast, Mine, Enemy Beam)
  for (let i = 0; i < world.nZ; i++) {
    const z = world.zn[i];
    if (z.kind === ZONE_KIND.VORTEX) {
      const k = Math.min(1, z.t * 4, (z.life - z.t) * 3);
      const scZ = z.r / 54;
      R.spr(S.disc, z.x, z.y, z.r / 28, z.r / 28, 0, 0.34 * k, 0x160d2e, 0, 0);
      R.spr(S.swirl, z.x, z.y, scZ, scZ, z.a, 0.9 * k, 0xffffff, 0, 0);
      R.spr(S.ring, z.x, z.y, z.r / 56, z.r / 56, 0, 0.5 * k, z.evo ? 0xffad36 : 0xa78bfa, 1, 0);
      if (z.evo) R.spr(S.rune, z.x, z.y, z.r / 50, z.r / 50, -z.a * 0.3, 0.4 * k, 0xffd166, 1, 0);
    } else if (z.kind === ZONE_KIND.BLAST) {
      const k = z.t / z.life;
      const blink = 0.6 + 0.4 * Math.sin(t * 26);
      R.spr(S.disc, z.x, z.y, z.r / 30, z.r / 30, 0, 0.2 + 0.1 * blink, 0xf43f5e, 0, 0);
      R.spr(S.disc, z.x, z.y, (z.r / 30) * k, (z.r / 30) * k, 0, 0.32, 0xfb7185, 1, 0);
      R.spr(S.rune, z.x, z.y, z.r / 56, z.r / 56, t * 2.4, 0.8, 0xfb7185, 1, 0);
      R.spr(S.ring, z.x, z.y, z.r / 56, z.r / 56, 0, 0.9, 0xfecdd3, 1, 0);
    } else if (z.kind === ZONE_KIND.MINE) {
      const k = Math.min(1, z.t / z.arm);
      const mx = z.ox + (z.x - z.ox) * k;
      const my = z.oy + (z.y - z.oy) * k - Math.sin(k * Math.PI) * 22;
      const scM = (z.mini ? 0.64 : 1) * (1 + 0.08 * Math.sin(t * 12 + i));
      R.spr(S.shadow, z.x, z.y + 9, 0.6 * scM, 0.6 * scM, 0, 0.5 * k, 0xffffff, 0, 0);
      if (k >= 1) R.spr(S.glow, mx, my, 1, 1, 0, 0.32 + 0.2 * Math.sin(t * 10 + i), 0xff6b6b, 1, 0);
      R.spr(S.mine, mx, my, scM, scM, Math.sin(t * 3 + i) * 0.2, 1, 0xffffff, 0, 0);
    } else if (z.kind === ZONE_KIND.EBEAM) {
      const ux = Math.cos(z.a), uy = Math.sin(z.a);
      const bx = z.x + (ux * z.len) / 2, by = z.y + (uy * z.len) / 2;
      if (z.t < z.arm) {
        R.spr(S.px, bx, by, z.len / 6, 3.6, z.a, 0.12 + 0.08 * Math.sin(t * 30), 0xf43f5e, 0, 0);
        R.spr(S.px, bx, by, z.len / 6, 0.5, z.a, 0.65, 0xfda4af, 1, 0);
      } else {
        const w = 0.85 + 0.15 * Math.sin(t * 46);
        R.spr(S.beam, bx, by, z.len / 64, 2.6 * w, z.a, 0.48, 0xe11d48, 1, 0);
        R.spr(S.beam, bx, by, z.len / 64, 1.2, z.a, 0.82, 0xfb7185, 1, 0);
        R.spr(S.beam, bx, by, z.len / 64, 0.45, z.a, 1, 0xffffff, 1, 0);
      }
    }
  }

  // Compute Gems
  for (let i = 0; i < world.nG; i++) {
    const g = world.gm[i];
    if (g.x < x0 || g.x > x1 || g.y < y0 || g.y > y1) continue;
    const scG = 1 + 0.1 * Math.sin(t * 6 + i);
    const bob = Math.sin(t * 3 + i * 1.7) * 1.5;
    if (g.st === 1) {
      const ang = Math.atan2(p.y - g.y, p.x - g.x);
      const len = Math.min(2.2, 0.5 + g.sp / 420);
      R.spr(S.streak, g.x - Math.cos(ang) * 9 * len, g.y - Math.sin(ang) * 9 * len, 0.45 * len, 0.9, ang, 0.75, GEM_GLOWS[g.tier], 1, 0);
    } else if (!lite) {
      R.spr(S.glow, g.x, g.y + bob, 0.42 + g.tier * 0.1, 0.42 + g.tier * 0.1, 0, 0.42, GEM_GLOWS[g.tier], 1, 0);
    }
    R.spr(GEM_SPRS[g.tier], g.x, g.y + bob, scG, scG, 0, 1, 0xffffff, 0, 0);
  }

  // Pickups
  for (let i = 0; i < world.nPK; i++) {
    const pk = world.pk[i];
    const bob = Math.sin(t * 3.4 + i) * 3;
    const scP = Math.min(1, pk.t * 5);
    const big = pk.kind === PICK_KIND.NUKE || pk.kind === PICK_KIND.CHEST;
    const gcol =
      pk.kind === PICK_KIND.CHEST ? 0xffd166 :
      pk.kind === PICK_KIND.NUKE ? 0x5ce1e6 :
      pk.kind === PICK_KIND.HEART ? 0xfb7185 :
      pk.kind === PICK_KIND.MAGNET ? 0x2ee59d : 0xffad36;
    R.spr(S.shadow, pk.x, pk.y + 15, 0.6, 0.6, 0, 0.5, 0xffffff, 0, 0);
    R.spr(S.glow, pk.x, pk.y + bob, big ? 2.4 : 1.3, big ? 2.4 : 1.3, 0, 0.34 + 0.16 * Math.sin(t * 6 + i), gcol, 1, 0);
    if (big) {
      const rp = (t * 0.7) % 1;
      R.spr(S.ring, pk.x, pk.y + bob, 0.3 + rp * 0.4, 0.3 + rp * 0.4, 0, 1 - rp, 0xffffff, 1, 0);
    }
    R.spr(PICK_SPRS[pk.kind], pk.x, pk.y + bob, scP, scP, pk.kind === PICK_KIND.CHEST ? Math.sin(t * 5) * 0.06 : 0, 1, 0xffffff, 0, 0);
  }

  // Enemies & Bosses
  if (!lite) {
    for (let i = 0; i < world.nE; i++) {
      const e = world.en[i];
      if (!e.dead && e.x >= x0 && e.x <= x1 && e.y >= y0 && e.y <= y1) {
        R.spr(S.shadow, e.x, e.y + e.r * 1.12, e.r / 20, e.r / 20, 0, 0.6, 0xffffff, 0, 0);
      }
    }
  }

  for (let i = 0; i < world.nE; i++) {
    const e = world.en[i];
    if (e.dead) continue;
    if (e.tele > 0) {
      const ang = Math.atan2(e.ay, e.ax);
      const blink = 0.5 + 0.5 * Math.sin(t * 28);
      R.spr(S.px, e.x + (e.ax * e.tele) / 2, e.y + (e.ay * e.tele) / 2, e.tele / 6, (e.r * 2) / 6, ang, 0.14 + 0.1 * blink, 0xf43f5e, 0, 0);
      R.spr(S.px, e.x + (e.ax * e.tele) / 2, e.y + (e.ay * e.tele) / 2, e.tele / 6, 0.3, ang, 0.72, 0xfda4af, 1, 0);
    }
    if (e.x < x0 || e.x > x1 || e.y < y0 || e.y > y1) continue;

    const wob = Math.sin(t * 7 + e.ph) * 0.055;
    let mul = e.elite ? 1.6 : 1;
    if (e.swell > 0) mul *= 1 + e.swell * 0.7;
    let sx = mul * (1 + wob);
    let sy = mul * (1 - wob);
    let rot = 0;
    if (e.flash > 0) {
      sx *= 1.1;
      sy *= 0.9;
    }
    const alpha = Math.min(1, (world.t - e.born) * 4 + 0.15);
    const col = ENEMY_SPARK_COLS[e.type];

    if (e.elite) {
      R.spr(S.rune, e.x, e.y, e.r / 34, e.r / 34, t * 1.2, 0.8, 0xffd166, 1, 0);
      R.spr(S.glow, e.x, e.y, e.r / 9, e.r / 9, 0, 0.32, 0xffad36, 1, 0);
    } else if (!lite && !e.boss) {
      R.spr(S.glow, e.x, e.y, e.r / 11, e.r / 11, 0, 0.24, col, 1, 0);
    }

    if (e.ai === AI_TYPE.DART) {
      rot = Math.atan2(p.y - e.y, p.x - e.x) + Math.sin(t * 5 + e.ph) * 0.25;
    } else {
      sx *= e.face;
      if (e.ai === AI_TYPE.DASH && e.st === 1) {
        rot = Math.sin(t * 60) * 0.12;
        sx *= 1.12;
        sy *= 1.12;
      }
    }

    if (e.boss) {
      const sw = e.swell > 0 ? e.swell * 0.3 : 0;
      const bp = 0.5 + 0.5 * Math.sin(t * 3);
      R.spr(S.rune, e.x, e.y, e.r / 36, e.r / 36, -t * 0.6, 0.5 + 0.2 * bp, col, 1, 0);
      R.spr(S.glow, e.x, e.y, e.r / 7.5, e.r / 7.5, 0, 0.26 + 0.1 * bp, col, 1, 0);
      sx = (1 + wob * 0.6) * (1 + sw) * (e.boss === 4 ? e.face : 1);
      sy = (1 - wob * 0.6) * (1 + sw);
    }

    R.spr(
      ENEMY_SPRS[e.type], e.x, e.y + (e.ai === AI_TYPE.GHOST ? Math.sin(t * 4 + e.ph) * 2 : 0),
      sx, sy, rot, alpha, 0xffffff, 0, e.flash > 0 ? Math.min(0.7, e.flash * 7) : 0,
    );

    if (e.elite) {
      R.spr(S.crown, e.x, e.y - e.r - 7 + Math.sin(t * 4) * 1.2, 1.25, 1.25, 0, 1, 0xffffff, 0, 0);
      const hpR = e.hp / e.maxHp;
      R.spr(S.px, e.x, e.y + e.r + 9, 5, 0.6, 0, 0.7, 0x091316, 0, 0);
      R.spr(S.px, e.x - 15 * (1 - hpR), e.y + e.r + 9, 5 * hpR, 0.45, 0, 1, 0xffad36, 0, 0);
    }
  }

  // Orbitals & Lasers
  for (let i = 0; i < p.weapons.length && !world.over; i++) {
    const w = p.weapons[i];
    const wid = WEAPONS[w.k].id;
    const cfg = w.evo ? WEAPONS[w.k].evo : WEAPONS[w.k].lv[w.lv - 1];
    if (wid === "moat" && w.vis > 0.03) {
      for (let n = 0; n < w.np; n++) {
        const ox = w.pos[n * 2];
        const oy = w.pos[n * 2 + 1];
        const scM = (w.evo ? 1.28 : 1.08) * w.vis;
        R.spr(S.glow, ox, oy, 0.65, 0.65, 0, 0.55 * w.vis, w.evo ? 0xffad36 : 0xffd97d, 1, 0);
        R.spr(S.moat, ox, oy, scM, scM, t * 3 + n, w.vis, 0xffffff, 0, 0);
      }
    } else if (wid === "wafer" && w.evo) {
      for (let n = 0; n < w.np; n++) {
        const ox = w.pos[n * 2];
        const oy = w.pos[n * 2 + 1];
        const scW = cfg.size * Math.sqrt(p.st.area) * 0.92;
        const ang = Math.atan2(oy - p.y, ox - p.x) + Math.PI / 2;
        R.spr(S.streak, ox - Math.cos(ang) * 16, oy - Math.sin(ang) * 16, 0.9 * scW, 1.6 * scW, ang, 0.5, 0x5ce1e6, 1, 0);
        R.spr(S.glow, ox, oy, scW, scW, 0, 0.5, 0x5ce1e6, 1, 0);
        R.spr(S.wafer, ox, oy, scW, scW, t * 10 + n, 1, 0xffffff, 0, 0);
      }
    } else if (wid === "laser" && w.bt > 0) {
      const len = cfg.len * p.st.area;
      const widPx = cfg.wid * Math.sqrt(p.st.area);
      const fade = w.evo ? 1 : Math.min(1, w.bt * 6);
      const flk = 0.85 + 0.15 * Math.sin(t * 50);
      const bcol = w.evo ? 0x5ce1e6 : 0xffd166;
      for (let n = 0; n < cfg.n; n++) {
        const a = w.ang + (n / cfg.n) * TAU;
        const ux = Math.cos(a), uy = Math.sin(a);
        const cx = p.x + ux * (len / 2 + 8), cy = p.y + uy * (len / 2 + 8);
        R.spr(S.beam, cx, cy, len / 64, (widPx / 12.5) * flk, a, 0.34 * fade, bcol, 1, 0);
        R.spr(S.beam, cx, cy, len / 64, widPx / 26, a, 0.62 * fade, 0xffad36, 1, 0);
        R.spr(S.beam, cx, cy, len / 64, widPx / 60, a, 0.95 * fade, 0xffffff, 1, 0);
      }
    }
  }

  // Pulse Rings
  for (let i = 0; i < world.nZ; i++) {
    const z = world.zn[i];
    if (z.kind !== ZONE_KIND.PULSE) continue;
    const k = z.r / z.rMax;
    R.spr(S.ring2, z.x, z.y, z.r / 51, z.r / 51, 0, 1 - k * 0.7, z.slow ? 0x5ce1e6 : 0x2ee59d, 1, 0);
    R.spr(S.ring, z.x, z.y, (z.r / 56) * 0.86, (z.r / 56) * 0.86, 0, (1 - k) * 0.7, 0xffffff, 1, 0);
  }

  // Player Hero
  if (!world.over) {
    const sprId = S["h_" + view.hero + (view.state || "")] ?? S["h_" + view.hero];
    let bob = Math.sin(t * 5.5) * 1.4;
    let tilt = Math.max(-0.26, Math.min(0.26, (p.my || 0) * 0.24)) * p.face;
    const br = 1 + Math.sin(t * 5.5) * 0.025;
    let sx = br, sy = 1 / br, jx = 0;
    if (view.joy > 0) {
      bob -= Math.abs(Math.sin(t * 9)) * 7 * view.joy;
      sx *= 1 + 0.1 * view.joy;
      sy *= 1 + 0.1 * view.joy;
    }
    if (view.hurt > 0) {
      jx = Math.sin(t * 70) * 3 * view.hurt;
      sx *= 1 + 0.12 * view.hurt;
      sy *= 1 - 0.1 * view.hurt;
    }
    if (!lite) R.spr(S.shadow, p.x, p.y + 22, 1.08, 0.9, 0, 0.65, 0xffffff, 0, 0);
    R.spr(S.glow, p.x, p.y + bob, 1.6, 1.6, 0, 0.24, view.hero === "gold" ? 0xffd166 : 0xffad36, 1, 0);
    const invA = p.inv > 0 && !world.pending && view.state !== "_joy" ? (Math.sin(t * 38) > 0 ? 1 : 0.5) : 1;
    R.wave(
      sprId, p.x + jx, p.y + bob, p.face * sx, sy, tilt, invA, 0xffffff, 0,
      view.hurt > 0.4 ? 0.45 * view.hurt : 0, t * (p.moving ? 14 : 5), p.moving ? 2.4 : 1.1,
    );
  }

  // Player Projectiles
  for (let i = 0; i < world.nB; i++) {
    const b = world.bl[i];
    if (b.kind === BULLET_KIND.BLADE) {
      const ang = Math.atan2(b.vy, b.vx);
      const scB = b.size;
      R.spr(S.streak, b.x - Math.cos(ang) * 12, b.y - Math.sin(ang) * 12, 0.65 * scB, 1.5 * scB, ang, 0.55, b.pop ? 0xffd166 : 0xffad36, 1, 0);
      R.spr(S.glow, b.x, b.y, 0.52 * scB, 0.52 * scB, 0, 0.48, 0xffad36, 1, 0);
      R.spr(S.blade, b.x, b.y, scB, scB, ang, 1, 0xffffff, 0, 0);
    } else {
      const ang = Math.atan2(b.vy, b.vx) + (b.st ? Math.PI : 0);
      R.spr(S.streak, b.x - Math.cos(ang) * 14, b.y - Math.sin(ang) * 14, 0.9 * b.size, 2 * b.size, ang, 0.5, 0x5ce1e6, 1, 0);
      R.spr(S.glow, b.x, b.y, b.size * 0.9, b.size * 0.9, 0, 0.45, 0x5ce1e6, 1, 0);
      R.spr(S.wafer, b.x, b.y, b.size * 0.95, b.size * 0.95, b.rot, 1, 0xffffff, 0, 0);
    }
  }

  // Enemy Bullets
  for (let i = 0; i < world.nEB; i++) {
    const eb = world.eb[i];
    const scE = (eb.r / 6) * (1 + 0.12 * Math.sin(t * 16 + i));
    const ang = Math.atan2(eb.vy, eb.vx);
    R.spr(S.streak, eb.x - Math.cos(ang) * 9, eb.y - Math.sin(ang) * 9, 0.5 * scE, 1.5 * scE, ang, 0.6, 0xf43f5e, 1, 0);
    R.spr(S.ebullet, eb.x, eb.y, scE, scE, 0, 1, 0xffffff, 0, 0);
  }

  fx.draw(R);

  // Player Overhead HP Bar
  if (!world.over && !view.dim) {
    const hpR = Math.max(0, p.hp / p.maxHp);
    const hcol = hpR > 0.5 ? 0x2ee59d : hpR > 0.25 ? 0xffad36 : 0xf43f5e;
    R.spr(S.px, p.x, p.y + 29, 6.4, 0.95, 0, 0.75, 0x071012, 0, 0);
    R.spr(S.px, p.x - 18 * (1 - hpR), p.y + 29, 6 * hpR, 0.6, 0, 1, hcol, 0, 0);
  }

  fx.drawNums(R);

  // Screen-space Post-FX, Light Shafts, Floating Sparks & Virtual Joystick
  R.screen();
  const cw = view.cw, ch = view.ch;
  const shaftCol = rgbInt(zn.sc);
  const dustCol = rgbInt(zn.dust);
  const nShafts = lite ? 2 : 4;
  for (let i = 0; i < nShafts; i++) {
    const sx = cw * (0.12 + i * 0.26) + Math.sin(t * 0.3 + i * 2) * 30 - ((camX * sc * 0.06) % 40);
    R.spr(S.shaft, sx, ch * 0.34, (cw / 64) * (0.2 + (i % 3) * 0.07), ch / 256, 0.32, zn.shaft * (0.7 + 0.3 * Math.sin(t * 0.7 + i * 1.3)), shaftCol, 1, 0);
  }
  const nDust = lite ? 10 : 28;
  for (let i = 0; i < nDust; i++) {
    const h = cellHash(i, 77);
    const par = 0.3 + (i % 5) * 0.3;
    const mw = cw + 80, mh = ch + 80;
    let dx = (((h % 1000) / 1000) * mw - camX * sc * par + t * (6 + (i % 4) * 3)) % mw;
    if (dx < 0) dx += mw;
    let dy = ((((h >>> 10) % 1000) / 1000) * mh - camY * sc * par - t * (8 + (i % 3) * 5)) % mh;
    if (dy < 0) dy += mh;
    const dsz = (0.25 + par * 0.4) * (view.uz || 1);
    R.spr(S.dotp, dx - 40, dy - 40, dsz, dsz, 0, (0.16 + (i % 3) * 0.09) * (0.6 + 0.4 * Math.sin(t * 1.7 + i)), i % 4 ? dustCol : 0xffffff, 1, 0);
  }

  if (zn.wa > 0.005) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, zn.wa, rgbInt(zn.wash), 1, 0);
  R.spr(S.vig, cw / 2, ch / 2, (cw / 128) * 1.04, (ch / 128) * 1.04, 0, Math.min(1, zn.vig), 0xffffff, 0, 0);
  if (view.dim) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, view.dim, 0x050b0d, 0, 0);

  const lowHp = world.over ? 0 : Math.max(0, 1 - p.hp / p.maxHp / 0.35);
  const redV = Math.max(fx.hurtV * 0.75, view.dim ? 0 : lowHp * (0.32 + 0.14 * Math.sin(t * 6)));
  if (redV > 0.01) R.spr(S.vigr, cw / 2, ch / 2, (cw / 128) * 1.04, (ch / 128) * 1.04, 0, redV, 0xffffff, 0, 0);
  if (fx.tone > 0.01) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, fx.tone, fx.toneCol, 1, 0);
  if (fx.flash > 0.01) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, fx.flash, fx.flashCol, 1, 0);

  if (view.joyst && view.joyst.on) {
    const uz = view.uz || 1;
    R.spr(S.joy, view.joyst.bx, view.joyst.by, uz, uz, 0, 0.9, 0xffffff, 0, 0);
    R.spr(S.knob, view.joyst.kx, view.joyst.ky, uz, uz, 0, 0.95, 0xffffff, 0, 0);
  }
  R.end();
}
