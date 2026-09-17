'use client';

import { AlertTriangle, AlertCircle, Info, ShieldAlert } from 'lucide-react';
import { clsx } from 'clsx';

interface Alert {
  severity: 'info' | 'warning' | 'critical' | 'emergency';
  message: string;
  source: string;
}

const MOCK_ALERTS: Alert[] = [
  { severity: 'warning', message: 'BTC concentration at 42% (threshold: 40%)', source: 'PortfolioManager' },
  { severity: 'info', message: 'Aave V3 APY dropped to 4.2% (was 5.1%)', source: 'YieldOptimizer' },
];

export function AgentAlerts() {
  if (MOCK_ALERTS.length === 0) return null;

  return (
    <div className="space-y-2">
      {MOCK_ALERTS.map((alert, i) => (
        <div
          key={i}
          className={clsx(
            'flex items-center gap-3 p-3 rounded-lg border text-sm',
            alert.severity === 'emergency' && 'bg-red-900/20 border-red-800/50 text-red-300',
            alert.severity === 'critical' && 'bg-red-900/10 border-red-800/30 text-red-400',
            alert.severity === 'warning' && 'bg-amber-900/10 border-amber-800/30 text-amber-300',
            alert.severity === 'info' && 'bg-sky-900/10 border-sky-800/30 text-sky-300'
          )}
        >
          {alert.severity === 'emergency' && <ShieldAlert size={16} />}
          {alert.severity === 'critical' && <AlertTriangle size={16} />}
          {alert.severity === 'warning' && <AlertCircle size={16} />}
          {alert.severity === 'info' && <Info size={16} />}
          <span className="flex-1">{alert.message}</span>
          <span className="text-xs opacity-60">{alert.source}</span>
        </div>
      ))}
    </div>
  );
}   