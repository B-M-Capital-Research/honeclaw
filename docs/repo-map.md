# Repo Map

Last updated: 2026-08-29

## Stage 80 Controlled-Shadow Claim-First Execution Attempt

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_execution_attempts.rs`
  - 提供管理员 registry 和单次调用路由；任何工件或输入读取前先 create-once claim，成功、失败或中断都永久消费 Stage 79 授权。
  - claim 后重算当前二进制摘要，并重开精确 Stage 71 冻结模型链；点时输入须自哈希、来源白名单/内容寻址、65 项特征与预处理完全匹配。
  - 确定性初始化虚拟观察配置，并执行单股/主题/总敞口/现金/数量五重上限；首次输出明确为 0 个前向观察日且没有绩效指标、订单或交易权限。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts.rs`
  - 暴露内部只读 helper，仅供 Stage 80 在 claim 后重开同一不可变 Stage 71 模型工件链。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 注册 Stage 80 管理员路由并将 readiness 升级为 v77；Stage 80 输出仍阻断在未来 Stage 81 独立验证门前。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-execution-attempt-panel.tsx`
  - 接收内容寻址点时输入 JSON 和八项不可逆确认，明确授权先消费、无未来绩效、无真实账本/持仓/订单/交易。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`、`packages/app/src/components/public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts` 与 `packages/app/src/lib/types.ts`
  - 接入 Stage 80 管理面、readiness 卡、API 与类型合同。

本轮未生成真实 Stage 80 记录或调用；Stage 81 链外独立输出校验仍未实现。

## Stage 78–79 Controlled-Shadow Executable Artifact Governance Correction

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_isolated_runners.rs`
  - Stage 78 v2 runner specification now create-once binds an exact executable artifact SHA-256, code revision and fixed runtime identity while keeping callable entrypoint, mount and all data/execution authority closed.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations.rs`
  - Stage 79 v2 independently reviews artifact digest reproducibility and exact artifact availability in addition to the complete Stage 51–78 chain; artifact/code drift fails closed.
- `packages/app/src/components/public-admin-controlled-shadow-experiment-isolated-runner-panel.tsx`
  - Collects code revision and executable artifact digest and makes the artifact-bound/no-entrypoint distinction explicit.
- `packages/app/src/components/public-admin-controlled-shadow-experiment-first-execution-authorization-panel.tsx`
  - Adds independent artifact reproduction/availability confirmations and exact artifact/code bindings.
- `packages/app/src/lib/types.ts`
  - Carries the Stage 78/79 v2 artifact and runtime contracts end to end.

No real Stage 78/79 record existed, so this correction requires no data migration and does not authorize Stage 80 execution.

## Stage 79 Controlled-Shadow First-Execution Authorization Review (v1 superseded)

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations.rs`
  - 对当前 Stage 78 runner 规格建立追加式、自哈希、角色隔离且批准终止的独立复核链；精确重算 Stage 51–78 完整绑定。
  - 批准只提供 24 小时、一次性的未来 Stage 80 claim-first 尝试资格；本模块没有执行入口、输入挂载、影子运行、账本、持仓、订单、券商或交易能力。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露管理员 registry/review API，并将实证准备度升级为 v76 Stage 79 门禁。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-first-execution-authorization-panel.tsx`
  - 提供十五项独立确认、批准/要求修改/拒绝和 24 小时单次边界；页面没有 claim 或执行按钮。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`、`packages/app/src/components/public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts` 与 `packages/app/src/lib/types.ts`
  - 接入 Stage 79 管理面、总览卡、API 和完整类型合同，继续明确所有投资执行权限关闭。

## Stage 64 Independent Validation-Evaluation Output Recalculation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_output_validations.rs`
  - 对 Stage 63 完成输出建立 create-once 通过/失败校验记录，校验人排除执行者与 Stage 51–63 完整上游角色。
  - 独立重建 validation-only 投影和九候选预测，逐位复算 81 条指标、54 项 component-block bootstrap/Holm 检验及 9 条逐目标建议；原输出与第二实现输出必须 SHA-256 完全一致。
  - sealed holdout 继续不可读；通过只开放未来逐目标候选准入复核，不得选模、写库、生成 reward/影子仓位/订单或交易。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 64 registry/validate API，并把实证准备度升级为 v61 validation-evaluation-output-validation gate。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-output-validation-panel.tsx`
  - 提供六项独立复算确认、待验输出选择、不可变结果与 81/54/9 摘要。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`、`packages/app/src/components/public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts` 与 `packages/app/src/lib/types.ts`
  - 接入 Stage 64 操作面、readiness 卡和类型/契约，明示“通过仍不是正式选模”。

## Stage 63 Claim-First One-Shot Validation Evaluation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts.rs`
  - 对精确未消费 Stage 62 授权先 create-once 写不可变 claim，再由宿主标签代理重开原始结果并只投影 validation 行；成功、失败或中断均消费授权，sealed holdout 不进入固定 worker 输入。
  - 重放冻结的九候选模型，生成 81 条逐目标/算法/种子指标、54 项 component-block bootstrap + Holm 候选检验和九条不可信逐目标建议；禁止 seed shopping、调参、综合分和全局有效性声明。
  - 输出写入一次性目录、回读核对哈希后删除；result 如实记录 validation 输入访问，模型/指标库、训练更新、正式选模、reward、shadow、order、broker 与 trading 全部关闭。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 63 registry 与 `invoke-once` API，并把实证准备度升级为 v60；即使出现成功 envelope，也只能进入未来独立输出复算校验资格。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-execution-attempt-panel.tsx`
  - 提供七项不可逆执行确认，展示可领取授权、claim、成功/失败、冻结统计摘要、逐目标不可信建议和“进程内能力隔离不是 OS 沙箱”边界。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`、`packages/app/src/components/public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts` 与 `packages/app/src/lib/types.ts`
  - 接入 Stage 63 操作面、readiness 卡与完整前端合同；sealed holdout、全局有效性、正式选模、模型/指标库及投资执行权限继续关闭。

## Stage 62 Validation-Evaluation First-Execution Authorization Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_first_execution_authorizations.rs`
  - 对精确 Stage 61 runner 建立追加式、自哈希、单根无分叉且批准终止的独立复核链；复核人排除 Stage 61/60/59/58/57、完整上游和此前 Stage 62 复核者。
  - 十六项确认全部成立时，批准仅生成 24 小时内最多一次的未来隔离 validation-evaluation 调用资格；授权本身不 claim、不挂载数据、不读取标签、不评估、不选模、不生成输出。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 62 registry/review API，并把实证准备度升级为 v59；只有当前、未过期、单次且完整绑定的批准才可进入下一门禁候选。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-first-execution-authorization-panel.tsx`
  - 展示十六项独立授权确认、24 小时/一次性边界和追加复核历史；页面没有 claim 或执行按钮。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`、`packages/app/src/components/public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts` 与 `packages/app/src/lib/types.ts`
  - 接入 Stage 62 管理面和 readiness 卡；validation/sealed-holdout、评估、选模、输出、模型/指标库及投资执行权限仍关闭。

## Stage 61 Validation-Evaluation Isolated Runner Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_isolated_runners.rs`
  - 为一条当前有效的 Stage 60 独立批准 create-once 登记内容寻址、`registered_not_run` 且无入口的隔离评估 runner；精确绑定 Stage 59 实现/合同、Stage 58 validation、Stage 57 output 与九个候选工件，并排除完整上游角色。
  - 当前不挂载任何数据。未来经独立授权后也只允许只读挂载精确 validation features/labels 和九个候选；sealed holdout 永久隐藏，输出只允许 create-once 写入待独立校验的逐目标逐种子指标、bootstrap/Holm 诊断和逐目标建议。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 61 registry/registration API，并把实证准备度升级为 v58；登记只开放未来独立首次执行授权复核资格。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-isolated-runner-panel.tsx`
  - 展示十项零能力/未来只读挂载确认、内容寻址工件、资源上限和登记历史，明确“登记不是执行”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 61 操作面和 readiness 卡；标签访问、评估、选模、sealed holdout、模型/指标库和投资执行权限仍关闭。

## Stage 60 Validation-Evaluation Implementation Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementation_reviews.rs`
  - 由 Stage 57–59 与完整上游之外的新角色追加独立复核；服务端自行重算实现、合同与候选集合哈希，并独立核对精确 3×3 工件矩阵、65/9 合同、逐目标逐种子指标、bootstrap/Holm、效果/诊断/样本门槛、三种子稳健性和零能力边界。
  - 复核链 create-once、自哈希、单根、单链尖、无分叉/断链/循环，批准记录终止链；退回或拒绝不开放任何后续资格。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 60 registry/review API，并把实证准备度升级为 v57；独立批准只开放未来隔离 runner 规格登记资格。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-implementation-review-panel.tsx`
  - 展示独立审计、十一项语义/统计/隔离确认、追加式复核和失败关闭状态，明确勾选不能替代服务端复算。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 60 操作面和 readiness 卡；当前仍无 validation 标签访问、评估、选模、sealed holdout、模型/指标库或投资执行权限。

## Stage 59 Validation-Evaluation Implementation Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementations.rs`
  - 为一条当前有效的 Stage 58 通过记录 create-once 登记无入口、内容寻址且 `registered_not_reviewed_not_run` 的评估实现；精确绑定 Stage 57 输出、9 个三臂三种子工件、65 项预处理和九项目标。
  - 在任何 validation 标签访问前冻结逐目标逐种子指标、零预测配对基准、10,000 次 component block bootstrap、54 项 Holm 多重检验、5% 最低 MAE 改善、三种子稳健性、样本不足和固定 tie-break 规则；禁止 seed shopping、调参和综合分遮蔽。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露 Stage 59 registry/registration API，并把实证准备度升级为 v56；实现登记只开放未来独立实现复核资格。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-implementation-panel.tsx`
  - 十项预注册/隔离确认、精确 Stage 58 候选选择、不可变工件信息和审计状态；明确“先冻结规则，再看 validation”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 59 操作面和 readiness 卡。当前无标签访问、评估、选模、sealed holdout、模型/指标库或投资执行权限。

## Stage 58 Independent Training-Output Recomputational Validation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_output_validations.rs`
  - 由训练执行与完整上游之外的新管理员 create-once 验证一份 Stage 57 产物；独立重开精确 claim/result、授权、训练副本和冻结套件，不调用 Stage 57 私有拟合/诊断 helper，第二实现复算 65 项预处理、9 个模型工件与 81 项 train-only 诊断并按 f64 位模式和内容哈希核对。
  - 失败也写入不可变、自哈希且禁止重放的终态记录；通过只开放未来 validation 评估实现登记资格，validation/holdout 标签、选模、模型/指标库、reward、shadow、order、broker 与 trading 全部关闭。
- `crates/hone-web-api/src/routes/mod.rs` 与 `crates/hone-web-api/src/routes/investment_decisions.rs`
  - 暴露训练输出验证 registry/validate 路由，并把实证准备度升级为 v55；独立验证通过仍不能声称模型有效。
- `packages/app/src/components/public-admin-historical-outcome-training-output-validation-panel.tsx`
  - 五项边界确认、待验证/通过/失败计数与逐条不可变审计界面，明确“可重现 ≠ 有效，更不等于可交易”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 58 操作面与 readiness 状态卡；下一门禁最多只允许独立登记 validation 评估实现。

## Stage 57 Claim-First One-Shot Train-Only Execution Attempt

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts.rs`
  - 先消费精确 Stage 56 授权，再只用已独立校验 training-store 副本的 train 标签运行固定三臂三种子；保留显式缺失，validation 与 sealed holdout 标签隐藏，输出为 9 个内容寻址候选和 81 条 train-only 诊断。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露训练执行 registry GET 与按 runner ID `invoke-once` POST；失败也消费授权且禁止重放。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v54 汇总 claim、完成/失败、未验证工件和独立输出校验候选；未验证输出不构成模型有效性或晋级依据。
- `packages/app/src/components/public-admin-historical-outcome-training-execution-attempt-panel.tsx`
  - 七项执行边界确认、一次性调用和状态审计界面；明确“真实拟合 ≠ 模型有效”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 57 管理面板和状态卡；下一门禁只允许 Stage 58 独立训练输出校验。

## Stage 56 Training First-Execution Authorization Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_first_execution_authorizations.rs`
  - 追加式、自哈希、批准终止的首次执行授权独立复核；精确重算 Stage 55 runner 与 Stage 54/53/52/51 完整绑定，固定十六项确认、24 小时有效期和最多一次未来隔离调用资格。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 `/admin/investment-decisions/historical-outcome-feature-label-join-target-training-first-execution-authorizations` registry GET 与按 runner ID 追加复核的 POST；没有 claim 或执行入口。
- `packages/app/src/components/public-admin-historical-outcome-training-first-execution-authorization-panel.tsx`
  - 展示完整上游、十六项硬确认和“授权复核 ≠ 数据访问或训练运行”；批准后复核链终止。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 56 管理面板、状态卡与 readiness v53；下一门禁仅为 Stage 57 claim-first 一次性隔离训练执行尝试。

## Stage 55 Training Isolated Runner Specification Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_isolated_runners.rs`
  - create-once、内容寻址且 `registered_not_run` 的 runner 规格登记；精确绑定 Stage 54/53/52/51，冻结工件、固定运行时、未来只读输入、train/validation/sealed-holdout 边界、create-once 输出与资源上限。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 `/admin/investment-decisions/historical-outcome-feature-label-join-target-training-isolated-runners` GET/POST；登记路径没有执行入口。
- `packages/app/src/components/public-admin-historical-outcome-training-isolated-runner-panel.tsx`
  - 展示九项硬确认、完整上游摘要和“规格登记 ≠ 数据访问或训练授权”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 55 管理面板、状态卡与 readiness v52；下一门禁仅为独立首次执行授权复核。

## Stage 54 Training Implementation Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_implementation_reviews.rs`
  - 追加式、自哈希、单链尖且批准终止的独立复核；独立重算 Stage 53 记录/合同，复核三臂三种子、65/9、train/validation/holdout、逐目标指标、资源和零能力边界。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露训练实现独立复核 registry GET 与按 implementation ID 追加复核的 POST。
- `packages/app/src/components/public-admin-historical-outcome-training-implementation-review-panel.tsx`
  - 展示“实现复核 ≠ runner 或训练授权”，要求十四项硬确认。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 54 管理面板、状态卡与 readiness v51；批准只开放未来隔离 runner 规格登记。

## Stage 53 Training Implementation Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_implementations.rs`
  - create-once、内容寻址、无执行入口的训练实现登记；精确绑定 Stage 52/51，冻结工件、代码版本、算法、指标与资源合同。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 `/admin/investment-decisions/historical-outcome-feature-label-join-target-training-implementations` GET/POST。
- `packages/app/src/components/public-admin-historical-outcome-training-implementation-panel.tsx`
  - 展示“实现登记 ≠ 训练运行”，要求十二项零能力边界确认。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 接入 Stage 53 管理面板、状态卡与 readiness v50；不开放 runner、数据访问或训练。

## Purpose

- Give a new session or model a low-cost entry point: understand the structure first, then read the source in depth
- Record only high-value, relatively stable structural information; task-level state belongs in `docs/current-plan.md`

## Source of Truth

1. Code and tests
2. `README.md`
3. `Cargo.toml` and each crate `Cargo.toml`
4. `package.json` and each package config
5. `config.example.yaml`

`docs/technical-spec.md` has been refreshed to match the current implementation and can be read as a structured supplement, but its priority is still lower than code, tests, README, and the various manifests.

## Repository Overview

- `docs/`
  - `current-plan.md`: active task index
  - `current-plans/`: single-task plan pages for parallel work
  - `handoffs/`: handoff summaries that only keep information needed for the next person
  - `open-source-prep.md`: allowlist / denylist and cleanup checklist before copying to a public repo
- `crates/`
  - `hone-core`: foundational capabilities such as the config façade / submodules, logging, errors, agent context, and sanitized compile/runtime build identity from `build_info.rs`
  - `hone-llm`: model provider abstraction, profile resolver, OpenRouter integration, and generic OpenAI-compatible provider plumbing used by configured auxiliary/background routes. `provider.rs` defines a metadata-first native stream lifecycle: requested/effective/fallback tool choice, content/reasoning/usage/indexed tool-call fragments, exactly one typed finish, then exactly one internal DONE. The generic OpenAI-compatible adapter accepts either literal `[DONE]` or one typed finish followed by error-free clean HTTP/SSE EOF (needed by MiniMax), normalizing the latter into DONE only at the adapter boundary. It tracks raw CR/LF framing before event parsing so an unterminated tail is an error, and rejects post-finish content/reasoning/tool payload; no-finish/duplicate-finish/error EOF remains incomplete, while OpenRouter retains its explicit-sentinel contract. Both providers may retry `Required` once as `Auto` only for an explicit 400/422 tool-choice capability rejection, never for auth/rate/transport/5xx/unrelated errors. An empty-tools terminal stream removes `tools`, `tool_choice`, and `parallel_tool_calls` after generic request options are applied while preserving generation options and SSE.
  - `hone-tools`: tool traits, registry, and built-in tools; the skill subsystem centers on `src/skill_runtime.rs`, `skill_tool`, the local `discover_skills` index, the `skill_registry` enabled/disabled override layer, and the compatibility `load_skill` shim. `skill_tool` parses structured script `stdout` and validates local image/PDF artifact roots, extensions, and MIME before exposing them to the model. Native Codex still uses projected workspace skills for discovery, but retains `skill_tool` as the trusted host script boundary; `mcp_bridge` supplies an absolute `HONE_SKILLS_DIR` before the child changes into the actor sandbox.
  - `hone-integrations`: external integrations such as X, Feishu, and image generation
  - `hone-scheduler`: scheduled task orchestration
  - `hone-channels`: channel runtime, `HoneBotCore`, shared channel startup bootstrap, unified `agent_session` run orchestration, `turn_builder` prompt/skill turn construction, `response_finalizer` assistant output finalization, canonical `run_event` types, the shared `execution` preparation layer, and the separate `runners` execution layer. After the concrete runner is selected, `execution` prepares a typed `RunnerConversationInput` from its `AgentConversationStrategy` (`NativePersistent`, `StructuredReplay`, or `EphemeralCompiledPrompt`), so native current-turn input cannot accidentally inherit replay-only system/history/tool fields. ACP runners additionally declare adapter/version-specific stream dialects instead of assuming Codex and OpenCode have identical event detail; OpenCode also normalizes newer adapters' JSON-string `rawOutput.output` into structured tool results before artifact or side-effect decisions. Codex workspace skill projection is a third, independent typed capability so another native runner cannot inherit it accidentally. `response_finalizer` has two deliberate boundaries: legacy/typed paths retain operational fallback and normalization, while completed Interactive Agent bodies use only system/internal-protocol/path safety cleanup plus media stabilization and never market-copy rewrite, planning-prose veto, tool-outcome reconstruction, or investment completeness refusal. The crate also hosts shared `ingress` (incoming envelope / actor scope / dedup / session lock / group pretrigger window), `outbound` (placeholder / reasoning / chunking / stream probes，以及把助手文本里的 `file://` 本地图片 marker 拆成有序 text/image 片段的共享逻辑), repo-external actor sandbox management, prompt-audit / session-compaction helpers, the cross-channel pre-session intercept layer for commands such as `/register-admin` and `/report`, plus shared attachment ingest / PDF preview helpers under `attachments/{ingest,vision,vector_store}.rs`. Feishu / Discord / Telegram attachment size and image-dimension gates are also centralized here.
- `agents/`
  - `gemini_cli`, `codex_cli`: legacy CLI agent adapters still supported by the unified runner factory
  - `function_calling`: strict in-process actor-bound Agent. Every Interactive run begins with ordinary `Auto` discovery. Finance activates only for a budget-accepted call to the registered `data_fetch` tool whose arguments parse, whose `data_type` is supported, whose required target is present/valid, and whose identity-search shape is valid; unsupported or missing-target DataFetch and unrelated Web/file/skill turns remain ordinary and cannot activate deferred ACK. The initial instruction asks the Agent to read the complete query and, for each named company/security within the bounded turn, return a separate DataFetch search with a stable case-sensitive `entity_route` and call-scoped `identity_match`. Any explicit route missing a valid `identity_match=exact_symbol|name_or_alias` is rejected before observer/registry/provider-network access and creates no route. The six-route admission ceiling applies in the first batch as well as later batches; unique-symbol aliases may merge into an existing accepted route. Lowercase and mixed-case symbols in a securities context still use normalized exact-symbol lookup.
    - The finance-mode request flag is channel-independent; Web alone may publish a typed service-owned Session-time/evidence-policy first line through an ACKed committed-delta boundary. Explicit-security fast turns may ACK it before the first model call; name-only finance waits for the valid DataFetch activation boundary above, with every call in that batch registered, structurally valid, and known read-only. Ordinary non-finance never ACKs it. After ACK, a later batch is admitted only when every tool name exists in the active registry, every argument payload parses and passes structural validation, and every call is classified known read-only; a failing call rejects the whole batch before assistant tool framing, observer notification, registry execution, or network access.
    - Its model requests use the composed current input plus at most four prior user utterances / 4,000 characters for follow-up references, while excluding historical assistant/tool/reasoning claims, prices, financials, and conclusions. Active finance stays in that same Agent/context. Before the structural entity/evidence floor it exposes only real business tools under `Required`; afterward the same real tools use `Auto`, allowing more evidence calls or one natural nonempty `Stop + Done` DirectFinal. The complete turn is capped at six routes, three finance tool batches, 24 total calls, 20 DataFetch calls, and six Web calls. An ACKed prefix starts this accounting immediately, so pre-DataFetch Web-only batches consume the three-batch, total-call, and Web-call limits. Exhaustion clears tools for the next iteration so the same Agent writes one evidence-bounded natural final from its existing context. Production does not expose `finish_research` and does not run a structured handoff, source catalog, locator correction, separate terminal synthesizer, terminal audit, semantic review, or second generation; every retired implementation entry is test-build-only. Model-authored tool-round prose remains speculative; the final tail is emitted/persisted once, while an ACKed-prefix failure appends one fixed explicit failure disclosure and persists exactly the visible partial bytes. `FunctionCallingAgent` also owns the one absolute overall deadline, per-step deadlines, failed `ToolCallMade` trace, and shared persistent-effect replay boundary.
  - Current-turn DataFetch/Web results remain directly in the Agent context. Search is identity-only; quote/profile/financial/relationship statements need their own current evidence. Web rows are bounded snippets rather than page bodies or rankings. The natural-final prompt owns Session time, quote-source time, exchange-field, and neutral relationship-strength discipline; these are Agent evidence rules, not a service-side parser, fallback scan, or post-answer gate. If raw current-turn read-only tool results exceed the provider window, `FunctionCallingAgent` keeps the complete trace internally but rebuilds one `tools=[]` request from the original input plus a bounded valid-JSON evidence copy; every call identity is retained and truncated fields are explicit.
  - ACP runners live under `crates/hone-channels/src/runners/`; `gemini_acp` config remains only for migration/reference
- `memory/`
  - Local storage abstractions for sessions, identity quotas, portfolios, cron jobs, and LLM audit logs
  - `memory/src/company_profile/{mod,types,markdown,storage,transfer,tests}.rs` now splits company portraits into stable public types, Markdown/template parsing, actor-scoped storage CRUD, zip transfer helpers, and colocated regression tests. Local mode stores portraits under `company_profiles/<profile_id>/profile.md` plus append-only `events/*.md`; `cloud.mode=cloud` uses PG `cloud_company_profile_files` for the same logical files. Tracking frontmatter includes A/B/C coverage tier and investment horizon, while the default portrait template carries expectation baseline, valuation scenarios, management commitments, catalyst calendar, open questions, and decision log. Structured event frontmatter can also append immutable-ID research-question and management-commitment updates; `CompanyProfileDocument::research_ledger()` folds those events into the current actor-scoped state without rewriting the first statement. Storage reads and transfer/import paths tolerate legacy plain Markdown files/frontmatter by synthesizing minimal metadata and defaulting missing coverage fields to C/long-term
  - `memory/src/web_auth.rs` owns web invite-list users, domestic/international identity, hashed email challenges, hashed per-user Hone Cloud API keys, public-login cookie sessions, and the public-user administrator boundary. Local development keeps ordinary dev login non-administrator; the separate `HONE_PUBLIC_DEV_ADMIN` lane is valid only under local deployment/local storage/explicit dev login, and Web API startup plus each dev login synchronize the dedicated test identity so an old session loses privilege when the flag is removed. `memory/src/billing.rs` separately owns typed Stripe-only external `billing_entitlements` plus domestic-invite authority and the idempotent Stripe `billing_webhook_events` inbox in SQLite or PostgreSQL. A Stripe entitlement is explicitly either `recurring_subscription` keyed by Subscription ID or `fixed_term_purchase` keyed by PaymentIntent ID; fixed-term access ends at its stored 12-calendar-month boundary and receives no renewal grace. Startup performs one forward-only rebuild/migration that removes retired-provider rows and backfills legacy Stripe rows as recurring without a dual read. One active domestic phone or one normalized international purchase email maps to one stable `channel=web` actor.
  - `memory/src/session.rs` stores versioned sessions and explicitly persists `summary`, legacy `runtime.prompt.frozen_time_beijing`, recoverable `tool` result messages, and the session ownership field `session_identity`; current prompt assembly no longer uses that legacy frozen timestamp as the displayed "当前时间". Local mode stores session JSON and can mirror / read through SQLite. `cloud.mode=cloud` uses PG `cloud_sessions` as the session hot path and does not write local session JSON.
  - `memory/src/session_sqlite.rs` hosts the SQLite-backed session persistence used by both shadow backfill and runtime reads/writes when `storage.session_runtime_backend=sqlite`
  - `memory/src/cron_job/mod.rs` keeps cron definitions, execution history, and the Web-only scheduled push projection per actor. Local mode uses per-actor JSON files plus shared SQLite execution/push history; `cloud.mode=cloud` uses PG `cloud_cron_jobs` / `cloud_cron_job_runs` / `cloud_web_push_messages` and PG due-job claims so multiple worker processes cannot run the same cron slot twice. Web push rows keep summary and full content separately plus server-owned `read_at`; opening push N marks only N and older rows read.
  - `memory/src/quota.rs` stores `success_count` / `in_flight` by `ActorIdentity` and Beijing date; local mode uses JSON files, while `cloud.mode=cloud` uses PG `conversation_quota`
  - `memory/src/portfolio.rs` stores actor portfolios. Local mode uses `data/portfolio/portfolio_*.json`; `cloud.mode=cloud` uses PG `cloud_portfolios` for tool, Web, and event-engine reads/writes.
  - `memory/src/llm_audit.rs` stores LLM audit records. Local mode uses SQLite at `storage.llm_audit_db_path`; `cloud.mode=cloud` uses PG `cloud_llm_audit_records` for runtime writes and Web audit list/detail reads.
