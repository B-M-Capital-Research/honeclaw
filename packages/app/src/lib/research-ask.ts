/**
 * Hand-off channel for "ask the agent about this saved report".
 *
 * A research panel composes a grounded prompt (see saved-report-prompt.ts)
 * that can run to kilobytes — far too big for a URL. The panel stashes it
 * here and navigates to `/chat?ask=research`; the chat page collects it once
 * on mount. sessionStorage keeps the hand-off tab-local and survives the
 * route-level code-split boundary.
 */
const KEY = "hone-research-ask";

export function stashResearchAsk(message: string): void {
  try {
    sessionStorage.setItem(KEY, message);
  } catch {
    // Storage full or unavailable: the navigation still happens, the user
    // just lands on an empty composer instead of an auto-sent question.
  }
}

export function takeResearchAsk(): string | undefined {
  try {
    const value = sessionStorage.getItem(KEY);
    if (value !== null) sessionStorage.removeItem(KEY);
    return value ?? undefined;
  } catch {
    return undefined;
  }
}
