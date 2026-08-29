import { setLocale } from "./i18n";
import { afterEach, describe, expect, test } from "bun:test";
import {
  ApiError,
  createPublicAdminInvite,
  disablePublicAdminInvite,
  getPublicAdminUsage,
  getInvestmentDecisionEvaluation,
  getInvestmentEvidenceReviewQueue,
  getInvestmentDecisionReplay,
  getInvestmentCausalDatasetGovernance,
  getInvestmentCausalTrainingExperiments,
  getInvestmentRewardGovernance,
  getInvestmentShadowProtocolGovernance,
  getInvestmentShadowImplementations,
  reviewInvestmentDecision,
  reviewInvestmentCausalEvidence,
  getInvestmentFinancialEvidenceReviews,
  reviewInvestmentFinancialEvidence,
  reviewInvestmentCausalDatasetGovernance,
  registerInvestmentCausalTrainingExperiment,
  reviewInvestmentRewardGovernance,
  reviewInvestmentShadowProtocolGovernance,
  registerInvestmentShadowImplementation,
  getHistoricalOutcomeLabelers,
  registerHistoricalOutcomeLabeler,
  reviewHistoricalOutcomeLabeler,
  getControlledShadowFirstNaturalForwardCycleClaims,
  claimControlledShadowFirstNaturalForwardCycleOnce,
  getControlledShadowMarketDataAdapterAuthorizations,
  reviewControlledShadowMarketDataAdapterAuthorization,
  getControlledShadowMarketDataReceiptAttempts,
  claimAndReadControlledShadowMarketDataReceiptOnce,
  getControlledShadowMarketDataReceiptValidations,
  validateControlledShadowMarketDataReceiptOnce,
  getControlledShadowMarketDataParserSpecifications,
  registerControlledShadowMarketDataParserSpecificationOnce,
  getControlledShadowMarketDataParserSpecificationReviews,
  reviewControlledShadowMarketDataParserSpecificationOnce,
  getControlledShadowMarketDataParserImplementations,
  registerControlledShadowMarketDataParserImplementationOnce,
  getControlledShadowMarketDataParserImplementationReviews,
  reviewControlledShadowMarketDataParserImplementationOnce,
  getControlledShadowMarketDataParserIsolatedRunners,
  registerControlledShadowMarketDataParserIsolatedRunnerOnce,
  getControlledShadowMarketDataParserFirstExecutionAuthorizations,
  reviewControlledShadowMarketDataParserFirstExecutionAuthorizationOnce,
  getControlledShadowMarketDataParserExecutionAttemptClaims,
  claimControlledShadowMarketDataParserExecutionAttemptOnce,
  getControlledShadowMarketDataParserExecutionAttempts,
  executeControlledShadowMarketDataParserAttemptOnce,
  getControlledShadowMarketDataParserOutputValidations,
  validateControlledShadowMarketDataParserOutputOnce,
  getControlledShadowObservationInputAdmissionReviews,
  reviewControlledShadowObservationInputAdmission,
  getControlledShadowObservationMaterializationSpecifications,
  registerControlledShadowObservationMaterializationSpecification,
  getControlledShadowObservationMaterializationSpecificationReviews,
  reviewControlledShadowObservationMaterializationSpecification,
  getControlledShadowObservationMaterializationImplementations,
  registerControlledShadowObservationMaterializationImplementationOnce,
  getControlledShadowObservationMaterializationImplementationReviews,
  reviewControlledShadowObservationMaterializationImplementationOnce,
  getControlledShadowObservationMaterializationIsolatedRunners,
  registerControlledShadowObservationMaterializationIsolatedRunnerOnce,
  getControlledShadowObservationMaterializationFirstExecutionAuthorizations,
  reviewControlledShadowObservationMaterializationFirstExecutionAuthorizationOnce,
  getControlledShadowObservationMaterializationExecutionAttemptClaims,
  claimControlledShadowObservationMaterializationExecutionAttemptOnce,
  getControlledShadowObservationMaterializationExecutionAttempts,
  executeControlledShadowObservationMaterializationAttemptOnce,
  getControlledShadowObservationMaterializationOutputValidations,
  validateControlledShadowObservationMaterializationOutputOnce,
  getControlledShadowObservationEvidenceAdmissionReviews,
  reviewControlledShadowObservationEvidenceAdmission,
  getControlledShadowObservationLedgerTransitionSpecifications,
  registerControlledShadowObservationLedgerTransitionSpecification,
  getControlledShadowObservationLedgerTransitionSpecificationReviews,
  reviewControlledShadowObservationLedgerTransitionSpecification,
  getControlledShadowObservationLedgerTransitionImplementations,
  registerControlledShadowObservationLedgerTransitionImplementationOnce,
  getControlledShadowObservationLedgerTransitionImplementationReviews,
  reviewControlledShadowObservationLedgerTransitionImplementationOnce,
  getControlledShadowObservationLedgerTransitionIsolatedRunners,
  registerControlledShadowObservationLedgerTransitionIsolatedRunnerOnce,
  getControlledShadowObservationLedgerTransitionFirstExecutionAuthorizations,
  reviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationOnce,
  getControlledShadowObservationLedgerTransitionExecutionAttemptClaims,
  claimControlledShadowObservationLedgerTransitionExecutionAttemptOnce,
  getControlledShadowObservationLedgerTransitionExecutionAttempts,
  executeControlledShadowObservationLedgerTransitionAttemptOnce,
  getControlledShadowObservationLedgerTransitionOutputValidations,
  validateControlledShadowObservationLedgerTransitionOutputOnce,
  getControlledShadowObservationLedgerTransitionCandidateAdmissionReviews,
  reviewControlledShadowObservationLedgerTransitionCandidateAdmission,
  getOpeningPortfolioSnapshotGovernanceSpecifications,
  registerOpeningPortfolioSnapshotGovernanceSpecification,
  getOpeningPortfolioSnapshotGovernanceSpecificationReviews,
  reviewOpeningPortfolioSnapshotGovernanceSpecification,
  getOpeningPortfolioSourceArtifactReceiptImplementations,
  registerOpeningPortfolioSourceArtifactReceiptImplementation,
  getOpeningPortfolioSourceArtifactReceiptImplementationReviews,
  reviewOpeningPortfolioSourceArtifactReceiptImplementation,
  getOpeningPortfolioSourceArtifactReceiptIsolatedReceivers,
  registerOpeningPortfolioSourceArtifactReceiptIsolatedReceiver,
  getOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizations,
  reviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization,
  getOpeningPortfolioSourceArtifactReceiptExecutionAttempts,
  receiveOpeningPortfolioSourceArtifactReceiptAttemptOnce,
  getOpeningPortfolioSourceArtifactReceiptValidations,
  validateOpeningPortfolioSourceArtifactReceiptOnce,
  getOpeningPortfolioSnapshotMaterializationImplementations,
  registerOpeningPortfolioSnapshotMaterializationImplementation,
  getOpeningPortfolioSnapshotMaterializationImplementationReviews,
  reviewOpeningPortfolioSnapshotMaterializationImplementation,
  getOpeningPortfolioSnapshotMaterializationIsolatedMaterializers,
  registerOpeningPortfolioSnapshotMaterializationIsolatedMaterializer,
  screenHistoricalAnchorDiscovery,
  getPublicChatBootstrap,
  getPublicAuthMe,
  getPublicFinanceCalendar,
  getPublicGeneratedFileBlob,
  getPublicCommunity,
  getPublicCommunityResourceBlob,
  getPublicHistory,
  markPublicCommunitySeen,
  publicCommunityResourceDownloadName,
  publicCommunityResourceUrl,
  resetPublicCommunityEdgeDiscoveryForTests,
  resolvePublicCommunityResourceUrl,
  setPublicCommunityEdgeDiscoveryForTests,
  getPublicPushes,
  isUnauthorizedApiError,
  sendPublicChat,
  sendPublicFinanceCalendar,
  openPublicPush,
  publicLogout,
} from "./api";
import {
  FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
  resetApiFetchRetryDelayForTests,
  setApiFetchRetryDelayForTests,
} from "./backend";
import {
  cachedCommunityFeed,
  cachedPublicUser,
  setCachedCommunityFeed,
  setCachedPublicUser,
} from "./public-session-cache";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiFetchRetryDelayForTests();
  resetPublicCommunityEdgeDiscoveryForTests();
  setCachedPublicUser(null);
});

function mockFetch(response: Response) {
  globalThis.fetch = ((() => Promise.resolve(response)) as unknown) as typeof fetch;
}

async function expectApiError(
  action: () => Promise<unknown>,
): Promise<ApiError> {
  try {
    await action();
  } catch (error) {
    expect(error).toBeInstanceOf(ApiError);
    return error as ApiError;
  }
  throw new Error("expected API call to fail");
}

describe("public API errors", () => {
  test("preserves status for auth restore decisions", async () => {
    mockFetch(
      new Response(JSON.stringify({ error: "未登录" }), {
        status: 401,
        statusText: "Unauthorized",
      }),
    );

    const error = await expectApiError(getPublicAuthMe);
    expect(isUnauthorizedApiError(error)).toBe(true);
    expect(error.status).toBe(401);
    expect(error.message).toBe("未登录");
  });

  test("does not classify server errors as logged-out sessions", async () => {
    const error = new ApiError(
      "temporary outage",
      new Response("", { status: 502, statusText: "Bad Gateway" }),
    );

    expect(isUnauthorizedApiError(error)).toBe(false);
  });

  test("rewrites repeated 502 responses to a friendly message", async () => {
    setApiFetchRetryDelayForTests(0);
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.resolve(
        new Response("<html>Bad Gateway</html>", {
          status: 502,
          statusText: "Bad Gateway",
        }),
      );
    }) as unknown) as typeof fetch;

    const error = await expectApiError(getPublicAuthMe);

    expect(calls).toBe(2);
    expect(error.status).toBe(502);
    expect(error.message).toBe(FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE);
  });

  test("streaming public chat uses the same friendly backend failure", async () => {
    setApiFetchRetryDelayForTests(0);
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.resolve(
        new Response("nginx gateway failure\nwith stack details", {
          status: 503,
          statusText: "Service Unavailable",
        }),
      );
    }) as unknown) as typeof fetch;

    const error = await expectApiError(() => sendPublicChat("hello"));

    expect(calls).toBe(1);
    expect(error.status).toBe(503);
    expect(error.message).toBe(FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE);
  });

  test("streaming public chat never replays a POST after transport failure", async () => {
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.reject(new TypeError("Failed to fetch"));
    }) as unknown) as typeof fetch;

    await expect(sendPublicChat("hello")).rejects.toThrow(
      FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
    );
    expect(calls).toBe(1);
  });
});

describe("public earnings workflow API", () => {
  test("sends a structured workflow envelope instead of relying on prompt text", async () => {
    let requestedBody = "";
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      requestedBody = String(init?.body ?? "");
      return Promise.resolve(new Response("event: done\ndata: {}\n\n"));
    }) as unknown as typeof fetch;

    // The reported language is per-device state, so pin it rather than
    // inheriting whatever an earlier test file left behind.
    setLocale("zh");
    await sendPublicChat(
      "请为 NVDA 生成财报前瞻，并完成证据核验和可分享 PDF。",
      [],
      undefined,
      { kind: "preview", company: "NVDA" },
    );

    expect(JSON.parse(requestedBody)).toEqual({
      message: "请为 NVDA 生成财报前瞻，并完成证据核验和可分享 PDF。",
      attachments: [],
      earnings_workflow: { kind: "preview", company: "NVDA" },
      // The server must be told which language the user is reading, rather
      // than inferring it from the conversation.
      language: "zh",
    });
  });
});

describe("public chat bootstrap API", () => {
  test("loads auth and history through one startup request", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            user: { user_id: "web-user-1", remaining_today: 9, daily_limit: 10 },
            messages: [{ role: "user", content: "hello" }],
            history_start: 42,
            next_before: 42,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicChatBootstrap();

    expect(requestedUrl).toContain("/api/public/bootstrap");
    expect(payload.user.user_id).toBe("web-user-1");
    expect(payload.messages?.[0]?.content).toBe("hello");
    expect(payload.history_start).toBe(42);
    expect(payload.next_before).toBe(42);
    expect(requestedInit?.cache).toBe("no-store");
    expect(cachedPublicUser()?.user_id).toBe("web-user-1");
  });

  test("logout clears route caches before the request settles", async () => {
    setCachedPublicUser({ user_id: "old-user" } as never);
    setCachedCommunityFeed([{ id: "old-content" }]);
    let release!: () => void;
    globalThis.fetch = ((() =>
      new Promise<Response>((resolve) => {
        release = () =>
          resolve(
            new Response(JSON.stringify({ ok: true }), {
              headers: { "content-type": "application/json" },
            }),
          );
      })) as unknown) as typeof fetch;

    const pending = publicLogout();
    expect(cachedPublicUser()).toBeNull();
    expect(cachedCommunityFeed()).toBeNull();
    release();
    await pending;
  });

  test("requests the previous history page with a stable cursor", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({ messages: [], history_start: 20, next_before: 20 }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicHistory(40);

    expect(requestedUrl).toContain("/api/public/history?limit=20&before=40");
    expect(payload.history_start).toBe(20);
    expect(requestedInit?.cache).toBe("no-store");
  });
});

describe("public administrator whitelist API", () => {
  test("loads the administrator usage report without caching", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            generated_at: "2026-08-02T12:00:00+08:00",
            period_start: "2026-07-20",
            period_end: "2026-08-02",
            summary: {
              today: "2026-08-02",
              today_active_users: 2,
              today_question_count: 5,
              today_delivered_push_count: 3,
              last_week_same_day_active_users: 3,
              active_user_change: -1,
              leading_decline_question_delta: 2,
              text: "今日 HONE 总使用人数 2 人",
            },
            rows: [],
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const result = await getPublicAdminUsage(30);

    expect(requestedUrl).toContain("/api/public/admin/usage?days=30");
    expect(requestedInit?.cache).toBe("no-store");
    expect(result.period_days).toBe(14);
    expect(result.summary.today_question_count).toBe(5);
  });

  test("creates through a one-shot POST with the administrator action marker", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    let calls = 0;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      calls += 1;
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            invite: {
              user_id: "web-user-2",
              phone_number: "13900000000",
              created_at: "2026-07-31T10:00:00+08:00",
              enabled: true,
              can_disable: true,
            },
            daily_create_limit: 5,
            created_today: 1,
            remaining_today: 4,
            cleared_session_count: 0,
            message: "已加入会员白名单",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const result = await createPublicAdminInvite("13900000000");
    const headers = new Headers(requestedInit?.headers);

    expect(calls).toBe(1);
    expect(requestedUrl).toContain("/api/public/admin/invites");
    expect(requestedInit?.method).toBe("POST");
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
    expect(result.remaining_today).toBe(4);
  });

  test("disables through a one-shot marked POST to the exact user", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            invite: {
              user_id: "web-user/member",
              phone_number: "13900000000",
              created_at: "2026-07-31T10:00:00+08:00",
              enabled: false,
              can_disable: false,
            },
            daily_create_limit: 5,
            created_today: 1,
            remaining_today: 4,
            cleared_session_count: 1,
            message: "已禁用会员白名单",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await disablePublicAdminInvite("web-user/member");
    const headers = new Headers(requestedInit?.headers);

    expect(requestedUrl).toContain(
      "/api/public/admin/invites/web-user%2Fmember/disable",
    );
    expect(requestedInit?.method).toBe("POST");
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
  });

  test("keeps the server daily-limit explanation", async () => {
    mockFetch(
      new Response(
        JSON.stringify({ error: "今日新增白名单已达到 5 人上限" }),
        {
          status: 429,
          statusText: "Too Many Requests",
          headers: { "content-type": "application/json" },
        },
      ),
    );

    const error = await expectApiError(() =>
      createPublicAdminInvite("13900000000"),
    );
    expect(error.status).toBe(429);
    expect(error.message).toBe("今日新增白名单已达到 5 人上限");
  });
});

