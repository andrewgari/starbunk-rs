/* eslint-disable react/no-unescaped-entities */
"use client";


// The raw format BunkBot expects
export type RawCondition = Record<string, unknown>;

export interface TriggerConfig {
  name?: string;
  conditions: RawCondition;
  responses?: string[];
}

interface TriggerEditorProps {
  triggers: TriggerConfig[];
  onChange: (newTriggers: TriggerConfig[]) => void;
}

const CONDITION_KEYS = [
  "contains_phrase",
  "contains_word",
  "matches_regex",
  "matches_pattern",
  "from_user",
  "with_chance",
  "always",
  "all_of",
  "any_of",
  "none_of",
];

// Helper to determine the type and value of a raw condition
function getConditionDetails(cond: RawCondition): { key: string; value: unknown } {
  const keys = Object.keys(cond || {});
  if (keys.length === 0) return { key: "always", value: true };
  const key = keys[0];
  return { key, value: cond[key] };
}

function ConditionEditor({
  condition,
  onChange,
  onDelete,
}: {
  condition: RawCondition;
  onChange: (newCond: RawCondition) => void;
  onDelete?: () => void;
}) {
  const { key, value } = getConditionDetails(condition);

  const isCompound = ["all_of", "any_of", "none_of"].includes(key);

  const handleKeyChange = (newKey: string) => {
    const isNewCompound = ["all_of", "any_of", "none_of"].includes(newKey);
    let newValue: unknown = "";
    if (isNewCompound) {
      newValue = isCompound && Array.isArray(value) ? value : [{ contains_phrase: "" }];
    } else if (newKey === "with_chance") {
      newValue = 50;
    } else if (newKey === "always") {
      newValue = true;
    }
    onChange({ [newKey]: newValue });
  };

  const handleValueChange = (newVal: unknown) => {
    onChange({ [key]: newVal });
  };

  const renderValueInput = () => {
    if (isCompound) {
      const children = Array.isArray(value) ? value : [];
      return (
        <div className="pl-4 mt-2 border-l-2 border-indigo-500/50 flex flex-col gap-2">
          {children.map((child: RawCondition, idx: number) => (
            <ConditionEditor
              key={idx}
              condition={child}
              onChange={(updatedChild) => {
                const newChildren = [...children];
                newChildren[idx] = updatedChild;
                handleValueChange(newChildren);
              }}
              onDelete={() => {
                const newChildren = children.filter((_, i) => i !== idx);
                // If only 1 child remains, unwrap it automatically for better UX
                if (newChildren.length === 1) {
                  onChange(newChildren[0]);
                } else if (newChildren.length === 0) {
                  onChange({ always: true });
                } else {
                  handleValueChange(newChildren);
                }
              }}
            />
          ))}
          <button
            onClick={() => handleValueChange([...children, { contains_phrase: "" }])}
            className="text-xs text-indigo-400 hover:text-indigo-300 self-start mt-1 flex items-center gap-1 font-semibold"
          >
            + Add {key === "all_of" ? "AND" : key === "any_of" ? "OR" : "Nested"} Condition
          </button>
        </div>
      );
    }

    if (key === "with_chance") {
      return (
        <div className="flex items-center gap-2 flex-1">
          <input
            type="range"
            min="0"
            max="100"
            value={typeof value === "number" ? value : 50}
            onChange={(e) => handleValueChange(Number(e.target.value))}
            className="flex-1 h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-indigo-500"
          />
          <span className="text-xs text-slate-300 font-mono w-8 text-right">{String(value)}%</span>
        </div>
      );
    }

    if (key === "always") {
      return (
        <select
          value={value ? "true" : "false"}
          onChange={(e) => handleValueChange(e.target.value === "true")}
          className="flex-1 bg-slate-900 border border-slate-700 rounded p-1 text-xs text-white focus:border-indigo-500 outline-none"
        >
          <option value="true">True</option>
          <option value="false">False</option>
        </select>
      );
    }

    return (
      <input
        type="text"
        value={typeof value === "string" || typeof value === "number" ? value : ""}
        onChange={(e) => {
          const val = e.target.value;
          handleValueChange(val);
        }}
        placeholder="Value..."
        className="flex-1 bg-slate-900 border border-slate-700 rounded p-1 text-xs text-white focus:border-indigo-500 outline-none"
      />
    );
  };

  return (
    <div className="flex flex-col bg-slate-800/40 p-2.5 rounded border border-slate-700/60 shadow-sm relative group/condition">
      <div className="flex items-start gap-2">
        <select
          value={key}
          onChange={(e) => handleKeyChange(e.target.value)}
          className="bg-slate-900 border border-slate-700 rounded p-1 text-xs text-indigo-300 font-semibold focus:border-indigo-500 outline-none max-w-[140px]"
        >
          {CONDITION_KEYS.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>
        {!isCompound && renderValueInput()}
        {onDelete && (
          <button
            onClick={onDelete}
            className="text-red-400/50 hover:text-red-400 p-1 ml-auto font-bold opacity-0 group-hover/condition:opacity-100 transition-opacity"
            title="Remove Condition"
          >
            &times;
          </button>
        )}
      </div>
      {isCompound && renderValueInput()}
      
      {!isCompound && (
        <div className="flex gap-2 mt-2 opacity-0 group-hover/condition:opacity-100 transition-opacity h-0 overflow-hidden group-hover/condition:h-auto group-hover/condition:mt-2">
          <button 
            onClick={() => onChange({ all_of: [condition, { contains_phrase: "" }] })}
            className="text-[10px] bg-slate-700 hover:bg-indigo-600 text-slate-300 hover:text-white px-2 py-0.5 rounded transition-colors font-semibold"
          >
            + AND
          </button>
          <button 
            onClick={() => onChange({ any_of: [condition, { contains_phrase: "" }] })}
            className="text-[10px] bg-slate-700 hover:bg-indigo-600 text-slate-300 hover:text-white px-2 py-0.5 rounded transition-colors font-semibold"
          >
            + OR
          </button>
        </div>
      )}
    </div>
  );
}

export default function TriggerEditor({ triggers, onChange }: TriggerEditorProps) {
  const handleAddTrigger = () => {
    onChange([
      ...triggers,
      {
        name: "New Trigger",
        conditions: { contains_phrase: "" },
        responses: [],
      },
    ]);
  };

  const handleRemoveTrigger = (index: number) => {
    onChange(triggers.filter((_, i) => i !== index));
  };

  const handleUpdateTrigger = (index: number, updated: TriggerConfig) => {
    const newTriggers = [...triggers];
    newTriggers[index] = updated;
    onChange(newTriggers);
  };

  const handleAddResponse = (tIndex: number, newResp: string) => {
    if (!newResp.trim()) return;
    const trigger = triggers[tIndex];
    const responses = [...(trigger.responses || []), newResp.trim()];
    handleUpdateTrigger(tIndex, { ...trigger, responses });
  };

  const handleUpdateResponse = (tIndex: number, rIndex: number, newVal: string) => {
    const trigger = triggers[tIndex];
    const responses = [...(trigger.responses || [])];
    responses[rIndex] = newVal;
    handleUpdateTrigger(tIndex, { ...trigger, responses });
  };

  const handleRemoveResponse = (tIndex: number, rIndex: number) => {
    const trigger = triggers[tIndex];
    const responses = (trigger.responses || []).filter((_, i) => i !== rIndex);
    handleUpdateTrigger(tIndex, { ...trigger, responses });
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex justify-between items-center">
        <span className="text-xs text-slate-400 font-medium">Triggers ({triggers.length})</span>
        <button
          onClick={handleAddTrigger}
          className="text-xs bg-indigo-600 hover:bg-indigo-500 text-white px-2 py-1 rounded transition-colors"
        >
          + Add Trigger
        </button>
      </div>

      {triggers.length === 0 && (
        <div className="text-xs text-slate-500 italic p-4 text-center border border-dashed border-slate-700/50 rounded">
          No triggers defined. Bot will never fire unless conditions are added.
        </div>
      )}

      <div className="flex flex-col gap-4 max-h-96 overflow-y-auto pr-1">
        {triggers.map((trigger, idx) => (
          <div
            key={idx}
            className="bg-slate-900/60 border border-slate-700 rounded-lg p-3 flex flex-col gap-3 relative"
          >
            <div className="flex items-center justify-between gap-2">
              <input
                type="text"
                value={trigger.name || ""}
                onChange={(e) =>
                  handleUpdateTrigger(idx, { ...trigger, name: e.target.value })
                }
                placeholder="Trigger Name (optional)"
                className="bg-transparent border-b border-slate-700 px-1 py-0.5 text-sm font-semibold text-white focus:border-indigo-500 outline-none placeholder-slate-600 w-full"
              />
              <button
                onClick={() => handleRemoveTrigger(idx)}
                className="text-xs text-red-400 hover:text-red-300 px-2 py-1 rounded hover:bg-red-500/10 transition-colors whitespace-nowrap"
              >
                Delete
              </button>
            </div>

            {/* Conditions */}
            <div>
              <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-slate-500 uppercase tracking-wide font-bold">
                  Condition Logic
                </span>
                <span className="text-[10px] text-slate-400">
                  Tip: Use <code className="bg-slate-800 px-1 py-0.5 rounded">none_of</code> ➔ <code className="bg-slate-800 px-1 py-0.5 rounded">from_user</code> to block a user.
                </span>
              </div>
              <ConditionEditor
                condition={trigger.conditions || { always: true }}
                onChange={(newCond) =>
                  handleUpdateTrigger(idx, { ...trigger, conditions: newCond })
                }
              />
            </div>

            {/* Responses */}
            <div>
              <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-slate-500 uppercase tracking-wide font-bold">
                  Specific Responses (Overrides Global)
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
              <div className="flex flex-col gap-2">
                {(trigger.responses || []).map((r, rIdx) => (
                  <div key={rIdx} className="flex flex-col bg-slate-950 border border-slate-800 rounded p-1">
                    <div className="flex justify-between items-center bg-slate-900/50 px-2 py-1 border-b border-slate-800">
                      <span className="text-[10px] text-slate-500 font-mono">Response #{rIdx + 1}</span>
                      <button
                        onClick={() => handleRemoveResponse(idx, rIdx)}
                        className="text-red-400 hover:text-red-300 leading-none text-sm font-bold"
                        title="Delete response"
                      >
                        &times;
                      </button>
                    </div>
                    <textarea
                      value={r}
                      onChange={(e) => handleUpdateResponse(idx, rIdx, e.target.value)}
                      className="w-full bg-transparent border-none p-2 text-xs font-mono text-slate-300 focus:outline-none focus:ring-0 resize-y min-h-[60px]"
                      placeholder="Type response..."
                      spellCheck={false}
                    />
                  </div>
                ))}
                
                <div className="flex flex-col gap-1 mt-1">
                  <textarea
                    placeholder="Type new response here..."
                    className="w-full bg-slate-950 border border-slate-800 rounded px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500 resize-y min-h-[60px]"
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        const val = (e.target as HTMLTextAreaElement).value;
                        if (val.trim()) {
                          handleAddResponse(idx, val);
                          (e.target as HTMLTextAreaElement).value = "";
                        }
                      }
                    }}
                  />
                  <span className="text-[10px] text-slate-500">Press Enter to add, Shift+Enter for new line</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
