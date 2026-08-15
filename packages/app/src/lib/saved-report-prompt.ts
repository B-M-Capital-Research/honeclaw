/**
 * Builds the "ask the agent about a saved report" message.
 *
 * Every research panel used to hand-roll the same envelope: an HTML comment
 * carrying a compact JSON projection of the saved snapshot, an intro line and
 * a list of grounding rules. The envelope shape is a protocol — the backend
 * strips everything before the marker from visible history — so it lives in
 * exactly one place. Domain rules stay with the caller because they differ
 * meaningfully per report type.
 */
export function buildSavedReportPrompt(opts: {
  /** Marker name, e.g. `HONE_SAVED_DAILY_SIGNAL_REPORT`. */
  marker: string;
  /** Compact projection of the saved snapshot — only what answering needs. */
  payload: unknown;
  /** Subject clause, e.g. `已保存的宏观红绿灯（报告日 2026-08-12）`. */
  subject: string;
  question: string;
  /** Domain grounding rules, joined with `；`. */
  rules: string[];
}): string {
  return [
    `<!-- ${opts.marker}`,
    JSON.stringify(opts.payload),
    `END_${opts.marker} -->`,
    `基于${opts.subject}回答：${opts.question}`,
    `要求：${opts.rules.join("；")}。`,
  ].join("\n");
}
