import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  ArrowLeft,
  ArrowRight,
  BellRinging,
  BookmarkSimple,
  CaretDown,
  ChartLineUp,
  DotsThree,
  Sparkle,
  WarningCircle,
} from "@phosphor-icons/react";
import { CandlestickChart } from "@/components/CandlestickChart";
import { holdings, insights } from "@/data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const metrics = [
  ["总市值", "$3.12T", "全球半导体 1/68"],
  ["市盈率 TTM", "41.8×", "五年分位 64%"],
  ["Forward P/E", "34.6×", "FY2027E"],
  ["市销率", "22.1×", "五年分位 71%"],
  ["营收增速", "+68.2%", "最近财年"],
  ["毛利率", "74.6%", "同比 +1.1pct"],
  ["自由现金流率", "47.2%", "数据截至 2026 Q1"],
  ["ROIC", "86.4%", "最近十二个月"],
];

function PeriodTabs() {
  const [period, setPeriod] = useState("1m");
  return <div className="flex items-center gap-1">{[["1d","分时"],["5d","5日"],["1m","日K"],["1y","周K"],["5y","月K"]].map(([value,label])=><Button key={value} variant={period===value?"secondary":"ghost"} size="sm" className="h-7 px-2.5 text-xs" onClick={()=>setPeriod(value)}>{label}</Button>)}</div>;
}

function HoldingSummary({ holding, compact = false }) {
  const items = [
    ["持有市值", holding.value],
    ["持有数量", `${holding.quantity} 股`],
    ["平均成本", holding.cost],
    ["最新价格", holding.price],
    ["今日盈亏", `${holding.todayAmount} · ${holding.today}`],
    ["累计盈亏", `${holding.ytdAmount} · ${holding.ytd}`],
  ];
  return <div className={`grid ${compact?"grid-cols-3 gap-x-3 gap-y-4":"grid-cols-2 gap-x-6 gap-y-5"}`}>{items.map(([label,value])=><div key={label}><div className="text-[11px] text-muted-foreground">{label}</div><div className={`mt-1 font-semibold ${compact?"text-xs":"text-sm"}`}>{value}</div></div>)}</div>;
}

function MobileCompanyPage({ holding }) {
  const navigate = useNavigate();
  const [tab, setTab] = useState("trend");
  return <div className="lg:hidden">
    <div className="flex h-14 items-center justify-between border-b px-2"><Button variant="ghost" size="icon" onClick={()=>navigate(-1)} aria-label="返回投资"><ArrowLeft size={21}/></Button><div className="text-center"><div className="text-sm font-semibold">{holding.ticker}</div><div className="text-[10px] text-muted-foreground">NASDAQ · 延迟 15 分钟</div></div><Button variant="ghost" size="icon" aria-label="更多操作"><DotsThree size={23}/></Button></div>
    <div className="px-4 pb-5">
      <div className="flex items-start justify-between border-b py-4"><div><h1 className="text-lg font-semibold">{holding.name}</h1><div className="mt-2 flex items-baseline gap-2"><span className="text-2xl font-semibold tracking-tight">{holding.price}</span><span className="text-sm font-medium">{holding.today}</span></div></div><Badge variant="secondary" className="mt-1">已持仓 · {holding.weight}</Badge></div>
      <div className="border-b py-4"><HoldingSummary holding={holding} compact/></div>
      <div className="flex items-center justify-between py-3"><PeriodTabs/><Button variant="ghost" size="sm" className="h-7 px-2 text-xs">前复权 <CaretDown/></Button></div>
      <div className="-mx-1"><CandlestickChart compact/></div>
      <div className="grid grid-cols-4 border-y text-center">{[["trend","走势"],["position","持仓"],["financials","财务"],["research","研究"]].map(([value,label])=><button key={value} onClick={()=>setTab(value)} className={`relative py-3 text-xs ${tab===value?"font-semibold":"text-muted-foreground"}`}>{label}{tab===value&&<span className="absolute inset-x-5 bottom-0 h-0.5 bg-foreground"/>}</button>)}</div>
      {tab==="trend"&&<div className="grid grid-cols-2 gap-x-8 gap-y-3 py-4 text-xs">{[["今开","126.42"],["最高","129.08"],["最低","124.96"],["成交量","386.2M"],["52周最高","152.89"],["52周最低","86.62"]].map(([label,value])=><div key={label} className="flex justify-between"><span className="text-muted-foreground">{label}</span><b>{value}</b></div>)}</div>}
      {tab==="position"&&<div className="py-4"><HoldingSummary holding={holding}/><div className="mt-4 rounded-lg border p-3 text-xs leading-5 text-muted-foreground">成本线和历史买入点已标记在 K 线上；组合收益使用账户资金流校正。</div></div>}
      {tab==="financials"&&<div className="grid grid-cols-2 gap-3 py-4">{metrics.slice(0,6).map(([label,value,helper])=><div key={label} className="border-b pb-3"><div className="text-[11px] text-muted-foreground">{label}</div><div className="mt-1 text-sm font-semibold">{value}</div><div className="mt-0.5 text-[10px] text-muted-foreground">{helper}</div></div>)}</div>}
      {tab==="research"&&<button onClick={()=>navigate("/app/agent?company=NVDA")} className="mt-4 flex w-full items-start gap-3 rounded-xl border p-4 text-left"><span className="grid size-9 shrink-0 place-items-center rounded-full bg-foreground text-background"><Sparkle weight="fill"/></span><span><span className="text-xs text-muted-foreground">Agent 研究信号</span><span className="mt-1 block text-sm font-semibold">Blackwell 供给兑现仍是下一季关键变量</span><span className="mt-1 block text-xs leading-5 text-muted-foreground">结合持仓成本、产业事件和老王研究主线生成。</span></span></button>}
      <div className="mt-3 grid grid-cols-[1fr_1.4fr] gap-2"><Button variant="outline" onClick={()=>navigate("/app/tracking")}><BellRinging/>建立跟踪</Button><Button onClick={()=>navigate("/app/agent?company=NVDA")}><Sparkle/>询问 Agent</Button></div>
    </div>
  </div>;
}

