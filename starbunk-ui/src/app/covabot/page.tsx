import { getBotConfigs } from "../actions";
import ConfigManager from "@/components/ConfigManager";
import PersonalityStudio from "@/components/PersonalityStudio";
import Link from "next/link";

export const dynamic = "force-dynamic";

export default async function CovaBotLanding() {
  const configs = await getBotConfigs("covabot");

  return (
    <div className="flex flex-col h-full gap-6 max-w-6xl mx-auto py-6">
      <header className="flex justify-between items-start">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>CovaBot 🧠</span>
            <span className="text-xs px-2.5 py-1 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 uppercase font-mono tracking-wide">
              Personality Engine
            </span>
          </h1>
          <p className="text-slate-400 mt-1">
            AI personality emulator, model tier routing, user stance matrix, and social battery orchestration.
          </p>
        </div>

        <Link href="/covabot/personalities" className="btn-secondary text-xs">
          Raw Profile Editor &rarr;
        </Link>
      </header>

      {/* Personality Studio Component */}
      <PersonalityStudio />

      {/* Config Management */}
      <ConfigManager configs={configs} botName="covabot" />
    </div>
  );
}
