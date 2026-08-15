import { Show, createMemo, createSignal, onCleanup, onMount, type ParentProps } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useNavigate } from "@solidjs/router";
import { HoneBrand } from "@/components/hone-brand";
import { PublicCheckbox } from "@/components/public-checkbox";
import { PublicPrefsButton } from "@/components/public-prefs-button";
import {
  ApiError,
  createStripeCheckout,
  getPublicBillingConfig,
  publicEmailLogin,
  publicSendEmailCode,
  type StripeCheckoutOffer,
} from "@/lib/api";
import { TOS_VERSION } from "@/lib/tos";
import type { PublicBillingConfig } from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export default function PublicActivatePage() {
  const navigate = useNavigate();
  const [config, setConfig] = createSignal<PublicBillingConfig>();
  const [selectedOffer, setSelectedOffer] = createSignal<StripeCheckoutOffer>("subscription");
  const [emailAddress, setEmailAddress] = createSignal("");
  const [verifyCode, setVerifyCode] = createSignal("");
  const [remember, setRemember] = createSignal(true);
  const [agreed, setAgreed] = createSignal(false);
  const [sending, setSending] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [cooldown, setCooldown] = createSignal(0);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const configReady = createMemo(() => config() !== undefined);
  const normalizedEmail = createMemo(() => emailAddress().trim().toLowerCase());
  const emailOk = createMemo(() => EMAIL_PATTERN.test(normalizedEmail()));
  const codeOk = createMemo(() => /^[0-9]{6,12}$/.test(verifyCode().trim()));
  const sendReady = createMemo(
    () => config() !== undefined && emailOk() && !sending() && cooldown() <= 0,
  );
  const loginReady = createMemo(
    () => configReady() && emailOk() && codeOk() && agreed() && !submitting(),
  );
  const restoreOnly = createMemo(
    () => config()?.purchases_allowed_on_this_client === false,
  );
  const purchaseAvailable = createMemo(
    () =>
      !restoreOnly() &&
      (config()?.stripe.subscription.enabled === true ||
        config()?.stripe.fixed_term.enabled === true),
  );
  const selectedOfferAvailable = createMemo(() => {
    const stripe = config()?.stripe;
    return selectedOffer() === "subscription"
      ? stripe?.subscription.enabled === true
      : stripe?.fixed_term.enabled === true;
  });
  const advertisedPaymentMethods = (offer: StripeCheckoutOffer) => {
    const methods = config()?.stripe[offer].advertised_payment_methods;
    if (!methods) return "";
    const labels = [
      methods.card ? CONTENT.chat_page.activate_page.payment_method_card : "",
      methods.alipay ? CONTENT.chat_page.activate_page.payment_method_alipay : "",
      methods.wechat_pay ? CONTENT.chat_page.activate_page.payment_method_wechat_pay : "",
    ].filter(Boolean);
    return `${CONTENT.chat_page.activate_page.payment_methods_prefix}${labels.join(
      CONTENT.chat_page.activate_page.payment_methods_separator,
    )}`;
  };
  const activationSteps = createMemo(() => {
    if (!purchaseAvailable()) return [CONTENT.chat_page.activate_page.step_verify, CONTENT.chat_page.activate_page.step_login, CONTENT.chat_page.activate_page.step_restore];
    return [CONTENT.chat_page.activate_page.step_verify, CONTENT.chat_page.activate_page.step_pay, CONTENT.chat_page.activate_page.step_confirm];
  });

  let cooldownTimer: ReturnType<typeof setInterval> | undefined;
  onMount(async () => {
    try {
      setConfig(await getPublicBillingConfig());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  });
  onCleanup(() => {
    if (cooldownTimer) clearInterval(cooldownTimer);
  });

  const startCooldown = () => {
    setCooldown(60);
    if (cooldownTimer) clearInterval(cooldownTimer);
    cooldownTimer = setInterval(() => {
      setCooldown((value) => {
        if (value <= 1) {
          if (cooldownTimer) clearInterval(cooldownTimer);
          cooldownTimer = undefined;
          return 0;
        }
        return value - 1;
      });
    }, 1000);
  };

  const sendCode = async () => {
    if (!sendReady()) return;
    setError("");
    setNotice("");
    setSending(true);
    try {
      const response = await publicSendEmailCode(
        normalizedEmail(),
        purchaseAvailable() ? "stripe_checkout" : undefined,
      );
      setNotice(response.message);
      startCooldown();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSending(false);
    }
  };

  const submit = async () => {
    if (!loginReady()) return;
    setError("");
    setNotice("");
    setSubmitting(true);
    try {
      const user = await publicEmailLogin({
        email_address: normalizedEmail(),
        verify_code: verifyCode().trim(),
        remember: remember(),
        tos_version: TOS_VERSION,
      });
      if (restoreOnly() || user.billing.access_granted) {
        navigate("/me?billing=syncing");
        return;
      }
      if (!purchaseAvailable()) {
        setError(CONTENT.chat_page.activate_page.checkout_down);
        return;
      }
      if (!selectedOfferAvailable()) {
        setError(CONTENT.chat_page.activate_page.checkout_down);
        return;
      }
      const { checkout_url } = await createStripeCheckout(selectedOffer());
      window.location.assign(checkout_url);
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 409) {
        navigate("/me");
        return;
      }
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSubmitting(false);
    }
  };

  const title = () => {
    if (!configReady()) return CONTENT.chat_page.activate_page.confirming;
    if (!purchaseAvailable()) return CONTENT.chat_page.activate_page.restore_title;
    return CONTENT.chat_page.activate_page.checkout_title;
  };

  return (
    <main class="public-login-screen public-activate">
      <div class="public-login-preferences"><PublicPrefsButton /></div>
      <div class="public-activate-inner">
        <header class="public-activate-header">
          <HoneBrand class="public-login-brand" />
          <span class="public-activate-provider">
            {!configReady()
              ? CONTENT.chat_page.activate_page.channel_pending
              : restoreOnly()
                ? CONTENT.chat_page.activate_page.restore_badge
                : purchaseAvailable()
                  ? CONTENT.chat_page.activate_page.checkout_badge
                  : CONTENT.chat_page.activate_page.restore_badge}
          </span>
          <h1>{title()}</h1>
          <p>
            {!configReady()
              ? CONTENT.chat_page.activate_page.reading_methods
              : restoreOnly()
                ? CONTENT.chat_page.activate_page.restore_hint
                : purchaseAvailable()
                  ? CONTENT.chat_page.activate_page.checkout_hint
                  : CONTENT.chat_page.activate_page.checkout_maint}
          </p>
        </header>

        <Show
          when={configReady()}
          fallback={
            <section class="public-activate-card">
              <Feedback
                message={error() || CONTENT.chat_page.activate_page.loading_channels}
                error={Boolean(error())}
              />
              <Show when={error()}>
                <button
                  class="public-activate-submit"
                  type="button"
                  onClick={() => window.location.reload()}
                >
                  {CONTENT.chat_page.activate_page.reload}
                </button>
              </Show>
            </section>
          }
        >
          <section class="public-activate-card">
            <div class="public-activate-steps" aria-label={CONTENT.chat_page.activate_page.steps_title}>
              {activationSteps().map((label, index) => <span>{index + 1}. {label}</span>)}
            </div>

            <Show when={purchaseAvailable()}>
              <fieldset class="public-activate-offers">
                <legend>{CONTENT.chat_page.activate_page.offer_legend}</legend>
                <OfferOption
                  offer="subscription"
                  selected={selectedOffer() === "subscription"}
                  enabled={config()?.stripe.subscription.enabled === true}
                  title={CONTENT.chat_page.activate_page.subscription_title}
                  price={CONTENT.chat_page.activate_page.subscription_price.replace(
                    "{price}",
                    formatAmount(config()?.stripe.subscription.amount_minor),
                  )}
                  description={CONTENT.chat_page.activate_page.subscription_desc}
                  methods={advertisedPaymentMethods("subscription")}
                  onSelect={setSelectedOffer}
                />
                <OfferOption
                  offer="fixed_term"
                  selected={selectedOffer() === "fixed_term"}
                  enabled={config()?.stripe.fixed_term.enabled === true}
                  title={CONTENT.chat_page.activate_page.fixed_term_title}
                  price={CONTENT.chat_page.activate_page.fixed_term_price.replace(
                    "{price}",
                    formatAmount(config()?.stripe.fixed_term.amount_minor),
                  )}
                  description={CONTENT.chat_page.activate_page.fixed_term_desc}
                  methods={advertisedPaymentMethods("fixed_term")}
                  onSelect={setSelectedOffer}
                />
              </fieldset>
            </Show>

            <label class="public-activate-field">
              <FieldLabel>{CONTENT.chat_page.activate_page.account_email}</FieldLabel>
              <TextInput
                type="email"
                value={emailAddress()}
                onInput={setEmailAddress}
                placeholder="name@example.com"
                autoComplete="email"
                ariaLabel={CONTENT.chat_page.activate_page.account_email}
              />
            </label>

            <div class="public-activate-code-row">
              <label class="public-activate-field">
                <FieldLabel>{CONTENT.chat_page.activate_page.email_code}</FieldLabel>
                <TextInput
                  value={verifyCode()}
                  onInput={(value) => setVerifyCode(value.replace(/\D/g, "").slice(0, 12))}
                  placeholder={CONTENT.chat_page.activate_page.code_placeholder}
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  ariaLabel={CONTENT.chat_page.activate_page.email_code}
                  onEnter={submit}
                />
              </label>
              <button type="button" disabled={!sendReady()} onClick={sendCode}>
                {sending()
                  ? CONTENT.chat_page.activate_page.sending
                  : cooldown() > 0
                    ? CONTENT.chat_page.activate_page.resend_in.replace("{seconds}", String(cooldown()))
                    : CONTENT.chat_page.activate_page.send_code}
              </button>
            </div>

            <div class="public-activate-consents">
              <PublicCheckbox checked={remember()} onChange={setRemember}>{CONTENT.chat_page.activate_page.keep_signed_in}</PublicCheckbox>
              <PublicCheckbox checked={agreed()} onChange={setAgreed}>
                {CONTENT.chat_page.activate_page.agree_prefix}
                <a href="/terms" target="_blank" rel="noopener noreferrer">{CONTENT.chat_page.activate_page.tos_link}</a>
                {CONTENT.chat_page.activate_page.and}
                <a href="/privacy" target="_blank" rel="noopener noreferrer">{CONTENT.chat_page.activate_page.privacy_link}</a>
                （{CONTENT.chat_page.activate_page.version_label} {TOS_VERSION}）
              </PublicCheckbox>
            </div>

            <Show when={error()}>{(message) => <Feedback message={message()} error />}</Show>
            <Show when={notice()}>{(message) => <Feedback message={message()} />}</Show>

            <button
              class="public-activate-submit"
              type="button"
              disabled={!loginReady() || (purchaseAvailable() && !selectedOfferAvailable())}
              onClick={submit}
            >
              {submitting()
                ? CONTENT.chat_page.activate_page.processing
                : !purchaseAvailable()
                  ? CONTENT.chat_page.activate_page.verify_restore
                  : CONTENT.chat_page.activate_page.verify_stripe}
            </button>
          </section>

          <p class="public-activate-footer">
            {CONTENT.chat_page.activate_page.cn_user} <a href="/chat">{CONTENT.chat_page.activate_page.sms_login}</a>
          </p>
        </Show>
      </div>
    </main>
  );
}

