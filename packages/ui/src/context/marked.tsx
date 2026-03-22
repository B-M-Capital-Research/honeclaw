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

parser.use({
  renderer: {
    async code(token: any) {
      const language = token.lang?.trim() || "text"
      const html = await codeToHtml(token.text, {
        lang: language,
        theme: "github-dark",
      }).catch(async () => {
        return codeToHtml(token.text, {
          lang: "text",
          theme: "github-dark",
        })
      })

      return `<div class="hf-markdown-code">${html}</div>`
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
