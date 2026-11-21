// HONE Adventure V2 — Cleanroom AI Data Center Renderer, PCB Trace FX & High-Contrast Projectiles

import { AI_TYPE, ENEMIES, STAGES, WEAPONS } from "./data.js";
import { BULLET_KIND, FX_EVT, PICK_KIND, ZONE_KIND } from "./world.js";
import { SPR_IDX } from "./art.js";
import { playGemChime, playKillPop, playSfx } from "./audio.js";

const MAX_PARTS = 950;
const MAX_NUMS = 120;
const MAX_RINGS = 84;
const MAX_BOLTS = 56;
const TAU = Math.PI * 2;

const NUM_NORM = [];
const NUM_CRIT = [];
for (let i = 0; i < 10; i++) {
  NUM_NORM.push(SPR_IDX["n" + i]);
  NUM_CRIT.push(SPR_IDX["c" + i]);
}

export function createFxSystem(hooks) {
  const parts = Array.from({ length: MAX_PARTS }, () => ({
    x: 0, y: 0, vx: 0, vy: 0, life: 0, max: 1, s0: 1, s1: 1, rot: 0, vr: 0,
    col: 0xffffff, spr: 0, drag: 0, g: 0, ar: 1, a: 1,
  }));
  const nums = Array.from({ length: MAX_NUMS }, () => ({ x: 0, y: 0, life: 0, val: 0, crit: 0 }));
  const rings = Array.from({ length: MAX_RINGS }, () => ({
    x: 0, y: 0, r0: 0, r1: 0, life: 0, max: 1, spr: 0, a: 1, rot: 0,
  }));
  const bolts = Array.from({ length: MAX_BOLTS }, () => ({ x0: 0, y0: 0, x1: 0, y1: 0, life: 0, evo: 0 }));
  const timers = [];

  const S = SPR_IDX;
  const fx = {
    nP: 0, nN: 0, nR: 0, nBo: 0,
    trauma: 0, flash: 0, flashCol: 0xffffff, tone: 0, toneCol: 0x2563eb,
    hurtV: 0, q: 1, hitN: 0, killN: 0, numN: 0, gemN: 0, boomN: 0,
    stopCd: 0, shX: 0, shY: 0, quiet: false, shakeOn: true, t: 0, gemGlow: 0, hitI: 0,
  };

  const panAt = (wx) => {
    const p = (wx - hooks.camX()) / (hooks.viewW() * 0.5);
    return p < -1 ? -0.8 : p > 1 ? 0.8 : p * 0.8;
  };
  const sfx = (name, wx, rate, vol) => {
    if (!fx.quiet) playSfx(name, wx == null ? 0 : panAt(wx), rate, vol);
  };
  const qCnt = (n) => Math.max(1, Math.ceil(n * fx.q));

  function addPart(spr, x, y, vx, vy, life, s0, s1, col = 0xffffff, drag = 0, g = 0, rot = 0, vr = 0, ar = 1, a = 1) {
    if (fx.nP >= MAX_PARTS) return null;
    const p = parts[fx.nP++];
    p.spr = spr; p.x = x; p.y = y; p.vx = vx; p.vy = vy;
    p.life = life; p.max = life; p.s0 = s0; p.s1 = s1; p.col = col;
    p.drag = drag; p.g = g; p.rot = rot; p.vr = vr; p.ar = ar; p.a = a;
    return p;
  }

  function addRing(x, y, r0, r1, life, spr = S.ring, a = 0.85, rot = 0) {
    if (fx.nR >= MAX_RINGS) return;
    const r = rings[fx.nR++];
    r.x = x; r.y = y; r.r0 = r0; r.r1 = r1; r.life = life; r.max = life; r.spr = spr; r.a = a; r.rot = rot;
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
    slot.y = y - 12;
    slot.life = crit ? 0.82 : 0.56;
    slot.val = val;
    slot.crit = crit;
  }

  function burstChips(x, y, cnt, spd, sz, life) {
    cnt = qCnt(cnt);
    for (let i = 0; i < cnt; i++) {
      const a = Math.random() * TAU;
      const v = spd * (0.35 + Math.random() * 0.65);
      const spr = i % 3 === 0 ? S.spark : i % 3 === 1 ? S.shard : S.dotp;
      addPart(
        spr, x, y, Math.cos(a) * v, Math.sin(a) * v,
        life * (0.65 + Math.random() * 0.55), sz * (0.75 + Math.random() * 0.6), 0.12,
        0xffffff, 3.4, 0, Math.random() * TAU, (Math.random() - 0.5) * 14, 1, 0.92,
      );
    }
  }

  const later = (sec, fn) => timers.push({ t: sec, fn });

  fx.ring = (x, y, r0, r1, life) => addRing(x, y, r0, r1, life, S.ring, 0.8);
  fx.bubbles = () => {}; // Cleanroom: zero deep-sea bubbles!
  // Peak-envelope non-stacking impulse shake: never accumulates across rapid late-game hits/explosions
  fx.shake = (v, cap = 0.65) => {
    if (!fx.shakeOn || fx.quiet) return;
    const lim = Math.min(cap, v);
    if (lim > fx.trauma) fx.trauma = lim;
  };
  fx.setTone = (v, col) => {
    if (v > fx.tone) {
      fx.tone = v;
      fx.toneCol = col;
    }
  };

  fx.levelFx = (x, y) => {
    addPart(S.rune, x, y, 0, 0, 0.78, 0.5, 1.65, 0xffffff, 0, 0, 0, 2.2, 1, 0.85);
    addRing(x, y, 14, 175, 0.5, S.ring_gold, 0.9);
    addRing(x, y, 20, 245, 0.68, S.ring2, 0.75);
    burstChips(x, y, 14, 155, 0.95, 0.55);
  };

  fx.on = (evt, a, b, c, d, e) => {
    switch (evt) {
      case FX_EVT.HIT:
        fx.hitN++;
        if (d || fx.numN < 9) {
          addNum(a, b, c, d);
          fx.numN++;
        }
        if (fx.hitN < 10) {
          addPart(S.flare, a, b, 0, 0, 0.13, 0.3, 0.58, 0xffffff, 0, 0, Math.random() * 1.5, 0, 1, 0.85);
        }
        if (d) {
          if (fx.hitN < 18) {
            addRing(a, b, 5, 32, 0.2, S.ring_gold, 0.9);
            burstChips(a, b, 3, 190, 0.72, 0.22);
          }
          sfx("crit", a, 0.94 + Math.random() * 0.14);
        } else {
          sfx((fx.hitI = (fx.hitI | 0) + 1) & 1 ? "hit" : "hit2", a, 0.88 + Math.random() * 0.3);
        }
        break;

      case FX_EVT.KILL: {
        fx.killN++;
        const big = d > 0;
        const rad = ENEMIES[c].r * (d === 1 ? 1.6 : 1);
        burstChips(a, b, big ? 14 : fx.killN < 6 ? 5 : 3, big ? 250 : 165, big ? 1.35 : 0.9, 0.4);
        if (fx.killN < 18) {
          addRing(a, b, rad * 0.45, rad * 2.1, 0.22, big ? S.ring_gold : S.ring, 0.78);
        }
        if (!fx.quiet) playKillPop(panAt(a), big, hooks.combo());
        if (d === 1) {
          sfx("shatter", a);
          hooks.hitstop(0.045);
          fx.shake(0.26, 0.32);
          hooks.vibrate("medium");
        }
        break;
      }

      case FX_EVT.HURT:
        fx.shake(0.42, 0.48);
        fx.hurtV = 1;
        hooks.hitstop(0.05);
        sfx("hurt", b);
        hooks.vibrate("medium");
        hooks.hurt();
        addRing(b, c, 8, 68, 0.28, S.ring_warn, 0.92);
        burstChips(b, c, 7, 155, 0.95, 0.32);
        break;

      case FX_EVT.SHOT: {
        const wid = WEAPONS[a].id;
        if (wid === "hbm") sfx("shoot", b, d ? 1.25 + Math.random() * 0.2 : 0.98 + Math.random() * 0.12, d ? 0.7 : 1);
        else if (wid === "prism") sfx("beam", b, 1.18, 0.85);
        else if (wid === "tensor") sfx("drop", b, 0.95);
        else if (wid === "cache") sfx("magnet", b, 1.45, 0.4);
        else if (wid === "bus") sfx("zap", b, 0.92 + Math.random() * 0.2);
        else if (wid === "litho") sfx("beam", b, 0.85);
        else if (wid === "drone") sfx("star", b, 0.96 + Math.random() * 0.1);
        else if (wid === "cryo") sfx("whirl", b, d ? 0.82 : 1);
        break;
      }

      case FX_EVT.GEM:
        if (!fx.quiet) playGemChime(panAt(b), a);
        fx.gemGlow = 1;
        if (fx.gemN++ < 4) {
          addPart(S.spark, b, c, 0, -24, 0.18, 0.45, 0.1, 0xffffff, 0, 0, 0, 3, 1, 0.85);
        }
        break;

      case FX_EVT.PICK:
        addRing(b, c, 6, 48, 0.28, a === PICK_KIND.CHEST || a === PICK_KIND.CORE ? S.ring_gold : S.ring2, 0.9);
        burstChips(b, c, 6, 120, 0.85, 0.34);
        if (a === PICK_KIND.HEART) sfx("heart", b);
        else if (a === PICK_KIND.CORE) sfx("core", b);
        else if (a === PICK_KIND.CHEST) sfx("chest", b);
        break;

      case FX_EVT.BOLT:
        if (fx.nBo < MAX_BOLTS) {
          const bo = bolts[fx.nBo++];
          bo.x0 = a; bo.y0 = b; bo.x1 = c; bo.y1 = d;
          bo.life = 0.18; bo.evo = e ? 1 : 0;
        }
        addPart(S.via, c, d, 0, 0, 0.18, 0.95, 0.5, 0xffffff, 0, 0, 0, 0, 1, 0.95);
        break;

      case FX_EVT.RING:
        addRing(a, b, 14, c, 0.32, S.ring2, 0.85);
        break;

      case FX_EVT.MATRIX: {
        const full = fx.boomN++ < 4;
        addRing(a, b, c * 0.32, c * 1.04, 0.24, S.ring2, full ? 0.78 : 0.48, Math.PI / 6);
        if (full) burstChips(a, b, 5, c * 2.8, 0.78, 0.22);
        sfx("boom", a, 1.08 + Math.random() * 0.18, 0.76);
        // Zero camera shake on routine weapon matrix blasts so late-game mowing stays rock-steady!
        break;
      }

      case FX_EVT.BOOM: {
        const isEnemy = d === 99;
        const full = fx.boomN++ < 3;
        addRing(a, b, c * 0.25, c * 1.05, 0.28, isEnemy ? S.ring_warn : S.ring_gold, full ? 0.88 : 0.5);
        if (full) burstChips(a, b, 6, c * 3.5, 0.8, 0.24);
        sfx("boom", a, isEnemy ? 0.8 : 0.95 + Math.random() * 0.25, c > 80 ? 1.1 : 0.8);
        // Only enemy hazard blasts give a subtle non-stacking nudge; player weapon AoEs stay steady
        if (isEnemy) fx.shake(0.18, 0.22);
        break;
      }

      case FX_EVT.POP:
        addRing(a, b, 4, 16, 0.16, S.ring, 0.7);
        break;

      case FX_EVT.NUKE:
        fx.flash = Math.max(fx.flash, 0.18);
        fx.flashCol = 0xdbeafe;
        fx.shake(0.55, 0.6);
        hooks.hitstop(0.08);
        hooks.slowmo(0.35, 0.48);
        hooks.vibrate("heavy");
        sfx("nuke", null);
        addRing(a, b, 24, 580, 0.72, S.ring, 0.92);
        addRing(a, b, 18, 440, 0.56, S.ring_gold, 0.95);
        hooks.announce("超频脉冲 · 全机房故障清零！", "big");
        break;

      case FX_EVT.COMBO:
        hooks.announce("连击 ×" + a, "combo", b);
        sfx("combo", null, 1 + b * 0.06);
        break;

      case FX_EVT.WAVE:
        hooks.announce("第 " + a + " 波负载洪峰", "wave");
        sfx("wave", null);
        break;

      case FX_EVT.STAGE: {
        const stg = STAGES[a] || STAGES[0];
        hooks.announce("产线跃迁 · " + stg.name, "big");
        sfx("evo", null, 1.05, 0.85);
        break;
      }

      case FX_EVT.BOSSWARN:
        hooks.announce("巨兽级硬件瓶颈预警", "warn", a);
        sfx("warn", null);
        hooks.vibrate("heavy");
        break;

      case FX_EVT.BOSS:
        fx.shake(0.38, 0.45);
        sfx("elite", a, 0.7, 1.3);
        addRing(a, b, 30, 220, 0.6, S.ring_warn, 0.92);
        addRing(a, b, 30, 310, 0.78, S.ring_gold, 0.8);
        break;

      case FX_EVT.BOSSDEAD:
        fx.flash = Math.max(fx.flash, 0.22);
        fx.flashCol = 0xfef3c7;
        fx.shake(0.62, 0.65);
        hooks.hitstop(0.16);
        hooks.slowmo(0.25, 1.0);
        hooks.vibrate("heavy");
        sfx("bossdead", null);
        addRing(a, b, 30, 660, 0.88, S.ring_gold, 0.95);
        addRing(a, b, 30, 480, 0.68, S.ring, 0.9);
        burstChips(a, b, 42, 460, 1.65, 0.85);
        for (let i = 0; i < 5; i++) {
          later(0.12 + i * 0.13, () => {
            const rx = a + (Math.random() - 0.5) * 110;
            const ry = b + (Math.random() - 0.5) * 90;
            addRing(rx, ry, 10, 88, 0.28, S.ring_gold, 0.9);
            burstChips(rx, ry, 8, 280, 0.9, 0.28);
            sfx("boom", rx, 0.85 + Math.random() * 0.35);
          });
        }
        hooks.announce("瓶颈解除 · 击破「" + ENEMIES[8 + c].name + "」！", "big");
        break;

      case FX_EVT.ELITE:
        hooks.announce("机柜级高危异常出现", "wave");
        sfx("elite", a);
        break;

      case FX_EVT.DEAD:
        fx.flash = Math.max(fx.flash, 0.25);
        fx.flashCol = 0xfecdd3;
        fx.shake(0.55, 0.6);
        hooks.vibrate("heavy");
        sfx("dead", null);
        addRing(a, b, 10, 240, 0.8, S.ring_warn, 0.95);
        burstChips(a, b, 36, 300, 1.45, 0.85);
        break;

      case FX_EVT.REVIVE:
        fx.flash = Math.max(fx.flash, 0.24);
        fx.flashCol = 0xd1fae5;
        fx.shake(0.48, 0.52);
        sfx("revive", null);
        addRing(a, b, 20, 400, 0.7, S.ring2, 0.95);
        hooks.announce("UPS 灾备电源切入 · 恢复半血反击！", "big");
        hooks.slowmo(0.3, 0.55);
        break;

      case FX_EVT.EVO:
        fx.flash = Math.max(fx.flash, 0.22);
        fx.flashCol = 0xfef3c7;
        fx.shake(0.42, 0.48);
        sfx("evo", null);
        hooks.vibrate("heavy");
        hooks.slowmo(0.3, 0.65);
        addRing(b, c, 20, 420, 0.78, S.ring_gold, 0.95);
        addRing(b, c, 20, 520, 0.95, S.ring, 0.85);
        addPart(S.rune, b, c, 0, 0, 1.0, 0.75, 2.5, 0xffffff, 0, 0, 0, -2.2, 1, 0.9);
        burstChips(b, c, 24, 240, 1.2, 0.7);
        hooks.announce("超维流片成功 · " + WEAPONS[a].evoName + "！", "evo");
        break;

      case FX_EVT.ESHOT:
        sfx("eshot", a, d ? 0.72 : 1, d ? 1.5 : 1);
        break;

      case FX_EVT.MAGNET:
        sfx("magnet", null);
        addRing(a, b, 380, 20, 0.44, S.ring, 0.85);
        break;

      case FX_EVT.DASH:
        sfx("dash", a);
        break;

      case FX_EVT.HEAL:
        sfx("heart", a);
        addRing(a, b, 10, 64, 0.35, S.ring2, 0.85);
        break;

      case FX_EVT.NOVA:
        addRing(a, b, 14, c, 0.34, S.ring, 0.8);
        break;

      default:
        break;
    }
  };

  fx.update = (simDt, rawDt) => {
    fx.t += rawDt;
    fx.hitN = fx.killN = fx.numN = fx.gemN = fx.boomN = 0;
    if (fx.stopCd > 0) fx.stopCd -= rawDt;
    if (!fx.shakeOn) fx.trauma = 0;
    else fx.trauma = Math.max(0, fx.trauma - rawDt * 4.2);
    // Gentle, fast-damped micro-impulse (max ~2px even on Boss kill / Nuke)
    if (fx.trauma > 0) {
      const sh = fx.trauma * fx.trauma * 3.2;
      fx.shX = (Math.sin(fx.t * 34.7) + Math.sin(fx.t * 53.1) * 0.45) * sh * 0.55;
      fx.shY = (Math.cos(fx.t * 41.3) + Math.sin(fx.t * 61.9) * 0.45) * sh * 0.55;
    } else {
      fx.shX = 0;
      fx.shY = 0;
    }
    if (fx.flash > 0) fx.flash = Math.max(0, fx.flash - rawDt * 3.0);
    if (fx.tone > 0) fx.tone = Math.max(0, fx.tone - rawDt * 2.0);
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

  // Draw Particles, Shock Rings & 45°/90° Manhattan PCB Bus Traces!
  fx.draw = (R) => {
    for (let i = 0; i < fx.nR; i++) {
      const r = rings[i];
      const u = 1 - r.life / r.max;
      const ease = 1 - (1 - u) * (1 - u);
      const rad = r.r0 + (r.r1 - r.r0) * ease;
      const sc = rad / 54;
      R.spr(r.spr, r.x, r.y, sc, sc, r.rot, (1 - u) * r.a, 0xffffff, 0, 0);
    }

    for (let i = 0; i < fx.nP; i++) {
      const p = parts[i];
      const k = p.life / p.max;
      const sc = p.s1 + (p.s0 - p.s1) * k;
      R.spr(p.spr, p.x, p.y, sc, sc * p.ar, p.rot, (k < 0.45 ? k / 0.45 : 1) * p.a, p.col, 0, 0);
    }

    // 45°/90° PCB Trace Routing (instead of random jagged lightning!)
    for (let i = 0; i < fx.nBo; i++) {
      const bo = bolts[i];
      const alpha = Math.min(1, bo.life / 0.1);
      const dx = bo.x1 - bo.x0;
      const dy = bo.y1 - bo.y0;
      const adx = Math.abs(dx);
      const ady = Math.abs(dy);
      const diag = Math.min(adx, ady);
      const kx = bo.x0 + Math.sign(dx) * diag;
      const ky = bo.y0 + Math.sign(dy) * diag;
      const traceSpr = bo.evo ? S.p_bus : S.p_bus_c;

      // Segment 1: 45-degree diagonal trace
      const d1 = Math.sqrt((kx - bo.x0) ** 2 + (ky - bo.y0) ** 2);
      if (d1 > 2) {
        const a1 = Math.atan2(ky - bo.y0, kx - bo.x0);
        R.spr(traceSpr, (bo.x0 + kx) * 0.5, (bo.y0 + ky) * 0.5, (d1 + 4) / 64, 1, a1, alpha, 0xffffff, 0, 0);
      }
      // Segment 2: orthogonal horizontal/vertical trace
      const d2 = Math.sqrt((bo.x1 - kx) ** 2 + (bo.y1 - ky) ** 2);
      if (d2 > 2) {
        const a2 = Math.atan2(bo.y1 - ky, bo.x1 - kx);
        R.spr(traceSpr, (kx + bo.x1) * 0.5, (ky + bo.y1) * 0.5, (d2 + 4) / 64, 1, a2, alpha, 0xffffff, 0, 0);
      }
      // Solder via nodes at elbow & endpoint
      if (d1 > 4 && d2 > 4) R.spr(S.via, kx, ky, 0.85, 0.85, 0, alpha, 0xffffff, 0, 0);
      R.spr(S.via, bo.x1, bo.y1, 1.0, 1.0, 0, alpha, 0xffffff, 0, 0);
    }
  };

  fx.drawNums = (R) => {
    for (let i = 0; i < fx.nN; i++) {
      const n = nums[i];
      const dur = n.crit ? 0.82 : 0.56;
      const u = 1 - n.life / dur;
      const pop = u < 0.16 ? 1 + (1 - u / 0.16) * 0.85 : 1;
      const sc = (n.crit ? 1.24 : 0.8) * pop;
      const alpha = n.life < 0.18 ? n.life / 0.18 : 1;
      const y = n.y - (1 - (1 - u) * (1 - u)) * (n.crit ? 32 : 20);
      const glyphs = n.crit ? NUM_CRIT : NUM_NORM;
      const step = (n.crit ? 12 : 10.5) * sc;
      let val = n.val | 0;
      const digits = val >= 1e4 ? 5 : val >= 1e3 ? 4 : val >= 100 ? 3 : val >= 10 ? 2 : 1;
      let x = n.x + (digits - 1) * step * 0.5;
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

// 6 Light Cleanroom AI Data Center Zones (Matched 1:1 with STAGES)
export const ZONES = {};
STAGES.forEach((s) => {
  ZONES[s.id] = {
    name: s.name,
    gr: s.floorTint.slice(),
    gridCol: s.gridCol,
    propMode: s.propMode,
  };
});

export function pickZoneKey(world) {
  return STAGES[world.stageIdx]?.id || "storage";
}

export function cloneZone(key) {
  const src = ZONES[key] || ZONES.storage;
  return {
    name: src.name,
    gr: src.gr.slice(),
    gridCol: src.gridCol,
    propMode: src.propMode,
  };
}

export function lerpZone(cur, key, alpha) {
  const tgt = ZONES[key] || ZONES.storage;
  cur.gr[0] += (tgt.gr[0] - cur.gr[0]) * alpha;
  cur.gr[1] += (tgt.gr[1] - cur.gr[1]) * alpha;
  cur.gr[2] += (tgt.gr[2] - cur.gr[2]) * alpha;
  cur.gridCol = tgt.gridCol;
  cur.propMode = tgt.propMode;
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
  const WDM_SPRS = [S.p_w0, S.p_w1, S.p_w2];

  const t = view.t;
  const sc = view.sc;
  const camX = view.camX;
  const camY = view.camY;
  const hw = view.cw / 2 / sc;
  const hh = view.ch / 2 / sc;
  const p = world.p;
  const zn = view.zone;
  const lite = view.lite;
  const x0 = camX - hw - 80, x1 = camX + hw + 80;
  const y0 = camY - hh - 80, y1 = camY + hh + 80;

  R.begin();
  R.world(camX, camY, sc);

  // 1. Bright Cleanroom AI Data Center Raised Floor + Subtle Bus-Rail Blueprint Overlay
  R.tile(0, 0, 0, 384, 1, 0, rgbInt(zn.gr));
  R.tile(1, 0, 0, 512, 0.78, 0, zn.gridCol || 0x3b82f6);

  // 2. Orthogonal Cleanroom Aisle Bus Rails (100% flat on floor, zero entity occlusion!)
  const AISLE = 480;
  const railCol = zn.gridCol || 0x3b82f6;
  for (let gx = Math.floor(x0 / AISLE); gx <= Math.floor(x1 / AISLE); gx++) {
    const rx = gx * AISLE;
    R.spr(S.px, rx, camY, 0.75, (hh * 2 + 120) / 6, 0, 0.12, railCol, 0, 0);
  }
  for (let gy = Math.floor(y0 / AISLE); gy <= Math.floor(y1 / AISLE); gy++) {
    const ry = gy * AISLE;
    R.spr(S.px, camX, ry, (hw * 2 + 120) / 6, 0.75, 0, 0.12, railCol, 0, 0);
  }

  // 3. Sparse Cleanroom Equipment Bays (Wide 860px spacing, compact scale, drawn beneath everything!)
  const BAY_STEP = 860;
  const propList =
    zn.propMode === 1 ? [S.d_crystal, S.d_crystal] :
    zn.propMode === 3 ? [S.d_monolith, S.d_crystal] :
    zn.propMode === 2 ? [S.d_rack, S.d_crystal] : [S.d_rack, S.d_monolith];

  for (let gy = Math.floor((y0 - 120) / BAY_STEP); gy <= Math.floor((y1 + 120) / BAY_STEP); gy++) {
    for (let gx = Math.floor((x0 - 120) / BAY_STEP); gx <= Math.floor((x1 + 120) / BAY_STEP); gx++) {
      const h = cellHash(gx + 313, gy - 719);
      if (h % 100 > 46) continue;
      const wx = gx * BAY_STEP + 200 + ((h >>> 8) % (BAY_STEP - 400));
      const wy = gy * BAY_STEP + 200 + ((h >>> 16) % (BAY_STEP - 400));
      const spr = propList[(h >>> 4) % propList.length];
      R.spr(S.bay_pad, wx, wy + 10, 0.78, 0.78, 0, 0.7, 0xffffff, 0, 0);
      R.spr(S.shadow, wx, wy + 22, 1.3, 1.0, 0, 0.55, 0xffffff, 0, 0);
      R.spr(spr, wx, wy, 0.76, 0.76, 0, 0.76, 0xffffff, 0, 0);
    }
  }

  // 4. Ground Zones (Cryo Hex Plate, Tensor 3x3 Matrix, Hazard Blast Reticle, Boss Laser)
  for (let i = 0; i < world.nZ; i++) {
    const z = world.zn[i];
    if (z.kind === ZONE_KIND.CRYO) {
      const k = Math.min(1, z.t * 4, (z.life - z.t) * 3);
      const scZ = z.r / 56;
      R.spr(S.p_cryo, z.x, z.y, scZ, scZ, z.a * 0.35, 0.92 * k, 0xffffff, 0, 0);
      if (z.evo) R.spr(S.ring_gold, z.x, z.y, z.r / 54, z.r / 54, -z.a * 0.25, 0.65 * k, 0xffffff, 0, 0);
    } else if (z.kind === ZONE_KIND.TENSOR) {
      const k = Math.min(1, z.t / Math.max(0.12, z.arm));
      const snap = 1.18 - 0.18 * k;
      const scT = (z.r / 56) * snap;
      R.spr(S.p_tensor, z.x, z.y, scT, scT, (1 - k) * 0.22, 0.35 + 0.45 * k, 0xffffff, 0, 0);
    } else if (z.kind === ZONE_KIND.BLAST) {
      const k = z.t / z.life;
      const scB = z.r / 52;
      R.spr(S.p_hazard, z.x, z.y, scB, scB, 0, 0.92, 0xffffff, 0, 0);
      R.spr(S.px, z.x, z.y, (z.r * 2 * k) / 6, (z.r * 2 * k) / 6, 0, 0.18, 0xe11d48, 0, 0);
    } else if (z.kind === ZONE_KIND.EBEAM) {
      const ux = Math.cos(z.a), uy = Math.sin(z.a);
      const bx = z.x + (ux * z.len) / 2, by = z.y + (uy * z.len) / 2;
      if (z.t < z.arm) {
        const blink = 0.2 + 0.14 * Math.sin(t * 28);
        R.spr(S.px, bx, by, z.len / 6, 2.8, z.a, blink, 0xe11d48, 0, 0);
        R.spr(S.px, bx, by, z.len / 6, 0.45, z.a, 0.82, 0x9f1239, 0, 0);
      } else {
        const w = 0.9 + 0.12 * Math.sin(t * 42);
        R.spr(S.ebeam, bx, by, z.len / 64, 1.35 * w, z.a, 0.95, 0xffffff, 0, 0);
      }
    }
  }

  // 5. Compute Data Tokens (XP Gems)
  for (let i = 0; i < world.nG; i++) {
    const g = world.gm[i];
    if (g.x < x0 || g.x > x1 || g.y < y0 || g.y > y1) continue;
    const scG = 1 + 0.08 * Math.sin(t * 6 + i);
    const bob = Math.sin(t * 3 + i * 1.7) * 1.4;
    if (g.st === 1) {
      const ang = Math.atan2(p.y - g.y, p.x - g.x);
      R.spr(S.streak, g.x - Math.cos(ang) * 10, g.y - Math.sin(ang) * 10, 0.55, 0.9, ang, 0.75, 0xffffff, 0, 0);
    }
    R.spr(GEM_SPRS[g.tier], g.x, g.y + bob, scG, scG, 0, 1, 0xffffff, 0, 0);
  }

  // 6. Hardware Pickups
  for (let i = 0; i < world.nPK; i++) {
    const pk = world.pk[i];
    const bob = Math.sin(t * 3.4 + i) * 2.8;
    const scP = Math.min(1, pk.t * 5);
    const big = pk.kind === PICK_KIND.NUKE || pk.kind === PICK_KIND.CHEST;
    R.spr(S.shadow, pk.x, pk.y + 15, 0.7, 0.7, 0, 0.65, 0xffffff, 0, 0);
    if (big) {
      const rp = (t * 0.75) % 1;
      R.spr(S.ring_gold, pk.x, pk.y + bob, 0.32 + rp * 0.38, 0.32 + rp * 0.38, 0, (1 - rp) * 0.8, 0xffffff, 0, 0);
    }
    R.spr(PICK_SPRS[pk.kind], pk.x, pk.y + bob, scP, scP, pk.kind === PICK_KIND.CHEST ? Math.sin(t * 5) * 0.05 : 0, 1, 0xffffff, 0, 0);
  }

  // 7. Hardware Anomaly Enemies & 5 Sector Bosses
  if (!lite) {
    for (let i = 0; i < world.nE; i++) {
      const e = world.en[i];
      if (!e.dead && e.x >= x0 && e.x <= x1 && e.y >= y0 && e.y <= y1) {
        R.spr(S.shadow, e.x, e.y + e.r * 1.05, e.r / 19, e.r / 19, 0, 0.72, 0xffffff, 0, 0);
      }
    }
  }

  for (let i = 0; i < world.nE; i++) {
    const e = world.en[i];
    if (e.dead) continue;
    if (e.tele > 0) {
      const ang = Math.atan2(e.ay, e.ax);
      const blink = 0.5 + 0.5 * Math.sin(t * 28);
      R.spr(S.px, e.x + (e.ax * e.tele) / 2, e.y + (e.ay * e.tele) / 2, e.tele / 6, (e.r * 1.8) / 6, ang, 0.16 + 0.08 * blink, 0xe11d48, 0, 0);
      R.spr(S.px, e.x + (e.ax * e.tele) / 2, e.y + (e.ay * e.tele) / 2, e.tele / 6, 0.4, ang, 0.85, 0x9f1239, 0, 0);
    }
    if (e.x < x0 || e.x > x1 || e.y < y0 || e.y > y1) continue;

    const wob = Math.sin(t * 7 + e.ph) * 0.045;
    let mul = e.elite ? 1.58 : 1;
    if (e.swell > 0) mul *= 1 + e.swell * 0.6;
    let sx = mul * (1 + wob);
    let sy = mul * (1 - wob);
    let rot = 0;
    if (e.flash > 0) {
      sx *= 1.08;
      sy *= 0.92;
    }
    const alpha = Math.min(1, (world.t - e.born) * 4 + 0.2);

    if (e.elite) {
      R.spr(S.ring_gold, e.x, e.y, e.r / 34, e.r / 34, t * 1.1, 0.75, 0xffffff, 0, 0);
    }

    if (e.ai === AI_TYPE.DART) {
      rot = Math.atan2(p.y - e.y, p.x - e.x) + Math.sin(t * 5 + e.ph) * 0.2;
    } else {
      sx *= e.face;
      if (e.ai === AI_TYPE.DASH && e.st === 1) {
        rot = Math.sin(t * 60) * 0.12;
        sx *= 1.1;
        sy *= 1.1;
      }
    }

    if (e.boss) {
      const sw = e.swell > 0 ? e.swell * 0.25 : 0;
      R.spr(S.ring_warn, e.x, e.y, e.r / 38, e.r / 38, -t * 0.6, 0.65, 0xffffff, 0, 0);
      sx = (1 + wob * 0.5) * (1 + sw) * e.face;
      sy = (1 - wob * 0.5) * (1 + sw);
    }

    R.spr(
      ENEMY_SPRS[e.type],
      e.x, e.y + (e.ai === AI_TYPE.GHOST ? Math.sin(t * 4 + e.ph) * 2 : 0),
      sx, sy, rot, alpha, 0xffffff, 0, e.flash > 0 ? Math.min(0.65, e.flash * 6) : 0,
    );

    if (e.elite) {
      R.spr(S.crown, e.x, e.y - e.r - 8 + Math.sin(t * 4) * 1.2, 1.2, 1.2, 0, 1, 0xffffff, 0, 0);
      const hpR = e.hp / e.maxHp;
      R.spr(S.px, e.x, e.y + e.r + 9, 5.2, 0.7, 0, 0.85, 0x0f172a, 0, 0);
      R.spr(S.px, e.x - 15 * (1 - hpR), e.y + e.r + 9, 5 * hpR, 0.45, 0, 1, 0xf59e0b, 0, 0);
    }
  }

  // 8. Orbitals, Autonomous Drone Wingmen & EUV Lithography Scanner Beams
  for (let i = 0; i < p.weapons.length && !world.over; i++) {
    const w = p.weapons[i];
    const wid = WEAPONS[w.k].id;
    const cfg = w.evo ? WEAPONS[w.k].evo : WEAPONS[w.k].lv[w.lv - 1];

    if (wid === "cache" && w.vis > 0.03) {
      const n1 = cfg.n;
      // Draw geometric bus ring connecting inner cache dies
      for (let n = 0; n < n1; n++) {
        const ax = w.pos[n * 2], ay = w.pos[n * 2 + 1];
        const bx = w.pos[((n + 1) % n1) * 2], by = w.pos[((n + 1) % n1) * 2 + 1];
        const dx = bx - ax, dy = by - ay;
        const len = Math.sqrt(dx * dx + dy * dy);
        if (len > 2) {
          R.spr(S.p_bus_c, (ax + bx) * 0.5, (ay + by) * 0.5, len / 64, 0.55, Math.atan2(dy, dx), 0.55 * w.vis, 0xffffff, 0, 0);
        }
      }
      for (let n = 0; n < w.np; n++) {
        const ox = w.pos[n * 2];
        const oy = w.pos[n * 2 + 1];
        const scM = (w.evo ? 1.25 : 1.08) * w.vis;
        R.spr(S.p_die, ox, oy, scM, scM, t * 2.5 + n, w.vis, 0xffffff, 0, 0);
      }
    } else if (wid === "drone" && w.evo) {
      for (let n = 0; n < w.np; n++) {
        const ox = w.pos[n * 2];
        const oy = w.pos[n * 2 + 1];
        const scW = cfg.size * Math.sqrt(p.st.area) * 0.85;
        const ang = Math.atan2(oy - p.y, ox - p.x) + Math.PI / 2;
        R.spr(S.streak, ox - Math.cos(ang) * 16, oy - Math.sin(ang) * 16, 0.8 * scW, 1.3 * scW, ang, 0.75, 0xffffff, 0, 0);
        R.spr(S.p_drone, ox, oy, scW, scW, ang, 1, 0xffffff, 0, 0);
      }
    } else if (wid === "litho" && w.bt > 0) {
      const len = cfg.len * p.st.area;
      const widPx = cfg.wid * Math.sqrt(p.st.area);
      const fade = w.evo ? 1 : Math.min(1, w.bt * 6);
      for (let n = 0; n < cfg.n; n++) {
        const a = w.ang + (n / cfg.n) * TAU;
        const ux = Math.cos(a), uy = Math.sin(a);
        const cx = p.x + ux * (len / 2 + 8), cy = p.y + uy * (len / 2 + 8);
        R.spr(S.p_litho, cx, cy, len / 64, widPx / 22, a, 0.9 * fade, 0xffffff, 0, 0);
        R.spr(S.via, p.x + ux * (len + 8), p.y + uy * (len + 8), 0.9, 0.9, 0, 0.9 * fade, 0xffffff, 0, 0);
      }
    }
  }

  // 9. Hexagonal Cache-Flush Pulse Rings
  for (let i = 0; i < world.nZ; i++) {
    const z = world.zn[i];
    if (z.kind !== ZONE_KIND.PULSE) continue;
    const k = z.r / z.rMax;
    R.spr(S.ring2, z.x, z.y, z.r / 54, z.r / 54, 0, (1 - k * 0.72) * 0.9, 0xffffff, 0, 0);
  }

  // 10. Player Agent (Crisp ground shadow, zero underwater wave warp!)
  if (!world.over) {
    const sprId = S["h_" + view.hero + (view.state || "")] ?? S["h_" + view.hero];
    let bob = Math.sin(t * (p.moving ? 11 : 5)) * (p.moving ? 2.2 : 1.2);
    const tilt = Math.max(-0.22, Math.min(0.22, (p.my || 0) * 0.2)) * p.face;
    const br = 1 + Math.sin(t * 5.5) * 0.022;
    let sx = br, sy = 1 / br, jx = 0;
    if (view.joy > 0) {
      bob -= Math.abs(Math.sin(t * 9)) * 6 * view.joy;
      sx *= 1 + 0.09 * view.joy;
      sy *= 1 + 0.09 * view.joy;
    }
    if (view.hurt > 0) {
      jx = Math.sin(t * 70) * 3 * view.hurt;
      sx *= 1 + 0.1 * view.hurt;
      sy *= 1 - 0.08 * view.hurt;
    }
    R.spr(S.shadow, p.x, p.y + 22, 1.15, 0.95, 0, 0.8, 0xffffff, 0, 0);
    const invA = !view.dim && p.inv > 0 && !world.pending && view.state !== "_joy" ? (Math.sin(t * 38) > 0 ? 1 : 0.55) : 1;
    R.spr(
      sprId, p.x + jx, p.y + bob, p.face * sx, sy, tilt, invA, 0xffffff, 0,
      view.hurt > 0.4 ? 0.45 * view.hurt : 0,
    );
  }

  // 11. V2 Hardware Projectiles (HBM Bus Packets, WDM Prism Lances, ASIC Drones)
  for (let i = 0; i < world.nB; i++) {
    const b = world.bl[i];
    const ang = Math.atan2(b.vy, b.vx);
    if (b.kind === BULLET_KIND.HBM) {
      const scB = b.size;
      R.spr(S.streak, b.x - Math.cos(ang) * 14, b.y - Math.sin(ang) * 14, 0.65 * scB, 1.1 * scB, ang, 0.72, 0xffffff, 0, 0);
      R.spr(S.p_hbm, b.x, b.y, scB, scB, ang, 1, 0xffffff, 0, 0);
    } else if (b.kind === BULLET_KIND.PRISM_MAIN) {
      const scB = b.size;
      R.spr(S.streak, b.x - Math.cos(ang) * 15, b.y - Math.sin(ang) * 15, 0.75 * scB, 1.2 * scB, ang, 0.78, 0xffffff, 0, 0);
      R.spr(S.p_prism, b.x, b.y, scB, scB, ang, 1, 0xffffff, 0, 0);
    } else if (b.kind === BULLET_KIND.PRISM_SUB) {
      const scB = b.size;
      R.spr(WDM_SPRS[b.wl % 3], b.x, b.y, scB, scB, ang, 1, 0xffffff, 0, 0);
    } else {
      // BULLET_KIND.DRONE
      const dAng = b.rot || ang;
      R.spr(S.streak, b.x - Math.cos(dAng) * 15, b.y - Math.sin(dAng) * 15, 0.78 * b.size, 1.25 * b.size, dAng, 0.72, 0xffffff, 0, 0);
      R.spr(S.p_drone, b.x, b.y, b.size * 0.92, b.size * 0.92, dAng, 1, 0xffffff, 0, 0);
    }
  }

  // 12. Enemy Hazard Bullets (Square Sector Blocks & Outlined Orbs)
  for (let i = 0; i < world.nEB; i++) {
    const eb = world.eb[i];
    const scE = (eb.r / 6.2) * (1 + 0.08 * Math.sin(t * 16 + i));
    const rot = eb.sq ? t * 4 + i : 0;
    R.spr(eb.sq ? S.eb_sq : S.ebullet, eb.x, eb.y, scE, scE, rot, 1, 0xffffff, 0, 0);
  }

  fx.draw(R);

  // 13. Player Overhead HP Bar (Crisp Dark Frame on Light Floor)
  if (!world.over && !view.dim) {
    const hpR = Math.max(0, p.hp / p.maxHp);
    const hcol = hpR > 0.5 ? 0x10b981 : hpR > 0.25 ? 0xf59e0b : 0xe11d48;
    R.spr(S.px, p.x, p.y + 30, 6.8, 1.15, 0, 0.92, 0x0f172a, 0, 0);
    R.spr(S.px, p.x - 18 * (1 - hpR), p.y + 30, 6 * hpR, 0.7, 0, 1, hcol, 0, 0);
  }

  fx.drawNums(R);

  // 14. Cleanroom Screen-Space Post-FX & Virtual Joystick (NO underwater sunbeams or dark fog!)
  R.screen();
  const cw = view.cw, ch = view.ch;
  R.spr(S.vig, cw / 2, ch / 2, (cw / 128) * 1.04, (ch / 128) * 1.04, 0, 0.75, 0xffffff, 0, 0);
  if (view.dim) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, view.dim, 0xf1f5f9, 0, 0);

  const lowHp = world.over ? 0 : Math.max(0, 1 - p.hp / p.maxHp / 0.35);
  const redV = Math.max(fx.hurtV * 0.65, view.dim ? 0 : lowHp * (0.28 + 0.12 * Math.sin(t * 6)));
  if (redV > 0.01) R.spr(S.vigr, cw / 2, ch / 2, (cw / 128) * 1.04, (ch / 128) * 1.04, 0, redV, 0xffffff, 0, 0);
  if (fx.flash > 0.01) R.spr(S.px, cw / 2, ch / 2, cw / 5, ch / 5, 0, fx.flash, fx.flashCol, 0, 0);

  if (view.joyst && view.joyst.on) {
    const uz = view.uz || 1;
    R.spr(S.joy, view.joyst.bx, view.joyst.by, uz, uz, 0, 0.92, 0xffffff, 0, 0);
    R.spr(S.knob, view.joyst.kx, view.joyst.ky, uz, uz, 0, 0.96, 0xffffff, 0, 0);
  }
  R.end();
}
