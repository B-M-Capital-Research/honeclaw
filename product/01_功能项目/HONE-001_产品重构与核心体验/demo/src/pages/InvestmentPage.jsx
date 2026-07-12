import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  ArrowRight,
  ArrowsDownUp,
  Bank,
  CalendarDots,
  CaretDown,
  CheckCircle,
  CloudArrowDown,
  MagnifyingGlass,
  SlidersHorizontal,
  Sparkle,
  Stack,
  WarningCircle,
} from "@phosphor-icons/react";
import { CartesianGrid, Line, LineChart, ResponsiveContainer, XAxis, YAxis } from "recharts";
import { holdings, insights, performanceData, trackingEvents } from "@/data";
import { PortfolioDataDialog } from "@/components/PortfolioDataDialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

const chartConfig = {
  portfolio: { label: "我的组合", color: "var(--foreground)" },
  benchmark: { label: "纳斯达克100", color: "var(--muted-foreground)" },
};

function Metric({ label, value, helper }) {
  return <div><div className="text-xs text-muted-foreground">{label}</div><div className="mt-1 text-lg font-semibold tracking-tight">{value}</div>{helper&&<div className="mt-0.5 text-xs text-muted-foreground">{helper}</div>}</div>;
}

function MiniTrend({ data }) {
  return <div className="h-8 w-12"><ResponsiveContainer width="100%" height="100%"><LineChart data={data.map((value,index)=>({index,value}))}><Line dataKey="value" type="monotone" stroke="var(--foreground)" strokeWidth={1.4} dot={false} isAnimationActive={false}/></LineChart></ResponsiveContainer></div>;
}

function MobilePositionRow({ item, onOpen }) {
  return <button onClick={onOpen} className="grid w-full grid-cols-[minmax(0,1fr)_64px_58px_58px_38px] items-center gap-1.5 border-b py-2 text-left last:border-0"><div className="min-w-0"><div className="truncate text-[13px] font-semibold">{item.name}</div><div className="mt-0.5 text-[11px] text-muted-foreground">{item.ticker}</div></div><div className="text-right"><div className="text-[11px] font-medium">{item.value}</div><div className="mt-0.5 text-[10px] text-muted-foreground">{item.weight}</div></div><div className="text-right"><div className="text-[11px] font-medium">{item.todayAmount}</div><div className="mt-0.5 text-[10px] text-muted-foreground">{item.today}</div></div><div className="text-right"><div className="text-[11px] font-medium">{item.ytdAmount}</div><div className="mt-0.5 text-[10px] text-muted-foreground">{item.ytd}</div></div><MiniTrend data={item.spark}/></button>;
}

