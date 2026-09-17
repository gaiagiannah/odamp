'use client';

interface RiskData {
  sharpe_ratio: number;
  max_drawdown_30d: number;
  var_95_1d: number;
  cvar_95_1d: number;
  realized_vol_30d: number;
  defi_exposure_pct: number;
  stablecoin_pct: number;
  concentration_max_pct: number;
}

export function RiskPanel({ risk }: { risk: RiskData }) {
  const metrics = [
    { label: 'Sharpe', value: risk.sharpe_ratio.toFixed(2), color: risk.sharpe_ratio > 1 ? 'text-emerald-400' : 'text-amber-400' },
    { label: 'Max DD (30d)', value: `-${risk.max_drawdown_30d.toFixed(1)}%`, color: 'text-red-400' },
    { label: 'VaR 95% (1d)', value: `-${risk.var_95_1d.toFixed(1)}%`, color: 'text-amber-400' },
    { label: 'CVaR 95% (1d)', value: `-${risk.cvar_95_1d.toFixed(1)}%`, color: 'text-red-400' },
    { label: 'Vol (30d)', value: `${risk.realized_vol_30d.toFixed(1)}%`, color: 'text-slate-300' },
    { label: 'Stablecoins', value: `${risk.stablecoin_pct.toFixed(1)}%`, color: 'text-emerald-400' },
  ];

  return (
    <div className="card">
      <h3 className="text-sm font-semibold text-slate-300 mb-4">Risk Metrics</h3>
      <div className="grid grid-cols-2 gap-3">
        {metrics.map((m) => (
          <div key={m.label} className="bg-slate-800/50 rounded-lg p-3">
            <p className="text-xs text-slate-500">{m.label}</p>
            <p className={`text-lg font-bold mt-1 ${m.color}`}>{m.value}</p>
          </div>
        ))}
      </div>
      {risk.concentration_max_pct > 40 && (
        <div className="mt-4 p-3 bg-amber-900/20 border border-amber-800/50 rounded-lg">
          <p className="text-xs text-amber-300">
            ⚠️ Concentration risk: largest position at {risk.concentration_max_pct.toFixed(0)}% of portfolio
          </p>
        </div>
      )}
    </div>
  );
}   