- Event-engine Feishu direct delivery is assembled by `crates/hone-web-api/src/lib.rs` plus `crates/hone-event-engine/src/sinks/feishu.rs`: when building the event-engine sink, Web API reads both the cron-backed channel-target directory and direct Feishu session metadata, then passes unambiguous per-actor email/mobile targets into the Feishu sink so digest/card sends can resolve current-app `open_id` instead of reusing stale portfolio actor IDs. Ambiguous or non-contact targets are intentionally ignored to avoid cross-user delivery.
- Event-engine price policy is owned by `crates/hone-event-engine/src/prefs.rs`: `PriceAlertPolicyDefaults` converts canonical event-engine thresholds into one `EffectivePriceAlertPolicy` per actor. `router/{policy,dispatch}.rs` uses it for first-threshold promotion/demotion and actor repeat-step gating; `crates/hone-tools/src/schedule_view.rs` uses the same value for `notification_prefs.get_overview` and `/api/admin/schedule`, while its injected overview defaults add the system event-engine switch, global disabled kinds, shared per-category daily High cap, and the ordinary same-symbol cooldown's explicit intraday-price exception. `PricePoller` remains the producer of the global candidate-band grid, while actor notification cadence stays a routing concern.
- Event-engine Web delivery is also wired in `crates/hone-web-api/src/lib.rs`: the Web API registers a `web` `OutboundSink` that emits `push_message` through the shared `PushEvent` broadcast channel. Admin sessions consume it via `/api/events`, while the public user chat consumes it via `/api/public/events`; this path is separate from cron `scheduled_message` delivery but uses the same SSE transport to reach an open browser session.
- Earnings coverage is coordinated across `crates/hone-event-engine/src/{engine.rs,earnings_document.rs,earnings_continuity.rs,earnings_transcript.rs,subscription.rs,pollers/earnings_surprise.rs,pollers/earnings_quality.rs,pollers/corp_action.rs,pollers/news.rs,renderer.rs,router/dispatch.rs,digest/buffer.rs,sinks/multi.rs,store.rs}`. Portfolio holdings and explicitly tracked company profiles jointly feed subscriptions with A/B/C service cadence. The poller checks common US pre/post-market release windows on a dedicated interval, interprets FMP SEC `acceptedDate` as DST-aware US/Eastern, selects an earnings-looking 8-K exhibit, and uses that disclosure time as event time. The event-level quality review defaults to `earnings_quality` (`x-ai/grok-4.5`), preserves source B/M units plus conclusion, evidence, counterevidence, unknowns, and follow-ups, and can be manually compared with `examples/earnings_quality_models.rs`; numerically unsafe comma-B values fail closed and output shape is normalized before routing. A-tier actor continuity uses the separate `earnings_continuity` profile after T0 delivery to reconcile the append-only question/commitment ledger without changing the saved thesis. `examples/earnings_continuity_models.rs` plus the 24-event SEC fixture provide the opt-in paid four-quarter replay. Full transcript review is a second shared fact stage: `earnings_transcript.rs` separates prepared remarks from analyst Q&A, grades answer quality, persists only compact findings, and then lets the same actor continuity layer reconcile the profile under a transcript-specific event/job stage. `examples/earnings_transcript_models.rs`, its URL-only eight-call fixture, and the manual wrapper perform the opt-in official-IR PDF/DOCX replay without persisting source bodies. FMP `stock_news` transcript titles with an explicit `(TICKER)` conflicting with the supplied symbol are rejected. A shared canonical document key joins earnings-release 8-K support material to the structured card; a bounded quarterly research-object key additionally groups same-ticker transcript and 10-Q/10-K materials in either arrival order, with ticker filtering applied before candidate limits. A/B profiles archive unreviewed follow-up references as pending-review append-only evidence without an LLM call. `EventStore` also owns `earnings_continuity_jobs`: actor/object/stage-idempotent pending work, fifteen-minute leases, restart recovery, attempt-fenced terminal transitions, and exponential retry. Store-backed review dedup survives restarts without caching before durable insertion, and real channel errors remain failures even when a diagnostic log copy is written.
- Web scheduler delivery remains canonical in `crates/hone-channels/src/scheduler.rs`, but Web turns receive scheduler metadata and `crates/hone-web-api/src/routes/{events,history,public_pushes}.rs` projects deliverable results into summary cards. `/api/public/pushes` lazily imports pre-upgrade scheduler pairs with deterministic `legacy:*` ids, then lists actor-scoped summaries and aggregate unread count; `POST /api/public/pushes/:push_id/open` returns full content and applies mark-through read state. Feishu, Discord, Telegram, and iMessage keep their existing scheduled output.
- `bins/`
  - `hone-console-page`: Web console backend, static asset hosting, and API
  - `hone-cli`: local REPL
  - `hone-mcp`: local stdio MCP server that exposes Hone built-in tools to ACP runners
  - `hone-imessage`, `hone-telegram`, `hone-discord`, `hone-feishu`: channel entrypoints, with shared startup in `hone-channels::bootstrap` and per-channel sibling modules for scheduler / outbound / handlers where the protocol layer needs local ownership
  - `hone-desktop`: full Tauri desktop host with a thin `main.rs` façade, command handlers in `commands.rs`, backend / sidecar lifecycle in `sidecar.rs`, sidecar concern modules in `sidecar/{processes,runtime_env,settings}.rs`, tray extension points in `tray.rs`, and the local-runtime desktop packaging flow
  - `hone-user-app`: focused macOS Tauri shell for the production public user experience. It embeds only a startup/offline surface, opens directly to `https://hone-claw.com/chat`, persists WebKit login state, and intentionally has no Hone runtime, sidecar, ACP, MCP, channel, skill, config, or local-data dependency.
- `apps/hone-ios/`: independent native SwiftUI/WKWebView HONE client. It opens the same production `/chat` surface, persists login in the default WebKit store, keeps only HONE-owned HTTPS routes in-app, and delegates unrelated links to iOS without acquiring local runtime dependencies. The authenticated `/community` route is shared by Web, the macOS user shell, and iOS without a separate client-side data store.
- `config.yaml` / `data/runtime/`
  - `config.yaml` is the canonical user-writable config; dev uses the repo root copy, and packaged installs seed one under the user config dir
  - LLM provider credentials are config-owned: prefer `llm.providers.<symbol>.api_key/api_keys`, with legacy `llm.openrouter.*` readable only as config fallback; runtime LLM paths do not read parent process API-key env vars
  - `cloud.mode=local|cloud|auto` controls storage authority. `local` is the default and preserves JSON / SQLite / filesystem behavior even if PG / object-store env vars are present; `cloud` requires PG + object storage and exposes strict cloud status; `auto` keeps the older development behavior where env presence can enable cloud capabilities. `cloud.postgres` / `cloud.oss` define env-backed PG / object-store settings, including `HONE_POSTGRES_PROXY`, `HONE_OSS_PROVIDER=aliyun_oss|r2|s3`, and `HONE_OSS_PROXY`.
  - `cloud.community_delivery` controls the separately rolled-out public-community edge path. It defaults to `mode=off`; `shadow` and `prefer` may issue a short-lived, actor-bound `hone_community_edge` HttpOnly cookie signed from the environment-only `HONE_COMMUNITY_EDGE_HMAC_SECRET`, which must be 32..1024 UTF-8 bytes after trimming. Invalid secrets disable grants and clear the scoped cookie. Only `prefer` is accepted by the opt-in frontend discovery path, and the legacy authenticated community API remains available in every mode.
  - `crates/hone-core/src/cloud_runtime.rs` centralizes runtime role parsing, PG schema / health / document-index helpers, PG session / web auth / conversation quota / cron / skill registry / notification prefs / portfolio / LLM audit / company profile runtime helpers, actor-scoped object keys, Aliyun OSS / S3-compatible R2 signing, object-store proxy support, `.env` loading, and the cloud-mode local durable dependency report. `hone-cli cloud doctor` and `/api/meta` use this helper layer instead of inferring authority from config presence alone.
  - `bins/hone-cli/src/cloud.rs` provides `hone-cli cloud doctor`, `hone-cli cloud migrate`, `hone-cli cloud object-bench`, and the dry-run-first `hone-cli cloud community-publish`. Apply materializes the canonical PG timeline through a repeatable-read/read-only snapshot on the dedicated advisory-lock connection, fully hashes every eligible managed resource before any R2 publication write, rechecks lock liveness, then writes immutable descriptors, mutable resource `active.json`, immutable cursor pages, and mutable `latest.json` last; mutable writes are read back and explicit unlock failure is fatal. Dry-run performs only exact-key HEAD existence checks for resource bodies. The migrator dry-runs local `data/`, uploads recognized durable files to object storage under `users/{actor_storage_key}/documents/...`, indexes them in PG `cloud_documents`, imports legacy `sessions/*.json` rows into PG `cloud_sessions` with `--session-only` or as part of apply, imports legacy web auth SQLite rows into PG with `--web-auth-only` or as part of apply, imports legacy `conversation_quota/*.json` rows into PG with `--quota-only` or as part of apply, imports legacy `cron_jobs/*.json` rows into PG `cloud_cron_jobs` with `--cron-only` or as part of apply, imports legacy `runtime/skill_registry.json` into PG `cloud_skill_registry` with `--skill-registry-only` or as part of apply, imports legacy `notif_prefs/*.json` into PG `cloud_notification_prefs` with `--notification-prefs-only` or as part of apply, imports legacy `portfolio/*.json` into PG `cloud_portfolios` with `--portfolio-only` or as part of apply, imports legacy `llm_audit.sqlite3` rows into PG `cloud_llm_audit_records` with `--llm-audit-only` or as part of apply, and imports legacy actor-scoped `company_profiles/**/*.md` into PG `cloud_company_profile_files` with `--company-profiles-only` or as part of apply; remaining non-hot-path SQLite files are counted but skipped.
  - `data/runtime/effective-config.yaml` is the generated runtime snapshot for processes that want a materialized runtime config file
  - legacy `data/runtime/config_runtime.yaml` and sibling `.overrides.yaml` should not be recreated
- Actor sandbox research docs live under a repo-external `agent-sandboxes/<channel>/<scope__user>/company_profiles/<profile_id>/profile.md` plus `events/*.md` in local mode; in cloud mode the same logical company portrait files live in PG `cloud_company_profile_files`. Native-file runner edits remain compatible because response finalization syncs actor sandbox `company_profiles` Markdown to PG after successful turns. In local mode portfolio JSON must stay in `storage.portfolio_dir`, never inside actor sandboxes; in cloud mode portfolio state belongs to PG `cloud_portfolios`.
- `packages/`
  - `app`: SolidJS Web console and public user client. Public visual ownership is layered: `src/pages/public-foundation.css` owns HONE tokens and global interaction foundations; `public-site.css` retains the broad public-page legacy/layout rules; `public-polish.css` owns shared public navigation/push refinement; `public-chat.css` owns conversation surfaces; `public-agent-workspace.css` and `components/public-agent-workspace.tsx` own Agent workspace primitives; and `components/public-workspace-shell.tsx` plus `public-workspace.css` apply the same desktop sidebar, topbar, mobile brand bar, five-tab safe area, typography, and surface rules to `/community`, `/portfolio`, and `/me`. The Agent home uses a desktop three-column / mobile five-tab layout. `/community` is the Insights surface with an explicit “官方动态 / 讨论区” split: the first is HONE-published read-only material, while `components/community-forum.tsx` owns member posts, comments, likes, reports and the link back to the official curation workflow. `/portfolio` is Tracking with a real finance-calendar desktop month grid and a separate mobile agenda, `/activate` is the single Stripe international purchase-email verification surface with server-owned recurring and fixed-term offers and fails closed when server Checkout policy is disabled, and `/me` shows server-authoritative identity, entitlement kind, validity/renewal, and recurring-only Portal state rather than inferring payment from login or redirects. A server-authoritative public administrator additionally sees `PublicAdminUsagePanel` and `PublicAdminWhitelistPanel` under `/me`: `/api/public/admin/usage` reads the latest 14 Beijing natural days of real Web user questions plus scheduled execution/delivery history, while `/api/public/admin/invites` owns whitelist management. Both APIs recheck the database-backed administrator role, and the responsive page is inherited by browser, macOS WebView, and iOS WKWebView. Selecting history or sending a prompt switches the Agent home into the existing conversation view without navigation or a second message store. `/chat` startup uses one `PublicChatStartup` shell whose restore skeleton mirrors the same workspace chrome; `/api/public/bootstrap` returns auth/quota plus the newest 20 projected messages, and `/api/public/history?before=<cursor>&limit=20` supplies older pages. The official community archive rows come from `/api/public/community`, its cursor is timestamp-ordered through opaque content IDs, and `/api/public/community/resources/:resource_id` proxies only resources already stored in object storage; source-protected files remain metadata-only. Absolute projected offsets keep IDs stable while pages prepend, and each assistant turn remains one in-thread card across thinking, streaming, completion, recovery, abort, and error. Public chat consumes real runner `assistant_delta` events, batches token updates once per animation frame, and applies transient `assistant_reset` in place when a tool-call branch supersedes a visible preamble. A non-Abort stream interruption keeps that same card pending while bootstrap/history recovery runs and becomes an error only after recovery is exhausted; explicit abort/stop closes immediately. The chat runtime blocks browser-level multi-touch zoom while allowlisting the calendar lightbox's bounded custom pinch surface. Share-card export layout is self-contained in `chat-share-card.tsx`, with testable helpers in `chat-share-export.ts`. New finance-calendar sends still author desktop and mobile PNGs once, but the backend persists both as structured message metadata and bootstrap/history returns only the device-selected actor-owned path; `finance-calendar-message.tsx` never regenerates or swaps a history image. The portrait PNG renderer remains `src/lib/finance-calendar-mobile-renderer.ts` with deterministic 2x Canvas painting.
  - Public chat activity is tracked separately from quota. `crates/hone-web-api/src/state.rs` owns the single-process actor/session active-run registry; `routes/chat.rs` registers one `run_id` with an RAII lifecycle, emits safe `run_progress` / `tool_call.public_status_text`, and exposes the admin drain count; `routes/public.rs` projects the authenticated session's `active_run` or `interrupted_run` into bootstrap/history. `packages/app/src/lib/backend.ts` retries only idempotent `GET`/`HEAD`; public-chat POST and all other non-idempotent requests are one-shot even on transient response or transport failure. `packages/app/src/pages/chat.tsx` resumes elapsed time from the original server start without replaying POST, recovers a disconnected stream before rendering an error, keeps guarded investment drafts final-only, and renders a stable interruption card when the runner died. Raw tool/provider/reasoning fields are not public progress.
  - Public administrator earnings research is a structured branch of the same chat turn, not a separate workflow service. `packages/app/src/pages/chat.tsx` places `财报前瞻` / `财报分析` beside `持仓分析`, submits `{kind, company}` plus normal uploaded attachments, renders the existing run-progress/final-history surfaces, and downloads completed artifacts through an authenticated Blob request with visible progress/error state. `crates/hone-web-api/src/routes/public.rs` rechecks the database-backed administrator flag, owns the user-visible request text, blocks non-admin structured or direct slash activation, injects the exact `/earnings-research` invocation, and only then resolves the config-owned `agent.earnings_workflow` route. `routes/chat.rs` carries that server-only route into `AgentRunOptions`; `crates/hone-channels/src/agent_session/` rejects any trusted override without verified administrator prompt authority, switches context ownership from the global Codex native turn to OpenCode's ephemeral compiled replay, clears durable prior messages/compact summaries for this self-contained workflow while leaving them visible in public history, installs a dedicated system profile that bypasses generic Interactive investment enrichment/timestamp/output contracts, buffers all answer text, and rejects success unless the exact earnings renderer trace is followed by a PDF actually collected/persisted into the response. `crates/hone-channels/src/runners/opencode_acp.rs` treats an incomplete pre-render `end_turn` as failure and does not rely on post-compaction same-session prompts, which OpenCode may turn into zero-token no-ops. `agent_session` may rebuild the already-isolated current turn once only after the exact OpenCode/Gemini `400 invalid_request: Corrupted thought signature`, the exact OpenCode/OpenRouter `504 Upstream idle timeout exceeded` with `error_type=timeout`, or a renderer rejection that explicitly occurred before file creation; every observed non-renderer call must be known read-only, and artifacts, unknown calls or uncertain side effects block replay. After renderer success it publishes the trusted tool result's exact `validated_report_markdown` plus the artifact marker deterministically; `skill_tool` copies that field from the successful script payload because ACP arguments are not terminal trace authority. Ordinary chat and fuzzy errors are never replayed through this boundary. `crates/hone-channels/src/execution.rs` constructs that one turn from the configured OpenCode ACP adapter and exact model while ordinary chat keeps the global runner/model and normal history behavior, and untrusted actors retain strict fallback. OpenRouter credentials come from the existing config-owned `llm.providers.openrouter` pool and are injected only into the child process. The resulting PDF is a `document` artifact under the actor sandbox; `/api/public/file` may proxy only that authenticated actor's sandbox or an existing managed upload. If response sanitization exposes `<absolute-path>/<filename>`, the public route resolves only that basename inside the current authenticated actor sandbox and the shared file route canonicalizes both candidate and roots before containment checks, including macOS `/var` aliases.
  - Community archive repair is explicit and dry-run first: `hone-cli cloud community-contents` is a complete-timeline bootstrap/recovery reconciler, not a weekly append path, while `hone-cli cloud community-assets` validates local magic/size/SHA, verifies an immutable full-SHA R2 object, and atomically promotes only legitimately captured resources. Stored resource URLs carry a short SHA-derived version; the API verifies the full R2 SHA, emits strong ETags, gives versioned responses private immutable caching, and forces unversioned legacy requests to revalidate.
  - Public-community edge discovery is compile-time opt-in through `HONE_APP_COMMUNITY_EDGE_DISCOVERY=1`. The client first requests `POST /api/public/community/edge-session`, switches only for an enabled `prefer` grant with the exact `/_community/v1` base path, validates every resource `delivery_path`, and falls back immediately to the existing `/api/public/community*` calls when discovery, feed, image, PDF preflight, or download delivery fails. Personal unread/seen state remains on the backend API rather than in shared snapshots.
  - Daily investment dashboards are a cached-report branch above the same authenticated chat. `crates/hone-web-api/src/routes/daily_signals.rs` owns the 20:00 Asia/Shanghai worker, atomic `data/daily_signals/{macro,ai}/{latest.json,history/*.json}` snapshots, FRED macro chain, SEC/FMP-backed cloud-company financial evidence, scoring provenance and stale-success preservation. Macro v2 includes 10Y/30Y Treasury yields, effective Fed funds, employment-population ratio and VIX with risk-direction scoring. AI v2 excludes AI revenue, RPO/orders, specialized monetization and hardware-realization factors until a stable first-party series exists; missing unsupported factors are not rendered and are not part of the coverage denominator. `GET /api/public/daily-signals/{macro|ai}` and `/history` are read-only; opening the UI never regenerates a report. `packages/app/src/components/daily-signal-dashboard.tsx` owns the two launchers, gauges, trends, evidence/history/company-financial expansion and cached-report Q&A envelope; `stripAttachmentMarkers` hides that machine context from the visible user bubble while retaining it for the Agent.
  - Transcript-informed company ratings are another cached authenticated branch. `crates/hone-web-api/src/routes/company_ratings.rs` owns the 19:30 Asia/Shanghai snapshot and consumes transcript cards plus current quotes/financials. A fresh reviewed FMP bridge remains the preferred dynamic financial score input. When that bridge is empty, v7 reuses the decision engine's exact point-in-time SEC claim projection for visible revenue/margin/working-capital/cash-flow/capex evidence, claim IDs, official URLs, calculations and warnings. `crates/hone-web-api/src/routes/financial_evidence_review.rs` owns the separate administrator-only immutable review chain under `data/company_ratings/financial-evidence-reviews/audit/<SYMBOL>/`, canonical evidence fingerprint, six-confirmation approval contract and stale/invalid-audit projection. It also owns `hone-financial-review-readiness-batch-v1`: a default five-company actionable queue ranked only by audit/readiness state, with explicit full-queue and single-symbol reads. Only an exact `approved_for_rating` audit tip may supply growth-quality, pricing-power and financial-quality factors; every other SEC row remains a separately counted observation and cannot enter factors, peers, caps, valuation or actions. Queue selection and review never authorize training, reward, portfolio, shadow or trade use. `GET/POST /api/public/admin/investment-decisions/financial-evidence-reviews[/:symbol]` expose this workflow, and `packages/app/src/components/public-admin-financial-evidence-review.tsx` renders the batch/full-queue switch, readiness reasons, metrics, warnings, calculations, official links, checks and bounded verdicts inside the decision-brain administrator page. Valuation is nullable and can participate only from a same-day three-scenario artifact at `data/company_ratings/valuations/latest.json`: either a human-reviewed `hari-invest-v1 / verified` artifact or the reproducible `hone-valuation-v2 / computed` contract below. Computed rows require at least two methods, high/medium confidence, fresh dates, ordered scenarios and quote consistency; legacy `hone-valuation-v1` rows are rejected. The 15% valuation dimension scores probability-weighted upside, shrinks conviction by confidence and applies a two-method penalty. When no valuation passes, its weight is removed and the remaining dimensions are normalized. `packages/app/src/components/company-rating-dashboard.tsx` renders admitted valuation scenarios and review-only SEC evidence with explicit boundaries rather than a false zero-coverage state.
  - `crates/hone-web-api/src/routes/investment_decisions.rs` is the first durable decision-brain projection over company ratings. Every successful rating refresh writes a current record plus an immutable revision under `data/investment_decisions/{current,history}/`; authenticated `GET /api/public/investment-decisions/:symbol` returns the current version and builds it on demand when the cache is absent. The `hone-investment-decision-v1` record keeps historical research, current market/financial/forward evidence, valuation, missing crowding inputs, point-in-time market regime, action zone, falsifiers and next checks separate. Financial verification now carries year-over-year receivables, payables, inventory and PPE plus comparable-TTM operating cash flow, capital expenditure and free cash flow changes; true capacity, utilization and expansion timing still require company/industry evidence and stay missing. Six versioned first-principles families cover 47 of 52 company cards across storage/HBM, compute/semiconductor capacity, optical interconnect, data-center power/cooling, cloud/model platforms and AI applications. Each separates demand, effective supply, scarcity/differentiation and company value capture. Every driver has a causal-observation ledger that distinguishes direct financial metrics, explicit operating/capital/inventory proxies, confirmed primary/regulatory key-event context, event-bound structured earnings claims and deterministic two-period comparisons; each row freezes identity, relationship, date, source/URL/tier and remains `training_only_pending_human_review`. `crates/hone-event-engine/src/earnings_claim.rs` validates claim kind, metric definition, period, value/unit, speaker/locator and active/corrected/withdrawn disposition without parsing old prose summaries. The decision projection deterministically resolves superseded, withdrawn and conflicting claims. `hone-sec-period-comparison-v1` compares only active same-metric/same-basis/same-unit SEC reported facts, retains both filing traces, recomputes its formula during validation and forbids false sequential capex calculations. `hone-causal-promotion-v1` requires two active accepted-`supports` claims, two distinct source events and URLs, two periods and a 45-day span; computed comparisons cannot satisfy that gate. Accepted `falsifies` evidence creates a falsification block, conflict/rejection also blocks the driver, and mixed/context-only labels remain policy-inert. Two promoted drivers can raise confidence one level, while blocked conflict/rejection/falsification lowers it one level. Research zone and action are computed first and cannot be changed by this layer. Future, other-company and clue-only events are excluded. `hone-market-regime-v1` admits only a fresh, already-published, sourced macro report and freezes supportive/balanced/defensive/stress plus the original score, timestamps, cutoff and URLs. The 20:00 macro worker writes a separate revision when the market state changes but does not alter the company action. The key-event worker re-projects decisions only on a causal-model delta, using the later source-snapshot timestamp without duplicating unchanged samples; an immutable human causal review also triggers a new time-stamped projection. Every persisted record rejects future-dated research, financial, forward, market, valuation, causal evidence, regime input or review time. Each revision also writes `hone-investment-training-sample-v1` under `data/investment_decisions/training/`, freezing the state/action with pending human review, empty 20/60/250-market-session outcomes and no reward. Only administrators can export a symbol's chronological replay through `/api/public/admin/investment-decisions/replay/:symbol`. The administrator-only `/api/public/admin/investment-decisions/review-queue` deduplicates source claims and computed comparisons across daily samples, carries the latest verdict and effect (`supports`, `falsifies`, `mixed`, `context_only`) and returns chronological priority/status/kind filters without adding a write path. The daily maturity labeler uses FMP adjusted closes, counts common real market rows, excludes the open New York session, records SPY-relative return and maximum drawdown, and leaves incomplete periods pending; `broad-market-spy-v1` is fixed before outcomes to prevent benchmark selection bias. Administrator review requires mutation confirmation and optimistic revision IDs, records thesis verdict, corrected zone/action, explicit error attribution and per-observation causal acceptance/rejection plus effect and reason, then appends an immutable chained audit that replay can recover by revision links. Corrected, superseded, conflicted, withdrawn, legacy-unspecified claims and malformed computed traces cannot be accepted. `/api/public/admin/investment-decisions/evaluation` reports outcomes by horizon/action/confidence, original-versus-corrected direction, causal-link review coverage, deduplicated computed-review coverage, source/derived corpus counts, and latest human effect distributions by first-principles driver, source metric and earliest frozen market regime. These distributions remain labels rather than accuracy before outcomes mature. Its reward-design-review gate requires 100 complete 250-session rows, 30 non-overlapping company episodes, 20 symbols, eight decision quarters and 80% matured-sample review; none authorizes execution. `packages/app/src/components/public-admin-decision-brain-panel.tsx` exposes this workflow only under the server-authoritative administrator branch of “我的”, including the shared evidence queue, lifecycle/promotion/formula provenance review, causal-effect/regime calibration, responsive large-type Chinese controls and no reward or broker switch. Rewards remain unconfigured. This is a research/training path, not a broker execution path.
  - The same route now owns `hone-decision-completeness-v3-financial-quality-gate` and current `hone-financial-verification-v5-valuation-input-preparation`. It keeps eight research/portfolio layers separate, projects SEC claims with exact trace IDs/URLs and visible calculations, rejects duration/unit/time mismatches, and carries cash, the current XBRL long-term-debt tag, their explicitly incomplete difference, plus comparable OCF-minus-capex FCF absolute values. These prepared scalars are deterministically replayed during point-in-time validation; a non-positive FCF period suppresses growth instead of falling back to stale rating data. Historical v2/v3/v4 records retain version-bounded replay. Training/evaluation and administrator symbol replay quarantine invalid transitional records one by one; the symbol response exposes bounded filename/reason diagnostics while valid history remains usable. Quarantined rows never enter training or evaluation.
  - `crates/hone-web-api/src/routes/valuation_lab.rs` owns the global 19:20 Asia/Shanghai daily valuation run and authenticated `GET /api/public/valuation-lab`. Its reproducible provider path reads current FMP quote, quarterly cash-flow/income statements, the latest balance sheet and annual estimates, routes companies by business model, and requires at least two fresh, reasonably convergent methods before emitting bear/base/bull scenarios eligible for company ratings. The current v4 path also accepts a narrowly authorized SEC supplemental packet, but only through `crates/hone-web-api/src/routes/valuation_input_review.rs`: `hone-sec-valuation-input-review-v1` stores an immutable single-chain administrator review bound to the exact SEC evidence fingerprint, exact supplemental-input fingerprint, complete shares/net-cash inputs, at least two prepared methods, sources, eight confirmations and a seven-day input lifetime. Evidence changes, expiry or chain/binding errors revoke use. Rating projection uses `hone-valuation-v3-reviewed-input-binding` and rechecks the current review ID, both fingerprints and input date before accepting the valuation factor. `GET/POST /api/public/admin/investment-decisions/valuation-input-reviews[/:symbol]` and `packages/app/src/components/public-admin-valuation-input-review.tsx` expose the separate administrator workflow; `packages/app/src/pages/public-valuation-lab.tsx` shows rating-financial and valuation-use review states independently. Numeric thresholds remain transparent HONE engineering defaults, not universal old-Wang parameters, and no review grants training, portfolio, shadow or trade authority.
  - Actor-scoped portfolio news is the fourth cached dashboard. `crates/hone-web-api/src/routes/portfolio_news.rs` owns the 20:00 Asia/Shanghai worker and `GET /api/public/portfolio-news`: it lists real actor portfolios through `PortfolioStorage`, excludes watchlist-only rows, maps options to their underlying, pulls the last 48 hours through the existing FMP `NewsPoller`, admits only exact-symbol news that passes source/importance filters, and stores actor-isolated latest/history snapshots under `data/portfolio_news/<actor_storage_key>/`. The configured digest model receives only bounded news facts and must return validated impact JSON; identity, cost and position weight never leave HONE. HONE applies combined stock/option weights only to local ranking. Every new snapshot carries `hone-model-analysis-health-v1-fail-closed` with resolved model identity and normalized completion/failure counts; source-only facts stay visible but cannot become impact or action. `position_management.rs` checks this independently per symbol and forces low-confidence review when source or model coverage is incomplete. `packages/app/src/components/portfolio-news-dashboard.tsx` owns the read-only launcher, impact filters, provenance links, visible model-health gate and saved-report Q&A envelope; it does not change positions or fabricate an action when analysis is unavailable.
  - `crates/hone-web-api/src/routes/model_analysis_health.rs` is the shared fail-closed constructor and legacy default for all cached model interpretations. `key_event_chain.rs` applies one total analysis budget across topics; `influencer_digest.rs` keeps timed-out author material source-only; `weekly_brief.rs` scopes the inherited key-event health to only the industry rows it consumes. Their dashboard components carry the same health/per-item status into visible warnings and saved-report chat envelopes.
  - Actor-scoped position management is the fifth cached dashboard. After each portfolio-news refresh, `crates/hone-web-api/src/routes/position_management.rs` combines the actor's real positions with the latest company-rating, macro, portfolio-news and validated company-decision snapshots, then atomically stores latest/history under `data/position_management/<actor_storage_key>/`. A preliminary company candidate requires current quote plus quarterly financial evidence, a verified same-day valuation below the base case, green company quality, a current macro green light, no thesis-weakening news and no concentration alert; company admission additionally requires the same-symbol/current-rating `hone-hari-confirmed-logic-gate-v2-applicability` decision to pass LOG-V0001/2/6. The snapshot-level `hone-hari-portfolio-readiness-v1` then freezes LOG-V0003/4/5 evidence and named gaps. Because exact bull/bear exposure, barbell-role/allocation/correlation and sector-budget rules remain unconfirmed, its four authorization fields are false and a passed company candidate becomes low-confidence portfolio review instead of an increase candidate. HONE-owned 15%/25% single-position plus 45% theme bands remain separate product alerts. Negative thesis news, red company quality, high valuation and concentration can still produce bounded review/reduce labels. `GET /api/public/position-management` is read-only, invalidates old model versions, hides advice after portfolio changes and never mutates holdings or connects to a broker. `packages/app/src/components/position-management-dashboard.tsx` renders company and portfolio gates, structure, rationale, risks, falsifiers, evidence and a saved-report envelope that forbids promoting an incomplete candidate or claiming execution.
  - The global influencer brief is the next cached dashboard. `crates/hone-web-api/src/routes/influencer_digest.rs` refreshes at 19:50 Asia/Shanghai, reads only exact registered aliases from `event_engine.sources.rss_feeds`, keeps a 36-hour window and atomically stores `data/influencer_digest/{latest.json,history/*.json}`. SemiAnalysis uses its official feed. User-confirmed aichainmap Serenity uses the public `serenity-webhook.pages.dev/feed` as a translation/aggregation layer, but every item must pass an exact `x.com/aleabitoreddit/status/{id}` identity check and the X URL remains the content source; Jukan remains visibly unconfigured until a lawful bridge exists. The JSON fetch has HTTPS-host/path pinning, 15-second timeout and 2MB limit. Search snippets, other mirrors and similarly named accounts are never substitutes. The optional digest model receives bounded public excerpts and must return ID- and enum-validated JSON; tickers are exposed only when the source text names them. Missing source/model and snapshots older than 36 hours remain explicit. `GET /api/public/influencer-digest` is authenticated and read-only. `packages/app/src/components/influencer-digest-dashboard.tsx` separates original-author opinion, HONE summary and counterpoint, links both X original and the translation/aggregation page, and forbids converting a saved brief directly into a trade or position action.
  - `crates/hone-web-api/src/routes/key_event_chain.rs` owns the 19:55 Asia/Shanghai global key-event chain and `GET /api/public/key-event-chains`. Over a 30-day window it organizes twelve explicit first-principles chains — models, applications, data centers, ASIC, Rubin, HBM, HBF, NAND/SSD, 800G/1.6T optics, CPO, NPO and SOFC — and requires a topic match plus a closed milestone class before admission. It combines registered official feeds, attributed public sources and authorized global research, then applies `hone-key-event-identity-v1-high-confidence` before analysis and caps: same-topic/same-milestone sources within 96 hours merge only with shared entity/product anchors and exact or at-least-0.80 similar titles, while generic, numerically distinct and distant events stay separate. Canonical regulatory/primary selection is deterministic, every source remains exposed and source multiplicity never increases event weight. It keeps company/SEC confirmation separate from research/opinion/secondary clues and stores atomic `data/key_event_chains/{latest.json,history/*.json}`. Confirmation is topic-specific; model analysis is ID/enum bounded and cannot admit or upgrade an event. Missing source or model remains explicit. `packages/app/src/components/key-event-chain-dashboard.tsx` owns the industry-layer navigator, unique-event/source counts, event fingerprints, expandable source groups, first-principle statements, confirmed/clue filter and non-trading saved-report boundary; the legacy ten-day view is no longer serialized or rendered.
  - `crates/hone-web-api/src/routes/weekly_brief.rs` owns authenticated `GET /api/public/weekly-brief`. It composes the previous and next Beijing Monday–Sunday windows from structured macro/earnings calendar rows plus only confirmed, deduplicated key-event evidence. Past schedule-only rows remain result-pending, future rows remain reminders, and missing FMP coverage is explicit. Earnings scope combines actor holdings with HONE's 52-company research universe and uses the finance-calendar symbol cache. Industry rows preserve each event fingerprint and all supporting sources; their saved-chat contract forbids treating several sources for one event as several events. `packages/app/src/components/weekly-brief-dashboard.tsx` owns the standalone launcher, readable two-week agenda, same-event source disclosure and source-bounded saved-report Q&A envelope; no PNG is used for reading.
  - `crates/hone-web-api/src/routes/research_library.rs` owns the authenticated “我的知识源” and its trust ladder. It stores actor-scoped `personal`, isolated `community_candidate` and administrator-owned `hone_global` material, keeps provenance/date/SHA-256/parse status/tickers/topics/review state, and requires explicit authorization for chat, key-event-chain or portfolio-news consumption. Only successfully parsed personal items can be submitted; candidates never enter retrieval, and only a server-authoritative administrator can approve one into a copied global artifact or reject it with a review note. Relevant personal/global material is injected into both public chat paths as untrusted evidence and removed from visible history. Key-event chains may use authorized global material only as a topic-matched research clue, never as primary confirmation; portfolio news consumes actor-personal plus global items only for exact held tickers in the active 48-hour window. Local mode stores manifests and bytes under `storage.sessions_dir/research-library`; a multi-instance cloud rollout must migrate metadata to PostgreSQL and bytes to existing object storage first. `packages/app/src/pages/public-research-library.tsx` exposes `/research-library`, linked from `/me`, for connector-aware import, provenance, filtering, authorization, candidate submission, administrator review, download and deletion. Knowledge Planet enters only as a user-side official Skill/OAuth export or sync package and iMA as a user export until an official multi-tenant authorization contract exists; HONE does not scrape signed-in/private content or accept browser cookies.
  - `crates/hone-web-api/src/routes/community_forum.rs` owns the authenticated untrusted discussion domain. It stores a single-node atomic manifest plus optional attachment bytes under `community-forum`, projects only SHA-derived author aliases, supports posts/comments/actor-deduplicated likes/reports, auto-hides after three unique reports and lets only owners or server-authoritative administrators delete/moderate. One attachment is limited to 10 MB and to magic-verified PDF, UTF-8 text/Markdown or passive images. Forum APIs are not referenced by prompt, ratings, signals, event-chain, portfolio-news or research-library retrieval; `tests/regression/ci/test_community_forum_research_boundary.sh` locks that separation. Production must move the forum to PostgreSQL/object storage before enablement.
  - The investment-decision route also owns `hone-hari-confirmed-logic-gate-v2-applicability`, which freezes `hari-invest@0.1.0` and exactly LOG-V0001–V0006; v1 remains replay-only. V1/V2/V6 are evidence-admission gates for provisional increase candidates; V3/V4/V5 are explicitly delegated to the actor portfolio layer. Candidate logic is excluded, missing gates fail closed to research-only, portfolio authorization remains false, and point-in-time validation recomputes the projection to reject tampering.
  - `ui`: shared UI components and context; Markdown rendering is centered on `src/lib/markdown.ts` (`parseMarkdown`) plus the `Markdown` component / `MarkedProvider`, with base prose styles in `src/styles/index.css`
