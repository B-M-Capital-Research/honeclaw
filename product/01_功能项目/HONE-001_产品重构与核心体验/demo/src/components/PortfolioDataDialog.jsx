import { useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Bank,
  Check,
  CheckCircle,
  FileCsv,
  FilePdf,
  ImageSquare,
  LockKey,
  PencilSimple,
  ShieldCheck,
  UploadSimple,
} from "@phosphor-icons/react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { toast } from "sonner";

const methods = [
  { id: "connect", icon: Bank, title: "连接券商账户", desc: "只读同步持仓、余额与交易记录", badge: "最省事" },
  { id: "statement", icon: FileCsv, title: "导入对账单", desc: "支持 CSV、XLSX 或券商 PDF", badge: "推荐首次" },
  { id: "image", icon: ImageSquare, title: "识别持仓截图", desc: "快速建立当前持仓快照", badge: "快速" },
  { id: "manual", icon: PencilSimple, title: "手动快速录入", desc: "只填证券、数量和平均成本", badge: "通用兜底" },
];

function SourceChoice({ method, onChange }) {
  return <div className="grid gap-3 sm:grid-cols-2">{methods.map(({id,icon:Icon,title,desc,badge})=><button key={id} onClick={()=>onChange(id)} className={`rounded-xl border p-4 text-left transition-colors ${method===id?"border-foreground bg-muted":"hover:bg-muted/30"}`}><div className="flex items-start justify-between gap-3"><span className="grid size-9 place-items-center rounded-full bg-background"><Icon size={19}/></span><Badge variant="secondary">{badge}</Badge></div><div className="mt-4 text-sm font-semibold">{title}</div><p className="mt-1 text-xs leading-5 text-muted-foreground">{desc}</p></button>)}</div>;
}

function ConnectStep() {
  return <div className="space-y-4"><div className="rounded-xl border p-4"><div className="flex gap-3"><ShieldCheck className="mt-0.5 shrink-0"/><div><div className="text-sm font-semibold">只读授权</div><p className="mt-1 text-xs leading-5 text-muted-foreground">HONE 只能读取你选择的账户、持仓、余额和交易历史，无法下单、转账或修改券商账户。</p></div></div></div><div><label className="mb-2 block text-sm font-medium">选择券商</label><div className="grid grid-cols-2 gap-2">{["Interactive Brokers","富途证券","Charles Schwab","Fidelity"].map((name,index)=><button key={name} className={`rounded-xl border px-3 py-3 text-left text-sm ${index===0?"border-foreground bg-muted":""}`}>{name}<span className="mt-1 block text-xs text-muted-foreground">{index<2?"支持只读连接":"需验证地区"}</span></button>)}</div></div><div className="flex items-center gap-3 rounded-xl bg-muted p-4 text-xs leading-5"><LockKey size={20}/><span>授权将在券商或安全连接器中完成，HONE 不保存你的券商密码。</span></div></div>;
}

function ImportStep({ image }) {
  return <div className="space-y-4"><button className="flex min-h-40 w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/20 p-6 text-center"><span className="grid size-11 place-items-center rounded-full bg-background"><UploadSimple size={22}/></span><div className="mt-3 text-sm font-semibold">{image?"上传持仓截图":"上传券商对账单"}</div><p className="mt-1 text-xs text-muted-foreground">{image?"PNG、JPG · 可一次上传多张":"CSV、XLSX、PDF · 单文件不超过 20MB"}</p><Badge className="mt-4" variant="secondary">{image?"已识别 5 项持仓":"IBKR_Statement_July.csv"}</Badge></button><div className="grid gap-3 sm:grid-cols-3">{[["识别账户","IBKR 主账户"],["数据日期","2026-07-12"],["基础币种","USD"]].map(([label,value])=><div key={label} className="rounded-xl border p-3"><div className="text-xs text-muted-foreground">{label}</div><div className="mt-1 text-sm font-medium">{value}</div></div>)}</div><p className="text-xs leading-5 text-muted-foreground">下一步会先展示新增、更新、重复和无法识别的数据，不会直接覆盖现有持仓。</p></div>;
}

