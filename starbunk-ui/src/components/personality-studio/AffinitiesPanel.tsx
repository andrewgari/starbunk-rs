"use client";

import { useState } from "react";
import { WeightedPreference } from "./types";

interface AffinitiesPanelProps {
  interests: string[];
  setInterests: (val: string[]) => void;
  preferences: WeightedPreference[];
  setPreferences: (val: WeightedPreference[]) => void;
}

export default function AffinitiesPanel({
  interests,
  setInterests,
  preferences,
  setPreferences,
}: AffinitiesPanelProps) {
  const [newInterest, setNewInterest] = useState("");
  const [newPrefItem, setNewPrefItem] = useState("");
  const [newPrefWeight, setNewPrefWeight] = useState(0);

  const handleAddInterest = () => {
    if (newInterest.trim()) {
      setInterests([...interests, newInterest.trim()]);
      setNewInterest("");
    }
  };

  const handleAddPreference = () => {
    if (newPrefItem.trim()) {
      setPreferences([...preferences, { item: newPrefItem.trim(), weight: newPrefWeight }]);
      setNewPrefItem("");
      setNewPrefWeight(0);
    }
  };

  return (
    <section className="glass-panel p-6 border-emerald-500/20">
      <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
        <span>❤️</span> Affinities &amp; Passions
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {/* Interests */}
        <div>
          <h3 className="text-sm font-semibold text-slate-300 mb-3 border-b border-slate-800 pb-2">Set of Interests</h3>
          <div className="flex flex-wrap gap-2 mb-4">
            {interests.map((interest, idx) => (
              <span key={idx} className="bg-emerald-950/30 border border-emerald-500/20 text-emerald-200 text-xs px-2.5 py-1.5 rounded-md flex items-center gap-2 group">
                {interest}
                <button
                  onClick={() => setInterests(interests.filter((_, i) => i !== idx))}
                  className="text-emerald-500/50 group-hover:text-red-400 transition-colors"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
          <div className="flex gap-2 text-xs">
            <input
              type="text"
              placeholder="Add an interest..."
              value={newInterest}
              onChange={(e) => setNewInterest(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddInterest()}
              className="bg-slate-950/50 border border-slate-800 rounded px-3 py-2 text-white flex-1 focus:outline-none focus:border-emerald-500/50"
            />
            <button onClick={handleAddInterest} className="bg-slate-800 hover:bg-slate-700 text-slate-200 rounded px-3 py-2 transition-colors">
              Add
            </button>
          </div>
        </div>

        {/* Weighted Likes/Dislikes */}
        <div>
          <h3 className="text-sm font-semibold text-slate-300 mb-3 border-b border-slate-800 pb-2">Likes &amp; Dislikes (Weighted)</h3>
          <div className="flex flex-col gap-2 mb-4 max-h-[150px] overflow-y-auto pr-2 custom-scrollbar">
            {preferences.map((item, idx) => (
              <div key={idx} className="flex items-center justify-between bg-slate-900/50 border border-slate-800/80 rounded px-3 py-2 group">
                <span className="text-xs text-slate-200">{item.item}</span>
                <div className="flex items-center gap-3">
                  <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded-full border ${item.weight >= 0 ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border-rose-500/20'}`}>
                    {item.weight > 0 ? `+${item.weight}` : item.weight}
                  </span>
                  <button
                    onClick={() => setPreferences(preferences.filter((_, i) => i !== idx))}
                    className="text-slate-500 group-hover:text-red-400 transition-colors"
                  >
                    ×
                  </button>
                </div>
              </div>
            ))}
          </div>
          
          <div className="flex gap-2 text-xs items-center bg-slate-950/50 p-2 rounded border border-slate-800">
            <input
              type="text"
              placeholder="Item (e.g. Pineapples on Pizza)"
              value={newPrefItem}
              onChange={(e) => setNewPrefItem(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddPreference()}
              className="bg-transparent text-white flex-1 focus:outline-none placeholder-slate-600 px-1"
            />
            <input
              type="number"
              min="-10"
              max="10"
              placeholder="0"
              value={newPrefWeight === 0 ? "" : newPrefWeight}
              onChange={(e) => setNewPrefWeight(Number(e.target.value))}
              onKeyDown={(e) => e.key === 'Enter' && handleAddPreference()}
              className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-white w-16 focus:outline-none focus:border-emerald-500/50"
            />
            <button onClick={handleAddPreference} className="text-emerald-400 hover:text-emerald-300 font-medium px-2 transition-colors">
              Add
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
