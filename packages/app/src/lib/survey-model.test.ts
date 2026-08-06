import { describe, expect, it } from "bun:test"

import {
  OTHER_VALUE,
  SURVEY_DRAFT_TTL_MS,
  type SurveyQuestion,
  answeredCount,
  buildSubmission,
  isChoiceDisabled,
  isSubmittable,
  otherKeyFor,
  parseDraft,
  serializeDraft,
  toggleChoice,
} from "./survey-model"

const QUESTIONS: SurveyQuestion[] = [
  {
    id: "q1",
    type: "single",
    title: "",
    hint: "",
    max: 0,
    allow_other: false,
    options: [
      { value: "daily", label: "" },
      { value: "weekly", label: "" },
    ],
  },
  {
    id: "q2",
    type: "multi",
    title: "",
    hint: "",
    max: 2,
    allow_other: true,
    options: [
      { value: "fundamentals", label: "" },
      { value: "sector", label: "" },
      { value: OTHER_VALUE, label: "" },
    ],
  },
  {
    id: "q11",
    type: "text",
    title: "",
    hint: "",
    max: 0,
    allow_other: false,
    options: [],
  },
]

describe("multi-select limits", () => {
  it("ignores an extra pick instead of silently replacing an earlier one", () => {
    // Dropping the user's first answer to make room is the confusing behaviour:
    // they would see a box quietly uncheck itself somewhere else on the page.
    const atLimit = ["fundamentals", "sector"]
    expect(toggleChoice(atLimit, OTHER_VALUE, 2)).toEqual(atLimit)
    expect(isChoiceDisabled(atLimit, OTHER_VALUE, 2)).toBe(true)
  })

  it("always allows deselecting, even at the limit", () => {
    expect(toggleChoice(["fundamentals", "sector"], "sector", 2)).toEqual([
      "fundamentals",
    ])
    expect(isChoiceDisabled(["fundamentals", "sector"], "sector", 2)).toBe(false)
  })

  it("treats max 0 as unlimited", () => {
    expect(toggleChoice(["a", "b", "c"], "d", 0)).toEqual(["a", "b", "c", "d"])
    expect(isChoiceDisabled(["a", "b", "c"], "d", 0)).toBe(false)
  })
})

describe("submission payload", () => {
  it("drops blank answers and trims prose", () => {
    const payload = buildSubmission(QUESTIONS, {
      q1: "daily",
      q2: [],
      q11: "  想要每天早上的持仓提醒  ",
    })

    expect(payload).toEqual({ q1: "daily", q11: "想要每天早上的持仓提醒" })
  })

  it("keeps the other text only while other is actually selected", () => {
    const withOther = buildSubmission(QUESTIONS, {
      q2: ["fundamentals", OTHER_VALUE],
      [otherKeyFor("q2")]: "期权策略",
    })
    expect(withOther[otherKeyFor("q2")]).toBe("期权策略")

    // Typed something, then unticked "other": that text now belongs to no
    // answer and must not be shipped as an orphan record.
    const unticked = buildSubmission(QUESTIONS, {
      q2: ["fundamentals"],
      [otherKeyFor("q2")]: "期权策略",
    })
    expect(unticked[otherKeyFor("q2")]).toBeUndefined()
  })

  it("refuses to submit a completely empty form", () => {
    expect(isSubmittable(QUESTIONS, {})).toBe(false)
    expect(isSubmittable(QUESTIONS, { q11: "   " })).toBe(false)
    expect(isSubmittable(QUESTIONS, { q1: "daily" })).toBe(true)
  })

  it("counts progress over answered questions only", () => {
    expect(answeredCount(QUESTIONS, {})).toBe(0)
    expect(answeredCount(QUESTIONS, { q1: "daily", q2: [], q11: "x" })).toBe(2)
  })
})

describe("draft persistence", () => {
  const now = 1_800_000_000_000

  it("round-trips answers and contact", () => {
    const raw = serializeDraft({ q1: "daily", q2: ["sector"] }, "a@example.com", now)
    expect(parseDraft(raw, now + 1_000)).toEqual({
      answers: { q1: "daily", q2: ["sector"] },
      contact: "a@example.com",
    })
  })

  it("discards a draft older than the retention window", () => {
    const raw = serializeDraft({ q1: "daily" }, "", now)
    expect(parseDraft(raw, now + SURVEY_DRAFT_TTL_MS + 1)).toBeNull()
  })

  it("never throws on damaged storage", () => {
    // localStorage is shared with everything else on the origin; a corrupt or
    // foreign value must not stop the page from rendering.
    expect(parseDraft(null, now)).toBeNull()
    expect(parseDraft("not json", now)).toBeNull()
    expect(parseDraft("[1,2,3]", now)).toBeNull()
    expect(parseDraft(JSON.stringify({ answers: {} }), now)).toBeNull()
    expect(
      parseDraft(JSON.stringify({ savedAt: now, answers: { q1: 42 } }), now),
    ).toEqual({ answers: {}, contact: "" })
  })
})
