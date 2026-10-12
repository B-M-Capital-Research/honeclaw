import { render } from "solid-js/web"
import { App } from "./app"
import { installAssetLoadRecovery } from "./lib/asset-recovery"

installAssetLoadRecovery()

const root = document.getElementById("root")

if (!root) {
  throw new Error("root element missing")
}

// index.html carries a static <title> for crawlers and the first paint. The
// browser always uses the first <title> in <head>, so once the app owns the
// document through @solidjs/meta that static tag would shadow every page's
// own <Title>; drop it before mounting.
for (const title of document.head.querySelectorAll("title")) title.remove()

root.replaceChildren()
render(() => <App />, root)
