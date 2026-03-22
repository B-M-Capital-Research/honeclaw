import { render } from "solid-js/web"
import { App } from "./app"

const root = document.getElementById("root")

if (!root) {
  throw new Error("root element missing")
}

render(() => <App />, root)
