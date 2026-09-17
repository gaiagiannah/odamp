'use client';

import { Bell, Search, User, Zap } from 'lucide-react';
import { useState } from 'react';

export function TopBar() {
  const [searchOpen, setSearchOpen] = useState(false);

  return (
    <header className="h-14 border-b border-slate-800 flex items-center justify-between px-6 bg-surface/80 backdrop-blur-sm">
      {/* Left: Search */}
      <div className="flex items-center gap-3">
        <button
          onClick={() => setSearchOpen(!searchOpen)}
          className="p-2 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white"
        >
          <Search size={18} />
        </button>
        {searchOpen && (
          <input
            autoFocus
            type="text"
            placeholder="Search assets, protocols, agents..."
            className="bg-slate-800 border border-slate-700 rounded-lg px-3 py-1.5 text-sm w-72 focus:outline-none focus:ring-1 focus:ring-sky-500"
            onBlur={() => setSearchOpen(false)}
          />
        )}
      </div>

      {/* Right: Status + Notifications + User */}
      <div className="flex items-center gap-4">
        {/* Network status */}
        <div className="flex items-center gap-1.5 text-xs text-emerald-400">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
          <span>Connected</span>
        </div>

        {/* Kill switch indicator */}
        <button className="p-2 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-red-400" title="Kill Switch">
          <Zap size={18} />
        </button>

        {/* Notifications */}
        <button className="relative p-2 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white">
          <Bell size={18} />
          <span className="absolute top-1 right-1 w-2 h-2 rounded-full bg-red-500" />
        </button>

        {/* User */}
        <button className="flex items-center gap-2 p-1.5 rounded-lg hover:bg-slate-800">
          <div className="w-7 h-7 rounded-full bg-gradient-to-br from-sky-400 to-blue-600 flex items-center justify-center text-xs font-bold">
            U
          </div>
        </button>
      </div>
    </header>
  );
}   