describe("administrator investment decision API", () => {
  test("screens one hash-bound transcript hit without creating a candidate", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-historical-anchor-discovery-screening-v2-correction-chain",
        screening_id: "screening-1",
        suggestion_id: "suggestion/1",
        verdict: "continue_candidate_review",
        candidate_created: false,
        speaker_identity_confirmed: false,
        investment_logic_confirmed: false,
        decision_training_eligible: false,
        trading_authorized: false,
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    const record = await screenHistoricalAnchorDiscovery("suggestion/1", {
      expected_source_sha256: "a".repeat(64),
      verdict: "continue_candidate_review",
    });
    expect(record.candidate_created).toBe(false);
    expect(requestedUrl).toContain("/historical-anchor-discovery/suggestion%2F1/screening");
    expect(requestedInit?.method).toBe("POST");
    expect(new Headers(requestedInit?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requestedInit?.body))).toEqual({
      expected_source_sha256: "a".repeat(64),
      verdict: "continue_candidate_review",
    });
  });

  test("appends a screening correction bound to the current immutable tip", async () => {
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((_url: RequestInfo | URL, init?: RequestInit) => {
      requestedInit = init;
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-historical-anchor-discovery-screening-v2-correction-chain",
        screening_id: "screening-2",
        previous_screening_id: "screening-1",
        suggestion_id: "suggestion-1",
        verdict: "continue_candidate_review",
        correction_reason: "结合前后原文后确认是本人仓位动作。",
        candidate_created: false,
        speaker_identity_confirmed: false,
        investment_logic_confirmed: false,
        decision_training_eligible: false,
        trading_authorized: false,
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    const record = await screenHistoricalAnchorDiscovery("suggestion-1", {
      expected_source_sha256: "a".repeat(64),
      expected_screening_id: "screening-1",
      verdict: "continue_candidate_review",
      correction_reason: "结合前后原文后确认是本人仓位动作。",
    });
    expect(record.previous_screening_id).toBe("screening-1");
    expect(JSON.parse(String(requestedInit?.body))).toEqual({
      expected_source_sha256: "a".repeat(64),
      expected_screening_id: "screening-1",
      verdict: "continue_candidate_review",
      correction_reason: "结合前后原文后确认是本人仓位动作。",
    });
  });

  test("reads evaluation and symbol replay without caching", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      const evaluation = String(url).includes("/evaluation");
      return Promise.resolve(
        new Response(
          JSON.stringify(
            evaluation
              ? {
                  schema_version: "hone-investment-offline-evaluation-v1",
                  generated_at: "2026-08-13T00:00:00Z",
                  reward_status: "unconfigured",
                  sample_count: 0,
                  review: { pending: 0, accepted: 0, corrected: 0, rejected: 0, review_rate_percent: 0 },
                  errors: [],
                  horizons: [],
                  action_horizons: [],
                  evidence_gate: {
                    status: "insufficient_evidence",
                    minimum_250_session_samples: 100,
                    observed_250_session_samples: 0,
                    minimum_review_rate_percent: 80,
                    observed_review_rate_percent: 0,
                    reasons: [],
                    scope: "不允许实盘",
                  },
                }
              : {
                  schema_version: "hone-investment-training-sample-v1",
                  symbol: "BRK.B",
                  sample_count: 0,
                  quarantined_sample_count: 0,
                  quarantine_warnings: [],
                  samples: [],
                },
          ),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getInvestmentDecisionEvaluation();
    await getInvestmentDecisionReplay("brk.b", 999);
    await getInvestmentEvidenceReviewQueue({
      symbol: "sndk",
      status: "rejected",
      kind: "computed_comparison",
      selection: "full_queue",
      limit: 999,
    });
    expect(requests[0].url).toContain(
      "/api/public/admin/investment-decisions/evaluation",
    );
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(
      "/api/public/admin/investment-decisions/replay/BRK.B?limit=500",
    );
    expect(requests[1].init?.cache).toBe("no-store");
    expect(requests[2].url).toContain(
      "/api/public/admin/investment-decisions/review-queue?symbol=SNDK&status=rejected&kind=computed_comparison&selection=full_queue&limit=500",
    );
    expect(requests[2].init?.cache).toBe("no-store");
  });

  test("submits review once with the administrator mutation marker", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    let calls = 0;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      calls += 1;
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            schema_version: "hone-investment-decision-review-v1",
            review_id: "review-2",
            sample_id: "SNDK/unsafe",
            symbol: "SNDK",
            submitted_at: "2026-08-13T00:00:00Z",
            review: {
              status: "accepted",
              review_id: "review-2",
              thesis_verdict: "supported",
              error_attributions: [],
            },
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await reviewInvestmentDecision("sndk", "SNDK/unsafe", {
      expected_review_id: "review-1",
      status: "accepted",
      thesis_verdict: "supported",
      error_attributions: [],
    });
    const headers = new Headers(requestedInit?.headers);
    expect(calls).toBe(1);
    expect(requestedUrl).toContain(
      "/review/SNDK/SNDK%2Funsafe",
    );
    expect(requestedInit?.method).toBe("POST");
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
  });

  test("submits one causal label without changing the thesis review", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-causal-evidence-review-v3-source-verified-distilled",
        review_id: "causal-2",
        previous_review_id: "causal-1",
        sample_id: "SNDK/sample",
        symbol: "SNDK",
        submitted_at: "2026-08-13T00:00:00Z",
        reviewer_id: "admin",
        driver_id: "storage-demand",
        observation_id: "cloud-asp",
        verdict: "accepted",
        effect: "supports",
        explanation: "一手经营口径支持需求驱动。",
        thesis_review_unchanged: true,
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    const record = await reviewInvestmentCausalEvidence("sndk", "SNDK/sample", {
      expected_review_id: "causal-1",
      driver_id: "storage-demand",
      observation_id: "cloud-asp",
      verdict: "accepted",
      effect: "supports",
      explanation: "一手经营口径支持需求驱动。",
      verbatim_judgment: "这条公司原始经营数据支持需求驱动。",
      applicability_boundary: "仅限公司定义未变化的可比期间。",
      falsifier: "后续订单、收入与价格同时转弱。",
      speaker_confirmation: "old_wang_confirmed",
      source_verification: "verified_against_source",
      source_verification_note: "已打开公司原文并核对数值、期间、单位和上下文。",
      old_wang_confirmation_attested: true,
    });
    expect(record.thesis_review_unchanged).toBe(true);
    expect(requestedUrl).toContain("/causal-review/SNDK/SNDK%2Fsample");
    expect(requestedInit?.method).toBe("POST");
    expect(new Headers(requestedInit?.headers).get("x-hone-admin-action")).toBe("whitelist");
  });

  test("reads and reviews one fingerprinted SEC financial projection", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-sec-financial-evidence-review-v1",
        policy_version: "hone-sec-financial-rating-admission-v1",
        generated_at: "2026-08-13T00:00:00Z",
        summary: {
          observed: 1,
          pending: 0,
          approved_for_rating: 1,
          changes_requested: 0,
          rejected: 0,
          stale_after_evidence_change: 0,
        },
        candidates: [],
        scope: "只确认财务质量",
        training_authorized: false,
        reward_authorized: false,
        portfolio_action_authorized: false,
        shadow_portfolio_authorized: false,
        trade_authorized: false,
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getInvestmentFinancialEvidenceReviews("sndk");
    await getInvestmentFinancialEvidenceReviews({ selection: "active_batch", limit: 5 });
    await reviewInvestmentFinancialEvidence("sndk", {
      expected_review_id: "review-1",
      expected_evidence_fingerprint_sha256: "a".repeat(64),
      verdict: "approved_for_rating",
      rationale: "已逐项核对官方财报、期间和计算公式",
      confirmations: {
        official_filings_opened: true,
        identity_periods_and_units_verified: true,
        calculations_recomputed: true,
        corporate_actions_and_restatements_checked: true,
        quality_warnings_resolved: true,
        no_unresolved_material_issue: true,
      },
    });
    expect(requests[0].url).toContain("financial-evidence-reviews?symbol=SNDK");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain("selection=active_batch");
    expect(requests[1].url).toContain("limit=5");
    expect(requests[2].url).toContain("financial-evidence-reviews/SNDK");
    expect(requests[2].init?.method).toBe("POST");
    expect(new Headers(requests[2].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[2].init?.body))).toMatchObject({
      expected_evidence_fingerprint_sha256: "a".repeat(64),
      verdict: "approved_for_rating",
    });
  });

  test("reads and reviews the immutable reward-governance contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(
        new Response(
          JSON.stringify({
            schema_version: "hone-reward-governance-review-v1",
            design_version: "hone-reward-design-proposal-v1",
            proposal_sha256: "a".repeat(64),
            evidence_gate_status: "insufficient_evidence",
            reward_computation_enabled: false,
            shadow_portfolio_authorized: false,
            trading_authorized: false,
            scope: "只允许离线治理审查",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getInvestmentRewardGovernance();
    await reviewInvestmentRewardGovernance({
      design_version: "hone-reward-design-proposal-v1",
      proposal_sha256: "a".repeat(64),
      verdict: "changes_requested",
      rationale: "需要进一步拆分产业判断和短期价格结果。",
    });
    expect(requests[0].url).toContain("/reward-governance");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    const headers = new Headers(requests[1].init?.headers);
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      verdict: "changes_requested",
    });
  });

  test("reads and reviews the immutable shadow-protocol governance contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(
        new Response(
          JSON.stringify({
            schema_version: "hone-shadow-protocol-governance-review-v1",
            policy_version: "hone-shadow-policy-v1",
            protocol_sha256: "b".repeat(64),
            review_requirements: [],
            evidence_gate_status: "insufficient_evidence",
            reward_governance_status: "not_reviewed",
            future_shadow_implementation_registration_allowed: false,
            shadow_ledger_enabled: false,
            shadow_portfolio_authorized: false,
            trading_authorized: false,
            broker_connected: false,
            scope: "只审批未来只读影子实现协议",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getInvestmentShadowProtocolGovernance();
    await reviewInvestmentShadowProtocolGovernance({
      policy_version: "hone-shadow-policy-v1",
      protocol_sha256: "b".repeat(64),
      verdict: "changes_requested",
      rationale: "需要进一步明确组合层证伪退出。",
    });
    expect(requests[0].url).toContain("/shadow-protocol-governance");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    const headers = new Headers(requests[1].init?.headers);
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      verdict: "changes_requested",
    });
  });

  test("reads and registers but does not start a shadow implementation", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-shadow-implementation-registry-v1",
        sandbox_policy_version: "hone-shadow-implementation-sandbox-v1",
        policy_version: "hone-shadow-policy-v1",
        protocol_sha256: "b".repeat(64),
        current_shadow_review_id: "shadow-review-1",
        current_reward_review_id: "reward-review-1",
        registration_allowed: true,
        allowed_implementation_kinds: ["deterministic_replay_specification"],
        implementations: [],
        shadow_ledger_enabled: false,
        shadow_run_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_connected: false,
        trading_authorized: false,
        scope: "只登记，不启动",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getInvestmentShadowImplementations();
    await registerInvestmentShadowImplementation({
      expected_shadow_review_id: "shadow-review-1",
      expected_reward_review_id: "reward-review-1",
      policy_version: "hone-shadow-policy-v1",
      protocol_sha256: "b".repeat(64),
      implementation_name: "确定性影子重放规范",
      implementation_kind: "deterministic_replay_specification",
      code_revision: "oldwang@abc123",
    });
    expect(requests[0].url).toContain("/shadow-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      implementation_kind: "deterministic_replay_specification",
      code_revision: "oldwang@abc123",
    });
  });

  test("registers and reviews a historical labeler without running it", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-historical-outcome-labeler-registry-v1",
        sandbox_policy_version: "hone-historical-outcome-labeler-sandbox-v1",
        protocol_version: "protocol-v1",
        protocol_sha256: "c".repeat(64),
        current_governance_review_id: "governance-review-1",
        registration_allowed: true,
        allowed_implementation_kinds: ["deterministic_common_session_adjusted_close"],
        implementations: [],
        current_binding_implementation_count: 0,
        reviewed_implementation_count: 0,
        labeler_review_status: "waiting_for_implementation_registration",
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "只登记与复核，不运行",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getHistoricalOutcomeLabelers();
    await registerHistoricalOutcomeLabeler({
      expected_governance_review_id: "governance-review-1",
      protocol_version: "protocol-v1",
      protocol_sha256: "c".repeat(64),
      implementation_name: "共同交易日确定性标签器",
      implementation_kind: "deterministic_common_session_adjusted_close",
      code_revision: "oldwang@abc123",
    });
    await reviewHistoricalOutcomeLabeler("labeler-1", {
      verdict: "changes_requested",
      rationale: "补充缺失行情失败关闭测试。",
      implementation_fingerprint_confirmed: false,
      protocol_binding_confirmed: false,
      adjusted_close_and_common_sessions_confirmed: false,
      deterministic_replay_confirmed: false,
      future_isolation_confirmed: false,
      missing_data_fail_closed_confirmed: false,
      no_network_or_production_writes_confirmed: false,
    });

    expect(requests[0].url).toContain("/historical-outcome-labelers");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      implementation_kind: "deterministic_common_session_adjusted_close",
      code_revision: "oldwang@abc123",
    });
    expect(requests[2].url).toContain("/historical-outcome-labelers/labeler-1/review");
    expect(requests[2].init?.method).toBe("POST");
    expect(new Headers(requests[2].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
  });

  test("reads and reviews the immutable causal-dataset governance contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(
        new Response(
          JSON.stringify({
            schema_version: "hone-causal-dataset-governance-review-v3",
            dataset: {
              policy_version: "hone-causal-training-dataset-v3-company-source-identity-component-isolated",
              status: "insufficient_human_labels",
              dataset_fingerprint_sha256: "a".repeat(64),
            },
            current_dataset_approved: false,
            offline_experiment_registration_allowed: false,
            offline_training_run_authorized: false,
            preference_learning_authorized: false,
            reinforcement_learning_authorized: false,
            deployment_authorized: false,
            trading_authorized: false,
            scope: "只允许数据集治理审查",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getInvestmentCausalDatasetGovernance();
    await reviewInvestmentCausalDatasetGovernance({
      dataset_policy_version: "hone-causal-training-dataset-v3-company-source-identity-component-isolated",
      dataset_fingerprint_sha256: "a".repeat(64),
      verdict: "changes_requested",
      rationale: "人工标签不足，继续积累。",
    });
    expect(requests[0].url).toContain("/causal-dataset-governance");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      verdict: "changes_requested",
    });
  });

  test("reads and registers but does not run a causal training experiment", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-causal-training-experiment-v1",
        sandbox_policy_version: "hone-offline-experiment-sandbox-v1",
        dataset_policy_version: "hone-causal-training-dataset-v3-company-source-identity-component-isolated",
        dataset_fingerprint_sha256: "a".repeat(64),
        current_dataset_review_id: "review-1",
        registration_allowed: true,
        allowed_algorithms: ["frozen_prompt_baseline", "supervised_causal_classifier"],
        experiments: [],
        offline_training_run_authorized: false,
        preference_learning_authorized: false,
        reinforcement_learning_authorized: false,
        deployment_authorized: false,
        trading_authorized: false,
        scope: "只登记，不执行",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getInvestmentCausalTrainingExperiments();
    await registerInvestmentCausalTrainingExperiment({
      expected_dataset_review_id: "review-1",
      dataset_policy_version: "hone-causal-training-dataset-v3-company-source-identity-component-isolated",
      dataset_fingerprint_sha256: "a".repeat(64),
      experiment_name: "冻结基线",
      algorithm: "frozen_prompt_baseline",
      base_model_id: "hone/base-model",
      base_model_version: "2026-08-13",
      random_seed: 42,
      max_epochs: 0,
    });
    expect(requests[0].url).toContain("/causal-training-experiments");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      algorithm: "frozen_prompt_baseline",
      max_epochs: 0,
    });
  });

  test("claims Stage 91 once without exposing a market-data execution route", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-first-natural-forward-cycle-claim-registry-v1",
        policy_version: "hone-controlled-shadow-first-natural-forward-cycle-claim-v1-create-once-consumes-authorization",
        eligible_authorizations: [],
        claims: [],
        authorization_candidate_count: 0,
        claim_eligible_count: 0,
        claim_count: 0,
        authorization_consumed_count: 0,
        waiting_for_separate_market_data_adapter_authorization_count: 0,
        claim_status: "waiting_for_active_stage_90_authorization",
        calendar_window_resolved: false,
        calendar_read_authorized: false,
        market_data_adapter_authorized: false,
        market_data_access_authorized: false,
        execution_endpoint_available: false,
        runtime_instantiated: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "claim only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowFirstNaturalForwardCycleClaims();
    await claimControlledShadowFirstNaturalForwardCycleOnce("a".repeat(32), {
      expected_authorization_review_sha256: "b".repeat(64),
      expected_validation_sha256: "c".repeat(64),
      expected_stage_88_attempt_id: "d".repeat(32),
      expected_stage_88_claim_sha256: "e".repeat(64),
      expected_stage_88_result_sha256: "f".repeat(64),
      expected_stage_88_output_sha256: "1".repeat(64),
      expected_initialization_manifest_sha256: "2".repeat(64),
      claim_reason: "建立不可执行首周期任务。",
      exact_stage_51_through_stage_90_binding_confirmed: true,
      claimant_independence_from_stage_90_and_complete_prior_chain_confirmed: true,
      authorization_current_unexpired_and_single_use_confirmed: true,
      claim_first_before_calendar_or_market_data_confirmed: true,
      separate_read_only_market_data_adapter_authorization_required_confirmed: true,
      natural_forward_only_no_backfill_and_create_once_confirmed: true,
      no_runtime_observation_ledger_position_or_performance_confirmed: true,
      no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("first-natural-forward-cycle-claims");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`forward-observation-first-natural-forward-cycle-claims/${"a".repeat(32)}/claim-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      claim_first_before_calendar_or_market_data_confirmed: true,
      separate_read_only_market_data_adapter_authorization_required_confirmed: true,
    });
  });

  test("reviews Stage 92 without making a market-data request", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-adapter-authorization-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-adapter-authorization-v1",
        adapter_specification: {},
        items: [],
        claimed_task_count: 0,
        review_eligible_count: 0,
        reviewed_count: 0,
        approved_count: 0,
        rejected_count: 0,
        active_authorization_count: 0,
        future_claim_first_read_only_market_data_receipt_eligible_count: 0,
        authorization_status: "waiting_for_stage_91_claim",
        market_data_request_made: false,
        market_data_accessed: false,
        trading_authorized: false,
        scope: "contract review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataAdapterAuthorizations();
    await reviewControlledShadowMarketDataAdapterAuthorization("a".repeat(32), {
      expected_cycle_claim_sha256: "b".repeat(64),
      expected_authorization_review_sha256: "c".repeat(64),
      expected_validation_sha256: "d".repeat(64),
      expected_initialization_manifest_sha256: "e".repeat(64),
      verdict: "approved_for_future_claim_first_read_only_market_data_receipt",
      rationale: "仅批准固定合同供后续独立收据阶段使用。",
      source_allowlist_assessment: "来源和路径均固定。",
      credential_and_request_minimization_assessment: "凭据仅服务端注入且不得持久化。",
      content_addressing_and_custody_assessment: "请求响应来源均需哈希并保留原文。",
      known_limitations: "本阶段没有验证来源可用性。",
      future_receipt_constraints: "后续必须 claim-first 且 create-once。",
      exact_stage_51_through_stage_91_binding_confirmed: true,
      reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: true,
      fixed_get_only_https_origin_and_path_allowlist_confirmed: true,
      calendar_security_spy_price_dividend_split_only_confirmed: true,
      exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: true,
      credentials_never_persisted_forwarded_or_returned_confirmed: true,
      request_response_source_and_retrieval_time_hashes_required_confirmed: true,
      natural_forward_only_no_backfill_or_history_rewrite_confirmed: true,
      approval_only_opens_future_claim_first_read_only_receipt_confirmed: true,
      no_data_request_calendar_resolution_or_runtime_started_confirmed: true,
      no_observation_ledger_position_performance_or_model_metric_write_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-adapter-authorizations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-adapter-authorizations/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      approval_only_opens_future_claim_first_read_only_receipt_confirmed: true,
      no_data_request_calendar_resolution_or_runtime_started_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
    });
  });

  test("claims Stage 93 through the dedicated single-read endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-receipt-attempt-registry-v2",
        policy_version: "hone-controlled-shadow-market-data-receipt-v2-explicit-actions-claim-first-single-use-untrusted-raw",
        invocation_endpoint_available: true,
        eligible_authorizations: [], items: [], invocation_eligible_authorization_count: 0,
        claim_count: 0, completed_untrusted_receipt_count: 0,
        failed_authorization_consumed_count: 0, interrupted_authorization_consumed_count: 0,
        independent_validation_eligible_count: 0, receipt_status: "waiting", scope: "untrusted raw only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataReceiptAttempts();
    await claimAndReadControlledShadowMarketDataReceiptOnce("a".repeat(32), {
      expected_adapter_authorization_sha256: "b".repeat(64),
      expected_cycle_claim_sha256: "c".repeat(64),
      expected_adapter_spec_sha256: "d".repeat(64),
      expected_subject_symbol_set_sha256: "e".repeat(64),
      expected_time_window_sha256: "f".repeat(64),
      execution_reason: "读取唯一自然前向窗口原始载荷。",
      claim_first_single_use_and_failure_consumes_authorization_confirmed: true,
      exact_stage_51_through_stage_92_binding_confirmed: true,
      executor_independent_from_stage_92_and_complete_prior_chain_confirmed: true,
      fixed_get_https_path_and_query_allowlist_confirmed: true,
      server_derived_subject_symbols_and_spy_only_confirmed: true,
      natural_forward_window_content_addressed_no_backfill_confirmed: true,
      credential_redacted_not_persisted_returned_or_logged_confirmed: true,
      raw_payload_hashes_timestamps_and_custody_retained_confirmed: true,
      receipt_untrusted_pending_independent_validation_confirmed: true,
      no_parsed_calendar_observation_ledger_position_performance_or_model_metric_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-receipt-attempts");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-receipt-attempts/${"a".repeat(32)}/claim-and-read-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.stringify(JSON.parse(String(requests[1].init?.body)))).not.toContain("apikey");
  });

  test("validates Stage 94 through the dedicated chain-external endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-receipt-validation-registry-v2",
        policy_version: "hone-controlled-shadow-market-data-receipt-independent-validation-v2-explicit-actions-no-parsing",
        validation_endpoint_available: true, candidates: [], validations: [],
        completed_untrusted_receipt_count: 0, pending_independent_validation_count: 0,
        independently_validated_receipt_count: 0, failed_independent_validation_count: 0,
        future_market_data_parser_review_eligible_count: 0, validation_status: "waiting",
        calendar_window_resolved: false, parsed_market_rows_created: false,
        forward_observation_started: false, ledger_created: false, position_written: false,
        performance_metric_written: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "integrity only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataReceiptValidations();
    await validateControlledShadowMarketDataReceiptOnce("a".repeat(32), {
      expected_claim_sha256: "1".repeat(64), expected_result_sha256: "2".repeat(64),
      expected_receipt_sha256: "3".repeat(64), expected_adapter_authorization_sha256: "4".repeat(64),
      expected_cycle_claim_sha256: "5".repeat(64), expected_adapter_spec_sha256: "6".repeat(64),
      expected_subject_symbol_set_sha256: "7".repeat(64), expected_time_window_sha256: "8".repeat(64),
      expected_canonical_request_set_sha256: "9".repeat(64),
      independent_chain_reopen_and_fingerprint_recomputation_confirmed: true,
      validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed: true,
      claim_first_single_terminal_result_and_no_replay_confirmed: true,
      redacted_fixed_request_set_independently_reconstructed_confirmed: true,
      every_raw_payload_reopened_size_and_sha256_recomputed_confirmed: true,
      source_identity_timestamp_and_content_addressed_custody_confirmed: true,
      credential_absence_from_persisted_artifacts_confirmed: true,
      successful_http_envelope_only_not_market_truth_confirmed: true,
      validation_does_not_parse_calendar_or_market_rows_confirmed: true,
      no_runtime_observation_ledger_position_performance_or_model_metric_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-receipt-validations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-receipt-validations/${"a".repeat(32)}/validate-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      validation_does_not_parse_calendar_or_market_rows_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
    });
  });

  test("registers Stage 95 through the dedicated zero-capability specification endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-specification-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-specification-create-once-v1-zero-capability",
        registration_endpoint_available: true, candidates: [], registrations: [],
        independently_validated_receipt_count: 0, registration_eligible_count: 0,
        parser_specification_registered_count: 0,
        future_chain_external_specification_review_eligible_count: 0,
        parser_specification_status: "waiting", parser_implementation_present: false,
        parsed_calendar_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "specification only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserSpecifications();
    await registerControlledShadowMarketDataParserSpecificationOnce("a".repeat(32), {
      expected_validation_sha256: "1".repeat(64), expected_receipt_sha256: "2".repeat(64),
      expected_claim_sha256: "3".repeat(64), expected_result_sha256: "4".repeat(64),
      expected_adapter_authorization_sha256: "5".repeat(64), expected_adapter_spec_sha256: "6".repeat(64),
      expected_canonical_request_set_sha256: "7".repeat(64), registration_reason: "freeze parser spec",
      known_limitations: "synthetic vectors do not prove provider semantics",
      future_review_constraints: "independent review before implementation",
      exact_stage_51_through_stage_94_binding_confirmed: true,
      registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed: true,
      explicit_price_dividend_split_and_official_calendar_sources_confirmed: true,
      strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed: true,
      duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed: true,
      no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed: true,
      spy_calendar_sync_and_cross_source_reconciliation_required_confirmed: true,
      synthetic_vectors_contain_no_market_fact_or_credential_confirmed: true,
      specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: true,
      no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed: true,
      no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      future_chain_external_specification_review_required_before_implementation_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-specifications");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-specifications/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
    });
  });

  test("reviews Stage 96 through the dedicated chain-external specification endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-specification-review-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-specification-chain-external-review-v1-no-parser",
        review_endpoint_available: true, items: [], parser_specification_registered_count: 0,
        review_eligible_count: 0, reviewed_count: 0, independently_approved_count: 0,
        changes_required_or_rejected_count: 0,
        future_zero_capability_parser_implementation_registration_eligible_count: 0,
        review_status: "waiting", parser_implementation_registered: false,
        parser_implementation_present: false, raw_payload_accessed: false,
        parsed_calendar_rows_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserSpecificationReviews();
    await reviewControlledShadowMarketDataParserSpecificationOnce("a".repeat(32), {
      expected_registration_sha256: "1".repeat(64), expected_parser_specification_sha256: "2".repeat(64),
      expected_validation_sha256: "3".repeat(64), expected_receipt_sha256: "4".repeat(64),
      expected_claim_sha256: "5".repeat(64), expected_result_sha256: "6".repeat(64),
      expected_adapter_authorization_sha256: "7".repeat(64), expected_adapter_spec_sha256: "8".repeat(64),
      expected_canonical_request_set_sha256: "9".repeat(64),
      verdict: "approved_for_future_zero_capability_parser_implementation_registration",
      rationale: "independent reconstruction passed", source_contract_assessment: "explicit sources match",
      schema_and_numeric_assessment: "strict bounded schema", calendar_and_reconciliation_assessment: "calendar aligned",
      synthetic_vector_assessment: "all eight rebuilt", failure_and_missing_data_assessment: "fail closed",
      known_limitations: "provider semantics remain unverified", future_implementation_constraints: "zero capability only",
      exact_stage_51_through_stage_95_binding_confirmed: true,
      reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed: true,
      independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed: true,
      independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: true,
      strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: true,
      duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: true,
      separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: true,
      spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: true,
      source_available_at_remains_unverified_until_separate_review_confirmed: true,
      specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed: true,
      approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-specification-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-specification-reviews/${"a".repeat(32)}/review-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: true,
      approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: true,
    });
  });

  test("registers Stage 97 through the zero-capability implementation-contract endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-implementation-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-implementation-create-once-v1-zero-capability",
        registration_endpoint_available: true, items: [], independently_approved_specification_count: 0,
        registration_eligible_count: 0, implementation_contract_count: 0,
        current_binding_implementation_contract_count: 0,
        independent_implementation_review_eligible_count: 0, implementation_status: "waiting",
        source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false, raw_payload_accessed: false,
        parsed_calendar_rows_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "contract only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserImplementations();
    await registerControlledShadowMarketDataParserImplementationOnce("a".repeat(32), {
      expected_specification_review_id: "a".repeat(32), expected_specification_review_sha256: "1".repeat(64),
      expected_registration_id: "b".repeat(32), expected_registration_sha256: "2".repeat(64),
      expected_parser_specification_sha256: "3".repeat(64), expected_validation_sha256: "4".repeat(64),
      expected_receipt_sha256: "5".repeat(64), expected_claim_sha256: "6".repeat(64),
      expected_result_sha256: "7".repeat(64), expected_adapter_authorization_sha256: "8".repeat(64),
      expected_adapter_spec_sha256: "9".repeat(64), expected_canonical_request_set_sha256: "0".repeat(64),
      implementation_name: "strict parser contract", immutable_code_revision: "sha256:immutable",
      implementation_description: "zero capability contract only", deterministic_parser_semantics: "pure deterministic functions",
      source_schema_and_numeric_semantics: "strict schemas and bounded finite decimals",
      calendar_action_and_reconciliation_semantics: "NYSE, subject, SPY and explicit actions reconciled",
      error_and_missing_data_semantics: "all malformed or missing inputs fail closed",
      known_limitations: "source available-at remains unverified",
      future_review_constraints: "Stage 98 independent review before runner",
      exact_stage_51_through_stage_96_binding_confirmed: true,
      registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_review_registration_and_specification_confirmed: true,
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: true,
      strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: true,
      duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
      spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed: true,
      all_eight_synthetic_vector_hashes_bound_confirmed: true,
      source_available_at_remains_unverified_until_separate_review_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-implementations/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_confirmed: true,
    });
  });

  test("reviews Stage 98 through the chain-external implementation-review endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-implementation-review-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-implementation-chain-external-review-v1-no-parser",
        review_endpoint_available: true, items: [], implementation_count: 0,
        review_eligible_count: 0, reviewed_count: 0, independently_approved_count: 0,
        changes_required_or_rejected_count: 0,
        future_isolated_parser_runner_specification_registration_eligible_count: 0,
        review_status: "waiting", isolated_runner_registered: false,
        source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false, raw_payload_accessed: false,
        parsed_calendar_rows_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserImplementationReviews();
    await reviewControlledShadowMarketDataParserImplementationOnce("a".repeat(32), {
      expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64),
      expected_specification_review_sha256: "3".repeat(64),
      expected_specification_registration_sha256: "4".repeat(64),
      expected_parser_specification_sha256: "5".repeat(64),
      expected_independent_audit_sha256: "6".repeat(64),
      verdict: "approved_for_future_isolated_market_data_parser_runner_specification_registration",
      rationale: "independent recomputation passed",
      binding_and_recomputation_assessment: "all hashes independently reproduced",
      deterministic_parser_semantics_assessment: "eight pure function contracts match",
      source_schema_calendar_action_and_reconciliation_assessment: "sources and calendar remain explicit",
      failure_and_missing_data_assessment: "all invalid inputs fail closed",
      zero_capability_assessment: "no source artifact or runtime exists",
      known_limitations: "source available-at remains unverified",
      future_runner_constraints: "Stage 99 specification only",
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
    });
    expect(requests[0].url).toContain("market-data-parser-implementation-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-implementation-reviews/${"a".repeat(32)}/review-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      all_eight_synthetic_vectors_independently_reconstructed_confirmed: true,
      approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed: true,
    });
  });

  test("registers Stage 99 isolated parser runner specification without execution authority", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-isolated-runner-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-isolated-runner-create-once-v1-no-execution",
        eligible_implementations: [], registration_eligible_count: 0, runner_count: 0,
        current_binding_runner_count: 0, first_execution_authorization_review_eligible_count: 0,
        items: [], runner_status: "waiting", source_artifact_present: false,
        executable_artifact_present: false, callable_entrypoint_present: false,
        runtime_instantiated: false, raw_payload_accessed: false,
        parsed_calendar_rows_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "specification only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserIsolatedRunners();
    await registerControlledShadowMarketDataParserIsolatedRunnerOnce("a".repeat(32), {
      expected_implementation_id: "a".repeat(32), expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64), expected_implementation_review_id: "b".repeat(32),
      expected_implementation_review_sha256: "3".repeat(64), expected_independent_audit_sha256: "4".repeat(64),
      expected_specification_review_sha256: "5".repeat(64), expected_specification_registration_sha256: "6".repeat(64),
      expected_parser_specification_sha256: "7".repeat(64), expected_validation_sha256: "8".repeat(64),
      expected_receipt_sha256: "9".repeat(64), expected_claim_sha256: "a".repeat(64), expected_result_sha256: "b".repeat(64),
      runner_name: "isolated parser", runner_kind: "ephemeral_deterministic_market_data_parser_specification",
      runner_spec_revision: "v1", proposed_runner_code_revision: "rev-1", proposed_runner_artifact_sha256: "c".repeat(64),
      artifact_reproduction_procedure: "rebuild independently", rationale: "freeze future runner",
      known_limitations: "no artifact or payload", future_input_constraints: "Stage 94 validated read-only only",
      future_output_constraints: "create-once untrusted only", exact_current_stage_51_through_stage_98_binding_confirmed: true,
      registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: true,
      strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-isolated-runners");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-isolated-runners/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
    });
  });

  test("reviews Stage 100 only through the server-verified artifact authorization endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-first-execution-authorization-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-first-execution-authorization-v1-server-rehashed-single-use-24h",
        items: [], runner_count: 0, artifact_verified_runner_count: 0,
        artifact_pending_runner_count: 0, review_eligible_runner_count: 0,
        reviewed_runner_count: 0, approved_runner_count: 0,
        unexpired_authorization_count: 0, one_shot_authorized_count: 0,
        future_claim_eligible_count: 0, authorization_status: "waiting",
        next_gate: "stage_101_claim_first_market_data_parser_execution_attempt",
        callable_entrypoint_present: false, runtime_instantiated: false,
        raw_payload_mount_present: false, raw_payload_read: false, parser_executed: false,
        parsed_calendar_rows_created: false, parsed_market_rows_created: false,
        forward_observation_started: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserFirstExecutionAuthorizations();
    await reviewControlledShadowMarketDataParserFirstExecutionAuthorizationOnce("a".repeat(32), {
      expected_review_id: undefined, expected_review_sha256: undefined,
      expected_isolated_runner_id: "a".repeat(32), expected_isolated_runner_spec_sha256: "1".repeat(64),
      expected_runner_contract_sha256: "2".repeat(64), expected_runner_spec_revision: "v1",
      expected_runner_code_revision: "rev-1", expected_runner_artifact_sha256: "3".repeat(64),
      expected_implementation_id: "b".repeat(32), expected_implementation_sha256: "4".repeat(64),
      expected_implementation_contract_sha256: "5".repeat(64), expected_implementation_review_id: "c".repeat(32),
      expected_implementation_review_sha256: "6".repeat(64), expected_independent_audit_sha256: "7".repeat(64),
      expected_specification_review_sha256: "8".repeat(64), expected_specification_registration_sha256: "9".repeat(64),
      expected_parser_specification_sha256: "a".repeat(64), expected_validation_sha256: "b".repeat(64),
      expected_receipt_sha256: "c".repeat(64), expected_claim_sha256: "d".repeat(64),
      expected_result_sha256: "e".repeat(64), expected_artifact_manifest_sha256: "f".repeat(64),
      artifact_reproduction_review_evidence: "server rehashed", sandbox_contract_review_evidence: "sandbox checked",
      verdict: "changes_requested_rebuild_artifact", rationale: "keep closed",
      exact_current_stage_51_through_stage_99_binding_confirmed: true,
      reviewer_independent_from_stage_99_builder_and_complete_prior_chain_confirmed: true,
      server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: true,
      self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: true,
      artifact_builder_and_reviewer_separation_confirmed: true,
      all_eight_parser_functions_and_canonical_schemas_remain_bound_confirmed: true,
      strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
      future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: true,
      future_output_create_once_untrusted_independently_validated_no_market_interpretation_or_order_intent_confirmed: true,
      source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
      authorization_single_use_24_hour_expiry_and_stage_101_claim_separation_confirmed: true,
      no_runtime_entrypoint_mount_payload_read_parser_execution_or_parsed_rows_confirmed: true,
      no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_stage_101_claim_first_attempt_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-first-execution-authorizations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-first-execution-authorizations/${"a".repeat(32)}/review-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_artifact_manifest_sha256: "f".repeat(64),
      approval_only_opens_future_stage_101_claim_first_attempt_confirmed: true,
    });
  });

  test("creates Stage 101 claim-first metadata before any parser execution endpoint exists", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-execution-attempt-claim-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-execution-attempt-claim-v1-create-once-consumes-stage-100-before-execution",
        claim_endpoint_available: true, eligible_authorizations: [], claims: [],
        authorization_candidate_count: 0, claim_eligible_count: 0, claim_count: 0,
        authorization_consumed_count: 0, waiting_for_stage_102_execution_count: 0,
        claim_status: "waiting", next_gate: "stage_102_single_claim_parser_execution_attempt",
        execution_attempt_endpoint_available: false, callable_entrypoint_present: false,
        runtime_instantiated: false, raw_payload_mount_present: false, raw_payload_read: false,
        parser_executed: false, parsed_calendar_rows_created: false,
        parsed_market_rows_created: false, forward_observation_started: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "claim only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserExecutionAttemptClaims();
    await claimControlledShadowMarketDataParserExecutionAttemptOnce("a".repeat(32), {
      expected_authorization_review_sha256: "1".repeat(64),
      expected_isolated_runner_spec_sha256: "2".repeat(64),
      expected_runner_artifact_sha256: "3".repeat(64),
      expected_artifact_manifest_sha256: "4".repeat(64),
      expected_stage_94_validation_sha256: "5".repeat(64),
      expected_stage_93_claim_sha256: "6".repeat(64),
      expected_stage_93_result_sha256: "7".repeat(64),
      expected_stage_93_receipt_sha256: "8".repeat(64),
      expected_canonical_request_set_sha256: "9".repeat(64),
      expected_fixed_input_manifest_sha256: "a".repeat(64),
      claim_reason: "freeze the exact input before execution",
      exact_current_stage_51_through_stage_100_binding_confirmed: true,
      claimant_independent_from_stage_100_and_complete_prior_chain_confirmed: true,
      authorization_unexpired_single_use_and_consumed_before_execution_confirmed: true,
      current_server_rehashed_artifact_and_manifest_binding_confirmed: true,
      fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: true,
      claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: true,
      no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: true,
      future_output_create_once_untrusted_and_independently_validated_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-execution-attempt-claims");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-execution-attempt-claims/${"a".repeat(32)}/claim-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_fixed_input_manifest_sha256: "a".repeat(64),
      authorization_unexpired_single_use_and_consumed_before_execution_confirmed: true,
      no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: true,
    });
  });

  test("executes one Stage 102 declarative parser attempt with no retry authority", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-execution-attempt-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-execution-v1-one-shot-in-process-declarative-fail-closed",
        execution_endpoint_available: true, pending_claims: [], results: [],
        pending_claim_count: 0, terminal_result_count: 0,
        successful_untrusted_output_count: 0, failed_consumed_claim_count: 0,
        next_gate: "stage_103_independent_parser_output_validation",
        arbitrary_artifact_execution_allowed: false, outbound_network_allowed: false,
        independent_validation_completed: false, forward_observation_started: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "one shot",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserExecutionAttempts();
    await executeControlledShadowMarketDataParserAttemptOnce("b".repeat(32), {
      expected_claim_sha256: "1".repeat(64),
      expected_authorization_review_sha256: "2".repeat(64),
      expected_runner_artifact_sha256: "3".repeat(64),
      expected_input_manifest_sha256: "4".repeat(64),
      execution_reason: "execute exact frozen parser input once",
      exact_stage_51_through_stage_101_binding_confirmed: true,
      executor_independent_from_complete_prior_chain_confirmed: true,
      one_shot_failure_consumes_claim_and_no_retry_confirmed: true,
      artifact_is_declarative_not_spawned_or_executed_confirmed: true,
      only_fixed_stage_94_payloads_are_read_only_opened_and_rehashed_confirmed: true,
      strict_parser_and_cross_source_reconciliation_fail_closed_confirmed: true,
      output_create_once_untrusted_and_requires_independent_validation_confirmed: true,
      no_network_environment_secret_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-execution-attempts");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-execution-attempts/${"b".repeat(32)}/execute-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      artifact_is_declarative_not_spawned_or_executed_confirmed: true,
      one_shot_failure_consumes_claim_and_no_retry_confirmed: true,
    });
  });

  test("validates one Stage 102 parser output with a chain-external second implementation", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-market-data-parser-output-validation-registry-v1",
        policy_version: "hone-controlled-shadow-market-data-parser-chain-external-full-reparse-validation-v1",
        validator_implementation_version: "validator-v1",
        validator_implementation_sha256: "a".repeat(64), items: [],
        validation_eligible_count: 0, validation_count: 0,
        independently_validated_output_count: 0, failed_validation_count: 0,
        future_observation_input_admission_review_eligible_count: 0,
        validation_status: "waiting_successful_stage_102_untrusted_output",
        next_gate: "stage_104_first_natural_forward_cycle_observation_input_admission_review",
        independent_output_validation_available: true,
        source_available_at_verified: false, forward_observation_started: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "independent full reparse",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowMarketDataParserOutputValidations();
    await validateControlledShadowMarketDataParserOutputOnce("c".repeat(32), {
      expected_claim_sha256: "1".repeat(64),
      expected_result_sha256: "2".repeat(64),
      expected_output_sha256: "3".repeat(64),
      expected_input_manifest_sha256: "4".repeat(64),
      expected_stage_94_validation_sha256: "5".repeat(64),
      validation_reason: "independently reparse exact frozen inputs",
      exact_current_stage_51_through_stage_102_binding_confirmed: true,
      validator_independent_from_executor_and_complete_prior_chain_confirmed: true,
      stage_102_result_output_and_create_once_custody_reopened_confirmed: true,
      fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed: true,
      second_implementation_does_not_call_stage_102_parser_helpers_confirmed: true,
      every_canonical_row_hash_and_complete_output_exactly_compared_confirmed: true,
      official_calendar_spy_coverage_subject_gaps_and_actions_fail_closed_confirmed: true,
      source_available_at_remains_unverified_confirmed: true,
      pass_only_opens_future_observation_input_admission_review_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("market-data-parser-output-validations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`market-data-parser-output-validations/${"c".repeat(32)}/validate-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_output_sha256: "3".repeat(64),
      second_implementation_does_not_call_stage_102_parser_helpers_confirmed: true,
      source_available_at_remains_unverified_confirmed: true,
    });
  });

  test("reviews one Stage 103 output for conservative Stage 104 input admission", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "hone-controlled-shadow-first-natural-forward-cycle-observation-input-admission-review-registry-v1",
        policy_version: "admission-v1", items: [],
        independently_validated_input_candidate_count: 0,
        review_eligible_candidate_count: 0, reviewed_candidate_count: 0,
        admitted_input_count: 0, changes_requested_or_rejected_count: 0,
        future_observation_materialization_specification_registration_eligible_count: 0,
        admission_status: "waiting_stage_103_independently_validated_parser_output",
        next_gate: "stage_105_first_natural_forward_cycle_observation_materialization_specification_registration",
        admission_review_available: true, provider_publication_time_verified: false,
        custody_retrieval_time_floor_required: true, forward_observation_started: false,
        ledger_created: false, position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "custody-time floor only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationInputAdmissionReviews();
    await reviewControlledShadowObservationInputAdmission("d".repeat(32), {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_stage_103_validation_id: "1".repeat(32),
      expected_stage_103_validation_sha256: "2".repeat(64),
      expected_stage_102_result_sha256: "3".repeat(64),
      expected_stage_102_output_sha256: "4".repeat(64),
      expected_stage_101_claim_sha256: "5".repeat(64),
      expected_stage_101_input_manifest_sha256: "6".repeat(64),
      expected_cycle_claim_sha256: "7".repeat(64),
      verdict: "approved_for_future_create_once_observation_materialization_specification_registration",
      rationale: "exact current input is structurally complete",
      known_limitations: "provider publication time remains unverified",
      exact_current_stage_51_through_stage_103_binding_confirmed: true,
      reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: true,
      stage_103_full_reparse_validation_current_and_passed_confirmed: true,
      cycle_claim_natural_forward_only_and_no_backfill_confirmed: true,
      fixed_subject_spy_window_and_request_identities_confirmed: true,
      every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: true,
      custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed: true,
      admitted_rows_within_frozen_window_and_available_before_admission_confirmed: true,
      official_sessions_and_spy_three_price_bases_complete_confirmed: true,
      subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: true,
      dividends_splits_and_three_price_bases_remain_separate_confirmed: true,
      exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: true,
      approval_only_opens_future_materialization_specification_registration_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("observation-input-admission-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-input-admission-reviews/${"d".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(new Headers(requests[1].init?.headers).get("x-hone-admin-action")).toBe("whitelist");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      verdict: "approved_for_future_create_once_observation_materialization_specification_registration",
      custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed: true,
    });
  });

  test("registers a zero-capability Stage 105 observation materialization specification", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "registry-v1", policy_version: "policy-v1",
        registration_endpoint_available: true, candidates: [], registrations: [],
        admitted_input_count: 0, registration_eligible_count: 0,
        specification_registered_count: 0,
        future_chain_external_specification_review_eligible_count: 0,
        specification_status: "waiting_stage_104_admitted_observation_input",
        next_gate: "stage_106_first_natural_forward_cycle_observation_materialization_specification_independent_review",
        implementation_present: false, observation_materialized: false,
        ledger_created: false, position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "specification only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationSpecifications();
    await registerControlledShadowObservationMaterializationSpecification("a".repeat(32), {
      expected_stage_104_review_sha256: "1".repeat(64),
      expected_stage_103_validation_sha256: "2".repeat(64),
      expected_stage_102_result_sha256: "3".repeat(64),
      expected_stage_102_output_sha256: "4".repeat(64),
      expected_stage_101_claim_sha256: "5".repeat(64),
      expected_stage_101_input_manifest_sha256: "6".repeat(64),
      expected_cycle_claim_sha256: "7".repeat(64),
      registration_reason: "freeze deterministic materialization semantics",
      known_limitations: "provider publication time remains unverified",
      future_review_constraints: "chain-external review required",
      exact_current_stage_51_through_stage_104_binding_confirmed: true,
      registrar_independent_from_stage_104_and_complete_prior_chain_confirmed: true,
      exact_admitted_output_only_no_refetch_or_reparse_confirmed: true,
      conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed: true,
      dividends_splits_and_price_bases_remain_separate_confirmed: true,
      initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed: true,
      deterministic_canonical_order_decimal_and_row_hash_rules_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed: true,
      spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed: true,
      specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: true,
      no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_chain_external_specification_review_required_before_implementation_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("observation-materialization-specifications");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-specifications/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      exact_admitted_output_only_no_refetch_or_reparse_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
  });
});

