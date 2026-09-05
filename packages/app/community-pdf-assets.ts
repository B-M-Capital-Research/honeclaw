import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import type { Plugin } from "vite";

/** PDF.js resolves these filenames itself; keep its fonts/decoders on our origin. */
export function communityPdfAssets(): Plugin {
  const root = dirname(createRequire(import.meta.url).resolve("pdfjs-dist/package.json"));
  const { version } = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
  const assets = new Map<string, string>();
  for (const directory of ["cmaps", "standard_fonts", "wasm", "iccs"]) {
    for (const entry of readdirSync(join(root, directory), { withFileTypes: true })) {
      if (entry.isFile()) {
        assets.set(`pdfjs/${version}/${directory}/${entry.name}`, join(root, directory, entry.name));
      }
    }
  }
  return {
    name: "community-pdf-assets",
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const pathname = new URL(request.url ?? "/", "http://localhost").pathname.slice(1);
        const path = assets.get(pathname);
        if (!path || !["GET", "HEAD"].includes(request.method ?? "")) return next();
        const contentType = path.endsWith(".wasm") ? "application/wasm"
          : /\.m?js$/.test(path) ? "text/javascript" : "application/octet-stream";
        response.setHeader("Content-Type", contentType);
        response.setHeader("X-Content-Type-Options", "nosniff");
        response.end(request.method === "HEAD" ? undefined : readFileSync(path));
      });
    },
    generateBundle() {
      for (const [fileName, path] of assets) {
        this.emitFile({ type: "asset", fileName, source: readFileSync(path) });
      }
    },
  };
}
