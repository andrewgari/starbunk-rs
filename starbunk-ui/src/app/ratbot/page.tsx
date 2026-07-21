"use client";

import { useState } from "react";

export default function RatBotLanding() {
  const [isAutomatedMode, setIsAutomatedMode] = useState(true);
  const [cronExpression, setCronExpression] = useState("0 0 1 12 *"); // Dec 1st annually
  const [participantsCount, setParticipantsCount] = useState(14);
  const [isPairingsGenerated, setIsPairingsGenerated] = useState(false);
  const [isDispatching, setIsDispatching] = useState(false);

  const handleGeneratePairings = () => {
    setIsPairingsGenerated(true);
  };

  const handleDispatchDms = () => {
    setIsDispatching(true);
    setTimeout(() => setIsDispatching(false), 1200);
  };

  return (
    <div className="flex flex-col h-full gap-6 max-w-5xl mx-auto py-6">
      <header className="mb-2">
        <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
          <span>RatBot 🎁</span>
          <span className="text-xs px-2.5 py-1 rounded bg-rose-500/20 text-rose-300 border border-rose-500/30 uppercase font-mono tracking-wide">
            Secret Santa
          </span>
        </h1>
        <p className="text-slate-400 mt-1">
          Secret Santa organizer for the guild&apos;s annual &quot;Ratmas&quot; gift exchange.
        </p>
      </header>

      {/* Mode Selector Card */}
      <div className="glass-panel p-6 border-rose-500/30">
        <div className="flex justify-between items-center mb-3">
          <div>
            <h2 className="text-xl font-bold text-white">Execution Mode</h2>
            <p className="text-xs text-slate-400 mt-0.5">
              Toggle between scheduled automated seasonal cron execution or ad-hoc manual Web UI control.
            </p>
          </div>

          <div className="flex items-center gap-2 bg-slate-900/80 p-1.5 rounded-lg border border-slate-800">
            <button
              onClick={() => setIsAutomatedMode(true)}
              className={`text-xs px-3 py-1.5 rounded transition-all font-medium ${
                isAutomatedMode ? "bg-rose-600 text-white font-bold shadow" : "text-slate-400 hover:text-white"
              }`}
            >
              Automated (Cron)
            </button>
            <button
              onClick={() => setIsAutomatedMode(false)}
              className={`text-xs px-3 py-1.5 rounded transition-all font-medium ${
                !isAutomatedMode ? "bg-amber-600 text-white font-bold shadow" : "text-slate-400 hover:text-white"
              }`}
            >
              Manual / Ad-Hoc
            </button>
          </div>
        </div>

        {isAutomatedMode ? (
          <div className="bg-slate-900/50 p-3.5 rounded-lg border border-slate-800/80 text-xs flex justify-between items-center">
            <div className="flex items-center gap-3">
              <span className="text-emerald-400 font-bold">● Scheduled Automated Mode Active</span>
              <span className="text-slate-400 font-mono">Cron: {cronExpression}</span>
            </div>
            <input
              type="text"
              value={cronExpression}
              onChange={(e) => setCronExpression(e.target.value)}
              className="bg-slate-950 border border-slate-700 text-white font-mono px-2 py-1 rounded text-xs"
            />
          </div>
        ) : (
          <div className="bg-amber-500/10 border border-amber-500/20 p-3.5 rounded-lg text-xs text-amber-300">
            ⚠ <strong>Manual Mode Active:</strong> Automatic pairing cron is paused. Execution must be triggered via the buttons below.
          </div>
        )}
      </div>

      {/* Secret Santa Status Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Registration Status</div>
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-emerald-500 animate-pulse"></span>
            <span className="text-xl font-bold text-white">Sign-ups Open</span>
          </div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Registered Rats</div>
          <div className="text-3xl font-bold text-white">{participantsCount}</div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Pairings Status</div>
          <div className={`text-xl font-bold ${isPairingsGenerated ? "text-emerald-400" : "text-amber-400"}`}>
            {isPairingsGenerated ? "Pairings Ready ✓" : "Pending Match"}
          </div>
        </div>
      </div>

      {/* Secret Santa Controls & Details */}
      <div className="glass-panel p-6">
        <h2 className="text-xl font-semibold mb-4 text-white">Ad-hoc Secret Santa Controls</h2>
        <div className="flex flex-wrap gap-4 mb-4">
          <button
            onClick={handleGeneratePairings}
            className="btn-primary bg-rose-600 hover:bg-rose-500 border-none text-white shadow-lg shadow-rose-600/20"
          >
            {isPairingsGenerated ? "Re-Generate Pairings Match" : "Trigger Pairings Match"}
          </button>
          <button
            onClick={handleDispatchDms}
            disabled={!isPairingsGenerated || isDispatching}
            className="btn-secondary text-slate-300 hover:bg-slate-700 disabled:opacity-40"
          >
            {isDispatching ? "Dispatching DMs..." : "Dispatch DM Notifications"}
          </button>
          <button
            onClick={() => {
              setIsPairingsGenerated(false);
              setParticipantsCount(0);
            }}
            className="btn-secondary text-slate-300 hover:bg-red-500/20 hover:text-red-400"
          >
            Reset Registrations
          </button>
        </div>
        <p className="text-xs text-slate-400">
          Pairing generation assigns secret gift targets using a random cycle guard to prevent self-matching.
        </p>
      </div>
    </div>
  );
}
