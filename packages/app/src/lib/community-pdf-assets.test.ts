import { describe, expect, it } from "bun:test";
import { communityPdfAssets } from "../../community-pdf-assets";
import { version } from "pdfjs-dist/package.json";

function readAsset(path: string, method = "GET") {
  const plugin = communityPdfAssets();
  const headers = new Map<string, string>();
  let body: Buffer | undefined;
  let next = false;
  let serve: (req: unknown, res: unknown, next: () => void) => void = () => {};
  if (typeof plugin.configureServer !== "function") throw new Error("missing dev asset handler");
  plugin.configureServer.call({} as never, {
    middlewares: { use(handler: typeof serve) { serve = handler; } },
  } as never);
  serve({ url: path, method }, {
    setHeader(key: string, value: string) { headers.set(key, value); },
    end(bytes: Buffer | undefined) { body = bytes; },
  }, () => { next = true; });
  return { headers, body, next };
}

describe("community PDF decoder assets", () => {
  it("serves the JS fallback with a module-compatible MIME despite nosniff", () => {
    const result = readAsset(`/pdfjs/${version}/wasm/openjpeg_nowasm_fallback.js`);
    expect(result.headers.get("Content-Type")).toBe("text/javascript");
    expect(result.headers.get("X-Content-Type-Options")).toBe("nosniff");
    expect(result.body!.length).toBeGreaterThan(1000);
    expect(result.next).toBe(false);
  });

  it("keeps wasm binary and supports HEAD without sending the body", () => {
    const result = readAsset(`/pdfjs/${version}/wasm/openjpeg.wasm`, "HEAD");
    expect(result.headers.get("Content-Type")).toBe("application/wasm");
    expect(result.body).toBeUndefined();
    expect(result.next).toBe(false);
  });

  it("serves only the known package assets", () => {
    expect(readAsset(`/pdfjs/${version}/wasm/../../package.json`).next).toBe(true);
    expect(readAsset(`/pdfjs/${version}/wasm/not-present.js`).next).toBe(true);
    expect(readAsset(`/pdfjs/${version}/wasm/openjpeg.wasm`, "POST").next).toBe(true);
  });
});
