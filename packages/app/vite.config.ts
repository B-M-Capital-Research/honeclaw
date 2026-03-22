import { defineConfig } from "vite"
import solid from "vite-plugin-solid"
import tailwindcss from "@tailwindcss/vite"

const backend = process.env.HONE_WEB_BACKEND_URL ?? "http://127.0.0.1:8077"

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "solid-js",
  },
  server: {
    host: "127.0.0.1",
    port: 3000,
    proxy: {
      "/api": backend,
    },
  },
  build: {
    target: "esnext",
    outDir: "dist",
    rollupOptions: {
      // mermaid 的 sankey 图表依赖 d3-sankey，但该包未被打包进来；
      // @tauri-apps/api/* 只在 Tauri 桌面壳中存在，Web 构建时动态 import
      // 永远不会被执行，但 Rollup 仍会尝试解析——将其外部化即可。
      external: ["d3-sankey", /^@tauri-apps\//],
    },
  },
  resolve: {
    conditions: ["solid", "browser", "module", "import", "default"],
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
})
