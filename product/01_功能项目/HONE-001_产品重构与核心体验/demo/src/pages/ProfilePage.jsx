import { useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Bell,
  CaretRight,
  Check,
  CheckCircle,
  CreditCard,
  CurrencyCny,
  Database,
  Desktop,
  DeviceMobile,
  DownloadSimple,
  Fingerprint,
  Globe,
  Key,
  Moon,
  ShieldCheck,
  SignOut,
  Sparkle,
  Trash,
  UserCircle,
} from "@phosphor-icons/react";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
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
import { Progress } from "@/components/ui/progress";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";

function SettingRow({ icon: Icon, title, description, children }) {
  return (
    <div className="flex items-center gap-3 py-4 sm:gap-4">
      <span className="grid size-9 shrink-0 place-items-center rounded-full bg-muted">
        <Icon size={18} />
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium">{title}</div>
        <div className="mt-0.5 text-xs leading-5 text-muted-foreground">
          {description}
        </div>
      </div>
      {children}
    </div>
  );
}

function AccountDialog() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">管理账户</Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>账户资料</DialogTitle>
          <DialogDescription>管理公开身份和登录联系方式。</DialogDescription>
        </DialogHeader>
        <div className="space-y-5 py-2">
          <div className="flex items-center gap-4">
            <Avatar className="size-16">
              <AvatarFallback className="text-xl">王</AvatarFallback>
            </Avatar>
            <div>
              <Button variant="outline" size="sm">
                更换头像
              </Button>
              <div className="mt-1 text-xs text-muted-foreground">
                JPG/PNG，最大 5MB
              </div>
            </div>
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium">昵称</label>
            <Input defaultValue="老王" />
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium">登录手机号</label>
            <Input value="131****2525" disabled />
            <p className="mt-2 text-xs text-muted-foreground">
              修改手机号需要短信验证，Demo 暂用原号码演示。
            </p>
          </div>
          <div className="grid grid-cols-2 gap-3 text-xs">
            <div className="rounded-xl bg-muted p-3">
              <div className="text-muted-foreground">账号创建</div>
              <div className="mt-1 font-medium">2025-11-18</div>
            </div>
            <div className="rounded-xl bg-muted p-3">
              <div className="text-muted-foreground">最近登录</div>
              <div className="mt-1 font-medium">今天 16:08</div>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline">取消</Button>
          <Button onClick={() => toast.success("账户资料已保存")}>保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function ResearchPreferencesDialog() {
  const [topics, setTopics] = useState(["AI 基础设施", "半导体", "云计算"]);
  const [evidence, setEvidence] = useState(true);
  const [counter, setCounter] = useState(true);
  const allTopics = [
    "AI 基础设施",
    "半导体",
    "云计算",
    "AI 应用",
    "新能源",
    "生物科技",
  ];
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm" className="mt-4">
          <UserCircle />
          编辑研究偏好
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90svh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>编辑研究偏好</DialogTitle>
          <DialogDescription>
            这些设置影响后续排序和 Agent 回答，不会重写历史研究记录。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-5 py-2">
          <div>
            <label className="mb-2 block text-sm font-medium">投资周期</label>
            <div className="grid grid-cols-3 gap-2">
              {["1–3 年", "3–5 年", "5 年以上"].map((item, index) => (
                <button
                  key={item}
                  className={`rounded-xl border p-3 text-sm ${index === 1 ? "border-foreground bg-muted" : ""}`}
                >
                  {item}
                </button>
              ))}
            </div>
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium">关注主题</label>
            <div className="flex flex-wrap gap-2">
              {allTopics.map((topic) => (
                <button
                  key={topic}
                  onClick={() =>
                    setTopics((prev) =>
                      prev.includes(topic)
                        ? prev.filter((item) => item !== topic)
                        : [...prev, topic],
                    )
                  }
                  className={`rounded-full border px-3 py-1.5 text-sm ${topics.includes(topic) ? "border-foreground bg-foreground text-background" : ""}`}
                >
                  {topic}
                </button>
              ))}
            </div>
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium">
              特别关注的问题
            </label>
            <Textarea defaultValue="关注产业结构、长期护城河、供给约束和证伪条件；避免短期价格预测。" />
          </div>
          <div className="divide-y rounded-xl border px-4">
            <SettingRow
              icon={ShieldCheck}
              title="优先展示证据"
              description="结论前先说明事实、来源与数据时间"
            >
              <Switch checked={evidence} onCheckedChange={setEvidence} />
            </SettingRow>
            <SettingRow
              icon={Sparkle}
              title="主动给出反方观点"
              description="呈现主要风险与可能证伪当前判断的信号"
            >
              <Switch checked={counter} onCheckedChange={setCounter} />
            </SettingRow>
          </div>
          <div className="rounded-xl bg-muted p-4 text-xs leading-5 text-muted-foreground">
            系统识别出的研究画像会继续保留。你的手动偏好用于纠正和补充，而不是覆盖原始研究记录。
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline">恢复默认</Button>
          <Button onClick={() => toast.success("研究偏好已更新")}>
            保存偏好
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function DevicesDialog() {
  const [devices, setDevices] = useState([
    {
      id: 1,
      Icon: Desktop,
      name: "MacBook Pro",
      meta: "macOS · 上海",
      time: "2026-07-12 16:08 · 当前设备",
      current: true,
    },
    {
      id: 2,
      Icon: DeviceMobile,
      name: "iPhone 17 Pro",
      meta: "iOS · 上海",
      time: "2026-07-12 08:31",
      current: false,
    },
  ]);
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost" size="sm">
          管理
          <CaretRight />
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90svh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>设备与登录</DialogTitle>
          <DialogDescription>
            查看最近使用的设备，并结束不再使用的会话。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="divide-y rounded-xl border">
            {devices.map((device) => (
              <div key={device.id} className="flex items-center gap-4 p-4">
                <span className="grid size-10 place-items-center rounded-full bg-muted">
                  <device.Icon />
                </span>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 text-sm font-medium">
                    {device.name}
                    {device.current && <Badge variant="secondary">当前</Badge>}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {device.meta}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {device.time}
                  </div>
                </div>
                {!device.current && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setDevices((prev) =>
                        prev.filter((item) => item.id !== device.id),
                      );
                      toast.success("该设备已退出登录");
                    }}
                  >
                    <SignOut />
                    退出
                  </Button>
                )}
              </div>
            ))}
          </div>
          <Card className="shadow-none">
            <CardHeader>
              <CardTitle className="text-sm">最近安全活动</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              {[
                ["登录成功", "MacBook Pro · 上海", "今天 16:08"],
                ["同步新设备", "iPhone 17 Pro · 上海", "今天 08:31"],
                ["修改通知设置", "Web · 上海", "07月10日"],
              ].map(([title, meta, time]) => (
                <div key={title} className="flex gap-3">
                  <CheckCircle className="mt-0.5" />
                  <div className="flex-1">
                    <div className="font-medium">{title}</div>
                    <div className="text-xs text-muted-foreground">{meta}</div>
                  </div>
                  <div className="text-xs text-muted-foreground">{time}</div>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
        <DialogFooter className="gap-2">
          <Button variant="outline">
            <Key />
            修改密码
          </Button>
          <Button
            variant="outline"
            onClick={() => toast.info("已保留当前设备，其余会话已退出")}
          >
            退出其他设备
          </Button>
          <Button>完成</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function SubscriptionDialog() {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(1);
  const [plan, setPlan] = useState("pro-year");
  const [method, setMethod] = useState("card");
  const plans = [
    {
      id: "free",
      name: "基础版",
      price: "¥0",
      period: "永久免费",
      features: ["公开洞察", "每周 2 次 Agent 研究", "1 个跟踪任务"],
    },
    {
      id: "pro-month",
      name: "专业版月付",
      price: "¥129",
      period: "每月",
      features: ["完整洞察", "每周 12 次 Agent 研究", "12 个跟踪任务"],
    },
    {
      id: "pro-year",
      name: "专业版年付",
      price: "¥1,299",
      period: "每年 · 省 ¥249",
      features: ["专业版全部能力", "跨端同步", "会员社群与直播权益"],
      recommended: true,
    },
  ];
  const selected = plans.find((item) => item.id === plan);
  const reset = () => {
    setOpen(false);
    setStep(1);
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
        <Button>管理订阅 / 升级</Button>
      </DialogTrigger>
      <DialogContent className="max-h-[92svh] overflow-y-auto sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>
            {step === 1
              ? "选择订阅方案"
              : step === 2
                ? "确认订单"
                : step === 3
                  ? "完成支付"
                  : "订阅已生效"}
          </DialogTitle>
          <DialogDescription>
            {step === 1
              ? "选择适合你的研究频率，当前方案会按剩余有效期折算。"
              : step === 2
                ? "确认方案、价格与续费方式。"
                : step === 3
                  ? "这是 Live Demo 支付流程，不会产生真实扣款。"
                  : "新的权益已经更新到你的账户。"}
          </DialogDescription>
        </DialogHeader>
        {step === 1 && (
          <div className="grid gap-3 py-3 md:grid-cols-3">
            {plans.map((item) => (
              <button
                key={item.id}
                onClick={() => setPlan(item.id)}
                className={`relative rounded-2xl border p-5 text-left ${plan === item.id ? "border-foreground ring-1 ring-foreground" : ""}`}
              >
                {item.recommended && (
                  <Badge className="absolute right-3 top-3">推荐</Badge>
                )}
                <div className="text-sm font-semibold">{item.name}</div>
                <div className="mt-4 text-3xl font-semibold">{item.price}</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  {item.period}
                </div>
                <div className="mt-5 space-y-2">
                  {item.features.map((feature) => (
                    <div key={feature} className="flex gap-2 text-xs">
                      <CheckCircle weight="fill" />
                      <span>{feature}</span>
                    </div>
                  ))}
                </div>
              </button>
            ))}
          </div>
        )}
        {step === 2 && (
          <div className="space-y-4 py-3">
            <div className="rounded-xl border p-5">
              <div className="flex items-start justify-between">
                <div>
                  <div className="font-semibold">{selected.name}</div>
                  <div className="mt-1 text-sm text-muted-foreground">
                    订阅立即生效，下次续费前可随时取消。
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-2xl font-semibold">{selected.price}</div>
                  <div className="text-xs text-muted-foreground">
                    {selected.period}
                  </div>
                </div>
              </div>
            </div>
            <div className="rounded-xl bg-muted p-4 text-sm">
              <div className="flex justify-between">
                <span>当前专业版剩余价值抵扣</span>
                <b>-¥836</b>
              </div>
              <div className="mt-3 flex justify-between border-t pt-3">
                <span>本次应付</span>
                <b>¥463</b>
              </div>
            </div>
            <label className="flex items-start gap-3 rounded-xl border p-4 text-sm">
              <input type="checkbox" defaultChecked className="mt-1" />
              <span>
                我同意《会员服务协议》，并确认订阅将于 2028-06-30
                自动续费；可在续费日前取消。
              </span>
            </label>
          </div>
        )}
        {step === 3 && (
          <div className="grid gap-5 py-3 md:grid-cols-[1fr_280px]">
            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-2">
                {[
                  ["card", "银行卡", CreditCard],
                  ["wallet", "移动支付", DeviceMobile],
                ].map(([id, label, Icon]) => (
                  <button
                    key={id}
                    onClick={() => setMethod(id)}
                    className={`rounded-xl border p-4 text-left ${method === id ? "border-foreground bg-muted" : ""}`}
                  >
                    <Icon />
                    <div className="mt-2 text-sm font-medium">{label}</div>
                  </button>
                ))}
              </div>
              {method === "card" ? (
                <div className="space-y-3">
                  <Input placeholder="卡号 0000 0000 0000 0000" />
                  <div className="grid grid-cols-2 gap-3">
                    <Input placeholder="MM / YY" />
                    <Input placeholder="CVV" />
                  </div>
                  <Input placeholder="持卡人姓名" />
                </div>
              ) : (
                <div className="rounded-xl border p-6 text-center">
                  <DeviceMobile className="mx-auto" size={30} />
                  <div className="mt-3 text-sm font-medium">
                    请在手机上确认 ¥463 支付
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    二维码仅作 Demo 示意
                  </div>
                </div>
              )}
            </div>
            <div className="h-fit rounded-xl bg-muted p-4 text-sm">
              <div className="font-semibold">订单摘要</div>
              <div className="mt-4 flex justify-between">
                <span>{selected.name}</span>
                <span>{selected.price}</span>
              </div>
              <div className="mt-2 flex justify-between text-muted-foreground">
                <span>抵扣</span>
                <span>-¥836</span>
              </div>
              <div className="mt-4 flex justify-between border-t pt-4 text-base font-semibold">
                <span>应付</span>
                <span>¥463</span>
              </div>
              <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground">
                <ShieldCheck />
                支付信息加密处理
              </div>
            </div>
          </div>
        )}
        {step === 4 && (
          <div className="py-8 text-center">
            <span className="mx-auto grid size-16 place-items-center rounded-full bg-foreground text-background">
              <Check size={30} />
            </span>
            <h3 className="mt-5 text-xl font-semibold">专业版已续费成功</h3>
            <p className="mx-auto mt-2 max-w-md text-sm leading-6 text-muted-foreground">
              会员有效期已延长至 2028-06-30。完整洞察、Agent
              额度和持续跟踪权益已经更新。
            </p>
            <div className="mx-auto mt-5 grid max-w-md grid-cols-3 gap-2">
              {[
                ["Agent", "12 / 周"],
                ["跟踪", "12 个"],
                ["跨端", "已开通"],
              ].map(([label, value]) => (
                <div key={label} className="rounded-xl bg-muted p-3">
                  <div className="text-xs text-muted-foreground">{label}</div>
                  <div className="mt-1 text-sm font-semibold">{value}</div>
                </div>
              ))}
            </div>
          </div>
        )}
        <DialogFooter className="gap-2">
          {step > 1 && step < 4 && (
            <Button variant="outline" onClick={() => setStep(step - 1)}>
              <ArrowLeft />
              上一步
            </Button>
          )}
          {step === 1 && (
            <Button variant="outline" onClick={reset}>
              取消
            </Button>
          )}
          {step < 3 && (
            <Button
              disabled={plan === "free"}
              onClick={() => setStep(step + 1)}
            >
              继续
              <ArrowRight />
            </Button>
          )}
          {step === 3 && (
            <Button onClick={() => setStep(4)}>
              <CreditCard />
              确认支付 ¥463
            </Button>
          )}
          {step === 4 && <Button onClick={reset}>完成</Button>}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function PrivacyDialog() {
  const [memory, setMemory] = useState(true);
  const [improve, setImprove] = useState(false);
  return (
    <Dialog>
      <DialogTrigger asChild>
        <button className="flex w-full items-center gap-4 px-5 py-4 text-left hover:bg-muted/30 sm:px-6">
          <ShieldCheck size={20} />
          <span className="flex-1">
            <span className="block text-sm font-medium">数据、隐私与安全</span>
            <span className="mt-0.5 block text-xs text-muted-foreground">
              记忆、数据使用、导出与删除
            </span>
          </span>
          <CaretRight />
        </button>
      </DialogTrigger>
      <DialogContent className="max-h-[90svh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>数据、隐私与安全</DialogTitle>
          <DialogDescription>
            决定 HONE 记住什么，以及如何使用你的研究数据。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-5 py-3">
          <Card className="shadow-none">
            <CardContent className="divide-y">
              <SettingRow
                icon={Sparkle}
                title="使用长期记忆"
                description="让 Agent 使用你的持仓、偏好、主线与已保存研究。"
              >
                <Switch checked={memory} onCheckedChange={setMemory} />
              </SettingRow>
              <SettingRow
                icon={Database}
                title="帮助改进 HONE"
                description="仅使用去标识化的产品互动数据。"
              >
                <Switch checked={improve} onCheckedChange={setImprove} />
              </SettingRow>
            </CardContent>
          </Card>
          <div className="space-y-3">
            <button
              onClick={() => toast.success("数据导出已排队，准备好后会通知你")}
              className="flex w-full items-center gap-3 rounded-xl border p-4 text-left"
            >
              <DownloadSimple />
              <span className="flex-1">
                <span className="block text-sm font-medium">导出我的数据</span>
                <span className="text-xs text-muted-foreground">
                  个人资料、持仓、研究、记忆与任务
                </span>
              </span>
              <CaretRight />
            </button>
            <button
              onClick={() => toast.info("危险操作需要再次短信验证")}
              className="flex w-full items-center gap-3 rounded-xl border p-4 text-left"
            >
              <Trash />
              <span className="flex-1">
                <span className="block text-sm font-medium">
                  删除记忆或账户
                </span>
                <span className="text-xs text-muted-foreground">
                  可单独清空记忆，或发起账户删除
                </span>
              </span>
              <CaretRight />
            </button>
          </div>
        </div>
        <DialogFooter>
          <Button>完成</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function ProfilePage() {
  const [quiet, setQuiet] = useState(true);
  const [theme, setTheme] = useState(false);
  const [weekly, setWeekly] = useState(true);
  return (
    <div className="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:py-8">
      <div>
        <h1 className="text-3xl font-semibold tracking-tight">我的</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          账户、会员权益、研究偏好与数据控制
        </p>
      </div>
      <Card className="mt-6 shadow-none">
        <CardContent className="grid gap-5 p-5 md:grid-cols-[1fr_auto] md:items-center md:p-6">
          <div className="flex items-center gap-4">
            <Avatar className="size-14">
              <AvatarFallback className="text-lg">王</AvatarFallback>
            </Avatar>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-xl font-semibold">老王</h2>
                <Badge>专业版</Badge>
              </div>
              <div className="mt-1 text-sm text-muted-foreground">
                131****2525 · 会员有效至 2027-06-30
              </div>
              <div className="mt-2 flex items-center gap-2 text-xs">
                <CheckCircle weight="fill" />
                账户安全状态正常
              </div>
            </div>
          </div>
          <div className="flex gap-2">
            <AccountDialog />
            <SubscriptionDialog />
          </div>
        </CardContent>
      </Card>
      <div className="mt-5 grid gap-5 lg:grid-cols-[1.1fr_0.9fr]">
        <Card className="shadow-none">
          <CardHeader className="flex-row items-center justify-between space-y-0">
            <CardTitle className="text-sm">会员与用量</CardTitle>
            <Badge variant="secondary">本周</Badge>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <div className="flex items-center justify-between text-sm">
                <span>Agent 深度研究</span>
                <b>9 / 12</b>
              </div>
              <Progress value={75} className="mt-2" />
              <div className="mt-1 text-xs text-muted-foreground">
                每周一 00:00 恢复
              </div>
            </div>
            <div>
              <div className="flex items-center justify-between text-sm">
                <span>持续跟踪任务</span>
                <b>3 / 12</b>
              </div>
              <Progress value={25} className="mt-2" />
              <div className="mt-1 text-xs text-muted-foreground">
                可在跟踪中心暂停或替换
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-xl border p-4">
                <div className="text-sm font-medium">完整洞察</div>
                <div className="mt-1 text-xs text-muted-foreground">已开通</div>
              </div>
              <div className="rounded-xl border p-4">
                <div className="text-sm font-medium">跨端同步</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  2 台设备
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-none">
          <CardHeader>
            <CardTitle className="text-sm">你的研究画像</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {[
                "长期科技投资",
                "AI 基础设施",
                "半导体",
                "关注产业结构",
                "不做高频交易",
              ].map((item) => (
                <Badge key={item} variant="secondary">
                  {item}
                </Badge>
              ))}
            </div>
            <div className="mt-6 rounded-xl border p-4">
              <div className="text-sm font-medium">Agent 回答偏好</div>
              <p className="mt-2 text-xs leading-5 text-muted-foreground">
                优先给出证据与反方风险，区分事实、判断与待验证项。
              </p>
              <ResearchPreferencesDialog />
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-none">
          <CardHeader>
            <CardTitle className="text-sm">投资与时间偏好</CardTitle>
          </CardHeader>
          <CardContent className="divide-y">
            <SettingRow
              icon={CurrencyCny}
              title="基础币种"
              description="组合汇总与收益换算"
            >
              <Select defaultValue="cny">
                <SelectTrigger className="w-28">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="cny">人民币</SelectItem>
                  <SelectItem value="usd">美元</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>
            <SettingRow
              icon={Globe}
              title="时区"
              description="日历与任务执行时间"
            >
              <Select defaultValue="sh">
                <SelectTrigger className="w-32">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="sh">上海 GMT+8</SelectItem>
                  <SelectItem value="ny">纽约 EST/EDT</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>
            <SettingRow
              icon={Globe}
              title="语言"
              description="仅示意界面语言配置"
            >
              <Select defaultValue="zh">
                <SelectTrigger className="w-28">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zh">简体中文</SelectItem>
                  <SelectItem value="en">English</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>
          </CardContent>
        </Card>
        <Card className="shadow-none">
          <CardHeader>
            <CardTitle className="text-sm">通知与显示</CardTitle>
          </CardHeader>
          <CardContent className="divide-y">
            <SettingRow
              icon={Bell}
              title="勿扰模式"
              description="23:00–07:30 合并为早间摘要"
            >
              <Switch checked={quiet} onCheckedChange={setQuiet} />
            </SettingRow>
            <SettingRow
              icon={Bell}
              title="每周研究摘要"
              description="周日 18:00 生成组合、洞察与任务摘要"
            >
              <Switch checked={weekly} onCheckedChange={setWeekly} />
            </SettingRow>
            <SettingRow
              icon={Moon}
              title="深色模式"
              description="跟随系统或手动设置"
            >
              <Switch
                checked={theme}
                onCheckedChange={(value) => {
                  setTheme(value);
                  document.documentElement.classList.toggle("dark", value);
                }}
              />
            </SettingRow>
            <SettingRow
              icon={DeviceMobile}
              title="设备与登录"
              description="2 台设备最近使用"
            >
              <DevicesDialog />
            </SettingRow>
          </CardContent>
        </Card>
        <Card className="shadow-none lg:col-span-2">
          <CardContent className="divide-y p-0">
            <PrivacyDialog />
            <button
              onClick={() => toast.info("登录保护已开启：新设备需要短信验证")}
              className="flex w-full items-center gap-4 px-5 py-4 text-left hover:bg-muted/30 sm:px-6"
            >
              <Fingerprint size={20} />
              <span className="flex-1">
                <span className="block text-sm font-medium">
                  安全与登录保护
                </span>
                <span className="mt-0.5 block text-xs text-muted-foreground">
                  新设备验证、异常登录提醒与最近安全活动
                </span>
              </span>
              <CaretRight />
            </button>
            <button
              onClick={() => toast.info("HONE Live Demo v0.4 · SnowDrift")}
              className="flex w-full items-center gap-4 px-5 py-4 text-left hover:bg-muted/30 sm:px-6"
            >
              <DeviceMobile size={20} />
              <span className="flex-1">
                <span className="block text-sm font-medium">关于 HONE</span>
                <span className="mt-0.5 block text-xs text-muted-foreground">
                  Live Demo v0.4 · 服务条款 · 隐私政策
                </span>
              </span>
              <CaretRight />
            </button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
