#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const [inputPath, outputPath] = process.argv.slice(2);
if (!inputPath || !outputPath) {
  throw new Error("usage: build_company_cards.mjs <workbooks.json> <output.json>");
}

const workbookDump = JSON.parse(fs.readFileSync(inputPath, "utf8"));
const rows = workbookDump["AI稀缺性_Leopold式美股组合分析.xlsx"]?.["1_Leopold组合"]?.values;
if (!Array.isArray(rows) || rows.length < 2) throw new Error("research sheet not found");

const excluded = new Set(["核电篮子", "量子板块"]);
const symbolOverrides = new Map([
  ["SpaceX", "SPCX"],
  ["Sivers Semiconductors", "SIVEF"],
]);
const transcriptOverrides = new Map([
  ["GOOGL", "20251120202745-谷歌-逐字稿文本-1.txt"],
]);
const lowConfidence = new Set(["SIVEF", "CBRS", "CAI", "SPCX"]);

function valuationMethod(theme, symbol) {
  if (["WDC", "STX", "MU", "SNDK"].includes(symbol)) return "中周期 forward P/E 与 EV/EBITDA，结合供需、合约覆盖和库存交叉验证";
  if (["LRCX", "AMAT", "KLAC", "TER"].includes(symbol)) return "跨 WFE 周期的 forward P/E、FCF 与服务收入质量；比较订单/装机基数";
  if (["NBIS", "CRWV", "IREN"].includes(symbol)) return "EV/S 与合同 backlog，扣除债务、资本开支、折旧和融资成本后验证";
  if (["RXRX", "TEM", "CAI"].includes(symbol)) return "EV/S、现金跑道和商业化里程碑；不以远期管线概率替代收入";
  if (["RKLB", "RDW", "SPCX"].includes(symbol)) return "分部 EV/S 与订单质量，结合发射/交付里程碑、资本开支和稀释情景";
  if (["CBRS", "ARM", "FIG", "ALAB", "CRDO"].includes(symbol)) return "forward EV/S 与增长、毛利、经营杠杆的相对估值，辅以盈利拐点情景";
  if (theme?.includes("平台") || ["GOOGL", "MSFT", "AMZN", "META", "APP"].includes(symbol)) return "forward P/E 与 FCF/DCF，检验资本开支转化为收入和自由现金流的速度";
  return "forward P/E、FCF yield 与同类公司/自身历史区间交叉验证，并做增长和毛利敏感性分析";
}

function fromRow(row) {
  const [name, rawSymbol, theme, chain, business, moat, scarcity, pricing, visibility, execution, valuationRisk, staticScore, , , , , , , , , thesis, risk, currentNote, transcript] = row;
  const symbol = symbolOverrides.get(name) ?? String(rawSymbol).split("/").at(-1);
  const canonicalTranscript = transcriptOverrides.get(symbol)
    ?? String(transcript ?? "").replace("-1(1).txt", "-1.txt");
  return {
    name,
    symbol,
    market_scope: symbol === "SIVEF" ? "us_otc_foreign_ordinary" : "us_listed",
    theme,
    value_chain: chain,
    business_model: business,
    moat,
    thesis_summary: thesis,
    valuation_method: valuationMethod(theme, symbol),
    dimensions_1_to_5: { scarcity, pricing_quality: pricing, visibility, execution, valuation_risk: valuationRisk },
    static_score: staticScore,
    confidence: lowConfidence.has(symbol) ? "low" : "medium",
    watch_items: [currentNote].filter(Boolean),
    risks: [risk].filter(Boolean),
    falsifiers: ["最新财报或订单数据连续两个季度违背核心增长逻辑", "竞争或供给变化使研究卡所述稀缺性和议价能力明显下降"],
    transcript_sources: [canonicalTranscript].filter(Boolean),
    source_updated_at: "2026-06-20",
  };
}

const cards = rows.slice(1).filter((row) => !excluded.has(row[0])).map(fromRow);

