export type DataCenterZoneId =
  "chip" | "storage" | "optical" | "power" | "cooling" | "software";

export interface DataCenterZone {
  id: DataCenterZoneId;
  shortLabel: string;
  title: string;
  subtitle: string;
  description: string;
  location: string;
  color: string;
  components: string[];
  focus: string[];
  industries: { id: string; name: string }[];
}

// A spatial introduction to the existing industry map, not a separate taxonomy.
// Equipment is upstream of the facility; cooling belongs to power, and the
// software layer links to cloud/platform research rather than a software sector.
export const DATA_CENTER_ZONES: readonly DataCenterZone[] = [
  {
    id: "chip",
    shortLabel: "芯",
    title: "AI 芯片与算力",
    subtitle: "让模型真正运转的计算核心",
    description:
      "GPU 与定制 ASIC 在算力机柜中执行训练和推理。服务器把芯片、内存与互联集成为可交付系统；先进制程、封装和半导体设备则是机房外的上游产能支撑。",
    location: "算力机柜 · 加速器与服务器",
    color: "#d69462",
    components: ["GPU / ASIC", "AI 服务器", "先进制程与封装"],
    focus: ["芯片与整机交付", "先进封装产能", "算力密度与成本"],
    industries: [
      { id: "ai-chip", name: "AI 芯片" },
      { id: "server-oem", name: "AI 服务器与整机" },
      { id: "equipment", name: "半导体设备" },
    ],
  },
  {
    id: "storage",
    shortLabel: "存",
    title: "内存与存储",
    subtitle: "让数据跟得上算力",
    description:
      "HBM 紧邻加速器，DRAM 服务运行中的任务，企业级 SSD 与硬盘承载数据和模型。它们分布在计算与存储机柜中，共同影响容量、带宽和数据供给。",
    location: "存储机柜 · 同时延伸至芯片侧",
    color: "#a895cf",
    components: ["HBM 高带宽内存", "服务器 DRAM", "企业级 SSD / HDD"],
    focus: ["每颗芯片的内存容量", "内存与存储供给", "带宽和容量的成本"],
    industries: [{ id: "storage", name: "存储" }],
  },
  {
    id: "optical",
    shortLabel: "光",
    title: "光通信与互联",
    subtitle: "把一颗颗芯片连成一个集群",
    description:
      "交换机与光模块把机柜连接起来，让训练数据和计算结果在集群中流动。机柜内还会使用铜缆等连接方式；不同距离与拓扑，对带宽、时延和功耗的要求各不相同。",
    location: "网络机柜 · 机柜间连接",
    color: "#55bba8",
    components: ["交换机与光模块", "光器件与 DSP", "AEC 有源电缆 / CPO"],
    focus: ["交换端口与带宽升级", "光器件产能", "互联拓扑与光化进展"],
    industries: [{ id: "optical", name: "光通信" }],
  },
  {
    id: "power",
    shortLabel: "电",
    title: "电力与配电",
    subtitle: "从接入电网到每一座机柜",
    description:
      "电力从园区接入，经变压与配电送到机柜。数据中心能否按期投入使用，既取决于电源，也取决于并网、设备交付和真正可用的供电容量。",
    location: "供配电区 · 园区电力入口",
    color: "#d4ad64",
    components: ["电网与发电", "变压器与开关设备", "机柜供配电"],
    focus: ["并网与上电进度", "电力设备交期", "机柜功率与设施能耗"],
    industries: [{ id: "power", name: "电力" }],
  },
  {
    id: "cooling",
    shortLabel: "冷",
    title: "液冷与散热",
    subtitle: "把高密度计算产生的热量带走",
    description:
      "冷却系统把热量从芯片和机柜带到室外。高密度算力会改变风冷与液冷的组合，也影响设施能耗；散热与供配电共同构成电力行业分析的一部分。",
    location: "冷却区 · 连接算力机柜与室外",
    color: "#68adc4",
    components: ["机柜液冷", "冷却循环", "室外散热设备"],
    focus: ["机柜功率密度", "冷却系统交付", "PUE 与能耗效率"],
    industries: [{ id: "power", name: "电力" }],
  },
  {
    id: "software",
    shortLabel: "AI",
    title: "AI 软件与云平台",
    subtitle: "把基础设施变成可用的 AI 服务",
    description:
      "软件在整座数据中心之上组织算力，承接训练、推理与云服务。这是跨越机柜的服务层；相关行业研究从云厂与 AI 平台、新云展开。",
    location: "软件服务层 · 覆盖整个数据中心",
    color: "#8aa9cf",
    components: ["算力调度", "模型训练与推理", "AI 云与平台服务"],
    focus: ["算力需求与资本开支", "云服务收入与合同兑现", "算力租赁与客户结构"],
    industries: [
      { id: "hyperscaler", name: "云厂与 AI 平台" },
      { id: "neocloud", name: "新云" },
    ],
  },
];

export function industryHref(id: string): string {
  return `/industry-map?industry=${encodeURIComponent(id)}`;
}
