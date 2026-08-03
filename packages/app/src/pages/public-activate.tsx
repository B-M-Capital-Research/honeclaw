import { Show, createMemo, createSignal, onCleanup, onMount, type ParentProps } from "solid-js";
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
} from "@/lib/api";
import { TOS_VERSION } from "@/lib/tos";
import type { PublicBillingConfig } from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export default function PublicActivatePage() {
  const navigate = useNavigate();
  const provider = new URLSearchParams(window.location.search).get("provider") === "whop"
    ? "whop"
    : "stripe";
  const [config, setConfig] = createSignal<PublicBillingConfig>();
  const [emailAddress, setEmailAddress] = createSignal("");
  const [verifyCode, setVerifyCode] = createSignal("");
  const [remember, setRemember] = createSignal(true);
  const [agreed, setAgreed] = createSignal(false);
  const [sending, setSending] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [cooldown, setCooldown] = createSignal(0);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const normalizedEmail = createMemo(() => emailAddress().trim().toLowerCase());
  const emailOk = createMemo(() => EMAIL_PATTERN.test(normalizedEmail()));
  const codeOk = createMemo(() => /^[0-9]{6,12}$/.test(verifyCode().trim()));
  const sendReady = createMemo(
    () => config() !== undefined && emailOk() && !sending() && cooldown() <= 0,
  );
  const loginReady = createMemo(
    () => emailOk() && codeOk() && agreed() && !submitting(),
  );
  const restoreOnly = createMemo(
    () => config()?.purchases_allowed_on_this_client === false,
  );
  const activationSteps = createMemo(() => {
    if (restoreOnly()) return ["验证邮箱", "登录账户", "恢复权益"];
    return provider === "stripe"
      ? ["验证邮箱", "Stripe 付款", "确认权益"]
      : ["Whop 付款", "验证邮箱", "恢复权益"];
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
        provider === "stripe" && !restoreOnly() ? "stripe_checkout" : undefined,
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
      if (provider === "whop" || restoreOnly() || user.billing.access_granted) {
        navigate("/me?billing=syncing");
        return;
      }
      const { checkout_url } = await createStripeCheckout();
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
    if (restoreOnly()) return "恢复你的 HONE 会员权益";
    return provider === "stripe" ? "验证邮箱并安全结账" : "连接已有 Whop 会员";
  };

  return (
    <main class="public-login-screen public-activate">
      <div class="public-login-preferences"><PublicPrefsButton /></div>
      <div class="public-activate-inner">
        <header class="public-activate-header">
          <HoneBrand class="public-login-brand" />
          <span class="public-activate-provider">
            {restoreOnly() ? "会员恢复" : provider === "stripe" ? "STRIPE 安全结账" : "WHOP 会员连接"}
          </span>
          <h1>{title()}</h1>
          <p>
            {restoreOnly()
              ? "在这里登录并恢复已在网站购买的权益；App 内不展示价格，也不会跳转外部购买。"
              : provider === "stripe"
                ? "邮箱验证后由 HONE 创建 Stripe Checkout。付款完成需等待服务端确认，成功跳转本身不会开通权益。"
                : "请输入 Whop 付款时使用的邮箱。HONE 会根据已签名的付款事件恢复对应权益。"}
          </p>
        </header>

        <section class="public-activate-card">
          <div class="public-activate-steps" aria-label="开通步骤">
            {activationSteps().map((label, index) => <span>{index + 1}. {label}</span>)}
          </div>

          <label class="public-activate-field">
            <FieldLabel>{provider === "whop" ? "Whop 购买邮箱" : "账户邮箱"}</FieldLabel>
            <TextInput
              type="email"
              value={emailAddress()}
              onInput={setEmailAddress}
              placeholder="name@example.com"
              autoComplete="email"
              ariaLabel="账户邮箱"
            />
          </label>

          <div class="public-activate-code-row">
            <label class="public-activate-field">
              <FieldLabel>邮箱验证码</FieldLabel>
              <TextInput
                value={verifyCode()}
                onInput={(value) => setVerifyCode(value.replace(/\D/g, "").slice(0, 12))}
                placeholder="8 位验证码"
                inputMode="numeric"
                autoComplete="one-time-code"
                ariaLabel="邮箱验证码"
                onEnter={submit}
              />
            </label>
            <button type="button" disabled={!sendReady()} onClick={sendCode}>
              {sending()
                ? "发送中…"
                : cooldown() > 0
                  ? `${cooldown()} 秒后重发`
                  : "发送验证码"}
            </button>
          </div>

          <div class="public-activate-consents">
            <PublicCheckbox checked={remember()} onChange={setRemember}>保持登录 30 天</PublicCheckbox>
            <PublicCheckbox checked={agreed()} onChange={setAgreed}>
              我已阅读并同意<a href="/terms" target="_blank" rel="noopener noreferrer">《用户协议》</a>
              与<a href="/privacy" target="_blank" rel="noopener noreferrer">《隐私政策》</a>
              （版本 {TOS_VERSION}）
            </PublicCheckbox>
          </div>

          <Show when={error()}>{(message) => <Feedback message={message()} error />}</Show>
          <Show when={notice()}>{(message) => <Feedback message={message()} />}</Show>

          <button class="public-activate-submit" type="button" disabled={!loginReady()} onClick={submit}>
            {submitting()
              ? "正在处理…"
              : restoreOnly() || provider === "whop"
                ? "验证并恢复权益"
                : "验证并前往 Stripe"}
          </button>
        </section>

        <p class="public-activate-footer">
          国内手机号用户？ <a href="/chat">使用短信验证码登录</a>
          {provider === "stripe" && config()?.whop_new_purchases_enabled ? (
            <> · 已在 Whop 购买？ <a href="/activate?provider=whop">连接 Whop</a></>
          ) : null}
        </p>
      </div>
    </main>
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
