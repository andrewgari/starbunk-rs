// Shared (client + server) types and pure helpers for the Ragtime question editor.
// Contains no secrets — safe to import from client components.

export type QuestionId = number | string;

/** Wire shape of a trivia question as returned/accepted by the ragtime admin API. */
export interface RagtimeQuestion {
  id: QuestionId;
  question: string;
  display_answer: string;
  acceptable_answers: string[];
}

/** Payload for create/update (the API assigns ids). */
export type RagtimeQuestionInput = Omit<RagtimeQuestion, "id">;

export type ActionResult<T = undefined> =
  | { success: true; data: T }
  | { success: false; error: string };

/**
 * Parse the "Acceptable answers" textarea. Accepts one answer per line and/or
 * comma-separated answers: split on newlines and commas, trim, drop blanks.
 */
export function parseAcceptableAnswers(text: string): string[] {
  return text
    .split(/[\n,]/)
    .map((a) => a.trim())
    .filter((a) => a.length > 0);
}

/** Render acceptable answers for the textarea: one per line. */
export function formatAcceptableAnswers(answers: string[]): string {
  return answers.join("\n");
}

/** Normalise + validate user input. Returns an error message or the clean payload. */
export function validateQuestionInput(
  input: RagtimeQuestionInput,
): { ok: true; value: RagtimeQuestionInput } | { ok: false; error: string } {
  const question = (input.question ?? "").trim();
  const display_answer = (input.display_answer ?? "").trim();
  const acceptable_answers = (input.acceptable_answers ?? [])
    .map((a) => String(a).trim())
    .filter((a) => a.length > 0);

  if (!question) return { ok: false, error: "Question must not be empty." };
  if (!display_answer) return { ok: false, error: "Display answer must not be empty." };
  return { ok: true, value: { question, display_answer, acceptable_answers } };
}