function DesktopCompanyPage({ holding }) {
  const navigate = useNavigate();
  return <div className="mx-auto hidden max-w-[1500px] px-6 py-6 lg:block">
    <Button variant="ghost" size="sm" onClick={()=>navigate(-1)} className="mb-4 -ml-2"><ArrowLeft/>返回投资</Button>
    <div className="mb-5 flex items-start justify-between"><div><div className="flex items-center gap-2"><h1 className="text-3xl font-semibold tracking-tight">{holding.name}</h1><Badge variant="secondary">{holding.ticker}</Badge><Badge variant="outline">NASDAQ</Badge></div><div className="mt-3 flex items-baseline gap-3"><span className="text-2xl font-semibold">{holding.price}</span><span className="text-sm font-medium">{holding.today} 今日</span><span className="text-xs text-muted-foreground">延迟 15 分钟</span></div></div><div className="flex gap-2"><Button variant="outline"><BookmarkSimple/>已持仓</Button><Button variant="outline" onClick={()=>navigate("/app/tracking")}><BellRinging/>建立跟踪</Button><Button onClick={()=>navigate("/app/agent?company=NVDA")}><Sparkle/>询问 Agent</Button></div></div>
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
      <div className="min-w-0 space-y-4">
        <Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0 pb-2"><div><CardTitle className="text-sm">价格走势</CardTitle><div className="mt-1 text-xs text-muted-foreground">K 线 · 成交量 · 我的平均成本与买入点</div></div><div className="flex items-center gap-3"><PeriodTabs/><Button variant="outline" size="sm">指标 <CaretDown/></Button></div></CardHeader><CardContent><CandlestickChart/></CardContent></Card>
        <Card className="shadow-none"><CardHeader><CardTitle className="text-sm">市场与财务数据</CardTitle></CardHeader><CardContent className="grid grid-cols-4 gap-x-8 gap-y-6">{metrics.map(([label,value,helper])=><div key={label}><div className="text-xs text-muted-foreground">{label}</div><div className="mt-1 text-lg font-semibold">{value}</div><div className="mt-0.5 text-xs text-muted-foreground">{helper}</div></div>)}</CardContent></Card>
        <Tabs defaultValue="thesis"><TabsList><TabsTrigger value="thesis">投资主线</TabsTrigger><TabsTrigger value="financials">财务预测</TabsTrigger><TabsTrigger value="insights">关联洞察</TabsTrigger></TabsList><TabsContent value="thesis" className="mt-3"><Card className="shadow-none"><CardContent className="grid gap-6 p-5 lg:grid-cols-[1.4fr_1fr]"><div><Badge variant="secondary">2026-07-11 更新</Badge><h2 className="mt-4 text-xl font-semibold">AI 计算平台与 CUDA 生态仍是核心护城河</h2><p className="mt-3 text-sm leading-7 text-muted-foreground">未来两个季度关注 Blackwell 供给兑现、HBM4 与先进封装产能、推理工作负载占比，以及云厂商自研芯片是否改变平台议价能力。</p></div><div className="rounded-lg border p-4"><div className="flex items-center gap-2 text-sm font-semibold"><WarningCircle/>证伪条件</div><p className="mt-3 text-xs leading-6 text-muted-foreground">软件迁移成本显著下降，或推理侧性能/成本优势连续两个季度落后。</p></div></CardContent></Card></TabsContent><TabsContent value="financials" className="mt-3"><Card className="shadow-none"><CardContent className="grid grid-cols-3 gap-8 p-6">{[["FY2026E 营收","$201.4B","同比 +52%"],["FY2027E EPS","$4.18","一致预期 +6%"],["FY2027E FCF","$102.8B","FCF率 48.7%"]].map(([label,value,helper])=><div key={label}><div className="text-xs text-muted-foreground">{label}</div><div className="mt-2 text-2xl font-semibold">{value}</div><div className="mt-1 text-xs text-muted-foreground">{helper}</div></div>)}</CardContent></Card></TabsContent><TabsContent value="insights" className="mt-3"><div className="grid grid-cols-2 gap-3">{insights.slice(0,2).map(item=><Card key={item.slug} className="cursor-pointer shadow-none" onClick={()=>navigate(`/app/insights/${item.slug}`)}><CardContent className="p-5"><div className="text-xs text-muted-foreground">{item.type} · {item.time}</div><h3 className="mt-2 font-semibold">{item.title}</h3><p className="mt-2 line-clamp-2 text-sm leading-6 text-muted-foreground">{item.excerpt}</p></CardContent></Card>)}</div></TabsContent></Tabs>
      </div>
      <aside className="space-y-4">
        <Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0"><CardTitle className="text-sm">我的持仓</CardTitle><Badge variant="secondary">{holding.weight}</Badge></CardHeader><CardContent><HoldingSummary holding={holding}/><div className="mt-5 border-t pt-4"><div className="mb-2 flex justify-between text-xs"><span>距平均成本</span><b>+24.1%</b></div><Progress value={62}/></div></CardContent></Card>
        <Card className="border-foreground shadow-none"><CardContent className="p-5"><div className="flex items-center gap-2"><Sparkle weight="fill"/><span className="text-xs text-muted-foreground">Agent 研究信号</span></div><h3 className="mt-3 text-base font-semibold leading-6">Blackwell 供给兑现仍是下一季关键变量</h3><p className="mt-2 text-xs leading-6 text-muted-foreground">结合持仓成本、公司事件、老王洞察与历史研究主线生成。</p><Button className="mt-4 w-full" onClick={()=>navigate("/app/agent?company=NVDA")}>继续研究 <ArrowRight/></Button></CardContent></Card>
        <Card className="shadow-none"><CardHeader><CardTitle className="text-sm">关键催化与风险</CardTitle></CardHeader><CardContent className="space-y-4 text-xs"><div className="border-l-2 pl-3"><div className="font-semibold">下一季财报 · 8 月 27 日</div><p className="mt-1 leading-5 text-muted-foreground">Blackwell 收入占比与毛利率指引。</p></div><div className="border-l-2 border-muted-foreground pl-3"><div className="font-semibold">出口限制更新</div><p className="mt-1 leading-5 text-muted-foreground">中国区替代方案与潜在订单影响。</p></div></CardContent></Card>
      </aside>
    </div>
  </div>;
}

export function CompanyPage() {
  const { ticker = "NVDA" } = useParams();
  const holding = holdings.find((item)=>item.ticker===ticker) ?? holdings[0];
  return <><MobileCompanyPage holding={holding}/><DesktopCompanyPage holding={holding}/></>;
}
