import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserImplementationReviews,
  reviewControlledShadowMarketDataParserImplementationOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataParserImplementationReviewRegistry,
  ReviewControlledShadowMarketDataParserImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–97 当前不可变责任链",
  "复核者不是 Stage 97 登记者，也不是此前完整责任链成员",
  "已独立重算实现契约、规格复核、规格登记和 parser 规格哈希",
  "已逐项复核八个确定性函数标识及规范化 schema",
  "价格、分红、拆股和官方 NYSE 日历来源保持显式分离",
  "UTF-8、JSON/HTML、日期与有限数值边界保持严格",
  "重复、越界、缺失和格式错误均失败关闭",
  "不去重、不前填、不插值、不回退、不推断公司行动",
  "SPY 官方交易日覆盖、标的缺口和跨来源对账均显式处理",
  "已独立重建全部八组合成测试向量",
  "source_available_at 仍未验证，须等待独立证据链",
  "未来输出仍须内容寻址、create-once、不可信并独立校验",
  "没有源码/可执行工件、入口、runtime、原始载荷挂载读取、环境、秘密、网络、工具或子进程",
  "没有解析行、观察、账本、持仓、绩效、模型、训练、奖励、订单、券商或交易权限",
  "通过只开放未来 Stage 99 隔离 parser runner 规格登记资格",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  rationale: "",
  binding_and_recomputation_assessment: "",
  deterministic_parser_semantics_assessment: "",
  source_schema_calendar_action_and_reconciliation_assessment: "",
  failure_and_missing_data_assessment: "",
  zero_capability_assessment: "",
  known_limitations: "",
  future_runner_constraints: "",
});

export function PublicAdminControlledShadowMarketDataParserImplementationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<
    ReviewControlledShadowMarketDataParserImplementationRequest["verdict"]
  >("approved_for_future_isolated_market_data_parser_runner_specification_registration");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserImplementationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(eligible[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 98 实现独立复核表读取失败");
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
    const request: ReviewControlledShadowMarketDataParserImplementationRequest = {
      expected_previous_review_id: item.latest_review?.review_id,
      expected_previous_review_sha256: item.latest_review?.review_sha256,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: implementation.implementation_contract.contract_sha256,
      expected_specification_review_sha256: audit.specification_review_sha256,
      expected_specification_registration_sha256: audit.specification_registration_sha256,
      expected_parser_specification_sha256: audit.parser_specification_sha256,
      expected_independent_audit_sha256: audit.audit_sha256,
      verdict: verdict(),
      ...fields(),
      exact_current_stage_51_through_stage_97_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed: true,
      all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
      explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: true,
      strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: true,
      duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
      spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed: true,
      all_eight_synthetic_vectors_independently_reconstructed_confirmed: true,
      source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(
        await reviewControlledShadowMarketDataParserImplementationOnce(
          implementation.implementation_id,
          request,
        ),
      );
      setFields(emptyFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 98 责任链外复核已 create-once 写入；通过也只开放 Stage 99 runner 规格登记资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 98 实现独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const textFields = [
    ["rationale", "复核结论与理由"],
    ["binding_and_recomputation_assessment", "全链绑定与独立重算评估"],
    ["deterministic_parser_semantics_assessment", "确定性解析语义评估"],
    ["source_schema_calendar_action_and_reconciliation_assessment", "来源、schema、交易日、公司行动与对账评估"],
    ["failure_and_missing_data_assessment", "失败关闭与缺失数据评估"],
    ["zero_capability_assessment", "零能力边界评估"],
    ["known_limitations", "已知限制"],
    ["future_runner_constraints", "未来 Stage 99 runner 规格约束"],
  ] as const;

  return (
    <Show when={registry()}>
      {(current) => (
        <section
          class="public-admin-reward-governance"
          aria-label="Stage 98 行情解析器实现责任链外独立复核"
        >
          <header>
            <strong>第 98 阶段 · 行情解析器实现责任链外独立复核</strong>
            <span>{current().review_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>Stage 97 契约</span><strong>{current().implementation_count}</strong></div>
            <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
            <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
          </div>
          <p class="public-admin-anchor-boundary">
            第二套实现独立重算 Stage 97 实现与契约、Stage 96 复核、Stage 95 登记和规格哈希；不读取任何原始载荷。
          </p>
          <Show when={current().review_eligible_count > 0} fallback={<p>当前没有待责任链外复核的 Stage 97 实现契约。</p>}>
            <label>
              <span>Stage 97 实现契约</span>
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
                    event.currentTarget.value as ReviewControlledShadowMarketDataParserImplementationRequest["verdict"],
                  )}
              >
                <option value="approved_for_future_isolated_market_data_parser_runner_specification_registration">独立通过，仅开放未来 Stage 99 runner 规格登记</option>
                <option value="changes_required_rebuild_market_data_parser_implementation_contract">要求修改并重建不可变实现契约</option>
                <option value="rejected_market_data_parser_implementation_contract">拒绝实现契约</option>
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
                          )
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
              {busy() ? "正在独立复核…" : "写入 Stage 98 终态复核"}
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
                  Stage 99 runner 规格登记资格：{item.latest_review!.future_isolated_parser_runner_specification_registration_eligible ? "已开放" : "未开放"}；仍无 parser runner、载荷访问或交易权限。
                </p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
