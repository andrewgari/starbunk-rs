"use client";

import { useState, useEffect } from "react";
import AddBotModal from "@/components/AddBotModal";
import SubBotCard, { SubBotData } from "@/components/SubBotCard";
import { useSSE } from "@/components/useSSE";
import { saveBunkBotConfigJson } from "./actions";

export default function BunkBotMagnumOpus() {
  const [subBots, setSubBots] = useState<SubBotData[]>([]);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [globalRateLimit, setGlobalRateLimit] = useState(10);
  const [globalEnabled, setGlobalEnabled] = useState(true);
  const [isReloading, setIsReloading] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const { isConnected, bunkbotAudits } = useSSE();

  const loadBots = async () => {
    try {
      const [botsRes, statusRes] = await Promise.all([
        fetch("/api/bots"),
        fetch("/api/bots/status"),
      ]);
      if (!botsRes.ok || !statusRes.ok) return;

      const bots = await botsRes.json();
      const statuses = await statusRes.json();

      const mapped: SubBotData[] = bots.map((b: any) => {
        const st = statuses.find((s: any) => s.name === b.name) || {};
        return {
          name: b.name,
          enabled: st.enabled ?? true,
          frequency: st.current_frequency ?? b.frequency ?? 100,
          ignore_bots: b.ignore_bots ?? true,
          ignore_humans: b.ignore_humans ?? false,
          ignore_self: b.ignore_self ?? true,
          identityType: b.identity.type,
          bot_name: b.identity.bot_name,
          avatar_url: b.identity.avatar_url,
          user_id: b.identity.user_id,
          responses: b.responses || [],
          triggersCount: b.triggers?.length || 0,
          yamlSnippet: JSON.stringify(b, null, 2),
          triggersToday: st.triggers_today || 0,
          botConfig: b,
        };
      });
      setSubBots(mapped);
    } catch (err) {
      console.error("Failed to load bots", err);
    }
  };

  useEffect(() => {
    loadBots();
  }, []);

  const handleAddBotYaml = async (jsonInput: string) => {
    try {
      const newBot = JSON.parse(jsonInput);
      const updatedList = [...subBots.map(b => b.botConfig), newBot];

      const res = await fetch("/api/bots", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(updatedList),
      });
      const result = await saveBunkBotConfigJson(updatedList);
      if (!result.success) {
        setSaveError(result.error || "Failed to save configuration");
        return;
      }
      setSaveError(null);
      if (res.ok) {
        await loadBots();
      }
    } catch (e) {
      console.error(e);
      throw new Error("Failed to parse or save bot JSON");
    }
  };

  const handleUpdateBot = async (updated: SubBotData) => {
    const oldBot = subBots.find((b) => b.name === updated.name);
    if (!oldBot) return;

    if (oldBot.enabled !== updated.enabled) {
      await fetch(`/api/bots/${updated.name}/${updated.enabled ? "enable" : "disable"}`, {
        method: "POST",
      });
    }

    if (oldBot.frequency !== updated.frequency) {
      await fetch(`/api/bots/${updated.name}/frequency`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ frequency: updated.frequency }),
      });
    }

    if (
      oldBot.yamlSnippet !== updated.yamlSnippet ||
      oldBot.identityType !== updated.identityType ||
      oldBot.ignore_bots !== updated.ignore_bots ||
      oldBot.ignore_humans !== updated.ignore_humans ||
      oldBot.ignore_self !== updated.ignore_self ||
      oldBot.bot_name !== updated.bot_name ||
      oldBot.avatar_url !== updated.avatar_url ||
      oldBot.user_id !== updated.user_id
    ) {
      try {
        const snippetChanged = oldBot.yamlSnippet !== updated.yamlSnippet;
        let newBotConfig = JSON.parse(updated.yamlSnippet);

        if (!newBotConfig.identity || typeof newBotConfig.identity !== "object") {
          newBotConfig.identity = {};
        }

        if (snippetChanged) {
          if (updated.identityType === "static") {
            if (updated.bot_name == null) updated.bot_name = newBotConfig.identity.bot_name;
            if (updated.avatar_url == null) updated.avatar_url = newBotConfig.identity.avatar_url;
          } else if (updated.identityType === "mimic") {
            if (updated.user_id == null) updated.user_id = newBotConfig.identity.user_id;
          }
        }

        // Also apply the UI toggled ignore rules & identity if they were modified from the UI directly
        newBotConfig.ignore_bots = updated.ignore_bots;
        newBotConfig.ignore_humans = updated.ignore_humans;
        newBotConfig.ignore_self = updated.ignore_self;

        newBotConfig.identity.type = updated.identityType;
        if (updated.identityType === "static") {
          newBotConfig.identity.bot_name = updated.bot_name || "";
          newBotConfig.identity.avatar_url = updated.avatar_url || "";
          delete newBotConfig.identity.user_id;
        } else if (updated.identityType === "mimic") {
          newBotConfig.identity.user_id = updated.user_id || "";
          delete newBotConfig.identity.bot_name;
          delete newBotConfig.identity.avatar_url;
        } else {
          delete newBotConfig.identity.bot_name;
          delete newBotConfig.identity.avatar_url;
          delete newBotConfig.identity.user_id;
        }

        const updatedList = subBots.map(b =>
          b.name === updated.name ? newBotConfig : b.botConfig
        );
        await fetch("/api/bots", {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(updatedList),
        });
        const result = await saveBunkBotConfigJson(updatedList);
        if (!result.success) {
          setSaveError(result.error || "Failed to save configuration");
          return;
        }
        setSaveError(null);
        updated.yamlSnippet = JSON.stringify(newBotConfig, null, 2);
      } catch (e) {
        console.error("Invalid JSON snippet", e);
      }
    }

    setSubBots((prev) => prev.map((b) => (b.name === updated.name ? updated : b)));
  };

  const handleDeleteBot = async (name: string) => {
    const updatedList = subBots.filter(b => b.name !== name).map(b => b.botConfig);
    const res = await fetch("/api/bots", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(updatedList),
    });
    const result = await saveBunkBotConfigJson(updatedList);
    if (!result.success) {
      setSaveError(result.error || "Failed to save configuration");
      return;
    }
    setSaveError(null);
    if (res.ok) {
      await loadBots();
    }
  };

  const handleHotReload = async () => {
    setIsReloading(true);
    await loadBots();
    setIsReloading(false);
  };

  return (
    <div className="flex flex-col h-full gap-6 max-w-6xl mx-auto py-6">
      {/* Header */}
      <header className="flex justify-between items-start">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>BunkBot</span>
            <span className="text-xs px-2.5 py-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/30 uppercase font-mono tracking-wide">
              Magnum Opus
            </span>
          </h1>
          <p className="text-slate-400 mt-1">
            Global administrative backbone &amp; per-bot modular reply engine control plane.
          </p>
        </div>

        <button
          onClick={() => setIsAddModalOpen(true)}
          className="btn-primary flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-500/20"
        >
          <span>+</span> Add Bot
        </button>
      </header>

      {/* Save Error Banner */}
      {saveError && (
        <div className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-300 flex items-center justify-between">
          <span><strong className="font-semibold">Config save failed:</strong> {saveError}</span>
          <button
            onClick={() => setSaveError(null)}
            className="ml-4 text-red-400 hover:text-red-200 font-bold text-lg leading-none"
            aria-label="Dismiss error"
          >
            &times;
          </button>
        </div>
      )}

      {/* Global Controls & HUD */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="glass-panel p-6 border-indigo-500/30">
          <div className="text-xs font-semibold text-indigo-400 uppercase tracking-wider mb-1">Global Master Switch</div>
          <div className="flex items-center gap-3 mt-2">
            <button
              onClick={() => setGlobalEnabled(!globalEnabled)}
              className={`w-12 h-6 rounded-full transition-colors relative flex items-center px-1 ${
                globalEnabled ? "bg-emerald-500" : "bg-slate-700"
              }`}
            >
              <div className={`w-4 h-4 rounded-full bg-white transition-transform ${
                globalEnabled ? "translate-x-6" : "translate-x-0"
              }`} />
            </button>
            <span className="text-xl font-bold text-white">
              {globalEnabled ? "Active" : "Paused"}
            </span>
          </div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">Sub-Bot Army</div>
          <div className="text-3xl font-bold text-white flex items-baseline gap-2">
            <span>{subBots.filter((b) => b.enabled).length}</span>
            <span className="text-xs text-slate-500 font-normal">/ {subBots.length} Enabled</span>
          </div>
        </div>

        <div className="glass-panel p-6">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">Global Rate Limit</div>
          <div className="flex flex-col gap-1 mt-1">
            <div className="flex justify-between text-xs text-slate-300 font-mono">
              <span>Limit:</span>
              <span className="text-amber-400 font-bold">{globalRateLimit} msg/sec</span>
            </div>
            <input
              type="range"
              min="1"
              max="50"
              value={globalRateLimit}
              onChange={(e) => setGlobalRateLimit(Number(e.target.value))}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-amber-500"
            />
          </div>
        </div>

        <div className="glass-panel p-6 flex flex-col justify-between">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">Hot-Reload Engine</div>
          <button
            onClick={handleHotReload}
            disabled={isReloading}
            className="btn-secondary w-full text-xs text-indigo-300 border-indigo-500/30 hover:bg-indigo-500/10 mt-2"
          >
            {isReloading ? "Reloading Strategies..." : "⚡ Trigger Hot-Reload"}
          </button>
        </div>
      </div>

      {/* Sub-Bots Grid */}
      <section className="flex flex-col gap-4">
        <div className="flex justify-between items-center">
          <h2 className="text-xl font-semibold text-white">Managed Reply Bots ({subBots.length})</h2>
          <span className="text-xs text-slate-400">Configure triggers, frequency limiters, and identities per bot.</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {subBots.map((bot) => (
            <SubBotCard
              key={bot.name}
              bot={bot}
              onUpdateBot={handleUpdateBot}
              onDeleteBot={handleDeleteBot}
            />
          ))}
        </div>
      </section>

      {/* Real-time Per-Bot Trigger Audit Log Stream */}
      <section className="glass-panel p-6">
        <div className="flex justify-between items-center mb-4">
          <div>
            <h2 className="text-xl font-semibold text-white flex items-center gap-2">
              <span>📡</span> Per-Bot Trigger Audit Stream
            </h2>
            <p className="text-xs text-slate-400 mt-0.5">Real-time live audit of sub-bot responses across channels.</p>
          </div>
          <div className="flex items-center gap-2">
            <span className={`w-2.5 h-2.5 rounded-full ${isConnected ? "bg-emerald-500 animate-pulse" : "bg-slate-500"}`} />
            <span className="text-xs text-slate-400 font-mono">{isConnected ? "SSE Stream Connected" : "Connecting..."}</span>
          </div>
        </div>

        <div className="bg-slate-950/80 rounded-lg p-4 border border-slate-800 font-mono text-xs max-h-60 overflow-y-auto flex flex-col gap-2">
          {bunkbotAudits.length === 0 ? (
            <div className="text-slate-500 text-center py-6">Listening for live trigger audit events...</div>
          ) : (
            bunkbotAudits.map((evt) => (
              <div key={evt.id} className="flex items-center justify-between p-2 rounded bg-slate-900/60 border border-slate-800/60">
                <div className="flex items-center gap-3">
                  <span className="text-indigo-400 font-semibold">[{evt.subBot || "BunkBot"}]</span>
                  <span className="text-slate-300">{evt.user}:</span>
                  <span className="text-emerald-400">&quot;{evt.response}&quot;</span>
                </div>
                <span className="text-slate-500 text-[10px]">{evt.timestamp}</span>
              </div>
            ))
          )}
        </div>
      </section>

      {/* Add Bot Modal */}
      <AddBotModal
        isOpen={isAddModalOpen}
        onClose={() => setIsAddModalOpen(false)}
        onAddBot={handleAddBotYaml}
      />
    </div>
  );
}
