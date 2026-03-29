import { ApiClient } from './client';
import { Trade } from './models';

export interface BridgeProvider {
  name: string;
  oracle_address: string;
  is_active: boolean;
  fee_bps: number;
  supported_chains: string[];
  max_trade_amount: string;
  min_trade_amount: string;
}

export interface CrossChainTrade {
  trade_id: number;
  source_chain: string;
  dest_chain: string;
  source_tx_hash: string;
  bridge_provider: string;
  attestation_id: string;
  attestation_status: 'pending' | 'confirmed' | 'failed' | 'expired';
  attestation_timestamp: number;
  retry_count: number;
  current_confirmations: number;
  bridge_fee: string;
}

export interface BridgeAttestation {
  attestation_id: string;
  trade_id: number;
  source_chain: string;
  source_tx_hash: string;
  amount: string;
  recipient: string;
  timestamp: number;
  signature: string;
  provider: string;
}

export interface BridgeValidation {
  valid: boolean;
  error_code?: number;
  error_message?: string;
  confirmations: number;
}

export interface ProviderStatus {
  provider: string;
  healthy: boolean;
  latency_ms: number;
  last_check: number;
  error?: string;
}

/**
 * Bridge API for cross-chain trade operations
 */
export class BridgeApi {
  constructor(private client: ApiClient) {}

  /**
   * Register a new bridge provider
   */
  async registerProvider(provider: Partial<BridgeProvider>): Promise<BridgeProvider> {
    return this.client.post('/bridge/providers', provider);
  }

  /**
   * Get all registered bridge providers
   */
  async getProviders(): Promise<BridgeProvider[]> {
    return this.client.get('/bridge/providers');
  }

  /**
   * Get a specific bridge provider
   */
  async getProvider(name: string): Promise<BridgeProvider> {
    return this.client.get(`/bridge/providers/${name}`);
  }

  /**
   * Deactivate a bridge provider
   */
  async deactivateProvider(name: string): Promise<void> {
    return this.client.delete(`/bridge/providers/${name}`);
  }

  /**
   * Create a cross-chain trade
   */
  async createCrossChainTrade(
    tradeId: number,
    sourceChain: string,
    sourceTxHash: string,
    bridgeProvider: string,
    attestationId: string,
    bridgeFee: string
  ): Promise<CrossChainTrade> {
    return this.client.post('/bridge/trades', {
      trade_id: tradeId,
      source_chain: sourceChain,
      source_tx_hash: sourceTxHash,
      bridge_provider: bridgeProvider,
      attestation_id: attestationId,
      bridge_fee: bridgeFee,
    });
  }

  /**
   * Get cross-chain trade info
   */
  async getCrossChainTrade(tradeId: number): Promise<CrossChainTrade> {
    return this.client.get(`/bridge/trades/${tradeId}`);
  }

  /**
   * Submit bridge attestation
   */
  async submitAttestation(attestation: BridgeAttestation): Promise<BridgeValidation> {
    return this.client.post('/bridge/attestations', attestation);
  }

  /**
   * Get attestation status
   */
  async getAttestationStatus(attestationId: string): Promise<BridgeValidation> {
    return this.client.get(`/bridge/attestations/${attestationId}`);
  }

  /**
   * Retry failed attestation
   */
  async retryAttestation(tradeId: number): Promise<void> {
    return this.client.post(`/bridge/trades/${tradeId}/retry`, {});
  }

  /**
   * Rollback failed cross-chain trade
   */
  async rollbackTrade(tradeId: number, reason: string): Promise<CrossChainTrade> {
    return this.client.post(`/bridge/trades/${tradeId}/rollback`, { reason });
  }

  /**
   * Get bridge provider health status
   */
  async getProviderStatus(provider: string): Promise<ProviderStatus> {
    return this.client.get(`/bridge/providers/${provider}/status`);
  }

  /**
   * Get all provider statuses
   */
  async getAllProviderStatuses(): Promise<ProviderStatus[]> {
    return this.client.get('/bridge/providers/status/all');
  }

  /**
   * Pause bridge operations
   */
  async pauseBridge(): Promise<void> {
    return this.client.post('/bridge/pause', {});
  }

  /**
   * Resume bridge operations
   */
  async resumeBridge(): Promise<void> {
    return this.client.post('/bridge/resume', {});
  }

  /**
   * Get bridge operational status
   */
  async getBridgeStatus(): Promise<{ paused: boolean; providers_active: number }> {
    return this.client.get('/bridge/status');
  }

  /**
   * Query cross-chain trades with filters
   */
  async queryCrossChainTrades(filters: {
    source_chain?: string;
    bridge_provider?: string;
    attestation_status?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ items: CrossChainTrade[]; total: number; has_more: boolean }> {
    const params = new URLSearchParams();
    if (filters.source_chain) params.append('source_chain', filters.source_chain);
    if (filters.bridge_provider) params.append('bridge_provider', filters.bridge_provider);
    if (filters.attestation_status) params.append('attestation_status', filters.attestation_status);
    if (filters.limit) params.append('limit', filters.limit.toString());
    if (filters.offset) params.append('offset', filters.offset.toString());

    return this.client.get(`/bridge/trades?${params}`);
  }

  /**
   * Get bridge statistics
   */
  async getBridgeStats(): Promise<{
    total_cross_chain_trades: number;
    successful_attestations: number;
    failed_attestations: number;
    total_bridge_volume: string;
    average_confirmation_time_secs: number;
  }> {
    return this.client.get('/bridge/stats');
  }
}