describe("controlled shadow observation materialization specification review API", () => {
  test("uses chain-external Stage 106 review routes and preserves all confirmations", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "review-registry-v1", policy_version: "review-v1",
        review_endpoint_available: true, items: [], specification_count: 0,
        review_eligible_count: 0, reviewed_count: 0, independently_approved_count: 0,
        changes_required_or_rejected_count: 0,
        future_zero_capability_implementation_registration_eligible_count: 0,
        review_status: "waiting_stage_105", implementation_registered: false,
        observation_materialized: false, ledger_created: false, position_written: false,
        performance_metric_written: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationSpecificationReviews();
    await reviewControlledShadowObservationMaterializationSpecification("a".repeat(32), {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_registration_sha256: "1".repeat(64),
      expected_specification_sha256: "2".repeat(64),
      expected_independent_audit_sha256: "3".repeat(64),
      verdict: "approved_for_future_zero_capability_observation_materialization_implementation_registration",
      rationale: "independent rebuild passed",
      binding_and_second_implementation_assessment: "all bindings reproduced",
      session_price_basis_and_gap_assessment: "sessions, bases and gaps preserved",
      corporate_action_decimal_order_and_hash_assessment: "actions and hashes preserved",
      initial_allocation_and_availability_assessment: "bindings and custody time preserved",
      zero_capability_assessment: "all authority closed",
      known_limitations: "no real observation",
      future_implementation_constraints: "zero-capability registration only",
      exact_current_stage_51_through_stage_105_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      registration_and_specification_hashes_independently_reproduced_confirmed: true,
      complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed: true,
      rebuilt_specification_exactly_matches_registered_specification_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_and_price_bases_remain_separate_confirmed: true,
      decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
      initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
      conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_zero_capability_implementation_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-specification-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-specification-reviews/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
  });
});

describe("controlled shadow observation materialization implementation API", () => {
  test("registers only a create-once Stage 107 zero-capability contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "implementation-registry-v1", policy_version: "implementation-v1",
        registration_endpoint_available: true, items: [],
        independently_approved_specification_count: 0, registration_eligible_count: 0,
        implementation_contract_count: 0, current_binding_implementation_contract_count: 0,
        independent_implementation_review_eligible_count: 0,
        implementation_status: "waiting_stage_106", source_artifact_present: false,
        executable_artifact_present: false, callable_entrypoint_present: false,
        runtime_present: false, input_mounted_or_read: false, observation_materialized: false,
        ledger_created: false, position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "contract only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationImplementations();
    await registerControlledShadowObservationMaterializationImplementationOnce("a".repeat(32), {
      expected_specification_review_id: "a".repeat(32),
      expected_specification_review_sha256: "1".repeat(64),
      expected_independent_audit_sha256: "2".repeat(64),
      expected_registration_id: "b".repeat(32),
      expected_registration_sha256: "3".repeat(64),
      expected_specification_sha256: "4".repeat(64),
      implementation_name: "deterministic materialization contract",
      immutable_code_revision: "revision-107",
      implementation_description: "contract only",
      deterministic_projection_semantics: "pure deterministic projections",
      session_price_basis_and_gap_semantics: "official sessions and explicit gaps",
      corporate_action_decimal_order_and_hash_semantics: "separate actions and canonical hashes",
      initial_allocation_and_availability_semantics: "preserve bindings and conservative availability",
      error_and_missing_data_semantics: "fail closed",
      known_limitations: "no source artifact or runtime",
      future_review_constraints: "Stage 108 independent review first",
      exact_stage_51_through_stage_106_binding_confirmed: true,
      registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_review_registration_specification_and_audit_confirmed: true,
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      exact_stage_104_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_and_price_bases_remain_separate_confirmed: true,
      decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
      initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
      conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-implementations/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization implementation review API", () => {
  test("submits only a Stage 108 chain-external zero-capability review", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "review-registry-v1", policy_version: "review-v1", items: [],
        implementation_count: 0, review_eligible_count: 0, reviewed_count: 0,
        independently_approved_count: 0, changes_required_or_rejected_count: 0,
        future_isolated_observation_materialization_runner_specification_registration_eligible_count: 0,
        review_status: "waiting_stage_107", isolated_runner_registered: false,
        source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false,
        input_mounted_or_read: false, observation_materialized: false,
        ledger_created: false, position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationImplementationReviews();
    await reviewControlledShadowObservationMaterializationImplementationOnce("a".repeat(32), {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64),
      expected_specification_review_sha256: "3".repeat(64),
      expected_specification_independent_audit_sha256: "4".repeat(64),
      expected_specification_registration_sha256: "5".repeat(64),
      expected_observation_materialization_specification_sha256: "6".repeat(64),
      expected_independent_audit_sha256: "7".repeat(64),
      verdict: "approved_for_future_isolated_observation_materialization_runner_specification_registration",
      rationale: "independent review",
      binding_and_recomputation_assessment: "recomputed",
      deterministic_projection_semantics_assessment: "eight pure functions",
      session_price_basis_gap_and_company_action_assessment: "explicit",
      initial_allocation_availability_and_output_assessment: "preserved",
      zero_capability_assessment: "closed",
      known_limitations: "no runtime",
      future_runner_constraints: "Stage 109 only",
      exact_current_stage_51_through_stage_107_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: true,
      all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
      exact_stage_104_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: true,
      initial_shadow_allocation_and_conservative_availability_preserved_confirmed: true,
      provider_publication_time_remains_unverified_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-implementation-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-implementation-reviews/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
      approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization isolated runner API", () => {
  test("registers only a Stage 109 proposed-artifact zero-capability runner specification", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "runner-registry-v1", policy_version: "runner-v1",
        eligible_implementations: [], registration_eligible_count: 0, runner_count: 0,
        current_binding_runner_count: 0, first_execution_authorization_review_eligible_count: 0,
        items: [], runner_status: "waiting_stage_108", source_artifact_present: false,
        executable_artifact_present: false, callable_entrypoint_present: false,
        runtime_instantiated: false, input_accessed: false, sessions_materialized: false,
        price_observations_materialized: false, observation_materialized: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "specification only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationIsolatedRunners();
    await registerControlledShadowObservationMaterializationIsolatedRunnerOnce("a".repeat(32), {
      expected_implementation_id: "a".repeat(32),
      expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64),
      expected_implementation_review_id: "b".repeat(32),
      expected_implementation_review_sha256: "3".repeat(64),
      expected_independent_audit_sha256: "4".repeat(64),
      expected_specification_review_sha256: "5".repeat(64),
      expected_specification_registration_sha256: "6".repeat(64),
      expected_observation_materialization_specification_sha256: "7".repeat(64),
      expected_stage_104_admission_review_sha256: "8".repeat(64),
      expected_stage_103_validation_sha256: "9".repeat(64),
      expected_stage_102_result_sha256: "a".repeat(64),
      expected_stage_101_claim_sha256: "b".repeat(64),
      expected_cycle_claim_sha256: "c".repeat(64),
      runner_name: "isolated materializer", runner_kind: "ephemeral_deterministic_observation_materialization_specification",
      runner_spec_revision: "v1", proposed_runner_code_revision: "rev-109",
      proposed_runner_artifact_sha256: "d".repeat(64), artifact_reproduction_procedure: "reproduce independently",
      rationale: "freeze boundary", known_limitations: "artifact absent",
      future_input_constraints: "Stage 104 only", future_output_constraints: "create-once untrusted",
      exact_current_stage_51_through_stage_108_binding_confirmed: true,
      registrar_independent_from_stage_108_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_eight_observation_materialization_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: true,
      session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: true,
      no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_preserved_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      provider_publication_time_remains_unverified_until_separate_evidence_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-isolated-runners");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-isolated-runners/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      proposed_runner_artifact_sha256: "d".repeat(64),
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization first-execution authorization API", () => {
  test("reviews a server-rehashed Stage 110 artifact without executing it", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "authorization-registry-v1", policy_version: "authorization-v1",
        items: [], runner_count: 0, artifact_verified_runner_count: 0,
        artifact_pending_runner_count: 0, review_eligible_runner_count: 0,
        reviewed_runner_count: 0, approved_runner_count: 0,
        unexpired_authorization_count: 0, one_shot_authorized_count: 0,
        future_claim_eligible_count: 0, authorization_status: "waiting_stage_109",
        next_gate: "stage_111_claim_first_observation_materialization_execution_attempt",
        callable_entrypoint_present: false, runtime_instantiated: false,
        input_mount_present: false, input_read: false,
        observation_materialization_executed: false, sessions_materialized: false,
        price_observations_materialized: false, observation_materialized: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationFirstExecutionAuthorizations();
    await reviewControlledShadowObservationMaterializationFirstExecutionAuthorizationOnce("a".repeat(32), {
      expected_isolated_runner_id: "a".repeat(32),
      expected_isolated_runner_spec_sha256: "1".repeat(64),
      expected_runner_contract_sha256: "2".repeat(64),
      expected_runner_spec_revision: "v1", expected_runner_code_revision: "rev-110",
      expected_runner_artifact_sha256: "3".repeat(64),
      expected_implementation_id: "b".repeat(32), expected_implementation_sha256: "4".repeat(64),
      expected_implementation_contract_sha256: "5".repeat(64),
      expected_implementation_review_id: "c".repeat(32), expected_implementation_review_sha256: "6".repeat(64),
      expected_independent_audit_sha256: "7".repeat(64),
      expected_specification_review_sha256: "8".repeat(64),
      expected_specification_registration_sha256: "9".repeat(64),
      expected_observation_materialization_specification_sha256: "a".repeat(64),
      expected_stage_104_admission_review_sha256: "b".repeat(64),
      expected_stage_103_validation_sha256: "c".repeat(64),
      expected_stage_102_result_sha256: "d".repeat(64), expected_stage_102_output_sha256: "e".repeat(64),
      expected_stage_101_claim_sha256: "f".repeat(64), expected_stage_101_input_manifest_sha256: "0".repeat(64),
      expected_cycle_claim_sha256: "1".repeat(64), expected_artifact_manifest_sha256: "2".repeat(64),
      artifact_reproduction_review_evidence: "independent reproduction", sandbox_contract_review_evidence: "sandbox reviewed",
      verdict: "changes_requested_rebuild_artifact", rationale: "rebuild before approval",
      exact_current_stage_51_through_stage_109_binding_confirmed: true,
      reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed: true,
      server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: true,
      self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: true,
      artifact_builder_and_reviewer_separation_confirmed: true,
      all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed: true,
      session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: true,
      no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
      future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      provider_publication_time_remains_unverified_until_separate_evidence_confirmed: true,
      authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed: true,
      no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed: true,
      no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_stage_111_claim_first_attempt_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-first-execution-authorizations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-first-execution-authorizations/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_artifact_manifest_sha256: "2".repeat(64),
      approval_only_opens_future_stage_111_claim_first_attempt_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization execution-attempt claim API", () => {
  test("permanently consumes Stage 110 before any Stage 112 execution exists", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "claim-registry-v1", policy_version: "claim-v1",
        claim_endpoint_available: true, eligible_authorizations: [], claims: [],
        authorization_candidate_count: 0, claim_eligible_count: 0, claim_count: 0,
        authorization_consumed_count: 0, waiting_for_stage_112_execution_count: 0,
        claim_status: "waiting_stage_110", next_gate: "stage_112_single_claim_observation_materialization_execution_attempt",
        execution_attempt_endpoint_available: false, callable_entrypoint_present: false,
        runtime_instantiated: false, input_mount_present: false, input_read: false,
        observation_materialization_executed: false, sessions_materialized: false,
        price_observations_materialized: false, observation_materialized: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "claim only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationExecutionAttemptClaims();
    await claimControlledShadowObservationMaterializationExecutionAttemptOnce("a".repeat(32), {
      expected_authorization_review_sha256: "1".repeat(64),
      expected_isolated_runner_spec_sha256: "2".repeat(64),
      expected_runner_contract_sha256: "3".repeat(64),
      expected_runner_artifact_sha256: "4".repeat(64),
      expected_artifact_manifest_sha256: "5".repeat(64),
      expected_implementation_sha256: "6".repeat(64),
      expected_implementation_contract_sha256: "7".repeat(64),
      expected_implementation_review_sha256: "8".repeat(64),
      expected_observation_materialization_specification_sha256: "9".repeat(64),
      expected_stage_104_admission_review_sha256: "a".repeat(64),
      expected_stage_103_validation_sha256: "b".repeat(64),
      expected_stage_102_result_sha256: "c".repeat(64),
      expected_stage_102_output_sha256: "d".repeat(64),
      expected_stage_101_claim_sha256: "e".repeat(64),
      expected_stage_101_input_manifest_sha256: "f".repeat(64),
      expected_cycle_claim_sha256: "0".repeat(64),
      claim_reason: "freeze one attempt identity before execution",
      exact_current_stage_51_through_stage_110_binding_confirmed: true,
      claimant_independent_from_stage_110_and_complete_prior_chain_confirmed: true,
      authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: true,
      current_server_rehashed_artifact_and_manifest_binding_confirmed: true,
      exact_stage_104_admitted_input_remains_content_addressed_read_only_and_unread_confirmed: true,
      claim_contains_only_existing_metadata_and_hashes_confirmed: true,
      no_entrypoint_runtime_input_mount_input_read_or_observation_materialization_execution_confirmed: true,
      future_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: true,
      no_retry_release_or_authorization_restoration_after_claim_confirmed: true,
      no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-execution-attempt-claims");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-execution-attempt-claims/${"a".repeat(32)}/claim-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_stage_104_admission_review_sha256: "a".repeat(64),
      no_retry_release_or_authorization_restoration_after_claim_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization one-shot execution API", () => {
  test("uses the fixed Stage 112 endpoint and sends every fail-closed confirmation", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "execution-registry-v1", policy_version: "execution-v1",
        execution_endpoint_available: true, pending_claims: [], results: [],
        pending_claim_count: 0, terminal_result_count: 0,
        successful_untrusted_observation_count: 0, failed_consumed_claim_count: 0,
        next_gate: "stage_113_independent_observation_materialization_output_validation",
        arbitrary_artifact_execution_allowed: false, outbound_network_allowed: false,
        independent_validation_completed: false, observation_envelope_created: false,
        forward_observation_started: false, ledger_created: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "one shot only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationExecutionAttempts();
    await executeControlledShadowObservationMaterializationAttemptOnce("a".repeat(32), {
      expected_claim_sha256: "1".repeat(64),
      expected_authorization_review_sha256: "2".repeat(64),
      expected_runner_artifact_sha256: "3".repeat(64),
      expected_artifact_manifest_sha256: "4".repeat(64),
      expected_implementation_contract_sha256: "5".repeat(64),
      expected_observation_materialization_specification_sha256: "6".repeat(64),
      expected_stage_104_admission_review_sha256: "7".repeat(64),
      expected_stage_102_output_sha256: "8".repeat(64),
      expected_stage_101_input_manifest_sha256: "9".repeat(64),
      expected_cycle_claim_sha256: "a".repeat(64),
      execution_reason: "materialize one exact natural-forward envelope",
      exact_stage_51_through_stage_111_binding_confirmed: true,
      executor_independent_from_complete_prior_chain_and_claimant_confirmed: true,
      start_marker_consumes_claim_before_artifact_or_input_read_confirmed: true,
      one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: true,
      artifact_is_declarative_not_spawned_or_executed_confirmed: true,
      only_exact_stage_104_admitted_output_is_read_only_opened_and_rehashed_confirmed: true,
      deterministic_session_price_gap_action_allocation_and_availability_projection_confirmed: true,
      no_refetch_reparse_fill_interpolation_substitution_backfill_or_correction_confirmed: true,
      output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: true,
      no_network_environment_secret_tool_subprocess_or_production_io_confirmed: true,
      no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-execution-attempts");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-execution-attempts/${"a".repeat(32)}/execute-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_stage_104_admission_review_sha256: "7".repeat(64),
      one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: true,
      no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
    });
  });
});

describe("controlled shadow observation materialization independent output validation API", () => {
  test("uses the fixed Stage 113 endpoint and sends every fail-closed confirmation", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "validation-registry-v1", policy_version: "validation-v1",
        validator_implementation_version: "independent-second-projection-v1",
        validator_implementation_sha256: "f".repeat(64), items: [],
        validation_eligible_count: 0, validation_count: 0,
        independently_validated_observation_count: 0, failed_validation_count: 0,
        future_stage_114_observation_evidence_admission_review_eligible_count: 0,
        validation_status: "waiting_for_stage_112_output", next_gate: "stage_114_validated_observation_envelope_admission_review",
        independent_output_validation_available: true, ledger_created: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "independent validation only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationMaterializationOutputValidations();
    await validateControlledShadowObservationMaterializationOutputOnce("a".repeat(32), {
      expected_claim_sha256: "1".repeat(64),
      expected_result_sha256: "2".repeat(64),
      expected_output_sha256: "3".repeat(64),
      expected_specification_sha256: "4".repeat(64),
      expected_stage_104_review_sha256: "5".repeat(64),
      expected_stage_102_output_sha256: "6".repeat(64),
      validation_reason: "independently reproject the exact observation envelope",
      exact_current_stage_51_through_stage_112_binding_confirmed: true,
      validator_independent_from_executor_and_complete_prior_chain_confirmed: true,
      stage_112_result_and_create_once_output_reopened_and_rehashed_confirmed: true,
      exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed: true,
      second_projection_does_not_call_stage_112_materializer_helpers_confirmed: true,
      sessions_prices_gaps_actions_allocation_availability_independently_recomputed_confirmed: true,
      every_row_hash_sort_order_and_complete_envelope_exactly_compared_confirmed: true,
      pass_only_opens_future_stage_114_observation_evidence_admission_review_confirmed: true,
      no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-materialization-output-validations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-materialization-output-validations/${"a".repeat(32)}/validate-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_stage_104_review_sha256: "5".repeat(64),
      second_projection_does_not_call_stage_112_materializer_helpers_confirmed: true,
      no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
    });
  });
});

describe("controlled shadow observation evidence admission review API", () => {
  test("uses the fixed Stage 114 endpoint and preserves the no-ledger boundary", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "admission-registry-v1", policy_version: "admission-v1", items: [],
        independently_validated_candidate_count: 0, review_eligible_candidate_count: 0,
        reviewed_candidate_count: 0, admitted_observation_evidence_count: 0,
        changes_requested_or_rejected_count: 0,
        future_observation_ledger_transition_specification_registration_eligible_count: 0,
        admission_status: "waiting_stage_113", next_gate: "stage_115_observation_ledger_transition_specification_registration",
        admission_review_available: true, provider_publication_time_verified: false,
        original_envelope_mutated: false, ledger_created: false, nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "admission only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationEvidenceAdmissionReviews();
    await reviewControlledShadowObservationEvidenceAdmission("a".repeat(32), {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_stage_113_validation_id: "b".repeat(32),
      expected_stage_113_validation_sha256: "1".repeat(64),
      expected_stage_112_result_sha256: "2".repeat(64),
      expected_stage_112_output_sha256: "3".repeat(64),
      expected_stage_111_claim_sha256: "4".repeat(64),
      verdict: "admitted_for_future_observation_ledger_transition_specification_registration",
      rationale: "exact evidence independently admitted",
      known_limitations: "provider publication time unverified",
      exact_current_stage_51_through_stage_113_binding_confirmed: true,
      reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: true,
      stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: true,
      stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: true,
      exact_stage_104_admitted_input_binding_preserved_confirmed: true,
      sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: true,
      natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed: true,
      provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: true,
      admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed: true,
      approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed: true,
      no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-evidence-admission-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-evidence-admission-reviews/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      expected_stage_113_validation_sha256: "1".repeat(64),
      provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: true,
      no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: true,
    });
  });
});

describe("controlled shadow observation ledger transition specification API", () => {
  test("uses the fixed Stage 115 endpoints and sends every no-ledger prerequisite", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "ledger-transition-registry-v1", policy_version: "stage-115-v1",
        registration_endpoint_available: true, candidates: [], registrations: [],
        admitted_observation_evidence_count: 0, registration_eligible_count: 0,
        registered_specification_count: 0, future_stage_116_independent_review_eligible_count: 0,
        opening_portfolio_snapshot_missing_count: 0, registration_status: "waiting_stage_114",
        next_gate: "stage_116", implementation_present: false,
        opening_portfolio_snapshot_present: false, ledger_created: false,
        ledger_event_written: false, nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "specification only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionSpecifications();
    await registerControlledShadowObservationLedgerTransitionSpecification("r".repeat(32), {
      expected_stage_114_review_sha256: "1".repeat(64),
      expected_stage_113_validation_sha256: "2".repeat(64),
      expected_stage_112_result_sha256: "3".repeat(64),
      expected_stage_112_output_sha256: "4".repeat(64),
      expected_stage_111_claim_sha256: "5".repeat(64),
      registration_reason: "freeze deterministic transition semantics",
      known_limitations: "opening snapshot absent",
      future_review_constraints: "independent reconstruction required",
      exact_current_stage_51_through_stage_114_binding_confirmed: true,
      registrar_independent_from_stage_114_and_complete_prior_chain_confirmed: true,
      stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed: true,
      stage_88_binding_not_treated_as_opening_positions_confirmed: true,
      separately_admitted_opening_portfolio_snapshot_required_confirmed: true,
      no_default_notional_cash_positions_or_share_quantities_confirmed: true,
      raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed: true,
      explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed: true,
      dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed: true,
      exact_decimal_append_only_idempotent_and_available_at_rules_confirmed: true,
      corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed: true,
      specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: true,
      no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      future_chain_external_specification_review_required_before_implementation_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-ledger-transition-specifications");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-specifications/${"r".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      stage_88_binding_not_treated_as_opening_positions_confirmed: true,
      no_default_notional_cash_positions_or_share_quantities_confirmed: true,
      no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
    });
  });
});

describe("controlled shadow observation ledger transition specification review API", () => {
  test("uses the fixed Stage 116 endpoints and sends the independent no-ledger audit", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "ledger-review-registry-v1", policy_version: "stage-116-v1",
        review_endpoint_available: true, items: [], specification_count: 0,
        review_eligible_count: 0, reviewed_count: 0, independently_approved_count: 0,
        changes_required_or_rejected_count: 0,
        future_zero_capability_implementation_registration_eligible_count: 0,
        opening_portfolio_snapshot_missing_count: 0, review_status: "waiting_stage_115",
        implementation_registered: false, ledger_created: false, ledger_event_written: false,
        nav_or_performance_written: false, training_or_rl_feedback_authorized: false,
        order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false, scope: "independent review only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionSpecificationReviews();
    await reviewControlledShadowObservationLedgerTransitionSpecification("g".repeat(32), {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_registration_sha256: "1".repeat(64),
      expected_specification_sha256: "2".repeat(64),
      expected_independent_audit_sha256: "3".repeat(64),
      verdict: "approved_for_future_zero_capability_ledger_transition_implementation_registration",
      rationale: "second implementation exactly reproduced the contract",
      binding_and_second_implementation_assessment: "current chain and independent rebuild match",
      opening_portfolio_prerequisite_assessment: "opening snapshot remains absent and no state is inferred",
      price_basis_gap_and_nav_assessment: "raw close only; adjusted non-accounting; gaps block NAV",
      corporate_action_and_double_count_assessment: "notices only until holdings and terms are admitted",
      decimal_idempotency_correction_and_order_assessment: "exact decimal append-only idempotent double-entry",
      zero_capability_assessment: "all runtime and financial authority closed",
      known_limitations: "opening snapshot absent",
      future_implementation_constraints: "Stage 117 registration only",
      exact_current_stage_51_through_stage_115_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      registration_and_specification_hashes_independently_reproduced_confirmed: true,
      complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed: true,
      rebuilt_specification_exactly_matches_registered_specification_confirmed: true,
      stage_88_binding_not_opening_positions_confirmed: true,
      separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: true,
      raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: true,
      explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: true,
      dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: true,
      exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: true,
      corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed: true,
      conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
      no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed: true,
      no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_zero_capability_implementation_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-ledger-transition-specification-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-specification-reviews/${"g".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      stage_88_binding_not_opening_positions_confirmed: true,
      raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: true,
      approval_only_opens_future_zero_capability_implementation_registration_confirmed: true,
    });
  });
});

describe("controlled shadow observation ledger transition implementation API", () => {
  test("uses Stage 117 create-once endpoints and sends the zero-capability contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response(JSON.stringify({
        schema_version: "ledger-implementation-registry-v1", policy_version: "stage-117-v1",
        registration_endpoint_available: true, items: [], independently_approved_specification_count: 0,
        registration_eligible_count: 0, implementation_contract_count: 0,
        current_binding_implementation_contract_count: 0, independent_implementation_review_eligible_count: 0,
        opening_portfolio_snapshot_missing_count: 0, implementation_status: "waiting_stage_116",
        source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false, input_mounted_or_read: false,
        opening_portfolio_snapshot_present: false, ledger_created: false, ledger_event_written: false,
        position_written: false, cash_written: false, nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false, scope: "contract only",
      }), { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    const request = {
      expected_specification_review_id: "1".repeat(32), expected_specification_review_sha256: "1".repeat(64),
      expected_independent_audit_sha256: "2".repeat(64), expected_registration_id: "3".repeat(32),
      expected_registration_sha256: "3".repeat(64), expected_specification_sha256: "4".repeat(64),
      implementation_name: "ledger transition contract", immutable_code_revision: "revision-1",
      implementation_description: "zero capability only", deterministic_projection_semantics: "deterministic or fail",
      session_price_basis_and_gap_semantics: "raw only and gaps block nav",
      corporate_action_decimal_order_and_hash_semantics: "notice exact decimal idempotent double entry",
      initial_allocation_and_availability_semantics: "opening snapshot required",
      error_and_missing_data_semantics: "no fill interpolation substitution", known_limitations: "no opening snapshot",
      future_review_constraints: "Stage 118 independent review only",
      exact_stage_51_through_stage_116_binding_confirmed: true,
      registrar_independent_from_stage_116_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_review_registration_specification_and_audit_confirmed: true,
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      exact_stage_114_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_and_price_bases_remain_separate_confirmed: true,
      decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
      initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
      conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    await getControlledShadowObservationLedgerTransitionImplementations();
    await registerControlledShadowObservationLedgerTransitionImplementationOnce("1".repeat(32), request);

    expect(requests[0].url).toContain("observation-ledger-transition-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-implementations/${"1".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      exact_stage_51_through_stage_116_binding_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
    });
  });
});

describe("controlled shadow observation ledger transition implementation review API", () => {
  test("uses Stage 118 review endpoints and sends the exact independent audit binding", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(
        new Response(JSON.stringify({ items: [], implementation_count: 0 }), {
          headers: { "content-type": "application/json" },
        }),
      );
    }) as typeof fetch;

    const request = {
      expected_previous_review_id: null,
      expected_previous_review_sha256: null,
      expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64),
      expected_specification_review_sha256: "3".repeat(64),
      expected_specification_independent_audit_sha256: "4".repeat(64),
      expected_specification_registration_sha256: "5".repeat(64),
      expected_observation_ledger_transition_specification_sha256: "6".repeat(64),
      expected_independent_audit_sha256: "7".repeat(64),
      verdict: "approved_for_future_isolated_observation_ledger_transition_runner_specification_registration" as const,
      rationale: "independent contract rebuild passed",
      binding_and_recomputation_assessment: "all hashes reproduced",
      deterministic_projection_semantics_assessment: "eight pure contracts exact",
      session_price_basis_gap_and_company_action_assessment: "raw only, gap blocks nav, actions notice only",
      initial_allocation_availability_and_output_assessment: "opening snapshot missing and required",
      zero_capability_assessment: "all authority closed",
      known_limitations: "no opening snapshot or ledger",
      future_runner_constraints: "Stage 119 specification only",
      exact_current_stage_51_through_stage_117_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: true,
      all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
      exact_stage_114_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: true,
      initial_shadow_allocation_and_conservative_availability_preserved_confirmed: true,
      provider_publication_time_remains_unverified_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    await getControlledShadowObservationLedgerTransitionImplementationReviews();
    await reviewControlledShadowObservationLedgerTransitionImplementationOnce(
      "8".repeat(32),
      request,
    );

    expect(requests[0].url).toContain("observation-ledger-transition-implementation-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(
      `observation-ledger-transition-implementation-reviews/${"8".repeat(32)}/review`,
    );
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      exact_current_stage_51_through_stage_117_binding_confirmed: true,
      expected_independent_audit_sha256: "7".repeat(64),
    });
  });
});

describe("Stage 119 observation ledger transition isolated runner API", () => {
  test("uses read-only registry and create-once registration endpoints", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionIsolatedRunners();
    await registerControlledShadowObservationLedgerTransitionIsolatedRunnerOnce("a".repeat(32), {
      expected_implementation_id: "a".repeat(32),
      expected_implementation_sha256: "b".repeat(64),
      expected_implementation_contract_sha256: "c".repeat(64),
      expected_implementation_review_id: "d".repeat(32),
      expected_implementation_review_sha256: "e".repeat(64),
      expected_independent_audit_sha256: "f".repeat(64),
      expected_specification_review_sha256: "1".repeat(64),
      expected_specification_registration_sha256: "2".repeat(64),
      expected_observation_ledger_transition_specification_sha256: "3".repeat(64),
      expected_stage_114_admission_review_sha256: "4".repeat(64),
      expected_stage_113_validation_sha256: "5".repeat(64),
      expected_stage_112_result_sha256: "6".repeat(64),
      expected_stage_111_claim_sha256: "7".repeat(64),
      runner_name: "ledger runner",
      runner_kind: "ephemeral_deterministic_observation_ledger_transition_specification",
      runner_spec_revision: "v1",
      proposed_runner_code_revision: "rev-1",
      proposed_runner_artifact_sha256: "8".repeat(64),
      artifact_reproduction_procedure: "reproduce and compare hash",
      rationale: "freeze only",
      known_limitations: "no artifact or snapshot",
      future_input_constraints: "exact Stage 114 admitted input only",
      future_output_constraints: "create-once untrusted candidate only",
      exact_current_stage_51_through_stage_118_binding_confirmed: true,
      registrar_independent_from_stage_118_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_eight_observation_ledger_transition_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: true,
      session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: true,
      no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_preserved_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
      future_financial_events_require_separately_admitted_opening_snapshot_confirmed: true,
      provider_publication_time_remains_unverified_until_separate_evidence_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("observation-ledger-transition-isolated-runners");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-isolated-runners/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
    });
  });
});

describe("Stage 120 observation ledger transition first-execution authorization API", () => {
  test("uses the server-rehashed registry and append-only review endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionFirstExecutionAuthorizations();
    await reviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationOnce(
      "a".repeat(32),
      {
        expected_isolated_runner_id: "a".repeat(32),
        expected_isolated_runner_spec_sha256: "1".repeat(64),
        expected_runner_contract_sha256: "2".repeat(64),
        expected_runner_spec_revision: "v1",
        expected_runner_code_revision: "rev-1",
        expected_runner_artifact_sha256: "3".repeat(64),
        expected_implementation_id: "b".repeat(32),
        expected_implementation_sha256: "4".repeat(64),
        expected_implementation_contract_sha256: "5".repeat(64),
        expected_implementation_review_id: "c".repeat(32),
        expected_implementation_review_sha256: "6".repeat(64),
        expected_independent_audit_sha256: "7".repeat(64),
        expected_specification_review_sha256: "8".repeat(64),
        expected_specification_registration_sha256: "9".repeat(64),
        expected_observation_ledger_transition_specification_sha256: "a".repeat(64),
        expected_stage_114_admission_review_sha256: "b".repeat(64),
        expected_stage_113_validation_sha256: "c".repeat(64),
        expected_stage_112_result_sha256: "d".repeat(64),
        expected_stage_111_claim_sha256: "e".repeat(64),
        expected_artifact_manifest_sha256: "f".repeat(64),
        artifact_reproduction_review_evidence: "server hash verified",
        sandbox_contract_review_evidence: "sandbox limits verified",
        verdict: "changes_requested_rebuild_artifact",
        rationale: "test fail-closed review",
        exact_current_stage_51_through_stage_119_binding_confirmed: true,
        reviewer_independent_from_stage_119_builder_and_complete_prior_chain_confirmed: true,
        server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: true,
        self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: true,
        artifact_builder_and_reviewer_separation_confirmed: true,
        all_eight_observation_ledger_transition_functions_and_canonical_schemas_remain_bound_confirmed: true,
        session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: true,
        no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: true,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
        future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: true,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
        opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
        future_financial_events_require_separately_admitted_opening_snapshot_confirmed: true,
        future_attempt_limited_to_non_financial_notice_candidate_without_authoritative_state_confirmed: true,
        provider_publication_time_remains_unverified_until_separate_evidence_confirmed: true,
        authorization_single_use_24_hour_expiry_and_stage_121_claim_separation_confirmed: true,
        no_runtime_entrypoint_mount_input_read_observation_ledger_transition_execution_or_candidate_output_confirmed: true,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
        no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
        approval_only_opens_future_stage_121_claim_first_attempt_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    );

    expect(requests[0].url).toContain("observation-ledger-transition-first-execution-authorizations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(
      `observation-ledger-transition-first-execution-authorizations/${"a".repeat(32)}/review`,
    );
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
      future_attempt_limited_to_non_financial_notice_candidate_without_authoritative_state_confirmed: true,
    });
  });
});

