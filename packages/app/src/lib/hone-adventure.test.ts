import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

import { setLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";
// @ts-expect-error standalone public ES module without .d.ts
import { DIFFICULTIES, ENEMIES, ENEMY_IDX, HEROES, HONECLAW_GITHUB_URL, PASSIVES, RELICS, RULES, STAGES, STAGE_SEC, SYNERGIES, TALENTS, TALENT_BRANCHES, TITLES, WEAPONS, branchPoints, getDailyChallenge, getHeroMasteryBonuses, getPromotionBonuses, getTitle } from "../../public/hone-adventure/data.js?v=3.0.0";
// @ts-expect-error standalone public ES module without .d.ts
import { BULLET_KIND, FX_EVT, GameWorld, PICK_KIND, ZONE_KIND } from "../../public/hone-adventure/world.js?v=3.0.0";
// @ts-expect-error standalone public ES module without .d.ts
import { createFxSystem } from "../../public/hone-adventure/fx.js?v=3.0.0";

const chatSource = readFileSync(new URL("../pages/chat.tsx", import.meta.url), "utf8");
const appSource = readFileSync(new URL("../../public/hone-adventure/app.js", import.meta.url), "utf8");
const worldSource = readFileSync(new URL("../../public/hone-adventure/world.js", import.meta.url), "utf8");
const audioSource = readFileSync(new URL("../../public/hone-adventure/audio.js", import.meta.url), "utf8");
const headersSource = readFileSync(new URL("../../public/_headers", import.meta.url), "utf8");
const viteSource = readFileSync(new URL("../../vite.config.ts", import.meta.url), "utf8");

describe("HONE Adventure chat entry & localization", () => {
  it("places HONE Adventure right beside 3D Data Center in /chat composer actions", () => {
    const dcIdx = chatSource.indexOf('id: "data-center"');
    const haIdx = chatSource.indexOf('id: "hone-adventure"');
    const infIdx = chatSource.indexOf('id: "influencer-digest"');
    expect(dcIdx).toBeGreaterThan(-1);
    expect(haIdx).toBeGreaterThan(dcIdx);
    expect(infIdx).toBeGreaterThan(haIdx);
    expect(chatSource).toContain('"/hone-adventure/"');
    expect(chatSource).toContain("CONTENT.chat_page.workspace.hone_adventure_tip");
    expect(chatSource).toContain("CONTENT.chat_page.composer.hone_adventure_hint");
  });

  it("provides Chinese and English copy for the HONE Adventure shortcut", () => {
    setLocale("zh");
    expect(CONTENT.chat_page.workspace.hone_adventure_tip).toBe("HONE 历险记");
    expect(CONTENT.chat_page.composer.hone_adventure_hint).toContain("HONE");
    setLocale("en");
    expect(CONTENT.chat_page.workspace.hone_adventure_tip).toBe("HONE Adventure");
    expect(CONTENT.chat_page.composer.hone_adventure_hint).toContain("roguelite");
    setLocale("zh");
  });

  it("exposes top-left homepage return link (/) and honeclaw GitHub Star CTA", () => {
    expect(HONECLAW_GITHUB_URL).toBe("https://github.com/B-M-Capital-Research/honeclaw");
    expect(appSource).toContain('a.href = "/"');
    expect(appSource).toContain("回到首页");
    expect(appSource).toContain("HONECLAW_GITHUB_URL");
    expect(appSource).toContain("gh-star-card");
  });

  it("enforces zero-stale-cache updates via _headers, Vite build content-hash stamping, and idle lobby version check", () => {
    expect(headersSource).toContain("/hone-adventure/*");
    expect(headersSource).toContain("Cache-Control: public, max-age=0, must-revalidate");
    expect(viteSource).toContain("closeBundle()");
    expect(viteSource).toContain('createHash("sha256")');
    expect(appSource).toContain("checkLiveBundleUpdate");
  });
});

describe("HONE Adventure V3 deep progression, clean de-templating, stage nodes & active skills", () => {
  it("removes base-template fingerprints (no gift skin button, no 12s scripted nuke ring, new 138 BPM 8-bar soundtrack)", () => {
    expect(appSource).not.toContain("领曜金皮");
    expect(worldSource).not.toContain("tutDone");
    expect(audioSource).toContain("const BPM = 138");
    expect(audioSource).toContain("setBgmStage");
  });

  it("defines 6 AI stages, 3 difficulty tiers, 10x10 weapons, 5 agents, 6 synergies, 8 relics, 16 infra nodes & 12 promotion ranks", () => {
    expect(STAGES).toHaveLength(6);
    expect(DIFFICULTIES).toHaveLength(3);
    expect(WEAPONS).toHaveLength(10);
    expect(PASSIVES).toHaveLength(10);
    expect(HEROES).toHaveLength(5);
    expect(SYNERGIES).toHaveLength(6);
    expect(RELICS).toHaveLength(8);
    expect(TALENT_BRANCHES).toHaveLength(4);
    expect(TALENTS).toHaveLength(16);
    expect(TITLES).toHaveLength(12);
    expect(RULES).toHaveLength(8);
    expect(ENEMIES).toHaveLength(14);
    for (const w of WEAPONS) {
      expect(PASSIVES.some((p: { id: string }) => p.id === w.partner)).toBe(true);
    }
  });

  it("supports route locking, tactical sector jump overdrive, and interactive cleanroom sector nodes", () => {
    const lockedWorld = new GameWorld({ seed: 42, stage: "optics", routeMode: "lock" });
    lockedWorld.nE = 0;
    lockedWorld.update(STAGE_SEC + 1);
    // In lock mode, stays in optics until manually jumped, with jumpReady charged
    expect(STAGES[lockedWorld.stageIdx].id).toBe("optics");
    expect(lockedWorld.jumpReady).toBe(true);

    expect(lockedWorld.jumpStage("gpu")).toBe(true);
    expect(STAGES[lockedWorld.stageIdx].id).toBe("gpu");
    expect(lockedWorld.jumpBuffT).toBeGreaterThan(8);

    // Capturing a Cleanroom Sector Node grants stage-specific rewards
    const node = lockedWorld.spawnSectorNode()!;
    expect(node.kind).toBe(ZONE_KIND.NODE);
    lockedWorld.p.x = node.x;
    lockedWorld.p.y = node.y;
    for (let i = 0; i < 45; i++) lockedWorld.updZones(0.05);
    expect(lockedWorld.nodesCaptured).toBe(1);
  });

  it("executes all 5 hero active tactical skills, mastery bonuses, and home-sector affinities", () => {
    expect(getHeroMasteryBonuses(1).freeWeapon).toBe(false);
    expect(getHeroMasteryBonuses(2).freeWeapon).toBe(true);
    expect(getHeroMasteryBonuses(5).startCores).toBe(15);

    for (const hero of HEROES) {
      const w = new GameWorld({
        seed: 77,
        char: hero.id,
        masteryLv: 5,
        stage: hero.homeStages[0],
        manualLevel: true,
      });
      expect(w.isHomeStage()).toBe(true);
      expect(w.castSkill()).toBe(true);
      expect(w.skillCd).toBeGreaterThan(0);
      expect(w.castSkill()).toBe(false);
    }
  });

  it("supports NVLink laser mesh, Liquid-N2 cross frost mortar, dual-weapon synergies, super overclock ★+1~+5, and mythic relics", () => {
    const world = new GameWorld({ seed: 101, char: "astra", weapon: "nvlink", manualLevel: true });
    world.addWeapon("bus");
    world.addWeapon("liquid_n2");
    world.addWeapon("prism");

    // Both syn_nvlink_bus and syn_n2_prism become available in buildPool
    const pool = world.buildPool();
    expect(pool.some((c: { kind: string; id: string }) => c.kind === "syn" && c.id === "syn_nvlink_bus")).toBe(true);
    expect(pool.some((c: { kind: string; id: string }) => c.kind === "syn" && c.id === "syn_n2_prism")).toBe(true);

    world.apply({ kind: "syn", id: "syn_n2_prism" });
    expect(world.hasSynergy("syn_n2_prism")).toBe(true);

    // Evolve nvlink and verify ★+1 ~ ★+5 overclock progression
    const nv = world.hasWeapon("nvlink")!;
    nv.lv = 5;
    world.apply({ kind: "p", id: "ups" });
    expect(world.canEvolve(nv)).toBe(true);
    world.apply({ kind: "evo", id: "nvlink" });
    expect(nv.evo).toBe(true);
    expect(world.buildPool().some((c: { kind: string; id: string }) => c.kind === "oc" && c.id === "nvlink")).toBe(true);
    world.apply({ kind: "oc", id: "nvlink" });
    expect(nv.oc).toBe(1);

    // Mythic Relics: slot expansion, shield absorption & 16% execution threshold
    world.apply({ kind: "relic", id: "r_slots" });
    expect(world.wslots()).toBe(6);
    expect(world.pslots()).toBe(7); // Astra 5 + 1 extraPSlot + 1 r_slots

    world.apply({ kind: "relic", id: "r_shield" });
    expect(world.p.shields).toBeGreaterThan(0);
    const hpBefore = world.p.hp;
    world.p.inv = 0;
    world.hurt(50, 0);
    expect(world.p.hp).toBe(hpBefore); // absorbed by energy shield!

    world.apply({ kind: "relic", id: "r_execute" });
    const tank = world.spawnEnemy(ENEMY_IDX.thermal, 30, 0, 1)!;
    tank.hp = Math.floor(tank.maxHp * 0.15);
    world.hit(tank, 1, 0, 0, 1, 0);
    expect(tank.dead).toBe(true);

    // Liquid-N2 canister shatter spawns FROST_RAIL + syn_n2_prism needles
    const can = world.addBullet(BULLET_KIND.N2_CANISTER, 9, 50, 50, 10, 0, 10, 30, 0, 0.01)!;
    can.rays = 4;
    can.rayLen = 110;
    world.updBullets(0.05);
    expect(world.zn.slice(0, world.nZ).some((z: { kind: number }) => z.kind === ZONE_KIND.FROST_RAIL)).toBe(true);
    expect(world.bl.slice(0, world.nB).some((b: { kind: number }) => b.kind === BULLET_KIND.PRISM_SUB)).toBe(true);
  });

  it("activates 4-pillar 6pt & 10pt infrastructure milestones and 12-tier promotion bonuses", () => {
    const world = new GameWorld({
      seed: 12,
      char: "hone",
      talents: {
        might: 5, haste: 4, area: 2, // 11 pts interconnect -> +10% might & +1 pierce
        hp: 5, regen: 4, revive: 1, // 10 pts thermal -> +2 armor & +1 starting shield
      },
      bestTime: 960, // L12 Chief Compute Officer!
      manualLevel: true,
    });
    const promo = getPromotionBonuses(960);
    expect(promo.startShield).toBe(1);
    expect(getTitle(960).rank).toBe(12);
    expect(world.p.st.pierceBonus).toBe(1);
    expect(world.p.shields).toBeGreaterThanOrEqual(2);
  });

  it("eliminates late-game continuous screen shake on mass kills/AoEs and respects shake toggle", () => {
    const fx = createFxSystem({
      camX: () => 0,
      camY: () => 0,
      viewW: () => 1280,
      viewH: () => 720,
      combo: () => 50,
      hitstop: () => {},
      slowmo: () => {},
      vibrate: () => {},
      announce: () => {},
      hurt: () => {},
    });
    for (let i = 0; i < 120; i++) fx.on(FX_EVT.KILL, 10, 10, ENEMY_IDX.packet, 0, 1);
    for (let i = 0; i < 30; i++) fx.on(FX_EVT.MATRIX, 20, 20, 96, 2);
    for (let i = 0; i < 30; i++) fx.on(FX_EVT.BOOM, 20, 20, 90, 0);
    fx.update(1 / 60, 1 / 60);
    expect(fx.trauma).toBe(0);
    expect(fx.shX).toBe(0);
    expect(fx.shY).toBe(0);

    fx.shakeOn = false;
    fx.on(FX_EVT.NUKE, 0, 0);
    fx.update(1 / 60, 1 / 60);
    expect(fx.trauma).toBe(0);
    expect(fx.shX).toBe(0);
    expect(fx.shY).toBe(0);
  });

  it("generates deterministic daily challenges with sector stage per date", () => {
    const d1 = getDailyChallenge("2026-11-24");
    const d2 = getDailyChallenge("2026-11-24");
    const d3 = getDailyChallenge("2026-11-25");
    expect(d1).toEqual(d2);
    expect(STAGES.some((s: { id: string }) => s.id === d1.stage)).toBe(true);
    expect(d1.seed).not.toBe(d3.seed);
  });
});
