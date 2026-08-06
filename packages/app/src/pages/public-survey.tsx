// public-survey.tsx — 无登录用户调研问卷。题库来自 CONTENT，逻辑在 survey-model。

import { For, Show, createEffect, createSignal, onMount } from "solid-js"
import { Meta, Title } from "@solidjs/meta"
import { useNavigate } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import {
  OTHER_VALUE,
  SURVEY_DRAFT_KEY,
  type SurveyAnswers,
  type SurveyQuestion,
  answeredCount,
  buildSubmission,
  isChoiceDisabled,
  isSubmittable,
  otherKeyFor,
  parseDraft,
  serializeDraft,
  submitSurvey,
  toggleChoice,
} from "@/lib/survey-model"
import "./public-site.css"
import "./public-survey.css"

function readDraftStorage(key: string): string | null {
  try {
    return window.localStorage.getItem(key)
  } catch {
    return null
  }
}

function writeDraftStorage(key: string, value: string | null) {
  try {
    if (value === null) window.localStorage.removeItem(key)
    else window.localStorage.setItem(key, value)
  } catch {
    // Private-browsing quota errors must not break the form.
  }
}

export default function PublicSurveyPage() {
  const navigate = useNavigate()
  const S = () => CONTENT.survey
  const questions = () => S().questions as unknown as SurveyQuestion[]

  const [answers, setAnswers] = createSignal<SurveyAnswers>({})
  const [contact, setContact] = createSignal("")
  const [submitting, setSubmitting] = createSignal(false)
  const [done, setDone] = createSignal(false)
  const [error, setError] = createSignal("")
  const [restored, setRestored] = createSignal(false)

  onMount(() => {
    const draft = parseDraft(readDraftStorage(SURVEY_DRAFT_KEY), Date.now())
    if (!draft) return
    setAnswers(draft.answers)
    setContact(draft.contact)
    setRestored(Object.keys(draft.answers).length > 0)
  })

  // Saving on every change is what makes a 12-question form survive an
  // accidental refresh or a phone call mid-answer.
  createEffect(() => {
    const current = answers()
    const currentContact = contact()
    if (done()) return
    if (Object.keys(current).length === 0 && !currentContact) return
    writeDraftStorage(SURVEY_DRAFT_KEY, serializeDraft(current, currentContact, Date.now()))
  })

  const setAnswer = (id: string, value: string | string[]) => {
    setError("")
    setRestored(false)
    setAnswers((previous) => ({ ...previous, [id]: value }))
  }

  const total = () => questions().length
  const answered = () => answeredCount(questions(), answers())
  const progressLabel = () =>
    S().progress.replace("{done}", String(answered())).replace("{total}", String(total()))

  const pickedOther = (question: SurveyQuestion) => {
    const value = answers()[question.id]
    return Array.isArray(value) ? value.includes(OTHER_VALUE) : value === OTHER_VALUE
  }

  const submit = async () => {
    if (submitting()) return
    const payload = buildSubmission(questions(), answers())
    if (!isSubmittable(questions(), answers())) {
      setError(S().empty_error)
      return
    }
    setSubmitting(true)
    setError("")
    const result = await submitSurvey({ answers: payload, contact: contact() })
    setSubmitting(false)
    if (result.ok) {
      writeDraftStorage(SURVEY_DRAFT_KEY, null)
      setDone(true)
      window.scrollTo({ top: 0, left: 0, behavior: "smooth" })
      return
    }
    setError(result.message || S().error)
  }

  return (
    <>
      <Title>{S().title} | HONE</Title>
      <Meta name="description" content={S().intro} />
      <PublicNav />
      <main class="survey-main">
        <Show
          when={!done()}
          fallback={
            <section class="survey-done">
              <div class="survey-done-mark" aria-hidden="true">
                ✓
              </div>
              <h1>{S().success_title}</h1>
              <p>{S().success_body}</p>
              <div class="survey-done-actions">
                <button class="survey-secondary" onClick={() => navigate("/")}>
                  {S().success_back}
                </button>
                <button class="survey-primary" onClick={() => navigate("/chat")}>
                  {S().success_chat}
                </button>
              </div>
            </section>
          }
        >
          <header class="survey-head">
            <div class="survey-eyebrow">{S().eyebrow}</div>
            <h1>{S().title}</h1>
            <p class="survey-intro">{S().intro}</p>
            <p class="survey-privacy">{S().privacy}</p>
          </header>

          <div class="survey-progress" role="status">
            <div class="survey-progress-track">
              <div
                class="survey-progress-fill"
                style={{ width: `${total() ? (answered() / total()) * 100 : 0}%` }}
              />
            </div>
            <span>{progressLabel()}</span>
          </div>

          <Show when={restored()}>
            <p class="survey-restored">{S().draft_restored}</p>
          </Show>

          <ol class="survey-questions">
            <For each={questions()}>
              {(question, index) => (
                <li class="survey-question">
                  <div class="survey-question-head">
                    <span class="survey-question-index">{index() + 1}</span>
                    <div>
                      <h2>{question.title}</h2>
                      <Show when={question.hint}>
                        <p class="survey-question-hint">{question.hint}</p>
                      </Show>
                      <Show when={question.type === "multi" && question.max > 0}>
                        <p class="survey-question-limit">
                          {S().limit_hint.replace("{max}", String(question.max))}
                        </p>
                      </Show>
                    </div>
                  </div>

                  <Show when={question.type === "text"}>
                    <textarea
                      class="survey-textarea"
                      rows={4}
                      placeholder={S().optional}
                      value={(answers()[question.id] as string) ?? ""}
                      onInput={(event) => setAnswer(question.id, event.currentTarget.value)}
                    />
                  </Show>

                  <Show when={question.type !== "text"}>
                    <div class="survey-options">
                      <For each={question.options}>
                        {(option) => {
                          const selected = () => {
                            const value = answers()[question.id]
                            return Array.isArray(value)
                              ? value.includes(option.value)
                              : value === option.value
                          }
                          const disabled = () =>
                            question.type === "multi" &&
                            isChoiceDisabled(
                              answers()[question.id] as string[] | undefined,
                              option.value,
                              question.max,
                            )
                          return (
                            <button
                              type="button"
                              class="survey-option"
                              classList={{
                                "is-selected": selected(),
                                "is-disabled": disabled(),
                              }}
                              aria-pressed={selected()}
                              disabled={disabled()}
                              onClick={() => {
                                if (question.type === "single") {
                                  setAnswer(
                                    question.id,
                                    selected() ? "" : option.value,
                                  )
                                  return
                                }
                                setAnswer(
                                  question.id,
                                  toggleChoice(
                                    answers()[question.id] as string[] | undefined,
                                    option.value,
                                    question.max,
                                  ),
                                )
                              }}
                            >
                              {option.label}
                            </button>
                          )
                        }}
                      </For>
                      <Show when={question.allow_other}>
                        {(() => {
                          const selected = () => pickedOther(question)
                          return (
                            <button
                              type="button"
                              class="survey-option"
                              classList={{ "is-selected": selected() }}
                              aria-pressed={selected()}
                              onClick={() => {
                                if (question.type === "single") {
                                  setAnswer(question.id, selected() ? "" : OTHER_VALUE)
                                  return
                                }
                                setAnswer(
                                  question.id,
                                  toggleChoice(
                                    answers()[question.id] as string[] | undefined,
                                    OTHER_VALUE,
                                    question.max,
                                  ),
                                )
                              }}
                            >
                              {S().other_label}
                            </button>
                          )
                        })()}
                      </Show>
                    </div>
                    <Show when={question.allow_other && pickedOther(question)}>
                      <input
                        class="survey-other-input"
                        type="text"
                        placeholder={S().other_placeholder}
                        value={(answers()[otherKeyFor(question.id)] as string) ?? ""}
                        onInput={(event) =>
                          setAnswer(otherKeyFor(question.id), event.currentTarget.value)
                        }
                      />
                    </Show>
                  </Show>
                </li>
              )}
            </For>
          </ol>

          <section class="survey-contact">
            <h2>{S().contact_title}</h2>
            <label for="survey-contact-input">{S().contact_label}</label>
            <input
              id="survey-contact-input"
              type="text"
              autocomplete="off"
              placeholder={S().contact_placeholder}
              value={contact()}
              onInput={(event) => {
                setError("")
                setContact(event.currentTarget.value)
              }}
            />
            <p class="survey-contact-hint">{S().contact_hint}</p>
          </section>

          <Show when={error()}>
            <p class="survey-error" role="alert">
              {error()}
            </p>
          </Show>

          <div class="survey-submit-row">
            <button
              class="survey-primary"
              disabled={submitting()}
              onClick={() => void submit()}
            >
              {submitting() ? S().submitting : S().submit}
            </button>
            <span class="survey-submit-note">{progressLabel()}</span>
          </div>
        </Show>
      </main>
      <PublicFooter />
    </>
  )
}
