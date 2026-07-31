"use client";

import { useState, useEffect } from "react";
import { getPersonality, patchPersonality } from "../app/actions";

export interface RelationshipEntry {
  userId: string;
  stance: string;
}

export interface TopicAffinity {
  topic: string;
  passionScore: number; // -10 to +10
}

export default function PersonalityStudio() {
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Model Tier States
  const [highTierProvider, setHighTierProvider] = useState("anthropic");
  const [highTierModel, setHighTierModel] = useState("claude-3-5-sonnet-latest");
  const [medTierProvider, setMedTierProvider] = useState("google");
  const [medTierModel, setMedTierModel] = useState("gemini-1.5-flash");
  const [lowTierProvider, setLowTierProvider] = useState("openai");
  const [lowTierModel, setLowTierModel] = useState("text-embedding-3-small");

  // Core Identity & Soul
  const [systemPrompt, setSystemPrompt] = useState("");
  const [speechPatterns, setSpeechPatterns] = useState<string[]>([]);
  const [topics, setTopics] = useState<TopicAffinity[]>([]);
  const [relationships, setRelationships] = useState<RelationshipEntry[]>([]);

  // Social Battery Sliders
  const [batteryMax, setBatteryMax] = useState(100);
  const [depletionRate, setDepletionRate] = useState(12);
  const [rechargeRate, setRechargeRate] = useState(5);

  const [newTopic, setNewTopic] = useState("");
  const [newSpeechPattern, setNewSpeechPattern] = useState("");
  const [newRelUser, setNewRelUser] = useState("");
  const [newRelStance, setNewRelStance] = useState("");

  useEffect(() => {
    async function load() {
      setIsLoading(true);
      const data = await getPersonality();
      if (data) {
        if (data.identity) setSystemPrompt(data.identity);
        if (data.speech_patterns) setSpeechPatterns(data.speech_patterns);
        if (data.affinities) {
          setTopics(data.affinities.map((a: string) => ({ topic: a, passionScore: 5 })));
        }
        if (data.relationships) {
          setRelationships(
            Object.entries(data.relationships).map(([userId, stance]) => ({
              userId,
              stance: stance as string
            }))
          );
        }
      }
      setIsLoading(false);
    }
    load();
  }, []);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      const affinities = topics.map(t => t.topic);
      const rels = relationships.reduce((acc, rel) => {
        acc[rel.userId] = rel.stance;
        return acc;
      }, {} as Record<string, string>);
      
      await patchPersonality({
        identity: systemPrompt,
        speech_patterns: speechPatterns,
        affinities,
        relationships: rels
      });
      
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    } catch (e) {
      console.error(e);
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddTopic = () => {
    if (newTopic.trim()) {
      setTopics([...topics, { topic: newTopic.trim(), passionScore: 5 }]);
      setNewTopic("");
    }
  };

  const handleAddSpeechPattern = () => {
    if (newSpeechPattern.trim()) {
      setSpeechPatterns([...speechPatterns, newSpeechPattern.trim()]);
      setNewSpeechPattern("");
    }
  };

  const handleAddRelationship = () => {
    if (newRelUser.trim() && newRelStance.trim()) {
      setRelationships([
        ...relationships,
        { userId: newRelUser.trim(), stance: newRelStance.trim() },
      ]);
      setNewRelUser("");
      setNewRelStance("");
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="w-12 h-12 border-4 border-indigo-500/30 border-t-indigo-500 rounded-full animate-spin"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 pb-24">
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
            <div className="flex gap-2 text-xs">
              <input
                type="text"
                placeholder="Add new speech pattern..."
                value={newSpeechPattern}
                onChange={(e) => setNewSpeechPattern(e.target.value)}
                className="bg-slate-950 border border-slate-800 rounded px-2.5 py-1.5 text-white flex-1 focus:outline-none focus:border-indigo-500/50"
              />
              <button onClick={handleAddSpeechPattern} className="btn-secondary text-xs px-3 py-1.5">
                + Add Pattern
              </button>
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
            <div key={rel.userId} className="p-3 bg-slate-900/60 rounded-lg border border-slate-800 flex justify-between items-start transition-all hover:border-indigo-500/30">
              <div>
                <div className="text-sm font-bold text-white flex items-center gap-2">
                  <span className="text-xs text-slate-400 font-mono font-normal">ID: {rel.userId}</span>
                </div>
                <div className="text-xs text-indigo-300 mt-1 font-medium">&quot;{rel.stance}&quot;</div>
              </div>
              <button
                onClick={() => setRelationships(relationships.filter((r) => r.userId !== rel.userId))}
                className="text-xs text-slate-500 hover:text-red-400 bg-slate-800/50 hover:bg-slate-800 rounded-full w-6 h-6 flex items-center justify-center transition-colors"
              >
                ✕
              </button>
            </div>
          ))}
        </div>

        <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800 flex flex-wrap gap-2 items-center text-xs">
          <input
            type="text"
            placeholder="User ID (Snowflake)"
            value={newRelUser}
            onChange={(e) => setNewRelUser(e.target.value)}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1 focus:outline-none focus:border-indigo-500/50"
          />
          <input
            type="text"
            placeholder="Stance / Opinion"
            value={newRelStance}
            onChange={(e) => setNewRelStance(e.target.value)}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-2 focus:outline-none focus:border-indigo-500/50"
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

      {/* Floating Action Bar */}
      <div className="fixed bottom-0 left-0 right-0 bg-slate-950/80 backdrop-blur-md border-t border-slate-800 p-4 z-50 flex justify-end items-center px-8 shadow-[0_-10px_40px_rgba(0,0,0,0.5)]">
        <div className="flex items-center gap-4 max-w-7xl w-full mx-auto justify-end">
          {saveSuccess && (
            <span className="text-emerald-400 text-sm font-medium animate-pulse flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path></svg>
              Saved Successfully
            </span>
          )}
          <button 
            onClick={handleSave} 
            disabled={isSaving}
            className={`
              relative overflow-hidden px-8 py-2.5 rounded-lg font-bold text-sm tracking-wide transition-all
              ${isSaving ? 'bg-slate-700 text-slate-400 cursor-not-allowed' : 'bg-gradient-to-r from-indigo-500 to-purple-500 text-white hover:shadow-[0_0_20px_rgba(99,102,241,0.5)] hover:-translate-y-0.5'}
            `}
          >
            {isSaving ? (
              <span className="flex items-center gap-2">
                <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></div>
                Saving...
              </span>
            ) : (
              <span className="flex items-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path></svg>
                Sync to DB
              </span>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
