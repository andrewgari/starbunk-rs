"use server";

// Server actions for the /ragtime question editor. Every action re-checks the
// Discord OAuth admin session (server actions are reachable by direct POST, so
// the page-level gate is not enough). The admin token is only used inside
// @/lib/ragtime-api, which is server-only — the browser only ever talks to these
// actions, never to the ragtime API directly.

import { checkAdmin } from "@/lib/admin-session";
import {
  apiCreateQuestion,
  apiDeleteQuestion,
  apiListQuestions,
  apiUpdateQuestion,
} from "@/lib/ragtime-api";
import {
  validateQuestionInput,
  type ActionResult,
  type QuestionId,
  type RagtimeQuestion,
  type RagtimeQuestionInput,
} from "./questions";

async function guard(): Promise<string | null> {
  const gate = await checkAdmin();
  if (gate.status === "unauthenticated") return "Unauthorized: sign in with Discord.";
  if (gate.status === "forbidden") return "Forbidden: Discord ADMINISTRATOR permission required.";
  return null;
}

function message(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function validId(id: unknown): id is QuestionId {
  return (typeof id === "number" && Number.isFinite(id)) || (typeof id === "string" && id.length > 0);
}

export async function listQuestions(): Promise<ActionResult<RagtimeQuestion[]>> {
  const denied = await guard();
  if (denied) return { success: false, error: denied };
  try {
    return { success: true, data: await apiListQuestions() };
  } catch (e) {
    console.error("ragtime: list questions failed", e);
    return { success: false, error: message(e) };
  }
}

export async function createQuestion(input: RagtimeQuestionInput): Promise<ActionResult<RagtimeQuestion>> {
  const denied = await guard();
  if (denied) return { success: false, error: denied };
  const v = validateQuestionInput(input);
  if (!v.ok) return { success: false, error: v.error };
  try {
    return { success: true, data: await apiCreateQuestion(v.value) };
  } catch (e) {
    console.error("ragtime: create question failed", e);
    return { success: false, error: message(e) };
  }
}

export async function updateQuestion(
  id: QuestionId,
  input: RagtimeQuestionInput,
): Promise<ActionResult<RagtimeQuestion>> {
  const denied = await guard();
  if (denied) return { success: false, error: denied };
  if (!validId(id)) return { success: false, error: "Invalid question id." };
  const v = validateQuestionInput(input);
  if (!v.ok) return { success: false, error: v.error };
  try {
    return { success: true, data: await apiUpdateQuestion(id, v.value) };
  } catch (e) {
    console.error("ragtime: update question failed", e);
    return { success: false, error: message(e) };
  }
}

export async function deleteQuestion(id: QuestionId): Promise<ActionResult> {
  const denied = await guard();
  if (denied) return { success: false, error: denied };
  if (!validId(id)) return { success: false, error: "Invalid question id." };
  try {
    await apiDeleteQuestion(id);
    return { success: true, data: undefined };
  } catch (e) {
    console.error("ragtime: delete question failed", e);
    return { success: false, error: message(e) };
  }
}
