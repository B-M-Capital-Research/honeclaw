import { describe, expect, it } from "bun:test";
import type { PublicPushListItem } from "./types";
import {
  ALL_PUBLIC_PUSHES,
  filterPublicPushes,
  publicPushCategories,
} from "./public-push-inbox";

const pushes: PublicPushListItem[] = [
  {
    push_id: "p3",
    job_id: "portfolio",
    title: "持仓晨报",
    summary: "latest",
    created_at: "2026-08-09T08:00:00Z",
  },
  {
    push_id: "p2",
    job_id: "news",
    title: "新闻监测",
    summary: "news",
    created_at: "2026-08-08T08:00:00Z",
  },
  {
    push_id: "p1",
    job_id: "portfolio",
    title: "旧名称不应覆盖最新名称",
    summary: "older",
    created_at: "2026-08-07T08:00:00Z",
  },
];

describe("public push inbox categories", () => {
  it("groups by stable job id and keeps the newest label", () => {
    expect(publicPushCategories(pushes)).toEqual([
      { jobId: "portfolio", title: "持仓晨报", count: 2 },
      { jobId: "news", title: "新闻监测", count: 1 },
    ]);
  });

  it("filters without changing newest-first order", () => {
    expect(filterPublicPushes(pushes, "portfolio").map((item) => item.push_id)).toEqual([
      "p3",
      "p1",
    ]);
    expect(filterPublicPushes(pushes, ALL_PUBLIC_PUSHES)).toBe(pushes);
  });
});
