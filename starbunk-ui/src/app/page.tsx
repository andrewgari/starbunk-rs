"use client";

import { useState, useEffect } from "react";
import { getBotStatuses, BotStatus } from "./actions";

function statusClasses(status: BotStatus["status"]) {
  switch (status) {
    case "Running":
      return {
        dot: "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse",
        badge: "text-green-400 bg-green-400/10 border border-green-400/20",
      };
    case "Down":
      return {
        dot: "bg-red-500",
        badge: "text-red-400 bg-red-400/10 border border-red-400/20",
      };
    case "Starting":
      return {
        dot: "bg-yellow-500",
        badge: "text-yellow-400 bg-yellow-400/10 border border-yellow-400/20",
      };
    default:
      return {
        dot: "bg-slate-500",
        badge: "text-slate-400 bg-slate-400/10 border border-slate-400/20",
      };
  }
}

export default function Dashboard() {
  const [bots, setBots] = useState<BotStatus[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let active = true;
    const load = async () => {
      const data = await getBotStatuses();
      if (!active) return;
      setBots(data);
      setIsLoading(false);
    };
    void load();
    const interval = setInterval(() => void load(), 5000);
    return () => { active = false; clearInterval(interval); };
  }, []);

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-slate-400 mt-1">
          Status overview of all bots, probed via each bot&apos;s HTTP API. Start, stop and restart
          are handled by the Docker Compose stack on Tower (Portainer).
        </p>
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
          bots.map((bot) => {
            const cls = statusClasses(bot.status);
            return (
              <div key={bot.name} className="glass-panel p-6 flex flex-col h-full">
                <div className="flex justify-between items-start mb-4">
                  <h2 className="text-xl font-bold capitalize text-white flex items-center gap-3">
                    <span className={`w-3 h-3 rounded-full ${cls.dot}`}></span>
                    {bot.name}
                  </h2>
                  <div className={`px-2 py-1 text-xs font-semibold rounded-md ${cls.badge}`}>
                    {bot.status}
                  </div>
                </div>

                <div className="text-sm text-slate-400 flex-1 break-all">{bot.detail}</div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