function ManualStep() {
  return <div className="space-y-4"><div><label className="mb-2 block text-sm font-medium">证券</label><Input defaultValue="NVIDIA (NVDA) · NASDAQ · USD"/></div><div className="grid grid-cols-2 gap-3"><div><label className="mb-2 block text-sm font-medium">持有数量</label><Input defaultValue="402.6" inputMode="decimal"/></div><div><label className="mb-2 block text-sm font-medium">平均成本</label><Input defaultValue="102.43" inputMode="decimal"/></div></div><div className="grid grid-cols-2 gap-3"><div><label className="mb-2 block text-sm font-medium">币种</label><Select defaultValue="usd"><SelectTrigger className="w-full"><SelectValue/></SelectTrigger><SelectContent><SelectItem value="usd">USD</SelectItem><SelectItem value="hkd">HKD</SelectItem><SelectItem value="cny">CNY</SelectItem></SelectContent></Select></div><div><label className="mb-2 block text-sm font-medium">数据日期</label><Input defaultValue="2026-07-12"/></div></div><div className="rounded-xl bg-muted p-4 text-xs leading-5">快照模式可立即计算当前未实现盈亏；历史收益、已实现盈亏和买卖点需要后续补充交易记录。</div></div>;
}

function ReviewStep({ method }) {
  const sourceLabel = methods.find((item)=>item.id===method)?.title;
  return <div className="space-y-4"><div className="flex items-center gap-3 rounded-xl border p-4"><CheckCircle size={24} weight="fill"/><div><div className="text-sm font-semibold">数据已准备好</div><div className="mt-1 text-xs text-muted-foreground">来源：{sourceLabel}</div></div></div><div className="overflow-hidden rounded-xl border"><div className="grid grid-cols-[1fr_auto_auto] gap-3 bg-muted/40 px-4 py-2 text-xs text-muted-foreground"><span>变更</span><span>数量</span><span>处理</span></div>{[["新增持仓",5,"导入"],["更新持仓",3,"合并"],["重复交易",2,"已忽略"],["无法识别",1,"需确认"]].map(([label,count,status])=><div key={label} className="grid grid-cols-[1fr_auto_auto] gap-3 border-t px-4 py-3 text-sm"><span>{label}</span><b>{count}</b><Badge variant={status==="需确认"?"default":"secondary"}>{status}</Badge></div>)}</div><div className="rounded-xl bg-muted p-4 text-xs leading-5">确认后会刷新组合市值与盈亏。本次变更可在「数据源 → 导入记录」中回看或回滚。</div></div>;
}

export function PortfolioDataDialog({ children }) {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(1);
  const [method, setMethod] = useState("connect");
  const reset = () => { setStep(1); setMethod("connect"); };
  const finish = () => { toast.success("持仓数据已更新，组合结果正在重新计算"); setOpen(false); reset(); };
  return <Dialog open={open} onOpenChange={(value)=>{setOpen(value); if(!value) reset();}}><DialogTrigger asChild>{children}</DialogTrigger><DialogContent className="max-h-[94svh] overflow-y-auto sm:max-w-2xl"><DialogHeader><DialogTitle>{step===1?"同步或导入持仓":step===2?methods.find((item)=>item.id===method)?.title:"确认数据变更"}</DialogTitle><DialogDescription>{step===1?"选择最适合你的方式。你可以随时更换或断开数据源。":step===2?"本步只准备数据，不会直接更改你的组合。":"检查本次新增和更新的持仓，再写入组合。"}</DialogDescription></DialogHeader><div className="py-3">{step===1&&<SourceChoice method={method} onChange={setMethod}/>} {step===2&&method==="connect"&&<ConnectStep/>}{step===2&&method==="statement"&&<ImportStep/>}{step===2&&method==="image"&&<ImportStep image/>}{step===2&&method==="manual"&&<ManualStep/>}{step===3&&<ReviewStep method={method}/>}</div><DialogFooter className="gap-2"><Button variant="outline" onClick={()=>step===1?setOpen(false):setStep(step-1)}>{step===1?"取消":<><ArrowLeft/>上一步</>}</Button>{step<3?<Button onClick={()=>setStep(step+1)}>下一步 <ArrowRight/></Button>:<Button onClick={finish}><Check/> 确认更新</Button>}</DialogFooter></DialogContent></Dialog>;
}
