import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");

const page = read("./public-industry-map.tsx");
const css = read("./public-industry-map.css");
const modules = {
  shared: read("./public-industry-map/shared.tsx"),
  brief: read("./public-industry-map/brief.tsx"),
  changes: read("./public-industry-map/changes.tsx"),
  lens: read("./public-industry-map/company-lens.tsx"),
  watch: read("./public-industry-map/watch-sources.tsx"),
  dossier: read("./public-industry-map/dossier.tsx"),
  admin: read("./public-industry-map/admin-editors.tsx"),
};
const everything = [page, ...Object.values(modules)].join("\n");

describe("industry map reading order", () => {
  it("opens with the question, then what changed, then who it touches, then what to verify; the dossier folds last", () => {
    // 首屏是「带着问题看行业」：先当前重点、最近变化，再公司、关注点；方法论与估值卡折进底稿，
    // 来源垫底。顺序由 JSX 顺序决定（手机上单列，DOM 顺序就是阅读顺序），所以在源码上锁死。
    const order = [
      "<CompanyLensCard",
      "<IndustryBriefSection",
      "<RecentChanges",
      "<MembersTable",
      "<WatchList",
      "<ResearchDossier",
      "<SourcesList",
    ].map((marker) => page.indexOf(marker));
    expect(order.every((at) => at > -1)).toBe(true);
    expect([...order].sort((a, b) => a - b)).toEqual(order);
  });

  it("keeps the industry heading a bare name and moves the dates below it", () => {
    // e2e 用 toHaveText 全等匹配 `.industry-detail > h2`；日期一旦进了 h2 就会连带 3D 页的
    // 导航测试一起挂。日期单独成行，并且区分「内容截至」与「本行最近改动」两种时钟。
    const heading = page.slice(page.indexOf('class="industry-detail" id="industry-detail"'));
    const h2 = heading.slice(heading.indexOf("<h2>"), heading.indexOf("</h2>"));
    expect(h2).toContain("{industry().name}");
    expect(h2).not.toContain("内容截至");
    expect(h2).not.toContain("最近改动");
    expect(heading.indexOf('class="industry-detail-dates"')).toBeGreaterThan(heading.indexOf("</h2>"));
    expect(page).toContain("内容截至 {industry().content_as_of ?? contentAsOf(industry()) ?? \"—\"}");
    expect(page).toContain("本行最近改动");
    expect(page).toContain("本行自基线后未改动");
    // 页头的底稿日期改叫基线，并说明它不是新鲜度。
    expect(page).toContain("研究底稿基线：{data().generated_at}");
    expect(page).not.toContain("研究底稿更新：");
  });

  it("lets the URL own both the industry and the company lens", () => {
    // 刷新、分享、浏览器后退都要一致，所以公司视角不是本地 signal 而是 ?symbol=。
    expect(page).toContain("resolveIndustryMapLens(current(), searchParams.symbol)");
    // setSearchParams 是合并语义：切行业必须显式清 symbol，否则视角残留到别的行业。
    expect(page).toContain("setSearchParams({ industry: next, symbol: undefined }");
    // 不合法的 symbol 规范化掉，而不是留着一个指向不存在成员的 URL。
    expect(page).toContain("setSearchParams({ symbol: undefined }, { replace: true, scroll: false })");
    expect(modules.lens).toContain('aria-label="退出公司视角"');
    expect(modules.lens).toContain("aria-label={`查看 ${member.symbol}`}");
    // 找公司命中即进入视角（replace，不堆历史）。
    expect(page).toContain("viewMember(hit.industry.id, hit.member.symbol, true)");
  });

  it("folds the dossier with <details>, never with conditional rendering", () => {
    // 里面的编辑器草稿依赖组件实例存活；条件渲染会在展开/收起时丢掉正在改的内容。
    expect(modules.dossier).toContain('id="dossier"');
    expect(modules.dossier).toMatch(/<details[\s\S]{0,200}id="dossier"/);
    expect(modules.dossier).not.toMatch(/<Show when=\{props\.open\}/);
    expect(page).toContain("open={dossierOpen() || editMode()}");
    // 锚点进封闭的 details 浏览器不会自动展开，所以跳转先展开再滚。
    expect(page).toContain("if ((DOSSIER_IDS as readonly string[]).includes(id)) setDossierOpen(true);");
    for (const id of ["method", "valuation-card", "valuation-logic", "driver-chain"]) {
      expect(modules.dossier).toContain(`id="${id}"`);
    }
    // 方法论条搬进底稿后不能再带自己的 h2：详情区只允许一个直接子 h2。
    expect(modules.dossier).not.toContain("<h2>");
  });

  it("speaks to the reader in plain titles and dates every block", () => {
    for (const title of [
      "当前重点",
      "最近变化",
      "哪些外部变化会影响业绩",
      "相关公司与影响",
      "接下来重点看什么",
      "这类公司怎么估值",
      "什么情况下适用",
      "完整研究底稿",
      "研报与数据来源",
    ]) {
      expect(everything).toContain(title);
    }
    for (const old of ["<h3>上游信号</h3>", "<h3>估值执行卡</h3>", "<h3>核心关注点</h3>", "典型 State</span>"]) {
      expect(everything).not.toContain(old);
    }
    // 同一个日期标签走遍所有块：简报、变化、关注点、来源、变量表。
    expect(modules.shared).toContain("export function AsOfTag(");
    for (const name of ["brief", "changes", "watch", "dossier"]) {
      expect(modules[name as keyof typeof modules]).toContain("<AsOfTag");
    }
    // 每个锚点都真的存在于页面上。
    for (const id of ["brief", "changes", "members", "watch", "sources", "company-lens"]) {
      expect(everything).toContain(`id="${id}"`);
    }
  });

  it("hands every question to chat through the URL, with the context of the block it came from", () => {
    // 与研究台一致：/chat?q=…&send=1，到达即发送，没有 stash、没有第二次点击。
    expect(modules.changes).toContain("askSignalPrompt(");
    expect(modules.changes).toContain("askSourcePrompt(");
    expect(modules.watch).toContain("askWatchPrompt(");
    expect(modules.brief).toContain("askNextStepPrompt(");
    expect(everything).not.toContain("sessionStorage");
    expect(page).toContain("const onAsk = (href: string) => navigate(href);");
  });

  it("keeps the administrator's editors: signals, watch items and the brief itself", () => {
    // 「最近变化」是派生视图，编辑态退回逐条上游信号编辑器；关注点可整条改并带截至日；简报可写可清。
    expect(modules.changes).toContain("function SignalsEditor(");
    expect(modules.changes).toContain('kind: "set_upstream_latest"');
    expect(modules.watch).toContain('kind: "set_watch"');
    expect(modules.brief).toContain('kind: "set_brief"');
    expect(modules.brief).toContain('kind: "clear_brief"');
    expect(page).toContain("移除此行业");
    expect(page).toContain('placeholder="为什么改（必填，展示给其它管理员）"');
  });

  it("styles the new blocks with foundation tokens, in one stylesheet, single-column on phones", () => {
    expect(page).toMatch(
      /import "\.\/public-foundation\.css";\s*import "\.\/public-site\.css";\s*import "\.\/public-polish\.css";\s*import "\.\/public-industry-map\.css";/,
    );
    const added = css.slice(css.indexOf("阅读顺序重组"));
    expect(added.length).toBeGreaterThan(1000);
    expect(added).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
    expect(added).not.toContain("!important");
    for (const selector of [
      ".industry-brief {",
      ".industry-change {",
      ".industry-lens-card {",
      ".industry-dossier-nav {",
      ".industry-asof {",
      ".industry-related-tag {",
    ]) {
      expect(added).toContain(selector);
    }
    const mobile = added.slice(added.lastIndexOf("@media (max-width: 760px)"));
    expect(mobile).toContain(".industry-lens-card {");
    expect(mobile).toContain("grid-template-columns: minmax(0, 1fr);");
  });
});
