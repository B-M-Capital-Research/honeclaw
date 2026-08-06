// survey-model.ts — 问卷的纯逻辑：选项开关、进度、草稿、提交载荷。
//
// 抽出来单独放的原因：这些规则（最多选 N 项、"其他" 怎么存、草稿何时失效）
// 每一条都能悄无声息地坏掉，而在组件里它们只能靠点开页面手动点才能验证。

import { buildApiUrl } from "./backend"
import { useLocale } from "./i18n"

export type SurveyQuestionType = "single" | "multi" | "text"

export type SurveyOption = { value: string; label: string }

export type SurveyQuestion = {
  id: string
  type: SurveyQuestionType
  title: string
  hint: string
  /** 0 = 不限。仅对 multi 生效。 */
  max: number
  allow_other: boolean
  options: SurveyOption[]
}

/** 单选与开放题存字符串，多选存数组；与后端的结构校验一一对应。 */
export type SurveyAnswers = Record<string, string | string[]>

export const OTHER_VALUE = "other"
export const SURVEY_DRAFT_KEY = "hone.survey.draft.v1"
/** 草稿保留一周：再久之前填的半份问卷，用户多半已经忘了自己填过什么。 */
export const SURVEY_DRAFT_TTL_MS = 7 * 24 * 60 * 60 * 1000

export function otherKeyFor(questionId: string): string {
  return `${questionId}_other`
}

/**
 * 切换一个多选项。已达上限时**忽略新增而不是替换**：静默顶掉用户之前的选择，
 * 比拒绝新增更让人困惑。
 */
export function toggleChoice(
  current: string[] | undefined,
  value: string,
  max: number,
): string[] {
  const selected = current ?? []
  if (selected.includes(value)) {
    return selected.filter((item) => item !== value)
  }
  if (max > 0 && selected.length >= max) {
    return selected
  }
  return [...selected, value]
}

export function isChoiceDisabled(
  current: string[] | undefined,
  value: string,
  max: number,
): boolean {
  const selected = current ?? []
  return max > 0 && selected.length >= max && !selected.includes(value)
}

function hasAnswer(value: string | string[] | undefined): boolean {
  if (Array.isArray(value)) return value.length > 0
  return typeof value === "string" && value.trim().length > 0
}

export function answeredCount(
  questions: SurveyQuestion[],
  answers: SurveyAnswers,
): number {
  return questions.filter((question) => hasAnswer(answers[question.id])).length
}

/**
 * 构造提交载荷。两条规则：空答案不发（后端也会丢，但不发能少一次无谓的往返），
 * 以及只有真的勾了 "其他" 才带上它的补充文本——否则用户勾了又取消，
 * 那段文字会变成一条没有归属的孤儿答案。
 */
export function buildSubmission(
  questions: SurveyQuestion[],
  answers: SurveyAnswers,
): SurveyAnswers {
  const payload: SurveyAnswers = {}
  for (const question of questions) {
    const value = answers[question.id]
    if (hasAnswer(value)) {
      payload[question.id] = Array.isArray(value)
        ? [...value]
        : (value as string).trim()
    }
    if (!question.allow_other) continue
    const selected = answers[question.id]
    const pickedOther = Array.isArray(selected)
      ? selected.includes(OTHER_VALUE)
      : selected === OTHER_VALUE
    if (!pickedOther) continue
    const other = answers[otherKeyFor(question.id)]
    if (hasAnswer(other) && typeof other === "string") {
      payload[otherKeyFor(question.id)] = other.trim()
    }
  }
  return payload
}

export function isSubmittable(
  questions: SurveyQuestion[],
  answers: SurveyAnswers,
): boolean {
  return Object.keys(buildSubmission(questions, answers)).length > 0
}

type StoredDraft = { savedAt: number; answers: SurveyAnswers; contact?: string }

export function serializeDraft(
  answers: SurveyAnswers,
  contact: string,
  now: number,
): string {
  return JSON.stringify({ savedAt: now, answers, contact } satisfies StoredDraft)
}

/** 解析草稿。过期、损坏或形状不对时一律当作没有草稿，而不是抛错拦住页面。 */
export function parseDraft(
  raw: string | null,
  now: number,
): { answers: SurveyAnswers; contact: string } | null {
  if (!raw) return null
  try {
    const parsed = JSON.parse(raw) as StoredDraft
    if (!parsed || typeof parsed !== "object") return null
    if (typeof parsed.savedAt !== "number") return null
    if (now - parsed.savedAt > SURVEY_DRAFT_TTL_MS) return null
    const answers = parsed.answers
    if (!answers || typeof answers !== "object" || Array.isArray(answers)) return null
    const cleaned: SurveyAnswers = {}
    for (const [key, value] of Object.entries(answers)) {
      if (typeof value === "string") cleaned[key] = value
      else if (Array.isArray(value) && value.every((item) => typeof item === "string")) {
        cleaned[key] = value as string[]
      }
    }
    return {
      answers: cleaned,
      contact: typeof parsed.contact === "string" ? parsed.contact : "",
    }
  } catch {
    return null
  }
}

export async function submitSurvey(input: {
  answers: SurveyAnswers
  contact: string
}): Promise<{ ok: true } | { ok: false; message: string }> {
  const contact = input.contact.trim()
  const response = await fetch(buildApiUrl("/api/public/survey"), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      locale: useLocale(),
      answers: input.answers,
      ...(contact ? { contact } : {}),
    }),
  })
  if (response.ok) return { ok: true }
  // The server's message is the useful one — a rate-limit reply explains
  // exactly why nothing was saved.
  const message = await response
    .json()
    .then((body: { error?: string; message?: string }) => body?.error ?? body?.message ?? "")
    .catch(() => "")
  return { ok: false, message }
}