- `workers/`
  - `public-community-edge`: Cloudflare Worker for the exact `hone-claw.com/_community/v1/*` route. It is disabled unless `EDGE_DISABLED=false` is explicitly present, requires a trimmed 32..1024-byte backend-compatible HMAC secret, and validates the grant before touching Cache API or private R2. Every resource request reads mutable R2 `active.json` before shared-cache lookup, so inactive/old versions fail immediately even if bytes remain cached; missing/invalid indexes fail closed. Feed/resource GET misses may fall back only to the fixed origin, while resource HEAD, inactive versions, invalid descriptors, and integrity failures never do. Shared resource cache TTL is at most one hour, shared keys contain no user identity, and legacy/error responses are never inserted.
- `skills/`
  - In-repo skill definitions; runtime also supports `data/custom_skills/<id>/SKILL.md` and nested `.hone/skills/<id>/SKILL.md` with nearer dynamic directories taking precedence
  - Trusted persistent Codex ACP turns project each enabled source skill as a `hone__*` symlink under the repo-external actor workspace's `.agents/skills/`; Codex then performs native metadata discovery and progressive `SKILL.md` loading. Actor-owned entries in that directory are preserved, while legacy runners continue to use Hone's MCP skill bridge.
  - `SKILL.md` frontmatter now also supports an opt-in `script` entrypoint that `skill_tool(..., execute_script=true)` can run from the skill directory
  - `skills/stock_research/` is now the canonical equity-research skill surface: it covers single-company research, valuation framing, and criteria-based screening through one prompt plus compatibility aliases such as `valuation`, `OWGZ`, `stock screener`, and `OWXG`
  - `skills/scheduled_task/` now also owns portfolio event reminder linkage; the former standalone `major_alert` prompt has been folded into this skill
  - `skills/chart_visualization/` 是内置图表 skill：`SKILL.md` 定义 chart spec 与 `file:///abs/path.png` 输出契约，`skills/chart_visualization/scripts/render_chart.py` 用 Python `matplotlib` 把 PNG 写进 Hone runtime 的 `gen_images` 目录
  - `skills/earnings-research/` owns the administrator-only earnings preview/post-report workflow across trusted Codex and OpenCode runners. Its fixed stages require current-turn entity/period/evidence reconciliation and produce the final Markdown report before the runner-neutral host `skill_tool` boundary serializes a structured `script_payload` and invokes `scripts/render_report_pdf.py`; models never hand-escape the long renderer JSON. The renderer produces semantic tables in an A4 searchable PDF with the HONE watermark, disclaimer, and `packages/app/public/membership_zsxq.jpg` sharing page. Preview audits carry a guidance/segment-model anchor, exact per-driver bridge with primary/model source class and recognition basis, historical guidance bias, evidence-derived neutral tolerance, explicit report scale/unit, actual issuing-institution views and company-relevance-classified news. The renderer validates source/display arithmetic, a scale-aware inclusive tolerance boundary, institution identity/disclosures and final news depth. The live Dify `V2-财报前瞻` and linked news-module prompts own the content sequence: both the opening and `1.2.1` begin with beat/miss/in-line, assumptions publish growth and profit/margin logic, each institution gets its own dated comparison, and every news paragraph separates short-term/current-quarter impact from long-term and product/competitive effects plus a verification signal. News uses plain source names, a primary/company-direct majority and no public URLs. It explicitly forbids delegating execution to Dify/BamangResearch.
  - `crates/hone-event-engine/src/earnings_claim.rs` is the shared claim boundary between earnings materials and the investment decision brain. It admits only explicit structured arrays from company releases, formal filings and calls; validates ticker-bound HTTPS provenance, claim kind, canonical metric, period, numeric unit, speaker requirements and a short document/call locator; and stamps every admitted row `training_only_pending_human_review`. `crates/hone-event-engine/src/operating_kpi_claim.rs` is the stricter issuer-operating-measure boundary and the single shared six-model catalog. It maps each supported symbol to storage, compute, optical, power, platform or application, exposes only that model's KPI IDs to release/transcript extraction, preserves verbatim issuer definitions, scope, comparison basis and definition changes, and rejects unsupported symbols, cross-industry IDs, generic prose, spot-price substitutions and invented numbers. `crates/hone-event-engine/examples/operating_kpi_backfill.rs` provides the bounded dry-run-first path for source-verifying and idempotently inserting selected official historical documents without persisting their bodies; its network host allowlist remains deliberately narrower than the catalog and must be expanded only after each issuer domain is verified. The historical storage fixture remains replayable. `crates/hone-event-engine/src/sec_company_facts.rs` adds the deterministic SEC historical path: configured tickers are resolved against the SEC ticker map, Company Facts values are joined to exact submissions/accessions and emitted as research-only 10-Q/10-K/20-F events with official filing links and acceptance timestamps. The maintenance task may run independently while notification channels remain disabled; its events bypass routing, survive short-term retention and are idempotent by ticker/accession. Annual IFRS 20-F facts are admitted only under one exact `ifrs-full` taxonomy and one original allowlisted reporting currency per filing; HONE performs no FX conversion. Form 6-K, unresolved tickers, unsupported taxonomy/currency and fetched issuers without admissible Company Facts remain explicit, separately diagnosed coverage gaps. `EventStore::list_earnings_source_claim_events` excludes prose-only summaries while admitting either explicit financial or operating-KPI arrays. `crates/hone-web-api/src/routes/investment_decisions.rs` requires its current KPI registry to match the shared catalog's exact KPI-to-driver pairs, maps admitted rows to exact first-principles drivers as `structured_source_claim` or `operating_kpi_claim` observations, and puts the target KPI IDs directly into the measurement backlog. Operating KPI rows may partially measure only their registered driver; neither claim family changes action policy. Administrator evaluation and review expose corpus coverage, lifecycle, exact definitions and the full trace for human labels.
  - `skills/company_portrait/` now follows a lighter Codex-style pattern: keep the trigger/workflow contract in `SKILL.md`, and move the detailed portrait framework / event template / research-trail guidance into `references/`
- `data/runtime/skill_registry.json`
  - Local-mode global skill enabled/disabled override layer for registered skills; cloud mode stores the same layer in PG `cloud_skill_registry`
- `tests/regression/`
  - `ci/`: CI-safe
  - `manual/`: manual regression tests that depend on an external CLI, external account, or local machine state; live wrappers that call real services must stay opt-in behind explicit `RUN_*_LIVE_SMOKES=1` gates
- Linux production runtime delivery:
  - `.github/workflows/runtime-image.yml` builds the six managed server binaries inside fixed Debian Bookworm `linux/amd64`, publishes the exact revision to `ghcr.io/b-m-capital-research/honeclaw-runtime`, and reuses the scoped BuildKit GHA cache.
  - `deploy/runtime/Dockerfile` creates a `scratch` artifact image containing only `/release`; `scripts/package_runtime_bundle.sh` and `scripts/verify_runtime_bundle.sh` own the exact file/metadata/SHA-256 contract.
  - `scripts/stage_ghcr_runtime.sh` uses daemonless `crane` on the managed backend host to export, verify, and stage a release. It deliberately does not update `/opt/hone/current`, edit secrets, or restart systemd; the existing idle-drain and atomic-cutover runbook owns those steps.

## Key Entry Points

- Web console backend: `bins/hone-console-page/src/main.rs`
- Web console frontend: `packages/app/src/app.tsx`
  - 管理端与用户端现在按端口和构建产物分离：管理端默认走 `HONE_WEB_PORT` + `packages/app/dist`，用户端默认走 `HONE_PUBLIC_WEB_PORT` + `packages/app/dist-public`
  - 用户可见的长期研究记忆入口现只保留 `/memory` 下的公司画像视图；KB 页面与知识记忆 tab 已移除
- CLI: `bins/hone-cli/src/main.rs`
  - `hone-cli` now has explicit subcommands for `chat`, `config`, `configure`, `models`, `channels`, `status`, `doctor`, `start`, and `web`; `web admin-ui` / `web user-ui` start or locate the admin and user Web surfaces; `channels targets [--json]` inspects the typed cron-backed channel-target directory; no-subcommand mode still drops into the local chat REPL
- Standalone public macOS user app: `bins/hone-user-app/src/main.rs`; build with `scripts/build_user_app.sh` and operate it via `docs/runbooks/public-user-macos-app.md`
- Standalone public iOS user app: `apps/hone-ios/HONE.xcodeproj`; operate it via `docs/runbooks/public-user-ios-app.md`
- Channel runtime export: `crates/hone-channels/src/lib.rs`
- Shared channel bootstrap: `crates/hone-channels/src/bootstrap.rs`
- `AgentSession` abstraction: `crates/hone-channels/src/agent_session/mod.rs`
- Prompt/skill turn construction: `crates/hone-channels/src/turn_builder.rs`
  - Owns query-relevant skill hints, discovery guidance, slash-skill expansion, and invoked-skill runtime input composition
  - Public investment dialogue uses `skills/hari-invest` v0.2 as a decision layer. It is a redacted derivative of the supplied internal v0.4.0 package plus the installed v0.4.1 tool-agnostic research rule, not a runtime exposure of internal team workflows. Its contract requires a current-data boundary followed by an opportunity/hold/risk/data-insufficient zone, confidence, core reason, short/mid/long horizons, strongest counterargument and observable change conditions. `crates/hone-channels/src/prompt.rs` also embeds the 52-card `company-thesis-ratings` corpus plus a Chinese/English/ticker alias index. `turn_builder.rs` projects at most eight cards explicitly named in the current question into a private historical-baseline system section. The section forces both `company-thesis-ratings` and `hari-invest` to load; company cards outrank generic model memory for business model, fundamental structure, moat, value-chain position, risks and falsifiers, while current prices, filings, guidance, orders, news, industry state and valuation inputs remain owned by the existing current-turn evidence tools. Short tickers require explicit uppercase syntax to avoid matching ordinary words, and uncovered companies receive no card projection.
- Assistant response finalization: `crates/hone-channels/src/response_finalizer.rs`
  - Owns leaked-system/internal-protocol/path safety and local image marker stabilization for every response. Legacy and typed noninteractive routes retain empty-success/operational fallback behavior; Interactive Agent success uses `finalize_agent_owned_interactive_response`, which preserves business prose and layout and does not apply market-copy normalization, planning-sentence vetoes, tool-result reconstruction, or typed investment completeness checks.
- Canonical runner/session run events: `crates/hone-channels/src/run_event.rs`
- Shared execution preparation: `crates/hone-channels/src/execution.rs`
  - Centralizes prompt-audit write, tool registry creation, runner creation, and actor-sandbox-backed `AgentRunnerRequest` assembly for both session and transient task flows. Persistent requests receive a narrow authoritative session-metadata checkpoint; transient tasks do not acquire a durable native-session binding.
- Shared ingress model: `crates/hone-channels/src/ingress.rs`
- Shared outbound model: `crates/hone-channels/src/outbound.rs`
  - 同时也是 canonical 本地图片 marker 解析入口；Web 历史提取与外部通道图片投递都复用这里的 `file:///abs/path.png` 分段规则
- Runtime config mutation/materialization source of truth: `crates/hone-core/src/config/{mutation.rs,materialize.rs,yaml.rs}`; `mod.rs` re-exports the public helpers
- ACP MCP bridge: `crates/hone-channels/src/mcp_bridge.rs`
- Actor sandbox: `crates/hone-channels/src/sandbox.rs`
- Attachment ingest / preview helpers: `crates/hone-channels/src/attachments.rs` and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
  - Enforces shared attachment gates across channels: 5 MB for generic attachments, 3 MB for images, plus rejection of extreme aspect ratio, resolution, or pixel-count cases. Rejected attachments never enter the prompt. Successfully downloaded images on macOS pass through `attachments/vision.rs` Apple Vision OCR before the Agent runs; normalized bounded rows enter `attachments/ingest.rs` as the trusted `【图片文字提取】` block while the original attachment metadata remains available. Unsupported hosts and empty OCR never fabricate image facts.
- Runner contract and ACP / Gemini execution layer: `crates/hone-channels/src/runners/`
  - `crates/hone-channels/src/runners.rs`: runner module wiring and exports
  - `types.rs`: shared runner trait / request / event / result types, typed conversation input, the host-owned `AgentSessionMetadataCheckpoint`, ACP adapter kinds, and versioned stream profiles
  - `acp_common/`: shared helpers for ACP stdio / JSON-RPC, including `version.rs` as the single live-`initialize.agentInfo.name/version` profile selector/persistence boundary and ACP child-process-group cleanup so stdio MCP grandchildren such as `hone-mcp` are terminated on success, error, and timeout paths
  - `gemini_cli.rs`, `codex_acp.rs`, `opencode_acp.rs`, `hone_cloud.rs`: active primary runner implementations; Codex ACP validates npm Codex `>=0.146.0` inside the known major, selects the adapter dialect from the matching real initialize identity/version, applies model/reasoning/safety/developer instructions through `CODEX_CONFIG`, maps one Hone logical session to one persisted native ID through one initial `session/new` and later fail-closed `session/resume`, checkpoints a new ID before its first prompt, and sends only the canonical current turn through every `session/prompt`. Mode/fingerprint changes and native `contextCompaction` are telemetry, never session-rotation or user-message-reseed triggers. OpenCode remains a separately validated fresh-session replay runner and independently selects its live profile; `tool_reasoning.rs` owns the non-user-selectable strict function-calling fallback that keeps non-admin actors off trusted-host CLI/ACP subprocesses
- ACP external wire fixtures: `tests/fixtures/acp/`; filenames and embedded metadata identify the captured adapter version/date rather than mirroring private implementation structs
- Direct source deployment: `scripts/deploy_source_runtime.sh` builds a revision-stamped immutable Web/Discord/MCP package from the worktree-local `source-runtime` Cargo profile (`target/source-runtime`, line-level debuginfo, no incremental state) with `build.source=direct_source_runtime`, drains live chats, rejects unknown backend-port owners, distinguishes loaded launchd jobs from live PIDs, and explicitly migrates the legacy single `hone-cli start` supervisor into persistent revision-bound Web/Discord LaunchAgents. It waits supervisor/child PIDs and locks, verifies `/api/meta` Git SHA plus build-source provenance and channel login, and restores either the prior managed jobs or the legacy plist/job as one rollback unit. After commit it retains `current` plus `previous` immutable releases and fail-safe preserves unrecognized release directories. Development startup remains documented separately in `docs/runbooks/source-web-startup.md`.
- Prompt layering: `crates/hone-channels/src/prompt.rs`
  - `soul.md` is the complete investment reasoning and response-format contract; hard live-data, company-profile, channel, cron, privacy, and security rules are added by Rust
  - `crates/hone-core/src/config/materialize.rs` refreshes generated runtime prompt assets from canonical `soul.md`
  - `security_identifier.rs` owns lexical source-span scanning and supported bare/cashtag/exchange-qualified/share-class/index/crypto-pair syntax. It emits ordered spans, excludes URLs/email/source paths, and consumes unsupported composites without rescanning their suffix as a US ticker. Provider-dialect lookup, canonical keys, and equivalence are a separate shared truth source in `hone-core::provider_symbol`; the three channel helpers are thin delegates rather than a second alias table. `investment_response_guard.rs` owns `EntityResolutionScope`: `AgentToolDiscovery`, `Portfolio`, `Securities`, `Broad`, or empty `PassThrough`. Every nonempty interactive turn uses `AgentToolDiscovery`, including portfolio/watchlist wording; lexical `EntityMention` values retain provenance and span hints but are injected only as non-factual seeds. The configured main runner reads the full current query, calls `portfolio(view)` itself when membership matters, and performs current-turn DataFetch discovery, so no phrase grammar, exact-ticker count, comparison connector, broad-market marker, or auxiliary LLM closes the interactive entity set before the Agent runs. Scheduled/heartbeat turns may retain deterministic `Securities` / `Portfolio` / `Broad` preparation under their typed origin.
  - After an interactive run, `build_agent_discovered_investment` may consume the current DataFetch search/refinement and exact-quote trace for observational logs and regression evidence. Empty broad/enriched attempts may be skipped when diagnosing later exact-symbol refinement, and DataFetch envelopes are never treated as provider rows: a Chinese lookup such as `英伟达` is understood from the returned `NVDA` row, not the wrapper query. This Agent-discovered structure is not a publication contract. Whether it is complete, partial, or absent, `AgentSession` preserves the successful Agent's business body through the security-only finalizer: it cannot add a time/entity/quote prefix, normalize market prose, veto planning-like prose, reconstruct tool outcomes, launch an omitted-seed continuation or repair, rewrite content, reject the answer, or change `success=true`. The exact finalized body sent in the one Interactive segment is the body persisted in history. The Agent-selected financials, holdings, news, web, earnings, sector, market, and extended-hours results stay in current context for its one-pass synthesis. Typed scheduled/heartbeat preparation, server-owned fact blocks, strict validation, deterministic fallback, and fail-closed behavior remain a separate caller-supplied contract path.
  - The deterministic `Securities` / `Portfolio` / `Broad` routes used by typed noninteractive work resolve provider-canonical symbols before downstream generation, reuse verified probes, apply span-local numeric market/asset binding, and read the actor-scoped portfolio when the typed task requires it. Provider transport failure, empty coverage, ambiguity, wrong symbol, and stale time remain distinct states. `FmpConfig::default()` uses the same nonzero timeout and base URL as serde deserialization, preventing programmatic default configurations from creating immediate zero-second requests.
  - The guard supports single and mixed-asset turns, but Interactive reconstruction is observational only and performs no post-run answer validation or composition. Production Interactive has no internal finish handoff, locator correction, separate terminal synthesis, terminal audit, answer rewrite, or fixed completeness refusal. Session persistence and the Web SSE layer enforce one canonical natural answer and terminal event. Typed scheduled/heartbeat contracts retain complete field-aware single-security/fund/crypto/market/sector or comparison formatting, validation, repair/fallback safety, typed origin, and original scheduler task body. Unconstrained prose or an observed dynamic contract cannot be used as a publication gate.
  - `agent_session/emitter.rs` owns the one-output boundary. Guarded Interactive attempts expose progress/tool status but withhold typed prefixes, tool-capable answer deltas, resets, thoughts, and attempt-local errors because those rounds may still branch or fail. When the same Agent returns a complete natural DirectFinal, its security-finalized time-first body publishes once and the exact bytes persist. Typed scheduled/heartbeat fact-prefix validation remains separate. `tool_trace.rs` owns canonical aliases and read-only replay safety, and `hone-web-api/routes/chat.rs` maps complete `Done` or failed infrastructure outcomes to one authoritative `run_finished` while ignoring late frames.
  - Public chat uses a sync/send generation fence around bootstrap reconciliation. `handleSend` invalidates and aborts every older restore before appending the optimistic pair, so the preceding turn's asynchronous restore cannot delete the next assistant card and cause its deltas to disappear. GET/HEAD remain the only automatically retryable methods; chat POST is one-shot.
  - `turn_builder.rs` places only up to five query-relevant skill summaries in the current user turn and relies on `discover_skills` for misses; it no longer injects the full skill catalog into every static system prompt