describe("Stage 121 observation ledger transition execution-attempt claim API", () => {
  test("uses the create-once claim-first registry and irreversible claim endpoint", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionExecutionAttemptClaims();
    await claimControlledShadowObservationLedgerTransitionExecutionAttemptOnce(
      "a".repeat(32),
      {
        expected_authorization_review_sha256: "1".repeat(64),
        expected_isolated_runner_spec_sha256: "2".repeat(64),
        expected_runner_contract_sha256: "3".repeat(64),
        expected_runner_artifact_sha256: "4".repeat(64),
        expected_artifact_manifest_sha256: "5".repeat(64),
        expected_implementation_sha256: "6".repeat(64),
        expected_implementation_contract_sha256: "7".repeat(64),
        expected_implementation_review_sha256: "8".repeat(64),
        expected_observation_ledger_transition_specification_sha256: "9".repeat(64),
        expected_stage_114_admission_review_sha256: "a".repeat(64),
        expected_stage_113_validation_sha256: "b".repeat(64),
        expected_stage_112_result_sha256: "c".repeat(64),
        expected_stage_111_claim_sha256: "d".repeat(64),
        claim_reason: "consume the exact authorization before any execution",
        exact_current_stage_51_through_stage_120_binding_confirmed: true,
        claimant_independent_from_stage_120_and_complete_prior_chain_confirmed: true,
        authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: true,
        current_server_rehashed_artifact_and_manifest_binding_confirmed: true,
        exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed: true,
        claim_contains_only_existing_metadata_and_hashes_confirmed: true,
        no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed: true,
        future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: true,
        no_retry_release_or_authorization_restoration_after_claim_confirmed: true,
        no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    );

    expect(requests[0].url).toContain("observation-ledger-transition-execution-attempt-claims");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-execution-attempt-claims/${"a".repeat(32)}/claim-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      no_retry_release_or_authorization_restoration_after_claim_confirmed: true,
      exact_current_stage_51_through_stage_120_binding_confirmed: true,
    });
  });
});

