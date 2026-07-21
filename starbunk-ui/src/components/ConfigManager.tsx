"use client";

import { useState } from "react";
import { updateBotConfig } from "@/app/actions";

export default function ConfigManager({ configs, botName }: { configs: Record<string, string>, botName: string }) {
  const fileKeys = Object.keys(configs);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [content, setContent] = useState<string>("");
  const [isSaving, setIsSaving] = useState(false);
  const [newFileName, setNewFileName] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  // Determine currently active file name
  const activeFile = isCreating
    ? null
    : selectedFile && configs[selectedFile] !== undefined
    ? selectedFile
    : fileKeys[0] || null;

  // React recommended pattern: Adjust state during render when props/selection change
  const [syncedKey, setSyncedKey] = useState<string | null>(null);
  const currentKey = `${activeFile}:${configs[activeFile || ""]}`;
  if (syncedKey !== currentKey && !isCreating) {
    setSyncedKey(currentKey);
    if (activeFile) {
      setContent(configs[activeFile] || "");
    }
  }

  const handleSelect = (filename: string) => {
    setSelectedFile(filename);
    setContent(configs[filename] || "");
    setIsCreating(false);
  };

  const handleCreateNew = () => {
    setIsCreating(true);
    setSelectedFile(null);
    setNewFileName("");
    setContent("");
  };

  const handleSave = async () => {
    const filename = isCreating ? newFileName.trim() : activeFile;
    if (!filename) return;

    const finalFileName = filename.endsWith(".yml") || filename.endsWith(".yaml")
      ? filename
      : `${filename}.yml`;

    setIsSaving(true);
    const res = await updateBotConfig(botName, finalFileName, content);
    setIsSaving(false);

    if (res.success) {
      setIsCreating(false);
      setSelectedFile(finalFileName);
    } else {
      alert(`Error saving config: ${res.error}`);
    }
  };

  const handleDelete = async () => {
    if (!activeFile) return;
    if (!confirm(`Are you sure you want to delete ${activeFile}?`)) return;

    setIsSaving(true);
    const res = await updateBotConfig(botName, activeFile, null);
    setIsSaving(false);

    if (res.success) {
      setSelectedFile(null);
    } else {
      alert(`Error deleting config: ${res.error}`);
    }
  };

  return (
    <div className="glass-panel p-6">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold text-white capitalize">{botName} Configurations</h2>
        <button
          onClick={handleCreateNew}
          className="btn-secondary text-xs px-3 py-1.5"
        >
          + New File
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        {/* Sidebar file list */}
        <div className="flex flex-col gap-1 border-r border-slate-700/50 pr-4">
          <div className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2">Files</div>
          {fileKeys.length === 0 && !isCreating && (
            <div className="text-xs text-slate-500 py-2">No config files found.</div>
          )}
          {fileKeys.map((file) => (
            <button
              key={file}
              onClick={() => handleSelect(file)}
              className={`text-left text-sm px-3 py-2 rounded-lg transition-colors truncate ${
                !isCreating && activeFile === file
                  ? "bg-accent/20 text-white font-medium border border-accent/30"
                  : "text-slate-400 hover:text-white hover:bg-slate-800/50"
              }`}
            >
              {file}
            </button>
          ))}
          {isCreating && (
            <div className="text-left text-sm px-3 py-2 rounded-lg bg-indigo-500/20 text-indigo-300 font-medium border border-indigo-500/30">
              New File...
            </div>
          )}
        </div>

        {/* Editor main panel */}
        <div className="md:col-span-3 flex flex-col gap-4">
          {isCreating ? (
            <div>
              <label className="block text-xs text-slate-400 mb-1">Filename</label>
              <input
                type="text"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                placeholder="e.g. custom_settings.yml"
                className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-accent"
              />
            </div>
          ) : activeFile ? (
            <div className="flex justify-between items-center">
              <span className="text-sm font-mono text-indigo-300 font-semibold">{activeFile}</span>
              <button
                onClick={handleDelete}
                disabled={isSaving}
                className="text-xs text-red-400 hover:text-red-300 hover:bg-red-500/10 px-2 py-1 rounded transition-colors"
              >
                Delete
              </button>
            </div>
          ) : null}

          {(activeFile || isCreating) ? (
            <>
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={12}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-xs font-mono text-slate-200 focus:outline-none focus:border-accent resize-y"
                placeholder="Enter YAML configuration here..."
              />

              <div className="flex justify-end gap-2">
                {isCreating && (
                  <button
                    onClick={() => setIsCreating(false)}
                    className="btn-secondary text-xs"
                  >
                    Cancel
                  </button>
                )}
                <button
                  onClick={handleSave}
                  disabled={isSaving}
                  className="btn-primary text-xs px-4"
                >
                  {isSaving ? "Saving..." : "Save Config"}
                </button>
              </div>
            </>
          ) : (
            <div className="text-sm text-slate-500 p-8 text-center border border-dashed border-slate-800 rounded-lg">
              Select a file on the left or create a new one to edit.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