- Session compaction service: `crates/hone-channels/src/session_compactor.rs`
- Prompt audit writer: `crates/hone-channels/src/prompt_audit.rs`
- Tool registry entry point: `crates/hone-tools/src/lib.rs`
- FMP-backed DataFetch implementation: `crates/hone-tools/src/data_fetch.rs`; its model-facing contract requires the Agent to declare a stable, distinct, case-sensitive `entity_route` for each named security, send one separate identity search per route (parallel calls are allowed), provide call-scoped `identity_match` on every search, and reuse the route verbatim on refinement, quote, profile/snapshot, and later calls. A formal-English-name or canonical-ticker refinement stays on the same route when a Chinese-name/alias lookup is empty; `refines_query` may link that exact empty query, while `supersedes_query` may migrate one earlier search that omitted a route. Both are case-sensitive verbatim metadata, mutually exclusive, not sent to FMP, and never used to parse natural language. Explicit ticker results must match the same code and cannot be replaced by a product whose name merely embeds it. Public `effective_data_fetch_{data_type,target,security_target}` and `validated_data_fetch_{symbols,search_query}` helpers are the single executor/ledger request parser: a present but wrongly typed higher-priority field does not fall through, broad types ignore ticker fields, only ASCII commas delimit a batch, and empty segments, controls, or inputs over 512 bytes are rejected. Every accepted symbol is URL-encoded per symbol. `extended_hours` returns one normalized latest minute bar and labels `pre / regular / post / closed` using US session boundaries before the investment guard applies exact-symbol/session/freshness checks.
  - Distinguishes semantic-empty results from authentication/provider failures, avoids caching empty identity/quote/profile/financial/holdings evidence, and rotates keys only for authentication, quota, or rate-limit failures rather than fanning transport/server errors across credentials.
- Tavily-backed current-event search: `crates/hone-tools/src/web_search.rs`
  - Supplies dated news, causes, policy, relationship, and event context after the security/market scope is known. Requests use basic search without answer/raw-page content and responses are locally capped again to three results. A trusted root `hone_search_contract` marks snippet-only scope, no full page, and that search order/score is not real-world rank; each result receives locally overwritten `hone_evidence` citation metadata, while a result without an original URL is non-citable. Provider-spoofed fields cannot loosen this envelope. Investment evidence remains untrusted external data until reduced to current entity-matching records; a URL locates its title/content/snippet but does not prove facts absent from that result, and web query dates cannot masquerade as event dates.
- Skill runtime source of truth: `crates/hone-tools/src/skill_runtime.rs`
  - Local mode stores global skill enabled/disabled overrides in `data/runtime/skill_registry.json`
  - Cloud mode stores the same registry in PG `cloud_skill_registry`; `HoneBotCore::new` injects the cloud runtime into `hone_tools::skill_registry`
  - Skill discovery supports bounded 2–16-character Chinese trigger fragments inside natural questions, so phrases such as “能买吗”“加仓”“护城河” can select `hari-invest` without slash syntax. One-character fragments remain excluded to avoid broad accidental activation.
  - Loading the Skill does not supply current facts or make a runner safe. Current price, filings, news and portfolio evidence still come from turn-scoped tools. Ordinary public actors must use an actor-safe function-calling provider or `hone_cloud`; host-capable Codex ACP remains an administrator-only runner and the missing-provider state fails closed.
- Channel settings surfaces: `bins/hone-desktop/src/sidecar/settings.rs` for Tauri/Desktop commands and `crates/hone-web-api/src/routes/channel_settings.rs` for normal Web mode. Both read and write the canonical config for enable flags, credentials, `chat_scope`, allowlists, and iMessage `target_handle`, then regenerate the effective runtime config.
- Feishu channel split: `bins/hone-feishu/src/{handler.rs,scheduler.rs,outbound.rs}`; `markdown.rs` parses standard Markdown tables and legacy raw table strings before message splitting, emits JSON 2.0 native root-level `table` elements (maximum five per card), and uses readable Markdown-list fallback for malformed, over-limit, or Markdown-only streaming paths
- Feishu image upload client: `bins/hone-feishu/src/client.rs`
- Telegram scheduler split: `bins/hone-telegram/src/scheduler.rs`
- Telegram outbound text/image interleave handling: `bins/hone-telegram/src/listener.rs`
- Discord outbound text/image interleave handling: `bins/hone-discord/src/utils.rs`
- Page-level pure state/data helpers: `packages/app/src/pages/{settings,users,notifications,task-health}-model.ts`
- Config sample: `config.example.yaml`
- GitHub install script: `scripts/install_hone_cli.sh`

## Main Flow

1. A channel entrypoint or the Web API receives user input and performs protocol parsing, allowlist checks, and explicit-trigger detection on the channel side
2. Before entering `AgentSession::run()`, channel entrypoints may short-circuit shared pre-session intercept commands in `hone-channels::core`, including runtime admin registration and the local report-workflow bridge (`/report 公司名`, `/report 进度`)
3. `hone-channels::ingress` centralizes actor scope, chat mode, deduplication, session serialization, shared group pretrigger buffering, and `IncomingEnvelope`
   - Event-engine pushes and scheduled/heartbeat results cross a separate outbound-confirmation boundary. Only an actual channel ACK or durable Web delivery calls `EventStore::log_confirmed_delivery`, which atomically appends the normal delivery audit plus an actor-scoped `delivered_push_context` journal row. Discord/Telegram register only the successfully sent segment prefix; Feishu registers after send success; Web/iMessage register after durable/HTTP delivery. Plain delivery audit never creates context implicitly.
4. `hone-channels::AgentSession::run()` orchestrates session locking, fast user-message persistence, quota, runner invocation, listener dispatch, compaction, and final persistence. `prompt.rs` takes one Beijing clock reading for the current-turn time anchor and the exact Web finance prefix/final-answer anchor; overflow recovery reuses it. Hone-managed runners receive the fuller `Session 上下文`, while a trusted persistent Codex ACP Interactive turn is reduced by `turn_builder.rs` to `【当前时间】 + 【本轮用户输入】` only. The latter keeps normalized attachment/image content and explicit slash-skill tasks but omits local history/session ID, routing metadata, related-skill hints, and repeated entity/answer contracts because the native Codex thread and seeded system prompt own those concerns. Every nonempty Interactive turn runs once with actor-bound tools. The first round is open `Auto` discovery; its prompt asks the Agent to place per-entity DataFetch search and ticker-independent Web/news/filing/industry candidate discovery in one model tool-call batch, while quote/profile and other symbol-dependent calls wait for route-bound search results; execution remains serial. Finance activation requires the registered, budget-accepted, supported/target-complete/structurally valid DataFetch boundary. The same context then carries route-correct quote/profile and question-specific evidence. When the current strict Interactive input is self-contained and contains an explicit security seed, one durable Session snapshot supplies active context, at most four earlier user-only reference utterances, and separately restored currently active invoked-skill prompts; synchronous pre-run compaction and compact-summary loading are skipped unless context overflow forces recovery. Referential follow-ups keep the full restore path. For runners that rely on Hone-owned history, a Session-level overflow force-compacts once and retries with the resulting summary; a second overflow automatically rebuilds from only the current request and its already prepared suffix, excluding compact summary, old assistant/tool protocol, and restored skill snapshots. Persistent Codex ACP sessions are excluded from that replay path because the native harness owns their history and compaction. Unknown/write-capable/uncertain work is never replayed, and public events/results/history never expose context/token/manual-compact/path/new-session instructions. Only valid identity searches enter the route ledger. Explicit-route searches missing a valid call-scoped `identity_match` and routes beyond the six-route ceiling—including a seventh route in the first batch—are rejected before observer/registry/network access and never pollute the ledger. The ledger moves real business tools from `Required` to `Auto` after the structural floor. Finance caps the turn at three tool batches, 24 total calls, 20 DataFetch calls, and six Web calls; exhaustion gives the same Agent one `tools=[]` natural-final iteration from its existing evidence. Production never exposes `finish_research` or starts handoff/correction/separate-terminal/audit/second-generation phases. On Web, the typed Session-time/neutral-evidence-policy line remains the mandatory first-line contract, but `commit_before_model=false` and `TerminalStreamPolicy::Disabled` keep the whole Interactive answer buffered until success. Once finance is active, an unregistered, malformed, structurally invalid, unknown-effect, or write-capable batch executes zero calls and gives the same Agent one tools-disabled evidence-bounded answer attempt. A one-off active empty/provider/protocol/step-timeout failure gets the same bounded recovery. If recovery also fails, no business prefix/body or fixed research-failure suffix is published. Post-run reconstruction remains audit-only and cannot normalize, retry, repair, rewrite, reject, or append to a successful body. Quote time comes from `hone_quote_time.beijing`, New York market dates cannot establish NYSE/close, exchange comes from structured exchange fields, and unsupported relationship strength stays neutral through the Agent prompt. Portfolio truth comes from the Agent's own `portfolio(view)`; typed scheduled/heartbeat contracts remain separate. Prior automation/failed groups are removed only from runtime context, not durable history. Keep an explicit distinction between:
    - `ActorIdentity`: who is executing this request
    - `SessionIdentity`: which history this message should be written into (group-chat shared sessions are controlled by it)
   - At Interactive ingress, `AgentSession` fixes the cutoff before waiting for the per-session lock, then claims only confirmed pushes delivered no later than that cutoff. The claim is reused across overflow/retry preparation, completed only after Agent success, and released on failure. `RunnerConversationInput` projects the same typed batch according to ownership: native persistence receives a bounded delivered-fact block before `【本轮用户输入】`; structured/fresh replay receives an assistant/context record before the current user turn. Neither path mutates the persisted user message or rebuilds system/history/tool protocol as user input.
5. `hone-channels::execution` builds the concrete execution plan for both persistent conversations and transient tasks: prompt audit, tool registry, effective per-actor runner selection, actor-sandbox-backed `AgentRunnerRequest`, and a host-owned metadata checkpoint only for persistent logical sessions. Codex uses that checkpoint after its only allowed `session/new` and before the first `session/prompt`; the runner still cannot read session storage. Native CLI/ACP runners are reserved for explicit administrators; when the configured runner requires trusted-host access, a non-admin actor executes through the strict in-process function-calling fallback with its actor-bound registry. Restore/compaction policy must use that effective runner rather than assuming the global `agent.runner` owns context; missing fallback LLM configuration fails closed.
6. `hone-channels::runners` executes the selected runtime and maps provider/CLI events into unified session events. `runners/tool_reasoning.rs` adapts the strict in-process Agent: tool-round and final deltas remain speculative behind the Session emitter, and active finance retains only actor-bound business tools in one loop. The route ledger keeps `Required` guidance until structural coverage or bounded no-coverage, then switches those same tools to `Auto`; it never adds a completion-control schema. The typed Web prefix is still passed into the same Agent as an exact final-answer contract, but production disables early terminal/header commitment and publishes the complete finalized answer once. The same Agent's DirectFinal has no ledger veto, terminal request, second generation, semantic filter, or rewrite. A blocked unsafe batch, active empty stream, one-off provider/protocol error, or step timeout may schedule exactly one same-Agent no-tools continuation; a persistent tool execution failure or uncertain mutation still stops without replay. Provider adapters report requested/effective/fallback tool choice plus typed finish and DONE; incomplete lifecycle output never counts as completion. Provider reasoning remains protocol-only and non-persistent. Configured step/overall deadlines bound the complete run, failed tools remain in `ToolCallMade`, and uncertain writes stop before replay. ACP runners include local `hone-mcp`; CLI child process groups are cleaned up on all terminal paths.
7. `hone-channels::AgentSession::run()` stores parseable tool-call results returned by the runner into the session for future cross-turn recovery; `hone-channels::outbound` and each channel adapter consume the unified events and finish placeholder / reasoning / chunked / streaming responses according to platform capability。Local mode keeps generated chart/media markers as inline `file://` paths: Web renders them inline, while Feishu / Telegram / Discord send them as ordered image messages. Cloud mode uploads generated images from either actor sandboxes or `gen_images` to OSS during response finalization and returns `oss://...` markers. Web direct 成功回复还会把本轮新生成、且 final 正文提到文件名的 actor sandbox 文件追加为 `[附件: ...]` marker，供 public history 转成可下载附件 metadata。
8. `hone-tools` provides data, skills, search, scheduled-task, and other capabilities
   - Skill disclosure is now two-phase: the model first sees a compact listing, and full `SKILL.md` bodies are only expanded into the turn after `skill_tool(...)` or a user slash skill is invoked
   - Invoked skill prompts are persisted in session metadata so context restoration can re-inject them after compression instead of relying on historic tool results
   - 用户可见的研究记忆相关 skill 目前只保留 `company_portrait`
   - Declared skill scripts run as trusted repository code with a cleared child environment; only execution/artifact variables and basic process variables are passed through, never server database/object-store/model credentials
9. `memory` reads and writes sessions, quotas, portfolios, and cron jobs
  - `memory/src/quota.rs` keeps a daily successful-reply quota for each user-initiated conversation; local mode writes JSON, cloud mode writes PG, the runtime limit comes from `agent.daily_conversation_limit`, whose default is `100`, and `0` means unlimited
    - `memory/src/llm_audit.rs` uses SQLite in local mode and PG `cloud_llm_audit_records` in cloud mode to record LLM call audit logs archived by `ActorIdentity`
    - Session persistence is controlled by `storage.session_runtime_backend` in local mode; `json` reads from local files, `sqlite` reads from `storage.session_sqlite_db_path`, and JSON can still be dual-written as a rollback mirror through `storage.session_sqlite_shadow_write_enabled`. In cloud mode, `SessionStorage::new_cloud` uses PG `cloud_sessions` as the runtime truth source while still optionally dual-writing the local `sessions.sqlite3` mirror for bug triage / recovery tooling when the sqlite shadow path is configured.
    - Session compaction is now boundary-based: compacted sessions write a `Conversation compacted` marker plus a compact summary message, and the active context window is restored from the most recent boundary forward
    - `codex_acp` and `opencode_acp` session turns persist restorable assistant/tool transcript structure locally as `assistant(tool_calls)` + `tool` messages. OpenCode continues to compile that restored transcript into each fresh ACP session. Codex never serializes the local transcript into its native user prompt: it provisions developer instructions through `CODEX_CONFIG`, creates exactly one native thread when no ID exists, checkpoints that binding before prompt, resumes the persisted ID regardless of later mode/fingerprint changes, and sends only current Beijing time plus current normalized user/attachment content. Codex continuation uses `session/resume`, never `session/load`.
    - `AgentSession::run()` now also supports explicit `/compact` requests, reusing the same compaction pipeline without charging user conversation quota or persisting the slash command as a normal transcript message
    - Cron persistence is local JSON / SQLite in local mode and PG-backed in cloud mode. Heartbeat-style cron jobs are still stored in the same cron store; they are identified by `repeat=heartbeat` and a `heartbeat` tag, then polled every 30 minutes instead of a fixed clock time. In cloud mode, due-slot claims use PG before execution so multiple workers do not duplicate the same cron run.
    - Portfolio persistence is local JSON in local mode and PG-backed in cloud mode. `HoneBotCore::new` injects the PG runtime into `PortfolioStorage` so tool/Web/event-engine callers share the same backend without changing their constructor API.
9. Responses are sent back to the originating channel; the Web console streams `run_started / run_progress / assistant_delta / assistant_reset / tool_call / run_error / run_finished` via v2 SSE events. `run_finished` is terminal and unique; the client closes its observer and discards later frames. Browser retries are limited to GET/HEAD, chat POST is one-shot, and a non-Abort stream disconnect enters bootstrap/history recovery before error presentation. Refresh/recovery observes the existing run and server-owned original start time without replaying POST or resetting elapsed time. Controlled CLI shutdown polls `/api/runtime/active-chat-runs` before terminating the backend.

The public-chat page also fences bootstrap reconciliation with sync/send generations. Starting a new local send invalidates and aborts an older restore before the optimistic pair is appended, preventing the previous turn's asynchronous history response from deleting the new assistant card or dropping its deltas.

## Desktop Structure

- `bins/hone-user-app/` is the remote-only public macOS lane. Its local UI is limited to branded startup/offline recovery, its first-party navigation allowlist is `hone-claw.com`, and unrelated links open in the system browser. It must not acquire local runtime or sidecar dependencies.
- `scripts/build_user_app.sh` produces a Universal macOS `.app` and `.dmg` from the dedicated crate without preparing the full desktop sidecars. It uses a separate Cargo target directory so user-app builds cannot collide with `hone-desktop` artifacts.
- `apps/hone-ios/` is a separate Apple client lane rather than a `hone-desktop` target. Its native SwiftUI shell owns startup/offline recovery and WebKit navigation policy while the production public Web app remains the feature source of truth.
- The Tauri host lives in `bins/hone-desktop/`
- `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}` now separates the builder façade, Tauri command handlers, backend lifecycle, and tray extension point
- `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs` keeps process supervision, runtime environment/path wiring, and persisted desktop settings / overlay writes out of the main Tauri command surface
- Desktop sidecars are prepared by `scripts/prepare_tauri_sidecar.mjs`, which detects the target triple, builds the supported channel bins plus `hone-mcp`, resolves/bundles macOS `opencode`, copies them into `bins/hone-desktop/binaries/`, and writes `bins/hone-desktop/tauri.generated.conf.json` for `bunx tauri dev/build`
- The same script also supports target-override / skip-build self-checks, so macOS packaging expectations can be verified by regenerating config for `*-apple-darwin` without requiring a full build
- Root `make_dmg_release.sh` is the macOS release entrypoint: it prepares bundled binaries for `aarch64-apple-darwin` and `x86_64-apple-darwin`, runs `tauri build --target`, and collects DMGs into `dist/dmg/`
- Tag release workflow emits installable CLI bundles (`honeclaw-darwin-aarch64.tar.gz`, `honeclaw-darwin-x86_64.tar.gz`, `honeclaw-linux-x86_64.tar.gz`) plus a Universal HONE macOS DMG, iOS Simulator app zip, and iOS Xcode source zip. It requires `docs/releases/vX.Y.Z.md`; the Apple client lane also uploads a dedicated checksum file and never labels an unsigned Simulator build as a device IPA. `scripts/install_hone_cli.sh` consumes CLI assets, while the workflow updates the dedicated Homebrew tap for CLI installs.
- Release-oriented Rust builds are warmed in two layers: `.github/workflows/release-cache-warm.yml` prebuilds the three shipped targets on `main`, `Swatinem/rust-cache` stores dependency/`target` state per release target, and `sccache` stores compiler outputs so tag releases mostly reuse warmed caches instead of compiling cold
- Windows desktop packaging intentionally excludes `hone-imessage`; macOS packaging keeps it, and runtime support still uses `cfg!(target_os = "macos")` as the source of truth
- Source-checkout runtime startup goes through `cargo run -p hone-cli -- start --build --detach`; the foreground CLI builds local runtime binaries, then spawns a detached hone-cli supervisor that writes `data/logs/hone-cli-start.log`, generates `data/runtime/effective-config.yaml`, starts `hone-console-page` plus enabled channels, and writes `data/runtime/current.pid`
- Desktop dev starts explicitly through Tauri tooling: `bun run tauri:prep:dev -- --skip-dev-command` plus `bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json`; use `--shell-only` prep when connecting the desktop shell to an already running CLI backend
- `launch.sh` is only a compatibility shim that points users to the CLI source or installed startup path
- `hone-cli onboard` is the first-install guided setup path for bundled CLI installs and repo-local use: it defaults to `codex_acp`, can collect `agent.hone_cloud.api_key`, detects local `codex` / `codex-acp` / `opencode`, can switch to `opencode_acp` without forcing Hone-side provider config, guides channel enablement with mandatory local fields plus prerequisite notes, lets the user back out of a mistaken channel enablement by disabling that channel mid-flow, and requires an explicit configure-or-skip decision for `FMP` / `Tavily` API keys
- `hone-cli start` is the local launch entry for bundled CLI installs and repo-local use: it loads canonical `config.yaml`, generates `data/runtime/effective-config.yaml`, starts `hone-console-page`, waits for `/api/meta`, then starts enabled channel listeners; `--build` adds source-checkout runtime binary builds before startup
- `hone-cli cleanup` is the explicit installed-layout teardown helper: it can interactively remove `~/.honeclaw` config, runtime data, and downloaded release bundles before the user runs `brew uninstall honeclaw` or removes the wrapper manually
- Desktop startup now uses per-process runtime lock files under `data/runtime/locks/` (or the app runtime dir in packaged mode). `hone-desktop` must hold its own lock, each standalone channel/backend binary must hold its own lock, and bundled desktop mode preflights the full `hone-console-page` + enabled-channel set before startup. When the conflict still points at a live matching Hone process, desktop startup now attempts one lock-targeted cleanup by pid and then retries before surfacing the blocking error.
- The desktop app supports two backend modes:
  - `bundled`: Tauri starts the built-in `hone-console-page` sidecar and points the frontend API at a local loopback address
  - `remote`: Tauri does not start a local backend; the frontend connects directly to a remote HTTP base URL
- Persistent user config now lives in canonical `config.yaml`; CLI/start flows and desktop-managed sidecars export the generated `data/runtime/effective-config.yaml`, while settings surfaces mutate the canonical file through shared config services. Browser Web mode uses `/api/channel-settings` for channel config; Desktop/Tauri uses sidecar commands. Desktop dev/runtime uses the desktop config dir as the canonical location and may only promote missing values one-way from legacy `data/runtime/config_runtime.yaml`, including the OpenCode route, enabled channels, Tavily search keys, and FMP keys
- In packaged desktop mode, runtime data, locks, logs, and actor sandboxes live under the app sandbox data directory by default; the desktop host also hydrates key login-shell environment variables and exports bundled binary paths (`HONE_MCP_BIN`, bundled `opencode`, `HONE_AGENT_SANDBOX_DIR`) before starting the embedded backend or channel sidecars
- Desktop agent settings expose Hone Cloud (`agent.hone_cloud.base_url/api_key/model`), the primary opencode/OpenRouter model, `llm.profiles` bindings for background/event-engine routes, and the direct `llm.auxiliary` OpenAI-compatible fallback for session compression. `llm.openrouter.sub_model` remains only as the final legacy fallback model name for the auxiliary path
- In `bundled` mode, Tauri also starts or stops `hone-imessage` / `hone-discord` / `hone-feishu` / `hone-telegram` according to the layered runtime config in the application data directory; each channel process now posts heartbeat snapshots carrying `channel + pid` back to the console backend via `HONE_CONSOLE_URL`, and `/api/channels` aggregates those live registrations into per-channel multi-process status. Desktop channel status also merges OS process scanning so duplicate listener processes are visible even when an older instance is not bound to the current backend heartbeat registry, and the desktop shell exposes a cleanup command that keeps only one process per channel. The legacy `runtime/*.heartbeat.json` files still exist as a compatibility fallback for non-desktop paths
- Desktop log pages read from `/api/logs`; the backend route now merges the in-memory log ring with recent `data/runtime/logs/*.log` tails so bundled desktop mode can display channel/runtime logs even when they were written by sibling processes instead of the current web process
- Frontend backend runtime lives in `packages/app/src/context/backend.tsx` and `packages/app/src/lib/backend.ts`
- Assistant message parser for inline local images: `packages/app/src/lib/messages.ts`
- `hone-console-page` `/api/meta` handles version and capability negotiation
- `hone-console-page` admin app only serves `/api/*` and console SPA on the admin port; the public app serves the public SPA routes (`/`, `/roadmap`, `/blog`, `/blog/:slug`, `/chat`, `/community`, `/research-library`, `/me`, `/activate`, `/portfolio`, `/terms`, `/privacy`) plus `/api/public/*` on the public port. `/blog` is a static bilingual content surface backed by `packages/app/src/lib/public-blog.ts`, Markdown files under `packages/app/src/content/blog/`, and public images under `packages/app/public/blog/`; Cloudflare Pages metadata for Blog article sharing is injected by `packages/app/public/_worker.js` for crawlers that do not execute the SPA. Domestic `/chat` login uses SMS-verified invite-list web users. International buyers use `/activate`; after HONE-owned email verification the server creates either subscription Checkout (`mode=subscription`, USD 199.99/year) or fixed-term Checkout (`mode=payment`, USD 229.99/12 months), and only verified paid Stripe webhooks write the Billing ledger. `/api/public/auth/email/send` uses the injected sender, remains fail-closed when unconfigured, and applies per-address cooldown plus IP/address attempt budgets; `/api/public/auth/email/login` establishes the same HttpOnly session after purchase-email verification. `/api/public/auth/me` remains available to an authenticated inactive user for billing status, while paid routes enforce the Billing ledger and return `402`. Authenticated public administrators use `GET/POST /api/public/admin/invites` and `POST /api/public/admin/invites/:user_id/disable`; every request rechecks the PG role, mutations require the custom admin-action header, and the response excludes invite/API/password secrets. Startup reads `/api/public/bootstrap` for session auth/quota and the newest 20 projected messages, while older projected pages come from cursor-based `/api/public/history`. It renders non-image attachment cards through `/api/public/file`, scheduled results as summary cards with full-content drill-down, and the unified push center. “我的财经日历” calls `/api/public/finance-calendar?month=YYYY-MM`, creates and uploads desktop/mobile PNGs once, then `/api/public/finance-calendar/send` validates both and stores structured variant metadata. Bootstrap/history choose one variant from User-Agent and legacy markers, and `/api/public/image` serves it through the actor-scoped proxy with private immutable caching. `/api/public/community` exposes the authenticated HONE Official read-only timeline and `/api/public/community/resources/:resource_id` proxies only archived object-store bytes; `/api/public/community/forum*` owns the separate local member-discussion APIs and does not enter investment retrieval. `/api/public/auth/sms/send` and `/api/public/auth/sms/login` keep Aliyun SMS verification while the admin invite table remains the domestic admission source. `/api/public/v1/chat/completions` is the API-key-authenticated OpenAI-compatible public chat endpoint used by Hone Cloud clients and enforces the same Billing entitlement decision.
- The backend adds `GET /api/public/community/state` for actor-specific unread state and `POST /api/public/community/edge-session` for a short-lived edge grant. The grant response never exposes the token, secret, or actor ID; the token exists only in the scoped HttpOnly cookie. Logout clears both the normal public session cookie and the edge cookie. Existing `/api/public/community`, `/seen`, and `/resources/:resource_id` contracts remain the compatibility path.
- `hone-console-page` `/api/skills*` serves the skill management surface: registered listing, detail view, enable/disable mutation, and reset
- `hone-console-page` `/api/company-profiles*` now serves actor-space listing, portrait detail, full deletion, and actor-scoped portrait bundle transfer (`export`, `import/preview`, `import/apply`) for actor-local portrait docs; portrait creation and section/event updates still rely on runner-native file operations inside the actor sandbox rather than dedicated mutation APIs
- `packages/app/src/context/company-profiles.tsx` now acts as the memory-page transfer orchestrator: it merges portrait actor spaces with recent session users into one target-selector model, supports manual target entry for first-time imports, runs bundle preview/apply, keeps post-import highlights plus optional pre-import backup blobs, and auto-selects the first company in the current target space so the right panel does not fall back to a false empty state

## Web Console Structure

- Route entrypoint: `packages/app/src/app.tsx`
- Pages: `packages/app/src/pages/`
  - admin surface keeps `/start` and the management console routes
  - public surface exposes `/`, `/roadmap`, `/blog`, `/blog/:slug`, `/chat`, `/community`, `/research-library`, `/me`, `/activate`, `/portfolio`, `/terms`, and `/privacy`; domestic admission uses phone + SMS, while international Stripe users use HONE-owned email verification and server-authoritative Billing state
- Page-level pure state/data helpers: `packages/app/src/pages/{settings,users,notifications,task-health}-model.ts`
- Domain state: `packages/app/src/context/`
- Composite components: `packages/app/src/components/`
- API access and data transformation: `packages/app/src/lib/`

## Common Coupled Changes

- Adding a tool:
  - Change `crates/hone-tools/src/*`
  - Update the MCP bridge and any native runner adapters that expose the tool
  - If the Web UI needs to show it, also update `bins/hone-console-page/src/main.rs` and the frontend pages
