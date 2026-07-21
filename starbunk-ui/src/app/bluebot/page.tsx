"use client";

import { useSSE } from "@/components/useSSE";

export default function BlueBotLanding() {
  const { isConnected, bluebotAudits } = useSSE();

  return (
    <div className="flex flex-col h-full gap-6 max-w-5xl mx-auto py-6">
      <header className="mb-2">
        <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
          <span>BlueBot 💙</span>
          <span className="text-xs px-2.5 py-1 rounded bg-blue-500/20 text-blue-300 border border-blue-500/30 uppercase font-mono tracking-wide">
            Pattern Matcher
          </span>
        </h1>
        <p className="text-slate-400 mt-1">
          Pattern-matches &quot;blue&quot; and Blue Mage references with catchphrases.
        </p>
      </header>

      {/* Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Status</div>
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-blue-500 animate-pulse shadow-[0_0_8px_rgba(59,130,246,0.6)]"></span>
            <span className="text-xl font-bold text-white">Active</span>
          </div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Catchphrase Response</div>
          <div className="text-xl font-bold text-blue-300">&quot;Did somebody say Blu?&quot;</div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Cooldown Mode</div>
          <div className="text-xl font-bold text-white">5-Min Channel Gate</div>
        </div>
      </div>

      {/* Pattern Coverage Inspector */}
      <div className="glass-panel p-6">
        <h2 className="text-xl font-semibold mb-3 text-white">Pattern Coverage Inspector</h2>
        <p className="text-sm text-slate-400 mb-4">
          BlueBot matches single-word variants across languages (French: <em>bleu</em>, Spanish: <em>azul</em>, German: <em>blau</em>) while guarding against compound words (<em>bluetooth</em>, <em>blueprint</em>).
        </p>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
          <div className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50 text-slate-300">
            <span className="font-semibold text-blue-400">blue / bloo / bluu</span>
            <div className="text-xs text-slate-500 mt-1">Standard trigger</div>
          </div>
          <div className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50 text-slate-300">
            <span className="font-semibold text-blue-400">bleu / azul / blau</span>
            <div className="text-xs text-slate-500 mt-1">Multilingual match</div>
          </div>
          <div className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50 text-slate-300">
            <span className="font-semibold text-blue-400">blew</span>
            <div className="text-xs text-slate-500 mt-1">Archaic spelling</div>
          </div>
          <div className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50 text-slate-300">
            <span className="font-semibold text-blue-400">bluebot</span>
            <div className="text-xs text-slate-500 mt-1">Bot name match</div>
          </div>
        </div>
      </div>

      {/* Live Response Audit Stream */}
      <section className="glass-panel p-6">
        <div className="flex justify-between items-center mb-4">
          <div>
            <h2 className="text-xl font-semibold text-white flex items-center gap-2">
              <span>💙</span> Trigger Event Stream &amp; Response Audit
            </h2>
            <p className="text-xs text-slate-400 mt-0.5">Live real-time feed of detected blue references and bot catchphrase replies.</p>
          </div>
          <div className="flex items-center gap-2">
            <span className={`w-2.5 h-2.5 rounded-full ${isConnected ? "bg-blue-500 animate-pulse" : "bg-slate-500"}`} />
            <span className="text-xs text-slate-400 font-mono">{isConnected ? "SSE Stream Connected" : "Connecting..."}</span>
          </div>
        </div>

        <div className="bg-slate-950/80 rounded-lg p-4 border border-slate-800 font-mono text-xs max-h-64 overflow-y-auto flex flex-col gap-2">
          {bluebotAudits.length === 0 ? (
            <div className="text-slate-500 text-center py-6">Listening for live BlueBot triggers...</div>
          ) : (
            bluebotAudits.map((evt) => (
              <div key={evt.id} className="flex items-center justify-between p-2.5 rounded bg-slate-900/70 border border-blue-500/20">
                <div className="flex items-center gap-3">
                  <span className="text-blue-400 font-bold">[{evt.channel || "#general"}]</span>
                  <span className="text-slate-300">{evt.user}:</span>
                  <span className="text-slate-400">Matched variant <code className="text-blue-300 font-bold">&quot;{evt.matchedVariant}&quot;</code></span>
                  <span className="text-blue-300 font-semibold">&rarr; {evt.response}</span>
                </div>
                <span className="text-slate-500 text-[10px]">{evt.timestamp}</span>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
