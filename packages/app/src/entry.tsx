import { render } from "solid-js/web"
import { App } from "./app"
import { installAssetLoadRecovery } from "./lib/asset-recovery"

installAssetLoadRecovery()

const root = document.getElementById("root")

if (!root) {
  throw new Error("root element missing")
}

root.replaceChildren()
render(() => <App />, root)
