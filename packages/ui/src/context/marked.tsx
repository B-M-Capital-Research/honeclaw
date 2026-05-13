import DOMPurify from "dompurify"
import { Marked } from "marked"
import { codeToHtml } from "shiki"
import { createMemo, type ParentProps } from "solid-js"
import { createSimpleContext } from "./helper"

type MarkedContextValue = {
  parse: (markdown: string) => Promise<string>
}

const [MarkedContextProvider, useMarked] = createSimpleContext<MarkedContextValue>("Marked")

export { useMarked }

const parser = new Marked({
  async: true,
  breaks: true,
  gfm: true,
}) as any

// Pre-render code blocks via shiki in walkTokens (which marked awaits before
// rendering) and stash the highlighted HTML on the token. The renderer then
// reads it back synchronously — making the renderer itself async, as marked
// only awaits walkTokens, will stringify the returned Promise to the literal
// "[object Promise]" right inside the rendered HTML.
parser.use({
  async walkTokens(token: any) {
    if (token.type !== "code") return
    const language = token.lang?.trim() || "text"
    const html = await codeToHtml(token.text, {
      lang: language,
      theme: "github-light-default",
    }).catch(async () => {
      return codeToHtml(token.text, {
        lang: "text",
        theme: "github-light-default",
      })
    })
    token._highlightedHtml = `<div class="hf-markdown-code">${html}</div>`
  },
  renderer: {
    code(token: any) {
      return token._highlightedHtml ?? `<pre><code>${token.text}</code></pre>`
    },
  },
})

export function MarkedProvider(props: ParentProps) {
  const value = createMemo<MarkedContextValue>(() => ({
    parse: async (markdown: string) => {
      const html = await parser.parse(markdown ?? "")
      return DOMPurify.sanitize(html, {
        USE_PROFILES: { html: true },
      })
    },
  }))

  return <MarkedContextProvider value={value()}>{props.children}</MarkedContextProvider>
}
