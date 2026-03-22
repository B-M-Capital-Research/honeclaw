import {
  createContext,
  createEffect,
  onCleanup,
  useContext,
  type ParentProps,
} from "solid-js"
import { createStore, produce } from "solid-js/store"
import { getResearchStatus, startResearch } from "@/lib/api"
import { readStoredResearchTasks, writeStoredResearchTasks } from "@/lib/persist"
import type { ResearchTask } from "@/lib/types"
import { useBackend } from "./backend"

// ── 类型 ─────────────────────────────────────────────────────────────────────

type ResearchState = {
  tasks: ResearchTask[]
  selectedTaskId: string | null
  /** 正在提交新任务 */
  submitting: boolean
  /** 提交错误信息 */
  submitError: string | null
}

type ResearchContextValue = ReturnType<typeof createResearchState>

// ── Context ───────────────────────────────────────────────────────────────────

const ResearchContext = createContext<ResearchContextValue>()

// ── 状态工厂 ──────────────────────────────────────────────────────────────────

function createResearchState() {
  const backend = useBackend()
  const [state, setState] = createStore<ResearchState>({
    tasks: readStoredResearchTasks(),
    selectedTaskId: null,
    submitting: false,
    submitError: null,
  })

  // 每次 tasks 变化时持久化到 localStorage
  createEffect(() => {
    writeStoredResearchTasks(state.tasks)
  })

  // ── 轮询逻辑 ──────────────────────────────────────────────────────────────

  const pollTask = async (taskId: string) => {
    if (!backend.state.connected || !backend.hasCapability("research")) {
      return
    }
    try {
      const resp = await getResearchStatus(taskId)

      // 映射外部状态到内部状态
      let newStatus: ResearchTask["status"] = "running"
      if (resp.status === "已完成") {
        newStatus = "completed"
      } else if (resp.status === "异常") {
        newStatus = "error"
      }

      setState(
        "tasks",
        (t) => t.task_id === taskId,
        produce((t) => {
          t.status = newStatus
          t.progress = resp.progress
          t.updated_at = resp.updated_at
          if (resp.completed_at) t.completed_at = resp.completed_at
          if (resp.answer_file_path) t.answer_file_path = resp.answer_file_path
          // 完成时直接存储 Markdown 原文
          if (resp.answer_markdown) t.answer_markdown = resp.answer_markdown
        }),
      )
    } catch (err) {
      console.warn("轮询任务失败:", err)
    }
  }

  // 每 10 秒轮询所有 running/pending 状态的任务
  const pollInterval = window.setInterval(async () => {
      const activeTasks = state.tasks.filter(
        (t) => (t.status === "running" || t.status === "pending") && !t.answer_markdown,
      )
    for (const task of activeTasks) {
      await pollTask(task.task_id)
    }
  }, 10_000)

  onCleanup(() => window.clearInterval(pollInterval))

  // ── 公开 API ──────────────────────────────────────────────────────────────

  const startTask = async (companyName: string) => {
    setState("submitting", true)
    setState("submitError", null)
    try {
      // ── TEST 模式：输入 "TEST"（不区分大小写）直接注入预置 Markdown ──────────
      if (companyName.trim().toUpperCase() === "TEST") {
        const mockId = `local-test-${Date.now()}`
        const now = new Date().toISOString()
        const mockTask: ResearchTask = {
          id: mockId,
          task_id: "test-task-id-00000000",
          task_name: "测试公司_0000000000_deepresearch",
          company_name: "测试公司（TEST）",
          status: "completed",
          progress: "100%",
          created_at: now,
          updated_at: now,
          completed_at: now,
          answer_markdown: TEST_MARKDOWN,
        }
        setState("tasks", (prev) => [mockTask, ...prev])
        setState("selectedTaskId", mockId)
        return
      }

      // ── 正常流程 ────────────────────────────────────────────────────────────
      if (!backend.state.connected || !backend.hasCapability("research")) {
        throw new Error("当前 backend 不支持研究能力")
      }
      const resp = await startResearch(companyName)

      const newTask: ResearchTask = {
        id: `local-${Date.now()}-${Math.random().toString(36).slice(2)}`,
        task_id: resp.task_id,
        task_name: resp.task_name,
        company_name: companyName,
        status: "running",
        progress: "0%",
        created_at: new Date().toISOString(),
      }

      setState("tasks", (prev) => [newTask, ...prev])
      setState("selectedTaskId", newTask.id)
    } catch (err) {
      setState("submitError", String(err))
    } finally {
      setState("submitting", false)
    }
  }

  const selectTask = (id: string | null) => {
    setState("selectedTaskId", id)
  }

  const selectedTask = () => state.tasks.find((t) => t.id === state.selectedTaskId) ?? null

  const deleteTask = (id: string) => {
    setState("tasks", (prev) => prev.filter((t) => t.id !== id))
    if (state.selectedTaskId === id) {
      setState("selectedTaskId", null)
    }
  }

  /** 手动触发单次轮询 */
  const refreshTask = async (taskId: string) => {
    await pollTask(taskId)
  }

  return {
    state,
    selectedTask,
    startTask,
    selectTask,
    deleteTask,
    refreshTask,
  }
}

// ── Provider & Hook ───────────────────────────────────────────────────────────

export function ResearchProvider(props: ParentProps) {
  const value = createResearchState()
  return <ResearchContext.Provider value={value}>{props.children}</ResearchContext.Provider>
}

export function useResearch() {
  const value = useContext(ResearchContext)
  if (!value) {
    throw new Error("ResearchProvider missing")
  }
  return value
}

// ── TEST 模式预置 Markdown ────────────────────────────────────────────────────

