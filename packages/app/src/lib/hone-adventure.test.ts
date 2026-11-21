import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

import { setLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";
// @ts-expect-error standalone public ES module without .d.ts
import { ENEMIES, ENEMY_IDX, HEROES, HONECLAW_GITHUB_URL, PASSIVES, RULES, STAGES, STAGE_SEC, TALENTS, TALENT_BRANCHES, TITLES, WEAPONS, branchPoints, getDailyChallenge, getPromotionBonuses, getTitle } from "../../public/hone-adventure/data.js";
// @ts-expect-error standalone public ES module without .d.ts
import { BULLET_KIND, GameWorld, PICK_KIND, ZONE_KIND } from "../../public/hone-adventure/world.js";

const chatSource = readFileSync(new URL("../pages/chat.tsx", import.meta.url), "utf8");
const appSource = readFileSync(new URL("../../public/hone-adventure/app.js", import.meta.url), "utf8");

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
});

describe("HONE Adventure V2 AI Data Center stages, weapons, bosses & promotion", () => {
  it("defines 6 AI hardware sector stages, 8x8 hardware evolutions, 4x3 infrastructure matrix, 10 promotion tiers, and 14 enemies/bosses", () => {
    expect(STAGES.map((s: { id: string }) => s.id)).toEqual([
      "storage",
      "optics",
      "cpu",
      "gpu",
      "packaging",
      "hyperscale",
    ]);
    expect(HEROES.map((h: { id: string }) => h.id)).toEqual(["hone", "rubin", "photon", "quant"]);
    expect(WEAPONS.map((w: { id: string }) => w.id)).toEqual([
      "hbm",
      "prism",
      "tensor",
      "cache",
      "bus",
      "litho",
      "drone",
      "cryo",
    ]);
    expect(PASSIVES.length).toBe(9);
    expect(TALENT_BRANCHES.length).toBe(4);
    expect(TALENTS.length).toBe(12);
    expect(RULES.length).toBe(7);
    expect(TITLES.length).toBe(10);
    expect(ENEMIES.length).toBe(14);
    expect(getTitle(0).name).toContain("L1");
    expect(getTitle(780).name).toContain("L10 · HONE 首席算力官");
  });

  it("switches stages cleanly and advances through the sector pipeline over time", () => {
    const world = new GameWorld({ seed: 101, char: "hone", stage: "optics", manualLevel: true });
    expect(world.stage().id).toBe("optics");
    expect(world.nextBossTypeIndex()).toBe(ENEMY_IDX.boss2);

    expect(world.setStage("gpu")).toBe(true);
    expect(world.stage().id).toBe("gpu");
    expect(world.nextBossTypeIndex()).toBe(ENEMY_IDX.boss4);

    // Advance past STAGE_SEC (75s) from startStageIdx ("optics" = 1 -> step 1 = "cpu" = 2)
    world.t = STAGE_SEC - 0.01;
    world.p.inv = 5;
    world.update(0.05);
    expect(world.stage().id).toBe("cpu");
    expect(world.stagesVisited).toContain("cpu");
  });

  it("activates 4-pillar 6-point synergies and 10-tier permanent promotion bonuses", () => {
    const talents = { might: 4, haste: 2, crit: 4, proj: 2, hp: 4, regen: 2, pickup: 4, core: 2 };
    expect(branchPoints(talents, "interconnect")).toBe(6);
    expect(branchPoints(talents, "litho")).toBe(6);
    expect(branchPoints(talents, "thermal")).toBe(6);
    expect(branchPoints(talents, "cluster")).toBe(6);

    const promo = getPromotionBonuses(500);
    expect(promo.maxHp).toBeGreaterThan(0);
    expect(promo.reroll).toBe(1);

    const world = new GameWorld({ seed: 42, char: "hone", stage: "storage", talents, bestTime: 500, manualLevel: true });
    expect(world.p.lvl).toBe(2);
    expect(world.pendLv).toBe(1);
    expect(world.p.st.armor).toBe(2);
    expect(world.rerolls).toBe(2);
  });

  it("refracts silicon-photonics prism lance into WDM wavelength sub-beams on first impact", () => {
    const world = new GameWorld({ seed: 314, char: "photon", stage: "optics", manualLevel: true });
    world.nE = 0;
    const target = world.spawnEnemy(ENEMY_IDX.thermal, 60, 0, 0)!;
    target.hp = 500;
    world.compact();
    world.buildGrid();
    world.findNear();

    const prism = world.hasWeapon("prism")!;
    prism.t = 0;
    world.updWeapons(0.016);
    expect(world.nB).toBe(1);
    expect(world.bl[0].kind).toBe(BULLET_KIND.PRISM_MAIN);

    // Step projectile until it strikes target and splits into 3 WDM sub-beams
    for (let i = 0; i < 20 && world.bl[0]?.kind === BULLET_KIND.PRISM_MAIN; i++) {
      world.updBullets(0.016);
    }
    const subBeams = world.bl.slice(0, world.nB).filter((b: { kind: number }) => b.kind === BULLET_KIND.PRISM_SUB);
    expect(subBeams.length).toBe(3);
  });

  it("evolves hardware weapons with partner passive (or Lv.3 under tapeout rule) and preserves locked cards across rerolls", () => {
    const world = new GameWorld({ seed: 777, char: "hone", talents: { reroll: 2 }, rule: "tapeout", manualLevel: true });
    const hbm = world.hasWeapon("hbm")!;
    hbm.lv = 3;
    expect(world.canEvolve(hbm)).toBe(false);

    world.apply({ kind: "p", id: "clock", lv: 1, isNew: true });
    expect(world.canEvolve(hbm)).toBe(true);

    world.pendLv = 1;
    world.openLevel();
    expect(world.pending.choices.some((c: { kind: string; id: string }) => c.kind === "evo" && c.id === "hbm")).toBe(true);

    const targetToLock = world.pending.choices[1];
    expect(world.toggleLockChoice(1)).toBe(true);
    expect(world.reroll()).toBe(true);
    expect(world.pending.choices[0].id).toBe(targetToLock.id);
    expect(world.pending.choices[0].locked).toBe(true);

    const evoIdx = world.pending.choices.findIndex((c: { kind: string; id: string }) => c.kind === "evo" && c.id === "hbm");
    expect(evoIdx).toBeGreaterThan(-1);
    world.choose(evoIdx);
    expect(hbm.evo).toBe(true);
    expect(world.evos).toContain("hbm");
  });

  it("handles memory-leak cube splitting, short-circuit blast reticles, 5-boss vault drops, and UPS revive", () => {
    const world = new GameWorld({ seed: 99, char: "rubin", talents: { revive: 1 }, manualLevel: true });
    world.nE = 0;

    const splitter = world.spawnEnemy(ENEMY_IDX.leak, 40, 0, 0)!;
    world.kill(splitter, 0);
    world.compact();
    expect(world.nE).toBe(3);
    expect(world.en[0].type).toBe(ENEMY_IDX.frag);

    const bomb = world.spawnEnemy(ENEMY_IDX.short, -40, 0, 0)!;
    world.kill(bomb, 0);
    expect(world.nZ).toBeGreaterThan(0);
    expect(world.zn[world.nZ - 1].kind).toBe(ZONE_KIND.BLAST);

    const boss = world.spawnEnemy(ENEMY_IDX.boss5, 0, -100, 0)!;
    world.boss = boss;
    world.kill(boss, 1);
    world.compact();
    world.buildGrid();
    expect(world.bosses).toBe(1);
    expect(world.pk.slice(0, world.nPK).some((p: { kind: number; n: number }) => p.kind === PICK_KIND.CHEST && p.n === 3)).toBe(true);

    world.p.inv = 0;
    world.hurt(9999, ENEMY_IDX.boss5);
    expect(world.over).toBe(false);
    expect(world.revives).toBe(0);

    world.p.inv = 0;
    world.hurt(9999, ENEMY_IDX.boss5);
    expect(world.over).toBe(true);
  });

  it("generates deterministic daily challenges with sector stage per date", () => {
    const d1 = getDailyChallenge("2026-11-21");
    const d2 = getDailyChallenge("2026-11-21");
    const d3 = getDailyChallenge("2026-11-22");
    expect(d1).toEqual(d2);
    expect(STAGES.some((s: { id: string }) => s.id === d1.stage)).toBe(true);
    expect(d1.seed).not.toBe(d3.seed);
  });
});
