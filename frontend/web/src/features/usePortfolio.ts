'use client';

import useSWR from 'swr';
import { fetcher } from '@/lib/api';

export interface PortfolioData {
  user_id: string;
  generated_at: string;
  total_value_usd: number;
  total_cost_basis_usd: number;
  total_unrealized_gain_usd: number;
  total_unrealized_gain_pct: number;
  by_asset_type: { asset_type: string; value_usd: number; pct_of_total: number }[];
  by_chain: { chain: string; value_usd: number; pct_of_total: number }[];
  top_positions: {
    symbol: string;
    chain: string;
    amount: number;
    value_usd: number;
    pct_of_total: number;
    change_24h_pct: number;
    change_7d_pct: number;
  }[];
  risk: {
    sharpe_ratio: number;
    max_drawdown_30d: number;
    var_95_1d: number;
    cvar_95_1d: number;
    realized_vol_30d: number;
    defi_exposure_pct: number;
    stablecoin_pct: number;
    concentration_max_pct: number;
  };
  tax_exposure_usd: number;
  concentration_alerts: string[];
}

export function usePortfolio() {
  return useSWR<PortfolioData>('/api/v1/portfolio', fetcher, {
    refreshInterval: 30_000,
  });
}   