// public-password-field.tsx — 密码输入框 + 显/隐切换 + 实时规则提示

import { Show, createMemo, createSignal, type JSX } from "solid-js"
import { checkPasswordStrength } from "@/lib/password"

type Props = {
  value: string
  onInput: (value: string) => void
  placeholder?: string
  showRules?: boolean
  autoComplete?: "current-password" | "new-password"
  ariaLabel?: string
  disabled?: boolean
  onEnter?: () => void
}

export function PublicPasswordField(props: Props): JSX.Element {
  const [reveal, setReveal] = createSignal(false)
  const check = createMemo(() => checkPasswordStrength(props.value))

  return (
    <div style={{ display: "flex", "flex-direction": "column", gap: "8px" }}>
      <div style={{ position: "relative" }}>
        <input
          type={reveal() ? "text" : "password"}
          value={props.value}
          aria-label={props.ariaLabel}
          autocomplete={props.autoComplete ?? "current-password"}
          placeholder={props.placeholder ?? "密码"}
          disabled={props.disabled}
          onInput={(e) => props.onInput(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && props.onEnter) {
              e.preventDefault()
              props.onEnter()
            }
          }}
          style={{
            width: "100%",
            padding: "10px 44px 10px 12px",
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
        <button
          type="button"
          onClick={() => setReveal((r) => !r)}
          tabIndex={-1}
          aria-label={reveal() ? "隐藏密码" : "显示密码"}
          style={{
            position: "absolute",
            right: "8px",
            top: "50%",
            transform: "translateY(-50%)",
            width: "30px",
            height: "30px",
            border: "none",
            background: "transparent",
            cursor: "pointer",
            color: reveal() ? "#d97706" : "#94a3b8",
            display: "inline-flex",
            "align-items": "center",
            "justify-content": "center",
            "border-radius": "6px",
          }}
        >
          {reveal() ? (
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                d="M2 8s2.4-4.2 6-4.2S14 8 14 8s-2.4 4.2-6 4.2S2 8 2 8z"
                stroke="currentColor"
                stroke-width="1.4"
                fill="none"
              />
              <circle cx="8" cy="8" r="1.8" stroke="currentColor" stroke-width="1.4" fill="none" />
            </svg>
          ) : (
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                d="M2 8s2.4-4.2 6-4.2S14 8 14 8s-2.4 4.2-6 4.2S2 8 2 8z"
                stroke="currentColor"
                stroke-width="1.4"
                fill="none"
              />
              <circle cx="8" cy="8" r="1.8" stroke="currentColor" stroke-width="1.4" fill="none" />
              <path d="M3 3 L13 13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
          )}
        </button>
      </div>
      <Show when={props.showRules && props.value.length > 0}>
        <div style={{ display: "flex", "flex-direction": "column", gap: "4px", "font-size": "11.5px" }}>
          <Rule ok={check().rules.lengthOk} text="8–128 位" />
          <Rule ok={check().rules.hasLetter} text="至少一个字母" />
          <Rule ok={check().rules.hasDigit} text="至少一个数字" />
        </div>
      </Show>
    </div>
  )
}

function Rule(props: { ok: boolean; text: string }) {
  return (
    <span
      style={{
        display: "inline-flex",
        "align-items": "center",
        gap: "6px",
        color: props.ok ? "#15803d" : "#94a3b8",
      }}
    >
      <span
        style={{
          width: "12px",
          height: "12px",
          "border-radius": "50%",
          display: "inline-flex",
          "align-items": "center",
          "justify-content": "center",
          background: props.ok ? "rgba(34,197,94,0.18)" : "rgba(15,23,42,0.06)",
          "font-size": "9px",
          "font-weight": "700",
        }}
      >
        {props.ok ? "✓" : ""}
      </span>
      {props.text}
    </span>
  )
}
