import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationMaterializationSpecifications,
  registerControlledShadowObservationMaterializationSpecification,
} from "@/lib/api";
import type { ControlledShadowObservationMaterializationSpecificationRegistry } from "@/lib/types";

const SPECIFICATION_CHECKS = [
  "精确绑定当前 Stage 51–104 完整责任链",
  "登记者独立于 Stage 104 reviewer 和全部既有责任人",
  "只投影已准入输出，不重新抓取或重新解析行情",
  "保留保守 available_at 下限及供应商发布时间未验证限制",
  "逐官方交易日保留股票、SPY 和三种价格口径矩阵",
  "个股缺失只记显式 gap，不填充、插值或跨口径替代",
  "分红、拆股和三种价格口径继续分别保存",
  "只绑定既有初始影子组合，不重算组合或执行会计转换",
  "固定排序、原始十进制字符串和逐行摘要规则",
  "每周期一个 create-once envelope，不覆盖、回填或原地纠错",
  "SPY 缺失、重复、越界或摘要漂移一律失败关闭",
  "仅登记规格，没有实现、工件、入口、runtime 或输入挂载",
  "没有网络、环境继承、密钥、工具、子进程或生产读写",
  "不生成观察、账本、持仓、绩效、模型、训练、reward、订单、券商或交易",
  "实现登记前必须先通过 Stage 106 责任链外独立复核",
  "未把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationMaterializationSpecificationPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowObservationMaterializationSpecificationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [limitations, setLimitations] = createSignal("供应商发布时间仍未验证；规格只能保留 Stage 104 的保守 available_at，未来输出仍为非可信观察候选。");
  const [constraints, setConstraints] = createSignal("Stage 106 必须由责任链外角色独立重算规格摘要、矩阵完整性、缺口语义、初始组合绑定和全部零权限边界。");
  const [checks, setChecks] = createSignal(SPECIFICATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationMaterializationSpecifications();
      setRegistry(next);
      if (!next.candidates.some((item) => item.stage_104_review_id === selectedReviewId())) {
        setSelectedReviewId(next.candidates[0]?.stage_104_review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 105 观察物化规格登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.candidates.find(
    (item) => item.stage_104_review_id === selectedReviewId(),
  ));
  const disabled = createMemo(() => busy() || !selected() || !reason().trim()
    || !limitations().trim() || !constraints().trim() || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = checks();
      const next = await registerControlledShadowObservationMaterializationSpecification(
        item.stage_104_review_id,
        {
          expected_stage_104_review_sha256: item.stage_104_review_sha256,
          expected_stage_103_validation_sha256: item.stage_103_validation_sha256,
          expected_stage_102_result_sha256: item.stage_102_result_sha256,
          expected_stage_102_output_sha256: item.stage_102_output_sha256,
          expected_stage_101_claim_sha256: item.stage_101_claim_sha256,
          expected_stage_101_input_manifest_sha256: item.stage_101_input_manifest_sha256,
          expected_cycle_claim_sha256: item.cycle_claim_sha256,
          registration_reason: reason().trim(),
          known_limitations: limitations().trim(),
          future_review_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_104_binding_confirmed: values[0] as boolean,
          registrar_independent_from_stage_104_and_complete_prior_chain_confirmed: values[1] as boolean,
          exact_admitted_output_only_no_refetch_or_reparse_confirmed: values[2] as boolean,
          conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed: values[3] as boolean,
          official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: values[4] as boolean,
          subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed: values[5] as boolean,
          dividends_splits_and_price_bases_remain_separate_confirmed: values[6] as boolean,
          initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed: values[7] as boolean,
          deterministic_canonical_order_decimal_and_row_hash_rules_confirmed: values[8] as boolean,
          one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed: values[9] as boolean,
          spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed: values[10] as boolean,
          specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: values[11] as boolean,
          no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed: values[12] as boolean,
          no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: values[13] as boolean,
          future_chain_external_specification_review_required_before_implementation_confirmed: values[14] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[15] as boolean,
        },
      );
      setRegistry(next);
      setNotice("Stage 105 零能力规格已 create-once 登记；只进入 Stage 106 独立复核，观察尚未生成。");
      setReason("");
      setChecks(SPECIFICATION_CHECKS.map(() => false));
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 105 观察物化规格登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="首次自然前向周期观察物化规格登记">
      <header><strong>第 105 阶段 · 观察物化规格</strong><span>create-once · 零执行能力</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>已准入输入</span><strong>{current().admitted_input_count}</strong></div>
        <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记</span><strong>{current().specification_registered_count}</strong></div>
        <div><span>待独立复核</span><strong>{current().future_chain_external_specification_review_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">本阶段不会生成观察文件；只冻结未来确定性投影的 schema、缺口、口径、摘要和失败关闭规则。</p>
      <Show when={current().candidates.length > 0} fallback={<p>当前没有待登记的 Stage 104 已准入输入。</p>}>
        <label><span>Stage 104 已准入输入</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
          <For each={current().candidates}>{(item) => <option value={item.stage_104_review_id}>
            {item.subject_symbols.join(", ")} · {item.official_market_session_count} 个交易日 · {item.stage_104_review_sha256.slice(0, 12)}…
          </option>}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>冻结矩阵</strong><span>基准 {item().benchmark_symbol}</span></header>
          <p>保守 available_at：{item().admitted_available_at_utc}</p>
          <p>官方交易日 {item().official_market_session_count} · 显式缺口 {item().explicit_gap_count} · 三价格口径分别保留</p>
        </article>}</Show>
        <label><span>登记理由</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>未来复核约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={SPECIFICATION_CHECKS}>{(label, index) => <label><input
          type="checkbox" checked={checks()[index()]}
          onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))}
        /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
          {busy() ? "正在写入不可变规格…" : "登记 Stage 105 零能力物化规格"}
        </button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().registrations}>{(registration) => <article class="public-admin-reward-governance">
        <header><strong>规格已登记 · 等待 Stage 106</strong><span>{registration.registered_at}</span></header>
        <p>{registration.specification.subject_symbols.join(", ")} · {registration.specification.official_market_session_count} 个交易日 · SPY</p>
        <p>三口径：{registration.specification.allowed_price_bases.join(" / ")}</p>
        <p>未来输出：{registration.specification.future_output_relative_path_template}</p>
        <p class="public-admin-anchor-boundary">{registration.known_limitations}</p>
      </article>}</For>
    </section>
  )}</Show>;
}
