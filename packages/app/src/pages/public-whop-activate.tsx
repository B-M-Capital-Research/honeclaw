import { useNavigate } from "@solidjs/router";
import {
  Show,
  createMemo,
  createSignal,
  onCleanup,
  type ParentProps,
} from "solid-js";
import { HoneBrand } from "@/components/hone-brand";
import { PublicCheckbox } from "@/components/public-checkbox";
import { PublicPrefsButton } from "@/components/public-prefs-button";
import { publicEmailLogin, publicSendEmailCode } from "@/lib/api";
import { TOS_VERSION } from "@/lib/tos";

import "./public-foundation.css";
import "./public-site.css";

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export default function PublicWhopActivatePage() {
  const navigate = useNavigate();
  const [emailAddress, setEmailAddress] = createSignal("");
  const [verifyCode, setVerifyCode] = createSignal("");
  const [remember, setRemember] = createSignal(true);
  const [agreed, setAgreed] = createSignal(false);
  const [sending, setSending] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [cooldown, setCooldown] = createSignal(0);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const normalizedEmail = createMemo(() =>
    emailAddress().trim().toLowerCase(),
  );
  const emailOk = createMemo(() => EMAIL_PATTERN.test(normalizedEmail()));
  const codeOk = createMemo(() => /^[0-9]{6,12}$/.test(verifyCode().trim()));
  const sendReady = createMemo(
    () => emailOk() && !sending() && cooldown() <= 0,
  );
  const loginReady = createMemo(
    () => emailOk() && codeOk() && agreed() && !submitting(),
  );

  let cooldownTimer: ReturnType<typeof setInterval> | undefined;
  onCleanup(() => {
    if (cooldownTimer) clearInterval(cooldownTimer);
  });

  const clearFeedback = () => {
    setError("");
    setNotice("");
  };

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
    clearFeedback();
    setSending(true);
    try {
      const response = await publicSendEmailCode(normalizedEmail());
      setNotice(response.message);
      startCooldown();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSending(false);
    }
  };

  const submitLogin = async () => {
    if (!loginReady()) return;
    clearFeedback();
    setSubmitting(true);
    try {
      await publicEmailLogin({
        email_address: normalizedEmail(),
        verify_code: verifyCode().trim(),
        remember: remember(),
        tos_version: TOS_VERSION,
      });
      navigate("/me?checkout=success");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main
      class="public-login-screen public-whop-activate"
      style={{
        "min-height": "100dvh",
        padding: "48px 20px",
        background: "var(--hone-paper-100)",
        display: "grid",
        "place-items": "center",
        "box-sizing": "border-box",
        "font-family": "var(--hone-font-body)",
        "overflow-y": "auto",
        "-webkit-overflow-scrolling": "touch",
      }}
    >
      <div class="public-login-preferences">
        <PublicPrefsButton />
      </div>
      <div class="public-whop-activate-inner" style={{ width: "min(100%, 500px)" }}>
        <header style={{ "text-align": "center", "margin-bottom": "22px" }}>
          <HoneBrand class="public-login-brand" />
          <div
            style={{
              display: "inline-flex",
              padding: "5px 10px",
              "border-radius": "999px",
              background:
                "color-mix(in srgb, var(--hone-coral-500) 10%, transparent)",
              color: "var(--hone-link)",
              "font-size": "12px",
              "font-weight": "700",
              "margin-bottom": "12px",
            }}
          >
            WHOP 会员开通
          </div>
          <h1
            style={{
              margin: "0 0 8px",
              color: "var(--hone-ink-950)",
              "font-size": "26px",
              "line-height": "1.25",
            }}
          >
            开通你的 HONE 账号
          </h1>
          <p
            style={{
              margin: "0",
              color: "var(--hone-ink-600)",
              "font-size": "13px",
              "line-height": "1.65",
            }}
          >
            使用 Whop 付款时填写的邮箱验证身份，无需注册 Whop 账号，也无需提供手机号。
          </p>
        </header>

        <section
          class="public-whop-activate-card"
          style={{
            padding: "24px",
            "border-radius": "var(--hone-radius-md)",
            border: "1px solid var(--hone-line)",
            background: "var(--hone-surface-raised)",
            color: "var(--hone-ink-950)",
            "box-shadow": "var(--hone-login-shadow)",
          }}
        >
          <ol
            style={{
              margin: "0 0 22px",
              padding: "0",
              display: "grid",
              "grid-template-columns": "repeat(3, 1fr)",
              gap: "8px",
              "list-style": "none",
            }}
          >
            {["完成付款", "验证邮箱", "进入 HONE"].map((label, index) => (
              <li
                style={{
                  padding: "8px 4px",
                  "text-align": "center",
                  "border-radius": "8px",
                  background:
                    index === 1
                      ? "color-mix(in srgb, var(--hone-action-bg) 12%, var(--hone-surface-raised))"
                      : "var(--hone-paper-100)",
                  color:
                    index === 1
                      ? "var(--hone-link)"
                      : "var(--hone-ink-500)",
                  "font-size": "11px",
                  "font-weight": "700",
                }}
              >
                {index + 1}. {label}
              </li>
            ))}
          </ol>

          <div style={{ display: "grid", gap: "7px", "margin-bottom": "14px" }}>
            <FieldLabel>购买邮箱</FieldLabel>
            <TextInput
              type="email"
              value={emailAddress()}
              onInput={setEmailAddress}
              placeholder="name@example.com"
              autoComplete="email"
              ariaLabel="Whop 购买邮箱"
            />
          </div>

          <div
            class="public-whop-code-row"
            style={{
              display: "grid",
              "grid-template-columns": "minmax(0, 1fr) 124px",
              gap: "10px",
              "margin-bottom": "14px",
            }}
          >
            <div style={{ display: "grid", gap: "7px" }}>
              <FieldLabel>邮箱验证码</FieldLabel>
              <TextInput
                value={verifyCode()}
                onInput={(value) =>
                  setVerifyCode(value.replace(/\D/g, "").slice(0, 12))
                }
                placeholder="8 位验证码"
                inputMode="numeric"
                autoComplete="one-time-code"
                ariaLabel="邮箱验证码"
                onEnter={submitLogin}
              />
            </div>
            <button
              type="button"
              disabled={!sendReady()}
              onClick={sendCode}
              style={{
                "align-self": "end",
                height: "41px",
                border: "1px solid var(--hone-line-strong)",
                "border-radius": "var(--hone-radius-sm)",
                background: sendReady()
                  ? "var(--hone-control-surface)"
                  : "var(--hone-paper-200)",
                color: sendReady()
                  ? "var(--hone-link)"
                  : "var(--hone-ink-600)",
                cursor: sendReady() ? "pointer" : "not-allowed",
                "font-family": "inherit",
                "font-size": "12px",
                "font-weight": "700",
              }}
            >
              {sending()
                ? "发送中…"
                : cooldown() > 0
                  ? `${cooldown()} 秒后重发`
                  : "发送验证码"}
            </button>
          </div>

          <p
            style={{
              margin: "0 0 16px",
              color: "var(--hone-ink-500)",
              "font-size": "12px",
              "line-height": "1.6",
            }}
          >
            Whop 的付款通知可能需要几十秒到达。若未收到邮件，请确认地址与付款邮箱一致后重试。
          </p>

          <div style={{ display: "grid", gap: "10px", "margin-bottom": "16px" }}>
            <PublicCheckbox checked={remember()} onChange={setRemember}>
              保持登录 30 天
            </PublicCheckbox>
            <PublicCheckbox checked={agreed()} onChange={setAgreed}>
              我已阅读并同意
              <a href="/terms" target="_blank" rel="noopener noreferrer">
                《用户协议》
              </a>
              与
              <a href="/privacy" target="_blank" rel="noopener noreferrer">
                《隐私政策》
              </a>
              （版本 {TOS_VERSION}）
            </PublicCheckbox>
          </div>

          <Show when={error()}>
            <Feedback message={error()} error />
          </Show>
          <Show when={notice()}>
            <Feedback message={notice()} />
          </Show>

          <button
            type="button"
            disabled={!loginReady()}
            onClick={submitLogin}
            style={{
              width: "100%",
              padding: "12px 18px",
              border: "0",
              "border-radius": "var(--hone-radius-sm)",
              background: loginReady()
                ? "var(--hone-action-bg)"
                : "var(--hone-paper-200)",
              color: loginReady()
                ? "var(--hone-action-fg)"
                : "var(--hone-ink-600)",
              cursor: loginReady() ? "pointer" : "not-allowed",
              "font-family": "inherit",
              "font-size": "15px",
              "font-weight": "700",
            }}
          >
            {submitting() ? "正在验证…" : "验证并进入 HONE"}
          </button>
        </section>

        <p
          style={{
            margin: "16px 0 0",
            "text-align": "center",
            color: "var(--hone-ink-500)",
            "font-size": "12px",
          }}
        >
          国内手机号用户？{" "}
          <a class="public-login-member-link" href="/chat">
            使用短信验证码登录
          </a>
        </p>
      </div>
    </main>
  );
}

