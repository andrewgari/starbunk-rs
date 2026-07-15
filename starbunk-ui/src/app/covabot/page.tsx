import { getBotConfigs } from "../actions";
import ConfigManager from "@/components/ConfigManager";

export default async function CovaBotLanding() {
  const configs = await getBotConfigs("covabot");

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">CovaBot</h1>
        <p className="text-slate-400 mt-1">Personality emulation and AI integration.</p>
      </header>
      
      <div className="mb-8 grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="glass-panel p-6">
          <h2 className="text-xl font-semibold mb-4 text-white">Personality Layers</h2>
          <p className="text-slate-400 mb-2">Create and edit individual layers that compose CovaBot's personality.</p>
          <p className="text-xs text-indigo-300">Note: Changes will be applied on the next bot restart.</p>
        </div>
      </div>

      <ConfigManager configs={configs} botName="covabot" />
    </div>
  );
}