function MobileInvestmentPage() {
  const navigate = useNavigate();
  const [view, setView] = useState("holdings");
  return <div className="lg:hidden">
    <div className="flex h-16 items-center justify-between border-b px-4"><h1 className="text-2xl font-semibold tracking-tight">投资</h1><div className="flex items-center gap-1"><Button variant="ghost" size="icon" aria-label="搜索"><MagnifyingGlass size={21}/></Button><PortfolioDataDialog><Button variant="ghost" size="icon" aria-label="同步持仓"><CloudArrowDown size={21}/></Button></PortfolioDataDialog></div></div>
    <div className="px-4 pb-4">
      <div className="grid grid-cols-3 border-b text-center">{[["assets","资产"],["holdings","持仓"],["performance","表现"]].map(([value,label])=><button key={value} onClick={()=>setView(value)} className={`relative py-3 text-sm ${view===value?"font-semibold":"text-muted-foreground"}`}>{label}{view===value&&<span className="absolute inset-x-7 bottom-0 h-0.5 bg-foreground"/>}</button>)}</div>
      <div className="grid grid-cols-[1.2fr_1fr_1fr] gap-3 border-b py-4"><div><div className="text-[11px] text-muted-foreground">总资产（CNY）</div><div className="mt-1 text-xl font-semibold tracking-tight">¥1,284,620</div></div><div className="border-l pl-3"><div className="text-[11px] text-muted-foreground">今日变化</div><div className="mt-1 text-sm font-semibold">+¥22,620</div><div className="text-[11px] text-muted-foreground">+1.8%</div></div><div className="border-l pl-3"><div className="text-[11px] text-muted-foreground">累计变化</div><div className="mt-1 text-sm font-semibold">+¥162,430</div><div className="text-[11px] text-muted-foreground">+14.48%</div></div></div>
      <div className="flex items-center gap-2 border-b py-3 text-xs"><Bank size={17}/><span>只读 · 2 个账户 · 刚刚同步</span><PortfolioDataDialog><Button variant="ghost" size="sm" className="ml-auto h-7 px-2">管理数据 <ArrowRight/></Button></PortfolioDataDialog></div>
      <button onClick={()=>navigate("/app/agent?company=TSM")} className="flex w-full items-start gap-3 border-b py-4 text-left"><span className="grid size-10 shrink-0 place-items-center rounded-full bg-foreground text-background"><Sparkle size={19} weight="fill"/></span><span className="min-w-0 flex-1"><span className="text-[11px] text-muted-foreground">研究信号</span><span className="mt-0.5 block text-sm font-semibold">TSMC 财报还有 5 天 · 2 个问题待补充</span><span className="mt-1 block truncate text-xs text-muted-foreground">HBM 需求指引与先进制程毛利率是关键。</span></span><ArrowRight className="mt-4 shrink-0"/></button>

      {view==="holdings"&&<div><div className="flex items-center gap-1 border-b py-2"><Button variant="ghost" size="sm" className="px-2"><Stack/>全部账户 <CaretDown/></Button><Button variant="ghost" size="sm" className="px-2"><ArrowsDownUp/>市值</Button><Button variant="ghost" size="icon" className="ml-auto" aria-label="搜索持仓"><MagnifyingGlass/></Button><Button variant="ghost" size="icon" aria-label="筛选"><SlidersHorizontal/></Button></div><div className="grid grid-cols-[minmax(0,1fr)_64px_58px_58px_38px] gap-1.5 border-b py-2 text-[9px] text-muted-foreground"><span>名称 / 代码</span><span className="text-right">市值 / 权重</span><span className="text-right">今日</span><span className="text-right">累计</span><span className="text-right">30日</span></div>{holdings.map((item)=><MobilePositionRow key={item.ticker} item={item} onOpen={()=>navigate(`/app/invest/company/${item.ticker}`)}/>) }<div className="grid grid-cols-[1fr_auto] gap-3 border-b py-2 text-xs"><span>持仓合计（5）</span><span className="text-right font-medium">¥955,569 · +¥19,396</span></div><div className="grid grid-cols-[1fr_auto] gap-3 py-2 text-xs"><span>现金</span><span className="text-right font-medium">¥98,420 · 7.7%</span></div></div>}
      {view==="assets"&&<div className="py-5"><div className="grid grid-cols-2 gap-3"><div className="rounded-xl border p-4"><div className="text-xs text-muted-foreground">IBKR 主账户</div><div className="mt-2 text-lg font-semibold">¥1,086,200</div><div className="mt-1 text-xs text-muted-foreground">持仓 14 · 现金 6.2%</div></div><div className="rounded-xl border p-4"><div className="text-xs text-muted-foreground">富途证券</div><div className="mt-2 text-lg font-semibold">¥198,420</div><div className="mt-1 text-xs text-muted-foreground">持仓 4 · 现金 15.8%</div></div></div><div className="mt-5 text-sm font-semibold">资产结构</div><div className="mt-3 space-y-4">{[["AI 芯片与半导体",68],["云软件",18],["现金",8],["其他",6]].map(([label,value])=><div key={label}><div className="mb-1 flex justify-between text-xs"><span>{label}</span><b>{value}%</b></div><Progress value={value}/></div>)}</div></div>}
      {view==="performance"&&<div className="py-5"><div className="flex items-center justify-between"><div><div className="text-xs text-muted-foreground">年初至今</div><div className="mt-1 text-2xl font-semibold">+14.48%</div></div><Badge variant="secondary">超越基准 +5.51%</Badge></div><div className="mt-5 h-52"><ResponsiveContainer width="100%" height="100%"><LineChart data={performanceData}><CartesianGrid vertical={false} strokeDasharray="3 3"/><XAxis dataKey="month" tickLine={false} axisLine={false}/><YAxis tickFormatter={(value)=>`${value}%`} tickLine={false} axisLine={false}/><Line type="monotone" dataKey="portfolio" stroke="var(--foreground)" strokeWidth={2} dot={false}/><Line type="monotone" dataKey="benchmark" stroke="var(--muted-foreground)" strokeWidth={1} dot={false}/></LineChart></ResponsiveContainer></div><div className="mt-5 grid grid-cols-3 gap-2">{[["已实现","+¥28,920"],["未实现","+¥133,510"],["最大回撤","-8.6%"]].map(([label,value])=><div key={label} className="rounded-xl border p-3"><div className="text-[11px] text-muted-foreground">{label}</div><div className="mt-1 text-sm font-semibold">{value}</div></div>)}</div></div>}
      <div className="flex items-center border-t py-2 text-[10px] text-muted-foreground"><span>数据截至 2026-07-12 16:00 · 延迟行情</span><PortfolioDataDialog><Button variant="ghost" size="sm" className="ml-auto h-7 px-2 text-foreground"><CloudArrowDown/>同步 / 导入</Button></PortfolioDataDialog></div>
    </div>
  </div>;
}

