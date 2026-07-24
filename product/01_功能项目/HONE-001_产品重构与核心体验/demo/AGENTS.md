# Prototype Instructions

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

## HONE Live Demo decisions

- The selected visual source is `design-reference.png`, a refinement of visual option 1.
- Preserve option 1's professional investment research-desk structure.
- Reuse option 2's editorial reading hierarchy for the Insights experience.
- Reuse option 3's context chips, source-aware answers, and structured Agent workspace.
- Use actual shadcn/Radix components with a monochrome black, white, and neutral-gray theme.
- Use Phosphor icons for application navigation and actions; do not draw custom icons.
- The demo must include the public introduction homepage plus Investment, Insights, Agent, Tracking, and Me.
- Mobile uses five persistent bottom destinations with Agent in the emphasized center position; desktop uses a left sidebar.
- Keep the demo isolated from production code and use realistic mock data only.
- The selected Investment V3 direction is `investment-v3-reference.png`: use option 2's professional, holdings-first density as the primary structure, and absorb only option 3's compact Agent research-signal module. Do not turn the Investment home into a chat or content feed.
- Insights uses one unified post format. A post may contain text, images, and an optional deep-article attachment; do not add short-post/article category tabs. The current publishing model is one-way from Lao Wang/authorized editors, while the information architecture should remain extensible to future user posting and discussion.
- Tracking CTAs must demonstrate their destination states: event detail, generated result, preparation state, all-task management, task editing, calendar-day detail, and historical result.
- Me and the account menu must demonstrate research-preference editing, device/session management, quiet mode, dark mode, language selection as a UI-only example, and a complete plan-selection/order/payment/success subscription flow. No real payment is performed.
- Desktop sidebar priority is Workspace first, AI Research second: place Investment/Insights/Tracking directly below the brand, then New Research, research-history search, recent conversations, and All Research Records. Fix the account/avatar menu to the bottom-left rail. The desktop top-right area contains only global search; notifications and account controls remain available on mobile. Do not include a Return to Website action. Mobile keeps the existing five-item bottom navigation with centered Agent.
