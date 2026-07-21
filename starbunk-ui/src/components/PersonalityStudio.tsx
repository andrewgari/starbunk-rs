"use client";

import { useState } from "react";

export interface RelationshipEntry {
  userId: string;
  alias: string;
  stance: string;
}

export interface TopicAffinity {
  topic: string;
  passionScore: number; // -10 to +10
}

export default function PersonalityStudio() {
  // Model Tier States
  const [highTierProvider, setHighTierProvider] = useState("anthropic");
  const [highTierModel, setHighTierModel] = useState("claude-3-5-sonnet-latest");
  const [medTierProvider, setMedTierProvider] = useState("google");
  const [medTierModel, setMedTierModel] = useState("gemini-1.5-flash");
  const [lowTierProvider, setLowTierProvider] = useState("openai");
  const [lowTierModel, setLowTierModel] = useState("text-embedding-3-small");

  // Core Identity & Soul
  const [systemPrompt, setSystemPrompt] = useState(
    "You are Cova, a sharp, cynical Discord user with strong opinions on games and tech. Respond like a real person, not an assistant."
  );
  const [speechPatterns, setSpeechPatterns] = useState(["Casual tone", "Lowercase preference", "No exclamation overload"]);

  // Topic Affinities
  const [topics, setTopics] = useState<TopicAffinity[]>([
    { topic: "Final Fantasy XIV", passionScore: 9 },
    { topic: "Rust Programming", passionScore: 8 },
    { topic: "Unprocessed Fast Food", passionScore: -6 },
  ]);

  // Relationships
  const [relationships, setRelationships] = useState<RelationshipEntry[]>([
    { userId: "102938475", alias: "Andrew", stance: "Close Friend & Architect" },
    { userId: "987654321", alias: "Ratbot", stance: "Suspicious Seasonal rival" },
  ]);

  // Social Battery Sliders
  const [batteryMax, setBatteryMax] = useState(100);
  const [depletionRate, setDepletionRate] = useState(12);
  const [rechargeRate, setRechargeRate] = useState(5);

  const [newTopic, setNewTopic] = useState("");
  const [newRelUser, setNewRelUser] = useState("");
  const [newRelAlias, setNewRelAlias] = useState("");
  const [newRelStance, setNewRelStance] = useState("");

  const handleAddTopic = () => {
    if (newTopic.trim()) {
      setTopics([...topics, { topic: newTopic.trim(), passionScore: 5 }]);
      setNewTopic("");
    }
  };

  const handleAddRelationship = () => {
    if (newRelUser.trim() && newRelStance.trim()) {
      setRelationships([
        ...relationships,
        { userId: newRelUser.trim(), alias: newRelAlias.trim() || "User", stance: newRelStance.trim() },
      ]);
      setNewRelUser("");
      setNewRelAlias("");
      setNewRelStance("");
    }
  };

  return (
    <div className="flex flex-col gap-6">
      {/* 1. Model Tier Matrix */}
      <section className="glass-panel p-6">
        <h2 className="text-xl font-semibold text-white mb-1 flex items-center gap-2">
          <span>🧠</span> LLM Model Tier Routing Matrix
        </h2>
        <p className="text-xs text-slate-400 mb-6">
          Code requests capability tiers rather than fixed models. Select backend provider matrix below.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* High Tier */}
          <div className="bg-slate-900/60 p-4 rounded-lg border border-indigo-500/30">
            <div className="text-xs font-semibold text-indigo-400 uppercase tracking-wider mb-2">High Tier (Response Generation)</div>
            <div className="flex flex-col gap-2 text-xs">
              <div>
                <label className="text-slate-400">Provider</label>
                <select
                  value={highTierProvider}
                  onChange={(e) => setHighTierProvider(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1"
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
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1"
                />
              </div>
            </div>
          </div>

          {/* Med Tier */}
          <div className="bg-slate-900/60 p-4 rounded-lg border border-cyan-500/30">
            <div className="text-xs font-semibold text-cyan-400 uppercase tracking-wider mb-2">Med Tier (Stance Evolution &amp; Summary)</div>
            <div className="flex flex-col gap-2 text-xs">
              <div>
                <label className="text-slate-400">Provider</label>
                <select
                  value={medTierProvider}
                  onChange={(e) => setMedTierProvider(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1"
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
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1"
                />
              </div>
            </div>
          </div>

          {/* Low Tier */}
          <div className="bg-slate-900/60 p-4 rounded-lg border border-purple-500/30">
            <div className="text-xs font-semibold text-purple-400 uppercase tracking-wider mb-2">Low Tier (Relevance Gate &amp; Vectors)</div>
            <div className="flex flex-col gap-2 text-xs">
              <div>
                <label className="text-slate-400">Provider</label>
                <select
                  value={lowTierProvider}
                  onChange={(e) => setLowTierProvider(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white mt-1"
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
                  className="w-full bg-slate-950 border border-slate-800 rounded p-1.5 text-white font-mono mt-1"
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* 2. Core Heart & Soul (System Prompt & Persona Essence) */}
      <section className="glass-panel p-6">
        <h2 className="text-xl font-semibold text-white mb-1 flex items-center gap-2">
          <span>💖</span> Core Essence &amp; System Soul
        </h2>
        <p className="text-xs text-slate-400 mb-4">
          Separating Cova&apos;s prompt identity, voice, and opinions from Rust orchestration.
        </p>

        <div className="flex flex-col gap-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">System Prompt / Identity</label>
            <textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              rows={4}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-xs font-mono text-slate-200 focus:outline-none focus:border-accent"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">Speech Patterns &amp; Cadence Quirks</label>
            <div className="flex flex-wrap gap-2 mb-4">
              {speechPatterns.map((pattern, idx) => (
                <span key={idx} className="bg-slate-800 text-slate-200 text-xs px-2.5 py-1 rounded-full flex items-center gap-1.5">
                  {pattern}
                  <button
                    onClick={() => setSpeechPatterns(speechPatterns.filter((_, i) => i !== idx))}
                    className="text-slate-400 hover:text-red-400 text-xs"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">Topic Affinities &amp; Passion Weight (-10 to +10)</label>
            <div className="flex flex-wrap gap-2 mb-3">
              {topics.map((item, idx) => (
                <span key={idx} className="bg-indigo-950/60 border border-indigo-500/30 text-indigo-200 text-xs px-2.5 py-1 rounded-lg flex items-center gap-2">
                  <span className="font-semibold">{item.topic}</span>
                  <span className={`text-[10px] font-mono px-1 rounded ${item.passionScore >= 0 ? 'bg-emerald-500/20 text-emerald-300' : 'bg-rose-500/20 text-rose-300'}`}>
                    {item.passionScore > 0 ? `+${item.passionScore}` : item.passionScore}
                  </span>
                  <button
                    onClick={() => setTopics(topics.filter((_, i) => i !== idx))}
                    className="text-slate-400 hover:text-red-400"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>

            <div className="flex gap-2 text-xs">
              <input
                type="text"
                placeholder="Add new topic..."
                value={newTopic}
                onChange={(e) => setNewTopic(e.target.value)}
                className="bg-slate-950 border border-slate-800 rounded px-2.5 py-1.5 text-white flex-1"
              />
              <button onClick={handleAddTopic} className="btn-secondary text-xs px-3 py-1.5">
                + Add Topic
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* 3. User Relationship Matrix */}
      <section className="glass-panel p-6">
        <h2 className="text-xl font-semibold text-white mb-1 flex items-center gap-2">
          <span>👥</span> User Relationship Stance Matrix
        </h2>
        <p className="text-xs text-slate-400 mb-4">
          Define Cova&apos;s evolving opinions and relationship stances toward specific Discord users.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          {relationships.map((rel) => (
            <div key={rel.userId} className="p-3 bg-slate-900/60 rounded-lg border border-slate-800 flex justify-between items-start">
              <div>
                <div className="text-sm font-bold text-white flex items-center gap-2">
                  <span>{rel.alias}</span>
                  <span className="text-xs text-slate-500 font-mono font-normal">({rel.userId})</span>
                </div>
                <div className="text-xs text-indigo-300 mt-1 font-medium">&quot;{rel.stance}&quot;</div>
              </div>
              <button
                onClick={() => setRelationships(relationships.filter((r) => r.userId !== rel.userId))}
                className="text-xs text-slate-500 hover:text-red-400"
              >
                ✕
              </button>
            </div>
          ))}
        </div>

        {/* Add Relationship Form */}
        <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800 flex flex-wrap gap-2 items-center text-xs">
          <input
            type="text"
            placeholder="User ID (Snowflake)"
            value={newRelUser}
            onChange={(e) => setNewRelUser(e.target.value)}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1"
          />
          <input
            type="text"
            placeholder="User Alias"
            value={newRelAlias}
            onChange={(e) => setNewRelAlias(e.target.value)}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1"
          />
          <input
            type="text"
            placeholder="Stance / Opinion"
            value={newRelStance}
            onChange={(e) => setNewRelStance(e.target.value)}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-2"
          />
          <button onClick={handleAddRelationship} className="btn-primary px-3 py-1.5 text-xs">
            + Add Stance
          </button>
        </div>
      </section>

      {/* 4. Social Battery & Energy Sliders */}
      <section className="glass-panel p-6">
        <h2 className="text-xl font-semibold text-white mb-1 flex items-center gap-2">
          <span>⚡</span> Social Battery &amp; Restraint Controls
        </h2>
        <p className="text-xs text-slate-400 mb-4">
          Restraint modulates low-pull chatter without vetoing high-pull direct mentions.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-slate-900/40 p-4 rounded-lg border border-slate-800 flex flex-col gap-2">
            <div className="flex justify-between text-xs">
              <span className="text-slate-400 font-medium">Max Battery Capacity</span>
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
              <span className="text-slate-400 font-medium">Depletion Rate per Message</span>
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
              <span className="text-slate-400 font-medium">Recharge Rate (per min)</span>
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
      </section>
    </div>
  );
}