function FieldLabel(props: ParentProps) {
  return (
    <span
      style={{
        color: "var(--hone-ink-800)",
        "font-size": "12px",
        "font-weight": "700",
      }}
    >
      {props.children}
    </span>
  );
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
      onKeyDown={(event) => {
        if (event.key === "Enter") props.onEnter?.();
      }}
      placeholder={props.placeholder}
      autocomplete={props.autoComplete}
      inputmode={props.inputMode}
      aria-label={props.ariaLabel}
      style={{
        width: "100%",
        padding: "10px 12px",
        border: "1px solid var(--hone-line-strong)",
        "border-radius": "var(--hone-radius-sm)",
        background: "var(--hone-control-surface)",
        color: "var(--hone-ink-950)",
        "box-sizing": "border-box",
        "font-family": "inherit",
        "font-size": "14px",
        outline: "none",
      }}
    />
  );
}

function Feedback(props: { message: string; error?: boolean }) {
  return (
    <div
      role={props.error ? "alert" : "status"}
      style={{
        padding: "10px 12px",
        "margin-bottom": "12px",
        "border-radius": "var(--hone-radius-sm)",
        border: props.error
          ? "1px solid color-mix(in srgb, #b0443b 20%, transparent)"
          : "1px solid rgba(22,163,74,0.2)",
        background: props.error
          ? "color-mix(in srgb, #b0443b 8%, transparent)"
          : "rgba(22,163,74,0.06)",
        color: props.error
          ? "var(--hone-error-600)"
          : "var(--hone-success-600)",
        "font-size": "12.5px",
        "line-height": "1.5",
      }}
    >
      {props.message}
    </div>
  );
}