cards.push(
  {
    name: "Lam Research", symbol: "LRCX", market_scope: "us_listed", theme: "半导体设备", value_chain: "刻蚀/沉积/晶圆设备服务",
    business_model: "设备销售加装机基数服务与耗材；服务业务降低单一资本开支周期波动",
    moat: "3D NAND 高深宽比刻蚀与沉积工艺、长期验证、装机数据和服务网络形成高转换成本",
    thesis_summary: "NAND 层数提升和企业级 SSD 拉动工艺强度；相对 DRAM/HBM 更偏 2027 NAND 资本开支弹性",
    valuation_method: valuationMethod("", "LRCX"), dimensions_1_to_5: { scarcity: 5, pricing_quality: 5, visibility: 4, execution: 5, valuation_risk: 4 }, static_score: 85, confidence: "medium",
    watch_items: ["NAND 客户资本开支与 CSBG 增速", "中国收入及出口限制影响", "先进封装与 DRAM/HBM 份额"],
    risks: ["WFE 周期和 NAND 扩产推迟", "中国限制与国产替代", "高估值压缩"],
    falsifiers: ["NAND 工艺强度提升未转化为订单", "服务收入增速和装机利用率持续下降"],
    transcript_sources: ["20260625202409-LRCX-逐字稿文本-1.txt"], source_updated_at: "2026-06-25",
  },
  {
    name: "Applied Materials", symbol: "AMAT", market_scope: "us_listed", theme: "半导体设备", value_chain: "材料工程/沉积/刻蚀/先进封装",
    business_model: "广覆盖半导体设备平台加长期服务，横跨逻辑、DRAM、NAND 和封装",
    moat: "数十年工艺配方、广泛装机基数、良率集成和客户共同开发形成平台护城河",
    thesis_summary: "DRAM/HBM、GAA、背面供电和先进封装共同提高材料工程强度，业务广度使周期更稳",
    valuation_method: valuationMethod("", "AMAT"), dimensions_1_to_5: { scarcity: 5, pricing_quality: 5, visibility: 4, execution: 5, valuation_risk: 4 }, static_score: 85, confidence: "medium",
    watch_items: ["DRAM/HBM 与先进封装收入", "服务业务增长", "中国限制后的收入结构"],
    risks: ["中国限制和国产替代", "WFE 周期", "估值已反映较强增长"],
    falsifiers: ["新架构材料强度未带来份额或订单提升", "服务和自由现金流质量持续下降"],
    transcript_sources: ["20260702201824-AMAT-逐字稿文本-1.txt"], source_updated_at: "2026-07-02",
  },
  {
    name: "KLA", symbol: "KLAC", market_scope: "us_listed", theme: "半导体设备", value_chain: "过程控制/检测/良率管理",
    business_model: "高毛利检测设备加装机服务；制程复杂度提升带动检测强度",
    moat: "前端检测份额、缺陷数据、算法与客户工艺反馈形成数据飞轮和高转换成本",
    thesis_summary: "先进制程、HBM 和封装复杂度上升使良率控制成为刚需，收入质量通常优于纯周期设备",
    valuation_method: valuationMethod("", "KLAC"), dimensions_1_to_5: { scarcity: 5, pricing_quality: 5, visibility: 5, execution: 5, valuation_risk: 4 }, static_score: 91, confidence: "medium",
    watch_items: ["过程控制占 WFE 比例", "服务收入和毛利率", "先进封装检测订单"],
    risks: ["高估值", "WFE 节奏和中国限制", "竞争技术追赶"],
    falsifiers: ["检测强度随制程复杂度上升的关系减弱", "份额、毛利或服务续约持续下滑"],
    transcript_sources: ["20260709202737-KLAC-逐字稿文本-1.txt"], source_updated_at: "2026-07-09",
  },
  {
    name: "Micron", symbol: "MU", market_scope: "us_listed", theme: "存储/HBM", value_chain: "DRAM/HBM/NAND 制造",
    business_model: "存储芯片制造；价格、位元供给和产品组合共同决定利润",
    moat: "先进 DRAM/HBM 工艺、客户认证和资本密集型供给纪律；护城河低于独占型平台但供给集中",
    thesis_summary: "AI 推理提高 HBM 与 DRAM 内容量，长期协议可能降低周期性；仍必须用供给和中周期利润检验",
    valuation_method: valuationMethod("", "MU"), dimensions_1_to_5: { scarcity: 4, pricing_quality: 4, visibility: 4, execution: 4, valuation_risk: 4 }, static_score: 76, confidence: "medium",
    watch_items: ["HBM 收入/份额和毛利", "长期协议覆盖", "行业资本开支和库存"],
    risks: ["2027 下半年至 2028 新供给释放", "价格正常化", "韩国与中国竞争"],
    falsifiers: ["HBM 份额或认证进度持续落后", "供给增速重新显著超过需求并导致价格连续下跌"],
    transcript_sources: ["20260716202748-美光-逐字稿文本-1.txt"], source_updated_at: "2026-07-16",
  },
  {
    name: "SK hynix", symbol: "SKHY", market_scope: "us_adr", theme: "存储/HBM", value_chain: "DRAM/HBM/NAND 制造",
    business_model: "存储芯片制造，以 HBM 领先地位参与 AI 加速器供应链",
    moat: "HBM 量产良率、先进封装和头部客户认证形成当前领先，但需持续研发和资本投入",
    thesis_summary: "作为 HBM 龙头提供更纯的 AI 存储敞口；美国 ADR 上市时间短，估值和流动性口径需谨慎",
    valuation_method: valuationMethod("", "MU"), dimensions_1_to_5: { scarcity: 5, pricing_quality: 5, visibility: 5, execution: 5, valuation_risk: 4 }, static_score: 91, confidence: "low",
    watch_items: ["HBM 代际量产和客户份额", "DRAM 价格与资本开支", "ADR 流动性和折溢价"],
    risks: ["HBM 竞争追赶", "存储周期", "ADR 历史和数据覆盖较短"],
    falsifiers: ["下一代 HBM 认证或量产落后主要竞争者", "HBM 溢价和份额连续下降"],
    transcript_sources: ["20260723202325-海力士-逐字稿文本-1.txt"], source_updated_at: "2026-07-23",
  },
  {
    name: "Sandisk", symbol: "SNDK", market_scope: "us_listed", theme: "存储/企业级 SSD", value_chain: "NAND/控制器/固件/SSD",
    business_model: "NAND 制造与企业/消费 SSD，控制器和固件把闪存转化为系统产品",
    moat: "NAND 制造、控制器、固件、长期认证和 CSP 合同组合，企业级产品转换成本较高",
    thesis_summary: "AI 推理和 KV cache 提高企业级 SSD 需求与价值量，但仍受 NAND 供给周期约束",
    valuation_method: valuationMethod("", "SNDK"), dimensions_1_to_5: { scarcity: 4, pricing_quality: 4, visibility: 4, execution: 4, valuation_risk: 4 }, static_score: 76, confidence: "medium",
    watch_items: ["数据中心收入占比和毛利", "价格与出货量贡献", "NAND 行业资本开支"],
    risks: ["2027 下半年至 2028 供给增加", "NAND 价格周期", "竞争和客户集中"],
    falsifiers: ["企业级份额/认证未改善", "收入增长主要由短期价格而非产品结构驱动且价格开始反转"],
    transcript_sources: ["20260806202703-SNDK-逐字稿文本-1.txt"], source_updated_at: "2026-08-06",
  },
);

cards.sort((a, b) => b.static_score - a.static_score || a.symbol.localeCompare(b.symbol));
const payload = {
  schema_version: 1,
  generated_from: "authorized internal transcript research workbook plus reviewed newer transcripts",
  companies: cards,
  evidence_only_topics: ["核电篮子", "量子板块", "年度策略与问答"],
  evidence_sources: [
    "20250911202938-宏观+量子-逐字稿文本-1.txt",
    "20251211210019-核电-逐字稿文本-1.txt",
    "20260101202609-跨年分享-逐字稿文本-1.txt",
    "20260730202247-答疑专场-逐字稿文本-1 (1).txt",
  ],
};
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(payload, null, 2)}\n`);
console.log(`wrote ${cards.length} company cards to ${outputPath}`);