describe("Stage 122 observation ledger transition one-shot execution API", () => {
  test("uses the irreversible execution endpoint and binds the exact admitted evidence chain", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionExecutionAttempts();
    await executeControlledShadowObservationLedgerTransitionAttemptOnce(
      "a".repeat(32),
      {
        expected_claim_sha256: "1".repeat(64),
        expected_authorization_review_sha256: "2".repeat(64),
        expected_runner_contract_sha256: "3".repeat(64),
        expected_runner_artifact_sha256: "4".repeat(64),
        expected_artifact_manifest_sha256: "5".repeat(64),
        expected_implementation_contract_sha256: "6".repeat(64),
        expected_observation_ledger_transition_specification_sha256: "7".repeat(64),
        expected_stage_114_admission_review_sha256: "8".repeat(64),
        expected_stage_113_validation_sha256: "9".repeat(64),
        expected_stage_112_result_sha256: "a".repeat(64),
        expected_stage_112_output_sha256: "b".repeat(64),
        expected_stage_111_claim_sha256: "c".repeat(64),
        execution_reason: "project the exact admitted evidence into notice candidates once",
        exact_stage_51_through_stage_121_binding_confirmed: true,
        executor_independent_from_complete_prior_chain_and_claimant_confirmed: true,
        start_marker_consumes_claim_before_artifact_or_input_read_confirmed: true,
        one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: true,
        artifact_is_declarative_not_spawned_or_executed_confirmed: true,
        only_exact_stage_114_admitted_output_is_read_only_reopened_and_rehashed_confirmed: true,
        opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed: true,
        non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed: true,
        raw_security_close_and_dividend_adjusted_spy_benchmark_separated_confirmed: true,
        explicit_gap_blocks_nav_and_corporate_actions_remain_pending_validation_confirmed: true,
        output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: true,
        no_network_environment_secret_tool_subprocess_or_production_io_confirmed: true,
        no_authoritative_financial_state_model_metric_training_reward_order_broker_or_trading_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    );

    expect(requests[0].url).toContain("observation-ledger-transition-execution-attempts");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-execution-attempts/${"a".repeat(32)}/execute-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed: true,
      non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed: true,
    });
  });
});

