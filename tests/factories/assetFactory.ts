import { stellarAddress, pick } from './stellar';

export interface AssetRecord {
  code: string;          // e.g. "USDC"
  issuer: string;        // synthetic G-address
  decimals: number;
  is_native: boolean;
}

const ASSET_CODES = ['USDC', 'USDT', 'XLM', 'BTC', 'ETH'] as const;

export function assetFactory(overrides: Partial<AssetRecord> = {}): AssetRecord {
  const code = pick(ASSET_CODES);
  return {
    code,
    issuer: code === 'XLM' ? 'native' : stellarAddress(),
    decimals: 7,
    is_native: code === 'XLM',
    ...overrides,
  };
}

/** Canonical USDC asset used across test fixtures */
export const USDC_ASSET: AssetRecord = assetFactory({ code: 'USDC', is_native: false });
