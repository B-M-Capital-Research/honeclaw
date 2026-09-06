import { useNavigate } from "react-router-dom";
import { ArrowLeft, BookmarkSimple, ShareNetwork, Sparkle } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

export function InsightArticlePage() {
  const navigate = useNavigate();
  return (
    <div className="mx-auto max-w-5xl px-4 py-6 sm:px-6 lg:py-10">
      <Button variant="ghost" size="sm" onClick={()=>navigate(-1)} className="-ml-2"><ArrowLeft /> 返回洞察</Button>
      <article className="mx-auto mt-8 max-w-[760px]">
        <div className="flex flex-wrap items-center gap-2"><Badge>深度解读</Badge><Badge variant="outline">NVIDIA</Badge><Badge variant="outline">AMD</Badge><span className="text-xs text-muted-foreground">2026-07-12 08:30 · 8 分钟</span></div>
        <h1 className="mt-6 text-4xl font-semibold leading-[1.12] tracking-[-0.04em] sm:text-5xl">从 HBM 竞赛看 GPU 代际后的护城河</h1>
        <p className="mt-6 text-lg leading-8 text-muted-foreground">HBM4 进入量产窗口，显存带宽与堆叠能力成为下一代 GPU 性能跃迁的关键变量。真正的优势不再只体现为芯片设计，而在于与存储生态的深度绑定。</p>
        <div className="mt-7 flex items-center justify-between border-y py-4"><div><div className="text-sm font-medium">老王</div><div className="text-xs text-muted-foreground">巴芒科技 · 已由编辑审核</div></div><div className="flex gap-2"><Button variant="ghost" size="icon"><BookmarkSimple /></Button><Button variant="ghost" size="icon"><ShareNetwork /></Button></div></div>
        <div className="prose-hone mt-10">
          <h2>供给瓶颈正在从单点变成系统协同</h2>
          <p>过去讨论 GPU 供给，市场通常只看晶圆产能。进入下一代产品周期后，真正的交付上限由先进制程、CoWoS 封装、HBM 产能和系统验证共同决定。任何一个环节延后，都会让最终可交付算力低于芯片设计层面的理论值。</p>
          <p>这意味着，领先者的优势不只是“更快的芯片”，而是更早锁定产能、更深参与供应链协同，并让客户的软件与系统围绕自己的节奏完成适配。</p>
          <div className="my-8 border-l-2 border-foreground bg-muted/40 p-5 text-base leading-7"><b>核心判断：</b>未来两代 GPU 的竞争，本质是芯片、存储、封装、网络和软件共同构成的系统交付能力。</div>
          <h2>NVIDIA 与 AMD 的差异正在重新定义</h2>
          <p>NVIDIA 依托 CUDA、NVLink 和完整系统方案，仍然拥有最强的平台协同。AMD 的机会则不在于完全复制 NVIDIA，而是在推理侧和成本敏感场景中成为可信的第二选择。</p>
          <ul><li>NVIDIA：重点观察 Blackwell Ultra、GB300 与网络产品的协同交付。</li><li>AMD：重点观察 MI350 的软件成熟度与云厂商实际部署规模。</li><li>台积电与存储厂商：重点观察先进封装和 HBM4 的扩产兑现。</li></ul>
          <h2>对长期投资者意味着什么</h2>
          <p>短期价格会围绕财报预期波动，但长期主线需要关注的是：系统交付能力是否继续增强，客户迁移成本是否仍然存在，以及竞争者是否真正缩小了软件和网络层面的差距。</p>
        </div>
        <Separator className="my-10" />
        <div className="rounded-2xl border p-6"><div className="flex items-center gap-2"><Sparkle size={22} weight="duotone"/><h2 className="font-semibold">把这篇洞察带进 Agent</h2></div><p className="mt-2 text-sm text-muted-foreground">Agent 会同时参考文章、你的持仓和相关公司画像。</p><Button className="mt-5" onClick={()=>navigate("/app/agent?insight=hbm-moat")}>基于本文继续研究</Button></div>
      </article>
    </div>
  );
}
