// public-me.tsx — Hone Public Site Account / Me page

import { createMemo, createSignal, onMount, Show, type ParentProps } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { PublicModal } from "@/components/public-modal"
import { PublicCheckbox } from "@/components/public-checkbox"
import { PublicPasswordField } from "@/components/public-password-field"
import { PasswordSetupGuard } from "@/components/password-setup-guard"
import { CONTENT } from "@/lib/public-content"
import {
  changePublicPassword,
  getPublicAuthMe,
  publicInviteLogin,
  publicLogout,
  publicPasswordLogin,
} from "@/lib/api"
import { normalizeInviteCode, normalizePhoneNumber } from "@/lib/public-chat"
import { checkPasswordStrength } from "@/lib/password"
import { TOS_VERSION } from "@/lib/tos"
import type { PublicAuthUserInfo } from "@/lib/types"
import "./public-site.css"

function formatDate(iso: string | undefined): string {
  if (!iso) return CONTENT.me.date_placeholder
  try {
    return new Date(iso).toLocaleDateString(CONTENT.me.date_locale, { year: "numeric", month: "long", day: "numeric" })
  } catch {
    return iso
  }
}

function StatCard(props: { label: string; value: string | number; sub?: string; accent?: boolean }) {
  return (
    <div
      style={{
        padding: "24px",
        "border-radius": "10px",
        border: `1px solid ${props.accent ? "rgba(245,158,11,0.25)" : "rgba(0,0,0,0.08)"}`,
        background: props.accent ? "rgba(245,158,11,0.04)" : "#fff",
      }}
    >
      <div
        style={{
          "font-size": "11px",
          "font-weight": "600",
          "letter-spacing": "0.15em",
          "text-transform": "uppercase",
          color: props.accent ? "#d97706" : "#94a3b8",
          "margin-bottom": "10px",
        }}
      >
        {props.label}
      </div>
      <div
        style={{
          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
          "font-size": "28px",
          "font-weight": "700",
          color: props.accent ? "#f59e0b" : "#0f172a",
          "line-height": "1",
        }}
      >
        {props.value}
      </div>
      <Show when={props.sub}>
        <div style={{ "font-size": "12px", color: "#94a3b8", "margin-top": "6px" }}>{props.sub}</div>
      </Show>
    </div>
  )
}

function InfoRow(props: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        "align-items": "center",
        "justify-content": "space-between",
        padding: "14px 0",
        "border-bottom": "1px solid rgba(0,0,0,0.06)",
      }}
    >
      <span style={{ "font-size": "13px", color: "#94a3b8", "font-weight": "500" }}>{props.label}</span>
      <span
        style={{
          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
          "font-size": "13px",
          color: "#0f172a",
          "font-weight": "500",
        }}
      >
        {props.value}
      </span>
    </div>
  )
}

type ActionBtnVariant = "default" | "primary" | "ghost" | "danger"

function ActionBtn(props: ParentProps<{
  onClick?: () => void
  href?: string
  variant?: ActionBtnVariant
}>) {
  const variant = () => props.variant ?? "default"

  const getStyle = () => {
    const base = {
      padding: "10px 20px",
      "border-radius": "8px",
      cursor: "pointer",
      "font-family": "inherit",
      "font-size": "14px",
      "font-weight": "600",
      "letter-spacing": "0.01em",
      transition: "all 0.2s",
      display: "inline-flex",
      "align-items": "center",
      gap: "6px",
      "text-decoration": "none",
    }
    if (variant() === "primary") {
      return { ...base, background: "#f59e0b", border: "1px solid #f59e0b", color: "#fff", "box-shadow": "0 2px 8px rgba(245,158,11,0.25)" }
    }
    if (variant() === "ghost") {
      return { ...base, background: "transparent", border: "1px solid rgba(0,0,0,0.08)", color: "#94a3b8" }
    }
    if (variant() === "danger") {
      return { ...base, background: "transparent", border: "1px solid rgba(239,68,68,0.20)", color: "#ef4444" }
    }
    return { ...base, background: "#fff", border: "1px solid rgba(0,0,0,0.10)", color: "#475569" }
  }

  return (
    <Show
      when={!props.href}
      fallback={
        <a href={props.href} style={getStyle()}>
          {props.children}
        </a>
      }
    >
      <button onClick={props.onClick} style={getStyle()}>
        {props.children}
      </button>
    </Show>
  )
}

