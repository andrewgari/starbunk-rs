"use client";

import { useState, useEffect, useTransition } from "react";
import { getBotDeployments, setBotState, BotDeploymentStatus } from "./actions";

export default function Dashboard() {
  const [bots, setBots] = useState<BotDeploymentStatus[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isPending, startTransition] = useTransition();

  const load = async () => {
    setIsLoading(true);
    const data = await getBotDeployments();
    setBots(data);
    setIsLoading(false);
  };

  useEffect(() => {
    let active = true;
    (async () => { if (active) await load(); })();
    const interval = setInterval(() => { if (active) void load(); }, 5000);
    return () => { active = false; clearInterval(interval); };
  }, []);

  const handleAction = (botName: string, action: "start" | "stop" | "restart") => {
    startTransition(async () => {
      await setBotState(botName, action);
      await load();
    });
  };

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-slate-400 mt-1">Lifecycle management and overview of all bots.</p>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-6">
        {isLoading && bots.length === 0 ? (
          <div className="col-span-full flex items-center justify-center p-12">
            <div className="animate-pulse flex flex-col items-center gap-3">
              <div className="h-8 w-8 rounded-full border-2 border-accent border-t-transparent animate-spin"></div>
              <div className="text-slate-400">Loading bot statuses...</div>
            </div>
          </div>
        ) : (
          bots.map((bot) => (
            <div key={bot.name} className="glass-panel p-6 flex flex-col h-full">
              <div className="flex justify-between items-start mb-4">
                <h2 className="text-xl font-bold capitalize text-white flex items-center gap-3">
                  <span className={`w-3 h-3 rounded-full ${bot.status === 'Running' ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse' : bot.status === 'Stopped' ? 'bg-slate-500' : 'bg-yellow-500'}`}></span>
                  {bot.name}
                </h2>
                <div className={`px-2 py-1 text-xs font-semibold rounded-md ${bot.status === 'Running' ? 'text-green-400 bg-green-400/10 border border-green-400/20' : bot.status === 'Stopped' ? 'text-slate-400 bg-slate-400/10 border border-slate-400/20' : 'text-yellow-400 bg-yellow-400/10 border border-yellow-400/20'}`}>
                  {bot.status}
                </div>
              </div>

              <div className="text-sm text-slate-400 mb-6 flex-1">
                Replicas: {bot.readyReplicas} / {bot.replicas}
              </div>

              <div className="flex gap-2 mt-auto">
                {bot.status === 'Stopped' ? (
                  <button 
                    className="flex-1 btn-primary bg-green-600 hover:bg-green-500 border-none text-white" 
                    onClick={() => handleAction(bot.name, 'start')}
                    disabled={isPending}
                  >
                    Start
                  </button>
                ) : (
                  <button 
                    className="flex-1 btn-secondary text-slate-300 hover:bg-red-500/20 hover:text-red-400 transition-colors" 
                    onClick={() => handleAction(bot.name, 'stop')}
                    disabled={isPending}
                  >
                    Stop
                  </button>
                )}
                
                <button 
                  className="flex-1 btn-secondary" 
                  onClick={() => handleAction(bot.name, 'restart')}
                  disabled={isPending || bot.status === 'Stopped' || bot.status === 'Unknown'}
                >
                  Restart
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
