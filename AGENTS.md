# AGENTS

## LLM 协作上下文约定

### 稳定文档（长期规则）

- `AGENTS.md`：仓库级协作规则、测试/发布契约、完成定义
- `README.md` / `README_ZH.md`：产品说明、快速开始、对外入口
- `resources/architecture.html`：交互式系统架构示意（浏览器本地打开）
- `CONTRIBUTING.md`、`SECURITY.md`、`CODE_OF_CONDUCT.md`：贡献与安全政策

### `docs/` 目录

- 除 `docs/README.md` 外，`docs/` 下文件**默认不进 Git**（见 `.gitignore`），仅供本地计划、草稿、交接笔记等。
- 需要写进版本库的长期说明，请更新根目录 `README`、`AGENTS.md` 或 `CONTRIBUTING.md`，而不是塞进已忽略的 `docs/`。

### 真相源优先级

1. 代码与测试
2. `README.md`、`Cargo.toml`、`package.json`、`config.example.yaml`
3. `AGENTS.md`、`CONTRIBUTING.md`
4. 其它说明性文档

### 文档维护规则

- 改动对外行为、安装方式或用户可见能力时，同步更新 `README.md` / `README_ZH.md`（中英文都需考虑）。
- 改动协作流程、CI 门禁、完成定义时，更新 `AGENTS.md`。
- 涉及 opencode / OpenRouter / 本地模型接入等可复用排障步骤时，优先写入 `CONTRIBUTING.md` 或根 `README` 的合适小节，而不是依赖未跟踪的本地文件。

### 外部 CLI 与模型配置

- 安装、鉴权与默认模型等**可对外复用的步骤**，落在 `README` 或 `CONTRIBUTING.md`；个人环境差异写在本地 `docs/`（已被忽略）即可。

## Agent Workflow

- 开始实现前，agent 必须先形成一个任务计划或 todo，至少包含：目标、涉及文件、验证步骤、文档同步步骤
- 不允许直接跳过计划进入编码；简单任务也应至少有一个简短计划，可只保留在当前会话，或写在本地 `docs/`（该目录默认不提交）
- todo 中必须显式包含“验证”和“更新上下文文档或说明无需更新的原因”
- 若任务影响行为、结构或工作流，todo 中的文档步骤须指向将更新的**已跟踪**文件（如 `README.md`、`AGENTS.md`、`CONTRIBUTING.md`）
- 关闭任务前，agent 必须检查 todo 是否全部完成，尤其是验证和文档同步两项
- 若任务中途范围变化，agent 应先更新 todo，再继续实施

## 完成定义

- 任务不以“代码改完”为完成，而以“相关验证和上下文资产同步完成”为完成
- 如果改动影响行为、结构或运维方式，必须同步更新对应文档
- 如果本次工作没有代码改动，也要在交付说明（或 PR 描述）中明确原因、影响范围和未验证项

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

## CI 契约

- PR / push 默认门禁仅包含：
  - Rust 格式检查（仅改动文件，`bash scripts/ci/check_fmt_changed.sh`）
  - Rust 编译检查（`cargo check --workspace --all-targets`）
  - Rust 测试（`cargo test --workspace --all-targets`）
  - CI-safe 回归脚本（`bash tests/regression/run_ci.sh`）
- 任何需要外部账号凭证的检查都必须放到 `tests/regression/manual/`，不阻塞主干合并

## CD 契约

- 以 `v*` tag 触发 release 构建
- 构建产物为多平台二进制包（当前仓库所有 bins）
- 发布流程只做“构建与产物发布”，不做自动部署到生产环境

## 运行约定

- CI-safe 回归：`bash tests/regression/run_ci.sh`
- 手工回归：`bash tests/regression/run_manual.sh`
- 单项手工回归：`bash tests/regression/manual/test_<topic>.sh`
