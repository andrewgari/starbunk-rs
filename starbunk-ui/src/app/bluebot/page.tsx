export default function BlueBotLanding() {
  return (
    <div className="flex flex-col h-full gap-4 max-w-5xl mx-auto py-6">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">BlueBot</h1>
        <p className="text-slate-400 mt-1">Pattern-matches blue mage references.</p>
      </header>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="glass-panel p-6">
          <h2 className="text-xl font-semibold mb-4 text-white">Core Essentials</h2>
          <p className="text-slate-400">Status: Running</p>
          <p className="text-slate-400">Spells Learned: --</p>
        </div>
      </div>
    </div>
  );
}