describe("Stage 123 observation ledger transition output validation API", () => {
  test("uses the chain-external validate-once endpoint and freezes the non-financial boundary", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionOutputValidations();
    await validateControlledShadowObservationLedgerTransitionOutputOnce(
      "a".repeat(32),
      {
        expected_claim_sha256: "1".repeat(64),
        expected_result_sha256: "2".repeat(64),
        expected_candidate_sha256: "3".repeat(64),
        expected_specification_sha256: "4".repeat(64),
        expected_stage_114_review_sha256: "5".repeat(64),
        expected_stage_112_output_sha256: "6".repeat(64),
        validation_reason: "independently rebuild and compare every non-financial notice",
        exact_current_stage_51_through_stage_122_binding_confirmed: true,
        validator_independent_from_executor_claimant_and_complete_prior_chain_confirmed: true,
        stage_122_result_and_create_once_candidate_reopened_and_rehashed_confirmed: true,
        exact_stage_114_admitted_observation_envelope_reopened_and_rehashed_confirmed: true,
        second_projection_does_not_call_stage_122_projector_helpers_confirmed: true,
        every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed: true,
        opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
        pass_only_opens_future_stage_124_non_financial_candidate_admission_review_confirmed: true,
        no_ledger_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    );

    expect(requests[0].url).toContain("observation-ledger-transition-output-validations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-output-validations/${"a".repeat(32)}/validate-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      second_projection_does_not_call_stage_122_projector_helpers_confirmed: true,
      opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
    });
  });
});

