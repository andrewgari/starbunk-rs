import { getDjcovaState } from "./actions";
import Link from "next/link";

export default async function DJCovaLanding() {
  const state = await getDjcovaState();
  const activeGuilds = state?.guilds.length ?? 0;
  const totalTracks = state?.guilds.reduce((sum, g) => sum + g.queue_length, 0) ?? 0;

  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">DJCova</h1>
        <p className="text-slate-400 mt-1">Voice channel music streaming bot.</p>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Status</div>
          <div className="flex items-center gap-2">
            <span className={`w-3 h-3 rounded-full ${state ? 'bg-green-500 animate-pulse' : 'bg-slate-500'}`}></span>
            <span className="text-xl font-bold text-white">{state ? "Online" : "Offline"}</span>
          </div>
        </div>
        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Active Guilds</div>
          <div className="text-3xl font-bold text-white">{activeGuilds}</div>
        </div>
        <div className="glass-panel p-6">
          <div className="text-sm font-medium text-slate-400 mb-1">Queued Tracks</div>
          <div className="text-3xl font-bold text-white">{totalTracks}</div>
        </div>
      </div>

      <div className="glass-panel p-6 mt-2">
        <h2 className="text-xl font-semibold mb-4 text-white">Quick Navigation</h2>
        <Link
          href="/djcova/controls"
          className="inline-flex items-center gap-2 btn-primary"
        >
          Open Controls
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </Link>
        {/* TODO(human): Should this landing page auto-refresh the stats, or is a static
            server-rendered snapshot with a manual "Refresh" button enough? The /controls
            page already polls every 10s. Consider whether duplicating that here adds value. */}
      </div>
    </div>
  );
}
