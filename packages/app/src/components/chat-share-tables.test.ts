import { describe, expect, test } from "bun:test";

import {
  decideShareTableLayout,
  estimateTextEm,
  reflowShareTables,
  shareTableCapacityEm,
  transposeTable,
} from "./chat-share-tables";

const CAPACITY = shareTableCapacityEm(15);

const EVENT_MATRIX = {
  header: ["公司", "事件", "时间", "类型", "影响", "期限", "来源", "置信度"],
  rows: [
    ["NVDA", "训练集群确认采用 GB300 NVL72", "2026-09-03", "订单 / 产品", "正面：推理算力需求上修", "6-12 个月", "NVDA 8-K", "高"],
    ["MSFT", "Azure 独家承载 API 首发", "2026-09-04", "渠道 / 分成", "正面：Azure AI 收入占比抬升", "3-6 个月", "MSFT 官方博客", "高"],
    ["AVGO", "定制 XPU 进入量产", "2026-09-05", "定制芯片", "正面：ASIC 收入进入兑现期", "12 个月以上", "财报电话会", "中"],
    ["AMZN", "追加 Trainium 3 采购", "2026-09-05", "反向对冲", "中性偏正", "6-12 个月", "re:Invent 预告", "中"],
  ],
};

const TWO_ROW_WIDE = {
  header: ["公司", "营收（亿美元）", "同比", "净利润（亿美元）", "EPS", "PE（TTM）", "目标价"],
  rows: [
    ["NVDA", "468.2", "+56%", "264.4", "1.08", "42.6", "$215"],
    ["AMD", "92.4", "+32%", "12.1", "0.74", "58.3", "$198"],
  ],
};

const NARROW = {
  header: ["指标", "数值", "说明"],
  rows: [
    ["数据中心营收", "412 亿", "含网络业务"],
    ["毛利率", "74.8%", "Non-GAAP"],
  ],
};

function tableHtml(shape: { header: string[]; rows: string[][] }) {
  const cell = (tag: string, text: string) => `<${tag}>${text}</${tag}>`;
  return (
    "<table><thead><tr>" +
    shape.header.map((h) => cell("th", h)).join("") +
    "</tr></thead><tbody>" +
    shape.rows.map((r) => "<tr>" + r.map((c) => cell("td", c)).join("") + "</tr>").join("") +
    "</tbody></table>"
  );
}

describe("share card table layout", () => {
  test("capacity follows the message size: bigger type, fewer em of room", () => {
    expect(shareTableCapacityEm(14)).toBeGreaterThan(shareTableCapacityEm(18));
    expect(shareTableCapacityEm(15)).toBeCloseTo(380 / 13.5, 3);
  });

  test("CJK counts as a full em, Latin and digits as about half", () => {
    expect(estimateTextEm("营收")).toBe(2);
    expect(estimateTextEm("2026")).toBeCloseTo(2.32, 5);
    expect(estimateTextEm("NVDA")).toBeCloseTo(2.72, 5);
  });

  test("a narrow table stays a table", () => {
    expect(decideShareTableLayout(NARROW, CAPACITY)).toBe("keep");
  });

  test("one or two columns always stay a table, however long the cells", () => {
    const shape = {
      header: ["指标", "说明"],
      rows: [["数据中心营收", "包含网络业务、系统与软件，以及 DGX Cloud 的全部收入，按季度口径统计，不含一次性项目。"]],
    };
    expect(decideShareTableLayout(shape, CAPACITY)).toBe("keep");
  });

  test("a wide table with two data rows is rotated", () => {
    expect(decideShareTableLayout(TWO_ROW_WIDE, CAPACITY)).toBe("transpose");
  });

  test("a wide, tall table becomes one block per row", () => {
    expect(decideShareTableLayout(EVENT_MATRIX, CAPACITY)).toBe("list");
  });

  test("transposing keeps the corner cell and swaps axes", () => {
    const rotated = transposeTable(TWO_ROW_WIDE);
    expect(rotated.header).toEqual(["公司", "NVDA", "AMD"]);
    expect(rotated.rows[0]).toEqual(["营收（亿美元）", "468.2", "92.4"]);
    expect(rotated.rows).toHaveLength(6);
  });
});

describe("reflowShareTables", () => {
  test("leaves html without tables untouched, byte for byte", () => {
    const html = "<p>没有表格的<strong>回答</strong></p>";
    expect(reflowShareTables(html, CAPACITY)).toBe(html);
  });

  test("keeps a narrow table as a plain table", () => {
    const out = reflowShareTables(tableHtml(NARROW), CAPACITY);
    expect(out).toContain("<table>");
    expect(out).not.toContain("data-hone-share-table");
  });

  test("rotates a wide short table into row headers", () => {
    const out = reflowShareTables(tableHtml(TWO_ROW_WIDE), CAPACITY);
    expect(out).toContain('data-hone-share-table="transposed"');
    expect(out).toContain("<thead><tr><th>公司</th><th>NVDA</th><th>AMD</th></tr></thead>");
    expect(out).toContain('<tr><th scope="row">营收（亿美元）</th><td>468.2</td><td>92.4</td></tr>');
  });

  test("turns a wide tall table into titled blocks of header/value lines", () => {
    const out = reflowShareTables(tableHtml(EVENT_MATRIX), CAPACITY);
    expect(out).not.toContain("<table");
    expect(out).toContain('class="hf-share-tlist"');
    expect((out.match(/hf-share-tlist__item/g) ?? []).length).toBe(4);
    expect(out).toContain('<div class="hf-share-tlist__title">NVDA</div>');
    // Every block carries the labels — headers are cloned, not moved.
    expect((out.match(/>置信度<\/span>/g) ?? []).length).toBe(4);
    // The title column's header is not repeated as a line.
    expect(out).not.toContain(">公司</span>");
    expect(out).toContain(">置信度</span><span class=\"hf-share-tlist__v\">高</span>");
  });

  test("moves the sanitiser's inline nodes instead of re-parsing text", () => {
    const html = tableHtml({
      header: ["公司", "事件", "时间", "类型", "影响", "期限", "来源", "置信度"],
      rows: [
        ["<strong>NVDA</strong>", "<a href=\"https://example.com\">8-K</a>", "2026-09-03", "订单", "正面", "6-12 个月", "来源", "高"],
        ["MSFT", "x", "2026-09-04", "渠道", "正面", "3-6 个月", "来源", "高"],
        ["AVGO", "x", "2026-09-05", "芯片", "正面", "12 个月", "来源", "中"],
        ["AMZN", "x", "2026-09-05", "对冲", "中性", "6-12 个月", "来源", "中"],
      ],
    });
    const out = reflowShareTables(html, CAPACITY);
    expect(out).toContain('<div class="hf-share-tlist__title"><strong>NVDA</strong></div>');
    expect(out).toContain('<a href="https://example.com">8-K</a>');
  });

  test("an empty cell in a block shows a dash rather than a blank line", () => {
    const shape = {
      ...EVENT_MATRIX,
      rows: EVENT_MATRIX.rows.map((row, i) => (i === 0 ? row.map((c, j) => (j === 7 ? "" : c)) : row)),
    };
    const out = reflowShareTables(tableHtml(shape), CAPACITY);
    expect(out).toContain('>置信度</span><span class="hf-share-tlist__v">—</span>');
  });
});
