import { Link } from "react-router-dom";
import { ArrowRight, Brain, CalendarDots, ChartLineUp, FileText, ShieldCheck } from "@phosphor-icons/react";
import { Brand } from "@/components/Brand";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

const modules = [
  { icon: ChartLineUp, title: "投资", text: "在一个研究桌里理解组合、公司与市场，而不是完成交易。" },
  { icon: FileText, title: "洞察", text: "把老王的短观点与深度解读沉淀成可检索的研究资产。" },
  { icon: Brain, title: "Agent", text: "带着你的持仓、主线和当前页面上下文继续研究。" },
  { icon: CalendarDots, title: "跟踪", text: "用今日、日历与任务管理真正值得关注的变化。" },
];

export function MarketingHome() {
  return (
    <div className="min-h-svh bg-background text-foreground">
      <header className="sticky top-0 z-30 border-b bg-background/95 backdrop-blur">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-5 lg:px-8">
          <Brand />
          <nav className="hidden items-center gap-7 text-sm text-muted-foreground md:flex">
            <a href="#product" className="hover:text-foreground">产品</a>
            <a href="#workflow" className="hover:text-foreground">研究闭环</a>
            <a href="#principles" className="hover:text-foreground">原则</a>
          </nav>
          <Button asChild size="sm"><Link to="/app/invest">进入 Live Demo <ArrowRight /></Link></Button>
        </div>
      </header>

      <main>
        <section className="mx-auto grid max-w-7xl gap-12 px-5 pb-16 pt-20 lg:grid-cols-[0.8fr_1.2fr] lg:px-8 lg:pb-24 lg:pt-28">
          <div className="flex flex-col justify-center">
            <Badge variant="outline" className="mb-5 w-fit">AI 投资研究工作台</Badge>
            <h1 className="max-w-xl text-4xl font-semibold tracking-[-0.045em] sm:text-5xl lg:text-6xl lg:leading-[1.05]">
              把数据、洞察与 Agent，连接成持续的投资研究。
            </h1>
            <p className="mt-6 max-w-lg text-base leading-7 text-muted-foreground sm:text-lg">
              HONE 面向长期科技投资者。它不替你交易，而是帮助你理解公司、维护投资主线，并及时关注真正改变判断的事情。
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Button asChild size="lg"><Link to="/app/invest">体验产品 <ArrowRight /></Link></Button>
              <Button asChild variant="outline" size="lg"><a href="#product">了解产品结构</a></Button>
            </div>
            <div className="mt-10 flex items-center gap-3 text-sm text-muted-foreground">
              <ShieldCheck size={20} className="text-foreground" /> 不提供交易 · 不承诺收益 · 重要操作由你确认
            </div>
          </div>
          <div className="overflow-hidden rounded-2xl border bg-muted/30 p-2 shadow-sm">
            <img src="/product-preview.png" alt="HONE 投资研究桌面端预览" className="h-full w-full rounded-xl border object-cover object-left-top" />
          </div>
        </section>

        <Separator />

        <section id="product" className="mx-auto max-w-7xl px-5 py-20 lg:px-8 lg:py-28">
          <div className="max-w-2xl">
            <p className="text-sm font-medium text-muted-foreground">一个产品，四个研究动作</p>
            <h2 className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">从“发生了什么”走到“接下来关注什么”</h2>
          </div>
          <div className="mt-12 grid gap-px overflow-hidden rounded-2xl border bg-border md:grid-cols-2 lg:grid-cols-4">
            {modules.map(({ icon: Icon, title, text }) => (
              <Card key={title} className="rounded-none border-0 shadow-none">
                <CardContent className="p-7">
                  <Icon size={26} weight="duotone" />
                  <h3 className="mt-8 text-lg font-semibold">{title}</h3>
                  <p className="mt-2 text-sm leading-6 text-muted-foreground">{text}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </section>

        <section id="workflow" className="border-y bg-muted/25">
          <div className="mx-auto grid max-w-7xl gap-12 px-5 py-20 lg:grid-cols-2 lg:px-8 lg:py-28">
            <div>
              <p className="text-sm font-medium text-muted-foreground">围绕公司持续积累</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">研究不会在一次聊天后消失</h2>
              <p className="mt-5 max-w-lg leading-7 text-muted-foreground">
                公司连接行情、财务、老王洞察、你的持仓与主线；Agent 的结论可以继续形成跟踪任务，让下一次变化重新回到你的研究链路。
              </p>
            </div>
            <ol className="space-y-0 border-l">
              {["查看持仓与自选", "发现财报、观点或公司事件", "让 Agent 结合上下文研究", "更新主线并建立跟踪"].map((item, index) => (
                <li key={item} className="relative border-b py-5 pl-8 text-sm last:border-b-0">
                  <span className="absolute -left-3 top-4 grid size-6 place-items-center rounded-full border bg-background text-xs">{index + 1}</span>
                  {item}
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section id="principles" className="mx-auto max-w-7xl px-5 py-20 lg:px-8 lg:py-28">
          <div className="grid gap-10 lg:grid-cols-[0.7fr_1.3fr]">
            <h2 className="text-3xl font-semibold tracking-tight">投资纪律，比聊天的迎合更重要。</h2>
            <div className="grid gap-8 sm:grid-cols-3">
              <div><h3 className="font-semibold">来源透明</h3><p className="mt-2 text-sm leading-6 text-muted-foreground">时间敏感的结论展示来源与数据时间。</p></div>
              <div><h3 className="font-semibold">用户掌控</h3><p className="mt-2 text-sm leading-6 text-muted-foreground">修改持仓、主线和任务前都需要确认。</p></div>
              <div><h3 className="font-semibold">长期记忆</h3><p className="mt-2 text-sm leading-6 text-muted-foreground">研究结果围绕公司与用户上下文持续积累。</p></div>
            </div>
          </div>
        </section>

        <section className="border-t">
          <div className="mx-auto flex max-w-7xl flex-col items-start justify-between gap-6 px-5 py-14 sm:flex-row sm:items-center lg:px-8">
            <div><h2 className="text-2xl font-semibold">现在进入 HONE 研究桌</h2><p className="mt-2 text-sm text-muted-foreground">这是基于真实产品需求构建的交互式 Live Demo。</p></div>
            <Button asChild size="lg"><Link to="/app/invest">开始体验 <ArrowRight /></Link></Button>
          </div>
        </section>
      </main>
      <footer className="border-t py-8 text-center text-xs text-muted-foreground">HONE Live Demo · 2026 · 仅用于产品设计验证</footer>
    </div>
  );
}
