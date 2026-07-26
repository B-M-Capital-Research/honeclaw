import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const publicPlan = readFileSync(
  new URL("./public-plan.tsx", import.meta.url),
  "utf8",
);

describe("public plan purchase contract", () => {
  it("opens the canonical Whop membership for English buyers", () => {
    expect(publicPlan).toContain(
      "https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/",
    );
    expect(publicPlan).not.toContain("/vip-copy-18/");
  });
});
