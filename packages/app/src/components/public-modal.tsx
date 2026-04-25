// public-modal.tsx — Hone Public Site modal (淡入遮罩 + 居中卡)
// blockClose=true 时不渲染 × 按钮且点击遮罩不关闭,用于强制设密码

import { Show, type JSX, type ParentProps, createEffect, onCleanup } from "solid-js"
import { Portal } from "solid-js/web"

type Props = ParentProps<{
  open: boolean
  title?: string
  onClose?: () => void
  blockClose?: boolean
  width?: string
  footer?: JSX.Element
}>

export function PublicModal(props: Props) {
  createEffect(() => {
    if (!props.open) return
    const prev = document.body.style.overflow
    document.body.style.overflow = "hidden"
    onCleanup(() => {
      document.body.style.overflow = prev
    })
  })

  const handleBackdrop = () => {
    if (props.blockClose) return
    props.onClose?.()
  }

  return (
    <Show when={props.open}>
      <Portal>
        <div
          onClick={handleBackdrop}
          style={{
            position: "fixed",
            inset: "0",
            "z-index": "1200",
            background: "rgba(15,23,42,0.45)",
            "backdrop-filter": "blur(2px)",
            "-webkit-backdrop-filter": "blur(2px)",
            display: "flex",
            "align-items": "center",
            "justify-content": "center",
            padding: "20px",
            animation: "pub-fadeup 0.18s ease forwards",
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "#fff",
              "border-radius": "14px",
              "box-shadow": "0 30px 80px rgba(15,23,42,0.22), 0 4px 12px rgba(15,23,42,0.06)",
              width: props.width ?? "440px",
              "max-width": "100%",
              "max-height": "calc(100vh - 40px)",
              display: "flex",
              "flex-direction": "column",
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            }}
          >
            <Show when={props.title || !props.blockClose}>
              <div
                style={{
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "space-between",
                  padding: "18px 22px",
                  "border-bottom": "1px solid rgba(15,23,42,0.06)",
                }}
              >
                <h3
                  style={{
                    margin: "0",
                    "font-size": "16px",
                    "font-weight": "700",
                    color: "#0f172a",
                    "letter-spacing": "-0.01em",
                  }}
                >
                  {props.title ?? ""}
                </h3>
                <Show when={!props.blockClose}>
                  <button
                    type="button"
                    onClick={() => props.onClose?.()}
                    aria-label="关闭"
                    style={{
                      width: "30px",
                      height: "30px",
                      "border-radius": "8px",
                      border: "none",
                      background: "transparent",
                      cursor: "pointer",
                      color: "#94a3b8",
                      "font-size": "20px",
                      "line-height": "1",
                      display: "flex",
                      "align-items": "center",
                      "justify-content": "center",
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(15,23,42,0.06)")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                  >
                    ×
                  </button>
                </Show>
              </div>
            </Show>
            <div
              style={{
                padding: "22px",
                "overflow-y": "auto",
                "flex-grow": "1",
              }}
            >
              {props.children}
            </div>
            <Show when={props.footer}>
              <div
                style={{
                  padding: "16px 22px",
                  "border-top": "1px solid rgba(15,23,42,0.06)",
                  display: "flex",
                  "justify-content": "flex-end",
                  gap: "10px",
                }}
              >
                {props.footer}
              </div>
            </Show>
          </div>
        </div>
      </Portal>
    </Show>
  )
}
