import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  ArrowRight,
  Bell,
  CalendarBlank,
  CaretLeft,
  CaretRight,
  Check,
  CheckCircle,
  Clock,
  DotsThree,
  EnvelopeSimple,
  FileText,
  Funnel,
  Pause,
  PencilSimple,
  Play,
  Plus,
  Sparkle,
  Target,
  Trash,
  WarningCircle,
} from "@phosphor-icons/react";
import { trackingEvents } from "@/data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";

const taskSeed = [
  {
    id: "weekly",
    title: "每周持仓复盘",
    rule: "每周日 16:00",
    next: "今天 16:00",
    enabled: true,
    last: "今天 16:02 成功",
    object: "全部持仓",
  },
  {
    id: "tsmc",
    title: "TSMC 财报发布后分析",
    rule: "事件触发",
    next: "07月17日",
    enabled: true,
    last: "等待事件",
    object: "TSMC",
  },
  {
    id: "nvda",
    title: "NVDA 主线证伪检查",
    rule: "每个美股交易日",
    next: "明天 09:30",
    enabled: false,
    last: "07月11日完成",
    object: "NVDA",
  },
  {
    id: "infra",
    title: "AI 基础设施周报",
    rule: "每周一 08:30",
    next: "明天 08:30",
    enabled: true,
    last: "上周成功",
    object: "AI 基础设施",
  },
];

const calendarDays = [
  [29, false],
  [30, false],
  [1, true],
  [2, true],
  [3, true],
  [4, true],
  [5, true],
  [6, true],
  [7, true],
  [8, true],
  [9, true],
  [10, true],
  [11, true],
  [12, true],
  [13, true],
  [14, true],
  [15, true],
  [16, true],
  [17, true],
  [18, true],
  [19, true],
  [20, true],
  [21, true],
  [22, true],
  [23, true],
  [24, true],
  [25, true],
  [26, true],
  [27, true],
  [28, true],
  [29, true],
  [30, true],
  [31, true],
  [1, false],
  [2, false],
];
const calendarEvents = {
  3: [{ title: "美国非农数据", kind: "宏观" }],
  8: [{ title: "台积电月度营收", kind: "数据" }],
  12: [{ title: "本周持仓复盘", kind: "任务" }],
  17: [
    { title: "TSMC Q2 财报", kind: "财报" },
    { title: "Agent 财报分析", kind: "任务" },
  ],
  23: [{ title: "Microsoft Build", kind: "活动" }],
  24: [{ title: "NVIDIA GTC Tokyo", kind: "活动" }],
  29: [{ title: "FOMC 利率决议", kind: "宏观" }],
  31: [{ title: "Microsoft Q4 财报", kind: "财报" }],
};

