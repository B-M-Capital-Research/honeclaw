import { For, Show, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptValidations,
  validateOpeningPortfolioSourceArtifactReceiptOnce,
} from "@/lib/api";
import type {
  OpeningPortfolioSourceArtifactReceiptValidationRegistry,
  ValidateOpeningPortfolioSourceArtifactReceiptRequest,
} from "@/lib/types";

const CHECKS = [
  "重新打开并核对 Stage 51–132 完整责任链",
  "验证人独立于 Stage 132 执行人、Stage 131 领取人及完整前序责任链",
  "独立重算 Stage 132 result 与 receipt manifest 指纹",
  "manifest 与内容地址路径均由服务端推导，不接受客户端路径",
  "逐件核验密文为只读普通文件，并重算长度和 SHA-256",
  "核验密钥指纹，并以独立实现完成 AES-256-GCM 认证解密",
  "独立重算明文长度、SHA-256 与内容地址",
  "再次独立执行格式 magic、安全结构及敏感字段筛查",
  "receipt 不含原文件名、账号、凭证或秘密",
  "验证记录只创建一次、形成终态且不可重放",
  "本阶段只验证 receipt，不解析金融行、不物化期初快照",
  "不开放快照准入、金融白名单、账本、持仓、现金、净值、模型、训练、订单、券商或交易权限",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSourceArtifactReceiptValidationPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptValidationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptValidations();
      setRegistry(next);
      if (!next.candidates.some((item) => item.stage_131_attempt_id === selectedId())) {
        setSelectedId(next.candidates[0]?.stage_131_attempt_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 133 receipt 独立验证表读取失败");
    }
  };
  onMount(() => void load());

  const ready = () => Boolean(
    registry()?.encryption_key_configured
      && selectedId()
      && reason().trim()
      && checks().every(Boolean),
  );

  const submit = async () => {
    const candidate = registry()?.candidates.find((item) => item.stage_131_attempt_id === selectedId());
    if (!candidate || busy() || !ready()) return;
    const values = checks();
    const request: ValidateOpeningPortfolioSourceArtifactReceiptRequest = {
      expected_stage_131_claim_sha256: candidate.stage_131_claim_sha256,
      expected_stage_132_result_sha256: candidate.stage_132_result_sha256,
      expected_receipt_manifest_sha256: candidate.receipt_manifest_sha256,
      expected_stage_130_authorization_review_sha256: candidate.stage_130_authorization_review_sha256,
      expected_stage_129_isolated_receiver_spec_sha256: candidate.stage_129_isolated_receiver_spec_sha256,
      expected_stage_127_implementation_contract_sha256: candidate.stage_127_implementation_contract_sha256,
      expected_stage_125_specification_sha256: candidate.stage_125_specification_sha256,
      validation_reason: reason().trim(),
      exact_stage_51_through_stage_132_chain_reopened_confirmed: values[0] as boolean,
      validator_independent_from_stage_132_executor_stage_131_claimant_and_complete_prior_chain_confirmed: values[1] as boolean,
      result_and_receipt_fingerprints_independently_recomputed_confirmed: values[2] as boolean,
      server_derived_manifest_and_content_addressed_paths_only_confirmed: values[3] as boolean,
      ciphertext_regular_read_only_size_and_sha256_recomputed_confirmed: values[4] as boolean,
      encryption_key_fingerprint_and_aead_authenticated_decryption_confirmed: values[5] as boolean,
      plaintext_length_sha256_and_content_address_independently_recomputed_confirmed: values[6] as boolean,
      format_magic_safe_structure_and_sensitive_field_screening_independently_repeated_confirmed: values[7] as boolean,
      receipt_redaction_and_no_original_filename_account_number_or_credential_confirmed: values[8] as boolean,
      terminal_create_once_validation_no_replay_confirmed: values[9] as boolean,
      receipt_validation_only_no_financial_row_parsing_or_snapshot_materialization_confirmed: values[10] as boolean,
      no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[11] as boolean,
      no_unconfirmed_hari_or_old_wang_logic_claimed: values[12] as boolean,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await validateOpeningPortfolioSourceArtifactReceiptOnce(candidate.stage_131_attempt_id, request));
      setReason("");
      setChecks(CHECKS.map(() => false));
      setNotice("已形成不可变的独立验证终态；通过只说明加密 receipt 完整可信，仍不是实际持仓。下一步仅开放第 134 阶段零能力实现登记。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 133 独立验证失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="来源工件 receipt 独立验证">
    <header><strong>第 133 阶段 · 加密 receipt 责任链外独立验证</strong><span>{current().next_gate}</span></header>
    <p>{current().scope}</p>
    <div class="public-admin-decision-metrics">
      <div><span>未受信 receipt</span><strong>{current().completed_untrusted_receipt_count}</strong></div>
      <div><span>待独立验证</span><strong>{current().pending_independent_validation_count}</strong></div>
      <div><span>验证通过</span><strong>{current().independently_validated_receipt_count}</strong></div>
      <div><span>终态失败</span><strong>{current().failed_independent_validation_count}</strong></div>
    </div>
    <p><strong>边界：</strong>这里只验证来源文件的保管、密文、认证解密、内容哈希、格式和脱敏完整性；不证明文件里的持仓数字真实。</p>
    <Show when={!current().encryption_key_configured}><p class="public-admin-error">独立验证环境没有同一 AES-256 密钥。当前会在创建终态前失败，修复配置后仍可验证。</p></Show>
    <Show when={current().candidates.length > 0} fallback={<p>当前没有待验证 receipt；零状态、已验证或已有终态均符合设计。</p>}>
      <label><span>Stage 132 未受信 receipt</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().candidates}>{(candidate) => <option value={candidate.stage_131_attempt_id}>{candidate.receipt_id.slice(0, 8)} · {candidate.artifact_count} 件 · {candidate.total_original_byte_length} 字节</option>}</For></select></label>
      <label><span>独立验证理由</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} placeholder="说明本次责任链外验证的身份、目的和范围" /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((items) => items.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={busy() || !ready()} onClick={() => void submit()}>{busy() ? "正在独立重开并校验…" : "创建一次性独立验证终态"}</button>
    </Show>
    <Show when={current().validations.length > 0}><div class="public-admin-review-history"><For each={current().validations}>{(validation) => <article><strong>{validation.source_artifact_receipt_independently_validated ? "独立验证通过" : "独立验证失败"}</strong><span>{validation.validation_id.slice(0, 8)} · {validation.artifact_count} 件 · {validation.mismatch_reasons.join("；") || "只开放 Stage 134"}</span></article>}</For></div></Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
