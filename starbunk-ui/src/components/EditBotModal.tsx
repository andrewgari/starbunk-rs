/* eslint-disable react/no-unescaped-entities, react-hooks/set-state-in-effect */
"use client";

import { useState, useEffect } from "react";
import TriggerEditor, { TriggerConfig } from "./TriggerEditor";
import { SubBotData } from "./SubBotCard";

interface EditBotModalProps {
  bot: SubBotData;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedBot: SubBotData) => void;
}

export default function EditBotModal({ bot, isOpen, onClose, onSave }: EditBotModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [botName, setBotName] = useState(bot.name);
  const [identityType, setIdentityType] = useState(bot.identityType);
  const [staticBotName, setStaticBotName] = useState(bot.bot_name || "");
  const [staticAvatarUrl, setStaticAvatarUrl] = useState(bot.avatar_url || "");
  const [mimicUserId, setMimicUserId] = useState(bot.user_id || "");
  
  const [frequency, setFrequency] = useState(bot.frequency);
  const [targetAudience, setTargetAudience] = useState<"humans" | "bots">(bot.ignore_bots && !bot.ignore_humans ? "humans" : "bots");
  
  const [triggers, setTriggers] = useState<TriggerConfig[]>(JSON.parse(bot.yamlSnippet).triggers || []);
  const [responses, setResponses] = useState<string[]>(bot.responses);
  const [newResponse, setNewResponse] = useState("");

  // Sync state when bot changes (if they open the modal for a different bot)
  useEffect(() => {
     
    if (isOpen) {
       
      setBotName(bot.name);
      setIdentityType(bot.identityType);
      setStaticBotName(bot.bot_name || "");
      setStaticAvatarUrl(bot.avatar_url || "");
      setMimicUserId(bot.user_id || "");
      setFrequency(bot.frequency);
      setTargetAudience(bot.ignore_bots && !bot.ignore_humans ? "humans" : "bots");
      setTriggers(JSON.parse(bot.yamlSnippet).triggers || []);
      setResponses(bot.responses);
      setNewResponse("");
    }
  }, [bot, isOpen]);

  if (!isOpen) return null;

  const handleAddResponse = () => {
    const trimmed = newResponse.trim();
    if (!trimmed) return;
    setResponses([...responses, trimmed]);
    setNewResponse("");
  };

  const handleUpdateResponse = (index: number, newVal: string) => {
    const newResponses = [...responses];
    newResponses[index] = newVal;
    setResponses(newResponses);
  };

  const handleRemoveResponse = (index: number) => {
    setResponses(responses.filter((_, i) => i !== index));
  };

  const handleSubmit = () => {
    setIsSubmitting(true);
    
    // Construct the updated SubBotData
    const updatedYamlSnippetObj = { ...JSON.parse(bot.yamlSnippet) };
    updatedYamlSnippetObj.triggers = triggers;
    updatedYamlSnippetObj.responses = responses;

    const updatedBot: SubBotData = {
      ...bot,
      name: botName,
      identityType,
      bot_name: staticBotName,
      avatar_url: staticAvatarUrl,
      user_id: mimicUserId,
      frequency,
      ignore_bots: targetAudience === "humans",
      ignore_humans: targetAudience === "bots",
      ignore_self: true, // Always true per previous requirements
      responses,
      triggersCount: triggers.length,
      yamlSnippet: JSON.stringify(updatedYamlSnippetObj, null, 2)
    };

    onSave(updatedBot);
    setIsSubmitting(false);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="glass-panel max-w-2xl w-full max-h-[90vh] flex flex-col border border-indigo-500/30 shadow-2xl rounded-xl">
        <div className="flex justify-between items-center border-b border-slate-700/50 p-4 shrink-0">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <span>🤖</span> Edit Reply Bot: {bot.name}
            </h2>
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-white text-lg font-bold">✕</button>
        </div>

        <div className="p-6 overflow-y-auto flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-slate-400">Bot Name (ID - Cannot be changed)</label>
            <input type="text" value={botName} disabled className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-slate-500 cursor-not-allowed outline-none" />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Identity Profile</label>
              <select value={identityType} onChange={e => setIdentityType(e.target.value as string)} className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white focus:border-indigo-500 outline-none">
                <option value="static">Custom Profile</option>
                <option value="mimic">Copy Specific User</option>
                <option value="random">Random Server Member</option>
                <option value="mimic_poster">Copy the Message Sender</option>
              </select>
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Target Audience</label>
              <select value={targetAudience} onChange={e => setTargetAudience(e.target.value as "humans" | "bots")} className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white focus:border-indigo-500 outline-none">
                <option value="humans">Humans (Ignore other bots)</option>
                <option value="bots">Bots (Ignore humans)</option>
              </select>
            </div>
          </div>

          <div className="flex flex-col gap-1">
            <div className="flex justify-between items-center text-xs">
              <span className="text-slate-400 font-medium">Trigger Frequency Rate</span>
              <span className="text-emerald-400 font-bold font-mono">{frequency}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="100"
              value={frequency}
              onChange={(e) => setFrequency(Number(e.target.value))}
              className="w-full h-1.5 bg-slate-950 rounded-lg appearance-none cursor-pointer accent-emerald-500"
            />
          </div>

          {identityType === "static" && (
            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-1">
                <label className="text-xs text-slate-400">Bot Display Name</label>
                <input
                  type="text"
                  value={staticBotName}
                  onChange={e => setStaticBotName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white focus:border-indigo-500 outline-none"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label className="text-xs text-slate-400">Avatar URL</label>
                <input
                  type="text"
                  value={staticAvatarUrl}
                  onChange={e => setStaticAvatarUrl(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white focus:border-indigo-500 outline-none"
                />
              </div>
            </div>
          )}

          {identityType === "mimic" && (
            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Discord User ID</label>
              <input
                type="text"
                value={mimicUserId}
                onChange={e => setMimicUserId(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white focus:border-indigo-500 outline-none"
              />
            </div>
          )}

          {/* Response Pool Editor */}
          <div className="flex flex-col gap-2 p-3 border border-slate-700 rounded-lg bg-slate-900/50 mt-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-slate-400 font-medium">
                Global Response Pool ({responses.length})
              </span>
              <span className="text-[10px] text-slate-400 group relative cursor-help">
                💡 Template Cheat Sheet
                <div className="hidden group-hover:flex absolute right-0 bottom-full mb-2 w-64 flex-col gap-1 bg-slate-800 p-2 text-xs rounded shadow-lg border border-slate-600 z-10">
                  <p><strong>URL</strong>: Auto-embeds images/links</p>
                  <p><strong>{"{start}"}</strong>: Excerpt of trigger msg</p>
                  <p><strong>{"{swap_message:a:b}"}</strong>: Swap words a & b</p>
                  <p><strong>{"{random:1-5:e}"}</strong>: Repeats 'e' 1-5 times</p>
                </div>
              </span>
            </div>
            
            {responses.length > 0 && (
              <div className="flex flex-col gap-2 max-h-48 overflow-y-auto pr-1">
                {responses.map((r, i) => (
                  <div key={i} className="flex flex-col bg-slate-950 border border-slate-800 rounded p-1">
                    <div className="flex justify-between items-center bg-slate-900/50 px-2 py-1 border-b border-slate-800">
                      <span className="text-[10px] text-slate-500 font-mono">Response #{i + 1}</span>
                      <button
                        onClick={() => handleRemoveResponse(i)}
                        className="text-red-400 hover:text-red-300 leading-none text-sm font-bold"
                        title="Delete response"
                      >
                        &times;
                      </button>
                    </div>
                    <textarea
                      value={r}
                      onChange={(e) => handleUpdateResponse(i, e.target.value)}
                      className="w-full bg-transparent border-none p-2 text-xs font-mono text-slate-300 focus:outline-none focus:ring-0 resize-y min-h-[60px]"
                      placeholder="Type response..."
                      spellCheck={false}
                    />
                  </div>
                ))}
              </div>
            )}
            
            <div className="flex flex-col gap-1 mt-1">
              <textarea
                value={newResponse}
                onChange={(e) => setNewResponse(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    handleAddResponse();
                  }
                }}
                placeholder="Type new response here..."
                className="w-full bg-slate-950 border border-slate-800 rounded px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500 resize-y min-h-[60px]"
              />
              <span className="text-[10px] text-slate-500">Press Enter to add, Shift+Enter for new line</span>
            </div>
          </div>

          <div className="mt-2 border-t border-slate-800 pt-3">
            <label className="text-xs text-indigo-400 font-semibold mb-2 block">Triggers & Logic Gates</label>
            <TriggerEditor triggers={triggers} onChange={setTriggers} />
          </div>
        </div>

        <div className="flex justify-end gap-3 p-4 border-t border-slate-700/50 shrink-0">
          <button onClick={onClose} className="btn-secondary text-xs">Cancel</button>
          <button
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="btn-primary text-xs px-5"
          >
            {isSubmitting ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </div>
    </div>
  );
}
