"use client";

import { useState } from "react";

interface CoreIdentityPanelProps {
  systemPrompt: string;
  setSystemPrompt: (val: string) => void;
  conversationalStyle: string;
  setConversationalStyle: (val: string) => void;
  speechPatterns: string[];
  setSpeechPatterns: (val: string[]) => void;
}

export default function CoreIdentityPanel({
  systemPrompt,
  setSystemPrompt,
  conversationalStyle,
  setConversationalStyle,
  speechPatterns,
  setSpeechPatterns,
}: CoreIdentityPanelProps) {
  const [newSpeechPattern, setNewSpeechPattern] = useState("");

  const handleAddSpeechPattern = () => {
    if (newSpeechPattern.trim()) {
      setSpeechPatterns([...speechPatterns, newSpeechPattern.trim()]);
      setNewSpeechPattern("");
    }
  };

  return (
    <section className="glass-panel p-6 border-indigo-500/20">
      <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
        <span>🎭</span> Core Identity &amp; Voice
      </h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="flex flex-col gap-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">System Prompt / Identity</label>
            <textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              rows={4}
              className="w-full bg-slate-950/50 border border-slate-800 rounded-lg p-3 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500/50 transition-colors custom-scrollbar"
              placeholder="Core definition of the bot..."
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">Conversational Style &amp; Behaviors</label>
            <textarea
              value={conversationalStyle}
              onChange={(e) => setConversationalStyle(e.target.value)}
              rows={3}
              className="w-full bg-slate-950/50 border border-slate-800 rounded-lg p-3 text-xs text-slate-200 focus:outline-none focus:border-indigo-500/50 transition-colors custom-scrollbar"
              placeholder="How they act in a conversation..."
            />
          </div>
        </div>

        <div>
          <label className="block text-xs font-medium text-slate-300 mb-2">Manner of Speech</label>
          <div className="flex flex-wrap gap-2 mb-3">
            {speechPatterns.map((pattern, idx) => (
              <span key={idx} className="bg-indigo-950/40 border border-indigo-500/30 text-indigo-200 text-xs px-2.5 py-1 rounded-full flex items-center gap-1.5 group">
                {pattern}
                <button
                  onClick={() => setSpeechPatterns(speechPatterns.filter((_, i) => i !== idx))}
                  className="text-indigo-400/50 group-hover:text-red-400 transition-colors text-xs"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
          <div className="flex gap-2 text-xs">
            <input
              type="text"
              placeholder="Add a speech pattern..."
              value={newSpeechPattern}
              onChange={(e) => setNewSpeechPattern(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddSpeechPattern()}
              className="bg-slate-950/50 border border-slate-800 rounded px-3 py-2 text-white flex-1 focus:outline-none focus:border-indigo-500/50"
            />
            <button onClick={handleAddSpeechPattern} className="btn-secondary text-xs px-3 py-2">
              Add
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
