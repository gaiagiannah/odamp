'use client';

import { TrendingUp, TrendingDown, DollarSign, Percent } from 'lucide-react';
import { clsx } from 'clsx';
import type { PortfolioData } from './usePortfolio';

export function PortfolioSummary({
  data,
  timeframe,
}: {
  data: PortfolioData;
  timeframe: string;
}) {
  const gain = data.total_unrealized_gain_usd;
  const isPositive = gain >= 0;

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {/* Total Value */}
      <div className="card">
        <div className="flex items-center gap-2 text-sm text-slate-400">
          <DollarSign size={16} />
          <span>Total Value</span>
        </div>
        <p className="text-2xl font-bold mt-2">
          ${data.total_value_usd.toLocaleString('en-US', { maximumFractionDigits: 2 })}
        </p>
      </div>

      {/* Unrealized Gain */}
      <div className="card">
        <div className="flex items-center gap-2 text-sm text-slate-400">
          {isPositive ? <TrendingUp size={16} className="text-emerald-400" /> : <TrendingDown size={16} className="text-red-400" />}
          <span>Unrealized Gain</span>
        </div>
        <p className={clsx('text-2xl font-bold mt-2', isPositive ? 'text-emerald-400' : 'text-red-400')}>
          {isPositive ? '+' : ''}
          {gain.toLocaleString('en-US', { maximumFractionDigits: 2 })}
          <span className="text-sm ml-2">
            ({data.total_unrealized_gain_pct.toFixed(1)}%)
          </span>
        </p>
      </div>

      {/* Risk Score */}
      <div className="card">
        <div className="flex items-center gap-2 text-sm text-slate-400">
          <Percent size={16} />
          <span>Sharpe Ratio</span>
        </div>
        <p className="text-2xl font-bold mt-2">{data.risk.sharpe_ratio.toFixed(2)}</p>
        <p className="text-xs text-slate-500 mt-1">
          Vol: {data.risk.realized_vol_30d.toFixed(1)}% | MaxDD: {data.risk.max_drawdown_30d.toFixed(1)}%
        </p>
      </div>

      {/* DeFi Exposure */}
      <div className="card">
        <div className="flex items-center gap-2 text-sm text-slate-400">
          <span>DeFi Exposure</span>
        </div>
        <p className="text-2xl font-bold mt-2">
          {data.risk.defi_exposure_pct.toFixed(1)}%
        </p>
        <div className="mt-2 h-1.5 bg-slate-700 rounded-full overflow-hidden">
          <div
            className={clsx(
              'h-full rounded-full transition-all',
              data.risk.defi_exposure_pct > 50 ? 'bg-red-500' : data.risk.defi_exposure_pct > 30 ? 'bg-amber-500' : 'bg-emerald-500'
            )}
            style={{ width: `${data.risk.defi_exposure_pct}%` }}
          />
        </div>
      </div>
    </div>
  );
}   