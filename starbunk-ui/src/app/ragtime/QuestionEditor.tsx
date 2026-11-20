"use client";

import { useCallback, useEffect, useState } from "react";
import { createQuestion, deleteQuestion, listQuestions, updateQuestion } from "./actions";
import {
  formatAcceptableAnswers,
  parseAcceptableAnswers,
  type QuestionId,
  type RagtimeQuestion,
  type RagtimeQuestionInput,
} from "./questions";

interface Row {
  key: string; // stable React key (ids don't exist for unsaved rows)
  id: QuestionId | null; // null = new, not yet saved
  question: string;
  displayAnswer: string;
  answersText: string;
  saved: { question: string; displayAnswer: string; answers: string[] } | null;
  error?: string;
}

type Flash = { type: "success" | "error"; text: string } | null;

let keySeq = 0;
const nextKey = () => `row-${++keySeq}`;

function rowFromQuestion(q: RagtimeQuestion): Row {
  return {
    key: nextKey(),
    id: q.id,
    question: q.question,
    displayAnswer: q.display_answer,
    answersText: formatAcceptableAnswers(q.acceptable_answers),
    saved: { question: q.question, displayAnswer: q.display_answer, answers: q.acceptable_answers },
  };
}

function toInput(r: Row): RagtimeQuestionInput {
  return {
    question: r.question.trim(),
    display_answer: r.displayAnswer.trim(),
    acceptable_answers: parseAcceptableAnswers(r.answersText),
  };
}

function isDirty(r: Row): boolean {
  if (!r.saved) return true;
  const cur = toInput(r);
  return (
    cur.question !== r.saved.question.trim() ||
    cur.display_answer !== r.saved.displayAnswer.trim() ||
    JSON.stringify(cur.acceptable_answers) !== JSON.stringify(r.saved.answers)
  );
}

const inputClass =
  "w-full bg-black/30 border border-white/10 rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-blue-500";

