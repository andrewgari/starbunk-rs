"use client";

import { useState, useEffect, useTransition } from "react";
import { getDjcovaState, skipTrack, kickBot, DjcovaState } from "../actions";

export default function DJCovaPage() {
  const [state, setState] = useState<DjcovaState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isPending, startTransition] = useTransition();

  const load = async () => {
    setIsLoading(true);
    const data = await getDjcovaState();
    setState(data);
    setIsLoading(false);
  };

  useEffect(() => {
    load();
    const interval = setInterval(load, 10000); // auto refresh every 10s
    return () => clearInterval(interval);
  }, []);

  const handleSkip = (guildId: number) => {
    startTransition(async () => {
      await skipTrack(guildId);
      await load();
    });
  };

  const handleKick = (guildId: number) => {
    startTransition(async () => {
      await kickBot(guildId);
      await load();
    });
  };

  const formatDuration = (secs?: number) => {
    if (!secs) return "--:--";
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  };

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="flex justify-between items-end mb-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">DJCova Controls</h1>
          <p className="text-slate-400 mt-1">Monitor queues, skip tracks, and kick the bot from voice channels.</p>
        </div>
        <button className="btn-secondary" onClick={() => startTransition(load)} disabled={isPending || isLoading}>
          Refresh
        </button>
      </header>

      {isLoading && !state ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="animate-pulse flex flex-col items-center gap-3">
            <div className="h-8 w-8 rounded-full border-2 border-accent border-t-transparent animate-spin"></div>
            <div className="text-slate-400">Loading DJCova state...</div>
          </div>
        </div>
      ) : !state || state.guilds.length === 0 ? (
        <div className="glass-panel p-12 text-center text-slate-400">
          <svg className="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"></path></svg>
          <p className="text-lg">DJCova is not currently active in any voice channels.</p>
        </div>
      ) : (
        <div className="grid gap-6">
          {state.guilds.map((guild) => (
            <div key={guild.guild_id} className="glass-panel p-6">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h2 className="text-xl font-bold text-white flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-green-500 animate-pulse"></span>
                    Guild {guild.guild_id}
                  </h2>
                  <div className="flex gap-4 mt-2 text-sm text-slate-400">
                    <span>Volume: {guild.volume}%</span>
                    <span>•</span>
                    <span>Repeat: {guild.repeat_mode}</span>
                    <span>•</span>
                    <span>Queue: {guild.queue_length} tracks</span>
                  </div>
                </div>
                <div className="flex gap-2">
                  <button 
                    className="btn-secondary" 
                    onClick={() => handleSkip(guild.guild_id)}
                    disabled={isPending}
                  >
                    Skip
                  </button>
                  <button 
                    className="bg-red-500/20 hover:bg-red-500/30 text-red-400 px-4 py-2 rounded-lg font-medium transition-colors" 
                    onClick={() => handleKick(guild.guild_id)}
                    disabled={isPending}
                  >
                    Kick
                  </button>
                </div>
              </div>

              {guild.current_track ? (
                <div className="bg-slate-900/50 rounded-lg p-4 border border-white/5 flex justify-between items-center">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 bg-accent/20 text-accent rounded-lg flex items-center justify-center shrink-0">
                      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                    </div>
                    <div className="overflow-hidden">
                      <div className="font-semibold text-white truncate">{guild.current_track.title}</div>
                      <div className="text-sm text-slate-400 truncate">Requested by {guild.current_track.requester}</div>
                    </div>
                  </div>
                  <div className="text-sm font-medium text-slate-300 ml-4">
                    {formatDuration(guild.current_track.duration_secs)}
                  </div>
                </div>
              ) : (
                <div className="bg-slate-900/50 rounded-lg p-4 border border-white/5 text-slate-500 text-center italic">
                  Nothing currently playing.
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
