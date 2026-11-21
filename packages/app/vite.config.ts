import { createHash } from "node:crypto"
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { defineConfig } from "vite"
import solid from "vite-plugin-solid"
import tailwindcss from "@tailwindcss/vite"
import { communityPdfAssets } from "./community-pdf-assets"

const backend = process.env.HONE_WEB_BACKEND_URL ?? "http://127.0.0.1:8077"
const port = Number(process.env.HONE_APP_PORT ?? "3000")
const outDir = process.env.HONE_APP_OUT_DIR ?? "dist"
const appSurface = process.env.HONE_APP_SURFACE ?? process.env.VITE_HONE_APP_SURFACE ?? "admin"
const communityEdgeDiscovery = process.env.HONE_APP_COMMUNITY_EDGE_DISCOVERY ?? "0"
const desktopRelativeBase =
  process.env.HONE_APP_RELATIVE_BASE === "1" || Boolean(process.env.TAURI_ENV_PLATFORM)

export default defineConfig({
  base: desktopRelativeBase ? "./" : "/",
  define: {
    "import.meta.env.VITE_HONE_APP_SURFACE": JSON.stringify(appSurface),
    "import.meta.env.VITE_HONE_APP_COMMUNITY_EDGE_DISCOVERY": JSON.stringify(
      communityEdgeDiscovery,
    ),
  },
  plugins: [
    solid(),
    tailwindcss(),
    communityPdfAssets(),
    {
      name: "hone-adventure-static-dir",
      configureServer(server) {
        server.middlewares.use((req, _res, next) => {
          if (req.url === "/hone-adventure/") {
            req.url = "/hone-adventure/index.html";
          } else if (req.url?.startsWith("/hone-adventure/?")) {
            req.url = "/hone-adventure/index.html" + req.url.slice("/hone-adventure/".length);
          }
          next();
        });
      },
      closeBundle() {
        const gameDir = resolve(process.cwd(), outDir, "hone-adventure");
        if (!existsSync(gameDir)) return;
        const files = readdirSync(gameDir)
          .filter((f) => statSync(join(gameDir, f)).isFile())
          .sort();
        const hasher = createHash("sha256");
        for (const f of files) {
          hasher.update(f + "\0");
          if (/\.(?:html|js|css)$/i.test(f)) {
            const norm = readFileSync(join(gameDir, f), "utf8").replace(
              /(\.\/[a-z0-9_-]+\.(?:js|css|webp))(?:\?v=[^"'\s)]+)?/gi,
              "$1",
            );
            hasher.update(norm);
          } else {
            hasher.update(readFileSync(join(gameDir, f)));
          }
        }
        const digest = hasher.digest("hex").slice(0, 10);
        for (const f of files) {
          if (!/\.(?:html|js)$/i.test(f)) continue;
          const p = join(gameDir, f);
          const src = readFileSync(p, "utf8");
          const stamped = src.replace(
            /(\.\/[a-z0-9_-]+\.(?:js|css|webp))(?:\?v=[^"'\s)]+)?/gi,
            `$1?v=${digest}`,
          );
          if (stamped !== src) writeFileSync(p, stamped);
        }
      },
    },
  ],
  optimizeDeps: {
    // Avoid a dev-server reload when the PDF display module is first opened.
    include: ["pdfjs-dist/legacy/build/pdf.mjs"],
  },
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
      output: {
        // 打开 /chat 会拉起二十多个分块，其中十几个不到 5 KB：下载各自只要
        // 几毫秒，一次往返却躲不掉。在延迟高的链路上，代价全在请求个数上，
        // 跟体积没关系。把碎块并进它们的邻居，省下的是往返次数。
        // 阈值取 24 KB：再往上（48 KB）请求数不再下降，只是徒增合并范围；
        // 再往下（8 KB）会漏掉一批 10 KB 上下的碎块。
        experimentalMinChunkSize: 24 * 1024,
      },
    },
  },
  resolve: {
    conditions: ["solid", "browser", "module", "import", "default"],
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
})
