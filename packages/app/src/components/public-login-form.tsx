// public-login-form.tsx — /me 和 /chat 共用的登录卡（tab: 密码登录 / 邀请码激活）
//
// 默认展示"密码登录"；"邀请码激活"用于新用户首次进入。顶部一行引导文案 +
// 每个 tab 下一行小字，让用户不用猜该选哪边。

import { Show, createMemo, createSignal, type JSX, type ParentProps } from "solid-js"
import { PublicCheckbox } from "./public-checkbox"
import { PublicPasswordField } from "./public-password-field"
import { publicInviteLogin, publicPasswordLogin } from "@/lib/api"
import { CONTENT } from "@/lib/public-content"
import { normalizeInviteCode, normalizePhoneNumber } from "@/lib/public-chat"
import { TOS_VERSION } from "@/lib/tos"
import type { PublicAuthUserInfo } from "@/lib/types"

type LoginTab = "password" | "invite"

type Props = {
  onLogin: (user: PublicAuthUserInfo) => void | Promise<void>
  title?: string
  subtitle?: string
}

export function PublicLoginForm(props: Props) {
  const [tab, setTab] = createSignal<LoginTab>("password")
  const [phoneNumber, setPhoneNumber] = createSignal("")
  const [password, setPassword] = createSignal("")
  const [inviteCode, setInviteCode] = createSignal("")
  const [remember, setRemember] = createSignal(true)
  const [agreed, setAgreed] = createSignal(false)
  const [submitting, setSubmitting] = createSignal(false)
  const [error, setError] = createSignal("")

  const phoneOk = createMemo(() => normalizePhoneNumber(phoneNumber()).length >= 5)
  const passwordReady = createMemo(
    () => phoneOk() && password().length > 0 && agreed() && !submitting(),
  )
  const inviteReady = createMemo(
    () =>
      phoneOk() &&
      normalizeInviteCode(inviteCode()).length > 0 &&
      agreed() &&
      !submitting(),
  )

  const submitPassword = async () => {
    if (!passwordReady()) return
    setSubmitting(true)
    setError("")
    try {
      const user = await publicPasswordLogin({
        phone_number: normalizePhoneNumber(phoneNumber()),
        password: password(),
        remember: remember(),
      })
      await props.onLogin(user)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  const submitInvite = async () => {
    if (!inviteReady()) return
    setSubmitting(true)
    setError("")
    try {
      const user = await publicInviteLogin(
        normalizeInviteCode(inviteCode()),
        normalizePhoneNumber(phoneNumber()),
      )
      await props.onLogin(user)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  const tabBtnStyle = (active: boolean): JSX.CSSProperties => ({
    flex: "1",
    padding: "10px 8px",
    "border-radius": "8px",
    border: "none",
    background: active ? "#fff" : "transparent",
    color: active ? "#0f172a" : "#64748b",
    "font-family": "inherit",
    "font-size": "13px",
    "font-weight": active ? "700" : "500",
    cursor: "pointer",
    "box-shadow": active ? "0 1px 3px rgba(15,23,42,0.08)" : "none",
    transition: "all 0.15s ease",
  })

  return (
    <div
      style={{
        "padding-top": "56px",
        "min-height": "100vh",
        background: "#f8fafc",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
      }}
    >
      <div style={{ "max-width": "440px", width: "100%", margin: "0 auto", padding: "0 24px" }}>
        {/* Header */}
        <div style={{ "margin-bottom": "22px", "text-align": "center" }}>
          <img src="/logo.svg" style={{ height: "36px", "margin-bottom": "16px" }} alt="Hone" />
          <h1
            style={{
              "font-size": "22px",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0 0 8px",
              "letter-spacing": "-0.01em",
            }}
          >
            {props.title ?? CONTENT.auth.login.title}
          </h1>
          <p style={{ "font-size": "13px", color: "#64748b", margin: "0", "line-height": "1.6" }}>
            {props.subtitle ?? CONTENT.auth.login.subtitle}
          </p>
        </div>

        {/* Card */}
        <div
          style={{
            padding: "22px",
            "border-radius": "14px",
            border: "1px solid rgba(15,23,42,0.06)",
            background: "#fff",
            "box-shadow": "0 4px 24px rgba(15,23,42,0.05)",
          }}
        >
          {/* Tabs */}
          <div
            style={{
              display: "flex",
              gap: "4px",
              padding: "4px",
              background: "#f1f5f9",
              "border-radius": "10px",
              "margin-bottom": "10px",
            }}
          >
            <button
              type="button"
              onClick={() => {
                setTab("password")
                setError("")
              }}
              style={tabBtnStyle(tab() === "password")}
              data-testid="tab-password"
            >
              {CONTENT.auth.login.tab_password}
            </button>
            <button
              type="button"
              onClick={() => {
                setTab("invite")
                setError("")
              }}
              style={tabBtnStyle(tab() === "invite")}
              data-testid="tab-invite"
            >
              {CONTENT.auth.login.tab_invite}
            </button>
          </div>

          {/* Per-tab hint */}
          <p
            style={{
              margin: "0 0 16px",
              "font-size": "12px",
              color: "#94a3b8",
              "line-height": "1.55",
              "text-align": "center",
            }}
          >
            <Show
              when={tab() === "password"}
              fallback={CONTENT.auth.login.hint_invite}
            >
              {CONTENT.auth.login.hint_password}
            </Show>
          </p>

          {/* Phone (shared) */}
          <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
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

          {/* Tab body */}
          <Show
            when={tab() === "password"}
            fallback={
              <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
                <FieldLabel>{CONTENT.auth.login.invite_label}</FieldLabel>
                <TextInput
                  value={inviteCode()}
                  onInput={setInviteCode}
                  placeholder={CONTENT.auth.login.invite_placeholder}
                  ariaLabel={CONTENT.auth.login.invite_aria}
                  onEnter={submitInvite}
                />
              </div>
            }
          >
            <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
              <FieldLabel>{CONTENT.auth.login.password_label}</FieldLabel>
              <PublicPasswordField
                value={password()}
                onInput={setPassword}
                placeholder={CONTENT.auth.login.password_placeholder}
                autoComplete="current-password"
                ariaLabel={CONTENT.auth.login.password_aria}
                onEnter={submitPassword}
              />
            </div>
            <div style={{ "margin-bottom": "12px" }}>
              <PublicCheckbox checked={remember()} onChange={setRemember}>
                <span style={{ "font-size": "13px" }}>{CONTENT.auth.login.remember_30d}</span>
              </PublicCheckbox>
            </div>
          </Show>

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

          <SubmitButton
            disabled={tab() === "password" ? !passwordReady() : !inviteReady()}
            loading={submitting()}
            label={
              tab() === "password"
                ? CONTENT.auth.login.submit_password
                : CONTENT.auth.login.submit_invite
            }
            onClick={tab() === "password" ? submitPassword : submitInvite}
          />
        </div>
      </div>
    </div>
  )
}

// ── local helpers ─────────────────────────────────────────────────────────────

function TextInput(props: {
  value: string
  onInput: (v: string) => void
  placeholder?: string
  type?: string
  ariaLabel?: string
  onEnter?: () => void
  autoComplete?: string
}) {
  return (
    <input
      type={props.type ?? "text"}
      value={props.value}
      placeholder={props.placeholder}
      autocomplete={props.autoComplete}
      aria-label={props.ariaLabel}
      onInput={(e) => props.onInput(e.currentTarget.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && props.onEnter) {
          e.preventDefault()
          props.onEnter()
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
        e.currentTarget.style.borderColor = "#f59e0b"
        e.currentTarget.style.boxShadow = "0 0 0 3px rgba(245,158,11,0.15)"
      }}
      onBlur={(e) => {
        e.currentTarget.style.borderColor = "rgba(15,23,42,0.14)"
        e.currentTarget.style.boxShadow = "none"
      }}
    />
  )
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
  )
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
  )
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
  )
}

function SubmitButton(props: {
  disabled: boolean
  loading: boolean
  label: string
  onClick: () => void
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
        "box-shadow": props.disabled ? "none" : "0 4px 14px rgba(245,158,11,0.28)",
        transition: "background 0.15s ease",
      }}
    >
      {props.loading ? CONTENT.auth.login.loading : props.label}
    </button>
  )
}
