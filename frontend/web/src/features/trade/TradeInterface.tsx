'use client';

import { useState } from 'react';
import { clsx } from 'clsx';
import { ArrowDownUp, Zap, Shield, Settings } from 'lucide-react';

export function TradeInterface() {
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [symbol, setSymbol] = useState('BTC');
  const [amount, setAmount] = useState('');
  const [limitPrice, setLimitPrice] = useState('');
  const [orderType, setOrderType] = useState<'market' | 'limit'>('market');
  const [mevProtection, setMevProtection] = useState(true);
  const [simulation, setSimulation] = useState<null | {
    slippage_bps: number;
    fee_usd: number;
    total_cost: number;
    venue: string;
    is_safe: boolean;
  }>(null);

  const handleSimulate = () => {
    // Mock simulation
    setSimulation({
      slippage_bps: 3.2,
      fee_usd: parseFloat(amount || '0') * 0.0012,
      total_cost: parseFloat(amount || '0') * 1.0012,
      venue: 'Binance',
      is_safe: true,
    });
  };

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Trade</h1>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Order Form */}
        <div className="lg:col-span-2 space-y-4">
          <div className="card space-y-4">
            {/* Buy/Sell Toggle */}
            <div className="flex gap-2">
              <button
                onClick={() => setSide('buy')}
                className={clsx(
                  'flex-1 py-3 rounded-lg font-semibold transition-colors',
                  side === 'buy' ? 'bg-emerald-600 text-white' : 'bg-slate-800 text-slate-400 hover:text-white'
                )}
              >
                Buy
              </button>
              <button
                onClick={() => setSide('sell')}
                className={clsx(
                  'flex-1 py-3 rounded-lg font-semibold transition-colors',
                  side === 'sell' ? 'bg-red-600 text-white' : 'bg-slate-800 text-slate-400 hover:text-white'
                )}
              >
                Sell
              </button>
            </div>

            {/* Symbol */}
            <div>
              <label className="text-sm text-slate-400">Asset</label>
              <select
                value={symbol}
                onChange={(e) => setSymbol(e.target.value)}
                className="mt-1 w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2.5 text-sm focus:outline-none focus:ring-1 focus:ring-sky-500"
              >
                <option value="BTC">Bitcoin (BTC)</option>
                <option value="ETH">Ethereum (ETH)</option>
                <option value="SOL">Solana (SOL)</option>
                <option value="USDC">USD Coin (USDC)</option>
                <option value="PAXG">Pax Gold (PAXG)</option>
                <option value="Ondo">Ondo Finance (ONDO)</option>
              </select>
            </div>

            {/* Order Type */}
            <div>
              <label className="text-sm text-slate-400">Order Type</label>
              <div className="mt-1 flex gap-2">
                {(['market', 'limit'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => setOrderType(t)}
                    className={clsx(
                      'px-4 py-2 rounded-lg text-sm font-medium',
                      orderType === t ? 'bg-sky-500/20 text-sky-400' : 'bg-slate-800 text-slate-400'
                    )}
                  >
                    {t === 'market' ? 'Market' : 'Limit'}
                  </button>
                ))}
              </div>
            </div>

            {/* Amount */}
            <div>
              <label className="text-sm text-slate-400">Amount (USD)</label>
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                className="mt-1 w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2.5 text-sm font-mono focus:outline-none focus:ring-1 focus:ring-sky-500"
              />
            </div>

            {/* Limit Price */}
            {orderType === 'limit' && (
              <div>
                <label className="text-sm text-slate-400">Limit Price (USD)</label>
                <input
                  type="number"
                  value={limitPrice}
                  onChange={(e) => setLimitPrice(e.target.value)}
                  placeholder="0.00"
                  className="mt-1 w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2.5 text-sm font-mono focus:outline-none focus:ring-1 focus:ring-sky-500"
                />
              </div>
            )}

            {/* MEV Protection */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 text-sm">
                <Shield size={16} className="text-sky-400" />
                <span>MEV Protection</span>
              </div>
              <button
                onClick={() => setMevProtection(!mevProtection)}
                className={clsx(
                  'w-10 h-5 rounded-full transition-colors relative',
                  mevProtection ? 'bg-sky-500' : 'bg-slate-700'
                )}
              >
                <span
                  className={clsx(
                    'absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform',
                    mevProtection ? 'translate-x-5' : 'translate-x-0.5'
                  )}
                />
              </button>
            </div>

            {/* Actions */}
            <div className="flex gap-3 pt-2">
              <button
                onClick={handleSimulate}
                className="flex-1 py-3 rounded-lg bg-slate-700 hover:bg-slate-600 text-sm font-medium flex items-center justify-center gap-2"
              >
                <Zap size={16} />
                Simulate
              </button>
              <button
                className={clsx(
                  'flex-1 py-3 rounded-lg text-sm font-semibold text-white',
                  side === 'buy' ? 'bg-emerald-600 hover:bg-emerald-500' : 'bg-red-600 hover:bg-red-500'
                )}
              >
                {side === 'buy' ? 'Buy' : 'Sell'} {symbol}
              </button>
            </div>
          </div>

          {/* Simulation Result */}
          {simulation && (
            <div className="card border-sky-800/50">
              <h4 className="text-sm font-semibold text-sky-400 mb-3">Simulation Result</h4>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-sm">
                <div>
                  <p className="text-slate-500 text-xs">Venue</p>
                  <p className="font-medium">{simulation.venue}</p>
                </div>
                <div>
                  <p className="text-slate-500 text-xs">Slippage</p>
                  <p className="font-mono">{simulation.slippage_bps.toFixed(1)} bps</p>
                </div>
                <div>
                  <p className="text-slate-500 text-xs">Fee</p>
                  <p className="font-mono">${simulation.fee_usd.toFixed(2)}</p>
                </div>
                <div>
                  <p className="text-slate-500 text-xs">Status</p>
                  <p className={simulation.is_safe ? 'text-emerald-400' : 'text-red-400'}>
                    {simulation.is_safe ? '✓ Safe' : '⚠️ Review'}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Right Panel: Venues + Strategies */}
        <div className="space-y-4">
          <div className="card">
            <h3 className="text-sm font-semibold text-slate-300 mb-3">Available Venues</h3>
            <div className="space-y-2">
              {[
                { name: 'Binance', fee: '0.10%', latency: '30ms' },
                { name: 'Coinbase', fee: '1.20%', latency: '50ms' },
                { name: 'Kraken', fee: '0.26%', latency: '45ms' },
                { name: 'Uniswap V3', fee: '0.30%', latency: '2s' },
                { name: 'Curve', fee: '0.04%', latency: '2s' },
              ].map((v) => (
                <div key={v.name} className="flex items-center justify-between text-sm py-1.5 border-b border-slate-800/50 last:border-0">
                  <span className="text-slate-300">{v.name}</span>
                  <div className="flex gap-3 text-xs text-slate-500">
                    <span>{v.fee}</span>
                    <span>{v.latency}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div   