import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionImplementationReviews,
  reviewControlledShadowObservationLedgerTransitionImplementationOnce,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionImplementationReviewRegistry,
  ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–117 当前不可变责任链",
  "复核者不是 Stage 117 登记者，也不是此前完整责任链成员",
  "已用第二套实现完整重建 Stage 117 合同，不调用 Stage 117 builder",
  "已独立重算实现、合同、Stage 116 复核/审计、Stage 115 登记/规格哈希",
  "已逐项复核八个纯合同函数及 canonical event/双分录 schema",
  "未来唯一输入仍是 Stage 114 当前准入观察证据",
  "Stage 88 只是初始化来源，不能冒充 opening portfolio snapshot",
  "不得默认或推断本金、现金、持仓、股数和目标权重",
  "raw close 才能用于未来证券会计，adjusted prices 只作非会计研究",
  "显式 gap 阻断 NAV/绩效，不填充、不插值、不跨口径替代",
  "分红和拆股在持仓与条款独立准入前只能记 notice",
  "精确十进制、幂等事件、双分录、append-only 纠错与保守 available-at 保持不变",
  "provider_publication_time 仍未验证，不得伪装为已知",
  "未来 ledger/event stream 只能 create-once、append-only 且必须独立验证",
  "没有源码/可执行工件、入口、runtime、输入挂载读取、环境、秘密、网络、工具或子进程",
  "没有 opening snapshot、账本/事件、持仓、现金、NAV/绩效、训练/RL/reward、订单、券商或交易权限",
  "通过只开放未来 Stage 119 隔离账本转换 runner 规格登记资格",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  rationale: "",
  binding_and_recomputation_assessment: "",
  deterministic_projection_semantics_assessment: "",
  session_price_basis_gap_and_company_action_assessment: "",
  initial_allocation_availability_and_output_assessment: "",
  zero_capability_assessment: "",
  known_limitations: "",
  future_runner_constraints: "",
});

