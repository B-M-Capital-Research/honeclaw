// public-checkbox.tsx — 琥珀色自绘 checkbox + 标签

import { type JSX, type ParentProps } from "solid-js"

type Props = ParentProps<{
  checked: boolean
  onChange: (checked: boolean) => void
  disabled?: boolean
  ariaLabel?: string
}>

export function PublicCheckbox(props: Props): JSX.Element {
  const toggle = () => {
    if (props.disabled) return
    props.onChange(!props.checked)
  }

  return (
    <label
      onClick={(e) => {
        e.preventDefault()
        toggle()
      }}
      style={{
        display: "inline-flex",
        "align-items": "flex-start",
        gap: "10px",
        cursor: props.disabled ? "not-allowed" : "pointer",
        "user-select": "none",
        "line-height": "1.5",
        "font-size": "13px",
        color: "#0f172a",
        opacity: props.disabled ? "0.6" : "1",
      }}
    >
      <span
        role="checkbox"
        aria-checked={props.checked}
        aria-label={props.ariaLabel}
        tabIndex={props.disabled ? -1 : 0}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault()
            toggle()
          }
        }}
        style={{
          width: "18px",
          height: "18px",
          "border-radius": "5px",
          border: props.checked ? "1.5px solid #f59e0b" : "1.5px solid rgba(15,23,42,0.18)",
          background: props.checked ? "#f59e0b" : "#fff",
          display: "inline-flex",
          "align-items": "center",
          "justify-content": "center",
          "flex-shrink": "0",
          "margin-top": "1px",
          transition: "background 0.15s ease, border-color 0.15s ease",
        }}
      >
        {props.checked ? (
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M2 6.4 L4.8 9 L10 3.4"
              stroke="#fff"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        ) : null}
      </span>
      <span>{props.children}</span>
    </label>
  )
}