describe("Stage 124 non-financial observation candidate admission API", () => {
  test("uses the append-only review endpoint and preserves the empty financial boundary", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getControlledShadowObservationLedgerTransitionCandidateAdmissionReviews();
    await reviewControlledShadowObservationLedgerTransitionCandidateAdmission(
      "a".repeat(32),
      {
        expected_previous_review_id: null,
        expected_previous_review_sha256: null,
        expected_stage_123_validation_id: "1".repeat(32),
        expected_stage_123_validation_sha256: "2".repeat(64),
        expected_stage_122_result_sha256: "3".repeat(64),
        expected_stage_122_candidate_sha256: "4".repeat(64),
        expected_stage_121_claim_sha256: "5".repeat(64),
        expected_stage_114_review_sha256: "6".repeat(64),
        expected_stage_112_output_sha256: "7".repeat(64),
        verdict: "admitted_as_formal_non_financial_observation_evidence_for_future_opening_portfolio_governance",
        rationale: "reopen exact candidate and admit only separate non-financial evidence",
        known_limitations: "opening portfolio absent",
        exact_current_stage_51_through_stage_123_binding_confirmed: true,
        reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed: true,
        stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: true,
        stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: true,
        exact_stage_114_admitted_observation_binding_preserved_confirmed: true,
        every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: true,
        admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed: true,
        opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed: true,
        approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed: true,
        no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    );

    expect(requests[0].url).toContain("observation-ledger-transition-candidate-admission-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`observation-ledger-transition-candidate-admission-reviews/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed: true,
      approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed: true,
    });
  });
});

describe("Stage 125 opening portfolio snapshot governance specification API", () => {
  test("registers only a zero-capability source and canonical snapshot contract", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getOpeningPortfolioSnapshotGovernanceSpecifications();
    await registerOpeningPortfolioSnapshotGovernanceSpecification("a".repeat(32), {
      expected_stage_124_review_id: "a".repeat(32),
      expected_stage_124_review_sha256: "1".repeat(64),
      expected_stage_123_validation_sha256: "2".repeat(64),
      expected_stage_122_candidate_sha256: "3".repeat(64),
      expected_stage_114_review_sha256: "4".repeat(64),
      expected_stage_112_output_sha256: "5".repeat(64),
      source_kind: "broker_or_custodian_machine_export",
      source_provider_name: "independent custodian",
      portfolio_scope_alias: "primary_portfolio",
      reporting_currency: "USD",
      source_timezone: "America/New_York",
      snapshot_as_of_utc: "2026-08-28T20:00:00Z",
      expected_account_count: 1,
      registration_reason: "define the exact future external source contract",
      known_limitations: "no source artifact or financial state exists",
      future_review_constraints: "Stage 126 must independently reopen and review this specification",
      exact_current_stage_51_through_stage_124_binding_confirmed: true,
      registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed: true,
      stage_124_admission_reopened_rehashed_and_current_confirmed: true,
      external_source_artifact_required_and_manual_balances_forbidden_confirmed: true,
      account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed: true,
      all_cash_positions_liabilities_and_unsettled_activity_required_confirmed: true,
      exact_decimal_signed_quantities_and_no_default_or_inference_confirmed: true,
      instrument_identity_and_corporate_action_reconciliation_required_confirmed: true,
      statement_market_values_are_informational_not_accounting_marks_confirmed: true,
      complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed: true,
      source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed: true,
      specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: true,
      no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      future_stage_126_independent_specification_review_required_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("controlled-shadow-opening-portfolio-snapshot-governance-specifications");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      external_source_artifact_required_and_manual_balances_forbidden_confirmed: true,
      specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: true,
      future_stage_126_independent_specification_review_required_confirmed: true,
    });
  });
});

describe("Stage 126 opening portfolio snapshot governance specification review API", () => {
  test("independently reviews the complete contract without receiving a source artifact", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getOpeningPortfolioSnapshotGovernanceSpecificationReviews();
    await reviewOpeningPortfolioSnapshotGovernanceSpecification("a".repeat(32), {
      expected_registration_sha256: "1".repeat(64),
      expected_specification_sha256: "2".repeat(64),
      expected_independent_audit_sha256: "3".repeat(64),
      verdict: "approved_for_future_zero_capability_source_artifact_receipt_implementation_registration",
      rationale: "independent rebuild matched",
      binding_and_second_implementation_assessment: "complete binding matched",
      source_artifact_and_identity_assessment: "original bytes and identity are mandatory",
      account_scope_and_snapshot_completeness_assessment: "all accounts and fields are mandatory",
      valuation_and_nav_prerequisite_assessment: "independent marks FX and derivatives are required",
      zero_capability_assessment: "no artifact or financial state exists",
      known_limitations: "no source artifact has been received",
      future_implementation_constraints: "Stage 127 remains a zero-capability registration",
      exact_current_stage_51_through_stage_125_binding_confirmed: true,
      reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
      registration_and_specification_hashes_independently_reproduced_confirmed: true,
      complete_specification_rebuilt_without_stage_125_builder_confirmed: true,
      rebuilt_specification_exactly_matches_registered_specification_confirmed: true,
      original_external_artifact_provenance_and_pseudonymization_contract_confirmed: true,
      complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: true,
      exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed: true,
      instrument_identity_cost_basis_and_corporate_action_contract_confirmed: true,
      statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: true,
      source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed: true,
      no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: true,
      no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("opening-portfolio-snapshot-governance-specification-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: true,
      no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: true,
    });
  });
});

describe("Stage 127 opening portfolio source artifact receipt implementation API", () => {
  test("registers only a zero-capability contract and never uploads source bytes", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getOpeningPortfolioSourceArtifactReceiptImplementations();
    await registerOpeningPortfolioSourceArtifactReceiptImplementation("a".repeat(32), {
      expected_stage_126_review_id: "a".repeat(32),
      expected_stage_126_review_sha256: "1".repeat(64),
      expected_stage_126_independent_audit_sha256: "2".repeat(64),
      expected_stage_125_registration_id: "b".repeat(32),
      expected_stage_125_registration_sha256: "3".repeat(64),
      expected_stage_125_specification_sha256: "4".repeat(64),
      implementation_name: "private source receipt contract",
      immutable_code_revision: "revision-127",
      implementation_description: "contract only",
      transport_and_authentication_semantics: "authenticated stream only",
      streaming_hash_length_and_atomic_commit_semantics: "stream SHA-256 and length before create-new",
      format_magic_and_active_content_rejection_semantics: "validate magic and reject active content",
      pseudonymization_and_secret_redaction_semantics: "pseudonymize accounts and redact credentials",
      quarantine_cleanup_and_idempotency_semantics: "private quarantine, cleanup and idempotency",
      audit_and_retention_semantics: "redacted append-only manifest",
      known_limitations: "no upload endpoint or source artifact",
      future_review_constraints: "Stage 128 independent review required",
      exact_current_stage_51_through_stage_126_binding_confirmed: true,
      registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: true,
      review_registration_specification_and_audit_hashes_recomputed_confirmed: true,
      exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: true,
      original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: true,
      content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed: true,
      archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed: true,
      source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed: true,
      private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: true,
      server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed: true,
      duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: true,
      receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: true,
      receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed: true,
      contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: true,
      no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      future_stage_128_independent_implementation_review_required_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("opening-portfolio-source-artifact-receipt-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: true,
      future_stage_128_independent_implementation_review_required_confirmed: true,
    });
  });
});

describe("Stage 128 opening portfolio source artifact receipt implementation review API", () => {
  test("reviews an independently rebuilt contract without uploading or parsing source bytes", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getOpeningPortfolioSourceArtifactReceiptImplementationReviews();
    await reviewOpeningPortfolioSourceArtifactReceiptImplementation("a".repeat(32), {
      expected_implementation_sha256: "1".repeat(64),
      expected_implementation_contract_sha256: "2".repeat(64),
      expected_stage_126_review_sha256: "3".repeat(64),
      expected_stage_126_independent_audit_sha256: "4".repeat(64),
      expected_stage_125_registration_sha256: "5".repeat(64),
      expected_stage_125_specification_sha256: "6".repeat(64),
      expected_independent_audit_sha256: "7".repeat(64),
      verdict: "approved_for_future_isolated_source_artifact_receiver_specification_registration",
      rationale: "independent rebuild matched",
      binding_and_recomputation_assessment: "all upstream hashes reproduced",
      transport_resource_and_format_assessment: "formats and ceilings matched",
      privacy_storage_and_manifest_assessment: "private quarantine and redacted manifest matched",
      separation_and_zero_capability_assessment: "all data and financial authority remains closed",
      known_limitations: "no source artifact has been received",
      future_receiver_constraints: "Stage 129 may register only an isolated receiver specification",
      confirmations: {
        exact_current_stage_51_through_stage_127_binding_confirmed: true,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
        implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: true,
        complete_contract_rebuilt_without_stage_127_builder_confirmed: true,
        all_stage_127_registration_confirmations_revalidated_confirmed: true,
        original_provider_formats_and_resource_ceilings_preserved_confirmed: true,
        administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: true,
        streaming_sha256_length_private_quarantine_and_atomic_commit_confirmed: true,
        format_magic_safe_structure_and_active_content_rejection_confirmed: true,
        account_pseudonymization_and_secret_redaction_confirmed: true,
        encryption_content_addressing_create_new_idempotency_and_failure_cleanup_confirmed: true,
        server_received_time_redacted_manifest_and_untrusted_receipt_confirmed: true,
        receipt_validation_materialization_output_validation_and_admission_remain_separate_confirmed: true,
        no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed: true,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
        approval_only_opens_future_stage_129_isolated_receiver_specification_registration_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    });

    expect(requests[0].url).toContain("opening-portfolio-source-artifact-receipt-implementation-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${"a".repeat(32)}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      verdict: "approved_for_future_isolated_source_artifact_receiver_specification_registration",
      confirmations: {
        complete_contract_rebuilt_without_stage_127_builder_confirmed: true,
        no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed: true,
      },
    });
  });
});

