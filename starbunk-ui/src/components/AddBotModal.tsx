"use client";

import { useState } from "react";
import TriggerEditor, { TriggerConfig } from "./TriggerEditor";

interface AddBotModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAddBot: (botConfigJson: string) => Promise<void>;
}

const TEMPLATES: Record<string, { label: string; code: string }> = {
  keyword: {
    label: "Keyword Reply Bot",
    code: `{
  "name": "KeywordResponder",
  "identity": {
    "type": "static",
    "bot_name": "HelperBot",
    "avatar_url": "https://example.com/avatar.png"
  },
  "frequency": 100,
  "ignore_bots": true,
  "ignore_humans": false,
  "ignore_self": true,
  "responses": [
    "Hello there! How can I help?"
  ],
  "triggers": [
    {
      "name": "Greetings",
      "conditions": {
        "contains_phrase": "hello"
      },
      "responses": [
        "Hey! Need any assistance?"
      ]
    }
  ]
}`,
  },
  mimic: {
    label: "Mimic Poster Bot",
    code: `{
  "name": "GhostPoster",
  "identity": {
    "type": "mimic_poster"
  },
  "frequency": 80,
  "ignore_bots": true,
  "ignore_humans": false,
  "ignore_self": true,
  "responses": [
    "I totally agree with that!"
  ],
  "triggers": [
    {
      "name": "Agreement",
      "conditions": {
        "contains_word": "agree"
      }
    }
  ]
}`,
  },
};

