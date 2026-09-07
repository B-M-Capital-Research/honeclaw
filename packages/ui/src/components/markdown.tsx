import { createResource } from "solid-js"
import { useMarked } from "../context/marked"

export function Markdown(props: {
  text: string
  class?: string
  /**
   * Optional post-pass over the sanitised HTML before it reaches the DOM.
   * Surfaces with a fixed width (the exported share card) use it to reshape
   * content that only makes sense at the conversation's width.
   */
  transform?: (html: string) => string
}) {
  const marked = useMarked()
  const [html] = createResource(
    () => [props.text, props.transform] as const,
    async ([text, transform]) => {
      const parsed = await marked.parse(text)
      return transform ? transform(parsed) : parsed
    },
  )

  return (
    <div
      class={["hf-markdown", props.class].filter(Boolean).join(" ")}
      innerHTML={html.latest ?? ""}
    />
  )
}
