"use client";

import { useState } from "react";
import { UserRelationship, NonUserRelationship } from "./types";

interface RelationshipsPanelProps {
  userRelationships: UserRelationship[];
  setUserRelationships: (val: UserRelationship[]) => void;
  nonUserRelationships: NonUserRelationship[];
  setNonUserRelationships: (val: NonUserRelationship[]) => void;
}

export default function RelationshipsPanel({
  userRelationships,
  setUserRelationships,
  nonUserRelationships,
  setNonUserRelationships,
}: RelationshipsPanelProps) {
  const [newRelUser, setNewRelUser] = useState("");
  const [newRelAlias, setNewRelAlias] = useState("");
  const [newRelStance, setNewRelStance] = useState("");

  const [newNonUser, setNewNonUser] = useState("");
  const [newNonUserStance, setNewNonUserStance] = useState("");

  const handleAddUserRelationship = () => {
    if (newRelUser.trim() && newRelStance.trim()) {
      setUserRelationships([
        ...userRelationships,
        { userId: newRelUser.trim(), alias: newRelAlias.trim() || "User", stance: newRelStance.trim() },
      ]);
      setNewRelUser("");
      setNewRelAlias("");
      setNewRelStance("");
    }
  };

  const handleAddNonUserRelationship = () => {
    if (newNonUser.trim() && newNonUserStance.trim()) {
      setNonUserRelationships([
        ...nonUserRelationships,
        { entity: newNonUser.trim(), stance: newNonUserStance.trim() },
      ]);
      setNewNonUser("");
      setNewNonUserStance("");
    }
  };

  return (
    <section className="glass-panel p-6 border-cyan-500/20">
      <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
        <span>🤝</span> Relationships &amp; Dynamics
      </h2>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* User Relationships */}
        <div>
          <h3 className="text-sm font-semibold text-slate-300 mb-3 border-b border-slate-800 pb-2">Users (Discord)</h3>
          <div className="flex flex-col gap-3 mb-4 max-h-[250px] overflow-y-auto pr-2 custom-scrollbar">
            {userRelationships.map((rel) => (
              <div key={rel.userId} className="p-3 bg-slate-900/60 rounded-lg border border-slate-800 flex justify-between items-start group">
                <div>
                  <div className="text-sm font-bold text-white flex items-center gap-2">
                    <span>{rel.alias}</span>
                    <span className="text-[10px] text-slate-500 font-mono bg-slate-950 px-1.5 py-0.5 rounded">{rel.userId}</span>
                  </div>
                  <div className="text-xs text-cyan-300/90 mt-1.5 font-medium">&quot;{rel.stance}&quot;</div>
                </div>
                <button
                  onClick={() => setUserRelationships(userRelationships.filter((r) => r.userId !== rel.userId))}
                  className="text-xs text-slate-500 group-hover:text-red-400 transition-colors"
                >
                  ✕
                </button>
              </div>
            ))}
          </div>

          <div className="bg-slate-950/40 p-3 rounded-lg border border-slate-800/80 flex flex-col gap-2 text-xs">
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="User ID (Snowflake)"
                value={newRelUser}
                onChange={(e) => setNewRelUser(e.target.value)}
                className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1 focus:outline-none focus:border-cyan-500/50"
              />
              <input
                type="text"
                placeholder="Alias"
                value={newRelAlias}
                onChange={(e) => setNewRelAlias(e.target.value)}
                className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1 focus:outline-none focus:border-cyan-500/50"
              />
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Stance / Dynamics"
                value={newRelStance}
                onChange={(e) => setNewRelStance(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddUserRelationship()}
                className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-[2] focus:outline-none focus:border-cyan-500/50"
              />
              <button onClick={handleAddUserRelationship} className="bg-cyan-600 hover:bg-cyan-500 text-white font-medium rounded px-3 py-1.5 transition-colors">
                Add User
              </button>
            </div>
          </div>
        </div>

        {/* Non-User Relationships */}
        <div>
          <h3 className="text-sm font-semibold text-slate-300 mb-3 border-b border-slate-800 pb-2">Non-Users (Entities / Concepts)</h3>
          <div className="flex flex-col gap-3 mb-4 max-h-[250px] overflow-y-auto pr-2 custom-scrollbar">
            {nonUserRelationships.map((rel, idx) => (
              <div key={idx} className="p-3 bg-slate-900/60 rounded-lg border border-slate-800 flex justify-between items-start group">
                <div>
                  <div className="text-sm font-bold text-slate-200">{rel.entity}</div>
                  <div className="text-xs text-amber-300/80 mt-1.5 font-medium">&quot;{rel.stance}&quot;</div>
                </div>
                <button
                  onClick={() => setNonUserRelationships(nonUserRelationships.filter((_, i) => i !== idx))}
                  className="text-xs text-slate-500 group-hover:text-red-400 transition-colors"
                >
                  ✕
                </button>
              </div>
            ))}
          </div>

          <div className="bg-slate-950/40 p-3 rounded-lg border border-slate-800/80 flex flex-col gap-2 text-xs">
            <input
              type="text"
              placeholder="Entity / Concept (e.g. JavaScript, The Government)"
              value={newNonUser}
              onChange={(e) => setNewNonUser(e.target.value)}
              className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white w-full focus:outline-none focus:border-amber-500/50"
            />
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Stance / Opinion"
                value={newNonUserStance}
                onChange={(e) => setNewNonUserStance(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddNonUserRelationship()}
                className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-white flex-1 focus:outline-none focus:border-amber-500/50"
              />
              <button onClick={handleAddNonUserRelationship} className="bg-amber-600/80 hover:bg-amber-500/80 text-white font-medium rounded px-3 py-1.5 transition-colors">
                Add Entity
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
