import { defineConfig } from "vite"
import solid from "vite-plugin-solid"
import tailwindcss from "@tailwindcss/vite"

const backend = process.env.HONE_WEB_BACKEND_URL ?? "http://127.0.0.1:8077"
const port = Number(process.env.HONE_APP_PORT ?? "3000")
const outDir = process.env.HONE_APP_OUT_DIR ?? "dist"
const appSurface = process.env.HONE_APP_SURFACE ?? process.env.VITE_HONE_APP_SURFACE ?? "admin"
const desktopRelativeBase =
  process.env.HONE_APP_RELATIVE_BASE === "1" || Boolean(process.env.TAURI_ENV_PLATFORM)

export default defineConfig({
  base: desktopRelativeBase ? "./" : "/",
  define: {
    "import.meta.env.VITE_HONE_APP_SURFACE": JSON.stringify(appSurface),
  },
  plugins: [solid(), tailwindcss()],
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "solid-js",
  },
  server: {
    host: "127.0.0.1",
    port,
    allowedHosts: ["hone.viphk.nnhk.cc", "hone-claw.com", "localhost", "127.0.0.1"],
    proxy: {
      "/api": backend,
    },
  },
  build: {
    target: "esnext",
    outDir,
    rollupOptions: {
      // mermaid 的 sankey 图表依赖 d3-sankey，但该包未被打包进来；
      // 该包未被打包进来，需要保持 external。
      // Tauri API 需要在桌面产物中正常打包，否则运行时会留下裸模块导入。
      external: ["d3-sankey"],
    },
  },
  resolve: {
    conditions: ["solid", "browser", "module", "import", "default"],
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
})