export default function QuestionEditor() {
  const [rows, setRows] = useState<Row[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [flash, setFlash] = useState<Flash>(null);

  const load = useCallback(async () => {
    setLoading(true);
    const res = await listQuestions();
    if (res.success) {
      setRows(res.data.map(rowFromQuestion));
    } else {
      setFlash({ type: "error", text: `Failed to load questions: ${res.error}` });
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    let active = true;
    (async () => {
      const res = await listQuestions();
      if (!active) return;
      if (res.success) setRows(res.data.map(rowFromQuestion));
      else setFlash({ type: "error", text: `Failed to load questions: ${res.error}` });
      setLoading(false);
    })();
    return () => {
      active = false;
    };
  }, []);

  const dirtyCount = rows.filter(isDirty).length;

  // Warn before navigating away with unsaved edits.
  useEffect(() => {
    if (dirtyCount === 0) return;
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      e.preventDefault();
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [dirtyCount]);

  const patchRow = (key: string, patch: Partial<Row>) =>
    setRows((rs) => rs.map((r) => (r.key === key ? { ...r, ...patch, error: undefined } : r)));

  const addRow = () => {
    setRows((rs) => [
      ...rs,
      { key: nextKey(), id: null, question: "", displayAnswer: "", answersText: "", saved: null },
    ]);
    setFlash(null);
  };

  const removeRow = async (row: Row) => {
    const label = row.question.trim() || "this new question";
    if (!window.confirm(`Delete "${label}"? This cannot be undone.`)) return;
    setFlash(null);
    if (row.id === null) {
      setRows((rs) => rs.filter((r) => r.key !== row.key));
      return;
    }
    setBusy(true);
    const res = await deleteQuestion(row.id);
    setBusy(false);
    if (res.success) {
      setRows((rs) => rs.filter((r) => r.key !== row.key));
      setFlash({ type: "success", text: "Question deleted." });
    } else {
      setRows((rs) => rs.map((r) => (r.key === row.key ? { ...r, error: res.error } : r)));
      setFlash({ type: "error", text: `Delete failed: ${res.error}` });
    }
  };

  const saveAll = async () => {
    const pending = rows.filter(isDirty);
    if (pending.length === 0) return;
    setBusy(true);
    setFlash(null);

    const results = new Map<string, { saved?: RagtimeQuestion; error?: string }>();
    // Sequential keeps ordering of new rows stable and the API load trivial.
    for (const row of pending) {
      const input = toInput(row);
      const res = row.id === null ? await createQuestion(input) : await updateQuestion(row.id, input);
      results.set(row.key, res.success ? { saved: res.data } : { error: res.error });
    }

    setRows((rs) =>
      rs.map((r) => {
        const out = results.get(r.key);
        if (!out) return r;
        if (out.error) return { ...r, error: out.error };
        const q = out.saved!;
        return { ...rowFromQuestion(q), key: r.key };
      }),
    );
    setBusy(false);

    const failed = [...results.values()].filter((r) => r.error).length;
    const ok = results.size - failed;
    setFlash(
      failed === 0
        ? { type: "success", text: `Saved ${ok} question${ok === 1 ? "" : "s"}.` }
        : { type: "error", text: `Saved ${ok}, failed ${failed}. See highlighted rows.` },
    );
  };

  const reload = async () => {
    if (dirtyCount > 0 && !window.confirm("Discard unsaved changes and reload from the server?")) return;
    setFlash(null);
    await load();
  };

  return (
    <div className="flex flex-col gap-4 max-w-6xl mx-auto py-6">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Ragtime Questions</h1>
          <p className="text-slate-400 mt-1">
            Trivia questions. Acceptable answers: one per line or comma-separated.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {dirtyCount > 0 ? (
            <span className="text-amber-400 text-sm font-medium">● {dirtyCount} unsaved</span>
          ) : (
            !loading && <span className="text-slate-500 text-sm">All changes saved</span>
          )}
          <button onClick={reload} disabled={busy || loading} className="btn-secondary disabled:opacity-50">
            Reload
          </button>
          <button onClick={addRow} disabled={busy || loading} className="btn-secondary disabled:opacity-50">
            + Add row
          </button>
          <button
            onClick={saveAll}
            disabled={busy || loading || dirtyCount === 0}
            className="btn-primary disabled:opacity-50"
          >
            {busy ? "Saving…" : "Save"}
          </button>
        </div>
      </header>

      {flash && (
        <div
          role="status"
          className={`px-4 py-2 rounded-lg text-sm border ${
            flash.type === "success"
              ? "bg-green-500/10 text-green-400 border-green-500/30"
              : "bg-red-500/10 text-red-400 border-red-500/30"
          }`}
        >
          {flash.text}
        </div>
      )}

      {loading ? (
        <div className="text-slate-400 animate-pulse">Loading questions…</div>
      ) : rows.length === 0 ? (
        <div className="glass-panel p-6 text-slate-400">No questions yet. Click “+ Add row” to create one.</div>
      ) : (
        <div className="flex flex-col gap-3">
          <div className="hidden md:grid grid-cols-[1fr_14rem_14rem_5rem] gap-3 px-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">
            <div>Question</div>
            <div>Display answer</div>
            <div>Acceptable answers</div>
            <div />
          </div>
          {rows.map((row) => {
            const dirty = isDirty(row);
            return (
              <div
                key={row.key}
                className={`glass-panel p-4 grid grid-cols-1 md:grid-cols-[1fr_14rem_14rem_5rem] gap-3 items-start ${
                  row.error ? "border-red-500/50" : dirty ? "border-amber-500/40" : ""
                }`}
              >
                <textarea
                  aria-label="Question"
                  placeholder="Question"
                  rows={3}
                  value={row.question}
                  onChange={(e) => patchRow(row.key, { question: e.target.value })}
                  className={inputClass}
                />
                <input
                  aria-label="Display answer"
                  placeholder="Display answer"
                  type="text"
                  value={row.displayAnswer}
                  onChange={(e) => patchRow(row.key, { displayAnswer: e.target.value })}
                  className={inputClass}
                />
                <textarea
                  aria-label="Acceptable answers"
                  placeholder={"one per line\nor, comma, separated"}
                  rows={3}
                  value={row.answersText}
                  onChange={(e) => patchRow(row.key, { answersText: e.target.value })}
                  className={`${inputClass} font-mono`}
                />
                <div className="flex flex-col items-stretch gap-2">
                  <button
                    onClick={() => removeRow(row)}
                    disabled={busy}
                    className="btn-secondary text-sm text-red-400 disabled:opacity-50"
                  >
                    Delete
                  </button>
                  <span className="text-xs text-center">
                    {row.id === null ? (
                      <span className="text-amber-400">new</span>
                    ) : dirty ? (
                      <span className="text-amber-400">edited</span>
                    ) : (
                      <span className="text-slate-600">#{String(row.id)}</span>
                    )}
                  </span>
                </div>
                {row.error && <div className="md:col-span-4 text-sm text-red-400">{row.error}</div>}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
