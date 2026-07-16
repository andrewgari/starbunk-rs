"use client";

import { useState } from "react";
import { updateBotConfig } from "@/app/actions";

export default function ConfigManager({ configs, botName }: { configs: Record<string, string>, botName: "bunkbot" | "covabot" }) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [content, setContent] = useState<string>("");
  const [isSaving, setIsSaving] = useState(false);
  const [newFileName, setNewFileName] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  const handleSelect = (filename: string) => {
    setSelectedFile(filename);
    setContent(configs[filename]);
    setIsCreating(false);
  };

  const handleCreateNew = () => {
    setIsCreating(true);
    setSelectedFile(null);
    setContent("");
    setNewFileName("");
  };

  const handleSave = async () => {
    const targetFile = isCreating ? newFileName : selectedFile;
    if (!targetFile) return;
    
    // Ensure it ends with .yml
    const finalFilename = targetFile.endsWith(".yml") || targetFile.endsWith(".yaml") ? targetFile : `${targetFile}.yml`;

    setIsSaving(true);
    const res = await updateBotConfig(botName, finalFilename, content);
    setIsSaving(false);
    
    if (res.success) {
      if (isCreating) {
        setIsCreating(false);
        setSelectedFile(finalFilename);
      }
    } else {
      alert(`Error saving: ${res.error}`);
    }
  };

  const handleDelete = async () => {
    if (!selectedFile) return;
    if (!confirm(`Are you sure you want to delete ${selectedFile}?`)) return;
    
    setIsSaving(true);
    const res = await updateBotConfig(botName, selectedFile, null);
    setIsSaving(false);
    
    if (res.success) {
      setSelectedFile(null);
    } else {
      alert(`Error deleting: ${res.error}`);
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
      <div className="md:col-span-1 flex flex-col gap-2">
        <div className="glass-panel p-4 flex flex-col gap-2">
          <h3 className="text-lg font-semibold text-white mb-2">Configuration Files</h3>
          {Object.keys(configs).map((filename) => (
            <button
              key={filename}
              onClick={() => handleSelect(filename)}
              className={`text-left px-3 py-2 rounded-md transition-colors ${selectedFile === filename ? "bg-indigo-500/20 border border-indigo-500/50 text-indigo-200" : "hover:bg-white/5 text-slate-300"}`}
            >
              {filename}
            </button>
          ))}
          <button
            onClick={handleCreateNew}
            className="mt-4 flex items-center justify-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white py-2 px-4 rounded-md transition-colors text-sm font-medium"
          >
            + Create New
          </button>
        </div>
      </div>
      
      <div className="md:col-span-3">
        {(selectedFile || isCreating) ? (
          <div className="glass-panel p-6 flex flex-col h-[600px]">
            <div className="flex justify-between items-center mb-4">
              {isCreating ? (
                <input 
                  type="text" 
                  value={newFileName}
                  onChange={(e) => setNewFileName(e.target.value)}
                  placeholder="filename.yml"
                  className="bg-black/20 border border-white/10 rounded-md px-3 py-1.5 text-white focus:outline-none focus:border-indigo-500"
                />
              ) : (
                <h3 className="text-xl font-semibold text-white">{selectedFile}</h3>
              )}
              
              <div className="flex gap-3">
                {!isCreating && (
                  <button 
                    onClick={handleDelete}
                    disabled={isSaving}
                    className="text-red-400 hover:text-red-300 px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
                  >
                    Delete
                  </button>
                )}
                <button 
                  onClick={handleSave}
                  disabled={isSaving || (isCreating && !newFileName.trim())}
                  className="bg-indigo-600 hover:bg-indigo-500 text-white px-4 py-1.5 rounded-md text-sm font-medium transition-colors shadow-lg shadow-indigo-500/20 disabled:opacity-50"
                >
                  {isSaving ? "Saving..." : "Save Changes"}
                </button>
              </div>
            </div>
            
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="flex-1 w-full bg-black/40 border border-white/10 rounded-lg p-4 text-emerald-300 font-mono text-sm focus:outline-none focus:border-indigo-500/50 resize-none"
              placeholder="Enter YAML configuration here..."
              spellCheck={false}
            />
          </div>
        ) : (
          <div className="glass-panel p-12 flex flex-col items-center justify-center h-full text-slate-400">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-16 w-16 mb-4 opacity-20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p>Select a configuration file to edit or create a new one.</p>
          </div>
        )}
      </div>
    </div>
  );
}
