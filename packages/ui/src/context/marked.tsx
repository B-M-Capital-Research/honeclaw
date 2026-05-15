import { createMemo, type ParentProps } from "solid-js"
import { createSimpleContext } from "./helper"
import { parseMarkdown } from "../lib/markdown"

type MarkedContextValue = {
  parse: (markdown: string) => Promise<string>
}

const [MarkedContextProvider, useMarked] = createSimpleContext<MarkedContextValue>("Marked")

export { useMarked }

export function MarkedProvider(props: ParentProps) {
  const value = createMemo<MarkedContextValue>(() => ({
    parse: parseMarkdown,
  }))

  return <MarkedContextProvider value={value()}>{props.children}</MarkedContextProvider>
}
