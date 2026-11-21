import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

import { setLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";
// @ts-expect-error standalone public ES module without .d.ts
import { ENEMIES, ENEMY_IDX, HEROES, PASSIVES, RULES, TALENTS, TALENT_BRANCHES, WEAPONS, branchPoints, getDailyChallenge, getTitle } from "../../public/hone-adventure/data.js";
// @ts-expect-error standalone public ES module without .d.ts
import { GameWorld, PICK_KIND, ZONE_KIND } from "../../public/hone-adventure/world.js";

const chatSource = readFileSync(new URL("../pages/chat.tsx", import.meta.url), "utf8");

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
});

describe("HONE Adventure game engine & upgrade paths", () => {
  it("defines complete redesigned HONE rosters, 8x8 weapon evolutions, 3-branch talents, and 13 enemies", () => {
    expect(HEROES.map((h: { id: string }) => h.id)).toEqual(["hone", "rubin", "photon", "quant"]);
    expect(WEAPONS.length).toBe(8);
    expect(PASSIVES.length).toBe(9);
    expect(TALENT_BRANCHES.length).toBe(3);
    expect(TALENTS.length).toBe(9);
    expect(RULES.length).toBe(7);
    expect(ENEMIES.length).toBe(13);
    expect(getTitle(0).name).toBe("实习研究员");
    expect(getTitle(720).name).toBe("HONE 磨刃宗师");
  });

  it("activates 3-branch 6-point talent milestones and instant Lv.2 economy headstart", () => {
    const talents = { might: 4, crit: 2, hp: 3, speed: 3, pickup: 4, core: 2 };
    expect(branchPoints(talents, "offense")).toBe(6);
    expect(branchPoints(talents, "defense")).toBe(6);
    expect(branchPoints(talents, "economy")).toBe(6);

    const world = new GameWorld({ seed: 42, char: "hone", talents, manualLevel: true });
    expect(world.p.lvl).toBe(2);
    expect(world.pendLv).toBe(1);
    expect(world.p.st.armor).toBe(2);
    // Base might (1 + 4*0.07) * 1.08 offense milestone
    expect(world.p.st.might).toBeCloseTo(1.28 * 1.08, 5);
  });

  it("evolves weapons at Lv.5 with partner passive (or Lv.3 under fast_evo rule) and supports card locking across rerolls", () => {
    const world = new GameWorld({ seed: 777, char: "hone", talents: { reroll: 2 }, rule: "fast_evo", manualLevel: true });
    const blade = world.hasWeapon("blade")!;
    blade.lv = 3;
    expect(world.canEvolve(blade)).toBe(false);

    world.apply({ kind: "p", id: "haste", lv: 1, isNew: true });
    expect(world.canEvolve(blade)).toBe(true);

    world.pendLv = 1;
    world.openLevel();
    expect(world.pending.choices.some((c: { kind: string; id: string }) => c.kind === "evo" && c.id === "blade")).toBe(true);

    // Lock second choice and reroll — locked card must stay at slot 0
    const targetToLock = world.pending.choices[1];
    expect(world.toggleLockChoice(1)).toBe(true);
    expect(world.reroll()).toBe(true);
    expect(world.pending.choices[0].id).toBe(targetToLock.id);
    expect(world.pending.choices[0].locked).toBe(true);

    // Pick the evolution card and confirm super-weapon activation
    const evoIdx = world.pending.choices.findIndex((c: { kind: string; id: string }) => c.kind === "evo" && c.id === "blade");
    expect(evoIdx).toBeGreaterThan(-1);
    world.choose(evoIdx);
    expect(blade.evo).toBe(true);
    expect(world.evos).toContain("blade");
  });

  it("handles bubble_mob splitting, margin_bomb blast zones, boss vault drops, and circuit-breaker revive", () => {
    const world = new GameWorld({ seed: 99, char: "rubin", talents: { revive: 1 }, manualLevel: true });
    world.nE = 0;

    const splitter = world.spawnEnemy(ENEMY_IDX.bubble_mob, 40, 0, 0)!;
    world.kill(splitter, 0);
    world.compact();
    expect(world.nE).toBe(3);
    expect(world.en[0].type).toBe(ENEMY_IDX.mini);

    const bomb = world.spawnEnemy(ENEMY_IDX.margin_bomb, -40, 0, 0)!;
    world.kill(bomb, 0);
    expect(world.nZ).toBeGreaterThan(0);
    expect(world.zn[world.nZ - 1].kind).toBe(ZONE_KIND.BLAST);

    const boss = world.spawnEnemy(ENEMY_IDX.boss4, 0, -100, 0)!;
    world.boss = boss;
    world.kill(boss, 1);
    world.compact();
    world.buildGrid();
    expect(world.bosses).toBe(1);
    expect(world.pk.slice(0, world.nPK).some((p: { kind: number; n: number }) => p.kind === PICK_KIND.CHEST && p.n === 3)).toBe(true);

    // Fatal damage triggers revive ("熔断保护") instead of game over
    world.p.inv = 0;
    world.hurt(9999, ENEMY_IDX.boss4);
    expect(world.over).toBe(false);
    expect(world.revives).toBe(0);
    expect(world.p.hp).toBe(Math.round(world.p.maxHp * 0.5));

    // Second fatal hit ends the run
    world.p.inv = 0;
    world.hurt(9999, ENEMY_IDX.boss4);
    expect(world.over).toBe(true);
  });

  it("generates deterministic daily challenges per date", () => {
    const d1 = getDailyChallenge("2026-11-21");
    const d2 = getDailyChallenge("2026-11-21");
    const d3 = getDailyChallenge("2026-11-22");
    expect(d1).toEqual(d2);
    expect(d1.seed).not.toBe(d3.seed);
  });
});
