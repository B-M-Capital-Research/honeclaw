import { createMemo, createSignal, type ParentProps } from "solid-js"
import { createSimpleContext } from "./helper"

type ThemeContextValue = {
  mode: () => "dark"
  setMode: (mode: "dark") => void
}

const [ThemeContextProvider, useTheme] = createSimpleContext<ThemeContextValue>("Theme")

export { useTheme }

export function ThemeProvider(props: ParentProps) {
  const [mode, setMode] = createSignal<"dark">("dark")

  const value = createMemo<ThemeContextValue>(() => ({
    mode,
    setMode,
  }))

  return <ThemeContextProvider value={value()}>{props.children}</ThemeContextProvider>
}
