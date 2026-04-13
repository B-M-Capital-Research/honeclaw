# AGENTS

## LLM 协作上下文约定

### 稳定文档（长期规则）

- `AGENTS.md`：仓库级协作规则、测试/发布契约、完成定义
- `docs/repo-map.md`：代码库地图、入口、模块边界、常见联动改动
- `docs/invariants.md`：不可轻易破坏的约束、真相源优先级、验证边界
- `docs/open-source-prep.md`：公开仓库复制前的白名单 / 黑名单与清理清单
- `docs/decisions.md` / `docs/adr/*.md`：长期有效的决策记录
- `docs/deliverables.md`：重大交付物的留存范围、最小证据与链接约定
- `docs/archive/index.md`：历史事项索引；按主题串联 plan / handoff / decision / PR / runbook
- `docs/templates/*.md`：plan / handoff / decision 的统一模板
- `docs/runbooks/*.md`：长期可复用的操作手册；涉及环境搭建、外部 CLI 安装/鉴权/模型配置时优先查对应 runbook

### 动态文档（任务接力）

- `docs/current-plan.md`：动态任务索引页，记录当前活跃任务列表、状态与对应计划文件
- `docs/current-plans/*.md`：单任务计划页；并行任务必须拆分到不同文件，避免共享一个 plan
- `docs/handoffs/*.md`：单次任务结束后的交接摘要
- `docs/archive/plans/*.md`：已完成计划页归档；从 `docs/current-plans/*.md` 移入，不在活跃索引中继续保留

### 真相源优先级

1. 代码与测试
2. `README.md`、`Cargo.toml`、`package.json`、`config.example.yaml`
3. `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`
4. 其它说明性文档

### 文档维护规则

- 改动模块边界、入口、主要数据流时，更新 `docs/repo-map.md`
- 改动长期约束、测试流程、目录约定时，更新 `AGENTS.md` 或 `docs/invariants.md`
- 新增或调整可复用的环境搭建 / 运维 / 外部工具接入流程时，更新对应 `docs/runbooks/*.md`
- 新增或调整“重大交付物”留存规则时，更新 `docs/deliverables.md`
- 只有满足“动态计划准入标准”的任务，才更新 `docs/current-plan.md`
- 满足“动态计划准入标准”且需要单独跟踪时，才在 `docs/current-plans/` 下新增或复用对应任务文件，再更新 `docs/current-plan.md` 索引
- 完成一个中等及以上且需要交接、暂停续做或存在后续风险的任务时，新增或更新一份 `docs/handoffs/*.md`
- 涉及架构取舍、跨模块长期行为变化时，更新 `docs/decisions.md`，必要时补 ADR
- 完成并移出活跃跟踪的任务，需同步更新 `docs/archive/index.md`；若原来有计划页，归档到 `docs/archive/plans/`

### 外部 CLI 与模型配置

- `opencode` 的安装、OpenRouter 鉴权、默认模型与 variant 配置，统一参考 `docs/runbooks/opencode-setup.md`
- 如果任务涉及 `opencode_acp` 调试、换机、复现环境、让他人在新电脑上落地，默认先同步或引用 `docs/runbooks/opencode-setup.md`

### 动态文档治理

- `docs/current-plan.md` 不再承载单个任务的详细 todo，只做“索引页”
- `docs/current-plan.md` / `docs/current-plans/*.md` 只记录“需要持续跟踪”的任务；必须满足以下任一硬条件：
  - 预计跨回合或跨会话完成
  - 涉及两个及以上模块，或存在模块边界变化
  - 涉及行为变化、结构变化或工作流变化
  - 存在明确阻塞、依赖、并行协作或交接需求
  - 用户明确要求留档
- 不满足准入标准的任务，不进入 `docs/current-plan.md` / `docs/current-plans/*.md`；在当前会话里保留简短 todo 即可
- 明确不记录的典型小任务：单次 commit / sync / rebase、轻量脚本或配置修补、无行为变化的小补丁、纯文案 / 格式修正、一次性信息查询
- 每个并行任务只能对应一个 `docs/current-plans/<topic>.md`
- 同一任务的后续回合优先更新原计划文件，不要为同一主题重复开新计划页
- `docs/handoffs/*.md` 只在满足以下任一条件时写：
  - 任务完成，且产出了值得复用的交付物、验证结论或操作经验
  - 任务暂停，需要交接给下一位执行者
  - 存在已知风险、未决项、后续跟进建议
  - 任务包含非显然的验证步骤、迁移步骤或回滚注意事项
