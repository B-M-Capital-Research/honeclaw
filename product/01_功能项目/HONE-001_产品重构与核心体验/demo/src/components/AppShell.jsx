import { useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import {
  Bell,
  BellSlash,
  CaretDown,
  ChartPieSlice,
  ClockCounterClockwise,
  Gear,
  Question,
  Lightbulb,
  MagnifyingGlass,
  SignOut,
  Sparkle,
  TrendUp,
  Translate,
  UserCircle,
  Moon,
  Plus,
} from "@phosphor-icons/react";
import { Brand } from "@/components/Brand";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

const mobileNavItems = [
  { to: "/app/invest", label: "投资", icon: ChartPieSlice },
  { to: "/app/insights", label: "洞察", icon: Lightbulb },
  { to: "/app/agent", label: "Agent", icon: Sparkle, agent: true },
  { to: "/app/tracking", label: "跟踪", icon: TrendUp },
  { to: "/app/me", label: "我的", icon: UserCircle },
];

const workspaceNavItems = [
  { to: "/app/invest", label: "投资", icon: ChartPieSlice },
  { to: "/app/insights", label: "洞察", icon: Lightbulb },
  { to: "/app/tracking", label: "跟踪", icon: TrendUp },
];

const researchConversations = [
  { id: "portfolio-move", label: "组合今日上涨原因", group: "今天" },
  { id: "nvda-amd", label: "NVDA 与 AMD 推理侧对比", group: "今天" },
  { id: "tsmc-earnings", label: "TSMC 财报问题清单", group: "今天" },
  { id: "ai-supply-chain", label: "AI 芯片供应链跟踪", group: "过去 7 天" },
  { id: "msft-capex", label: "Microsoft 资本开支复盘", group: "过去 7 天" },
];

function SidebarAccountMenu() {
  const navigate = useNavigate();
  const location = useLocation();
  const [quietMode, setQuietMode] = useState(true);
  const [darkMode, setDarkMode] = useState(false);
  const [language, setLanguage] = useState("zh");
  const isProfile = location.pathname.startsWith("/app/me");
  const toggleDark = (value) => {
    setDarkMode(value);
    document.documentElement.classList.toggle("dark", value);
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          className={cn(
            "h-12 w-full justify-start gap-3 rounded-xl px-2.5",
            isProfile && "bg-muted ring-1 ring-border",
          )}
          aria-label="打开账户菜单"
        >
          <Avatar className="size-8">
            <AvatarFallback>王</AvatarFallback>
          </Avatar>
          <span className="min-w-0 flex-1 text-left">
            <span className="block truncate text-sm font-medium">老王</span>
            <span className="block text-[11px] font-normal text-muted-foreground">
              专业版
            </span>
          </span>
          <CaretDown className="text-muted-foreground" size={14} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="right"
        align="end"
        sideOffset={8}
        className="w-72 p-2"
      >
        <DropdownMenuLabel className="px-2 py-2">
          <div className="text-sm font-semibold text-foreground">老王</div>
          <div className="mt-0.5 font-normal">131****2525 · 专业版</div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuLabel className="px-2 py-1.5">快捷设置</DropdownMenuLabel>
        <DropdownMenuCheckboxItem
          className="px-2 py-2.5"
          checked={quietMode}
          onCheckedChange={setQuietMode}
          onSelect={(event) => event.preventDefault()}
        >
          <BellSlash /> 勿扰模式
          <span className="ml-auto mr-5 text-xs text-muted-foreground">
            23:00–07:30
          </span>
        </DropdownMenuCheckboxItem>
        <DropdownMenuCheckboxItem
          className="px-2 py-2.5"
          checked={darkMode}
          onCheckedChange={toggleDark}
          onSelect={(event) => event.preventDefault()}
        >
          <Moon /> 深色模式
        </DropdownMenuCheckboxItem>
        <DropdownMenuLabel className="mt-1 flex items-center gap-2 px-2">
          <Translate /> 界面语言（示意）
        </DropdownMenuLabel>
        <DropdownMenuRadioGroup value={language} onValueChange={setLanguage}>
          <DropdownMenuRadioItem value="zh" className="px-2 py-2">
            简体中文
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="en" className="px-2 py-2">
            English
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="px-2 py-2.5"
          onSelect={() => navigate("/app/me")}
        >
          <UserCircle /> 个人中心
        </DropdownMenuItem>
        <DropdownMenuItem
          className="px-2 py-2.5"
          onSelect={() => navigate("/app/me?section=membership")}
        >
          <Sparkle /> 会员与订阅
        </DropdownMenuItem>
        <DropdownMenuItem
          className="px-2 py-2.5"
          onSelect={() => navigate("/app/me?section=settings")}
        >
          <Gear /> 设置与偏好
        </DropdownMenuItem>
        <DropdownMenuItem className="px-2 py-2.5">
          <Question /> 帮助与反馈
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="px-2 py-2.5 text-muted-foreground">
          <SignOut /> 退出登录
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function DesktopNav() {
  const navigate = useNavigate();
  const location = useLocation();
  const [researchQuery, setResearchQuery] = useState("");
  const activeConversation = new URLSearchParams(location.search).get("conversation");
  const visibleConversations = researchConversations.filter((item) =>
    item.label.toLowerCase().includes(researchQuery.trim().toLowerCase()),
  );

  const startNewResearch = () => {
    setResearchQuery("");
    navigate("/app/agent?new=1");
  };

  return (
    <aside className="fixed inset-y-0 left-0 z-30 hidden w-[248px] flex-col border-r bg-background px-3 py-5 lg:flex">
      <div className="px-2">
        <Brand />
      </div>
      <nav className="mt-10 flex min-h-0 flex-1 flex-col" aria-label="主导航">
        <div>
          <div className="px-2 text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
            工作台
          </div>
          <div className="mt-2 flex flex-col gap-1.5">
            {workspaceNavItems.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                className={({ isActive }) =>
                  cn(
                    "flex h-11 items-center gap-3 rounded-xl px-3 text-sm font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                    isActive &&
                      "bg-muted text-foreground ring-1 ring-border",
                  )
                }
              >
                <Icon size={20} />
                <span>{label}</span>
              </NavLink>
            ))}
          </div>
        </div>

        <div className="mt-6 flex min-h-0 flex-1 flex-col border-t pt-5">
          <div className="px-2 text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
            AI 研究
          </div>
          <Button
            className="mt-2 h-11 w-full justify-start rounded-xl"
            onClick={startNewResearch}
          >
            <Plus size={18} weight="bold" /> 新研究
          </Button>
          <div className="relative mt-3">
            <MagnifyingGlass
              className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
              size={16}
            />
            <Input
              value={researchQuery}
              onChange={(event) => setResearchQuery(event.target.value)}
              className="h-9 rounded-lg pl-9 text-xs"
              placeholder="搜索研究记录"
              aria-label="搜索研究记录"
            />
          </div>

          <div className="mt-5 min-h-0 flex-1 overflow-y-auto pr-1">
            {researchQuery && visibleConversations.length === 0 ? (
              <div className="rounded-lg border border-dashed px-3 py-5 text-center text-xs text-muted-foreground">
                没有匹配的研究记录
              </div>
            ) : (
              ["今天", "过去 7 天"].map((group) => {
                const items = visibleConversations.filter(
                  (item) => item.group === group,
                );
                if (!items.length) return null;
                return (
                  <div key={group} className="mb-4">
                    <div className="px-2 text-[11px] font-medium text-muted-foreground">
                      {group}
                    </div>
                    <div className="mt-1 space-y-0.5">
                      {items.map((item) => (
                        <button
                          key={item.id}
                          type="button"
                          onClick={() =>
                            navigate(`/app/agent?conversation=${item.id}`)
                          }
                          aria-current={
                            activeConversation === item.id ? "page" : undefined
                          }
                          className={cn(
                            "block w-full truncate rounded-lg px-2.5 py-2 text-left text-[13px] transition-colors hover:bg-muted",
                            activeConversation === item.id &&
                              "bg-muted font-medium text-foreground",
                          )}
                          title={item.label}
                        >
                          {item.label}
                        </button>
                      ))}
                    </div>
                  </div>
                );
              })
            )}
            <Button
              variant="ghost"
              size="sm"
              className="h-9 w-full justify-start px-2.5 text-xs text-muted-foreground"
              onClick={() => navigate("/app/agent?history=all")}
            >
              <ClockCounterClockwise size={16} /> 全部研究记录
            </Button>
          </div>
        </div>
      </nav>
      <div className="mt-4 border-t pt-3">
        <SidebarAccountMenu />
      </div>
    </aside>
  );
}

function MobileBottomNav() {
  return (
    <nav
      className="fixed inset-x-0 bottom-0 z-40 grid h-[74px] grid-cols-5 border-t bg-background/95 px-1 pb-[env(safe-area-inset-bottom)] backdrop-blur lg:hidden"
      aria-label="主导航"
    >
      {mobileNavItems.map(({ to, label, icon: Icon, agent }) => (
        <NavLink
          key={to}
          to={to}
          className={({ isActive }) =>
            cn(
              "relative flex flex-col items-center justify-center gap-1 text-[10px] font-medium text-muted-foreground",
              isActive && "text-foreground",
            )
          }
        >
          <span
            className={cn(
              "grid size-8 place-items-center rounded-full",
              agent &&
                "-mt-5 size-12 border-4 border-background bg-foreground text-background shadow-lg",
            )}
          >
            <Icon size={agent ? 23 : 21} weight={agent ? "fill" : "regular"} />
          </span>
          <span>{label}</span>
        </NavLink>
      ))}
    </nav>
  );
}

export function AppShell() {
  const navigate = useNavigate();
  const location = useLocation();
  const isAgent = location.pathname.startsWith("/app/agent");
  const isInvestment = location.pathname.startsWith("/app/invest");
  const isProfile = location.pathname.startsWith("/app/me");
  const [quietMode, setQuietMode] = useState(true);
  const [darkMode, setDarkMode] = useState(false);
  const [language, setLanguage] = useState("zh");
  const toggleDark = (value) => {
    setDarkMode(value);
    document.documentElement.classList.toggle("dark", value);
  };

  return (
    <div className="min-h-svh bg-background text-foreground">
      <DesktopNav />
      <header
        className={cn(
          "sticky top-0 z-20 flex h-16 items-center justify-between border-b bg-background/95 px-4 backdrop-blur lg:ml-[248px] lg:px-6",
          isInvestment && "hidden lg:flex",
        )}
      >
        <div className="lg:hidden">
          <Brand />
        </div>
        <div className="hidden text-sm text-muted-foreground lg:block">
          {isAgent ? "你的投资研究智能体" : "长期研究，理性决策，复利为王。"}
        </div>
        <div className="flex items-center gap-2">
          <div className="relative hidden md:block">
            <MagnifyingGlass
              className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
              size={17}
            />
            <Input
              className="w-[260px] pl-9"
              placeholder="搜索公司、主题或洞察"
            />
          </div>
          <Sheet>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="lg:hidden"
                aria-label="查看提醒"
              >
                <Bell size={20} />
              </Button>
            </SheetTrigger>
            <SheetContent>
              <SheetHeader>
                <SheetTitle>最新提醒</SheetTitle>
              </SheetHeader>
              <div className="mt-6 space-y-5">
                <button
                  onClick={() => navigate("/app/tracking")}
                  className="w-full border-b pb-4 text-left"
                >
                  <div className="text-sm font-medium">TSMC 财报还有 5 天</div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    与你的持仓相关 · 07月17日
                  </div>
                </button>
                <button
                  onClick={() => navigate("/app/insights/hbm-moat")}
                  className="w-full border-b pb-4 text-left"
                >
                  <div className="text-sm font-medium">
                    老王发布了新的 HBM 深度解读
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    8 分钟前
                  </div>
                </button>
              </div>
            </SheetContent>
          </Sheet>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                className={cn(
                  "h-10 gap-2 rounded-full px-1.5 pr-2 lg:hidden",
                  isProfile && "bg-muted ring-1 ring-border",
                )}
                aria-label="打开账户菜单"
              >
                <Avatar className="size-8">
                  <AvatarFallback>王</AvatarFallback>
                </Avatar>
                <CaretDown
                  className="hidden text-muted-foreground sm:block"
                  size={14}
                />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-72 p-2">
              <DropdownMenuLabel className="px-2 py-2">
                <div className="text-sm font-semibold text-foreground">
                  老王
                </div>
                <div className="mt-0.5 font-normal">131****2525 · 专业版</div>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuLabel className="px-2 py-1.5">
                快捷设置
              </DropdownMenuLabel>
              <DropdownMenuCheckboxItem
                className="px-2 py-2.5"
                checked={quietMode}
                onCheckedChange={setQuietMode}
                onSelect={(event) => event.preventDefault()}
              >
                <BellSlash /> 勿扰模式{" "}
                <span className="ml-auto mr-5 text-xs text-muted-foreground">
                  23:00–07:30
                </span>
              </DropdownMenuCheckboxItem>
              <DropdownMenuCheckboxItem
                className="px-2 py-2.5"
                checked={darkMode}
                onCheckedChange={toggleDark}
                onSelect={(event) => event.preventDefault()}
              >
                <Moon /> 深色模式
              </DropdownMenuCheckboxItem>
              <DropdownMenuLabel className="mt-1 flex items-center gap-2 px-2">
                <Translate />
                界面语言（示意）
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={language}
                onValueChange={setLanguage}
              >
                <DropdownMenuRadioItem value="zh" className="px-2 py-2">
                  简体中文
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="en" className="px-2 py-2">
                  English
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="px-2 py-2.5"
                onSelect={() => navigate("/app/me")}
              >
                <UserCircle /> 个人中心
              </DropdownMenuItem>
              <DropdownMenuItem
                className="px-2 py-2.5"
                onSelect={() => navigate("/app/me?section=membership")}
              >
                <Sparkle /> 会员与订阅
              </DropdownMenuItem>
              <DropdownMenuItem
                className="px-2 py-2.5"
                onSelect={() => navigate("/app/me?section=settings")}
              >
                <Gear /> 设置与偏好
              </DropdownMenuItem>
              <DropdownMenuItem className="px-2 py-2.5">
                <Question /> 帮助与反馈
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem className="px-2 py-2.5 text-muted-foreground">
                <SignOut /> 退出登录
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
      <main className="min-w-0 pb-24 lg:ml-[248px] lg:pb-0">
        <Outlet />
      </main>
      <MobileBottomNav />
    </div>
  );
}
