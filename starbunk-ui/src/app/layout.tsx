import type { Metadata } from "next";
import { Inter } from "next/font/google";
import Link from "next/link";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "StarBunk Bot Management",
  description: "Centralized control for all StarBunk bots.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.className} flex h-screen overflow-hidden`}>
        {/* Sidebar */}
        <aside className="w-64 flex-shrink-0 glass-panel m-4 flex flex-col p-4 shadow-xl">
          <div className="text-2xl font-bold mb-8 text-white tracking-tight">StarBunk</div>
          <nav className="flex flex-col gap-1">
            <Link href="/" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">Dashboard</Link>
            <Link href="/history" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">History & Audit</Link>
            
            <div className="mt-4 mb-1 px-2 text-xs font-semibold text-slate-500 uppercase tracking-wider">Bots</div>
            
            <Link href="/covabot" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">CovaBot</Link>
            <Link href="/covabot/personalities" className="p-2 ml-4 rounded-lg hover:bg-white/10 transition-colors text-sm text-slate-400 hover:text-white flex items-center gap-2">
              <div className="w-1 h-1 rounded-full bg-slate-600"></div>Personalities
            </Link>

            <Link href="/bunkbot" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">BunkBot</Link>
            <Link href="/bunkbot/strategies" className="p-2 ml-4 rounded-lg hover:bg-white/10 transition-colors text-sm text-slate-400 hover:text-white flex items-center gap-2">
              <div className="w-1 h-1 rounded-full bg-slate-600"></div>Strategies
            </Link>

            <Link href="/djcova" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">DJCova</Link>
            <Link href="/djcova/controls" className="p-2 ml-4 rounded-lg hover:bg-white/10 transition-colors text-sm text-slate-400 hover:text-white flex items-center gap-2">
              <div className="w-1 h-1 rounded-full bg-slate-600"></div>Controls
            </Link>

            <Link href="/bluebot" className="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-300 hover:text-white">BlueBot</Link>
          </nav>
        </aside>
        
        {/* Main Content */}
        <main className="flex-1 p-4 overflow-y-auto">
          {children}
        </main>
      </body>
    </html>
  );
}
