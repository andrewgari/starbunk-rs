"use client";

import { useState, useEffect } from "react";
import { getHistory, AuditRecord } from "./actions";

export default function HistoryPage() {
  const [history, setHistory] = useState<AuditRecord[]>([]);
  const [filter, setFilter] = useState("All");
  const [isLoading, setIsLoading] = useState(true);

  const load = async () => {
    setIsLoading(true);
    const data = await getHistory(filter);
    setHistory(data);
    setIsLoading(false);
  };

  useEffect(() => {
    let active = true;
    (async () => { if (active) await load(); })();
    return () => { active = false; };
  }, [filter]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="flex justify-between items-end mb-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Audit History</h1>
          <p className="text-slate-400 mt-1">Review bot responses, conditions, and correctness.</p>
        </div>
        <div className="flex items-center gap-3">
          <label className="text-sm text-slate-400">Filter:</label>
          <select 
            className="bg-slate-900 border border-white/10 rounded-lg px-3 py-2 text-white outline-none"
            value={filter}
            onChange={e => setFilter(e.target.value)}
          >
            <option>All</option>
            <option>BunkBot</option>
            <option>CovaBot</option>
            <option>DJCova</option>
            <option>BlueBot</option>
          </select>
          <button className="btn-secondary" onClick={load}>Refresh</button>
        </div>
      </header>

      <div className="glass-panel flex-1 overflow-auto">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="border-b border-white/5">
              <th className="p-4 text-slate-400 font-medium">Time</th>
              <th className="p-4 text-slate-400 font-medium">Bot</th>
              <th className="p-4 text-slate-400 font-medium">Condition / Trigger</th>
              <th className="p-4 text-slate-400 font-medium">Output</th>
              <th className="p-4 text-slate-400 font-medium w-24">Expected</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr>
                <td colSpan={5} className="p-8 text-center text-slate-400">
                  <div className="flex items-center justify-center gap-3">
                    <div className="h-6 w-6 rounded-full border-2 border-accent border-t-transparent animate-spin"></div>
                    Loading records...
                  </div>
                </td>
              </tr>
            ) : history.length === 0 ? (
              <tr>
                <td colSpan={5} className="p-8 text-center text-slate-500 italic">
                  {process.env.DATABASE_URL
                    ? "No records found."
                    : "No database configured — set DATABASE_URL to enable audit history."}
                </td>
              </tr>
            ) : (
              history.map(record => (
                <tr key={record.id} className="border-b border-white/5 hover:bg-white/5 transition-colors">
                  <td className="p-4 text-sm text-slate-300 align-top whitespace-nowrap">
                    {new Date(record.created_at).toLocaleString()}
                  </td>
                  <td className="p-4 align-top">
                    <span className="bg-accent/20 text-accent px-2 py-1 rounded-md text-xs font-bold">
                      {record.bot_name}
                    </span>
                  </td>
                  <td className="p-4 text-sm text-slate-300 align-top font-mono">
                    {record.trigger_condition}
                  </td>
                  <td className="p-4 text-sm text-slate-300 align-top">
                    {record.output_message}
                  </td>
                  <td className="p-4 align-top">
                    {record.expected === true ? (
                      <span className="text-green-400 bg-green-400/10 px-2 py-1 rounded-md text-xs">Yes</span>
                    ) : record.expected === false ? (
                      <span className="text-red-400 bg-red-400/10 px-2 py-1 rounded-md text-xs">No</span>
                    ) : (
                      <span className="text-slate-500 bg-slate-500/10 px-2 py-1 rounded-md text-xs">N/A</span>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