const TEST_MARKDOWN = `# 测试公司深度研究报告

> 本报告为**前端渲染测试用例**，数据均为虚构，仅用于验证 Markdown 渲染效果。

---

## 一、公司概况

测试公司（Test Corp）成立于 2020 年，总部位于中国上海，主营业务涵盖 ==人工智能==、++大数据分析++ 及云计算服务。截至 2025 年底，公司员工规模超过 5,000 人，年收入约 **120 亿元**。

| 指标 | 2023 年 | 2024 年 | 2025 年 |
|------|---------|---------|---------|
| 营业收入（亿元） | 78.3 | 98.6 | 120.1 |
| 净利润（亿元） | 12.1 | 16.8 | 22.4 |
| 毛利率 | 41.2% | 43.5% | 46.0% |
| 研发投入占比 | 15.3% | 17.1% | 18.9% |

---

## 二、行业与竞争格局

### 2.1 市场规模

据测试研究院统计，2025 年中国 AI 应用市场规模约为 **3,800 亿元**，同比增长 32%。预计到 2030 年将突破万亿规模。

### 2.2 竞争对手对比

\`\`\`
公司         市值（亿）   毛利率    研发占比
测试公司       850        46%       19%
竞争对手A      1,200      38%       14%
竞争对手B      620        51%       22%
竞争对手C      430        33%       11%
\`\`\`

### 2.3 护城河分析

1. **技术壁垒**：拥有核心专利 230 项，在 NLP 和计算机视觉领域处于行业领先地位
2. **数据积累**：累积超过 10 亿条标注数据，形成显著的数据飞轮效应
3. **客户粘性**：Top 100 客户续约率达 **94.3%**，平均合同周期 3.2 年

---

## 三、财务深度分析

### 3.1 收入结构

\`\`\`mermaid
pie title 2025年收入结构
    "SaaS 订阅" : 55
    "项目定制" : 28
    "数据服务" : 12
    "其他" : 5
\`\`\`

### 3.2 现金流状况

> **关键风险提示**：公司 2024 年经营性现金流为 18.3 亿元，但资本开支持续加大，自由现金流仅 6.7 亿元，需关注未来融资需求。

- 经营性现金流：18.3 亿元 ↑ 31%
- 资本开支：11.6 亿元 ↑ 45%
- 自由现金流：**6.7 亿元** ↓ 3%

### 3.3 资产负债表摘要

| 科目 | 2024A | 2025A | 变化 |
|------|-------|-------|------|
| 货币资金 | 45.2 | 52.8 | +17% |
| 应收账款 | 18.6 | 24.1 | +30% |
| 有息负债 | 22.0 | 28.5 | +30% |
| 净资产 | 110.3 | 132.7 | +20% |

---

## 四、技术亮点

### 4.1 核心算法示例

\`\`\`python
# 测试公司自研 TurboLLM 推理优化核心逻辑（示意）
import numpy as np

def kv_cache_compress(attention_weights: np.ndarray, ratio: float = 0.3) -> np.ndarray:
    """
    基于重要性评分的 KV Cache 动态压缩
    ratio: 保留 token 比例
    """
    scores = attention_weights.mean(axis=0)
    k = max(1, int(len(scores) * ratio))
    topk_idx = np.argsort(scores)[-k:]
    return attention_weights[:, np.sort(topk_idx)]
\`\`\`

### 4.2 技术路线图

\`\`\`mermaid
gantt
    title 2025-2027 产品路线图
    dateFormat  YYYY-MM
    section 基础模型
    TurboLLM v2.0     :done,    2025-01, 2025-06
    TurboLLM v3.0     :active,  2025-07, 2026-03
    多模态大模型       :         2026-04, 2027-01
    section 应用产品
    智能客服 2.0       :done,    2025-03, 2025-09
    代码生成助手       :active,  2025-10, 2026-06
    AI Agent 平台      :         2026-07, 2027-06
\`\`\`

---

## 五、估值分析

### 5.1 DCF 估值

采用三阶段 DCF 模型：

| 阶段 | 年份 | 收入增速 | 自由现金流率 |
|------|------|---------|-------------|
| 高速增长期 | 2026–2028 | 28% | 8% |
| 过渡期 | 2029–2031 | 18% | 12% |
| 稳定期 | 2032+ | 8% | 15% |

**WACC**：9.5%　　**永续增长率**：3.5%

> 折现后合理市值区间：**780–920 亿元**，对应目标股价 **78–92 元**（当前股价 68 元）。

### 5.2 可比公司估值

| 公司 | 市盈率(TTM) | 市销率 | EV/EBITDA |
|------|-----------|-------|-----------|
| 测试公司 | 38x | 7.1x | 24x |
| 行业均值 | 42x | 6.8x | 27x |
| 估值分位 | 40% | 55% | 35% |

---

## 六、风险提示

1. **宏观风险**：全球经济放缓可能导致企业 IT 预算收缩，影响公司订单增速
2. **监管风险**：国内 AI 监管政策持续收紧，合规成本或显著上升
3. **竞争风险**：头部互联网平台加大 AI 投入，可能对公司市场份额形成冲击
4. **技术风险**：大模型技术迭代速度超预期，现有技术积累可能加速折旧
5. **客户集中风险**：前 10 大客户贡献收入占比 **38%**，单一客户流失影响较大

---

## 七、投资建议

综合以上分析，我们给予测试公司 **买入** 评级，12 个月目标价 **85 元**，对应上行空间约 25%。

核心逻辑：
- 行业 β 大，享受 AI 景气红利
- 管理层执行力强，业绩兑现度高
- 估值仍处历史中低位，安全边际充足

*本报告由 Hone Financial AI 系统自动生成，仅供测试，不构成投资建议。*
`
