// public-login-form.tsx — /me 和 /chat 共用的手机号验证码登录卡

import {
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  type ParentProps,
} from "solid-js";
import { PublicCheckbox } from "./public-checkbox";
import {
  getPublicCaptchaConfig,
  publicSendSmsCode,
  publicSmsLogin,
} from "@/lib/api";
import {
  AliyunCaptchaController,
  type PublicCaptchaConfig,
} from "@/lib/aliyun-captcha";
import { CONTENT } from "@/lib/public-content";
import { normalizePhoneNumber } from "@/lib/public-chat";
import { TOS_VERSION } from "@/lib/tos";
import type { PublicAuthUserInfo } from "@/lib/types";

type Props = {
  onLogin: (user: PublicAuthUserInfo) => void | Promise<void>;
  title?: string;
  subtitle?: string;
};

let captchaIdSeed = 0;

export function PublicLoginForm(props: Props) {
  const [phoneNumber, setPhoneNumber] = createSignal("");
  const [verifyCode, setVerifyCode] = createSignal("");
  const [remember, setRemember] = createSignal(true);
  const [agreed, setAgreed] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [sending, setSending] = createSignal(false);
  const [cooldown, setCooldown] = createSignal(0);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const captchaElementId = `public-login-captcha-${++captchaIdSeed}`;
  const captchaButtonId = `public-login-captcha-button-${captchaIdSeed}`;
  let captchaConfigPromise: Promise<PublicCaptchaConfig> | undefined;
  let captchaController: AliyunCaptchaController | undefined;
  const clearFeedback = () => {
    setError("");
    setNotice("");
  };

  const phoneOk = createMemo(
    () => normalizePhoneNumber(phoneNumber()).length >= 5,
  );
  const codeOk = createMemo(() =>
    /^[0-9A-Za-z]{4,8}$/.test(verifyCode().trim()),
  );
  const sendReady = createMemo(
    () => phoneOk() && !sending() && cooldown() <= 0,
  );
  const loginReady = createMemo(
    () => phoneOk() && codeOk() && agreed() && !submitting(),
  );

  let cooldownTimer: ReturnType<typeof setInterval> | undefined;
  onCleanup(() => {
    if (cooldownTimer) clearInterval(cooldownTimer);
  });
  onMount(() => {
    void preloadCaptcha();
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
    setSending(true);
    clearFeedback();
    try {
      const captchaVerifyParam = await verifyCaptchaIfEnabled();
      await publicSendSmsCode(
        normalizePhoneNumber(phoneNumber()),
        captchaVerifyParam,
      );
      setNotice(CONTENT.auth.login.code_sent);
      startCooldown();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSending(false);
    }
  };

  const verifyCaptchaIfEnabled = async () => {
    const captcha = await ensureCaptchaController();
    if (!captcha) return undefined;
    return captcha.verify();
  };

  const preloadCaptcha = async () => {
    try {
      const captcha = await ensureCaptchaController();
      await captcha?.prepare();
    } catch {
      // Surface initialization errors only when the user actively requests SMS.
    }
  };

  const ensureCaptchaController = async () => {
    captchaConfigPromise ??= getPublicCaptchaConfig();
    const config = await captchaConfigPromise;
    if (!config.enabled) return undefined;
    captchaController ??= new AliyunCaptchaController(
      config,
      captchaElementId,
      captchaButtonId,
    );
    return captchaController;
  };

  const submitLogin = async () => {
    if (!loginReady()) return;
    setSubmitting(true);
    clearFeedback();
    try {
      const user = await publicSmsLogin({
        phone_number: normalizePhoneNumber(phoneNumber()),
        verify_code: verifyCode().trim(),
        remember: remember(),
        tos_version: TOS_VERSION,
      });
      await props.onLogin(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      class="public-login-screen"
      style={{
        "padding-top": "56px",
        "min-height": "100dvh",
        background: "#f8fafc",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
        "box-sizing": "border-box",
        "overflow-y": "auto",
        "-webkit-overflow-scrolling": "touch",
        width: "100%",
      }}
    >
      <div
        class="public-login-card-wrap"
        style={{
          "max-width": "440px",
          width: "100%",
          margin: "0 auto",
          padding: "0 24px",
        }}
      >
        {/* Header */}
        <div style={{ "margin-bottom": "22px", "text-align": "center" }}>
          <img
            src="/logo.svg"
            style={{ height: "36px", "margin-bottom": "16px" }}
            alt="Hone"
          />
          <h1
            style={{
              "font-size": "22px",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0 0 8px",
              "letter-spacing": "0",
            }}
          >
            {props.title ?? CONTENT.auth.login.title}
          </h1>
          <p
            style={{
              "font-size": "13px",
              color: "#64748b",
              margin: "0",
              "line-height": "1.6",
            }}
          >
            {props.subtitle ?? CONTENT.auth.login.subtitle}
          </p>
        </div>

        {/* Card */}
        <div
          class="public-login-card"
          style={{
            padding: "22px",
            "border-radius": "14px",
            border: "1px solid rgba(15,23,42,0.06)",
            background: "#fff",
            "box-shadow": "0 4px 24px rgba(15,23,42,0.05)",
          }}
        >
          <p
            style={{
              margin: "0 0 16px",
              "font-size": "12px",
              color: "#94a3b8",
              "line-height": "1.55",
              "text-align": "center",
            }}
          >
            {CONTENT.auth.login.hint_sms}
          </p>

          <div
            style={{
              display: "flex",
              "flex-direction": "column",
              "margin-bottom": "12px",
            }}
          >
            <FieldLabel>{CONTENT.auth.login.phone_label}</FieldLabel>
            <TextInput
              value={phoneNumber()}
              onInput={setPhoneNumber}
              type="tel"
              placeholder={CONTENT.auth.login.phone_placeholder}
              autoComplete="tel"
              ariaLabel={CONTENT.auth.login.phone_aria}
            />
          </div>

          <div
            class="public-login-code-row"
            style={{ display: "flex", gap: "10px", "margin-bottom": "12px" }}
          >
            <div
              style={{ flex: "1", display: "flex", "flex-direction": "column" }}
            >
              <FieldLabel>{CONTENT.auth.login.code_label}</FieldLabel>
              <TextInput
                value={verifyCode()}
                onInput={setVerifyCode}
                placeholder={CONTENT.auth.login.code_placeholder}
                ariaLabel={CONTENT.auth.login.code_aria}
                inputMode="numeric"
                onEnter={submitLogin}
              />
            </div>
            <button
              class="public-login-code-button"
              type="button"
              disabled={!sendReady()}
              onClick={sendCode}
              style={{
                "align-self": "end",
                width: "116px",
                height: "40px",
                "border-radius": "8px",
                border: "1px solid rgba(245,158,11,0.38)",
                background: sendReady() ? "#fff7ed" : "#f8fafc",
                color: sendReady() ? "#b45309" : "#94a3b8",
                cursor: sendReady() ? "pointer" : "not-allowed",
                "font-family": "inherit",
                "font-size": "13px",
                "font-weight": "700",
              }}
            >
              {sending()
                ? CONTENT.auth.login.sending_code
                : cooldown() > 0
                  ? CONTENT.auth.login.resend_in.replace(
                      "{seconds}",
                      String(cooldown()),
                    )
                  : CONTENT.auth.login.send_code}
            </button>
          </div>
          <div
            id={captchaElementId}
            style={{ position: "relative", "z-index": "20" }}
          />
          <button
            id={captchaButtonId}
            type="button"
            tabindex="-1"
            aria-hidden="true"
            style={{
              position: "absolute",
              width: "1px",
              height: "1px",
              padding: "0",
              border: "0",
              opacity: "0",
              overflow: "hidden",
              "pointer-events": "none",
            }}
          />

          <div style={{ "margin-bottom": "12px" }}>
            <PublicCheckbox checked={remember()} onChange={setRemember}>
              <span style={{ "font-size": "13px" }}>
                {CONTENT.auth.login.remember_30d}
              </span>
            </PublicCheckbox>
          </div>

          <div style={{ "margin-bottom": "16px" }}>
            <PublicCheckbox checked={agreed()} onChange={setAgreed}>
              <TosLink />
            </PublicCheckbox>
          </div>

          <Show when={error()}>
            <div style={{ "margin-bottom": "12px" }}>
              <ErrorBox message={error()} />
            </div>
          </Show>
          <Show when={notice()}>
            <div style={{ "margin-bottom": "12px" }}>
              <NoticeBox message={notice()} />
            </div>
          </Show>

          <SubmitButton
            disabled={!loginReady()}
            loading={submitting()}
            label={CONTENT.auth.login.submit_sms}
            onClick={submitLogin}
          />
        </div>
      </div>
    </div>
  );
}

// ── local helpers ─────────────────────────────────────────────────────────────

function TextInput(props: {
  value: string;
  onInput: (v: string) => void;
  placeholder?: string;
  type?: string;
  ariaLabel?: string;
  onEnter?: () => void;
  autoComplete?: string;
  inputMode?:
    | "none"
    | "text"
    | "tel"
    | "url"
    | "email"
    | "numeric"
    | "decimal"
    | "search";
}) {
  return (
    <input
      type={props.type ?? "text"}
      value={props.value}
      placeholder={props.placeholder}
      autocomplete={props.autoComplete}
      aria-label={props.ariaLabel}
      inputmode={props.inputMode}
      onInput={(e) => props.onInput(e.currentTarget.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && props.onEnter) {
          e.preventDefault();
          props.onEnter();
        }
      }}
      style={{
        width: "100%",
        padding: "10px 12px",
        "border-radius": "8px",
        border: "1px solid rgba(15,23,42,0.14)",
        "font-size": "14px",
        "font-family": "inherit",
        color: "#0f172a",
        background: "#fff",
        outline: "none",
        "box-sizing": "border-box",
        transition: "border-color 0.15s ease, box-shadow 0.15s ease",
      }}
      onFocus={(e) => {
        e.currentTarget.style.borderColor = "#f59e0b";
        e.currentTarget.style.boxShadow = "0 0 0 3px rgba(245,158,11,0.15)";
      }}
      onBlur={(e) => {
        e.currentTarget.style.borderColor = "rgba(15,23,42,0.14)";
        e.currentTarget.style.boxShadow = "none";
      }}
    />
  );
}

function FieldLabel(props: ParentProps) {
  return (
    <span
      style={{
        "font-size": "12px",
        "font-weight": "600",
        color: "#475569",
        "letter-spacing": "0.02em",
        "margin-bottom": "6px",
      }}
    >
      {props.children}
    </span>
  );
}

function ErrorBox(props: { message: string }) {
  return (
    <div
      style={{
        padding: "10px 12px",
        "border-radius": "8px",
        background: "rgba(220,38,38,0.06)",
        border: "1px solid rgba(220,38,38,0.2)",
        color: "#b91c1c",
        "font-size": "12.5px",
      }}
    >
      {props.message}
    </div>
  );
}

function NoticeBox(props: { message: string }) {
  return (
    <div
      style={{
        padding: "10px 12px",
        "border-radius": "8px",
        background: "rgba(22,163,74,0.06)",
        border: "1px solid rgba(22,163,74,0.2)",
        color: "#15803d",
        "font-size": "12.5px",
      }}
    >
      {props.message}
    </div>
  );
}

function TosLink() {
  return (
    <>
      {CONTENT.auth.tos.prefix}
      <a
        href="/terms"
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: "#d97706", "text-decoration": "underline" }}
      >
        {CONTENT.auth.tos.terms}
      </a>
      {CONTENT.auth.tos.and}
      <a
        href="/privacy"
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: "#d97706", "text-decoration": "underline" }}
      >
        {CONTENT.auth.tos.privacy}
      </a>
      {CONTENT.auth.tos.version_template.replace("{version}", TOS_VERSION)}
    </>
  );
}

function SubmitButton(props: {
  disabled: boolean;
  loading: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      disabled={props.disabled}
      onClick={props.onClick}
      style={{
        display: "block",
        width: "100%",
        padding: "12px 18px",
        "border-radius": "8px",
        background: props.disabled ? "rgba(245,158,11,0.5)" : "#f59e0b",
        border: "none",
        cursor: props.disabled ? "not-allowed" : "pointer",
        "font-family": "inherit",
        "font-size": "15px",
        "font-weight": "700",
        color: "#fff",
        "box-shadow": props.disabled
          ? "none"
          : "0 4px 14px rgba(245,158,11,0.28)",
        transition: "background 0.15s ease",
      }}
    >
      {props.loading ? CONTENT.auth.login.loading : props.label}
    </button>
  );
}
