'use client';

import { useState } from 'react';
import { PortfolioSummary } from './PortfolioSummary';
import { AllocationChart } from './AllocationChart';
import { PositionsTable } from './PositionsTable';
import { RiskPanel } from './RiskPanel';
import { TaxExposureCard } from './TaxExposureCard';
import { AgentAlerts } from './AgentAlerts';
import { usePortfolio } from './usePortfolio';

export function PortfolioDashboard() {
  const { data, isLoading, error } = usePortfolio();
  const [timeframe, setTimeframe] = useState<'24h' | '7d' | '30d' | '1y'>('30d');

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-400">Loading portfolio...</div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-red-400">Failed to load portfolio. {error}</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Portfolio</h1>
          <p className="text-sm text-slate-400 mt-1">
            Last updated: {new Date(data.generated_at).toLocaleString()}
          </p>
        </div>
        <div className="flex gap-1">
          {(['24h', '7d', '30d', '1y'] as const).map((tf) => (
            <button
              key={tf}
              onClick={() => setTimeframe(tf)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                timeframe === tf
                  ? 'bg-sky-500/20 text-sky-400'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              {tf}
            </button>
          ))}
        </div>
      </div>

      {/* Alerts */}
      <AgentAlerts />

      {/* Summary Cards */}
      <PortfolioSummary data={data} timeframe={timeframe} />

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <AllocationChart data={data} />
          <PositionsTable positions={data.top_positions} />
        </div>
        <div className="space-y-6">
          <RiskPanel risk={data.risk} />
          <TaxExposureCard exposure={data.tax_exposure_usd} />
        </div>
      </div>
    </div>
  );
}   