export default function AddBotModal({ isOpen, onClose, onAddBot }: AddBotModalProps) {
  const [rawInput, setRawInput] = useState(TEMPLATES.keyword.code);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState("");

  // Basic visual editor state
  const [visualMode, setVisualMode] = useState(false);
  const [botName, setBotName] = useState("NewBot");
  const [identityType, setIdentityType] = useState("static");
  // Identity-specific fields
  const [staticBotName, setStaticBotName] = useState("HelperBot");
  const [staticAvatarUrl, setStaticAvatarUrl] = useState("");
  const [mimicUserId, setMimicUserId] = useState("");
  const [frequency, setFrequency] = useState(100);
  const [targetAudience, setTargetAudience] = useState<"humans" | "bots">("humans");
  
  const [triggers, setTriggers] = useState<TriggerConfig[]>([
    { name: "Trigger_1", conditions: { contains_phrase: "ping" }, responses: [] }
  ]);
  const [responses, setResponses] = useState<string[]>(["pong"]);
  const [newResponse, setNewResponse] = useState("");

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

  if (!isOpen) return null;

  const handleTemplateSelect = (key: string) => {
    const tmpl = TEMPLATES[key];
    if (tmpl) {
      setRawInput(tmpl.code);
      setErrorMsg("");
    }
  };

  const handleSubmit = async () => {
    setErrorMsg("");
    setIsSubmitting(true);

    try {
      let jsonOutput = rawInput;

      if (visualMode) {
        const identityPayload =
          identityType === "static"
            ? { type: "static", bot_name: staticBotName, avatar_url: staticAvatarUrl }
            : identityType === "mimic"
              ? { type: "mimic", user_id: mimicUserId }
              : { type: identityType };

        const generated = {
          name: botName,
          identity: identityPayload,
          frequency: frequency,
          ignore_bots: targetAudience === "humans",
          ignore_humans: targetAudience === "bots",
          ignore_self: true,
          responses: responses,
          triggers: triggers
        };
        jsonOutput = JSON.stringify(generated, null, 2);
      } else {
        // Validate JSON
        JSON.parse(rawInput);
      }

      await onAddBot(jsonOutput);
      setIsSubmitting(false);
      onClose();
    } catch (err) {
      setIsSubmitting(false);
      setErrorMsg(err instanceof Error ? err.message : "Failed to parse bot input");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="glass-panel max-w-2xl w-full p-6 flex flex-col gap-4 border border-indigo-500/30 shadow-2xl">
        <div className="flex justify-between items-center border-b border-slate-700/50 pb-3">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <span>🤖</span> Add New Reply Bot
            </h2>
            <p className="text-xs text-slate-400">
              Select visual mode or input single sub-bot definition in JSON format.
            </p>
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-white text-lg">✕</button>
        </div>

        {/* Editor Mode Toggle */}
        <div className="flex justify-center mb-2">
          <div className="bg-slate-900 rounded-lg p-1 flex gap-1">
            <button
              onClick={() => setVisualMode(true)}
              className={`px-4 py-1.5 rounded-md text-xs font-semibold transition-colors ${visualMode ? "bg-indigo-600 text-white" : "text-slate-400 hover:text-white"}`}
            >
              Visual Builder
            </button>
            <button
              onClick={() => setVisualMode(false)}
              className={`px-4 py-1.5 rounded-md text-xs font-semibold transition-colors ${!visualMode ? "bg-indigo-600 text-white" : "text-slate-400 hover:text-white"}`}
            >
              JSON Editor
            </button>
          </div>
        </div>

        {visualMode ? (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Bot Name</label>
              <input type="text" value={botName} onChange={e => setBotName(e.target.value)} className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white" />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Identity Profile</label>
              <select value={identityType} onChange={e => setIdentityType(e.target.value)} className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white">
                <option value="static">Custom Profile</option>
                <option value="mimic">Copy Specific User</option>
                <option value="random">Random Server Member</option>
                <option value="mimic_poster">Copy the Message Sender</option>
              </select>
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">Target Audience</label>
              <select value={targetAudience} onChange={e => setTargetAudience(e.target.value as "humans" | "bots")} className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white">
                <option value="humans">Humans (Ignore other bots)</option>
                <option value="bots">Bots (Ignore humans)</option>
              </select>
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
              <>
                <div className="flex flex-col gap-1">
                  <label className="text-xs text-slate-400">Bot Display Name</label>
                  <input
                    type="text"
                    value={staticBotName}
                    onChange={e => setStaticBotName(e.target.value)}
                    className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label className="text-xs text-slate-400">Avatar URL</label>
                  <input
                    type="text"
                    value={staticAvatarUrl}
                    onChange={e => setStaticAvatarUrl(e.target.value)}
                    className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white"
                  />
                </div>
              </>
            )}

            {identityType === "mimic" && (
              <div className="flex flex-col gap-1">
                <label className="text-xs text-slate-400">Discord User ID</label>
                <input
                  type="text"
                  value={mimicUserId}
                  onChange={e => setMimicUserId(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-sm text-white"
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
        ) : (
          <>
            {/* Quick Templates */}
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-1">
                <span className="text-xs text-slate-400">Template:</span>
                {Object.entries(TEMPLATES).map(([key, t]) => (
                  <button
                    key={key}
                    onClick={() => handleTemplateSelect(key)}
                    className="text-xs bg-slate-800/80 hover:bg-slate-700 text-slate-300 px-2 py-1 rounded"
                  >
                    {t.label}
                  </button>
                ))}
              </div>
            </div>

            {/* Text Area Input */}
            <div>
              <textarea
                value={rawInput}
                onChange={(e) => setRawInput(e.target.value)}
                rows={10}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-xs font-mono text-slate-200 focus:outline-none focus:border-accent"
                placeholder={`Enter single sub-bot definition in JSON...`}
              />
            </div>
          </>
        )}

        {errorMsg && (
          <div className="text-xs text-red-400 bg-red-500/10 border border-red-500/20 p-2 rounded">
            {errorMsg}
          </div>
        )}

        <div className="flex justify-end gap-3 pt-2">
          <button onClick={onClose} className="btn-secondary text-xs">Cancel</button>
          <button
            onClick={handleSubmit}
            disabled={isSubmitting || (!visualMode && !rawInput.trim())}
            className="btn-primary text-xs px-5"
          >
            {isSubmitting ? "Adding Bot..." : "Add Sub-Bot to BunkBot"}
          </button>
        </div>
      </div>
    </div>
  );
}
