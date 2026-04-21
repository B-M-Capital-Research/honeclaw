import { createResource } from "solid-js"
import { useMarked } from "../context/marked"

export function Markdown(props: { text: string; class?: string }) {
  const marked = useMarked()
  const [html] = createResource(
    () => props.text,
    (value) => marked.parse(value),
  )

  return (
    <div
      class={["hf-markdown", props.class].filter(Boolean).join(" ")}
      innerHTML={html.latest ?? ""}
    />
  )
}
