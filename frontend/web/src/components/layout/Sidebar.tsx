'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  LayoutDashboard,
  ArrowLeftRight,
  Bot,
  Settings,
  Shield,
  Globe,
  FileText,
  X,
} from 'lucide-react';
import { clsx } from 'clsx';

const navItems = [
  { href: '/portfolio', label: 'Portfolio', icon: LayoutDashboard },
  { href: '/trade', label: 'Trade', icon: ArrowLeftRight },
  { href: '/agents', label: 'AI Agents', icon: Bot },
  { href: '/settings', label: 'Settings', icon: Settings },
];

export function Sidebar({
  open,
  onToggle,
}: {
  open: boolean;
  onToggle: () => void;
}) {
  const pathname = usePathname();

  return (
    <aside
      className={clsx(
        'flex flex-col border-r border-slate-800 bg-surface-raised/50 transition-all duration-200',
        open ? 'w-56' : 'w-16'
      )}
    >
      {/* Logo */}
      <div className="flex items-center gap-2 px-4 py-5">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-sky-500 to-blue-600 flex items-center justify-center text-white font-bold text-sm">
          O
        </div>
        {open && (
          <span className="font-semibold text-lg tracking-tight">ODAMP</span>
        )}
      </div>

      {/* Nav */}
      <nav className="flex-1 px-2 space-y-1">
        {navItems.map(({ href, label, icon: Icon }) => (
          <Link
            key={href}
            href={href}
            className={clsx(
              'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors',
              pathname === href
                ? 'bg-sky-500/10 text-sky-400'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
            )}
          >
            <Icon size={18} />
            {open && <span>{label}</span>}
          </Link>
        ))}
      </nav>

      {/* Bottom */}
      <div className="px-2 py-4 space-y-1 border-t border-slate-800">
        <Link
          href="/settings/security"
          className={clsx(
            'flex items-center gap-3 px-3 py-2 rounded-lg text-sm',
            'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          )}
        >
          <Shield size={18} />
          {open && <span>Security</span>}
        </Link>
        <Link
          href="/settings/geopolitical"
          className={clsx(
            'flex items-center gap-3 px-3 py-2 rounded-lg text-sm',
            'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          )}
        >
          <Globe size={18} />
          {open && <span>Geopolitical</span>}
        </Link>
        <Link
          href="/settings/tax"
          className={clsx(
            'flex items-center gap-3 px-3 py-2 rounded-lg text-sm',
            'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          )}
        >
          <FileText size={18} />
          {open && <span>Tax</span>}
        </Link>
      </div>

      {/* Toggle */}
      <button
        onClick={onToggle}
        className="absolute -right-3 top-16 w-6 h-6 rounded-full bg-surface-raised border border-slate-700 flex items-center justify-center text-slate-400 hover:text-white"
      >
        <X size={12} />
      </button>
    </aside>
  );
}   