function formatAmount(amountMinor?: number) {
  return typeof amountMinor === "number" ? (amountMinor / 100).toFixed(2) : "—";
}

function OfferOption(props: {
  offer: StripeCheckoutOffer;
  selected: boolean;
  enabled: boolean;
  title: string;
  price: string;
  description: string;
  methods: string;
  onSelect: (offer: StripeCheckoutOffer) => void;
}) {
  return (
    <button
      type="button"
      class="public-activate-offer"
      classList={{ "is-selected": props.selected }}
      disabled={!props.enabled}
      aria-pressed={props.selected}
      onClick={() => props.onSelect(props.offer)}
    >
      <span class="public-activate-offer-head">
        <strong>{props.title}</strong>
        <b>{props.price}</b>
      </span>
      <span>{props.description}</span>
      <small>{props.methods}</small>
    </button>
  );
}

function FieldLabel(props: ParentProps) {
  return <span>{props.children}</span>;
}

function TextInput(props: {
  value: string;
  onInput: (value: string) => void;
  type?: string;
  placeholder?: string;
  autoComplete?: string;
  inputMode?: "text" | "email" | "numeric";
  ariaLabel: string;
  onEnter?: () => void;
}) {
  return (
    <input
      class="public-login-input"
      type={props.type ?? "text"}
      value={props.value}
      onInput={(event) => props.onInput(event.currentTarget.value)}
      onKeyDown={(event) => event.key === "Enter" && props.onEnter?.()}
      placeholder={props.placeholder}
      autocomplete={props.autoComplete}
      inputmode={props.inputMode}
      aria-label={props.ariaLabel}
    />
  );
}

function Feedback(props: { message: string; error?: boolean }) {
  return <div class="public-activate-feedback" classList={{ "is-error": props.error }} role={props.error ? "alert" : "status"}>{props.message}</div>;
}
