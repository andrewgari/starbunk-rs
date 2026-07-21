"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { useSSE } from "@/components/useSSE";

export default function DJCovaLanding() {
  const { isConnected } = useSSE();
  const [isPlaying, setIsPlaying] = useState(true);
  const [currentTrack, setCurrentTrack] = useState({
    title: "Lofi Hip Hop Radio - Beats to Relax/Study to",
    uploader: "Lofi Girl",
    duration: "3:45",
    progressMs: 145000,
    totalMs: 225000,
    guildName: "StarBunk Hangout",
    channelName: "Music Lounge #1",
  });

  const [queue, setQueue] = useState([
    { id: 1, title: "Chocobo Theme (Remix)", duration: "2:30", requestedBy: "Andrew" },
    { id: 2, title: "Blue Mage Battle Theme", duration: "4:15", requestedBy: "BluFan" },
    { id: 3, title: "Synthwave Dreams", duration: "3:10", requestedBy: "Cova" },
  ]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (isPlaying) {
        setCurrentTrack((prev) => ({
          ...prev,
          progressMs: Math.min(prev.progressMs + 1000, prev.totalMs),
        }));
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [isPlaying]);

  const progressPercent = Math.round((currentTrack.progressMs / currentTrack.totalMs) * 100);

  return (
    <div className="flex flex-col h-full gap-6 max-w-5xl mx-auto py-6">
      <header className="flex justify-between items-start">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>DJCova 🎵</span>
            <span className="text-xs px-2.5 py-1 rounded bg-violet-500/20 text-violet-300 border border-violet-500/30 uppercase font-mono tracking-wide">
              Music Streaming HUD
            </span>
          </h1>
          <p className="text-slate-400 mt-1">
            Voice channel YouTube music streaming service with live queue &amp; playback HUD.
          </p>
        </div>

        <Link href="/djcova/controls" className="btn-primary flex items-center gap-2">
          Open Controls &rarr;
        </Link>
      </header>

      {/* Real-time Streaming HUD */}
      <div className="glass-panel p-6 border-violet-500/30 flex flex-col gap-6">
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-violet-500 animate-pulse shadow-[0_0_8px_rgba(139,92,246,0.6)]"></span>
            <span className="text-sm font-semibold text-white">Live Voice Connection</span>
            <span className="text-xs text-slate-400">({currentTrack.guildName} • {currentTrack.channelName})</span>
          </div>

          <div className="flex items-center gap-2">
            <span className={`w-2 h-2 rounded-full ${isConnected ? "bg-emerald-500" : "bg-slate-500"}`} />
            <span className="text-xs text-slate-400 font-mono">{isConnected ? "SSE Telemetry Active" : "Polling Active"}</span>
          </div>
        </div>

        {/* Now Playing Widget */}
        <div className="bg-slate-950/80 p-5 rounded-xl border border-slate-800 flex flex-col gap-4">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-lg bg-gradient-to-tr from-violet-600 to-indigo-600 flex items-center justify-center text-2xl font-bold text-white shadow-lg">
              🎵
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-xs font-semibold text-violet-400 uppercase tracking-wider mb-0.5">Now Playing</div>
              <h3 className="text-lg font-bold text-white truncate">{currentTrack.title}</h3>
              <div className="text-xs text-slate-400">{currentTrack.uploader}</div>
            </div>
          </div>

          {/* Track Progress Bar */}
          <div className="flex flex-col gap-1.5">
            <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden">
              <div
                className="bg-violet-500 h-full transition-all duration-300"
                style={{ width: `${progressPercent}%` }}
              />
            </div>
            <div className="flex justify-between text-xs text-slate-500 font-mono">
              <span>{Math.floor(currentTrack.progressMs / 60000)}:{String(Math.floor((currentTrack.progressMs % 60000) / 1000)).padStart(2, "0")}</span>
              <span>{currentTrack.duration}</span>
            </div>
          </div>

          {/* Quick HUD Playback Buttons */}
          <div className="flex items-center justify-center gap-4 pt-1">
            <button
              onClick={() => setIsPlaying(!isPlaying)}
              className="btn-primary bg-violet-600 hover:bg-violet-500 px-6"
            >
              {isPlaying ? "Pause" : "Play"}
            </button>
            <button
              onClick={() => {
                if (queue.length > 0) {
                  const [next, ...rest] = queue;
                  setCurrentTrack({
                    ...currentTrack,
                    title: next.title,
                    uploader: `Requested by ${next.requestedBy}`,
                    duration: next.duration,
                    progressMs: 0,
                  });
                  setQueue(rest);
                }
              }}
              className="btn-secondary text-xs"
            >
              Skip Track ⏭
            </button>
          </div>
        </div>
      </div>

      {/* Up Next Queue HUD */}
      <div className="glass-panel p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold text-white">Queued Tracks ({queue.length})</h2>
          <span className="text-xs text-slate-400">Voice channel audio buffer</span>
        </div>

        <div className="flex flex-col gap-2">
          {queue.length === 0 ? (
            <div className="text-slate-500 text-xs text-center py-4">Queue is empty.</div>
          ) : (
            queue.map((item, idx) => (
              <div key={item.id} className="flex items-center justify-between p-3 rounded bg-slate-900/60 border border-slate-800 text-xs">
                <div className="flex items-center gap-3">
                  <span className="text-slate-500 font-mono font-bold">#{idx + 1}</span>
                  <span className="text-white font-medium">{item.title}</span>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-slate-400">By {item.requestedBy}</span>
                  <span className="text-slate-500 font-mono">{item.duration}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
