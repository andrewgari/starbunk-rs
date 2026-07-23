"use client";

import { useState, useEffect } from "react";

export interface SubBotData {
  name: string;
  enabled: boolean;
  frequency: number;
  ignore_bots: boolean;
  ignore_humans: boolean;
  ignore_self: boolean;
  identityType: "static" | "mimic" | "random" | "mimic_poster";
  bot_name?: string;
  avatar_url?: string;
  user_id?: string;
  responses: string[];
  triggersCount: number;
  yamlSnippet: string;
  triggersToday?: number;
  botConfig?: any;
}

interface SubBotCardProps {
  bot: SubBotData;
  onUpdateBot: (updated: SubBotData) => void;
  onDeleteBot: (name: string) => void;
}

export default function SubBotCard({ bot, onUpdateBot, onDeleteBot }: SubBotCardProps) {
  const [isEditingCode, setIsEditingCode] = useState(false);
  const [snippet, setSnippet] = useState(bot.yamlSnippet);
  const [newResponse, setNewResponse] = useState("");

  const handleAddResponse = () => {
    const trimmed = newResponse.trim();
    if (!trimmed) return;
    const newResponses = [...bot.responses, trimmed];
    const updatedConfig = { ...JSON.parse(bot.yamlSnippet), responses: newResponses };
    onUpdateBot({ ...bot, responses: newResponses, yamlSnippet: JSON.stringify(updatedConfig, null, 2) });
    setNewResponse("");
  };

  const handleRemoveResponse = (index: number) => {
    const newResponses = bot.responses.filter((_, i) => i !== index);
    const updatedConfig = { ...JSON.parse(bot.yamlSnippet), responses: newResponses };
    onUpdateBot({ ...bot, responses: newResponses, yamlSnippet: JSON.stringify(updatedConfig, null, 2) });
  };

  const [staticBotName, setStaticBotName] = useState(bot.bot_name || "");
  const [staticAvatarUrl, setStaticAvatarUrl] = useState(bot.avatar_url || "");
  const [mimicUserId, setMimicUserId] = useState(bot.user_id || "");

  useEffect(() => {
    setStaticBotName(bot.bot_name || "");
    setStaticAvatarUrl(bot.avatar_url || "");
    setMimicUserId(bot.user_id || "");
  }, [bot.bot_name, bot.avatar_url, bot.user_id]);

  useEffect(() => {
    if (!isEditingCode) {
      setSnippet(bot.yamlSnippet);
    }
  }, [bot.yamlSnippet, isEditingCode]);

  const handleIdentityDetailUpdate = (updates: Partial<SubBotData>) => {
    onUpdateBot({ ...bot, ...updates });
  };

  const toggleEnabled = () => {
    onUpdateBot({ ...bot, enabled: !bot.enabled });
  };

  const handleFrequencyChange = (val: number) => {
    onUpdateBot({ ...bot, frequency: val });
  };

  const handleIdentityChange = (type: SubBotData["identityType"]) => {
    onUpdateBot({ ...bot, identityType: type });
  };

  const handleSaveSnippet = () => {
    onUpdateBot({ ...bot, yamlSnippet: snippet });
    setIsEditingCode(false);
  };

  return (
    <div className={`glass-panel p-5 flex flex-col gap-4 border transition-all ${
      bot.enabled ? "border-slate-700/60" : "border-slate-800/40 opacity-60"
    }`}>
      {/* Header */}
      <div className="flex justify-between items-start">
        <div className="flex items-center gap-3">
          <button
            onClick={toggleEnabled}
            className={`w-10 h-5 rounded-full transition-colors relative flex items-center px-0.5 ${
              bot.enabled ? "bg-emerald-500" : "bg-slate-700"
            }`}
          >
            <div className={`w-4 h-4 rounded-full bg-white transition-transform ${
              bot.enabled ? "translate-x-5" : "translate-x-0"
            }`} />
          </button>
          <div>
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              {bot.name}
              <span className="text-xs px-2 py-0.5 rounded bg-slate-800 font-mono text-slate-400">
                {bot.identityType}
              </span>
            </h3>
            <div className="text-xs text-slate-400">
              {bot.triggersCount} Triggers • {bot.responses.length} Responses Pool
            </div>
            <div className="text-xs text-indigo-400 mt-0.5">
              {bot.triggersToday ?? 0} Triggers Today
            </div>
          </div>
        </div>

        <button
          onClick={() => onDeleteBot(bot.name)}
          className="text-xs text-red-400 hover:text-red-300 p-1 hover:bg-red-500/10 rounded"
          title="Delete Sub-Bot"
        >
          🗑️
        </button>
      </div>

      {/* Frequency Rate Slider */}
      <div className="bg-slate-900/40 p-3 rounded-lg border border-slate-800/60 flex flex-col gap-1.5">
        <div className="flex justify-between items-center text-xs">
          <span className="text-slate-400 font-medium">Trigger Frequency Rate</span>
          <span className="text-emerald-400 font-bold font-mono">{bot.frequency}%</span>
        </div>
        <input
          type="range"
          min="0"
          max="100"
          value={bot.frequency}
          onChange={(e) => handleFrequencyChange(Number(e.target.value))}
          className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-500"
        />
      </div>

      {/* Ignore Flags */}
      <div className="grid grid-cols-3 gap-2 text-xs">
        <label className="flex items-center gap-1.5 text-slate-300 cursor-pointer bg-slate-800/30 p-2 rounded">
          <input
            type="checkbox"
            checked={bot.ignore_bots}
            onChange={(e) => onUpdateBot({ ...bot, ignore_bots: e.target.checked })}
            className="accent-accent"
          />
          <span>Ignore Bots</span>
        </label>
        <label className="flex items-center gap-1.5 text-slate-300 cursor-pointer bg-slate-800/30 p-2 rounded">
          <input
            type="checkbox"
            checked={bot.ignore_humans}
            onChange={(e) => onUpdateBot({ ...bot, ignore_humans: e.target.checked })}
            className="accent-accent"
          />
          <span>Ignore Humans</span>
        </label>
        <label className="flex items-center gap-1.5 text-slate-300 cursor-pointer bg-slate-800/30 p-2 rounded">
          <input
            type="checkbox"
            checked={bot.ignore_self}
            onChange={(e) => onUpdateBot({ ...bot, ignore_self: e.target.checked })}
            className="accent-accent"
          />
          <span>Ignore Self</span>
        </label>
      </div>

      {/* Identity Selector */}
      <div className="flex flex-col gap-1.5">
        <span className="text-xs text-slate-400 font-medium">Persona Identity Mode</span>
        <div className="grid grid-cols-4 gap-1.5 text-xs">
          {(["static", "mimic", "random", "mimic_poster"] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => handleIdentityChange(mode)}
              className={`p-1.5 rounded text-center truncate ${
                bot.identityType === mode
                  ? "bg-indigo-600 text-white font-semibold"
                  : "bg-slate-800/60 text-slate-400 hover:text-white"
              }`}
            >
              {mode.replace("_", " ")}
            </button>
          ))}
        </div>
      </div>

      {bot.identityType === "static" && (
        <>
          <div className="flex flex-col gap-1.5 mt-2">
            <span className="text-xs text-slate-400 font-medium">Bot Display Name</span>
            <input
              type="text"
              value={staticBotName}
              onChange={(e) => setStaticBotName(e.target.value)}
              onBlur={() => handleIdentityDetailUpdate({ bot_name: staticBotName })}
              className="w-full bg-slate-900 border border-slate-700 rounded p-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              placeholder="e.g. HelperBot"
            />
          </div>
          <div className="flex flex-col gap-1.5 mt-2">
            <span className="text-xs text-slate-400 font-medium">Avatar URL</span>
            <input
              type="text"
              value={staticAvatarUrl}
              onChange={(e) => setStaticAvatarUrl(e.target.value)}
              onBlur={() => handleIdentityDetailUpdate({ avatar_url: staticAvatarUrl })}
              className="w-full bg-slate-900 border border-slate-700 rounded p-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              placeholder="https://..."
            />
          </div>
        </>
      )}

      {bot.identityType === "mimic" && (
        <div className="flex flex-col gap-1.5 mt-2">
          <span className="text-xs text-slate-400 font-medium">Discord User ID</span>
          <input
            type="text"
            value={mimicUserId}
            onChange={(e) => setMimicUserId(e.target.value)}
            onBlur={() => handleIdentityDetailUpdate({ user_id: mimicUserId })}
            className="w-full bg-slate-900 border border-slate-700 rounded p-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
            placeholder="e.g. 123456789012345678"
          />
        </div>
      )}

      {/* Response Pool Editor */}
      <div className="flex flex-col gap-2">
        <span className="text-xs text-slate-400 font-medium">
          Response Pool ({bot.responses.length})
        </span>
        {bot.responses.length > 0 && (
          <ul className="flex flex-col gap-1 max-h-32 overflow-y-auto">
            {bot.responses.map((r, i) => (
              <li
                key={i}
                className="flex items-center justify-between gap-2 bg-slate-900/50 border border-slate-800 rounded px-2 py-1 text-xs font-mono text-slate-300"
              >
                <span className="truncate">{r}</span>
                <button
                  onClick={() => handleRemoveResponse(i)}
                  className="text-red-400 hover:text-red-300 shrink-0 leading-none"
                  title="Remove response"
                >
                  &times;
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="flex gap-1">
          <input
            type="text"
            value={newResponse}
            onChange={(e) => setNewResponse(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddResponse()}
            placeholder="Add response…"
            className="flex-1 bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500"
          />
          <button
            onClick={handleAddResponse}
            className="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded"
          >
            Add
          </button>
        </div>
      </div>

      {/* Code Snippet Drawer */}
      <div className="mt-1 border-t border-slate-800 pt-3">
        <div className="flex justify-between items-center mb-2">
          <span className="text-xs font-mono text-slate-400">Sub-Bot Definition Snippet</span>
          <button
            onClick={() => setIsEditingCode(!isEditingCode)}
            className="text-xs text-indigo-400 hover:text-indigo-300 font-medium"
          >
            {isEditingCode ? "Close Snippet" : "Edit Snippet (JSON/YAML)"}
          </button>
        </div>

        {isEditingCode && (
          <div className="flex flex-col gap-2">
            <textarea
              value={snippet}
              onChange={(e) => setSnippet(e.target.value)}
              rows={6}
              className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-xs font-mono text-slate-200"
            />
            <div className="flex justify-end">
              <button
                onClick={handleSaveSnippet}
                className="btn-primary text-xs px-3 py-1"
              >
                Apply Snippet
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
