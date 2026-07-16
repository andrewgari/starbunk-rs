"use client";

import { useState, useEffect, useTransition } from "react";
import { listCovaBotProfiles, getCovaBotProfile, saveCovaBotProfile } from "./actions";

export default function CovaBotPage() {
  const [profiles, setProfiles] = useState<string[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<string>("default");
  const [newProfileName, setNewProfileName] = useState("");
  
  const [yaml, setYaml] = useState<string>("");
  const [originalYaml, setOriginalYaml] = useState<string>("");
  const [isLoading, setIsLoading] = useState(true);
  const [isPending, startTransition] = useTransition();
  const [message, setMessage] = useState<{type: 'success' | 'error', text: string} | null>(null);

  useEffect(() => {
    async function init() {
      const p = await listCovaBotProfiles();
      setProfiles(p);
      if (p.length > 0 && !p.includes("default")) {
        setSelectedProfile(p[0]);
      }
    }
    init();
  }, []);

  useEffect(() => {
    async function load() {
      setIsLoading(true);
      setMessage(null);
      const config = await getCovaBotProfile(selectedProfile);
      if (config !== null) {
        setYaml(config);
        setOriginalYaml(config);
      } else {
        setMessage({ type: "error", text: "Failed to load personality profile." });
      }
      setIsLoading(false);
    }
    load();
  }, [selectedProfile]);

  const handleSave = () => {
    setMessage(null);
    startTransition(async () => {
      const result = await saveCovaBotProfile(selectedProfile, yaml);
      if (result.success) {
        setMessage({ type: "success", text: "Personality profile saved successfully!" });
        setOriginalYaml(yaml);
        if (!profiles.includes(selectedProfile)) {
          setProfiles([...profiles, selectedProfile]);
        }
      } else {
        setMessage({ type: "error", text: `Failed to save: ${result.error}` });
      }
    });
  };

  const handleCreateProfile = () => {
    if (newProfileName.trim() && !profiles.includes(newProfileName.trim())) {
      setSelectedProfile(newProfileName.trim());
      setNewProfileName("");
    }
  };

  const hasChanges = yaml !== originalYaml;

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="flex justify-between items-end mb-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">CovaBot Personalities</h1>
          <p className="text-slate-400 mt-1">Manage multiple AI personality and behavior patterns.</p>
        </div>
        <div className="flex gap-3">
          {hasChanges && (
            <button 
              className="btn-secondary" 
              onClick={() => setYaml(originalYaml)}
              disabled={isPending}
            >
              Discard Changes
            </button>
          )}
          <button 
            className="btn-primary flex items-center gap-2" 
            onClick={handleSave}
            disabled={isPending || (!hasChanges && profiles.includes(selectedProfile))}
          >
            {isPending ? (
              <svg className="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            ) : "Save Profile"}
          </button>
        </div>
      </header>

      {message && (
        <div className={`p-4 rounded-lg font-medium ${message.type === 'success' ? 'bg-green-500/10 text-green-400 border border-green-500/20' : 'bg-red-500/10 text-red-400 border border-red-500/20'}`}>
          {message.text}
        </div>
      )}
      
      <div className="glass-panel p-4 flex gap-4 items-center">
        <label className="text-sm font-medium text-slate-300 whitespace-nowrap">Active Profile:</label>
        <select 
          className="bg-slate-900 border border-white/10 rounded-lg px-3 py-2 text-white outline-none flex-1 max-w-xs"
          value={selectedProfile}
          onChange={e => setSelectedProfile(e.target.value)}
        >
          {profiles.map(p => (
             <option key={p} value={p}>{p}</option>
          ))}
          {/* Ensure the currently typed/selected new profile shows up if it's not saved yet */}
          {!profiles.includes(selectedProfile) && (
            <option value={selectedProfile}>{selectedProfile} (New)</option>
          )}
        </select>
        
        <div className="w-px h-6 bg-white/10 mx-2"></div>
        
        <input 
          type="text" 
          placeholder="New profile name..." 
          className="bg-slate-900 border border-white/10 rounded-lg px-3 py-2 text-white outline-none flex-1 max-w-xs text-sm"
          value={newProfileName}
          onChange={e => setNewProfileName(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && handleCreateProfile()}
        />
        <button 
          className="btn-secondary text-sm" 
          onClick={handleCreateProfile}
          disabled={!newProfileName.trim()}
        >
          Create
        </button>
      </div>

      <div className="glass-panel flex-1 flex flex-col p-6 min-h-0 relative">
        <label className="text-sm font-medium text-slate-300 mb-2">YAML Configuration</label>
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="animate-pulse flex flex-col items-center gap-3">
              <div className="h-8 w-8 rounded-full border-2 border-accent border-t-transparent animate-spin"></div>
              <div className="text-slate-400">Loading profile...</div>
            </div>
          </div>
        ) : (
          <textarea
            value={yaml}
            onChange={(e) => setYaml(e.target.value)}
            className="flex-1 w-full bg-slate-900/50 border border-white/5 rounded-lg p-4 font-mono text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-accent/50 resize-none"
            spellCheck={false}
          />
        )}
      </div>
    </div>
  );
}
