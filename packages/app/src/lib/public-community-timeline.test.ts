import { describe, expect, test } from "bun:test";
import { mergeCommunityTimeline } from "./public-community-timeline";
import type { PublicCommunityContent, PublicCommunityPage } from "./types";

const item = (id: number, body = String(id)) => ({
  content_id: id,
  body_text: body,
  resources: [],
}) as unknown as PublicCommunityContent;

const page = (items: PublicCommunityContent[], next_before: number | null) => ({
  items, next_before,
}) as PublicCommunityPage;

describe("community timeline refresh", () => {
  test("refreshes current records while retaining contiguous older pages and their cursor", () => {
    const result = mergeCommunityTimeline(
      [item(9), item(80), item(7), item(60)], 60,
      page([item(100), item(9, "updated attachment"), item(80)], 80), false,
    );
    expect(result.items.map((entry) => entry.content_id)).toEqual([100, 9, 80, 7, 60]);
    expect(result.items[1]?.body_text).toBe("updated attachment");
    expect(result.nextBefore).toBe(60);
  });

  test("resets pagination when a refresh is newer than every loaded item instead of hiding a gap", () => {
    const result = mergeCommunityTimeline(
      [item(5), item(40)], 40,
      page([item(9), item(80)], 80), false,
    );
    expect(result.items.map((entry) => entry.content_id)).toEqual([9, 80]);
    expect(result.nextBefore).toBe(80);
  });

  test("does not duplicate overlapping records returned by pagination", () => {
    const result = mergeCommunityTimeline(
      [item(9), item(80)], 80,
      page([item(80), item(7)], null), true,
    );
    expect(result.items.map((entry) => entry.content_id)).toEqual([9, 80, 7]);
    expect(result.nextBefore).toBeNull();
  });

  test("replaces an empty archive and its old cursor", () => {
    expect(mergeCommunityTimeline([item(9)], 9, page([], null), false)).toEqual({
      items: [], nextBefore: null,
    });
  });

  test("does not retain removed tail records when the canonical refresh is the whole archive", () => {
    const result = mergeCommunityTimeline(
      [item(9), item(80), item(7)], 7,
      page([item(9), item(80)], null), false,
    );
    expect(result.items.map((entry) => entry.content_id)).toEqual([9, 80]);
    expect(result.nextBefore).toBeNull();
  });

  test("preserves server order and reaches every record across page-sized gaps and non-monotonic IDs", () => {
    const canonical = Array.from({ length: 75 }, (_, index) => ({
      ...item((index * 37) % 997 + 1),
      published_at_raw: "2026-09-05 12:00",
    }));
    for (const newCount of [0, 1, 19, 20, 21]) {
      const current = canonical.slice(newCount, newCount + 40);
      let result = mergeCommunityTimeline(
        current, current.at(-1)!.content_id,
        page(canonical.slice(0, 20), canonical[19]!.content_id), false,
      );
      while (result.nextBefore != null) {
        const start = canonical.findIndex((entry) => entry.content_id === result.nextBefore) + 1;
        const incoming = canonical.slice(start, start + 20);
        const cursor = start + 20 < canonical.length ? incoming.at(-1)!.content_id : null;
        result = mergeCommunityTimeline(result.items, result.nextBefore, page(incoming, cursor), true);
      }
      expect(result.items.map((entry) => entry.content_id)).toEqual(canonical.map((entry) => entry.content_id));
    }
  });
});
