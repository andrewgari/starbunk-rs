"use client";

import { useState } from "react";

interface AdvancedSettingsPanelProps {
  highTierProvider: string;
  setHighTierProvider: (val: string) => void;
  highTierModel: string;
  setHighTierModel: (val: string) => void;
  medTierProvider: string;
  setMedTierProvider: (val: string) => void;
  medTierModel: string;
  setMedTierModel: (val: string) => void;
  lowTierProvider: string;
  setLowTierProvider: (val: string) => void;
  lowTierModel: string;
  setLowTierModel: (val: string) => void;
  batteryMax: number;
  setBatteryMax: (val: number) => void;
  depletionRate: number;
  setDepletionRate: (val: number) => void;
  rechargeRate: number;
  setRechargeRate: (val: number) => void;
}

export default function AdvancedSettingsPanel({
  highTierProvider,
  setHighTierProvider,
  highTierModel,
  setHighTierModel,
  medTierProvider,
  setMedTierProvider,
  medTierModel,
  setMedTierModel,
  lowTierProvider,
  setLowTierProvider,
  lowTierModel,
  setLowTierModel,
  batteryMax,
  setBatteryMax,
  depletionRate,
  setDepletionRate,
  rechargeRate,
  setRechargeRate,
}: AdvancedSettingsPanelProps) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <section className="glass-panel overflow-hidden border border-slate-800/50">
      <button 
        onClick={() => setShowAdvanced(!showAdvanced)}
        className="w-full flex items-center justify-between p-4 bg-slate-900/40 hover:bg-slate-800/60 transition-colors focus:outline-none focus:bg-slate-800/60"
      >
        <div className="flex items-center gap-3">
          <span className="text-slate-400 text-lg">⚙️</span>
          <span className="font-semibold text-slate-300 text-sm tracking-wide uppercase">Advanced System &amp; Routing Controls</span>
        </div>
        <div className={`text-slate-400 transform transition-transform duration-300 flex items-center justify-center w-6 h-6 rounded-full bg-slate-800 ${showAdvanced ? 'rotate-180' : ''}`}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </div>
      </button>

      <div className={`transition-all duration-500 ease-in-out ${showAdvanced ? 'max-h-[2000px] opacity-100' : 'max-h-0 opacity-0'} overflow-hidden bg-slate-950/30`}>
        <div className="p-6 border-t border-slate-800/50 flex flex-col gap-8">
          
          {/* LLM Routing */}
          <div>
            <h3 className="text-sm font-semibold text-slate-300 mb-4 flex items-center gap-2">
              <span className="text-indigo-400">🧠</span> LLM Model Tier Routing Matrix
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              {/* High Tier */}
              <div className="bg-slate-900/60 p-4 rounded-lg border border-indigo-500/30">
                <div className="text-xs font-semibold text-indigo-400 uppercase tracking-wider mb-2">High Tier (Generation)</div>
                <div className="flex flex-col gap-2 text-xs">
                  <div>
                    <label className="text-slate-400">Provider</label>
                    <select
                      value={highTierProvider}
                      onChange={(e) => setHighTierProvider(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1 focus:outline-none focus:border-indigo-500/50"
                    >
                      <option value="anthropic">Anthropic</option>
                      <option value="google">Google Gemini</option>
                      <option value="openai">OpenAI</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-slate-400">Model Name</label>
                    <input
                      type="text"
                      value={highTierModel}
                      onChange={(e) => setHighTierModel(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1 focus:outline-none focus:border-indigo-500/50"
                    />
                  </div>
                </div>
              </div>

              {/* Med Tier */}
              <div className="bg-slate-900/60 p-4 rounded-lg border border-cyan-500/30">
                <div className="text-xs font-semibold text-cyan-400 uppercase tracking-wider mb-2">Med Tier (Stance / Summary)</div>
                <div className="flex flex-col gap-2 text-xs">
                  <div>
                    <label className="text-slate-400">Provider</label>
                    <select
                      value={medTierProvider}
                      onChange={(e) => setMedTierProvider(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1 focus:outline-none focus:border-cyan-500/50"
                    >
                      <option value="google">Google Gemini</option>
                      <option value="openai">OpenAI</option>
                      <option value="ollama">Ollama (Local)</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-slate-400">Model Name</label>
                    <input
                      type="text"
                      value={medTierModel}
                      onChange={(e) => setMedTierModel(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1 focus:outline-none focus:border-cyan-500/50"
                    />
                  </div>
                </div>
              </div>

              {/* Low Tier */}
              <div className="bg-slate-900/60 p-4 rounded-lg border border-purple-500/30">
                <div className="text-xs font-semibold text-purple-400 uppercase tracking-wider mb-2">Low Tier (Relevance)</div>
                <div className="flex flex-col gap-2 text-xs">
                  <div>
                    <label className="text-slate-400">Provider</label>
                    <select
                      value={lowTierProvider}
                      onChange={(e) => setLowTierProvider(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1 focus:outline-none focus:border-purple-500/50"
                    >
                      <option value="openai">OpenAI</option>
                      <option value="ollama">Ollama (Local)</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-slate-400">Model Name</label>
                    <input
                      type="text"
                      value={lowTierModel}
                      onChange={(e) => setLowTierModel(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1 focus:outline-none focus:border-purple-500/50"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Social Battery Controls */}
          <div>
            <h3 className="text-sm font-semibold text-slate-300 mb-4 flex items-center gap-2">
              <span className="text-emerald-400">⚡</span> Social Battery &amp; Restraint
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="bg-slate-900/40 p-4 rounded-lg border border-slate-800 flex flex-col gap-2">
                <div className="flex justify-between text-xs">
                  <span className="text-slate-400 font-medium">Max Capacity</span>
                  <span className="text-emerald-400 font-bold">{batteryMax} pts</span>
                </div>
                <input
                  type="range"
                  min="20"
                  max="200"
                  value={batteryMax}
                  onChange={(e) => setBatteryMax(Number(e.target.value))}
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-500"
                />
              </div>

              <div className="bg-slate-900/40 p-4 rounded-lg border border-slate-800 flex flex-col gap-2">
                <div className="flex justify-between text-xs">
                  <span className="text-slate-400 font-medium">Depletion / Msg</span>
                  <span className="text-amber-400 font-bold">-{depletionRate} pts</span>
                </div>
                <input
                  type="range"
                  min="1"
                  max="30"
                  value={depletionRate}
                  onChange={(e) => setDepletionRate(Number(e.target.value))}
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-amber-500"
                />
              </div>

              <div className="bg-slate-900/40 p-4 rounded-lg border border-slate-800 flex flex-col gap-2">
                <div className="flex justify-between text-xs">
                  <span className="text-slate-400 font-medium">Recharge / Min</span>
                  <span className="text-cyan-400 font-bold">+{rechargeRate} pts</span>
                </div>
                <input
                  type="range"
                  min="1"
                  max="20"
                  value={rechargeRate}
                  onChange={(e) => setRechargeRate(Number(e.target.value))}
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                />
              </div>
            </div>
          </div>
          
        </div>
      </div>
    </section>
  );
}
