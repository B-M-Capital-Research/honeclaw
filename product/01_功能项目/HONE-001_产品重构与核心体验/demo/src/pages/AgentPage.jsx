import { useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import {
  ArrowRight,
  ArrowSquareOut,
  BookOpenText,
  Buildings,
  CalendarDots,
  ChartLineUp,
  CheckCircle,
  FilePdf,
  PaperPlaneTilt,
  Paperclip,
  SlidersHorizontal,
  Sparkle,
  Target,
} from "@phosphor-icons/react";
import { agentSources } from "@/data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";

const suggestions = [
  { icon: ChartLineUp, title: "解释组合波动", prompt: "解释今天组合上涨的主要原因", meta: "组合 · 今日" },
  { icon: Buildings, title: "比较两家公司", prompt: "比较 NVDA 与 AMD 的推理侧机会", meta: "公司 · 对比" },
  { icon: FilePdf, title: "阅读财报材料", prompt: "帮我读 TSMC 财报并列出需要验证的主线", meta: "材料 · 深度" },
  { icon: Target, title: "建立跟踪计划", prompt: "为 TSMC 财报建立跟踪", meta: "任务 · 持续" },
];

const clues = [
  { label: "持仓变化", title: "NVDA 今日贡献组合收益约 0.77%", detail: "Blackwell 供给预期与 AI 训练需求共振" },
  { label: "即将发生", title: "TSMC Q2 财报还有 5 天", detail: "主线清单尚有 2 个数据点待补充" },
  { label: "新洞察", title: "老王更新了 HBM 深度解读", detail: "已匹配到 NVDA、AMD 与 TSMC 投资主线" },
];

export function AgentPage() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [mode, setMode] = useState("research");
  const [text, setText] = useState("");
  const [sent, setSent] = useState(
    Boolean(
      params.get("company") ||
        params.get("insight") ||
        params.get("conversation"),
    ),
  );
  const contexts = useMemo(() => {
    const items = ["我的组合", "今日事件"];
    if (params.get("company")) items.push(params.get("company"));
    if (params.get("insight")) items.push("当前洞察");
    return items;
  }, [params]);
  const submit = (prompt) => {
    const next = typeof prompt === "string" ? prompt : text;
    if (!next.trim()) return;
    setText(""); setSent(true); toast.success("Agent 已结合当前上下文完成分析");
  };

  useEffect(() => {
    if (params.get("new")) {
      setSent(false);
      setText("");
      return;
    }
    if (params.get("conversation")) setSent(true);
  }, [params]);

  return (
    <div className="mx-auto grid max-w-[1500px] gap-0 xl:grid-cols-[minmax(0,1fr)_300px]">
      <section className="flex min-h-[calc(100svh-64px)] min-w-0 flex-col px-4 py-5 sm:px-6">
        <div className="mx-auto w-full max-w-3xl">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"><div><h1 className="text-2xl font-semibold">Agent</h1><div className="mt-2 flex flex-wrap items-center gap-2 text-xs"><span className="font-medium">正在基于：</span>{contexts.map((item)=><Badge key={item} variant="secondary">{item}</Badge>)}</div></div><Tabs value={mode} onValueChange={setMode}><TabsList><TabsTrigger value="quick">快速问答</TabsTrigger><TabsTrigger value="research">深度研究</TabsTrigger><TabsTrigger value="task">任务</TabsTrigger></TabsList></Tabs></div>
        </div>

        <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col py-6">
          {!sent ? (
            <div>
              <div className="rounded-2xl border bg-muted/20 p-5 sm:p-6"><div className="flex items-center gap-3"><span className="grid size-11 place-items-center rounded-full bg-foreground text-background"><Sparkle size={22} weight="fill"/></span><div><h2 className="text-xl font-semibold">下午好，老王</h2><p className="mt-1 text-sm text-muted-foreground">今天有 3 条值得继续研究的线索。</p></div></div></div>
              <div className="mt-6 flex items-center justify-between"><h3 className="text-sm font-semibold">快速开始</h3><span className="text-xs text-muted-foreground">{mode === "research" ? "会展开来源与推理过程" : "使用当前上下文"}</span></div>
              <div className="mt-3 grid gap-3 sm:grid-cols-2">{suggestions.map(({icon:Icon,title,prompt,meta})=><button key={title} onClick={()=>submit(prompt)} className="rounded-xl border p-4 text-left transition-colors hover:bg-muted/40"><Icon size={21}/><div className="mt-4 font-medium">{title}</div><p className="mt-1 text-sm leading-6 text-muted-foreground">{prompt}</p><div className="mt-3 text-xs text-muted-foreground">{meta}</div></button>)}</div>
              <div className="mt-7 flex items-center justify-between"><h3 className="text-sm font-semibold">今日研究线索</h3><Button variant="ghost" size="sm" onClick={()=>navigate("/app/tracking")}>查看跟踪 <ArrowRight /></Button></div>
              <div className="mt-3 divide-y rounded-xl border">{clues.map((clue)=><button key={clue.title} onClick={()=>submit(clue.title)} className="flex w-full items-start gap-4 p-4 text-left hover:bg-muted/30"><span className="mt-1 size-2 shrink-0 rounded-full bg-foreground"/><span className="flex-1"><span className="text-xs text-muted-foreground">{clue.label}</span><span className="mt-1 block text-sm font-medium">{clue.title}</span><span className="mt-1 block text-xs leading-5 text-muted-foreground">{clue.detail}</span></span><ArrowRight className="mt-4 shrink-0"/></button>)}</div>
            </div>
          ) : (
            <div className="space-y-6">
              <div className="flex justify-end"><div className="max-w-[85%] rounded-2xl rounded-br-md bg-muted px-4 py-3 text-sm">结合我的持仓和老王最近的判断，解释今天组合上涨的主要原因。</div></div>
              <div><div className="mb-3 flex items-center gap-2 text-sm font-medium"><span className="grid size-7 place-items-center rounded-full bg-foreground text-background"><Sparkle size={15}/></span>组合上涨主要由三条主线驱动</div><Card className="shadow-none"><CardContent className="p-5"><ol className="space-y-5">{[["NVIDIA 贡献最大","Blackwell 供给预期与 AI 训练需求增强，组合贡献约 +0.77%。"],["AMD 跟随推理侧走强","MI350 订单与性价比叙事增强，组合贡献约 +0.43%。"],["TSMC 受益财报预期","先进制程利用率维持高位，组合贡献约 +0.32%。"]].map(([title,desc],index)=><li key={title} className="grid grid-cols-[28px_1fr] gap-3"><span className="grid size-7 place-items-center rounded-full bg-foreground text-xs text-background">{index+1}</span><div><div className="font-medium">{title}</div><p className="mt-1 text-sm leading-6 text-muted-foreground">{desc}</p></div></li>)}</ol><div className="mt-6 rounded-lg bg-muted p-4 text-sm"><b>结论：</b>今日上涨由核心持仓共同驱动，但估值抬升快于盈利修正，仍需关注 7 月 17 日 TSMC 财报。</div></CardContent></Card>
                <div className="mt-4"><div className="text-xs font-medium">参考来源（4）</div><div className="mt-2 divide-y rounded-xl border">{agentSources.map((source)=><div key={source.title} className="flex items-center justify-between gap-3 px-3 py-2 text-xs"><span className="truncate">{source.title}</span><span className="shrink-0 text-muted-foreground">{source.time}<ArrowSquareOut className="ml-1 inline"/></span></div>)}</div></div>
                <div className="mt-4 flex flex-wrap gap-2"><Button variant="outline" onClick={()=>toast.success("已保存为组合研究记录")}><CheckCircle /> 保存研究</Button><Button variant="outline" onClick={()=>toast.success("已创建 TSMC 财报跟踪")}><CalendarDots /> 创建跟踪</Button></div>
              </div>
            </div>
          )}
        </div>

        <div className="sticky bottom-[74px] mx-auto w-full max-w-3xl rounded-2xl border bg-background p-2 shadow-sm lg:bottom-3"><Textarea value={text} onChange={(event)=>setText(event.target.value)} className="min-h-[70px] resize-none border-0 shadow-none focus-visible:ring-0" placeholder="向 Agent 提问，粘贴材料，或输入 / 调用指令"/><div className="flex items-center justify-between"><div className="flex"><Button variant="ghost" size="icon" aria-label="添加附件"><Paperclip /></Button><Button variant="ghost" size="icon" aria-label="研究设置"><SlidersHorizontal /></Button><Button variant="ghost" size="sm"><BookOpenText /> 来源</Button></div><Button size="icon" onClick={()=>submit(text)} aria-label="发送"><PaperPlaneTilt weight="fill"/></Button></div></div>
      </section>

      <aside className="hidden min-h-[calc(100svh-64px)] border-l p-4 xl:block">
        <div className="text-sm font-semibold">即将到来的事件</div><div className="mt-3 space-y-3">{[["TSMC 2026 Q2 财报","07月17日 08:00","2 个问题待补充"],["NVIDIA GTC Tokyo","07月24日 09:00","跟踪 Blackwell 与网络路线"]].map(([title,date,meta])=><button key={title} onClick={()=>navigate("/app/tracking")} className="w-full rounded-xl border p-4 text-left hover:bg-muted/30"><div className="text-sm font-medium">{title}</div><div className="mt-1 text-xs text-muted-foreground">{date}</div><div className="mt-3 text-xs">{meta}</div></button>)}</div>
        <div className="mt-6 flex items-center justify-between"><div className="text-sm font-semibold">已保存的研究</div><Button variant="ghost" size="sm">全部</Button></div><div className="mt-3 space-y-3">{[["AI 芯片供应链与先进制程跟踪","07月11日"],["Microsoft AI 投入回报框架","07月08日"]].map(([title,date])=><Card key={title} className="shadow-none"><CardContent className="p-4"><div className="text-sm font-medium">{title}</div><div className="mt-2 text-xs text-muted-foreground">更新于 {date}</div></CardContent></Card>)}</div>
      </aside>
    </div>
  );
}
