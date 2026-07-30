"use client";

import { useState, useEffect } from "react";
import EditBotModal from "./EditBotModal";

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

  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [mimicIdentity, setMimicIdentity] = useState<{username: string, avatar_url: string} | null>(null);

  useEffect(() => {
    if (bot.identityType === "mimic" && bot.user_id) {
      fetch(`/api/user/${bot.user_id}`)
        .then(res => res.json())
        .then(data => {
          if (data && data.username) {
            setMimicIdentity({
              username: data.nickname || data.username,
              avatar_url: data.avatar_url
            });
          }
        })
        .catch(err => console.error("Failed to fetch mimic user:", err));
    }
  }, [bot.identityType, bot.user_id]);


  const toggleEnabled = () => {
    onUpdateBot({ ...bot, enabled: !bot.enabled });
  };

  // Helper to determine target audience text
  const getAudienceText = () => {
    if (bot.ignore_bots && !bot.ignore_humans) return "Humans";
    if (bot.ignore_humans && !bot.ignore_bots) return "Bots";
    return "Everyone";
  };

const getDisplayName = () => {
    if (bot.identityType === "static" && bot.bot_name) return bot.bot_name;
    if (bot.identityType === "mimic" && bot.user_id) return mimicIdentity ? mimicIdentity.username : `Mimicking ${bot.user_id}`;
    if (bot.identityType === "mimic_poster") return "Copy Message Sender";
    if (bot.identityType === "random") return "Random Server Member";
    return "Reply Bot";
  };

  return (
    <>
      <div className={`glass-panel p-5 flex flex-col gap-3 border transition-all ${
        bot.enabled ? "border-slate-700/60" : "border-slate-800/40 opacity-60"
      }`}>
        {/* Header */}
        <div className="flex justify-between items-start">
          <div className="flex items-center gap-3 min-w-0">
            <button
              onClick={toggleEnabled}
              className={`w-10 h-5 rounded-full transition-colors relative flex items-center px-0.5 shrink-0 ${
                bot.enabled ? "bg-emerald-500" : "bg-slate-700"
              }`}
            >
              <div className={`w-4 h-4 rounded-full bg-white transition-transform ${
                bot.enabled ? "translate-x-5" : "translate-x-0"
              }`} />
            </button>
            
            {/* AvatarUrl */}
            {bot.avatar_url ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img src={bot.avatar_url} alt="Avatar" className="w-12 h-12 rounded-full object-cover border border-slate-700 shrink-0" />
            ) : (bot.identityType === "mimic" && mimicIdentity && mimicIdentity.avatar_url) ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img src={mimicIdentity.avatar_url} alt="Mimic Avatar" className="w-12 h-12 rounded-full object-cover border border-slate-700 shrink-0" />
            ) : (
              <div className="w-12 h-12 rounded-full bg-slate-800 flex items-center justify-center text-2xl border border-slate-700 shrink-0">
                🤖
              </div>
            )}
            
            <div className="flex flex-col justify-center min-w-0">
              {/* Bot name (small) */}
              <div className="text-xs text-slate-400 font-mono flex items-center gap-2">
                {bot.name}
                <span className="text-[10px] px-1.5 py-0 rounded bg-slate-800 text-slate-500 uppercase tracking-wider">
                  {bot.identityType}
                </span>
              </div>
              
              {/* Display Name (Large) with Hover Percent Change */}
              <h3 className="text-xl font-bold text-white flex items-center gap-2 group cursor-default h-7 w-full overflow-hidden">
                <span className="truncate flex-shrink">{getDisplayName()}</span>
                <span className="opacity-0 group-hover:opacity-100 transition-opacity text-xs px-2 py-0.5 rounded bg-slate-800/50 font-mono text-emerald-400 border border-emerald-500/20">
                  {bot.frequency}% Chance
                </span>
              </h3>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={() => setIsEditModalOpen(true)}
              className="text-xs text-indigo-400 hover:text-indigo-300 p-1 hover:bg-indigo-500/10 rounded font-semibold"
            >
              Edit Details
            </button>
            <button
              onClick={() => onDeleteBot(bot.name)}
              className="text-xs text-red-400 hover:text-red-300 p-1 hover:bg-red-500/10 rounded"
              title="Delete Sub-Bot"
            >
              🗑️
            </button>
          </div>
        </div>

        {/* The rest of the fine details */}
        <div className="mt-1 pl-[4.5rem]">
          <div className="text-xs text-slate-400 flex items-center gap-2">
            <span>{bot.triggersCount} Triggers</span>
            <span className="text-slate-600">•</span>
            <span>{bot.responses.length} Responses</span>
            <span className="text-slate-600">•</span>
            <span>Target: {getAudienceText()}</span>
          </div>
          <div className="text-xs text-indigo-400 mt-0.5">
            {bot.triggersToday ?? 0} Triggers Today
          </div>
          
          {/* Quick Preview of Responses */}
          {bot.responses.length > 0 && (
            <div className="mt-2 bg-slate-900/50 border border-slate-800 rounded p-2 text-xs text-slate-400 font-mono italic truncate">
              &quot;{bot.responses[0]}&quot; {bot.responses.length > 1 && `(+${bot.responses.length - 1} more)`}
            </div>
          )}
        </div>
      </div>

      <EditBotModal
        bot={bot}
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onSave={(updatedBot) => onUpdateBot(updatedBot)}
      />
    </>
  );
}
