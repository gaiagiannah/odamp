'use client';

export function TaxExposureCard({ exposure }: { exposure: number }) {
  return (
    <div className="card">
      <h3 className="text-sm font-semibold text-slate-300 mb-2">Tax Exposure</h3>
      <p className="text-2xl font-bold text-amber-400">
        ${exposure.toLocaleString('en-US', { maximumFractionDigits: 0 })}
      </p>
      <p className="text-xs text-slate-500 mt-1">Unrealized gains subject to tax if realized</p>
      <div className="mt-3 text-xs text-slate-400">
        Next filing: April 15, 2027
      </div>
    </div>
  );
}   