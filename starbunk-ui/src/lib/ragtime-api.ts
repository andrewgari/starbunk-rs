// Server-only client for the ragtime admin API.
// RAGTIME_ADMIN_TOKEN is read here and only here; `server-only` makes the build
// fail if a client component ever imports this module, so the token can never
// end up in a browser bundle.
import "server-only";

import type { QuestionId, RagtimeQuestion, RagtimeQuestionInput } from "@/app/ragtime/questions";

const RAGTIME_API_URL = (process.env.RAGTIME_API_URL || "http://localhost:9085").replace(/\/+$/, "");

export class RagtimeApiError extends Error {
  constructor(message: string, public status?: number) {
    super(message);
    this.name = "RagtimeApiError";
  }
}

function headers(json: boolean): HeadersInit {
  const token = process.env.RAGTIME_ADMIN_TOKEN || "";
  return {
    Accept: "application/json",
    ...(json ? { "Content-Type": "application/json" } : {}),
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

async function errorFrom(res: Response): Promise<RagtimeApiError> {
  let detail = "";
  try {
    const text = await res.text();
    try {
      const body = JSON.parse(text) as { error?: unknown; message?: unknown };
      detail = String(body.error ?? body.message ?? text);
    } catch {
      detail = text;
    }
  } catch {
    // ignore body read failures
  }
  return new RagtimeApiError(
    `Ragtime API ${res.status}${detail ? `: ${detail}` : ` ${res.statusText}`}`,
    res.status,
  );
}

async function call(path: string, init: RequestInit & { json?: unknown } = {}): Promise<Response> {
  const { json, ...rest } = init;
  let res: Response;
  try {
    res = await fetch(`${RAGTIME_API_URL}${path}`, {
      ...rest,
      cache: "no-store",
      headers: headers(json !== undefined),
      body: json !== undefined ? JSON.stringify(json) : rest.body,
    });
  } catch (e) {
    throw new RagtimeApiError(`Ragtime API unreachable: ${e instanceof Error ? e.message : String(e)}`);
  }
  if (!res.ok) throw await errorFrom(res);
  return res;
}

const BASE = "/api/ragtime/questions";
const one = (id: QuestionId) => `${BASE}/${encodeURIComponent(String(id))}`;

export async function apiListQuestions(): Promise<RagtimeQuestion[]> {
  const res = await call(BASE);
  const body = (await res.json()) as RagtimeQuestion[] | { questions?: RagtimeQuestion[] };
  return Array.isArray(body) ? body : body.questions ?? [];
}

export async function apiCreateQuestion(input: RagtimeQuestionInput): Promise<RagtimeQuestion> {
  const res = await call(BASE, { method: "POST", json: input });
  return (await res.json()) as RagtimeQuestion;
}

export async function apiUpdateQuestion(id: QuestionId, input: RagtimeQuestionInput): Promise<RagtimeQuestion> {
  const res = await call(one(id), { method: "PUT", json: { id, ...input } });
  const text = await res.text();
  return text ? (JSON.parse(text) as RagtimeQuestion) : { id, ...input };
}

export async function apiDeleteQuestion(id: QuestionId): Promise<void> {
  await call(one(id), { method: "DELETE" });
}
