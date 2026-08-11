import { describe, expect, it } from "bun:test";

const component = await Bun.file(new URL("./community-forum.tsx", import.meta.url)).text();
const css = await Bun.file(new URL("./community-forum.css", import.meta.url)).text();

describe("community forum contract", () => {
  it("supports the member discussion loop", () => {
    for (const label of ["发起讨论", "发布讨论", "评论", "赞", "举报", "删除"]) {
      expect(component).toContain(label);
    }
  });

  it("keeps forum material outside official research", () => {
    expect(component).toContain("不会自动进入投资助手、评级或每日产品");
    expect(component).toContain("等待管理员核验与采纳");
    expect(component).toContain("未经 HONE 核验");
  });

  it("limits attachments and exposes moderation", () => {
    expect(component).toContain("最多 10 MB");
    expect(component).toContain("管理员隐藏");
    expect(component).toContain("pending_review");
  });

  it("has mobile and dark layouts", () => {
    expect(css).toContain("@media(max-width:768px)");
    expect(css).toContain("[data-theme=dark]");
  });
});
