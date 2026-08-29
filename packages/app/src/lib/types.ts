export type UserInfo = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  session_id: string;
  session_kind: "direct" | "group" | string;
  session_label: string;
  actor_user_id?: string;
  last_message: string;
  last_role: string;
  last_time: string;
  message_count: number;
};

export type SkillInfo = {
  id: string;
  display_name: string;
  description: string;
  when_to_use?: string;
  aliases: string[];
  allowed_tools: string[];
  user_invocable: boolean;
  context: string;
  loaded_from: string;
  enabled: boolean;
  disabled_reason?: string;
  has_script: boolean;
  has_path_gate: boolean;
  paths: string[];
};

export type SkillDetailInfo = {
  summary: SkillInfo;
  markdown: string;
  detail_path: string;
};

export type HistoryMsg = {
  role: "user" | "assistant" | "system" | string;
  content: string;
  /** RFC3339 时间戳，用于聊天记录分组与日期分隔。 */
  at?: string;
  subtype?:
    "compact_boundary" | "compact_summary" | "compact_skill_snapshot" | string;
  synthetic?: boolean;
  transcript_only?: boolean;
  attachments: HistoryAttachment[];
  scheduled_push?: HistoryScheduledPush;
  finance_calendar?: HistoryFinanceCalendar;
};

export type HistoryFinanceCalendar = {
  month: string;
  image_path: string;
  variant: "desktop" | "mobile" | string;
};

export type PublicHistoryPageResponse = {
  messages: HistoryMsg[];
  history_start: number;
  next_before?: number | null;
};

/** Server-authoritative state for one public-chat run that is still active. */
export type PublicChatActiveRun = {
  run_id: string;
  started_at_ms: number;
  phase: "thinking" | "running";
  status_text: string;
  updated_at_ms: number;
  /** Stages this run already passed through, newest last. */
  steps?: string[];
};

export type PublicChatBootstrapResponse = PublicHistoryPageResponse & {
  user: PublicAuthUserInfo;
  active_run?: PublicChatActiveRun | null;
  interrupted_run?: boolean;
};

export type HistoryScheduledPush = {
  push_id?: string;
  title: string;
  summary: string;
  fallback_content?: string;
};

export type PublicPushListItem = {
  push_id: string;
  job_id: string;
  title: string;
  summary: string;
  created_at: string;
};

export type PublicPushDetail = PublicPushListItem & {
  content: string;
};

export type PublicPushListResponse = {
  items: PublicPushListItem[];
  unread_count: number;
  next_before?: string | null;
};

export type PublicPushOpenResponse = {
  push: PublicPushDetail;
  unread_count: number;
};

export type PublicCommunityResource = {
  resource_id: number;
  ordinal: number;
  resource_kind: "image" | "file" | string;
  display_name?: string | null;
  content_type?: string | null;
  byte_size?: number | null;
  version?: string | null;
  /** Same-origin private edge path; legacy API responses intentionally omit it. */
  delivery_path?: string | null;
  access_state: "stored" | "protected_in_app" | "metadata_only" | string;
};

export type PublicCommunityContent = {
  content_id: number;
  author_name: string;
  published_at?: string | null;
  published_at_raw?: string | null;
  content_type: string;
  body_text: string;
  body_blocks: unknown[];
  crawl_status: "complete" | "partial" | string;
  resources: PublicCommunityResource[];
};

export type PublicCommunityPage = {
  community: { id: string; name: string };
  items: PublicCommunityContent[];
  next_before?: number | null;
  unread: boolean;
  latest_content_id?: number | null;
};

export type CommunityForumAttachment = {
  id: string;
  filename: string;
  content_type: string;
  byte_size: number;
  sha256: string;
};

export type CommunityForumComment = {
  id: string;
  author_label: string;
  body: string;
  created_at: string;
  moderation_status: "visible" | "deleted" | string;
  can_delete: boolean;
};

export type CommunityForumPost = {
  id: string;
  author_label: string;
  title: string;
  body: string;
  tickers: string[];
  topics: string[];
  source_url?: string | null;
  created_at: string;
  updated_at: string;
  moderation_status: "visible" | "pending_review" | "hidden" | "deleted" | string;
  attachment?: CommunityForumAttachment | null;
  like_count: number;
  liked_by_me: boolean;
  report_count?: number | null;
  can_delete: boolean;
  comments: CommunityForumComment[];
};

export type CommunityForumPage = {
  items: CommunityForumPost[];
  is_admin: boolean;
  policy: {
    forum_content_is_research: false;
    attachment_max_bytes: number;
    auto_hide_report_count: number;
  };
};

export type HistoryAttachment = {
  path: string;
  name: string;
  kind: "image" | "pdf" | "file" | string;
};

export type FinanceCalendarEvent = {
  date: string;
  title: string;
  kind: "macro" | "earnings" | string;
  ticker?: string;
  subtitle?: string;
  source: string;
};

export type FinanceCalendarMonth = {
  value: string;
  label: string;
};

export type FinanceCalendarPayload = {
  today: string;
  month: string;
  months: FinanceCalendarMonth[];
  holdings: string[];
  events: FinanceCalendarEvent[];
  earnings_status: string;
  errors: string[];
};

export type WebInviteInfo = {
  user_id: string;
  invite_code: string;
  phone_number: string;
  created_at: string;
  last_login_at?: string;
  revoked_at?: string;
  api_key_prefix?: string;
  api_key_created_at?: string;
  api_key_last_used_at?: string;
  api_key?: string;
  enabled: boolean;
  active_session_count: number;
  daily_limit: number;
  success_count: number;
  in_flight: number;
  remaining_today: number;
};

export type WebInviteActionResult = {
  invite: WebInviteInfo;
  cleared_session_count: number;
  message: string;
};

export type PublicAuthUserInfo = {
  user_id: string;
  created_at: string;
  last_login_at?: string;
  daily_limit: number;
  success_count: number;
  in_flight: number;
  remaining_today: number;
  has_password: boolean;
  tos_accepted_at?: string;
  tos_version?: string;
  identity_kind: "domestic_invite" | "international_email" | string;
  email_hint?: string;
  billing: PublicBillingSummary;
  is_admin: boolean;
};

export type PublicBillingEntitlement = {
  entitlement_id: string;
  provider: "stripe" | "domestic_invite" | string;
  entitlement_kind:
    | "recurring_subscription"
    | "fixed_term_purchase"
    | "domestic_invite"
    | string;
  raw_status: string;
  access_state: "pending" | "active" | "grace" | "inactive" | string;
  grants_access: boolean;
  current_period_start?: string;
  current_period_end?: string;
  cancel_at_period_end: boolean;
  manage_url?: string;
  grace_expires_at?: string;
};

export type PublicBillingSummary = {
  access_granted: boolean;
  entitlements: PublicBillingEntitlement[];
  has_duplicate_active_subscriptions: boolean;
};

export type PublicBillingConfig = {
  stripe: {
    subscription: PublicBillingOfferConfig;
    fixed_term: PublicBillingOfferConfig;
  };
  purchases_allowed_on_this_client: boolean;
  management_allowed_on_this_client: boolean;
};

export type PublicBillingOfferConfig = {
  enabled: boolean;
  amount_minor: number;
  currency: string;
  term_months: number;
  auto_renews: boolean;
};

export type PublicBillingStatus = {
  billing: PublicBillingSummary;
  config: PublicBillingConfig;
};

export type PublicAdminInviteInfo = {
  user_id: string;
  phone_number: string;
  created_at: string;
  last_login_at?: string;
  enabled: boolean;
  can_disable: boolean;
};

export type PublicAdminInviteList = {
  invites: PublicAdminInviteInfo[];
  daily_create_limit: number;
  created_today: number;
  remaining_today: number;
};

export type PublicAdminInviteMutation = {
  invite: PublicAdminInviteInfo;
  daily_create_limit: number;
  created_today: number;
  remaining_today: number;
  cleared_session_count: number;
  message: string;
};

export type PublicAdminUsageQuestion = {
  asked_at: string;
  text: string;
};

export type PublicAdminUsageRow = {
  date: string;
  channel: "web" | "feishu" | "telegram" | "discord" | "imessage" | string;
  user_id: string;
  user_label: string;
  question_count: number;
  questions: PublicAdminUsageQuestion[];
  scheduled_run_count: number;
  delivered_push_count: number;
  failed_delivery_count: number;
  latest_activity_at: string;
};

export type PublicAdminUsageReport = {
  generated_at: string;
  period_days: number;
  period_start: string;
  period_end: string;
  summary: {
    today: string;
    today_active_users: number;
    today_question_count: number;
    today_delivered_push_count: number;
    last_week_same_day_active_users: number;
    active_user_change: number;
    leading_decline_user_label?: string;
    leading_decline_question_delta: number;
    text: string;
  };
  rows: PublicAdminUsageRow[];
};

export type InvestmentResearchZone =
  | "opportunity"
  | "hold"
  | "risk"
  | "insufficient_data";

export type InvestmentExposureAction =
  | "increase_candidate"
  | "maintain"
  | "reduce_candidate"
  | "research_only";

export type InvestmentReviewStatus =
  | "pending"
  | "accepted"
  | "corrected"
  | "rejected";

export type InvestmentThesisVerdict =
  | "pending"
  | "supported"
  | "weakened"
  | "invalidated"
  | "inconclusive";

export type InvestmentDecisionCompletenessCheck = {
  check_id: string;
  label: string;
  status: "pass" | "partial" | "missing";
  required_for_directional_research: boolean;
  required_for_portfolio_decision: boolean;
  evidence: string[];
  gaps: string[];
};

export type InvestmentDecisionCompleteness = {
  policy_version: string;
  status: "research_incomplete" | "directional_research_ready" | "portfolio_ready" | string;
  passed_checks: number;
  total_checks: number;
  completeness_percent: number;
  directional_research_ready: boolean;
  portfolio_decision_ready: boolean;
  checks: InvestmentDecisionCompletenessCheck[];
  scope: string;
};

export type InvestmentCrowdingComponent = {
  component_id: string;
  label: string;
  raw_value_percent: number;
  pressure_score: number;
  weight: number;
  as_of: string;
  source: string;
};

export type CompanyShortInterest = {
  policy_version: string;
  as_of: string;
  source: string;
  source_url: string;
  current_shares_short: number;
  previous_shares_short: number;
  change_percent: number;
  average_daily_share_volume: number;
  days_to_cover: number;
  observation_count: number;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
  interpretation: string;
};

export type CompanyOptionsPositioning = {
  policy_version: string;
  as_of: string;
  source: string;
  source_url: string;
  expiration_date: string;
  days_to_expiration: number;
  spot_price: number;
  call_open_interest: number;
  put_open_interest: number;
  put_call_open_interest_ratio?: number | null;
  call_volume: number;
  put_volume: number;
  put_call_volume_ratio?: number | null;
  contract_rows: number;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
  interpretation: string;
};

export type CompanyNewsAttention = {
  policy_version: string;
  as_of: string;
  source: string;
  source_url: string;
  window_days: number;
  recent_window_days: number;
  recent_article_count: number;
  prior_article_count: number;
  recent_daily_rate: number;
  prior_daily_rate: number;
  activity_ratio?: number | null;
  unique_publishers: number;
  observed_article_count: number;
  oldest_observed_date: string;
  result_limit: number;
  truncated_window: boolean;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
  interpretation: string;
};

export type CompanyInstitutionalHoldings = {
  policy_version: string;
  observed_on: string;
  source: string;
  source_url: string;
  institutional_ownership_percent: number;
  institutional_holders: number;
  total_shares_held: number;
  total_reported_records: number;
  top_sample_rows: number;
  holder_table_truncated: boolean;
  earliest_report_period: string;
  latest_report_period: string;
  report_period_count: number;
  latest_period_rows_in_sample: number;
  increased_positions_holders: number;
  increased_positions_shares: number;
  decreased_positions_holders: number;
  decreased_positions_shares: number;
  held_positions_holders: number;
  held_positions_shares: number;
  new_positions_holders: number;
  new_positions_shares: number;
  sold_out_positions_holders: number;
  sold_out_positions_shares: number;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
  interpretation: string;
};

export type CompanyAnalystConsensus = {
  policy_version: string;
  observed_on: string;
  source: string;
  source_url: string;
  buy_count: number;
  hold_count: number;
  sell_count: number;
  recommendation_count: number;
  buy_share_percent: number;
  hold_share_percent: number;
  sell_share_percent: number;
  dominant_rating: string;
  dominant_count: number;
  dominant_share_percent: number;
  consensus_target_price: number;
  low_target_price: number;
  high_target_price: number;
  target_range_width_percent: number;
  historical_month_count: number;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
  interpretation: string;
};

export type InvestmentCrowdingState = {
  policy_version: string;
  status: "unmeasured" | "partially_measured" | "measured";
  score?: number | null;
  label: string;
  components: InvestmentCrowdingComponent[];
  short_interest?: CompanyShortInterest | null;
  options_positioning?: CompanyOptionsPositioning | null;
  news_attention?: CompanyNewsAttention | null;
  institutional_holdings?: CompanyInstitutionalHoldings | null;
  analyst_consensus?: CompanyAnalystConsensus | null;
  observations: string[];
  missing_checks: string[];
  scope: string;
};

export type InvestmentDecisionErrorKind =
  | "industry_thesis"
  | "company_value_capture"
  | "financial_transmission"
  | "valuation"
  | "timing_crowding"
  | "data_quality"
  | "policy_mapping"
  | "other";

export type InvestmentDecisionErrorAttribution = {
  kind: InvestmentDecisionErrorKind;
  severity: "minor" | "material" | "critical";
  explanation: string;
  evidence_ids: string[];
};

export type InvestmentCausalObservationRelationship =
  | "direct_metric"
  | "proxy"
  | "confirmed_context"
  | "structured_source_claim"
  | "computed_comparison"
  | "computed_ratio"
  | "computed_ratio_trend"
  | "operating_kpi_claim";

export type InvestmentCausalClaimProvenance = {
  claim_kind: "reported_fact" | "management_guidance" | "management_commentary";
  metric_id: string;
  metric_basis: string;
  period: string;
  numeric_value?: number | null;
  unit: string;
  speaker?: string;
  source_event_id: string;
  source_document: string;
  source_locator: string;
  quote_excerpt: string;
  disposition: "active" | "corrected" | "withdrawn";
  lifecycle_status: "active" | "superseded" | "conflicted" | "withdrawn";
  superseded_by?: string | null;
  conflicting_claim_ids: string[];
};

export type InvestmentCausalComputedProvenance = {
  formula_version: string;
  comparison_kind: "year_over_year" | "sequential_quarter";
  metric_id: string;
  metric_basis: string;
  current_claim_id: string;
  prior_claim_id: string;
  current_period: string;
  prior_period: string;
  current_numeric_value: number;
  prior_numeric_value: number;
  unit: string;
  change_percent: number;
  current_published_at: string;
  prior_published_at: string;
  current_source_url: string;
  prior_source_url: string;
};

export type InvestmentCausalRatioProvenance = {
  formula_version: string;
  ratio_kind: "gross_margin" | "operating_margin";
  metric_id: string;
  numerator_metric_id: string;
  numerator_metric_basis: string;
  numerator_claim_id: string;
  numerator_numeric_value: number;
  denominator_metric_id: string;
  denominator_metric_basis: string;
  denominator_claim_id: string;
  denominator_numeric_value: number;
  period: string;
  result_percent: number;
  published_at: string;
  source_url: string;
};

export type InvestmentCausalRatioTrendProvenance = {
  formula_version: string;
  comparison_kind: "year_over_year" | "sequential_quarter";
  metric_id: string;
  current: InvestmentCausalRatioProvenance;
  prior: InvestmentCausalRatioProvenance;
  change_percentage_points: number;
};

export type InvestmentCausalOperatingKpiProvenance = {
  schema_version: string;
  claim_kind: "reported_fact" | "management_guidance" | "contract_milestone";
  kpi_id: string;
  issuer_metric_name: string;
  issuer_definition: string;
  definition_key: string;
  period: string;
  numeric_value?: number | null;
  unit: string;
  value_text: string;
  measurement_scope: string;
  comparison_basis:
    | "year_over_year"
    | "sequential_quarter"
    | "point_in_time"
    | "period_total"
    | "period_average"
    | "period_end";
  speaker?: string | null;
  source_event_id: string;
  source_document: string;
  source_locator: string;
  evidence_quote: string;
  source_time_precision?: "exact" | "date_only_conservative_end_of_day" | null;
  source_artifact?: {
    schema_version: string;
    source_sha256: string;
    extracted_text_sha256: string;
    byte_length: number;
    format: "pdf" | "html";
    object_path: string;
  } | null;
  definition_changed: boolean;
  disposition: "active" | "corrected" | "withdrawn";
  lifecycle_status: "active" | "superseded" | "conflicted" | "withdrawn";
  superseded_by?: string | null;
  conflicting_claim_ids: string[];
};

export type InvestmentCausalPromotionStatus =
  | "training_only"
  | "pending_repeat_evidence"
  | "pending_human_review"
  | "blocked_conflict"
  | "blocked_human_rejection"
  | "blocked_falsification"
  | "promoted_confidence_only";

export type InvestmentCausalPromotion = {
  policy_version: string;
  status: InvestmentCausalPromotionStatus;
  active_claim_count: number;
  accepted_claim_count: number;
  distinct_source_events: number;
  distinct_periods: number;
  evidence_span_days: number;
  accepted_observation_ids: string[];
  rejected_observation_ids: string[];
  falsifying_observation_ids: string[];
  reasons: string[];
  reviewed_through?: string | null;
};

export type InvestmentCausalObservation = {
  observation_id: string;
  relationship: InvestmentCausalObservationRelationship;
  label: string;
  value: string;
  as_of: string;
  source: string;
  source_url?: string;
  source_tier: string;
  policy_status: "training_only_pending_human_review" | string;
  claim?: InvestmentCausalClaimProvenance | null;
  computed?: InvestmentCausalComputedProvenance | null;
  ratio?: InvestmentCausalRatioProvenance | null;
  ratio_trend?: InvestmentCausalRatioTrendProvenance | null;
  operating_kpi?: InvestmentCausalOperatingKpiProvenance | null;
};

export type InvestmentCausalLinkReview = {
  driver_id: string;
  observation_id: string;
  verdict: "accepted" | "rejected";
  effect?: "unclassified" | "supports" | "falsifies" | "mixed" | "context_only";
  explanation: string;
  verbatim_judgment?: string;
  applicability_boundary?: string;
  falsifier?: string;
  speaker_confirmation?: "unconfirmed" | "source_checked_not_speaker_confirmed" | "old_wang_confirmed" | "old_wang_confirmed_after_source_check" | "evidence_mismatch" | "insufficient_source_context";
  review_id?: string;
  reviewed_at?: string;
  reviewer_id?: string;
  old_wang_reviewer_identity_verified?: boolean;
  evidence_identity_sha256?: string | null;
  source_review_id?: string | null;
  source_review_sample_id?: string | null;
};

export type InvestmentCausalEvidenceReviewRequest = {
  expected_review_id?: string;
  expected_source_review_id?: string;
  driver_id: string;
  observation_id: string;
  verdict: "accepted" | "rejected";
  effect: "unclassified" | "supports" | "falsifies" | "mixed" | "context_only";
  explanation: string;
  verbatim_judgment: string;
  applicability_boundary: string;
  falsifier: string;
  speaker_confirmation: "source_checked_not_speaker_confirmed" | "old_wang_confirmed";
  source_verification: "verified_against_source" | "evidence_mismatch" | "insufficient_source_context";
  source_verification_note: string;
  old_wang_confirmation_attested: boolean;
};

export type InvestmentCausalSourceReview = {
  driver_id: string;
  observation_id: string;
  evidence_fingerprint_sha256: string;
  verdict: "verified_against_source" | "evidence_mismatch" | "insufficient_source_context";
  note: string;
  review_id: string;
  reviewed_at: string;
  reviewer_id: string;
};

export type InvestmentCausalSourceReviewRequest = {
  expected_review_id?: string;
  driver_id: string;
  observation_id: string;
  verdict: "verified_against_source" | "evidence_mismatch" | "insufficient_source_context";
  note: string;
};

export type InvestmentCausalSourceReviewRecord = InvestmentCausalSourceReview & {
  schema_version: string;
  previous_review_id?: string;
  sample_id: string;
  symbol: string;
  submitted_at: string;
  causal_label_created: false;
  training_label_eligible: false;
  thesis_review_unchanged: true;
};

export type InvestmentCausalEvidenceReviewRecord = {
  schema_version: string;
  review_id: string;
  previous_review_id?: string;
  sample_id: string;
  symbol: string;
  submitted_at: string;
  reviewer_id: string;
  driver_id: string;
  observation_id: string;
  verdict: "accepted" | "rejected";
  effect: "unclassified" | "supports" | "falsifies" | "mixed" | "context_only";
  explanation: string;
  verbatim_judgment?: string;
  applicability_boundary?: string;
  falsifier?: string;
  speaker_confirmation?: "unconfirmed" | "source_checked_not_speaker_confirmed" | "old_wang_confirmed" | "old_wang_confirmed_after_source_check" | "evidence_mismatch" | "insufficient_source_context";
  source_verification?: "unchecked" | "verified_against_source" | "evidence_mismatch" | "insufficient_source_context";
  source_verification_note?: string;
  old_wang_confirmation_attested?: boolean;
  old_wang_reviewer_identity_verified?: boolean;
  thesis_review_unchanged: boolean;
};

export type InvestmentFinancialSourceClaimTrace = {
  claim_id: string;
  metric_id: string;
  metric_basis: string;
  period: string;
  numeric_value: number;
  unit: string;
  source_url: string;
  published_at: string;
};

export type InvestmentFinancialVerificationState = {
  policy_version: string;
  status: "unmeasured" | "partially_measured" | "measured" | string;
  financial_as_of?: string;
  revenue_growth_percent?: number;
  gross_margin_percent?: number;
  gross_margin_change_pp?: number;
  ebit_margin_percent?: number;
  fcf_margin_percent?: number;
  accounts_receivable_growth_percent?: number;
  accounts_payable_growth_percent?: number;
  inventory_growth_percent?: number;
  property_plant_equipment_growth_percent?: number;
  operating_cash_flow_growth_percent?: number;
  capital_expenditure_growth_percent?: number;
  free_cash_flow_growth_percent?: number;
  cash_and_equivalents?: number;
  long_term_debt?: number;
  net_cash?: number;
  current_free_cash_flow?: number;
  prior_free_cash_flow?: number;
  financial_value_unit?: string;
  forward_metric_label?: string;
  forward_metric_value?: string;
  forward_metric_growth_percent?: number;
  forward_metric_as_of?: string;
  source_claim_ids: string[];
  source_urls: string[];
  source_calculations: string[];
  source_claims?: InvestmentFinancialSourceClaimTrace[];
  quality_warnings: string[];
  missing_checks: string[];
};

export type InvestmentFinancialEvidenceReviewConfirmations = {
  official_filings_opened: boolean;
  identity_periods_and_units_verified: boolean;
  calculations_recomputed: boolean;
  corporate_actions_and_restatements_checked: boolean;
  quality_warnings_resolved: boolean;
  no_unresolved_material_issue: boolean;
};

export type InvestmentFinancialEvidenceReviewVerdict =
  | "approved_for_rating"
  | "changes_requested"
  | "rejected";

export type InvestmentFinancialEvidenceReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  previous_review_id?: string;
  symbol: string;
  submitted_at: string;
  reviewer_id: string;
  evidence_fingerprint_sha256: string;
  evidence_snapshot: InvestmentFinancialVerificationState;
  verdict: InvestmentFinancialEvidenceReviewVerdict;
  rationale: string;
  confirmations: InvestmentFinancialEvidenceReviewConfirmations;
  rating_factor_authorized: boolean;
  valuation_authorized: false;
  training_authorized: false;
  reward_authorized: false;
  portfolio_action_authorized: false;
  shadow_portfolio_authorized: false;
  trade_authorized: false;
  old_wang_logic_confirmed: false;
};

export type InvestmentFinancialEvidenceReviewCandidate = {
  symbol: string;
  evidence_fingerprint_sha256: string;
  evidence: InvestmentFinancialVerificationState;
  review_status: string;
  score_eligible: boolean;
  blocking_reasons: string[];
  latest_review?: InvestmentFinancialEvidenceReviewRecord;
  review_priority_rank: number;
  review_priority_reasons: string[];
};

export type InvestmentFinancialEvidenceReviewResponse = {
  schema_version: string;
  policy_version: string;
  generated_at: string;
  summary: {
    observed: number;
    pending: number;
    approved_for_rating: number;
    changes_requested: number;
    rejected: number;
    stale_after_evidence_change: number;
  };
  candidates: InvestmentFinancialEvidenceReviewCandidate[];
  selection_mode: "active_batch" | "full_queue";
  selection_policy_version: string;
  selection_scope: string;
  eligible_queue: number;
  returned: number;
  scope: string;
  training_authorized: false;
  reward_authorized: false;
  portfolio_action_authorized: false;
  shadow_portfolio_authorized: false;
  trade_authorized: false;
};

export type InvestmentFinancialEvidenceReviewRequest = {
  expected_review_id?: string;
  expected_evidence_fingerprint_sha256: string;
  verdict: InvestmentFinancialEvidenceReviewVerdict;
  rationale: string;
  confirmations: InvestmentFinancialEvidenceReviewConfirmations;
};

export type InvestmentValuationInputReviewConfirmations = {
  official_sources_opened: boolean;
  sec_financial_values_recomputed: boolean;
  diluted_share_count_and_corporate_actions_verified: boolean;
  complete_net_cash_or_debt_verified: boolean;
  forward_or_midcycle_inputs_verified: boolean;
  cyclicality_and_normalization_checked: boolean;
  cross_method_comparability_checked: boolean;
  no_unresolved_material_issue: boolean;
};

export type InvestmentSupplementalValuationInputs = {
  input_as_of: string;
  currency: "USD" | string;
  diluted_shares_millions?: number;
  complete_net_cash_millions?: number;
  forward_eps?: number;
  forward_revenue_millions?: number;
  normalized_ebit_margin_percent?: number;
  annual_fcf_history_millions: number[];
  source_urls: string[];
  source_note: string;
};

export type InvestmentValuationInputReviewVerdict =
  | "approved_for_valuation"
  | "changes_requested"
  | "rejected";

export type InvestmentValuationInputReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  previous_review_id?: string;
  symbol: string;
  submitted_at: string;
  reviewer_id: string;
  financial_evidence_fingerprint_sha256: string;
  financial_evidence_snapshot: InvestmentFinancialVerificationState;
  input_fingerprint_sha256: string;
  supplemental_inputs: InvestmentSupplementalValuationInputs;
  verdict: InvestmentValuationInputReviewVerdict;
  rationale: string;
  confirmations: InvestmentValuationInputReviewConfirmations;
  valuation_authorized: boolean;
  rating_factor_authorized: false;
  training_authorized: false;
  reward_authorized: false;
  portfolio_action_authorized: false;
  shadow_portfolio_authorized: false;
  trade_authorized: false;
  old_wang_logic_confirmed: false;
};

export type InvestmentValuationInputReviewCandidate = {
  symbol: string;
  financial_evidence_fingerprint_sha256: string;
  financial_evidence: InvestmentFinancialVerificationState;
  review_status: string;
  valuation_authorized: boolean;
  blocking_reasons: string[];
  prepared_method_ids: string[];
  latest_review?: InvestmentValuationInputReviewRecord;
};

export type InvestmentValuationInputReviewResponse = {
  schema_version: string;
  policy_version: string;
  generated_at: string;
  observed: number;
  pending: number;
  authorized: number;
  stale: number;
  candidates: InvestmentValuationInputReviewCandidate[];
  scope: string;
  rating_factor_authorized: false;
  training_authorized: false;
  reward_authorized: false;
  portfolio_action_authorized: false;
  shadow_portfolio_authorized: false;
  trade_authorized: false;
};

export type InvestmentValuationInputReviewRequest = {
  expected_review_id?: string;
  expected_financial_evidence_fingerprint_sha256: string;
  verdict: InvestmentValuationInputReviewVerdict;
  rationale: string;
  confirmations: InvestmentValuationInputReviewConfirmations;
  supplemental_inputs: InvestmentSupplementalValuationInputs;
};

export type InvestmentModelDriver = {
  driver_id: string;
  label: string;
  mechanism: string;
  required_observations: string[];
  measurement_status: "unmeasured" | "partially_measured" | "measured";
  observations: InvestmentCausalObservation[];
  promotion: InvestmentCausalPromotion;
};

export type InvestmentOperatingKpiComparability =
  | "standardized_metric"
  | "within_issuer_only"
  | "contract_milestone";

export type InvestmentOperatingKpiDefinition = {
  kpi_id: string;
  label: string;
  driver_id: string;
  definition: string;
  unit: string;
  period_policy: string;
  source_priority: string[];
  comparability_policy: InvestmentOperatingKpiComparability;
  issuer_definition_required: boolean;
  cross_company_comparable: boolean;
  acceptance_requirements: string[];
  forbidden_inference: string;
};

export type InvestmentOperatingKpiRegistry = {
  version: string;
  model_id: string;
  entries: InvestmentOperatingKpiDefinition[];
};

export type InvestmentHumanReview = {
  status: InvestmentReviewStatus;
  review_id?: string;
  reviewed_at?: string;
  reviewer_id?: string;
  correction_note?: string;
  thesis_verdict: InvestmentThesisVerdict;
  corrected_zone?: InvestmentResearchZone;
  corrected_action?: InvestmentExposureAction;
  error_attributions: InvestmentDecisionErrorAttribution[];
  causal_link_reviews?: InvestmentCausalLinkReview[];
  causal_source_reviews?: InvestmentCausalSourceReview[];
};

export type InvestmentForwardOutcome = {
  horizon_market_sessions: 20 | 60 | 250 | number;
  status: "pending_market_sessions" | "observed" | "invalid";
  period_end?: string;
  asset_return_percent?: number;
  excess_return_percent?: number;
  max_drawdown_percent?: number;
};

export type InvestmentDecisionTrainingSample = {
  schema_version: string;
  sample_id: string;
  observed_at: string;
  selected_action: InvestmentExposureAction;
  human_review: InvestmentHumanReview;
  outcomes: InvestmentForwardOutcome[];
  reward: { status: "unconfigured" | "computed"; value?: number };
  state: {
    symbol: string;
    company_name: string;
    theme: string;
    source_rating_score: number;
    source_rating_light: string;
    data_status: string;
    evidence_coverage: number;
    first_principles?: {
      model_id: string;
      version: string;
      status: string;
      demand_equation: string;
      effective_supply_equation: string;
      scarcity_equation: string;
      value_capture_equation: string;
      demand_drivers: InvestmentModelDriver[];
      supply_drivers: InvestmentModelDriver[];
      value_capture_drivers: InvestmentModelDriver[];
      operating_kpi_registry: InvestmentOperatingKpiRegistry;
      confirmation_signals: string[];
      invalidation_conditions: string[];
    };
    market_regime?: {
      taxonomy_version: string;
      status: "unavailable" | "observed";
      label: "supportive" | "balanced" | "defensive" | "stress" | string;
      macro_score?: number;
      macro_signal: string;
      macro_phase: string;
      report_generated_at?: string;
      data_cutoff?: string;
      source_model_version: string;
      source_urls: string[];
      missing_reason: string;
    };
    decision_completeness?: InvestmentDecisionCompleteness;
    crowding?: InvestmentCrowdingState;
    decision: {
      zone: InvestmentResearchZone;
      action: InvestmentExposureAction;
      confidence: string;
      causal_confidence?: {
        policy_version: string;
        base_confidence: string;
        effective_confidence: string;
        promoted_driver_count: number;
        blocked_conflict_count: number;
        blocked_human_rejection_count: number;
        blocked_falsification_count: number;
        adjustment: "unchanged" | "upgraded" | "downgraded" | string;
        reasons: string[];
      };
      rationale: string[];
      falsifiers: string[];
      next_checks: string[];
      methodology?: {
        policy_version: string;
        skill_id: string;
        skill_version: string;
        confirmed_logic_ids: string[];
        candidate_logic_used: boolean;
        pre_methodology_zone: string;
        pre_methodology_action: string;
        increase_candidate_authorized: boolean;
        portfolio_action_authorized: boolean;
        rules: Array<{
          logic_id: string;
          logic_version: string;
          label: string;
          status: "passed" | "blocked" | "delegated_to_portfolio" | string;
          decision_effect: "increase_gate" | "portfolio_gate" | string;
          evidence: string[];
          gaps: string[];
        }>;
        blocking_reasons: string[];
        scope: string;
      };
    };
    evidence: Array<{ evidence_id: string; label: string; value: string; as_of: string; source: string }>;
  };
};

export type InvestmentDecisionReplay = {
  schema_version: string;
  symbol: string;
  sample_count: number;
  quarantined_sample_count: number;
  quarantine_warnings: Array<{
    file_name: string;
    reason: string;
  }>;
  samples: InvestmentDecisionTrainingSample[];
};

export type InvestmentDecisionReviewRequest = {
  expected_review_id?: string;
  status: Exclude<InvestmentReviewStatus, "pending">;
  thesis_verdict: Exclude<InvestmentThesisVerdict, "pending">;
  correction_note?: string;
  corrected_zone?: InvestmentResearchZone;
  corrected_action?: InvestmentExposureAction;
  error_attributions: InvestmentDecisionErrorAttribution[];
  causal_link_reviews?: InvestmentCausalLinkReview[];
};

export type InvestmentDecisionReviewRecord = {
  schema_version: string;
  review_id: string;
  sample_id: string;
  symbol: string;
  submitted_at: string;
  previous_review_id?: string;
  review: InvestmentHumanReview;
};

export type InvestmentDecisionEvaluation = {
  schema_version: string;
  generated_at: string;
  symbol_filter?: string;
  reward_status: "unconfigured" | "computed";
  sample_count: number;
  hari_logic_scenario_benchmark: {
    policy_version: string;
    skill_id: string;
    skill_version: string;
    confirmed_logic_ids: string[];
    scenario_count: number;
    passed_scenario_count: number;
    failed_scenario_count: number;
    all_passed: boolean;
    cases: Array<{
      case_id: string;
      label: string;
      covered_logic_ids: string[];
      expected_company_increase_authorized: boolean;
      actual_company_increase_authorized: boolean;
      expected_blocking_logic_ids: string[];
      actual_blocking_logic_ids: string[];
      expected_portfolio_delegated: boolean;
      actual_portfolio_delegated: boolean;
      passed: boolean;
      failure_reasons: string[];
    }>;
    synthetic_scenarios_only: boolean;
    training_label_created: boolean;
    decision_authorized: boolean;
    portfolio_action_authorized: boolean;
    shadow_portfolio_authorized: boolean;
    trading_authorized: boolean;
    scope: string;
  };
  empirical_validation_readiness?: {
    policy_version: string;
    status: string;
    causal_dataset_status: string;
    causal_dataset_governance_review_ready: boolean;
    causal_eligible_example_count: number;
    causal_distinct_symbols: number;
    causal_distinct_drivers: number;
    confirmed_historical_anchor_count: number;
    reconstruction_candidate_count: number;
    benchmark_state_ready_count: number;
    stale_reconstruction_count: number;
    state_reconstruction_status: string;
    historical_outcome_protocol_version: string;
    historical_outcome_protocol_sha256: string;
    historical_outcome_protocol_review_status: string;
    historical_outcome_implementation_review_ready: boolean;
    historical_outcome_labeler_implementation_count: number;
    historical_outcome_labeler_current_binding_count: number;
    historical_outcome_labeler_reviewed_count: number;
    historical_outcome_labeler_review_status: string;
    historical_outcome_offline_dry_run_authorization_review_eligible: boolean;
    historical_outcome_price_snapshot_count: number;
    historical_outcome_fully_covered_snapshot_count: number;
    historical_outcome_dry_run_registration_eligible_count: number;
    historical_outcome_dry_run_authorization_status: string;
    historical_outcome_dry_run_implementation_count: number;
    historical_outcome_dry_run_current_binding_count: number;
    historical_outcome_dry_run_implementation_status: string;
    historical_outcome_dry_run_execution_authorization_review_eligible_count: number;
    historical_outcome_dry_run_execution_authorization_reviewed_count: number;
    historical_outcome_dry_run_runner_registration_eligible_count: number;
    historical_outcome_dry_run_execution_authorization_status: string;
    historical_outcome_dry_run_isolated_runner_count: number;
    historical_outcome_dry_run_isolated_runner_current_binding_count: number;
    historical_outcome_dry_run_first_execution_authorization_review_eligible_count: number;
    historical_outcome_dry_run_isolated_runner_status: string;
    historical_outcome_dry_run_first_execution_authorization_reviewed_count: number;
    historical_outcome_dry_run_one_shot_first_execution_authorized_count: number;
    historical_outcome_dry_run_unexpired_first_execution_authorization_count: number;
    historical_outcome_dry_run_first_execution_authorization_status: string;
    historical_outcome_dry_run_execution_attempt_count: number;
    historical_outcome_dry_run_completed_attempt_count: number;
    historical_outcome_dry_run_failed_attempt_count: number;
    historical_outcome_dry_run_untrusted_output_count: number;
    historical_outcome_dry_run_execution_attempt_status: string;
    historical_outcome_dry_run_output_validation_eligible_count: number;
    historical_outcome_dry_run_output_validation_count: number;
    historical_outcome_dry_run_validated_output_count: number;
    historical_outcome_dry_run_failed_output_validation_count: number;
    historical_outcome_dry_run_output_validation_status: string;
    historical_outcome_label_admission_reviewed_output_count: number;
    historical_outcome_label_admitted_output_count: number;
    historical_outcome_label_admission_rejected_or_changes_requested_count: number;
    historical_outcome_label_admission_status: string;
    historical_outcome_label_materialization_implementation_count: number;
    historical_outcome_label_materialization_current_binding_count: number;
    historical_outcome_label_materialization_run_authorization_review_eligible_count: number;
    historical_outcome_label_materialization_implementation_status: string;
    historical_outcome_label_materialization_run_authorization_reviewed_count: number;
    historical_outcome_label_materialization_runner_registration_eligible_count: number;
    historical_outcome_label_materialization_run_authorization_status: string;
    historical_outcome_label_materialization_isolated_runner_count: number;
    historical_outcome_label_materialization_isolated_runner_current_binding_count: number;
    historical_outcome_label_materialization_first_execution_authorization_review_eligible_count: number;
    historical_outcome_label_materialization_isolated_runner_status: string;
    historical_outcome_label_materialization_first_execution_authorization_reviewed_count: number;
    historical_outcome_label_materialization_one_shot_first_execution_authorized_count: number;
    historical_outcome_label_materialization_unexpired_first_execution_authorization_count: number;
    historical_outcome_label_materialization_first_execution_authorization_status: string;
    historical_outcome_label_materialization_execution_attempt_count: number;
    historical_outcome_label_materialization_completed_attempt_count: number;
    historical_outcome_label_materialization_failed_attempt_count: number;
    historical_outcome_label_materialization_untrusted_envelope_count: number;
    historical_outcome_label_materialization_independent_validation_eligible_count: number;
    historical_outcome_label_materialization_execution_attempt_status: string;
    historical_outcome_label_materialization_output_validation_eligible_count: number;
    historical_outcome_label_materialization_output_validation_count: number;
    historical_outcome_label_materialization_validated_envelope_count: number;
    historical_outcome_label_materialization_failed_output_validation_count: number;
    historical_outcome_label_materialization_output_validation_status: string;
    historical_outcome_label_write_authorization_review_eligible_count: number;
    historical_outcome_label_write_authorization_reviewed_count: number;
    historical_outcome_label_write_authorization_one_shot_authorized_count: number;
    historical_outcome_label_write_authorization_unexpired_count: number;
    historical_outcome_label_write_authorization_status: string;
    historical_outcome_formal_label_write_eligible_authorization_count: number;
    historical_outcome_formal_label_write_claim_count: number;
    historical_outcome_formal_label_written_count: number;
    historical_outcome_formal_label_failed_write_count: number;
    historical_outcome_formal_label_incomplete_fail_closed_claim_count: number;
    historical_outcome_formal_label_write_status: string;
    historical_outcome_formal_label_validation_eligible_count: number;
    historical_outcome_formal_label_validation_count: number;
    historical_outcome_formal_label_admitted_training_candidate_count: number;
    historical_outcome_formal_label_failed_validation_count: number;
    historical_outcome_formal_label_validation_status: string;
    historical_outcome_offline_dataset_assembly_eligible_count: number;
    historical_outcome_offline_dataset_count: number;
    historical_outcome_offline_dataset_current_binding_count: number;
    historical_outcome_offline_dataset_latest_entry_count: number;
    historical_outcome_offline_dataset_assembly_status: string;
    historical_outcome_offline_dataset_governance_review_eligible_count: number;
    historical_outcome_offline_dataset_governance_reviewed_count: number;
    historical_outcome_offline_dataset_governance_approved_count: number;
    historical_outcome_offline_dataset_governance_current_binding_approved_count: number;
    historical_outcome_offline_dataset_governance_status: string;
    historical_outcome_offline_dataset_transformation_spec_registration_eligible_count: number;
    historical_outcome_offline_dataset_transformation_spec_registered_count: number;
    historical_outcome_offline_dataset_transformation_spec_current_binding_registered_count: number;
    historical_outcome_offline_dataset_transformation_spec_independent_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_spec_status: string;
    historical_outcome_offline_dataset_transformation_spec_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_spec_reviewed_count: number;
    historical_outcome_offline_dataset_transformation_spec_approved_count: number;
    historical_outcome_offline_dataset_transformation_spec_current_binding_approved_count: number;
    historical_outcome_offline_dataset_transformation_implementation_registration_eligible_count: number;
    historical_outcome_offline_dataset_transformation_spec_review_status: string;
    historical_outcome_offline_dataset_transformation_implementation_count: number;
    historical_outcome_offline_dataset_transformation_implementation_current_binding_count: number;
    historical_outcome_offline_dataset_transformation_implementation_independent_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_implementation_status: string;
    historical_outcome_offline_dataset_transformation_implementation_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_implementation_reviewed_count: number;
    historical_outcome_offline_dataset_transformation_implementation_approved_count: number;
    historical_outcome_offline_dataset_transformation_implementation_current_binding_approved_count: number;
    historical_outcome_offline_dataset_transformation_runner_registration_eligible_count: number;
    historical_outcome_offline_dataset_transformation_implementation_review_status: string;
    historical_outcome_offline_dataset_transformation_runner_count: number;
    historical_outcome_offline_dataset_transformation_runner_current_binding_count: number;
    historical_outcome_offline_dataset_transformation_runner_first_execution_authorization_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_runner_status: string;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_review_eligible_count: number;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_reviewed_count: number;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_approved_count: number;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_unexpired_count: number;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_one_shot_authorized_count: number;
    historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count: number;
    historical_outcome_offline_dataset_transformation_first_execution_authorization_status: string;
    historical_outcome_offline_dataset_transformation_execution_attempt_count: number;
    historical_outcome_offline_dataset_transformation_completed_attempt_count: number;
    historical_outcome_offline_dataset_transformation_failed_attempt_count: number;
    historical_outcome_offline_dataset_transformation_untrusted_candidate_envelope_count: number;
    historical_outcome_offline_dataset_transformation_independent_validation_eligible_count: number;
    historical_outcome_offline_dataset_transformation_execution_attempt_status: string;
    historical_outcome_offline_dataset_transformation_output_validation_eligible_count: number;
    historical_outcome_offline_dataset_transformation_output_validation_count: number;
    historical_outcome_offline_dataset_transformation_validated_candidate_envelope_count: number;
    historical_outcome_offline_dataset_transformation_failed_output_validation_count: number;
    historical_outcome_offline_dataset_transformation_output_validation_status: string;
    historical_outcome_offline_dataset_transformation_candidate_admission_reviewed_count: number;
    historical_outcome_offline_dataset_transformation_candidate_admitted_count: number;
    historical_outcome_offline_dataset_transformation_candidate_admission_rejected_or_changes_requested_count: number;
    historical_outcome_offline_dataset_transformation_candidate_admission_status: string;
    historical_outcome_offline_dataset_transformation_official_artifact_materialization_claimed_count: number;
    historical_outcome_offline_dataset_transformation_official_artifact_materialization_completed_count: number;
    historical_outcome_offline_dataset_transformation_official_artifact_materialization_failed_or_incomplete_count: number;
    historical_outcome_offline_dataset_transformation_unvalidated_official_artifact_pair_count: number;
    historical_outcome_offline_dataset_transformation_official_artifact_materialization_status: string;
    historical_outcome_offline_dataset_transformation_official_artifact_output_validation_eligible_count: number;
    historical_outcome_offline_dataset_transformation_official_artifact_output_validation_count: number;
    historical_outcome_offline_dataset_transformation_independently_validated_official_artifact_pair_count: number;
    historical_outcome_offline_dataset_transformation_failed_official_artifact_output_validation_count: number;
    historical_outcome_offline_dataset_transformation_official_artifact_output_validation_status: string;
    historical_outcome_feature_label_join_target_spec_registration_eligible_count: number;
    historical_outcome_feature_label_join_target_specification_count: number;
    historical_outcome_feature_label_join_target_current_binding_specification_count: number;
    historical_outcome_feature_label_join_target_stale_or_mismatched_specification_count: number;
    historical_outcome_feature_label_join_target_independent_review_eligible_count: number;
    historical_outcome_feature_label_join_target_specification_status: string;
    historical_outcome_feature_label_join_target_spec_review_eligible_count: number;
    historical_outcome_feature_label_join_target_spec_reviewed_count: number;
    historical_outcome_feature_label_join_target_spec_approved_count: number;
    historical_outcome_feature_label_join_target_spec_current_binding_approved_count: number;
    historical_outcome_feature_label_join_target_implementation_registration_eligible_count: number;
    historical_outcome_feature_label_join_target_spec_review_status: string;
    historical_outcome_feature_label_join_target_implementation_count: number;
    historical_outcome_feature_label_join_target_implementation_current_binding_count: number;
    historical_outcome_feature_label_join_target_implementation_independent_review_eligible_count: number;
    historical_outcome_feature_label_join_target_implementation_status: string;
    historical_outcome_feature_label_join_target_implementation_review_eligible_count: number;
    historical_outcome_feature_label_join_target_implementation_reviewed_count: number;
    historical_outcome_feature_label_join_target_implementation_approved_count: number;
    historical_outcome_feature_label_join_target_implementation_current_binding_approved_count: number;
    historical_outcome_feature_label_join_target_runner_registration_eligible_count: number;
    historical_outcome_feature_label_join_target_implementation_review_status: string;
    historical_outcome_feature_label_join_target_isolated_runner_count: number;
    historical_outcome_feature_label_join_target_isolated_runner_current_binding_count: number;
    historical_outcome_feature_label_join_target_first_execution_authorization_review_eligible_count: number;
    historical_outcome_feature_label_join_target_isolated_runner_status: string;
    historical_outcome_feature_label_join_target_first_execution_authorization_reviewed_count: number;
    historical_outcome_feature_label_join_target_first_execution_authorization_approved_count: number;
    historical_outcome_feature_label_join_target_unexpired_first_execution_authorization_count: number;
    historical_outcome_feature_label_join_target_one_shot_first_execution_authorized_count: number;
    historical_outcome_feature_label_join_target_execution_attempt_eligible_count: number;
    historical_outcome_feature_label_join_target_first_execution_authorization_status: string;
    historical_outcome_feature_label_join_target_execution_invocation_eligible_authorization_count: number;
    historical_outcome_feature_label_join_target_execution_attempt_count: number;
    historical_outcome_feature_label_join_target_completed_execution_attempt_count: number;
    historical_outcome_feature_label_join_target_failed_execution_attempt_count: number;
    historical_outcome_feature_label_join_target_untrusted_candidate_envelope_count: number;
    historical_outcome_feature_label_join_target_independent_output_validation_eligible_count: number;
    historical_outcome_feature_label_join_target_execution_status: string;
    historical_outcome_feature_label_join_target_output_validation_eligible_count: number;
    historical_outcome_feature_label_join_target_output_validation_count: number;
    historical_outcome_feature_label_join_target_independently_validated_untrusted_candidate_count: number;
    historical_outcome_feature_label_join_target_failed_output_validation_count: number;
    historical_outcome_feature_label_join_target_candidate_admission_review_eligible_count: number;
    historical_outcome_feature_label_join_target_output_validation_status: string;
    historical_outcome_feature_label_join_target_candidate_admission_reviewed_count: number;
    historical_outcome_feature_label_join_target_candidate_admitted_count: number;
    historical_outcome_feature_label_join_target_candidate_admission_rejected_or_changes_requested_count: number;
    historical_outcome_feature_label_join_target_future_official_joined_dataset_materialization_eligible_count: number;
    historical_outcome_feature_label_join_target_candidate_admission_status: string;
    historical_outcome_feature_label_join_target_official_dataset_materialization_eligible_count: number;
    historical_outcome_feature_label_join_target_official_dataset_admitted_candidate_count: number;
    historical_outcome_feature_label_join_target_official_dataset_materialization_claim_count: number;
    historical_outcome_feature_label_join_target_official_dataset_materialization_completed_count: number;
    historical_outcome_feature_label_join_target_official_dataset_materialization_failed_count: number;
    historical_outcome_feature_label_join_target_official_dataset_pending_independent_validation_count: number;
    historical_outcome_feature_label_join_target_official_dataset_materialization_status: string;
    historical_outcome_feature_label_join_target_official_dataset_output_validation_eligible_count: number;
    historical_outcome_feature_label_join_target_official_dataset_output_validation_count: number;
    historical_outcome_feature_label_join_target_independently_validated_official_joined_dataset_count: number;
    historical_outcome_feature_label_join_target_official_dataset_failed_output_validation_count: number;
    historical_outcome_feature_label_join_target_future_training_store_copy_admission_review_eligible_count: number;
    historical_outcome_feature_label_join_target_official_dataset_output_validation_status: string;
    historical_outcome_feature_label_join_target_training_store_copy_admission_reviewed_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_candidate_admitted_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_admission_rejected_or_changes_requested_count: number;
    historical_outcome_feature_label_join_target_future_create_once_training_store_copy_eligible_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_admission_status: string;
    historical_outcome_feature_label_join_target_training_store_copy_admitted_dataset_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_eligible_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_claim_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_completed_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_failed_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_pending_independent_validation_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_status: string;
    historical_outcome_feature_label_join_target_training_store_copy_output_validation_eligible_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_output_validation_count: number;
    historical_outcome_feature_label_join_target_independently_validated_training_store_copy_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_failed_output_validation_count: number;
    historical_outcome_feature_label_join_target_future_training_registration_review_eligible_count: number;
    historical_outcome_feature_label_join_target_training_store_copy_output_validation_status: string;
    historical_outcome_feature_label_join_target_training_registration_admission_reviewed_count: number;
    historical_outcome_feature_label_join_target_training_registration_candidate_admitted_count: number;
    historical_outcome_feature_label_join_target_training_registration_admission_rejected_or_changes_requested_count: number;
    historical_outcome_feature_label_join_target_future_create_once_training_registration_eligible_count: number;
    historical_outcome_feature_label_join_target_training_registration_admission_status: string;
    historical_outcome_training_experiment_registration_admitted_candidate_count: number;
    historical_outcome_training_experiment_registration_claim_count: number;
    historical_outcome_training_experiment_registered_not_run_count: number;
    historical_outcome_training_experiment_registration_failed_or_incomplete_count: number;
    historical_outcome_training_experiment_pending_independent_review_count: number;
    historical_outcome_training_experiment_registration_status: string;
    historical_outcome_training_experiment_registration_review_eligible_count: number;
    historical_outcome_training_experiment_registration_reviewed_count: number;
    historical_outcome_training_experiment_registration_independently_approved_count: number;
    historical_outcome_training_experiment_registration_rejected_or_changes_requested_count: number;
    historical_outcome_future_training_implementation_registration_eligible_count: number;
    historical_outcome_training_experiment_registration_review_status: string;
    historical_outcome_training_implementation_registration_eligible_count: number;
    historical_outcome_training_implementation_count: number;
    historical_outcome_training_implementation_current_binding_count: number;
    historical_outcome_training_implementation_pending_independent_review_count: number;
    historical_outcome_training_implementation_status: string;
    historical_outcome_training_implementation_review_eligible_count: number;
    historical_outcome_training_implementation_reviewed_count: number;
    historical_outcome_training_implementation_independently_approved_count: number;
    historical_outcome_training_implementation_review_rejected_or_changes_requested_count: number;
    historical_outcome_future_isolated_training_runner_registration_eligible_count: number;
    historical_outcome_training_implementation_review_status: string;
    historical_outcome_training_isolated_runner_count: number;
    historical_outcome_training_isolated_runner_current_binding_count: number;
    historical_outcome_training_first_execution_authorization_review_eligible_count: number;
    historical_outcome_training_isolated_runner_status: string;
    historical_outcome_training_first_execution_authorization_reviewed_count: number;
    historical_outcome_training_first_execution_authorization_approved_count: number;
    historical_outcome_training_unexpired_first_execution_authorization_count: number;
    historical_outcome_training_one_shot_first_execution_authorized_count: number;
    historical_outcome_training_execution_attempt_eligible_count: number;
    historical_outcome_training_first_execution_authorization_status: string;
    historical_outcome_training_execution_claim_count: number;
    historical_outcome_training_completed_execution_attempt_count: number;
    historical_outcome_training_failed_execution_attempt_count: number;
    historical_outcome_training_untrusted_artifact_envelope_count: number;
    historical_outcome_training_independent_output_validation_eligible_count: number;
    historical_outcome_training_execution_attempt_status: string;
    historical_outcome_training_output_validation_eligible_count: number;
    historical_outcome_training_output_validation_count: number;
    historical_outcome_training_independently_validated_train_only_artifact_envelope_count: number;
    historical_outcome_training_failed_output_validation_count: number;
    historical_outcome_future_validation_evaluation_implementation_registration_eligible_count: number;
    historical_outcome_training_output_validation_status: string;
    historical_outcome_validation_evaluation_implementation_registration_eligible_count: number;
    historical_outcome_validation_evaluation_implementation_count: number;
    historical_outcome_validation_evaluation_implementation_current_binding_count: number;
    historical_outcome_validation_evaluation_implementation_independent_review_eligible_count: number;
    historical_outcome_validation_evaluation_implementation_status: string;
    historical_outcome_validation_evaluation_implementation_review_eligible_count: number;
    historical_outcome_validation_evaluation_implementation_reviewed_count: number;
    historical_outcome_validation_evaluation_implementation_independently_approved_count: number;
    historical_outcome_validation_evaluation_implementation_review_rejected_or_changes_requested_count: number;
    historical_outcome_future_isolated_validation_evaluation_runner_registration_eligible_count: number;
    historical_outcome_validation_evaluation_implementation_review_status: string;
    historical_outcome_validation_evaluation_isolated_runner_count: number;
    historical_outcome_validation_evaluation_isolated_runner_current_binding_count: number;
    historical_outcome_validation_evaluation_first_execution_authorization_review_eligible_count: number;
    historical_outcome_validation_evaluation_isolated_runner_status: string;
    historical_outcome_validation_evaluation_first_execution_authorization_reviewed_count: number;
    historical_outcome_validation_evaluation_first_execution_authorization_approved_count: number;
    historical_outcome_validation_evaluation_first_execution_authorization_unexpired_count: number;
    historical_outcome_validation_evaluation_first_execution_authorization_one_shot_count: number;
    historical_outcome_validation_evaluation_execution_attempt_eligible_count: number;
    historical_outcome_validation_evaluation_first_execution_authorization_status: string;
    historical_outcome_validation_evaluation_execution_claim_count: number;
    historical_outcome_validation_evaluation_completed_attempt_count: number;
    historical_outcome_validation_evaluation_failed_attempt_count: number;
    historical_outcome_validation_evaluation_untrusted_envelope_count: number;
    historical_outcome_validation_evaluation_independent_output_validation_eligible_count: number;
    historical_outcome_validation_evaluation_execution_attempt_status: string;
    historical_outcome_validation_evaluation_output_validation_eligible_count: number;
    historical_outcome_validation_evaluation_output_validation_count: number;
    historical_outcome_validation_evaluation_independently_validated_untrusted_envelope_count: number;
    historical_outcome_validation_evaluation_failed_output_validation_count: number;
    historical_outcome_future_per_target_candidate_admission_review_eligible_count: number;
    historical_outcome_validation_evaluation_output_validation_status: string;
    historical_outcome_validation_evaluation_per_target_candidate_count: number;
    historical_outcome_validation_evaluation_per_target_candidate_reviewed_count: number;
    historical_outcome_validation_evaluation_per_target_candidate_admitted_count: number;
    historical_outcome_validation_evaluation_per_target_candidate_rejected_or_changes_requested_count: number;
    historical_outcome_validation_evaluation_per_target_insufficient_evidence_count: number;
    historical_outcome_validation_evaluation_per_target_no_candidate_passed_count: number;
    historical_outcome_future_sealed_holdout_evaluation_protocol_review_eligible_target_count: number;
    historical_outcome_validation_evaluation_per_target_candidate_admission_status: string;
    historical_outcome_sealed_holdout_evaluation_protocol_admitted_target_count: number;
    historical_outcome_sealed_holdout_evaluation_protocol_reviewed_count: number;
    historical_outcome_sealed_holdout_evaluation_protocol_independently_approved_count: number;
    historical_outcome_sealed_holdout_evaluation_protocol_rejected_or_changes_requested_count: number;
    historical_outcome_future_sealed_holdout_evaluation_implementation_registration_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_protocol_review_status: string;
    historical_outcome_sealed_holdout_evaluation_implementation_registration_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_current_binding_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_independent_review_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_status: string;
    historical_outcome_sealed_holdout_evaluation_implementation_review_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_reviewed_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_independently_approved_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_rejected_or_changes_requested_count: number;
    historical_outcome_future_isolated_sealed_holdout_evaluation_runner_registration_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_implementation_review_status: string;
    historical_outcome_sealed_holdout_evaluation_isolated_runner_registration_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_isolated_runner_count: number;
    historical_outcome_sealed_holdout_evaluation_isolated_runner_current_binding_count: number;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_review_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_isolated_runner_status: string;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_reviewed_count: number;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_approved_count: number;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_unexpired_count: number;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_one_shot_count: number;
    historical_outcome_sealed_holdout_evaluation_execution_attempt_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_first_execution_authorization_status: string;
    historical_outcome_sealed_holdout_evaluation_execution_claim_count: number;
    historical_outcome_sealed_holdout_evaluation_completed_attempt_count: number;
    historical_outcome_sealed_holdout_evaluation_failed_attempt_count: number;
    historical_outcome_sealed_holdout_evaluation_untrusted_confirmation_envelope_count: number;
    historical_outcome_sealed_holdout_evaluation_independent_output_validation_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_execution_attempt_status: string;
    historical_outcome_sealed_holdout_evaluation_output_validation_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_output_validation_count: number;
    historical_outcome_sealed_holdout_evaluation_independently_validated_confirmation_count: number;
    historical_outcome_sealed_holdout_evaluation_failed_output_validation_count: number;
    historical_outcome_future_confirmatory_result_adjudication_review_eligible_count: number;
    historical_outcome_sealed_holdout_evaluation_output_validation_status: string;
    historical_outcome_sealed_holdout_confirmatory_result_adjudication_candidate_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_quantitative_pass_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_quantitative_fail_or_insufficient_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_adjudication_reviewed_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_adjudication_approved_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_adjudication_changes_or_rejected_count: number;
    historical_outcome_future_controlled_shadow_experiment_design_registration_eligible_count: number;
    historical_outcome_sealed_holdout_confirmatory_result_adjudication_status: string;
    historical_outcome_controlled_shadow_experiment_design_adjudicated_candidate_count: number;
    historical_outcome_controlled_shadow_experiment_design_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_design_registered_count: number;
    historical_outcome_future_independent_shadow_design_review_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_design_registration_status: string;
    historical_outcome_controlled_shadow_experiment_design_review_registered_design_count: number;
    historical_outcome_controlled_shadow_experiment_design_review_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_design_reviewed_count: number;
    historical_outcome_controlled_shadow_experiment_design_independently_approved_count: number;
    historical_outcome_controlled_shadow_experiment_design_changes_or_rejected_count: number;
    historical_outcome_future_zero_capability_shadow_implementation_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_design_review_status: string;
    historical_outcome_controlled_shadow_experiment_implementation_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_current_binding_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_independent_review_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_status: string;
    historical_outcome_controlled_shadow_experiment_implementation_review_implementation_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_review_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_reviewed_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_independently_approved_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_changes_requested_or_rejected_count: number;
    historical_outcome_future_isolated_shadow_runner_specification_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_implementation_review_status: string;
    historical_outcome_controlled_shadow_experiment_isolated_runner_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_isolated_runner_count: number;
    historical_outcome_controlled_shadow_experiment_isolated_runner_current_binding_count: number;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_review_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_isolated_runner_status: string;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_reviewed_count: number;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_approved_count: number;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_unexpired_count: number;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_one_shot_count: number;
    historical_outcome_controlled_shadow_experiment_execution_attempt_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_first_execution_authorization_status: string;
    historical_outcome_controlled_shadow_experiment_execution_claim_count: number;
    historical_outcome_controlled_shadow_experiment_execution_completed_count: number;
    historical_outcome_controlled_shadow_experiment_execution_failed_count: number;
    historical_outcome_controlled_shadow_experiment_untrusted_initial_observation_count: number;
    historical_outcome_controlled_shadow_experiment_independent_output_validation_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_execution_status: string;
    historical_outcome_controlled_shadow_experiment_output_validation_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_output_validation_count: number;
    historical_outcome_controlled_shadow_experiment_independently_validated_initial_observation_count: number;
    historical_outcome_controlled_shadow_experiment_failed_output_validation_count: number;
    historical_outcome_future_forward_observation_protocol_registration_eligible_count: number;
    historical_outcome_controlled_shadow_experiment_output_validation_status: string;
    historical_outcome_forward_observation_protocol_registration_eligible_count: number;
    historical_outcome_forward_observation_protocol_registered_count: number;
    historical_outcome_forward_observation_protocol_current_binding_count: number;
    historical_outcome_future_independent_protocol_review_eligible_count: number;
    historical_outcome_forward_observation_protocol_registration_status: string;
    historical_outcome_forward_observation_protocol_review_registered_count: number;
    historical_outcome_forward_observation_protocol_review_eligible_count: number;
    historical_outcome_forward_observation_protocol_reviewed_count: number;
    historical_outcome_forward_observation_protocol_independently_approved_count: number;
    historical_outcome_forward_observation_protocol_changes_required_or_rejected_count: number;
    historical_outcome_future_zero_capability_forward_observation_implementation_registration_eligible_count: number;
    historical_outcome_forward_observation_protocol_review_status: string;
    historical_outcome_forward_observation_implementation_registration_eligible_count: number;
    historical_outcome_forward_observation_implementation_count: number;
    historical_outcome_forward_observation_implementation_current_binding_count: number;
    historical_outcome_forward_observation_implementation_independent_review_eligible_count: number;
    historical_outcome_forward_observation_implementation_status: string;
    historical_outcome_forward_observation_implementation_review_registered_count: number;
    historical_outcome_forward_observation_implementation_review_eligible_count: number;
    historical_outcome_forward_observation_implementation_reviewed_count: number;
    historical_outcome_forward_observation_implementation_independently_approved_count: number;
    historical_outcome_forward_observation_implementation_changes_required_or_rejected_count: number;
    historical_outcome_future_isolated_forward_observation_runner_specification_registration_eligible_count: number;
    historical_outcome_forward_observation_implementation_review_status: string;
    historical_outcome_forward_observation_isolated_runner_registration_eligible_count: number;
    historical_outcome_forward_observation_isolated_runner_count: number;
    historical_outcome_forward_observation_isolated_runner_current_binding_count: number;
    historical_outcome_forward_observation_first_execution_authorization_review_eligible_count: number;
    historical_outcome_forward_observation_isolated_runner_status: string;
    historical_outcome_forward_observation_first_execution_authorization_reviewed_count: number;
    historical_outcome_forward_observation_first_execution_authorization_approved_count: number;
    historical_outcome_forward_observation_first_execution_authorization_unexpired_count: number;
    historical_outcome_forward_observation_first_execution_authorization_one_shot_count: number;
    historical_outcome_forward_observation_future_attempt_eligible_count: number;
    historical_outcome_forward_observation_first_execution_authorization_status: string;
    historical_outcome_forward_observation_execution_attempt_eligible_count: number;
    historical_outcome_forward_observation_execution_claim_count: number;
    historical_outcome_forward_observation_execution_completed_count: number;
    historical_outcome_forward_observation_execution_failed_count: number;
    historical_outcome_forward_observation_execution_interrupted_count: number;
    historical_outcome_forward_observation_execution_independent_validation_eligible_count: number;
    historical_outcome_forward_observation_execution_status: string;
    historical_outcome_forward_observation_output_validation_eligible_count: number;
    historical_outcome_forward_observation_output_validation_count: number;
    historical_outcome_forward_observation_independently_validated_initialization_receipt_count: number;
    historical_outcome_forward_observation_failed_output_validation_count: number;
    historical_outcome_future_first_natural_forward_cycle_authorization_review_eligible_count: number;
    historical_outcome_forward_observation_output_validation_status: string;
    historical_outcome_first_natural_forward_cycle_authorization_review_eligible_count: number;
    historical_outcome_first_natural_forward_cycle_authorization_reviewed_count: number;
    historical_outcome_first_natural_forward_cycle_authorization_approved_count: number;
    historical_outcome_first_natural_forward_cycle_authorization_active_count: number;
    historical_outcome_first_natural_forward_cycle_future_attempt_eligible_count: number;
    historical_outcome_first_natural_forward_cycle_authorization_status: string;
    historical_outcome_first_natural_forward_cycle_claim_authorization_candidate_count: number;
    historical_outcome_first_natural_forward_cycle_claim_eligible_count: number;
    historical_outcome_first_natural_forward_cycle_claim_count: number;
    historical_outcome_first_natural_forward_cycle_authorization_consumed_count: number;
    historical_outcome_first_natural_forward_cycle_waiting_for_market_data_adapter_authorization_count: number;
    historical_outcome_first_natural_forward_cycle_claim_status: string;
    historical_outcome_market_data_adapter_claimed_task_count: number;
    historical_outcome_market_data_adapter_authorization_review_eligible_count: number;
    historical_outcome_market_data_adapter_authorization_reviewed_count: number;
    historical_outcome_market_data_adapter_authorization_approved_count: number;
    historical_outcome_market_data_adapter_authorization_rejected_count: number;
    historical_outcome_market_data_adapter_active_authorization_count: number;
    historical_outcome_future_claim_first_read_only_market_data_receipt_eligible_count: number;
    historical_outcome_market_data_adapter_authorization_status: string;
    historical_outcome_market_data_receipt_invocation_eligible_authorization_count: number;
    historical_outcome_market_data_receipt_claim_count: number;
    historical_outcome_market_data_receipt_completed_untrusted_count: number;
    historical_outcome_market_data_receipt_failed_consumed_count: number;
    historical_outcome_market_data_receipt_interrupted_consumed_count: number;
    historical_outcome_market_data_receipt_independent_validation_eligible_count: number;
    historical_outcome_market_data_receipt_status: string;
    historical_outcome_market_data_receipt_validation_completed_untrusted_count: number;
    historical_outcome_market_data_receipt_validation_pending_count: number;
    historical_outcome_market_data_receipt_independently_validated_count: number;
    historical_outcome_market_data_receipt_validation_failed_count: number;
    historical_outcome_future_market_data_parser_review_eligible_count: number;
    historical_outcome_market_data_receipt_validation_status: string;
    historical_outcome_market_data_parser_spec_independently_validated_receipt_count: number;
    historical_outcome_market_data_parser_spec_registration_eligible_count: number;
    historical_outcome_market_data_parser_spec_registered_count: number;
    historical_outcome_future_market_data_parser_spec_review_eligible_count: number;
    historical_outcome_market_data_parser_spec_status: string;
    historical_outcome_market_data_parser_spec_review_registered_count: number;
    historical_outcome_market_data_parser_spec_review_eligible_count: number;
    historical_outcome_market_data_parser_spec_reviewed_count: number;
    historical_outcome_market_data_parser_spec_independently_approved_count: number;
    historical_outcome_market_data_parser_spec_review_changes_required_or_rejected_count: number;
    historical_outcome_future_zero_capability_market_data_parser_implementation_registration_eligible_count: number;
    historical_outcome_market_data_parser_spec_review_status: string;
    historical_outcome_market_data_parser_implementation_independently_approved_specification_count: number;
    historical_outcome_market_data_parser_implementation_registration_eligible_count: number;
    historical_outcome_market_data_parser_implementation_contract_count: number;
    historical_outcome_market_data_parser_implementation_current_binding_contract_count: number;
    historical_outcome_market_data_parser_implementation_independent_review_eligible_count: number;
    historical_outcome_market_data_parser_implementation_status: string;
    historical_outcome_market_data_parser_implementation_review_implementation_count: number;
    historical_outcome_market_data_parser_implementation_review_eligible_count: number;
    historical_outcome_market_data_parser_implementation_reviewed_count: number;
    historical_outcome_market_data_parser_implementation_independently_approved_count: number;
    historical_outcome_market_data_parser_implementation_review_changes_required_or_rejected_count: number;
    historical_outcome_future_isolated_market_data_parser_runner_specification_registration_eligible_count: number;
    historical_outcome_market_data_parser_implementation_review_status: string;
    historical_outcome_market_data_parser_isolated_runner_registration_eligible_count: number;
    historical_outcome_market_data_parser_isolated_runner_count: number;
    historical_outcome_market_data_parser_isolated_runner_current_binding_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_review_eligible_count: number;
    historical_outcome_market_data_parser_isolated_runner_status: string;
    historical_outcome_market_data_parser_first_execution_authorization_runner_count: number;
    historical_outcome_market_data_parser_reproduced_artifact_verified_runner_count: number;
    historical_outcome_market_data_parser_reproduced_artifact_pending_runner_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_review_ready_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_reviewed_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_approved_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_unexpired_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_one_shot_count: number;
    historical_outcome_market_data_parser_future_claim_first_attempt_eligible_count: number;
    historical_outcome_market_data_parser_first_execution_authorization_status: string;
    historical_outcome_market_data_parser_execution_attempt_authorization_candidate_count: number;
    historical_outcome_market_data_parser_execution_attempt_claim_eligible_count: number;
    historical_outcome_market_data_parser_execution_attempt_claim_count: number;
    historical_outcome_market_data_parser_execution_attempt_authorization_consumed_count: number;
    historical_outcome_market_data_parser_execution_attempt_waiting_for_stage_102_count: number;
    historical_outcome_market_data_parser_execution_attempt_claim_status: string;
    historical_outcome_market_data_parser_execution_pending_claim_count: number;
    historical_outcome_market_data_parser_execution_terminal_result_count: number;
    historical_outcome_market_data_parser_execution_successful_untrusted_output_count: number;
    historical_outcome_market_data_parser_execution_failed_consumed_claim_count: number;
    historical_outcome_market_data_parser_output_validation_eligible_count: number;
    historical_outcome_market_data_parser_output_validation_count: number;
    historical_outcome_market_data_parser_output_independently_validated_count: number;
    historical_outcome_market_data_parser_output_validation_failed_count: number;
    historical_outcome_market_data_parser_future_observation_input_admission_review_eligible_count: number;
    historical_outcome_market_data_parser_output_validation_status: string;
    historical_outcome_observation_input_admission_candidate_count: number;
    historical_outcome_observation_input_admission_review_eligible_count: number;
    historical_outcome_observation_input_admission_reviewed_count: number;
    historical_outcome_observation_input_admitted_count: number;
    historical_outcome_observation_input_admission_changes_requested_or_rejected_count: number;
    historical_outcome_future_observation_materialization_specification_registration_eligible_count: number;
    historical_outcome_observation_input_admission_status: string;
    historical_outcome_observation_materialization_specification_admitted_input_count: number;
    historical_outcome_observation_materialization_specification_registration_eligible_count: number;
    historical_outcome_observation_materialization_specification_registered_count: number;
    historical_outcome_observation_materialization_specification_future_independent_review_eligible_count: number;
    historical_outcome_observation_materialization_specification_status: string;
    historical_outcome_observation_materialization_specification_review_specification_count: number;
    historical_outcome_observation_materialization_specification_review_eligible_count: number;
    historical_outcome_observation_materialization_specification_reviewed_count: number;
    historical_outcome_observation_materialization_specification_independently_approved_count: number;
    historical_outcome_observation_materialization_specification_review_changes_required_or_rejected_count: number;
    historical_outcome_future_zero_capability_observation_materialization_implementation_registration_eligible_count: number;
    historical_outcome_observation_materialization_specification_review_status: string;
    historical_outcome_observation_materialization_implementation_approved_specification_count: number;
    historical_outcome_observation_materialization_implementation_registration_eligible_count: number;
    historical_outcome_observation_materialization_implementation_contract_count: number;
    historical_outcome_observation_materialization_implementation_current_binding_count: number;
    historical_outcome_observation_materialization_implementation_independent_review_eligible_count: number;
    historical_outcome_observation_materialization_implementation_status: string;
    historical_outcome_observation_materialization_implementation_review_implementation_count: number;
    historical_outcome_observation_materialization_implementation_review_eligible_count: number;
    historical_outcome_observation_materialization_implementation_reviewed_count: number;
    historical_outcome_observation_materialization_implementation_review_independently_approved_count: number;
    historical_outcome_observation_materialization_implementation_review_changes_required_or_rejected_count: number;
    historical_outcome_future_isolated_observation_materialization_runner_specification_registration_eligible_count: number;
    historical_outcome_observation_materialization_implementation_review_status: string;
    historical_outcome_observation_materialization_isolated_runner_registration_eligible_count: number;
    historical_outcome_observation_materialization_isolated_runner_count: number;
    historical_outcome_observation_materialization_isolated_runner_current_binding_count: number;
    historical_outcome_observation_materialization_isolated_runner_first_execution_authorization_review_eligible_count: number;
    historical_outcome_observation_materialization_isolated_runner_status: string;
    historical_outcome_observation_materialization_first_execution_authorization_runner_count: number;
    historical_outcome_observation_materialization_reproduced_artifact_verified_runner_count: number;
    historical_outcome_observation_materialization_reproduced_artifact_pending_runner_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_review_ready_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_reviewed_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_approved_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_unexpired_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_one_shot_count: number;
    historical_outcome_observation_materialization_future_claim_first_attempt_eligible_count: number;
    historical_outcome_observation_materialization_first_execution_authorization_status: string;
    historical_outcome_observation_materialization_execution_attempt_authorization_candidate_count: number;
    historical_outcome_observation_materialization_execution_attempt_claim_eligible_count: number;
    historical_outcome_observation_materialization_execution_attempt_claim_count: number;
    historical_outcome_observation_materialization_execution_attempt_authorization_consumed_count: number;
    historical_outcome_observation_materialization_waiting_for_stage_112_execution_count: number;
    historical_outcome_observation_materialization_execution_attempt_claim_status: string;
    historical_outcome_observation_materialization_execution_pending_claim_count: number;
    historical_outcome_observation_materialization_execution_terminal_result_count: number;
    historical_outcome_observation_materialization_execution_successful_untrusted_observation_count: number;
    historical_outcome_observation_materialization_execution_failed_consumed_claim_count: number;
    historical_outcome_observation_materialization_waiting_for_stage_113_validation_count: number;
    historical_outcome_observation_materialization_execution_status: string;
    historical_outcome_observation_materialization_output_validation_eligible_count: number;
    historical_outcome_observation_materialization_output_validation_count: number;
    historical_outcome_observation_materialization_independently_validated_observation_count: number;
    historical_outcome_observation_materialization_output_validation_failed_count: number;
    historical_outcome_observation_materialization_future_stage_114_observation_evidence_admission_review_eligible_count: number;
    historical_outcome_observation_materialization_output_validation_status: string;
    historical_outcome_observation_evidence_independently_validated_candidate_count: number;
    historical_outcome_observation_evidence_admission_review_eligible_count: number;
    historical_outcome_observation_evidence_admission_reviewed_count: number;
    historical_outcome_observation_evidence_admitted_count: number;
    historical_outcome_observation_evidence_changes_requested_or_rejected_count: number;
    historical_outcome_observation_evidence_future_stage_115_ledger_transition_specification_registration_eligible_count: number;
    historical_outcome_observation_evidence_admission_status: string;
    historical_outcome_observation_ledger_transition_specification_admitted_evidence_count: number;
    historical_outcome_observation_ledger_transition_specification_registration_eligible_count: number;
    historical_outcome_observation_ledger_transition_specification_registered_count: number;
    historical_outcome_observation_ledger_transition_specification_future_stage_116_independent_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_specification_opening_portfolio_snapshot_missing_count: number;
    historical_outcome_observation_ledger_transition_specification_status: string;
    historical_outcome_observation_ledger_transition_specification_review_specification_count: number;
    historical_outcome_observation_ledger_transition_specification_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_specification_reviewed_count: number;
    historical_outcome_observation_ledger_transition_specification_independently_approved_count: number;
    historical_outcome_observation_ledger_transition_specification_changes_required_or_rejected_count: number;
    historical_outcome_observation_ledger_transition_specification_future_stage_117_zero_capability_implementation_registration_eligible_count: number;
    historical_outcome_observation_ledger_transition_specification_review_opening_portfolio_snapshot_missing_count: number;
    historical_outcome_observation_ledger_transition_specification_review_status: string;
    historical_outcome_observation_ledger_transition_implementation_independently_approved_specification_count: number;
    historical_outcome_observation_ledger_transition_implementation_registration_eligible_count: number;
    historical_outcome_observation_ledger_transition_implementation_contract_count: number;
    historical_outcome_observation_ledger_transition_implementation_current_binding_count: number;
    historical_outcome_observation_ledger_transition_implementation_future_stage_118_independent_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_implementation_opening_portfolio_snapshot_missing_count: number;
    historical_outcome_observation_ledger_transition_implementation_status: string;
    historical_outcome_observation_ledger_transition_implementation_review_implementation_count: number;
    historical_outcome_observation_ledger_transition_implementation_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_implementation_reviewed_count: number;
    historical_outcome_observation_ledger_transition_implementation_independently_approved_count: number;
    historical_outcome_observation_ledger_transition_implementation_changes_required_or_rejected_count: number;
    historical_outcome_observation_ledger_transition_implementation_future_stage_119_isolated_runner_specification_registration_eligible_count: number;
    historical_outcome_observation_ledger_transition_implementation_review_status: string;
    historical_outcome_observation_ledger_transition_isolated_runner_registration_eligible_count: number;
    historical_outcome_observation_ledger_transition_isolated_runner_count: number;
    historical_outcome_observation_ledger_transition_isolated_runner_current_binding_count: number;
    historical_outcome_observation_ledger_transition_isolated_runner_future_stage_120_first_execution_authorization_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_isolated_runner_status: string;
    historical_outcome_observation_ledger_transition_first_execution_authorization_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_artifact_verified_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_artifact_pending_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_review_eligible_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_reviewed_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_approved_runner_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_unexpired_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_future_stage_121_claim_first_attempt_eligible_count: number;
    historical_outcome_observation_ledger_transition_first_execution_authorization_status: string;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_authorization_candidate_count: number;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_eligible_count: number;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_count: number;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_authorization_consumed_count: number;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_waiting_for_stage_122_execution_count: number;
    historical_outcome_observation_ledger_transition_execution_attempt_claim_status: string;
    historical_outcome_observation_ledger_transition_execution_pending_claim_count: number;
    historical_outcome_observation_ledger_transition_execution_terminal_result_count: number;
    historical_outcome_observation_ledger_transition_execution_successful_untrusted_candidate_count: number;
    historical_outcome_observation_ledger_transition_execution_failed_consumed_claim_count: number;
    historical_outcome_observation_ledger_transition_execution_status: string;
    historical_outcome_observation_ledger_transition_output_validation_eligible_count: number;
    historical_outcome_observation_ledger_transition_output_validation_count: number;
    historical_outcome_observation_ledger_transition_independently_validated_candidate_count: number;
    historical_outcome_observation_ledger_transition_output_validation_failed_count: number;
    historical_outcome_observation_ledger_transition_future_stage_124_admission_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_output_validation_status: string;
    historical_outcome_observation_ledger_transition_candidate_admission_review_eligible_count: number;
    historical_outcome_observation_ledger_transition_candidate_admission_reviewed_count: number;
    historical_outcome_observation_ledger_transition_admitted_non_financial_observation_evidence_count: number;
    historical_outcome_observation_ledger_transition_candidate_admission_changes_requested_or_rejected_count: number;
    historical_outcome_observation_ledger_transition_future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count: number;
    historical_outcome_observation_ledger_transition_candidate_admission_status: string;
    historical_outcome_opening_portfolio_snapshot_governance_stage_124_admitted_evidence_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_registration_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_registered_specification_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_future_stage_126_independent_specification_review_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_registration_status: string;
    historical_outcome_opening_portfolio_snapshot_governance_specification_review_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_specification_reviewed_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_specification_independently_approved_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_specification_changes_requested_or_rejected_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_governance_specification_review_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_independently_approved_specification_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_registration_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_contract_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_current_binding_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_128_independent_implementation_review_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_implementation_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_reviewed_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_independently_approved_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_changes_required_or_rejected_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_129_isolated_receiver_specification_registration_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_registration_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_current_binding_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_future_stage_130_first_execution_authorization_review_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_receiver_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_verified_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_pending_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_review_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_reviewed_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_approved_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_unexpired_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_131_claim_first_attempt_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_authorization_candidate_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_authorization_consumed_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_waiting_for_stage_132_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_pending_claim_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_terminal_result_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_successful_untrusted_receipt_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_failed_consumed_claim_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_encryption_key_configured: boolean;
    historical_outcome_opening_portfolio_source_artifact_receipt_execution_status: string;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_completed_untrusted_receipt_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_pending_independent_validation_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_independently_validated_receipt_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_failed_independent_validation_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_future_stage_134_eligible_count: number;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_encryption_key_configured: boolean;
    historical_outcome_opening_portfolio_source_artifact_receipt_validation_status: string;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_independently_validated_receipt_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_registration_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_contract_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_current_binding_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_future_stage_135_independent_review_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_status: string;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_implementation_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_reviewed_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_independently_approved_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_changes_required_or_rejected_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_future_stage_136_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_status: string;
    historical_outcome_opening_portfolio_snapshot_materialization_isolated_materializer_registration_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_isolated_materializer_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_isolated_materializer_current_binding_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_isolated_materializer_future_stage_137_first_execution_authorization_review_eligible_count: number;
    historical_outcome_opening_portfolio_snapshot_materialization_isolated_materializer_status: string;
    historical_outcome_offline_dry_run_enabled: boolean;
    outcome_label_generation_enabled: boolean;
    empirical_validation_ready: boolean;
    blocking_reasons: string[];
    training_authorized: boolean;
    reward_authorized: boolean;
    shadow_portfolio_authorized: boolean;
    trading_authorized: boolean;
    scope: string;
  };
  first_principles_hypothesis_map?: {
    policy_version: string;
    status: string;
    latest_decision_at?: string;
    map_fingerprint_sha256: string;
    model_count: number;
    company_count: number;
    source_sample_ids: string[];
    models: Array<{
      model_id: string;
      model_version: string;
      state: string;
      latest_decision_at: string;
      company_count: number;
      symbols: string[];
      demand_traceable_company_count: number;
      supply_traceable_company_count: number;
      value_capture_traceable_company_count: number;
      fully_traceable_company_count: number;
      demand_measured_company_count: number;
      supply_measured_company_count: number;
      value_capture_measured_company_count: number;
      fully_measured_company_count: number;
      evidence_pathway: {
        unique_observation_count: number;
        direct_metric_count: number;
        proxy_count: number;
        confirmed_context_count: number;
        structured_source_claim_count: number;
        computed_comparison_count: number;
        computed_ratio_count: number;
        computed_ratio_trend_count: number;
        operating_kpi_claim_count: number;
      };
      promoted_driver_count: number;
      blocked_conflict_driver_count: number;
      blocked_rejection_driver_count: number;
      blocked_falsification_driver_count: number;
      missing_checks: string[];
      interpretation: string;
    }>;
    measurement_backlog: {
      policy_version: string;
      status: string;
      total_driver_count: number;
      measured_driver_count: number;
      ready_for_review_count: number;
      rejected_needs_new_evidence_count: number;
      metricization_required_count: number;
      operating_kpi_required_count: number;
      no_traceable_evidence_count: number;
      items: Array<{
        symbol: string;
        company_name: string;
        sample_id: string;
        decision_at: string;
        model_id: string;
        driver_family: "demand" | "supply" | "value_capture" | string;
        driver_id: string;
        driver_label: string;
        status: string;
        measurement_status: "unmeasured" | "partially_measured" | "measured";
        traceable_observation_count: number;
        review_candidate_count: number;
        pending_review_candidate_count: number;
        admitted_candidate_count: number;
        rejected_candidate_count: number;
        required_observations: string[];
        target_operating_kpi_ids: string[];
        next_check: string;
      }>;
      investment_ranking_enabled: boolean;
      action_authorized: boolean;
      scope: string;
    };
    opportunity_ranking_enabled: boolean;
    action_authorized: boolean;
    scope: string;
  };
  review: {
    pending: number;
    accepted: number;
    corrected: number;
    rejected: number;
    review_rate_percent: number;
  };
  causal_review?: {
    available_links: number;
    reviewed_links: number;
    accepted_links: number;
    rejected_links: number;
    review_rate_percent: number;
  };
  causal_effects?: {
    available_links: number;
    reviewed_links: number;
    accepted_links: number;
    rejected_links: number;
    unclassified_accepted_links: number;
    supporting_links: number;
    falsifying_links: number;
    mixed_links: number;
    context_only_links: number;
    review_rate_percent: number;
    effect_classification_rate_percent: number;
    support_share_percent?: number;
    falsification_share_percent?: number;
    by_driver: InvestmentCausalEffectCohort[];
    by_metric: InvestmentCausalEffectCohort[];
    by_market_regime: InvestmentCausalEffectCohort[];
    market_regime_status: string;
    note: string;
  };
  causal_training_dataset?: InvestmentCausalTrainingDataset;
  operating_kpi_review?: {
    available_claims: number;
    active_claims: number;
    superseded_claims: number;
    conflicted_claims: number;
    withdrawn_claims: number;
    definition_change_claims: number;
    distinct_symbols: number;
    distinct_kpis: number;
    distinct_definitions: number;
    reviewed_claims: number;
    accepted_claims: number;
    rejected_claims: number;
    review_rate_percent: number;
  };
  computed_review?: {
    available_comparisons: number;
    reviewed_comparisons: number;
    accepted_comparisons: number;
    rejected_comparisons: number;
    year_over_year_comparisons: number;
    sequential_comparisons: number;
    available_ratios: number;
    reviewed_ratios: number;
    accepted_ratios: number;
    rejected_ratios: number;
    gross_margin_ratios: number;
    operating_margin_ratios: number;
    available_ratio_trends: number;
    reviewed_ratio_trends: number;
    year_over_year_ratio_trends: number;
    sequential_ratio_trends: number;
    review_rate_percent: number;
  };
  claim_corpus?: {
    status: "empty" | "partial" | "ready_for_human_causal_review" | string;
    claim_count: number;
    source_event_count: number;
    distinct_symbols: number;
    distinct_periods: number;
    symbols_with_repeated_periods: number;
    active_claims: number;
    superseded_claims: number;
    conflicted_claims: number;
    withdrawn_claims: number;
    human_accepted_claims: number;
    human_rejected_claims: number;
    derived_comparison_count: number;
    year_over_year_comparison_count: number;
    sequential_comparison_count: number;
    derived_ratio_count: number;
    gross_margin_ratio_count: number;
    operating_margin_ratio_count: number;
    derived_ratio_trend_count: number;
    year_over_year_ratio_trend_count: number;
    sequential_ratio_trend_count: number;
    earliest_published_at?: string;
    latest_published_at?: string;
    metric_coverage: Array<{
      metric_id: string;
      claim_count: number;
      distinct_symbols: number;
      distinct_periods: number;
    }>;
    note: string;
  };
  errors: Array<{
    kind: InvestmentDecisionErrorKind;
    count: number;
    material_or_critical_count: number;
  }>;
  horizons: Array<{
    horizon_market_sessions: number;
    observed_count: number;
    average_asset_return_percent?: number;
    median_asset_return_percent?: number;
    average_excess_return_percent?: number;
    median_excess_return_percent?: number;
    positive_excess_rate_percent?: number;
    average_max_drawdown_percent?: number;
  }>;
  action_horizons: Array<{
    action: InvestmentExposureAction;
    horizon_market_sessions: number;
    observed_count: number;
    average_excess_return_percent?: number;
    positive_excess_rate_percent?: number;
    average_max_drawdown_percent?: number;
    directional_sample_count: number;
    directional_success_rate_percent?: number;
  }>;
  confidence_horizons: Array<{
    confidence: string;
    horizon_market_sessions: number;
    observed_count: number;
    directional_sample_count: number;
    directional_success_rate_percent?: number;
    average_excess_return_percent?: number;
  }>;
  correction_comparisons: Array<{
    horizon_market_sessions: number;
    corrected_sample_count: number;
    comparable_direction_count: number;
    improved_direction_count: number;
    worsened_direction_count: number;
    unchanged_direction_count: number;
    not_comparable_count: number;
  }>;
  evidence_gate: {
    status: "insufficient_evidence" | "eligible_for_reward_design_review" | string;
    minimum_250_session_samples: number;
    observed_250_session_samples: number;
    minimum_non_overlapping_250_session_episodes: number;
    observed_non_overlapping_250_session_episodes: number;
    minimum_distinct_symbols: number;
    observed_distinct_symbols: number;
    minimum_decision_quarters: number;
    observed_decision_quarters: number;
    minimum_review_rate_percent: number;
    observed_review_rate_percent: number;
    reasons: string[];
    scope: string;
  };
  shadow_policy?: {
    policy_version: string;
    status: "insufficient_evidence" | "eligible_for_protocol_review" | string;
    authorization: "not_authorized" | string;
    execution_mode: "read_only_protocol_not_started" | string;
    benchmark_symbol: string;
    constraints: {
      virtual_notional_usd: number;
      long_only: boolean;
      common_stock_only: boolean;
      options_allowed: boolean;
      leverage_allowed: boolean;
      shorting_allowed: boolean;
      maximum_single_name_weight_percent: number;
      maximum_theme_weight_percent: number;
      maximum_gross_exposure_percent: number;
      minimum_cash_weight_percent: number;
      maximum_position_count: number;
      rebalance_frequency: string;
      execution_assumption: string;
      slippage_bps_per_side: number;
    };
    review_requirements: Array<{
      requirement_id: string;
      label: string;
      definition: string;
    }>;
    readiness_reasons: string[];
    candidates: Array<{
      symbol: string;
      company_name: string;
      theme: string;
      sample_id: string;
      decision_at: string;
      action: InvestmentExposureAction;
      zone: InvestmentResearchZone;
      confidence: string;
      market_regime: string;
      status: "blocked" | "eligible_for_protocol_review" | string;
      target_weight_min_percent?: number;
      target_weight_max_percent?: number;
      blocking_reasons: string[];
    }>;
    scope: string;
  };
  reward_design?: {
    design_version: string;
    status: "waiting_for_evidence_gate" | "awaiting_human_objective_approval" | string;
    authorization: "not_approved" | string;
    reward_computation_enabled: boolean;
    human_approval_required: boolean;
    approved_by?: string;
    hard_gates: Array<{
      gate_id: string;
      label: string;
      requirement: string;
      failure_effect: string;
    }>;
    proposed_components: Array<{
      component_id: string;
      label: string;
      proposed_weight_percent: number;
      measurement: string;
      anti_shortcut: string;
    }>;
    proposed_weight_total_percent: number;
    counterfactual_protocol: {
      protocol_version: string;
      status: "not_started" | string;
      point_in_time_only: boolean;
      walk_forward_required: boolean;
      minimum_market_regimes: number;
      comparators: Array<{
        comparator_id: string;
        label: string;
        definition: string;
        purpose: string;
      }>;
      promotion_requirements: string[];
    };
    readiness_reasons: string[];
    scope: string;
  };
};

export type InvestmentCausalTrainingDataset = {
  policy_version: string;
  status: "insufficient_human_labels" | "eligible_for_dataset_governance_review" | string;
  dataset_fingerprint_sha256: string;
  eligible_example_count: number;
  train_example_count: number;
  validation_example_count: number;
  holdout_test_example_count: number;
  distinct_symbols: number;
  distinct_drivers: number;
  development_target_counts: Record<string, number>;
  excluded_unclassified_links: number;
  excluded_future_evidence: number;
  deduplicated_review_rows: number;
  company_split_isolation_verified: boolean;
  source_group_split_isolation_verified: boolean;
  connected_component_count: number;
  shared_source_group_count: number;
  largest_component_symbol_count: number;
  holdout_labels_withheld: boolean;
  training_authorized: boolean;
  readiness_reasons: string[];
  feature_scope: string;
  split_scope: string;
  authorization_scope: string;
};

export type InvestmentCausalDatasetGovernanceVerdict =
  | "changes_requested"
  | "approved_for_offline_experiment"
  | "rejected";

export type InvestmentCausalDatasetGovernanceRecord = {
  schema_version: "hone-causal-dataset-governance-review-v3" | string;
  review_id: string;
  previous_review_id?: string;
  dataset_policy_version: string;
  dataset_fingerprint_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: InvestmentCausalDatasetGovernanceVerdict;
  rationale: string;
  eligible_example_count: number;
  distinct_symbols: number;
  distinct_drivers: number;
  company_split_isolation_confirmed: boolean;
  source_group_split_isolation_confirmed: boolean;
  holdout_seal_confirmed: boolean;
  future_leakage_audit_confirmed: boolean;
};

export type InvestmentCausalDatasetGovernance = {
  schema_version: "hone-causal-dataset-governance-review-v3" | string;
  dataset: InvestmentCausalTrainingDataset;
  latest_review?: InvestmentCausalDatasetGovernanceRecord;
  current_dataset_approved: boolean;
  offline_experiment_registration_allowed: boolean;
  offline_training_run_authorized: false | boolean;
  preference_learning_authorized: false | boolean;
  reinforcement_learning_authorized: false | boolean;
  deployment_authorized: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type InvestmentCausalDatasetGovernanceRequest = {
  expected_review_id?: string;
  dataset_policy_version: string;
  dataset_fingerprint_sha256: string;
  verdict: InvestmentCausalDatasetGovernanceVerdict;
  rationale: string;
  company_split_isolation_confirmed?: boolean;
  source_group_split_isolation_confirmed?: boolean;
  holdout_seal_confirmed?: boolean;
  future_leakage_audit_confirmed?: boolean;
};

export type InvestmentCausalTrainingAlgorithm =
  | "frozen_prompt_baseline"
  | "supervised_causal_classifier";

export type InvestmentCausalTrainingExperimentRecord = {
  schema_version: "hone-causal-training-experiment-v1" | string;
  experiment_id: string;
  registered_at: string;
  registered_by: string;
  dataset_review_id: string;
  dataset_policy_version: string;
  dataset_fingerprint_sha256: string;
  experiment_name: string;
  algorithm: InvestmentCausalTrainingAlgorithm;
  base_model_id: string;
  base_model_version: string;
  random_seed: number;
  max_epochs: number;
  status: "registered_not_run" | string;
  task_contract: string;
  allowed_input_splits: string[];
  holdout_access_allowed: false | boolean;
  outbound_network_allowed: false | boolean;
  external_tools_allowed: false | boolean;
  production_writes_allowed: false | boolean;
  arbitrary_code_allowed: false | boolean;
  run_authorized: false | boolean;
  deployment_authorized: false | boolean;
  trading_authorized: false | boolean;
};

export type InvestmentCausalTrainingExperimentRegistry = {
  schema_version: "hone-causal-training-experiment-v1" | string;
  sandbox_policy_version: string;
  dataset_policy_version: string;
  dataset_fingerprint_sha256: string;
  current_dataset_review_id?: string;
  registration_allowed: boolean;
  allowed_algorithms: InvestmentCausalTrainingAlgorithm[];
  experiments: InvestmentCausalTrainingExperimentRecord[];
  blind_evaluation_protocol: {
    policy_version: string;
    status: string;
    candidate_scope: string;
    minimum_distinct_seeds: number;
    development_splits: string[];
    sealed_split: string;
    holdout_labels_visible_to_training_worker: boolean;
    independent_evaluator_required: boolean;
    frozen_baseline_required: boolean;
    metric_gates: Array<{
      metric_id: string;
      label: string;
      comparison: "at_least" | "at_most" | string;
      threshold: number;
    }>;
    thresholds_origin: string;
    promotion_scope: string;
  };
  drift_monitoring_protocol: {
    policy_version: string;
    status: string;
    minimum_audited_examples: number;
    rolling_window_days: number;
    gates: Array<{
      metric_id: string;
      label: string;
      comparison: string;
      warning_threshold: number;
      disable_threshold: number;
    }>;
    schema_change_is_hard_stop: boolean;
    future_leakage_is_hard_stop: boolean;
    warning_action: string;
    disable_action: string;
    thresholds_origin: string;
  };
  offline_training_run_authorized: false | boolean;
  preference_learning_authorized: false | boolean;
  reinforcement_learning_authorized: false | boolean;
  deployment_authorized: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type InvestmentCausalTrainingExperimentRequest = {
  expected_dataset_review_id: string;
  dataset_policy_version: string;
  dataset_fingerprint_sha256: string;
  experiment_name: string;
  algorithm: InvestmentCausalTrainingAlgorithm;
  base_model_id: string;
  base_model_version: string;
  random_seed: number;
  max_epochs: number;
};

export type InvestmentRewardGovernanceVerdict =
  | "changes_requested"
  | "approved_for_offline_research"
  | "rejected";

export type InvestmentRewardGovernanceComponentWeight = {
  component_id: string;
  weight_percent: number;
};

export type InvestmentRewardGovernanceRecord = {
  schema_version: "hone-reward-governance-review-v1" | string;
  review_id: string;
  previous_review_id?: string;
  design_version: string;
  proposal_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: InvestmentRewardGovernanceVerdict;
  rationale: string;
  component_weights: InvestmentRewardGovernanceComponentWeight[];
  confirmed_hard_gate_ids: string[];
  counterfactual_protocol_confirmed: boolean;
};

export type InvestmentRewardGovernance = {
  schema_version: "hone-reward-governance-review-v1" | string;
  design_version: string;
  proposal_sha256: string;
  evidence_gate_status: string;
  latest_review?: InvestmentRewardGovernanceRecord;
  reward_computation_enabled: false | boolean;
  shadow_portfolio_authorized: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type InvestmentRewardGovernanceRequest = {
  expected_review_id?: string;
  design_version: string;
  proposal_sha256: string;
  verdict: InvestmentRewardGovernanceVerdict;
  rationale: string;
  component_weights?: InvestmentRewardGovernanceComponentWeight[];
  confirmed_hard_gate_ids?: string[];
  counterfactual_protocol_confirmed?: boolean;
};

export type InvestmentShadowProtocolGovernanceVerdict =
  | "changes_requested"
  | "approved_for_future_shadow_implementation"
  | "rejected";

export type InvestmentShadowProtocolGovernanceRecord = {
  schema_version: "hone-shadow-protocol-governance-review-v1" | string;
  review_id: string;
  previous_review_id?: string;
  policy_version: string;
  protocol_sha256: string;
  reward_design_version: string;
  reward_proposal_sha256: string;
  reward_review_id?: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: InvestmentShadowProtocolGovernanceVerdict;
  rationale: string;
  confirmed_requirement_ids: string[];
  implementation_boundary_confirmed: boolean;
};

export type InvestmentShadowProtocolGovernance = {
  schema_version: "hone-shadow-protocol-governance-review-v1" | string;
  policy_version: string;
  protocol_sha256: string;
  review_requirements: Array<{
    requirement_id: string;
    label: string;
    definition: string;
  }>;
  evidence_gate_status: string;
  reward_governance_status: string;
  reward_review_id?: string;
  latest_review?: InvestmentShadowProtocolGovernanceRecord;
  future_shadow_implementation_registration_allowed: boolean;
  shadow_ledger_enabled: false | boolean;
  shadow_portfolio_authorized: false | boolean;
  trading_authorized: false | boolean;
  broker_connected: false | boolean;
  scope: string;
};

export type InvestmentShadowProtocolGovernanceRequest = {
  expected_review_id?: string;
  expected_reward_review_id?: string;
  policy_version: string;
  protocol_sha256: string;
  verdict: InvestmentShadowProtocolGovernanceVerdict;
  rationale: string;
  confirmed_requirement_ids?: string[];
  implementation_boundary_confirmed?: boolean;
};

export type InvestmentShadowImplementationKind =
  | "deterministic_replay_specification";

export type InvestmentShadowImplementationRecord = {
  schema_version: "hone-shadow-implementation-registry-v1" | string;
  implementation_id: string;
  implementation_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  shadow_review_id: string;
  reward_review_id: string;
  policy_version: string;
  protocol_sha256: string;
  implementation_name: string;
  implementation_kind: InvestmentShadowImplementationKind;
  code_revision: string;
  status: "registered_not_started" | string;
  input_contract: string;
  accounting_contract: string;
  benchmark_symbol: string;
  execution_assumption: string;
  deterministic_replay_required: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  production_writes_allowed: boolean;
  ledger_creation_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  run_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  trading_authorized: boolean;
};

export type InvestmentShadowImplementationRegistry = {
  schema_version: "hone-shadow-implementation-registry-v1" | string;
  sandbox_policy_version: "hone-shadow-implementation-sandbox-v1" | string;
  policy_version: string;
  protocol_sha256: string;
  current_shadow_review_id?: string;
  current_reward_review_id?: string;
  registration_allowed: boolean;
  allowed_implementation_kinds: InvestmentShadowImplementationKind[];
  implementations: InvestmentShadowImplementationRecord[];
  shadow_ledger_enabled: false | boolean;
  shadow_run_authorized: false | boolean;
  shadow_portfolio_authorized: false | boolean;
  order_generation_authorized: false | boolean;
  broker_connected: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type InvestmentShadowImplementationRegistrationRequest = {
  expected_shadow_review_id: string;
  expected_reward_review_id: string;
  policy_version: string;
  protocol_sha256: string;
  implementation_name: string;
  implementation_kind: InvestmentShadowImplementationKind;
  code_revision: string;
};

export type HistoricalDecisionAnchorAction =
  | "increase"
  | "maintain"
  | "reduce"
  | "exit"
  | "research_only";

export type HistoricalDecisionAnchorReviewVerdict =
  | "confirmed"
  | "revised"
  | "rejected";

export type HistoricalDecisionAnchorSource = {
  source_item_id: string;
  source_sha256: string;
  title: string;
  filename: string;
  source_name: string;
  source_date: string;
  tickers: string[];
  parse_status: string;
};

export type HistoricalDecisionAnchorCandidate = {
  schema_version: "hone-historical-decision-anchor-candidate-v1" | string;
  candidate_id: string;
  candidate_sha256: string;
  source_policy_version: string;
  source_item_id: string;
  source_sha256: string;
  source_title: string;
  source_filename: string;
  source_name: string;
  claimed_source_date: string;
  symbol: string;
  source_locator: string;
  verbatim_excerpt: string;
  candidate_action: HistoricalDecisionAnchorAction;
  candidate_thesis: string;
  candidate_origin: string;
  created_at: string;
  created_by: string;
  human_confirmation_status: "pending" | string;
  benchmark_eligible: false | boolean;
  decision_training_eligible: false | boolean;
  reward_evidence_eligible: false | boolean;
  shadow_evidence_eligible: false | boolean;
  trading_authorized: false | boolean;
};

export type HistoricalDecisionAnchorReview = {
  schema_version: "hone-historical-decision-anchor-review-v2-available-at" | string;
  review_id: string;
  previous_review_id?: string;
  candidate_id: string;
  candidate_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalDecisionAnchorReviewVerdict;
  confirmation_statement: string;
  decision_available_at?: string;
  source_time_confirmed: boolean;
  speaker_identity_confirmed: boolean;
  later_evidence_excluded_confirmed: boolean;
  final_action?: HistoricalDecisionAnchorAction;
  final_thesis?: string;
  benchmark_eligible: boolean;
  decision_training_eligible: false | boolean;
  reward_evidence_eligible: false | boolean;
  shadow_evidence_eligible: false | boolean;
  trading_authorized: false | boolean;
};

export type HistoricalDecisionAnchorRegistry = {
  schema_version: "hone-historical-decision-anchor-registry-v1" | string;
  source_policy_version: string;
  benchmark_policy_version: string;
  source_count: number;
  source_symbol_count: number;
  earliest_source_date?: string;
  latest_source_date?: string;
  pending_candidate_count: number;
  confirmed_anchor_count: number;
  rejected_candidate_count: number;
  sources: HistoricalDecisionAnchorSource[];
  anchors: Array<{
    candidate: HistoricalDecisionAnchorCandidate;
    latest_review?: HistoricalDecisionAnchorReview;
  }>;
  automatic_extraction_authorized: false | boolean;
  automatic_confirmation_authorized: false | boolean;
  benchmark_outcome_labeling_enabled: false | boolean;
  decision_training_authorized: false | boolean;
  reward_evidence_authorized: false | boolean;
  shadow_evidence_authorized: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type HistoricalAnchorDiscoverySuggestion = {
  suggestion_id: string;
  source_item_id: string;
  source_sha256: string;
  source_title: string;
  source_filename: string;
  source_name: string;
  source_date: string;
  tickers: string[];
  speaker_label?: string;
  dominant_source_speaker: boolean;
  personal_decision_context: boolean;
  context_flags: string[];
  review_priority_reasons: string[];
  screening_status: "pending" | HistoricalAnchorDiscoveryScreeningVerdict | string;
  screening_record_id?: string;
  source_locator: string;
  verbatim_excerpt: string;
  context_window: {
    start_line: number;
    end_line: number;
    verbatim_context: string;
    context_sha256: string;
    truncated: boolean;
  };
  matched_action_cues: string[];
  suggested_action?: HistoricalDecisionAnchorAction;
  interpretation_status: "unconfirmed_search_hit" | string;
  already_saved_candidate: boolean;
  requires_manual_thesis: boolean;
  requires_speaker_identity_confirmation: boolean;
  requires_exact_time_confirmation: boolean;
  benchmark_eligible: false | boolean;
  decision_training_eligible: false | boolean;
  reward_evidence_eligible: false | boolean;
  shadow_evidence_eligible: false | boolean;
  trading_authorized: false | boolean;
  rank_score: number;
};

export type HistoricalAnchorDiscoveryResponse = {
  schema_version: "hone-historical-anchor-discovery-v1" | string;
  discovery_policy_version: string;
  source_count: number;
  matched_source_count: number;
  suggestion_count: number;
  suggestions: HistoricalAnchorDiscoverySuggestion[];
  active_review_batch_policy_version: string;
  active_review_batch_size: number;
  active_review_batch: HistoricalAnchorDiscoverySuggestion[];
  screened_suggestion_count: number;
  pending_screening_count: number;
  shortlisted_review_count: number;
  shortlisted_review: HistoricalAnchorDiscoverySuggestion[];
  automatic_candidate_creation_authorized: false | boolean;
  automatic_confirmation_authorized: false | boolean;
  benchmark_outcome_labeling_enabled: false | boolean;
  decision_training_authorized: false | boolean;
  reward_evidence_authorized: false | boolean;
  shadow_evidence_authorized: false | boolean;
  trading_authorized: false | boolean;
  scope: string;
};

export type HistoricalAnchorDiscoveryScreeningVerdict =
  | "continue_candidate_review"
  | "not_decision_context"
  | "needs_more_context";

export type HistoricalAnchorDiscoveryScreeningRecord = {
  schema_version: "hone-historical-anchor-discovery-screening-v2-correction-chain" | string;
  screening_id: string;
  previous_screening_id?: string;
  suggestion_id: string;
  discovery_policy_version: string;
  review_batch_policy_version: string;
  source_item_id: string;
  source_sha256: string;
  source_locator: string;
  excerpt_sha256: string;
  verdict: HistoricalAnchorDiscoveryScreeningVerdict;
  submitted_at: string;
  submitted_by: string;
  correction_reason?: string;
  candidate_created: false | boolean;
  speaker_identity_confirmed: false | boolean;
  investment_logic_confirmed: false | boolean;
  benchmark_eligible: false | boolean;
  decision_training_eligible: false | boolean;
  reward_evidence_eligible: false | boolean;
  shadow_evidence_eligible: false | boolean;
  trading_authorized: false | boolean;
};

export type ScreenHistoricalAnchorDiscoveryRequest = {
  expected_source_sha256: string;
  expected_screening_id?: string;
  verdict: HistoricalAnchorDiscoveryScreeningVerdict;
  correction_reason?: string;
};

export type CreateHistoricalDecisionAnchorCandidateRequest = {
  source_item_id: string;
  expected_source_sha256: string;
  symbol: string;
  source_locator: string;
  verbatim_excerpt: string;
  candidate_action: HistoricalDecisionAnchorAction;
  candidate_thesis: string;
};

export type ReviewHistoricalDecisionAnchorRequest = {
  expected_review_id?: string;
  verdict: HistoricalDecisionAnchorReviewVerdict;
  confirmation_statement: string;
  decision_available_at?: string;
  source_time_confirmed: boolean;
  speaker_identity_confirmed: boolean;
  later_evidence_excluded_confirmed: boolean;
  revised_action?: HistoricalDecisionAnchorAction;
  revised_thesis?: string;
};

export type HistoricalStateComponentId =
  | "industry_thesis"
  | "company_fundamentals"
  | "financial_verification"
  | "valuation"
  | "crowding"
  | "market_regime"
  | "portfolio_context";

export type HistoricalStateComponentStatus =
  | "evidence_backed"
  | "explicitly_missing";

export type HistoricalStateReviewVerdict =
  | "approved_for_benchmark"
  | "changes_requested"
  | "rejected";

export type HistoricalStateEvidence = {
  evidence_sha256: string;
  source_item_id: string;
  source_sha256: string;
  source_title: string;
  source_name: string;
  source_date: string;
  claimed_available_at: string;
  source_locator: string;
  verbatim_excerpt: string;
  normalized_claim: string;
};

export type HistoricalStateComponent = {
  component_id: HistoricalStateComponentId;
  status: HistoricalStateComponentStatus;
  evidence: HistoricalStateEvidence[];
  missing_reason?: string;
};

export type HistoricalStateReconstructionCandidate = {
  schema_version: "hone-historical-state-reconstruction-candidate-v1" | string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  component_policy_version: string;
  anchor_candidate_id: string;
  anchor_candidate_sha256: string;
  anchor_review_id: string;
  symbol: string;
  anchor_action: HistoricalDecisionAnchorAction;
  anchor_thesis: string;
  decision_available_at: string;
  components: HistoricalStateComponent[];
  created_at: string;
  created_by: string;
  human_review_status: string;
  benchmark_state_eligible: boolean;
  outcome_labeling_eligible: boolean;
  decision_training_eligible: boolean;
  reward_evidence_eligible: boolean;
  shadow_evidence_eligible: boolean;
  trading_authorized: boolean;
};

export type HistoricalStateReconstructionReview = {
  schema_version: "hone-historical-state-reconstruction-review-v1" | string;
  review_id: string;
  previous_review_id?: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalStateReviewVerdict;
  review_statement: string;
  anchor_binding_confirmed: boolean;
  source_bytes_confirmed: boolean;
  availability_times_confirmed: boolean;
  no_future_information_confirmed: boolean;
  missingness_preserved_confirmed: boolean;
  component_interpretations_confirmed: boolean;
  benchmark_state_eligible: boolean;
  outcome_labeling_eligible: boolean;
  decision_training_eligible: boolean;
  reward_evidence_eligible: boolean;
  shadow_evidence_eligible: boolean;
  trading_authorized: boolean;
};

export type HistoricalStateReconstructionRegistry = {
  schema_version: "hone-historical-state-reconstruction-registry-v1" | string;
  component_policy_version: string;
  outcome_protocol: {
    protocol_version: string;
    horizons_market_sessions: number[];
    asset_price_basis: string;
    benchmark_symbol: string;
    benchmark_price_basis: string;
    start_rule: string;
    metrics: string[];
    missing_session_rule: string;
    future_information_rule: string;
    automatic_labeling_enabled: boolean;
  };
  confirmed_anchor_count: number;
  reconstruction_candidate_count: number;
  benchmark_ready_count: number;
  stale_reconstruction_count: number;
  confirmed_anchors: Array<{
    candidate_id: string;
    candidate_sha256: string;
    review_id: string;
    symbol: string;
    final_action: HistoricalDecisionAnchorAction;
    final_thesis: string;
    decision_available_at: string;
  }>;
  required_components: Array<{
    component_id: HistoricalStateComponentId;
    label: string;
    requirement: string;
  }>;
  reconstructions: Array<{
    candidate: HistoricalStateReconstructionCandidate;
    latest_review?: HistoricalStateReconstructionReview;
    anchor_binding_current: boolean;
    benchmark_state_ready: boolean;
  }>;
  state_reconstruction_status: string;
  automatic_reconstruction_authorized: boolean;
  benchmark_outcome_labeling_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type CreateHistoricalStateReconstructionRequest = {
  anchor_candidate_id: string;
  expected_anchor_candidate_sha256: string;
  expected_anchor_review_id: string;
  components: Array<{
    component_id: HistoricalStateComponentId;
    status: HistoricalStateComponentStatus;
    evidence: Array<{
      source_item_id: string;
      expected_source_sha256: string;
      claimed_available_at: string;
      source_locator: string;
      verbatim_excerpt: string;
      normalized_claim: string;
    }>;
    missing_reason?: string;
  }>;
};

export type ReviewHistoricalStateReconstructionRequest = {
  expected_review_id?: string;
  verdict: HistoricalStateReviewVerdict;
  review_statement: string;
  anchor_binding_confirmed: boolean;
  source_bytes_confirmed: boolean;
  availability_times_confirmed: boolean;
  no_future_information_confirmed: boolean;
  missingness_preserved_confirmed: boolean;
  component_interpretations_confirmed: boolean;
};

export type HistoricalOutcomeGovernanceVerdict =
  | "approved_for_implementation_review"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeGovernanceReview = {
  schema_version: "hone-historical-outcome-governance-review-v1" | string;
  review_id: string;
  previous_review_id?: string;
  protocol_version: string;
  protocol_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeGovernanceVerdict;
  rationale: string;
  benchmark_state_count_at_review: number;
  protocol_frozen_pre_outcome_confirmed: boolean;
  adjusted_close_source_confirmed: boolean;
  common_session_rule_confirmed: boolean;
  benchmark_rule_confirmed: boolean;
  future_isolation_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
  labeler_implementation_registration_eligible: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeGovernanceRegistry = {
  schema_version: "hone-historical-outcome-governance-registry-v1" | string;
  protocol: HistoricalStateReconstructionRegistry["outcome_protocol"];
  protocol_sha256: string;
  benchmark_ready_count: number;
  latest_review?: HistoricalOutcomeGovernanceReview;
  protocol_review_status: string;
  labeler_implementation_registration_eligible: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeGovernanceRequest = {
  expected_review_id?: string;
  verdict: HistoricalOutcomeGovernanceVerdict;
  rationale: string;
  protocol_frozen_pre_outcome_confirmed: boolean;
  adjusted_close_source_confirmed: boolean;
  common_session_rule_confirmed: boolean;
  benchmark_rule_confirmed: boolean;
  future_isolation_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
};

export type HistoricalOutcomeLabelerImplementationKind =
  "deterministic_common_session_adjusted_close";

export type HistoricalOutcomeLabelerReviewVerdict =
  | "approved_for_offline_dry_run_authorization_review"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeLabelerImplementationRecord = {
  schema_version: "hone-historical-outcome-labeler-implementation-v1" | string;
  implementation_id: string;
  implementation_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  governance_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  sandbox_policy_version: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeLabelerImplementationKind;
  code_revision: string;
  status: string;
  input_contract: string;
  output_contract: string;
  price_snapshot_source: string;
  price_basis: string;
  benchmark_symbol: string;
  horizons_market_sessions: number[];
  metrics: string[];
  common_session_rule_required: boolean;
  deterministic_replay_required: boolean;
  future_information_isolation_required: boolean;
  missing_data_fail_closed_required: boolean;
  max_parallel_series: number;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  label_writes_allowed: boolean;
  run_authorized: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelerReview = {
  schema_version: "hone-historical-outcome-labeler-review-v1" | string;
  review_id: string;
  previous_review_id?: string;
  implementation_id: string;
  implementation_spec_sha256: string;
  governance_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeLabelerReviewVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  protocol_binding_confirmed: boolean;
  adjusted_close_and_common_sessions_confirmed: boolean;
  deterministic_replay_confirmed: boolean;
  future_isolation_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
  no_network_or_production_writes_confirmed: boolean;
  offline_dry_run_authorization_review_eligible: boolean;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelerRegistryItem = {
  implementation: HistoricalOutcomeLabelerImplementationRecord;
  latest_review?: HistoricalOutcomeLabelerReview;
  governance_binding_current: boolean;
  offline_dry_run_authorization_review_eligible: boolean;
};

export type HistoricalOutcomeLabelerRegistry = {
  schema_version: "hone-historical-outcome-labeler-registry-v1" | string;
  sandbox_policy_version: "hone-historical-outcome-labeler-sandbox-v1" | string;
  protocol_version: string;
  protocol_sha256: string;
  current_governance_review_id?: string;
  registration_allowed: boolean;
  allowed_implementation_kinds: HistoricalOutcomeLabelerImplementationKind[];
  implementations: HistoricalOutcomeLabelerRegistryItem[];
  current_binding_implementation_count: number;
  reviewed_implementation_count: number;
  labeler_review_status: string;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeLabelerRequest = {
  expected_governance_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeLabelerImplementationKind;
  code_revision: string;
};

export type ReviewHistoricalOutcomeLabelerRequest = {
  expected_review_id?: string;
  verdict: HistoricalOutcomeLabelerReviewVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  protocol_binding_confirmed: boolean;
  adjusted_close_and_common_sessions_confirmed: boolean;
  deterministic_replay_confirmed: boolean;
  future_isolation_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
  no_network_or_production_writes_confirmed: boolean;
};

export type ApprovedHistoricalBenchmarkState = {
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  anchor_candidate_id: string;
  anchor_review_id: string;
  symbol: string;
  decision_available_at: string;
};

export type ApprovedHistoricalOutcomeLabeler = {
  implementation_id: string;
  implementation_spec_sha256: string;
  implementation_review_id: string;
  governance_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  code_revision: string;
};

export type SealedAdjustedClosePoint = {
  date: string;
  adjusted_close: number;
};

export type HistoricalOutcomePriceSnapshot = {
  schema_version: "hone-historical-outcome-price-snapshot-v1" | string;
  ingestion_policy_version: "hone-historical-outcome-price-ingestion-v1" | string;
  snapshot_id: string;
  snapshot_sha256: string;
  sealed_at: string;
  sealed_by: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  anchor_candidate_id: string;
  anchor_review_id: string;
  decision_available_at: string;
  implementation_id: string;
  implementation_spec_sha256: string;
  implementation_review_id: string;
  governance_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  code_revision: string;
  provider: string;
  provider_endpoint_template: string;
  price_basis: string;
  asset_symbol: string;
  benchmark_symbol: string;
  requested_from: string;
  requested_to: string;
  asset_payload_sha256: string;
  benchmark_payload_sha256: string;
  asset_series_sha256: string;
  benchmark_series_sha256: string;
  asset_points: SealedAdjustedClosePoint[];
  benchmark_points: SealedAdjustedClosePoint[];
  common_session_count: number;
  covered_horizons_market_sessions: number[];
  all_protocol_horizons_covered: boolean;
  outcome_metrics_computed: boolean;
  label_written: boolean;
  historical_state_mutated: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomePriceSnapshotItem = {
  snapshot: HistoricalOutcomePriceSnapshot;
  reconstruction_binding_current: boolean;
  implementation_binding_current: boolean;
  dry_run_authorization_review_eligible: boolean;
};

export type HistoricalOutcomePriceSnapshotRegistry = {
  schema_version: "hone-historical-outcome-price-snapshot-registry-v1" | string;
  ingestion_policy_version: string;
  protocol_version: string;
  protocol_sha256: string;
  eligible_benchmark_states: ApprovedHistoricalBenchmarkState[];
  eligible_labelers: ApprovedHistoricalOutcomeLabeler[];
  snapshots: HistoricalOutcomePriceSnapshotItem[];
  current_snapshot_count: number;
  fully_covered_snapshot_count: number;
  price_snapshot_ingestion_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type IngestHistoricalOutcomePriceSnapshotRequest = {
  reconstruction_id: string;
  expected_reconstruction_sha256: string;
  expected_reconstruction_review_id: string;
  implementation_id: string;
  expected_implementation_spec_sha256: string;
  expected_implementation_review_id: string;
  expected_protocol_sha256: string;
};

export type HistoricalOutcomeDryRunAuthorizationVerdict =
  | "approved_for_dry_run_implementation_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeDryRunAuthorizationReview = {
  schema_version: "hone-historical-outcome-dry-run-authorization-review-v1" | string;
  authorization_policy_version: string;
  review_id: string;
  previous_review_id?: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  implementation_id: string;
  implementation_spec_sha256: string;
  implementation_review_id: string;
  protocol_version: string;
  protocol_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeDryRunAuthorizationVerdict;
  rationale: string;
  current_bindings_confirmed: boolean;
  sealed_snapshot_integrity_confirmed: boolean;
  provider_provenance_confirmed: boolean;
  complete_common_session_coverage_confirmed: boolean;
  deterministic_fixture_confirmed: boolean;
  isolated_output_confirmed: boolean;
  no_label_or_production_writes_confirmed: boolean;
  dry_run_implementation_registration_eligible: boolean;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunAuthorizationItem = {
  snapshot_id: string;
  snapshot_sha256: string;
  asset_symbol: string;
  common_session_count: number;
  current_binding: boolean;
  latest_review?: HistoricalOutcomeDryRunAuthorizationReview;
  dry_run_implementation_registration_eligible: boolean;
};

export type HistoricalOutcomeDryRunAuthorizationRegistry = {
  schema_version: "hone-historical-outcome-dry-run-authorization-registry-v1" | string;
  authorization_policy_version: string;
  items: HistoricalOutcomeDryRunAuthorizationItem[];
  reviewed_snapshot_count: number;
  registration_eligible_snapshot_count: number;
  authorization_status: string;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeDryRunAuthorizationRequest = {
  expected_review_id?: string;
  expected_snapshot_sha256: string;
  expected_implementation_spec_sha256: string;
  verdict: HistoricalOutcomeDryRunAuthorizationVerdict;
  rationale: string;
  current_bindings_confirmed: boolean;
  sealed_snapshot_integrity_confirmed: boolean;
  provider_provenance_confirmed: boolean;
  complete_common_session_coverage_confirmed: boolean;
  deterministic_fixture_confirmed: boolean;
  isolated_output_confirmed: boolean;
  no_label_or_production_writes_confirmed: boolean;
};

export type ApprovedHistoricalOutcomeDryRunAuthorization = {
  authorization_review_id: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  implementation_id: string;
  implementation_spec_sha256: string;
  implementation_review_id: string;
  labeler_code_revision: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  requested_from: string;
  requested_to: string;
  asset_payload_sha256: string;
  benchmark_payload_sha256: string;
  asset_series_sha256: string;
  benchmark_series_sha256: string;
  common_session_count: number;
  covered_horizons_market_sessions: number[];
};

export type HistoricalOutcomeDryRunImplementationKind =
  "deterministic_isolated_common_session_replay";

export type HistoricalOutcomeDryRunImplementationRecord = {
  schema_version: "hone-historical-outcome-dry-run-implementation-v1" | string;
  dry_run_implementation_id: string;
  dry_run_implementation_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  authorization_review_id: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  labeler_implementation_id: string;
  labeler_implementation_spec_sha256: string;
  labeler_implementation_review_id: string;
  labeler_code_revision: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  requested_from: string;
  requested_to: string;
  asset_payload_sha256: string;
  benchmark_payload_sha256: string;
  asset_series_sha256: string;
  benchmark_series_sha256: string;
  common_session_count: number;
  covered_horizons_market_sessions: number[];
  sandbox_policy_version: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeDryRunImplementationKind;
  code_revision: string;
  status: "registered_not_run" | string;
  input_contract: string;
  output_contract: string;
  metrics: string[];
  deterministic_replay_required: boolean;
  isolated_output_required: boolean;
  future_information_isolation_required: boolean;
  missing_data_fail_closed_required: boolean;
  max_parallel_series: number;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  outcome_label_writes_allowed: boolean;
  training_writes_allowed: boolean;
  reward_writes_allowed: boolean;
  shadow_writes_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  run_authorized: boolean;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunImplementationItem = {
  implementation: HistoricalOutcomeDryRunImplementationRecord;
  authorization_binding_current: boolean;
  run_authorization_review_eligible: boolean;
};

export type HistoricalOutcomeDryRunImplementationRegistry = {
  schema_version: "hone-historical-outcome-dry-run-implementation-registry-v1" | string;
  sandbox_policy_version: string;
  eligible_authorizations: ApprovedHistoricalOutcomeDryRunAuthorization[];
  allowed_implementation_kinds: HistoricalOutcomeDryRunImplementationKind[];
  registration_allowed: boolean;
  implementations: HistoricalOutcomeDryRunImplementationItem[];
  implementation_count: number;
  current_binding_implementation_count: number;
  run_authorization_review_eligible_count: number;
  implementation_status: string;
  offline_dry_run_enabled: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeDryRunImplementationRequest = {
  snapshot_id: string;
  expected_authorization_review_id: string;
  expected_snapshot_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_protocol_sha256: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeDryRunImplementationKind;
  code_revision: string;
};

export type HistoricalOutcomeDryRunRunAuthorizationVerdict =
  | "approved_for_isolated_runner_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeDryRunRunAuthorizationReview = {
  schema_version: "hone-historical-outcome-dry-run-run-authorization-review-v1" | string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  dry_run_implementation_id: string;
  dry_run_implementation_spec_sha256: string;
  authorization_review_id: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  labeler_implementation_id: string;
  labeler_implementation_spec_sha256: string;
  labeler_implementation_review_id: string;
  labeler_code_revision: string;
  protocol_version: string;
  protocol_sha256: string;
  sandbox_policy_version: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeDryRunImplementationKind;
  code_revision: string;
  implementation_status: "registered_not_run" | string;
  implementation_registered_by: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  code_revision_reproducible_confirmed: boolean;
  sealed_input_read_only_confirmed: boolean;
  deterministic_common_session_replay_confirmed: boolean;
  isolated_ephemeral_output_confirmed: boolean;
  resource_bounds_confirmed: boolean;
  no_network_or_external_tools_confirmed: boolean;
  no_production_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
  reviewer_independent_from_registrant: boolean;
  isolated_runner_registration_eligible: boolean;
  run_authorized: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunRunAuthorizationItem = {
  implementation: HistoricalOutcomeDryRunImplementationRecord;
  current_binding: boolean;
  latest_review?: HistoricalOutcomeDryRunRunAuthorizationReview;
  isolated_runner_registration_eligible: boolean;
};

export type HistoricalOutcomeDryRunRunAuthorizationRegistry = {
  schema_version: "hone-historical-outcome-dry-run-run-authorization-registry-v1" | string;
  policy_version: string;
  items: HistoricalOutcomeDryRunRunAuthorizationItem[];
  review_eligible_implementation_count: number;
  reviewed_implementation_count: number;
  isolated_runner_registration_eligible_count: number;
  authorization_status: string;
  run_authorized: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeDryRunRunAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_implementation_spec_sha256: string;
  expected_authorization_review_id: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  code_revision_reproducible_confirmed: boolean;
  sealed_input_read_only_confirmed: boolean;
  deterministic_common_session_replay_confirmed: boolean;
  isolated_ephemeral_output_confirmed: boolean;
  resource_bounds_confirmed: boolean;
  no_network_or_external_tools_confirmed: boolean;
  no_production_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
};

export type ApprovedHistoricalOutcomeDryRunRunAuthorization = {
  implementation: HistoricalOutcomeDryRunImplementationRecord;
  review: HistoricalOutcomeDryRunRunAuthorizationReview;
};

export type HistoricalOutcomeDryRunIsolatedRunnerKind =
  "ephemeral_deterministic_process";

export type HistoricalOutcomeDryRunIsolatedRunnerRecord = {
  schema_version: "hone-historical-outcome-dry-run-isolated-runner-v1" | string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  run_authorization_review_id: string;
  run_authorization_review_sha256: string;
  run_authorization_reviewer_id: string;
  dry_run_implementation_id: string;
  dry_run_implementation_spec_sha256: string;
  dry_run_implementation_code_revision: string;
  dry_run_implementation_kind: HistoricalOutcomeDryRunImplementationKind;
  authorization_review_id: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  reconstruction_review_id: string;
  labeler_implementation_id: string;
  labeler_implementation_spec_sha256: string;
  labeler_implementation_review_id: string;
  labeler_code_revision: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  requested_from: string;
  requested_to: string;
  asset_series_sha256: string;
  benchmark_series_sha256: string;
  common_session_count: number;
  covered_horizons_market_sessions: number[];
  runtime_policy_version: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  implementation_review_sha256: string;
  protocol_review_sha256: string;
  protocol_registration_sha256: string;
  protocol_specification_sha256: string;
  design_specification_sha256: string;
  initial_observation_validation_sha256: string;
  status: "registered_not_run" | string;
  input_mount_contract: string;
  output_contract: string;
  invocation_contract: string;
  callable_entrypoint_registered: boolean;
  input_mount_read_only_required: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  output_validation_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  host_environment_inherited: boolean;
  allowed_environment_variables: string[];
  secrets_available: boolean;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  outcome_label_writes_allowed: boolean;
  training_writes_allowed: boolean;
  reward_writes_allowed: boolean;
  shadow_writes_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  invocation_authorized: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunIsolatedRunnerItem = {
  runner: HistoricalOutcomeDryRunIsolatedRunnerRecord;
  run_authorization_binding_current: boolean;
  execution_authorization_review_eligible: boolean;
};

export type HistoricalOutcomeDryRunIsolatedRunnerRegistry = {
  schema_version: "hone-historical-outcome-dry-run-isolated-runner-registry-v1" | string;
  runtime_policy_version: string;
  eligible_authorizations: ApprovedHistoricalOutcomeDryRunRunAuthorization[];
  allowed_runner_kinds: HistoricalOutcomeDryRunIsolatedRunnerKind[];
  registration_allowed: boolean;
  current_runtime_artifact_sha256?: string;
  current_runtime_git_sha?: string;
  current_runtime_build_source: string;
  runners: HistoricalOutcomeDryRunIsolatedRunnerItem[];
  runner_count: number;
  current_binding_runner_count: number;
  execution_authorization_review_eligible_count: number;
  runner_status: string;
  invocation_authorized: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest = {
  dry_run_implementation_id: string;
  expected_run_authorization_review_id: string;
  expected_run_authorization_review_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
};

export type HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict =
  | "approved_for_one_shot_first_execution"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeDryRunFirstExecutionAuthorizationReview = {
  schema_version: "hone-historical-outcome-dry-run-first-execution-authorization-review-v1" | string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  isolated_runner_registered_by: string;
  run_authorization_review_id: string;
  run_authorization_review_sha256: string;
  dry_run_implementation_id: string;
  dry_run_implementation_spec_sha256: string;
  dry_run_implementation_code_revision: string;
  dry_run_implementation_kind: HistoricalOutcomeDryRunImplementationKind;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  labeler_implementation_id: string;
  labeler_implementation_spec_sha256: string;
  labeler_code_revision: string;
  protocol_version: string;
  protocol_sha256: string;
  runtime_policy_version: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  runner_status: string;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict;
  rationale: string;
  runner_spec_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  artifact_digest_independently_verified: boolean;
  artifact_reproducible_and_available_confirmed: boolean;
  sealed_inputs_and_root_read_only_confirmed: boolean;
  unprivileged_no_new_privileges_confirmed: boolean;
  ephemeral_output_and_validation_confirmed: boolean;
  resource_limits_confirmed: boolean;
  no_host_environment_or_secrets_confirmed: boolean;
  no_network_or_external_tools_confirmed: boolean;
  no_production_history_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
  single_use_and_expiry_confirmed: boolean;
  reviewer_independent_from_runner_registrant: boolean;
  one_shot_invocation_limit: number;
  one_shot_first_execution_authorized: boolean;
  authorization_consumed: boolean;
  invocation_endpoint_available: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunFirstExecutionAuthorizationItem = {
  runner: HistoricalOutcomeDryRunIsolatedRunnerRecord;
  current_binding: boolean;
  latest_review?: HistoricalOutcomeDryRunFirstExecutionAuthorizationReview;
  one_shot_first_execution_authorized: boolean;
  authorization_unexpired: boolean;
};

export type HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry = {
  schema_version: "hone-historical-outcome-dry-run-first-execution-authorization-registry-v1" | string;
  policy_version: string;
  items: HistoricalOutcomeDryRunFirstExecutionAuthorizationItem[];
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  one_shot_first_execution_authorized_count: number;
  unexpired_authorization_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  offline_dry_run_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_run_authorization_review_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  verdict: HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict;
  rationale: string;
  runner_spec_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  artifact_digest_independently_verified: boolean;
  artifact_reproducible_and_available_confirmed: boolean;
  sealed_inputs_and_root_read_only_confirmed: boolean;
  unprivileged_no_new_privileges_confirmed: boolean;
  ephemeral_output_and_validation_confirmed: boolean;
  resource_limits_confirmed: boolean;
  no_host_environment_or_secrets_confirmed: boolean;
  no_network_or_external_tools_confirmed: boolean;
  no_production_history_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
  single_use_and_expiry_confirmed: boolean;
};

export type HistoricalOutcomeDryRunMetric = {
  horizon_market_sessions: number;
  start_date: string;
  end_date: string;
  asset_return: number;
  benchmark_return: number;
  excess_return: number;
  asset_max_drawdown: number;
};

export type HistoricalOutcomeDryRunUntrustedOutput = {
  schema_version: string;
  snapshot_id: string;
  snapshot_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  asset_series_sha256: string;
  benchmark_series_sha256: string;
  common_session_count: number;
  metrics: HistoricalOutcomeDryRunMetric[];
  deterministic_replay_only: boolean;
  output_is_untrusted: boolean;
  outcome_label_written: boolean;
  training_target_written: boolean;
  reward_written: boolean;
  shadow_position_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeDryRunExecutionAttemptClaim = {
  schema_version: string;
  execution_policy_version: string;
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  runner_code_revision: string;
  dry_run_implementation_id: string;
  dry_run_implementation_spec_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  labeler_implementation_id: string;
  labeler_implementation_spec_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  claimed_at: string;
  invoked_by: string;
  isolation_backend: string;
  artifact_digest_reverified: boolean;
  sealed_snapshot_revalidated: boolean;
  authorization_consumed: boolean;
  invocation_started: boolean;
  child_process_spawned: boolean;
  ambient_filesystem_capability_available: boolean;
  ambient_environment_capability_available: boolean;
  network_capability_available: boolean;
  external_tool_capability_available: boolean;
  production_write_capability_available: boolean;
  historical_state_mutation_allowed: boolean;
  outcome_label_writes_allowed: boolean;
  training_writes_allowed: boolean;
  reward_writes_allowed: boolean;
  shadow_writes_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeDryRunExecutionAttemptResult = {
  schema_version: string;
  execution_policy_version: string;
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  completed_at: string;
  duration_millis: number;
  status: "completed_with_untrusted_output" | "failed_authorization_consumed";
  exit_code: number;
  stdout_sha256: string;
  stderr_sha256: string;
  stdout_bytes: number;
  stderr_bytes: number;
  output_sha256?: string;
  untrusted_output?: HistoricalOutcomeDryRunUntrustedOutput;
  ephemeral_directory_removed: boolean;
  output_structural_validation_completed: boolean;
  output_independent_validation_authorized: boolean;
  outcome_label_admission_authorized: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  attempt_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_output_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeDryRunExecutionAttemptClaim;
    result?: HistoricalOutcomeDryRunExecutionAttemptResult;
    current_authorization_binding: boolean;
  }>;
  output_independent_validation_authorized: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeDryRunRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
};

export type HistoricalOutcomeDryRunOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  execution_invoked_by: string;
  runner_registered_by: string;
  first_execution_authorization_reviewer_id: string;
  run_authorization_reviewer_id: string;
  validator_independent_from_execution_and_prior_reviewers: boolean;
  immutable_chain_integrity_verified: boolean;
  current_sealed_snapshot_binding_verified: boolean;
  canonical_output_hash_verified: boolean;
  output_structure_verified: boolean;
  deterministic_recomputation_match: boolean;
  recomputed_metrics: HistoricalOutcomeDryRunMetric[];
  mismatch_reasons: string[];
  verdict: "validated_deterministic_match" | "failed_structural_or_recomputation_mismatch";
  output_validated: boolean;
  outcome_label_admission_authorized: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeDryRunOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeDryRunExecutionAttemptClaim;
      result: HistoricalOutcomeDryRunExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeDryRunOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  validated_output_count: number;
  failed_validation_count: number;
  validation_status: string;
  output_validation_available: boolean;
  outcome_label_admission_authorized: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeDryRunOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
};

export type HistoricalOutcomeLabelAdmissionVerdict =
  | "approved_for_future_label_materialization"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeLabelAdmissionReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  validation_id: string;
  validation_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_by: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_horizons_market_sessions: number[];
  metric_start_date: string;
  metric_end_dates: string[];
  recomputed_metrics_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeLabelAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  reviewer_independent_from_validation_and_execution_chain: boolean;
  exact_validation_current_binding_confirmed: boolean;
  frozen_protocol_applicability_confirmed: boolean;
  complete_horizons_and_common_session_endpoints_confirmed: boolean;
  adjusted_close_and_corporate_action_basis_confirmed: boolean;
  benchmark_comparability_confirmed: boolean;
  event_time_and_future_isolation_confirmed: boolean;
  missingness_and_survivorship_bias_reviewed: boolean;
  no_manual_metric_override_confirmed: boolean;
  label_semantics_and_direction_not_inferred_confirmed: boolean;
  downstream_authority_remains_closed_confirmed: boolean;
  outcome_label_input_admitted: boolean;
  future_label_materialization_eligible: boolean;
  outcome_label_written: boolean;
  label_materialization_started: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    validation: HistoricalOutcomeDryRunOutputValidationRecord;
    asset_symbol: string;
    benchmark_symbol: string;
    decision_available_at: string;
    latest_review?: HistoricalOutcomeLabelAdmissionReview;
    current_binding: boolean;
    review_eligible: boolean;
    outcome_label_input_admitted: boolean;
  }>;
  independently_validated_output_count: number;
  review_eligible_output_count: number;
  reviewed_output_count: number;
  admitted_output_count: number;
  changes_requested_or_rejected_count: number;
  admission_status: string;
  outcome_label_input_admission_available: boolean;
  outcome_label_materialization_enabled: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeLabelAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  verdict: HistoricalOutcomeLabelAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_validation_current_binding_confirmed: boolean;
  frozen_protocol_applicability_confirmed: boolean;
  complete_horizons_and_common_session_endpoints_confirmed: boolean;
  adjusted_close_and_corporate_action_basis_confirmed: boolean;
  benchmark_comparability_confirmed: boolean;
  event_time_and_future_isolation_confirmed: boolean;
  missingness_and_survivorship_bias_reviewed: boolean;
  no_manual_metric_override_confirmed: boolean;
  label_semantics_and_direction_not_inferred_confirmed: boolean;
  downstream_authority_remains_closed_confirmed: boolean;
};

export type HistoricalOutcomeLabelMaterializationImplementationKind =
  "deterministic_raw_validated_outcome_envelope";

export type AdmittedHistoricalOutcomeProjection = {
  attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  validation_id: string;
  validation_sha256: string;
  output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  known_limitations: string;
};

export type HistoricalOutcomeLabelMaterializationImplementationRecord = {
  schema_version: string;
  materialization_policy_version: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  admission_review_id: string;
  admission_review_sha256: string;
  admission_reviewer_id: string;
  admission_known_limitations: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  validation_id: string;
  validation_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_horizons_market_sessions: number[];
  metric_start_date: string;
  metric_end_dates: string[];
  recomputed_metrics_sha256: string;
  output_label_schema_version: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind;
  code_revision: string;
  status: "registered_not_run";
  input_contract: string;
  output_contract: string;
  output_fields: string[];
  deterministic_projection_required: boolean;
  exact_metric_bit_preservation_required: boolean;
  create_once_output_required: boolean;
  isolated_output_required: boolean;
  known_limitations_preservation_required: boolean;
  missing_data_fail_closed_required: boolean;
  manual_metric_override_allowed: boolean;
  direction_inference_allowed: boolean;
  rating_inference_allowed: boolean;
  investment_action_inference_allowed: boolean;
  position_sizing_inference_allowed: boolean;
  reward_semantics_inference_allowed: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  label_materialization_run_authorized: boolean;
  outcome_label_write_allowed: boolean;
  label_materialization_enabled: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationImplementationRegistry = {
  schema_version: string;
  materialization_policy_version: string;
  output_label_schema_version: string;
  eligible_admissions: AdmittedHistoricalOutcomeProjection[];
  allowed_implementation_kinds: HistoricalOutcomeLabelMaterializationImplementationKind[];
  registration_allowed: boolean;
  implementations: Array<{
    implementation: HistoricalOutcomeLabelMaterializationImplementationRecord;
    admission_binding_current: boolean;
    run_authorization_review_eligible: boolean;
  }>;
  admitted_output_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  run_authorization_review_eligible_count: number;
  implementation_status: string;
  label_materialization_enabled: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeLabelMaterializationImplementationRequest = {
  attempt_id: string;
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind;
  code_revision: string;
};

export type HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict =
  | "approved_for_materialization_runner_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeLabelMaterializationRunAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  materialization_implementation_registered_by: string;
  implementation_name: string;
  implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind;
  code_revision: string;
  implementation_status: "registered_not_run";
  admission_review_id: string;
  admission_review_sha256: string;
  admission_reviewer_id: string;
  validation_id: string;
  validation_sha256: string;
  validated_by: string;
  execution_invoked_by: string;
  runner_registered_by: string;
  first_execution_authorization_reviewer_id: string;
  run_authorization_reviewer_id: string;
  output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  admission_known_limitations: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  code_revision_reproducible_confirmed: boolean;
  deterministic_raw_envelope_only_confirmed: boolean;
  exact_metric_bit_preservation_confirmed: boolean;
  provenance_and_limitations_preserved_confirmed: boolean;
  create_once_isolated_output_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
  no_network_tools_or_production_access_confirmed: boolean;
  no_semantic_action_position_or_reward_inference_confirmed: boolean;
  no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: boolean;
  reviewer_independent_from_implementation_and_prior_chain: boolean;
  materialization_runner_registration_eligible: boolean;
  materialization_runner_registered: boolean;
  label_materialization_run_authorized: boolean;
  label_materialization_started: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: HistoricalOutcomeLabelMaterializationImplementationRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeLabelMaterializationRunAuthorizationReview;
    materialization_runner_registration_eligible: boolean;
  }>;
  review_eligible_implementation_count: number;
  reviewed_implementation_count: number;
  materialization_runner_registration_eligible_count: number;
  authorization_status: string;
  materialization_runner_registered: boolean;
  label_materialization_run_authorized: boolean;
  label_materialization_started: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeLabelMaterializationIsolatedRunnerKind =
  "ephemeral_deterministic_process";

export type HistoricalOutcomeLabelMaterializationIsolatedRunnerRecord = {
  schema_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  materialization_run_authorization_review_id: string;
  materialization_run_authorization_review_sha256: string;
  materialization_run_authorization_reviewer_id: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  materialization_implementation_registered_by: string;
  materialization_policy_version: string;
  materialization_implementation_name: string;
  materialization_implementation_code_revision: string;
  materialization_implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind;
  admission_review_id: string;
  admission_review_sha256: string;
  admission_reviewer_id: string;
  admission_known_limitations: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  validation_id: string;
  validation_sha256: string;
  validated_by: string;
  execution_invoked_by: string;
  source_runner_registered_by: string;
  source_first_execution_authorization_reviewer_id: string;
  source_run_authorization_reviewer_id: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_horizons_market_sessions: number[];
  metric_start_date: string;
  metric_end_dates: string[];
  recomputed_metrics_sha256: string;
  output_label_schema_version: string;
  runtime_policy_version: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeLabelMaterializationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  status: "registered_not_run";
  callable_entrypoint_registered: boolean;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  invocation_authorized: boolean;
  label_materialization_run_authorized: boolean;
  label_materialization_started: boolean;
  output_artifact_created: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry = {
  schema_version: string;
  runtime_policy_version: string;
  eligible_authorizations: Array<{
    implementation: HistoricalOutcomeLabelMaterializationImplementationRecord;
    review: HistoricalOutcomeLabelMaterializationRunAuthorizationReview;
  }>;
  allowed_runner_kinds: HistoricalOutcomeLabelMaterializationIsolatedRunnerKind[];
  registration_allowed: boolean;
  current_runtime_artifact_sha256?: string;
  current_runtime_git_sha?: string;
  current_runtime_build_source: string;
  runners: Array<{
    runner: HistoricalOutcomeLabelMaterializationIsolatedRunnerRecord;
    run_authorization_binding_current: boolean;
    execution_authorization_review_eligible: boolean;
  }>;
  runner_count: number;
  current_binding_runner_count: number;
  execution_authorization_review_eligible_count: number;
  runner_status: string;
  invocation_authorized: boolean;
  label_materialization_run_authorized: boolean;
  label_materialization_started: boolean;
  output_artifact_created: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeLabelMaterializationIsolatedRunnerRequest = {
  materialization_implementation_id: string;
  expected_run_authorization_review_id: string;
  expected_run_authorization_review_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeLabelMaterializationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
};

export type HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_shot_first_execution"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  isolated_runner_registered_by: string;
  materialization_run_authorization_review_id: string;
  materialization_run_authorization_review_sha256: string;
  materialization_run_authorization_reviewer_id: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  materialization_implementation_registered_by: string;
  materialization_policy_version: string;
  materialization_implementation_name: string;
  materialization_implementation_code_revision: string;
  materialization_implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind;
  admission_review_id: string;
  admission_review_sha256: string;
  admission_reviewer_id: string;
  admission_known_limitations: string;
  validation_id: string;
  validation_sha256: string;
  output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  runtime_policy_version: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeLabelMaterializationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  runner_status: string;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict;
  rationale: string;
  runner_spec_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  artifact_digest_independently_verified: boolean;
  artifact_reproducible_and_available_confirmed: boolean;
  sealed_inputs_and_root_read_only_confirmed: boolean;
  unprivileged_no_new_privileges_confirmed: boolean;
  ephemeral_output_and_validation_confirmed: boolean;
  resource_limits_confirmed: boolean;
  no_host_environment_or_secrets_confirmed: boolean;
  no_network_external_tools_or_child_processes_confirmed: boolean;
  raw_envelope_only_no_semantic_inference_confirmed: boolean;
  no_production_history_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
  single_use_and_expiry_confirmed: boolean;
  reviewer_independent_from_runner_and_prior_chain: boolean;
  one_shot_invocation_limit: number;
  one_shot_first_execution_authorized: boolean;
  authorization_consumed: boolean;
  invocation_endpoint_available: boolean;
  label_materialization_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeLabelMaterializationIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationReview;
    one_shot_first_execution_authorized: boolean;
    authorization_unexpired: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  one_shot_first_execution_authorized_count: number;
  unexpired_authorization_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  label_materialization_enabled: boolean;
  execution_started: boolean;
  output_artifact_created: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_materialization_run_authorization_review_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
  verdict: HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict;
  rationale: string;
  runner_spec_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  artifact_digest_independently_verified: boolean;
  artifact_reproducible_and_available_confirmed: boolean;
  sealed_inputs_and_root_read_only_confirmed: boolean;
  unprivileged_no_new_privileges_confirmed: boolean;
  ephemeral_output_and_validation_confirmed: boolean;
  resource_limits_confirmed: boolean;
  no_host_environment_or_secrets_confirmed: boolean;
  no_network_external_tools_or_child_processes_confirmed: boolean;
  raw_envelope_only_no_semantic_inference_confirmed: boolean;
  no_production_history_label_training_reward_shadow_writes_confirmed: boolean;
  no_order_broker_or_trading_confirmed: boolean;
  single_use_and_expiry_confirmed: boolean;
};

export type HistoricalOutcomeLabelMaterializationExecutionAttemptStatus =
  | "completed_with_untrusted_envelope"
  | "failed_authorization_consumed";

export type HistoricalOutcomeLabelMaterializationUntrustedEnvelope = {
  schema_version: string;
  output_label_schema_version: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  validation_id: string;
  validation_sha256: string;
  source_attempt_id: string;
  source_claim_sha256: string;
  source_result_id: string;
  source_result_sha256: string;
  source_output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_start_date: string;
  metric_end_dates: string[];
  recomputed_metrics_sha256: string;
  raw_validated_metrics: HistoricalOutcomeDryRunMetric[];
  known_limitations: string;
  deterministic_projection_only: boolean;
  exact_metric_bits_preserved: boolean;
  provenance_preserved: boolean;
  known_limitations_preserved: boolean;
  output_is_untrusted: boolean;
  independent_validation_completed: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  direction_inferred: boolean;
  rating_inferred: boolean;
  investment_action_inferred: boolean;
  position_size_inferred: boolean;
  training_target_written: boolean;
  reward_written: boolean;
  shadow_position_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeLabelMaterializationExecutionAttemptClaim = {
  schema_version: string;
  execution_policy_version: string;
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  runner_code_revision: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  validation_id: string;
  validation_sha256: string;
  source_attempt_id: string;
  source_output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  max_wall_clock_seconds: number;
  max_memory_mib: number;
  max_cpu_millicores: number;
  max_process_count: number;
  max_output_bytes: number;
  claimed_at: string;
  invoked_by: string;
  isolation_backend: string;
  artifact_digest_reverified: boolean;
  current_admission_chain_revalidated: boolean;
  authorization_consumed: boolean;
  invocation_started: boolean;
  child_process_spawned: boolean;
  ambient_filesystem_capability_available: boolean;
  ambient_environment_capability_available: boolean;
  network_capability_available: boolean;
  external_tool_capability_available: boolean;
  production_data_capability_available_to_projection: boolean;
  historical_state_mutation_allowed: boolean;
  outcome_label_writes_allowed: boolean;
  training_writes_allowed: boolean;
  reward_writes_allowed: boolean;
  shadow_writes_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeLabelMaterializationExecutionAttemptResult = {
  schema_version: string;
  execution_policy_version: string;
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  completed_at: string;
  duration_millis: number;
  status: HistoricalOutcomeLabelMaterializationExecutionAttemptStatus;
  exit_code: number;
  stdout_sha256: string;
  stderr_sha256: string;
  stdout_bytes: number;
  stderr_bytes: number;
  output_sha256?: string;
  untrusted_envelope?: HistoricalOutcomeLabelMaterializationUntrustedEnvelope;
  ephemeral_directory_removed: boolean;
  independent_validation_completed: boolean;
  outcome_label_admission_authorized: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  attempt_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_envelope_count: number;
  independent_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeLabelMaterializationExecutionAttemptClaim;
    result?: HistoricalOutcomeLabelMaterializationExecutionAttemptResult;
    current_authorization_binding: boolean;
  }>;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeLabelMaterializationOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_spec_sha256: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
};

export type HistoricalOutcomeLabelMaterializationOutputValidationVerdict =
  | "validated_structure_provenance_and_bitwise_match"
  | "failed_structure_provenance_or_bitwise_mismatch";

export type HistoricalOutcomeLabelMaterializationOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  materialization_attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  materialization_implementation_id: string;
  materialization_implementation_spec_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  source_validation_id: string;
  source_validation_sha256: string;
  source_attempt_id: string;
  source_output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  materialization_invoked_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_materialization_and_prior_chain: boolean;
  immutable_chain_integrity_verified: boolean;
  current_admitted_source_binding_verified: boolean;
  canonical_output_hash_verified: boolean;
  output_structure_verified: boolean;
  provenance_match: boolean;
  exact_metric_bits_match: boolean;
  known_limitations_match: boolean;
  independently_validated_metrics: HistoricalOutcomeDryRunMetric[];
  mismatch_reasons: string[];
  verdict: HistoricalOutcomeLabelMaterializationOutputValidationVerdict;
  untrusted_envelope_validated: boolean;
  outcome_label_admission_authorized: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelMaterializationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeLabelMaterializationExecutionAttemptClaim;
      result: HistoricalOutcomeLabelMaterializationExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeLabelMaterializationOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  validated_envelope_count: number;
  failed_validation_count: number;
  validation_status: string;
  output_validation_available: boolean;
  outcome_label_generation_enabled: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeLabelMaterializationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
};

export type HistoricalOutcomeLabelWriteAuthorizationVerdict =
  | "approved_for_one_shot_formal_label_write"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeLabelWriteAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  materialization_validation_id: string;
  materialization_validation_sha256: string;
  materialization_validated_at: string;
  materialization_validated_by: string;
  materialization_attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  source_validation_id: string;
  source_validation_sha256: string;
  source_attempt_id: string;
  source_output_sha256: string;
  snapshot_id: string;
  snapshot_sha256: string;
  reconstruction_id: string;
  reconstruction_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_horizons_market_sessions: number[];
  metric_start_date: string;
  metric_end_dates: string[];
  known_limitations: string;
  formal_label_schema_version: string;
  formal_label_semantics_version: string;
  label_contract_sha256: string;
  allowed_label_fields: string[];
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeLabelWriteAuthorizationVerdict;
  rationale: string;
  one_shot_label_write_limit: number;
  one_shot_formal_label_write_authorized: boolean;
  authorization_consumed: boolean;
  label_writer_endpoint_available: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeLabelWriteAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  formal_label_schema_version: string;
  formal_label_semantics_version: string;
  label_contract_sha256: string;
  allowed_label_fields: string[];
  items: Array<{
    materialization_validation_id: string;
    materialization_validation_sha256: string;
    materialization_attempt_id: string;
    claim_sha256: string;
    result_sha256: string;
    output_sha256: string;
    admission_review_sha256: string;
    source_validation_sha256: string;
    source_output_sha256: string;
    snapshot_sha256: string;
    protocol_sha256: string;
    recomputed_metrics_sha256: string;
    asset_symbol: string;
    benchmark_symbol: string;
    decision_available_at: string;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeLabelWriteAuthorizationReview;
    review_eligible: boolean;
    one_shot_formal_label_write_authorized: boolean;
    authorization_consumed_by_formal_label_writer: boolean;
    authorization_unexpired: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  one_shot_authorized_count: number;
  unexpired_authorization_count: number;
  authorization_status: string;
  label_writer_endpoint_available: boolean;
  outcome_label_write_allowed: boolean;
  outcome_label_written: boolean;
  decision_training_authorized: boolean;
  reward_evidence_authorized: boolean;
  shadow_evidence_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeLabelWriteAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_materialization_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_admission_review_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
  expected_label_contract_sha256: string;
  verdict: HistoricalOutcomeLabelWriteAuthorizationVerdict;
  rationale: string;
  exact_validated_envelope_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  formal_label_schema_confirmed: boolean;
  raw_outcome_semantics_only_confirmed: boolean;
  exact_metric_bits_and_provenance_confirmed: boolean;
  known_limitations_preserved_confirmed: boolean;
  create_once_no_overwrite_writer_confirmed: boolean;
  single_use_and_expiry_confirmed: boolean;
  label_store_isolated_from_training_confirmed: boolean;
  no_semantic_inference_or_reward_confirmed: boolean;
  no_network_tools_or_unrelated_production_access_confirmed: boolean;
  no_training_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFormalLabelWriteClaim = {
  schema_version: string;
  writer_policy_version: string;
  writer_implementation_version: string;
  writer_implementation_sha256: string;
  claim_id: string;
  claim_sha256: string;
  target_label_id: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  materialization_validation_id: string;
  materialization_validation_sha256: string;
  materialization_claim_sha256: string;
  materialization_result_sha256: string;
  materialization_output_sha256: string;
  admission_review_sha256: string;
  source_validation_sha256: string;
  source_output_sha256: string;
  snapshot_sha256: string;
  reconstruction_sha256: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  formal_label_schema_version: string;
  formal_label_semantics_version: string;
  label_contract_sha256: string;
  claimed_at: string;
  invoked_by: string;
  authorization_consumed: boolean;
  create_once_no_overwrite: boolean;
  semantic_inference_allowed: boolean;
  training_write_allowed: boolean;
  reward_write_allowed: boolean;
  shadow_write_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeFormalLabel = {
  schema_version: string;
  semantics_version: string;
  writer_policy_version: string;
  writer_implementation_version: string;
  label_id: string;
  label_sha256: string;
  claim_id: string;
  claim_sha256: string;
  created_at: string;
  written_by: string;
  payload: {
    asset_symbol: string;
    benchmark_symbol: string;
    decision_available_at: string;
    common_session_count: number;
    raw_validated_metrics: HistoricalOutcomeDryRunMetric[];
    source_provenance: Record<string, string | string[]>;
    known_limitations: string;
    immutable_chain_bindings: Record<string, string>;
  };
  exact_metric_bits_preserved: boolean;
  provenance_preserved: boolean;
  known_limitations_preserved: boolean;
  formal_label_written: boolean;
  independently_validated_for_training_admission: boolean;
  admitted_to_offline_training_dataset_candidate: boolean;
  direction_inferred: boolean;
  rating_inferred: boolean;
  investment_action_inferred: boolean;
  position_size_inferred: boolean;
  training_target_written: boolean;
  reward_written: boolean;
  shadow_position_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeFormalLabelWriteFailure = {
  failure_id: string;
  failure_sha256: string;
  claim_id: string;
  claim_sha256: string;
  failed_at: string;
  error_message: string;
  error_sha256: string;
  authorization_consumed: boolean;
  formal_label_written: boolean;
};

export type HistoricalOutcomeFormalLabelWriteRegistry = {
  schema_version: string;
  writer_policy_version: string;
  writer_implementation_version: string;
  writer_implementation_sha256: string;
  formal_label_schema_version: string;
  formal_label_semantics_version: string;
  label_contract_sha256: string;
  allowed_label_fields: string[];
  writer_endpoint_available: boolean;
  eligible_authorization_count: number;
  claim_count: number;
  formal_label_count: number;
  failed_write_count: number;
  incomplete_fail_closed_claim_count: number;
  write_status: string;
  eligible_authorizations: Array<{
    authorization_review_id: string;
    authorization_review_sha256: string;
    authorization_valid_until: string;
    materialization_validation_id: string;
    materialization_validation_sha256: string;
    materialization_claim_sha256: string;
    materialization_result_sha256: string;
    materialization_output_sha256: string;
    admission_review_sha256: string;
    source_validation_sha256: string;
    source_output_sha256: string;
    snapshot_sha256: string;
    protocol_sha256: string;
    recomputed_metrics_sha256: string;
    label_contract_sha256: string;
    asset_symbol: string;
    benchmark_symbol: string;
    decision_available_at: string;
  }>;
  writes: Array<{
    claim: HistoricalOutcomeFormalLabelWriteClaim;
    label?: HistoricalOutcomeFormalLabel;
    failure?: HistoricalOutcomeFormalLabelWriteFailure;
    write_status: string;
  }>;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type WriteHistoricalOutcomeFormalLabelOnceRequest = {
  expected_authorization_review_sha256: string;
  expected_materialization_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_admission_review_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
  expected_label_contract_sha256: string;
};

export type HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  label_id: string;
  label_sha256: string;
  label_schema_version: string;
  label_semantics_version: string;
  label_contract_sha256: string;
  write_claim_id: string;
  write_claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  materialization_validation_sha256: string;
  materialization_output_sha256: string;
  admission_review_sha256: string;
  source_validation_sha256: string;
  source_output_sha256: string;
  snapshot_sha256: string;
  reconstruction_sha256: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  metric_horizons_market_sessions: number[];
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  formal_label_written_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_writer_and_complete_prior_chain: boolean;
  current_upstream_binding_verified: boolean;
  canonical_label_hash_verified: boolean;
  canonical_claim_hash_verified: boolean;
  fixed_eight_field_payload_verified: boolean;
  exact_metric_bits_verified: boolean;
  provenance_verified: boolean;
  known_limitations_verified: boolean;
  no_semantic_or_downstream_authority_verified: boolean;
  independently_validated_metrics: HistoricalOutcomeDryRunMetric[];
  mismatch_reasons: string[];
  verdict: "admitted_to_offline_training_dataset_candidate" | "failed_independent_validation";
  independently_validated_for_training_admission: boolean;
  admitted_to_offline_training_dataset_candidate: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  training_target_written: boolean;
  reward_authorized: boolean;
  reward_written: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFormalLabelValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validation_available: boolean;
  validation_eligible_count: number;
  validation_count: number;
  admitted_candidate_count: number;
  failed_validation_count: number;
  validation_status: string;
  items: Array<{
    formal_label: {
      claim: HistoricalOutcomeFormalLabelWriteClaim;
      label: HistoricalOutcomeFormalLabel;
    };
    validation?: HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord;
    validation_eligible: boolean;
  }>;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeFormalLabelRequest = {
  expected_label_sha256: string;
  expected_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_materialization_validation_sha256: string;
  expected_materialization_output_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  expected_recomputed_metrics_sha256: string;
  expected_label_contract_sha256: string;
};

export type HistoricalOutcomeOfflineDatasetCandidateBinding = {
  label_id: string;
  label_sha256: string;
  write_claim_id: string;
  write_claim_sha256: string;
  validation_id: string;
  validation_sha256: string;
};

export type HistoricalOutcomeOfflineDatasetEntry = HistoricalOutcomeOfflineDatasetCandidateBinding & {
  schema_version: string;
  ordinal: number;
  entry_id: string;
  entry_sha256: string;
  authorization_review_sha256: string;
  materialization_validation_sha256: string;
  materialization_output_sha256: string;
  admission_review_sha256: string;
  source_validation_sha256: string;
  source_output_sha256: string;
  snapshot_sha256: string;
  reconstruction_sha256: string;
  protocol_sha256: string;
  recomputed_metrics_sha256: string;
  label_contract_sha256: string;
  asset_symbol: string;
  benchmark_symbol: string;
  decision_available_at: string;
  common_session_count: number;
  raw_validated_metrics: HistoricalOutcomeDryRunMetric[];
  source_provenance: Record<string, string | string[]>;
  known_limitations: string;
  immutable_chain_bindings: Record<string, string>;
  formal_label_written_by: string;
  independently_validated_at: string;
  independently_validated_by: string;
  excluded_prior_actor_ids: string[];
  raw_outcome_only: boolean;
  feature_vector_present: boolean;
  semantic_target_assigned: boolean;
  split_assigned: boolean;
  reward_present: boolean;
};

export type HistoricalOutcomeOfflineDataset = {
  schema_version: string;
  policy_version: string;
  assembler_implementation_version: string;
  assembler_implementation_sha256: string;
  dataset_id: string;
  dataset_version: string;
  version_number: number;
  dataset_content_sha256: string;
  manifest_sha256: string;
  parent_dataset_id?: string;
  parent_manifest_sha256?: string;
  candidate_set_sha256: string;
  assembled_at: string;
  assembled_by: string;
  purpose: string;
  entry_count: number;
  added_entry_count: number;
  distinct_symbol_count: number;
  earliest_decision_available_at: string;
  latest_decision_available_at: string;
  entries: HistoricalOutcomeOfflineDatasetEntry[];
  complete_candidate_set_frozen: boolean;
  monotonic_append_only_lineage: boolean;
  point_in_time_lineage_preserved: boolean;
  duplicate_labels_rejected: boolean;
  conflicting_decision_identities_rejected: boolean;
  split_policy_status: string;
  copied_to_isolated_offline_dataset_store: boolean;
  copied_to_training_store: boolean;
  feature_join_performed: boolean;
  semantic_targets_assigned: boolean;
  dataset_governance_approved: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  reward_written: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetRegistry = {
  schema_version: string;
  policy_version: string;
  assembler_implementation_version: string;
  assembler_implementation_sha256: string;
  assembly_available: boolean;
  current_candidate_count: number;
  current_candidate_set_sha256: string;
  current_candidates: HistoricalOutcomeOfflineDatasetCandidateBinding[];
  dataset_count: number;
  current_binding_dataset_count: number;
  latest_dataset?: HistoricalOutcomeOfflineDataset;
  datasets: HistoricalOutcomeOfflineDataset[];
  assembly_status: string;
  copied_to_training_store: boolean;
  feature_join_performed: boolean;
  semantic_targets_assigned: boolean;
  dataset_governance_approved: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type AssembleHistoricalOutcomeOfflineDatasetRequest = {
  expected_candidate_set_sha256: string;
  expected_candidates: HistoricalOutcomeOfflineDatasetCandidateBinding[];
  purpose: "historical_raw_outcome_research_only";
  complete_current_candidate_set_confirmed: boolean;
  monotonic_version_lineage_confirmed: boolean;
  point_in_time_lineage_preserved_confirmed: boolean;
  no_semantic_target_or_split_inference_confirmed: boolean;
  no_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetGovernanceVerdict =
  | "approved_for_split_and_point_in_time_feature_join_spec_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeOfflineDatasetSplitPolicy = {
  policy_version: string;
  policy_sha256: string;
  connected_component_axes: string[];
  component_rule: string;
  deterministic_assignment_algorithm: string;
  train_percent: number;
  validation_percent: number;
  sealed_holdout_percent: number;
  temporal_order_required: boolean;
  max_outcome_horizon_market_sessions: number;
  purge_embargo_market_sessions: number;
  sealed_holdout_labels_withheld_from_training_worker: boolean;
  assignments_created_by_this_review: boolean;
};

export type HistoricalOutcomeOfflineDatasetFeatureJoinPolicy = {
  policy_version: string;
  policy_sha256: string;
  availability_rule: string;
  required_feature_provenance_fields: string[];
  forbidden_feature_namespaces: string[];
  missing_or_ambiguous_availability_policy: string;
  backfill_or_interpolation_allowed: boolean;
  immutable_feature_bundle_required: boolean;
  independent_feature_bundle_review_required: boolean;
  feature_join_performed_by_this_review: boolean;
};

export type HistoricalOutcomeOfflineDatasetGovernanceSubject = {
  dataset_id: string;
  dataset_version: string;
  version_number: number;
  dataset_content_sha256: string;
  manifest_sha256: string;
  candidate_set_sha256: string;
  entry_count: number;
  distinct_symbol_count: number;
  earliest_decision_available_at: string;
  latest_decision_available_at: string;
  assembled_at: string;
  assembled_by: string;
  complete_actor_ids: string[];
  distinct_reconstruction_count: number;
  distinct_snapshot_count: number;
  raw_outcome_only: boolean;
  split_assigned: boolean;
  feature_join_performed: boolean;
  semantic_targets_assigned: boolean;
};

export type HistoricalOutcomeOfflineDatasetGovernanceReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  subject: HistoricalOutcomeOfflineDatasetGovernanceSubject;
  split_policy: HistoricalOutcomeOfflineDatasetSplitPolicy;
  feature_join_policy: HistoricalOutcomeOfflineDatasetFeatureJoinPolicy;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_complete_dataset_chain: boolean;
  verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_dataset_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  complete_candidate_and_lineage_confirmed: boolean;
  company_event_source_component_isolation_confirmed: boolean;
  deterministic_split_and_sealed_holdout_confirmed: boolean;
  temporal_order_and_max_horizon_embargo_confirmed: boolean;
  point_in_time_feature_availability_confirmed: boolean;
  immutable_feature_provenance_confirmed: boolean;
  outcome_and_label_feature_exclusion_confirmed: boolean;
  missing_or_ambiguous_availability_fail_closed_confirmed: boolean;
  no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_transformation_spec_registration_eligible: boolean;
  split_assignment_authorized: boolean;
  split_assignment_performed: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetGovernanceRegistry = {
  schema_version: string;
  policy_version: string;
  split_policy: HistoricalOutcomeOfflineDatasetSplitPolicy;
  feature_join_policy: HistoricalOutcomeOfflineDatasetFeatureJoinPolicy;
  items: Array<{
    subject: HistoricalOutcomeOfflineDatasetGovernanceSubject;
    complete_review_actor_ids: string[];
    current_binding: boolean;
    latest_review?: HistoricalOutcomeOfflineDatasetGovernanceReview;
    review_eligible: boolean;
    future_transformation_spec_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  governance_status: string;
  split_assignment_authorized: boolean;
  split_assignment_performed: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_dataset_content_sha256: string;
  expected_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_split_policy_sha256: string;
  expected_feature_join_policy_sha256: string;
  verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_dataset_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  complete_candidate_and_lineage_confirmed: boolean;
  company_event_source_component_isolation_confirmed: boolean;
  deterministic_split_and_sealed_holdout_confirmed: boolean;
  temporal_order_and_max_horizon_embargo_confirmed: boolean;
  point_in_time_feature_availability_confirmed: boolean;
  immutable_feature_provenance_confirmed: boolean;
  outcome_and_label_feature_exclusion_confirmed: boolean;
  missing_or_ambiguous_availability_fail_closed_confirmed: boolean;
  no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type DeterministicSplitManifestSpecification = {
  schema_version: string;
  specification_sha256: string;
  governed_split_policy_version: string;
  governed_split_policy_sha256: string;
  component_identity_fields: string[];
  component_construction_algorithm: string;
  component_identity_algorithm: string;
  chronological_order_algorithm: string;
  boundary_assignment_algorithm: string;
  boundary_objective: string;
  minimum_partition_rule: string;
  market_session_calendar_rule: string;
  purge_embargo_algorithm: string;
  empty_partition_after_purge_policy: string;
  train_percent: number;
  validation_percent: number;
  sealed_holdout_percent: number;
  purge_embargo_market_sessions: number;
  max_outcome_horizon_market_sessions: number;
  output_manifest_fields: string[];
  sealed_holdout_labels_withheld_from_training_worker: boolean;
  content_addressed_output_required: boolean;
  create_once_output_required: boolean;
  split_assignments_generated: boolean;
};

export type PointInTimeFeatureDefinition = {
  namespace: string;
  feature_id: string;
  value_kind: string;
  source_authority_contract: string;
};

export type PointInTimeFeatureBundleSpecification = {
  schema_version: string;
  specification_sha256: string;
  governed_feature_join_policy_version: string;
  governed_feature_join_policy_sha256: string;
  join_key: string;
  allowed_feature_namespaces: string[];
  allowed_features: PointInTimeFeatureDefinition[];
  feature_id_must_be_allowlisted: boolean;
  namespace_cannot_override_feature_semantics: boolean;
  required_feature_record_fields: string[];
  availability_rule: string;
  observation_time_rule: string;
  forbidden_feature_namespaces: string[];
  missingness_values: string[];
  missing_or_ambiguous_availability_policy: string;
  artifact_revision_policy: string;
  qualitative_feature_review_policy: string;
  market_snapshot_policy: string;
  portfolio_snapshot_policy: string;
  output_bundle_fields: string[];
  backfill_allowed: boolean;
  interpolation_allowed: boolean;
  content_addressed_output_required: boolean;
  create_once_output_required: boolean;
  feature_bundle_generated: boolean;
  feature_join_performed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationSpecRecord = {
  schema_version: string;
  policy_version: string;
  transformation_spec_id: string;
  transformation_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  subject: HistoricalOutcomeOfflineDatasetGovernanceSubject;
  governance_review_id: string;
  governance_review_sha256: string;
  governance_reviewer_id: string;
  governance_known_limitations: string;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_dataset_and_governance_chain: boolean;
  specification_name: string;
  code_revision: string;
  rationale: string;
  known_limitations: string;
  split_manifest_specification: DeterministicSplitManifestSpecification;
  feature_bundle_specification: PointInTimeFeatureBundleSpecification;
  transformation_body_sha256: string;
  status: string;
  exact_dataset_and_governance_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  transitive_component_isolation_confirmed: boolean;
  chronological_boundaries_and_hash_tie_break_confirmed: boolean;
  purge_embargo_and_sealed_holdout_confirmed: boolean;
  point_in_time_availability_and_provenance_confirmed: boolean;
  seven_layer_namespace_allowlist_confirmed: boolean;
  label_outcome_and_future_information_exclusion_confirmed: boolean;
  missingness_fail_closed_without_imputation_confirmed: boolean;
  registration_review_execution_separation_confirmed: boolean;
  no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_independent_spec_review_eligible: boolean;
  independent_spec_review_completed: boolean;
  split_assignment_authorized: boolean;
  split_assignment_performed: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationSpecRegistry = {
  schema_version: string;
  policy_version: string;
  split_manifest_specification: DeterministicSplitManifestSpecification;
  feature_bundle_specification: PointInTimeFeatureBundleSpecification;
  eligible_subjects: Array<{
    subject: HistoricalOutcomeOfflineDatasetGovernanceSubject;
    governance_review_id: string;
    governance_review_sha256: string;
    governance_reviewer_id: string;
    split_policy_sha256: string;
    feature_join_policy_sha256: string;
  }>;
  items: Array<{
    specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord;
    upstream_binding_current: boolean;
    future_independent_spec_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  registered_count: number;
  current_binding_registered_count: number;
  independent_review_eligible_count: number;
  transformation_spec_status: string;
  split_assignment_authorized: boolean;
  split_assignment_performed: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest = {
  expected_dataset_content_sha256: string;
  expected_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_governance_review_id: string;
  expected_governance_review_sha256: string;
  expected_split_policy_sha256: string;
  expected_feature_join_policy_sha256: string;
  specification_name: string;
  code_revision: string;
  rationale: string;
  known_limitations: string;
  exact_dataset_and_governance_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  transitive_component_isolation_confirmed: boolean;
  chronological_boundaries_and_hash_tie_break_confirmed: boolean;
  purge_embargo_and_sealed_holdout_confirmed: boolean;
  point_in_time_availability_and_provenance_confirmed: boolean;
  seven_layer_namespace_allowlist_confirmed: boolean;
  label_outcome_and_future_information_exclusion_confirmed: boolean;
  missingness_fail_closed_without_imputation_confirmed: boolean;
  registration_review_execution_separation_confirmed: boolean;
  no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict =
  | "approved_for_future_isolated_transformation_implementation_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract = {
  schema_version: string;
  contract_sha256: string;
  semantic_audit_implementation: string;
  required_split_checks: string[];
  required_feature_checks: string[];
  approval_scope: string;
  implementation_registration_separate: boolean;
  transformation_execution_separate: boolean;
  output_validation_separate: boolean;
  target_definition_separate: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationSpecReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord;
  review_contract: HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_complete_registration_chain: boolean;
  verdict: HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_specification_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  independent_hash_and_schema_reproduction_confirmed: boolean;
  transitive_component_identity_and_indivisibility_confirmed: boolean;
  chronological_contiguous_boundary_objective_confirmed: boolean;
  equal_time_hash_tie_break_only_confirmed: boolean;
  market_session_purge_embargo_and_empty_partition_failure_confirmed: boolean;
  sealed_holdout_label_isolation_confirmed: boolean;
  exact_seven_layer_feature_id_allowlist_confirmed: boolean;
  point_in_time_artifact_and_revision_provenance_confirmed: boolean;
  qualitative_market_and_portfolio_source_contracts_confirmed: boolean;
  explicit_missingness_without_backfill_or_interpolation_confirmed: boolean;
  outcome_label_future_and_namespace_smuggling_exclusion_confirmed: boolean;
  content_addressed_create_once_outputs_and_later_validation_confirmed: boolean;
  review_implementation_execution_target_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_isolated_transformation_implementation_registration_eligible: boolean;
  transformation_implementation_registered: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_contract: HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract;
  items: Array<{
    specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord;
    complete_review_actor_ids: string[];
    upstream_binding_current: boolean;
    latest_review?: HistoricalOutcomeOfflineDatasetTransformationSpecReview;
    review_eligible: boolean;
    future_isolated_transformation_implementation_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  implementation_registration_eligible_count: number;
  review_status: string;
  transformation_implementation_registered: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_transformation_spec_sha256: string;
  expected_transformation_body_sha256: string;
  expected_dataset_content_sha256: string;
  expected_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_governance_review_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  expected_review_contract_sha256: string;
  verdict: HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_specification_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  independent_hash_and_schema_reproduction_confirmed: boolean;
  transitive_component_identity_and_indivisibility_confirmed: boolean;
  chronological_contiguous_boundary_objective_confirmed: boolean;
  equal_time_hash_tie_break_only_confirmed: boolean;
  market_session_purge_embargo_and_empty_partition_failure_confirmed: boolean;
  sealed_holdout_label_isolation_confirmed: boolean;
  exact_seven_layer_feature_id_allowlist_confirmed: boolean;
  point_in_time_artifact_and_revision_provenance_confirmed: boolean;
  qualitative_market_and_portfolio_source_contracts_confirmed: boolean;
  explicit_missingness_without_backfill_or_interpolation_confirmed: boolean;
  outcome_label_future_and_namespace_smuggling_exclusion_confirmed: boolean;
  content_addressed_create_once_outputs_and_later_validation_confirmed: boolean;
  review_implementation_execution_target_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationContract = {
  schema_version: string;
  contract_sha256: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  split_implementation_id: string;
  split_implementation_version: string;
  feature_implementation_id: string;
  feature_implementation_version: string;
  canonical_serializer_version: string;
  input_schema_version: string;
  output_schema_version: string;
  input_contract: string;
  output_contract: string;
  maximum_parallel_subjects: number;
  maximum_memory_mebibytes: number;
  callable_entrypoint_present: boolean;
  environment_inheritance_allowed: boolean;
  environment_variables_allowed: boolean;
  secrets_allowed: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  approved_review: HistoricalOutcomeOfflineDatasetTransformationSpecReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_review_chain: boolean;
  implementation_name: string;
  rationale: string;
  known_limitations: string;
  implementation_contract: HistoricalOutcomeOfflineDatasetTransformationImplementationContract;
  status: string;
  exact_approved_review_and_specification_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  implementation_artifact_and_code_revision_immutable_confirmed: boolean;
  deterministic_split_and_feature_implementation_confirmed: boolean;
  canonical_serialization_and_fixed_schema_confirmed: boolean;
  sealed_read_only_input_and_create_once_output_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: boolean;
  registration_review_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: HistoricalOutcomeOfflineDatasetTransformationSpecReview[];
  items: Array<{
    implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest = {
  expected_review_id: string;
  expected_review_sha256: string;
  expected_review_contract_sha256: string;
  expected_transformation_spec_id: string;
  expected_transformation_spec_sha256: string;
  expected_transformation_body_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  expected_dataset_content_sha256: string;
  expected_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_governance_review_id: string;
  expected_governance_review_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_approved_review_and_specification_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  implementation_artifact_and_code_revision_immutable_confirmed: boolean;
  deterministic_split_and_feature_implementation_confirmed: boolean;
  canonical_serialization_and_fixed_schema_confirmed: boolean;
  sealed_read_only_input_and_create_once_output_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: boolean;
  registration_review_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict =
  | "approved_for_future_isolated_transformation_runner_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract = {
  schema_version: string;
  contract_sha256: string;
  independent_audit_implementation: string;
  required_artifact_checks: string[];
  required_sandbox_checks: string[];
  approval_scope: string;
  runner_registration_separate: boolean;
  execution_authorization_separate: boolean;
  transformation_execution_separate: boolean;
  output_validation_separate: boolean;
  target_definition_separate: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord;
  review_contract: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_complete_registration_chain: boolean;
  verdict: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_implementation_and_upstream_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  artifact_digest_independently_reproduced_confirmed: boolean;
  immutable_code_revision_reproducible_confirmed: boolean;
  deterministic_split_implementation_matches_specification_confirmed: boolean;
  exact_65_feature_implementation_matches_allowlist_confirmed: boolean;
  canonical_serializer_and_schema_determinism_confirmed: boolean;
  sealed_read_only_input_and_create_once_output_contract_confirmed: boolean;
  bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  review_runner_execution_output_target_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_isolated_transformation_runner_registration_eligible: boolean;
  transformation_runner_registered: boolean;
  transformation_execution_authorized: boolean;
  transformation_execution_started: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_contract: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract;
  items: Array<{
    implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord;
    complete_review_actor_ids: string[];
    upstream_binding_current: boolean;
    latest_review?: HistoricalOutcomeOfflineDatasetTransformationImplementationReview;
    review_eligible: boolean;
    future_isolated_transformation_runner_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  runner_registration_eligible_count: number;
  review_status: string;
  transformation_runner_registered: boolean;
  transformation_execution_authorized: boolean;
  transformation_execution_started: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_specification_review_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_transformation_body_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  expected_dataset_content_sha256: string;
  expected_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_governance_review_sha256: string;
  expected_review_contract_sha256: string;
  verdict: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_implementation_and_upstream_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  artifact_digest_independently_reproduced_confirmed: boolean;
  immutable_code_revision_reproducible_confirmed: boolean;
  deterministic_split_implementation_matches_specification_confirmed: boolean;
  exact_65_feature_implementation_matches_allowlist_confirmed: boolean;
  canonical_serializer_and_schema_determinism_confirmed: boolean;
  sealed_read_only_input_and_create_once_output_contract_confirmed: boolean;
  bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  review_runner_execution_output_target_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerKind =
  "ephemeral_deterministic_process";

export type HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerContract = {
  schema_version: string;
  contract_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  input_mount_contract: string;
  output_contract: string;
  invocation_contract: string;
  next_gate: string;
  callable_entrypoint_registered: boolean;
  input_mount_read_only_required: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  content_addressed_create_once_output_required: boolean;
  independent_output_validation_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  host_environment_inherited: boolean;
  allowed_environment_variables: string[];
  secrets_available: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  maximum_parallel_subjects: number;
  maximum_memory_mebibytes: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord;
  implementation_review: HistoricalOutcomeOfflineDatasetTransformationImplementationReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_approval_chain: boolean;
  runner_name: string;
  runner_kind: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  runner_contract: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerContract;
  status: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  sealed_read_only_input_and_content_addressed_create_once_output_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  transformation_execution_started: boolean;
  output_artifact_created: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: Array<{
    implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord;
    review: HistoricalOutcomeOfflineDatasetTransformationImplementationReview;
  }>;
  allowed_runner_kinds: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerKind[];
  registration_allowed: boolean;
  items: Array<{
    runner: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  runner_status: string;
  callable_entrypoint_registered: boolean;
  first_execution_authorized: boolean;
  transformation_execution_started: boolean;
  output_artifact_created: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_transformation_spec_sha256: string;
  expected_transformation_body_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  expected_dataset_content_sha256: string;
  expected_governance_review_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  sealed_read_only_input_and_content_addressed_create_once_output_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_transformation_invocation"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  sealed_read_only_inputs_and_root_filesystem_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  deterministic_split_feature_and_canonical_schema_contract_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  one_shot_invocation_limit: number;
  one_future_isolated_transformation_invocation_authorized: boolean;
  authorization_claimed: boolean;
  invocation_endpoint_available: boolean;
  transformation_execution_started: boolean;
  output_artifact_created: boolean;
  output_validation_authorized: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationReview;
    one_future_isolated_transformation_invocation_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  transformation_execution_started: boolean;
  output_artifact_created: boolean;
  output_validation_authorized: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus =
  | "completed_with_untrusted_candidate_envelope"
  | "failed_authorization_consumed";

export type HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim = {
  schema_version: string;
  execution_policy_version: string;
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  runner_code_revision: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_review_id: string;
  implementation_review_sha256: string;
  transformation_spec_id: string;
  transformation_spec_sha256: string;
  transformation_body_sha256: string;
  split_specification_sha256: string;
  feature_specification_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  governance_review_id: string;
  governance_review_sha256: string;
  claimed_at: string;
  invoked_by: string;
  authorization_consumed: boolean;
  current_complete_upstream_chain_revalidated: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope = {
  schema_version: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  transformation_spec_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  runner_id: string;
  runner_spec_sha256: string;
  authorization_review_id: string;
  entry_count: number;
  component_count: number;
  feature_catalog_count: number;
  feature_catalog_sha256: string;
  feature_schema_sha256: string;
  boundary_audit: {
    candidate_pair_count: number;
    selected_train_component_end_exclusive: number;
    selected_validation_component_end_exclusive: number;
    pre_purge_train_entry_count: number;
    pre_purge_validation_entry_count: number;
    pre_purge_sealed_holdout_entry_count: number;
    objective_tuple: number[];
    audit_sha256: string;
  };
  split_manifest_candidate: Array<{
    dataset_entry_id: string;
    component_id: string;
    split: "train" | "validation" | "sealed_holdout";
    purged_or_embargoed: boolean;
    purge_reason?: string;
  }>;
  feature_bundle_candidate: Array<{
    dataset_entry_id: string;
    feature_id: string;
    feature_namespace: string;
    value?: string;
    is_missing: boolean;
    missingness_reason: string;
    source_identity: string;
  }>;
  sealed_holdout_labels_withheld: boolean;
  deterministic_projection_only: boolean;
  explicit_missingness_preserved: boolean;
  output_is_untrusted: boolean;
  independent_validation_completed: boolean;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_started: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult = {
  schema_version: string;
  execution_policy_version: string;
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  completed_at: string;
  duration_millis: number;
  status: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus;
  exit_code: number;
  bounded_error?: string;
  output_sha256?: string;
  untrusted_candidate_envelope?: HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope;
  ephemeral_directory_removed: boolean;
  independent_validation_completed: boolean;
  official_split_manifest_authorized: boolean;
  official_feature_bundle_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  eligible_authorizations: Array<{
    runner: HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRecord;
    review: HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationReview;
  }>;
  attempt_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_candidate_envelope_count: number;
  independent_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim;
    result?: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult;
    current_authorization_binding: boolean;
  }>;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_transformation_body_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  expected_dataset_id: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
};

export type HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict =
  | "validated_independent_structure_and_deterministic_match"
  | "failed_structure_or_independent_recomputation_mismatch";

export type HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  transformation_attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  transformation_spec_sha256: string;
  split_specification_sha256: string;
  feature_specification_sha256: string;
  validated_at: string;
  validated_by: string;
  execution_invoked_by: string;
  runner_registered_by: string;
  authorization_reviewer_id: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_execution_and_complete_prior_chain: boolean;
  immutable_chain_integrity_verified: boolean;
  current_dataset_binding_verified: boolean;
  current_sealed_snapshot_bindings_verified: boolean;
  canonical_output_hash_verified: boolean;
  output_structure_verified: boolean;
  independent_component_recomputation_match: boolean;
  independent_boundary_recomputation_match: boolean;
  independent_purge_embargo_recomputation_match: boolean;
  independent_feature_recomputation_match: boolean;
  sealed_holdout_withholding_verified: boolean;
  mismatch_reasons: string[];
  verdict: HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict;
  untrusted_candidate_envelope_validated: boolean;
  official_split_manifest_authorized: boolean;
  official_feature_bundle_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim;
      result: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  validated_candidate_envelope_count: number;
  failed_validation_count: number;
  validation_status: string;
  output_validation_available: boolean;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
};

export type HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict =
  | "approved_for_future_create_once_official_artifact_materialization"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  transformation_attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  validation_id: string;
  validation_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  transformation_spec_sha256: string;
  split_specification_sha256: string;
  feature_specification_sha256: string;
  recomputed_boundary_audit_sha256: string;
  recomputed_split_manifest_candidate_sha256: string;
  recomputed_feature_bundle_candidate_sha256: string;
  recomputed_exclusion_audit_sha256: string;
  entry_count: number;
  component_count: number;
  feature_catalog_count: number;
  split_record_count: number;
  feature_record_count: number;
  exclusion_audit_record_count: number;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  reviewer_independent_from_validation_execution_and_complete_prior_chain: boolean;
  transformation_candidate_admitted: boolean;
  future_create_once_official_artifact_materialization_eligible: boolean;
  official_artifact_materialization_started: boolean;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      attempt: {
        claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim;
        result: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult;
      };
      validation: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord;
    };
    latest_review?: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview;
    current_binding: boolean;
    review_eligible: boolean;
    transformation_candidate_admitted: boolean;
  }>;
  independently_validated_candidate_count: number;
  review_eligible_candidate_count: number;
  reviewed_candidate_count: number;
  admitted_candidate_count: number;
  changes_requested_or_rejected_count: number;
  admission_status: string;
  candidate_admission_review_available: boolean;
  official_artifact_materialization_enabled: boolean;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  verdict: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_candidate_and_validation_chain_confirmed: boolean;
  transitive_component_isolation_confirmed: boolean;
  deterministic_chronological_boundary_and_full_objective_audit_confirmed: boolean;
  purge_embargo_and_non_empty_partitions_confirmed: boolean;
  sealed_holdout_labels_withheld_confirmed: boolean;
  point_in_time_feature_allowlist_and_provenance_confirmed: boolean;
  explicit_missingness_without_imputation_confirmed: boolean;
  outcome_future_and_current_portfolio_exclusion_confirmed: boolean;
  official_artifact_contract_and_create_once_scope_confirmed: boolean;
  admission_materialization_and_output_validation_separation_confirmed: boolean;
  downstream_authority_remains_closed_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus =
  | "completed_pending_independent_validation"
  | "failed_claim_consumed";

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim = {
  schema_version: string;
  policy_version: string;
  materialization_id: string;
  claim_sha256: string;
  transformation_attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  validation_id: string;
  validation_sha256: string;
  source_output_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  transformation_spec_sha256: string;
  split_specification_sha256: string;
  feature_specification_sha256: string;
  materialized_by: string;
  claimed_at: string;
  claim_consumed: boolean;
  official_artifact_materialization_started: boolean;
  feature_join_allowed: boolean;
  semantic_target_assignment_allowed: boolean;
  training_allowed: boolean;
  reward_allowed: boolean;
  shadow_portfolio_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult = {
  schema_version: string;
  policy_version: string;
  result_id: string;
  result_sha256: string;
  materialization_id: string;
  claim_sha256: string;
  completed_at: string;
  status: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus;
  error?: string;
  split_manifest_sha256?: string;
  feature_bundle_sha256?: string;
  combined_artifact_sha256?: string;
  total_artifact_bytes: number;
  official_split_manifest_created: boolean;
  official_feature_bundle_created: boolean;
  exact_validated_candidate_copy_completed: boolean;
  independent_output_validation_completed: boolean;
  official_artifacts_eligible_for_feature_join: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    admitted_candidate: {
      candidate: {
        attempt: {
          claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim;
          result: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult;
        };
        validation: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord;
      };
      admission_review: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview;
    };
    attempt?: {
      claim: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim;
      result?: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult;
      split_manifest?: {
        manifest_sha256: string;
        entry_count: number;
        component_count: number;
        independently_validated_after_materialization: boolean;
        eligible_for_feature_join: boolean;
      };
      feature_bundle?: {
        feature_bundle_sha256: string;
        feature_catalog_count: number;
        records: unknown[];
        independently_validated_after_materialization: boolean;
        joined_to_outcome_labels: boolean;
      };
    };
    materialization_eligible: boolean;
    official_artifacts_created_pending_independent_validation: boolean;
  }>;
  admitted_candidate_count: number;
  materialization_eligible_candidate_count: number;
  claimed_candidate_count: number;
  completed_materialization_count: number;
  failed_or_incomplete_materialization_count: number;
  unvalidated_official_artifact_pair_count: number;
  materialization_status: string;
  official_artifact_materialization_enabled: boolean;
  independent_official_artifact_validation_enabled: boolean;
  feature_join_enabled: boolean;
  semantic_target_assignment_enabled: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest = {
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_split_specification_sha256: string;
  expected_feature_specification_sha256: string;
  exact_copy_only_confirmed: boolean;
  create_once_and_failure_consumes_confirmed: boolean;
  no_join_target_training_or_trading_confirmed: boolean;
  independent_output_validation_required_confirmed: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  transformation_attempt_id: string;
  materialization_id: string;
  materialization_claim_sha256: string;
  materialization_result_id: string;
  materialization_result_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  source_validation_id: string;
  source_validation_sha256: string;
  source_output_sha256: string;
  split_manifest_sha256: string;
  feature_bundle_sha256: string;
  combined_artifact_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  materialized_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_materializer_and_complete_prior_chain: boolean;
  exact_current_admission_and_source_candidate_verified: boolean;
  materialization_claim_fingerprint_verified: boolean;
  materialization_result_fingerprint_verified: boolean;
  split_manifest_fingerprint_verified: boolean;
  feature_bundle_fingerprint_verified: boolean;
  combined_artifact_fingerprint_verified: boolean;
  exact_split_candidate_copy_verified: boolean;
  exact_feature_candidate_copy_verified: boolean;
  sealed_holdout_withholding_verified: boolean;
  explicit_missingness_and_exclusion_verified: boolean;
  downstream_authority_closed_verified: boolean;
  mismatch_reasons: string[];
  verdict:
    | "validated_exact_official_artifact_pair"
    | "failed_official_artifact_structure_or_binding_mismatch";
  official_artifact_pair_independently_validated: boolean;
  future_feature_label_join_specification_registration_eligible: boolean;
  feature_join_performed: boolean;
  semantic_target_assigned: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    artifact_pair: {
      admitted_candidate: {
        candidate: {
          attempt: {
            claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim;
            result: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult;
          };
          validation: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord;
        };
        admission_review: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview;
      };
      claim: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim;
      result: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult;
      split_manifest: {
        manifest_sha256: string;
        entry_count: number;
        component_count: number;
        independently_validated_after_materialization: boolean;
        eligible_for_feature_join: boolean;
      };
      feature_bundle: {
        feature_bundle_sha256: string;
        feature_catalog_count: number;
        records: unknown[];
        independently_validated_after_materialization: boolean;
        joined_to_outcome_labels: boolean;
      };
    };
    validation?: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_artifact_pair_count: number;
  failed_validation_count: number;
  validation_status: string;
  independent_official_artifact_validation_enabled: boolean;
  future_join_specification_registration_enabled: boolean;
  feature_join_enabled: boolean;
  semantic_target_assignment_enabled: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest = {
  expected_materialization_id: string;
  expected_materialization_claim_sha256: string;
  expected_materialization_result_sha256: string;
  expected_admission_review_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  exact_artifact_pair_binding_confirmed: boolean;
  independent_validator_confirmed: boolean;
  no_join_target_training_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinSpecification = {
  schema_version: string;
  specification_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  split_manifest_sha256: string;
  feature_bundle_sha256: string;
  combined_artifact_sha256: string;
  dataset_entry_key: string;
  split_record_key: string;
  feature_record_key_fields: string[];
  raw_outcome_record_key: string;
  join_cardinality_rule: string;
  split_authority_rule: string;
  purged_or_embargoed_row_policy: string;
  train_target_visibility_policy: string;
  validation_target_visibility_policy: string;
  sealed_holdout_target_visibility_policy: string;
  feature_availability_rule: string;
  explicit_missingness_rule: string;
  feature_catalog_count: number;
  feature_catalog_sha256: string;
  feature_schema_sha256: string;
  allowed_label_horizons_market_sessions: number[];
  forbidden_join_inputs: string[];
  joined_row_schema_fields: string[];
  one_to_one_outcome_join_required: boolean;
  all_allowlisted_feature_records_preserved: boolean;
  imputation_allowed: boolean;
  interpolation_allowed: boolean;
  sealed_holdout_labels_opened: boolean;
  join_executed: boolean;
};

export type HistoricalOutcomeSemanticTargetDefinition = {
  target_id: string;
  horizon_market_sessions: number;
  source_metric_field: string;
  source_selector: string;
  value_kind: string;
  unit: string;
  transformation: string;
  role: string;
  semantics: string;
};

export type HistoricalOutcomeSemanticTargetSpecification = {
  schema_version: string;
  specification_sha256: string;
  prediction_task: string;
  target_definitions: HistoricalOutcomeSemanticTargetDefinition[];
  primary_supervised_target_id: string;
  risk_target_id: string;
  auxiliary_target_ids: string[];
  benchmark_return_role: string;
  target_vector_order: string[];
  duplicate_horizon_policy: string;
  missing_horizon_policy: string;
  train_target_access_policy: string;
  validation_target_access_policy: string;
  sealed_holdout_target_access_policy: string;
  exact_f64_bits_preserved: boolean;
  normalization_allowed: boolean;
  winsorization_allowed: boolean;
  rank_transform_allowed: boolean;
  categorical_action_label_defined: boolean;
  buy_hold_sell_threshold_defined: boolean;
  portfolio_weight_target_defined: boolean;
  scalar_reward_defined: boolean;
  semantic_target_assignment_performed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetSpecRecord = {
  schema_version: string;
  policy_version: string;
  specification_id: string;
  specification_sha256: string;
  transformation_attempt_id: string;
  validation_id: string;
  validation_sha256: string;
  materialization_id: string;
  materialization_claim_sha256: string;
  materialization_result_sha256: string;
  split_manifest_sha256: string;
  feature_bundle_sha256: string;
  combined_artifact_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  registered_at: string;
  registered_by: string;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_prior_chain: boolean;
  specification_name: string;
  code_revision: string;
  rationale: string;
  known_limitations: string;
  join_specification: HistoricalOutcomeFeatureLabelJoinSpecification;
  target_specification: HistoricalOutcomeSemanticTargetSpecification;
  specification_body_sha256: string;
  status: string;
  exact_validated_artifact_pair_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  exact_dataset_entry_one_to_one_join_confirmed: boolean;
  purged_and_embargoed_rows_excluded_confirmed: boolean;
  point_in_time_feature_availability_confirmed: boolean;
  sealed_holdout_target_isolation_confirmed: boolean;
  exact_raw_metric_bits_without_transform_confirmed: boolean;
  continuous_target_vector_not_action_or_reward_confirmed: boolean;
  explicit_missingness_without_imputation_confirmed: boolean;
  registration_review_execution_separation_confirmed: boolean;
  no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_independent_spec_review_eligible: boolean;
  independent_spec_review_completed: boolean;
  join_execution_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry = {
  schema_version: string;
  policy_version: string;
  subjects: Array<{
    transformation_attempt_id: string;
    validation_id: string;
    validation_sha256: string;
    materialization_id: string;
    materialization_claim_sha256: string;
    materialization_result_sha256: string;
    split_manifest_sha256: string;
    feature_bundle_sha256: string;
    combined_artifact_sha256: string;
    dataset_id: string;
    dataset_content_sha256: string;
    dataset_manifest_sha256: string;
    candidate_set_sha256: string;
    feature_catalog_count: number;
    feature_catalog_sha256: string;
    feature_schema_sha256: string;
    registered_specification?: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord;
    registration_eligible: boolean;
  }>;
  registration_eligible_count: number;
  specification_count: number;
  current_binding_specification_count: number;
  stale_or_mismatched_specification_count: number;
  independent_review_eligible_count: number;
  registration_status: string;
  registration_enabled: boolean;
  independent_review_enabled: boolean;
  join_execution_enabled: boolean;
  semantic_target_assignment_enabled: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest = {
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_materialization_id: string;
  expected_materialization_claim_sha256: string;
  expected_materialization_result_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_id: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  specification_name: string;
  code_revision: string;
  rationale: string;
  known_limitations: string;
  exact_validated_artifact_pair_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  exact_dataset_entry_one_to_one_join_confirmed: boolean;
  purged_and_embargoed_rows_excluded_confirmed: boolean;
  point_in_time_feature_availability_confirmed: boolean;
  sealed_holdout_target_isolation_confirmed: boolean;
  exact_raw_metric_bits_without_transform_confirmed: boolean;
  continuous_target_vector_not_action_or_reward_confirmed: boolean;
  explicit_missingness_without_imputation_confirmed: boolean;
  registration_review_execution_separation_confirmed: boolean;
  no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict =
  | "approved_for_future_isolated_join_target_implementation_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract = {
  schema_version: string;
  contract_sha256: string;
  semantic_audit_implementation: string;
  required_join_checks: string[];
  required_target_checks: string[];
  approval_scope: string;
  primary_target_is_engineering_candidate_not_strategy_truth: boolean;
  implementation_registration_separate: boolean;
  join_execution_separate: boolean;
  output_validation_separate: boolean;
  training_and_reward_governance_separate: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  specification_id: string;
  specification_sha256: string;
  specification_body_sha256: string;
  join_specification_sha256: string;
  target_specification_sha256: string;
  combined_artifact_sha256: string;
  record_hash_independently_reproduced: boolean;
  specification_body_hash_independently_reproduced: boolean;
  join_hash_independently_reproduced: boolean;
  target_hash_independently_reproduced: boolean;
  exact_current_artifact_binding_reproduced: boolean;
  exact_feature_catalog_binding_reproduced: boolean;
  join_cardinality_and_split_semantics_valid: boolean;
  point_in_time_and_missingness_semantics_valid: boolean;
  forbidden_input_and_holdout_isolation_valid: boolean;
  exact_nine_continuous_target_semantics_valid: boolean;
  primary_and_risk_roles_are_explicit_engineering_candidates: boolean;
  no_action_position_threshold_ranking_or_reward_semantics: boolean;
  all_execution_and_downstream_authority_closed: boolean;
  target_ids: string[];
  mismatch_reasons: string[];
};

export type HistoricalOutcomeFeatureLabelJoinTargetSpecReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  specification: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord;
  review_contract: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract;
  independent_audit: HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_complete_prior_chain: boolean;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_specification_and_artifact_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  independent_record_join_target_hash_reproduction_confirmed: boolean;
  one_to_one_entry_join_and_duplicate_missing_failure_confirmed: boolean;
  purge_embargo_exclusion_and_official_split_authority_confirmed: boolean;
  point_in_time_feature_and_explicit_missingness_confirmed: boolean;
  forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: boolean;
  split_specific_target_visibility_and_sealed_holdout_confirmed: boolean;
  exact_nine_continuous_target_semantics_confirmed: boolean;
  primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: boolean;
  exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: boolean;
  review_implementation_execution_and_output_validation_separation_confirmed: boolean;
  no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_isolated_join_target_implementation_registration_eligible: boolean;
  join_target_implementation_registered: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_contract: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract;
  items: Array<{
    specification: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord;
    current_independent_audit: HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit;
    complete_review_actor_ids: string[];
    upstream_binding_current: boolean;
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetSpecReview;
    review_eligible: boolean;
    future_isolated_join_target_implementation_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  implementation_registration_eligible_count: number;
  review_status: string;
  join_target_implementation_registered: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_validation_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_review_contract_sha256: string;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_specification_and_artifact_binding_confirmed: boolean;
  reviewer_independence_confirmed: boolean;
  independent_record_join_target_hash_reproduction_confirmed: boolean;
  one_to_one_entry_join_and_duplicate_missing_failure_confirmed: boolean;
  purge_embargo_exclusion_and_official_split_authority_confirmed: boolean;
  point_in_time_feature_and_explicit_missingness_confirmed: boolean;
  forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: boolean;
  split_specific_target_visibility_and_sealed_holdout_confirmed: boolean;
  exact_nine_continuous_target_semantics_confirmed: boolean;
  primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: boolean;
  exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: boolean;
  review_implementation_execution_and_output_validation_separation_confirmed: boolean;
  no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationContract = {
  schema_version: string;
  contract_sha256: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  join_implementation_id: string;
  join_implementation_version: string;
  target_implementation_id: string;
  target_implementation_version: string;
  canonical_serializer_version: string;
  input_schema_version: string;
  output_schema_version: string;
  input_contract: string;
  output_contract: string;
  exact_feature_count: number;
  exact_target_count: number;
  exact_horizons_market_sessions: number[];
  maximum_parallel_datasets: number;
  maximum_memory_mebibytes: number;
  callable_entrypoint_present: boolean;
  environment_inheritance_allowed: boolean;
  environment_variables_allowed: boolean;
  secrets_allowed: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  label_store_reads_allowed: boolean;
  training_store_reads_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  approved_review: HistoricalOutcomeFeatureLabelJoinTargetSpecReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_review_chain: boolean;
  implementation_name: string;
  rationale: string;
  known_limitations: string;
  implementation_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationContract;
  status: string;
  exact_approved_review_specification_and_artifact_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  implementation_artifact_and_code_revision_immutable_confirmed: boolean;
  exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: boolean;
  point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: boolean;
  exact_nine_raw_f64_target_projection_without_transform_confirmed: boolean;
  sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: boolean;
  canonical_serialization_and_fixed_input_output_schema_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: boolean;
  registration_review_runner_execution_and_output_validation_separation_confirmed: boolean;
  no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  label_access_authorized: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  output_validation_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: HistoricalOutcomeFeatureLabelJoinTargetSpecReview[];
  items: Array<{
    implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  label_access_authorized: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest = {
  expected_review_id: string;
  expected_review_sha256: string;
  expected_review_contract_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_id: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_id: string;
  expected_dataset_content_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_approved_review_specification_and_artifact_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  implementation_artifact_and_code_revision_immutable_confirmed: boolean;
  exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: boolean;
  point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: boolean;
  exact_nine_raw_f64_target_projection_without_transform_confirmed: boolean;
  sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: boolean;
  canonical_serialization_and_fixed_input_output_schema_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: boolean;
  registration_review_runner_execution_and_output_validation_separation_confirmed: boolean;
  no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict =
  | "approved_for_future_isolated_join_target_runner_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract = {
  schema_version: string;
  contract_sha256: string;
  independent_audit_implementation: string;
  required_fingerprint_checks: string[];
  required_semantic_checks: string[];
  required_sandbox_checks: string[];
  approval_scope: string;
  runner_registration_separate: boolean;
  first_execution_authorization_separate: boolean;
  join_execution_separate: boolean;
  output_validation_separate: boolean;
  training_and_reward_governance_separate: boolean;
  targets_remain_engineering_candidates_not_strategy_truth: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  exact_current_review_specification_artifact_and_dataset_binding_valid: boolean;
  exact_one_to_one_join_implementation_valid: boolean;
  exact_nine_raw_f64_target_projection_valid: boolean;
  point_in_time_missingness_purge_embargo_and_split_isolation_valid: boolean;
  sealed_holdout_inaccessible_to_training_and_tuning: boolean;
  canonical_serializer_schema_and_resource_contract_valid: boolean;
  no_action_position_threshold_rank_or_reward_semantics: boolean;
  no_entrypoint_environment_secret_network_tool_child_process_or_data_store_access: boolean;
  all_runner_execution_training_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord;
  review_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract;
  independent_audit: HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_complete_prior_chain: boolean;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  future_isolated_join_target_runner_registration_eligible: boolean;
  isolated_runner_registered: boolean;
  first_execution_authorization_review_eligible: boolean;
  label_access_authorized: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  output_validation_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract;
  items: Array<{
    implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord;
    current_independent_audit: HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    upstream_binding_current: boolean;
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetImplementationReview;
    review_eligible: boolean;
    future_isolated_join_target_runner_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  runner_registration_eligible_count: number;
  review_status: string;
  isolated_runner_registered: boolean;
  first_execution_authorization_review_eligible: boolean;
  label_access_authorized: boolean;
  join_execution_authorized: boolean;
  join_executed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  output_validation_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_specification_review_sha256: string;
  expected_specification_review_audit_sha256: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  expected_review_contract_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_implementation_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  implementation_record_and_contract_hashes_independently_reproduced_confirmed: boolean;
  implementation_artifact_digest_and_code_revision_reproducible_confirmed: boolean;
  exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: boolean;
  exact_nine_raw_f64_target_projection_without_transform_confirmed: boolean;
  point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: boolean;
  sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed: boolean;
  canonical_serializer_fixed_schemas_and_resource_limits_confirmed: boolean;
  no_action_position_threshold_rank_or_reward_semantics_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed: boolean;
  review_runner_authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerKind =
  "ephemeral_deterministic_process";

export type HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerContract = {
  schema_version: string;
  contract_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  input_mount_contract: string;
  output_contract: string;
  invocation_contract: string;
  next_gate: string;
  callable_entrypoint_registered: boolean;
  input_mount_read_only_required: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  content_addressed_create_once_output_required: boolean;
  independent_output_validation_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  host_environment_inherited: boolean;
  allowed_environment_variables: string[];
  secrets_available: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  label_store_reads_allowed: boolean;
  training_store_reads_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  historical_state_mutation_allowed: boolean;
  maximum_parallel_subjects: number;
  maximum_memory_mebibytes: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord;
  implementation_review: HistoricalOutcomeFeatureLabelJoinTargetImplementationReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_approval_chain: boolean;
  runner_name: string;
  runner_kind: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  runner_contract: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerContract;
  status: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  sealed_read_only_input_and_content_addressed_create_once_output_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  join_target_execution_started: boolean;
  output_artifact_created: boolean;
  split_manifest_generation_authorized: boolean;
  split_manifest_generated: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_bundle_generated: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: Array<{
    implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord;
    review: HistoricalOutcomeFeatureLabelJoinTargetImplementationReview;
  }>;
  allowed_runner_kinds: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerKind[];
  registration_allowed: boolean;
  items: Array<{
    runner: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  runner_status: string;
  callable_entrypoint_registered: boolean;
  first_execution_authorized: boolean;
  join_target_execution_started: boolean;
  output_artifact_created: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  semantic_target_assignment_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_specification_review_sha256: string;
  expected_specification_review_audit_sha256: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  expected_review_contract_sha256: string;
  expected_independent_audit_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  sealed_read_only_input_and_content_addressed_create_once_output_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_join_target_invocation"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  sealed_read_only_inputs_and_root_filesystem_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  deterministic_one_to_one_join_nine_target_and_canonical_schema_contract_confirmed: boolean;
  point_in_time_missingness_purge_embargo_split_and_sealed_holdout_confirmed: boolean;
  no_generic_label_or_training_store_access_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
  one_shot_invocation_limit: number;
  one_future_isolated_join_target_invocation_authorized: boolean;
  authorization_claimed: boolean;
  invocation_endpoint_available: boolean;
  join_target_execution_started: boolean;
  output_artifact_created: boolean;
  output_validation_authorized: boolean;
  label_access_authorized: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationReview;
    one_future_isolated_join_target_invocation_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  join_target_execution_started: boolean;
  output_artifact_created: boolean;
  output_validation_authorized: boolean;
  label_access_authorized: boolean;
  split_manifest_generation_authorized: boolean;
  feature_bundle_generation_authorized: boolean;
  feature_join_authorized: boolean;
  feature_join_performed: boolean;
  semantic_target_assignment_authorized: boolean;
  semantic_target_assigned: boolean;
  joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_review_contract_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_specification_review_id: string;
  expected_specification_review_sha256: string;
  expected_specification_review_audit_sha256: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  sealed_read_only_inputs_and_root_filesystem_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  deterministic_one_to_one_join_nine_target_and_canonical_schema_contract_confirmed: boolean;
  point_in_time_missingness_purge_embargo_split_and_sealed_holdout_confirmed: boolean;
  no_generic_label_or_training_store_access_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_specification_id: string;
  expected_specification_sha256: string;
  expected_specification_body_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_id: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  create_once_claim_and_failure_consumes_confirmed: boolean;
  exact_one_to_one_join_and_nine_raw_target_projection_confirmed: boolean;
  validation_and_sealed_holdout_target_values_withheld_confirmed: boolean;
  no_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetValue = {
  target_id: string;
  horizon_market_sessions: number;
  source_metric_field: string;
  value_kind: string;
  unit: string;
  role: string;
  exact_f64_bits_hex: string;
};

export type HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope = {
  schema_version: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  runner_id: string;
  runner_spec_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_review_sha256: string;
  specification_id: string;
  specification_sha256: string;
  specification_body_sha256: string;
  join_specification_sha256: string;
  target_specification_sha256: string;
  validation_id: string;
  validation_sha256: string;
  split_manifest_sha256: string;
  feature_bundle_sha256: string;
  combined_artifact_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  dataset_entry_count: number;
  active_candidate_row_count: number;
  excluded_purge_or_embargo_row_count: number;
  feature_catalog_count: number;
  target_count: number;
  train_target_vector_count: number;
  validation_target_withheld_count: number;
  sealed_holdout_target_withheld_count: number;
  rows: Array<{
    dataset_entry_id: string;
    split: "train" | "validation" | "sealed_holdout";
    component_id: string;
    decision_available_at: string;
    feature_records: Array<{
      feature_id: string;
      feature_namespace: string;
      value?: string;
      is_missing: boolean;
      missingness_reason: string;
      source_identity: string;
      available_at_utc: string;
    }>;
    target_visibility:
      | "train_candidate_raw_targets"
      | "validation_targets_withheld"
      | "sealed_holdout_targets_withheld";
    target_vector?: HistoricalOutcomeFeatureLabelJoinTargetValue[];
    target_commitment_sha256: string;
    source_binding_sha256: string;
  }>;
  excluded_rows: Array<{
    dataset_entry_id: string;
    split: "train" | "validation" | "sealed_holdout";
    purge_reason: string;
    feature_record_count: number;
    target_values_opened: boolean;
  }>;
  one_to_one_join_satisfied: boolean;
  exact_raw_f64_bits_preserved: boolean;
  validation_targets_withheld: boolean;
  sealed_holdout_targets_withheld: boolean;
  output_is_untrusted: boolean;
  independent_output_validation_completed: boolean;
  official_joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_started: boolean;
  reward_written: boolean;
  shadow_position_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  implementation_id: string;
  specification_id: string;
  validation_id: string;
  dataset_id: string;
  claimed_at: string;
  invoked_by: string;
  authorization_consumed: boolean;
  exact_bound_raw_outcome_read_allowed: boolean;
  generic_label_store_read_allowed: boolean;
  training_store_read_allowed: boolean;
  official_joined_dataset_write_allowed: boolean;
  training_write_allowed: boolean;
  reward_write_allowed: boolean;
  shadow_write_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  completed_at: string;
  duration_millis: number;
  status:
    | "completed_with_untrusted_joined_target_candidate_envelope"
    | "failed_authorization_consumed";
  bounded_error?: string;
  output_sha256?: string;
  untrusted_candidate_envelope?: HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope;
  ephemeral_directory_removed: boolean;
  independent_output_validation_completed: boolean;
  official_joined_dataset_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  eligible_authorizations: Array<{
    runner: HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRecord;
    review: HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationReview;
  }>;
  attempt_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_candidate_envelope_count: number;
  independent_output_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim;
    result?: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult;
    current_authorization_binding: boolean;
  }>;
  official_joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  independent_recomputation_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  output_remains_untrusted_pending_admission_confirmed: boolean;
  no_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  specification_id: string;
  specification_sha256: string;
  join_specification_sha256: string;
  target_specification_sha256: string;
  split_manifest_sha256: string;
  feature_bundle_sha256: string;
  combined_artifact_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  validated_at: string;
  validated_by: string;
  execution_invoked_by: string;
  validator_independent_from_execution_and_complete_prior_chain: boolean;
  exact_one_to_one_entry_join_recomputed: boolean;
  exact_65_feature_catalog_recomputed: boolean;
  point_in_time_and_explicit_missingness_recomputed: boolean;
  official_purge_embargo_and_split_recomputed: boolean;
  exact_nine_raw_f64_target_bits_recomputed: boolean;
  target_commitments_recomputed: boolean;
  train_only_target_exposure_verified: boolean;
  validation_targets_withheld_verified: boolean;
  sealed_holdout_targets_withheld_verified: boolean;
  downstream_authority_closed_verified: boolean;
  recomputed_rows_sha256: string;
  recomputed_excluded_rows_sha256: string;
  recomputed_target_commitments_sha256: string;
  mismatch_reasons: string[];
  verdict:
    | "validated_untrusted_candidate_for_future_admission_review"
    | "failed_independent_structure_or_recomputation_mismatch";
  untrusted_candidate_independently_validated: boolean;
  future_candidate_admission_review_eligible: boolean;
  official_joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict =
  | "approved_for_future_create_once_official_joined_dataset_materialization"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_specification_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_candidate_validation_and_complete_chain_confirmed: boolean;
  exact_one_to_one_entry_join_and_cardinality_confirmed: boolean;
  exact_65_feature_catalog_confirmed: boolean;
  point_in_time_and_explicit_missingness_confirmed: boolean;
  official_split_purge_and_embargo_confirmed: boolean;
  train_only_target_visibility_confirmed: boolean;
  validation_targets_withheld_confirmed: boolean;
  sealed_holdout_targets_withheld_confirmed: boolean;
  exact_nine_raw_f64_bits_and_commitments_confirmed: boolean;
  no_action_position_or_reward_semantics_confirmed: boolean;
  create_once_materialization_and_post_materialization_validation_separation_confirmed: boolean;
  downstream_authority_remains_closed_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  validation_id: string;
  validation_sha256: string;
  output_sha256: string;
  recomputed_rows_sha256: string;
  recomputed_excluded_rows_sha256: string;
  recomputed_target_commitments_sha256: string;
  dataset_entry_count: number;
  active_candidate_row_count: number;
  excluded_purge_or_embargo_row_count: number;
  feature_catalog_count: number;
  target_count: number;
  train_target_vector_count: number;
  validation_target_withheld_count: number;
  sealed_holdout_target_withheld_count: number;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  reviewer_independent_from_validator_executor_and_complete_prior_chain: boolean;
  join_target_candidate_admitted: boolean;
  future_create_once_official_joined_dataset_materialization_eligible: boolean;
  official_joined_dataset_materialization_started: boolean;
  official_joined_dataset_created: boolean;
  independently_validated_after_materialization: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      attempt: {
        claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim;
        result: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult;
      };
      validation: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord;
    };
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview;
    current_binding: boolean;
    review_eligible: boolean;
    join_target_candidate_admitted: boolean;
  }>;
  independently_validated_candidate_count: number;
  review_eligible_candidate_count: number;
  reviewed_candidate_count: number;
  admitted_candidate_count: number;
  changes_requested_or_rejected_count: number;
  future_official_joined_dataset_materialization_eligible_count: number;
  admission_status: string;
  candidate_admission_review_available: boolean;
  official_joined_dataset_materialization_enabled: boolean;
  official_joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest = {
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_specification_sha256: string;
  expected_join_specification_sha256: string;
  expected_target_specification_sha256: string;
  expected_split_manifest_sha256: string;
  expected_feature_bundle_sha256: string;
  expected_combined_artifact_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  exact_admitted_candidate_copy_only_confirmed: boolean;
  create_once_and_failure_consumes_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  independent_post_materialization_validation_required_confirmed: boolean;
  no_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim = {
  materialization_id: string;
  claim_sha256: string;
  attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  validation_id: string;
  validation_sha256: string;
  source_claim_sha256: string;
  source_result_id: string;
  source_result_sha256: string;
  source_output_sha256: string;
  materialized_by: string;
  excluded_prior_actor_ids: string[];
  claimed_at: string;
  claim_consumed: boolean;
  official_joined_dataset_materialization_started: boolean;
  training_store_write_allowed: boolean;
  training_allowed: boolean;
  reward_allowed: boolean;
  shadow_portfolio_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult = {
  result_id: string;
  result_sha256: string;
  materialization_id: string;
  claim_sha256: string;
  completed_at: string;
  status: "completed_pending_independent_validation" | "failed_claim_consumed";
  error?: string;
  official_joined_dataset_sha256?: string;
  official_joined_dataset_bytes: number;
  official_joined_dataset_created: boolean;
  exact_admitted_candidate_copy_completed: boolean;
  independent_post_materialization_validation_completed: boolean;
  eligible_for_training_store_copy: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    admitted_candidate: {
      candidate: {
        attempt: {
          claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim;
          result: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult;
        };
        validation: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord;
      };
      admission_review: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview;
    };
    attempt?: {
      claim: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim;
      result?: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult;
      official_joined_dataset?: {
        official_joined_dataset_sha256: string;
        active_row_count: number;
        excluded_purge_or_embargo_row_count: number;
        feature_catalog_count: number;
        target_count: number;
        validation_targets_withheld: boolean;
        sealed_holdout_targets_withheld: boolean;
        independently_validated_after_materialization: boolean;
        eligible_for_training_store_copy: boolean;
      };
    };
    materialization_eligible: boolean;
  }>;
  admitted_candidate_count: number;
  materialization_eligible_count: number;
  claim_count: number;
  completed_materialization_count: number;
  failed_materialization_count: number;
  pending_independent_validation_count: number;
  materialization_status: string;
  create_once_materialization_available: boolean;
  official_joined_dataset_created: boolean;
  independently_validated_after_materialization: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest = {
  expected_materialization_id: string;
  expected_materialization_claim_sha256: string;
  expected_materialization_result_sha256: string;
  expected_official_joined_dataset_sha256: string;
  expected_admission_review_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  independent_reopen_and_recomputation_confirmed: boolean;
  exact_current_admitted_candidate_binding_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  no_training_store_copy_training_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  materialization_id: string;
  materialization_claim_sha256: string;
  materialization_result_id: string;
  materialization_result_sha256: string;
  official_joined_dataset_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  source_validation_id: string;
  source_validation_sha256: string;
  source_output_sha256: string;
  dataset_id: string;
  dataset_content_sha256: string;
  dataset_manifest_sha256: string;
  candidate_set_sha256: string;
  recomputed_rows_sha256: string;
  recomputed_excluded_rows_sha256: string;
  recomputed_target_commitments_sha256: string;
  validated_at: string;
  validated_by: string;
  materialized_by: string;
  excluded_prior_actor_ids: string[];
  mismatch_reasons: string[];
  verdict: "validated_official_joined_dataset_for_future_training_store_copy_admission_review" | "failed_independent_post_materialization_validation";
  official_joined_dataset_independently_validated: boolean;
  future_training_store_copy_admission_review_eligible: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    materialization: {
      admitted_candidate: {
        candidate: {
          validation: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord;
        };
        admission_review: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview;
      };
      claim: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim;
      result: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult;
      official_joined_dataset: {
        official_joined_dataset_sha256: string;
        dataset_entry_count: number;
        active_row_count: number;
        excluded_purge_or_embargo_row_count: number;
        feature_catalog_count: number;
        target_count: number;
        train_target_vector_count: number;
        validation_target_withheld_count: number;
        sealed_holdout_target_withheld_count: number;
        validation_targets_withheld: boolean;
        sealed_holdout_targets_withheld: boolean;
      };
    };
    validation?: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_official_joined_dataset_count: number;
  failed_validation_count: number;
  future_training_store_copy_admission_review_eligible_count: number;
  validation_status: string;
  independent_post_materialization_validation_available: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict =
  | "approved_for_future_create_once_training_store_copy"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_materialization_id: string;
  expected_materialization_claim_sha256: string;
  expected_materialization_result_sha256: string;
  expected_official_joined_dataset_sha256: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_admission_review_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_stage_46_validation_and_complete_chain_confirmed: boolean;
  immutable_official_dataset_fingerprint_confirmed: boolean;
  exact_one_to_one_entry_join_and_cardinality_confirmed: boolean;
  exact_65_feature_catalog_confirmed: boolean;
  point_in_time_and_explicit_missingness_confirmed: boolean;
  official_split_purge_and_embargo_confirmed: boolean;
  exact_nine_raw_f64_bits_and_commitments_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  schema_contract_suitable_for_future_copy_only_confirmed: boolean;
  no_action_position_or_reward_semantics_confirmed: boolean;
  create_once_copy_and_post_copy_validation_remain_separate_confirmed: boolean;
  no_copy_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  materialization_id: string;
  materialization_claim_sha256: string;
  materialization_result_sha256: string;
  official_joined_dataset_sha256: string;
  output_validation_id: string;
  output_validation_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  training_store_copy_candidate_admitted: boolean;
  future_create_once_training_store_copy_eligible: boolean;
  training_store_copy_started: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    dataset: {
      materialization: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry["items"][number]["materialization"];
      validation: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord;
    };
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview;
    current_binding: boolean;
    review_eligible: boolean;
    training_store_copy_candidate_admitted: boolean;
  }>;
  independently_validated_official_joined_dataset_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  admitted_count: number;
  changes_requested_or_rejected_count: number;
  future_create_once_training_store_copy_eligible_count: number;
  admission_status: string;
  training_store_copy_admission_review_available: boolean;
  training_store_copy_enabled: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest = {
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_materialization_id: string;
  expected_materialization_claim_sha256: string;
  expected_materialization_result_sha256: string;
  expected_official_joined_dataset_sha256: string;
  expected_source_validation_sha256: string;
  expected_source_output_sha256: string;
  expected_dataset_content_sha256: string;
  expected_dataset_manifest_sha256: string;
  expected_candidate_set_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  exact_current_stage_47_admission_and_complete_chain_confirmed: boolean;
  claim_first_create_once_and_failure_consumes_confirmed: boolean;
  exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  independent_post_copy_validation_required_confirmed: boolean;
  no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim = {
  copy_id: string;
  claim_sha256: string;
  attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  output_validation_id: string;
  output_validation_sha256: string;
  materialization_id: string;
  materialization_claim_sha256: string;
  materialization_result_sha256: string;
  official_joined_dataset_sha256: string;
  copied_by: string;
  excluded_prior_actor_ids: string[];
  claimed_at: string;
  claim_consumed: boolean;
  training_store_copy_started: boolean;
  exact_target_directory_write_allowed: boolean;
  generic_training_store_read_allowed: boolean;
  generic_training_store_write_allowed: boolean;
  training_registration_allowed: boolean;
  training_run_allowed: boolean;
  reward_allowed: boolean;
  shadow_portfolio_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset = {
  training_store_dataset_sha256: string;
  copy_id: string;
  attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  output_validation_id: string;
  output_validation_sha256: string;
  official_joined_dataset_sha256: string;
  dataset_entry_count: number;
  active_row_count: number;
  excluded_purge_or_embargo_row_count: number;
  feature_catalog_count: number;
  target_count: number;
  train_target_vector_count: number;
  validation_target_withheld_count: number;
  sealed_holdout_target_withheld_count: number;
  exact_official_dataset_copy: boolean;
  validation_targets_withheld: boolean;
  sealed_holdout_targets_withheld: boolean;
  copied_to_training_store: boolean;
  independently_validated_after_training_store_copy: boolean;
  eligible_for_training_registration_review: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult = {
  result_id: string;
  result_sha256: string;
  copy_id: string;
  claim_sha256: string;
  completed_at: string;
  status: "completed_pending_independent_validation" | "failed_claim_consumed";
  error?: string;
  training_store_dataset_sha256?: string;
  training_store_dataset_bytes: number;
  copied_to_training_store: boolean;
  exact_official_dataset_copy_completed: boolean;
  independent_post_copy_validation_completed: boolean;
  eligible_for_training_registration_review: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    admitted_dataset: {
      dataset: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry["items"][number]["dataset"];
      admission_review: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview;
    };
    attempt?: {
      claim: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim;
      result?: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult;
      training_store_dataset?: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
    };
    copy_eligible: boolean;
  }>;
  admitted_dataset_count: number;
  copy_eligible_count: number;
  claim_count: number;
  completed_copy_count: number;
  failed_copy_count: number;
  pending_independent_post_copy_validation_count: number;
  copy_status: string;
  create_once_copy_available: boolean;
  copied_to_training_store: boolean;
  independently_validated_after_training_store_copy: boolean;
  training_registration_available: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest = {
  expected_copy_id: string;
  expected_copy_claim_sha256: string;
  expected_copy_result_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_admission_review_sha256: string;
  expected_output_validation_sha256: string;
  expected_official_joined_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  independent_reopen_and_recomputation_confirmed: boolean;
  exact_current_stage_47_and_stage_48_binding_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  copy_id: string;
  copy_claim_sha256: string;
  copy_result_id: string;
  copy_result_sha256: string;
  training_store_dataset_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  official_joined_dataset_sha256: string;
  recomputed_rows_sha256: string;
  recomputed_excluded_rows_sha256: string;
  recomputed_target_commitments_sha256: string;
  validated_at: string;
  validated_by: string;
  copied_by: string;
  mismatch_reasons: string[];
  verdict: "validated_training_store_copy_for_future_training_registration_review" | "failed_independent_post_copy_validation";
  training_store_copy_independently_validated: boolean;
  future_training_registration_review_eligible: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    copied_dataset: {
      admitted_dataset: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry["items"][number]["admitted_dataset"];
      attempt: NonNullable<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry["items"][number]["attempt"]>;
    };
    validation?: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_training_store_copy_count: number;
  failed_validation_count: number;
  future_training_registration_review_eligible_count: number;
  validation_status: string;
  independent_post_copy_validation_available: boolean;
  training_registration_available: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_copy_output_validation_id: string;
  expected_copy_output_validation_sha256: string;
  expected_copy_id: string;
  expected_copy_claim_sha256: string;
  expected_copy_result_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_copy_admission_review_sha256: string;
  expected_official_joined_dataset_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  verdict: "approved_for_future_create_once_training_registration" | "changes_requested" | "rejected";
  rationale: string;
  known_limitations: string;
  exact_current_stage_49_validation_and_complete_chain_confirmed: boolean;
  immutable_copy_and_validation_fingerprints_confirmed: boolean;
  independent_validation_passed_without_mismatch_confirmed: boolean;
  exact_official_to_training_store_copy_confirmed: boolean;
  exact_one_to_one_entry_join_and_cardinality_confirmed: boolean;
  exact_65_feature_catalog_confirmed: boolean;
  point_in_time_and_explicit_missingness_confirmed: boolean;
  official_split_purge_and_embargo_confirmed: boolean;
  exact_nine_raw_f64_bits_and_target_visibility_confirmed: boolean;
  no_action_position_or_reward_semantics_confirmed: boolean;
  create_once_registration_and_training_authorization_remain_separate_confirmed: boolean;
  no_registration_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  copy_output_validation_id: string;
  copy_output_validation_sha256: string;
  copy_id: string;
  copy_claim_sha256: string;
  copy_result_id: string;
  copy_result_sha256: string;
  training_store_dataset_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: "approved_for_future_create_once_training_registration" | "changes_requested" | "rejected";
  rationale: string;
  known_limitations: string;
  training_registration_candidate_admitted: boolean;
  future_create_once_training_registration_eligible: boolean;
  training_registration_started: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    dataset: {
      copied_dataset: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry["items"][number]["copied_dataset"];
      validation: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord;
    };
    latest_review?: HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionReview;
    current_binding: boolean;
    review_eligible: boolean;
    training_registration_candidate_admitted: boolean;
  }>;
  independently_validated_training_store_copy_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  admitted_count: number;
  changes_requested_or_rejected_count: number;
  future_create_once_training_registration_eligible_count: number;
  admission_status: string;
  training_registration_admission_review_available: boolean;
  training_registration_available: boolean;
  training_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeTrainingExperimentSuiteRequest = {
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_copy_output_validation_id: string;
  expected_copy_output_validation_sha256: string;
  expected_copy_id: string;
  expected_training_store_dataset_sha256: string;
  expected_recomputed_rows_sha256: string;
  expected_recomputed_excluded_rows_sha256: string;
  expected_recomputed_target_commitments_sha256: string;
  experiment_name: string;
  research_hypothesis: string;
  known_limitations: string;
  exact_current_stage_50_admission_and_complete_chain_confirmed: boolean;
  claim_first_create_once_and_failure_consumes_confirmed: boolean;
  fixed_three_arm_three_seed_suite_confirmed: boolean;
  train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: boolean;
  exact_65_feature_nine_raw_target_contract_confirmed: boolean;
  no_scalar_reward_action_position_or_ranking_semantics_confirmed: boolean;
  independent_registration_review_required_before_training_authorization_confirmed: boolean;
  no_training_run_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingExperimentSuiteSpecification = {
  schema_version: string;
  suite_version: string;
  specification_sha256: string;
  feature_catalog_count: number;
  target_count: number;
  target_contract_version: string;
  target_vector_order: string[];
  primary_supervised_target_id: string;
  risk_target_id: string;
  arms: Array<{
    algorithm: "frozen_zero_prediction_baseline" | "ridge_multi_target_regression" | "gradient_boosted_multi_target_regression";
    role: string;
    random_seeds: number[];
    max_epochs_or_boosting_rounds: number;
    learning_rate_micros: number;
    l2_regularization_micros: number;
    maximum_tree_depth: number;
    deterministic_replay_required: boolean;
  }>;
  fit_split: string;
  model_selection_split: string;
  sealed_holdout_split: string;
  feature_preprocessing_contract: string;
  objective_contract: string;
  model_selection_contract: string;
  reported_metric_ids: string[];
  sealed_holdout_access_allowed: boolean;
  sealed_holdout_labels_visible_to_training_worker: boolean;
  scalar_reward_defined: boolean;
  action_position_or_ranking_semantics_defined: boolean;
  resource_ceilings: {
    maximum_wall_clock_seconds: number;
    maximum_memory_mib: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes: number;
  };
  ambient_environment_available: boolean;
  network_available: boolean;
  external_tools_available: boolean;
  arbitrary_code_allowed: boolean;
  production_state_write_available: boolean;
};

export type HistoricalOutcomeTrainingExperimentRegistrationRecord = {
  registration_id: string;
  registration_sha256: string;
  claim_sha256: string;
  attempt_id: string;
  admission_review_id: string;
  admission_review_sha256: string;
  copy_output_validation_id: string;
  copy_output_validation_sha256: string;
  copy_id: string;
  training_store_dataset_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  dataset_entry_count: number;
  active_row_count: number;
  excluded_row_count: number;
  experiment_name: string;
  research_hypothesis: string;
  known_limitations: string;
  suite_specification: HistoricalOutcomeTrainingExperimentSuiteSpecification;
  registered_at: string;
  registered_by: string;
  excluded_prior_actor_ids: string[];
  status: "registered_not_run" | string;
  training_experiment_registered: boolean;
  independently_reviewed_after_registration: boolean;
  future_independent_registration_review_eligible: boolean;
  runner_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingExperimentRegistrationRegistry = {
  schema_version: string;
  policy_version: string;
  suite_version: string;
  items: Array<{
    admitted_dataset: {
      dataset: HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRegistry["items"][number]["dataset"];
      admission_review: HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionReview;
    };
    attempt?: {
      claim: {
        registration_id: string;
        claim_sha256: string;
        attempt_id: string;
        admission_review_id: string;
        training_store_dataset_sha256: string;
        experiment_name: string;
        research_hypothesis: string;
        known_limitations: string;
        suite_specification_sha256: string;
        registered_by: string;
        excluded_prior_actor_ids: string[];
        claimed_at: string;
        claim_consumed: boolean;
        training_run_allowed: boolean;
        reward_allowed: boolean;
        shadow_portfolio_allowed: boolean;
        order_generation_allowed: boolean;
        broker_access_allowed: boolean;
        trading_allowed: boolean;
      };
      result?: {
        result_id: string;
        result_sha256: string;
        status: "completed_pending_independent_review" | "failed_registration";
        error?: string;
        registration_sha256?: string;
        training_experiment_registered: boolean;
        future_independent_registration_review_eligible: boolean;
        training_authorized: boolean;
        training_started: boolean;
        trading_authorized: boolean;
      };
      registration?: HistoricalOutcomeTrainingExperimentRegistrationRecord;
    };
    registration_eligible: boolean;
  }>;
  admitted_candidate_count: number;
  registration_eligible_count: number;
  claim_count: number;
  completed_registration_count: number;
  failed_or_incomplete_registration_count: number;
  pending_independent_registration_review_count: number;
  registration_status: string;
  create_once_registration_available: boolean;
  training_experiment_registered: boolean;
  independent_registration_review_completed: boolean;
  runner_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_claim_sha256: string;
  expected_result_id: string;
  expected_result_sha256: string;
  expected_admission_review_id: string;
  expected_admission_review_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  expected_suite_specification_sha256: string;
  verdict: "approved_for_future_training_implementation_registration" | "changes_requested" | "rejected";
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_registration_and_complete_chain_confirmed: boolean;
  immutable_claim_registration_result_and_suite_hashes_confirmed: boolean;
  claim_first_create_once_success_and_registered_not_run_confirmed: boolean;
  registrar_and_reviewer_independence_confirmed: boolean;
  fixed_three_arm_three_seed_suite_confirmed: boolean;
  exact_65_feature_nine_raw_continuous_target_contract_confirmed: boolean;
  train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: boolean;
  per_target_per_seed_metrics_without_composite_masking_confirmed: boolean;
  fixed_resource_ceilings_and_deterministic_replay_confirmed: boolean;
  no_scalar_reward_action_position_or_ranking_semantics_confirmed: boolean;
  implementation_registration_runner_and_run_authorization_remain_separate_confirmed: boolean;
  no_training_run_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingExperimentRegistrationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  registration_id: string;
  registration_sha256: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  admission_review_id: string;
  admission_review_sha256: string;
  copy_output_validation_id: string;
  copy_output_validation_sha256: string;
  copy_id: string;
  training_store_dataset_sha256: string;
  source_official_joined_dataset_sha256: string;
  source_dataset_id: string;
  source_dataset_content_sha256: string;
  source_dataset_manifest_sha256: string;
  source_candidate_set_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  dataset_entry_count: number;
  active_row_count: number;
  excluded_row_count: number;
  feature_catalog_count: number;
  target_count: number;
  train_target_vector_count: number;
  validation_target_withheld_count: number;
  sealed_holdout_target_withheld_count: number;
  suite_version: string;
  suite_specification_sha256: string;
  arm_count: number;
  random_seeds: number[];
  registered_at: string;
  reviewed_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest["verdict"];
  rationale: string;
  known_limitations: string;
  reviewer_independent_from_registrar_and_complete_prior_chain: boolean;
  training_experiment_registration_independently_approved: boolean;
  future_training_implementation_registration_eligible: boolean;
  training_implementation_registered: boolean;
  runner_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    registered_experiment: {
      admitted_dataset: HistoricalOutcomeTrainingExperimentRegistrationRegistry["items"][number]["admitted_dataset"];
      attempt: NonNullable<HistoricalOutcomeTrainingExperimentRegistrationRegistry["items"][number]["attempt"]>;
    };
    latest_review?: HistoricalOutcomeTrainingExperimentRegistrationReview;
    current_binding: boolean;
    review_eligible: boolean;
    independently_approved: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  rejected_or_changes_requested_count: number;
  future_training_implementation_registration_eligible_count: number;
  review_status: string;
  training_implementation_registered: boolean;
  runner_registered: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeTrainingImplementationRequest = {
  expected_review_id: string;
  expected_review_sha256: string;
  expected_attempt_id: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_claim_sha256: string;
  expected_result_id: string;
  expected_result_sha256: string;
  expected_suite_specification_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_stage_52_review_and_stage_51_registration_binding_confirmed: boolean;
  registrar_independent_from_complete_prior_chain_confirmed: boolean;
  immutable_artifact_and_code_revision_confirmed: boolean;
  fixed_three_arm_three_seed_implementation_confirmed: boolean;
  exact_65_feature_nine_raw_continuous_target_contract_confirmed: boolean;
  train_only_preprocessing_and_fit_confirmed: boolean;
  validation_selection_and_sealed_holdout_isolation_confirmed: boolean;
  per_target_per_seed_metrics_without_composite_masking_confirmed: boolean;
  deterministic_replay_and_fixed_resource_ceilings_confirmed: boolean;
  no_scalar_reward_action_position_or_ranking_semantics_confirmed: boolean;
  implementation_review_runner_and_run_authorization_separation_confirmed: boolean;
  no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingImplementationContract = {
  schema_version: string;
  contract_sha256: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  suite_version: string;
  suite_specification_sha256: string;
  target_contract_version: string;
  input_schema_version: string;
  output_schema_version: string;
  canonical_serializer_version: string;
  preprocessor_implementation_version: string;
  algorithm_implementation_versions: string[];
  exact_feature_count: number;
  exact_target_count: number;
  exact_random_seeds: number[];
  reported_metric_ids: string[];
  maximum_wall_clock_seconds: number;
  maximum_memory_mib: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
  input_contract: string;
  output_contract: string;
  callable_entrypoint_present: boolean;
  ambient_environment_available: boolean;
  environment_variables_allowed: boolean;
  secrets_allowed: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  training_store_reads_allowed: boolean;
  validation_labels_visible_to_fit_worker: boolean;
  sealed_holdout_labels_visible_to_fit_or_selection_worker: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  scalar_reward_defined: boolean;
  action_position_or_ranking_semantics_defined: boolean;
};

export type HistoricalOutcomeTrainingImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  approved_registration_review: HistoricalOutcomeTrainingExperimentRegistrationReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_prior_chain: boolean;
  implementation_name: string;
  rationale: string;
  known_limitations: string;
  implementation_contract: HistoricalOutcomeTrainingImplementationContract;
  status: "registered_not_reviewed_not_run";
  training_implementation_registered: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_started: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: HistoricalOutcomeTrainingExperimentRegistrationReview[];
  items: Array<{
    implementation: HistoricalOutcomeTrainingImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  runner_registered: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_started: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeTrainingImplementationReviewVerdict =
  | "approved_for_future_isolated_training_runner_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeTrainingImplementationReviewContract = {
  schema_version: string;
  contract_sha256: string;
  independent_audit_implementation: string;
  required_fingerprint_checks: string[];
  required_training_semantic_checks: string[];
  required_sandbox_checks: string[];
  approval_scope: string;
  runner_registration_separate: boolean;
  data_access_authorization_separate: boolean;
  training_execution_separate: boolean;
  output_validation_separate: boolean;
  reward_governance_separate: boolean;
  targets_remain_engineering_candidates_not_strategy_truth: boolean;
};

export type HistoricalOutcomeTrainingImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  exact_stage_52_review_and_stage_51_chain_binding_valid: boolean;
  immutable_artifact_and_code_revision_valid: boolean;
  fixed_three_arm_three_seed_contract_valid: boolean;
  exact_65_feature_nine_raw_continuous_target_contract_valid: boolean;
  train_only_preprocessing_and_fit_valid: boolean;
  validation_only_selection_and_sealed_holdout_isolation_valid: boolean;
  per_target_per_seed_metrics_without_composite_masking_valid: boolean;
  deterministic_replay_and_fixed_resource_ceilings_valid: boolean;
  no_scalar_reward_action_position_or_ranking_semantics: boolean;
  no_entrypoint_environment_secret_network_tool_child_process_or_data_access: boolean;
  all_runner_training_artifact_metric_reward_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type HistoricalOutcomeTrainingImplementationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: HistoricalOutcomeTrainingImplementationRecord;
  review_contract: HistoricalOutcomeTrainingImplementationReviewContract;
  independent_audit: HistoricalOutcomeTrainingImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_registrar_and_complete_prior_chain: boolean;
  verdict: HistoricalOutcomeTrainingImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  training_implementation_independently_approved: boolean;
  future_isolated_training_runner_registration_eligible: boolean;
  isolated_training_runner_registered: boolean;
  data_access_authorization_review_eligible: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_started: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ReviewHistoricalOutcomeTrainingImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_stage_52_review_sha256: string;
  expected_stage_51_registration_sha256: string;
  expected_stage_51_claim_sha256: string;
  expected_stage_51_result_sha256: string;
  expected_suite_specification_sha256: string;
  expected_review_contract_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: HistoricalOutcomeTrainingImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_implementation_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_record_and_contract_hashes_independently_reproduced_confirmed: boolean;
  immutable_artifact_digest_and_code_revision_reproducible_confirmed: boolean;
  fixed_three_arm_three_seed_implementation_confirmed: boolean;
  exact_65_feature_nine_raw_continuous_target_contract_confirmed: boolean;
  train_only_preprocessing_and_fit_confirmed: boolean;
  validation_only_selection_and_sealed_holdout_isolation_confirmed: boolean;
  per_target_per_seed_metrics_without_composite_masking_confirmed: boolean;
  deterministic_replay_and_fixed_resource_ceilings_confirmed: boolean;
  no_scalar_reward_action_position_or_ranking_semantics_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed: boolean;
  review_runner_data_access_training_output_validation_and_reward_separation_confirmed: boolean;
  no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_contract: HistoricalOutcomeTrainingImplementationReviewContract;
  items: Array<{
    implementation: HistoricalOutcomeTrainingImplementationRecord;
    current_independent_audit: HistoricalOutcomeTrainingImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    upstream_binding_current: boolean;
    latest_review?: HistoricalOutcomeTrainingImplementationReview;
    review_eligible: boolean;
    future_isolated_training_runner_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  current_binding_approved_count: number;
  future_isolated_runner_registration_eligible_count: number;
  changes_requested_or_rejected_count: number;
  review_status: string;
  isolated_training_runner_registered: boolean;
  data_access_authorization_review_eligible: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_started: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeTrainingIsolatedRunnerKind =
  "ephemeral_deterministic_training_process";

export type HistoricalOutcomeTrainingIsolatedRunnerContract = {
  schema_version: string;
  contract_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  input_mount_contract: string;
  output_contract: string;
  invocation_contract: string;
  next_gate: string;
  callable_entrypoint_registered: boolean;
  input_mount_read_only_required: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  content_addressed_create_once_output_required: boolean;
  independent_output_validation_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  host_environment_inherited: boolean;
  allowed_environment_variables: string[];
  secrets_available: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  exact_training_dataset_mount_registered: boolean;
  training_store_reads_allowed: boolean;
  validation_labels_visible_to_fit_worker: boolean;
  sealed_holdout_labels_visible_to_fit_or_selection_worker: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  training_store_writes_allowed: boolean;
  model_artifact_store_writes_allowed: boolean;
  metric_store_writes_allowed: boolean;
  maximum_parallel_experiments: number;
  maximum_memory_mib: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type HistoricalOutcomeTrainingIsolatedRunnerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: HistoricalOutcomeTrainingImplementationRecord;
  implementation_review: HistoricalOutcomeTrainingImplementationReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_approval_chain: boolean;
  runner_name: string;
  runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  runner_contract: HistoricalOutcomeTrainingIsolatedRunnerContract;
  status: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: boolean;
  train_validation_and_sealed_holdout_mount_isolation_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_run_allowed: boolean;
  training_started: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: Array<{
    implementation: HistoricalOutcomeTrainingImplementationRecord;
    review: HistoricalOutcomeTrainingImplementationReview;
  }>;
  allowed_runner_kinds: HistoricalOutcomeTrainingIsolatedRunnerKind[];
  registration_allowed: boolean;
  items: Array<{
    runner: HistoricalOutcomeTrainingIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  runner_status: string;
  callable_entrypoint_registered: boolean;
  first_execution_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  training_started: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_stage_52_review_sha256: string;
  expected_stage_51_registration_sha256: string;
  expected_stage_51_claim_sha256: string;
  expected_stage_51_result_sha256: string;
  expected_suite_specification_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  expected_review_contract_sha256: string;
  expected_independent_audit_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_and_code_revision_immutable_confirmed: boolean;
  exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: boolean;
  train_validation_and_sealed_holdout_mount_isolation_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_training_invocation"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeTrainingFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_review_contract_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_stage_52_review_sha256: string;
  expected_stage_51_registration_sha256: string;
  expected_stage_51_claim_sha256: string;
  expected_stage_51_result_sha256: string;
  expected_suite_specification_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  verdict: HistoricalOutcomeTrainingFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  sealed_read_only_inputs_and_root_filesystem_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  fixed_three_arm_three_seed_sixty_five_feature_nine_target_suite_confirmed: boolean;
  train_validation_and_sealed_holdout_isolation_confirmed: boolean;
  exact_read_only_training_store_mount_and_no_other_data_access_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: HistoricalOutcomeTrainingIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeTrainingFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_invocation_limit: number;
  one_future_isolated_training_invocation_authorized: boolean;
  authorization_claimed: boolean;
  invocation_endpoint_available: boolean;
  training_run_started: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeTrainingIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeTrainingFirstExecutionAuthorizationReview;
    one_future_isolated_training_invocation_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  training_run_started: boolean;
  model_artifact_created: boolean;
  metrics_created: boolean;
  output_validation_authorized: boolean;
  training_data_access_authorized: boolean;
  training_authorized: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeTrainingOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_suite_specification_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  claim_first_create_once_and_failure_consumes_confirmed: boolean;
  exact_read_only_training_store_dataset_only_confirmed: boolean;
  train_only_fit_and_explicit_missingness_preserved_confirmed: boolean;
  validation_and_sealed_holdout_labels_remain_withheld_confirmed: boolean;
  fixed_three_arm_three_seed_suite_confirmed: boolean;
  untrusted_content_addressed_output_and_independent_validation_confirmed: boolean;
  no_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_review_sha256: string;
  suite_specification_sha256: string;
  training_store_dataset_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  claimed_at: string;
  invoked_by: string;
  isolation_backend: string;
  authorization_consumed: boolean;
  train_target_read_allowed: boolean;
  validation_target_read_allowed: boolean;
  sealed_holdout_target_read_allowed: boolean;
  reward_write_allowed: boolean;
  shadow_write_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeTrainingExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  completed_at: string;
  duration_millis: number;
  status:
    | "completed_with_untrusted_train_only_artifacts"
    | "failed_authorization_consumed";
  output_sha256?: string;
  output_bytes: number;
  bounded_error?: string;
  untrusted_artifact_envelope?: {
    train_row_count: number;
    validation_row_count_with_targets_withheld: number;
    sealed_holdout_row_count_with_targets_withheld: number;
    model_artifacts: Array<{
      artifact_sha256: string;
      algorithm: "frozen_zero_prediction_baseline" | "ridge_multi_target_regression" | "gradient_boosted_multi_target_regression";
      random_seed: number;
      validation_selected: boolean;
      sealed_holdout_accessed: boolean;
    }>;
    fit_diagnostics: Array<{
      split: string;
      model_selection_metric: boolean;
    }>;
    validation_labels_accessed: boolean;
    validation_selection_completed: boolean;
    sealed_holdout_labels_accessed: boolean;
    output_is_untrusted: boolean;
    independent_output_validation_completed: boolean;
  };
  ephemeral_directory_removed: boolean;
  independent_output_validation_completed: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_artifact_envelope_count: number;
  independent_output_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeTrainingExecutionAttemptClaim;
    result?: HistoricalOutcomeTrainingExecutionAttemptResult;
  }>;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeTrainingOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_suite_specification_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  independent_reopen_and_second_implementation_recomputation_confirmed: boolean;
  exact_current_stage_51_through_stage_57_binding_confirmed: boolean;
  all_nine_model_artifacts_and_eighty_one_diagnostics_bitwise_recomputed_confirmed: boolean;
  validation_and_sealed_holdout_targets_remain_withheld_confirmed: boolean;
  no_model_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeTrainingOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  suite_specification_sha256: string;
  training_store_dataset_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  invoked_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_execution_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_57_chain_verified: boolean;
  claim_fingerprint_independently_verified: boolean;
  result_fingerprint_independently_verified: boolean;
  envelope_fingerprint_independently_verified: boolean;
  exact_training_store_dataset_and_suite_verified: boolean;
  exact_65_feature_preprocessing_bitwise_recomputed: boolean;
  exact_nine_model_artifacts_bitwise_recomputed: boolean;
  exact_eighty_one_train_only_diagnostics_bitwise_recomputed: boolean;
  validation_targets_withheld_verified: boolean;
  sealed_holdout_targets_withheld_verified: boolean;
  no_model_selection_or_downstream_authority_verified: boolean;
  recomputed_model_artifact_count: number;
  recomputed_fit_diagnostic_count: number;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_train_only_artifacts"
    | "failed_independent_training_output_validation";
  training_output_independently_validated: boolean;
  future_validation_evaluation_implementation_registration_eligible: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeTrainingOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeTrainingExecutionAttemptClaim;
      result: HistoricalOutcomeTrainingExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeTrainingOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_train_only_artifact_envelope_count: number;
  failed_validation_count: number;
  future_validation_evaluation_implementation_registration_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  validation_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeValidationEvaluationImplementationRequest = {
  expected_validation_id: string;
  expected_validation_sha256: string;
  expected_attempt_id: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_suite_specification_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  expected_candidate_set_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_stage_58_validation_and_stage_57_output_binding_confirmed: boolean;
  registrar_independent_from_complete_prior_chain_confirmed: boolean;
  immutable_artifact_revision_and_protocol_confirmed: boolean;
  evaluation_rules_frozen_before_validation_label_access_confirmed: boolean;
  all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed: boolean;
  zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed: boolean;
  no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed: boolean;
  validation_only_and_sealed_holdout_isolation_confirmed: boolean;
  independent_review_runner_and_one_shot_authorization_required_confirmed: boolean;
  no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeValidationEvaluationCandidateBinding = {
  algorithm_id: string;
  random_seed: number;
  artifact_sha256: string;
  exact_target_model_count: number;
};

export type HistoricalOutcomeValidationEvaluationImplementationRecord = {
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_validation: HistoricalOutcomeTrainingOutputValidationRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_prior_chain: boolean;
  implementation_name: string;
  rationale: string;
  known_limitations: string;
  implementation_contract: {
    contract_sha256: string;
    validation_sha256: string;
    implementation_protocol_version: string;
    implementation_artifact_sha256: string;
    immutable_code_revision: string;
    candidate_set_sha256: string;
    candidate_bindings: HistoricalOutcomeValidationEvaluationCandidateBinding[];
    feature_order_sha256: string;
    preprocessing_sha256: string;
    target_vector_order: string[];
    exact_feature_count: number;
    exact_target_count: number;
    exact_artifact_count: number;
    exact_random_seeds: number[];
    reported_metric_ids: string[];
    bootstrap_replications: number;
    family_wise_error_correction: string;
    exact_candidate_hypothesis_count: number;
    minimum_relative_mae_improvement_ppm: number;
    minimum_spearman_millionths: number;
    minimum_directional_accuracy_millionths: number;
    minimum_calibration_slope_millionths: number;
    maximum_calibration_slope_millionths: number;
    minimum_validation_rows: number;
    minimum_independent_components: number;
    all_three_seeds_must_pass: boolean;
    tie_break_preferred_algorithm_id: string;
    no_composite_score_or_global_model_validity_claim: boolean;
    validation_labels_access_allowed: boolean;
    sealed_holdout_labels_access_allowed: boolean;
    candidate_selection_allowed: boolean;
  };
  status: string;
  validation_evaluation_implementation_registered: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  validation_label_access_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_outputs: Array<{
    validation: HistoricalOutcomeTrainingOutputValidationRecord;
    candidate_bindings: HistoricalOutcomeValidationEvaluationCandidateBinding[];
    candidate_set_sha256: string;
    feature_order_sha256: string;
    preprocessing_sha256: string;
    target_vector_order: string[];
  }>;
  items: Array<{
    implementation: HistoricalOutcomeValidationEvaluationImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  validation_label_access_authorized: boolean;
  evaluation_started: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeValidationEvaluationImplementationReviewVerdict =
  | "approved_for_future_isolated_validation_evaluation_runner_registration"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeValidationEvaluationImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_candidate_set_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_upstream_validation_sha256: string;
  expected_upstream_output_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: HistoricalOutcomeValidationEvaluationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_stage_57_through_stage_59_chain_confirmed: boolean;
  reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed: boolean;
  exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed: boolean;
  exact_65_feature_nine_target_and_per_target_metric_contract_confirmed: boolean;
  component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: boolean;
  minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: boolean;
  no_seed_shopping_tuning_or_composite_masking_confirmed: boolean;
  rules_frozen_before_validation_label_access_confirmed: boolean;
  independent_runner_authorization_and_output_validation_separation_confirmed: boolean;
  no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeValidationEvaluationImplementationIndependentAudit = {
  audit_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  candidate_set_hash_independently_reproduced: boolean;
  exact_stage_58_validation_and_stage_57_output_binding_valid: boolean;
  exact_three_algorithm_three_seed_nine_artifact_matrix_valid: boolean;
  exact_65_feature_nine_target_order_valid: boolean;
  per_target_per_seed_metric_contract_valid: boolean;
  paired_component_block_bootstrap_holm_contract_valid: boolean;
  minimum_effect_diagnostics_and_sample_gates_valid: boolean;
  all_three_seed_no_shopping_no_composite_contract_valid: boolean;
  rules_frozen_before_label_access_valid: boolean;
  all_evaluation_selection_store_reward_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type HistoricalOutcomeValidationEvaluationImplementationReviewRecord = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: HistoricalOutcomeValidationEvaluationImplementationRecord;
  independent_audit: HistoricalOutcomeValidationEvaluationImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeValidationEvaluationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  validation_evaluation_implementation_independently_approved: boolean;
  future_isolated_runner_registration_eligible: boolean;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: HistoricalOutcomeValidationEvaluationImplementationRecord;
    current_independent_audit: HistoricalOutcomeValidationEvaluationImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review?: HistoricalOutcomeValidationEvaluationImplementationReviewRecord;
    review_eligible: boolean;
    future_isolated_runner_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_requested_or_rejected_count: number;
  future_isolated_runner_registration_eligible_count: number;
  review_status: string;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeValidationEvaluationIsolatedRunnerKind =
  "ephemeral_deterministic_per_target_validation_evaluator";

export type RegisterHistoricalOutcomeValidationEvaluationIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_candidate_set_sha256: string;
  expected_upstream_validation_sha256: string;
  expected_upstream_output_sha256: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeValidationEvaluationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_approved_review_and_complete_upstream_binding_confirmed: boolean;
  registrar_independence_confirmed: boolean;
  runner_artifact_code_runtime_and_protocol_immutable_confirmed: boolean;
  future_exact_read_only_validation_and_candidate_mounts_confirmed: boolean;
  sealed_holdout_and_training_update_isolation_confirmed: boolean;
  per_target_per_seed_untrusted_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_first_execution_and_output_validation_separation_confirmed: boolean;
  no_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeValidationEvaluationIsolatedRunnerRecord = {
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: HistoricalOutcomeValidationEvaluationImplementationRecord;
  implementation_review: HistoricalOutcomeValidationEvaluationImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_complete_approval_chain: boolean;
  runner_name: string;
  runner_kind: HistoricalOutcomeValidationEvaluationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  runner_contract: {
    contract_sha256: string;
    runtime_identity: string;
    runtime_version: string;
    input_mount_contract: string;
    output_contract: string;
    invocation_contract: string;
    next_gate: string;
    callable_entrypoint_registered: boolean;
    exact_validation_mount_registered: boolean;
    exact_candidate_artifact_mount_registered: boolean;
    validation_features_access_allowed: boolean;
    validation_labels_access_allowed: boolean;
    training_or_preprocessing_update_allowed: boolean;
    sealed_holdout_features_access_allowed: boolean;
    sealed_holdout_labels_access_allowed: boolean;
    future_untrusted_per_target_selection_envelope_required: boolean;
    no_composite_score_or_global_model_validity_claim_required: boolean;
    maximum_parallel_evaluations: number;
    maximum_memory_mib: number;
    maximum_wall_clock_seconds: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes: number;
  };
  status: string;
  confirmations_complete: boolean;
  exact_current_stage_51_through_stage_98_binding_confirmed: boolean;
  registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: boolean;
  all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: boolean;
  future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: boolean;
  strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  source_available_at_remains_unverified_until_separate_evidence_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  untrusted_output_created: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: Array<{
    implementation: HistoricalOutcomeValidationEvaluationImplementationRecord;
    review: HistoricalOutcomeValidationEvaluationImplementationReviewRecord;
  }>;
  allowed_runner_kinds: HistoricalOutcomeValidationEvaluationIsolatedRunnerKind[];
  registration_allowed: boolean;
  items: Array<{
    runner: HistoricalOutcomeValidationEvaluationIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  runner_status: string;
  callable_entrypoint_registered: boolean;
  first_execution_authorized: boolean;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  untrusted_output_created: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_validation_evaluation_invocation"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_candidate_set_sha256: string;
  expected_upstream_validation_sha256: string;
  expected_upstream_output_sha256: string;
  verdict: HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  future_exact_read_only_validation_and_candidate_mounts_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  fixed_three_arm_three_seed_sixty_five_feature_nine_target_protocol_confirmed: boolean;
  validation_only_no_training_update_and_sealed_holdout_isolation_confirmed: boolean;
  exact_read_only_validation_and_candidate_mounts_and_no_other_data_access_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_selection_separation_confirmed: boolean;
  no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: HistoricalOutcomeValidationEvaluationIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_invocation_limit: number;
  one_future_isolated_validation_evaluation_invocation_authorized: boolean;
  authorization_claimed: boolean;
  invocation_endpoint_available: boolean;
  validation_feature_access_authorized: boolean;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeValidationEvaluationIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationReview;
    one_future_isolated_validation_evaluation_invocation_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  validation_feature_access_authorized: boolean;
  validation_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeValidationEvaluationOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_candidate_set_sha256: string;
  expected_upstream_validation_sha256: string;
  expected_upstream_output_sha256: string;
  claim_first_create_once_and_failure_consumes_confirmed: boolean;
  exact_validation_features_labels_and_nine_candidates_only_confirmed: boolean;
  frozen_metrics_component_bootstrap_and_holm_confirmed: boolean;
  no_seed_shopping_tuning_composite_or_global_claim_confirmed: boolean;
  validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: boolean;
  untrusted_content_addressed_output_and_independent_validation_confirmed: boolean;
  no_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeValidationEvaluationAlgorithm =
  | "frozen_zero_prediction_baseline"
  | "ridge_multi_target_regression"
  | "gradient_boosted_multi_target_regression";

export type HistoricalOutcomeValidationEvaluationMetric = {
  algorithm: HistoricalOutcomeValidationEvaluationAlgorithm;
  random_seed: number;
  target_id: string;
  validation_row_count: number;
  independent_component_count: number;
  mae_f64_bits_hex: string;
  zero_baseline_mae_f64_bits_hex: string;
  relative_mae_improvement_f64_bits_hex: string;
  component_block_bootstrap_p_value_f64_bits_hex?: string;
  holm_adjusted_p_value_f64_bits_hex?: string;
  spearman_f64_bits_hex?: string;
  directional_accuracy_f64_bits_hex: string;
  calibration_slope_f64_bits_hex?: string;
  evidence_status: string;
  all_preregistered_thresholds_passed: boolean;
  official_model_selection_metric: boolean;
};

export type HistoricalOutcomeValidationEvaluationPerTargetRecommendation = {
  target_id: string;
  status: string;
  recommended_algorithm?: HistoricalOutcomeValidationEvaluationAlgorithm;
  three_seed_median_mae_f64_bits_hex?: string;
  rationale: string;
  all_three_seeds_passed: boolean;
  official_selection: boolean;
};

export type HistoricalOutcomeValidationEvaluationUntrustedEnvelope = {
  schema_version: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  implementation_review_sha256: string;
  implementation_independent_audit_sha256: string;
  upstream_validation_sha256: string;
  upstream_output_sha256: string;
  training_store_dataset_sha256: string;
  candidate_set_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  validation_projection_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  target_order: string[];
  validation_row_count: number;
  independent_component_count: number;
  exact_artifact_count: number;
  exact_metric_count: number;
  exact_candidate_hypothesis_count: number;
  metrics: HistoricalOutcomeValidationEvaluationMetric[];
  per_target_recommendations: HistoricalOutcomeValidationEvaluationPerTargetRecommendation[];
  validation_features_accessed: boolean;
  validation_labels_accessed: boolean;
  validation_evaluation_completed: boolean;
  sealed_holdout_features_accessed: boolean;
  sealed_holdout_labels_accessed: boolean;
  training_or_preprocessing_updated: boolean;
  output_is_untrusted: boolean;
  independent_output_validation_completed: boolean;
  official_candidate_selection_completed: boolean;
  composite_score_created: boolean;
  global_model_validity_claimed: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  scalar_reward_written: boolean;
  shadow_position_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type HistoricalOutcomeValidationEvaluationExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_review_sha256: string;
  candidate_set_sha256: string;
  upstream_validation_sha256: string;
  upstream_output_sha256: string;
  training_store_dataset_sha256: string;
  rows_sha256: string;
  excluded_rows_sha256: string;
  target_commitments_sha256: string;
  claimed_at: string;
  invoked_by: string;
  isolation_backend: string;
  authorization_consumed: boolean;
  validation_feature_read_allowed: boolean;
  validation_label_read_allowed: boolean;
  sealed_holdout_feature_read_allowed: boolean;
  sealed_holdout_label_read_allowed: boolean;
  official_candidate_selection_allowed: boolean;
  trading_allowed: boolean;
};

export type HistoricalOutcomeValidationEvaluationExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  completed_at: string;
  duration_millis: number;
  status:
    | "completed_with_untrusted_validation_evaluation"
    | "failed_authorization_consumed";
  output_sha256?: string;
  bounded_error?: string;
  untrusted_evaluation_envelope?: HistoricalOutcomeValidationEvaluationUntrustedEnvelope;
  ephemeral_directory_removed: boolean;
  validation_features_accessed: boolean;
  validation_labels_accessed: boolean;
  evaluation_completed: boolean;
  sealed_holdout_features_accessed: boolean;
  sealed_holdout_labels_accessed: boolean;
  independent_output_validation_completed: boolean;
  official_candidate_selection_completed: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_evaluation_envelope_count: number;
  independent_output_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: HistoricalOutcomeValidationEvaluationExecutionAttemptClaim;
    result?: HistoricalOutcomeValidationEvaluationExecutionAttemptResult;
  }>;
  sealed_holdout_access_authorized: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeValidationEvaluationOutputValidationVerdict =
  | "independently_validated_untrusted_validation_evaluation"
  | "failed_independent_validation_evaluation_output_validation";

export type HistoricalOutcomeValidationEvaluationOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  authorization_review_sha256: string;
  isolated_runner_spec_sha256: string;
  implementation_sha256: string;
  implementation_review_sha256: string;
  candidate_set_sha256: string;
  upstream_validation_sha256: string;
  training_store_dataset_sha256: string;
  validation_projection_sha256: string;
  validated_at: string;
  validated_by: string;
  invoked_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_execution_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_63_chain_verified: boolean;
  validation_projection_independently_reconstructed: boolean;
  exact_nine_candidate_predictions_bitwise_recomputed: boolean;
  exact_eighty_one_metrics_bitwise_recomputed: boolean;
  exact_fifty_four_component_bootstrap_and_holm_tests_bitwise_recomputed: boolean;
  exact_nine_per_target_recommendations_bitwise_recomputed: boolean;
  sealed_holdout_non_access_verified: boolean;
  no_selection_or_downstream_authority_verified: boolean;
  recomputed_metric_count: number;
  recomputed_candidate_hypothesis_count: number;
  recomputed_per_target_recommendation_count: number;
  mismatch_reasons: string[];
  verdict: HistoricalOutcomeValidationEvaluationOutputValidationVerdict;
  validation_evaluation_output_independently_validated: boolean;
  future_per_target_candidate_admission_review_eligible: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeValidationEvaluationExecutionAttemptClaim;
      result: HistoricalOutcomeValidationEvaluationExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeValidationEvaluationOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_untrusted_envelope_count: number;
  failed_validation_count: number;
  future_per_target_candidate_admission_review_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeValidationEvaluationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_candidate_set_sha256: string;
  expected_upstream_validation_sha256: string;
  expected_upstream_output_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_rows_sha256: string;
  expected_excluded_rows_sha256: string;
  expected_target_commitments_sha256: string;
  expected_validation_projection_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  independent_reopen_and_second_implementation_recomputation_confirmed: true;
  exact_current_stage_51_through_stage_63_binding_confirmed: true;
  exact_validation_projection_and_nine_candidate_predictions_confirmed: true;
  all_eighty_one_metrics_fifty_four_hypotheses_and_nine_recommendations_bitwise_recomputed_confirmed: true;
  sealed_holdout_remains_unread_confirmed: true;
  no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true;
};

export type HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict =
  | "admitted_for_future_sealed_holdout_evaluation_protocol_review"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  target_id: string;
  output_validation_id: string;
  output_validation_sha256: string;
  target_bundle_sha256: string;
  recommendation_sha256: string;
  target_metric_count: number;
  target_algorithm_count: number;
  frozen_seed_count: number;
  recommendation_status: string;
  recommended_algorithm?: string;
  three_seed_median_mae_f64_bits_hex?: string;
  all_three_seeds_passed: boolean;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  per_target_candidate_admitted: boolean;
  future_sealed_holdout_evaluation_protocol_review_eligible: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      source: {
        attempt: {
          claim: HistoricalOutcomeValidationEvaluationExecutionAttemptClaim;
          result: HistoricalOutcomeValidationEvaluationExecutionAttemptResult;
        };
        validation: HistoricalOutcomeValidationEvaluationOutputValidationRecord;
      };
      target_id: string;
      target_bundle_sha256: string;
      recommendation_sha256: string;
      metrics: HistoricalOutcomeValidationEvaluationMetric[];
      recommendation: HistoricalOutcomeValidationEvaluationPerTargetRecommendation;
      exact_nine_metrics_three_algorithms_three_seeds: boolean;
      recommendation_admissible: boolean;
    };
    latest_review?: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview;
    review_eligible: boolean;
    per_target_candidate_admitted: boolean;
  }>;
  independently_validated_output_count: number;
  target_candidate_count: number;
  review_eligible_target_count: number;
  reviewed_target_count: number;
  admitted_target_count: number;
  changes_requested_or_rejected_target_count: number;
  insufficient_evidence_target_count: number;
  no_candidate_passed_target_count: number;
  future_sealed_holdout_evaluation_protocol_review_eligible_target_count: number;
  admission_status: string;
  per_target_candidate_admission_review_available: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_candidate_set_sha256: string;
  expected_upstream_validation_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_validation_projection_sha256: string;
  expected_target_bundle_sha256: string;
  expected_recommendation_sha256: string;
  verdict: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_64_binding_confirmed: true;
  exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed: true;
  target_evidence_status_and_thresholds_confirmed: true;
  recommended_algorithm_and_three_seed_median_confirmed: true;
  no_cross_target_composite_or_masking_confirmed: true;
  sealed_holdout_remains_unread_confirmed: true;
  next_gate_is_protocol_review_not_holdout_execution_confirmed: true;
  no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true;
};

export type HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict =
  | "approved_for_future_sealed_holdout_evaluation_implementation_registration"
  | "changes_requested"
  | "rejected";

export type HistoricalOutcomeSealedHoldoutEvaluationProtocol = {
  schema_version: string;
  protocol_version: string;
  protocol_sha256: string;
  attempt_id: string;
  target_id: string;
  stage_65_admission_review_id: string;
  stage_65_admission_review_sha256: string;
  output_validation_sha256: string;
  candidate_set_sha256: string;
  training_store_dataset_sha256: string;
  rows_sha256: string;
  target_commitments_sha256: string;
  validation_projection_sha256: string;
  target_bundle_sha256: string;
  recommendation_sha256: string;
  selected_algorithm_three_seed_binding_sha256: string;
  sealed_holdout_split_commitment_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  frozen_candidate_algorithm_id: string;
  exact_random_seeds: number[];
  exact_feature_count: number;
  exact_target_count: number;
  target_vector_order: string[];
  benchmark_algorithm_id: string;
  reported_metric_ids: string[];
  bootstrap_unit: string;
  bootstrap_replications: number;
  bootstrap_random_seed: number;
  family_wise_error_correction: string;
  family_wise_alpha_millionths: number;
  exact_candidate_hypothesis_count: number;
  minimum_relative_mae_improvement_ppm: number;
  minimum_spearman_millionths: number;
  minimum_directional_accuracy_millionths: number;
  minimum_calibration_slope_millionths: number;
  maximum_calibration_slope_millionths: number;
  minimum_sealed_holdout_rows: number;
  minimum_independent_components: number;
  all_three_seeds_must_pass: boolean;
  one_shot_evaluation_required: boolean;
  insufficient_sample_rule: string;
  confirmatory_decision_rule: string;
  no_feedback_reuse_rule: string;
  no_composite_score_or_cross_target_masking: boolean;
  protocol_review_only: boolean;
  callable_entrypoint_present: boolean;
  sealed_holdout_features_access_allowed: boolean;
  sealed_holdout_labels_access_allowed: boolean;
  training_or_preprocessing_update_allowed: boolean;
  hyperparameter_or_threshold_tuning_allowed: boolean;
  candidate_reselection_allowed: boolean;
  model_store_write_allowed: boolean;
  metric_store_write_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  outbound_network_allowed: boolean;
  secrets_allowed: boolean;
  scalar_reward_defined: boolean;
  action_position_or_ranking_semantics_defined: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationProtocolReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  target_id: string;
  stage_65_admission_review_id: string;
  stage_65_admission_review_sha256: string;
  protocol_version: string;
  protocol_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict;
  rationale: string;
  known_limitations: string;
  protocol_independently_approved: boolean;
  future_sealed_holdout_evaluation_implementation_registration_eligible: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  sealed_holdout_evaluation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    subject: {
      admitted: {
        candidate: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry["items"][number]["candidate"];
        admission_review: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview;
      };
      protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol;
    };
    latest_review?: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview;
    review_eligible: boolean;
    protocol_independently_approved: boolean;
  }>;
  admitted_target_count: number;
  protocol_review_eligible_count: number;
  protocol_reviewed_count: number;
  protocol_independently_approved_count: number;
  protocol_rejected_or_changes_requested_count: number;
  future_sealed_holdout_evaluation_implementation_registration_eligible_count: number;
  protocol_review_status: string;
  protocol_review_available: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  sealed_holdout_evaluation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_stage_65_admission_review_id: string;
  expected_stage_65_admission_review_sha256: string;
  expected_output_validation_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_target_bundle_sha256: string;
  expected_recommendation_sha256: string;
  expected_protocol_sha256: string;
  verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_65_binding_confirmed: true;
  reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: true;
  one_target_one_algorithm_three_frozen_seeds_only_confirmed: true;
  immutable_candidate_feature_preprocessing_and_target_confirmed: true;
  sealed_holdout_single_use_and_no_feedback_reuse_confirmed: true;
  fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: true;
  all_three_seeds_must_pass_and_failures_remain_visible_confirmed: true;
  insufficient_sample_fails_closed_confirmed: true;
  no_cross_target_composite_tuning_refit_or_reselection_confirmed: true;
  protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: true;
  next_gate_is_implementation_registration_not_data_access_confirmed: true;
  no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationContract = {
  schema_version: string;
  contract_sha256: string;
  implementation_protocol_version: string;
  implementation_artifact_sha256: string;
  immutable_code_revision: string;
  stage_66_protocol_review_id: string;
  stage_66_protocol_review_sha256: string;
  sealed_holdout_evaluation_protocol_sha256: string;
  stage_65_admission_review_sha256: string;
  output_validation_sha256: string;
  candidate_set_sha256: string;
  training_store_dataset_sha256: string;
  target_bundle_sha256: string;
  recommendation_sha256: string;
  selected_algorithm_three_seed_binding_sha256: string;
  sealed_holdout_split_commitment_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  exact_random_seeds: number[];
  exact_feature_count: number;
  exact_target_count: number;
  exact_candidate_hypothesis_count: number;
  reported_metric_ids: string[];
  bootstrap_unit: string;
  bootstrap_replications: number;
  bootstrap_random_seed: number;
  family_wise_error_correction: string;
  family_wise_alpha_millionths: number;
  minimum_relative_mae_improvement_ppm: number;
  minimum_spearman_millionths: number;
  minimum_directional_accuracy_millionths: number;
  minimum_calibration_slope_millionths: number;
  maximum_calibration_slope_millionths: number;
  minimum_sealed_holdout_rows: number;
  minimum_independent_components: number;
  all_three_seeds_must_pass: boolean;
  one_shot_evaluation_required: boolean;
  deterministic_evaluator_function_id: string;
  canonical_input_projection_schema: string;
  canonical_untrusted_output_schema: string;
  future_output_create_once: boolean;
  future_output_independent_validation_required: boolean;
  no_feedback_reuse: boolean;
  insufficient_sample_fails_closed: boolean;
  no_composite_score_or_cross_target_masking: boolean;
  independent_implementation_review_required: boolean;
  isolated_runner_registration_required: boolean;
  one_shot_access_authorization_required: boolean;
  callable_entrypoint_present: boolean;
  input_mount_present: boolean;
  sealed_holdout_data_adapter_present: boolean;
  sealed_holdout_features_access_allowed: boolean;
  sealed_holdout_labels_access_allowed: boolean;
  sealed_holdout_evaluation_allowed: boolean;
  training_or_preprocessing_update_allowed: boolean;
  hyperparameter_or_threshold_tuning_allowed: boolean;
  candidate_reselection_allowed: boolean;
  official_candidate_selection_allowed: boolean;
  model_store_write_allowed: boolean;
  metric_store_write_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  outbound_network_allowed: boolean;
  environment_inheritance_allowed: boolean;
  secrets_allowed: boolean;
  tools_allowed: boolean;
  subprocesses_allowed: boolean;
  scalar_reward_defined: boolean;
  action_position_or_ranking_semantics_defined: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol;
  upstream_protocol_review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_66_and_complete_prior_chain: boolean;
  implementation_name: string;
  rationale: string;
  known_limitations: string;
  implementation_contract: HistoricalOutcomeSealedHoldoutEvaluationImplementationContract;
  status: string;
  sealed_holdout_evaluation_implementation_registered: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  sealed_holdout_access_authorized: boolean;
  sealed_holdout_evaluation_authorized: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_protocols: Array<{
    protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol;
    protocol_review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview;
  }>;
  items: Array<{
    implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  callable_entrypoint_present: boolean;
  input_mount_present: boolean;
  sealed_holdout_access_authorized: boolean;
  sealed_holdout_evaluation_authorized: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest = {
  expected_protocol_review_id: string;
  expected_protocol_review_sha256: string;
  expected_protocol_sha256: string;
  expected_stage_65_admission_review_sha256: string;
  expected_output_validation_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_target_bundle_sha256: string;
  expected_recommendation_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_66_binding_confirmed: true;
  registrar_independent_from_stage_66_and_complete_prior_chain_confirmed: true;
  immutable_artifact_revision_protocol_and_serialization_confirmed: true;
  one_target_one_algorithm_three_frozen_seeds_only_confirmed: true;
  no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed: true;
  one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed: true;
  fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: true;
  no_tuning_refit_reselection_or_cross_target_composite_confirmed: true;
  future_output_create_once_untrusted_and_independent_validation_required_confirmed: true;
  independent_review_runner_and_one_shot_authorization_remain_separate_confirmed: true;
  no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict =
  | "approved_for_future_isolated_sealed_holdout_evaluation_runner_registration"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_sealed_holdout_evaluation_protocol_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_stage_66_protocol_review_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_67_chain_confirmed: boolean;
  reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_and_protocol_hashes_independently_reproduced_confirmed: boolean;
  exact_one_artifact_one_algorithm_three_seed_matrix_confirmed: boolean;
  exact_65_feature_one_target_and_metric_contract_confirmed: boolean;
  component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: boolean;
  minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: boolean;
  no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed: boolean;
  one_shot_no_feedback_create_once_untrusted_output_confirmed: boolean;
  independent_runner_authorization_and_output_validation_separation_confirmed: boolean;
  no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  sealed_holdout_evaluation_protocol_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  sealed_holdout_evaluation_protocol_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_67_binding_valid: boolean;
  exact_one_algorithm_three_seed_one_target_contract_valid: boolean;
  exact_65_feature_one_target_order_valid: boolean;
  per_target_per_seed_metric_contract_valid: boolean;
  paired_component_block_bootstrap_holm_contract_valid: boolean;
  minimum_effect_diagnostics_and_sample_gates_valid: boolean;
  all_three_seed_no_shopping_no_composite_contract_valid: boolean;
  one_shot_no_feedback_create_once_untrusted_output_contract_valid: boolean;
  all_access_evaluation_selection_store_reward_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
  independent_audit: HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict;
  rationale: string;
  known_limitations: string;
  sealed_holdout_evaluation_implementation_independently_approved: boolean;
  future_isolated_runner_registration_eligible: boolean;
  isolated_runner_registered: boolean;
  sealed_holdout_features_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
    current_independent_audit: HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review?: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord;
    review_eligible: boolean;
    future_isolated_runner_registration_eligible: boolean;
  }>;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_requested_or_rejected_count: number;
  future_isolated_runner_registration_eligible_count: number;
  review_status: string;
  sealed_holdout_features_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  official_candidate_selection_authorized: boolean;
  sealed_holdout_access_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind =
  "ephemeral_deterministic_one_target_three_seed_sealed_holdout_evaluator";

export type HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract = {
  schema_version: string;
  contract_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  stage_68_implementation_review_sha256: string;
  stage_67_implementation_sha256: string;
  stage_66_protocol_review_sha256: string;
  sealed_holdout_evaluation_protocol_sha256: string;
  selected_algorithm_three_seed_binding_sha256: string;
  sealed_holdout_split_commitment_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  exact_random_seeds: number[];
  canonical_input_projection_schema: string;
  canonical_untrusted_output_schema: string;
  input_mount_contract: string;
  output_contract: string;
  invocation_contract: string;
  next_gate: string;
  callable_entrypoint_registered: boolean;
  current_sealed_holdout_mount_present: boolean;
  current_candidate_artifact_mount_present: boolean;
  future_exact_sealed_holdout_read_only_mount_required: boolean;
  future_exact_three_candidate_artifact_read_only_mount_required: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  content_addressed_create_once_output_required: boolean;
  independent_output_validation_required: boolean;
  one_shot_evaluation_required: boolean;
  no_feedback_reuse_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  host_environment_inherited: boolean;
  allowed_environment_variables: string[];
  secrets_available: boolean;
  outbound_network_allowed: boolean;
  external_tools_allowed: boolean;
  child_process_allowed: boolean;
  sealed_holdout_features_access_allowed: boolean;
  sealed_holdout_labels_access_allowed: boolean;
  training_or_preprocessing_update_allowed: boolean;
  candidate_reselection_allowed: boolean;
  cross_target_read_or_composite_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  training_store_writes_allowed: boolean;
  model_artifact_store_writes_allowed: boolean;
  metric_store_writes_allowed: boolean;
  future_untrusted_one_target_three_seed_confirmation_envelope_required: boolean;
  no_composite_score_or_global_model_validity_claim_required: boolean;
  maximum_parallel_evaluations: number;
  maximum_memory_mib: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
  implementation_review: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_68_and_complete_prior_chain: boolean;
  runner_name: string;
  runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  runner_contract: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract;
  status: string;
  exact_current_stage_51_through_stage_68_binding_confirmed: boolean;
  registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: boolean;
  runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: boolean;
  future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: boolean;
  training_validation_cross_target_and_feedback_isolation_confirmed: boolean;
  one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: boolean;
  create_once_untrusted_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_access_authorization_execution_and_output_validation_separation_confirmed: boolean;
  no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  sealed_holdout_feature_access_authorized: boolean;
  sealed_holdout_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  official_candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_reviews: Array<{
    implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
    review: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord;
  }>;
  allowed_runner_kinds: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind[];
  registration_allowed: boolean;
  items: Array<{
    runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  runner_status: string;
  callable_entrypoint_registered: boolean;
  current_input_mount_present: boolean;
  first_execution_authorized: boolean;
  sealed_holdout_feature_access_authorized: boolean;
  sealed_holdout_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  official_candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_stage_66_protocol_review_sha256: string;
  expected_sealed_holdout_evaluation_protocol_sha256: string;
  expected_target_bundle_sha256: string;
  expected_recommendation_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  runner_name: string;
  runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_68_binding_confirmed: boolean;
  registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: boolean;
  runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: boolean;
  future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: boolean;
  training_validation_cross_target_and_feedback_isolation_confirmed: boolean;
  one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: boolean;
  create_once_untrusted_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_identity_and_bounded_resource_contract_confirmed: boolean;
  no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: boolean;
  registration_access_authorization_execution_and_output_validation_separation_confirmed: boolean;
  no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_sealed_holdout_evaluation_invocation"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_artifact_sha256: string;
  expected_immutable_code_revision: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_candidate_set_sha256: string;
  expected_stage_66_protocol_review_sha256: string;
  expected_sealed_holdout_evaluation_protocol_sha256: string;
  expected_target_bundle_sha256: string;
  expected_recommendation_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  verdict: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed: boolean;
  one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed: boolean;
  exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_selection_separation_confirmed: boolean;
  no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_invocation_limit: number;
  one_future_isolated_sealed_holdout_evaluation_invocation_authorized: boolean;
  authorization_claimed: boolean;
  invocation_endpoint_available: boolean;
  sealed_holdout_feature_access_authorized: boolean;
  sealed_holdout_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  other_target_or_unscoped_sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview;
    one_future_isolated_sealed_holdout_evaluation_invocation_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  invocation_endpoint_available: boolean;
  sealed_holdout_feature_access_authorized: boolean;
  sealed_holdout_label_access_authorized: boolean;
  evaluation_authorized: boolean;
  evaluation_started: boolean;
  evaluation_completed: boolean;
  candidate_selection_authorized: boolean;
  untrusted_output_created: boolean;
  output_validation_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  other_target_or_unscoped_sealed_holdout_access_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest = {
  expected_first_execution_authorization_review_id: string;
  expected_first_execution_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_protocol_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  claim_first_single_use_and_failure_consumes_confirmed: boolean;
  exact_one_target_one_algorithm_three_seed_projection_confirmed: boolean;
  sealed_holdout_only_and_no_other_partition_or_target_access_confirmed: boolean;
  frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed: boolean;
  no_feedback_tuning_refit_reselection_or_composite_confirmed: boolean;
  untrusted_content_addressed_output_and_independent_validation_confirmed: boolean;
  no_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_confirmation_envelope_count: number;
  independent_output_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: {
      attempt_id: string;
      claim_sha256: string;
      claimed_at: string;
      invoked_by: string;
      isolation_backend: string;
      authorization_review_sha256: string;
      isolated_runner_spec_sha256: string;
      implementation_sha256: string;
      implementation_review_sha256: string;
      implementation_independent_audit_sha256: string;
      protocol_sha256: string;
      candidate_set_sha256: string;
      training_store_dataset_sha256: string;
      selected_algorithm_three_seed_binding_sha256: string;
      sealed_holdout_split_commitment_sha256: string;
      target_id: string;
      frozen_candidate_algorithm_id: string;
      exact_random_seeds: number[];
    };
    result?: {
      result_sha256: string;
      output_sha256?: string;
      completed_at: string;
      status: "completed_with_untrusted_sealed_holdout_confirmation" | "failed_authorization_consumed";
      bounded_error?: string;
      untrusted_confirmation_envelope?: {
        sealed_holdout_projection_sha256: string;
        feature_order_sha256: string;
        preprocessing_sha256: string;
        target_id: string;
        frozen_candidate_algorithm_id: string;
        exact_random_seeds: number[];
        sealed_holdout_row_count: number;
        independent_component_count: number;
        exact_metric_count: number;
        exact_candidate_hypothesis_count: number;
        confirmation_status: string;
        all_three_seeds_passed: boolean;
        insufficient_evidence: boolean;
        metrics: Array<{
          random_seed: number;
          evidence_status: string;
          all_preregistered_thresholds_passed: boolean;
          mae_f64_bits_hex: string;
          relative_mae_improvement_f64_bits_hex: string;
          holm_adjusted_p_value_f64_bits_hex: string;
        }>;
      };
    };
  }>;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_review_sha256: string;
  expected_implementation_independent_audit_sha256: string;
  expected_protocol_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  expected_sealed_holdout_projection_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  independent_reopen_and_second_implementation_recomputation_confirmed: boolean;
  exact_current_stage_51_through_stage_71_binding_confirmed: boolean;
  claim_first_authorization_consumption_and_no_replay_confirmed: boolean;
  exact_one_target_one_algorithm_three_seed_prediction_recomputation_confirmed: boolean;
  exact_three_metrics_component_bootstrap_holm_and_thresholds_bitwise_recomputed_confirmed: boolean;
  output_remains_untrusted_pending_future_adjudication_confirmed: boolean;
  no_selection_store_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_sha256: string;
  output_sha256: string;
  candidate_set_sha256: string;
  training_store_dataset_sha256: string;
  selected_algorithm_three_seed_binding_sha256: string;
  sealed_holdout_split_commitment_sha256: string;
  sealed_holdout_projection_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  validated_at: string;
  validated_by: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  recomputed_metric_count: number;
  recomputed_candidate_hypothesis_count: number;
  recomputed_all_three_seeds_passed: boolean;
  recomputed_insufficient_evidence: boolean;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_untrusted_sealed_holdout_confirmation"
    | "failed_independent_sealed_holdout_output_validation";
  sealed_holdout_confirmation_independently_validated: boolean;
  future_confirmatory_result_adjudication_review_eligible: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry["attempts"][number] & {
      result: NonNullable<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry["attempts"][number]["result"]>;
    };
    validation?: HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_untrusted_confirmation_count: number;
  failed_validation_count: number;
  future_confirmatory_result_adjudication_review_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict =
  | "approved_for_future_controlled_shadow_experiment_design_registration"
  | "changes_requested"
  | "rejected";

export type ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_envelope_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_sealed_holdout_projection_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  verdict: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict;
  statistical_interpretation: string;
  economic_interpretation: string;
  known_limitations: string;
  falsification_conditions: string;
  next_experiment_constraints: string;
  exact_current_stage_51_through_stage_72_binding_confirmed: boolean;
  stage_72_second_implementation_reproducibility_confirmed: boolean;
  exact_one_target_one_algorithm_three_frozen_seeds_confirmed: boolean;
  all_three_preregistered_seed_tests_and_thresholds_reviewed: boolean;
  sample_component_and_multiple_testing_sufficiency_reviewed: boolean;
  target_semantics_and_economic_relevance_reviewed: boolean;
  effect_size_not_p_value_only_reviewed: boolean;
  data_coverage_selection_bias_and_failure_modes_reviewed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  reproducibility_not_profitability_or_generalization_confirmed: boolean;
  approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: boolean;
  no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  output_validation_id: string;
  output_validation_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict;
  statistical_interpretation: string;
  economic_interpretation: string;
  known_limitations: string;
  falsification_conditions: string;
  next_experiment_constraints: string;
  quantitative_approval_eligible: boolean;
  quantitative_ineligibility_reasons: string[];
  confirmatory_result_adjudicated: boolean;
  future_controlled_shadow_experiment_design_registration_eligible: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      source: {
        attempt: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry["attempts"][number] & {
          result: NonNullable<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry["attempts"][number]["result"]>;
        };
        validation: HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord;
      };
      envelope_sha256: string;
      confirmation_status: string;
      sealed_holdout_row_count: number;
      independent_component_count: number;
      metric_count: number;
      all_three_seeds_passed: boolean;
      insufficient_evidence: boolean;
      quantitative_approval_eligible: boolean;
      quantitative_ineligibility_reasons: string[];
    };
    latest_review?: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview;
    review_eligible: boolean;
    confirmatory_result_adjudicated: boolean;
  }>;
  candidate_count: number;
  quantitative_pass_count: number;
  quantitative_fail_or_insufficient_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  changes_requested_or_rejected_count: number;
  future_controlled_shadow_experiment_design_registration_eligible_count: number;
  adjudication_status: string;
  adjudication_review_available: boolean;
  official_candidate_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowExperimentDesignRequest = {
  expected_adjudication_review_id: string;
  expected_adjudication_review_sha256: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_envelope_sha256: string;
  expected_candidate_set_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  experiment_name: string;
  research_hypothesis: string;
  economic_thesis: string;
  known_limitations: string;
  falsification_conditions: string;
  exact_stage_73_adjudication_and_complete_chain_confirmed: boolean;
  registrar_independent_from_complete_prior_chain_confirmed: boolean;
  experimental_candidate_not_official_model_selection_confirmed: boolean;
  point_in_time_forward_only_and_no_retroactive_revision_confirmed: boolean;
  benchmark_comparators_costs_and_rebalance_frozen_confirmed: boolean;
  portfolio_caps_cash_floor_and_long_only_boundary_confirmed: boolean;
  minimum_observation_windows_and_no_early_promotion_confirmed: boolean;
  separate_metrics_multiple_testing_and_no_composite_confirmed: boolean;
  stop_rules_and_falsification_are_frozen_confirmed: boolean;
  independent_design_review_required_before_any_shadow_run_request_confirmed: boolean;
  no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: boolean;
};

export type ControlledShadowExperimentDesignRegistration = {
  schema_version: string;
  policy_version: string;
  registration_id: string;
  registration_sha256: string;
  attempt_id: string;
  adjudication_review_id: string;
  adjudication_review_sha256: string;
  output_validation_id: string;
  output_validation_sha256: string;
  claim_sha256: string;
  result_sha256: string;
  output_sha256: string;
  envelope_sha256: string;
  candidate_set_sha256: string;
  training_store_dataset_sha256: string;
  selected_algorithm_three_seed_binding_sha256: string;
  sealed_holdout_split_commitment_sha256: string;
  sealed_holdout_projection_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  experiment_name: string;
  research_hypothesis: string;
  economic_thesis: string;
  known_limitations: string;
  falsification_conditions: string;
  registered_at: string;
  registered_by: string;
  design_specification: {
    schema_version: string;
    specification_sha256: string;
    experimental_candidate_only: boolean;
    target_id: string;
    frozen_candidate_algorithm_id: string;
    random_seeds: number[];
    candidate_set_sha256: string;
    feature_order_sha256: string;
    preprocessing_sha256: string;
    benchmark_symbol: string;
    comparator_ids: string[];
    universe_contract: string;
    signal_contract: string;
    portfolio_constraints: {
      virtual_notional_usd: number;
      long_only: boolean;
      common_stock_only: boolean;
      options_allowed: boolean;
      leverage_allowed: boolean;
      shorting_allowed: boolean;
      maximum_single_name_weight_bps: number;
      maximum_theme_weight_bps: number;
      maximum_gross_exposure_bps: number;
      minimum_cash_weight_bps: number;
      maximum_position_count: number;
    };
    execution_contract: {
      signal_cutoff: string;
      assumed_execution: string;
      rebalance_frequency: string;
      slippage_bps_per_side: number;
      point_in_time_data_only: boolean;
      lookahead_or_retroactive_revision_allowed: boolean;
    };
    observation_contract: {
      minimum_forward_market_sessions: number;
      checkpoint_market_sessions: number[];
      minimum_independent_signal_count: number;
      minimum_distinct_symbol_count: number;
      minimum_distinct_market_quarter_count: number;
      early_promotion_allowed: boolean;
    };
    metric_contract: {
      metric_ids: string[];
      composite_score_allowed: boolean;
      all_metrics_reported_separately: boolean;
      multiple_testing_adjustment_required: boolean;
      transaction_costs_included: boolean;
    };
    stop_contract: {
      stop_rule_ids: string[];
      automatic_trade_or_position_action_allowed: boolean;
      stopped_design_can_be_restarted_in_place: boolean;
    };
    scalar_reward_defined: boolean;
    official_model_selected: boolean;
    model_artifact_materialized: boolean;
    shadow_ledger_enabled: boolean;
  };
  controlled_shadow_experiment_design_registered: boolean;
  future_independent_design_review_eligible: boolean;
  design_independently_approved: boolean;
  official_model_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentDesignRegistrationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    source: {
      candidate: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry["items"][number]["candidate"];
      review: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview;
    };
    registration?: ControlledShadowExperimentDesignRegistration;
    registration_eligible: boolean;
  }>;
  adjudicated_candidate_count: number;
  registration_eligible_count: number;
  registered_design_count: number;
  future_independent_design_review_eligible_count: number;
  registration_status: string;
  design_registration_available: boolean;
  official_model_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowExperimentDesignRegistrationReviewVerdict =
  | "approved_for_future_zero_capability_shadow_implementation_registration"
  | "changes_requested_requires_new_design_registration"
  | "rejected";

export type ReviewControlledShadowExperimentDesignRegistrationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_adjudication_review_id: string;
  expected_adjudication_review_sha256: string;
  expected_output_validation_id: string;
  expected_output_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_envelope_sha256: string;
  expected_candidate_set_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_design_specification_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict;
  rationale: string;
  risk_assessment: string;
  known_limitations: string;
  falsification_assessment: string;
  future_implementation_constraints: string;
  exact_current_stage_51_through_stage_74_binding_confirmed: boolean;
  independent_recomputation_of_registration_and_design_fingerprints_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  experimental_candidate_not_official_model_selection_confirmed: boolean;
  point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: boolean;
  benchmark_and_all_counterfactual_semantics_reviewed: boolean;
  signal_timing_execution_cost_dividends_and_rebalance_reviewed: boolean;
  long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: boolean;
  minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: boolean;
  separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: boolean;
  stop_rules_falsification_and_no_in_place_restart_reviewed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed: boolean;
  no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: boolean;
};

export type ControlledShadowExperimentDesignRegistrationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  attempt_id: string;
  registration_id: string;
  registration_sha256: string;
  design_specification_sha256: string;
  independently_recomputed_registration_sha256: string;
  independently_recomputed_design_specification_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict;
  rationale: string;
  risk_assessment: string;
  known_limitations: string;
  falsification_assessment: string;
  future_implementation_constraints: string;
  design_registration_independently_approved: boolean;
  future_zero_capability_shadow_implementation_registration_eligible: boolean;
  shadow_implementation_registered: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentDesignRegistrationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    registered_design: {
      source: ControlledShadowExperimentDesignRegistrationRegistry["items"][number]["source"];
      registration: ControlledShadowExperimentDesignRegistration;
    };
    latest_review?: ControlledShadowExperimentDesignRegistrationReview;
    review_eligible: boolean;
    independently_approved: boolean;
  }>;
  registered_design_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_requested_or_rejected_count: number;
  future_zero_capability_shadow_implementation_registration_eligible_count: number;
  review_status: string;
  independent_review_available: boolean;
  official_model_selection_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_implementation_registered: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowExperimentImplementationRequest = {
  expected_design_review_id: string;
  expected_design_review_sha256: string;
  expected_design_registration_id: string;
  expected_design_registration_sha256: string;
  expected_design_specification_sha256: string;
  expected_adjudication_review_sha256: string;
  expected_output_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_envelope_sha256: string;
  expected_candidate_set_sha256: string;
  expected_training_store_dataset_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_sealed_holdout_projection_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_replay_notes: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_75_binding_confirmed: boolean;
  registrar_independent_from_stage_75_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_design_review_registration_and_specification_confirmed: boolean;
  zero_capability_specification_only_not_executable_artifact_confirmed: boolean;
  point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed: boolean;
  signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed: boolean;
  long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed: boolean;
  observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed: boolean;
  deterministic_create_once_content_addressed_replay_contract_confirmed: boolean;
  no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_store_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  no_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_required_before_runner_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowExperimentImplementationRecord = {
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_design_registration: ControlledShadowExperimentDesignRegistration;
  upstream_design_review: ControlledShadowExperimentDesignRegistrationReview;
  implementation_name: string;
  implementation_description: string;
  deterministic_replay_notes: string;
  known_limitations: string;
  future_review_constraints: string;
  status: string;
  implementation_contract: {
    contract_sha256: string;
    immutable_code_revision: string;
    stage_75_design_review_sha256: string;
    stage_74_design_registration_sha256: string;
    design_specification_sha256: string;
    candidate_set_sha256: string;
    selected_algorithm_three_seed_binding_sha256: string;
    sealed_holdout_split_commitment_sha256: string;
    feature_order_sha256: string;
    preprocessing_sha256: string;
    target_id: string;
    frozen_candidate_algorithm_id: string;
    random_seeds: number[];
    exact_design_specification: ControlledShadowExperimentDesignRegistration["design_specification"];
    registered_not_run: boolean;
    callable_entrypoint_present: boolean;
    executable_artifact_present: boolean;
    runtime_present: boolean;
    outbound_network_allowed: boolean;
    production_reads_allowed: boolean;
    production_writes_allowed: boolean;
    shadow_run_allowed: boolean;
    shadow_ledger_creation_allowed: boolean;
    shadow_position_write_allowed: boolean;
    order_generation_allowed: boolean;
    broker_access_allowed: boolean;
    trading_allowed: boolean;
  };
  zero_capability_shadow_implementation_registered: boolean;
  future_independent_shadow_implementation_review_eligible: boolean;
  independent_shadow_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_designs: Array<{
    design_registration: ControlledShadowExperimentDesignRegistration;
    design_review: ControlledShadowExperimentDesignRegistrationReview;
  }>;
  items: Array<{
    implementation: ControlledShadowExperimentImplementationRecord;
    upstream_binding_current: boolean;
    future_independent_shadow_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  callable_entrypoint_present: boolean;
  executable_artifact_present: boolean;
  runtime_present: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowExperimentImplementationReviewVerdict =
  | "approved_for_future_isolated_shadow_runner_specification_registration"
  | "changes_requested"
  | "rejected";

export type ControlledShadowExperimentImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  design_review_sha256: string;
  design_registration_sha256: string;
  design_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  design_review_hash_independently_reproduced: boolean;
  design_registration_hash_independently_reproduced: boolean;
  design_specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_76_binding_valid: boolean;
  deterministic_replay_function_and_schema_contract_valid: boolean;
  point_in_time_universe_delisting_and_no_lookahead_contract_valid: boolean;
  execution_cost_dividend_rebalance_and_counterfactual_contract_valid: boolean;
  long_only_caps_cash_floor_and_instrument_boundary_valid: boolean;
  observation_checkpoint_metric_multiple_testing_and_stop_contract_valid: boolean;
  create_once_untrusted_output_and_no_order_payload_contract_valid: boolean;
  all_runtime_store_feedback_shadow_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ReviewControlledShadowExperimentImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_review_sha256: string;
  expected_design_registration_sha256: string;
  expected_design_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowExperimentImplementationReviewVerdict;
  rationale: string;
  implementation_verification_notes: string;
  risk_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  exact_current_stage_51_through_stage_76_binding_confirmed: boolean;
  reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed: boolean;
  pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: boolean;
  point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: boolean;
  signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: boolean;
  long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: boolean;
  observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: boolean;
  deterministic_stop_falsification_and_no_in_place_restart_confirmed: boolean;
  future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  no_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowExperimentImplementationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: ControlledShadowExperimentImplementationRecord;
  independent_audit: ControlledShadowExperimentImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowExperimentImplementationReviewVerdict;
  rationale: string;
  implementation_verification_notes: string;
  risk_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  reviewer_independent_from_stage_76_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_76_binding_confirmed: boolean;
  reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed: boolean;
  pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: boolean;
  point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: boolean;
  signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: boolean;
  long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: boolean;
  observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: boolean;
  deterministic_stop_falsification_and_no_in_place_restart_confirmed: boolean;
  future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  no_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  zero_capability_implementation_independently_approved: boolean;
  future_isolated_shadow_runner_specification_registration_eligible: boolean;
  isolated_shadow_runner_registered: boolean;
  runner_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mount_present: boolean;
  production_read_authorized: boolean;
  production_write_authorized: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: ControlledShadowExperimentImplementationRecord;
    current_independent_audit: ControlledShadowExperimentImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review?: ControlledShadowExperimentImplementationReviewRecord;
    review_eligible: boolean;
    future_isolated_shadow_runner_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_requested_or_rejected_count: number;
  future_isolated_shadow_runner_specification_registration_eligible_count: number;
  review_status: string;
  isolated_shadow_runner_registered: boolean;
  runner_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowExperimentIsolatedRunnerKind =
  "ephemeral_deterministic_forward_replay_specification";

export type RegisterControlledShadowExperimentIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_design_review_sha256: string;
  expected_design_registration_sha256: string;
  expected_design_specification_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  runner_name: string;
  runner_kind: ControlledShadowExperimentIsolatedRunnerKind;
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  future_mount_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_77_binding_confirmed: boolean;
  registrar_independent_from_stage_77_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_design_hashes_reproduced_confirmed: boolean;
  runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: boolean;
  no_callable_entrypoint_or_current_mount_confirmed: boolean;
  future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  future_create_once_untrusted_independently_validated_output_confirmed: boolean;
  deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  no_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_independent_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowExperimentIsolatedRunnerContract = {
  schema_version: string;
  contract_sha256: string;
  stage_77_implementation_review_id: string;
  stage_77_implementation_review_sha256: string;
  stage_77_independent_audit_sha256: string;
  stage_76_implementation_id: string;
  stage_76_implementation_sha256: string;
  stage_76_implementation_contract_sha256: string;
  stage_75_design_review_sha256: string;
  stage_74_design_registration_sha256: string;
  design_specification_sha256: string;
  exact_approved_implementation_contract: ControlledShadowExperimentImplementationRecord["implementation_contract"];
  runner_spec_revision: string;
  runtime_identity: string;
  runtime_version: string;
  future_input_envelope: string;
  future_output_envelope: string;
  next_gate: string;
  specification_registered: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mount_present: boolean;
  data_access_authorized: boolean;
  root_filesystem_read_only_required: boolean;
  ephemeral_working_directory_required: boolean;
  run_as_unprivileged_required: boolean;
  no_new_privileges_required: boolean;
  future_input_read_only_required: boolean;
  future_input_point_in_time_required: boolean;
  future_input_content_addressed_required: boolean;
  future_input_allowlisted_required: boolean;
  future_output_create_once_required: boolean;
  future_output_untrusted_required: boolean;
  future_output_independent_validation_required: boolean;
  future_output_order_intent_allowed: boolean;
  future_output_broker_payload_allowed: boolean;
  environment_inheritance_allowed: boolean;
  allowed_environment_variables: string[];
  secrets_allowed: boolean;
  outbound_network_allowed: boolean;
  tools_allowed: boolean;
  subprocesses_allowed: boolean;
  production_reads_allowed: boolean;
  production_writes_allowed: boolean;
  model_store_writes_allowed: boolean;
  metric_store_writes_allowed: boolean;
  training_feedback_allowed: boolean;
  scalar_reward_defined: boolean;
  maximum_parallel_runs: number;
  maximum_memory_mib: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type ControlledShadowExperimentIsolatedRunnerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: ControlledShadowExperimentImplementationRecord;
  implementation_review: ControlledShadowExperimentImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_77_and_complete_prior_chain: boolean;
  runner_name: string;
  runner_kind: ControlledShadowExperimentIsolatedRunnerKind;
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  rationale: string;
  known_limitations: string;
  future_mount_constraints: string;
  future_output_constraints: string;
  runner_contract: ControlledShadowExperimentIsolatedRunnerContract;
  status: string;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  shadow_run_started: boolean;
  shadow_run_completed: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: ControlledShadowExperimentImplementationRecord;
    review: ControlledShadowExperimentImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  allowed_runner_kinds: ControlledShadowExperimentIsolatedRunnerKind[];
  items: Array<{
    runner: ControlledShadowExperimentIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_status: string;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mount_present: boolean;
  shadow_run_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowExperimentFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_isolated_controlled_shadow_execution_attempt"
  | "changes_requested"
  | "rejected";

export type ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_spec_revision: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_design_review_sha256: string;
  expected_design_registration_sha256: string;
  expected_design_specification_sha256: string;
  expected_selected_algorithm_three_seed_binding_sha256: string;
  expected_sealed_holdout_split_commitment_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  verdict: ControlledShadowExperimentFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_78_binding_confirmed: boolean;
  reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: boolean;
  runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  no_callable_entrypoint_or_current_mount_confirmed: boolean;
  future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed: boolean;
  deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_claim_execution_and_output_validation_separation_confirmed: boolean;
  no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowExperimentFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: ControlledShadowExperimentIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowExperimentFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_78_binding_confirmed: boolean;
  reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: boolean;
  runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  no_callable_entrypoint_or_current_mount_confirmed: boolean;
  future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed: boolean;
  deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_model_metric_store_training_feedback_composite_or_reward_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_claim_execution_and_output_validation_separation_confirmed: boolean;
  no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  one_shot_execution_attempt_limit: number;
  one_future_isolated_controlled_shadow_execution_attempt_authorized: boolean;
  authorization_claimed: boolean;
  execution_attempt_endpoint_available: boolean;
  input_manifest_attached: boolean;
  point_in_time_input_access_authorized: boolean;
  shadow_execution_authorized: boolean;
  shadow_run_started: boolean;
  shadow_run_completed: boolean;
  untrusted_output_created: boolean;
  independent_output_validation_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: ControlledShadowExperimentIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: ControlledShadowExperimentFirstExecutionAuthorizationReview;
    one_future_isolated_controlled_shadow_execution_attempt_authorized: boolean;
    authorization_unexpired: boolean;
    execution_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  execution_attempt_eligible_count: number;
  authorization_status: string;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mount_present: boolean;
  execution_attempt_endpoint_available: boolean;
  input_manifest_attached: boolean;
  point_in_time_input_access_authorized: boolean;
  shadow_execution_authorized: boolean;
  shadow_run_started: boolean;
  shadow_run_completed: boolean;
  untrusted_output_created: boolean;
  independent_output_validation_authorized: boolean;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowPointInTimeInputEnvelope = {
  schema_version: string;
  input_manifest_sha256: string;
  candidate_set_sha256: string;
  feature_order: string[];
  preprocessing_sha256: string;
  signal_cutoff_at: string;
  captured_at: string;
  expected_next_full_market_session_at: string;
  benchmark_symbol: string;
  benchmark_adjusted_close_f64_bits_hex: string;
  sources: Array<{
    source_kind:
      | "sec_filing"
      | "company_investor_relations"
      | "licensed_market_data"
      | "exchange_official_data"
      | "government_official_data";
    source_id: string;
    content_sha256: string;
    available_at: string;
  }>;
  rows: Array<{
    symbol: string;
    frozen_theme_id: string;
    security_type: string;
    available_at: string;
    eligible_in_frozen_universe: boolean;
    tradable_at_signal_cutoff: boolean;
    adjusted_close_f64_bits_hex: string;
    feature_values_f64_bits_hex: Array<string | null>;
    source_content_sha256s: string[];
  }>;
  point_in_time_read_only: boolean;
  content_addressed: boolean;
  allowlisted_sources_only: boolean;
  no_retroactive_revision: boolean;
};

export type InvokeControlledShadowExperimentOnceRequest = {
  expected_authorization_review_id: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_specification_sha256: string;
  expected_candidate_set_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  expected_input_manifest_sha256: string;
  input: ControlledShadowPointInTimeInputEnvelope;
  claim_first_single_use_and_failure_consumes_confirmed: boolean;
  exact_stage_51_through_stage_79_binding_confirmed: boolean;
  current_binary_digest_reverification_after_claim_confirmed: boolean;
  point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  deterministic_three_seed_long_only_initialization_confirmed: boolean;
  no_future_performance_or_checkpoint_fabrication_confirmed: boolean;
  create_once_untrusted_output_requires_independent_validation_confirmed: boolean;
  no_ledger_position_order_broker_or_trading_confirmed: boolean;
  no_model_metric_store_feedback_composite_or_reward_confirmed: boolean;
};

export type ControlledShadowExperimentExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  isolation_backend: string;
  invocation_endpoint_available: boolean;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_attempt_count: number;
  failed_attempt_count: number;
  untrusted_initial_observation_count: number;
  independent_output_validation_eligible_count: number;
  execution_status: string;
  attempts: Array<{
    claim: {
      attempt_id: string;
      claim_sha256: string;
      authorization_review_id: string;
      isolated_runner_id: string;
      runner_artifact_sha256: string;
      input_manifest_sha256: string;
      claimed_at: string;
      invoked_by: string;
      authorization_consumed: boolean;
    };
    result?: {
      result_id: string;
      result_sha256: string;
      status:
        | "completed_with_untrusted_initial_observation"
        | "failed_authorization_consumed";
      output_sha256?: string;
      bounded_error?: string;
      initialization_completed: boolean;
      independent_output_validation_completed: boolean;
      shadow_ledger_created: boolean;
      shadow_position_written: boolean;
      order_generated: boolean;
      broker_accessed: boolean;
      trade_executed: boolean;
    };
  }>;
  shadow_ledger_enabled: boolean;
  shadow_position_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateControlledShadowExperimentOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_specification_sha256: string;
  expected_candidate_set_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  expected_input_manifest_sha256: string;
  input: ControlledShadowPointInTimeInputEnvelope;
  independent_reopen_and_second_implementation_recomputation_confirmed: boolean;
  exact_current_stage_51_through_stage_80_binding_confirmed: boolean;
  validator_independent_from_executor_and_complete_prior_chain_confirmed: boolean;
  exact_content_addressed_point_in_time_input_resubmitted_confirmed: boolean;
  exact_three_seed_predictions_ranking_and_five_caps_recomputed_confirmed: boolean;
  zero_forward_sessions_and_no_performance_fabrication_confirmed: boolean;
  validated_output_remains_untrusted_pending_forward_observation_confirmed: boolean;
  no_ledger_position_store_feedback_reward_order_broker_or_trading_confirmed: boolean;
};

export type ControlledShadowExperimentOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  input_manifest_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  excluded_prior_actor_ids: string[];
  validator_independent_from_execution_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_80_chain_verified: boolean;
  claim_fingerprint_independently_verified: boolean;
  result_fingerprint_independently_verified: boolean;
  original_envelope_fingerprint_independently_verified: boolean;
  input_manifest_fingerprint_independently_verified: boolean;
  exact_training_artifact_and_frozen_contract_verified: boolean;
  exact_three_seed_predictions_bitwise_recomputed: boolean;
  exact_ranking_and_tie_break_recomputed: boolean;
  single_name_theme_gross_cash_and_position_caps_recomputed: boolean;
  zero_forward_sessions_and_no_performance_verified: boolean;
  no_downstream_authority_verified: boolean;
  independently_recomputed_output_sha256: string;
  independently_recomputed_allocation_count: number;
  independently_recomputed_virtual_gross_exposure_bps: number;
  independently_recomputed_virtual_cash_weight_bps: number;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_untrusted_initial_observation"
    | "failed_independent_initial_observation_validation";
  initial_observation_independently_validated: boolean;
  future_forward_observation_protocol_registration_eligible: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowExperimentOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: {
        attempt_id: string;
        claim_sha256: string;
        authorization_review_sha256: string;
        isolated_runner_spec_sha256: string;
        runner_artifact_sha256: string;
        implementation_contract_sha256: string;
        design_specification_sha256: string;
        candidate_set_sha256: string;
        feature_order_sha256: string;
        preprocessing_sha256: string;
        target_id: string;
        frozen_candidate_algorithm_id: string;
        input_manifest_sha256: string;
        invoked_by: string;
      };
      result: {
        result_id: string;
        result_sha256: string;
        output_sha256?: string;
        completed_at: string;
      };
    };
    validation?: ControlledShadowExperimentOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_initial_observation_count: number;
  failed_validation_count: number;
  future_forward_observation_protocol_registration_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowForwardObservationProtocolRequest = {
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_input_manifest_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_specification_sha256: string;
  expected_candidate_set_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  protocol_rationale: string;
  source_custody_plan: string;
  market_calendar_plan: string;
  corporate_action_correction_policy: string;
  stop_execution_plan: string;
  exact_stage_51_through_stage_81_binding_confirmed: boolean;
  registrar_independent_from_stage_81_and_complete_prior_chain_confirmed: boolean;
  natural_forward_only_no_backfill_confirmed: boolean;
  weekly_claim_first_content_addressed_observation_confirmed: boolean;
  official_us_market_calendar_and_spy_synchronization_confirmed: boolean;
  point_in_time_allowlisted_source_custody_confirmed: boolean;
  adjusted_prices_dividends_and_append_only_corrections_confirmed: boolean;
  next_full_session_fill_and_registered_costs_confirmed: boolean;
  checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed: boolean;
  stop_rules_fail_closed_and_no_in_place_restart_confirmed: boolean;
  independent_protocol_review_required_before_observation_confirmed: boolean;
  no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed: boolean;
};

export type ControlledShadowForwardObservationProtocolRegistration = {
  protocol_registration_id: string;
  protocol_registration_sha256: string;
  attempt_id: string;
  validation_id: string;
  validation_sha256: string;
  claim_sha256: string;
  result_sha256: string;
  output_sha256: string;
  input_manifest_sha256: string;
  authorization_review_sha256: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  implementation_contract_sha256: string;
  design_specification_sha256: string;
  candidate_set_sha256: string;
  feature_order_sha256: string;
  preprocessing_sha256: string;
  target_id: string;
  frozen_candidate_algorithm_id: string;
  registered_at: string;
  registered_by: string;
  protocol_rationale: string;
  protocol_specification: {
    specification_sha256: string;
    observation_not_before: string;
    signal_cadence: string;
    official_market_calendar: string;
    benchmark_symbol: string;
    price_basis: string;
    performance_before_natural_checkpoint_allowed: boolean;
    forward_observation_started: boolean;
    ledger_created: boolean;
  };
  future_independent_protocol_review_eligible: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationProtocolRegistrationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    source: {
      attempt: ControlledShadowExperimentOutputValidationRegistry["items"][number]["attempt"];
      validation: ControlledShadowExperimentOutputValidationRecord;
    };
    registration?: ControlledShadowForwardObservationProtocolRegistration;
    registration_eligible: boolean;
  }>;
  protocol_registration_eligible_count: number;
  protocol_registered_count: number;
  current_binding_count: number;
  future_independent_protocol_review_eligible_count: number;
  protocol_registration_status: string;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowForwardObservationProtocolRegistrationReviewVerdict =
  | "approved_for_future_zero_capability_forward_observation_implementation_registration"
  | "changes_required_rebuild_forward_observation_protocol"
  | "rejected_forward_observation_protocol";

export type ReviewControlledShadowForwardObservationProtocolRegistrationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_protocol_registration_id: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_input_manifest_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_specification_sha256: string;
  expected_candidate_set_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  verdict: ControlledShadowForwardObservationProtocolRegistrationReviewVerdict;
  rationale: string;
  natural_forward_assessment: string;
  calendar_and_timing_assessment: string;
  source_custody_and_correction_assessment: string;
  metric_and_stop_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  exact_current_stage_51_through_stage_82_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed: boolean;
  observation_not_before_and_no_retroactive_backfill_reviewed: boolean;
  weekly_claim_first_create_once_reviewed: boolean;
  official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: boolean;
  point_in_time_allowlist_content_addressing_and_source_availability_reviewed: boolean;
  raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed: boolean;
  next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: boolean;
  long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: boolean;
  checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: boolean;
  separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: boolean;
  stop_falsification_fail_closed_and_no_in_place_restart_reviewed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed: boolean;
};

export type ControlledShadowForwardObservationProtocolRegistrationReview = {
  review_id: string;
  review_sha256: string;
  submitted_at: string;
  reviewer_id: string;
  verdict: ControlledShadowForwardObservationProtocolRegistrationReviewVerdict;
  rationale: string;
  natural_forward_assessment: string;
  calendar_and_timing_assessment: string;
  source_custody_and_correction_assessment: string;
  metric_and_stop_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  protocol_registration_independently_approved: boolean;
  future_zero_capability_forward_observation_implementation_registration_eligible: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationProtocolRegistrationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    registered_protocol: {
      source: ControlledShadowForwardObservationProtocolRegistrationRegistry["items"][number]["source"];
      registration: ControlledShadowForwardObservationProtocolRegistration;
    };
    latest_review?: ControlledShadowForwardObservationProtocolRegistrationReview;
    review_eligible: boolean;
    independently_approved: boolean;
  }>;
  protocol_registered_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_zero_capability_forward_observation_implementation_registration_eligible_count: number;
  review_status: string;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowForwardObservationImplementationRequest = {
  expected_protocol_review_id: string;
  expected_protocol_review_sha256: string;
  expected_protocol_registration_id: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_validation_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_input_manifest_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_design_specification_sha256: string;
  expected_candidate_set_sha256: string;
  expected_feature_order_sha256: string;
  expected_preprocessing_sha256: string;
  expected_target_id: string;
  expected_frozen_candidate_algorithm_id: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_observation_semantics: string;
  evidence_custody_and_correction_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_83_binding_confirmed: boolean;
  registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_review_registration_protocol_and_design_confirmed: boolean;
  zero_capability_specification_only_no_executable_artifact_confirmed: boolean;
  natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: boolean;
  weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: boolean;
  official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: boolean;
  next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: boolean;
  checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed: boolean;
  deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed: boolean;
  no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: boolean;
  no_production_read_write_observation_ledger_position_or_performance_write_confirmed: boolean;
  no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_required_before_runner_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationImplementationRecord = {
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation_name: string;
  implementation_description: string;
  deterministic_observation_semantics: string;
  evidence_custody_and_correction_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  status: "registered_not_reviewed_not_run";
  implementation_contract: {
    contract_sha256: string;
    validation_sha256: string;
    immutable_code_revision: string;
    stage_83_protocol_review_id: string;
    stage_82_protocol_registration_id: string;
    target_id: string;
    deterministic_weekly_claim_function_id: string;
    deterministic_market_calendar_function_id: string;
    deterministic_point_in_time_source_custody_function_id: string;
    deterministic_corporate_action_correction_function_id: string;
    deterministic_signal_projection_function_id: string;
    deterministic_portfolio_transition_function_id: string;
    deterministic_fill_cost_and_counterfactual_function_id: string;
    deterministic_checkpoint_metric_and_stop_function_id: string;
    registered_not_run: boolean;
    independent_implementation_review_required: boolean;
    isolated_runner_registration_required_after_review: boolean;
    authority_boundary: Record<string, boolean>;
  };
  future_independent_implementation_review_eligible: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    source: {
      registered_protocol: ControlledShadowForwardObservationProtocolRegistrationReviewRegistry["items"][number]["registered_protocol"];
      review: ControlledShadowForwardObservationProtocolRegistrationReview;
    };
    implementation?: ControlledShadowForwardObservationImplementationRecord;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  registration_eligible_count: number;
  implementation_count: number;
  current_binding_implementation_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  callable_entrypoint_present: boolean;
  executable_artifact_present: boolean;
  runtime_present: boolean;
  input_mount_present: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  model_artifact_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowForwardObservationImplementationReviewVerdict =
  | "approved_for_future_isolated_forward_observation_runner_specification_registration"
  | "changes_required_rebuild_forward_observation_implementation"
  | "rejected_forward_observation_implementation";

export type ReviewControlledShadowForwardObservationImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_protocol_review_sha256: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowForwardObservationImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_semantics_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  exact_current_stage_51_through_stage_84_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed: boolean;
  natural_forward_no_backfill_and_observation_not_before_confirmed: boolean;
  weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: boolean;
  signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: boolean;
  future_schema_names_uninstantiated_confirmed: boolean;
  no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: boolean;
  no_production_read_write_observation_ledger_position_or_performance_write_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: ControlledShadowForwardObservationImplementationRecord & {
      upstream_protocol_review: { review_sha256: string };
      upstream_protocol_registration: {
        protocol_registration_sha256: string;
        protocol_specification: {
          specification_sha256: string;
          exact_design_specification: { specification_sha256: string };
        };
      };
    };
    current_independent_audit: { audit_sha256: string; mismatch_reasons: string[] };
    complete_review_actor_ids: string[];
    latest_review?: {
      review_id: string;
      review_sha256: string;
      submitted_at: string;
      reviewer_id: string;
      verdict: ControlledShadowForwardObservationImplementationReviewVerdict;
      rationale: string;
      future_isolated_forward_observation_runner_specification_registration_eligible: boolean;
      trading_authorized: boolean;
    };
    review_eligible: boolean;
    future_isolated_forward_observation_runner_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_isolated_forward_observation_runner_specification_registration_eligible_count: number;
  review_status: string;
  isolated_runner_registered: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowForwardObservationIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_protocol_review_sha256: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  runner_name: string;
  runner_kind: "ephemeral_natural_forward_observation_specification";
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  artifact_reproducibility_procedure: string;
  rationale: string;
  known_limitations: string;
  future_mount_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_85_binding_confirmed: boolean;
  registrar_independent_from_stage_85_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed: boolean;
  executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed: boolean;
  no_callable_entrypoint_or_current_mount_confirmed: boolean;
  natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: boolean;
  weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed: boolean;
  future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  corporate_action_evidence_and_append_only_corrections_preserved_confirmed: boolean;
  future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed: boolean;
  deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_independent_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationImplementationReviewRecord = {
  review_id: string;
  review_sha256: string;
  reviewer_id: string;
  submitted_at: string;
  verdict: ControlledShadowForwardObservationImplementationReviewVerdict;
  rationale: string;
  independent_audit: {
    audit_sha256: string;
    mismatch_reasons: string[];
    protocol_review_sha256: string;
    protocol_registration_sha256: string;
    protocol_specification_sha256: string;
    design_specification_sha256: string;
  };
  future_isolated_forward_observation_runner_specification_registration_eligible: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationIsolatedRunnerRecord = {
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: ControlledShadowForwardObservationImplementationRecord & {
    implementation_contract: ControlledShadowForwardObservationImplementationRecord["implementation_contract"] & {
      canonical_future_input_manifest_schema: string;
      canonical_future_cycle_claim_schema: string;
      canonical_future_untrusted_observation_schema: string;
    };
  };
  implementation_review: ControlledShadowForwardObservationImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_85_and_complete_prior_chain: boolean;
  runner_name: string;
  runner_kind: "ephemeral_natural_forward_observation_specification";
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  artifact_reproducibility_procedure: string;
  rationale: string;
  known_limitations: string;
  future_mount_constraints: string;
  future_output_constraints: string;
  runner_contract: {
    contract_sha256: string;
    stage_85_implementation_review_id: string;
    stage_85_implementation_review_sha256: string;
    stage_85_independent_audit_sha256: string;
    stage_84_implementation_id: string;
    stage_84_implementation_sha256: string;
    stage_84_implementation_contract_sha256: string;
    stage_83_protocol_review_sha256: string;
    stage_82_protocol_registration_sha256: string;
    stage_82_protocol_specification_sha256: string;
    stage_74_design_specification_sha256: string;
    runtime_identity: string;
    runtime_version: string;
    next_gate: string;
    executable_artifact_present: boolean;
    callable_entrypoint_present: boolean;
    runtime_identity_bound: boolean;
    runtime_instantiated: boolean;
    input_mount_present: boolean;
    future_input_read_only_required: boolean;
    future_input_point_in_time_required: boolean;
    future_input_content_addressed_required: boolean;
    future_input_allowlisted_required: boolean;
    future_cycle_claim_first_required: boolean;
    future_cycle_create_once_required: boolean;
    future_output_create_once_required: boolean;
    future_output_untrusted_required: boolean;
    future_output_independent_validation_required: boolean;
    future_output_order_intent_allowed: boolean;
    environment_inheritance_allowed: boolean;
    secrets_allowed: boolean;
    outbound_network_allowed: boolean;
    tools_allowed: boolean;
    subprocesses_allowed: boolean;
    production_reads_allowed: boolean;
    production_writes_allowed: boolean;
    trading_allowed: boolean;
    maximum_parallel_runs: number;
    maximum_memory_mib: number;
    maximum_wall_clock_seconds_per_cycle: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes_per_cycle: number;
  };
  status: "registered_not_authorized_not_run";
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  forward_observation_started: boolean;
  forward_observation_completed: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: ControlledShadowForwardObservationIsolatedRunnerRecord["implementation"];
    review: ControlledShadowForwardObservationImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  allowed_runner_kinds: Array<"ephemeral_natural_forward_observation_specification">;
  items: Array<{
    runner: ControlledShadowForwardObservationIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_status: string;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  forward_observation_authorized: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_claim_first_forward_observation_attempt"
  | "changes_requested_rebuild_runner"
  | "rejected";

export type ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_id: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_spec_revision: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_protocol_review_sha256: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  independently_reproduced_runner_artifact_sha256: string;
  artifact_reproduction_evidence: string;
  verdict: ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_86_binding_confirmed: boolean;
  reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed: boolean;
  runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed: boolean;
  runner_artifact_digest_independently_reproduced_and_matched_confirmed: boolean;
  immutable_code_revision_and_artifact_availability_confirmed: boolean;
  natural_forward_no_backfill_and_observation_not_before_confirmed: boolean;
  weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed: boolean;
  point_in_time_read_only_content_addressed_allowlisted_input_confirmed: boolean;
  corporate_action_evidence_and_append_only_corrections_confirmed: boolean;
  create_once_untrusted_independently_validated_no_order_payload_output_confirmed: boolean;
  deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed: boolean;
  no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_88_claim_first_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationFirstExecutionAuthorizationReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: ControlledShadowForwardObservationIsolatedRunnerRecord;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  independently_reproduced_runner_artifact_sha256: string;
  artifact_reproduction_evidence: string;
  artifact_digest_matches_registered_runner: boolean;
  verdict: ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_execution_attempt_limit: number;
  one_future_claim_first_forward_observation_attempt_authorized: boolean;
  authorization_claimed: boolean;
  execution_attempt_endpoint_available: boolean;
  runtime_instantiated: boolean;
  input_manifest_attached: boolean;
  data_access_authorized: boolean;
  forward_observation_started: boolean;
  forward_observation_completed: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: ControlledShadowForwardObservationIsolatedRunnerRecord;
    current_binding: boolean;
    latest_review?: ControlledShadowForwardObservationFirstExecutionAuthorizationReview;
    authorization_unexpired: boolean;
    future_attempt_eligible: boolean;
  }>;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  future_attempt_eligible_count: number;
  authorization_status: string;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  data_access_authorized: boolean;
  forward_observation_started: boolean;
  forward_observation_ledger_created: boolean;
  shadow_position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowForwardObservationInitializationManifest = {
  schema_version: string;
  manifest_sha256: string;
  requested_at: string;
  observation_not_before: string;
  signal_cadence: string;
  first_eligible_signal_rule: string;
  official_market_calendar: string;
  official_market_calendar_source_url: string;
  official_market_calendar_content_sha256: string;
  benchmark_symbol: string;
  initial_observation_validation_sha256: string;
  natural_forward_only: boolean;
  retroactive_backfill_allowed: boolean;
  market_data_rows_attached: boolean;
  point_in_time_content_addressed_allowlisted_sources_required: boolean;
  synchronized_security_and_benchmark_observation_required: boolean;
  initialization_only: boolean;
};

export type InvokeControlledShadowForwardObservationOnceRequest = {
  expected_authorization_review_id: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_sha256: string;
  expected_protocol_review_sha256: string;
  expected_protocol_registration_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  expected_initial_observation_validation_sha256: string;
  expected_initialization_manifest_sha256: string;
  initialization_manifest: ControlledShadowForwardObservationInitializationManifest;
  claim_first_single_use_and_failure_consumes_confirmed: boolean;
  exact_current_stage_51_through_stage_87_binding_confirmed: boolean;
  executor_independent_from_stage_87_and_complete_prior_chain_confirmed: boolean;
  current_binary_digest_reverification_after_claim_confirmed: boolean;
  natural_forward_observation_not_before_and_no_backfill_confirmed: boolean;
  official_calendar_and_spy_synchronization_confirmed: boolean;
  initialization_manifest_contains_no_market_data_confirmed: boolean;
  initialization_receipt_is_untrusted_and_requires_independent_validation_confirmed: boolean;
  no_runtime_mount_data_access_observation_ledger_position_or_performance_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationExecutionAttemptClaim = {
  schema_version: string;
  execution_policy_version: string;
  attempt_id: string;
  claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_valid_until: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_contract_sha256: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  implementation_review_sha256: string;
  protocol_review_sha256: string;
  protocol_registration_sha256: string;
  protocol_specification_sha256: string;
  design_specification_sha256: string;
  initial_observation_validation_sha256: string;
  initialization_manifest_sha256: string;
  claimed_at: string;
  invoked_by: string;
  authorization_consumed: boolean;
  invocation_started: boolean;
  initialization_manifest_opened: boolean;
  persistent_runtime_instantiation_allowed: boolean;
  market_data_access_allowed: boolean;
  forward_observation_write_allowed: boolean;
  ledger_write_allowed: boolean;
  position_write_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type ControlledShadowForwardObservationUntrustedInitializationReceipt = {
  schema_version: string;
  initialization_manifest_sha256: string;
  observation_not_before: string;
  requested_at: string;
  signal_cadence: string;
  first_eligible_signal_rule: string;
  official_market_calendar: string;
  official_market_calendar_source_url: string;
  official_market_calendar_content_sha256: string;
  benchmark_symbol: string;
  initial_observation_validation_sha256: string;
  natural_forward_only: boolean;
  retroactive_backfill_allowed: boolean;
  point_in_time_content_addressed_allowlisted_sources_required: boolean;
  synchronized_security_and_benchmark_observation_required: boolean;
  initialization_only: boolean;
  output_is_untrusted: boolean;
  independent_output_validation_completed: boolean;
  market_data_rows_attached: boolean;
  natural_forward_market_sessions_observed: number;
  persistent_runtime_instantiated: boolean;
  market_data_accessed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_intent_present: boolean;
  broker_payload_present: boolean;
  trade_executed: boolean;
};

export type ValidateControlledShadowForwardObservationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  expected_initial_observation_validation_sha256: string;
  expected_initialization_manifest_sha256: string;
  independent_reopen_and_manifest_receipt_reconstruction_confirmed: boolean;
  exact_current_stage_51_through_stage_88_binding_confirmed: boolean;
  validator_independent_from_executor_stage_87_and_complete_prior_chain_confirmed: boolean;
  claim_first_ordering_and_single_terminal_result_confirmed: boolean;
  zero_market_data_natural_forward_only_and_no_backfill_confirmed: boolean;
  official_calendar_https_content_hash_and_spy_confirmed: boolean;
  zero_runtime_observation_ledger_position_and_performance_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  validation_only_opens_future_first_natural_forward_cycle_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowForwardObservationOutputValidationRecord = {
  schema_version: string;
  policy_version: string;
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_id: string;
  result_sha256: string;
  output_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  implementation_contract_sha256: string;
  protocol_specification_sha256: string;
  design_specification_sha256: string;
  initial_observation_validation_sha256: string;
  initialization_manifest_sha256: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validated_at: string;
  validated_by: string;
  invoked_by: string;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_zero_market_initialization_receipt"
    | "failed_independent_zero_market_initialization_receipt_validation";
  initialization_receipt_independently_validated: boolean;
  future_first_natural_forward_cycle_authorization_review_eligible: boolean;
  persistent_runtime_instantiated: boolean;
  market_data_accessed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowForwardObservationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: ControlledShadowForwardObservationExecutionAttemptClaim;
      result: ControlledShadowForwardObservationExecutionAttemptResult;
    };
    validation?: ControlledShadowForwardObservationOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_initialization_receipt_count: number;
  failed_validation_count: number;
  future_first_natural_forward_cycle_authorization_review_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  persistent_runtime_instantiated: boolean;
  market_data_accessed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_validation_sha256: string;
  expected_attempt_id: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_protocol_specification_sha256: string;
  expected_design_specification_sha256: string;
  expected_initial_observation_validation_sha256: string;
  expected_initialization_manifest_sha256: string;
  verdict:
    | "approved_for_one_future_claim_first_natural_forward_cycle_attempt"
    | "changes_requested_revalidate_initialization"
    | "rejected";
  rationale: string;
  exact_current_stage_51_through_stage_89_binding_confirmed: boolean;
  reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed: boolean;
  zero_market_initialization_receipt_independently_validated_confirmed: boolean;
  natural_forward_only_no_backfill_and_observation_not_before_confirmed: boolean;
  official_https_calendar_content_identity_and_security_spy_sync_confirmed: boolean;
  point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: boolean;
  corporate_action_evidence_and_append_only_corrections_confirmed: boolean;
  claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: boolean;
  deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed: boolean;
  single_use_seven_day_window_and_future_attempt_separation_confirmed: boolean;
  current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_claim_first_cycle_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowFirstNaturalForwardCycleAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_sha256: string;
  output_sha256: string;
  authorization_review_sha256: string;
  isolated_runner_spec_sha256: string;
  runner_artifact_sha256: string;
  implementation_contract_sha256: string;
  protocol_specification_sha256: string;
  design_specification_sha256: string;
  initial_observation_validation_sha256: string;
  initialization_manifest_sha256: string;
  observation_not_before: string;
  submitted_at: string;
  authorization_not_before: string;
  authorization_valid_until: string;
  reviewer_id: string;
  verdict: ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest["verdict"];
  rationale: string;
  one_shot_execution_attempt_limit: number;
  one_future_claim_first_natural_forward_cycle_attempt_authorized: boolean;
  authorization_claimed: boolean;
  cycle_execution_endpoint_available: boolean;
  calendar_read_authorized: boolean;
  market_data_adapter_authorized: boolean;
  market_data_access_authorized: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    attempt: {
      claim: ControlledShadowForwardObservationExecutionAttemptClaim;
      result: ControlledShadowForwardObservationExecutionAttemptResult;
    };
    validation: ControlledShadowForwardObservationOutputValidationRecord;
    latest_review?: ControlledShadowFirstNaturalForwardCycleAuthorizationReview;
    current_binding: boolean;
    authorization_claimed: boolean;
    authorization_active: boolean;
    future_attempt_eligible: boolean;
  }>;
  review_eligible_initialization_count: number;
  reviewed_initialization_count: number;
  approved_initialization_count: number;
  active_authorization_count: number;
  future_attempt_eligible_count: number;
  authorization_status: string;
  calendar_read_authorized: boolean;
  market_data_adapter_authorized: boolean;
  market_data_access_authorized: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimControlledShadowFirstNaturalForwardCycleRequest = {
  expected_authorization_review_sha256: string;
  expected_validation_sha256: string;
  expected_stage_88_attempt_id: string;
  expected_stage_88_claim_sha256: string;
  expected_stage_88_result_sha256: string;
  expected_stage_88_output_sha256: string;
  expected_initialization_manifest_sha256: string;
  claim_reason: string;
  exact_stage_51_through_stage_90_binding_confirmed: boolean;
  claimant_independence_from_stage_90_and_complete_prior_chain_confirmed: boolean;
  authorization_current_unexpired_and_single_use_confirmed: boolean;
  claim_first_before_calendar_or_market_data_confirmed: boolean;
  separate_read_only_market_data_adapter_authorization_required_confirmed: boolean;
  natural_forward_only_no_backfill_and_create_once_confirmed: boolean;
  no_runtime_observation_ledger_position_or_performance_confirmed: boolean;
  no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowFirstNaturalForwardCycleClaim = {
  schema_version: string;
  policy_version: string;
  cycle_claim_id: string;
  cycle_claim_sha256: string;
  authorization_review_id: string;
  authorization_review_sha256: string;
  authorization_not_before: string;
  authorization_valid_until: string;
  validation_id: string;
  validation_sha256: string;
  stage_88_attempt_id: string;
  stage_88_claim_sha256: string;
  stage_88_result_sha256: string;
  stage_88_output_sha256: string;
  initialization_manifest_sha256: string;
  observation_eligibility_anchor: string;
  cycle_ordinal: number;
  claimed_at: string;
  claimed_by: string;
  excluded_prior_actor_ids: string[];
  claimant_independent_from_stage_90_and_complete_prior_chain: boolean;
  claim_reason: string;
  authorization_consumed: boolean;
  create_once: boolean;
  claim_first: boolean;
  task_status: string;
  calendar_window_resolved: boolean;
  calendar_read_authorized: boolean;
  market_data_adapter_authorized: boolean;
  market_data_access_authorized: boolean;
  execution_endpoint_available: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowFirstNaturalForwardCycleClaimRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_authorizations: Array<{
    validation: ControlledShadowForwardObservationOutputValidationRecord;
    authorization: ControlledShadowFirstNaturalForwardCycleAuthorizationReview;
  }>;
  claims: ControlledShadowFirstNaturalForwardCycleClaim[];
  authorization_candidate_count: number;
  claim_eligible_count: number;
  claim_count: number;
  authorization_consumed_count: number;
  waiting_for_separate_market_data_adapter_authorization_count: number;
  claim_status: string;
  calendar_window_resolved: boolean;
  calendar_read_authorized: boolean;
  market_data_adapter_authorized: boolean;
  market_data_access_authorized: boolean;
  execution_endpoint_available: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewControlledShadowMarketDataAdapterAuthorizationRequest = {
  expected_cycle_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_validation_sha256: string;
  expected_initialization_manifest_sha256: string;
  verdict:
    | "approved_for_future_claim_first_read_only_market_data_receipt"
    | "rejected_market_data_adapter_contract";
  rationale: string;
  source_allowlist_assessment: string;
  credential_and_request_minimization_assessment: string;
  content_addressing_and_custody_assessment: string;
  known_limitations: string;
  future_receipt_constraints: string;
  exact_stage_51_through_stage_91_binding_confirmed: boolean;
  reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: boolean;
  fixed_get_only_https_origin_and_path_allowlist_confirmed: boolean;
  calendar_security_spy_price_dividend_split_only_confirmed: boolean;
  exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: boolean;
  credentials_never_persisted_forwarded_or_returned_confirmed: boolean;
  request_response_source_and_retrieval_time_hashes_required_confirmed: boolean;
  natural_forward_only_no_backfill_or_history_rewrite_confirmed: boolean;
  approval_only_opens_future_claim_first_read_only_receipt_confirmed: boolean;
  no_data_request_calendar_resolution_or_runtime_started_confirmed: boolean;
  no_observation_ledger_position_performance_or_model_metric_write_confirmed: boolean;
  no_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowReadOnlyMarketDataAdapterSpec = {
  schema_version: string;
  adapter_id: string;
  adapter_spec_sha256: string;
  allowed_http_methods: string[];
  allowed_https_origin_and_path_prefixes: string[];
  allowed_query_parameter_names: string[];
  credential_query_parameter_name: string;
  credential_redaction_required: boolean;
  credential_excluded_from_canonical_request_sha256: boolean;
  allowed_data_classes: string[];
  benchmark_symbol: string;
  exact_future_subject_symbol_set_content_hash_required: boolean;
  exact_future_time_window_content_hash_required: boolean;
  request_sha256_required: boolean;
  response_body_sha256_required: boolean;
  source_document_sha256_required: boolean;
  retrieved_at_utc_required: boolean;
  source_available_at_utc_required: boolean;
  raw_payload_retention_required: boolean;
  append_only_correction_required: boolean;
  credentials_may_be_persisted: boolean;
  credentials_may_be_returned: boolean;
  redirects_allowed: boolean;
  non_https_allowed: boolean;
  arbitrary_url_allowed: boolean;
  arbitrary_symbol_allowed: boolean;
  retroactive_backfill_allowed: boolean;
  maximum_response_bytes: number;
};

export type ControlledShadowMarketDataAdapterAuthorization = {
  schema_version: string;
  policy_version: string;
  adapter_authorization_id: string;
  adapter_authorization_sha256: string;
  cycle_claim_id: string;
  cycle_claim_sha256: string;
  upstream_authorization_review_sha256: string;
  validation_sha256: string;
  initialization_manifest_sha256: string;
  adapter_specification: ControlledShadowReadOnlyMarketDataAdapterSpec;
  submitted_at: string;
  authorized_not_before: string;
  authorized_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  reviewer_independent_from_claimant_and_complete_prior_chain: boolean;
  verdict: ReviewControlledShadowMarketDataAdapterAuthorizationRequest["verdict"];
  rationale: string;
  source_allowlist_assessment: string;
  credential_and_request_minimization_assessment: string;
  content_addressing_and_custody_assessment: string;
  known_limitations: string;
  future_receipt_constraints: string;
  adapter_contract_authorized: boolean;
  future_claim_first_read_only_market_data_receipt_eligible: boolean;
  market_data_request_made: boolean;
  calendar_window_resolved: boolean;
  market_data_accessed: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataAdapterAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  adapter_specification: ControlledShadowReadOnlyMarketDataAdapterSpec;
  items: Array<{
    claim: ControlledShadowFirstNaturalForwardCycleClaim;
    authorization?: ControlledShadowMarketDataAdapterAuthorization;
    review_eligible: boolean;
    adapter_contract_authorized: boolean;
    future_claim_first_read_only_market_data_receipt_eligible: boolean;
  }>;
  claimed_task_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  approved_count: number;
  rejected_count: number;
  active_authorization_count: number;
  future_claim_first_read_only_market_data_receipt_eligible_count: number;
  authorization_status: string;
  market_data_request_made: boolean;
  calendar_window_resolved: boolean;
  market_data_accessed: boolean;
  runtime_instantiated: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimAndReadControlledShadowMarketDataReceiptRequest = {
  expected_adapter_authorization_sha256: string;
  expected_cycle_claim_sha256: string;
  expected_adapter_spec_sha256: string;
  expected_subject_symbol_set_sha256: string;
  expected_time_window_sha256: string;
  execution_reason: string;
  claim_first_single_use_and_failure_consumes_authorization_confirmed: boolean;
  exact_stage_51_through_stage_92_binding_confirmed: boolean;
  executor_independent_from_stage_92_and_complete_prior_chain_confirmed: boolean;
  fixed_get_https_path_and_query_allowlist_confirmed: boolean;
  server_derived_subject_symbols_and_spy_only_confirmed: boolean;
  natural_forward_window_content_addressed_no_backfill_confirmed: boolean;
  credential_redacted_not_persisted_returned_or_logged_confirmed: boolean;
  raw_payload_hashes_timestamps_and_custody_retained_confirmed: boolean;
  receipt_untrusted_pending_independent_validation_confirmed: boolean;
  no_parsed_calendar_observation_ledger_position_performance_or_model_metric_confirmed: boolean;
  no_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataReceiptCandidate = {
  adapter_authorization_id: string;
  adapter_authorization_sha256: string;
  cycle_claim_sha256: string;
  adapter_spec_sha256: string;
  subject_symbols: string[];
  subject_symbol_set_sha256: string;
  benchmark_symbol: string;
  window_start_date: string;
  window_end_date: string;
  time_window_sha256: string;
  expected_request_count: number;
  executor_excluded_actor_ids: string[];
  fmp_configured: boolean;
};

export type ControlledShadowMarketDataReceiptAttemptRegistry = {
  schema_version: string;
  policy_version: string;
  invocation_endpoint_available: boolean;
  eligible_authorizations: ControlledShadowMarketDataReceiptCandidate[];
  items: Array<{
    claim: {
      attempt_id: string;
      claim_sha256: string;
      adapter_authorization_id: string;
      subject_symbols: string[];
      window_start_date: string;
      window_end_date: string;
      expected_request_count: number;
      claimed_at: string;
      claimed_by: string;
    };
    result?: {
      result_id: string;
      status: "completed_with_untrusted_raw_market_data_receipt" | "failed_authorization_consumed";
      bounded_error_code?: string;
      market_data_accessed: boolean;
      forward_observation_started: boolean;
      trading_authorized: boolean;
      untrusted_raw_market_data_receipt?: {
        receipt_sha256: string;
        total_response_bytes: number;
        raw_payload_count: number;
        independent_validation_completed: boolean;
      };
    };
    interrupted_after_claim: boolean;
  }>;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_untrusted_receipt_count: number;
  failed_authorization_consumed_count: number;
  interrupted_authorization_consumed_count: number;
  independent_validation_eligible_count: number;
  receipt_status: string;
  scope: string;
};

export type ValidateControlledShadowMarketDataReceiptRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_receipt_sha256: string;
  expected_adapter_authorization_sha256: string;
  expected_cycle_claim_sha256: string;
  expected_adapter_spec_sha256: string;
  expected_subject_symbol_set_sha256: string;
  expected_time_window_sha256: string;
  expected_canonical_request_set_sha256: string;
  independent_chain_reopen_and_fingerprint_recomputation_confirmed: boolean;
  validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed: boolean;
  claim_first_single_terminal_result_and_no_replay_confirmed: boolean;
  redacted_fixed_request_set_independently_reconstructed_confirmed: boolean;
  every_raw_payload_reopened_size_and_sha256_recomputed_confirmed: boolean;
  source_identity_timestamp_and_content_addressed_custody_confirmed: boolean;
  credential_absence_from_persisted_artifacts_confirmed: boolean;
  successful_http_envelope_only_not_market_truth_confirmed: boolean;
  validation_does_not_parse_calendar_or_market_rows_confirmed: boolean;
  no_runtime_observation_ledger_position_performance_or_model_metric_confirmed: boolean;
  no_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataReceiptValidationCandidate = {
  attempt_id: string;
  claim_sha256: string;
  result_sha256: string;
  receipt_sha256: string;
  adapter_authorization_sha256: string;
  cycle_claim_sha256: string;
  adapter_spec_sha256: string;
  subject_symbols: string[];
  subject_symbol_set_sha256: string;
  window_start_date: string;
  window_end_date: string;
  time_window_sha256: string;
  canonical_request_set_sha256: string;
  raw_payload_count: number;
  total_response_bytes: number;
  validator_excluded_actor_ids: string[];
};

export type ControlledShadowMarketDataReceiptValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_sha256: string;
  receipt_sha256: string;
  validated_at: string;
  validated_by: string;
  stage_93_executor_id: string;
  verdict:
    | "independently_validated_untrusted_raw_market_data_receipt"
    | "failed_independent_raw_market_data_receipt_validation";
  raw_market_data_receipt_independently_validated: boolean;
  future_market_data_parser_review_eligible: boolean;
  raw_payload_custody_manifest_sha256: string;
  mismatch_reasons: string[];
  calendar_window_resolved: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataReceiptValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validation_endpoint_available: boolean;
  candidates: ControlledShadowMarketDataReceiptValidationCandidate[];
  validations: ControlledShadowMarketDataReceiptValidationRecord[];
  completed_untrusted_receipt_count: number;
  pending_independent_validation_count: number;
  independently_validated_receipt_count: number;
  failed_independent_validation_count: number;
  future_market_data_parser_review_eligible_count: number;
  validation_status: string;
  calendar_window_resolved: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowMarketDataParserSpecificationRequest = {
  expected_validation_sha256: string;
  expected_receipt_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_adapter_authorization_sha256: string;
  expected_adapter_spec_sha256: string;
  expected_canonical_request_set_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_stage_51_through_stage_94_binding_confirmed: boolean;
  registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed: boolean;
  explicit_price_dividend_split_and_official_calendar_sources_confirmed: boolean;
  strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed: boolean;
  duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed: boolean;
  no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed: boolean;
  spy_calendar_sync_and_cross_source_reconciliation_required_confirmed: boolean;
  synthetic_vectors_contain_no_market_fact_or_credential_confirmed: boolean;
  specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: boolean;
  no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed: boolean;
  no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed: boolean;
  no_training_feedback_reward_order_broker_or_trading_confirmed: boolean;
  future_chain_external_specification_review_required_before_implementation_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserSpecificationCandidate = {
  validation_id: string;
  validation_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  result_sha256: string;
  receipt_sha256: string;
  adapter_authorization_sha256: string;
  adapter_spec_sha256: string;
  canonical_request_set_sha256: string;
  subject_symbols: string[];
  raw_payload_count: number;
  registrar_excluded_actor_ids: string[];
};

export type ControlledShadowMarketDataParserSpecificationRegistration = {
  registration_id: string;
  registration_sha256: string;
  registered_at: string;
  registered_by: string;
  stage_94_validation_id: string;
  stage_94_validation_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  status: string;
  parser_specification_registered: boolean;
  future_chain_external_specification_review_eligible: boolean;
  specification_review_completed: boolean;
  parser_implementation_registration_eligible: boolean;
  parser_specification: {
    parser_specification_sha256: string;
    parser_protocol_version: string;
    source_contract_revision: string;
    external_reference_urls: string[];
    accepted_source_kinds: string[];
    synthetic_test_vectors: Array<{
      vector_id: string;
      source_kind: string;
      input_fixture_sha256: string;
      expected_outcome: string;
      synthetic_only_no_market_truth: boolean;
    }>;
    forward_fill_allowed: boolean;
    interpolation_allowed: boolean;
    unadjusted_close_fallback_allowed: boolean;
    inferred_dividend_or_split_allowed: boolean;
    parser_output_create_once_and_untrusted: boolean;
    parser_output_independent_validation_required: boolean;
  };
  parsed_calendar_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserSpecificationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  candidates: ControlledShadowMarketDataParserSpecificationCandidate[];
  registrations: ControlledShadowMarketDataParserSpecificationRegistration[];
  independently_validated_receipt_count: number;
  registration_eligible_count: number;
  parser_specification_registered_count: number;
  future_chain_external_specification_review_eligible_count: number;
  parser_specification_status: string;
  parser_implementation_present: boolean;
  parsed_calendar_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewControlledShadowMarketDataParserSpecificationRequest = {
  expected_registration_sha256: string;
  expected_parser_specification_sha256: string;
  expected_validation_sha256: string;
  expected_receipt_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_adapter_authorization_sha256: string;
  expected_adapter_spec_sha256: string;
  expected_canonical_request_set_sha256: string;
  verdict:
    | "approved_for_future_zero_capability_parser_implementation_registration"
    | "changes_required_rebuild_parser_specification"
    | "rejected_parser_specification";
  rationale: string;
  source_contract_assessment: string;
  schema_and_numeric_assessment: string;
  calendar_and_reconciliation_assessment: string;
  synthetic_vector_assessment: string;
  failure_and_missing_data_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  exact_stage_51_through_stage_95_binding_confirmed: boolean;
  reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed: boolean;
  independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed: boolean;
  independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: boolean;
  strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: boolean;
  duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: boolean;
  separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: boolean;
  spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: boolean;
  source_available_at_remains_unverified_until_separate_review_confirmed: boolean;
  specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed: boolean;
  approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: boolean;
  no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserSpecificationReview = {
  review_id: string;
  review_sha256: string;
  registration_id: string;
  registration_sha256: string;
  parser_specification_sha256: string;
  validation_sha256: string;
  receipt_sha256: string;
  claim_sha256: string;
  result_sha256: string;
  adapter_authorization_sha256: string;
  adapter_spec_sha256: string;
  canonical_request_set_sha256: string;
  reviewed_at: string;
  reviewed_by: string;
  excluded_prior_actor_ids: string[];
  verdict: ReviewControlledShadowMarketDataParserSpecificationRequest["verdict"];
  rationale: string;
  source_contract_assessment: string;
  schema_and_numeric_assessment: string;
  calendar_and_reconciliation_assessment: string;
  synthetic_vector_assessment: string;
  failure_and_missing_data_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  confirmations_complete: boolean;
  validation_chain_independently_recomputed: boolean;
  explicit_source_request_set_independently_reconstructed: boolean;
  parser_specification_independently_recomputed: boolean;
  synthetic_vectors_independently_reconstructed: boolean;
  strict_fail_closed_semantics_independently_verified: boolean;
  zero_capability_boundary_independently_verified: boolean;
  independent_audit_passed: boolean;
  mismatch_reasons: string[];
  parser_specification_independently_approved: boolean;
  future_zero_capability_parser_implementation_registration_eligible: boolean;
};

export type ControlledShadowMarketDataParserSpecificationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_endpoint_available: boolean;
  items: Array<{
    registration: ControlledShadowMarketDataParserSpecificationRegistration;
    validation_sha256: string;
    receipt_sha256: string;
    claim_sha256: string;
    result_sha256: string;
    adapter_authorization_sha256: string;
    adapter_spec_sha256: string;
    canonical_request_set_sha256: string;
    subject_symbols: string[];
    raw_payload_count: number;
    latest_review: ControlledShadowMarketDataParserSpecificationReview | null;
    review_eligible: boolean;
    independently_approved: boolean;
  }>;
  parser_specification_registered_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_zero_capability_parser_implementation_registration_eligible_count: number;
  review_status: string;
  parser_implementation_registered: boolean;
  parser_implementation_present: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowMarketDataParserImplementationRequest = {
  expected_specification_review_id: string;
  expected_specification_review_sha256: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_parser_specification_sha256: string;
  expected_validation_sha256: string;
  expected_receipt_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_adapter_authorization_sha256: string;
  expected_adapter_spec_sha256: string;
  expected_canonical_request_set_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_parser_semantics: string;
  source_schema_and_numeric_semantics: string;
  calendar_action_and_reconciliation_semantics: string;
  error_and_missing_data_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_stage_51_through_stage_96_binding_confirmed: boolean;
  registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_review_registration_and_specification_confirmed: boolean;
  zero_capability_contract_only_no_source_or_executable_artifact_confirmed: boolean;
  fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: boolean;
  strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: boolean;
  duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: boolean;
  spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed: boolean;
  all_eight_synthetic_vector_hashes_bound_confirmed: boolean;
  source_available_at_remains_unverified_until_separate_review_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: boolean;
  no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_required_before_isolated_runner_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserImplementationRecord = {
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation_name: string;
  implementation_description: string;
  status: string;
  confirmations_complete: boolean;
  zero_capability_parser_implementation_contract_registered: boolean;
  parser_implementation_present: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  upstream_specification_registration: ControlledShadowMarketDataParserSpecificationRegistration;
  upstream_specification_review: ControlledShadowMarketDataParserSpecificationReview;
  implementation_contract: {
    schema_version: string;
    contract_sha256: string;
    implementation_protocol_version: string;
    immutable_code_revision: string;
    validation_sha256: string;
    receipt_sha256: string;
    claim_sha256: string;
    result_sha256: string;
    registered_not_run: boolean;
    independent_implementation_review_required: boolean;
    isolated_runner_registration_required_after_review: boolean;
    strict_envelope_dispatch_function_id: string;
    fmp_price_array_parser_function_id: string;
    fmp_dividend_event_parser_function_id: string;
    fmp_split_event_parser_function_id: string;
    nyse_calendar_table_parser_function_id: string;
    calendar_subject_spy_reconciliation_function_id: string;
    canonical_row_serialization_and_hash_function_id: string;
    synthetic_vector_conformance_function_id: string;
  };
};

export type ControlledShadowMarketDataParserImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  items: Array<{
    specification_review: ControlledShadowMarketDataParserSpecificationReview;
    specification_registration: ControlledShadowMarketDataParserSpecificationRegistration;
    subject_symbols: string[];
    raw_payload_count: number;
    implementation: ControlledShadowMarketDataParserImplementationRecord | null;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  independently_approved_specification_count: number;
  registration_eligible_count: number;
  implementation_contract_count: number;
  current_binding_implementation_contract_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowMarketDataParserImplementationReviewVerdict =
  | "approved_for_future_isolated_market_data_parser_runner_specification_registration"
  | "changes_required_rebuild_market_data_parser_implementation_contract"
  | "rejected_market_data_parser_implementation_contract";

export type ReviewControlledShadowMarketDataParserImplementationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_parser_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowMarketDataParserImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_parser_semantics_assessment: string;
  source_schema_calendar_action_and_reconciliation_assessment: string;
  failure_and_missing_data_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  exact_current_stage_51_through_stage_97_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: boolean;
  explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: boolean;
  strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: boolean;
  duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: boolean;
  spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed: boolean;
  all_eight_synthetic_vectors_independently_reconstructed_confirmed: boolean;
  source_available_at_remains_unverified_until_separate_evidence_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: boolean;
  no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  specification_review_sha256: string;
  specification_registration_sha256: string;
  parser_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  specification_review_hash_independently_reproduced: boolean;
  specification_registration_hash_independently_reproduced: boolean;
  parser_specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_97_binding_valid: boolean;
  eight_function_ids_and_canonical_schemas_valid: boolean;
  explicit_source_calendar_action_and_reconciliation_contract_valid: boolean;
  strict_schema_numeric_failure_and_missing_data_contract_valid: boolean;
  eight_synthetic_vectors_bound_and_synthetic_only: boolean;
  source_available_at_still_unverified: boolean;
  all_artifact_runtime_raw_payload_store_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ControlledShadowMarketDataParserImplementationReviewRecord = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  implementation: ControlledShadowMarketDataParserImplementationRecord;
  independent_audit: ControlledShadowMarketDataParserImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowMarketDataParserImplementationReviewVerdict;
  rationale: string;
  zero_capability_implementation_independently_approved: boolean;
  future_isolated_parser_runner_specification_registration_eligible: boolean;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_endpoint_available: boolean;
  items: Array<{
    implementation: ControlledShadowMarketDataParserImplementationRecord;
    current_independent_audit: ControlledShadowMarketDataParserImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review: ControlledShadowMarketDataParserImplementationReviewRecord | null;
    review_eligible: boolean;
    future_isolated_parser_runner_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_isolated_parser_runner_specification_registration_eligible_count: number;
  review_status: string;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowMarketDataParserIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_parser_specification_sha256: string;
  expected_validation_sha256: string;
  expected_receipt_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_market_data_parser_specification";
  runner_spec_revision: string;
  proposed_runner_code_revision: string;
  proposed_runner_artifact_sha256: string;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_98_binding_confirmed: boolean;
  registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: boolean;
  all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: boolean;
  future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: boolean;
  strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  source_available_at_remains_unverified_until_separate_evidence_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserIsolatedRunnerRecord = {
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: ControlledShadowMarketDataParserImplementationRecord;
  implementation_review: ControlledShadowMarketDataParserImplementationReviewRecord;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_market_data_parser_specification";
  runner_contract: {
    contract_sha256: string;
    runner_spec_revision: string;
    proposed_runner_code_revision: string;
    proposed_runner_artifact_sha256: string;
    runtime_identity: string;
    runtime_version: string;
    future_input_envelope: string;
    future_output_envelope: string;
    next_gate: string;
    future_runner_artifact_identity_bound: boolean;
    source_artifact_present: boolean;
    executable_artifact_present: boolean;
    callable_entrypoint_present: boolean;
    runtime_instantiated: boolean;
    raw_payload_mount_present: boolean;
    raw_payload_read_allowed: boolean;
    maximum_parallel_runs: number;
    maximum_memory_mib: number;
    maximum_wall_clock_seconds: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes: number;
  };
  status: string;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: ControlledShadowMarketDataParserImplementationRecord;
    review: ControlledShadowMarketDataParserImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  items: Array<{
    runner: ControlledShadowMarketDataParserIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  raw_payload_accessed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowMarketDataParserFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_claim_first_parser_attempt"
  | "changes_requested_rebuild_artifact"
  | "rejected";

export type ControlledShadowMarketDataParserReproducedArtifactManifest = {
  schema_version: string;
  manifest_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_contract_sha256: string;
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  artifact_byte_length: number;
  artifact_file_name: string;
  artifact_media_type: string;
  source_bundle_sha256: string;
  artifact_reproduction_procedure_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  reproduced_at: string;
  reproduced_by: string;
  source_and_artifact_reproduced_from_immutable_revision: boolean;
  artifact_is_read_only_regular_file: boolean;
  artifact_was_not_executed: boolean;
  raw_market_data_was_not_read: boolean;
};

export type ControlledShadowMarketDataParserArtifactInspection = {
  custody_locator: string;
  manifest_present: boolean;
  artifact_present: boolean;
  manifest: ControlledShadowMarketDataParserReproducedArtifactManifest | null;
  server_computed_artifact_sha256: string | null;
  server_observed_artifact_byte_length: number | null;
  artifact_verified: boolean;
  status: string;
};

export type ReviewControlledShadowMarketDataParserFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_id: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_spec_revision: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_parser_specification_sha256: string;
  expected_validation_sha256: string;
  expected_receipt_sha256: string;
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_artifact_manifest_sha256: string;
  artifact_reproduction_review_evidence: string;
  sandbox_contract_review_evidence: string;
  verdict: ControlledShadowMarketDataParserFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_99_binding_confirmed: boolean;
  reviewer_independent_from_stage_99_builder_and_complete_prior_chain_confirmed: boolean;
  server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: boolean;
  self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: boolean;
  artifact_builder_and_reviewer_separation_confirmed: boolean;
  all_eight_parser_functions_and_canonical_schemas_remain_bound_confirmed: boolean;
  strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: boolean;
  no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: boolean;
  future_output_create_once_untrusted_independently_validated_no_market_interpretation_or_order_intent_confirmed: boolean;
  source_available_at_remains_unverified_until_separate_evidence_confirmed: boolean;
  authorization_single_use_24_hour_expiry_and_stage_101_claim_separation_confirmed: boolean;
  no_runtime_entrypoint_mount_payload_read_parser_execution_or_parsed_rows_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_101_claim_first_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserFirstExecutionAuthorizationReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: ControlledShadowMarketDataParserIsolatedRunnerRecord;
  artifact_manifest: ControlledShadowMarketDataParserReproducedArtifactManifest;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  server_computed_artifact_sha256: string;
  server_observed_artifact_byte_length: number;
  verdict: ControlledShadowMarketDataParserFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_execution_attempt_limit: number;
  one_future_claim_first_parser_attempt_authorized: boolean;
  authorization_claimed: boolean;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  raw_payload_mount_present: boolean;
  raw_payload_read: boolean;
  parser_executed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: ControlledShadowMarketDataParserIsolatedRunnerRecord;
    artifact_inspection: ControlledShadowMarketDataParserArtifactInspection;
    latest_review: ControlledShadowMarketDataParserFirstExecutionAuthorizationReview | null;
    authorization_unexpired: boolean;
    future_claim_eligible: boolean;
  }>;
  runner_count: number;
  artifact_verified_runner_count: number;
  artifact_pending_runner_count: number;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  future_claim_eligible_count: number;
  authorization_status: string;
  next_gate: string;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  raw_payload_mount_present: boolean;
  raw_payload_read: boolean;
  parser_executed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimControlledShadowMarketDataParserExecutionAttemptRequest = {
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_stage_94_validation_sha256: string;
  expected_stage_93_claim_sha256: string;
  expected_stage_93_result_sha256: string;
  expected_stage_93_receipt_sha256: string;
  expected_canonical_request_set_sha256: string;
  expected_fixed_input_manifest_sha256: string;
  claim_reason: string;
  exact_current_stage_51_through_stage_100_binding_confirmed: boolean;
  claimant_independent_from_stage_100_and_complete_prior_chain_confirmed: boolean;
  authorization_unexpired_single_use_and_consumed_before_execution_confirmed: boolean;
  current_server_rehashed_artifact_and_manifest_binding_confirmed: boolean;
  fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: boolean;
  claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: boolean;
  no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: boolean;
  future_output_create_once_untrusted_and_independently_validated_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserFixedInputManifest = {
  schema_version: string;
  input_manifest_sha256: string;
  stage_94_validation: ControlledShadowMarketDataReceiptValidationRecord;
  stage_93_claim: {
    attempt_id: string;
    claim_sha256: string;
    subject_symbols: string[];
    subject_symbol_set_sha256: string;
    benchmark_symbol: string;
    window_start_date: string;
    window_end_date: string;
    time_window_sha256: string;
    canonical_request_set_sha256: string;
  };
  stage_93_result_sha256: string;
  stage_93_receipt_sha256: string;
  subject_symbols: string[];
  benchmark_symbol: string;
  window_start_date: string;
  window_end_date: string;
  subject_symbol_set_sha256: string;
  time_window_sha256: string;
  canonical_request_set_sha256: string;
  raw_payload_custody_manifest_sha256: string;
  raw_payloads: Array<{
    source_id: string;
    canonical_request_sha256: string;
    response_body_sha256: string;
    source_document_sha256: string;
    response_bytes: number;
    retrieved_at_utc: string;
    source_available_at_utc: string;
    raw_payload_relative_path: string;
  }>;
  raw_payload_count: number;
  total_response_bytes: number;
  input_metadata_only: boolean;
  raw_payloads_opened_by_claim: boolean;
  fixed_stage_94_independently_validated_input: boolean;
};

export type ControlledShadowMarketDataParserExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization: ControlledShadowMarketDataParserFirstExecutionAuthorizationReview;
  fixed_input_manifest: ControlledShadowMarketDataParserFixedInputManifest;
  claimed_at: string;
  claimed_by: string;
  excluded_prior_actor_ids: string[];
  claim_reason: string;
  authorization_consumed: boolean;
  create_once: boolean;
  claim_first: boolean;
  task_status: string;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  raw_payload_mount_present: boolean;
  raw_payload_read: boolean;
  parser_executed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  output_written: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserExecutionAttemptClaimRegistry = {
  schema_version: string;
  policy_version: string;
  claim_endpoint_available: boolean;
  eligible_authorizations: Array<{
    authorization: ControlledShadowMarketDataParserFirstExecutionAuthorizationReview;
    fixed_input_manifest: ControlledShadowMarketDataParserFixedInputManifest;
    claimant_excluded_actor_ids: string[];
  }>;
  claims: ControlledShadowMarketDataParserExecutionAttemptClaim[];
  authorization_candidate_count: number;
  claim_eligible_count: number;
  claim_count: number;
  authorization_consumed_count: number;
  waiting_for_stage_102_execution_count: number;
  claim_status: string;
  next_gate: string;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  raw_payload_mount_present: boolean;
  raw_payload_read: boolean;
  parser_executed: boolean;
  parsed_calendar_rows_created: boolean;
  parsed_market_rows_created: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ExecuteControlledShadowMarketDataParserAttemptRequest = {
  expected_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_input_manifest_sha256: string;
  execution_reason: string;
  exact_stage_51_through_stage_101_binding_confirmed: boolean;
  executor_independent_from_complete_prior_chain_confirmed: boolean;
  one_shot_failure_consumes_claim_and_no_retry_confirmed: boolean;
  artifact_is_declarative_not_spawned_or_executed_confirmed: boolean;
  only_fixed_stage_94_payloads_are_read_only_opened_and_rehashed_confirmed: boolean;
  strict_parser_and_cross_source_reconciliation_fail_closed_confirmed: boolean;
  output_create_once_untrusted_and_requires_independent_validation_confirmed: boolean;
  no_network_environment_secret_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  stage_101_attempt_id: string;
  stage_101_claim_sha256: string;
  completed_at: string;
  executed_by: string;
  execution_reason: string;
  duration_millis: number;
  status: "completed_with_untrusted_output" | "failed_claim_consumed";
  bounded_error_code: string | null;
  output_sha256: string | null;
  output_relative_path: string | null;
  claim_consumed: boolean;
  artifact_revalidated: boolean;
  artifact_spawned_or_executed: boolean;
  raw_payloads_opened: boolean;
  parser_executed_in_process: boolean;
  output_untrusted: boolean;
  independent_validation_completed: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserExecutionAttemptRegistry = {
  schema_version: string;
  policy_version: string;
  execution_endpoint_available: boolean;
  pending_claims: ControlledShadowMarketDataParserExecutionAttemptClaim[];
  results: ControlledShadowMarketDataParserExecutionAttemptResult[];
  pending_claim_count: number;
  terminal_result_count: number;
  successful_untrusted_output_count: number;
  failed_consumed_claim_count: number;
  next_gate: string;
  arbitrary_artifact_execution_allowed: boolean;
  outbound_network_allowed: boolean;
  independent_validation_completed: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateControlledShadowMarketDataParserOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_input_manifest_sha256: string;
  expected_stage_94_validation_sha256: string;
  validation_reason: string;
  exact_current_stage_51_through_stage_102_binding_confirmed: boolean;
  validator_independent_from_executor_and_complete_prior_chain_confirmed: boolean;
  stage_102_result_output_and_create_once_custody_reopened_confirmed: boolean;
  fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed: boolean;
  second_implementation_does_not_call_stage_102_parser_helpers_confirmed: boolean;
  every_canonical_row_hash_and_complete_output_exactly_compared_confirmed: boolean;
  official_calendar_spy_coverage_subject_gaps_and_actions_fail_closed_confirmed: boolean;
  source_available_at_remains_unverified_confirmed: boolean;
  pass_only_opens_future_observation_input_admission_review_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowMarketDataParserOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  stage_102_attempt_id: string;
  stage_101_claim_sha256: string;
  stage_102_result_id: string;
  stage_102_result_sha256: string;
  stage_102_output_sha256: string;
  stage_101_input_manifest_sha256: string;
  stage_94_validation_sha256: string;
  validated_at: string;
  validated_by: string;
  validation_reason: string;
  excluded_prior_actor_ids: string[];
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validator_independent_from_executor_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_102_chain_verified: boolean;
  claim_fingerprint_independently_verified: boolean;
  result_fingerprint_independently_verified: boolean;
  output_file_custody_and_fingerprint_verified: boolean;
  raw_payload_custody_and_fingerprints_verified: boolean;
  canonical_rows_independently_reparsed: boolean;
  every_row_hash_independently_verified: boolean;
  complete_output_exact_match_verified: boolean;
  official_calendar_and_spy_coverage_verified: boolean;
  source_available_at_verified: boolean;
  no_downstream_authority_verified: boolean;
  recomputed_claim_sha256: string;
  recomputed_result_sha256: string;
  recomputed_persisted_output_sha256: string;
  independently_recomputed_output_sha256: string;
  observed_output_bytes: number;
  observed_raw_payload_count: number;
  observed_raw_payload_bytes: number;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_exact_canonical_parse_output"
    | "failed_independent_canonical_parse_output_validation";
  canonical_parse_output_independently_validated: boolean;
  future_observation_input_admission_review_eligible: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_or_metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowMarketDataParserOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    claim: ControlledShadowMarketDataParserExecutionAttemptClaim;
    result: ControlledShadowMarketDataParserExecutionAttemptResult;
    validation: ControlledShadowMarketDataParserOutputValidationRecord | null;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_output_count: number;
  failed_validation_count: number;
  future_observation_input_admission_review_eligible_count: number;
  validation_status: string;
  next_gate: string;
  independent_output_validation_available: boolean;
  source_available_at_verified: boolean;
  forward_observation_started: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationInputAdmissionVerdict =
  | "approved_for_future_create_once_observation_materialization_specification_registration"
  | "changes_requested"
  | "rejected";

export type ReviewControlledShadowObservationInputAdmissionRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_stage_103_validation_id: string;
  expected_stage_103_validation_sha256: string;
  expected_stage_102_result_sha256: string;
  expected_stage_102_output_sha256: string;
  expected_stage_101_claim_sha256: string;
  expected_stage_101_input_manifest_sha256: string;
  expected_cycle_claim_sha256: string;
  verdict: ControlledShadowObservationInputAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_103_binding_confirmed: boolean;
  reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: boolean;
  stage_103_full_reparse_validation_current_and_passed_confirmed: boolean;
  cycle_claim_natural_forward_only_and_no_backfill_confirmed: boolean;
  fixed_subject_spy_window_and_request_identities_confirmed: boolean;
  every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: boolean;
  custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed: boolean;
  admitted_rows_within_frozen_window_and_available_before_admission_confirmed: boolean;
  official_sessions_and_spy_three_price_bases_complete_confirmed: boolean;
  subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: boolean;
  dividends_splits_and_three_price_bases_remain_separate_confirmed: boolean;
  exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: boolean;
  approval_only_opens_future_materialization_specification_registration_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationInputAdmissionReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id: string | null;
  previous_review_sha256: string | null;
  stage_102_attempt_id: string;
  stage_101_claim_sha256: string;
  stage_101_input_manifest_sha256: string;
  stage_102_result_id: string;
  stage_102_result_sha256: string;
  stage_102_output_sha256: string;
  stage_103_validation_id: string;
  stage_103_validation_sha256: string;
  cycle_claim_id: string;
  cycle_claim_sha256: string;
  subject_symbols: string[];
  benchmark_symbol: string;
  window_start_date: string;
  window_end_date: string;
  source_receipt_count: number;
  latest_source_retrieved_at_utc: string;
  parser_completed_at_utc: string;
  independently_validated_at_utc: string;
  submitted_at: string;
  admitted_available_at_utc: string;
  availability_basis: string;
  provider_publication_time_limitation: string;
  provider_publication_time_verified: boolean;
  custody_retrieval_time_verified: boolean;
  official_market_session_count: number;
  price_row_count: number;
  dividend_row_count: number;
  split_row_count: number;
  explicit_gap_count: number;
  earliest_market_session_date: string;
  latest_market_session_date: string;
  submitted_by: string;
  verdict: ControlledShadowObservationInputAdmissionVerdict;
  rationale: string;
  known_limitations: string;
  structural_input_audit_passed: boolean;
  observation_input_admitted: boolean;
  future_create_once_observation_materialization_specification_registration_eligible: boolean;
  observation_materialization_specification_registered: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_or_metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationInputAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      cycle_claim: ControlledShadowFirstNaturalForwardCycleClaim;
      parser_output: {
        claim: ControlledShadowMarketDataParserExecutionAttemptClaim;
        result: ControlledShadowMarketDataParserExecutionAttemptResult;
        validation: ControlledShadowMarketDataParserOutputValidationRecord;
      };
    };
    latest_review: ControlledShadowObservationInputAdmissionReview | null;
    current_binding: boolean;
    review_eligible: boolean;
    observation_input_admitted: boolean;
  }>;
  independently_validated_input_candidate_count: number;
  review_eligible_candidate_count: number;
  reviewed_candidate_count: number;
  admitted_input_count: number;
  changes_requested_or_rejected_count: number;
  future_observation_materialization_specification_registration_eligible_count: number;
  admission_status: string;
  next_gate: string;
  admission_review_available: boolean;
  provider_publication_time_verified: boolean;
  custody_retrieval_time_floor_required: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationMaterializationSpecificationRequest = {
  expected_stage_104_review_sha256: string;
  expected_stage_103_validation_sha256: string;
  expected_stage_102_result_sha256: string;
  expected_stage_102_output_sha256: string;
  expected_stage_101_claim_sha256: string;
  expected_stage_101_input_manifest_sha256: string;
  expected_cycle_claim_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_104_binding_confirmed: boolean;
  registrar_independent_from_stage_104_and_complete_prior_chain_confirmed: boolean;
  exact_admitted_output_only_no_refetch_or_reparse_confirmed: boolean;
  conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed: boolean;
  dividends_splits_and_price_bases_remain_separate_confirmed: boolean;
  initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed: boolean;
  deterministic_canonical_order_decimal_and_row_hash_rules_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed: boolean;
  spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed: boolean;
  specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: boolean;
  no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  future_chain_external_specification_review_required_before_implementation_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationSpecificationRegistration = {
  schema_version: string;
  policy_version: string;
  registration_id: string;
  registration_sha256: string;
  registered_at: string;
  registered_by: string;
  stage_104_review_id: string;
  stage_104_review_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  specification: {
    schema_version: string;
    specification_sha256: string;
    materialization_protocol_version: string;
    stage_104_review_id: string;
    stage_104_review_sha256: string;
    stage_103_validation_id: string;
    stage_103_validation_sha256: string;
    stage_102_attempt_id: string;
    stage_102_result_sha256: string;
    stage_102_output_sha256: string;
    stage_101_claim_sha256: string;
    stage_101_input_manifest_sha256: string;
    cycle_claim_id: string;
    cycle_claim_sha256: string;
    subject_symbols: string[];
    benchmark_symbol: string;
    window_start_date: string;
    window_end_date: string;
    official_market_session_count: number;
    admitted_available_at_utc: string;
    availability_basis: string;
    provider_publication_time_verified: boolean;
    allowed_price_bases: string[];
    future_output_relative_path_template: string;
    one_envelope_per_admitted_cycle: boolean;
    create_once_required: boolean;
    overwrite_allowed: boolean;
    retroactive_backfill_allowed: boolean;
    initial_shadow_allocation_recomputed: boolean;
    accounting_transition_applied: boolean;
    future_output_untrusted: boolean;
    future_output_independent_validation_required: boolean;
  };
  status: string;
  confirmations_complete: boolean;
  specification_registered: boolean;
  future_chain_external_specification_review_eligible: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationSpecificationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  candidates: Array<{
    stage_104_review_id: string;
    stage_104_review_sha256: string;
    stage_103_validation_sha256: string;
    stage_102_attempt_id: string;
    stage_102_result_sha256: string;
    stage_102_output_sha256: string;
    stage_101_claim_sha256: string;
    stage_101_input_manifest_sha256: string;
    cycle_claim_sha256: string;
    subject_symbols: string[];
    benchmark_symbol: string;
    admitted_available_at_utc: string;
    official_market_session_count: number;
    explicit_gap_count: number;
    registrar_excluded_actor_ids: string[];
  }>;
  registrations: ControlledShadowObservationMaterializationSpecificationRegistration[];
  admitted_input_count: number;
  registration_eligible_count: number;
  specification_registered_count: number;
  future_chain_external_specification_review_eligible_count: number;
  specification_status: string;
  next_gate: string;
  implementation_present: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationMaterializationSpecificationReviewVerdict =
  | "approved_for_future_zero_capability_observation_materialization_implementation_registration"
  | "changes_required_rebuild_observation_materialization_specification"
  | "rejected_observation_materialization_specification";

export type ReviewControlledShadowObservationMaterializationSpecificationRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_registration_sha256: string;
  expected_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowObservationMaterializationSpecificationReviewVerdict;
  rationale: string;
  binding_and_second_implementation_assessment: string;
  session_price_basis_and_gap_assessment: string;
  corporate_action_decimal_order_and_hash_assessment: string;
  initial_allocation_and_availability_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  exact_current_stage_51_through_stage_105_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed: boolean;
  rebuilt_specification_exactly_matches_registered_specification_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_and_price_bases_remain_separate_confirmed: boolean;
  decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: boolean;
  initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: boolean;
  conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_zero_capability_implementation_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationSpecificationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  registration_id: string;
  registration_sha256: string;
  specification_sha256: string;
  registration_hash_independently_reproduced: boolean;
  specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_105_binding_valid: boolean;
  complete_specification_rebuilt_without_stage_105_builder: boolean;
  rebuilt_specification_exactly_matches_registration: boolean;
  session_subject_spy_three_price_basis_and_gap_contract_valid: boolean;
  corporate_action_decimal_order_hash_and_output_path_contract_valid: boolean;
  initial_shadow_allocation_and_availability_contract_valid: boolean;
  all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ControlledShadowObservationMaterializationSpecificationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id: string | null;
  previous_review_sha256: string | null;
  registration: ControlledShadowObservationMaterializationSpecificationRegistration;
  independent_audit: ControlledShadowObservationMaterializationSpecificationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowObservationMaterializationSpecificationReviewVerdict;
  rationale: string;
  known_limitations: string;
  future_implementation_constraints: string;
  specification_independently_approved: boolean;
  future_zero_capability_implementation_registration_eligible: boolean;
  implementation_registered: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationSpecificationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_endpoint_available: boolean;
  items: Array<{
    registration: ControlledShadowObservationMaterializationSpecificationRegistration;
    current_independent_audit: ControlledShadowObservationMaterializationSpecificationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review: ControlledShadowObservationMaterializationSpecificationReviewRecord | null;
    review_eligible: boolean;
    future_zero_capability_implementation_registration_eligible: boolean;
  }>;
  specification_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_zero_capability_implementation_registration_eligible_count: number;
  review_status: string;
  implementation_registered: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationMaterializationImplementationRequest = {
  expected_specification_review_id: string;
  expected_specification_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_specification_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_projection_semantics: string;
  session_price_basis_and_gap_semantics: string;
  corporate_action_decimal_order_and_hash_semantics: string;
  initial_allocation_and_availability_semantics: string;
  error_and_missing_data_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_stage_51_through_stage_106_binding_confirmed: boolean;
  registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_review_registration_specification_and_audit_confirmed: boolean;
  zero_capability_contract_only_no_source_or_executable_artifact_confirmed: boolean;
  exact_stage_104_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_and_price_bases_remain_separate_confirmed: boolean;
  decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: boolean;
  initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: boolean;
  conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_specification_registration: ControlledShadowObservationMaterializationSpecificationRegistration;
  upstream_specification_review: ControlledShadowObservationMaterializationSpecificationReviewRecord;
  implementation_name: string;
  implementation_description: string;
  implementation_contract: {
    schema_version: string;
    contract_sha256: string;
    implementation_protocol_version: string;
    immutable_code_revision: string;
    stage_106_specification_review_id: string;
    stage_106_specification_review_sha256: string;
    stage_106_independent_audit_sha256: string;
    stage_105_registration_id: string;
    stage_105_registration_sha256: string;
    observation_materialization_specification_sha256: string;
    exact_observation_materialization_specification: ControlledShadowObservationMaterializationSpecificationRegistration["specification"];
    current_source_binding_validation_function_id: string;
    canonical_session_projection_function_id: string;
    three_price_basis_projection_function_id: string;
    explicit_gap_and_spy_fail_closed_function_id: string;
    corporate_action_separation_function_id: string;
    initial_allocation_binding_function_id: string;
    conservative_availability_function_id: string;
    canonical_envelope_serialization_and_hash_function_id: string;
    future_exact_admitted_input_read_only_and_content_addressed: boolean;
    future_observation_output_create_once_and_untrusted: boolean;
    future_observation_output_independent_validation_required: boolean;
    registered_not_run: boolean;
    independent_implementation_review_required: boolean;
    isolated_runner_registration_required_after_review: boolean;
  };
  status: string;
  confirmations_complete: boolean;
  zero_capability_implementation_contract_registered: boolean;
  observation_materialization_implementation_present: boolean;
  future_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  isolated_runner_registration_eligible: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  items: Array<{
    specification_review: ControlledShadowObservationMaterializationSpecificationReviewRecord;
    specification_registration: ControlledShadowObservationMaterializationSpecificationRegistration;
    implementation: ControlledShadowObservationMaterializationImplementationRecord | null;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  independently_approved_specification_count: number;
  registration_eligible_count: number;
  implementation_contract_count: number;
  current_binding_implementation_contract_count: number;
  independent_implementation_review_eligible_count: number;
  implementation_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationMaterializationImplementationReviewVerdict =
  | "approved_for_future_isolated_observation_materialization_runner_specification_registration"
  | "changes_required_rebuild_observation_materialization_implementation"
  | "rejected_observation_materialization_implementation";

export type ReviewControlledShadowObservationMaterializationImplementationRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_independent_audit_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_materialization_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowObservationMaterializationImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_projection_semantics_assessment: string;
  session_price_basis_gap_and_company_action_assessment: string;
  initial_allocation_availability_and_output_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  exact_current_stage_51_through_stage_107_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: boolean;
  exact_stage_104_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: boolean;
  initial_shadow_allocation_and_conservative_availability_preserved_confirmed: boolean;
  provider_publication_time_remains_unverified_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  specification_review_sha256: string;
  specification_independent_audit_sha256: string;
  specification_registration_sha256: string;
  observation_materialization_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  specification_review_hash_independently_reproduced: boolean;
  specification_independent_audit_hash_independently_reproduced: boolean;
  specification_registration_hash_independently_reproduced: boolean;
  observation_materialization_specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_107_binding_valid: boolean;
  eight_function_ids_and_canonical_schemas_valid: boolean;
  admitted_input_session_price_gap_and_company_action_contract_valid: boolean;
  allocation_availability_create_once_and_output_path_contract_valid: boolean;
  provider_publication_time_still_unverified: boolean;
  all_artifact_runtime_input_observation_store_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ControlledShadowObservationMaterializationImplementationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id: string | null;
  previous_review_sha256: string | null;
  implementation: ControlledShadowObservationMaterializationImplementationRecord;
  independent_audit: ControlledShadowObservationMaterializationImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowObservationMaterializationImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_projection_semantics_assessment: string;
  session_price_basis_gap_and_company_action_assessment: string;
  initial_allocation_availability_and_output_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  reviewer_independent_from_registrar_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_107_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: boolean;
  exact_stage_104_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: boolean;
  initial_shadow_allocation_and_conservative_availability_preserved_confirmed: boolean;
  provider_publication_time_remains_unverified_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  zero_capability_implementation_independently_approved: boolean;
  future_isolated_observation_materialization_runner_specification_registration_eligible: boolean;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: ControlledShadowObservationMaterializationImplementationRecord;
    current_independent_audit: ControlledShadowObservationMaterializationImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review: ControlledShadowObservationMaterializationImplementationReviewRecord | null;
    review_eligible: boolean;
    future_isolated_observation_materialization_runner_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_isolated_observation_materialization_runner_specification_registration_eligible_count: number;
  review_status: string;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  observation_materialized: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationMaterializationIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_materialization_specification_sha256: string;
  expected_stage_104_admission_review_sha256: string;
  expected_stage_103_validation_sha256: string;
  expected_stage_102_result_sha256: string;
  expected_stage_101_claim_sha256: string;
  expected_cycle_claim_sha256: string;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_observation_materialization_specification";
  runner_spec_revision: string;
  proposed_runner_code_revision: string;
  proposed_runner_artifact_sha256: string;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_108_binding_confirmed: boolean;
  registrar_independent_from_stage_108_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: boolean;
  all_eight_observation_materialization_functions_and_canonical_schemas_preserved_confirmed: boolean;
  future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: boolean;
  session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: boolean;
  no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_preserved_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  provider_publication_time_remains_unverified_until_separate_evidence_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationIsolatedRunnerRecord = {
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: ControlledShadowObservationMaterializationImplementationRecord;
  implementation_review: ControlledShadowObservationMaterializationImplementationReviewRecord;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_observation_materialization_specification";
  runner_contract: {
    contract_sha256: string;
    runner_spec_revision: string;
    proposed_runner_code_revision: string;
    proposed_runner_artifact_sha256: string;
    runtime_identity: string;
    runtime_version: string;
    future_input_envelope: string;
    future_output_envelope: string;
    next_gate: string;
    future_runner_artifact_identity_bound: boolean;
    source_artifact_present: boolean;
    executable_artifact_present: boolean;
    callable_entrypoint_present: boolean;
    runtime_instantiated: boolean;
    input_mount_present: boolean;
    input_read_allowed: boolean;
    maximum_parallel_runs: number;
    maximum_memory_mib: number;
    maximum_wall_clock_seconds: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes: number;
  };
  status: string;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  input_accessed: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  observation_materialized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: ControlledShadowObservationMaterializationImplementationRecord;
    review: ControlledShadowObservationMaterializationImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  items: Array<{
    runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_accessed: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  observation_materialized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowForwardObservationExecutionAttemptResult = {
  schema_version: string;
  execution_policy_version: string;
  result_id: string;
  result_sha256: string;
  attempt_id: string;
  claim_sha256: string;
  status: "completed_with_untrusted_initialization_receipt" | "failed_authorization_consumed";
  started_at: string;
  finished_at: string;
  duration_millis: number;
  isolation_backend: string;
  exit_code: number;
  failure_reason?: string;
  current_binary_digest_reverified: boolean;
  initialization_manifest_validated: boolean;
  initialization_completed: boolean;
  untrusted_initialization_receipt?: ControlledShadowForwardObservationUntrustedInitializationReceipt;
  output_sha256?: string;
  independent_output_validation_completed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
};

export type ControlledShadowForwardObservationExecutionAttemptRegistry = {
  schema_version: string;
  execution_policy_version: string;
  attempts: Array<{
    claim: ControlledShadowForwardObservationExecutionAttemptClaim;
    result?: ControlledShadowForwardObservationExecutionAttemptResult;
  }>;
  invocation_eligible_authorization_count: number;
  claim_count: number;
  completed_count: number;
  failed_count: number;
  interrupted_count: number;
  independent_validation_eligible_count: number;
  execution_status: string;
  persistent_runtime_instantiated: boolean;
  market_data_accessed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_written: boolean;
  scalar_reward_written: boolean;
  order_generated: boolean;
  broker_accessed: boolean;
  trade_executed: boolean;
  scope: string;
};

export type HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    attempt: {
      claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim;
      result: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult;
    };
    validation?: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_untrusted_candidate_count: number;
  failed_validation_count: number;
  future_candidate_admission_review_eligible_count: number;
  validation_status: string;
  independent_output_validation_available: boolean;
  official_joined_dataset_created: boolean;
  copied_to_training_store: boolean;
  training_authorized: boolean;
  reward_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_runner_code_revision: string;
  expected_runner_contract_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_transformation_spec_sha256: string;
  expected_dataset_content_sha256: string;
  verdict: HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_runner_and_complete_upstream_binding_confirmed: boolean;
  reviewer_independence_from_complete_prior_chain_confirmed: boolean;
  runner_artifact_digest_independently_reproduced: boolean;
  immutable_code_revision_reproducible_and_artifact_available_confirmed: boolean;
  sealed_read_only_inputs_and_root_filesystem_confirmed: boolean;
  unprivileged_and_no_new_privileges_confirmed: boolean;
  ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: boolean;
  fixed_runtime_and_resource_limits_confirmed: boolean;
  no_host_environment_variables_or_secrets_confirmed: boolean;
  no_network_tools_child_process_production_or_history_access_confirmed: boolean;
  deterministic_split_feature_and_canonical_schema_contract_confirmed: boolean;
  authorization_single_use_and_24_hour_expiry_confirmed: boolean;
  authorization_execution_output_validation_and_training_separation_confirmed: boolean;
  no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: boolean;
};

export type ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_implementation_spec_sha256: string;
  expected_admission_review_sha256: string;
  expected_validation_sha256: string;
  expected_output_sha256: string;
  expected_snapshot_sha256: string;
  expected_protocol_sha256: string;
  verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict;
  rationale: string;
  implementation_fingerprint_confirmed: boolean;
  current_upstream_bindings_confirmed: boolean;
  code_revision_reproducible_confirmed: boolean;
  deterministic_raw_envelope_only_confirmed: boolean;
  exact_metric_bit_preservation_confirmed: boolean;
  provenance_and_limitations_preserved_confirmed: boolean;
  create_once_isolated_output_confirmed: boolean;
  missing_data_fail_closed_confirmed: boolean;
  no_network_tools_or_production_access_confirmed: boolean;
  no_semantic_action_position_or_reward_inference_confirmed: boolean;
  no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: boolean;
};

export type InvestmentCausalEffectCohort = {
  cohort_id: string;
  label: string;
  available_links: number;
  reviewed_links: number;
  accepted_links: number;
  rejected_links: number;
  supporting_links: number;
  falsifying_links: number;
  mixed_links: number;
  context_only_links: number;
  unclassified_accepted_links: number;
  review_rate_percent: number;
  support_share_percent?: number;
  falsification_share_percent?: number;
};

export type InvestmentEvidenceReviewQueueItem = {
  queue_id: string;
  symbol: string;
  company_name: string;
  sample_id: string;
  decision_at: string;
  driver_id: string;
  driver_label: string;
  mechanism: string;
  kind: "source_claim" | "operating_kpi" | "computed_comparison" | "computed_ratio";
  status: "pending" | "accepted" | "rejected";
  priority: "blocked" | "high" | "normal" | string;
  priority_reasons: string[];
  source_review_ready: boolean;
  source_review_contract: "numeric" | "qualitative";
  source_review_blockers: string[];
  evidence_identity_sha256?: string;
  observation: InvestmentCausalObservation;
  review_explanation?: string | null;
  review_effect?: "unclassified" | "supports" | "falsifies" | "mixed" | "context_only" | null;
  review_source_verification?: "unchecked" | "verified_against_source" | "evidence_mismatch" | "insufficient_source_context" | null;
  source_review_id?: string | null;
  source_review_note?: string | null;
  source_reviewed_at?: string | null;
  source_review_origin_sample_id?: string | null;
  source_review_reused_across_snapshots?: boolean;
  source_review_conflict?: boolean;
  training_label_eligible: boolean;
  reviewed_at?: string | null;
};

export type InvestmentEvidenceReviewQueue = {
  schema_version: string;
  generated_at: string;
  symbol_filter?: string | null;
  status_filter: "all" | "pending" | "accepted" | "rejected" | string;
  kind_filter: "all" | "source_claim" | "operating_kpi" | "computed_comparison" | "computed_ratio" | string;
  selection_mode: "full_queue" | "source_batch" | "old_wang_batch" | "active_batch" | string;
  selection_policy_version: string;
  selection_scope: string;
  selected_symbols: string[];
  selected_drivers: string[];
  total_candidates: number;
  pending_candidates: number;
  accepted_candidates: number;
  rejected_candidates: number;
  source_review_ready_candidates: number;
  source_blocked_candidates: number;
  source_unreviewed_candidates: number;
  source_verified_waiting_causal_candidates: number;
  source_excluded_candidates: number;
  source_review_reused_candidates?: number;
  source_review_conflicted_candidates?: number;
  old_wang_reviewer_configured: boolean;
  old_wang_submission_authorized: boolean;
  supporting_candidates: number;
  falsifying_candidates: number;
  mixed_candidates: number;
  context_only_candidates: number;
  items: InvestmentEvidenceReviewQueueItem[];
};

export type MetaInfo = {
  name: string;
  version: string;
  channel: string;
  supportsImessage: boolean;
  apiVersion: string;
  capabilities: string[];
  deploymentMode: "local" | "remote";
  /** Admin/console default UI language. "zh" or "en". Optional for backwards compat with older backends. */
  language?: "zh" | "en";
  build?: {
    git_sha?: string | null;
    build_timestamp?: string | null;
    profile: "debug" | "release" | string;
    source?: "workspace" | "direct_source_runtime" | "unknown";
    binary_sha256?: string | null;
  };
  acp_profiles?: Array<{
    runner: string;
    adapter: string;
    detected_version: string;
    baseline_version: string;
    dialect: string;
    compatibility: "validated" | "compatible_newer" | string;
    companion_versions: Record<string, string>;
    detected_at: string;
    build_git_sha?: string | null;
  }>;
};

export type BackendConfig = {
  mode: "bundled" | "remote";
  baseUrl: string;
  bearerToken: string;
};

export type BackendStatusInfo = {
  config: BackendConfig;
  resolvedBaseUrl?: string;
  connected: boolean;
  lastError?: string;
  meta?: MetaInfo;
  diagnostics?: {
    configDir: string;
    dataDir: string;
    logsDir: string;
    desktopLog: string;
    sidecarLog: string;
  };
};

export type DesktopChannelSettings = {
  configPath: string;
  imessageEnabled: boolean;
  imessageTargetHandle?: string;
  feishuEnabled: boolean;
  feishuAppId?: string;
  feishuAppSecret?: string;
  feishuChatScope?: string;
  feishuAllowEmails?: string[];
  feishuAllowMobiles?: string[];
  feishuAllowOpenIds?: string[];
  telegramEnabled: boolean;
  telegramBotToken?: string;
  telegramChatScope?: string;
  telegramAllowFrom?: string[];
  discordEnabled: boolean;
  discordBotToken?: string;
  discordChatScope?: string;
  discordAllowFrom?: string[];
};

/** agent.runner values accepted by config/admin surfaces; gemini_acp is legacy/parseable but disabled at runtime. */
export type AgentProvider =
  | "gemini_cli"
  | "gemini_acp"
  | "codex_cli"
  | "codex_acp"
  | "opencode_acp"
  | "hone_cloud";

export type AuxiliarySettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
};

export type HoneCloudSettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
};

export type LlmProfileEntrySettings = {
  id: string;
  provider: string;
  model: string;
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  reasoningEffort?: string;
  reasoningMaxTokens?: number;
  responseFormatJson: boolean;
};

export type LlmProfileSettings = {
  defaultProfile: string;
  auxiliaryProfile: string;
  polishProfile: string;
  newsClassifierProfile: string;
  filingSummaryProfile: string;
  earningsQualityProfile: string;
  digestPass1Profile: string;
  digestPass2Profile: string;
  digestEventDedupeProfile: string;
  mainlineDistillProfile: string;
  profiles: LlmProfileEntrySettings[];
};

export type AgentSettings = {
  /** AgentProvider value; gemini_acp is legacy/disabled at runtime. */
  runner: AgentProvider;
  /** codex_cli 专用；其他 provider 留空 */
  codexModel: string;
  /** OpenAI 协议渠道 Base URL（agent.opencode.api_base_url） */
  openaiUrl: string;
  /** OpenAI 协议渠道模型名（agent.opencode.model） */
  openaiModel: string;
  /** OpenAI 协议渠道 API Key（agent.opencode.api_key） */
  openaiApiKey: string;
  /** OpenAI-compatible auxiliary 配置，用于心跳/压缩等后台任务 */
  auxiliary?: AuxiliarySettings;
  /** HONE Cloud 用户端服务配置 */
  honeCloud?: HoneCloudSettings;
  /** Named LLM profiles used by runtime subsystems */
  llmProfiles?: LlmProfileSettings;
};

export type AgentSettingsUpdateResult = {
  settings: AgentSettings;
  restartedBundledBackend: boolean;
  message: string;
  backendStatus?: BackendStatusInfo;
};

export type CliCheckResult = {
  ok: boolean;
  message: string;
};

/** FMP API Key 设置（保存在 canonical config.yaml 的 fmp.api_keys，支持多 Key fallback） */
export type FmpSettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

/** Tavily API Key 设置（保存在 canonical config.yaml 的 search.api_keys，支持多 Key fallback） */
export type TavilySettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

export type DesktopChannelSettingsInput = Omit<
  DesktopChannelSettings,
  "configPath"
>;

export type DesktopChannelSettingsUpdateResult = {
  settings: DesktopChannelSettings;
  restartedBundledBackend: boolean;
  message: string;
  backendStatus?: BackendStatusInfo;
};

export type ChannelProcessCleanupEntry = {
  channel: string;
  keptPid?: number;
  removedPids: number[];
};

export type ChannelProcessCleanupResult = {
  entries: ChannelProcessCleanupEntry[];
  message: string;
};

export type ChannelStatusInfo = {
  id: string;
  label: string;
  enabled: boolean;
  running: boolean;
  status: "running" | "disabled" | "stopped" | "unsupported" | string;
  pid?: number;
  last_heartbeat_at?: string;
  detail: string;
  processes: ChannelProcessInfo[];
};

export type ChannelProcessInfo = {
  pid: number;
  running: boolean;
  started_at?: string;
  last_heartbeat_at?: string;
  managed_by_desktop?: boolean;
  source?: string;
};

export type ChatStreamEvent =
  | {
      event: "run_started";
      data: Partial<PublicChatActiveRun> & { runner?: string; text?: string };
    }
  | {
      event: "run_progress";
      data: Partial<PublicChatActiveRun> & { text?: string };
    }
  | { event: "assistant_delta"; data: { content?: string } }
  | { event: "reasoning_delta"; data: { content?: string } }
  | { event: "assistant_reset"; data: Record<string, never> }
  | {
      event: "tool_call";
      data: {
        public_status_text?: string;
        tool?: string;
        status?: string;
        text?: string;
        reasoning?: string;
      };
    }
  | { event: "run_error"; data: { message?: string } }
  | {
      event: "run_finished";
      data: { success?: boolean; partial?: boolean };
    }
  /** actor 创建失败等路径（chat.rs 早期返回） */
  | { event: "error"; data: { text?: string } }
  /** 流结束标记（与 run_finished 二选一出现） */
  | { event: "done"; data: Record<string, unknown> };

/** 消息处理阶段 */
export type PendingPhase =
  | "queued" // 已发出请求，等待后端确认
  | "thinking" // run_started 到达，AI 正在思考
  | "running" // tool_call 到达，正在调用工具
  | "streaming" // assistant_delta 到达，流式输出中
  | "error" // 发生错误
  | "timeout"; // 请求超时

/** 每个会话独立的消息处理状态（替代全局 thinking/sending/thinkingText） */
export type PendingState = {
  id: string;
  startedAt: number; // Date.now()，用于计算已运行时长
  phase: PendingPhase;
  statusText: string; // "正在思考…" / "调用工具: web_search" / 错误原因
  partialContent: string; // 流式累积的 assistant 文本
};

export type TimelineMessage =
  | {
      id: string;
      kind: "user";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
      scheduledPush?: HistoryScheduledPush;
      financeCalendar?: HistoryFinanceCalendar;
    }
  | {
      id: string;
      kind: "assistant";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
      scheduledPush?: HistoryScheduledPush;
      financeCalendar?: HistoryFinanceCalendar;
    }
  | {
      id: string;
      kind: "system";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
    }
  | {
      id: string;
      kind: "scheduled";
      content: string;
      jobName?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
    };

export type CronJobInfo = {
  id: string;
  channel: string;
  user_id: string;
  channel_scope?: string;
  name: string;
  task_prompt: string;
  schedule: {
    hour: number;
    minute: number;
    repeat: string;
    weekday?: number;
  };
  tags?: string[];
  push?: Record<string, unknown>;
  enabled: boolean;
  channel_target: string;
  created_at: string;
  updated_at: string;
  last_run_at?: string;
  next_run_at?: string;
};

export type CronJobExecutionInfo = {
  run_id: number;
  job_id: string;
  job_name: string;
  channel: string;
  user_id: string;
  channel_scope?: string;
  channel_target: string;
  heartbeat: boolean;
  executed_at: string;
  execution_status: string;
  message_send_status: string;
  should_deliver: boolean;
  delivered: boolean;
  response_preview?: string;
  error_message?: string;
  detail?: Record<string, unknown> | null;
};

export type CronJobDetailInfo = {
  job: CronJobInfo;
  executions: CronJobExecutionInfo[];
};

export type CronJobUpsertInput = {
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  name?: string;
  hour?: number;
  minute?: number;
  repeat?: string;
  weekday?: number;
  task_prompt?: string;
  push?: Record<string, unknown>;
  enabled?: boolean;
  channel_target?: string;
  tags?: string[];
};

export type HoldingInfo = {
  symbol: string;
  asset_type?: string;
  shares: number;
  avg_cost: number;
  underlying?: string;
  option_type?: string;
  strike_price?: number;
  expiration_date?: string;
  contract_multiplier?: number;
  holding_horizon?: "long_term" | "short_term";
  strategy_notes?: string;
  notes?: string;
  tracking_only?: boolean;
};

export type PortfolioInfo = {
  actor?: {
    channel: string;
    user_id: string;
    channel_scope?: string;
  };
  user_id: string;
  holdings: HoldingInfo[];
  updated_at: string;
} | null;

export type PortfolioSummary = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  holdings_count: number;
  watchlist_count?: number;
  total_shares: number;
  updated_at?: string;
};

export type HoldingUpsertInput = {
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  symbol?: string;
  asset_type?: string;
  shares?: number;
  quantity?: number;
  avg_cost?: number;
  cost_basis?: number;
  underlying?: string;
  option_type?: string;
  strike_price?: number;
  expiration_date?: string;
  contract_multiplier?: number;
  holding_horizon?: "long_term" | "short_term" | "";
  strategy_notes?: string;
  notes?: string;
  tracking_only?: boolean;
};

// ── 日志 ─────────────────────────────────────────────────────────────────────

export type LogEntry = {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  file?: string;
  line?: number;
  message_id?: string;
  state?: string;
  extra?: Record<string, unknown>;
};

// ── 个股深度研究 ─────────────────────────────────────────────────────────────

export type ResearchTaskStatus = "pending" | "running" | "completed" | "error";

export type ResearchTask = {
  /** 本地生成的唯一 ID（用于前端列表 key） */
  id: string;
  /** 外部 API 返回的 task_id */
  task_id: string;
  /** 外部 API 返回的 task_name */
  task_name: string;
  /** 用户输入的公司名称 */
  company_name: string;
  /** 当前任务状态 */
  status: ResearchTaskStatus;
  /** 进度字符串，例如 "60%"、"100%" */
  progress: string;
  /** 任务创建时间（ISO 字符串） */
  created_at: string;
  /** 最近更新时间 */
  updated_at?: string;
  /** 完成时间 */
  completed_at?: string;
  /** 研究结果 Markdown 文件的绝对路径（仅供参考，不再用于读取内容） */
  answer_file_path?: string;
  /** 研究报告 Markdown 原文（轮询完成时从 API 直接获取，本地持久化） */
  answer_markdown?: string;
  /** 错误信息 */
  error_message?: string;
};

// ── LLM Audit ────────────────────────────────────────────────────────────────

export type AuditRecordSummary = {
  id: string;
  created_at: string;
  session_id: string;
  actor_channel?: string;
  actor_user_id?: string;
  actor_scope?: string;
  source: string;
  operation: string;
  provider: string;
  model?: string;
  success: boolean;
  latency_ms?: number;
  prompt_tokens?: number;
  completion_tokens?: number;
  total_tokens?: number;
};

export type LlmAuditRecord = AuditRecordSummary & {
  request: unknown;
  response?: unknown;
  error?: string;
  metadata: unknown;
};

export type AuditQueryFilter = {
  actor_channel?: string;
  actor_user_id?: string;
  actor_scope?: string;
  session_id?: string;
  success?: boolean;
  source?: string;
  provider?: string;
  date_from?: string;
  date_to?: string;
  page?: number;
  page_size?: number;
};

// ── Company Profiles ────────────────────────────────────────────────────────

export type CompanyProfileEvent = {
  id: string;
  filename: string;
  title: string;
  updated_at?: string;
  markdown: string;
};

export type CompanyProfile = {
  profile_id: string;
  title: string;
  updated_at?: string;
  markdown: string;
  events: CompanyProfileEvent[];
};

export type CompanyProfileSummary = {
  profile_id: string;
  title: string;
  updated_at?: string;
  event_count: number;
};

export type CompanyProfileSpaceSummary = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  profile_count: number;
  updated_at?: string;
};

export type CompanyProfileConflictDecision = "skip" | "replace";

export type CompanyProfileImportMode =
  "keep_existing" | "replace_all" | "interactive";

export type CompanyProfileImportProfileSummary = {
  profile_id: string;
  company_name: string;
  stock_code: string;
  updated_at: string;
  event_count: number;
  mainline_excerpt: string;
};

export type CompanyProfileImportConflict = {
  imported: CompanyProfileImportProfileSummary;
  existing: CompanyProfileImportProfileSummary;
  reasons: string[];
};

export type CompanyProfileTransferManifestProfile = {
  profile_id: string;
  company_name: string;
  stock_code: string;
  event_count: number;
  updated_at: string;
};

export type CompanyProfileTransferManifest = {
  version: string;
  exported_at: string;
  profile_count: number;
  event_count: number;
  profiles: CompanyProfileTransferManifestProfile[];
};

export type CompanyProfileImportPreview = {
  manifest: CompanyProfileTransferManifest;
  profiles: CompanyProfileImportProfileSummary[];
  conflicts: CompanyProfileImportConflict[];
  importable_count: number;
  conflict_count: number;
  suggested_mode: CompanyProfileImportMode;
};

export type CompanyProfileImportApplyRequest = {
  mode: CompanyProfileImportMode;
  decisions: Record<string, CompanyProfileConflictDecision>;
};

export type CompanyProfileImportApplyResult = {
  imported_profile_ids: string[];
  replaced_profile_ids: string[];
  skipped_profile_ids: string[];
  changed_profile_ids: string[];
  imported_count: number;
  replaced_count: number;
  skipped_count: number;
};

/** One of the caller's own scheduled pushes, as the push page shows it. */
export type PublicSubscription = {
  job_id: string;
  name: string;
  task_prompt: string;
  enabled: boolean;
  channel: string;
  schedule: {
    hour: number;
    minute: number;
    repeat: string;
    weekday?: number | null;
    date?: string | null;
  };
  created_at?: string | null;
  last_run_at?: string | null;
};

// ── Daily company ratings ──────────────────────────────────────────────────

export type CompanyRatingLight = "green" | "yellow" | "red" | "unknown";
export type CompanyRatingDataStatus =
  "live" | "partial" | "transcript_only" | "stale" | "simulation";

export type CompanyRatingDimensions = {
  moat: number;
  scarcity: number;
  fundamentals: number;
  visibility: number;
  growth_quality?: number | null;
  pricing_power?: number | null;
  financial_quality?: number | null;
  valuation?: number | null;
  market_confirmation: number;
  timing?: number | null;
};

export type CompanyRatingMetrics = {
  revenue_growth_percent?: number | null;
  forward_revenue_growth_percent?: number | null;
  gross_margin_percent?: number | null;
  gross_margin_change_pp?: number | null;
  ebit_margin_percent?: number | null;
  fcf_margin_percent?: number | null;
  net_cash_to_revenue_percent?: number | null;
  accounts_receivable_growth_percent?: number | null;
  accounts_payable_growth_percent?: number | null;
  inventory_growth_percent?: number | null;
  property_plant_equipment_growth_percent?: number | null;
  operating_cash_flow_growth_percent?: number | null;
  capital_expenditure_growth_percent?: number | null;
  free_cash_flow_growth_percent?: number | null;
  financial_as_of?: string | null;
  financial_review_status?: string | null;
  financial_score_eligible?: boolean;
  financial_source_claim_ids?: string[];
  financial_source_urls?: string[];
  financial_calculations?: string[];
  financial_source_claims?: InvestmentFinancialSourceClaimTrace[];
  financial_quality_warnings?: string[];
  forward_metric_label?: string | null;
  forward_metric_value?: string | null;
  forward_metric_growth_percent?: number | null;
  forward_metric_as_of?: string | null;
  forward_metric_source_url?: string | null;
};

export type CompanyDailyValuation = {
  as_of: string;
  generated_at_beijing: string;
  currency: string;
  bear_case: number;
  base_case: number;
  bull_case: number;
  current_price: number;
  probability_weighted_value: number;
  expected_upside_percent: number;
  method_count: number;
  confidence: "high" | "medium" | "low" | string;
  current_position: string;
  position_percent: number;
  method: string;
  assumptions: string[];
  sources: string[];
};

export type CompanyMarketHistory = {
  policy_version: string;
  as_of: string;
  source: string;
  source_url: string;
  price_basis: string;
  session_count: number;
  latest_close: number;
  average_close_50?: number | null;
  average_close_200?: number | null;
  return_20_sessions_percent?: number | null;
  return_60_sessions_percent?: number | null;
  drawdown_from_60_session_high_percent?: number | null;
  recent_5_session_volume_vs_prior_55_percent?: number | null;
  quality_status: "usable" | "review_required" | string;
  quality_warnings: string[];
};

export type CompanyRating = {
  name: string;
  symbol: string;
  market_scope: string;
  theme: string;
  value_chain: string;
  score: number;
  light: CompanyRatingLight;
  confidence: "high" | "medium" | "low";
  data_status: CompanyRatingDataStatus;
  price?: number | null;
  change_percent?: number | null;
  price_avg50?: number | null;
  price_avg200?: number | null;
  year_low?: number | null;
  year_high?: number | null;
  market_history?: CompanyMarketHistory | null;
  short_interest?: CompanyShortInterest | null;
  options_positioning?: CompanyOptionsPositioning | null;
  news_attention?: CompanyNewsAttention | null;
  institutional_holdings?: CompanyInstitutionalHoldings | null;
  analyst_consensus?: CompanyAnalystConsensus | null;
  market_as_of?: string | null;
  financial_as_of?: string | null;
  thesis_summary: string;
  business_model: string;
  moat: string;
  valuation_method: string;
  valuation?: CompanyDailyValuation | null;
  valuation_unavailable_reason: string;
  dimensions: CompanyRatingDimensions;
  metrics?: CompanyRatingMetrics;
  score_cap_reason?: string;
  factor_coverage?: number;
  watch_items: string[];
  risks: string[];
  falsifiers: string[];
  research_updated_at: string;
  data_sources: string[];
};

export type CompanyRatingSnapshot = {
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: "Asia/Shanghai" | string;
  data_status: CompanyRatingDataStatus;
  methodology_version: string;
  simulation_note?: string;
  coverage: {
    companies: number;
    quotes: number;
    financials: number;
    financial_observations?: number;
    financials_review_required?: number;
    valuations: number;
  };
  disclaimer: string;
  items: CompanyRating[];
};

// ── Daily valuation lab ──────────────────────────────────────────────────

export type ValuationEvidence = {
  label: string;
  display_value: string;
  as_of: string;
  source: string;
  source_url: string;
};

export type ValuationScenario = {
  id: "bear" | "base" | "bull" | string;
  label: string;
  probability: number;
  initial_growth_rate: number;
  discount_rate: number;
  terminal_growth_rate: number;
  dcf_value?: number | null;
  multiple_value?: number | null;
  methods: Array<{
    id: string;
    label: string;
    value: number;
    weight: number;
    metric: string;
    assumption: string;
  }>;
  fair_value: number;
};

export type ValuationLabItem = {
  symbol: string;
  name: string;
  market_scope: string;
  theme: string;
  status: "ready" | "review_required" | "unavailable" | "stale" | string;
  confidence: "high" | "medium" | "low" | string;
  eligible_for_rating: boolean;
  unavailable_reason: string;
  currency: string;
  current_price?: number | null;
  market_as_of?: string | null;
  financial_as_of?: string | null;
  normalized_fcf?: number | null;
  normalized_fcf_per_share?: number | null;
  net_cash_per_share?: number | null;
  historical_fcf_growth_rate?: number | null;
  current_position: string;
  position_percent?: number | null;
  method: string;
  valuation_profile: string;
  company_method_hint: string;
  scenarios: ValuationScenario[];
  probability_weighted_value?: number | null;
  expected_upside_percent?: number | null;
  reverse_dcf?: {
    status: string;
    implied_growth_rate?: number | null;
    implied_forward_eps?: number | null;
    implied_forward_pe?: number | null;
    explanation: string;
  } | null;
  cross_check?: {
    status: string;
    method_count: number;
    dispersion_percent?: number | null;
    forward_eps?: number | null;
    forward_pe?: number | null;
    pe_value?: number | null;
    dcf_value?: number | null;
    gap_percent?: number | null;
    explanation: string;
  } | null;
  assumptions: string[];
  evidence: ValuationEvidence[];
  readiness: {
    schema_version: string;
    status: string;
    input_mode: string;
    display_price?: number | null;
    market_as_of?: string | null;
    financial_review_status: string;
    valuation_review_status: string;
    valuation_review_id?: string | null;
    valuation_input_fingerprint_sha256?: string | null;
    valuation_financial_evidence_fingerprint_sha256?: string | null;
    valuation_input_as_of?: string | null;
    rating_factor_authorized: boolean;
    sec_valuation_use_authorized: boolean;
    available_inputs: ValuationEvidence[];
    missing_inputs: string[];
    methods: Array<{
      id: string;
      label: string;
      status: "prepared" | "blocked" | string;
      missing_inputs: string[];
    }>;
    scope: string;
  };
};

export type ValuationLabSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: string;
  methodology_version: string;
  status: "live" | "partial" | "data_unavailable" | "stale" | string;
  coverage: {
    companies: number;
    calculated: number;
    cross_checked: number;
    eligible_for_rating: number;
  };
  summary: string;
  methodology_note: string;
  items: ValuationLabItem[];
  disclaimer: string;
};

// ── Daily actor-scoped portfolio news ─────────────────────────────────────

export type PortfolioNewsImpact =
  "positive" | "neutral" | "negative" | "unassessed";

export type ModelAnalysisHealth = {
  policy_version: string;
  status:
    | "healthy"
    | "partial"
    | "unavailable"
    | "unconfigured"
    | "not_required"
    | "pending"
    | "unknown_legacy"
    | string;
  provider_name?: string | null;
  profile_name?: string | null;
  model?: string | null;
  requested_items: number;
  analyzed_items: number;
  failed_items: number;
  failure_reasons: string[];
  decision_use_allowed: boolean;
};

export type PortfolioNewsItem = {
  id: string;
  symbol: string;
  title: string;
  published_at: string;
  published_at_beijing: string;
  source: string;
  source_url: string;
  source_summary: string;
  severity: "high" | "medium" | "low";
  impact: PortfolioNewsImpact;
  horizon: "short" | "medium" | "long" | "unknown";
  thesis_effect: "strengthens" | "unchanged" | "weakens" | "unassessed";
  summary: string;
  why_it_matters: string;
  attention: "立即复核" | "持续观察" | "无需动作";
  confidence: "high" | "medium" | "low";
  analysis_status: "model_analyzed" | "source_only";
  priority_score: number;
};

export type PortfolioNewsSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: "Asia/Shanghai" | string;
  model_version: string;
  status:
    | "live"
    | "partial"
    | "source_only"
    | "no_material_news"
    | "data_unavailable"
    | "no_portfolio"
    | "waiting_refresh"
    | "portfolio_changed"
    | "stale"
    | string;
  source_status: string;
  model_status: string;
  analysis_health: ModelAnalysisHealth;
  portfolio_updated_at: string;
  holdings_count: number;
  lookback_hours: number;
  covered_symbols: string[];
  missing_symbols: string[];
  coverage_items: Array<{
    symbol: string;
    status: "news_found" | "no_material_news" | "source_unavailable" | "pending" | string;
    label: string;
  }>;
  summary: string;
  counts: {
    total: number;
    positive: number;
    neutral: number;
    negative: number;
    immediate_review: number;
  };
  items: PortfolioNewsItem[];
  disclaimer: string;
};

// ── Daily actor-scoped position management ───────────────────────────────

export type PositionManagementAction =
  "increase_candidate" | "hold" | "reduce" | "review" | "insufficient_data";

export type PositionDecisionGate = {
  status: "passed" | "blocked" | "missing" | "mismatched" | "not_applicable" | string;
  revision_id: string;
  decision_at: string;
  policy_version: string;
  skill_version: string;
  pre_methodology_action: string;
  final_action: string;
  confirmed_logic_ids: string[];
  candidate_logic_used: boolean;
  increase_candidate_authorized: boolean;
  portfolio_action_authorized: boolean;
  blocking_reasons: string[];
};

export type PositionPortfolioGate = {
  policy_version: string;
  skill_id: string;
  skill_version: string;
  confirmed_logic_ids: string[];
  candidate_logic_used: boolean;
  status: "waiting_for_portfolio" | "incomplete_confirmed_parameters" | string;
  rules: Array<{
    logic_id: string;
    logic_version: string;
    label: string;
    status: string;
    evidence: string[];
    gaps: string[];
  }>;
  blocking_reasons: string[];
  increase_candidate_authorized: boolean;
  portfolio_action_authorized: boolean;
  shadow_portfolio_authorized: boolean;
  trade_authorized: boolean;
  scope: string;
};

export type PositionAdviceItem = {
  symbol: string;
  name: string;
  theme: string;
  weight: number;
  current_price?: number | null;
  avg_cost?: number | null;
  unrealized_return_percent?: number | null;
  rating_score?: number | null;
  rating_light: "green" | "yellow" | "red" | "unknown" | string;
  rating_status: string;
  valuation_position: string;
  news_impact: string;
  news_attention: string;
  action: PositionManagementAction;
  action_label: string;
  confidence: "high" | "medium" | "low" | string;
  rationale: string[];
  risks: string[];
  falsifiers: string[];
  framework_logic: string[];
  evidence_as_of: string[];
  evidence_sources: string[];
  priority_score: number;
  decision_gate?: PositionDecisionGate;
};

export type PositionManagementSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: "Asia/Shanghai" | string;
  model_version: string;
  framework_version: string;
  status: string;
  portfolio_updated_at: string;
  holdings_count: number;
  total_weight: number;
  unallocated_weight: number;
  concentration: {
    level: "balanced" | "elevated" | "high" | "unknown" | string;
    largest_symbol: string;
    largest_weight: number;
    top_three_weight: number;
    theme_exposures: Array<{ theme: string; weight: number }>;
  };
  macro_context: {
    signal: string;
    score?: number | null;
    phase: string;
    report_date: string;
    status: string;
  };
  portfolio_gate?: PositionPortfolioGate;
  counts: Record<PositionManagementAction, number>;
  summary: string;
  items: PositionAdviceItem[];
  methodology_note: string;
  disclaimer: string;
};

export type InfluencerDigestSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: string;
  lookback_hours: number;
  model_version: string;
  status: string;
  analysis_health: ModelAnalysisHealth;
  summary: string;
  coverage: {
    authors: number;
    configured: number;
    succeeded: number;
    items: number;
    analyzed: number;
  };
  authors: Array<{
    id: string;
    name: string;
    public_handle: string;
    focus: string;
    configured: boolean;
    source_status: string;
    item_count: number;
    last_published_at?: string | null;
  }>;
  items: Array<{
    id: string;
    author_id: string;
    author_name: string;
    public_handle: string;
    title: string;
    published_at: string;
    published_at_beijing: string;
    source_url: string;
    aggregation_source?: string | null;
    aggregation_url?: string | null;
    post_kind: string;
    source_excerpt: string;
    summary: string;
    stance: string;
    horizon: string;
    content_type: string;
    topics: string[];
    tickers: string[];
    counterpoint: string;
    analysis_status: string;
  }>;
  disclaimer: string;
};

export type KeyEventChainSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  timezone: string;
  lookback_days: number;
  model_version: string;
  status: string;
  analysis_health: ModelAnalysisHealth;
  summary: string;
  topics: Array<{
    id: string;
    name: string;
    layer: string;
    description: string;
    first_principle: string;
    priority: number;
    status: string;
    event_count: number;
    source_count?: number;
    deduplicated_source_count?: number;
    confirmed_count: number;
    clue_count: number;
    last_event_at?: string | null;
    latest_change: string;
    events: Array<{
      id: string;
      topic_id: string;
      event_identity_version?: string;
      event_fingerprint_sha256?: string;
      source_count?: number;
      supporting_sources?: Array<{
        source_id: string;
        source_name: string;
        source_url: string;
        published_at: string;
        published_at_beijing: string;
        source_tier: string;
        verification_status: string;
      }>;
      deduplication_status?: string;
      deduplication_note?: string;
      published_at: string;
      published_at_beijing: string;
      source_name: string;
      source_url: string;
      source_tier: string;
      verification_status: string;
      verification_note: string;
      title: string;
      excerpt: string;
      change_type: string;
      direction: string;
      impact: string;
      next_watch: string;
      tickers: string[];
      analysis_status: string;
    }>;
  }>;
  /** Legacy storage-only payload; no longer exposed by the public dashboard API. */
  ten_day_brief?: {
    review_start: string;
    review_end: string;
    outlook_start: string;
    outlook_end: string;
    previous_generated_at_beijing?: string | null;
    status: string;
    summary: string;
    version_summary: string;
    review: Array<{
      topic_id: string;
      topic_name: string;
      event_count: number;
      confirmed_count: number;
      clue_count: number;
      new_since_previous: number;
      direction_summary: string;
      latest_change: string;
      evidence_event_ids: string[];
    }>;
    questions: Array<{
      id: string;
      topic_id: string;
      topic_name: string;
      question: string;
      why_it_matters: string;
      status: string;
      review_by: string;
      evidence_event_ids: string[];
    }>;
    methodology_note: string;
  };
  disclaimer: string;
};

export type WeeklyBriefItem = {
  id: string;
  date: string;
  weekday: string;
  phase: "last_week" | "next_week" | "ai_outlook" | string;
  category:
    | "policy"
    | "inflation"
    | "labor"
    | "growth"
    | "macro"
    | "earnings"
    | "industry"
    | "ai_conference"
    | string;
  importance: "high" | "medium" | string;
  title: string;
  subtitle: string;
  ticker?: string | null;
  source_name: string;
  source_url?: string | null;
  source_count?: number;
  deduplicated_source_count?: number;
  supporting_sources?: Array<{
    source_id: string;
    source_name: string;
    source_url: string;
    published_at: string;
    published_at_beijing: string;
    source_tier: string;
    verification_status: string;
  }>;
  evidence_status:
    | "confirmed"
    | "schedule_passed"
    | "scheduled"
    | "official_schedule"
    | string;
  evidence_note: string;
  analysis_status: "model_analyzed" | "source_only" | "scheduled_context" | "editorial_framework" | string;
  analysis: string;
  attention: string;
};

export type WeeklyBriefPayload = {
  report_date: string;
  generated_at_beijing: string;
  timezone: string;
  status: "live" | "partial" | "empty" | string;
  industry_analysis_health: ModelAnalysisHealth;
  summary: string;
  previous_week: { start: string; end: string; label: string };
  next_week: { start: string; end: string; label: string };
  ai_outlook: { start: string; end: string; label: string };
  last_week_items: WeeklyBriefItem[];
  next_week_items: WeeklyBriefItem[];
  ai_outlook_items: WeeklyBriefItem[];
  earnings_status: string;
  earnings_scope_count: number;
  holdings: string[];
  errors: string[];
  methodology_note: string;
  disclaimer: string;
};

// ── Unified research library ──────────────────────────────────────────────

export type ResearchLibraryUse = "chat" | "key_event_chain" | "portfolio_news";
export type ResearchLibraryScope =
  "personal" | "community_candidate" | "hone_global";
export type ResearchLibraryReviewStatus =
  "not_required" | "pending" | "approved" | "rejected";
export type ResearchLibrarySourceType =
  "manual_upload" | "zsxq_export" | "ima_export" | "authorized_connector";

export type ResearchLibraryItem = {
  id: string;
  scope: ResearchLibraryScope;
  submitted_by?: string | null;
  title: string;
  filename: string;
  content_type: string;
  size: number;
  sha256: string;
  source_type: ResearchLibrarySourceType;
  source_name: string;
  source_url?: string | null;
  source_date: string;
  uploaded_at: string;
  updated_at: string;
  parse_status: "ready" | "stored" | "error" | string;
  excerpt: string;
  tickers: string[];
  topics: string[];
  uses: ResearchLibraryUse[];
  review_status: ResearchLibraryReviewStatus;
  review_note?: string | null;
  reviewed_at?: string | null;
  download_url: string;
};

export type ResearchConnectorStatus = {
  status: "available_via_import" | string;
  mode: "official_skill_export" | "file_export" | string;
  read_only: boolean;
  automatic_sync: boolean;
  guide_url?: string;
  note: string;
};

export type ResearchLibraryBundle = {
  items: ResearchLibraryItem[];
  is_admin: boolean;
  connector_status: {
    zsxq: ResearchConnectorStatus;
    ima: ResearchConnectorStatus;
  };
};

// ── Daily macro / AI signals ──────────────────────────────────────────────

export type DailySignalKind = "macro" | "ai";
export type DailySignalLight =
  "green" | "yellow" | "orange" | "red" | "unknown";
export type DailySignalStatus =
  "live" | "partial" | "framework_only" | "stale" | string;

export type DailySignalTrendPoint = { period: string; value: number };
export type DailySignalEvidence = {
  label: string;
  value?: number | null;
  display_value: string;
  unit: string;
  period?: string | null;
  released_at?: string | null;
  fetched_at: string;
  source: string;
  source_url: string;
  provenance: "reported_fact" | "model_inference" | "unavailable" | string;
};

export type DailySignalDimension = {
  id: string;
  label: string;
  role: string;
  score?: number | null;
  signal: DailySignalLight;
  trend_label: string;
  reason: string;
  threshold: string;
  trend: DailySignalTrendPoint[];
  evidence: DailySignalEvidence[];
};

export type DailySignalMetric = {
  id: string;
  label: string;
  score?: number | null;
  display_value: string;
  reason: string;
};

export type DailySignalCompanyScore = {
  symbol: string;
  name: string;
  score?: number | null;
  signal: DailySignalLight;
  capex?: number | null;
  capex_growth?: number | null;
  capex_peak_status: string;
  coverage: number;
  metric_total: number;
  metrics: DailySignalMetric[];
};

export type DailyHardwareSignal = {
  symbol: string;
  segment: string;
  signal: DailySignalLight;
  score?: number | null;
  price?: number | null;
  change_percent?: number | null;
  reason: string;
};

export type DailySignalReport = {
  kind: DailySignalKind;
  title: string;
  report_date: string;
  market_date?: string | null;
  data_cutoff?: string | null;
  generated_at: string;
  generated_at_beijing: string;
  next_refresh_at: string;
  model_version: string;
  status: DailySignalStatus;
  score?: number | null;
  raw_score?: number | null;
  signal: DailySignalLight;
  phase: string;
  summary: string;
  comparison_yesterday?: number | null;
  comparison_week?: number | null;
  changes: { label: string; direction: string; detail: string }[];
  dimensions: DailySignalDimension[];
  company_scores: DailySignalCompanyScore[];
  hardware_signals: DailyHardwareSignal[];
  alerts: string[];
  evidence: DailySignalEvidence[];
  sources: { label: string; url: string; source_type: string }[];
  full_report: string;
  stale: boolean;
  disclaimer: string;
};

export type DailySignalHistoryItem = Pick<
  DailySignalReport,
  | "report_date"
  | "generated_at_beijing"
  | "status"
  | "score"
  | "raw_score"
  | "signal"
  | "phase"
  | "summary"
>;

export type ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_claim_first_observation_materialization_attempt"
  | "changes_requested_rebuild_artifact"
  | "rejected";

export type ControlledShadowObservationMaterializationReproducedArtifactManifest = {
  schema_version: string;
  manifest_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_contract_sha256: string;
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  artifact_byte_length: number;
  artifact_file_name: string;
  artifact_media_type: string;
  source_bundle_sha256: string;
  artifact_reproduction_procedure_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  reproduced_at: string;
  reproduced_by: string;
  source_and_artifact_reproduced_from_immutable_revision: boolean;
  artifact_is_read_only_regular_file: boolean;
  artifact_was_not_executed: boolean;
  stage_104_admitted_input_was_not_read: boolean;
};

export type ControlledShadowObservationMaterializationArtifactInspection = {
  custody_locator: string;
  manifest_present: boolean;
  artifact_present: boolean;
  manifest: ControlledShadowObservationMaterializationReproducedArtifactManifest | null;
  server_computed_artifact_sha256: string | null;
  server_observed_artifact_byte_length: number | null;
  artifact_verified: boolean;
  status: string;
};

export type ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_id: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_spec_revision: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_materialization_specification_sha256: string;
  expected_stage_104_admission_review_sha256: string;
  expected_stage_103_validation_sha256: string;
  expected_stage_102_result_sha256: string;
  expected_stage_102_output_sha256: string;
  expected_stage_101_claim_sha256: string;
  expected_stage_101_input_manifest_sha256: string;
  expected_cycle_claim_sha256: string;
  expected_artifact_manifest_sha256: string;
  artifact_reproduction_review_evidence: string;
  sandbox_contract_review_evidence: string;
  verdict: ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_109_binding_confirmed: boolean;
  reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed: boolean;
  server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: boolean;
  self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: boolean;
  artifact_builder_and_reviewer_separation_confirmed: boolean;
  all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed: boolean;
  session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: boolean;
  no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  provider_publication_time_remains_unverified_until_separate_evidence_confirmed: boolean;
  authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed: boolean;
  no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_111_claim_first_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord;
  artifact_manifest: ControlledShadowObservationMaterializationReproducedArtifactManifest;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  server_computed_artifact_sha256: string;
  server_observed_artifact_byte_length: number;
  verdict: ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_execution_attempt_limit: number;
  one_future_claim_first_observation_materialization_attempt_authorized: boolean;
  authorization_claimed: boolean;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_materialization_executed: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  observation_materialized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord;
    artifact_inspection: ControlledShadowObservationMaterializationArtifactInspection;
    latest_review: ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview | null;
    authorization_unexpired: boolean;
    future_claim_eligible: boolean;
  }>;
  runner_count: number;
  artifact_verified_runner_count: number;
  artifact_pending_runner_count: number;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  future_claim_eligible_count: number;
  authorization_status: string;
  next_gate: string;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_materialization_executed: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  observation_materialized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimControlledShadowObservationMaterializationExecutionAttemptRequest = {
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_sha256: string;
  expected_observation_materialization_specification_sha256: string;
  expected_stage_104_admission_review_sha256: string;
  expected_stage_103_validation_sha256: string;
  expected_stage_102_result_sha256: string;
  expected_stage_102_output_sha256: string;
  expected_stage_101_claim_sha256: string;
  expected_stage_101_input_manifest_sha256: string;
  expected_cycle_claim_sha256: string;
  claim_reason: string;
  exact_current_stage_51_through_stage_110_binding_confirmed: boolean;
  claimant_independent_from_stage_110_and_complete_prior_chain_confirmed: boolean;
  authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: boolean;
  current_server_rehashed_artifact_and_manifest_binding_confirmed: boolean;
  exact_stage_104_admitted_input_remains_content_addressed_read_only_and_unread_confirmed: boolean;
  claim_contains_only_existing_metadata_and_hashes_confirmed: boolean;
  no_entrypoint_runtime_input_mount_input_read_or_observation_materialization_execution_confirmed: boolean;
  future_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: boolean;
  no_retry_release_or_authorization_restoration_after_claim_confirmed: boolean;
  no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization: ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview;
  claimed_at: string;
  claimed_by: string;
  claim_reason: string;
  authorization_consumed: boolean;
  create_once: boolean;
  claim_first: boolean;
  retry_allowed: boolean;
  release_allowed: boolean;
  authorization_restoration_allowed: boolean;
  task_status: string;
  execution_attempt_endpoint_available: boolean;
  input_read: boolean;
  observation_materialization_executed: boolean;
  observation_materialized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry = {
  schema_version: string;
  policy_version: string;
  claim_endpoint_available: boolean;
  eligible_authorizations: Array<{
    authorization: ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview;
    claimant_excluded_actor_ids: string[];
  }>;
  claims: ControlledShadowObservationMaterializationExecutionAttemptClaim[];
  authorization_candidate_count: number;
  claim_eligible_count: number;
  claim_count: number;
  authorization_consumed_count: number;
  waiting_for_stage_112_execution_count: number;
  claim_status: string;
  next_gate: string;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_materialization_executed: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  observation_materialized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ExecuteControlledShadowObservationMaterializationAttemptRequest = {
  expected_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_observation_materialization_specification_sha256: string;
  expected_stage_104_admission_review_sha256: string;
  expected_stage_102_output_sha256: string;
  expected_stage_101_input_manifest_sha256: string;
  expected_cycle_claim_sha256: string;
  execution_reason: string;
  exact_stage_51_through_stage_111_binding_confirmed: boolean;
  executor_independent_from_complete_prior_chain_and_claimant_confirmed: boolean;
  start_marker_consumes_claim_before_artifact_or_input_read_confirmed: boolean;
  one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: boolean;
  artifact_is_declarative_not_spawned_or_executed_confirmed: boolean;
  only_exact_stage_104_admitted_output_is_read_only_opened_and_rehashed_confirmed: boolean;
  deterministic_session_price_gap_action_allocation_and_availability_projection_confirmed: boolean;
  no_refetch_reparse_fill_interpolation_substitution_backfill_or_correction_confirmed: boolean;
  output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: boolean;
  no_network_environment_secret_tool_subprocess_or_production_io_confirmed: boolean;
  no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  stage_111_attempt_id: string;
  stage_111_claim_sha256: string;
  completed_at: string;
  executed_by: string;
  execution_reason: string;
  duration_millis: number;
  status: "completed_with_untrusted_observation_envelope" | "failed_claim_consumed";
  bounded_error_code: string | null;
  output_sha256: string | null;
  output_relative_path: string | null;
  claim_consumed: boolean;
  artifact_revalidated: boolean;
  artifact_spawned_or_executed: boolean;
  exact_admitted_input_revalidated_and_opened: boolean;
  materializer_executed_in_process: boolean;
  sessions_materialized: boolean;
  price_observations_materialized: boolean;
  explicit_gaps_materialized: boolean;
  corporate_actions_materialized: boolean;
  observation_envelope_created: boolean;
  output_untrusted: boolean;
  independent_validation_completed: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_or_metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationExecutionAttemptRegistry = {
  schema_version: string;
  policy_version: string;
  execution_endpoint_available: boolean;
  pending_claims: ControlledShadowObservationMaterializationExecutionAttemptClaim[];
  results: ControlledShadowObservationMaterializationExecutionAttemptResult[];
  pending_claim_count: number;
  terminal_result_count: number;
  successful_untrusted_observation_count: number;
  failed_consumed_claim_count: number;
  next_gate: string;
  arbitrary_artifact_execution_allowed: boolean;
  outbound_network_allowed: boolean;
  independent_validation_completed: boolean;
  observation_envelope_created: boolean;
  forward_observation_started: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateControlledShadowObservationMaterializationOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_output_sha256: string;
  expected_specification_sha256: string;
  expected_stage_104_review_sha256: string;
  expected_stage_102_output_sha256: string;
  validation_reason: string;
  exact_current_stage_51_through_stage_112_binding_confirmed: boolean;
  validator_independent_from_executor_and_complete_prior_chain_confirmed: boolean;
  stage_112_result_and_create_once_output_reopened_and_rehashed_confirmed: boolean;
  exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed: boolean;
  second_projection_does_not_call_stage_112_materializer_helpers_confirmed: boolean;
  sessions_prices_gaps_actions_allocation_availability_independently_recomputed_confirmed: boolean;
  every_row_hash_sort_order_and_complete_envelope_exactly_compared_confirmed: boolean;
  pass_only_opens_future_stage_114_observation_evidence_admission_review_confirmed: boolean;
  no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationMaterializationOutputValidation = {
  validation_id: string;
  validation_sha256: string;
  stage_111_attempt_id: string;
  stage_111_claim_sha256: string;
  stage_112_result_id: string;
  stage_112_result_sha256: string;
  stage_112_output_sha256: string;
  observation_materialization_specification_sha256: string;
  stage_104_review_sha256: string;
  stage_102_output_sha256: string;
  validated_at: string;
  validated_by: string;
  validation_reason: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  mismatch_reasons: string[];
  verdict: "independently_validated_exact_observation_envelope" | "failed_independent_observation_envelope_validation";
  observation_envelope_independently_validated: boolean;
  future_stage_114_observation_evidence_admission_review_eligible: boolean;
  observed_output_bytes: number;
  observed_session_count: number;
  observed_price_count: number;
  observed_gap_count: number;
  observed_dividend_count: number;
  observed_split_count: number;
  ledger_created: boolean;
  position_written: boolean;
  performance_metric_written: boolean;
  model_or_metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationMaterializationOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    claim: ControlledShadowObservationMaterializationExecutionAttemptClaim;
    result: ControlledShadowObservationMaterializationExecutionAttemptResult;
    validation: ControlledShadowObservationMaterializationOutputValidation | null;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_observation_count: number;
  failed_validation_count: number;
  future_stage_114_observation_evidence_admission_review_eligible_count: number;
  validation_status: string;
  next_gate: string;
  independent_output_validation_available: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReviewControlledShadowObservationEvidenceAdmissionRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_stage_113_validation_id: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_112_output_sha256: string;
  expected_stage_111_claim_sha256: string;
  verdict: "admitted_for_future_observation_ledger_transition_specification_registration" | "changes_requested" | "rejected";
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_113_binding_confirmed: boolean;
  reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: boolean;
  stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: boolean;
  stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: boolean;
  exact_stage_104_admitted_input_binding_preserved_confirmed: boolean;
  sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: boolean;
  natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed: boolean;
  provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: boolean;
  admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed: boolean;
  approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed: boolean;
  no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationEvidenceAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id: string | null;
  previous_review_sha256: string | null;
  stage_111_attempt_id: string;
  stage_111_claim_sha256: string;
  stage_112_result_id: string;
  stage_112_result_sha256: string;
  stage_112_output_sha256: string;
  stage_113_validation_id: string;
  stage_113_validation_sha256: string;
  admitted_available_at_utc: string;
  stage_113_validated_at: string;
  submitted_at: string;
  submitted_by: string;
  verdict: "admitted_for_future_observation_ledger_transition_specification_registration" | "changes_requested" | "rejected";
  rationale: string;
  known_limitations: string;
  observed_session_count: number;
  observed_price_count: number;
  observed_gap_count: number;
  observed_dividend_count: number;
  observed_split_count: number;
  provider_publication_time_verified: boolean;
  original_envelope_remains_untrusted_and_immutable: boolean;
  observation_evidence_admitted: boolean;
  future_observation_ledger_transition_specification_registration_eligible: boolean;
  ledger_created: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationEvidenceAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: {
      claim: ControlledShadowObservationMaterializationExecutionAttemptClaim;
      result: ControlledShadowObservationMaterializationExecutionAttemptResult;
      validation: ControlledShadowObservationMaterializationOutputValidation;
    };
    latest_review: ControlledShadowObservationEvidenceAdmissionReview | null;
    current_binding: boolean;
    review_eligible: boolean;
    observation_evidence_admitted: boolean;
  }>;
  independently_validated_candidate_count: number;
  review_eligible_candidate_count: number;
  reviewed_candidate_count: number;
  admitted_observation_evidence_count: number;
  changes_requested_or_rejected_count: number;
  future_observation_ledger_transition_specification_registration_eligible_count: number;
  admission_status: string;
  next_gate: string;
  admission_review_available: boolean;
  provider_publication_time_verified: boolean;
  original_envelope_mutated: boolean;
  ledger_created: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationLedgerTransitionSpecificationRequest = {
  expected_stage_114_review_sha256: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_112_output_sha256: string;
  expected_stage_111_claim_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_114_binding_confirmed: boolean;
  registrar_independent_from_stage_114_and_complete_prior_chain_confirmed: boolean;
  stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed: boolean;
  stage_88_binding_not_treated_as_opening_positions_confirmed: boolean;
  separately_admitted_opening_portfolio_snapshot_required_confirmed: boolean;
  no_default_notional_cash_positions_or_share_quantities_confirmed: boolean;
  raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed: boolean;
  explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed: boolean;
  dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed: boolean;
  exact_decimal_append_only_idempotent_and_available_at_rules_confirmed: boolean;
  corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed: boolean;
  specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: boolean;
  no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  future_chain_external_specification_review_required_before_implementation_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionSpecificationRegistration = {
  registration_id: string;
  registration_sha256: string;
  registered_at: string;
  registered_by: string;
  stage_114_review_id: string;
  stage_114_review_sha256: string;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  status: string;
  specification_registered: boolean;
  future_chain_external_specification_review_eligible: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  specification: {
    specification_sha256: string;
    transition_protocol_version: string;
    stage_114_review_sha256: string;
    stage_113_validation_sha256: string;
    stage_112_result_sha256: string;
    stage_112_output_sha256: string;
    stage_111_claim_sha256: string;
    admitted_available_at_utc: string;
    provider_publication_time_verified: boolean;
    stage_88_initialization_output_sha256: string;
    stage_88_initialization_manifest_sha256: string;
    subject_symbols: string[];
    benchmark_symbol: string;
    earliest_market_session_date: string;
    latest_market_session_date: string;
    observed_session_count: number;
    observed_price_count: number;
    observed_gap_count: number;
    observed_dividend_count: number;
    observed_split_count: number;
    opening_portfolio_prerequisite: {
      separately_admitted_opening_portfolio_snapshot_required: boolean;
      current_opening_portfolio_snapshot_available: boolean;
      stage_88_binding_is_initialization_provenance_not_opening_positions: boolean;
      default_notional_allowed: boolean;
      default_cash_allowed: boolean;
      infer_positions_from_subject_symbols_allowed: boolean;
      infer_share_quantities_from_prices_or_target_weights_allowed: boolean;
      financial_posting_before_opening_snapshot_admission_allowed: boolean;
      missing_opening_snapshot_result: string;
    };
    mapping_rules: {
      non_financial_event_type_allowlist: string[];
      financial_event_type_allowlist_before_opening_snapshot: string[];
      security_valuation_price_basis: string;
      benchmark_total_return_price_basis: string;
      split_adjusted_price_usage: string;
      dividend_adjusted_price_usage: string;
      explicit_gap_rule: string;
      dividend_rule: string;
      split_rule: string;
      correction_rule: string;
      decimal_rule: string;
      nav_completeness_rule: string;
    };
    financial_postings_currently_eligible: boolean;
    nav_or_performance_currently_eligible: boolean;
  };
};

export type ControlledShadowObservationLedgerTransitionSpecificationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  candidates: Array<{
    stage_114_review_id: string;
    stage_114_review_sha256: string;
    stage_113_validation_sha256: string;
    stage_112_result_sha256: string;
    stage_112_output_sha256: string;
    stage_111_claim_sha256: string;
    admitted_available_at_utc: string;
    subject_symbols: string[];
    observed_session_count: number;
    observed_price_count: number;
    observed_gap_count: number;
    registrar_excluded_actor_ids: string[];
  }>;
  registrations: ControlledShadowObservationLedgerTransitionSpecificationRegistration[];
  admitted_observation_evidence_count: number;
  registration_eligible_count: number;
  registered_specification_count: number;
  future_stage_116_independent_review_eligible_count: number;
  opening_portfolio_snapshot_missing_count: number;
  registration_status: string;
  next_gate: string;
  implementation_present: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict =
  | "approved_for_future_zero_capability_ledger_transition_implementation_registration"
  | "changes_required_rebuild_ledger_transition_specification"
  | "rejected_ledger_transition_specification";

export type ReviewControlledShadowObservationLedgerTransitionSpecificationRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_registration_sha256: string;
  expected_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict;
  rationale: string;
  binding_and_second_implementation_assessment: string;
  opening_portfolio_prerequisite_assessment: string;
  price_basis_gap_and_nav_assessment: string;
  corporate_action_and_double_count_assessment: string;
  decimal_idempotency_correction_and_order_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  exact_current_stage_51_through_stage_115_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed: boolean;
  rebuilt_specification_exactly_matches_registered_specification_confirmed: boolean;
  stage_88_binding_not_opening_positions_confirmed: boolean;
  separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: boolean;
  raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: boolean;
  explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: boolean;
  dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: boolean;
  exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: boolean;
  corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed: boolean;
  conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: boolean;
  no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed: boolean;
  no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_zero_capability_implementation_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  registration_id: string;
  registration_sha256: string;
  specification_sha256: string;
  registration_hash_independently_reproduced: boolean;
  specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_115_binding_valid: boolean;
  complete_specification_rebuilt_without_stage_115_builder: boolean;
  rebuilt_specification_exactly_matches_registration: boolean;
  opening_portfolio_prerequisite_and_no_invention_contract_valid: boolean;
  raw_price_adjusted_price_gap_and_nav_contract_valid: boolean;
  corporate_action_no_double_count_contract_valid: boolean;
  decimal_idempotency_append_only_correction_and_double_entry_contract_valid: boolean;
  availability_and_provider_time_contract_valid: boolean;
  all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ControlledShadowObservationLedgerTransitionSpecificationReview = {
  review_id: string;
  review_sha256: string;
  submitted_at: string;
  verdict: ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict;
  rationale: string;
  known_limitations: string;
  specification_independently_approved: boolean;
  future_zero_capability_implementation_registration_eligible: boolean;
};

export type ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_endpoint_available: boolean;
  items: Array<{
    registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration;
    current_independent_audit: ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review: ControlledShadowObservationLedgerTransitionSpecificationReview | null;
    review_eligible: boolean;
    future_zero_capability_implementation_registration_eligible: boolean;
  }>;
  specification_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_zero_capability_implementation_registration_eligible_count: number;
  opening_portfolio_snapshot_missing_count: number;
  review_status: string;
  implementation_registered: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationLedgerTransitionImplementationRequest = {
  expected_specification_review_id: string;
  expected_specification_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_registration_id: string;
  expected_registration_sha256: string;
  expected_specification_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_projection_semantics: string;
  session_price_basis_and_gap_semantics: string;
  corporate_action_decimal_order_and_hash_semantics: string;
  initial_allocation_and_availability_semantics: string;
  error_and_missing_data_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_stage_51_through_stage_116_binding_confirmed: boolean;
  registrar_independent_from_stage_116_and_complete_prior_chain_confirmed: boolean;
  independent_recomputation_of_review_registration_specification_and_audit_confirmed: boolean;
  zero_capability_contract_only_no_source_or_executable_artifact_confirmed: boolean;
  exact_stage_114_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_and_price_bases_remain_separate_confirmed: boolean;
  decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: boolean;
  initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: boolean;
  conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionSpecificationReviewForImplementation =
  ControlledShadowObservationLedgerTransitionSpecificationReview & {
    registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration;
    independent_audit: ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit;
    reviewer_id: string;
    excluded_prior_actor_ids: string[];
  };

export type ControlledShadowObservationLedgerTransitionImplementationRecord = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_specification_registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration;
  upstream_specification_review: ControlledShadowObservationLedgerTransitionSpecificationReviewForImplementation;
  implementation_name: string;
  implementation_description: string;
  implementation_contract: {
    schema_version: string;
    contract_sha256: string;
    implementation_protocol_version: string;
    immutable_code_revision: string;
    exact_observation_ledger_transition_specification: ControlledShadowObservationLedgerTransitionSpecificationRegistration["specification"];
    current_source_binding_validation_function_id: string;
    opening_portfolio_prerequisite_validation_function_id: string;
    non_financial_observation_event_projection_function_id: string;
    raw_close_accounting_and_adjusted_price_separation_function_id: string;
    explicit_gap_nav_fail_closed_function_id: string;
    corporate_action_notice_gating_function_id: string;
    exact_decimal_idempotency_and_double_entry_function_id: string;
    append_only_correction_and_conservative_availability_function_id: string;
    opening_portfolio_snapshot_currently_admitted: boolean;
    financial_postings_currently_eligible: boolean;
    nav_or_performance_currently_eligible: boolean;
    registered_not_run: boolean;
    independent_implementation_review_required: boolean;
  };
  status: string;
  confirmations_complete: boolean;
  zero_capability_implementation_contract_registered: boolean;
  observation_ledger_transition_implementation_present: boolean;
  future_independent_implementation_review_eligible: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  items: Array<{
    specification_review: ControlledShadowObservationLedgerTransitionSpecificationReviewForImplementation;
    specification_registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration;
    implementation: ControlledShadowObservationLedgerTransitionImplementationRecord | null;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_independent_implementation_review_eligible: boolean;
  }>;
  independently_approved_specification_count: number;
  registration_eligible_count: number;
  implementation_contract_count: number;
  current_binding_implementation_contract_count: number;
  independent_implementation_review_eligible_count: number;
  opening_portfolio_snapshot_missing_count: number;
  implementation_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationLedgerTransitionImplementationReviewVerdict =
  | "approved_for_future_isolated_observation_ledger_transition_runner_specification_registration"
  | "changes_required_rebuild_observation_ledger_transition_implementation"
  | "rejected_observation_ledger_transition_implementation";

export type ReviewControlledShadowObservationLedgerTransitionImplementationRequest = {
  expected_previous_review_id: string | null;
  expected_previous_review_sha256: string | null;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_independent_audit_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_ledger_transition_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: ControlledShadowObservationLedgerTransitionImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_projection_semantics_assessment: string;
  session_price_basis_gap_and_company_action_assessment: string;
  initial_allocation_availability_and_output_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  exact_current_stage_51_through_stage_117_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: boolean;
  exact_stage_114_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: boolean;
  initial_shadow_allocation_and_conservative_availability_preserved_confirmed: boolean;
  provider_publication_time_remains_unverified_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  specification_review_sha256: string;
  specification_independent_audit_sha256: string;
  specification_registration_sha256: string;
  observation_ledger_transition_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  specification_review_hash_independently_reproduced: boolean;
  specification_independent_audit_hash_independently_reproduced: boolean;
  specification_registration_hash_independently_reproduced: boolean;
  observation_ledger_transition_specification_hash_independently_reproduced: boolean;
  complete_implementation_contract_rebuilt_without_stage_117_builder: boolean;
  rebuilt_implementation_contract_exactly_matches_record: boolean;
  exact_current_stage_51_through_stage_117_binding_valid: boolean;
  eight_function_ids_and_canonical_schemas_valid: boolean;
  opening_portfolio_prerequisite_and_no_invention_contract_valid: boolean;
  raw_price_adjusted_price_gap_and_nav_contract_valid: boolean;
  corporate_action_decimal_idempotency_double_entry_and_correction_contract_valid: boolean;
  conservative_availability_create_once_and_output_path_contract_valid: boolean;
  provider_publication_time_still_unverified: boolean;
  all_source_artifact_runtime_input_ledger_financial_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type ControlledShadowObservationLedgerTransitionImplementationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id: string | null;
  previous_review_sha256: string | null;
  implementation: ControlledShadowObservationLedgerTransitionImplementationRecord;
  independent_audit: ControlledShadowObservationLedgerTransitionImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: ControlledShadowObservationLedgerTransitionImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  deterministic_projection_semantics_assessment: string;
  session_price_basis_gap_and_company_action_assessment: string;
  initial_allocation_availability_and_output_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_runner_constraints: string;
  reviewer_independent_from_registrar_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_117_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: boolean;
  exact_stage_114_admitted_output_is_only_future_input_confirmed: boolean;
  official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: boolean;
  explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: boolean;
  dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: boolean;
  initial_shadow_allocation_and_conservative_availability_preserved_confirmed: boolean;
  provider_publication_time_remains_unverified_confirmed: boolean;
  one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: boolean;
  future_output_untrusted_and_independent_validation_required_confirmed: boolean;
  no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
  zero_capability_implementation_independently_approved: boolean;
  future_isolated_observation_ledger_transition_runner_specification_registration_eligible: boolean;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: ControlledShadowObservationLedgerTransitionImplementationRecord;
    current_independent_audit: ControlledShadowObservationLedgerTransitionImplementationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review: ControlledShadowObservationLedgerTransitionImplementationReviewRecord | null;
    review_eligible: boolean;
    future_isolated_observation_ledger_transition_runner_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count: number;
  review_status: string;
  isolated_runner_registered: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_present: boolean;
  input_mounted_or_read: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterControlledShadowObservationLedgerTransitionIsolatedRunnerRequest = {
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_ledger_transition_specification_sha256: string;
  expected_stage_114_admission_review_sha256: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_111_claim_sha256: string;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_observation_ledger_transition_specification";
  runner_spec_revision: string;
  proposed_runner_code_revision: string;
  proposed_runner_artifact_sha256: string;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_118_binding_confirmed: boolean;
  registrar_independent_from_stage_118_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: boolean;
  all_eight_observation_ledger_transition_functions_and_canonical_schemas_preserved_confirmed: boolean;
  future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: boolean;
  session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: boolean;
  no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_preserved_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: boolean;
  future_financial_events_require_separately_admitted_opening_snapshot_confirmed: boolean;
  provider_publication_time_remains_unverified_until_separate_evidence_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionIsolatedRunnerRecord = {
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: ControlledShadowObservationLedgerTransitionImplementationRecord;
  implementation_review: ControlledShadowObservationLedgerTransitionImplementationReviewRecord;
  runner_name: string;
  runner_kind: "ephemeral_deterministic_observation_ledger_transition_specification";
  runner_contract: {
    contract_sha256: string;
    runner_spec_revision: string;
    proposed_runner_code_revision: string;
    proposed_runner_artifact_sha256: string;
    runtime_identity: string;
    runtime_version: string;
    future_input_envelope: string;
    future_output_envelope: string;
    next_gate: string;
    future_runner_artifact_identity_bound: boolean;
    source_artifact_present: boolean;
    executable_artifact_present: boolean;
    callable_entrypoint_present: boolean;
    runtime_instantiated: boolean;
    input_mount_present: boolean;
    input_read_allowed: boolean;
    opening_portfolio_snapshot_present: boolean;
    financial_event_allowlist: string[];
    financial_event_allowlist_empty_without_opening_snapshot_required: boolean;
    maximum_parallel_runs: number;
    maximum_memory_mib: number;
    maximum_wall_clock_seconds: number;
    maximum_cpu_millicores: number;
    maximum_process_count: number;
    maximum_output_bytes: number;
  };
  status: string;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  input_accessed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: ControlledShadowObservationLedgerTransitionImplementationRecord;
    review: ControlledShadowObservationLedgerTransitionImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  runner_count: number;
  current_binding_runner_count: number;
  first_execution_authorization_review_eligible_count: number;
  items: Array<{
    runner: ControlledShadowObservationLedgerTransitionIsolatedRunnerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  runner_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_accessed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_claim_first_observation_ledger_transition_attempt"
  | "changes_requested_rebuild_artifact"
  | "rejected";

export type ControlledShadowObservationLedgerTransitionReproducedArtifactManifest = {
  schema_version: string;
  manifest_sha256: string;
  isolated_runner_id: string;
  isolated_runner_spec_sha256: string;
  runner_contract_sha256: string;
  runner_spec_revision: string;
  runner_code_revision: string;
  runner_artifact_sha256: string;
  artifact_byte_length: number;
  artifact_file_name: string;
  artifact_media_type: string;
  source_bundle_sha256: string;
  artifact_reproduction_procedure_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  reproduced_at: string;
  reproduced_by: string;
  source_and_artifact_reproduced_from_immutable_revision: boolean;
  artifact_is_read_only_regular_file: boolean;
  artifact_was_not_executed: boolean;
  stage_114_admitted_input_was_not_read: boolean;
};

export type ControlledShadowObservationLedgerTransitionArtifactInspection = {
  custody_locator: string;
  manifest_present: boolean;
  artifact_present: boolean;
  manifest: ControlledShadowObservationLedgerTransitionReproducedArtifactManifest | null;
  server_computed_artifact_sha256: string | null;
  server_observed_artifact_byte_length: number | null;
  artifact_verified: boolean;
  status: string;
};

export type ReviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_runner_id: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_spec_revision: string;
  expected_runner_code_revision: string;
  expected_runner_artifact_sha256: string;
  expected_implementation_id: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_id: string;
  expected_implementation_review_sha256: string;
  expected_independent_audit_sha256: string;
  expected_specification_review_sha256: string;
  expected_specification_registration_sha256: string;
  expected_observation_ledger_transition_specification_sha256: string;
  expected_stage_114_admission_review_sha256: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_111_claim_sha256: string;
  expected_artifact_manifest_sha256: string;
  artifact_reproduction_review_evidence: string;
  sandbox_contract_review_evidence: string;
  verdict: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_119_binding_confirmed: boolean;
  reviewer_independent_from_stage_119_builder_and_complete_prior_chain_confirmed: boolean;
  server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: boolean;
  self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: boolean;
  artifact_builder_and_reviewer_separation_confirmed: boolean;
  all_eight_observation_ledger_transition_functions_and_canonical_schemas_remain_bound_confirmed: boolean;
  session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: boolean;
  no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: boolean;
  future_financial_events_require_separately_admitted_opening_snapshot_confirmed: boolean;
  future_attempt_limited_to_non_financial_notice_candidate_without_authoritative_state_confirmed: boolean;
  provider_publication_time_remains_unverified_until_separate_evidence_confirmed: boolean;
  authorization_single_use_24_hour_expiry_and_stage_121_claim_separation_confirmed: boolean;
  no_runtime_entrypoint_mount_input_read_observation_ledger_transition_execution_or_candidate_output_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_121_claim_first_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  runner: ControlledShadowObservationLedgerTransitionIsolatedRunnerRecord;
  artifact_manifest: ControlledShadowObservationLedgerTransitionReproducedArtifactManifest;
  submitted_at: string;
  authorization_valid_until: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  server_computed_artifact_sha256: string;
  server_observed_artifact_byte_length: number;
  artifact_reproduction_review_evidence: string;
  sandbox_contract_review_evidence: string;
  verdict: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict;
  rationale: string;
  one_shot_execution_attempt_limit: number;
  one_future_claim_first_observation_ledger_transition_attempt_authorized: boolean;
  authorization_claimed: boolean;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_ledger_transition_executed: boolean;
  candidate_output_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  performance_metric_written: boolean;
  model_store_written: boolean;
  metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    runner: ControlledShadowObservationLedgerTransitionIsolatedRunnerRecord;
    artifact_inspection: ControlledShadowObservationLedgerTransitionArtifactInspection;
    latest_review: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview | null;
    authorization_unexpired: boolean;
    future_claim_eligible: boolean;
  }>;
  runner_count: number;
  artifact_verified_runner_count: number;
  artifact_pending_runner_count: number;
  review_eligible_runner_count: number;
  reviewed_runner_count: number;
  approved_runner_count: number;
  unexpired_authorization_count: number;
  one_shot_authorized_count: number;
  future_claim_eligible_count: number;
  authorization_status: string;
  next_gate: string;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_ledger_transition_executed: boolean;
  candidate_output_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest = {
  expected_authorization_review_sha256: string;
  expected_isolated_runner_spec_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_implementation_review_sha256: string;
  expected_observation_ledger_transition_specification_sha256: string;
  expected_stage_114_admission_review_sha256: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_111_claim_sha256: string;
  claim_reason: string;
  exact_current_stage_51_through_stage_120_binding_confirmed: boolean;
  claimant_independent_from_stage_120_and_complete_prior_chain_confirmed: boolean;
  authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: boolean;
  current_server_rehashed_artifact_and_manifest_binding_confirmed: boolean;
  exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed: boolean;
  claim_contains_only_existing_metadata_and_hashes_confirmed: boolean;
  no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed: boolean;
  future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: boolean;
  no_retry_release_or_authorization_restoration_after_claim_confirmed: boolean;
  no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionExecutionAttemptClaim = {
  attempt_id: string;
  claim_sha256: string;
  authorization: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview;
  claimed_at: string;
  claimed_by: string;
  claim_reason: string;
  authorization_consumed: boolean;
  create_once: boolean;
  claim_first: boolean;
  retry_allowed: boolean;
  release_allowed: boolean;
  authorization_restoration_allowed: boolean;
  task_status: string;
  execution_attempt_endpoint_available: boolean;
  input_read: boolean;
  observation_ledger_transition_executed: boolean;
  candidate_notice_created: boolean;
  ledger_event_written: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry = {
  schema_version: string;
  policy_version: string;
  claim_endpoint_available: boolean;
  eligible_authorizations: Array<{
    authorization: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview;
    claimant_excluded_actor_ids: string[];
  }>;
  claims: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim[];
  authorization_candidate_count: number;
  claim_eligible_count: number;
  claim_count: number;
  authorization_consumed_count: number;
  waiting_for_stage_122_execution_count: number;
  claim_status: string;
  next_gate: string;
  execution_attempt_endpoint_available: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read: boolean;
  observation_ledger_transition_executed: boolean;
  candidate_notice_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_event_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ExecuteControlledShadowObservationLedgerTransitionAttemptRequest = {
  expected_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_runner_contract_sha256: string;
  expected_runner_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_observation_ledger_transition_specification_sha256: string;
  expected_stage_114_admission_review_sha256: string;
  expected_stage_113_validation_sha256: string;
  expected_stage_112_result_sha256: string;
  expected_stage_112_output_sha256: string;
  expected_stage_111_claim_sha256: string;
  execution_reason: string;
  exact_stage_51_through_stage_121_binding_confirmed: boolean;
  executor_independent_from_complete_prior_chain_and_claimant_confirmed: boolean;
  start_marker_consumes_claim_before_artifact_or_input_read_confirmed: boolean;
  one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: boolean;
  artifact_is_declarative_not_spawned_or_executed_confirmed: boolean;
  only_exact_stage_114_admitted_output_is_read_only_reopened_and_rehashed_confirmed: boolean;
  opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed: boolean;
  non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed: boolean;
  raw_security_close_and_dividend_adjusted_spy_benchmark_separated_confirmed: boolean;
  explicit_gap_blocks_nav_and_corporate_actions_remain_pending_validation_confirmed: boolean;
  output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: boolean;
  no_network_environment_secret_tool_subprocess_or_production_io_confirmed: boolean;
  no_authoritative_financial_state_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  stage_121_attempt_id: string;
  stage_121_claim_sha256: string;
  completed_at: string;
  executed_by: string;
  execution_reason: string;
  duration_millis: number;
  status: "completed_with_untrusted_non_financial_notice_candidate" | "failed_claim_consumed";
  bounded_error_code?: string | null;
  candidate_sha256?: string | null;
  candidate_relative_path?: string | null;
  notice_candidate_count: number;
  claim_consumed: boolean;
  artifact_revalidated: boolean;
  exact_stage_114_input_revalidated_and_opened: boolean;
  transition_projector_executed_in_process: boolean;
  candidate_envelope_created: boolean;
  output_untrusted: boolean;
  independent_validation_completed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  financial_posting_created: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry = {
  schema_version: string;
  policy_version: string;
  execution_endpoint_available: boolean;
  pending_claims: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim[];
  results: ControlledShadowObservationLedgerTransitionExecutionAttemptResult[];
  pending_claim_count: number;
  terminal_result_count: number;
  successful_untrusted_candidate_count: number;
  failed_consumed_claim_count: number;
  next_gate: string;
  arbitrary_artifact_execution_allowed: boolean;
  outbound_network_allowed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  independent_validation_completed: boolean;
  non_financial_notice_candidate_created: boolean;
  ledger_created: boolean;
  ledger_event_written: boolean;
  financial_posting_created: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateControlledShadowObservationLedgerTransitionOutputRequest = {
  expected_claim_sha256: string;
  expected_result_sha256: string;
  expected_candidate_sha256: string;
  expected_specification_sha256: string;
  expected_stage_114_review_sha256: string;
  expected_stage_112_output_sha256: string;
  validation_reason: string;
  exact_current_stage_51_through_stage_122_binding_confirmed: boolean;
  validator_independent_from_executor_claimant_and_complete_prior_chain_confirmed: boolean;
  stage_122_result_and_create_once_candidate_reopened_and_rehashed_confirmed: boolean;
  exact_stage_114_admitted_observation_envelope_reopened_and_rehashed_confirmed: boolean;
  second_projection_does_not_call_stage_122_projector_helpers_confirmed: boolean;
  every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed: boolean;
  opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: boolean;
  pass_only_opens_future_stage_124_non_financial_candidate_admission_review_confirmed: boolean;
  no_ledger_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionOutputValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  stage_121_attempt_id: string;
  stage_121_claim_sha256: string;
  stage_122_result_id: string;
  stage_122_result_sha256: string;
  stage_122_candidate_sha256: string;
  observation_ledger_transition_specification_sha256: string;
  stage_114_review_sha256: string;
  stage_112_output_sha256: string;
  validated_at: string;
  validated_by: string;
  validation_reason: string;
  excluded_prior_actor_ids: string[];
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  validator_independent_from_executor_claimant_and_complete_prior_chain: boolean;
  exact_current_stage_51_through_stage_122_chain_verified: boolean;
  claim_fingerprint_independently_verified: boolean;
  result_fingerprint_independently_verified: boolean;
  candidate_file_custody_and_fingerprint_verified: boolean;
  exact_stage_114_admitted_observation_revalidated: boolean;
  complete_candidate_independently_reprojected: boolean;
  every_notice_identity_and_hash_independently_verified: boolean;
  exact_decimal_fields_independently_verified: boolean;
  canonical_sort_and_complete_candidate_exact_match_verified: boolean;
  opening_portfolio_absence_and_empty_financial_allowlist_verified: boolean;
  no_downstream_authority_verified: boolean;
  recomputed_claim_sha256: string;
  recomputed_result_sha256: string;
  recomputed_persisted_candidate_sha256: string;
  independently_recomputed_candidate_sha256: string;
  observed_candidate_bytes: number;
  observed_notice_count: number;
  observed_event_type_counts: Record<string, number>;
  mismatch_reasons: string[];
  verdict:
    | "independently_validated_exact_non_financial_notice_candidate"
    | "failed_independent_non_financial_notice_candidate_validation";
  non_financial_notice_candidate_independently_validated: boolean;
  future_stage_124_non_financial_candidate_admission_review_eligible: boolean;
  candidate_remains_untrusted: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  authoritative_ledger_event_created: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionOutputValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validator_implementation_version: string;
  validator_implementation_sha256: string;
  items: Array<{
    claim: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim;
    result: ControlledShadowObservationLedgerTransitionExecutionAttemptResult;
    validation?: ControlledShadowObservationLedgerTransitionOutputValidationRecord;
    validation_eligible: boolean;
  }>;
  validation_eligible_count: number;
  validation_count: number;
  independently_validated_candidate_count: number;
  failed_validation_count: number;
  future_stage_124_admission_review_eligible_count: number;
  validation_status: string;
  next_gate: string;
  independent_output_validation_available: boolean;
  candidate_remains_untrusted: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  authoritative_ledger_event_created: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type UntrustedNonFinancialObservationNoticeCandidateEnvelope = {
  schema_version: string;
  specification_sha256: string;
  stage_114_review_sha256: string;
  stage_113_validation_sha256: string;
  stage_112_result_sha256: string;
  stage_112_output_sha256: string;
  stage_111_claim_sha256: string;
  admitted_available_at_utc: string;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist: string[];
  notices: Array<{
    notice_id: string;
    notice_sha256: string;
    event_type: string;
    effective_date: string;
    available_at_utc: string;
    exact_decimal_fields: Record<string, string>;
    non_financial: boolean;
    untrusted: boolean;
    authoritative: boolean;
    financial_posting_created: boolean;
    ledger_event_written: boolean;
  }>;
  candidate_sha256: string;
  create_once: boolean;
  untrusted: boolean;
  independent_validation_completed: boolean;
  ledger_created: boolean;
  authoritative_financial_state_created: boolean;
  nav_or_performance_calculated: boolean;
  order_intent_created: boolean;
};

export type IndependentlyValidatedNonFinancialObservationNoticeCandidate = {
  claim: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim;
  result: ControlledShadowObservationLedgerTransitionExecutionAttemptResult;
  validation: ControlledShadowObservationLedgerTransitionOutputValidationRecord;
  candidate: UntrustedNonFinancialObservationNoticeCandidateEnvelope;
};

export type ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest = {
  expected_previous_review_id?: string | null;
  expected_previous_review_sha256?: string | null;
  expected_stage_123_validation_id: string;
  expected_stage_123_validation_sha256: string;
  expected_stage_122_result_sha256: string;
  expected_stage_122_candidate_sha256: string;
  expected_stage_121_claim_sha256: string;
  expected_stage_114_review_sha256: string;
  expected_stage_112_output_sha256: string;
  verdict:
    | "admitted_as_formal_non_financial_observation_evidence_for_future_opening_portfolio_governance"
    | "changes_requested"
    | "rejected";
  rationale: string;
  known_limitations: string;
  exact_current_stage_51_through_stage_123_binding_confirmed: boolean;
  reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed: boolean;
  stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: boolean;
  stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: boolean;
  exact_stage_114_admitted_observation_binding_preserved_confirmed: boolean;
  every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: boolean;
  admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed: boolean;
  opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed: boolean;
  approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed: boolean;
  no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ControlledShadowObservationLedgerTransitionCandidateAdmissionReview = {
  review_id: string;
  review_sha256: string;
  previous_review_id?: string | null;
  previous_review_sha256?: string | null;
  stage_121_attempt_id: string;
  stage_121_claim_sha256: string;
  stage_122_result_sha256: string;
  stage_122_candidate_sha256: string;
  stage_123_validation_id: string;
  stage_123_validation_sha256: string;
  stage_114_review_sha256: string;
  stage_112_output_sha256: string;
  stage_123_validated_at: string;
  submitted_at: string;
  submitted_by: string;
  verdict: ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest["verdict"];
  rationale: string;
  known_limitations: string;
  notice_count: number;
  event_type_counts: Record<string, number>;
  original_candidate_remains_untrusted_and_immutable: boolean;
  formal_non_financial_observation_evidence_admitted: boolean;
  future_stage_125_opening_portfolio_snapshot_governance_specification_eligible: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  authoritative_ledger_event_created: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    candidate: IndependentlyValidatedNonFinancialObservationNoticeCandidate;
    latest_review?: ControlledShadowObservationLedgerTransitionCandidateAdmissionReview | null;
    current_binding: boolean;
    review_eligible: boolean;
    formal_non_financial_observation_evidence_admitted: boolean;
  }>;
  independently_validated_candidate_count: number;
  review_eligible_candidate_count: number;
  reviewed_candidate_count: number;
  admitted_non_financial_observation_evidence_count: number;
  changes_requested_or_rejected_count: number;
  future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count: number;
  admission_status: string;
  next_gate: string;
  admission_review_available: boolean;
  candidate_remains_untrusted: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  authoritative_ledger_event_created: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type OpeningPortfolioExternalSourceKind =
  | "broker_or_custodian_machine_export"
  | "broker_or_custodian_statement"
  | "verified_portfolio_accounting_system_export";

export type RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest = {
  expected_stage_124_review_id: string;
  expected_stage_124_review_sha256: string;
  expected_stage_123_validation_sha256: string;
  expected_stage_122_candidate_sha256: string;
  expected_stage_114_review_sha256: string;
  expected_stage_112_output_sha256: string;
  source_kind: OpeningPortfolioExternalSourceKind;
  source_provider_name: string;
  portfolio_scope_alias: string;
  reporting_currency: string;
  source_timezone: string;
  snapshot_as_of_utc: string;
  expected_account_count: number;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_124_binding_confirmed: boolean;
  registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed: boolean;
  stage_124_admission_reopened_rehashed_and_current_confirmed: boolean;
  external_source_artifact_required_and_manual_balances_forbidden_confirmed: boolean;
  account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed: boolean;
  all_cash_positions_liabilities_and_unsettled_activity_required_confirmed: boolean;
  exact_decimal_signed_quantities_and_no_default_or_inference_confirmed: boolean;
  instrument_identity_and_corporate_action_reconciliation_required_confirmed: boolean;
  statement_market_values_are_informational_not_accounting_marks_confirmed: boolean;
  complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed: boolean;
  source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed: boolean;
  specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: boolean;
  no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  future_stage_126_independent_specification_review_required_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactContract = {
  source_kind: OpeningPortfolioExternalSourceKind;
  source_provider_name: string;
  portfolio_scope_alias: string;
  reporting_currency: string;
  source_timezone: string;
  snapshot_as_of_utc: string;
  expected_account_count: number;
  accepted_artifact_formats: string[];
  original_bytes_required: boolean;
  content_sha256_and_byte_length_required: boolean;
  provider_statement_or_export_identifier_required: boolean;
  provider_generated_at_or_statement_as_of_required: boolean;
  hone_received_at_required: boolean;
  source_account_identifiers_must_be_pseudonymized: boolean;
  raw_account_numbers_or_credentials_allowed: boolean;
  manual_balance_or_position_entry_allowed: boolean;
  mutable_or_overwritable_artifact_allowed: boolean;
};

export type OpeningPortfolioCanonicalSnapshotSchema = {
  account_schema: string;
  cash_schema: string;
  position_schema: string;
  listed_option_extension_schema: string;
  liability_schema: string;
  unsettled_activity_schema: string;
  instrument_identity_precedence: string[];
  supported_asset_classes: string[];
  unsupported_asset_class_result: string;
  exact_decimal_rule: string;
  signed_quantity_rule: string;
  duplicate_instrument_rule: string;
  cost_basis_rule: string;
  statement_market_value_rule: string;
  account_scope_completeness_rule: string;
  cash_completeness_rule: string;
  liabilities_and_unsettled_activity_rule: string;
  corporate_action_reconciliation_rule: string;
  missing_or_ambiguous_field_rule: string;
  opening_nav_rule: string;
  performance_inception_rule: string;
  correction_rule: string;
};

export type ZeroCapabilityOpeningPortfolioAuthorityBoundary = {
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  parser_or_implementation_present: boolean;
  executable_artifact_or_entrypoint_present: boolean;
  runtime_present: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  ledger_event_write_allowed: boolean;
  position_write_allowed: boolean;
  cash_write_allowed: boolean;
  nav_or_performance_write_allowed: boolean;
  model_or_metric_store_write_allowed: boolean;
  training_or_rl_feedback_allowed: boolean;
  reward_allowed: boolean;
  order_generation_allowed: boolean;
  broker_access_allowed: boolean;
  trading_allowed: boolean;
};

export type OpeningPortfolioSnapshotGovernanceSpecification = {
  schema_version: string;
  specification_sha256: string;
  protocol_version: string;
  stage_124_review_id: string;
  stage_124_review_sha256: string;
  stage_123_validation_sha256: string;
  stage_122_candidate_sha256: string;
  stage_114_review_sha256: string;
  stage_112_output_sha256: string;
  source_contract: OpeningPortfolioSourceArtifactContract;
  canonical_snapshot_schema: OpeningPortfolioCanonicalSnapshotSchema;
  future_source_artifact_receipt_validation_required: boolean;
  future_canonical_snapshot_materialization_required: boolean;
  future_independent_snapshot_output_validation_required: boolean;
  future_opening_snapshot_admission_review_required: boolean;
  create_once_required: boolean;
  append_only_corrections_required: boolean;
  overwrite_allowed: boolean;
  default_notional_allowed: boolean;
  infer_cash_positions_quantities_cost_basis_or_weights_allowed: boolean;
  financial_postings_currently_eligible: boolean;
  nav_or_performance_currently_eligible: boolean;
  future_independent_specification_review_required: boolean;
  authority_boundary: ZeroCapabilityOpeningPortfolioAuthorityBoundary;
};

export type OpeningPortfolioSnapshotGovernanceSpecificationRegistration = {
  schema_version: string;
  policy_version: string;
  registration_id: string;
  registration_sha256: string;
  registered_at: string;
  registered_by: string;
  stage_124_review_id: string;
  stage_124_review_sha256: string;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_124_reviewer_and_complete_prior_chain: boolean;
  registration_reason: string;
  known_limitations: string;
  future_review_constraints: string;
  specification: OpeningPortfolioSnapshotGovernanceSpecification;
  status: string;
  confirmations_complete: boolean;
  specification_registered: boolean;
  future_stage_126_independent_specification_review_eligible: boolean;
  specification_review_completed: boolean;
  source_artifact_receipt_eligible: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  position_written: boolean;
  cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSnapshotGovernanceSpecificationCandidate = {
  stage_124_review_id: string;
  stage_124_review_sha256: string;
  stage_123_validation_sha256: string;
  stage_122_candidate_sha256: string;
  stage_114_review_sha256: string;
  stage_112_output_sha256: string;
  formal_non_financial_observation_notice_count: number;
  registrar_excluded_actor_ids: string[];
};

export type OpeningPortfolioSnapshotGovernanceSpecificationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  candidates: OpeningPortfolioSnapshotGovernanceSpecificationCandidate[];
  registrations: OpeningPortfolioSnapshotGovernanceSpecificationRegistration[];
  stage_124_admitted_evidence_count: number;
  registration_eligible_count: number;
  registered_specification_count: number;
  future_stage_126_independent_specification_review_eligible_count: number;
  registration_status: string;
  next_gate: string;
  source_artifact_present: boolean;
  opening_portfolio_snapshot_present: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict =
  | "approved_for_future_zero_capability_source_artifact_receipt_implementation_registration"
  | "changes_required_rebuild_opening_portfolio_governance_specification"
  | "rejected_opening_portfolio_governance_specification";

export type ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest = {
  expected_previous_review_id?: string;
  expected_previous_review_sha256?: string;
  expected_registration_sha256: string;
  expected_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict;
  rationale: string;
  binding_and_second_implementation_assessment: string;
  source_artifact_and_identity_assessment: string;
  account_scope_and_snapshot_completeness_assessment: string;
  valuation_and_nav_prerequisite_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  exact_current_stage_51_through_stage_125_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  complete_specification_rebuilt_without_stage_125_builder_confirmed: boolean;
  rebuilt_specification_exactly_matches_registered_specification_confirmed: boolean;
  original_external_artifact_provenance_and_pseudonymization_contract_confirmed: boolean;
  complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: boolean;
  exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed: boolean;
  instrument_identity_cost_basis_and_corporate_action_contract_confirmed: boolean;
  statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: boolean;
  source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed: boolean;
  no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: boolean;
  no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  registration_id: string;
  registration_sha256: string;
  specification_sha256: string;
  registration_hash_independently_reproduced: boolean;
  specification_hash_independently_reproduced: boolean;
  exact_current_stage_51_through_stage_125_binding_valid: boolean;
  complete_specification_rebuilt_without_stage_125_builder: boolean;
  rebuilt_specification_exactly_matches_registration: boolean;
  external_source_artifact_and_identity_contract_valid: boolean;
  complete_account_scope_and_snapshot_schema_contract_valid: boolean;
  exact_decimal_no_invention_and_append_only_contract_valid: boolean;
  statement_value_and_independent_valuation_prerequisite_contract_valid: boolean;
  future_gates_remain_separate_and_current_financial_state_closed: boolean;
  all_artifact_runtime_ledger_feedback_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  previous_review_id?: string;
  previous_review_sha256?: string;
  registration: OpeningPortfolioSnapshotGovernanceSpecificationRegistration;
  independent_audit: OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict;
  rationale: string;
  binding_and_second_implementation_assessment: string;
  source_artifact_and_identity_assessment: string;
  account_scope_and_snapshot_completeness_assessment: string;
  valuation_and_nav_prerequisite_assessment: string;
  zero_capability_assessment: string;
  known_limitations: string;
  future_implementation_constraints: string;
  confirmations_complete: boolean;
  specification_independently_approved: boolean;
  future_zero_capability_source_artifact_receipt_implementation_registration_eligible: boolean;
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  parser_or_runtime_present: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  review_endpoint_available: boolean;
  items: Array<{
    registration: OpeningPortfolioSnapshotGovernanceSpecificationRegistration;
    current_independent_audit: OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit;
    complete_review_actor_ids: string[];
    latest_review?: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord;
    review_eligible: boolean;
    future_zero_capability_source_artifact_receipt_implementation_registration_eligible: boolean;
  }>;
  specification_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count: number;
  review_status: string;
  source_artifact_present: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest = {
  expected_stage_126_review_id: string;
  expected_stage_126_review_sha256: string;
  expected_stage_126_independent_audit_sha256: string;
  expected_stage_125_registration_id: string;
  expected_stage_125_registration_sha256: string;
  expected_stage_125_specification_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  transport_and_authentication_semantics: string;
  streaming_hash_length_and_atomic_commit_semantics: string;
  format_magic_and_active_content_rejection_semantics: string;
  pseudonymization_and_secret_redaction_semantics: string;
  quarantine_cleanup_and_idempotency_semantics: string;
  audit_and_retention_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_126_binding_confirmed: boolean;
  registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: boolean;
  review_registration_specification_and_audit_hashes_recomputed_confirmed: boolean;
  exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: boolean;
  original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: boolean;
  content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed: boolean;
  archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed: boolean;
  source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed: boolean;
  private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: boolean;
  server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed: boolean;
  duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: boolean;
  receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: boolean;
  receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed: boolean;
  contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  future_stage_128_independent_implementation_review_required_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationContract = {
  schema_version: string;
  contract_sha256: string;
  protocol_version: string;
  immutable_code_revision: string;
  stage_126_review_id: string;
  stage_126_review_sha256: string;
  stage_126_independent_audit_sha256: string;
  stage_125_registration_id: string;
  stage_125_registration_sha256: string;
  stage_125_specification_sha256: string;
  exact_stage_125_specification: OpeningPortfolioSnapshotGovernanceSpecification;
  exact_source_artifact_contract: OpeningPortfolioSourceArtifactContract;
  future_transport_scope: string;
  future_maximum_artifact_bytes: number;
  future_maximum_receipt_bytes: number;
  future_maximum_artifact_count: number;
  validate_declared_metadata_before_byte_acceptance_function_id: string;
  stream_private_quarantine_while_hashing_and_counting_function_id: string;
  validate_format_magic_and_safe_structure_without_financial_parsing_function_id: string;
  reject_archive_active_content_password_and_unsafe_path_function_id: string;
  pseudonymize_account_identity_and_redact_secrets_function_id: string;
  atomic_content_addressed_create_new_commit_function_id: string;
  append_only_redacted_receipt_manifest_function_id: string;
  cleanup_partial_quarantine_on_failure_function_id: string;
  future_private_quarantine_relative_path_template: string;
  future_content_addressed_artifact_relative_path_template: string;
  future_receipt_manifest_schema: string;
  original_bytes_preserved_immutable: boolean;
  encryption_at_rest_required: boolean;
  server_owned_received_at_required: boolean;
  raw_account_numbers_or_credentials_in_paths_metadata_or_logs_allowed: boolean;
  overwrite_or_mutable_artifact_allowed: boolean;
  financial_row_parsing_allowed_in_receipt_stage: boolean;
  future_receipt_output_untrusted: boolean;
  future_independent_receipt_validation_required: boolean;
  future_snapshot_materialization_separate: boolean;
  future_snapshot_output_validation_separate: boolean;
  future_snapshot_admission_review_separate: boolean;
  registered_not_run: boolean;
  independent_implementation_review_required: boolean;
  future_isolated_receiver_registration_required_after_review: boolean;
  authority_boundary: Record<string, boolean>;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationRegistration = {
  schema_version: string;
  policy_version: string;
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  upstream_stage_126_review: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord;
  excluded_prior_actor_ids: string[];
  registrar_independent_from_stage_126_reviewer_and_complete_prior_chain: boolean;
  implementation_name: string;
  implementation_description: string;
  transport_and_authentication_semantics: string;
  streaming_hash_length_and_atomic_commit_semantics: string;
  format_magic_and_active_content_rejection_semantics: string;
  pseudonymization_and_secret_redaction_semantics: string;
  quarantine_cleanup_and_idempotency_semantics: string;
  audit_and_retention_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  implementation_contract: OpeningPortfolioSourceArtifactReceiptImplementationContract;
  status: string;
  confirmations: Record<string, boolean>;
  confirmations_complete: boolean;
  zero_capability_implementation_contract_registered: boolean;
  future_stage_128_independent_implementation_review_eligible: boolean;
  independent_implementation_review_completed: boolean;
  source_artifact_receipt_eligible: boolean;
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  items: Array<{
    specification_review: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord;
    implementation?: OpeningPortfolioSourceArtifactReceiptImplementationRegistration;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_stage_128_independent_implementation_review_eligible: boolean;
  }>;
  independently_approved_specification_count: number;
  registration_eligible_count: number;
  implementation_contract_count: number;
  current_binding_implementation_contract_count: number;
  future_stage_128_independent_implementation_review_eligible_count: number;
  implementation_status: string;
  upload_endpoint_present: boolean;
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict =
  | "approved_for_future_isolated_source_artifact_receiver_specification_registration"
  | "changes_required_rebuild_source_artifact_receipt_implementation"
  | "rejected_source_artifact_receipt_implementation";

export type OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations = {
  exact_current_stage_51_through_stage_127_binding_confirmed: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: boolean;
  complete_contract_rebuilt_without_stage_127_builder_confirmed: boolean;
  all_stage_127_registration_confirmations_revalidated_confirmed: boolean;
  original_provider_formats_and_resource_ceilings_preserved_confirmed: boolean;
  administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: boolean;
  streaming_sha256_length_private_quarantine_and_atomic_commit_confirmed: boolean;
  format_magic_safe_structure_and_active_content_rejection_confirmed: boolean;
  account_pseudonymization_and_secret_redaction_confirmed: boolean;
  encryption_content_addressing_create_new_idempotency_and_failure_cleanup_confirmed: boolean;
  server_received_time_redacted_manifest_and_untrusted_receipt_confirmed: boolean;
  receipt_validation_materialization_output_validation_and_admission_remain_separate_confirmed: boolean;
  no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_129_isolated_receiver_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest = {
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_stage_126_review_sha256: string;
  expected_stage_126_independent_audit_sha256: string;
  expected_stage_125_registration_sha256: string;
  expected_stage_125_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  transport_resource_and_format_assessment: string;
  privacy_storage_and_manifest_assessment: string;
  separation_and_zero_capability_assessment: string;
  known_limitations: string;
  future_receiver_constraints: string;
  confirmations: OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  stage_126_review_sha256: string;
  stage_126_independent_audit_sha256: string;
  stage_125_registration_sha256: string;
  stage_125_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  stage_126_review_hash_independently_reproduced: boolean;
  stage_126_independent_audit_hash_independently_reproduced: boolean;
  stage_125_registration_hash_independently_reproduced: boolean;
  stage_125_specification_hash_independently_reproduced: boolean;
  complete_contract_rebuilt_without_stage_127_builder: boolean;
  rebuilt_contract_exactly_matches_record: boolean;
  exact_current_stage_51_through_stage_127_binding_valid: boolean;
  all_stage_127_registration_confirmations_valid: boolean;
  source_formats_transport_and_resource_ceilings_valid: boolean;
  streaming_quarantine_format_and_active_content_rejection_valid: boolean;
  privacy_encryption_content_addressing_and_failure_cleanup_valid: boolean;
  manifest_untrusted_output_and_separation_contract_valid: boolean;
  all_upload_source_parser_financial_model_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration;
  independent_audit: OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  transport_resource_and_format_assessment: string;
  privacy_storage_and_manifest_assessment: string;
  separation_and_zero_capability_assessment: string;
  known_limitations: string;
  future_receiver_constraints: string;
  confirmations: OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations;
  confirmations_complete: boolean;
  reviewer_independent_from_registrar_and_complete_prior_chain: boolean;
  zero_capability_implementation_independently_approved: boolean;
  future_stage_129_isolated_receiver_specification_registration_eligible: boolean;
  isolated_receiver_specification_registered: boolean;
  upload_endpoint_present: boolean;
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  parser_or_runtime_present: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration;
    current_independent_audit: OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit;
    review?: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord;
    review_eligible: boolean;
    future_stage_129_isolated_receiver_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_stage_129_isolated_receiver_specification_registration_eligible_count: number;
  review_status: string;
  isolated_receiver_specification_registered: boolean;
  upload_endpoint_present: boolean;
  source_artifact_present: boolean;
  source_artifact_uploaded_or_read: boolean;
  parser_or_runtime_present: boolean;
  opening_portfolio_snapshot_present: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind =
  "ephemeral_deterministic_stream_only_receipt_specification";

export type RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest = {
  expected_stage_128_review_id: string;
  expected_stage_128_review_sha256: string;
  expected_stage_128_independent_audit_sha256: string;
  expected_stage_127_implementation_id: string;
  expected_stage_127_implementation_sha256: string;
  expected_stage_127_implementation_contract_sha256: string;
  expected_stage_126_review_sha256: string;
  expected_stage_126_independent_audit_sha256: string;
  expected_stage_125_registration_sha256: string;
  expected_stage_125_specification_sha256: string;
  receiver_name: string;
  receiver_kind: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind;
  receiver_spec_revision: string;
  proposed_receiver_code_revision: string;
  proposed_receiver_artifact_sha256: string;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_128_binding_confirmed: boolean;
  registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: boolean;
  review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: boolean;
  all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: boolean;
  exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: boolean;
  future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: boolean;
  future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: boolean;
  future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed: boolean;
  future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: boolean;
  future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: boolean;
  future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract = {
  schema_version: string;
  contract_sha256: string;
  stage_128_review_id: string;
  stage_128_review_sha256: string;
  stage_128_independent_audit_sha256: string;
  stage_127_implementation_id: string;
  stage_127_implementation_sha256: string;
  stage_127_implementation_contract_sha256: string;
  stage_126_review_sha256: string;
  stage_126_independent_audit_sha256: string;
  stage_125_registration_sha256: string;
  stage_125_specification_sha256: string;
  exact_approved_implementation_contract: OpeningPortfolioSourceArtifactReceiptImplementationContract;
  receiver_spec_revision: string;
  proposed_receiver_code_revision: string;
  proposed_receiver_artifact_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  future_input_envelope: string;
  future_output_envelope: string;
  next_gate: string;
  specification_registered: boolean;
  future_receiver_artifact_identity_bound: boolean;
  maximum_parallel_runs: number;
  maximum_memory_mib: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
  [key: string]: unknown;
};

export type OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord = {
  schema_version: string;
  policy_version: string;
  isolated_receiver_id: string;
  isolated_receiver_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration;
  implementation_review: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  receiver_name: string;
  receiver_kind: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  receiver_contract: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract;
  status: string;
  confirmations_complete: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  source_artifact_received_or_read: boolean;
  receipt_manifest_created: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  [key: string]: unknown;
};

export type OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration;
    review: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  isolated_receiver_count: number;
  current_binding_receiver_count: number;
  first_execution_authorization_review_eligible_count: number;
  items: Array<{
    receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  receiver_status: string;
  upload_endpoint_present: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_accessed: boolean;
  receipt_manifest_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict =
  | "approved_for_one_future_claim_first_source_artifact_receipt_attempt"
  | "changes_requested_rebuild_artifact"
  | "rejected";

export type OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest = {
  schema_version: string;
  manifest_sha256: string;
  isolated_receiver_id: string;
  isolated_receiver_spec_sha256: string;
  receiver_contract_sha256: string;
  receiver_spec_revision: string;
  receiver_code_revision: string;
  receiver_artifact_sha256: string;
  artifact_byte_length: number;
  artifact_file_name: string;
  artifact_media_type: string;
  source_bundle_sha256: string;
  artifact_reproduction_procedure_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  reproduced_at: string;
  reproduced_by: string;
  source_and_artifact_reproduced_from_immutable_revision: boolean;
  artifact_is_read_only_regular_file: boolean;
  artifact_was_not_executed: boolean;
  source_artifact_was_not_received_or_read: boolean;
};

export type ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest = {
  expected_review_id?: string;
  expected_review_sha256?: string;
  expected_isolated_receiver_id: string;
  expected_isolated_receiver_spec_sha256: string;
  expected_receiver_contract_sha256: string;
  expected_receiver_spec_revision: string;
  expected_receiver_code_revision: string;
  expected_receiver_artifact_sha256: string;
  expected_stage_128_review_id: string;
  expected_stage_128_review_sha256: string;
  expected_stage_128_independent_audit_sha256: string;
  expected_stage_127_implementation_sha256: string;
  expected_stage_127_implementation_contract_sha256: string;
  expected_stage_126_review_sha256: string;
  expected_stage_125_registration_sha256: string;
  expected_stage_125_specification_sha256: string;
  expected_artifact_manifest_sha256: string;
  artifact_reproduction_review_evidence: string;
  sandbox_contract_review_evidence: string;
  verdict: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict;
  rationale: string;
  exact_current_stage_51_through_stage_129_binding_confirmed: boolean;
  reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed: boolean;
  server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: boolean;
  self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: boolean;
  artifact_builder_and_reviewer_separation_confirmed: boolean;
  all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: boolean;
  exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: boolean;
  future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: boolean;
  future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: boolean;
  future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: boolean;
  future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: boolean;
  future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: boolean;
  authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: boolean;
  no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: boolean;
  no_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_131_claim_first_attempt_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview =
  ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest & {
    schema_version: string;
    policy_version: string;
    review_id: string;
    review_sha256: string;
    previous_review_id: string | null;
    previous_review_sha256: string | null;
    receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord;
    artifact_manifest: OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest;
    submitted_at: string;
    authorization_valid_until: string;
    reviewer_id: string;
    excluded_prior_actor_ids: string[];
    server_computed_artifact_sha256: string;
    server_observed_artifact_byte_length: number;
    one_shot_execution_attempt_limit: number;
    one_future_claim_first_source_artifact_receipt_attempt_authorized: boolean;
    authorization_claimed: boolean;
    [key: string]: unknown;
  };

export type OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord;
    artifact_inspection: {
      custody_locator: string;
      manifest_present: boolean;
      artifact_present: boolean;
      manifest: OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest | null;
      server_computed_artifact_sha256: string | null;
      server_observed_artifact_byte_length: number | null;
      artifact_verified: boolean;
      status: string;
    };
    latest_review: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview | null;
    authorization_unexpired: boolean;
    future_claim_eligible: boolean;
  }>;
  receiver_count: number;
  artifact_verified_receiver_count: number;
  artifact_pending_receiver_count: number;
  review_eligible_receiver_count: number;
  reviewed_receiver_count: number;
  approved_receiver_count: number;
  unexpired_authorization_count: number;
  future_claim_eligible_count: number;
  authorization_status: string;
  next_gate: string;
  upload_endpoint_present: boolean;
  runtime_instantiated: boolean;
  source_artifact_received_or_read: boolean;
  receipt_manifest_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest = {
  expected_authorization_review_sha256: string;
  expected_isolated_receiver_spec_sha256: string;
  expected_receiver_contract_sha256: string;
  expected_receiver_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_artifact_byte_length: number;
  claim_reason: string;
  exact_current_stage_51_through_stage_130_binding_confirmed: boolean;
  claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed: boolean;
  authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed: boolean;
  server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed: boolean;
  claim_contains_only_existing_metadata_and_hashes_confirmed: boolean;
  no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed: boolean;
  future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed: boolean;
  no_retry_release_or_authorization_restoration_after_claim_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry = {
  schema_version: string;
  policy_version: string;
  claim_endpoint_available: boolean;
  eligible_authorizations: Array<{
    authorization: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview;
    claimant_excluded_actor_ids: string[];
  }>;
  claims: Array<{
    attempt_id: string;
    claim_sha256: string;
    authorization: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview;
    claimed_at: string;
    claimed_by: string;
    claim_reason: string;
    task_status: string;
    [key: string]: unknown;
  }>;
  authorization_candidate_count: number;
  claim_eligible_count: number;
  claim_count: number;
  authorization_consumed_count: number;
  waiting_for_stage_132_attempt_count: number;
  claim_status: string;
  next_gate: string;
  stage_132_receipt_attempt_endpoint_available: boolean;
  upload_stream_opened: boolean;
  source_artifact_received_or_read: boolean;
  runtime_instantiated: boolean;
  receipt_manifest_created: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest = {
  expected_claim_sha256: string;
  expected_authorization_review_sha256: string;
  expected_isolated_receiver_spec_sha256: string;
  expected_receiver_contract_sha256: string;
  expected_receiver_artifact_sha256: string;
  expected_artifact_manifest_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_stage_125_specification_sha256: string;
  provider_statement_or_export_identifier: string;
  provider_generated_at_or_statement_as_of: string;
  artifacts: Array<{
    declared_format: "original_provider_pdf_statement" | "original_provider_csv_export" | "original_provider_json_export";
    source_account_aliases: string[];
  }>;
  execution_reason: string;
  exact_current_stage_51_through_stage_131_binding_confirmed: boolean;
  executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed: boolean;
  start_marker_consumes_claim_before_first_source_byte_confirmed: boolean;
  administrator_authenticated_stream_only_no_remote_fetch_confirmed: boolean;
  original_artifacts_already_account_pseudonymized_and_credentials_removed_confirmed: boolean;
  format_magic_safe_structure_archive_active_content_password_symlink_and_path_rejection_confirmed: boolean;
  streaming_sha256_length_private_quarantine_and_atomic_content_addressed_commit_confirmed: boolean;
  encryption_at_rest_and_redacted_manifest_confirmed: boolean;
  duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: boolean;
  receipt_create_once_untrusted_and_stage_133_independent_validation_required_confirmed: boolean;
  no_financial_row_parsing_snapshot_materialization_or_snapshot_admission_confirmed: boolean;
  no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult = {
  result_id: string;
  result_sha256: string;
  stage_131_attempt_id: string;
  stage_131_claim_sha256: string;
  completed_at: string;
  executed_by: string;
  execution_reason: string;
  status: "completed_with_untrusted_receipt" | "failed_claim_consumed";
  bounded_error_code: string | null;
  receipt_id: string | null;
  receipt_manifest_sha256: string | null;
  artifact_count: number;
  total_original_byte_length: number;
  source_artifact_received_or_read: boolean;
  source_artifact_may_have_been_read: boolean;
  receipt_untrusted: boolean;
  independent_receipt_validation_completed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  trading_authorized: boolean;
  [key: string]: unknown;
};

export type OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry = {
  schema_version: string;
  policy_version: string;
  receipt_endpoint_available: boolean;
  encryption_key_configured: boolean;
  pending_claims: OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry["claims"];
  results: OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult[];
  pending_claim_count: number;
  terminal_result_count: number;
  successful_untrusted_receipt_count: number;
  failed_consumed_claim_count: number;
  next_gate: string;
  receipt_manifest_created: boolean;
  independent_receipt_validation_completed: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};

export type ValidateOpeningPortfolioSourceArtifactReceiptRequest = {
  expected_stage_131_claim_sha256: string;
  expected_stage_132_result_sha256: string;
  expected_receipt_manifest_sha256: string;
  expected_stage_130_authorization_review_sha256: string;
  expected_stage_129_isolated_receiver_spec_sha256: string;
  expected_stage_127_implementation_contract_sha256: string;
  expected_stage_125_specification_sha256: string;
  validation_reason: string;
  exact_stage_51_through_stage_132_chain_reopened_confirmed: boolean;
  validator_independent_from_stage_132_executor_stage_131_claimant_and_complete_prior_chain_confirmed: boolean;
  result_and_receipt_fingerprints_independently_recomputed_confirmed: boolean;
  server_derived_manifest_and_content_addressed_paths_only_confirmed: boolean;
  ciphertext_regular_read_only_size_and_sha256_recomputed_confirmed: boolean;
  encryption_key_fingerprint_and_aead_authenticated_decryption_confirmed: boolean;
  plaintext_length_sha256_and_content_address_independently_recomputed_confirmed: boolean;
  format_magic_safe_structure_and_sensitive_field_screening_independently_repeated_confirmed: boolean;
  receipt_redaction_and_no_original_filename_account_number_or_credential_confirmed: boolean;
  terminal_create_once_validation_no_replay_confirmed: boolean;
  receipt_validation_only_no_financial_row_parsing_or_snapshot_materialization_confirmed: boolean;
  no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSourceArtifactReceiptValidationCandidate = {
  stage_131_attempt_id: string;
  stage_131_claim_sha256: string;
  stage_132_result_sha256: string;
  receipt_id: string;
  receipt_manifest_sha256: string;
  stage_130_authorization_review_sha256: string;
  stage_129_isolated_receiver_spec_sha256: string;
  stage_127_implementation_contract_sha256: string;
  stage_125_specification_sha256: string;
  artifact_count: number;
  total_original_byte_length: number;
  stage_132_executor_id: string;
  validator_excluded_actor_ids: string[];
};

export type OpeningPortfolioSourceArtifactReceiptValidationRecord = {
  validation_id: string;
  validation_sha256: string;
  stage_131_attempt_id: string;
  stage_132_result_sha256: string;
  receipt_id: string;
  validated_at: string;
  validated_by: string;
  validation_reason: string;
  verdict: "independently_validated_encrypted_untrusted_receipt" | "failed_independent_encrypted_receipt_validation";
  artifact_count: number;
  total_plaintext_byte_length: number;
  mismatch_reasons: string[];
  source_artifact_receipt_independently_validated: boolean;
  future_stage_134_snapshot_materialization_implementation_registration_eligible: boolean;
  financial_rows_parsed: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  [key: string]: unknown;
};

export type OpeningPortfolioSourceArtifactReceiptValidationRegistry = {
  schema_version: string;
  policy_version: string;
  validation_endpoint_available: boolean;
  encryption_key_configured: boolean;
  candidates: OpeningPortfolioSourceArtifactReceiptValidationCandidate[];
  validations: OpeningPortfolioSourceArtifactReceiptValidationRecord[];
  completed_untrusted_receipt_count: number;
  pending_independent_validation_count: number;
  independently_validated_receipt_count: number;
  failed_independent_validation_count: number;
  future_stage_134_snapshot_materialization_implementation_registration_eligible_count: number;
  validation_status: string;
  financial_rows_parsed: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  ledger_created: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  next_gate: string;
  scope: string;
};

export type RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest = {
  expected_stage_133_validation_id: string;
  expected_stage_133_validation_sha256: string;
  expected_stage_132_result_sha256: string;
  expected_stage_131_claim_sha256: string;
  expected_receipt_id: string;
  expected_receipt_manifest_sha256: string;
  expected_stage_125_specification_sha256: string;
  implementation_name: string;
  immutable_code_revision: string;
  implementation_description: string;
  deterministic_parser_and_adapter_semantics: string;
  account_scope_and_completeness_semantics: string;
  exact_decimal_and_signed_quantity_semantics: string;
  instrument_identity_and_corporate_action_semantics: string;
  row_provenance_and_redaction_semantics: string;
  whole_snapshot_failure_and_correction_semantics: string;
  known_limitations: string;
  future_review_constraints: string;
  exact_current_stage_51_through_stage_133_binding_confirmed: boolean;
  registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed: boolean;
  validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: boolean;
  exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: boolean;
  future_input_only_independently_validated_content_addressed_receipt_confirmed: boolean;
  future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: boolean;
  deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: boolean;
  account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: boolean;
  exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: boolean;
  instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: boolean;
  no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed: boolean;
  statement_market_values_informational_and_no_nav_or_performance_confirmed: boolean;
  every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed: boolean;
  future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed: boolean;
  contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: boolean;
  no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  future_stage_135_chain_external_independent_implementation_review_required_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSnapshotMaterializationImplementationCandidate = {
  stage_131_attempt_id: string;
  stage_133_validation_id: string;
  stage_133_validation_sha256: string;
  stage_132_result_sha256: string;
  stage_131_claim_sha256: string;
  receipt_id: string;
  receipt_manifest_sha256: string;
  stage_125_specification_sha256: string;
  source_provider_name: string;
  portfolio_scope_alias: string;
  artifact_count: number;
  registrar_excluded_actor_ids: string[];
};

export type OpeningPortfolioSnapshotMaterializationImplementationRegistration = {
  implementation_id: string;
  implementation_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation_name: string;
  upstream_stage_133_validation: {
    validation_sha256: string;
    stage_132_result_sha256: string;
    stage_131_claim_sha256: string;
    receipt_manifest_sha256: string;
    stage_125_specification_sha256: string;
    [key: string]: unknown;
  };
  immutable_code_revision?: string;
  status: string;
  confirmations_complete: boolean;
  zero_capability_implementation_contract_registered: boolean;
  future_stage_135_independent_implementation_review_eligible: boolean;
  receipt_decrypted_or_read: boolean;
  financial_rows_parsed: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  trading_authorized: boolean;
  implementation_contract: {
    immutable_code_revision: string;
    contract_sha256: string;
    stage_133_validation_sha256: string;
    stage_132_result_sha256: string;
    stage_131_claim_sha256: string;
    receipt_manifest_sha256: string;
    stage_125_specification_sha256: string;
    exact_canonical_snapshot_schema: Record<string, unknown>;
    [key: string]: unknown;
  };
  [key: string]: unknown;
};

export type OpeningPortfolioSnapshotMaterializationImplementationRegistry = {
  schema_version: string;
  policy_version: string;
  registration_endpoint_available: boolean;
  items: Array<{
    candidate: OpeningPortfolioSnapshotMaterializationImplementationCandidate;
    implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration | null;
    registration_eligible: boolean;
    upstream_binding_current: boolean;
    future_stage_135_independent_implementation_review_eligible: boolean;
  }>;
  independently_validated_receipt_count: number;
  registration_eligible_count: number;
  implementation_contract_count: number;
  current_binding_implementation_contract_count: number;
  future_stage_135_independent_implementation_review_eligible_count: number;
  implementation_status: string;
  receipt_decrypted_or_read: boolean;
  financial_rows_parsed: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_present: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  next_gate: string;
  scope: string;
};

export type OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict =
  | "approved_for_future_isolated_materializer_specification_registration"
  | "changes_required_rebuild_materialization_implementation"
  | "rejected_materialization_implementation";

export type OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations = {
  exact_current_stage_51_through_stage_134_binding_confirmed: boolean;
  reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed: boolean;
  implementation_contract_validation_result_claim_receipt_and_specification_hashes_independently_reproduced_confirmed: boolean;
  complete_contract_rebuilt_without_stage_134_builder_confirmed: boolean;
  all_stage_134_registration_confirmations_revalidated_confirmed: boolean;
  input_only_independently_validated_content_addressed_receipt_confirmed: boolean;
  future_decryption_only_in_isolated_ephemeral_memory_confirmed: boolean;
  deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: boolean;
  complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: boolean;
  exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: boolean;
  instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: boolean;
  no_default_manual_or_inferred_financial_values_and_whole_snapshot_failure_confirmed: boolean;
  statement_market_values_informational_and_no_nav_or_performance_confirmed: boolean;
  every_output_row_bound_to_artifact_hash_and_source_locator_with_redaction_confirmed: boolean;
  output_create_once_untrusted_and_separate_validation_and_admission_confirmed: boolean;
  no_key_input_read_decrypt_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  approval_only_opens_future_stage_136_isolated_materializer_specification_registration_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest = {
  expected_implementation_sha256: string;
  expected_implementation_contract_sha256: string;
  expected_stage_133_validation_sha256: string;
  expected_stage_132_result_sha256: string;
  expected_stage_131_claim_sha256: string;
  expected_receipt_manifest_sha256: string;
  expected_stage_125_specification_sha256: string;
  expected_independent_audit_sha256: string;
  verdict: OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  parser_schema_and_completeness_assessment: string;
  decimal_identity_and_provenance_assessment: string;
  failure_separation_and_zero_capability_assessment: string;
  known_limitations: string;
  future_materializer_constraints: string;
  confirmations: OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations;
};

export type OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit = {
  schema_version: string;
  audit_sha256: string;
  implementation_id: string;
  implementation_sha256: string;
  implementation_contract_sha256: string;
  stage_133_validation_sha256: string;
  stage_132_result_sha256: string;
  stage_131_claim_sha256: string;
  receipt_manifest_sha256: string;
  stage_125_specification_sha256: string;
  implementation_record_hash_independently_reproduced: boolean;
  implementation_contract_hash_independently_reproduced: boolean;
  complete_contract_rebuilt_without_stage_134_builder: boolean;
  rebuilt_contract_exactly_matches_record: boolean;
  exact_current_stage_51_through_stage_134_binding_valid: boolean;
  all_stage_134_registration_confirmations_valid: boolean;
  deterministic_adapter_and_resource_contract_valid: boolean;
  complete_financial_sections_and_whole_snapshot_failure_valid: boolean;
  exact_decimal_identity_corporate_action_and_provenance_valid: boolean;
  untrusted_output_validation_admission_separation_valid: boolean;
  all_key_input_parser_financial_model_order_broker_and_trading_authority_closed: boolean;
  mismatch_reasons: string[];
};

export type OpeningPortfolioSnapshotMaterializationImplementationReviewRecord = {
  schema_version: string;
  policy_version: string;
  review_id: string;
  review_sha256: string;
  implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration;
  independent_audit: OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit;
  submitted_at: string;
  reviewer_id: string;
  excluded_prior_actor_ids: string[];
  verdict: OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict;
  rationale: string;
  binding_and_recomputation_assessment: string;
  parser_schema_and_completeness_assessment: string;
  decimal_identity_and_provenance_assessment: string;
  failure_separation_and_zero_capability_assessment: string;
  known_limitations: string;
  future_materializer_constraints: string;
  confirmations: OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations;
  confirmations_complete: boolean;
  reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain: boolean;
  zero_capability_materialization_implementation_independently_approved: boolean;
  future_stage_136_isolated_materializer_specification_registration_eligible: boolean;
  isolated_materializer_specification_registered: boolean;
  decryption_key_or_input_accessed: boolean;
  receipt_decrypted_or_read: boolean;
  parser_artifact_entrypoint_or_runtime_present: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  opening_portfolio_snapshot_admitted: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  model_or_metric_store_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  reward_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry = {
  schema_version: string;
  policy_version: string;
  items: Array<{
    implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration;
    current_independent_audit: OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit;
    review?: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord;
    review_eligible: boolean;
    future_stage_136_isolated_materializer_specification_registration_eligible: boolean;
  }>;
  implementation_count: number;
  review_eligible_count: number;
  reviewed_count: number;
  independently_approved_count: number;
  changes_required_or_rejected_count: number;
  future_stage_136_isolated_materializer_specification_registration_eligible_count: number;
  review_status: string;
  isolated_materializer_specification_registered: boolean;
  decryption_key_or_input_accessed: boolean;
  receipt_decrypted_or_read: boolean;
  parser_artifact_entrypoint_or_runtime_present: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_present: boolean;
  financial_event_allowlist_nonempty: boolean;
  ledger_created: boolean;
  position_or_cash_written: boolean;
  nav_or_performance_written: boolean;
  training_or_rl_feedback_authorized: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  next_gate: string;
  scope: string;
};

export type OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind =
  "ephemeral_deterministic_pdf_csv_json_snapshot_materialization_specification";

export type RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest = {
  expected_stage_135_review_id: string;
  expected_stage_135_review_sha256: string;
  expected_stage_135_independent_audit_sha256: string;
  expected_stage_134_implementation_id: string;
  expected_stage_134_implementation_sha256: string;
  expected_stage_134_implementation_contract_sha256: string;
  expected_stage_133_validation_sha256: string;
  expected_stage_132_result_sha256: string;
  expected_stage_131_claim_sha256: string;
  expected_receipt_manifest_sha256: string;
  expected_stage_125_specification_sha256: string;
  materializer_name: string;
  materializer_kind: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind;
  materializer_spec_revision: string;
  proposed_materializer_code_revision: string;
  proposed_materializer_artifact_sha256: string;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  exact_current_stage_51_through_stage_135_binding_confirmed: boolean;
  registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: boolean;
  implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: boolean;
  proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: boolean;
  all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: boolean;
  future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed: boolean;
  complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed: boolean;
  exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed: boolean;
  future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed: boolean;
  deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: boolean;
  statement_market_values_informational_and_no_nav_or_performance_confirmed: boolean;
  future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: boolean;
  fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: boolean;
  no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: boolean;
  no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: boolean;
  registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed: boolean;
  no_unconfirmed_hari_or_old_wang_logic_claimed: boolean;
};

export type OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract = {
  schema_version: string;
  contract_sha256: string;
  stage_135_implementation_review_id: string;
  stage_135_implementation_review_sha256: string;
  stage_135_independent_audit_sha256: string;
  stage_134_implementation_id: string;
  stage_134_implementation_sha256: string;
  stage_134_implementation_contract_sha256: string;
  stage_133_validation_sha256: string;
  stage_132_result_sha256: string;
  stage_131_claim_sha256: string;
  receipt_manifest_sha256: string;
  stage_125_specification_sha256: string;
  materializer_spec_revision: string;
  proposed_materializer_code_revision: string;
  proposed_materializer_artifact_sha256: string;
  runtime_identity: string;
  runtime_version: string;
  future_input_envelope: string;
  future_output_envelope: string;
  next_gate: string;
  specification_registered: boolean;
  future_materializer_artifact_identity_bound: boolean;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_mount_present: boolean;
  input_read_allowed: boolean;
  data_access_authorized: boolean;
  maximum_parallel_runs: number;
  maximum_memory_mib: number;
  maximum_wall_clock_seconds: number;
  maximum_cpu_millicores: number;
  maximum_process_count: number;
  maximum_output_bytes: number;
};

export type OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord = {
  schema_version: string;
  policy_version: string;
  isolated_materializer_id: string;
  isolated_materializer_spec_sha256: string;
  registered_at: string;
  registered_by: string;
  implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration;
  implementation_review: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord;
  excluded_prior_actor_ids: string[];
  materializer_name: string;
  materializer_kind: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind;
  artifact_reproduction_procedure: string;
  rationale: string;
  known_limitations: string;
  future_input_constraints: string;
  future_output_constraints: string;
  materializer_contract: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract;
  status: string;
  confirmations_complete: boolean;
  first_execution_authorization_review_eligible: boolean;
  first_execution_authorized: boolean;
  input_accessed: boolean;
  receipt_decrypted_or_read: boolean;
  financial_rows_parsed: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_materialized: boolean;
  financial_event_allowlist_nonempty: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
};

export type OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry = {
  schema_version: string;
  policy_version: string;
  eligible_implementations: Array<{
    implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration;
    review: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord;
  }>;
  registration_eligible_count: number;
  materializer_count: number;
  current_binding_materializer_count: number;
  first_execution_authorization_review_eligible_count: number;
  items: Array<{
    materializer: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord;
    approved_review_binding_current: boolean;
    first_execution_authorization_review_eligible: boolean;
  }>;
  materializer_status: string;
  source_artifact_present: boolean;
  executable_artifact_present: boolean;
  callable_entrypoint_present: boolean;
  runtime_instantiated: boolean;
  input_accessed: boolean;
  receipt_decrypted_or_read: boolean;
  financial_rows_parsed: boolean;
  output_candidate_created: boolean;
  opening_portfolio_snapshot_present: boolean;
  financial_event_allowlist_nonempty: boolean;
  order_generation_authorized: boolean;
  broker_access_authorized: boolean;
  trading_authorized: boolean;
  scope: string;
};