describe("Stage 129 opening portfolio source artifact receipt isolated receiver API", () => {
  test("registers only a future receiver specification and never sends source bytes", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;

    await getOpeningPortfolioSourceArtifactReceiptIsolatedReceivers();
    await registerOpeningPortfolioSourceArtifactReceiptIsolatedReceiver("a".repeat(32), {
      expected_stage_128_review_id: "b".repeat(32), expected_stage_128_review_sha256: "1".repeat(64), expected_stage_128_independent_audit_sha256: "2".repeat(64),
      expected_stage_127_implementation_id: "a".repeat(32), expected_stage_127_implementation_sha256: "3".repeat(64), expected_stage_127_implementation_contract_sha256: "4".repeat(64),
      expected_stage_126_review_sha256: "5".repeat(64), expected_stage_126_independent_audit_sha256: "6".repeat(64), expected_stage_125_registration_sha256: "7".repeat(64), expected_stage_125_specification_sha256: "8".repeat(64),
      receiver_name: "isolated stream receiver", receiver_kind: "ephemeral_deterministic_stream_only_receipt_specification", receiver_spec_revision: "v1",
      proposed_receiver_code_revision: "revision-129", proposed_receiver_artifact_sha256: "9".repeat(64), artifact_reproduction_procedure: "reproduce then hash",
      rationale: "freeze execution boundary", known_limitations: "artifact absent", future_input_constraints: "admin stream only", future_output_constraints: "untrusted create-once manifest",
      exact_current_stage_51_through_stage_128_binding_confirmed: true, registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: true,
      review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed: true, proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: true,
      all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: true, exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: true,
      future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: true, future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: true,
      future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed: true, future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: true,
      future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: true, future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true, no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("opening-portfolio-source-artifact-receipt-isolated-receivers");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${"a".repeat(32)}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
    expect(JSON.parse(String(requests[1].init?.body))).toMatchObject({
      proposed_receiver_artifact_sha256: "9".repeat(64),
      registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: true,
    });
  });
});

describe("Stage 130 opening portfolio source artifact receipt first execution authorization API", () => {
  test("reviews only a server-custodied receiver identity and sends no source bytes", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const id = "a".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizations();
    await reviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization(id, {
      expected_isolated_receiver_id: id, expected_isolated_receiver_spec_sha256: sha,
      expected_receiver_contract_sha256: sha, expected_receiver_spec_revision: "stage-129-v1",
      expected_receiver_code_revision: "immutable-revision", expected_receiver_artifact_sha256: sha,
      expected_stage_128_review_id: id, expected_stage_128_review_sha256: sha,
      expected_stage_128_independent_audit_sha256: sha, expected_stage_127_implementation_sha256: sha,
      expected_stage_127_implementation_contract_sha256: sha, expected_stage_126_review_sha256: sha,
      expected_stage_125_registration_sha256: sha, expected_stage_125_specification_sha256: sha,
      expected_artifact_manifest_sha256: sha, artifact_reproduction_review_evidence: "reproduced independently",
      sandbox_contract_review_evidence: "sandbox contract rechecked", verdict: "rejected", rationale: "fixture",
      exact_current_stage_51_through_stage_129_binding_confirmed: false,
      reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed: true,
      server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: false,
      self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: false,
      artifact_builder_and_reviewer_separation_confirmed: true,
      all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: false,
      exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: false,
      future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: false,
      future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: false,
      future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: false,
      future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: false,
      future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed: false,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: false,
      authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: false,
      no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: true,
      no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      approval_only_opens_future_stage_131_claim_first_attempt_confirmed: false,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("first-execution-authorizations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${id}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
  });
});

describe("Stage 132 opening portfolio source artifact receipt one-shot API", () => {
  test("puts redacted metadata before source files and never serializes a remote URL", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const attemptId = "a".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSourceArtifactReceiptExecutionAttempts();
    await receiveOpeningPortfolioSourceArtifactReceiptAttemptOnce(attemptId, {
      expected_claim_sha256: sha, expected_authorization_review_sha256: sha,
      expected_isolated_receiver_spec_sha256: sha, expected_receiver_contract_sha256: sha,
      expected_receiver_artifact_sha256: sha, expected_artifact_manifest_sha256: sha,
      expected_implementation_contract_sha256: sha, expected_stage_125_specification_sha256: sha,
      provider_statement_or_export_identifier: "statement-2026Q2",
      provider_generated_at_or_statement_as_of: "2026-08-29T00:00:00Z",
      artifacts: [{ declared_format: "original_provider_json_export", source_account_aliases: ["broker_main"] }],
      execution_reason: "一次性接收脱敏测试工件",
      exact_current_stage_51_through_stage_131_binding_confirmed: true,
      executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed: true,
      start_marker_consumes_claim_before_first_source_byte_confirmed: true,
      administrator_authenticated_stream_only_no_remote_fetch_confirmed: true,
      original_artifacts_already_account_pseudonymized_and_credentials_removed_confirmed: true,
      format_magic_safe_structure_archive_active_content_password_symlink_and_path_rejection_confirmed: true,
      streaming_sha256_length_private_quarantine_and_atomic_content_addressed_commit_confirmed: true,
      encryption_at_rest_and_redacted_manifest_confirmed: true,
      duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: true,
      receipt_create_once_untrusted_and_stage_133_independent_validation_required_confirmed: true,
      no_financial_row_parsing_snapshot_materialization_or_snapshot_admission_confirmed: true,
      no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    }, [new File(["[{\"symbol\":\"SNDK\"}]"], "export.json", { type: "application/json" })]);

    expect(requests[0].url).toContain("source-artifact-receipt-execution-attempts");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${attemptId}/receive-once`);
    expect(requests[1].init?.method).toBe("POST");
    const body = requests[1].init?.body as FormData;
    expect(Array.from(body.keys())).toEqual(["request", "artifact"]);
    expect(String(body.get("request"))).not.toContain("http");
    expect(body.get("artifact")).toBeInstanceOf(File);
  });
});

describe("Stage 133 opening portfolio source artifact receipt independent validation API", () => {
  test("uses an authenticated JSON mutation and sends only expected hashes and confirmations", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const attemptId = "a".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSourceArtifactReceiptValidations();
    await validateOpeningPortfolioSourceArtifactReceiptOnce(attemptId, {
      expected_stage_131_claim_sha256: sha,
      expected_stage_132_result_sha256: sha,
      expected_receipt_manifest_sha256: sha,
      expected_stage_130_authorization_review_sha256: sha,
      expected_stage_129_isolated_receiver_spec_sha256: sha,
      expected_stage_127_implementation_contract_sha256: sha,
      expected_stage_125_specification_sha256: sha,
      validation_reason: "责任链外独立验证加密来源工件 receipt",
      exact_stage_51_through_stage_132_chain_reopened_confirmed: true,
      validator_independent_from_stage_132_executor_stage_131_claimant_and_complete_prior_chain_confirmed: true,
      result_and_receipt_fingerprints_independently_recomputed_confirmed: true,
      server_derived_manifest_and_content_addressed_paths_only_confirmed: true,
      ciphertext_regular_read_only_size_and_sha256_recomputed_confirmed: true,
      encryption_key_fingerprint_and_aead_authenticated_decryption_confirmed: true,
      plaintext_length_sha256_and_content_address_independently_recomputed_confirmed: true,
      format_magic_safe_structure_and_sensitive_field_screening_independently_repeated_confirmed: true,
      receipt_redaction_and_no_original_filename_account_number_or_credential_confirmed: true,
      terminal_create_once_validation_no_replay_confirmed: true,
      receipt_validation_only_no_financial_row_parsing_or_snapshot_materialization_confirmed: true,
      no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("source-artifact-receipt-validations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${attemptId}/validate-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
  });
});

describe("Stage 134 opening portfolio snapshot materialization zero-capability API", () => {
  test("registers only hashes, contract semantics and closed-authority confirmations", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const validationId = "a".repeat(32);
    const receiptId = "c".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSnapshotMaterializationImplementations();
    await registerOpeningPortfolioSnapshotMaterializationImplementation(validationId, {
      expected_stage_133_validation_id: validationId,
      expected_stage_133_validation_sha256: sha,
      expected_stage_132_result_sha256: sha,
      expected_stage_131_claim_sha256: sha,
      expected_receipt_id: receiptId,
      expected_receipt_manifest_sha256: sha,
      expected_stage_125_specification_sha256: sha,
      implementation_name: "deterministic materializer",
      immutable_code_revision: "revision-1",
      implementation_description: "contract only",
      deterministic_parser_and_adapter_semantics: "deterministic provider adapters",
      account_scope_and_completeness_semantics: "all accounts and sections",
      exact_decimal_and_signed_quantity_semantics: "decimal strings only",
      instrument_identity_and_corporate_action_semantics: "identity precedence and reconciliation",
      row_provenance_and_redaction_semantics: "artifact hash and source locator",
      whole_snapshot_failure_and_correction_semantics: "fail whole snapshot and create new correction",
      known_limitations: "not implemented or run",
      future_review_constraints: "second implementation required",
      exact_current_stage_51_through_stage_133_binding_confirmed: true,
      registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed: true,
      validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: true,
      exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: true,
      future_input_only_independently_validated_content_addressed_receipt_confirmed: true,
      future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: true,
      deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: true,
      account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: true,
      exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: true,
      instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: true,
      no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed: true,
      statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
      every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed: true,
      future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed: true,
      contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: true,
      no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      future_stage_135_chain_external_independent_implementation_review_required_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });
    expect(requests[0].url).toContain("snapshot-materialization-implementations");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${validationId}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
    expect(String(requests[1].init?.body)).not.toContain("decryption_key");
  });
});

describe("Stage 135 opening portfolio snapshot materialization independent review API", () => {
  test("sends only immutable hashes, assessments and closed-authority confirmations", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const implementationId = "a".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSnapshotMaterializationImplementationReviews();
    await reviewOpeningPortfolioSnapshotMaterializationImplementation(implementationId, {
      expected_implementation_sha256: sha,
      expected_implementation_contract_sha256: sha,
      expected_stage_133_validation_sha256: sha,
      expected_stage_132_result_sha256: sha,
      expected_stage_131_claim_sha256: sha,
      expected_receipt_manifest_sha256: sha,
      expected_stage_125_specification_sha256: sha,
      expected_independent_audit_sha256: sha,
      verdict: "approved_for_future_isolated_materializer_specification_registration",
      rationale: "第二实现完整重建合同并精确匹配",
      binding_and_recomputation_assessment: "Stage 125/131/132/133/134 摘要全部独立重算",
      parser_schema_and_completeness_assessment: "三种确定性适配器与完整账户结构匹配",
      decimal_identity_and_provenance_assessment: "精确十进制、证券身份与逐行来源匹配",
      failure_separation_and_zero_capability_assessment: "整批失败且全部执行与财务能力关闭",
      known_limitations: "尚无可执行物化器或真实来源读取",
      future_materializer_constraints: "Stage 136 仍只能登记隔离物化器规格",
      confirmations: {
        exact_current_stage_51_through_stage_134_binding_confirmed: true,
        reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed: true,
        implementation_contract_validation_result_claim_receipt_and_specification_hashes_independently_reproduced_confirmed: true,
        complete_contract_rebuilt_without_stage_134_builder_confirmed: true,
        all_stage_134_registration_confirmations_revalidated_confirmed: true,
        input_only_independently_validated_content_addressed_receipt_confirmed: true,
        future_decryption_only_in_isolated_ephemeral_memory_confirmed: true,
        deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: true,
        complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: true,
        exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: true,
        instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: true,
        no_default_manual_or_inferred_financial_values_and_whole_snapshot_failure_confirmed: true,
        statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
        every_output_row_bound_to_artifact_hash_and_source_locator_with_redaction_confirmed: true,
        output_create_once_untrusted_and_separate_validation_and_admission_confirmed: true,
        no_key_input_read_decrypt_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: true,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
        approval_only_opens_future_stage_136_isolated_materializer_specification_registration_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      },
    });

    expect(requests[0].url).toContain("snapshot-materialization-implementation-reviews");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${implementationId}/review`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("decryption_key");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
  });
});

describe("Stage 136 opening portfolio snapshot isolated materializer API", () => {
  test("sends only immutable identities, constraints and zero-capability confirmations", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ url: String(url), init });
      return Promise.resolve(new Response("{}", { headers: { "content-type": "application/json" } }));
    }) as typeof fetch;
    const implementationId = "a".repeat(32);
    const sha = "b".repeat(64);
    await getOpeningPortfolioSnapshotMaterializationIsolatedMaterializers();
    await registerOpeningPortfolioSnapshotMaterializationIsolatedMaterializer(implementationId, {
      expected_stage_135_review_id: "c".repeat(32),
      expected_stage_135_review_sha256: sha,
      expected_stage_135_independent_audit_sha256: sha,
      expected_stage_134_implementation_id: implementationId,
      expected_stage_134_implementation_sha256: sha,
      expected_stage_134_implementation_contract_sha256: sha,
      expected_stage_133_validation_sha256: sha,
      expected_stage_132_result_sha256: sha,
      expected_stage_131_claim_sha256: sha,
      expected_receipt_manifest_sha256: sha,
      expected_stage_125_specification_sha256: sha,
      materializer_name: "期初组合快照隔离物化器",
      materializer_kind: "ephemeral_deterministic_pdf_csv_json_snapshot_materialization_specification",
      materializer_spec_revision: "v1",
      proposed_materializer_code_revision: "immutable-revision",
      proposed_materializer_artifact_sha256: sha,
      artifact_reproduction_procedure: "固定源码、依赖和构建参数后复现工件 SHA-256",
      rationale: "只冻结未来物化器身份与边界",
      known_limitations: "工件不存在且未获执行授权",
      future_input_constraints: "只接受 Stage 133 独立验证的内容寻址加密 receipt",
      future_output_constraints: "只允许 create-once 不可信候选并另行验证",
      exact_current_stage_51_through_stage_135_binding_confirmed: true,
      registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed: true,
      complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed: true,
      exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed: true,
      future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed: true,
      deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: true,
      statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    });

    expect(requests[0].url).toContain("snapshot-materialization-isolated-materializers");
    expect(requests[0].init?.cache).toBe("no-store");
    expect(requests[1].url).toContain(`/${implementationId}/register-once`);
    expect(requests[1].init?.method).toBe("POST");
    expect(String(requests[1].init?.body)).not.toContain("decryption_key");
    expect(String(requests[1].init?.body)).not.toContain("source_artifact_bytes");
  });
});

describe("public community API", () => {
  test("uses an opaque content cursor and returns read-only timeline data", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 42, body_text: "hello", resources: [] }],
            next_before: 42,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity({ before: 88, limit: 20 });

    expect(requestedUrl).toContain("/api/public/community?before=88&limit=20");
    expect(payload.items[0]?.content_id).toBe(42);
    expect(payload.unread).toBe(true);
  });

  test("uses the private edge feed only after a prefer grant", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requested.push(String(url));
      if (String(url).includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (String(url).includes("/api/public/community/state")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({ unread: true, latest_content_id: 42 }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [
              {
                content_id: 42,
                body_text: "edge",
                resources: [
                  {
                    resource_id: 99,
                    version: "0123456789ab",
                    delivery_path:
                      "/_community/v1/resources/99/0123456789ab",
                  },
                ],
              },
            ],
            next_before: null,
            unread: false,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(requested).toHaveLength(3);
    expect(requested[0]).toContain("/api/public/community/edge-session");
    expect(requested).toContainEqual(
      expect.stringContaining("/_community/v1/feed/latest.json"),
    );
    expect(requested).toContainEqual(
      expect.stringContaining("/api/public/community/state"),
    );
    expect(payload.items[0]?.body_text).toBe("edge");
    expect(payload.unread).toBe(true);
    expect(payload.latest_content_id).toBe(42);
    expect(
      publicCommunityResourceUrl(
        99,
        "0123456789ab",
        payload.items[0]?.resources[0]?.delivery_path,
      ),
    ).toContain("/_community/v1/resources/99/0123456789ab");
  });

  test("falls back immediately to the legacy feed when the edge is unavailable", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/_community/v1/")) {
        return Promise.resolve(
          new Response("edge unavailable", {
            status: 503,
            statusText: "Service Unavailable",
          }),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 41, body_text: "legacy", resources: [] }],
            next_before: null,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(
      requested.filter((url) => url.includes("/_community/v1/feed/latest.json")),
    ).toHaveLength(1);
    expect(requested.at(-1)).toContain("/api/public/community");
    expect(payload.items[0]?.body_text).toBe("legacy");
  });

  test("falls back to the legacy feed when personal state is unavailable", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/_community/v1/")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              community: { id: "51115212285814", name: "HONE 官方社区" },
              items: [{ content_id: 42, body_text: "edge", resources: [] }],
              next_before: null,
              unread: false,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/api/public/community/state")) {
        return Promise.resolve(new Response("state unavailable", { status: 503 }));
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 41, body_text: "legacy", resources: [] }],
            next_before: null,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(
      requested.filter((url) => url.endsWith("/api/public/community")),
    ).toHaveLength(1);
    expect(payload.items[0]?.body_text).toBe("legacy");
    expect(payload.unread).toBe(true);
  });

  test("discovers a fresh edge grant after logout", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    let grants = 0;
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      if (raw.includes("/edge-session")) {
        grants += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/auth/logout")) {
        return Promise.resolve(
          new Response(JSON.stringify({ ok: true }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(
          new Response(JSON.stringify({ unread: false, latest_content_id: 42 }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [],
            next_before: null,
            unread: false,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getPublicCommunity();
    await publicLogout();
    await getPublicCommunity();

    expect(grants).toBe(2);
  });

  test("marks the latest community content as seen without sending a social action", async () => {
    let body = "";
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      body = String(init?.body);
      return Promise.resolve(
        new Response(JSON.stringify({ ok: true }), {
          headers: { "content-type": "application/json" },
        }),
      );
    }) as typeof fetch;

    await markPublicCommunitySeen(42);

    expect(body).toBe('{"content_id":42}');
  });

  test("downloads a protected community resource through the authenticated API", async () => {
    let requestedUrl = "";
    let credentials: RequestCredentials | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      credentials = init?.credentials;
      return Promise.resolve(
        new Response(new Uint8Array([1, 2, 3]), {
          headers: { "content-type": "image/jpeg" },
        }),
      );
    }) as typeof fetch;

    const blob = await getPublicCommunityResourceBlob(99, "0123456789ab");

    expect(requestedUrl).toContain(
      "/api/public/community/resources/99?v=0123456789ab",
    );
    expect(credentials).toBe("include");
    expect(blob.size).toBe(3);
  });

  test("downloads a generated file through the authenticated public route", async () => {
    let requestedUrl = "";
    let credentials: RequestCredentials | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      credentials = init?.credentials;
      return Promise.resolve(
        new Response(new Uint8Array([37, 80, 68, 70, 45]), {
          headers: { "content-type": "application/pdf" },
        }),
      );
    }) as typeof fetch;

    const blob = await getPublicGeneratedFileBlob(
      "<absolute-path>/ANET-财报前瞻.pdf",
    );

    expect(requestedUrl).toContain(
      "/api/public/file?path=%3Cabsolute-path%3E%2FANET-%E8%B4%A2%E6%8A%A5%E5%89%8D%E7%9E%BB.pdf",
    );
    expect(credentials).toBe("include");
    expect(blob.type).toBe("application/pdf");
    expect(blob.size).toBe(5);
  });

  test("keeps legacy resources revalidating while versioning hashed resources", () => {
    expect(publicCommunityResourceUrl(99)).toContain(
      "/api/public/community/resources/99",
    );
    expect(publicCommunityResourceUrl(99)).not.toContain("?v=");
    expect(publicCommunityResourceUrl(99, "0123456789ab")).toContain(
      "/api/public/community/resources/99?v=0123456789ab",
    );
  });

  test("rejects mismatched edge paths and preflights document fallback", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: Array<{ url: string; method: string }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      const raw = String(url);
      requested.push({ url: raw, method: init?.method ?? "GET" });
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/feed/latest.json")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              community: { id: "51115212285814", name: "HONE 官方社区" },
              items: [],
              next_before: null,
              unread: false,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/api/public/community/state")) {
        return Promise.resolve(
          new Response(JSON.stringify({ unread: false, latest_content_id: null }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      return Promise.resolve(new Response("edge unavailable", { status: 503 }));
    }) as typeof fetch;
    await getPublicCommunity();

    expect(
      publicCommunityResourceUrl(
        99,
        "0123456789ab",
        "/_community/v1/resources/100/0123456789ab",
      ),
    ).toContain("/api/public/community/resources/99?v=0123456789ab");
    expect(
      publicCommunityResourceUrl(99, "0123456789ab", "https://evil.example/x"),
    ).toContain("/api/public/community/resources/99?v=0123456789ab");

    const resolved = await resolvePublicCommunityResourceUrl(
      99,
      "0123456789ab",
      "/_community/v1/resources/99/0123456789ab",
    );
    expect(requested.at(-1)?.method).toBe("HEAD");
    expect(resolved).toContain("/api/public/community/resources/99?v=0123456789ab");
  });

  test("corrects a source-mislabeled OOXML workbook download extension", () => {
    expect(
      publicCommunityResourceDownloadName({
        resource_id: 295,
        display_name: "投资组合.xls",
        content_type:
          "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      }),
    ).toBe("投资组合.xlsx");
    expect(
      publicCommunityResourceDownloadName({
        resource_id: 1,
        display_name: "报告.pdf",
        content_type: "application/pdf",
      }),
    ).toBe("报告.pdf");
  });
});

describe("public finance calendar API", () => {
  test("loads a selected calendar month", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            today: "2026-06-29",
            month: "2026-07",
            months: [],
            holdings: [],
            events: [],
            earnings_status: "empty_portfolio",
            errors: [],
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await getPublicFinanceCalendar("2026-07");

    expect(requestedUrl).toContain("/api/public/finance-calendar?month=2026-07");
    expect(payload.month).toBe("2026-07");
  });

  test("sends a rendered calendar image", async () => {
    let requestBody: unknown;
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      requestBody = JSON.parse(String(init?.body ?? "{}"));
      return Promise.resolve(
        new Response(JSON.stringify({ ok: true, message: "done" }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    }) as unknown as typeof fetch;

    const result = await sendPublicFinanceCalendar({
      path: "/tmp/public/web-user/calendar.png",
      mobile_path: "/tmp/public/web-user/calendar-mobile.png",
      month: "2026-07",
    });

    expect(requestBody).toEqual({
      path: "/tmp/public/web-user/calendar.png",
      mobile_path: "/tmp/public/web-user/calendar-mobile.png",
      month: "2026-07",
    });
    expect(result.ok).toBe(true);
  });
});

describe("public push API", () => {
  test("loads a cursor page of scheduled pushes", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({ items: [], unread_count: 2, next_before: null }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await getPublicPushes("job-1:2026-07-10:20:00", 20);

    expect(requestedUrl).toContain("/api/public/pushes?");
    expect(requestedUrl).toContain("limit=20");
    expect(requestedUrl).toContain(
      "before=job-1%3A2026-07-10%3A20%3A00",
    );
    expect(payload.unread_count).toBe(2);
  });

  test("opens a push through a POST action", async () => {
    let requestedUrl = "";
    let requestedMethod = "";
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedMethod = init?.method ?? "GET";
      return Promise.resolve(
        new Response(
          JSON.stringify({
            push: {
              push_id: "job-1:2026-07-10:20:00",
              job_id: "job-1",
              title: "收盘复盘",
              summary: "摘要",
              content: "完整内容",
              created_at: "2026-07-10T20:00:00+08:00",
            },
            unread_count: 0,
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await openPublicPush("job-1:2026-07-10:20:00");

    expect(requestedMethod).toBe("POST");
    expect(requestedUrl).toContain(
      "/api/public/pushes/job-1%3A2026-07-10%3A20%3A00/open",
    );
    expect(payload.unread_count).toBe(0);
    expect(payload.push.content).toBe("完整内容");
  });
});
