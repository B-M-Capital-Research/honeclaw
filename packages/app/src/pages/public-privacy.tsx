// public-privacy.tsx — 隐私政策（bilingual: 绑定到 CONTENT.legal.privacy）

import { For, type ParentProps } from "solid-js"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { CONTENT, type LegalBlock, type LegalInline } from "@/lib/public-content"
import { TOS_VERSION, TOS_EFFECTIVE_DATE } from "@/lib/tos"
import { LegalToc, BackToTop, sectionAnchor } from "@/components/public-legal-toc"
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
        background: "color-mix(in srgb, var(--hone-coral-500) 8%, transparent)",
        border: "1px solid color-mix(in srgb, var(--hone-coral-500) 25%, transparent)",
        color: "var(--hone-coral-600)",
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
  const legalPart = props.part
  if (typeof legalPart === "string") return <>{legalPart}</>
  if ("strong" in legalPart) return <strong>{legalPart.strong}</strong>
  return <code>{legalPart.code}</code>
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

function Section(props: ParentProps<{ title: string; index: number }>) {
  return (
    <section id={sectionAnchor(props.index)} style={{ "margin-bottom": "32px", "scroll-margin-top": "80px" }}>
      <h2
        style={{
          "font-size": "18px",
          "font-weight": "700",
          color: "var(--hone-ink-950)",
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
          color: "var(--hone-ink-800)",
        }}
        class="pub-prose"
      >
        {props.children}
      </div>
    </section>
  )
}

export default function PublicPrivacyPage() {
  return (
    <div
      class="pub-page"
      style={{
        "min-height": "100vh",
        background: "#fff",
        "font-family": "var(--hone-font-body)",
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
            color: "var(--hone-ink-950)",
            margin: "20px 0 12px",
            "letter-spacing": "-0.02em",
          }}
        >
          {CONTENT.legal.privacy.page_title}
        </h1>
        <p
          style={{
            "font-size": "14px",
            color: "var(--hone-ink-400)",
            "margin-bottom": "40px",
            "line-height": "1.6",
          }}
        >
          {CONTENT.legal.privacy.intro}
        </p>

        <LegalToc sections={CONTENT.legal.privacy.sections} />

        <For each={CONTENT.legal.privacy.sections}>
          {(s, i) => (
            <Section title={s.title} index={i()}>
              <For each={s.body}>{(block) => <Block block={block} />}</For>
            </Section>
          )}
        </For>
      </div>

      <BackToTop />
      <PublicFooter />
    </div>
  )
}