- 小型纯执行类任务（如一次性 git sync、轻量文案修正、无行为变化的小补丁）默认不单独新增 handoff，除非用户要求或该任务需要异步接力
- 同一主题在同一天内多次推进时，优先在既有 handoff 中追加新的阶段节点；不要覆盖旧结论，也不要新增第二份碎片文档
- handoff 必须面向“接手者需要知道什么”，不要把过程性流水账、逐条命令回显、无风险的微小操作都写进去
- 动态文档必须显式写出状态；推荐使用：`planned`、`in_progress`、`blocked`、`done`、`archived`

### 历史归档与索引

- `docs/current-plan.md` 只保留活跃任务；任务完成后应尽快移除对应索引
- 已完成的 `docs/current-plans/*.md` 应移入 `docs/archive/plans/*.md`；原文件路径不再复用给其它主题
- `docs/archive/index.md` 是历史检索入口；每个完成或暂停的重要任务至少追加一条索引，串联：
  - 对应 plan / handoff / decision / ADR
  - 关联 PR / commit / runbook / regression
  - 当前结论、状态、后续入口
- `docs/decisions.md` / `docs/adr/*.md` 若被新决策替代，必须显式标注 `superseded_by` 或在正文说明替代关系

### 文档模板与最小字段

- `docs/templates/plan.md`、`docs/templates/handoff.md`、`docs/templates/decision.md` 为默认模板；没有特殊理由时不要自定义格式
- plan / handoff / decision 至少包含以下字段或同等语义小节：
  - `title`
  - `status`
  - `created_at`
  - `updated_at`
  - `owner`
  - `related_files`
  - `verification`
  - `risks`
- 如存在替代关系、后续接续或关联交付，应补充：
  - `related_prs`
  - `related_docs`
  - `supersedes`
  - `superseded_by`

### 交付物留存规则

- “重大交付物”不只包含代码；凡是会影响后续维护、排障、迁移、发布或回归判断的证据，都应按 `docs/deliverables.md` 留存
- 默认应保留或至少留链接的交付物包括：
  - 数据迁移说明与回滚注意事项
  - breaking change 摘要
  - 验证命令与关键结果
  - 回归脚本及其对应问题背景
  - 关键性能基线或容量结论
  - 关键接口输入 / 输出样例
  - 发布说明、风险与操作前置条件

### 文档同步触发矩阵

- 改模块边界、入口、主数据流：更新 `docs/repo-map.md`
- 改长期约束、目录约定、测试规则：更新 `AGENTS.md` 或 `docs/invariants.md`
- 改架构取舍、跨模块长期行为：更新 `docs/decisions.md` 或 `docs/adr/*.md`
- 改环境搭建、外部 CLI、运维流程：更新 `docs/runbooks/*.md`
- 改“重大交付物”保留规则：更新 `docs/deliverables.md`
- 任务跨回合 / 跨会话：更新 `docs/current-plan.md` 与对应 `docs/current-plans/*.md`
- 任务完成 / 暂停 / 交接且具有后续价值：更新 `docs/handoffs/*.md`
- 任务退出活跃态：更新 `docs/archive/index.md`，并视情况归档 `docs/archive/plans/*.md`

## Agent Workflow

- 开始实现前，agent 必须先形成一个任务计划或 todo，至少包含：目标、涉及文件、验证步骤、文档同步步骤
- 不允许直接跳过计划进入编码；简单任务也应至少有一个简短计划，但可只保留在当前会话，不必落盘到 `docs/current-plans/*.md`
- todo 中必须显式包含“验证”和“更新上下文文档或说明无需更新的原因”
- 若任务不满足动态计划准入标准，todo 中也必须明确写出“不落盘”的原因
- 如果任务满足“动态计划准入标准”并影响行为、结构或工作流，todo 中的文档步骤必须指向具体文件，如 `docs/current-plan.md`、`docs/current-plans/*.md`、`docs/decisions.md`、`docs/handoffs/*.md`
- 若任务结束时从活跃态转为完成态，todo 中必须包含归档或说明“无需归档”的步骤
- 关闭任务前，agent 必须检查 todo 是否全部完成，尤其是验证和文档同步两项
- 若任务中途范围变化，agent 应先更新 todo，再继续实施

## 完成定义

- 任务不以“代码改完”为完成，而以“相关验证和上下文资产同步完成”为完成
- 如果改动影响行为、结构或运维方式，必须同步更新对应文档
- 如果本次工作没有代码改动，也要在对应计划、handoff 或本次交付说明中明确说明原因、影响范围和未验证项

## 测试组织策略（长期维护）

### 1. Rust 单元测试（默认）

- 位置：与实现文件同目录、同文件下的 `#[cfg(test)] mod tests`
- 用途：模块内部逻辑、私有函数、边界条件
- 规则：这是本仓库默认做法，不强制迁移到 `tests/` 目录

### 2. Rust 集成测试

- 位置：`tests/integration/`（按需新增）
- 用途：跨 crate / 跨模块协作验证（黑盒）
- 规则：需要全链路行为验证时优先写到这里

