# Offline Mermaid renderer

`mermaid-11.12.3.min.js` is the self-contained `dist/mermaid.min.js` from the repository's Bun-locked `mermaid@11.12.3` package. MIT license is retained in `mermaid-LICENSE`.

SHA-256: `71dc54e481779b526de5e615c2eb00c99fa5acb75da9c5524d02e333d2fda89a`.

To refresh deliberately, install the locked frontend dependencies, copy that package's `dist/mermaid.min.js` and `LICENSE`, update the renderer filename and this record, then run the renderer regression and a real CJK/diagram PDF smoke. No network fetch occurs while rendering. Strict Mermaid security and the HTML CSP keep report text from supplying executable scripts or external resources.

KaTeX 0.16.22 is vendored from the official npm package (`npm pack katex@0.16.22`). The minified script and MIT license are unchanged; CSS retains and embeds the official WOFF2 fonts as data URLs. Math uses `trust: false` and bounded expansion. SHA-256:

- `katex-0.16.22.inline.css`: `c87d2b61698d44a07331ee70f61e01ec7afa1c102903403c03f1f7d44fa4f746`
- `katex-0.16.22.min.js`: `e8d885505949f3a5f4abdd5dd0d53696bd1371ad26ffbf4f310dcd77c8cdae89`
- `katex-LICENSE`: `766ccc1f306c885aa45542a9846bbd0a505b27a0374f146778171c2254ce18e3`