function DesktopInvestmentPage() {
  const navigate = useNavigate();
  return <div className="mx-auto hidden max-w-[1500px] px-6 py-6 lg:block">
    <div className="mb-5 flex items-end justify-between"><div><h1 className="text-2xl font-semibold tracking-tight">欢迎回来，老王</h1><p className="mt-1 text-sm text-muted-foreground">数据截至 2026-07-12 16:00 · 延迟行情</p></div><div className="flex items-center gap-3"><div className="text-right text-xs"><div className="font-medium">只读 · 2 个账户</div><div className="mt-1 text-muted-foreground">刚刚同步</div></div><PortfolioDataDialog><Button variant="outline"><CloudArrowDown/>管理持仓数据</Button></PortfolioDataDialog></div></div>
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_300px]"><div className="min-w-0 space-y-4">
      <Card className="shadow-none"><CardContent className="grid gap-6 p-5 xl:grid-cols-[1.4fr_minmax(0,2.2fr)] xl:items-start"><div><div className="text-sm font-medium">组合总览</div><div className="mt-3 text-3xl font-semibold tracking-tight">¥1,284,620</div><div className="mt-3 flex gap-5 text-sm"><span>今日 <b>+¥22,620</b>　+1.8%</span><span className="text-muted-foreground">累计 +¥162,430 · +14.48%</span></div></div><div className="grid grid-cols-4 gap-x-5 gap-y-6"><Metric label="持仓公司" value="18"/><Metric label="现金" value="¥98,420" helper="7.7%"/><Metric label="最大回撤" value="-8.6%" helper="2026-04-07"/><Metric label="Sharpe（年化）" value="1.32"/></div></CardContent></Card>
      <Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm">组合收益率</CardTitle><Button variant="outline" size="sm">日频 <CaretDown/></Button></CardHeader><CardContent><Tabs defaultValue="ytd" className="mb-3"><TabsList className="h-8"><TabsTrigger value="1m">1M</TabsTrigger><TabsTrigger value="3m">3M</TabsTrigger><TabsTrigger value="ytd">YTD</TabsTrigger><TabsTrigger value="1y">1Y</TabsTrigger></TabsList></Tabs><ChartContainer config={chartConfig} className="h-[310px] w-full"><LineChart data={performanceData} margin={{left:-18,right:18,top:8}}><CartesianGrid vertical={false} strokeDasharray="3 3"/><XAxis dataKey="month" tickLine={false} axisLine={false} tickMargin={10}/><YAxis tickFormatter={(value)=>`${value}%`} tickLine={false} axisLine={false}/><ChartTooltip content={<ChartTooltipContent/>}/><Line type="monotone" dataKey="portfolio" stroke="var(--foreground)" strokeWidth={2.2} dot={false}/><Line type="monotone" dataKey="benchmark" stroke="var(--muted-foreground)" strokeWidth={1.5} dot={false}/></LineChart></ChartContainer><div className="flex gap-5 text-xs text-muted-foreground"><span>我的组合 +14.48%</span><span>纳斯达克100 +8.97%</span></div></CardContent></Card>
      <Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0"><div><CardTitle className="text-sm">持仓明细</CardTitle><div className="mt-1 text-xs text-muted-foreground">共 18 项持仓 · 按市值排序</div></div><div className="flex gap-2"><Button variant="ghost" size="sm">全部账户 <CaretDown/></Button><PortfolioDataDialog><Button variant="outline" size="sm"><CloudArrowDown/>同步 / 导入</Button></PortfolioDataDialog></div></CardHeader><CardContent className="px-0 pb-2"><Table><TableHeader><TableRow><TableHead className="pl-6">名称</TableHead><TableHead>数量</TableHead><TableHead>仓位/市值</TableHead><TableHead>今日盈亏</TableHead><TableHead>累计盈亏</TableHead><TableHead>成本/最新价</TableHead><TableHead className="pr-6">数据源</TableHead></TableRow></TableHeader><TableBody>{holdings.map((item)=><TableRow key={item.ticker} className="cursor-pointer" onClick={()=>navigate(`/app/invest/company/${item.ticker}`)}><TableCell className="pl-6"><div className="font-medium">{item.name}</div><div className="text-xs text-muted-foreground">{item.ticker}</div></TableCell><TableCell>{item.quantity}</TableCell><TableCell><div>{item.weight}</div><div className="text-xs text-muted-foreground">{item.value}</div></TableCell><TableCell><div>{item.todayAmount}</div><div className="text-xs text-muted-foreground">{item.today}</div></TableCell><TableCell><div>{item.ytdAmount}</div><div className="text-xs text-muted-foreground">{item.ytd}</div></TableCell><TableCell><div>{item.cost}</div><div className="text-xs text-muted-foreground">{item.price}</div></TableCell><TableCell className="pr-6"><Badge variant="secondary">IBKR · 已同步</Badge></TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
      <Card className="border-foreground shadow-none"><CardContent className="flex items-center gap-3 p-4"><Sparkle size={22} weight="duotone"/><div className="flex-1"><div className="text-sm font-medium">让 Agent 解释组合变化</div><div className="mt-1 text-xs text-muted-foreground">基于持仓、公司事件、老王洞察和历史主线</div></div><Button onClick={()=>navigate("/app/agent")}>询问 Agent <ArrowRight/></Button></CardContent></Card>
    </div><aside className="space-y-4"><Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0"><CardTitle className="text-sm">老王洞察 · 最新</CardTitle><Button asChild variant="ghost" size="sm"><Link to="/app/insights">全部</Link></Button></CardHeader><CardContent><h3 className="text-lg font-semibold leading-7">{insights[0].title}</h3><p className="mt-3 text-sm leading-6 text-muted-foreground">{insights[0].excerpt}</p><Button asChild variant="outline" className="mt-4 w-full"><Link to="/app/insights/hbm-moat">阅读全文</Link></Button></CardContent></Card><Card className="shadow-none"><CardHeader className="flex-row items-center justify-between space-y-0"><CardTitle className="text-sm">今日与近期</CardTitle><CalendarDots/></CardHeader><CardContent className="space-y-5">{trackingEvents.slice(1,3).map((event)=><div key={event.id} className="border-l pl-4"><div className="text-xs text-muted-foreground">{event.date} · {event.time}</div><div className="mt-1 text-sm font-medium">{event.title}</div><div className="mt-1 text-xs text-muted-foreground">{event.meta}</div></div>)}<Button asChild variant="outline" className="w-full"><Link to="/app/tracking">查看跟踪</Link></Button></CardContent></Card><Card className="shadow-none"><CardHeader><CardTitle className="text-sm">数据源</CardTitle></CardHeader><CardContent className="space-y-4">{[["Interactive Brokers","刚刚同步",true],["富途证券","今天 15:58",true]].map(([name,time,ok])=><div key={name} className="flex items-start gap-3"><CheckCircle className="mt-0.5" weight="fill"/><div><div className="text-sm font-medium">{name}</div><div className="mt-1 text-xs text-muted-foreground">只读 · {time}</div></div></div>)}<PortfolioDataDialog><Button variant="outline" className="w-full"><Bank/>管理账户连接</Button></PortfolioDataDialog></CardContent></Card><Card className="shadow-none"><CardHeader><CardTitle className="text-sm">组合风险</CardTitle></CardHeader><CardContent className="space-y-4"><div><div className="mb-2 flex justify-between text-xs"><span>前三大持仓</span><b>56.1%</b></div><Progress value={56}/></div><div className="flex gap-2 rounded-lg border p-3"><WarningCircle className="mt-0.5 shrink-0"/><p className="text-xs leading-5 text-muted-foreground">AI 基础设施暴露 68.4%，需关注同向产业周期风险。</p></div></CardContent></Card></aside></div>
  </div>;
}

export function InvestmentPage() {
  const [isDesktop, setIsDesktop] = useState(()=>window.matchMedia("(min-width: 1024px)").matches);
  useEffect(()=>{
    const media = window.matchMedia("(min-width: 1024px)");
    const update = ()=>setIsDesktop(media.matches);
    media.addEventListener("change",update);
    return ()=>media.removeEventListener("change",update);
  },[]);
  return isDesktop?<DesktopInvestmentPage/>:<MobileInvestmentPage/>;
}
