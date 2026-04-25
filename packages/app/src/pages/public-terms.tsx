// public-terms.tsx — 用户协议（bilingual: 绑定到 CONTENT.legal.terms）

import { For, type ParentProps } from "solid-js"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { CONTENT, type LegalBlock, type LegalInline } from "@/lib/public-content"
import { TOS_VERSION, TOS_EFFECTIVE_DATE } from "@/lib/tos"
import "./public-site.css"

function VersionBanner() {
  const label = CONTENT.legal.version_banner_template
    .replace("{version}", TOS_VERSION)
    .replace("{date}", TOS_EFFECTIVE_DATE)
  return (
    <div
      style={{
        display: "inline-flex",
        "align-items": "center",
        gap: "10px",
        padding: "6px 12px",
        "border-radius": "999px",
        background: "rgba(245,158,11,0.08)",
        border: "1px solid rgba(245,158,11,0.25)",
        color: "#d97706",
        "font-size": "12px",
        "font-weight": "600",
        "letter-spacing": "0.02em",
      }}
    >
      {label}
    </div>
  )
}

function Inline(props: { part: LegalInline }) {
  const p = props.part
  if (typeof p === "string") return <>{p}</>
  if ("strong" in p) return <strong>{p.strong}</strong>
  return <code>{p.code}</code>
}

function Block(props: { block: LegalBlock }) {
  const b = props.block
  if (b.kind === "p") {
    return (
      <p>
        <For each={b.parts}>{(part) => <Inline part={part} />}</For>
      </p>
    )
  }
  return (
    <ul>
      <For each={b.items}>
        {(item) => (
          <li>
            <For each={item}>{(part) => <Inline part={part} />}</For>
          </li>
        )}
      </For>
    </ul>
  )
}

function Section(props: ParentProps<{ title: string }>) {
  return (
    <section style={{ "margin-bottom": "32px" }}>
      <h2
        style={{
          "font-size": "18px",
          "font-weight": "700",
          color: "#0f172a",
          margin: "0 0 12px",
          "letter-spacing": "-0.01em",
        }}
      >
        {props.title}
      </h2>
      <div
        style={{
          "font-size": "14.5px",
          "line-height": "1.75",
          color: "#334155",
        }}
        class="pub-prose"
      >
        {props.children}
      </div>
    </section>
  )
}

export default function PublicTermsPage() {
  return (
    <div
      class="pub-page"
      style={{
        "min-height": "100vh",
        background: "#fff",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
      }}
    >
      <PublicNav />

      <div
        style={{
          "max-width": "780px",
          margin: "0 auto",
          padding: "120px 24px 64px",
        }}
      >
        <VersionBanner />
        <h1
          style={{
            "font-size": "36px",
            "font-weight": "800",
            color: "#0f172a",
            margin: "20px 0 12px",
            "letter-spacing": "-0.02em",
          }}
        >
          {CONTENT.legal.terms.page_title}
        </h1>
        <p
          style={{
            "font-size": "14px",
            color: "#94a3b8",
            "margin-bottom": "40px",
            "line-height": "1.6",
          }}
        >
          {CONTENT.legal.terms.intro}
        </p>

        <For each={CONTENT.legal.terms.sections}>
          {(s) => (
            <Section title={s.title}>
              <For each={s.body}>{(block) => <Block block={block} />}</For>
            </Section>
          )}
        </For>
      </div>

      <PublicFooter />
    </div>
  )
}
