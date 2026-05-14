import { describe, expect, test } from "bun:test";

import {
  isRecoverableAssetLoadError,
  recoverFromAssetLoadError,
} from "./asset-recovery";

function memoryStorage() {
  const values = new Map<string, string>();
  return {
    getItem(key: string) {
      return values.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
  };
}

describe("asset recovery", () => {
  test("recognizes stale chunk and HTML-as-JS failures", () => {
    expect(
      isRecoverableAssetLoadError(
        "TypeError: 'text/html' is not a valid JavaScript MIME type.",
      ),
    ).toBe(true);
    expect(
      isRecoverableAssetLoadError(
        new TypeError("Failed to fetch dynamically imported module"),
      ),
    ).toBe(true);
    expect(isRecoverableAssetLoadError(new Error("normal app bug"))).toBe(false);
  });

  test("reloads once per page within the cooldown window", () => {
    const storage = memoryStorage();
    let reloads = 0;
    const reload = () => {
      reloads += 1;
    };

    expect(
      recoverFromAssetLoadError("text/html is not a valid JavaScript MIME type", {
        storage,
        reload,
        href: "https://hone-claw.com/chat",
        now: () => 1000,
        reloadDelayMs: 0,
      }),
    ).toBe(true);
    expect(
      recoverFromAssetLoadError("text/html is not a valid JavaScript MIME type", {
        storage,
        reload,
        href: "https://hone-claw.com/chat",
        now: () => 2000,
        reloadDelayMs: 0,
      }),
    ).toBe(false);
    expect(reloads).toBe(0);
  });
});