function NewTrackingDialog() {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(1);
  const [trackType, setTrackType] = useState("company");
  const [trigger, setTrigger] = useState("event");
  const [notify, setNotify] = useState(true);
  const close = () => {
    setOpen(false);
    setStep(1);
  };
  const create = () => {
    toast.success("跟踪已创建，Agent 将在首次执行前检查数据源");
    close();
  };
  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        setOpen(value);
        if (!value) setStep(1);
      }}
    >
      <DialogTrigger asChild>
        <Button>
          <Plus />
          新建跟踪
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90svh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>新建跟踪</DialogTitle>
          <DialogDescription>
            先明确关注对象，再设置触发方式和交付结果。
          </DialogDescription>
        </DialogHeader>
        <div className="mt-2 grid grid-cols-3 gap-2">
          {[
            [1, "对象与目标"],
            [2, "触发与时间"],
            [3, "交付与确认"],
          ].map(([number, label]) => (
            <div
              key={number}
              className={`rounded-lg border px-3 py-2 ${step === number ? "border-foreground bg-muted" : "text-muted-foreground"}`}
            >
              <div className="text-xs">0{number}</div>
              <div className="mt-1 text-xs font-medium sm:text-sm">{label}</div>
            </div>
          ))}
        </div>
        {step === 1 && (
          <div className="space-y-5 py-5">
            <div>
              <label className="mb-2 block text-sm font-medium">跟踪类型</label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  ["company", "公司", "持续观察一家公司"],
                  ["event", "事件", "财报、会议或数据"],
                  ["thesis", "主线", "验证或证伪判断"],
                ].map(([value, title, desc]) => (
                  <button
                    key={value}
                    onClick={() => setTrackType(value)}
                    className={`rounded-xl border p-3 text-left ${trackType === value ? "border-foreground bg-muted" : ""}`}
                  >
                    <Target />
                    <div className="mt-3 text-sm font-medium">{title}</div>
                    <p className="mt-1 text-xs leading-5 text-muted-foreground">
                      {desc}
                    </p>
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium">关注对象</label>
              <Input defaultValue="NVIDIA (NVDA)" />
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium">
                你想持续回答什么？
              </label>
              <Textarea defaultValue="检查 Blackwell 交付、数据中心需求与软件生态是否出现改变长期主线的信号。" />
            </div>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-5 py-5">
            <div>
              <label className="mb-2 block text-sm font-medium">何时触发</label>
              <div className="grid gap-2 sm:grid-cols-3">
                {[
                  ["event", "关键事件后", "财报、公告或重要数据"],
                  ["schedule", "固定时间", "按日、周或月运行"],
                  ["signal", "信号达到条件", "指标或主线变化"],
                ].map(([value, title, desc]) => (
                  <button
                    key={value}
                    onClick={() => setTrigger(value)}
                    className={`rounded-xl border p-3 text-left ${trigger === value ? "border-foreground bg-muted" : ""}`}
                  >
                    <Clock />
                    <div className="mt-3 text-sm font-medium">{title}</div>
                    <p className="mt-1 text-xs leading-5 text-muted-foreground">
                      {desc}
                    </p>
                  </button>
                ))}
              </div>
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label className="mb-2 block text-sm font-medium">
                  执行规则
                </label>
                <Select defaultValue="weekly">
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="weekly">每周日 16:00</SelectItem>
                    <SelectItem value="daily">每个交易日收盘后</SelectItem>
                    <SelectItem value="earnings">财报发布后 30 分钟</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="mb-2 block text-sm font-medium">时区</label>
                <Select defaultValue="sh">
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="sh">上海 GMT+8</SelectItem>
                    <SelectItem value="ny">纽约 EST/EDT</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="rounded-xl border p-4">
              <div className="flex gap-3">
                <CalendarBlank />
                <div>
                  <div className="text-sm font-medium">预计下次执行</div>
                  <p className="mt-1 text-sm text-muted-foreground">
                    2026年7月19日 · 周日 16:00 · 上海时间
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-5 py-5">
            <div className="rounded-xl border p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm font-medium">结构化研究报告</div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    变化摘要、证据、主线影响和下一步
                  </div>
                </div>
                <CheckCircle weight="fill" />
              </div>
            </div>
            <div className="grid gap-2 sm:grid-cols-2">
              <button className="rounded-xl border border-foreground bg-muted p-4 text-left">
                <Sparkle />
                <div className="mt-3 text-sm font-medium">HONE 跟踪中心</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  保留完整历史与来源
                </div>
              </button>
              <button className="rounded-xl border p-4 text-left">
                <EnvelopeSimple />
                <div className="mt-3 text-sm font-medium">邮件摘要</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  只发送结论和链接
                </div>
              </button>
            </div>
            <div className="flex items-center justify-between rounded-xl border p-4">
              <div className="pr-4">
                <div className="text-sm font-medium">重大变化立即提醒</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  普通结果按既定时间交付。
                </div>
              </div>
              <Switch checked={notify} onCheckedChange={setNotify} />
            </div>
            <div className="rounded-xl bg-muted p-4 text-sm leading-6">
              <b>创建后：</b>Agent 会先检查数据源。跟踪不会更改持仓或替你交易。
            </div>
          </div>
        )}
        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={step === 1 ? close : () => setStep(step - 1)}
          >
            {step === 1 ? (
              "取消"
            ) : (
              <>
                <ArrowLeft />
                上一步
              </>
            )}
          </Button>
          {step < 3 ? (
            <Button onClick={() => setStep(step + 1)}>
              下一步
              <ArrowRight />
            </Button>
          ) : (
            <Button onClick={create}>
              <Check />
              确认创建
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function EventDialog({ event, mode = "detail", open, onOpenChange }) {
  const navigate = useNavigate();
  if (!event) return null;
  const isResult = mode === "result" || event.id === 1;
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90svh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Badge variant={isResult ? "default" : "outline"}>
              {isResult ? "结果已生成" : event.state}
            </Badge>
            <span className="text-xs text-muted-foreground">
              {event.date} · {event.time}
            </span>
          </div>
          <DialogTitle className="pt-2 text-xl">{event.title}</DialogTitle>
          <DialogDescription>{event.meta} · 来源已完成去重</DialogDescription>
        </DialogHeader>
        {isResult ? (
          <div className="space-y-4 py-2">
            <div className="rounded-xl border p-4">
              <div className="text-xs text-muted-foreground">Agent 结论</div>
              <h3 className="mt-2 text-lg font-semibold">
                本周组合上涨主要由 NVIDIA 与台积电贡献，核心主线暂未出现证伪信号
              </h3>
              <p className="mt-3 text-sm leading-6 text-muted-foreground">
                AI 基础设施持仓贡献约 76% 的周度收益；组合集中度继续上升，需要把
                Blackwell 交付和先进封装产能作为下周优先验证项。
              </p>
            </div>
            <div className="grid grid-cols-3 gap-2">
              {[
                ["组合周收益", "+3.8%"],
                ["最大贡献", "NVIDIA"],
                ["风险变化", "集中度 ↑"],
              ].map(([label, value]) => (
                <div key={label} className="rounded-xl bg-muted p-3">
                  <div className="text-[11px] text-muted-foreground">
                    {label}
                  </div>
                  <div className="mt-1 text-sm font-semibold">{value}</div>
                </div>
              ))}
            </div>
            <div className="rounded-xl border p-4">
              <div className="text-sm font-semibold">证据与待验证</div>
              <ul className="mt-3 space-y-2 text-sm leading-6 text-muted-foreground">
                <li>• NVIDIA 数据中心收入预期未下修。</li>
                <li>• 台积电先进封装月度扩产节奏符合计划。</li>
                <li>• 待验证：云厂商下半年资本开支是否继续上调。</li>
              </ul>
            </div>
            <div className="text-xs text-muted-foreground">
              生成于 2026-07-12 16:02 · 数据截至 16:00 · 8 个来源
            </div>
          </div>
        ) : (
          <div className="space-y-4 py-2">
            <div className="rounded-xl border p-4">
              <div className="text-xs text-muted-foreground">为什么收到</div>
              <p className="mt-2 text-sm leading-6">
                该事件同时命中你的持仓、TSMC 跟踪任务和“先进封装”研究偏好。
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="rounded-xl bg-muted p-4">
                <div className="text-xs text-muted-foreground">事件状态</div>
                <div className="mt-1 font-semibold">{event.state}</div>
              </div>
              <div className="rounded-xl bg-muted p-4">
                <div className="text-xs text-muted-foreground">准备状态</div>
                <div className="mt-1 font-semibold">4 / 6 个问题已准备</div>
              </div>
            </div>
            <div>
              <div className="text-sm font-semibold">Agent 准备清单</div>
              <div className="mt-3 space-y-2">
                {[
                  "营收与毛利率预期差",
                  "先进制程利用率",
                  "CoWoS 扩产节奏",
                  "HBM 客户需求指引",
                ].map((item, index) => (
                  <div
                    key={item}
                    className="flex items-center gap-3 rounded-lg border p-3 text-sm"
                  >
                    {index < 3 ? <CheckCircle weight="fill" /> : <Clock />}
                    <span>{item}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            关闭
          </Button>
          {isResult ? (
            <>
              <Button
                variant="outline"
                onClick={() => toast.success("报告已保存到研究资料")}
              >
                保存报告
              </Button>
              <Button
                onClick={() => navigate("/app/agent?tracking=weekly-review")}
              >
                <Sparkle />
                继续追问
              </Button>
            </>
          ) : (
            <Button onClick={() => navigate("/app/agent?tracking=tsmc-prep")}>
              <Sparkle />
              继续准备
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function TaskManagerDialog({
  open,
  onOpenChange,
  tasks,
  setTasks,
  initialTask = null,
}) {
  const [selected, setSelected] = useState(initialTask);
  const updateTask = (id, patch) =>
    setTasks((prev) =>
      prev.map((task) => (task.id === id ? { ...task, ...patch } : task)),
    );
  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        onOpenChange(value);
        if (!value) setSelected(null);
      }}
    >
      <DialogContent className="max-h-[92svh] overflow-y-auto sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>
            {selected ? "编辑跟踪任务" : "管理全部任务"}
          </DialogTitle>
          <DialogDescription>
            {selected
              ? "修改只影响下一次执行，历史结果不会改变。"
              : "查看、暂停和调整所有持续跟踪任务。"}
          </DialogDescription>
        </DialogHeader>
        {selected ? (
          <div className="space-y-5 py-3">
            <div>
              <label className="mb-2 block text-sm font-medium">任务名称</label>
              <Input
                value={selected.title}
                onChange={(event) =>
                  setSelected({ ...selected, title: event.target.value })
                }
              />
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium">关注对象</label>
              <Input
                value={selected.object}
                onChange={(event) =>
                  setSelected({ ...selected, object: event.target.value })
                }
              />
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label className="mb-2 block text-sm font-medium">
                  执行规则
                </label>
                <Select defaultValue="weekly">
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="weekly">每周日 16:00</SelectItem>
                    <SelectItem value="daily">每个美股交易日</SelectItem>
                    <SelectItem value="event">事件触发</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="mb-2 block text-sm font-medium">时区</label>
                <Select defaultValue="sh">
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="sh">上海 GMT+8</SelectItem>
                    <SelectItem value="ny">纽约 EST/EDT</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium">研究问题</label>
              <Textarea defaultValue="识别影响长期投资主线的新事实、反方证据与待验证项。" />
            </div>
            <div className="flex items-center justify-between rounded-xl border p-4">
              <div>
                <div className="text-sm font-medium">任务状态</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  下次执行：{selected.next}
                </div>
              </div>
              <Switch
                checked={selected.enabled}
                onCheckedChange={(enabled) =>
                  setSelected({ ...selected, enabled })
                }
              />
            </div>
            <Button
              variant="ghost"
              className="text-muted-foreground"
              onClick={() => {
                setTasks((prev) =>
                  prev.filter((task) => task.id !== selected.id),
                );
                setSelected(null);
                toast.success("任务已删除，历史结果仍会保留");
              }}
            >
              <Trash />
              删除任务
            </Button>
          </div>
        ) : (
          <div className="space-y-3 py-2">
            <div className="flex items-center justify-between">
              <div className="text-xs text-muted-foreground">
                {tasks.filter((task) => task.enabled).length} 个运行中 ·{" "}
                {tasks.length} 个全部任务
              </div>
              <Button variant="outline" size="sm">
                <Funnel />
                筛选
              </Button>
            </div>
            {tasks.map((task) => (
              <div
                key={task.id}
                className="flex flex-col gap-3 rounded-xl border p-4 sm:flex-row sm:items-center"
              >
                <span className="grid size-10 shrink-0 place-items-center rounded-full bg-muted">
                  {task.enabled ? <Play /> : <Pause />}
                </span>
                <div className="min-w-0 flex-1">
                  <div className="font-semibold">{task.title}</div>
                  <div className="mt-1 text-sm text-muted-foreground">
                    {task.object} · {task.rule}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {task.last} · 下次 {task.next}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant={task.enabled ? "secondary" : "outline"}>
                    {task.enabled ? "运行中" : "已暂停"}
                  </Badge>
                  <Switch
                    checked={task.enabled}
                    onCheckedChange={(enabled) =>
                      updateTask(task.id, { enabled })
                    }
                  />
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSelected({ ...task })}
                  >
                    <PencilSimple />
                    编辑
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => (selected ? setSelected(null) : onOpenChange(false))}
          >
            {selected ? "返回任务列表" : "关闭"}
          </Button>
          {selected && (
            <Button
              onClick={() => {
                updateTask(selected.id, selected);
                setSelected(null);
                toast.success("任务设置已保存");
              }}
            >
              保存更改
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function MonthCalendar({ onOpenDay }) {
  const [selected, setSelected] = useState(17);
  return (
    <Card className="overflow-hidden shadow-none">
      <CardHeader className="flex-row items-center justify-between space-y-0 border-b">
        <div>
          <CardTitle className="text-xl">2026 年 7 月</CardTitle>
          <p className="mt-1 text-xs text-muted-foreground">
            上海时间 · 8 个与你相关的事件
          </p>
        </div>
        <div className="flex gap-1">
          <Button variant="outline" size="icon" aria-label="上一个月">
            <CaretLeft />
          </Button>
          <Button variant="outline" size="sm">
            今天
          </Button>
          <Button variant="outline" size="icon" aria-label="下一个月">
            <CaretRight />
          </Button>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div className="grid grid-cols-7 border-b bg-muted/30">
          {["周一", "周二", "周三", "周四", "周五", "周六", "周日"].map(
            (day) => (
              <div
                key={day}
                className="px-1 py-2 text-center text-[10px] text-muted-foreground sm:text-xs"
              >
                {day}
              </div>
            ),
          )}
        </div>
        <div className="grid grid-cols-7">
          {calendarDays.map(([day, current], index) => {
            const events = current ? calendarEvents[day] || [] : [];
            return (
              <button
                key={`${day}-${index}`}
                onClick={() => current && setSelected(day)}
                className={`min-h-[76px] border-b border-r p-1 text-left sm:min-h-[112px] sm:p-2 ${!current ? "bg-muted/20 text-muted-foreground" : selected === day ? "bg-muted/50" : ""}`}
              >
                <span
                  className={`inline-grid size-7 place-items-center rounded-full text-xs ${day === 12 && current ? "bg-foreground text-background" : ""}`}
                >
                  {day}
                </span>
                <div className="mt-1.5 space-y-1">
                  {events.map((event) => (
                    <div key={event.title}>
                      <span className="inline-block size-1.5 rounded-full bg-foreground sm:hidden" />
                      <div className="hidden truncate rounded-md border bg-background px-2 py-1 text-[11px] sm:block">
                        <span className="mr-1 text-muted-foreground">
                          {event.kind}
                        </span>
                        {event.title}
                      </div>
                    </div>
                  ))}
                </div>
              </button>
            );
          })}
        </div>
        <div className="grid gap-3 border-t p-4 sm:grid-cols-[1fr_auto] sm:items-center">
          <div>
            <div className="text-xs text-muted-foreground">
              7 月 {selected} 日
            </div>
            <div className="mt-1 text-sm font-medium">
              {calendarEvents[selected]?.[0]?.title || "没有安排的重要事件"}
            </div>
          </div>
          {calendarEvents[selected] && (
            <Button
              variant="outline"
              onClick={() =>
                onOpenDay({
                  id: 99,
                  date: `07月${selected}日`,
                  time: "全天",
                  title: calendarEvents[selected][0].title,
                  meta: "日历事件 · 与你的关注相关",
                  state: "即将发生",
                  kind: calendarEvents[selected][0].kind,
                })
              }
            >
              查看当日详情
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export function TrackingPage() {
  const [tasks, setTasks] = useState(taskSeed);
  const [eventState, setEventState] = useState(null);
  const [eventMode, setEventMode] = useState("detail");
  const [managerOpen, setManagerOpen] = useState(false);
  const [initialTask, setInitialTask] = useState(null);
  const openEvent = (event, mode = "detail") => {
    setEventState(event);
    setEventMode(mode);
  };
  const openManager = (task = null) => {
    setInitialTask(task);
    setManagerOpen(true);
  };
  return (
    <div className="mx-auto max-w-[1320px] px-4 py-6 sm:px-6 lg:py-7">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm text-muted-foreground">2026年7月12日 · 周日</p>
          <h1 className="mt-1 text-3xl font-semibold tracking-tight">跟踪</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            把即将发生的事、持续验证的主线和 Agent 任务放在一起。
          </p>
        </div>
        <NewTrackingDialog />
      </div>
      <Tabs defaultValue="today" className="mt-7">
        <TabsList>
          <TabsTrigger value="today">今日</TabsTrigger>
          <TabsTrigger value="calendar">日历</TabsTrigger>
          <TabsTrigger value="tasks">任务</TabsTrigger>
          <TabsTrigger value="history">历史</TabsTrigger>
        </TabsList>
        <TabsContent value="today" className="mt-5">
          <div className="grid grid-cols-3 gap-2 sm:gap-3">
            {[
              ["今天待关注", "4", "1 个任务 · 3 个信号"],
              [
                "运行中的跟踪",
                String(tasks.filter((task) => task.enabled).length),
                "1 个任务已暂停",
              ],
              ["本周关键时点", "2", "TSMC · Microsoft"],
            ].map(([label, value, helper]) => (
              <Card key={label} className="shadow-none">
                <CardContent className="p-3 sm:p-4">
                  <div className="text-[11px] text-muted-foreground sm:text-xs">
                    {label}
                  </div>
                  <div className="mt-2 text-2xl font-semibold">{value}</div>
                  <div className="mt-1 text-[10px] leading-4 sm:text-xs">
                    {helper}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
          <div className="mt-4 grid gap-5 lg:grid-cols-[minmax(0,1fr)_340px]">
            <div>
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-semibold">时间线</h2>
                <Button variant="ghost" size="sm">
                  只看持仓相关
                </Button>
              </div>
              <div className="relative space-y-3 before:absolute before:bottom-8 before:left-[29px] before:top-8 before:w-px before:bg-border">
                {trackingEvents.slice(0, 4).map((event, index) => (
                  <Card key={event.id} className="relative shadow-none">
                    <CardContent className="grid gap-4 p-5 sm:grid-cols-[64px_1fr_auto] sm:items-center">
                      <div className="relative z-[1]">
                        <div
                          className={`grid size-7 place-items-center rounded-full border bg-background ${index === 0 ? "bg-foreground text-background" : ""}`}
                        >
                          {index === 0 ? (
                            <Check size={14} />
                          ) : (
                            <span className="size-2 rounded-full bg-foreground" />
                          )}
                        </div>
                        <div className="mt-2 text-xs text-muted-foreground">
                          {event.date}
                        </div>
                        <div className="text-xs font-medium">{event.time}</div>
                      </div>
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <h3 className="font-semibold">{event.title}</h3>
                          <Badge variant={index === 0 ? "default" : "outline"}>
                            {event.kind}
                          </Badge>
                        </div>
                        <p className="mt-1 text-sm text-muted-foreground">
                          {event.meta}
                        </p>
                        {index === 1 && (
                          <div className="mt-3 text-xs">
                            Agent 已准备 4 个财报问题，还有 2 个数据点待补充。
                          </div>
                        )}
                      </div>
                      <div className="flex gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => openEvent(event, "detail")}
                        >
                          详情
                        </Button>
                        {index < 2 && (
                          <Button
                            size="sm"
                            onClick={() =>
                              openEvent(
                                event,
                                index === 0 ? "result" : "detail",
                              )
                            }
                          >
                            <Sparkle />
                            {index === 0 ? "查看结果" : "继续准备"}
                          </Button>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
            <aside className="space-y-4">
              <Card className="shadow-none">
                <CardHeader>
                  <CardTitle className="text-sm">正在运行</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {tasks
                    .filter((task) => task.enabled)
                    .slice(0, 3)
                    .map((task) => (
                      <div key={task.id} className="flex gap-3">
                        <span className="mt-0.5 grid size-7 place-items-center rounded-full bg-muted">
                          <Play size={13} weight="fill" />
                        </span>
                        <div>
                          <div className="text-sm font-medium">
                            {task.title}
                          </div>
                          <div className="mt-1 text-xs text-muted-foreground">
                            下次 {task.next}
                          </div>
                        </div>
                      </div>
                    ))}
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={() => openManager()}
                  >
                    管理全部任务
                  </Button>
                </CardContent>
              </Card>
              <Card className="shadow-none">
                <CardHeader>
                  <CardTitle className="text-sm">本周准备度</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  {[
                    ["TSMC 财报问题", "4 / 6"],
                    ["Microsoft Build 跟踪点", "3 / 3"],
                    ["GTC Tokyo 日程", "2 / 4"],
                  ].map(([title, status]) => (
                    <div
                      key={title}
                      className="flex items-center justify-between text-sm"
                    >
                      <span>{title}</span>
                      <Badge variant="secondary">{status}</Badge>
                    </div>
                  ))}
                </CardContent>
              </Card>
              <Card className="shadow-none">
                <CardContent className="flex gap-3 p-4">
                  <Bell className="mt-0.5" />
                  <div>
                    <div className="text-sm font-medium">
                      通知会按优先级合并
                    </div>
                    <p className="mt-1 text-xs leading-5 text-muted-foreground">
                      勿扰时段 23:00–07:30。普通信号进入早间摘要。
                    </p>
                  </div>
                </CardContent>
              </Card>
            </aside>
          </div>
        </TabsContent>
        <TabsContent value="calendar" className="mt-5">
          <MonthCalendar onOpenDay={(event) => openEvent(event, "detail")} />
        </TabsContent>
        <TabsContent value="tasks" className="mt-5">
          <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_300px]">
            <div className="space-y-3">
              {tasks.map((task) => (
                <Card key={task.id} className="shadow-none">
                  <CardContent className="flex flex-col gap-4 p-5 sm:flex-row sm:items-center">
                    <span className="grid size-10 place-items-center rounded-full bg-muted">
                      {task.enabled ? <Play /> : <Pause />}
                    </span>
                    <div className="flex-1">
                      <div className="font-semibold">{task.title}</div>
                      <div className="mt-1 text-sm text-muted-foreground">
                        {task.rule} · 下次 {task.next}
                      </div>
                      <div className="mt-2 text-xs">{task.last}</div>
                    </div>
                    <div className="flex items-center gap-3">
                      <Badge variant={task.enabled ? "secondary" : "outline"}>
                        {task.enabled ? "运行中" : "已暂停"}
                      </Badge>
                      <Switch
                        checked={task.enabled}
                        onCheckedChange={(enabled) =>
                          setTasks((prev) =>
                            prev.map((item) =>
                              item.id === task.id ? { ...item, enabled } : item,
                            ),
                          )
                        }
                      />
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openManager({ ...task })}
                      >
                        编辑
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
            <Card className="h-fit shadow-none">
              <CardHeader>
                <CardTitle className="text-sm">任务资源</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <div className="text-2xl font-semibold">
                    {tasks.filter((task) => task.enabled).length} / 12
                  </div>
                  <div className="text-xs text-muted-foreground">
                    已启用持续跟踪
                  </div>
                </div>
                <Button
                  variant="outline"
                  className="w-full"
                  onClick={() => openManager()}
                >
                  管理全部任务
                </Button>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
        <TabsContent value="history" className="mt-5">
          <div className="divide-y rounded-xl border">
            {[
              [
                "本周持仓复盘",
                "已生成完整报告",
                "今天 16:02",
                trackingEvents[0],
              ],
              [
                "AI 芯片主线检查",
                "未发现证伪信号",
                "07月11日",
                { ...trackingEvents[0], id: 11, title: "AI 芯片主线检查" },
              ],
              [
                "TSMC 公司事件摘要",
                "新增 2 条可验证信息",
                "07月10日",
                { ...trackingEvents[1], id: 12, title: "TSMC 公司事件摘要" },
              ],
              [
                "Microsoft 云增速复盘",
                "更新了资本开支假设",
                "07月08日",
                { ...trackingEvents[2], id: 13, title: "Microsoft 云增速复盘" },
              ],
            ].map(([title, status, date, event]) => (
              <button
                key={title}
                onClick={() => openEvent(event, "result")}
                className="flex w-full items-center gap-4 p-4 text-left hover:bg-muted/30 sm:p-5"
              >
                <CheckCircle size={22} />
                <span className="flex-1">
                  <span className="block font-medium">{title}</span>
                  <span className="mt-1 block text-sm text-muted-foreground">
                    {status}
                  </span>
                </span>
                <span className="hidden text-xs text-muted-foreground sm:block">
                  {date}
                </span>
                <ArrowRight />
              </button>
            ))}
          </div>
        </TabsContent>
      </Tabs>
      <EventDialog
        event={eventState}
        mode={eventMode}
        open={!!eventState}
        onOpenChange={(open) => !open && setEventState(null)}
      />
      <TaskManagerDialog
        open={managerOpen}
        onOpenChange={setManagerOpen}
        tasks={tasks}
        setTasks={setTasks}
        initialTask={initialTask}
      />
    </div>
  );
}
