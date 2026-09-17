'use client';

import { clsx } from 'clsx';

interface Position {
  symbol: string;
  chain: string;
  amount: number;
  value_usd: number;
  pct_of_total: number;
  change_24h_pct: number;
  change_7d_pct: number;
}

export function PositionsTable({ positions }: { positions: Position[] }) {
  return (
    <div className="card overflow-hidden">
      <h3 className="text-sm font-semibold text-slate-300 mb-4">Top Positions</h3>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-500 border-b border-slate-800">
              <th className="pb-2 pr-4">Asset</th>
              <th className="pb-2 pr-4">Chain</th>
              <th className="pb-2 pr-4 text-right">Amount</th>
              <th className="pb-2 pr-4 text-right">Value</th>
              <th className="pb-2 pr-4 text-right">24h</th>
              <th className="pb-2 pr-4 text-right">7d</th>
              <th className="pb-2 text-right">Weight</th>
            </tr>
          </thead>
          <tbody>
            {positions.map((pos) => (
              <tr
                key={pos.symbol}
                className="border-b border-slate-800/50 hover:bg-slate-800/30 transition-colors"
              >
                <td className="py-2.5 pr-4 font-medium">{pos.symbol}</td>
                <td className="py-2.5 pr-4 text-slate-400 capitalize">{pos.chain}</td>
                <td className="py-2.5 pr-4 text-right font-mono">
                  {pos.amount.toLocaleString('en-US', { maximumFractionDigits: 6 })}
                </td>
                <td className="py-2.5 pr-4 text-right font-mono">
                  ${pos.value_usd.toLocaleString()}
                </td>
                <td className={clsx('py-2.5 pr-4 text-right font-mono', pos.change_24h_pct >= 0 ? 'text-emerald-400' : 'text-red-400')}>
                  {pos.change_24h_pct >= 0 ? '+' : ''}{pos.change_24h_pct.toFixed(2)}%
                </td>
                <td className={clsx('py-2.5 pr-4 text-right font-mono', pos.change_7d_pct >= 0 ? 'text-emerald-400' : 'text-red-400')}>
                  {pos.change_7d_pct >= 0 ? '+' : ''}{pos.change_7d_pct.toFixed(2)}%
                </td>
                <td className="py-2.5 text-right text-slate-400">{pos.pct_of_total.toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}   