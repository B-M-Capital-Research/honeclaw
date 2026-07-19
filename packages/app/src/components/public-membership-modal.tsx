// public-membership-modal.tsx — 「购买完整服务」弹窗：权益表格 + 知识星球付费
// 加入二维码（立减券）+ 企业微信客服二维码。素材放在
// packages/app/public/membership_zsxq.jpg 与 membership_wechat.jpg。

import { For, Show, createEffect, onCleanup } from "solid-js"
import { Portal } from "solid-js/web"
import { CONTENT } from "@/lib/public-content"

export function PublicMembershipModal(props: { open: boolean; onClose: () => void }) {
  const C = () => CONTENT.membership

  createEffect(() => {
    if (!props.open) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose()
    }
    document.addEventListener("keydown", onKeyDown)
    onCleanup(() => document.removeEventListener("keydown", onKeyDown))
  })

  return (
    <Show when={props.open}>
      <Portal>
        <div class="pub-membership-backdrop" onClick={props.onClose}>
          <section
            class="pub-membership-modal"
            role="dialog"
            aria-modal="true"
            aria-label={C().title}
            onClick={(event) => event.stopPropagation()}
          >
            <header class="pub-membership-head">
              <div>
                <span class="pub-membership-eyebrow">{C().eyebrow}</span>
                <h2>{C().title}</h2>
                <p>{C().sub}</p>
              </div>
              <button type="button" class="pub-membership-close" aria-label={C().close_aria} onClick={props.onClose}>
                ×
              </button>
            </header>

            <div class="pub-membership-body">
              <table class="pub-membership-table">
                <thead>
                  <tr>
                    <th scope="col">{C().table_head_item}</th>
                    <th scope="col">{C().table_head_desc}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={C().items}>
                    {(item, i) => (
                      <tr>
                        <th scope="row">
                          <i>{String(i() + 1).padStart(2, "0")}</i>
                          {item.name}
                        </th>
                        <td>{item.desc}</td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>

              <div class="pub-membership-qrs">
                <div class="pub-membership-qr is-join">
                  <div class="pub-membership-qr-copy">
                    <strong>{C().join_title}</strong>
                    <span class="pub-membership-coupon">{C().join_coupon}</span>
                  </div>
                  <img src="/membership_zsxq.jpg" alt={C().join_hint} loading="lazy" />
                  <small>{C().join_hint}</small>
                </div>
                <div class="pub-membership-qr">
                  <div class="pub-membership-qr-copy">
                    <strong>{C().service_title}</strong>
                  </div>
                  <img src="/membership_wechat.jpg" alt={C().service_hint} loading="lazy" />
                  <small>{C().service_hint}</small>
                </div>
              </div>

              <p class="pub-membership-foot">{C().foot}</p>
            </div>
          </section>
        </div>
      </Portal>
    </Show>
  )
}