- Adjusting the skill runtime:
  - Start with `crates/hone-tools/src/skill_runtime.rs`, `crates/hone-tools/src/{skill_registry.rs,skill_tool.rs}`
  - Then check `crates/hone-channels/src/agent_session/mod.rs`, `crates/hone-channels/src/core/mod.rs`, `crates/hone-channels/src/execution.rs`, `crates/hone-channels/src/sandbox.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-channels/src/mcp_bridge.rs`, and `crates/hone-channels/src/runtime.rs`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/skills.rs` and `packages/app/src/{context/skills.tsx,components/skill-*.tsx,lib/skill-command.ts}`
- Adding a Web page or dashboard:
  - Change `packages/app/src/pages/*`
  - Change `packages/app/src/context/*` and / or `packages/app/src/lib/*`
  - If the backend API is insufficient, add the Web bin API
  - SMS-based public user flows also require checking `memory/src/web_auth.rs`, `crates/hone-web-api/src/aliyun_sms.rs`, and `crates/hone-web-api/src/routes/public.rs` instead of wiring directly into the console-only `/api/chat` / `/api/history` / `/api/users` routes; API-key based Hone Cloud access additionally touches `crates/hone-web-api/src/routes/web_users.rs` and `packages/app/src/pages/settings.tsx`
- Adjusting desktop backend switching or sidecar lifecycle:
  - Change `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}`
  - If the change is process supervision, runtime env, or persisted overlay wiring, start with `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
  - Change `packages/app/src/context/backend.tsx` and / or `packages/app/src/lib/backend.ts`
  - Update `bins/hone-console-page/src/main.rs` and the runtime config loading for the channel bins if needed
- Adding channel behavior:
  - Change the matching `bins/*`
  - If the change is startup / enable checks / heartbeat / process lock wiring, start with `crates/hone-channels/src/bootstrap.rs`
  - Feishu scheduled delivery and outbound rendering now live in `bins/hone-feishu/src/{scheduler.rs,outbound.rs}`; Telegram scheduled delivery lives in `bins/hone-telegram/src/scheduler.rs`
  - Update `hone-channels`, `hone-core`, or `memory` if needed
  - If the change touches incoming envelopes, dedup, actor scope, placeholder / streaming delivery, or attachment persistence, start with `crates/hone-channels/src/ingress.rs`, `crates/hone-channels/src/outbound.rs`, and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
- Adjusting persistence structure:
  - Start with `memory/`
  - Then check the Web API, channel entrypoints, and frontend pages that depend on it
- Adjusting company portraits:
  - Start with `memory/src/company_profile/{mod,types,markdown,storage,transfer}.rs`
  - Then check `crates/hone-channels/src/sandbox.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-channels/src/core/mod.rs`, and `crates/hone-web-api/src/routes/company_profiles.rs`
  - If the Web UI is affected, also check `packages/app/src/{context/company-profiles.tsx,components/company-profile-*.tsx,pages/memory.tsx}`
- Adjusting identity quotas or limits:
  - Start with `memory/src/quota.rs` and `memory/src/cron_job/mod.rs`
  - Then check `crates/hone-channels/src/agent_session/mod.rs` and `crates/hone-channels/src/scheduler.rs`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/chat.rs`, `crates/hone-web-api/src/routes/cron.rs`, and `packages/app/src/lib/api.ts`
- Adjusting the agent execution path:
  - Start with `crates/hone-channels/src/agent_session/mod.rs`
  - Then check `crates/hone-channels/src/prompt.rs`, `crates/hone-channels/src/core/mod.rs`, and `crates/hone-channels/src/sandbox.rs`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/chat.rs` and `packages/app/src/context/sessions.tsx`
- Adjusting LLM audit:
  - Start with `memory/src/llm_audit.rs`
  - Then check `crates/hone-channels/src/core/mod.rs`, `crates/hone-channels/src/runners/*`, and legacy `agents/*` if that path is still in use

### Decision-brain shadow protocol

- Offline investment-decision evaluation also projects `hone-shadow-policy-v1`, a read-only precommitment with fixed virtual-notional, concentration, cash, rebalance, execution-delay and slippage assumptions. It selects only the newest frozen sample per symbol and emits explicit blocks for evidence-gate, action/zone, confidence, live-data, valuation, market-regime, causal-conflict/falsification and exact-decision human-review failures. It has no ledger, holding, order, broker or activation path; authorization remains `not_authorized` even when a row is eligible for protocol review. The administrator decision-brain panel renders this contract and its blocking reasons.
- `/api/public/admin/investment-decisions/shadow-protocol-governance` is the independent review boundary before any shadow implementation. It fingerprints the exact policy constraints and eight explicit review requirements, binds an approval to the current reward-governance revision, and appends immutable optimistic-concurrency records under `data/investment_decisions/governance/shadow-protocol/hone-shadow-policy-v1/`. Passing every gate only allows a later implementation registration; the API has no ledger, holdings, orders or broker integration and always reports those authorities as false.
- `/api/public/admin/investment-decisions/shadow-implementations` is that later registration boundary, not a runner. It accepts only a deterministic replay specification after exact current upstream approvals, derives a SHA-256 over the upstream revisions, policy, code revision, fixed contracts and sandbox flags, and appends immutable `registered_not_started` records under `data/investment_decisions/shadow-implementations/hone-shadow-implementation-registry-v1/`. Network, tools, production writes, ledger creation, running, orders, broker access, shadow authority and trading all remain false.
- `crates/hone-web-api/src/routes/historical_decision_anchors.rs` owns the administrator-only `hone-historical-decision-anchor-registry-v1`. It reads complete hash-verified global text through a narrow helper in `research_library.rs`, rejects excerpts not present in the source, derives stable candidate identity from the full fingerprint, serializes candidate/review writes and revalidates source plus candidate-review bindings on every registry read. Its discovery path also emits bounded, hash-bound source context and stores screening revisions as an immutable per-suggestion chain whose latest valid single tip controls routing. It stores immutable candidates and a separate confirmation/revision/rejection chain, and keeps every confirmed row benchmark-only. `packages/app/src/components/public-admin-historical-anchor-panel.tsx` exposes source coverage, contextual screening and correction, candidate entry and explicit no-hindsight confirmation without admitting anything to current training, reward, shadow or trading paths.
- The same evaluation response carries inactive `hone-reward-design-proposal-v1` and `hone-counterfactual-evaluation-v1` contracts. They expose proposed hard gates/weights and fixed SPY/cash/equal-weight/simple-rule/human-correction comparisons so later evaluation cannot choose convenient objectives after seeing results. Authorization and reward computation remain off; there is no reward write or training trigger.
- `crates/hone-event-engine/src/sec_company_facts.rs` keeps its original per-filing base events and adds separate `financial-quality-v2` supplement events for gross profit, operating income, year-to-date operating cash flow, cash and long-term debt. The split is the migration boundary: old event IDs are never rewritten, supplements do not repeat v1 metrics, exact XBRL bases remain distinct, and reruns are idempotent.
- `crates/hone-web-api/src/routes/investment_decisions.rs` joins the split SEC metric families back together only through `hone-sec-same-filing-ratio-v1`: exact URL/time/period matches can create gross and operating margin rows with both claim traces. Ratios use a separate relationship/provenance type and administrator queue filter; they never masquerade as company-reported margin language or enter promotion automatically.
- The same decision route replays v1 crowding and currently projects `hone-crowding-v2-price-path-partial` from frozen company-rating quote, valuation and market-history fields. `company_ratings.rs` preserves FMP quote fields, the Nasdaq official quote/52-week fallback, a bounded `hone-market-history-v1-nasdaq-daily-close` summary with returns/drawdown/volume, plus separate versioned short-interest, standard-month options-positioning, 14-day syndicated-news-attention and institutional-13F observation records. All four contexts are deterministic, point-in-time and explicitly non-scored because their direction, independence or disclosure timing is ambiguous; truncated/review-required or unreconciled rows are not admitted. `public-admin-decision-brain-panel.tsx` renders every scored component, all admitted background cards, the 13F lag/mixed-period disclosure and each missing evidence class. Neither crowding version can claim full measurement, pass the timing gate or alter an action.
- Current company ratings additionally preserve `hone-analyst-consensus-v1-nasdaq-observation`, and current decisions project it through `hone-crowding-v3-analyst-consensus-context`; v1/v2 snapshots remain replay-only. The parser reconciles recommendation counts and deterministic concentration plus low/consensus/high target range, while the UI labels the card as non-scored and discloses that contributor count, individual rows and timestamps are unavailable. Missing, stale or inconsistent analyst rows never become zero or a model-inferred consensus, and this context cannot change the crowding score or any authority.
- The administrator evidence-review route exposes `selection=active_batch` through `hone-investment-evidence-review-queue-v2-source-readiness` and `hone-active-review-batch-v3-source-ready-diversity`. Before ranking, each row declares whether its frozen original URL, source metadata, lifecycle and type-specific provenance are sufficient for a human to locate or recompute the displayed evidence. Every active-batch pass admits only source-ready pending point-in-time rows, then covers operating-KPI, computed-comparison, computed-ratio and source-claim contracts while preserving company/driver diversity and a two-row-per-company cap before diversity-only sparse fallbacks. It cannot inspect future prices/outcomes, actions, human effects or rewards. `selection=full_queue` preserves blocked rows, exact blockers and ready/blocked counts rather than silently dropping source debt. `POST /api/public/admin/investment-decisions/causal-review/:symbol/:sample_id` writes one exact source-verification/exclusion or causal verdict to an independent immutable optimistic-concurrency chain and deliberately leaves the company thesis/action review unchanged; replay merges that projection with any separate full-decision review.
- Offline evaluation also compiles classified causal reviews through `hone-causal-training-dataset-v3-company-source-identity-component-isolated`. Its internal example type omits actions, decision outputs, reviewer prose, all forward outcomes and reward fields. Each example derives source identities from event IDs, normalized URLs, archived content SHA-256 values and referenced claim IDs; a company–source-identity graph then assigns every transitive connected component to one deterministic train, validation or sealed-holdout split. This catches distinct event aliases and mirrored URLs for the same raw document. The API/UI expose only readiness counts, exclusions, company/source isolation status, connected-component statistics and the deterministic fingerprint. No endpoint releases holdout examples and no training trigger exists; legacy v1/v2 governance is audit-only and cannot authorize this v3 projection.
- The same route separates four training authorities. `/api/public/admin/investment-decisions/causal-dataset-governance` stores an immutable single-chain review over the exact dataset fingerprint and can only open experiment registration. `/api/public/admin/investment-decisions/causal-training-experiments` stores immutable `registered_not_run` baseline/supervised-classifier proposals with a closed sandbox contract; it exposes no runner. The registry publishes the sealed three-seed blind-evaluation gates and post-promotion drift contract. Dataset drift invalidates approval; holdout leakage, future leakage, sandbox escape and contract drift fail closed. `public-admin-decision-brain-panel.tsx` shows the current fingerprint, governance rationale, experiment registry, blind-test boundary and drift boundary while keeping training, preference learning, RL, deployment, shadow and trading off.
- The administrator causal-review path now writes `hone-causal-evidence-review-v3-source-verified-distilled`. `investment_decisions.rs` first records whether the displayed number, period, unit and context match the original source. Mismatch and insufficient-context outcomes remain auditable exclusions and cannot masquerade as rejected causal relationships. A training-eligible row additionally needs the complete distilled judgment and explicit Old Wang self-attestation after source verification. Legacy v1/v2 records remain replayable but cannot compile into the current supervised dataset or admit measurement. `public-admin-decision-brain-panel.tsx` exposes source verification and causal distillation as separate single-question stages.
- `hone-sec-margin-trend-v1` then compares compatible ratio rows across same-slot years or Q2/Q1 and Q3/Q2. Each trend nests the two source ratios, preserves percentage-point arithmetic and is validated against definition drift and future leakage. The administrator panel renders current/prior ratios, both SEC links and a separate trend-review count under the shared profit-ratio filter.
- The same decision route embeds `hone-operating-kpi-registry-v1` into each of the six AI first-principles model families. The registry is a typed measurement contract—driver mapping, issuer definition, unit, period, source priority, comparability and acceptance context—not extracted evidence. Point-in-time validation rejects semantic tampering, while `packages/app/src/components/public-admin-decision-brain-panel.tsx` renders the applicable registry and cross-company restriction on each frozen sample.
- Administrator evaluation also projects `hone-first-principles-hypothesis-map-v1-latest-point-in-time`. It first selects the newest valid sample per normalized company, then groups the existing six model families and reports demand/effective-supply/value-capture coverage plus promotion/conflict/rejection/falsification state. `public-admin-decision-brain-panel.tsx` renders this as an evidence-gap map with a deterministic fingerprint; opportunity ranking and action authorization are structurally disabled.

## Fragile Areas / Notes

- `docs/technical-spec.md` is aligned with the current Rust implementation, but if module boundaries or default wiring change again, it still needs to be kept in sync so it does not drift
- Channel runners now start from a repo-external sandbox root by default; if a CLI starts reading higher-level repo rule files again, check `crates/hone-channels/src/sandbox.rs` and the runner `cwd` / config injection logic first
- `ChatMode` only means "this message came from a direct chat or a group chat"; do not treat it as the source of truth for session ownership. Use `SessionIdentity` for shared group context.
- Telegram / Discord / Feishu now gate direct-vs-group ingress through per-channel `chat_scope` (`DM_ONLY | GROUPCHAT_ONLY | ALL`), while group chats still share one model: untriggered text is buffered in a short pretrigger window, and only an explicit `@bot` / reply-to-bot trigger flushes that buffered text into the shared group session before `AgentSession::run()`.
- Group explicit triggers now expose a busy lifecycle: if one group session is still processing, the next explicit trigger gets an immediate “wait for the previous message” reply and its text is re-buffered into the pretrigger window instead of starting a second concurrent run.
- Scripts in `tests/regression/manual/` depend on local environment state or external accounts and must not be promoted to default CI gates; wrappers that call real services, send messages, or consume provider quota should skip by default unless their `RUN_*_LIVE_SMOKES=1` gate is set
- iMessage capabilities depend on local macOS permissions and cannot be assumed to work in CI or on non-macOS environments
- Desktop packaging depends on a local Rust + Tauri toolchain; if `cargo` or `bun` is missing, only static changes are possible, not a full compile verification
- Default repo-wide Rust verification should keep using `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`; each macOS app has a separate packaging validation lane.
- Validate the public user app itself with `cargo test -p hone-user-app`, `cargo check -p hone-user-app`, and `bash scripts/build_user_app.sh`; inspect the resulting bundle before distribution to ensure it remains sidecar-free.
- For local IDE / syntax checks on the desktop crate itself, use `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop` so Tauri skips bundled sidecar existence validation while still type-checking Rust code.
- Real desktop packaging validation must still use the generated Tauri config / prepared sidecars path (`bun run tauri:prep:*` + `bunx tauri dev/build`); the skip flag is not a substitute for release-time resource checks.
- `opencode_acp` now treats the user's local OpenCode config as the default source of provider/auth/model truth. The Hone runner may still inject a small custom `OPENCODE_CONFIG` for ACP permissions and explicit `agent.opencode.*` overrides, but it should not hide `~/.config/opencode/opencode.json` / `opencode.jsonc` by replacing the entire OpenCode config home.

## Decision-brain Global Evidence Isolation

- `crates/hone-web-api/src/routes/investment_decisions.rs` partitions global source-claim lifecycles, operating-KPI lifecycles, comparisons, same-filing ratios and ratio trends by normalized company symbol. Pair builders also reject cross-company rows, so administrator evaluation and future causal-training governance cannot manufacture issuer-crossed conflicts, comparisons or trends. This isolation repair changes evidence diagnostics only; it does not create a review label, action, reward, shadow holding or execution authority.
- `crates/hone-web-api/src/routes/historical_decision_anchors.rs` verifies hash-bound transcript excerpts and keeps AI/administrator candidates separate from Old Wang confirmation; its discovery queue scans explicit action language, preserves exact source/line binding and bounded surrounding context, and suppresses unsafe action prefill for conflicts, negation, third-party quotes and audience questions. A deterministic review batch ranks at most five pending rows using unique dominant speaker labels, explicit first-person context, risk exclusions and source/company diversity. Administrator triage appends a separate screening correction chain bound to the source, excerpt and prior tip; stale, unchanged, branched or disconnected histories fail closed, while only the latest valid tip controls routing. Its three verdicts still cannot create a candidate or confirm identity, action or logic, and continue verdicts only form a shortlist for the manual candidate form. Explicit other-company references stay in the full queue but not the default batch. The dominant label is never treated as identity, and no future/outcome/reward field is an input. Accepted candidate reviews additionally carry the exact decision-availability timestamp. `crates/hone-web-api/src/routes/historical_state_reconstructions.rs` then owns the independent seven-layer point-in-time reconstruction and its immutable benchmark review. It publishes a fixed but disabled 20/60/250-session SPY outcome protocol. `crates/hone-web-api/src/routes/historical_outcome_governance.rs` separately fingerprints and reviews that protocol before any labeler implementation can even be registered for review. `crates/hone-web-api/src/routes/historical_outcome_labeler_registry.rs` owns the next immutable implementation-specification and human-review chains, binds code revision and protocol fingerprint, and can only expose eligibility for a later offline dry-run authorization review; implementation running, outcome generation, training, reward, shadow and trading authority remain false.
- `packages/app/src/components/public-admin-historical-anchor-panel.tsx`, `packages/app/src/components/public-admin-historical-state-reconstruction-panel.tsx` and `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` expose three separate administrator workflows inside `public-admin-decision-brain-panel.tsx`: inspect exact surrounding transcript context, append a reasoned triage correction without rewriting history, confirm the exact historical judgment, reconstruct evidence or explicit missingness for every historical layer, then review the frozen future-outcome protocol without generating labels.
- `crates/hone-web-api/src/routes/historical_outcome_price_snapshots.rs` owns the next two non-executing boundaries. The first ingests FMP adjusted closes into immutable, hash-bound asset/SPY input snapshots only after an exact approved historical reconstruction and reviewed labeler implementation exist; it records common-session coverage for the frozen 20/60/250 horizons but computes no return, drawdown or label. The second stores an independent optimistic dry-run-authorization review bound to that exact snapshot, upstream revisions, protocol fingerprint and code revision. Approval exposes only later dry-run implementation-registration eligibility. `investment_decisions.rs` no longer calls its legacy outcome labeller from company-rating or key-event refreshes, so ordinary product refresh cannot fetch future prices or rewrite historical labels. The administrator governance panel renders snapshot and authorization status while keeping running, label writes, training, reward, shadow and trading false.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_implementations.rs` owns the next create-once, still non-running boundary. It server-projects the exact approved authorization, sealed snapshot/series, reconstruction, labeler and protocol identities into one fingerprinted deterministic isolated-replay specification. Every runtime, write, training, reward, shadow, order, broker and trading permission is false and status is fixed to `registered_not_run`. The administrator governance panel can register and inspect this object; `investment_decisions.rs` exposes it only as the seventh empirical-readiness stage, not as execution or outcome evidence.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_run_authorizations.rs` owns the eighth, still non-running boundary. Its append-only reviews bind one current `registered_not_run` implementation and every upstream fingerprint, carry their own SHA-256 plus the previous review hash, and require explicit reproducibility, isolation, resource and zero-authority checks. Approval exposes only eligibility to register a future isolated runner for another review; it cannot invoke code, create an output artifact, admit a label, train, reward, shadow, order, access a broker or trade.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_isolated_runners.rs` owns the ninth, still non-running boundary. It creates one immutable runner specification from an exact current approved run review and binds the runner artifact SHA-256, code revision, upstream implementation and all historical evidence identities. The fixed runtime contract has no callable entrypoint, no inherited environment or secrets, no network/tools/production writes, a read-only root/input boundary and bounded ephemeral output. Registration exposes only eligibility for a later first-execution authorization review; it cannot invoke the artifact or create an output.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_first_execution_authorizations.rs` owns the tenth, still non-invoking boundary. Its append-only, hash-linked review chain re-projects one exact current runner, artifact digest, code revision, upstream evidence identities and fixed sandbox/resource limits. The runner registrant cannot approve its own first execution. Approval is valid for 24 hours and at most one future invocation, but this module exposes no invocation endpoint, starts no process and creates no output. Expiry, stale bindings, missing checks, hash drift, self-approval, forks, cycles or downstream write/trading authority fail closed.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_execution_attempts.rs` owns the eleventh, one-shot execution boundary. It consumes one exact current unexpired authorization, re-hashes the running backend artifact, reloads the exact sealed snapshot/upstream chain and writes an immutable claim before computation. The only execution backend is a statically bounded pure function over sealed common-session prices; arbitrary code, environment, network, tools, child processes and production/downstream writes are unavailable. The host stages bounded JSON in a unique temporary directory, verifies and hashes the read-back, records real cleanup state and writes an immutable success-or-failure result. All computed metrics remain untrusted and wait for an independent validation/recomputation chain; labels, training, reward, shadow, orders, broker and trading remain closed. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` exposes the current backend digest, one-shot action and claim/result audit view, while `investment_decisions.rs` presents this as empirical-readiness stage eleven rather than outcome evidence.
- `crates/hone-web-api/src/routes/historical_outcome_dry_run_output_validations.rs` owns the twelfth, independent output-validation boundary. It reads one exact completed attempt and current sealed upstream chain, requires a validator distinct from the invoker, runner registrant and both authorization reviewers, verifies the canonical output/hash/provenance/closed capabilities, then independently reconstructs common sessions and recomputes every frozen metric without calling the execution implementation. Its create-once self-hashed pass/fail record is exposed through the administrator governance panel and readiness v9. A pass still cannot admit a label, train, reward, shadow, order, access a broker or trade.
- `crates/hone-web-api/src/routes/historical_outcome_label_admission_reviews.rs` owns the thirteenth, still non-materializing boundary. It accepts only an exact current independently validated output, requires a reviewer distinct from every validation/execution/registration/authorization role, and appends a self-hashed applicability-and-bias review bound to all output, snapshot and protocol identities. Approval requires explicit horizon, adjusted-close/corporate-action, SPY, future-isolation, missingness/sample-selection/survivorship, no-manual-override, no-semantic-inference and zero-downstream-authority checks plus rationale and limitations. It exposes only future label-materialization eligibility; no label is written and training, reward, shadow, orders, broker access and trading stay closed. The administrator governance panel owns its form/audit view.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_implementations.rs` owns the fourteenth, still non-running boundary. It server-projects one exact current admitted output and all review/validation/claim/result/snapshot/reconstruction/protocol hashes into a create-once content-addressed raw-outcome-envelope specification. The only output fields are the bitwise-preserved validated returns, drawdown, provenance and known limitations; metric override and direction/rating/action/position/reward inference are forbidden. Registration grants only later run-authorization-review eligibility. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` exposes the registration/audit surface, while `investment_decisions.rs` presents readiness v11 stage fourteen and retains the run/materialization/label blocker.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_run_authorizations.rs` owns the fifteenth, still non-running boundary. Its append-only self-hashed review chain re-projects one exact current materializer and all admission/validation/output/snapshot/protocol/code/limitation bindings, while excluding the implementation registrant and every relevant prior chain actor from approval. Approval exposes only future isolated materialization-runner registration eligibility. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx` owns the review/audit form and `investment_decisions.rs` exposes readiness v12 stage fifteen; runner registration, execution, label writing, training, reward, shadow, orders, broker and trading remain closed.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_isolated_runners.rs` owns the sixteenth, still non-running boundary and the GET/POST `/admin/investment-decisions/historical-outcome-label-materialization-isolated-runners` registry. It projects one exact current stage-fifteen approval and every upstream identity into a create-once content-addressed runner specification, binding artifact SHA-256, immutable code revision, read-only input/root, ephemeral work/output boundaries, fixed resources and no environment/secrets/network/tools/production/downstream capabilities. The record has no callable entrypoint and exposes only later first-execution-review eligibility. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/lib/api.ts` and `packages/app/src/lib/types.ts` own its registration/audit client surface; `investment_decisions.rs` exposes readiness v13 stage sixteen while execution, labels, training, reward, shadow, orders, broker and trading remain closed.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_first_execution_authorizations.rs` owns the seventeenth, still non-invoking boundary and the GET/POST review routes under `/admin/investment-decisions/historical-outcome-label-materialization-first-execution-authorizations`. Its append-only self-hashed chain re-projects one exact current runner, artifact/code/resources and the complete prior chain, excludes every relevant registrant/reviewer/invoker from self-approval and grants at most one future invocation for 24 hours. The module exposes no invocation endpoint and cannot claim, consume, run, materialize, create output or write a label. The administrator governance panel, API/types clients and decision-brain stage card expose the audit/readiness surface; `investment_decisions.rs` exposes readiness v14 stage seventeen while training, reward, shadow, orders, broker and trading remain closed.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_execution_attempts.rs` owns the eighteenth one-shot fixed-projection boundary and the GET registry plus POST `/admin/investment-decisions/historical-outcome-label-materialization-execution-attempts/{isolated_runner_id}/invoke-once` routes. It accepts only one exact current unexpired and unclaimed stage-seventeen authorization, re-hashes the runtime artifact and revalidates the complete chain, persists a create-once claim before projection, then uses a no-ambient-capability pure function to bitwise-copy validated 20/60/250 metrics, provenance and limitations into a create-once staged envelope. Success and failure both consume authorization and append immutable results. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/lib/api.ts` and `packages/app/src/lib/types.ts` expose the exact action/audit surface; `investment_decisions.rs` exposes readiness v15 stage eighteen. Every output remains untrusted, not a label, pending a later independent structural/provenance/bitwise validation; training, reward, shadow, orders, broker and trading remain closed.
- `crates/hone-web-api/src/routes/historical_outcome_label_materialization_output_validations.rs` owns the nineteenth create-once independent validation boundary and the GET registry plus POST `/admin/investment-decisions/historical-outcome-label-materialization-output-validations/{attempt_id}/validate` routes. It reloads one exact completed stage-eighteen attempt, current admitted source and full immutable chain, excludes the materialization and original execution actor set, verifies canonical structure/provenance/limitations and compares all 20/60/250 metrics bitwise without reusing the stage-eighteen projection. `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/lib/api.ts` and `packages/app/src/lib/types.ts` expose the exact validation/audit action; `investment_decisions.rs` exposes readiness v16 stage nineteen and removes already validated attempts from the effective pending count. Passing still does not create a formal label or enable label writing, training, reward, shadow, orders, broker or trading.
- `crates/hone-web-api/src/routes/historical_outcome_label_write_authorizations.rs` owns the twentieth append-only formal-label-write authorization review and GET registry plus POST `/admin/investment-decisions/historical-outcome-label-write-authorizations/{validation_id}/review` route. It reopens one exact current stage-nineteen passing envelope, binds every validation/source/protocol/metric hash and a fixed raw-outcome-only formal-label contract, excludes the complete prior actor chain and grants at most one future create-once write for 24 hours. The module has no writer endpoint, does not consume the allowance and writes no label. The administrator panel/API/types expose review and audit; `investment_decisions.rs` exposes readiness v17 stage twenty while all training, reward, shadow, order, broker and trading authority stays closed.
- `crates/hone-web-api/src/routes/historical_outcome_formal_label_writes.rs` owns the twenty-first fixed formal raw-label writer and GET registry plus POST `/admin/investment-decisions/historical-outcome-formal-label-writes/{authorization_review_id}/write-once` route. It revalidates one exact current stage-twenty allowance, writes a create-once claim before mutation, consumes the allowance on success, failure or interruption, and stores a create-once label with exactly the eight frozen raw outcome metrics outside training and reward stores. `historical_outcome_label_write_authorizations.rs` projects claimed allowances as consumed. The administrator panel/API/types expose invocation and immutable claim/label/failure audit; `investment_decisions.rs` exposes readiness v18 actual counts while training, reward, shadow, order, broker and trading authority remains closed.
- `crates/hone-web-api/src/routes/historical_outcome_formal_label_validations.rs` owns the twenty-second create-once independent formal-label validation and GET registry plus POST `/admin/investment-decisions/historical-outcome-formal-label-validations/{label_id}/validate` route. It reopens the exact stage-twenty approval and current full source chain, excludes the writer and all prior actors, independently verifies canonical hashes, the fixed eight fields, provenance, limitations and the bitwise/hash identity of all 20/60/250 metrics without reusing writer validation code. A pass adds only an immutable offline-training-dataset-candidate admission record. The administrator panel/API/types expose the exact action and audit, while `investment_decisions.rs` exposes readiness v19 and keeps training storage, dataset assembly/versioning, training, reward, shadow, order, broker and trading closed.
- `crates/hone-web-api/src/routes/historical_outcome_offline_datasets.rs` owns the twenty-third immutable raw-outcome dataset registry and GET/POST `/admin/investment-decisions/historical-outcome-offline-datasets` route. It assembles only the complete current stage-twenty-two passing set, hashes candidate bindings, entries, content and manifest, rejects duplicate/conflicting point-time identities, and forces later versions to preserve the full previous prefix while appending only new candidates. Objects live in the isolated `investment_decisions/historical_outcome_offline_datasets/objects` store and contain no features, targets or split assignments. The administrator panel/API/types expose guarded assembly and the version audit; `investment_decisions.rs` exposes readiness v20 while dataset governance, training, reward, shadow, order, broker and trading stay closed.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_governance.rs` owns the twenty-fourth independent offline-dataset governance registry plus GET `/admin/investment-decisions/historical-outcome-offline-dataset-governance` and POST `/{dataset_id}/review`. It appends content-addressed reviews under `investment_decisions/historical_outcome_offline_dataset_governance/reviews`, binds the exact current stage-twenty-three dataset and complete actor exclusions, freezes company/event/source connected-component isolation, deterministic 70/15/15, 250-session purge/embargo and sealed holdout rules, and separately freezes strict point-in-time feature provenance/exclusion rules. Approval exposes only future transformation-spec registration eligibility. `investment_decisions.rs` reports readiness v21, while `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/lib/api.ts` and `packages/app/src/lib/types.ts` provide the guarded review and audit UI; no split, feature join, target, training, reward, shadow, order, broker or trading capability exists.

## Decision-brain Shared Chat Projection

- `crates/hone-core/src/investment_decision_context.rs` owns the small, versioned and fail-closed `hone-investment-decision-chat-context-v1` contract shared across crates. It validates time windows, bounded research fields, valuation shape, completeness and the absence of portfolio/shadow/trading authority.
- `crates/hone-web-api/src/routes/investment_decisions.rs` remains the owner of the full decision and writes `data/investment_decisions/chat-context/current/<SYMBOL>.json` only after the source snapshot passes point-in-time validation. The sidecar is regenerated with every current decision and may be backfilled by reading existing current decisions.
- `crates/hone-channels/src/prompt.rs` resolves only symbols from the embedded research index, reads the matching bounded sidecars under the configured storage root and renders fresh, stale or invalid states without leaking file errors. `crates/hone-channels/src/turn_builder.rs` places that state next to the user question for both function-calling and native-Codex paths; historical company cards remain a separate, non-current baseline.

## Decision-brain Operating-KPI Source Artifacts

- `crates/hone-event-engine/src/operating_kpi_claim.rs` owns the six-model symbol/KPI/driver catalog and the content-addressed source-artifact contract. V2 events without a valid full-file SHA-256, extracted-text SHA-256, byte length, format and exact object path admit no operating claims.
- `crates/hone-event-engine/examples/operating_kpi_backfill.rs` is dry-run-first. Writable v2 runs refetch an allowlisted official PDF/HTML without redirects, verify issuer strings verbatim, compare the pinned digest, then create the immutable object under `data/investment_decisions/source-artifacts/operating-kpi/objects/` before idempotent event insertion. `tests/fixtures/event_engine/operating_kpi_backfill_power_v2.json` is the first real non-storage fixture.
- `tests/fixtures/event_engine/operating_kpi_backfill_storage_sndk_q4fy26_v2.json` is the exact-byte storage fixture. It binds SNDK's FY2025 10-K bit-demand fact, FY2026 Q4 8-K data-center revenue/agreement facts, and FY2026 10-K NBM RPO/post-balance-sheet agreement facts. Product mix remains excluded unless a source reports that distinct denominator directly.
- `crates/hone-web-api/src/routes/investment_decisions.rs` accepts current six-model registries, preserves explicitly bounded legacy registry replay, attaches only matching operating claims to the matching driver, and carries source artifact/time precision into administrator review. Storage v4 distinguishes NBM RPO from revenue, cash, pure data-center orders and agreement count; storage v2/v3 are replay-only. `packages/app/src/components/public-admin-decision-brain-panel.tsx` exposes the original URL, SHA-256 and archive locator without treating the claim as a reviewed causal label.

## Stage 25 Offline-dataset Transformation Specification Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_specs.rs` owns the create-once registry and registration endpoint. It binds one exact current stage-twenty-four approval and server-generates the hashed connected-component split-manifest contract plus the seven-layer, 65-feature-ID point-in-time feature-bundle contract. The split contract contains a unique integer boundary objective, frozen common-session calendar, exact purge/embargo and empty-partition failure semantics. Storage is under `investment_decisions/historical_outcome_offline_dataset_transformation_specs/records`.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_governance.rs` exposes every governance-review actor so a future transformation registrar cannot overlap any dataset or governance participant, including an earlier review in the same chain.
- `crates/hone-web-api/src/routes/investment_decisions.rs` exposes empirical readiness v23. A current registration is shown as independently reviewable but not run; even an approved review opens only future isolated implementation registration, while split, join, target, training, reward, shadow, order, broker and trading authority remain closed.
- `packages/app/src/components/public-admin-historical-outcome-transformation-spec-panel.tsx` presents the exact contracts, eleven fail-closed confirmations and immutable registration audit. `public-admin-historical-outcome-governance-panel.tsx` nests it after stage twenty-four, while `public-admin-decision-brain-panel.tsx` shows the v23 readiness gate.

## Stage 26 Independent Transformation-specification Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_spec_reviews.rs` owns the append-only self-hashed independent review registry. It binds the exact stage-twenty-five record and upstream chain, excludes all dataset/governance/registrar actors, independently audits the deterministic boundary contract and the complete 65-feature catalog, and exposes read/review endpoints without any execution capability.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET/POST routes under `/admin/investment-decisions/historical-outcome-offline-dataset-transformation-spec-reviews`.
- `packages/app/src/components/public-admin-historical-outcome-transformation-spec-review-panel.tsx` presents the exact eligible specification, sixteen confirmations, review contract and immutable history. `public-admin-historical-outcome-governance-panel.tsx` nests it after stage twenty-five; `public-admin-decision-brain-panel.tsx` exposes the v23 gate and narrow authority.
- `packages/app/src/lib/api.ts` and `packages/app/src/lib/types.ts` own the client contract. No implementation registration, transformation execution, manifest/bundle output, target, training, reward, shadow, order, broker or trading API is introduced by this stage.

## Stage 35 Independent Official-artifact Output Validation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_official_artifact_output_validations.rs` owns the create-once validation registry. It reopens the current admitted source candidate, materialization claim/result and both official files, independently recomputes all fingerprints and exact-copy checks, and persists pass or mismatch as an immutable record.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET and POST routes under `/admin/investment-decisions/historical-outcome-offline-dataset-transformation-official-artifact-output-validations`.
- `crates/hone-web-api/src/routes/investment_decisions.rs` exposes empirical readiness v32. A validated pair enables only future feature-label join/target governance-specification registration; actual join, target, training, reward, shadow, order, broker and trading remain closed.
- `packages/app/src/components/public-admin-historical-outcome-transformation-official-artifact-output-validation-panel.tsx` presents exact pair bindings, three explicit confirmations and immutable validation history. The governance panel nests it after Stage 34, and the decision-brain panel shows the Stage 35 readiness boundary.

## Stage 36 Feature-label Join and Semantic-target Governance Specification

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_specs.rs` owns create-once specification registration. It binds one exact current Stage 35 validation, freezes one-to-one entry joins, purge/embargo exclusion, point-in-time/missingness rules, split-specific target visibility and the exact nine-component continuous outcome vector.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_official_artifact_output_validations.rs` exposes only currently valid artifact pairs for Stage 36 and reopens/re-audits the official files before eligibility is returned.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET and POST registration routes under `/admin/investment-decisions/historical-outcome-feature-label-join-target-specs`; `investment_decisions.rs` exposes readiness v33 without enabling execution.
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-spec-panel.tsx` presents the target vector, eleven explicit confirmations, immutable records and zero-authority boundary. The historical governance panel nests it after Stage 35, and the decision-brain panel shows Stage 36 status.

## Stage 37 Independent Feature-label Join / Target Specification Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_spec_reviews.rs` owns the append-only self-hashed review chain. It independently reproduces registration, body, join and target fingerprints; rebinds the current official artifacts and 65-feature catalog; and audits exact join, leakage, missingness, holdout and nine-target semantics without reusing the Stage 36 validator.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_specs.rs` exposes only current, fully rebound specifications and artifact pairs for review. The review actor must be absent from the complete registration/upstream chain and every prior review.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET/POST review routes under `/admin/investment-decisions/historical-outcome-feature-label-join-target-spec-reviews`; `investment_decisions.rs` exposes readiness v34. Approval permits only a future isolated implementation registration, never join execution or training.
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-spec-review-panel.tsx` presents the independent audit, thirteen confirmations, engineering-candidate target warning and immutable history. The governance panel nests it after Stage 36, and the decision-brain panel shows Stage 37 status.

## Stage 38 Isolated Feature-label Join / Target Implementation Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_implementations.rs` owns create-once immutable implementation registration. It binds the exact current Stage 37 approval and freezes the artifact/revision, one-to-one join, exact-bit nine-target projection, serializer, schemas and zero-capability sandbox without exposing an entrypoint.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_spec_reviews.rs` exposes only currently rebound approved reviews for implementation registration. The implementation registrar must be absent from the complete upstream and review actor set.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET/POST routes under `/admin/investment-decisions/historical-outcome-feature-label-join-target-implementations`; `investment_decisions.rs` exposes readiness v35. Registration permits only future independent implementation review.
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-implementation-panel.tsx` presents exact bindings, eleven confirmations, immutable records and the zero-authority boundary. The governance panel nests it after Stage 37, and the decision-brain panel shows Stage 38 status.

## Stage 39 Independent Feature-label Join / Target Implementation Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_implementation_reviews.rs` owns the append-only self-hashed review chain and an independent fingerprint, semantic and sandbox audit. It reopens the current Stage 38 implementation and complete upstream binding, rejects role overlap or drift, and treats approval as terminal.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET/POST routes under `/admin/investment-decisions/historical-outcome-feature-label-join-target-implementation-reviews`; `investment_decisions.rs` exposes readiness v36. Approval permits only future isolated runner-spec registration.
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-implementation-review-panel.tsx` presents the independent audit, thirteen confirmations, immutable history and engineering-target warning. It exposes no runner, execution, join, training or trading action.

## Stage 40 Isolated Feature-label Join / Target Runner Specification

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_isolated_runners.rs` owns create-once, content-addressed `registered_not_run` runner specifications. It binds the current Stage 39 approval and complete upstream chain, freezes runner artifact/revision, fixed runtime, read-only inputs, create-once output and static resource limits, and exposes no callable entrypoint.
- `crates/hone-web-api/src/routes/mod.rs` exposes administrator GET/POST routes under `/admin/investment-decisions/historical-outcome-feature-label-join-target-isolated-runners`; `investment_decisions.rs` exposes readiness v37. Registration permits only a future independent first-execution authorization review.
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-isolated-runner-panel.tsx` presents exact bindings, eight confirmations, immutable records and the permanent zero-authority boundary. It provides registration, not execution.

## Suggested Reading Order

1. `AGENTS.md`
2. `docs/repo-map.md`
3. `docs/invariants.md`
4. `docs/current-plan.md`
5. The matching `docs/current-plans/*.md`
6. The relevant entry files and tests
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_implementations.rs`
  - 第 27 阶段隔离转换实现规范登记：精确绑定第 26 阶段批准，冻结实现工件/代码/算法/序列化/schema/沙箱，create-once 且无调用入口；只开放未来独立实现复核资格。
- `packages/app/src/components/public-admin-historical-outcome-transformation-implementation-panel.tsx`
  - 管理员登记及历史视图，展示可登记批准、当前绑定、工件摘要和全部零执行边界。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_transformation_implementation_reviews.rs`
  - 第 28 阶段隔离转换实现独立复核：追加式自哈希链，独立审计工件/代码/算法/序列化/schema/资源与零能力沙箱；批准只开放未来 runner 规范登记资格。
  - 第 29 阶段隔离转换 runner 规范登记：`historical_outcome_offline_dataset_transformation_isolated_runners.rs` 以 create-once 内容寻址记录冻结当前批准实现、runner 工件/代码、固定运行时、只读输入、create-once 输出和资源上限；无调用入口，唯一下一门禁是独立首次执行授权复核。
  - 第 30 阶段隔离转换首次执行授权复核：`historical_outcome_offline_dataset_transformation_first_execution_authorizations.rs` 为精确当前 runner 建立追加式自哈希独立复核链；批准只给 24 小时内最多一次的未来隔离调用资格，注册表无 claim/调用入口且不执行、不创建输出或开放下游权限。
  - 第 31 阶段隔离转换一次性执行：`historical_outcome_offline_dataset_transformation_execution_attempts.rs` 只消费当前未过期未 claim 的精确授权，claim 前重开完整绑定并重算制品，claim 后用固定纯函数生成待独立校验的切分/显式缺失特征候选；成功或失败都消费授权，正式 manifest/bundle、训练、奖励、影子和交易权限保持关闭。管理端入口在 `public-admin-historical-outcome-transformation-execution-attempt-panel.tsx`。
  - 第 32 阶段离线转换输出独立重算：`historical_outcome_offline_dataset_transformation_output_validations.rs` 为精确 claim/result/output 建立一次性不可变校验，重开当前链并以图遍历独立重算连通分量、边界、purge/embargo、65 项显式缺失与 canonical hash；通过仍不是正式工件。管理端入口在 `public-admin-historical-outcome-transformation-output-validation-panel.tsx`。
  - 第 33 阶段离线转换候选独立准入：`historical_outcome_offline_dataset_transformation_candidate_admission_reviews.rs` 为精确已验证候选建立追加式自哈希复核链，只开放未来 create-once 正式 manifest/feature bundle 物化资格；准入、物化与正式产物输出校验保持三门分离。管理端入口在 `public-admin-historical-outcome-transformation-candidate-admission-panel.tsx`。
  - 第 34 阶段正式工件一次性物化：`historical_outcome_offline_dataset_transformation_official_artifact_materializations.rs` 先写不可覆盖 claim，再把精确准入候选复制成内容寻址的 official split manifest 与 official feature bundle；成功后仍标记为待独立校验，不开放 join、target、training 或交易。管理端入口在 `public-admin-historical-outcome-transformation-official-artifact-materialization-panel.tsx`。
  - 管理员界面：`public-admin-historical-outcome-transformation-isolated-runner-panel.tsx` 展示批准绑定、固定零能力合同、登记表和八项确认；不提供运行按钮。
- `packages/app/src/components/public-admin-historical-outcome-transformation-implementation-review-panel.tsx`
  - 管理员独立实现复核面板，提交精确哈希与十二项确认并展示不可变复核历史，不提供运行入口。

## Stage 41 Feature-label Join / Target First-execution Authorization Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_first_execution_authorizations.rs`
  - 为精确当前 Stage 40 runner 建立追加式、自哈希、24 小时、最多一次的未来隔离调用授权复核链。
  - 重绑 runner/实现/两级独立复核/spec/body/join/target/正式工件/数据集完整摘要；复核人排除完整历史角色链。
  - 注册表没有 claim 或 invocation endpoint，不读取通用标签/训练库，不执行 join、目标分配或创建 joined/training rows。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-first-execution-authorization-panel.tsx`
  - 管理员查看当前 runner、完成十六项独立确认并追加不可变复核；只显示下一门禁候选，不提供执行按钮。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v38 区分待复核、已复核、未过期单次资格与下一门禁候选；所有训练、奖励、影子和交易权限继续关闭。

## Stage 42 One-shot Feature-label Join / Target Execution Attempt

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_execution_attempts.rs`
  - 先 create-once 消费精确 Stage 41 授权，再以固定纯函数连接当前 raw outcome、独立校验 official split 和 65 项 feature bundle；失败也消费且不可重放。
  - train 仅输出九项原始 f64 位模式，validation/sealed holdout 只输出目标承诺；候选始终不可信，所有训练与交易权限关闭。
- `crates/hone-web-api/src/routes/mod.rs`
  - 提供管理员只读登记表与按 runner 的 `invoke-once` 路由；没有通用标签库或训练库入口。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-execution-attempt-panel.tsx`
  - 展示四项不可逆确认、单次领取、执行历史、候选/隐藏目标计数与下一阶段独立校验边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v39 区分可领取授权、已消费尝试、失败、未验证候选和待独立输出校验；候选成功仍不授权训练或交易。

## Stage 43 Independent Feature-label Join / Target Output Validation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_output_validations.rs`
  - 对精确已结束 Stage 42 attempt 建立 create-once、自哈希校验记录；独立重算 claim/result/output 指纹、一对一连接、65 项 PIT 特征、official split/purge/embargo、九项原始 f64 位目标及 validation/sealed-holdout 承诺，不调用 Stage 42 投影或记录校验 helper。
  - 校验人排除执行调用人和完整上游角色链。通过结果仍是 `validated_untrusted_candidate_for_future_admission_review`，只开放未来独立候选准入复核资格。
- `crates/hone-web-api/src/routes/mod.rs`
  - 提供管理员只读 registry 与按 attempt 的 `validate` 路由；不提供正式物化、训练或交易入口。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-output-validation-panel.tsx`
  - 展示精确候选、四项独立确认、不可变校验历史、目标隐藏与零下游权限边界；治理面板将其置于 Stage 42 之后。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v40 与 ㊸ 状态卡区分待校验、失败、独立验证候选和未来准入复核资格；训练、奖励、影子与交易权限继续关闭。

## Stage 44 Independent Feature-label Join / Target Candidate Admission Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_candidate_admission_reviews.rs`
  - 对精确 Stage 43 通过候选建立追加式、自哈希且批准终止的准入复核链；十二项确认覆盖完整哈希绑定、角色隔离、重算行数/排除行数、目标承诺、固定 65 项特征与 9 项目标，以及正式物化和训练继续分门。
  - 复核人排除 Stage 43 校验、Stage 42 执行、完整上游和此前复核角色。批准只开放未来 create-once official joined dataset 物化资格，不创建任何正式或训练数据。
- `crates/hone-web-api/src/routes/mod.rs`
  - 提供管理员只读 registry 与按 attempt 的 `review` 路由；没有 materialize、training、reward、shadow 或 trading 入口。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-candidate-admission-panel.tsx`
  - 展示精确候选、十二项确认、追加式复核历史和批准终端状态，并明确“准入不是正式数据集”；治理面板将其置于 Stage 43 之后。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v41 与 ㊹ 状态卡区分待复核、驳回/要求修改、已准入和未来正式数据集物化资格；训练、奖励、影子与交易权限继续关闭。

## Stage 45 Create-once Official Joined Dataset Materialization

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_official_dataset_materializations.rs`
  - 提供 GET registry 与 POST `/admin/investment-decisions/historical-outcome-feature-label-join-target-official-dataset-materializations/{attempt_id}/materialize-once`。
  - 先不可变写 claim，再精确重开 Stage 44 admission 与完整 Stage 43/42 上游绑定；成功、失败或中断都消费资格，禁止重放、修补和覆盖。
  - 只复制 rows、purge/embargo 排除审计及目标承诺；validation/sealed-holdout 目标继续隐藏。成功产物仍标记未独立校验、不可复制训练库且全部下游权限关闭。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-official-dataset-materialization-panel.tsx`
  - 管理端展示已准入/可物化/已完成/待独立校验计数、五项不可逆确认、claim/result 审计及“正式数据集不是训练数据”边界。
  - 治理面板接在 Stage 44 后；决策大脑 ㊺ 状态卡与 readiness v42 显示物化资格、终态和下一独立校验阻断项。

## Stage 46 Independent Official Joined Dataset Output Validation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_official_dataset_output_validations.rs`
  - 独立重开 Stage 45 claim/result/official dataset 与 Stage 44 当前准入候选，重算三层工件、rows、excluded rows、target commitments 和完整结构边界。
  - 每个 attempt 只允许一条不可变校验记录；角色重合、重放、绑定/哈希/计数/可见性漂移均失败关闭。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-official-dataset-output-validation-panel.tsx`
  - 四项人工确认、校验队列、不可变结果、失败原因和零下游权限展示。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
- `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 治理面板接在 Stage 45 后；决策大脑 ㊻ 状态卡与 readiness v43 区分待校验、失败、独立通过和未来训练库复制准入复核资格。

## Stage 47 Independent Training-store Copy Admission Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_admission_reviews.rs`
  - 为精确 Stage 46 通过数据集建立追加式、自哈希且批准终止的独立准入链；重绑 materialization、output validation、admission/source validation、数据集、行、排除项和目标承诺。
  - 十二项确认覆盖 65 项特征、9 项原始目标、点时/缺失、切分隔离、目标隐藏、无动作/reward 语义及复制后独立校验；批准只产生未来 create-once copy 资格。
- `crates/hone-web-api/src/routes/mod.rs`
  - 提供管理员 GET registry 与按 attempt 的 POST `review`；不存在 copy、training、reward、shadow 或 trading 入口。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-training-store-copy-admission-panel.tsx`
  - 展示精确数据集、十二项检查、复核依据/局限、追加链和终端批准边界，并明确“准入不是复制，更不是训练”。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v44 与 ㊼ 状态卡区分待复核、退回/拒绝、已准入和未来 create-once copy 资格；全部下游权限继续关闭。

## Stage 48 Claim-first Create-once Training-store Copy

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_store_copies.rs`
  - 对精确 Stage 47 准入先写不可变 claim，再把正式 joined dataset 原样复制到 `investment_decisions/isolated-training-store/feature-label-join-target-copies/{attempt_id}`；失败也消费资格。
  - 只允许精确目标目录写入，通用训练存储读写、重算、修补、插补、训练登记/运行及所有交易能力保持关闭。
- `crates/hone-web-api/src/routes/mod.rs`
  - 管理员 GET registry 与按 attempt 的 POST `copy-once` 路由。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-training-store-copy-panel.tsx`

## Stage 49 Independent Post-copy Validation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations.rs`
  - 为每个已完成 Stage 48 副本建立一次性、自哈希独立校验记录。
  - 独立重算 copy claim/result/dataset、rows、excluded rows 与 target commitments，并和精确 Stage 47 正式数据集逐行逐位核对。
  - 通过只开放未来 training-registration admission review；训练、奖励、影子、订单、券商和交易仍关闭。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-training-store-copy-output-validation-panel.tsx`
  - 管理员选择副本、确认四项边界并提交不可变复制后校验。
- `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 显示决策大脑 ㊾ 状态卡和 readiness v46 的失败关闭计数。

## Stage 50 Independent Training-registration Admission Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_registration_admission_reviews.rs`
  - 为每条当前有效的 Stage 49 校验建立追加式、自哈希、批准终止的独立训练登记准入复核链。
  - 十二项确认覆盖完整链、不可变指纹、复制与独立校验、65-feature、9-target、PIT/missingness、split/purge/embargo、目标可见性和零下游权限。
  - 批准只开放未来 create-once 训练实验登记资格；不登记、不授权或启动训练，也不开放 reward、shadow、order、broker 或 trading。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 50 管理员 registry GET 与按 attempt 提交 review 的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-training-registration-admission-panel.tsx`
  - 提供十二项独立复核确认、结论、理由、已知限制和不可变复核历史。
- `packages/app/src/components/public-admin-governance-panel.tsx`
  - 将 Stage 50 面板接在 Stage 49 之后，保持训练登记准入与训练登记/运行分离。
- `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 显示决策大脑 ㊿ 状态卡与 readiness v47 的待复核、驳回和未来一次性登记资格。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v47 只有在 Stage 50 已独立准入且 future create-once registration eligible 时才解除本门阻塞；训练与交易权限仍关闭。

## Stage 51 Claim-first Create-once Training Experiment Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations.rs`
  - 对每条当前有效的 Stage 50 准入先写不可变 claim，再且仅再创建一次训练实验登记；成功、失败或中断均消费资格。
  - 固定零预测基线、岭回归多目标模型、梯度提升多目标模型三种实验臂及 17/29/43 三组确定性种子，绑定 65 项特征、九项原始连续结果目标、逐目标逐种子指标与固定资源上限。
  - 登记状态只能为 `registered_not_run`；不提供 runner、训练授权/启动、标量 reward、动作/仓位/排名、shadow、order、broker 或 trading。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 51 管理员 registry GET 与按 attempt 进行 claim-first create-once 登记的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-training-experiment-registration-panel.tsx`
  - 提供八项边界确认、可证伪研究假设、已知局限及固定三臂三种子实验合同；明确“登记 ≠ 训练运行”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 51 面板接在 Stage 50 之后，保持登记、独立登记复核、runner、运行授权和训练执行分门。
- `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 显示 Stage 51 状态卡，区分待 claim、claim 已消费但失败、`registered_not_run` 与待独立复核。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v48 纳入 Stage 51 登记状态，同时明确登记完成仍是阻断项；训练、奖励、影子和交易权限继续关闭。

## Stage 52 Independent Training Experiment Registration Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registration_reviews.rs`
  - 为精确 Stage 51 登记提供追加式、自哈希、单链尖且批准终止的独立复核链；独立重算 claim/specification/registration/result 与完整上游绑定。
  - 固定复核三模型臂、17/29/43、65/9 合同、切分隔离、逐目标逐种子指标、资源上限与零执行权限；批准只开放未来训练实现登记。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 52 registry GET 与按 attempt 追加独立复核的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-training-experiment-registration-review-panel.tsx`
  - 管理员逐项完成十二项确认，追加批准、退回修改或拒绝结论，并明确“登记复核 ≠ 训练授权”。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 52 面板接在 Stage 51 后，保持训练实现登记、实现复核、runner 与运行授权分门。
- `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - 显示 Stage 52 状态卡，区分等待独立复核、退回/拒绝和“已独立批准、等待训练实现登记”。
- `crates/hone-web-api/src/routes/investment_decisions.rs`
  - readiness v49 纳入 Stage 52 独立复核状态；批准仍是阻断项，不创建 runner 或训练权限。
  - 六项不可逆确认、claim/result、副本计数和“复制不是训练”边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v45 与 ㊽ 状态卡区分待 claim、复制失败、复制完成和待独立复制后校验；训练与执行权限继续关闭。

## Stage 65 Per-Target Candidate Admission Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_per_target_candidate_admission_reviews.rs`
  - 将 Stage 64 独立复算通过的 validation 输出拆成九个内容寻址目标包，逐目标重算 3 算法×3 种子形状、证据状态、三种子门槛和中位 MAE。
  - 保存 attempt/target 级追加式独立复核链；批准只开放未来 sealed-holdout 评估协议复核资格。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 65 registry GET 与按 attempt/target 追加复核的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-validation-evaluation-per-target-candidate-admission-panel.tsx`
  - 提供逐目标选择、证据门禁、八项边界确认、理由/局限及批准、退回、拒绝操作；不合格目标不能选择批准 verdict。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 65 面板接在 Stage 64 后，保持准入与后续 sealed-holdout 协议复核分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v62 展示九目标候选、已复核、已准入、证据不足/无候选通过及下一协议门禁状态；sealed holdout、模型/指标库、reward、shadow、order、broker 和 trading 继续关闭。

## Stage 66 Sealed-Holdout Evaluation Protocol Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_protocol_reviews.rs`
  - 从当前 Stage 65 逐目标准入生成内容寻址的一种算法/三种子确认性协议，冻结指标、门槛、official-component bootstrap、Holm、最小样本与 one-shot 无反馈复用规则。
  - 保存 attempt/target 级追加式、自哈希、单链尖、批准终止的独立复核链；批准只开放未来评估实现登记。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 66 registry GET 与按 attempt/target 追加独立复核的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-protocol-review-panel.tsx`
  - 提供协议/统计摘要、十二项边界确认、理由/局限及批准、退回或拒绝操作，并明确协议审查不打开 sealed holdout。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 66 面板接在 Stage 65 后，保持协议复核与未来实现、数据访问、runner 和执行分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v63 展示已准入目标、已复核/批准/退回协议及未来实现登记资格；sealed-holdout access/evaluation、正式选模、模型/指标库、reward、shadow、order、broker 和 trading 继续关闭。
# Stage 67 sealed-holdout evaluation implementation registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations.rs` — immutable create-once registry, exact Stage 66 binding, zero-capability contract and readiness summary.
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_protocol_reviews.rs` — exposes only currently approved Stage 66 protocols to the Stage 67 registrar.
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-implementation-panel.tsx` — admin registration surface and explicit “registration is not execution” boundary.
- `crates/hone-web-api/src/routes/investment_decisions.rs` — empirical readiness v64 aggregates the Stage 67 gate without granting runtime authority.

## Stage 68 Sealed-Holdout Evaluation Implementation Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementation_reviews.rs`
  - 为当前 Stage 67 实现保存追加式、自哈希、批准终止的独立复核链；独立重算实现、合同和 Stage 66 协议指纹，并强制完整角色隔离与零能力边界。
  - 批准只开放未来 Stage 69 无入口隔离 runner 登记资格，不读取或执行 sealed holdout。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 68 管理员 registry GET 与按 implementation 追加 review 的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-implementation-review-panel.tsx`
  - 提供独立审计摘要、十一项边界确认、理由/局限和批准、退回或拒绝操作。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 68 面板接在 Stage 67 后，保持复核、runner、授权、执行与输出校验分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v65 展示待复核、已独立批准、退回/拒绝及未来 runner 登记资格；所有运行与投资权限继续关闭。

## Stage 69 Sealed-Holdout Evaluation Isolated Runner Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_isolated_runners.rs`
  - 为每条当前 Stage 68 批准复核最多 create-once 登记一个不可变、内容寻址、无入口 runner，精确绑定 review/audit、implementation/contract、Stage 66 protocol、单目标/算法/三种子、sealed split、特征和预处理。
  - 当前无挂载、数据访问、环境、网络、工具、子进程或执行；未来输入/输出合同、静态资源边界和 Stage 70 一次性授权门禁均被固定。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 69 registry GET 与 create-once runner registration POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-isolated-runner-panel.tsx`
  - 提供唯一未登记批准复核选择、runner 工件身份、十一项边界确认和“登记不是访问，也不是执行”的管理面。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 69 面板接在 Stage 68 后，保持 runner、一次性授权、执行、输出校验和正式选择分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v66 展示可登记、runner 数、当前绑定及未来一次性授权复核资格；sealed-holdout access/evaluation、正式选模及所有投资执行权限继续关闭。

## Stage 70 Sealed-Holdout Evaluation First-Execution Authorization Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_first_execution_authorizations.rs`
  - 为每个当前有效 Stage 69 runner 保存追加式、自哈希、批准终止的链外独立复核链；重新绑定 Stage 66–69 制品、合同、协议、目标、算法、三种子、sealed split、65 项特征和预处理。
  - 批准只提供 24 小时内最多一次的未来隔离调用资格；模块没有 claim、挂载、读取、执行或输出入口。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 70 registry GET 与按 isolated runner 追加 review 的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-panel.tsx`
  - 提供十六项独立确认、理由/局限、审计链和批准、退回或拒绝操作，并明确授权不是访问或执行。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 70 面板接在 Stage 69 后，继续把授权、claim-first 执行、输出独立校验和正式选择分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v67 展示已复核、已批准、未过期、one-shot 与未来 Stage 71 执行尝试资格；当前 sealed-holdout access/evaluation 和所有投资执行权限仍关闭。

## Stage 71 Sealed-Holdout Claim-First One-Shot Evaluation Attempt

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts.rs`
  - 在任何 sealed-holdout 重开前 create-once claim 并消费精确 Stage 70 授权；重新验证 Stage 57–70 全链绑定，只执行一个目标、一个冻结算法和种子 17/29/43。
  - 生成逐种子冻结指标、component-block bootstrap/Holm 门禁和内容寻址的临时不可信信封；失败、中断或成功均不允许重试，临时挂载随后删除。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts.rs`
  - 将既有确定性预处理、目标投影、冻结预测器和确认统计辅助函数以 crate 内可见方式复用于 Stage 71，不改变 Stage 63 既有算法语义。
- `crates/hone-web-api/src/routes/mod.rs`
  - 暴露 Stage 71 registry GET 与按 isolated runner 执行一次的 POST 路由。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-execution-attempt-panel.tsx`
  - 提供七项不可逆确认、可用授权选择、执行/claim 历史和逐种子指标展示，并明确结果未经独立校验。
- `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`
  - 将 Stage 71 面板接在 Stage 70 后，保持执行、输出独立校验和正式选择分门。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v68 展示 claim、完成/失败、不可信信封与 Stage 72 独立输出校验资格；正式选模及所有投资执行权限仍关闭。

## Stage 72 Sealed-Holdout Independent Output Recomputation

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_output_validations.rs`
  - 为每个完整 Stage 71 attempt create-once 写独立验证；排除执行者和 Stage 51–71 完整责任链，重开精确 claim/result、已消费授权、冻结候选、独立验证训练副本和原始 outcome dataset。
  - 使用 Stage 64 第二实现重构 holdout 投影与三种子预测，逐位复算三指标、component bootstrap、Holm、样本与阈值门禁；任何不一致不可变失败关闭。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_output_validations.rs`
  - 只扩大既有 Stage 64 第二实现统计 helper 的 crate 内可见性，供 Stage 72 复算；不改变 Stage 64 或 Stage 71 既有语义。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 72 registry GET 和按 attempt create-once validate POST 路由/API。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-evaluation-output-validation-panel.tsx`
  - 提供七项边界确认、attempt 选择、第二路径说明以及待复算/通过/失败/待裁决状态。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v69 以 Stage 72 裁决复核资格取代 Stage 71 待验证资格作为最新门禁；通过仍不授权正式模型或任何投资执行。

## Stage 73 Sealed-Holdout Confirmatory-Result Adjudication

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_confirmatory_result_adjudication_reviews.rs`
  - 为每条 Stage 72 独立验证通过的确认结果建立追加式、自哈希、角色隔离裁决链；精确绑定 validation、claim/result/output/envelope、目标、算法、17/29/43、sealed split、投影、65 项特征与预处理。
  - 定量批准资格由预登记结果和 Stage 72 复算共同决定：证据不足、任一种子或指标失败、目标/算法边界异常时不可人工覆盖为通过。
  - 人工必须分别记录统计解释、经济解释、局限、证伪条件和下一实验约束，并确认效应量、样本/独立分量、多重检验、覆盖偏差、失败模式及未确认 Hari/老王逻辑隔离。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 73 registry GET 和按 attempt append-only review POST 路由/API。
- `packages/app/src/components/public-admin-historical-outcome-sealed-holdout-confirmatory-result-adjudication-panel.tsx`
  - 分开展示定量通过与失败/不足；失败时禁用批准，提供五类解释、十二项确认和批准/退回/拒绝操作。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v70 以未来受控影子实验设计登记资格作为最新门禁；Stage 73 通过仍不正式选模、不反馈训练/reward、不启动影子账本或任何交易执行。

## Stage 74 Controlled Shadow-Experiment Design Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations.rs`
  - 将一条 Stage 73 已裁决确认结果 create-once 登记为自哈希、不可变、仅供未来复核的前向影子实验设计；登记人排除 Stage 51–73 全部责任角色。
  - 固定 SPY/现金/等权/规则反事实、下一交易日执行、每边 25bp 滑点、仅多头普通股、单股/主题/总仓/现金边界、252 日观察、分项指标、多重检验和停止规则。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 74 registry GET 与按 attempt create-once register POST 路由/API。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-design-registration-panel.tsx`
  - 展示固定协议、五类人工说明和十一项边界确认；登记后只显示等待独立设计复核。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v71 以未来独立影子设计复核资格作为最新门禁；模型库、训练/reward、影子账本、持仓、订单、券商和交易继续关闭。

## Stage 75 Controlled Shadow-Experiment Design Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registration_reviews.rs`
  - 为每条当前 Stage 74 登记建立追加式、自哈希、角色隔离的独立复核链；独立重算 registration/design 指纹并精确重绑 Stage 51–74 全链。
  - 复核点时/成分股/退市与前视偏差、反事实、信号/分红/成本/调仓、组合边界、观察门槛、分项指标、多重检验、停止/证伪和未确认逻辑隔离。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 75 registry GET 与按 attempt append-only review POST 路由/API。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-design-registration-review-panel.tsx`
  - 展示五类解释、十四项确认和批准/要求新建设计/拒绝；批准文案明确只开放零能力实现规格登记。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v72 以未来零能力影子实现规格登记资格作为最新门禁；模型库、训练/reward、影子实现/运行、账本、持仓、订单、券商和交易继续关闭。

## Stage 76 Zero-Capability Controlled Shadow Implementation Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementations.rs`
  - 将一条 Stage 75 独立批准设计 create-once 登记为自哈希、内容寻址的纯实现规格；精确嵌入 Stage 74 design 并绑定完整上游。
  - 固定确定性信号、组合状态转移、成本/分红、反事实同步、检查点/停止和未来不可信输出信封；无入口、程序、runtime、网络、生产读写或投资执行能力。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 76 registry GET 与按 attempt create-once register POST 路由/API。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-implementation-panel.tsx`
  - 展示五类人工说明、十四项边界确认、当前绑定规格和“规格不是程序”的硬边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v73 以未来独立实现复核资格作为最新门禁；runner、影子运行、账本、持仓、订单、券商和交易继续关闭。

## Stage 77 Controlled Shadow Implementation Independent Review

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementation_reviews.rs`
  - 为每条当前 Stage 76 纯规格实现维护追加式、自哈希、角色隔离且批准终止的独立复核链。
  - 第二路径重算实现、合同、设计复核、设计登记和设计规格五层指纹；精确检查完整上游、确定性重放语义、点时/退市/禁止前视、组合/反事实/观察/停止规则及全部零权限边界。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 77 registry GET 与按 implementation append-only review POST 路由/API；读写均要求管理员鉴权。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-implementation-review-panel.tsx`
  - 展示五层独立审计、五类书面说明、十五项确认及批准/要求新建责任链/拒绝操作，并明确“独立复核，不是运行授权”。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v74 以未来隔离影子 runner 规格登记资格作为最新门禁；runner、输入挂载、影子运行、账本、持仓、订单、券商和交易继续关闭。

## Stage 78 Controlled Shadow Isolated Runner Specification Registration

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_isolated_runners.rs`
  - 对每条 Stage 77 独立批准实现 create-once 登记内容寻址 runner 规格，精确嵌入实现合同和完整 review/audit/design 指纹，并执行 Stage 51–77 角色隔离。
  - 冻结未来只读点时输入、不可信输出、非特权身份和资源上限，但没有程序、工件、入口、runtime、挂载、数据访问或任何投资执行权限。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加 Stage 78 registry GET 与按 implementation create-once register POST；读写均要求管理员鉴权。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-isolated-runner-panel.tsx`
  - 明确“规格，不是 runner 程序”，展示十三项边界确认、当前绑定和未来资源约束。
- `crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v75 以未来独立首次影子执行授权复核资格作为最新门禁；执行、挂载、账本、持仓、订单、券商和交易继续关闭。

## Stage 81 初始影子观察独立第二实现复算

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_output_validations.rs`
  - 提供 create-once Stage 81 registry/validate、完整角色隔离、同一输入 manifest 重验，以及不复用 Stage 80 helper 的预处理、三种子预测、排序和五重组合上限第二实现。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_execution_attempts.rs`
  - 只向独立验证链暴露已经通过不可变记录校验的完整 Stage 80 attempt；不扩大点时输入持久化权限。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET registry 与按 attempt create-once validate POST；读写分别执行管理员读取和 mutation header 鉴权。
- `packages/app/src/components/public-admin-controlled-shadow-experiment-output-validation-panel.tsx`
  - 要求重新提交与 claim 相同的点时输入，展示八项确认、manifest 不一致关闭、逐位通过/失败和前向协议登记资格。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v78 将 Stage 81 待复算、验证记录、逐位通过、失败关闭和未来前向观察协议登记资格纳入统一晋级链。

## Stage 82 受控前向观察协议登记

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registrations.rs`
  - 提供 create-once 协议 registry/register、完整角色隔离与哈希重绑；冻结自然前向、周度 claim-first、交易日历、SPY 同步、点时来源、复权/公司行动、成本、门槛和停止规则。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET registry 与按 validation create-once register POST；登记不启动观察。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-protocol-registration-panel.tsx`
  - 展示五项治理说明、九项确认、当前绑定与等待独立复核状态。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v79 纳入 Stage 82 待登记、已登记、当前绑定和未来独立协议复核资格，全部下游权限保持关闭。

## Stage 83 前向观察协议责任链外独立复核

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registration_reviews.rs`
  - 提供追加式独立复核 registry/review；强制完整角色隔离、三层独立指纹复算、单根单链尖与批准终止，并验证十六项自然前向和零权限边界。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registrations.rs`
  - 只向 Stage 83 暴露重新校验过且当前有效的 Stage 82 协议及其 Stage 81 来源；不会开始观察。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET review registry 与按 protocol registration 追加 POST review；读写继续执行管理员读取和 mutation header 鉴权。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-protocol-registration-review-panel.tsx`
  - 展示七类书面评估、十六项确认、批准/重建/拒绝、审计链尖和全部零权限边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v80 纳入已登记协议、待复核、已复核、独立通过、重建/拒绝和未来零能力观察实现登记资格。

## Stage 84 前向观察零能力实现规格登记

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_implementations.rs`
  - 提供按 Stage 83 批准 create-once 的 registry/register、完整角色隔离、四层来源重算、实现/合同自哈希、八类确定性纯函数标识和全关闭 authority boundary。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET registry 与按 protocol review create-once register POST；没有运行、观察或输入挂载端点。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-implementation-panel.tsx`
  - 展示七项书面规格、十五项确认、当前绑定、未来独立复核资格和“规格，不是程序”的零能力边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v81 纳入 Stage 84 待登记、已登记、当前绑定和待独立复核状态，观察、账本、持仓、绩效、模型、指标、训练、reward、订单、券商与交易继续关闭。

## Stage 85 前向观察实现责任链外独立复核

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_implementation_reviews.rs`
  - 提供追加式独立复核 registry/review；强制完整角色隔离、六层指纹独立重算、单根单链尖和批准终止，并复核八类纯函数、三个未来 schema 及全关闭 authority boundary。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET review registry 与按 Stage 84 implementation 追加 POST review；没有 runner、观察或交易端点。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-implementation-review-panel.tsx`
  - 展示六项书面评估、十二项确认、批准/重建/拒绝、当前精确绑定以及批准后仍无运行能力的边界。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v82 纳入 Stage 85 待复核、已复核、独立批准、重建/拒绝和未来 Stage 86 隔离 runner 规格登记资格；全部下游运行与交易权限继续关闭。

## Stage 86 前向观察隔离 runner 规格登记

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_isolated_runners.rs`
  - 提供按 Stage 85 独立批准 create-once 的 registry/register、完整角色隔离、精确上游指纹重绑、runner 工件/代码/runtime 身份/复现程序绑定、未来 I/O 合同、固定沙箱资源和全关闭运行/交易权限。
- `crates/hone-web-api/src/routes/mod.rs` 与 `packages/app/src/lib/api.ts`
  - 增加管理员 GET registry 与按 Stage 84 implementation create-once register POST；没有执行、观察、账本、持仓或交易端点。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-isolated-runner-panel.tsx`
  - 展示工件摘要、复现程序、九项书面约束、十六项确认、当前绑定和未来首跑授权复核资格，并明确工件存在但 runtime 未实例化。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v83 纳入 Stage 86 待登记、已登记、当前绑定和未来 Stage 87 首跑授权复核资格；全部观察、存储、训练与交易权限继续关闭。

## Stage 87 前向观察首次执行授权独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_execution_authorizations.rs`
  - GET 管理端列表并重建当前 Stage 86 runner 与 append-only 复核链。
  - POST `/{isolated_runner_id}/review` 要求独立复现工件 SHA-256、复现证据、完整 Stage 86/85/84/83/82/74 绑定和责任链外 reviewer。
  - 批准只形成 24 小时内一次未来 Stage 88 claim-first 尝试候选；模块没有执行入口、claim、mount 或观察写入。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-forward-observation-first-execution-authorization-panel.tsx`
  - 显示冻结与复现摘要、复现证据、18 项确认、批准/重建/拒绝，以及“批准不等于执行”边界。
- 汇总：`crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v84 纳入已复核、批准、未过期、一次性和未来尝试候选数量；所有实际执行和交易权限继续关闭。

## Stage 88 前向观察 claim-first 单次初始化

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_execution_attempts.rs`
  - 提供 GET registry 与按 runner 的一次性 POST；先 create-once claim 并消费 Stage 87 授权，再重验二进制和零行情初始化 manifest，成功仅生成不可信 day-0 收据。
- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_execution_authorizations.rs`
  - Stage 87 registry 读取 Stage 88 claims，已消费授权不再显示为 future-attempt eligible。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-execution-attempt-panel.tsx`
  - 展示不可逆授权消费、canonical manifest 输入、十项边界确认、attempt/result 与未来独立验证状态。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts`、`packages/app/src/lib/api.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v85 和管理端统一展示纳入 Stage 88 eligibility/claim/completed/failed/interrupted/validation-eligible 计数；全部行情、观察、存储、训练和交易权限继续关闭。

## Stage 89 前向观察零行情初始化收据独立验证

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations.rs`
  - 提供 GET registry 与按 attempt 的 create-once POST；独立重算 claim/result/receipt，从 v2 收据重建 manifest，并从 Stage 87–74 精确链构造预期收据。
- `packages/app/src/components/public-admin-controlled-shadow-forward-observation-output-validation-panel.tsx`
  - 展示责任链外验证确认、零行情/零观察边界、不可变 verdict、mismatch 与未来首周期复核资格。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts`、`packages/app/src/lib/api.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v86 纳入 Stage 89 待验证、已验证、独立通过、失败与未来首个自然前向周期授权复核资格；不新增运行或交易能力。

## Stage 90 首个自然前向周期一次性授权复核

- `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_authorizations.rs`
  - 提供管理员 GET registry 与按 Stage 89 validation 的 append-only POST review；精确绑定 Stage 51–89，执行完整角色排除，并把批准限制为首个合格自然周期起算 7 天内最多一次的未来 claim-first 尝试。
- `packages/app/src/components/public-admin-controlled-shadow-first-natural-forward-cycle-authorization-panel.tsx`
  - 显示 16 项边界确认、复核理由、授权窗口和 future-attempt/已消费状态；明确 Stage 91 只能另行领取不可执行任务，未来行情适配器仍须单独只读授权。
- `crates/hone-web-api/src/routes/investment_decisions.rs`、`packages/app/src/lib/types.ts`、`packages/app/src/lib/api.ts` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`
  - readiness v87 纳入 Stage 90 可复核、已复核、已批准、生效与未来单次资格；当前日历、行情、runtime、观察、账本、持仓、绩效、训练和交易权限继续关闭。

## Stage 91 首个自然前向周期任务声明

- 后端 registry/claim：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_claims.rs`
- 路由入口：`crates/hone-web-api/src/routes/mod.rs` 下的 `...first-natural-forward-cycle-claims` GET 与 `/{authorization_review_id}/claim-once` POST。
- Stage 90 授权消费回读：`...forward_observation_first_natural_forward_cycle_authorizations.rs` 通过 Stage 91 claim 摘要关闭已消费 eligibility。
- 前端 API/类型：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`
- 管理端组件：`packages/app/src/components/public-admin-controlled-shadow-first-natural-forward-cycle-claim-panel.tsx`
- 统一准备度：`crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx`

## Stage 92 只读行情适配器独立授权

- 后端合同与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_adapter_authorizations.rs`
  - 固定 GET/HTTPS 来源路径、查询参数、数据类别、SPY 同步、内容寻址、凭据脱敏、原始载荷保管、追加式更正和 16 MiB 上限；提供管理员 GET registry 与按 Stage 91 claim 的 create-once POST review。
- Stage 91 内部只读交接：`...forward_observation_first_natural_forward_cycle_claims.rs` 暴露仅供后续治理阶段读取当前 claims 的 helper；Stage 92 不调用外部数据。
- 路由入口：`crates/hone-web-api/src/routes/mod.rs` 下的 `...first-natural-forward-cycle-market-data-adapter-authorizations` GET 与 `/{cycle_claim_id}/review` POST。
- 前端 API/类型与复核面板：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-adapter-authorization-panel.tsx`。
- 统一准备度：`crates/hone-web-api/src/routes/investment_decisions.rs` 与 `packages/app/src/components/public-admin-decision-brain-panel.tsx` 使用 readiness v89 展示待复核、批准、拒绝、生效和未来只读收据资格；仍无数据访问或观察能力。

## Stage 93 claim-first 单次只读原始行情收据

- 后端 claim/读取/收据 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts.rs`
  - 从 Stage 89/81 验证链推导标的，从 Stage 92 授权推导纽约自然前向窗口；先写不可变 claim，再以禁重定向专用 client 执行固定 FMP/NYSE GET，按 SHA-256 create-once 保管原始字节并写单一成功/失败终态。
- Stage 92 授权消费回读：`...forward_observation_first_natural_forward_cycle_market_data_adapter_authorizations.rs` 读取 Stage 93 claims，已消费授权不再 active 或 eligible；授权窗修正为 7 天。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-receipt-attempts` GET 与 `/{adapter_authorization_id}/claim-and-read-once` POST。
- 前端 API/类型/执行面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-receipt-attempt-panel.tsx`；历史治理页和统一决策大脑卡片展示 eligibility/claim/未信任收据/失败/中断/待独立验证。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v90；成功收据仍不等于日历、行情观察、账本、持仓、绩效或交易事实。

## Stage 94 原始行情收据责任链外独立验证

- 后端独立验证与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_validations.rs`
  - 重开精确 Stage 92/93 链，独立重建脱敏 FMP/SPY/NYSE 请求，重算 claim/result/receipt/request/body/source/raw payload 指纹，复核内容寻址保管与凭据无落盘边界；只检查最小 JSON/HTML 信封，不解析市场语义。
- Stage 93 审计回读：`...forward_observation_first_natural_forward_cycle_market_data_receipt_attempts.rs` 提供完成记录的独立验证候选；该 helper 不预先拒绝 raw payload 缺失或篡改，使 Stage 94 能保存永久失败终态。
- Stage 92 精确授权审计：`...forward_observation_first_natural_forward_cycle_market_data_adapter_authorizations.rs` 只读回开已消费授权及 Stage 91 claim，不恢复 active/eligible 状态。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-receipt-validations` GET 与 `/{attempt_id}/validate-once` POST。
- 前端 API/类型/验证面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-receipt-validation-panel.tsx`；历史治理页和统一决策大脑卡片展示 pending/validated/failed/parser-review eligible。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v91；通过仍不等于行情已解析或自然前向观察已经开始。

## Stage 95 零能力行情 parser 规格登记

- 后端规格与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specifications.rs`
  - 从 Stage 94 只读 helper 取得独立验证候选，服务端构建并 create-once 冻结严格 parser 规格、权限边界和八个合成向量；没有 parser 实现或调用入口。
- Stage 92–94 v2 来源合同：`...market_data_adapter_authorizations.rs`、`...market_data_receipt_attempts.rs`、`...market_data_receipt_validations.rs` 共同固定五类 FMP stable 显式价格/分红/拆股来源与 NYSE 日历，并禁止 legacy 历史价格路径。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-specifications` GET 与 `/{validation_id}/register-once` POST。
- 前端 API/类型/登记面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-specification-panel.tsx`；历史治理页和统一决策大脑卡片展示待登记、已登记与待责任链外复核状态。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v92；登记仍不等于行情已解析或自然前向观察已经开始。

## Stage 96 行情 parser 规格责任链外独立复核

- 后端复核与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specification_reviews.rs`
  - 独立重算完整验证链、Stage 95 登记/规格、显式 FMP/NYSE 请求和八组合成向量，保存 create-once 终态复核；不访问原始载荷。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-specification-reviews` GET 与 `/{registration_id}/review-once` POST。
- 前端 API/类型/复核面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-specification-review-panel.tsx`；历史治理页和统一决策大脑卡片展示待复核、独立通过与需重建状态。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v93；独立通过仍不等于 parser 已实现、行情已解析或自然前向观察已开始。

## Stage 97 行情 parser 零能力实现契约登记

- 后端登记与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_implementations.rs`
  - 从 Stage 96 只读 helper 取得独立批准规格，精确绑定 Stage 95/96 与上游摘要，冻结八个纯函数标识、canonical schema、失败关闭语义和零能力权限边界；没有源码、工件、入口、runtime 或载荷访问。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-implementations` GET 与 `/{specification_review_id}/register-once` POST。
- 前端 API/类型/登记面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-implementation-panel.tsx`；历史治理页和统一决策大脑卡片展示待登记、当前绑定及待 Stage 98 独立复核状态。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v94；契约登记仍不等于 parser 已实现、行情已解析或自然前向观察已开始。

## Stage 98 行情 parser 实现责任链外独立复核

- 后端复核与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_implementation_reviews.rs`
  - 独立重算 Stage 97 implementation/contract、Stage 96 review、Stage 95 registration/specification，核验八个函数与八组合成向量，并保存 create-once 终态复核；不访问原始载荷。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-implementation-reviews` GET 与 `/{implementation_id}/review-once` POST。
- 前端 API/类型/复核面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-implementation-review-panel.tsx`；历史治理页和统一决策大脑卡片显示待复核、独立通过、拒绝与 Stage 99 资格。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v95；批准仍不等于 runner 已登记、parser 可运行、行情已解析或观察已开始。

## Stage 99 隔离行情 parser runner 规格登记

- 后端登记与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_isolated_runners.rs`
  - 只从 Stage 98 当前独立批准实现构建 create-once、自哈希规格，绑定 Stage 93–98 全链、未来工件身份、固定无特权运行环境与资源上限；不包含源码、可执行工件、入口或运行能力。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-isolated-runners` GET 与 `/{implementation_id}/register-once` POST。
- 前端 API/类型/登记面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-isolated-runner-panel.tsx`；历史治理页和统一决策大脑卡片显示登记资格、当前绑定和 Stage 100 首次执行授权复核资格。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v96；规格登记仍不等于工件存在、parser 可执行、载荷可读、行情已解析或观察已开始。

## Stage 100 行情 parser 首次执行授权责任链外复核

- 后端授权与工件检查：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_first_execution_authorizations.rs`
  - 从 Stage 99 当前 runner 派生固定保管目录，拒绝符号链接、可写/空/超限文件；独立验证 manifest 自哈希并重算工件字节 SHA-256。授权 append-only、24 小时、单次，且工件变化即失效。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-first-execution-authorizations` GET 与 `/{isolated_runner_id}/review-once` POST。
- 前端 API/类型/复核面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-first-execution-authorization-panel.tsx`；历史治理页和统一决策大脑卡片显示待工件、服务端已核验、已复核与 Stage 101 claim 资格。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v97；批准仍不等于 parser 已执行、载荷已读取、行情已解析或观察已开始。

## Stage 101 行情 parser 首次执行尝试 claim-first 声明

- 后端声明与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempt_claims.rs`
  - 从 Stage 100 当前有效授权构建 create-once、自哈希声明，永久消费授权，并冻结 Stage 94/93 固定输入 manifest；只读上游元数据，不打开 raw payload，也不运行 parser。
- Stage 100 消费识别：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_first_execution_authorizations.rs`
  - registry 读取 Stage 101 已消费授权，永久撤销对应 future-claim eligibility，同时保持 Stage 100 不可变记录本身不被覆盖。

## Stage 102 行情 parser 单次受限执行

- 后端执行与确定性解释器：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempts.rs`
  - GET registry 与 `/{attempt_id}/execute-once`；每条 Stage 101 claim 最多一个终态 result。
  - 严格声明式 artifact 合同；重新核验 Stage 100 工件/manifest，固定 Stage 93 custody 路径与载荷 SHA-256。
  - FMP price/dividend/split、NYSE holiday table/early close、canonical row hash、SPY coverage、subject explicit gap。
- 上游只读桥接：Stage 100 模块导出执行前工件重验证读取；Stage 101 模块导出 claim 读取/验证；Stage 93 模块导出固定 custody root。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-execution-attempt-panel.tsx`，配套 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`；历史治理页和统一决策大脑卡片显示待执行、终态、非可信输出和失败已消费。
- custody：`investment_decisions/controlled-shadow-market-data-parser-execution-attempts/starts/{stage101_attempt_id}.json`、`results/{stage101_attempt_id}.json` 与 `outputs/{output_sha256}.json`。start marker 在任何工件/载荷读取前 create-once 消费 claim；Stage 103 前输出不得进入观察或组合链。
- 路由：`crates/hone-web-api/src/routes/mod.rs` 注册 `...market-data-parser-execution-attempt-claims` GET 与 `/{authorization_review_id}/claim-once` POST。
- 前端 API/类型/声明面：`packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`、`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-execution-attempt-claim-panel.tsx`；历史治理页和统一决策大脑卡片显示可声明、已消费与等待 Stage 102 状态，且不提供执行按钮。
- readiness：`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v98；claim 仍不等于 parser 已运行、载荷已读取、行情已解析或观察已开始。

## Stage 103 行情 parser 输出责任链外独立校验

- 后端校验与第二实现：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_output_validations.rs`
  - GET registry 与 `/{attempt_id}/validate-once`；独立重开 Stage 102 output 和 Stage 94 raw payload，全量重解析并精确比对完整输出，不调用 Stage 102 parser helpers。
- 上游只读桥接：Stage 102 模块只导出已验证 execution result/output 读取和 custody root；Stage 103 不导入其解析 helper。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 Stage 103 路由；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v100 并只开放 Stage 104 观察输入准入复核候选。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-market-data-parser-output-validation-panel.tsx`，配套 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`；历史治理页和统一决策大脑卡片显示待校验、独立一致、失败关闭和 Stage 104 候选。
- custody：`investment_decisions/controlled-shadow-market-data-parser-output-validations/{attempt_id}/{validation_id}.json`；本轮没有真实记录。

## Stage 104 首次自然前向周期观察输入独立准入

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_input_admission_reviews.rs`
  - GET registry 与 `/{attempt_id}/review`；重开精确 Stage 102 output，重算官方交易日、SPY 覆盖、标的行/gap 矩阵、公司行动与 custody-time floor。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 Stage 104 路由；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v101 并只开放 Stage 105 观察物化规格登记候选。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-input-admission-panel.tsx`，配套 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`；历史治理页和统一决策大脑卡片明确显示 provider time 未验证、准入计数与 Stage 105 边界。
- custody：`investment_decisions/controlled-shadow-first-natural-forward-cycle-observation-input-admission-reviews/{attempt_id}/{review_id}.json`；本轮没有真实记录。

## Stage 105 首次自然前向周期观察物化规格登记

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specifications.rs`
  - GET registry、`/{review_id}/register-once`、create-once storage、角色隔离、当前上游重绑定、固定物化 schema 与零能力校验。
- 上游：Stage 104 模块导出 `admitted_controlled_shadow_observation_inputs_for_materialization_specification`；返回当前准入 candidate/review，并继续继承 Stage 104 的结构审计和 custody-time limitation。
- 路由与 readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v102）。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-specification-panel.tsx` 及同名 `.test.ts`；API/类型在 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`。
- 当前无 Stage 105 真实登记目录内容；无 future `observations/{cycle_claim_id}/{specification_sha256}.json` 输出。

## Stage 106 责任链外观察物化规格独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specification_reviews.rs`
  - GET registry、`/{registration_id}/review`、append-only 自哈希 review chain、角色隔离、批准终态、当前上游重绑定和第二实现完整规格重建。
- 上游：Stage 105 模块导出 `independently_reviewable_observation_materialization_specifications`；同时返回当前验证后的登记和 Stage 104 source，Stage 106 不复用 Stage 105 规格构造器。
- 路由与 readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v103）。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-specification-review-panel.tsx` 及同名 `.test.ts`；API/类型位于 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`。
- 当前无 Stage 106 真实复核记录；批准也不会生成观察或打开任何 runtime/交易能力。

## Stage 107 观察物化零能力实现契约登记

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_implementations.rs`。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v104）。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-implementation-panel.tsx`；API/类型位于 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`。
- 预留保管路径：`investment_decisions/controlled-shadow-first-natural-cycle-observation-materialization-implementations/{review_id}/implementation.json`；当前为空。

## Stage 108 观察物化实现责任链外独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_implementation_reviews.rs`。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v105）。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-implementation-review-panel.tsx` 及同名 `.test.ts`；API/类型在 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`，统一卡片在 `public-admin-decision-brain-panel.tsx`。
- 预留保管路径：`investment_decisions/controlled-shadow-first-natural-cycle-observation-materialization-implementation-reviews/{implementation_id}/{review_id}.json`；当前为空。

## Stage 109 观察物化隔离 runner 规格登记

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_isolated_runners.rs`。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v106）。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-isolated-runner-panel.tsx` 及同名 `.test.ts`；API/类型位于 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`，统一卡片位于 `public-admin-decision-brain-panel.tsx`。
- 预留保管路径：`investment_decisions/controlled-shadow-first-natural-forward-cycle-observation-materialization-isolated-runners/{implementation_id}/runner.json`；当前为空，不存在工件、runtime、输入读取或观察输出。

## Stage 110 观察物化首次执行授权独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_first_execution_authorizations.rs`。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs`、`crates/hone-web-api/src/routes/investment_decisions.rs`（v107）。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-first-execution-authorization-panel.tsx` 及测试，另接入 historical-governance、统一决策脑卡片、`packages/app/src/lib/api.ts`、`packages/app/src/lib/api.test.ts` 和 `packages/app/src/lib/types.ts`。
- 保管目录：`controlled-shadow-observation-materialization-reproduced-artifacts/{runner_id}/{artifact_sha256}/`；复核目录：`controlled-shadow-observation-materialization-first-execution-authorization-reviews/{runner_id}/`。本轮零真实目录、工件和记录。

## Stage 111 观察物化单次尝试 claim-first 声明

- 后端声明与消费：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempt_claims.rs`
  - 从当前、未过期且未消费的 Stage 110 授权派生 create-once、自哈希声明；在任何 runtime、工件执行或 Stage 104 输入读取前永久消费精确授权，并完整绑定 Stage 51–110、runner artifact/manifest 与 Stage 101–104 自然前向周期链。
  - claimant 排除 Stage 110 reviewer、工件构建者、Stage 109 registrar 和完整既有责任链；调用方不能选择输入、工件、路径、日期或标的。retry、release 与 authorization restoration 永久关闭。
- Stage 110 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_first_execution_authorizations.rs` 从 Stage 111 持久化声明计算已消费授权，不再使用空集合占位。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET registry 与 `/{authorization_review_id}/claim-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v108，仅显示 Stage 112 单次受控执行等待状态。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-execution-attempt-claim-panel.tsx` 及同名测试；API/类型位于 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`，并接入历史治理页和统一决策大脑卡片。
- 保管目录：`investment_decisions/controlled-shadow-first-natural-forward-cycle-observation-materialization-execution-attempt-claims/{authorization_review_id}/claim.json`；本轮零真实 claim、零真实工件、零输入读取、零观察输出，也没有持仓、绩效、训练、RL、订单、券商或交易能力。

## Stage 112 观察物化单次受控执行

- 后端执行与 registry：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempts.rs`
  - GET registry 与 `/{attempt_id}/execute-once`；start marker 先于所有工件/输入读取，失败、超时和中断永久消费 claim。
  - 严格声明式 artifact 由受信任进程内解释器执行；重验证 Stage 110 artifact/manifest，并通过 Stage 104 只读桥接重开精确 Stage 102 output，验证 session、三价格口径、explicit gap、公司行动、精确十进制、来源 hash 和 Stage 88 初始分配绑定。
- 上游桥接：Stage 104 模块导出执行专用的 admitted-output 重开/重哈希 helper；Stage 111 registry 从 Stage 112 start/result 计算仍待执行的 claim，保持不可变 claim 记录本身不被修改。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 Stage 112 GET/POST；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v109，分别显示待执行、永久失败和成功但等待 Stage 113 独立验证的数量。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-execution-attempt-panel.tsx` 及同名测试；API/类型位于 `packages/app/src/lib/api.ts`、`packages/app/src/lib/types.ts`，并接入历史治理页与统一决策大脑卡片。
- custody：`investment_decisions/controlled-shadow-observation-materialization-execution-attempts/starts/{attempt_id}.json`、`results/{attempt_id}.json`、`observations/{cycle_id}/{specification_sha256}.json`。本轮零真实记录；未来成功输出仍为 `untrusted`，Stage 113 前不能进入组合、绩效、训练或交易链。

## Stage 113 观察物化输出责任链外独立校验

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_output_validations.rs`；独立重开 Stage 112 output 与 Stage 104 admitted input，第二投影全量重算，不调用 Stage 112 materializer helper。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET/validate-once；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v110。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-materialization-output-validation-panel.tsx` 及测试；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- custody：`investment_decisions/controlled-shadow-observation-materialization-output-validations/{attempt_id}/validation.json`；本轮零真实记录。通过只进入 Stage 114 证据准入复核，不创建账本、持仓、绩效、训练或交易事实。

## Stage 114 观察证据责任链外独立准入

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_evidence_admission_reviews.rs`；Stage 113 模块另导出复核前 custody/full-reprojection revalidation helper。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET/review；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v111，并区分待复核、已准入、退回/拒绝与 Stage 115 候选。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-evidence-admission-panel.tsx` 及同名测试；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- custody：`investment_decisions/controlled-shadow-observation-evidence-admission-reviews/{attempt_id}/{review_id}.json`；本轮零真实记录。批准只开放 Stage 115 账本转换规格登记，不改写原 envelope，不建账、不算绩效、不训练或交易。

## Stage 115 观察证据到账本转换零能力规格登记

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_specifications.rs`；从当前重验证的 Stage 114 evidence 重建确定性规格，并复核完整责任链排除名单。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET registry 与 `/{review_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v112。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-specification-panel.tsx` 及同名测试；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- custody：`investment_decisions/historical-outcome-controlled-shadow-observation-ledger-transition-specifications/{registration_id}.json`；本轮零真实记录。登记只开放 Stage 116 独立规格复核，不创建任何账本、持仓、净值、绩效、训练或交易事实。

## Stage 116 账本转换规格责任链外独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_specification_reviews.rs`；第二套实现从 Stage 114 evidence 独立重建完整规格，不调用 Stage 115 builder。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET/review；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v113，并分别显示待复核、已复核、独立批准、退回/拒绝、opening snapshot 缺口与 Stage 117 候选。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-specification-review-panel.tsx` 及同名测试；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- custody：`investment_decisions/historical-outcome-controlled-shadow-observation-ledger-transition-specification-reviews/{registration_id}/{review_id}.json`；本轮零真实记录。批准只开放 Stage 117 零能力实现登记，不创建 ledger/event、position、cash、NAV/performance、训练或交易事实。

## Stage 117 账本转换零能力实现合同

- 后端模块：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_implementations.rs`，负责 current Stage 116 approved review 桥接、create-once/self-hashed 零能力合同、registry 和 Stage 118 readiness。
- 路由与总准备度：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{specification_review_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 升级为 v114。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-implementation-panel.tsx`、对应测试、`public-admin-historical-outcome-governance-panel.tsx`、`public-admin-decision-brain-panel.tsx`、`packages/app/src/lib/api.ts`、`api.test.ts`、`types.ts`。
- 存储：`investment_decisions/historical-outcome-controlled-shadow-observation-ledger-transition-implementations/{specification_review_id}/implementation.json`。当前未创建真实文件；合同没有任何工件、runtime、输入或财务写入能力。

## Stage 118 账本转换实现合同责任链外独立复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_implementation_reviews.rs`；独立重建完整 Stage 117 contract，不调用 Stage 117 builder，并复算全链哈希与零能力审计。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET/review；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v115，并显示待复核、已复核、独立批准、退回/拒绝与 Stage 119 runner-spec 候选。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-implementation-review-panel.tsx` 及同名 `.test.ts`；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- 存储：`investment_decisions/controlled-shadow-first-natural-cycle-observation-ledger-transition-implementation-reviews/{implementation_id}/{review_id}.json`。当前零真实记录；批准只开放 Stage 119 隔离 runner 规格登记，不开放工件、输入、账本、绩效、训练或交易能力。

## Stage 119 账本转换隔离 runner 规格登记

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_isolated_runners.rs`；绑定 current Stage 118 approval，登记 proposed artifact/hash、immutable revision、reproduction procedure、固定非特权 runtime、Stage 114 只读输入、untrusted create-once 候选输出、空金融事件白名单和资源上限。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET 与 `/{implementation_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v116，只开放 Stage 120 责任链外首次执行授权复核候选。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-isolated-runner-panel.tsx` 及 `.test.ts`；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- 存储：`investment_decisions/controlled-shadow-first-natural-forward-cycle-observation-ledger-transition-isolated-runners/{isolated_runner_id}.json`。当前零真实记录；没有工件、runtime、输入读取、opening snapshot、账本、绩效、训练/RL 或交易能力。

## Stage 120 账本转换首次执行授权责任链外复核

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_first_execution_authorizations.rs`；负责真实只读工件/manifest 检查、服务端重哈希、责任链隔离、append-only 自哈希复核和 24 小时一次性授权。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 注册 GET 与 `/{isolated_runner_id}/review`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 v117，只开放 Stage 121 claim-first 候选。
- 管理端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-first-execution-authorization-panel.tsx` 及 `.test.ts`；同时接入历史治理页、统一决策大脑卡片、`packages/app/src/lib/api.ts`、`api.test.ts` 与 `types.ts`。
- 存储：真实工件保管路径为 `investment_decisions/controlled-shadow-observation-ledger-transition-reproduced-artifacts/{isolated_runner_id}/{artifact_sha256}/`；复核路径为 `investment_decisions/controlled-shadow-observation-ledger-transition-first-execution-authorization-reviews/{isolated_runner_id}/{review_id}.json`。当前零真实工件、manifest 和 review；没有 execution/input/financial/training/trading authority。

## Stage 121 账本转换执行尝试 claim-first 原子认领

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempt_claims.rs`；负责 create-once/self-hashed claim、完整绑定校验、责任链隔离和永久消费 Stage 120 授权。
- 联动：Stage 120 authorization registry 从持久化 Stage 121 claims 派生 consumed review IDs；`routes/mod.rs` 提供 GET 与 `/{authorization_review_id}/claim-once`，`investment_decisions.rs` 使用 readiness v118。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-claim-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/controlled-shadow-observation-ledger-transition-execution-attempt-claims/{attempt_id}.json`。Stage 121 本身只认领、不执行；当前零真实记录。

## Stage 122 账本转换单次受控执行

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempts.rs`；负责 exact claim/artifact/manifest/admitted-output 重验、先写 start marker、固定声明式程序解释和不可重试终态。
- 执行边界：工件只能是无命令、无入口、无子进程的严格声明式 JSON；服务端在进程内执行八个固定纯函数。executor 排除 claimant、Stage 120 reviewer、artifact builder、Stage 119 registrar 与 Stage 51–121 完整责任链。
- 输出边界：opening portfolio snapshot 缟失且 financial-event allowlist 为空时，只能投影未受信的非财务 observation/market-session/raw-close/benchmark/gap/corporate-action notice candidate；不得生成 authoritative ledger event 或任何金融账本状态。
- 路由/readiness：`routes/mod.rs` 提供 GET 与 `/{attempt_id}/execute-once`；`investment_decisions.rs` 使用 readiness v119，成功候选只开放未来 Stage 123 独立验证，失败 claim 永久消耗。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：start/result 位于 `investment_decisions/controlled-shadow-observation-ledger-transition-execution-attempts/`，candidate 位于其 `candidates/{attempt_id}/{candidate_sha256}.json`。当前目录不存在、零真实执行和候选；`investment_decisions/shadow-ledgers` 也不存在。

## Stage 123 账本转换候选责任链外独立验证

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_output_validations.rs`；责任链外 validator 重新打开内容寻址候选与 Stage 114 evidence，并用第二套实现重投影全部 notice、精确十进制、身份、排序和完整候选哈希。
- 路由/readiness：`routes/mod.rs` 提供 GET 与 `/{attempt_id}/validate-once`；`investment_decisions.rs` 使用 readiness v120，分别显示待验证、已独立验证、失败与未来 Stage 124 准入复核候选。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-output-validation-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/controlled-shadow-observation-ledger-transition-output-validations/{attempt_id}/{validation_id}.json`。验证终态 create-once、append-only、自哈希且不可覆盖；通过只开放 Stage 124 非财务候选准入复核，不建立 authoritative ledger event、持仓、现金、净值、绩效、训练或交易状态。当前目录不存在、零真实验证。

## Stage 124 非财务观察候选责任链外独立准入

- 后端：`crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_candidate_admission_reviews.rs`；从 Stage 123 当前独立验证读取链重开 exact Stage 122 candidate，并建立唯一线性、append-only、自哈希复核链。
- 路由/readiness：`routes/mod.rs` 提供 GET 与 `/{attempt_id}/review`；`investment_decisions.rs` 使用 readiness v121，分别显示待复核、已复核、正式非财务证据、退回/拒绝及 Stage 125 快照治理规格候选。
- 前端：`packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-candidate-admission-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/controlled-shadow-observation-ledger-transition-candidate-admission-reviews/{attempt_id}/{review_id}.json`。批准只创建分离的正式非财务观察证据记录；原 candidate 始终 untrusted/immutable。下一门是 Stage 125 外部来源期初组合快照治理规格，不补造持仓、现金、NAV/绩效或交易状态。当前目录不存在、零真实准入记录。

## Stage 125 外部来源期初组合快照治理规格登记

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_snapshot_governance_specifications.rs`；从当前 Stage 124 正式非财务证据建立 create-once、自哈希规格，冻结外部来源合同及账户、现金、持仓、上市期权、负债、未结算活动、证券身份、精确十进制和 NAV 前置门。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{stage_124_review_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v122，显示 Stage 124 正式证据、可登记、已登记规格及未来 Stage 126 独立复核候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-snapshot-governance-specification-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-snapshot-governance-specifications/{registration_id}.json`。当前零真实记录；不接收来源文件、不物化 opening snapshot、不创建金融事件白名单、账本、持仓、现金、NAV/绩效、训练/RL 或交易能力。

## Stage 126 期初组合治理规格责任链外独立复核

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_snapshot_governance_specification_reviews.rs`；从当前 Stage 125 可独立复核登记重新读取完整绑定，用第二实现重建来源、完整快照 schema、证券身份、精确十进制及独立估值前置合同，并独立重算登记与规格哈希。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{registration_id}/review`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v123，显示待复核、已复核、独立批准、退回/拒绝及 Stage 127 零能力来源工件接收实现登记候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-snapshot-governance-specification-review-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-snapshot-governance-specification-reviews/{registration_id}/{review_id}.json`。当前零真实记录；批准也不接收或读取来源文件，不创建期初组合、账本、持仓、现金、NAV/绩效、训练/RL 或交易能力。

## Stage 127 来源工件接收零能力实现登记

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_implementations.rs`；从当前独立批准 Stage 126 来源登记纯合同，保存 17 项确认，冻结未来私有流式接收、资源上限、内容寻址、匿名化、脱敏和失败清理语义，但不提供上传或 parser。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{stage_126_review_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v124，显示独立批准规格、可登记、当前绑定实现合同及未来 Stage 128 独立复核候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-implementation-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-source-artifact-receipt-implementations/{implementation_id}.json`。读取时重新验证当前 Stage 126 集合和精确来源绑定；当前目录不存在、零真实登记。`opening-portfolio-source-artifact-quarantine` 与 `opening-portfolio-source-artifacts` 也不存在，没有来源字节、opening snapshot 或财务状态。

## Stage 128 来源工件接收实现责任链外独立复核

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_implementation_reviews.rs`；独立重建 Stage 127 合同和完整上游摘要，保存一次性终结复核及 17 项确认，不调用 Stage 127 builder，也不接收来源字节。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{implementation_id}/review`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v125，显示实现数、待复核、已复核、独立批准、退回/拒绝及未来 Stage 129 隔离接收器规格登记候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-implementation-review-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-source-artifact-receipt-implementation-reviews/{implementation_id}/{review_id}.json`。当前目录不存在、零真实复核；来源 quarantine/artifact、opening snapshot 和全部财务/交易状态继续为空。

## Stage 129 隔离来源工件接收器规格登记

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_isolated_receivers.rs`；从当前 Stage 128 独立批准集合登记 proposed artifact/hash、immutable revision、复现程序、固定非特权 runtime、管理员鉴权流式输入、未受信 create-once manifest 输出及严格资源上限。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{implementation_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v126，仅开放 Stage 130 首次执行授权复核候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-isolated-receiver-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-source-artifact-receipt-isolated-receivers/{isolated_receiver_id}.json`。当前目录不存在、零真实登记；没有上传入口、来源字节、工件、runtime、receipt、opening snapshot 或任何财务/交易状态。

## Stage 130 来源工件接收器首次执行授权

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_first_execution_authorizations.rs`；检查服务端派生内容寻址目录中的只读 `receiver.artifact` 与自哈希 `manifest.json`，重算摘要/长度并执行完整责任链独立性复核。
- API：GET `/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-first-execution-authorizations`；POST `/{isolated_receiver_id}/review`。没有 upload 或 execute 路由。
- 管理端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-first-execution-authorization-panel.tsx`；显示工件状态、19 项确认、append-only 复核及 24 小时一次性 Stage 131 候选。
- 存储：工件保管目录为 `investment_decisions/opening-portfolio-source-artifact-receipt-reproduced-receivers/{isolated_receiver_id}/{artifact_sha256}/`；复核记录为 `investment_decisions/opening-portfolio-source-artifact-receipt-first-execution-authorization-reviews/{isolated_receiver_id}/{review_id}.json`。当前零真实工件与授权。

## Stage 131 来源工件接收尝试 claim-first

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempt_claims.rs`；先永久消费精确 Stage 130 review，再允许未来独立 Stage 132。
- API：GET `/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempt-claims`；POST `/{authorization_review_id}/claim-once`。没有 upload 或 execute 路由。
- 管理端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-execution-attempt-claim-panel.tsx`，并接入历史治理与决策大脑 readiness v128。
- 存储：`investment_decisions/opening-portfolio-source-artifact-receipt-execution-attempt-claims/{attempt_id}.json`；当前目录不存在、零真实 claim 或来源字节。

## Stage 132 来源工件单次加密接收

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempts.rs`；消费 Stage 131 claim，在首个来源字节前写 start marker，执行 PDF/CSV/JSON 安全筛查、AES-256-GCM 加密、内容寻址原子提交，并输出未受信 receipt 或失败消费终态。
- API：GET `/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempts`；POST `/{attempt_id}/receive-once`。POST 只接受 request-first multipart，不支持 URL、客户端路径或重试。
- 管理端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-execution-attempt-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理和 readiness v129 决策大脑卡片。
- 配置：`.env.example` 记录 `HONE_OPENING_PORTFOLIO_RECEIPT_ENCRYPTION_KEY`；必须是稳定的 32-byte hex key，不得提交真实密钥。
- 存储：start/result 分别位于 `investment_decisions/opening-portfolio-source-artifact-receipt-execution-attempt-starts/` 与 `...-results/`；临时隔离位于 `opening-portfolio-source-artifact-quarantine/`；加密内容对象位于 `opening-portfolio-source-artifacts/{stage_125_registration_id}/{artifact_sha256}/original.bin.enc`；receipt 位于 `opening-portfolio-source-artifact-receipts/{attempt_id}/{receipt_id}/manifest.json`。当前所有目录均不存在，零真实来源或 receipt。

## Stage 133 加密 receipt 责任链外独立验证

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_validations.rs`；责任链外第二实现重开 Stage 131/132、receipt 与加密内容对象，独立重算哈希、认证解密、格式安全结构和脱敏证据，输出 create-once terminal validation。
- API：GET `/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-validations`；POST `/{attempt_id}/validate-once`。路由不接受文件、路径或来源字节。
- 管理端：`packages/app/src/components/public-admin-opening-portfolio-source-artifact-receipt-validation-panel.tsx` 及 `.test.ts`，并接入 API/types、历史治理和 readiness v130 决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-source-artifact-receipt-independent-validations/{attempt_id}/{validation_id}.json`；当前目录不存在，零真实验证、解密落盘或财务状态。

## Stage 134 期初快照物化零能力实现登记

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_snapshot_materialization_implementations.rs`；从当前独立通过 Stage 133 validation 登记确定性 PDF/CSV/JSON 物化合同，冻结完整账户 schema、精确十进制、证券身份、公司行动、逐行来源和整批失败语义，但不读取或解密 receipt。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{validation_id}/register-once`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v131，只开放 Stage 135 责任链外独立实现复核候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-snapshot-materialization-implementation-panel.tsx` 及 `.test.ts`，并接入 `api.ts`、`api.test.ts`、`types.ts`、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-snapshot-materialization-implementation-registrations/{implementation_id}.json`。当前目录不存在、零真实登记；没有 receipt 读取/解密、parser/runtime、候选/正式快照、持仓或任何财务/训练/交易状态。

## Stage 135 期初快照物化实现责任链外独立审查

- 后端：`crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_snapshot_materialization_implementation_reviews.rs`；第二实现独立重建 Stage 134 完整合同与 10 个函数，重算 Stage 125/131/132/133/134 责任链并输出 append-only 终态审查。
- 路由/readiness：`crates/hone-web-api/src/routes/mod.rs` 提供 GET 与 `/{implementation_id}/review`；`crates/hone-web-api/src/routes/investment_decisions.rs` 使用 readiness v132，只有独立批准才产生 Stage 136 隔离物化器规格登记候选。
- 前端：`packages/app/src/components/public-admin-opening-portfolio-snapshot-materialization-implementation-review-panel.tsx` 及 `.test.ts`，并接入 `api.ts`、`api.test.ts`、`types.ts`、历史治理页和统一决策大脑卡片。
- 存储：`investment_decisions/opening-portfolio-snapshot-materialization-implementation-reviews/{implementation_id}/{review_id}.json`。当前目录不存在、零真实审查；没有 key/input read、receipt 解密、parser/runtime、快照、财务、训练或交易状态。
