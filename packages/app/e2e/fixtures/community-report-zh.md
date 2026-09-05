# Community PDF rendering fixture

`community-report-zh.pdf` is an original two-page Chinese test document printed
with Playwright Chromium from HTML, with embedded PingFang glyph subsets. It
contains no user/community source material. Page one has a blue chart and page
two an orange chart, so the browser test can distinguish actual rendered pages
from an empty canvas, a repeated first page, or a blocked native PDF plugin.

The E2E verifies page count, Chinese text extraction, dark text and colored pixels,
zoom, an early download joining the pending authenticated request, and termination
of the PDF worker when the preview closes. `sample-report.pdf` remains the small
intentionally incomplete transport fixture used by unrelated chat upload tests.