// ── Logged out ────────────────────────────────────────────────────────────────

type LoginTab = "password" | "invite"

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
      我已阅读并同意{" "}
      <a
        href="/terms"
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: "#d97706", "text-decoration": "underline" }}
      >
        《用户协议》
      </a>{" "}
      和{" "}
      <a
        href="/privacy"
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: "#d97706", "text-decoration": "underline" }}
      >
        《隐私政策》
      </a>
      (v{TOS_VERSION})
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
      {props.loading ? "登录中…" : props.label}
    </button>
  )
}

function LoggedOutView(props: { onLogin: (user: PublicAuthUserInfo) => void }) {
  const [tab, setTab] = createSignal<LoginTab>("password")
  const [phoneNumber, setPhoneNumber] = createSignal("")
  const [password, setPassword] = createSignal("")
  const [inviteCode, setInviteCode] = createSignal("")
  const [remember, setRemember] = createSignal(true)
  const [agreed, setAgreed] = createSignal(false)
  const [submitting, setSubmitting] = createSignal(false)
  const [error, setError] = createSignal("")

  const phoneOk = createMemo(() => normalizePhoneNumber(phoneNumber()).length >= 5)
  const passwordTabReady = createMemo(
    () => phoneOk() && password().length > 0 && agreed() && !submitting(),
  )
  const inviteTabReady = createMemo(
    () =>
      phoneOk() &&
      normalizeInviteCode(inviteCode()).length > 0 &&
      agreed() &&
      !submitting(),
  )

  const submitPassword = async () => {
    if (!passwordTabReady()) return
    setSubmitting(true)
    setError("")
    try {
      const user = await publicPasswordLogin({
        phone_number: normalizePhoneNumber(phoneNumber()),
        password: password(),
        remember: remember(),
      })
      props.onLogin(user)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  const submitInvite = async () => {
    if (!inviteTabReady()) return
    setSubmitting(true)
    setError("")
    try {
      const user = await publicInviteLogin(
        normalizeInviteCode(inviteCode()),
        normalizePhoneNumber(phoneNumber()),
      )
      props.onLogin(user)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  const tabBtnStyle = (active: boolean) => ({
    flex: "1",
    padding: "10px 8px",
    "border-radius": "8px",
    border: "none",
    background: active ? "#fff" : "transparent",
    color: active ? "#0f172a" : "#64748b",
    "font-family": "inherit",
    "font-size": "13px",
    "font-weight": active ? ("700" as const) : ("500" as const),
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
      <div style={{ "max-width": "420px", width: "100%", margin: "0 auto", padding: "0 24px" }}>
        <div style={{ "margin-bottom": "24px", "text-align": "center" }}>
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
            登录 Hone
          </h1>
          <p style={{ "font-size": "13px", color: "#64748b", margin: "0", "line-height": "1.6" }}>
            老用户首次登录后,请尽快设置个人密码。
          </p>
        </div>

        <div
          style={{
            padding: "24px",
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
              "margin-bottom": "20px",
            }}
          >
            <button
              type="button"
              onClick={() => setTab("password")}
              style={tabBtnStyle(tab() === "password")}
              data-testid="tab-password"
            >
              密码登录
            </button>
            <button
              type="button"
              onClick={() => setTab("invite")}
              style={tabBtnStyle(tab() === "invite")}
              data-testid="tab-invite"
            >
              邀请码登录
            </button>
          </div>

          {/* Phone (shared) */}
          <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
            <FieldLabel>手机号</FieldLabel>
            <TextInput
              value={phoneNumber()}
              onInput={setPhoneNumber}
              type="tel"
              placeholder="例如 13800138000"
              autoComplete="tel"
              ariaLabel="手机号"
            />
          </div>

          {/* Tab body */}
          <Show
            when={tab() === "password"}
            fallback={
              <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
                <FieldLabel>邀请码</FieldLabel>
                <TextInput
                  value={inviteCode()}
                  onInput={setInviteCode}
                  placeholder="HONE-XXXXXX-XXXXXX"
                  ariaLabel="邀请码"
                  onEnter={submitInvite}
                />
              </div>
            }
          >
            <div style={{ display: "flex", "flex-direction": "column", "margin-bottom": "12px" }}>
              <FieldLabel>密码</FieldLabel>
              <PublicPasswordField
                value={password()}
                onInput={setPassword}
                placeholder="您的密码"
                autoComplete="current-password"
                ariaLabel="密码"
                onEnter={submitPassword}
              />
            </div>
            <div style={{ "margin-bottom": "12px" }}>
              <PublicCheckbox checked={remember()} onChange={setRemember}>
                <span style={{ "font-size": "13px" }}>保持登录(30 天)</span>
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
            disabled={tab() === "password" ? !passwordTabReady() : !inviteTabReady()}
            loading={submitting()}
            label={tab() === "password" ? "登录" : "使用邀请码登录"}
            onClick={tab() === "password" ? submitPassword : submitInvite}
          />
        </div>
      </div>
    </div>
  )
}

// ── Logged in ─────────────────────────────────────────────────────────────────
function LoggedInView(props: { user: PublicAuthUserInfo; onLogout: () => void }) {
  const navigate = useNavigate()
  const C = CONTENT.me
  const [changeOpen, setChangeOpen] = createSignal(false)

  const usedToday = () => props.user.daily_limit - props.user.remaining_today
  const pct = () => Math.min(100, Math.round((usedToday() / props.user.daily_limit) * 100))

  return (
    <div style={{ "padding-top": "56px", "min-height": "100vh", background: "#f8fafc" }}>
      <div style={{ "max-width": "800px", margin: "0 auto", padding: "56px 32px" }}>
        {/* Header */}
        <div
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            "margin-bottom": "48px",
          }}
        >
          <div>
            <div
              style={{
                "font-size": "11px",
                "font-weight": "700",
                "letter-spacing": "0.30em",
                "text-transform": "uppercase",
                color: "#f59e0b",
                "margin-bottom": "8px",
              }}
            >
              {C.logged_in_eyebrow}
            </div>
            <h1
              style={{
                "font-size": "28px",
                "font-weight": "700",
                color: "#0f172a",
                margin: "0",
                "letter-spacing": "-0.01em",
              }}
            >
              {C.logged_in_title}
            </h1>
          </div>
          <div
            style={{
              width: "48px",
              height: "48px",
              "border-radius": "50%",
              background: "#0f172a",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
            }}
          >
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                color: "#f59e0b",
                "font-size": "18px",
              }}
            >
              ✦
            </span>
          </div>
        </div>

        {/* Stats grid */}
        <div class="pub-me-stats">
          <StatCard
            label={C.stats.remaining_today_label}
            value={props.user.remaining_today}
            sub={C.stats.remaining_today_sub_template.replace("{daily}", String(props.user.daily_limit))}
            accent
          />
          <StatCard label={C.stats.total_label} value={props.user.success_count} sub={C.stats.total_sub} />
          <StatCard label={C.stats.daily_limit_label} value={props.user.daily_limit} sub={C.stats.daily_limit_sub} />
        </div>

        {/* Usage bar */}
        <div
          style={{
            padding: "20px 24px",
            "border-radius": "10px",
            border: "1px solid rgba(0,0,0,0.08)",
            background: "#fff",
            "margin-bottom": "24px",
          }}
        >
          <div
            style={{
              display: "flex",
              "justify-content": "space-between",
              "align-items": "center",
              "margin-bottom": "10px",
            }}
          >
            <span style={{ "font-size": "13px", "font-weight": "600", color: "#0f172a" }}>{C.usage_today_label}</span>
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                "font-size": "13px",
                color: "#64748b",
              }}
            >
              {usedToday()} / {props.user.daily_limit}
            </span>
          </div>
          <div style={{ height: "6px", "border-radius": "999px", background: "#f1f5f9", overflow: "hidden" }}>
            <div
              style={{
                height: "100%",
                "border-radius": "999px",
                background: pct() > 80 ? "#ef4444" : "#f59e0b",
                width: `${pct()}%`,
                transition: "width 0.6s ease",
              }}
            />
          </div>
        </div>

        {/* Account info */}
        <div
          style={{
            padding: "24px 28px",
            "border-radius": "12px",
            border: "1px solid rgba(0,0,0,0.08)",
            background: "#fff",
            "margin-bottom": "24px",
          }}
        >
          <h3
            style={{
              "font-size": "13px",
              "font-weight": "700",
              "letter-spacing": "0.10em",
              "text-transform": "uppercase",
              color: "#94a3b8",
              margin: "0 0 4px",
            }}
          >
            {C.account_info_title}
          </h3>
          <div>
            <InfoRow label={C.fields.user_id} value={props.user.user_id} />
            <InfoRow label={C.fields.created_at} value={formatDate(props.user.created_at)} />
            <InfoRow label={C.fields.last_login} value={formatDate(props.user.last_login_at)} />
          </div>
        </div>

        {/* Actions */}
        <div style={{ display: "flex", "flex-wrap": "wrap", gap: "10px", "margin-bottom": "40px" }}>
          <ActionBtn variant="primary" onClick={() => navigate("/chat")}>
            {C.actions.chat}
          </ActionBtn>
          <ActionBtn variant="default" onClick={() => navigate("/roadmap")}>
            {C.actions.roadmap}
          </ActionBtn>
          <ActionBtn variant="ghost" href="#">
            {C.actions.community}
          </ActionBtn>
          <Show when={props.user.has_password}>
            <ActionBtn variant="default" onClick={() => setChangeOpen(true)}>
              修改密码
            </ActionBtn>
          </Show>
          <ActionBtn variant="danger" onClick={props.onLogout}>
            {C.actions.logout}
          </ActionBtn>
        </div>

        <ChangePasswordModal open={changeOpen()} onClose={() => setChangeOpen(false)} />

        {/* Membership placeholder */}
        <div
          style={{
            padding: "24px 28px",
            "border-radius": "12px",
            border: "1px dashed rgba(245,158,11,0.20)",
            background: "rgba(245,158,11,0.03)",
          }}
        >
          <div style={{ display: "flex", "align-items": "center", gap: "12px" }}>
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                "font-size": "16px",
                color: "#f59e0b",
                opacity: "0.5",
              }}
            >
              ∞
            </span>
            <div>
              <div style={{ "font-size": "13px", "font-weight": "700", color: "#0f172a", "margin-bottom": "4px" }}>
                {C.membership.title}
              </div>
              <div style={{ "font-size": "12px", color: "#94a3b8" }}>
                {C.membership.desc}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function ChangePasswordModal(props: { open: boolean; onClose: () => void }) {
  const [current, setCurrent] = createSignal("")
  const [next, setNext] = createSignal("")
  const [confirmPwd, setConfirmPwd] = createSignal("")
  const [submitting, setSubmitting] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [done, setDone] = createSignal(false)

  const reset = () => {
    setCurrent("")
    setNext("")
    setConfirmPwd("")
    setError(null)
    setDone(false)
  }

  const close = () => {
    if (submitting()) return
    reset()
    props.onClose()
  }

  const strength = createMemo(() => checkPasswordStrength(next()))
  const matches = createMemo(() => confirmPwd().length > 0 && confirmPwd() === next())
  const canSubmit = createMemo(
    () => current().length > 0 && strength().ok && matches() && !submitting(),
  )

  const submit = async () => {
    if (!canSubmit()) return
    setSubmitting(true)
    setError(null)
    try {
      await changePublicPassword({
        current_password: current(),
        new_password: next(),
      })
      setDone(true)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <PublicModal open={props.open} title="修改密码" onClose={close}>
      <Show
        when={!done()}
        fallback={
          <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
            <p
              style={{
                margin: "0",
                "font-size": "14px",
                color: "#15803d",
                "line-height": "1.6",
              }}
            >
              ✓ 密码已更新。下次登录请使用新密码。
            </p>
            <div style={{ display: "flex", "justify-content": "flex-end" }}>
              <button
                type="button"
                onClick={close}
                style={{
                  padding: "9px 18px",
                  "border-radius": "8px",
                  border: "none",
                  background: "#f59e0b",
                  color: "#fff",
                  cursor: "pointer",
                  "font-family": "inherit",
                  "font-size": "13px",
                  "font-weight": "600",
                }}
              >
                好的
              </button>
            </div>
          </div>
        }
      >
        <div style={{ display: "flex", "flex-direction": "column", gap: "14px" }}>
          <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
            <FieldLabel>当前密码</FieldLabel>
            <PublicPasswordField
              value={current()}
              onInput={setCurrent}
              autoComplete="current-password"
              ariaLabel="当前密码"
              placeholder="当前密码"
            />
          </div>
          <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
            <FieldLabel>新密码</FieldLabel>
            <PublicPasswordField
              value={next()}
              onInput={setNext}
              autoComplete="new-password"
              ariaLabel="新密码"
              placeholder="至少 8 位,含字母与数字"
              showRules
            />
          </div>
          <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
            <FieldLabel>确认新密码</FieldLabel>
            <PublicPasswordField
              value={confirmPwd()}
              onInput={setConfirmPwd}
              autoComplete="new-password"
              ariaLabel="确认新密码"
              placeholder="再输入一次"
              onEnter={submit}
            />
            <Show when={confirmPwd().length > 0 && !matches()}>
              <span style={{ "font-size": "11.5px", color: "#dc2626" }}>两次输入的密码不一致</span>
            </Show>
          </div>
          <Show when={error()}>
            <ErrorBox message={error()!} />
          </Show>
          <div style={{ display: "flex", "justify-content": "flex-end", gap: "10px" }}>
            <button
              type="button"
              onClick={close}
              disabled={submitting()}
              style={{
                padding: "9px 16px",
                "border-radius": "8px",
                border: "1px solid rgba(15,23,42,0.14)",
                background: "#fff",
                color: "#475569",
                cursor: submitting() ? "not-allowed" : "pointer",
                "font-family": "inherit",
                "font-size": "13px",
              }}
            >
              取消
            </button>
            <button
              type="button"
              onClick={submit}
              disabled={!canSubmit()}
              style={{
                padding: "9px 18px",
                "border-radius": "8px",
                border: "none",
                background: canSubmit() ? "#f59e0b" : "rgba(245,158,11,0.5)",
                color: "#fff",
                cursor: canSubmit() ? "pointer" : "not-allowed",
                "font-family": "inherit",
                "font-size": "13px",
                "font-weight": "600",
              }}
            >
              {submitting() ? "保存中…" : "保存"}
            </button>
          </div>
        </div>
      </Show>
    </PublicModal>
  )
}

// ── PublicMePage ──────────────────────────────────────────────────────────────
export default function PublicMePage() {
  const navigate = useNavigate()
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(null)
  const [loading, setLoading] = createSignal(true)

  onMount(async () => {
    try {
      const me = await getPublicAuthMe()
      setUser(me)
    } catch {
      setUser(null)
    } finally {
      setLoading(false)
    }
  })

  const handleLogout = async () => {
    try {
      await publicLogout()
    } catch {
      // ignore
    }
    setUser(null)
    navigate("/chat")
  }

  return (
    <div class="pub-page" style={{ "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)", "-webkit-font-smoothing": "antialiased" }}>
      <PublicNav />
      <Show
        when={!loading()}
        fallback={
          <div
            style={{
              "padding-top": "56px",
              "min-height": "100vh",
              background: "#f8fafc",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
            }}
          >
            <div style={{ "font-size": "13px", color: "#94a3b8" }}>{CONTENT.me.loading}</div>
          </div>
        }
      >
        <Show when={user()} fallback={<LoggedOutView onLogin={setUser} />}>
          {(u) => (
            <PasswordSetupGuard user={u()} onPasswordSet={setUser}>
              <LoggedInView user={u()} onLogout={handleLogout} />
            </PasswordSetupGuard>
          )}
        </Show>
      </Show>
      <PublicFooter />
    </div>
  )
}
