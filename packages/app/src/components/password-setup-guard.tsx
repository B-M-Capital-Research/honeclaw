// password-setup-guard.tsx — 强制设密码 Modal
// 当 user.has_password === false 时拦截整张页面;成功后回调父层刷新 user

import { Show, createMemo, createSignal, type ParentProps } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { PublicModal } from "./public-modal"
import { PublicCheckbox } from "./public-checkbox"
import { PublicPasswordField } from "./public-password-field"
import { setPublicPassword, publicLogout } from "@/lib/api"
import { checkPasswordStrength } from "@/lib/password"
import { TOS_VERSION } from "@/lib/tos"
import type { PublicAuthUserInfo } from "@/lib/types"

type Props = ParentProps<{
  user: PublicAuthUserInfo
  onPasswordSet: (user: PublicAuthUserInfo) => void
}>

export function PasswordSetupGuard(props: Props) {
  const navigate = useNavigate()
  const [pwd, setPwd] = createSignal("")
  const [confirm, setConfirm] = createSignal("")
  const [agreed, setAgreed] = createSignal(false)
  const [submitting, setSubmitting] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  const strength = createMemo(() => checkPasswordStrength(pwd()))
  const matches = createMemo(() => confirm().length > 0 && confirm() === pwd())
  const canSubmit = createMemo(
    () => strength().ok && matches() && agreed() && !submitting(),
  )

  const submit = async () => {
    if (!canSubmit()) return
    setSubmitting(true)
    setError(null)
    try {
      const updated = await setPublicPassword({
        new_password: pwd(),
        tos_version: TOS_VERSION,
      })
      props.onPasswordSet(updated)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  const logoutAndLeave = async () => {
    try {
      await publicLogout()
    } catch {
      // ignore
    }
    navigate("/")
  }

  // Render guard overlay only when user.has_password is false
  return (
    <>
      {props.children}
      <Show when={props.user.has_password === false}>
        <PublicModal open={true} title="首次登录:请设置密码" blockClose width="460px">
          <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
            <p
              style={{
                margin: "0",
                "font-size": "13px",
                "line-height": "1.6",
                color: "#475569",
              }}
            >
              为保护账号安全,请设置一个个人密码。设置后,你将通过手机号 + 密码登录,无需再使用邀请码。
            </p>

            <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
              <Label>新密码</Label>
              <PublicPasswordField
                value={pwd()}
                onInput={setPwd}
                placeholder="至少 8 位,含字母与数字"
                showRules
                autoComplete="new-password"
                ariaLabel="新密码"
              />
            </div>

            <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
              <Label>确认密码</Label>
              <PublicPasswordField
                value={confirm()}
                onInput={setConfirm}
                placeholder="再输入一次"
                autoComplete="new-password"
                ariaLabel="确认密码"
                onEnter={submit}
              />
              <Show when={confirm().length > 0 && !matches()}>
                <span style={{ "font-size": "11.5px", color: "#dc2626" }}>两次输入的密码不一致</span>
              </Show>
            </div>

            <PublicCheckbox checked={agreed()} onChange={setAgreed}>
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
            </PublicCheckbox>

            <Show when={error()}>
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
                {error()}
              </div>
            </Show>

            <div style={{ display: "flex", "justify-content": "space-between", gap: "10px", "margin-top": "4px" }}>
              <button
                type="button"
                onClick={logoutAndLeave}
                disabled={submitting()}
                style={{
                  padding: "10px 16px",
                  "border-radius": "8px",
                  border: "1px solid rgba(15,23,42,0.14)",
                  background: "#fff",
                  color: "#475569",
                  cursor: submitting() ? "not-allowed" : "pointer",
                  "font-size": "13px",
                  "font-family": "inherit",
                }}
              >
                暂不设置 · 退出登录
              </button>
              <button
                type="button"
                onClick={submit}
                disabled={!canSubmit()}
                style={{
                  padding: "10px 20px",
                  "border-radius": "8px",
                  border: "none",
                  background: canSubmit() ? "#f59e0b" : "rgba(245,158,11,0.5)",
                  color: "#fff",
                  cursor: canSubmit() ? "pointer" : "not-allowed",
                  "font-size": "13px",
                  "font-weight": "600",
                  "font-family": "inherit",
                }}
              >
                {submitting() ? "保存中…" : "保存并继续"}
              </button>
            </div>
          </div>
        </PublicModal>
      </Show>
    </>
  )
}

function Label(props: ParentProps) {
  return (
    <span
      style={{
        "font-size": "12px",
        "font-weight": "600",
        color: "#475569",
        "letter-spacing": "0.02em",
      }}
    >
      {props.children}
    </span>
  )
}