### 3. 回归脚本（CI 门禁）

- 位置：`tests/regression/ci/`
- 命名：`test_<topic>.sh`
- 规则：必须无交互、可重复、可判定、无外部账号依赖；失败返回非 0

### 4. 回归脚本（手工执行）

- 位置：`tests/regression/manual/`
- 命名：`test_<topic>.sh`
- 用途：依赖外部 CLI/账号/本机状态的验证（例如 codex/gemini）
- 规则：长期保留，但不作为默认 CI 门禁

### 5. 临时脚本（一次性排障）

- 目录：`scripts/tmp/`
- 命名：`tmp_<topic>_<yyyymmdd>.sh`
- 规则：不进 CI；禁止硬编码密钥；问题解决后 14 天内删除

### 6. 手工诊断脚本（长期非门禁）

- 目录：`scripts/`
- 命名：`diagnose_<topic>.sh`
- 规则：可长期保留；失败返回非 0；避免修改业务数据

### 7. 开源共建测试覆盖目标

- 本仓库采用“分层覆盖”策略，不追求全仓统一 `100%` 覆盖率，也不设置统一的 `90%+` 覆盖率门槛
- 核心 Rust 纯逻辑模块必须保持系统化单元测试，重点覆盖配置解析、session / persistence、quota、scheduler、skill runtime、prompt 组装、消息流转，以及它们的正常路径、错误路径、边界值、历史兼容分支
- 关键跨模块链路必须至少保留一条自动化证明，重点覆盖会话创建 / 恢复、存储切换或迁移、技能调用、定时任务触发、消息 ingress / outbound、关键 API 路径
- 前端测试优先覆盖 `packages/app/src/lib` 与状态 / 数据变换逻辑；页面层只保留少量高价值 smoke / E2E，不为静态 UI 细节追求覆盖率数字
- 外部集成与本机依赖能力（如 Feishu、Discord、Telegram、iMessage、CLI / ACP）优先使用本地 contract / unit 测试覆盖转换逻辑，再用 `tests/regression/manual/` 保留真实联通性验证；不要把这类检查提升为默认 CI 门禁
- 覆盖率数字只作为辅助信号：核心纯逻辑模块目标约为 `70%` 到 `80%`；高风险决策模块按分支覆盖思路做到接近 `85%` 的有效覆盖即可
- 每个新功能至少附带一个自动化测试；每个 bugfix 至少附带一个回归测试；纯重构如果没有行为变化，可以不新增测试，但不得削弱已有覆盖面或删除现有回归证明

## CI 契约

- PR / push 默认门禁仅包含：
  - Rust 格式检查（仅改动文件，`bash scripts/ci/check_fmt_changed.sh`）
  - Rust 编译检查（`cargo check --workspace --all-targets --exclude hone-desktop`）
  - Rust 测试（`cargo test --workspace --all-targets --exclude hone-desktop`）
  - 前端单元测试（`bun run test:web`）
  - CI-safe 回归脚本（`bash tests/regression/run_ci.sh`）
- 任何需要外部账号凭证的检查都必须放到 `tests/regression/manual/`，不阻塞主干合并
- `hone-desktop` 依赖桌面 sidecar 资源与打包环境，不属于默认 PR / push 逻辑门禁；相关检查保留在桌面构建或 release 流程中处理

## 本地 Push 门禁

- 本仓库通过 `.githooks/pre-push` 在本地 push 前先检查“即将推送的 Rust 变更是否通过 `rustfmt --check`”，再运行 `gitleaks`
- 当前 clone 初次启用时执行：`bash scripts/install_gitleaks.sh`
- 若 hook 提示 rustfmt 失败，先执行 `cargo fmt --all`，把格式化结果提交进 commit 后再重新 push；不要指望 hook 在 push 过程中偷偷改写即将推送的提交
- pre-push 只扫描“即将推送的 commits”，不扫描整个工作区；若命中疑似 secret，必须先清理或更新 allowlist，再重新 push

## CD 契约

- 以 `v*` tag 触发 release 构建
- 任何新建或更新 `v*` tag 的正式发版，都必须先补齐对应的用户向 release notes：`docs/releases/vX.Y.Z.md`；缺失时不得打 tag
- 用户如果只要求“push / 推送代码”，但没有明确要求“打 tag / 发版 / release”，agent 不得默认执行正式发版；必须先询问是否需要本次顺带正式发版
- 构建产物为多平台二进制包（当前仓库所有 bins）
- 发布流程只做“构建与产物发布”，不做自动部署到生产环境

## 运行约定

- CI-safe 回归：`bash tests/regression/run_ci.sh`
- 手工回归：`bash tests/regression/run_manual.sh`
- 单项手工回归：`bash tests/regression/manual/test_<topic>.sh`