export function PublicAdminControlledShadowObservationLedgerTransitionImplementationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<
    ReviewControlledShadowObservationLedgerTransitionImplementationRequest["verdict"]
  >("approved_for_future_isolated_observation_ledger_transition_runner_specification_registration");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionImplementationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(eligible[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 118 实现独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.review_eligible && item.implementation.implementation_id === selectedId(),
    ),
  );
  const disabled = createMemo(
    () =>
      busy()
      || !selected()
      || Object.values(fields()).some((value) => !value.trim())
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const audit = item.current_independent_audit;
    const request: ReviewControlledShadowObservationLedgerTransitionImplementationRequest = {
      expected_previous_review_id: item.latest_review?.review_id ?? null,
      expected_previous_review_sha256: item.latest_review?.review_sha256 ?? null,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: implementation.implementation_contract.contract_sha256,
      expected_specification_review_sha256: audit.specification_review_sha256,
      expected_specification_independent_audit_sha256: audit.specification_independent_audit_sha256,
      expected_specification_registration_sha256: audit.specification_registration_sha256,
      expected_observation_ledger_transition_specification_sha256:
        audit.observation_ledger_transition_specification_sha256,
      expected_independent_audit_sha256: audit.audit_sha256,
      verdict: verdict(),
      ...fields(),
      exact_current_stage_51_through_stage_117_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        true,
      all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
      exact_stage_114_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: true,
      initial_shadow_allocation_and_conservative_availability_preserved_confirmed: true,
      provider_publication_time_remains_unverified_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        true,
      approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed:
        true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(
        await reviewControlledShadowObservationLedgerTransitionImplementationOnce(
          implementation.implementation_id,
          request,
        ),
      );
      setFields(emptyFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 118 责任链外复核已 create-once 写入；通过也只开放 Stage 119 runner 规格登记资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 118 实现独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const textFields = [
    ["rationale", "复核结论与理由"],
    ["binding_and_recomputation_assessment", "全链绑定与独立重算评估"],
    ["deterministic_projection_semantics_assessment", "确定性账本转换语义评估"],
    ["session_price_basis_gap_and_company_action_assessment", "会计价格、缺口与公司行动评估"],
    ["initial_allocation_availability_and_output_assessment", "opening portfolio、可用时间与输出评估"],
    ["zero_capability_assessment", "零能力边界评估"],
    ["known_limitations", "已知限制"],
    ["future_runner_constraints", "未来 Stage 119 runner 规格约束"],
  ] as const;

  return (
    <Show when={registry()}>
      {(current) => (
        <section
          class="public-admin-reward-governance"
          aria-label="Stage 118 账本转换实现责任链外独立复核"
        >
          <header>
            <strong>第 118 阶段 · 账本转换实现责任链外独立复核</strong>
            <span>{current().review_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>Stage 117 契约</span><strong>{current().implementation_count}</strong></div>
            <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
            <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
          </div>
          <p class="public-admin-anchor-boundary">
            第二套实现独立重建 Stage 117 完整合同并重算 Stage 116/115 全部绑定；不生成工件、opening snapshot、账本、NAV 或交易能力。
          </p>
          <Show
            when={current().review_eligible_count > 0}
            fallback={<p>当前没有待责任链外复核的 Stage 117 实现契约。</p>}
          >
            <label>
              <span>Stage 117 实现契约</span>
              <select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
                <For each={current().items.filter((item) => item.review_eligible)}>
                  {(item) => (
                    <option value={item.implementation.implementation_id}>
                      {item.implementation.implementation_id.slice(0, 12)}… · {item.implementation.implementation_name}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select
                value={verdict()}
                onChange={(event) =>
                  setVerdict(
                    event.currentTarget.value as ReviewControlledShadowObservationLedgerTransitionImplementationRequest["verdict"],
                  )}
              >
                <option value="approved_for_future_isolated_observation_ledger_transition_runner_specification_registration">独立通过，仅开放未来 Stage 119 runner 规格登记</option>
                <option value="changes_required_rebuild_observation_ledger_transition_implementation">要求修改并重建不可变实现契约</option>
                <option value="rejected_observation_ledger_transition_implementation">拒绝实现契约</option>
              </select>
            </label>
            <For each={textFields}>
              {([key, label]) => (
                <label>
                  <span>{label}</span>
                  <textarea
                    value={fields()[key]}
                    onInput={(event) =>
                      setFields((value) => ({ ...value, [key]: event.currentTarget.value }))
                    }
                  />
                </label>
              )}
            </For>
            <div class="public-admin-decision-checks">
              <For each={CHECKS}>
                {(label, index) => (
                  <label>
                    <input
                      type="checkbox"
                      checked={checks()[index()]}
                      onChange={(event) =>
                        setChecks((values) =>
                          values.map((value, i) =>
                            i === index() ? event.currentTarget.checked : value,
                          ),
                        )
                      }
                    />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button
              type="button"
              class="public-admin-decision-submit"
              disabled={disabled()}
              onClick={() => void submit()}
            >
              {busy() ? "正在独立复核…" : "写入 Stage 118 终态复核"}
            </button>
          </Show>
          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items.filter((item) => item.latest_review)}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>review {item.latest_review!.review_id}</strong>
                  <span>{item.latest_review!.verdict}</span>
                </header>
                <p>
                  独立审计 {item.latest_review!.independent_audit.mismatch_reasons.length === 0 ? "通过" : "未通过"}
                  {" · "}实现 {item.latest_review!.implementation.implementation_sha256.slice(0, 16)}…
                </p>
                <Show when={item.latest_review!.independent_audit.mismatch_reasons.length > 0}>
                  <p class="public-admin-error">{item.latest_review!.independent_audit.mismatch_reasons.join("；")}</p>
                </Show>
                <p class="public-admin-anchor-boundary">
                  Stage 119 runner 规格登记资格：{item.latest_review!.future_isolated_observation_ledger_transition_runner_specification_registration_eligible ? "已开放" : "未开放"}；仍无 runner、输入读取、opening snapshot、账本、NAV 或交易权限。
                </p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
